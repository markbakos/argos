use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::OnceLock,
    time::{Duration, Instant},
};

use argos_domain::{
    CpuTimeCounters, LoadUsage, MAX_PROCESS_CANDIDATES, PROCESS_SCAN_BUDGET, PartialReason,
    PressureUsage, PressureWindow, ProcessDetailField, ProcessDetails, ProcessIdentity,
    ProcessIoCounters, ProcessMemoryDetails, ProcessState, RawBlockDevice, RawCpuSnapshot,
    RawMemorySnapshot, RawNetworkInterface, RawProcessSnapshot, RawTaskManagerSnapshot,
    TaskManagerReadError, TaskManagerReader,
};

const KIB: u64 = 1_024;
const SECTOR_BYTES: u64 = 512;
const MAX_AGGREGATE_BYTES: u64 = 256 * 1_024;
const MAX_PROCESS_STAT_BYTES: u64 = 4 * 1_024;
const MAX_PROCESS_IO_BYTES: u64 = 4 * 1_024;
const MAX_PROCESS_STATUS_BYTES: u64 = 64 * 1_024;
const MAX_PROCESS_DETAIL_BYTES: u64 = 16 * 1_024;
const MAX_PROCESS_ARGUMENTS: usize = 256;
const MAX_CGROUPS: usize = 256;
const MAX_PROCESS_NAME_BYTES: usize = 256;

/// Bounded, read-only Linux task-manager source.
#[derive(Debug)]
pub struct LinuxTaskManagerReader {
    proc_root: PathBuf,
    sys_root: PathBuf,
    cpu_model: OnceLock<Option<String>>,
    block_capacities: OnceLock<Option<BTreeMap<String, Option<u64>>>>,
}

impl Default for LinuxTaskManagerReader {
    fn default() -> Self {
        Self {
            proc_root: PathBuf::from("/proc"),
            sys_root: PathBuf::from("/sys"),
            cpu_model: OnceLock::new(),
            block_capacities: OnceLock::new(),
        }
    }
}

impl LinuxTaskManagerReader {
    #[cfg(test)]
    fn with_roots(proc_root: PathBuf, sys_root: PathBuf) -> Self {
        Self {
            proc_root,
            sys_root,
            cpu_model: OnceLock::new(),
            block_capacities: OnceLock::new(),
        }
    }

    fn read_cpu(&self) -> Result<RawCpuSnapshot, TaskManagerReadError> {
        let contents =
            read_text(&self.proc_root.join("stat"), MAX_AGGREGATE_BYTES).map_err(snapshot_error)?;
        let (total, logical) =
            parse_cpu_stat(&contents).ok_or(TaskManagerReadError::SnapshotFailed)?;
        let model = self
            .cpu_model
            .get_or_init(|| {
                read_text(&self.proc_root.join("cpuinfo"), MAX_AGGREGATE_BYTES)
                    .ok()
                    .and_then(|contents| parse_cpu_model(&contents))
            })
            .clone();

        Ok(RawCpuSnapshot {
            total,
            logical,
            model,
        })
    }

    fn read_memory(&self) -> Result<RawMemorySnapshot, TaskManagerReadError> {
        let contents = read_text(&self.proc_root.join("meminfo"), MAX_AGGREGATE_BYTES)
            .map_err(snapshot_error)?;
        parse_meminfo(&contents).ok_or(TaskManagerReadError::SnapshotFailed)
    }

    fn read_processes(
        &self,
    ) -> Result<(Vec<RawProcessSnapshot>, u32, Option<PartialReason>), TaskManagerReadError> {
        let directory = fs::read_dir(&self.proc_root).map_err(snapshot_error)?;
        let (mut pids, candidate_limited) =
            bounded_process_ids(directory.flatten().filter_map(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .and_then(|value| value.parse().ok())
            }));
        pids.sort_unstable();

