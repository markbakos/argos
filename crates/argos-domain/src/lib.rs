//! Domain vocabulary, invariants, and ports for Argos.

mod action;
mod error;
mod identity;

pub use action::ActionClassification;
pub use error::{
    DomainError, ErrorCode, ErrorDetailError, ErrorDetails, ErrorNamespace,
    MAX_ERROR_DETAIL_CHARACTERS,
};
pub use identity::{ActorContext, ActorId, ActorIdError, ActorKind, CorrelationId};
