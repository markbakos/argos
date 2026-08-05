//! Application use cases and orchestration for Argos.

mod configuration;
mod error;
mod foundation;
mod modules;
mod system_identity;
mod task_manager;

pub use configuration::ConfigurationService;
pub use error::{ApplicationError, PublicError};
pub use foundation::{BoundaryProofResult, BoundaryProofService};
pub use modules::{
    COMPILED_MODULE_IDS, EffectiveModules, ModuleRegistry, TASK_MANAGER_MODULE_ID,
    compiled_module_registry,
};
pub use system_identity::{SystemIdentityResult, SystemIdentityService};
pub use task_manager::TaskManagerService;