        let observed_process_count =
            u32::try_from(pids.len() + usize::from(candidate_limited)).unwrap_or(u32::MAX);
        let started = Instant::now();
        let mut processes = Vec::with_capacity(pids.len());
        let mut time_limited = false;
        for pid in pids {
            if started.elapsed() >= PROCESS_SCAN_BUDGET {
                time_limited = true;
                break;
            }
            if let Some(process) = self.read_process(pid) {
                processes.push(process);
            }
        }

        let partial_reason = if time_limited {
            Some(PartialReason::TimeBudget)
        } else if candidate_limited {
            Some(PartialReason::CandidateLimit)
        } else {
            None
        };
        Ok((processes, observed_process_count, partial_reason))
    }

    fn read_process(&self, pid: u32) -> Option<RawProcessSnapshot> {
        let root = self.proc_root.join(pid.to_string());
        let stat = read_text(&root.join("stat"), MAX_PROCESS_STAT_BYTES).ok()?;
        let parsed = parse_process_stat(&stat, rustix::param::page_size() as u64)?;
        if parsed.identity.pid != pid {
            return None;
        }

        let (io, io_permission_denied) = match read_text(&root.join("io"), MAX_PROCESS_IO_BYTES) {
            Ok(contents) => (parse_process_io(&contents), false),
            Err(error) => (None, error.kind() == io::ErrorKind::PermissionDenied),
        };

        Some(RawProcessSnapshot {
            io,
            io_permission_denied,
            ..parsed
        })
    }

    fn read_optional<T>(
        &self,
        relative: &str,
        parser: impl FnOnce(&str) -> Option<T>,
    ) -> Option<T> {
        read_text(&self.proc_root.join(relative), MAX_AGGREGATE_BYTES)
            .ok()
            .and_then(|contents| parser(&contents))
    }

    fn read_block_devices(&self) -> Option<Vec<RawBlockDevice>> {
        let capacities = self.block_capacities.get_or_init(|| {
            let block_root = self.sys_root.join("block");
            let mut capacities = BTreeMap::new();
            for entry in fs::read_dir(&block_root).ok()?.flatten().take(256) {
                let name = entry.file_name().to_string_lossy().into_owned();
                let capacity_bytes = read_text(&entry.path().join("size"), 128)
                    .ok()
                    .and_then(|value| value.trim().parse::<u64>().ok())
                    .and_then(|sectors| sectors.checked_mul(SECTOR_BYTES));
                capacities.insert(name, capacity_bytes);
            }
            Some(capacities)
        });
        let contents = read_text(&self.proc_root.join("diskstats"), MAX_AGGREGATE_BYTES).ok()?;
        Some(parse_diskstats(&contents, capacities.as_ref()?))
    }

    fn read_network_interfaces(&self) -> Option<Vec<RawNetworkInterface>> {
        let contents = read_text(&self.proc_root.join("net/dev"), MAX_AGGREGATE_BYTES).ok()?;
        Some(parse_network_interfaces(&contents))
    }

    fn process_details(
        &self,
        identity: ProcessIdentity,
    ) -> Result<ProcessDetails, TaskManagerReadError> {
        let root = self.proc_root.join(identity.pid.to_string());
        let stat = read_text(&root.join("stat"), MAX_PROCESS_STAT_BYTES).map_err(|error| {
            if error.kind() == io::ErrorKind::PermissionDenied {
                TaskManagerReadError::PermissionDenied
            } else {
                TaskManagerReadError::ProcessGone
            }
        })?;
        let parsed = parse_process_stat(&stat, rustix::param::page_size() as u64)
            .ok_or(TaskManagerReadError::ProcessGone)?;
        if parsed.identity != identity {
            return Err(TaskManagerReadError::ProcessGone);
        }

        let (uid, memory, voluntary_context_switches, involuntary_context_switches) =
            read_text(&root.join("status"), MAX_PROCESS_STATUS_BYTES)
                .ok()
                .map_or(
                    (None, ProcessMemoryDetails::default(), None, None),
                    |contents| parse_process_status(&contents),
                );

        let mut restricted_fields = Vec::new();
        let command_line = read_detail_file(
            &root.join("cmdline"),
            ProcessDetailField::CommandLine,
            &mut restricted_fields,
            parse_command_line,
        );
        let executable = match fs::read_link(root.join("exe")) {
            Ok(path) => Some(truncate_text(
                &path.to_string_lossy(),
                MAX_PROCESS_DETAIL_BYTES as usize,
            )),
            Err(error) => {
                if error.kind() == io::ErrorKind::PermissionDenied {
                    restricted_fields.push(ProcessDetailField::Executable);
                }
                None
            }
        };
        let cgroups = match read_bytes(&root.join("cgroup"), MAX_PROCESS_DETAIL_BYTES) {
            Ok(bytes) => String::from_utf8_lossy(&bytes)
                .lines()
                .take(MAX_CGROUPS)
                .map(ToOwned::to_owned)
                .collect(),
            Err(error) => {
                if error.kind() == io::ErrorKind::PermissionDenied {
                    restricted_fields.push(ProcessDetailField::Cgroup);
                }
                Vec::new()
            }
        };
        let io = match read_text(&root.join("io"), MAX_PROCESS_IO_BYTES) {
            Ok(contents) => parse_process_io(&contents),
            Err(error) => {
                if error.kind() == io::ErrorKind::PermissionDenied {
                    restricted_fields.push(ProcessDetailField::Io);
                }
                None
            }
        };

        Ok(ProcessDetails {
            identity,
            parent_pid: parsed.parent_pid,
            name: parsed.name,
            state: parsed.state,
            uid,
            nice: parsed.nice,
            thread_count: parsed.thread_count,
            command_line,
            executable,
            cgroups,
            memory,
            io,
            voluntary_context_switches,
            involuntary_context_switches,
            restricted_fields,
        })
    }
}

