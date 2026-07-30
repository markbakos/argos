//! Serializable Rust-owned contracts for the frontend boundary.

mod base;
mod error;

pub use base::{
    ActionClassification, ActorId, ActorKind, ActorRef, Availability, BoundaryProof, BuildInfo,
    CoreEvent, CorrelationId, Cursor, EventEnvelope, HealthReason, HealthState, ModuleEnablement,
    ModuleId, Page, PageRequest, RuntimeProfile, SettingsCategory,
};
pub use error::{AppError, AppErrorCode, AppErrorDetails};
