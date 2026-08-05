use argos_domain::{
    ActionClassification as DomainActionClassification, ActorContext as DomainActorContext,
    ActorKind as DomainActorKind, CorrelationId as DomainCorrelationId, ModuleId as DomainModuleId,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Opaque request correlation identifier.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "CorrelationId.ts")]
pub struct CorrelationId(String);

impl CorrelationId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<DomainCorrelationId> for CorrelationId {
    fn from(value: DomainCorrelationId) -> Self {
        Self(value.to_string())
    }
}

/// Stable compiled module identifier.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export_to = "ModuleId.ts")]
pub struct ModuleId(String);

impl ModuleId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&DomainModuleId> for ModuleId {
    fn from(value: &DomainModuleId) -> Self {
        Self(value.as_str().to_owned())
    }
}

/// Stable actor identifier assigned by a trusted adapter.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export_to = "ActorId.ts")]
pub struct ActorId(String);

impl ActorId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque cursor returned by a bounded list operation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "Cursor.ts")]
pub struct Cursor(String);

/// Runtime data-isolation profile embedded or explicitly selected by Rust.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "RuntimeProfile.ts")]
pub enum RuntimeProfile {
    Production,
    Development,
    Test,
}

/// Static application/build information safe for the frontend.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "BuildInfo.ts")]
pub struct BuildInfo {
    pub version: String,
    pub build: String,
    pub profile: RuntimeProfile,
}

/// Bounded local machine identity safe for the local application shell.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "SystemIdentity.ts")]
pub struct SystemIdentity {
    pub hostname: String,
}

/// Side-effect-free bootstrap response used to prove the typed desktop boundary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "BoundaryProof.ts")]
pub struct BoundaryProof {
    pub message: String,
    pub correlation_id: CorrelationId,
}

/// Whether a required subsystem can currently be reached.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "Availability.ts")]
pub enum Availability {
    Available,
    Unavailable,
}

/// User-visible health state independent of module enablement.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "HealthState.ts")]
pub enum HealthState {
    Available,
    Unavailable,
    Degraded,
    Error,
}

/// Bounded reason shape for an unavailable or unhealthy component.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case", tag = "kind")]
#[ts(export_to = "HealthReason.ts")]
pub enum HealthReason {
    PlatformUnavailable {
        message: String,
    },
    PermissionDenied {
        message: String,
    },
    Dependency {
        module_id: ModuleId,
        message: String,
    },
    Internal {
        correlation_id: CorrelationId,
    },
}

/// Stored module enablement, kept separate from health.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "ModuleEnablement.ts")]
pub enum ModuleEnablement {
    Enabled,
    Disabled,
}

/// Security classification declared by an application use case.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "ActionClassification.ts")]
pub enum ActionClassification {
    Read,
    Write,
    Privileged,
    Destructive,
}

impl From<DomainActionClassification> for ActionClassification {
    fn from(value: DomainActionClassification) -> Self {
        match value {
            DomainActionClassification::Read => Self::Read,
            DomainActionClassification::Write => Self::Write,
            DomainActionClassification::Privileged => Self::Privileged,
            DomainActionClassification::Destructive => Self::Destructive,
        }
    }
}

/// Authenticated actor kind assigned by the owning adapter.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "ActorKind.ts")]
pub enum ActorKind {
    Human,
    Cli,
    Agent,
    Automation,
}

impl From<DomainActorKind> for ActorKind {
    fn from(value: DomainActorKind) -> Self {
        match value {
            DomainActorKind::Human => Self::Human,
            DomainActorKind::Cli => Self::Cli,
            DomainActorKind::Agent => Self::Agent,
            DomainActorKind::Automation => Self::Automation,
        }
    }
}

/// Public actor reference; it never accepts identity from React for authorization.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "ActorRef.ts")]
pub struct ActorRef {
    pub kind: ActorKind,
    pub id: ActorId,
}

impl From<&DomainActorContext> for ActorRef {
    fn from(value: &DomainActorContext) -> Self {
        Self {
            kind: value.kind().into(),
            id: ActorId(value.id().as_str().to_owned()),
        }
    }
}

/// Bounded list request. Rust validates the effective limit before use.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "PageRequest.ts")]
pub struct PageRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
    pub limit: u16,
}

