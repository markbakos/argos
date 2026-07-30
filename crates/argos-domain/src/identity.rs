use std::{fmt, str::FromStr};

use uuid::Uuid;

/// Opaque identifier used to correlate one request across boundaries.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    /// Creates a new random foundation correlation identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the canonical lowercase UUID representation.
    #[must_use]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for CorrelationId {
    type Err = uuid::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(value).map(Self)
    }
}

/// Kind of authenticated initiator responsible for an operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActorKind {
    Human,
    Cli,
    Agent,
    Automation,
}

/// Stable, bounded actor identifier assigned by a trusted adapter.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ActorId(String);

impl ActorId {
    pub const LOCAL_HUMAN: &'static str = "local-user";
    pub const MAX_CHARACTERS: usize = 128;

    /// Validates and constructs an actor identifier.
    pub fn parse(value: impl Into<String>) -> Result<Self, ActorIdError> {
        let value = value.into();
        let character_count = value.chars().count();

        if value.is_empty()
            || character_count > Self::MAX_CHARACTERS
            || value.chars().any(char::is_control)
        {
            return Err(ActorIdError);
        }

        Ok(Self(value))
    }

    /// Returns the validated actor identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ActorId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Safe validation error for an invalid actor identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActorIdError;

impl fmt::Display for ActorIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("actor identifier is invalid")
    }
}

impl std::error::Error for ActorIdError {}

/// Trusted identity and correlation data attached behind an adapter boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActorContext {
    kind: ActorKind,
    id: ActorId,
    correlation_id: CorrelationId,
}

impl ActorContext {
    /// Constructs a context after an adapter has authenticated the actor.
    #[must_use]
    pub fn new(kind: ActorKind, id: ActorId, correlation_id: CorrelationId) -> Self {
        Self {
            kind,
            id,
            correlation_id,
        }
    }

    /// Constructs the foundation desktop human context.
    #[must_use]
    pub fn local_human(correlation_id: CorrelationId) -> Self {
        let id = ActorId(ActorId::LOCAL_HUMAN.to_owned());
        Self::new(ActorKind::Human, id, correlation_id)
    }

    #[must_use]
    pub fn kind(&self) -> ActorKind {
        self.kind
    }

    #[must_use]
    pub fn id(&self) -> &ActorId {
        &self.id
    }

    #[must_use]
    pub fn correlation_id(&self) -> CorrelationId {
        self.correlation_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_ids_are_canonical_uuid_v4_values() {
        let id = CorrelationId::new();
        let rendered = id.to_string();

        assert_eq!(rendered, rendered.to_lowercase());
        assert_eq!(rendered.parse::<CorrelationId>(), Ok(id));
        assert_eq!(id.as_uuid().get_version_num(), 4);
    }

    #[test]
    fn desktop_context_assigns_the_human_actor_behind_the_boundary() {
        let correlation_id = CorrelationId::new();
        let actor = ActorContext::local_human(correlation_id);

        assert_eq!(actor.kind(), ActorKind::Human);
        assert_eq!(actor.id().as_str(), ActorId::LOCAL_HUMAN);
        assert_eq!(actor.correlation_id(), correlation_id);
    }

    #[test]
    fn actor_ids_are_bounded_and_reject_control_characters() {
        assert!(ActorId::parse("agent-1").is_ok());
        assert!(ActorId::parse("").is_err());
        assert!(ActorId::parse("a".repeat(ActorId::MAX_CHARACTERS + 1)).is_err());
        assert!(ActorId::parse("agent\n1").is_err());
    }
}