impl TaskManagerReader for LinuxTaskManagerReader {
    fn read_snapshot(&self) -> Result<RawTaskManagerSnapshot, TaskManagerReadError> {
        let cpu = self.read_cpu()?;
        let memory = self.read_memory()?;

        let uptime = self.read_optional("uptime", parse_uptime);
        let captured_at = uptime.unwrap_or_default();
        let load = self.read_optional("loadavg", |contents| {
            parse_load(contents, captured_at.as_secs())
        });
        let load_missing = load.is_none();
        let load = load.unwrap_or(LoadUsage {
            uptime_seconds: captured_at.as_secs(),
            ..LoadUsage::default()
        });
        let (processes, observed_process_count, process_partial) = self.read_processes()?;
        let block_devices = self.read_block_devices();
        let network_interfaces = self.read_network_interfaces();
        let aggregate_partial = uptime.is_none()
            || load_missing
            || block_devices.is_none()
            || network_interfaces.is_none();

        Ok(RawTaskManagerSnapshot {
            captured_at,
            cpu,
            memory,
            load,
            cpu_pressure: self.read_optional("pressure/cpu", parse_pressure),
            memory_pressure: self.read_optional("pressure/memory", parse_pressure),
            io_pressure: self.read_optional("pressure/io", parse_pressure),
            block_devices: block_devices.unwrap_or_default(),
            network_interfaces: network_interfaces.unwrap_or_default(),
            processes,
            observed_process_count,
            partial_reason: process_partial
                .or(aggregate_partial.then_some(PartialReason::SourceUnavailable)),
        })
    }

    fn read_process_details(
        &self,
        identity: ProcessIdentity,
    ) -> Result<ProcessDetails, TaskManagerReadError> {
        self.process_details(identity)
    }
}