/// One bounded page with an opaque continuation cursor.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "Page.ts")]
pub struct Page<T> {
    pub items: Vec<T>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
}

/// Bootstrap setting categories that can invalidate frontend snapshots.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "SettingsCategory.ts")]
pub enum SettingsCategory {
    Theme,
    Modules,
}

/// Foundation event hints. Consumers refetch authoritative snapshots.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case", tag = "event")]
#[ts(export_to = "CoreEvent.ts")]
pub enum CoreEvent {
    ModuleHealthChanged { module_id: ModuleId },
    SettingsChanged { category: SettingsCategory },
}

/// Versioned, correlated event envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "EventEnvelope.ts")]
pub struct EventEnvelope<T> {
    pub schema_version: u16,
    pub correlation_id: CorrelationId,
    pub payload: T,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde::{Serialize, de::DeserializeOwned};
    use ts_rs::{Config, TS};

    use super::*;

    fn assert_round_trip<T>(value: &T) -> Result<(), serde_json::Error>
    where
        T: DeserializeOwned + Eq + std::fmt::Debug + Serialize,
    {
        let json = serde_json::to_string(value)?;
        let decoded = serde_json::from_str(json.as_str())?;
        assert_eq!(value, &decoded);
        Ok(())
    }

    #[test]
    fn every_base_enum_variant_round_trips() -> Result<(), serde_json::Error> {
        for value in [
            RuntimeProfile::Production,
            RuntimeProfile::Development,
            RuntimeProfile::Test,
        ] {
            assert_round_trip(&value)?;
        }
        for value in [Availability::Available, Availability::Unavailable] {
            assert_round_trip(&value)?;
        }
        for value in [
            HealthState::Available,
            HealthState::Unavailable,
            HealthState::Degraded,
            HealthState::Error,
        ] {
            assert_round_trip(&value)?;
        }
        for value in [ModuleEnablement::Enabled, ModuleEnablement::Disabled] {
            assert_round_trip(&value)?;
        }
        for value in [
            ActionClassification::Read,
            ActionClassification::Write,
            ActionClassification::Privileged,
            ActionClassification::Destructive,
        ] {
            assert_round_trip(&value)?;
        }
        for value in [
            ActorKind::Human,
            ActorKind::Cli,
            ActorKind::Agent,
            ActorKind::Automation,
        ] {
            assert_round_trip(&value)?;
        }
        for value in [SettingsCategory::Theme, SettingsCategory::Modules] {
            assert_round_trip(&value)?;
        }

        Ok(())
    }

    #[test]
    fn tagged_health_and_event_shapes_round_trip() -> Result<(), serde_json::Error> {
        let correlation_id = CorrelationId::from(DomainCorrelationId::new());
        let reasons = [
            HealthReason::PlatformUnavailable {
                message: "Unsupported platform".to_owned(),
            },
            HealthReason::PermissionDenied {
                message: "Permission denied".to_owned(),
            },
            HealthReason::Dependency {
                module_id: ModuleId::new("systemd"),
                message: "Dependency unavailable".to_owned(),
            },
            HealthReason::Internal {
                correlation_id: correlation_id.clone(),
            },
        ];

        for reason in reasons {
            assert_round_trip(&reason)?;
        }

        assert_round_trip(&EventEnvelope {
            schema_version: 1,
            correlation_id,
            payload: CoreEvent::SettingsChanged {
                category: SettingsCategory::Theme,
            },
        })
    }

    #[test]
    fn system_identity_round_trips_the_hostname_only() -> Result<(), serde_json::Error> {
        assert_round_trip(&SystemIdentity {
            hostname: "argos-workstation".to_owned(),
        })
    }

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
    struct GeneratorShapeFixture {
        values: BTreeMap<String, String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        optional: Option<String>,
    }

    #[test]
    fn generator_preserves_safe_alias_generic_map_and_optional_shapes() {
        let config = Config::new();
        let fixture = GeneratorShapeFixture::decl(&config);

        assert_eq!(ActorId::decl(&config), "type ActorId = string;");
        assert_eq!(
            Page::<String>::decl(&config),
            "type Page<T> = { items: Array<T>, next_cursor?: Cursor | null, };"
        );
        assert!(fixture.contains("values: { [key in string]: string }"));
        assert!(fixture.contains("optional?: string | null"));
    }
}
