use std::fmt;

use crate::{CorrelationId, DomainError, ErrorCode};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModuleId(String);

impl ModuleId {
    pub fn parse(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 64
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-')
            })
            || value.starts_with('-')
            || value.ends_with('-')
        {
            return Err(DomainError::new(ErrorCode::ValidationInvalidFormat, None));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleCapability {
    TaskManagerRead,
    SystemdUserRead,
    SystemdSystemRead,
    LauncherRead,
    LauncherWrite,
    LauncherExecute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleEnablement {
    Enabled,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleHealthState {
    Available,
    Unavailable,
    Degraded,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModuleHealthReason {
    PlatformUnavailable {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleHealth {
    pub state: ModuleHealthState,
    pub reason: Option<ModuleHealthReason>,
}

impl ModuleHealth {
    #[must_use]
    pub const fn available() -> Self {
        Self {
            state: ModuleHealthState::Available,
            reason: None,
        }
    }

    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            state: ModuleHealthState::Unavailable,
            reason: Some(ModuleHealthReason::PlatformUnavailable {
                message: message.into(),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleManifest {
    pub id: ModuleId,
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub route: String,
    pub default_order: u16,
    pub default_enabled: bool,
    pub capabilities: Vec<ModuleCapability>,
    pub dependencies: Vec<ModuleId>,
    pub linux_required: bool,
    pub health: ModuleHealth,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModulePreference {
    pub module_id: ModuleId,
    pub enabled: Option<bool>,
    pub order: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveModule {
    pub manifest: ModuleManifest,
    pub enablement: ModuleEnablement,
    pub order: u16,
    pub health: ModuleHealth,
}