fn read_detail_file(
    path: &Path,
    field: ProcessDetailField,
    restricted_fields: &mut Vec<ProcessDetailField>,
    parser: impl FnOnce(&[u8]) -> Option<String>,
) -> Option<String> {
    match read_bytes(path, MAX_PROCESS_DETAIL_BYTES) {
        Ok(bytes) => parser(&bytes),
        Err(error) => {
            if error.kind() == io::ErrorKind::PermissionDenied {
                restricted_fields.push(field);
            }
            None
        }
    }
}

fn read_text(path: &Path, max_bytes: u64) -> io::Result<String> {
    let bytes = read_bytes(path, max_bytes)?;
    String::from_utf8(bytes).map_err(|_cause| io::Error::from(io::ErrorKind::InvalidData))
}

fn read_bytes(path: &Path, max_bytes: u64) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() > usize::try_from(max_bytes).unwrap_or(usize::MAX) {
        return Err(io::Error::from(io::ErrorKind::InvalidData));
    }
    Ok(bytes)
}

fn snapshot_error(error: io::Error) -> TaskManagerReadError {
    if error.kind() == io::ErrorKind::NotFound {
        TaskManagerReadError::Unavailable
    } else {
        TaskManagerReadError::SnapshotFailed
    }
}

fn bounded_process_ids(candidates: impl Iterator<Item = u32>) -> (Vec<u32>, bool) {
    let mut pids = Vec::new();
    for pid in candidates {
        if pids.len() == MAX_PROCESS_CANDIDATES {
            return (pids, true);
        }
        pids.push(pid);
    }
    (pids, false)
}

fn parse_cpu_stat(contents: &str) -> Option<(CpuTimeCounters, Vec<CpuTimeCounters>)> {
    let mut total = None;
    let mut logical = Vec::new();
    for line in contents.lines() {
        let mut fields = line.split_ascii_whitespace();
        let label = fields.next()?;
        if label == "cpu" {
            total = parse_cpu_counters(fields);
        } else if label.strip_prefix("cpu").is_some_and(|index| {
            !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit())
        }) {
            logical.push(parse_cpu_counters(fields)?);
        }
    }
    Some((total?, logical))
}

fn parse_cpu_counters<'a>(mut fields: impl Iterator<Item = &'a str>) -> Option<CpuTimeCounters> {
    Some(CpuTimeCounters {
        user: fields.next()?.parse().ok()?,
        nice: fields.next()?.parse().ok()?,
        system: fields.next()?.parse().ok()?,
        idle: fields.next()?.parse().ok()?,
        io_wait: fields
            .next()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0),
        irq: fields
            .next()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0),
        soft_irq: fields
            .next()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0),
        steal: fields
            .next()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0),
    })
}

fn parse_cpu_model(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        matches!(key.trim(), "model name" | "Hardware" | "Processor")
            .then(|| truncate_text(value.trim(), 256))
            .filter(|value| !value.is_empty())
    })
}

fn parse_meminfo(contents: &str) -> Option<RawMemorySnapshot> {
    let mut values = BTreeMap::new();
    for line in contents.lines() {
        let (key, value) = line.split_once(':')?;
        if let Some(bytes) = parse_kib(value) {
            values.insert(key, bytes);
        }
    }
    Some(RawMemorySnapshot {
        total_bytes: *values.get("MemTotal")?,
        available_bytes: *values.get("MemAvailable")?,
        cached_bytes: values.get("Cached").copied().unwrap_or(0),
        buffers_bytes: values.get("Buffers").copied().unwrap_or(0),
        swap_total_bytes: values.get("SwapTotal").copied().unwrap_or(0),
        swap_free_bytes: values.get("SwapFree").copied().unwrap_or(0),
    })
}

fn parse_kib(value: &str) -> Option<u64> {
    value
        .split_ascii_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?
        .checked_mul(KIB)
}

fn parse_uptime(contents: &str) -> Option<Duration> {
    let seconds = contents
        .split_ascii_whitespace()
        .next()?
        .parse::<f64>()
        .ok()?;
    Duration::try_from_secs_f64(seconds).ok()
}

