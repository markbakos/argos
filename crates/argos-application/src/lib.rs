//! Application use cases and orchestration for Argos.

mod error;
mod foundation;
mod system_identity;

pub use error::{ApplicationError, PublicError};
pub use foundation::{BoundaryProofResult, BoundaryProofService};
pub use system_identity::{SystemIdentityResult, SystemIdentityService};
