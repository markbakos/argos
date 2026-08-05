use argos_domain::{
    BlockDeviceUsage as DomainBlockDeviceUsage, CpuCoreUsage as DomainCpuCoreUsage,
    CpuUsage as DomainCpuUsage, LoadUsage as DomainLoadUsage, MemoryUsage as DomainMemoryUsage,
    NetworkInterfaceUsage as DomainNetworkInterfaceUsage, PartialReason as DomainPartialReason,
    PressureUsage as DomainPressureUsage, PressureWindow as DomainPressureWindow,
    ProcessDetailField as DomainProcessDetailField, ProcessDetails as DomainProcessDetails,
    ProcessIdentity as DomainProcessIdentity, ProcessIoCounters as DomainProcessIoCounters,
    ProcessMemoryDetails as DomainProcessMemoryDetails, ProcessSort as DomainProcessSort,
    ProcessState as DomainProcessState, ProcessSummary as DomainProcessSummary,
    SortDirection as DomainSortDirection, TaskManagerQuery as DomainTaskManagerQuery,
    TaskManagerSnapshot as DomainTaskManagerSnapshot,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "TaskManagerSort.ts")]
pub enum TaskManagerSort {
    Cpu,
    Memory,
    DiskRead,
    DiskWrite,
    Name,
    Pid,
    Threads,
}