fn parse_load(contents: &str, uptime_seconds: u64) -> Option<LoadUsage> {
    let mut fields = contents.split_ascii_whitespace();
    let one_minute = fields.next()?.parse().ok()?;
    let five_minutes = fields.next()?.parse().ok()?;
    let fifteen_minutes = fields.next()?.parse().ok()?;
    let (runnable_tasks, total_tasks) = fields.next()?.split_once('/')?;
    Some(LoadUsage {
        one_minute,
        five_minutes,
        fifteen_minutes,
        runnable_tasks: runnable_tasks.parse().ok()?,
        total_tasks: total_tasks.parse().ok()?,
        uptime_seconds,
    })
}

fn parse_pressure(contents: &str) -> Option<PressureUsage> {
    let mut usage = PressureUsage::default();
    for line in contents.lines() {
        let mut fields = line.split_ascii_whitespace();
        let kind = fields.next()?;
        let mut window = PressureWindow::default();
        for field in fields {
            let (key, value) = field.split_once('=')?;
            match key {
                "avg10" => window.average_10 = value.parse().ok()?,
                "avg60" => window.average_60 = value.parse().ok()?,
                "avg300" => window.average_300 = value.parse().ok()?,
                "total" => window.total_microseconds = value.parse().ok()?,
                _ => {}
            }
        }
        match kind {
            "some" => usage.some = Some(window),
            "full" => usage.full = Some(window),
            _ => {}
        }
    }
    (usage.some.is_some() || usage.full.is_some()).then_some(usage)
}

fn parse_diskstats(
    contents: &str,
    capacities: &BTreeMap<String, Option<u64>>,
) -> Vec<RawBlockDevice> {
    contents
        .lines()
        .filter_map(|line| {
            let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
            let name = *fields.get(2)?;
            let capacity_bytes = capacities.get(name)?;
            Some(RawBlockDevice {
                name: truncate_text(name, 256),
                sectors_read: fields.get(5)?.parse().ok()?,
                sectors_written: fields.get(9)?.parse().ok()?,
                io_in_progress: fields.get(11)?.parse().ok()?,
                io_milliseconds: fields.get(12)?.parse().ok()?,
                capacity_bytes: *capacity_bytes,
            })
        })
        .collect()
}

fn parse_network_interfaces(contents: &str) -> Vec<RawNetworkInterface> {
    contents
        .lines()
        .filter_map(|line| {
            let (name, counters) = line.split_once(':')?;
            let name = name.trim();
            let fields = counters.split_ascii_whitespace().collect::<Vec<_>>();
            Some(RawNetworkInterface {
                name: truncate_text(name, 256),
                received_bytes: fields.first()?.parse().ok()?,
                transmitted_bytes: fields.get(8)?.parse().ok()?,
                is_loopback: name == "lo",
            })
        })
        .collect()
}

fn parse_process_stat(contents: &str, page_size: u64) -> Option<RawProcessSnapshot> {
    let open = contents.find('(')?;
    let close = contents.rfind(") ")?;
    let pid = contents[..open].trim().parse().ok()?;
    let name = truncate_text(&contents[open + 1..close], MAX_PROCESS_NAME_BYTES);
    let fields = contents[close + 2..]
        .split_ascii_whitespace()
        .collect::<Vec<_>>();
    let user_ticks = fields.get(11)?.parse::<u64>().ok()?;
    let system_ticks = fields.get(12)?.parse::<u64>().ok()?;
    let resident_pages = fields.get(21)?.parse::<u64>().ok()?;
    Some(RawProcessSnapshot {
        identity: ProcessIdentity {
            pid,
            start_time_ticks: fields.get(19)?.parse().ok()?,
        },
        parent_pid: fields.get(1)?.parse().ok()?,
        name,
        state: parse_process_state(fields.first()?),
        cpu_ticks: user_ticks.checked_add(system_ticks)?,
        nice: fields.get(16)?.parse().ok()?,
        thread_count: fields.get(17)?.parse().ok()?,
        virtual_memory_bytes: fields.get(20)?.parse().ok()?,
        resident_memory_bytes: resident_pages.checked_mul(page_size)?,
        io: None,
        io_permission_denied: false,
    })
}

