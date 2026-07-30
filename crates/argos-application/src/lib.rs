//! Application use cases and orchestration for Argos.

mod error;
mod foundation;

pub use error::{ApplicationError, PublicError};
pub use foundation::{BoundaryProofResult, BoundaryProofService};
