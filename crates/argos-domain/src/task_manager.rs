use std::{fmt, time::Duration};

use crate::{DomainError, ErrorCode, ErrorDetails};

pub const MAX_PROCESS_CANDIDATES: usize = 4_096;
pub const MAX_PROCESS_RESULTS: u16 = 200;
pub const MAX_PROCESS_SEARCH_CHARACTERS: usize = 128;
pub const PROCESS_SCAN_BUDGET: Duration = Duration::from_millis(250);
pub const MAX_SAMPLE_GAP: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessSort {
    Cpu,
    Memory,
    DiskRead,
    DiskWrite,
    Name,
    Pid,
    Threads,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskManagerQuery {
    search: Option<String>,
    sort: ProcessSort,
    direction: SortDirection,
    limit: u16,
}

impl TaskManagerQuery {
    pub fn new(
        search: Option<String>,
        sort: ProcessSort,
        direction: SortDirection,
        limit: u16,
    ) -> Result<Self, DomainError> {
        if !(1..=MAX_PROCESS_RESULTS).contains(&limit) {
            return Err(validation_error(ErrorCode::ValidationOutOfRange, "limit"));
        }

        let search = search
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        if let Some(value) = &search {
            if value.chars().count() > MAX_PROCESS_SEARCH_CHARACTERS {
                return Err(validation_error(ErrorCode::ValidationOutOfRange, "search"));
            }
            if value.chars().any(char::is_control) {
                return Err(validation_error(
                    ErrorCode::ValidationInvalidFormat,
                    "search",
                ));
            }
        }

        Ok(Self {
            search,
            sort,
            direction,
            limit,
        })
    }

    #[must_use]
    pub fn search(&self) -> Option<&str> {
        self.search.as_deref()
    }

    #[must_use]
    pub fn sort(&self) -> ProcessSort {
        self.sort
    }

    #[must_use]
    pub fn direction(&self) -> SortDirection {
        self.direction
    }

    #[must_use]
    pub fn limit(&self) -> usize {
        usize::from(self.limit)
    }
}

fn validation_error(code: ErrorCode, field: &'static str) -> DomainError {
    DomainError::new(code, ErrorDetails::for_field(field).ok())
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub start_time_ticks: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessState {
    Running,
    Sleeping,
    DiskSleep,
    Stopped,
    TracingStop,
    Zombie,
    Dead,
    Idle,
    Unknown(String),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CpuTimeCounters {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub io_wait: u64,
    pub irq: u64,
    pub soft_irq: u64,
    pub steal: u64,
}

impl CpuTimeCounters {
    #[must_use]
    pub fn total(&self) -> Option<u64> {
        [
            self.user,
            self.nice,
            self.system,
            self.idle,
            self.io_wait,
            self.irq,
            self.soft_irq,
            self.steal,
        ]
        .into_iter()
        .try_fold(0_u64, u64::checked_add)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RawCpuSnapshot {
    pub total: CpuTimeCounters,
    pub logical: Vec<CpuTimeCounters>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RawMemorySnapshot {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub cached_bytes: u64,
    pub buffers_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_free_bytes: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoadUsage {
    pub one_minute: f64,
    pub five_minutes: f64,
    pub fifteen_minutes: f64,
    pub runnable_tasks: u32,
    pub total_tasks: u32,
    pub uptime_seconds: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PressureWindow {
    pub average_10: f64,
    pub average_60: f64,
    pub average_300: f64,
    pub total_microseconds: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PressureUsage {
    pub some: Option<PressureWindow>,
    pub full: Option<PressureWindow>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RawBlockDevice {
    pub name: String,
    pub sectors_read: u64,
    pub sectors_written: u64,
    pub io_milliseconds: u64,
    pub io_in_progress: u64,
    pub capacity_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RawNetworkInterface {
    pub name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
    pub is_loopback: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProcessIoCounters {
    pub characters_read: u64,
    pub characters_written: u64,
    pub read_syscalls: u64,
    pub write_syscalls: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawProcessSnapshot {
    pub identity: ProcessIdentity,
    pub parent_pid: u32,
    pub name: String,
    pub state: ProcessState,
    pub cpu_ticks: u64,
    pub nice: i64,
    pub thread_count: u32,
    pub virtual_memory_bytes: u64,
    pub resident_memory_bytes: u64,
    pub io: Option<ProcessIoCounters>,
    pub io_permission_denied: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PartialReason {
    CandidateLimit,
    TimeBudget,
    SourceUnavailable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RawTaskManagerSnapshot {
    pub captured_at: Duration,
    pub cpu: RawCpuSnapshot,
    pub memory: RawMemorySnapshot,
    pub load: LoadUsage,
    pub cpu_pressure: Option<PressureUsage>,
    pub memory_pressure: Option<PressureUsage>,
    pub io_pressure: Option<PressureUsage>,
    pub block_devices: Vec<RawBlockDevice>,
    pub network_interfaces: Vec<RawNetworkInterface>,
    pub processes: Vec<RawProcessSnapshot>,
    pub observed_process_count: u32,
    pub partial_reason: Option<PartialReason>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CpuCoreUsage {
    pub logical_index: u16,
    pub usage_percent: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CpuUsage {
    pub model: Option<String>,
    pub total_percent: Option<f64>,
    pub user_percent: Option<f64>,
    pub system_percent: Option<f64>,
    pub idle_percent: Option<f64>,
    pub io_wait_percent: Option<f64>,
    pub logical: Vec<CpuCoreUsage>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MemoryUsage {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub cached_bytes: u64,
    pub buffers_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockDeviceUsage {
    pub name: String,
    pub read_bytes_per_second: Option<f64>,
    pub write_bytes_per_second: Option<f64>,
    pub busy_percent: Option<f64>,
    pub io_in_progress: u64,
    pub capacity_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NetworkInterfaceUsage {
    pub name: String,
    pub received_bytes_per_second: Option<f64>,
    pub transmitted_bytes_per_second: Option<f64>,
    pub is_loopback: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessSummary {
    pub identity: ProcessIdentity,
    pub parent_pid: u32,
    pub name: String,
    pub state: ProcessState,
    pub cpu_percent: Option<f64>,
    pub resident_memory_bytes: u64,
    pub resident_memory_percent: f64,
    pub virtual_memory_bytes: u64,
    pub disk_read_bytes_per_second: Option<f64>,
    pub disk_write_bytes_per_second: Option<f64>,
    pub io_permission_denied: bool,
    pub thread_count: u32,
    pub nice: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskManagerSnapshot {
    pub is_baseline: bool,
    pub is_partial: bool,
    pub partial_reason: Option<PartialReason>,
    pub observed_process_count: u32,
    pub matched_process_count: u32,
    pub cpu: CpuUsage,
    pub memory: MemoryUsage,
    pub load: LoadUsage,
    pub cpu_pressure: Option<PressureUsage>,
    pub memory_pressure: Option<PressureUsage>,
    pub io_pressure: Option<PressureUsage>,
    pub block_devices: Vec<BlockDeviceUsage>,
    pub network_interfaces: Vec<NetworkInterfaceUsage>,
    pub processes: Vec<ProcessSummary>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessDetailField {
    CommandLine,
    Executable,
    Cgroup,
    Io,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProcessMemoryDetails {
    pub peak_virtual_bytes: Option<u64>,
    pub virtual_bytes: Option<u64>,
    pub peak_resident_bytes: Option<u64>,
    pub resident_bytes: Option<u64>,
    pub resident_anonymous_bytes: Option<u64>,
    pub resident_file_bytes: Option<u64>,
    pub resident_shared_bytes: Option<u64>,
    pub swap_bytes: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessDetails {
    pub identity: ProcessIdentity,
    pub parent_pid: u32,
    pub name: String,
    pub state: ProcessState,
    pub uid: Option<u32>,
    pub nice: i64,
    pub thread_count: u32,
    pub command_line: Option<String>,
    pub executable: Option<String>,
    pub cgroups: Vec<String>,
    pub memory: ProcessMemoryDetails,
    pub io: Option<ProcessIoCounters>,
    pub voluntary_context_switches: Option<u64>,
    pub involuntary_context_switches: Option<u64>,
    pub restricted_fields: Vec<ProcessDetailField>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskManagerReadError {
    Unavailable,
    SnapshotFailed,
    ProcessGone,
    PermissionDenied,
}

impl fmt::Display for TaskManagerReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Unavailable => "task manager data is unavailable",
            Self::SnapshotFailed => "task manager snapshot failed",
            Self::ProcessGone => "task manager process is gone",
            Self::PermissionDenied => "task manager process details are restricted",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TaskManagerReadError {}

pub trait TaskManagerReader: Send + Sync {
    fn read_snapshot(&self) -> Result<RawTaskManagerSnapshot, TaskManagerReadError>;

    fn read_process_details(
        &self,
        identity: ProcessIdentity,
    ) -> Result<ProcessDetails, TaskManagerReadError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_normalizes_search_and_enforces_bounds() -> Result<(), DomainError> {
        let query = TaskManagerQuery::new(
            Some("  argos  ".to_owned()),
            ProcessSort::Cpu,
            SortDirection::Descending,
            MAX_PROCESS_RESULTS,
        )?;

        assert_eq!(query.search(), Some("argos"));
        assert_eq!(query.limit(), usize::from(MAX_PROCESS_RESULTS));
        assert!(
            TaskManagerQuery::new(None, ProcessSort::Pid, SortDirection::Ascending, 0).is_err()
        );
        assert!(
            TaskManagerQuery::new(
                Some("x".repeat(MAX_PROCESS_SEARCH_CHARACTERS + 1)),
                ProcessSort::Name,
                SortDirection::Ascending,
                1,
            )
            .is_err()
        );
        assert!(
            TaskManagerQuery::new(
                Some("private\nquery".to_owned()),
                ProcessSort::Name,
                SortDirection::Ascending,
                1,
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn cpu_total_excludes_guest_counters_by_construction() {
        let counters = CpuTimeCounters {
            user: 1,
            nice: 2,
            system: 3,
            idle: 4,
            io_wait: 5,
            irq: 6,
            soft_irq: 7,
            steal: 8,
        };

        assert_eq!(counters.total(), Some(36));
    }
}