fn parse_process_state(value: &str) -> ProcessState {
    match value {
        "R" => ProcessState::Running,
        "S" => ProcessState::Sleeping,
        "D" => ProcessState::DiskSleep,
        "T" => ProcessState::Stopped,
        "t" => ProcessState::TracingStop,
        "Z" => ProcessState::Zombie,
        "X" | "x" => ProcessState::Dead,
        "I" => ProcessState::Idle,
        other => ProcessState::Unknown(truncate_text(other, 8)),
    }
}

fn parse_process_io(contents: &str) -> Option<ProcessIoCounters> {
    let mut values = BTreeMap::new();
    for line in contents.lines() {
        let (key, value) = line.split_once(':')?;
        values.insert(key.trim(), value.trim().parse::<u64>().ok()?);
    }
    Some(ProcessIoCounters {
        characters_read: values.get("rchar").copied().unwrap_or(0),
        characters_written: values.get("wchar").copied().unwrap_or(0),
        read_syscalls: values.get("syscr").copied().unwrap_or(0),
        write_syscalls: values.get("syscw").copied().unwrap_or(0),
        read_bytes: values.get("read_bytes").copied().unwrap_or(0),
        write_bytes: values.get("write_bytes").copied().unwrap_or(0),
    })
}

fn parse_process_status(
    contents: &str,
) -> (Option<u32>, ProcessMemoryDetails, Option<u64>, Option<u64>) {
    let mut uid = None;
    let mut memory = ProcessMemoryDetails::default();
    let mut voluntary = None;
    let mut involuntary = None;
    for line in contents.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        match key {
            "Uid" => {
                uid = value
                    .split_ascii_whitespace()
                    .next()
                    .and_then(|value| value.parse().ok())
            }
            "VmPeak" => memory.peak_virtual_bytes = parse_kib(value),
            "VmSize" => memory.virtual_bytes = parse_kib(value),
            "VmHWM" => memory.peak_resident_bytes = parse_kib(value),
            "VmRSS" => memory.resident_bytes = parse_kib(value),
            "RssAnon" => memory.resident_anonymous_bytes = parse_kib(value),
            "RssFile" => memory.resident_file_bytes = parse_kib(value),
            "RssShmem" => memory.resident_shared_bytes = parse_kib(value),
            "VmSwap" => memory.swap_bytes = parse_kib(value),
            "voluntary_ctxt_switches" => voluntary = value.trim().parse().ok(),
            "nonvoluntary_ctxt_switches" => involuntary = value.trim().parse().ok(),
            _ => {}
        }
    }
    (uid, memory, voluntary, involuntary)
}

fn parse_command_line(bytes: &[u8]) -> Option<String> {
    let arguments = bytes
        .split(|byte| *byte == 0)
        .filter(|argument| !argument.is_empty())
        .take(MAX_PROCESS_ARGUMENTS)
        .map(String::from_utf8_lossy)
        .map(|argument| argument.into_owned())
        .collect::<Vec<_>>();
    (!arguments.is_empty()).then(|| arguments.join(" "))
}

