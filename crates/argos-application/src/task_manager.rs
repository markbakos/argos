use std::{cmp::Ordering, collections::HashMap, io, sync::Mutex, time::Duration};

use argos_domain::{
    ActionClassification, ActorContext, BlockDeviceUsage, CpuCoreUsage, CpuTimeCounters, CpuUsage,
    DomainError, ErrorCode, MAX_SAMPLE_GAP, MemoryUsage, NetworkInterfaceUsage, ProcessIdentity,
    ProcessSort, ProcessSummary, RawProcessSnapshot, RawTaskManagerSnapshot, SortDirection,
    TaskManagerQuery, TaskManagerReadError, TaskManagerReader, TaskManagerSnapshot,
};

use crate::ApplicationError;

/// Read-only Task Manager use case with one bounded prior snapshot for rates.
pub struct TaskManagerService<R> {
    reader: R,
    previous: Mutex<Option<RawTaskManagerSnapshot>>,
}

impl<R> TaskManagerService<R> {
    #[must_use]
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            previous: Mutex::new(None),
        }
    }

    #[must_use]
    pub const fn classification() -> ActionClassification {
        ActionClassification::Read
    }
}

impl<R> Default for TaskManagerService<R>
where
    R: Default,
{
    fn default() -> Self {
        Self::new(R::default())
    }
}

impl<R> TaskManagerService<R>
where
    R: TaskManagerReader,
{
    pub fn snapshot(
        &self,
        actor: &ActorContext,
        query: &TaskManagerQuery,
        fresh_baseline: bool,
    ) -> Result<TaskManagerSnapshot, ApplicationError> {
        let mut previous = self.previous.lock().map_err(|_cause| {
            ApplicationError::internal(
                actor.correlation_id(),
                io::Error::other("task manager snapshot state is unavailable"),
            )
        })?;
        let current = self
            .reader
            .read_snapshot()
            .map_err(|error| map_read_error(error, actor))?;
        let usable_previous = previous.as_ref().filter(|older| {
            if fresh_baseline {
                return false;
            }
            current
                .captured_at
                .checked_sub(older.captured_at)
                .is_some_and(|gap| gap <= MAX_SAMPLE_GAP && !gap.is_zero())
        });
        let result = derive_snapshot(&current, usable_previous, query);
        *previous = Some(current);
        Ok(result)
    }

    pub fn process_details(
        &self,
        actor: &ActorContext,
        identity: ProcessIdentity,
    ) -> Result<argos_domain::ProcessDetails, ApplicationError> {
        let was_observed = self
            .previous
            .lock()
            .map_err(|_cause| {
                ApplicationError::internal(
                    actor.correlation_id(),
                    io::Error::other("task manager snapshot state is unavailable"),
                )
            })?
            .as_ref()
            .is_some_and(|snapshot| {
                snapshot
                    .processes
                    .iter()
                    .any(|process| process.identity == identity)
            });

        if !was_observed {
            return Err(ApplicationError::from_domain(
                DomainError::new(ErrorCode::TaskManagerProcessGone, None),
                actor.correlation_id(),
                false,
            ));
        }

        self.reader
            .read_process_details(identity)
            .map_err(|error| map_read_error(error, actor))
    }
}

fn map_read_error(error: TaskManagerReadError, actor: &ActorContext) -> ApplicationError {
    let (code, retryable) = match error {
        TaskManagerReadError::Unavailable => (ErrorCode::TaskManagerUnavailable, true),
        TaskManagerReadError::SnapshotFailed => (ErrorCode::TaskManagerSnapshotFailed, true),
        TaskManagerReadError::ProcessGone => (ErrorCode::TaskManagerProcessGone, false),
        TaskManagerReadError::PermissionDenied => (ErrorCode::PermissionDenied, false),
    };
    ApplicationError::from_domain(
        DomainError::new(code, None),
        actor.correlation_id(),
        retryable,
    )
}

