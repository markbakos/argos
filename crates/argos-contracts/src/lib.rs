//! Serializable Rust-owned contracts for the frontend boundary.

mod base;
mod error;
mod modules;
mod task_manager;

pub use base::{
    ActionClassification, ActorId, ActorKind, ActorRef, Availability, BoundaryProof, BuildInfo,
    CoreEvent, CorrelationId, Cursor, EventEnvelope, HealthReason, HealthState, ModuleEnablement,
    ModuleId, Page, PageRequest, RuntimeProfile, SettingsCategory, SystemIdentity,
};
pub use error::{AppError, AppErrorCode, AppErrorDetails};
pub use modules::{
    BootstrapSettings, EffectiveModule, ListModulesResponse, ModuleCapability, ModuleManifestView,
    SetThemeRequest, ThemePreference,
};
pub use task_manager::{
    TaskManagerBlockDeviceUsage, TaskManagerCpuCoreUsage, TaskManagerCpuUsage,
    TaskManagerLoadUsage, TaskManagerMemoryUsage, TaskManagerNetworkInterfaceUsage,
    TaskManagerPartialReason, TaskManagerPressureUsage, TaskManagerPressureWindow,
    TaskManagerProcessDetailField, TaskManagerProcessDetails, TaskManagerProcessIdentity,
    TaskManagerProcessIo, TaskManagerProcessMemoryDetails, TaskManagerProcessState,
    TaskManagerProcessSummary, TaskManagerSnapshot, TaskManagerSnapshotRequest, TaskManagerSort,
    TaskManagerSortDirection,
};