fn truncate_text(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_owned();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn fixture_roots() -> io::Result<(PathBuf, PathBuf)> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "argos-task-manager-{}-{unique}",
            std::process::id()
        ));
        let proc_root = root.join("proc");
        let sys_root = root.join("sys");
        fs::create_dir_all(&proc_root)?;
        fs::create_dir_all(&sys_root)?;
        Ok((proc_root, sys_root))
    }

    fn write_fixture(path: &Path, contents: &str) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)
    }

    #[test]
    fn adapter_reads_bounded_aggregate_process_and_detail_fields() -> io::Result<()> {
        let (proc_root, sys_root) = fixture_roots()?;
        write_fixture(
            &proc_root.join("stat"),
            "cpu  100 10 30 400 5 2 3 1 99 99\ncpu0 50 5 15 200 2 1 2 0 40 40\n",
        )?;
        write_fixture(
            &proc_root.join("cpuinfo"),
            "processor: 0\nmodel name: Synthetic CPU\n",
        )?;
        write_fixture(
            &proc_root.join("meminfo"),
            "MemTotal: 8000 kB\nMemAvailable: 3000 kB\nCached: 1000 kB\nBuffers: 100 kB\nSwapTotal: 2000 kB\nSwapFree: 500 kB\n",
        )?;
        write_fixture(&proc_root.join("uptime"), "123.45 20.0\n")?;
        write_fixture(&proc_root.join("loadavg"), "0.10 0.20 0.30 2/100 321\n")?;
        write_fixture(
            &proc_root.join("pressure/cpu"),
            "some avg10=1.00 avg60=2.00 avg300=3.00 total=400\n",
        )?;
        write_fixture(
            &proc_root.join("diskstats"),
            "8 0 sda 1 0 20 0 2 0 40 0 1 50 0 0 0 0 0 0\n8 1 sda1 1 0 10 0 1 0 10 0 0 10 0\n",
        )?;
        write_fixture(&sys_root.join("block/sda/size"), "1000\n")?;
        write_fixture(
            &proc_root.join("net/dev"),
            "Inter-| Receive | Transmit\n lo: 10 0 0 0 0 0 0 0 20 0 0 0 0 0 0 0\n eth0: 100 0 0 0 0 0 0 0 200 0 0 0 0 0 0 0\n",
        )?;
        let stat =
            "42 (synthetic ) worker) R 1 2 3 4 5 6 7 8 9 10 11 12 0 0 20 5 3 0 999 4096 2 0\n";
        write_fixture(&proc_root.join("42/stat"), stat)?;
        write_fixture(
            &proc_root.join("42/io"),
            "rchar: 1\nwchar: 2\nsyscr: 3\nsyscw: 4\nread_bytes: 5\nwrite_bytes: 6\n",
        )?;
        write_fixture(
            &proc_root.join("42/status"),
            "Uid:\t1000 1000 1000 1000\nVmPeak:\t10 kB\nVmSize:\t9 kB\nVmHWM:\t8 kB\nVmRSS:\t7 kB\nRssAnon:\t6 kB\nRssFile:\t1 kB\nRssShmem:\t0 kB\nVmSwap:\t2 kB\nvoluntary_ctxt_switches:\t12\nnonvoluntary_ctxt_switches:\t3\n",
        )?;
        write_fixture(&proc_root.join("42/cmdline"), "synthetic\0--safe\0")?;
        write_fixture(
            &proc_root.join("42/cgroup"),
            "0::/user.slice/synthetic.scope\n",
        )?;

        let fixture_root = proc_root
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| io::Error::other("fixture root is unavailable"))?;
        let reader = LinuxTaskManagerReader::with_roots(proc_root, sys_root);
        let snapshot = reader.read_snapshot().map_err(io::Error::other)?;
        assert_eq!(snapshot.cpu.model.as_deref(), Some("Synthetic CPU"));
        assert_eq!(snapshot.cpu.total.total(), Some(551));
        assert_eq!(snapshot.memory.available_bytes, 3_072_000);
        assert_eq!(snapshot.load.uptime_seconds, 123);
        assert_eq!(snapshot.block_devices.len(), 1);
        assert_eq!(snapshot.block_devices[0].capacity_bytes, Some(512_000));
        assert_eq!(snapshot.network_interfaces.len(), 2);
        assert_eq!(snapshot.processes[0].name, "synthetic ) worker");
        assert_eq!(snapshot.processes[0].identity.start_time_ticks, 999);
        assert_eq!(snapshot.processes[0].cpu_ticks, 23);

        write_fixture(
            &reader.proc_root.join("cpuinfo"),
            "model name: Changed CPU\n",
        )?;
        write_fixture(&reader.sys_root.join("block/sda/size"), "2000\n")?;
        fs::remove_file(reader.proc_root.join("net/dev"))?;
        let second = reader.read_snapshot().map_err(io::Error::other)?;
        assert_eq!(second.cpu.model.as_deref(), Some("Synthetic CPU"));
        assert_eq!(second.block_devices[0].capacity_bytes, Some(512_000));
        assert_eq!(
            second.partial_reason,
            Some(PartialReason::SourceUnavailable)
        );

        let details = reader
            .read_process_details(snapshot.processes[0].identity)
            .map_err(io::Error::other)?;
        assert_eq!(details.uid, Some(1000));
        assert_eq!(details.command_line.as_deref(), Some("synthetic --safe"));
        assert_eq!(details.memory.swap_bytes, Some(2_048));
        assert_eq!(details.cgroups, ["0::/user.slice/synthetic.scope"]);
        assert!(
            !details
                .restricted_fields
                .contains(&ProcessDetailField::Executable)
        );
        drop(reader);
        fs::remove_dir_all(fixture_root)?;
        Ok(())
    }

    #[test]
    fn process_parser_handles_parentheses_and_details_reject_pid_reuse() -> io::Result<()> {
        let stat = "7 (name with ) inside) S 1 0 0 0 0 0 0 0 0 0 4 6 0 0 20 0 2 0 99 1024 3";
        let process = parse_process_stat(stat, 4_096);
        assert_eq!(
            process.as_ref().map(|value| value.name.as_str()),
            Some("name with ) inside")
        );
        assert_eq!(process.as_ref().map(|value| value.cpu_ticks), Some(10));

        let (proc_root, sys_root) = fixture_roots()?;
        write_fixture(&proc_root.join("7/stat"), stat)?;
        let fixture_root = proc_root
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| io::Error::other("fixture root is unavailable"))?;
        let reader = LinuxTaskManagerReader::with_roots(proc_root, sys_root);
        assert_eq!(
            reader.read_process_details(ProcessIdentity {
                pid: 7,
                start_time_ticks: 98,
            }),
            Err(TaskManagerReadError::ProcessGone)
        );
        drop(reader);
        fs::remove_dir_all(fixture_root)?;
        Ok(())
    }

    #[test]
    fn process_candidate_collection_stops_at_the_fixed_bound() {
        let (pids, limited) = bounded_process_ids(1..=4_097);

        assert_eq!(pids.len(), MAX_PROCESS_CANDIDATES);
        assert!(limited);
    }

    #[test]
    #[ignore = "target-only aggregate timing measurement"]
    fn target_snapshot_wall_time_stays_within_budget() -> Result<(), TaskManagerReadError> {
        const SAMPLES: usize = 50;
        let reader = LinuxTaskManagerReader::default();
        let mut elapsed = Vec::with_capacity(SAMPLES);
        let mut maximum_processes = 0;
        for _ in 0..SAMPLES {
            let started = Instant::now();
            let snapshot = reader.read_snapshot()?;
            elapsed.push(started.elapsed());
            maximum_processes = maximum_processes.max(snapshot.observed_process_count);
        }
        elapsed.sort_unstable();
        let p95 = elapsed[(SAMPLES * 95 / 100).saturating_sub(1)];
        let average = elapsed.iter().sum::<Duration>() / u32::try_from(SAMPLES).unwrap_or(1);

        eprintln!(
            "Task Manager target timing: samples={SAMPLES}, max_processes={maximum_processes}, average_ms={}, p95_ms={}",
            average.as_millis(),
            p95.as_millis()
        );
        assert!(p95 < Duration::from_millis(250));
        Ok(())
    }
}