fn derive_snapshot(
    current: &RawTaskManagerSnapshot,
    previous: Option<&RawTaskManagerSnapshot>,
    query: &TaskManagerQuery,
) -> TaskManagerSnapshot {
    let elapsed = previous.and_then(|older| current.captured_at.checked_sub(older.captured_at));
    let total_cpu_delta =
        previous.and_then(|older| cpu_total_delta(&current.cpu.total, &older.cpu.total));
    let previous_processes = previous
        .map(|older| {
            older
                .processes
                .iter()
                .map(|process| (process.identity, process))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();

    let mut processes = current
        .processes
        .iter()
        .filter(|process| process_matches(process, query.search()))
        .map(|process| {
            derive_process(
                process,
                previous_processes.get(&process.identity).copied(),
                total_cpu_delta,
                elapsed,
                current.memory.total_bytes,
            )
        })
        .collect::<Vec<_>>();
    let matched_process_count = u32::try_from(processes.len()).unwrap_or(u32::MAX);
    sort_processes(&mut processes, query.sort(), query.direction());
    processes.truncate(query.limit());

    TaskManagerSnapshot {
        is_baseline: previous.is_none(),
        is_partial: current.partial_reason.is_some(),
        partial_reason: current.partial_reason,
        observed_process_count: current.observed_process_count,
        matched_process_count,
        cpu: derive_cpu(current, previous),
        memory: derive_memory(&current.memory),
        load: current.load.clone(),
        cpu_pressure: current.cpu_pressure.clone(),
        memory_pressure: current.memory_pressure.clone(),
        io_pressure: current.io_pressure.clone(),
        block_devices: derive_block_devices(current, previous, elapsed),
        network_interfaces: derive_network_interfaces(current, previous, elapsed),
        processes,
    }
}

fn derive_cpu(
    current: &RawTaskManagerSnapshot,
    previous: Option<&RawTaskManagerSnapshot>,
) -> CpuUsage {
    let previous_total = previous.map(|snapshot| &snapshot.cpu.total);
    let total_delta = previous_total.and_then(|older| cpu_total_delta(&current.cpu.total, older));
    let part_percent = |current_part: Option<u64>, previous_part: Option<u64>| {
        total_delta.and_then(|total| percentage(delta(current_part?, previous_part?), total))
    };

    let user_current = checked_sum(&[current.cpu.total.user, current.cpu.total.nice]);
    let user_previous = previous_total.and_then(|value| checked_sum(&[value.user, value.nice]));
    let system_current = checked_sum(&[
        current.cpu.total.system,
        current.cpu.total.irq,
        current.cpu.total.soft_irq,
    ]);
    let system_previous =
        previous_total.and_then(|value| checked_sum(&[value.system, value.irq, value.soft_irq]));
    let idle_percent = part_percent(
        Some(current.cpu.total.idle),
        previous_total.map(|value| value.idle),
    );
    let io_wait_percent = part_percent(
        Some(current.cpu.total.io_wait),
        previous_total.map(|value| value.io_wait),
    );
    let total_percent = match (idle_percent, io_wait_percent) {
        (Some(idle), Some(io_wait)) => Some((100.0 - idle - io_wait).clamp(0.0, 100.0)),
        _ => None,
    };
    let logical = current
        .cpu
        .logical
        .iter()
        .enumerate()
        .map(|(index, counters)| {
            let usage_percent = previous
                .and_then(|older| older.cpu.logical.get(index))
                .and_then(|older| {
                    let total = cpu_total_delta(counters, older)?;
                    let idle = delta(counters.idle, older.idle)?;
                    let io_wait = delta(counters.io_wait, older.io_wait)?;
                    percentage(total.checked_sub(idle)?.checked_sub(io_wait), total)
                });
            CpuCoreUsage {
                logical_index: u16::try_from(index).unwrap_or(u16::MAX),
                usage_percent,
            }
        })
        .collect();

    CpuUsage {
        model: current.cpu.model.clone(),
        total_percent,
        user_percent: part_percent(user_current, user_previous),
        system_percent: part_percent(system_current, system_previous),
        idle_percent,
        io_wait_percent,
        logical,
    }
}

fn derive_memory(memory: &argos_domain::RawMemorySnapshot) -> MemoryUsage {
    MemoryUsage {
        total_bytes: memory.total_bytes,
        available_bytes: memory.available_bytes.min(memory.total_bytes),
        used_bytes: memory.total_bytes.saturating_sub(memory.available_bytes),
        cached_bytes: memory.cached_bytes,
        buffers_bytes: memory.buffers_bytes,
        swap_total_bytes: memory.swap_total_bytes,
        swap_used_bytes: memory
            .swap_total_bytes
            .saturating_sub(memory.swap_free_bytes),
    }
}

fn derive_block_devices(
    current: &RawTaskManagerSnapshot,
    previous: Option<&RawTaskManagerSnapshot>,
    elapsed: Option<Duration>,
) -> Vec<BlockDeviceUsage> {
    let previous_by_name = previous
        .map(|snapshot| {
            snapshot
                .block_devices
                .iter()
                .map(|device| (device.name.as_str(), device))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    current
        .block_devices
        .iter()
        .map(|device| {
            let older = previous_by_name.get(device.name.as_str()).copied();
            BlockDeviceUsage {
                name: device.name.clone(),
                read_bytes_per_second: rate(
                    older
                        .and_then(|value| delta(device.sectors_read, value.sectors_read))
                        .and_then(|sectors| sectors.checked_mul(512)),
                    elapsed,
                ),
                write_bytes_per_second: rate(
                    older
                        .and_then(|value| delta(device.sectors_written, value.sectors_written))
                        .and_then(|sectors| sectors.checked_mul(512)),
                    elapsed,
                ),
                busy_percent: match (older, elapsed) {
                    (Some(value), Some(duration)) if !duration.is_zero() => {
                        delta(device.io_milliseconds, value.io_milliseconds).map(|milliseconds| {
                            (milliseconds as f64 / duration.as_secs_f64() / 10.0).clamp(0.0, 100.0)
                        })
                    }
                    _ => None,
                },
                io_in_progress: device.io_in_progress,
                capacity_bytes: device.capacity_bytes,
            }
        })
        .collect()
}

fn derive_network_interfaces(
    current: &RawTaskManagerSnapshot,
    previous: Option<&RawTaskManagerSnapshot>,
    elapsed: Option<Duration>,
) -> Vec<NetworkInterfaceUsage> {
    let previous_by_name = previous
        .map(|snapshot| {
            snapshot
                .network_interfaces
                .iter()
                .map(|interface| (interface.name.as_str(), interface))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    current
        .network_interfaces
        .iter()
        .map(|interface| {
            let older = previous_by_name.get(interface.name.as_str()).copied();
            NetworkInterfaceUsage {
                name: interface.name.clone(),
                received_bytes_per_second: rate(
                    older.and_then(|value| delta(interface.received_bytes, value.received_bytes)),
                    elapsed,
                ),
                transmitted_bytes_per_second: rate(
                    older.and_then(|value| {
                        delta(interface.transmitted_bytes, value.transmitted_bytes)
                    }),
                    elapsed,
                ),
                is_loopback: interface.is_loopback,
            }
        })
        .collect()
}

fn derive_process(
    process: &RawProcessSnapshot,
    previous: Option<&RawProcessSnapshot>,
    total_cpu_delta: Option<u64>,
    elapsed: Option<Duration>,
    total_memory_bytes: u64,
) -> ProcessSummary {
    let cpu_percent = match (previous, total_cpu_delta) {
        (Some(older), Some(total)) => percentage(delta(process.cpu_ticks, older.cpu_ticks), total),
        _ => None,
    };
    let io_rates = previous.and_then(|older| Some((process.io.as_ref()?, older.io.as_ref()?)));
    ProcessSummary {
        identity: process.identity,
        parent_pid: process.parent_pid,
        name: process.name.clone(),
        state: process.state.clone(),
        cpu_percent,
        resident_memory_bytes: process.resident_memory_bytes,
        resident_memory_percent: if total_memory_bytes == 0 {
            0.0
        } else {
            (process.resident_memory_bytes as f64 / total_memory_bytes as f64 * 100.0)
                .clamp(0.0, 100.0)
        },
        virtual_memory_bytes: process.virtual_memory_bytes,
        disk_read_bytes_per_second: rate(
            io_rates.and_then(|(current, older)| delta(current.read_bytes, older.read_bytes)),
            elapsed,
        ),
        disk_write_bytes_per_second: rate(
            io_rates.and_then(|(current, older)| delta(current.write_bytes, older.write_bytes)),
            elapsed,
        ),
        io_permission_denied: process.io_permission_denied,
        thread_count: process.thread_count,
        nice: process.nice,
    }
}

fn process_matches(process: &RawProcessSnapshot, search: Option<&str>) -> bool {
    let Some(search) = search else {
        return true;
    };
    let search = search.to_lowercase();
    process.name.to_lowercase().contains(&search) || process.identity.pid.to_string() == search
}

fn sort_processes(processes: &mut [ProcessSummary], sort: ProcessSort, direction: SortDirection) {
    processes.sort_by(|left, right| {
        let primary = match sort {
            ProcessSort::Cpu => compare_optional(left.cpu_percent, right.cpu_percent, direction),
            ProcessSort::Memory => compare(
                left.resident_memory_bytes.cmp(&right.resident_memory_bytes),
                direction,
            ),
            ProcessSort::DiskRead => compare_optional(
                left.disk_read_bytes_per_second,
                right.disk_read_bytes_per_second,
                direction,
            ),
            ProcessSort::DiskWrite => compare_optional(
                left.disk_write_bytes_per_second,
                right.disk_write_bytes_per_second,
                direction,
            ),
            ProcessSort::Name => compare(
                left.name.to_lowercase().cmp(&right.name.to_lowercase()),
                direction,
            ),
            ProcessSort::Pid => compare(left.identity.pid.cmp(&right.identity.pid), direction),
            ProcessSort::Threads => compare(left.thread_count.cmp(&right.thread_count), direction),
        };
        primary.then_with(|| left.identity.pid.cmp(&right.identity.pid))
    });
}

fn compare(ordering: Ordering, direction: SortDirection) -> Ordering {
    match direction {
        SortDirection::Ascending => ordering,
        SortDirection::Descending => ordering.reverse(),
    }
}

fn compare_optional(left: Option<f64>, right: Option<f64>, direction: SortDirection) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => compare(left.total_cmp(&right), direction),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn cpu_total_delta(current: &CpuTimeCounters, previous: &CpuTimeCounters) -> Option<u64> {
    checked_sum(&[
        delta(current.user, previous.user)?,
        delta(current.nice, previous.nice)?,
        delta(current.system, previous.system)?,
        delta(current.idle, previous.idle)?,
        delta(current.io_wait, previous.io_wait)?,
        delta(current.irq, previous.irq)?,
        delta(current.soft_irq, previous.soft_irq)?,
        delta(current.steal, previous.steal)?,
    ])
}

fn delta(current: u64, previous: u64) -> Option<u64> {
    current.checked_sub(previous)
}

fn checked_sum(values: &[u64]) -> Option<u64> {
    values.iter().copied().try_fold(0_u64, u64::checked_add)
}

fn percentage(part: Option<u64>, total: u64) -> Option<f64> {
    if total == 0 {
        return None;
    }
    Some((part? as f64 / total as f64 * 100.0).clamp(0.0, 100.0))
}

fn rate(delta: Option<u64>, elapsed: Option<Duration>) -> Option<f64> {
    let seconds = elapsed?.as_secs_f64();
    if seconds <= 0.0 {
        return None;
    }
    Some(delta? as f64 / seconds)
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::Mutex};

    use argos_domain::{
        CorrelationId, LoadUsage, ProcessIoCounters, ProcessState, RawCpuSnapshot,
        RawMemorySnapshot, RawTaskManagerSnapshot, TaskManagerReadError,
    };

    use super::*;

    struct FakeReader {
        snapshots: Mutex<VecDeque<RawTaskManagerSnapshot>>,
    }

    impl TaskManagerReader for FakeReader {
        fn read_snapshot(&self) -> Result<RawTaskManagerSnapshot, TaskManagerReadError> {
            self.snapshots
                .lock()
                .map_err(|_cause| TaskManagerReadError::SnapshotFailed)?
                .pop_front()
                .ok_or(TaskManagerReadError::SnapshotFailed)
        }

        fn read_process_details(
            &self,
            _identity: ProcessIdentity,
        ) -> Result<argos_domain::ProcessDetails, TaskManagerReadError> {
            Err(TaskManagerReadError::ProcessGone)
        }
    }

    fn process(pid: u32, start: u64, cpu: u64, read: u64) -> RawProcessSnapshot {
        RawProcessSnapshot {
            identity: ProcessIdentity {
                pid,
                start_time_ticks: start,
            },
            parent_pid: 1,
            name: format!("process-{pid}"),
            state: ProcessState::Running,
            cpu_ticks: cpu,
            nice: 0,
            thread_count: 2,
            virtual_memory_bytes: 2_000,
            resident_memory_bytes: 1_000,
            io: Some(ProcessIoCounters {
                read_bytes: read,
                write_bytes: read * 2,
                ..ProcessIoCounters::default()
            }),
            io_permission_denied: false,
        }
    }

    fn snapshot(
        at: u64,
        user: u64,
        idle: u64,
        processes: Vec<RawProcessSnapshot>,
    ) -> RawTaskManagerSnapshot {
        RawTaskManagerSnapshot {
            captured_at: Duration::from_secs(at),
            cpu: RawCpuSnapshot {
                total: CpuTimeCounters {
                    user,
                    idle,
                    ..CpuTimeCounters::default()
                },
                logical: vec![],
                model: Some("Fixture CPU".to_owned()),
            },
            memory: RawMemorySnapshot {
                total_bytes: 10_000,
                available_bytes: 4_000,
                swap_total_bytes: 1_000,
                swap_free_bytes: 250,
                ..RawMemorySnapshot::default()
            },
            load: LoadUsage::default(),
            cpu_pressure: None,
            memory_pressure: None,
            io_pressure: None,
            block_devices: vec![],
            network_interfaces: vec![],
            observed_process_count: u32::try_from(processes.len()).unwrap_or(u32::MAX),
            processes,
            partial_reason: None,
        }
    }

    fn query() -> Result<TaskManagerQuery, DomainError> {
        TaskManagerQuery::new(None, ProcessSort::Cpu, SortDirection::Descending, 200)
    }

    #[test]
    fn first_sample_is_baseline_and_second_derives_machine_share()
    -> Result<(), Box<dyn std::error::Error>> {
        let reader = FakeReader {
            snapshots: Mutex::new(VecDeque::from([
                snapshot(1, 10, 90, vec![process(10, 5, 2, 100)]),
                snapshot(3, 30, 170, vec![process(10, 5, 12, 1_100)]),
                snapshot(4, 40, 210, vec![process(10, 5, 14, 1_300)]),
            ])),
        };
        let service = TaskManagerService::new(reader);
        let actor = ActorContext::local_human(CorrelationId::new());

        let first = service.snapshot(&actor, &query()?, true)?;
        let second = service.snapshot(&actor, &query()?, false)?;
        let explicitly_fresh = service.snapshot(&actor, &query()?, true)?;

        assert!(first.is_baseline);
        assert_eq!(first.cpu.total_percent, None);
        assert!(!second.is_baseline);
        assert_eq!(second.cpu.total_percent, Some(20.0));
        assert_eq!(second.memory.used_bytes, 6_000);
        assert_eq!(second.memory.swap_used_bytes, 750);
        assert_eq!(second.processes[0].cpu_percent, Some(10.0));
        assert_eq!(second.processes[0].disk_read_bytes_per_second, Some(500.0));
        assert!(explicitly_fresh.is_baseline);
        assert_eq!(explicitly_fresh.cpu.total_percent, None);
        Ok(())
    }

    #[test]
    fn pid_reuse_resets_process_rates_and_missing_rates_sort_last()
    -> Result<(), Box<dyn std::error::Error>> {
        let reader = FakeReader {
            snapshots: Mutex::new(VecDeque::from([
                snapshot(1, 10, 90, vec![process(10, 5, 2, 100)]),
                snapshot(
                    3,
                    30,
                    170,
                    vec![process(10, 6, 20, 2_000), process(20, 1, 5, 200)],
                ),
            ])),
        };
        let service = TaskManagerService::new(reader);
        let actor = ActorContext::local_human(CorrelationId::new());

        service.snapshot(&actor, &query()?, true)?;
        let result = service.snapshot(&actor, &query()?, false)?;

        assert_eq!(result.processes[0].identity.pid, 10);
        assert_eq!(result.processes[0].cpu_percent, None);
        assert_eq!(result.processes[1].cpu_percent, None);
        Ok(())
    }

    #[test]
    fn counter_regression_and_long_gap_reset_rates() -> Result<(), Box<dyn std::error::Error>> {
        let reader = FakeReader {
            snapshots: Mutex::new(VecDeque::from([
                snapshot(1, 10, 90, vec![process(10, 5, 2, 100)]),
                snapshot(3, 9, 191, vec![process(10, 5, 12, 1_100)]),
                snapshot(10, 20, 280, vec![process(10, 5, 22, 2_100)]),
            ])),
        };
        let service = TaskManagerService::new(reader);
        let actor = ActorContext::local_human(CorrelationId::new());

        service.snapshot(&actor, &query()?, true)?;
        let regressed = service.snapshot(&actor, &query()?, false)?;
        let after_gap = service.snapshot(&actor, &query()?, false)?;

        assert_eq!(regressed.cpu.total_percent, None);
        assert_eq!(regressed.processes[0].cpu_percent, None);
        assert!(after_gap.is_baseline);
        assert_eq!(after_gap.processes[0].disk_read_bytes_per_second, None);
        Ok(())
    }

    #[test]
    fn every_process_sort_honors_direction() {
        let summary =
            |pid, name: &str, cpu, memory, disk_read, disk_write, threads| ProcessSummary {
                identity: ProcessIdentity {
                    pid,
                    start_time_ticks: 1,
                },
                parent_pid: 1,
                name: name.to_owned(),
                state: ProcessState::Running,
                cpu_percent: Some(cpu),
                resident_memory_bytes: memory,
                resident_memory_percent: 1.0,
                virtual_memory_bytes: memory,
                disk_read_bytes_per_second: Some(disk_read),
                disk_write_bytes_per_second: Some(disk_write),
                io_permission_denied: false,
                thread_count: threads,
                nice: 0,
            };
        let processes = [
            summary(10, "zeta", 10.0, 100, 30.0, 20.0, 4),
            summary(20, "alpha", 20.0, 200, 10.0, 30.0, 2),
        ];
        let expectations = [
            (ProcessSort::Cpu, 10, 20),
            (ProcessSort::Memory, 10, 20),
            (ProcessSort::DiskRead, 20, 10),
            (ProcessSort::DiskWrite, 10, 20),
            (ProcessSort::Name, 20, 10),
            (ProcessSort::Pid, 10, 20),
            (ProcessSort::Threads, 20, 10),
        ];

        for (sort, ascending_pid, descending_pid) in expectations {
            let mut ascending = processes.to_vec();
            sort_processes(&mut ascending, sort, SortDirection::Ascending);
            let mut descending = processes.to_vec();
            sort_processes(&mut descending, sort, SortDirection::Descending);
            assert_eq!(ascending[0].identity.pid, ascending_pid);
            assert_eq!(descending[0].identity.pid, descending_pid);
        }
    }

    #[test]
    fn search_sort_and_limit_are_applied_after_derivation() -> Result<(), Box<dyn std::error::Error>>
    {
        let reader = FakeReader {
            snapshots: Mutex::new(VecDeque::from([snapshot(
                1,
                10,
                90,
                vec![process(20, 1, 1, 0), process(10, 1, 1, 0)],
            )])),
        };
        let service = TaskManagerService::new(reader);
        let actor = ActorContext::local_human(CorrelationId::new());
        let query = TaskManagerQuery::new(
            Some("process".to_owned()),
            ProcessSort::Pid,
            SortDirection::Ascending,
            1,
        )?;

        let result = service.snapshot(&actor, &query, true)?;

        assert_eq!(result.matched_process_count, 2);
        assert_eq!(result.processes.len(), 1);
        assert_eq!(result.processes[0].identity.pid, 10);
        assert_eq!(
            TaskManagerService::<FakeReader>::classification(),
            ActionClassification::Read
        );
        Ok(())
    }
}