impl From<TaskManagerSort> for DomainProcessSort {
    fn from(value: TaskManagerSort) -> Self {
        match value {
            TaskManagerSort::Cpu => Self::Cpu,
            TaskManagerSort::Memory => Self::Memory,
            TaskManagerSort::DiskRead => Self::DiskRead,
            TaskManagerSort::DiskWrite => Self::DiskWrite,
            TaskManagerSort::Name => Self::Name,
            TaskManagerSort::Pid => Self::Pid,
            TaskManagerSort::Threads => Self::Threads,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "TaskManagerSortDirection.ts")]
pub enum TaskManagerSortDirection {
    Ascending,
    Descending,
}

impl From<TaskManagerSortDirection> for DomainSortDirection {
    fn from(value: TaskManagerSortDirection) -> Self {
        match value {
            TaskManagerSortDirection::Ascending => Self::Ascending,
            TaskManagerSortDirection::Descending => Self::Descending,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerSnapshotRequest.ts")]
pub struct TaskManagerSnapshotRequest {
    #[serde(default)]
    pub fresh_baseline: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    pub sort: TaskManagerSort,
    pub direction: TaskManagerSortDirection,
    pub limit: u16,
}

impl TryFrom<TaskManagerSnapshotRequest> for DomainTaskManagerQuery {
    type Error = argos_domain::DomainError;

    fn try_from(value: TaskManagerSnapshotRequest) -> Result<Self, Self::Error> {
        Self::new(
            value.search,
            value.sort.into(),
            value.direction.into(),
            value.limit,
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerProcessIdentity.ts")]
pub struct TaskManagerProcessIdentity {
    pub pid: u32,
    #[ts(type = "number")]
    pub start_time_ticks: u64,
}

impl From<DomainProcessIdentity> for TaskManagerProcessIdentity {
    fn from(value: DomainProcessIdentity) -> Self {
        Self {
            pid: value.pid,
            start_time_ticks: value.start_time_ticks,
        }
    }
}

impl From<TaskManagerProcessIdentity> for DomainProcessIdentity {
    fn from(value: TaskManagerProcessIdentity) -> Self {
        Self {
            pid: value.pid,
            start_time_ticks: value.start_time_ticks,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case", tag = "kind")]
#[ts(export_to = "TaskManagerProcessState.ts")]
pub enum TaskManagerProcessState {
    Running,
    Sleeping,
    DiskSleep,
    Stopped,
    TracingStop,
    Zombie,
    Dead,
    Idle,
    Unknown { value: String },
}

impl From<&DomainProcessState> for TaskManagerProcessState {
    fn from(value: &DomainProcessState) -> Self {
        match value {
            DomainProcessState::Running => Self::Running,
            DomainProcessState::Sleeping => Self::Sleeping,
            DomainProcessState::DiskSleep => Self::DiskSleep,
            DomainProcessState::Stopped => Self::Stopped,
            DomainProcessState::TracingStop => Self::TracingStop,
            DomainProcessState::Zombie => Self::Zombie,
            DomainProcessState::Dead => Self::Dead,
            DomainProcessState::Idle => Self::Idle,
            DomainProcessState::Unknown(value) => Self::Unknown {
                value: value.clone(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "TaskManagerPartialReason.ts")]
pub enum TaskManagerPartialReason {
    CandidateLimit,
    TimeBudget,
    SourceUnavailable,
}

impl From<DomainPartialReason> for TaskManagerPartialReason {
    fn from(value: DomainPartialReason) -> Self {
        match value {
            DomainPartialReason::CandidateLimit => Self::CandidateLimit,
            DomainPartialReason::TimeBudget => Self::TimeBudget,
            DomainPartialReason::SourceUnavailable => Self::SourceUnavailable,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerPressureWindow.ts")]
pub struct TaskManagerPressureWindow {
    pub average_10: f64,
    pub average_60: f64,
    pub average_300: f64,
    #[ts(type = "number")]
    pub total_microseconds: u64,
}

impl From<&DomainPressureWindow> for TaskManagerPressureWindow {
    fn from(value: &DomainPressureWindow) -> Self {
        Self {
            average_10: value.average_10,
            average_60: value.average_60,
            average_300: value.average_300,
            total_microseconds: value.total_microseconds,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerPressureUsage.ts")]
pub struct TaskManagerPressureUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub some: Option<TaskManagerPressureWindow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full: Option<TaskManagerPressureWindow>,
}

impl From<&DomainPressureUsage> for TaskManagerPressureUsage {
    fn from(value: &DomainPressureUsage) -> Self {
        Self {
            some: value.some.as_ref().map(Into::into),
            full: value.full.as_ref().map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerCpuCoreUsage.ts")]
pub struct TaskManagerCpuCoreUsage {
    pub logical_index: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_percent: Option<f64>,
}

impl From<&DomainCpuCoreUsage> for TaskManagerCpuCoreUsage {
    fn from(value: &DomainCpuCoreUsage) -> Self {
        Self {
            logical_index: value.logical_index,
            usage_percent: value.usage_percent,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerCpuUsage.ts")]
pub struct TaskManagerCpuUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_percent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_percent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_percent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_percent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_wait_percent: Option<f64>,
    pub logical: Vec<TaskManagerCpuCoreUsage>,
}

impl From<&DomainCpuUsage> for TaskManagerCpuUsage {
    fn from(value: &DomainCpuUsage) -> Self {
        Self {
            model: value.model.clone(),
            total_percent: value.total_percent,
            user_percent: value.user_percent,
            system_percent: value.system_percent,
            idle_percent: value.idle_percent,
            io_wait_percent: value.io_wait_percent,
            logical: value.logical.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerMemoryUsage.ts")]
pub struct TaskManagerMemoryUsage {
    #[ts(type = "number")]
    pub total_bytes: u64,
    #[ts(type = "number")]
    pub available_bytes: u64,
    #[ts(type = "number")]
    pub used_bytes: u64,
    #[ts(type = "number")]
    pub cached_bytes: u64,
    #[ts(type = "number")]
    pub buffers_bytes: u64,
    #[ts(type = "number")]
    pub swap_total_bytes: u64,
    #[ts(type = "number")]
    pub swap_used_bytes: u64,
}

impl From<&DomainMemoryUsage> for TaskManagerMemoryUsage {
    fn from(value: &DomainMemoryUsage) -> Self {
        Self {
            total_bytes: value.total_bytes,
            available_bytes: value.available_bytes,
            used_bytes: value.used_bytes,
            cached_bytes: value.cached_bytes,
            buffers_bytes: value.buffers_bytes,
            swap_total_bytes: value.swap_total_bytes,
            swap_used_bytes: value.swap_used_bytes,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerLoadUsage.ts")]
pub struct TaskManagerLoadUsage {
    pub one_minute: f64,
    pub five_minutes: f64,
    pub fifteen_minutes: f64,
    pub runnable_tasks: u32,
    pub total_tasks: u32,
    #[ts(type = "number")]
    pub uptime_seconds: u64,
}

impl From<&DomainLoadUsage> for TaskManagerLoadUsage {
    fn from(value: &DomainLoadUsage) -> Self {
        Self {
            one_minute: value.one_minute,
            five_minutes: value.five_minutes,
            fifteen_minutes: value.fifteen_minutes,
            runnable_tasks: value.runnable_tasks,
            total_tasks: value.total_tasks,
            uptime_seconds: value.uptime_seconds,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerBlockDeviceUsage.ts")]
pub struct TaskManagerBlockDeviceUsage {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_bytes_per_second: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write_bytes_per_second: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub busy_percent: Option<f64>,
    #[ts(type = "number")]
    pub io_in_progress: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub capacity_bytes: Option<u64>,
}

impl From<&DomainBlockDeviceUsage> for TaskManagerBlockDeviceUsage {
    fn from(value: &DomainBlockDeviceUsage) -> Self {
        Self {
            name: value.name.clone(),
            read_bytes_per_second: value.read_bytes_per_second,
            write_bytes_per_second: value.write_bytes_per_second,
            busy_percent: value.busy_percent,
            io_in_progress: value.io_in_progress,
            capacity_bytes: value.capacity_bytes,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerNetworkInterfaceUsage.ts")]
pub struct TaskManagerNetworkInterfaceUsage {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_bytes_per_second: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmitted_bytes_per_second: Option<f64>,
    pub is_loopback: bool,
}

impl From<&DomainNetworkInterfaceUsage> for TaskManagerNetworkInterfaceUsage {
    fn from(value: &DomainNetworkInterfaceUsage) -> Self {
        Self {
            name: value.name.clone(),
            received_bytes_per_second: value.received_bytes_per_second,
            transmitted_bytes_per_second: value.transmitted_bytes_per_second,
            is_loopback: value.is_loopback,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerProcessSummary.ts")]
pub struct TaskManagerProcessSummary {
    pub identity: TaskManagerProcessIdentity,
    pub parent_pid: u32,
    pub name: String,
    pub state: TaskManagerProcessState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_percent: Option<f64>,
    #[ts(type = "number")]
    pub resident_memory_bytes: u64,
    pub resident_memory_percent: f64,
    #[ts(type = "number")]
    pub virtual_memory_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_read_bytes_per_second: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_write_bytes_per_second: Option<f64>,
    pub io_permission_denied: bool,
    pub thread_count: u32,
    #[ts(type = "number")]
    pub nice: i64,
}

impl From<&DomainProcessSummary> for TaskManagerProcessSummary {
    fn from(value: &DomainProcessSummary) -> Self {
        Self {
            identity: value.identity.into(),
            parent_pid: value.parent_pid,
            name: value.name.clone(),
            state: (&value.state).into(),
            cpu_percent: value.cpu_percent,
            resident_memory_bytes: value.resident_memory_bytes,
            resident_memory_percent: value.resident_memory_percent,
            virtual_memory_bytes: value.virtual_memory_bytes,
            disk_read_bytes_per_second: value.disk_read_bytes_per_second,
            disk_write_bytes_per_second: value.disk_write_bytes_per_second,
            io_permission_denied: value.io_permission_denied,
            thread_count: value.thread_count,
            nice: value.nice,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerSnapshot.ts")]
pub struct TaskManagerSnapshot {
    pub is_baseline: bool,
    pub is_partial: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_reason: Option<TaskManagerPartialReason>,
    pub observed_process_count: u32,
    pub matched_process_count: u32,
    pub cpu: TaskManagerCpuUsage,
    pub memory: TaskManagerMemoryUsage,
    pub load: TaskManagerLoadUsage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_pressure: Option<TaskManagerPressureUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_pressure: Option<TaskManagerPressureUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_pressure: Option<TaskManagerPressureUsage>,
    pub block_devices: Vec<TaskManagerBlockDeviceUsage>,
    pub network_interfaces: Vec<TaskManagerNetworkInterfaceUsage>,
    pub processes: Vec<TaskManagerProcessSummary>,
}

impl From<&DomainTaskManagerSnapshot> for TaskManagerSnapshot {
    fn from(value: &DomainTaskManagerSnapshot) -> Self {
        Self {
            is_baseline: value.is_baseline,
            is_partial: value.is_partial,
            partial_reason: value.partial_reason.map(Into::into),
            observed_process_count: value.observed_process_count,
            matched_process_count: value.matched_process_count,
            cpu: (&value.cpu).into(),
            memory: (&value.memory).into(),
            load: (&value.load).into(),
            cpu_pressure: value.cpu_pressure.as_ref().map(Into::into),
            memory_pressure: value.memory_pressure.as_ref().map(Into::into),
            io_pressure: value.io_pressure.as_ref().map(Into::into),
            block_devices: value.block_devices.iter().map(Into::into).collect(),
            network_interfaces: value.network_interfaces.iter().map(Into::into).collect(),
            processes: value.processes.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "TaskManagerProcessDetailField.ts")]
pub enum TaskManagerProcessDetailField {
    CommandLine,
    Executable,
    Cgroup,
    Io,
}

impl From<DomainProcessDetailField> for TaskManagerProcessDetailField {
    fn from(value: DomainProcessDetailField) -> Self {
        match value {
            DomainProcessDetailField::CommandLine => Self::CommandLine,
            DomainProcessDetailField::Executable => Self::Executable,
            DomainProcessDetailField::Cgroup => Self::Cgroup,
            DomainProcessDetailField::Io => Self::Io,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerProcessIo.ts")]
pub struct TaskManagerProcessIo {
    #[ts(type = "number")]
    pub characters_read: u64,
    #[ts(type = "number")]
    pub characters_written: u64,
    #[ts(type = "number")]
    pub read_syscalls: u64,
    #[ts(type = "number")]
    pub write_syscalls: u64,
    #[ts(type = "number")]
    pub read_bytes: u64,
    #[ts(type = "number")]
    pub write_bytes: u64,
}

impl From<&DomainProcessIoCounters> for TaskManagerProcessIo {
    fn from(value: &DomainProcessIoCounters) -> Self {
        Self {
            characters_read: value.characters_read,
            characters_written: value.characters_written,
            read_syscalls: value.read_syscalls,
            write_syscalls: value.write_syscalls,
            read_bytes: value.read_bytes,
            write_bytes: value.write_bytes,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerProcessMemoryDetails.ts")]
pub struct TaskManagerProcessMemoryDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub peak_virtual_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub virtual_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub peak_resident_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub resident_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub resident_anonymous_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub resident_file_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub resident_shared_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub swap_bytes: Option<u64>,
}

impl From<&DomainProcessMemoryDetails> for TaskManagerProcessMemoryDetails {
    fn from(value: &DomainProcessMemoryDetails) -> Self {
        Self {
            peak_virtual_bytes: value.peak_virtual_bytes,
            virtual_bytes: value.virtual_bytes,
            peak_resident_bytes: value.peak_resident_bytes,
            resident_bytes: value.resident_bytes,
            resident_anonymous_bytes: value.resident_anonymous_bytes,
            resident_file_bytes: value.resident_file_bytes,
            resident_shared_bytes: value.resident_shared_bytes,
            swap_bytes: value.swap_bytes,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "TaskManagerProcessDetails.ts")]
pub struct TaskManagerProcessDetails {
    pub identity: TaskManagerProcessIdentity,
    pub parent_pid: u32,
    pub name: String,
    pub state: TaskManagerProcessState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uid: Option<u32>,
    #[ts(type = "number")]
    pub nice: i64,
    pub thread_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_line: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executable: Option<String>,
    pub cgroups: Vec<String>,
    pub memory: TaskManagerProcessMemoryDetails,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io: Option<TaskManagerProcessIo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub voluntary_context_switches: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | null")]
    pub involuntary_context_switches: Option<u64>,
    pub restricted_fields: Vec<TaskManagerProcessDetailField>,
}

impl From<&DomainProcessDetails> for TaskManagerProcessDetails {
    fn from(value: &DomainProcessDetails) -> Self {
        Self {
            identity: value.identity.into(),
            parent_pid: value.parent_pid,
            name: value.name.clone(),
            state: (&value.state).into(),
            uid: value.uid,
            nice: value.nice,
            thread_count: value.thread_count,
            command_line: value.command_line.clone(),
            executable: value.executable.clone(),
            cgroups: value.cgroups.clone(),
            memory: (&value.memory).into(),
            io: value.io.as_ref().map(Into::into),
            voluntary_context_switches: value.voluntary_context_switches,
            involuntary_context_switches: value.involuntary_context_switches,
            restricted_fields: value
                .restricted_fields
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use argos_domain::{
        CpuUsage, LoadUsage, MemoryUsage, PartialReason, TaskManagerSnapshot as DomainSnapshot,
    };

    use super::*;

    #[test]
    fn request_validation_runs_before_a_reader() {
        let request = TaskManagerSnapshotRequest {
            fresh_baseline: true,
            search: Some("  argos  ".to_owned()),
            sort: TaskManagerSort::Cpu,
            direction: TaskManagerSortDirection::Descending,
            limit: 200,
        };
        let query = DomainTaskManagerQuery::try_from(request);

        assert_eq!(
            query.as_ref().ok().and_then(DomainTaskManagerQuery::search),
            Some("argos")
        );
        assert!(
            DomainTaskManagerQuery::try_from(TaskManagerSnapshotRequest {
                fresh_baseline: false,
                search: None,
                sort: TaskManagerSort::Pid,
                direction: TaskManagerSortDirection::Ascending,
                limit: 0,
            })
            .is_err()
        );
    }

    #[test]
    fn snapshot_round_trips_optional_rates_and_partial_state() -> Result<(), serde_json::Error> {
        let domain = DomainSnapshot {
            is_baseline: true,
            is_partial: true,
            partial_reason: Some(PartialReason::TimeBudget),
            observed_process_count: 42,
            matched_process_count: 0,
            cpu: CpuUsage::default(),
            memory: MemoryUsage::default(),
            load: LoadUsage::default(),
            cpu_pressure: None,
            memory_pressure: None,
            io_pressure: None,
            block_devices: vec![],
            network_interfaces: vec![],
            processes: vec![],
        };
        let contract = TaskManagerSnapshot::from(&domain);
        let json = serde_json::to_string(&contract)?;

        assert_eq!(
            serde_json::from_str::<TaskManagerSnapshot>(&json)?,
            contract
        );
        assert!(json.contains("time_budget"));
        assert!(!json.contains("total_percent"));
        Ok(())
    }
}
