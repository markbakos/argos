//! Domain vocabulary, invariants, and ports for Argos.

mod action;
mod configuration;
mod error;
mod identity;
mod modules;
mod system_identity;
mod task_manager;

pub use action::ActionClassification;
pub use configuration::{
    BOOTSTRAP_CONFIG_VERSION, BootstrapConfig, BootstrapConfigRepository, RuntimeProfile,
    ThemePreference,
};
pub use error::{
    DomainError, ErrorCode, ErrorDetailError, ErrorDetails, ErrorNamespace,
    MAX_ERROR_DETAIL_CHARACTERS,
};
pub use identity::{ActorContext, ActorId, ActorIdError, ActorKind, CorrelationId};
pub use modules::{
    EffectiveModule, ModuleCapability, ModuleEnablement, ModuleHealth, ModuleHealthReason,
    ModuleHealthState, ModuleId, ModuleManifest, ModulePreference,
};
pub use system_identity::{Hostname, HostnameError, SystemIdentityReader};
pub use task_manager::{
    BlockDeviceUsage, CpuCoreUsage, CpuTimeCounters, CpuUsage, LoadUsage, MAX_PROCESS_CANDIDATES,
    MAX_PROCESS_RESULTS, MAX_PROCESS_SEARCH_CHARACTERS, MAX_SAMPLE_GAP, MemoryUsage,
    NetworkInterfaceUsage, PROCESS_SCAN_BUDGET, PartialReason, PressureUsage, PressureWindow,
    ProcessDetailField, ProcessDetails, ProcessIdentity, ProcessIoCounters, ProcessMemoryDetails,
    ProcessSort, ProcessState, ProcessSummary, RawBlockDevice, RawCpuSnapshot, RawMemorySnapshot,
    RawNetworkInterface, RawProcessSnapshot, RawTaskManagerSnapshot, SortDirection,
    TaskManagerQuery, TaskManagerReadError, TaskManagerReader, TaskManagerSnapshot,
};
