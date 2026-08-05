use argos_domain::{
    EffectiveModule as DomainEffectiveModule, ModuleCapability as DomainModuleCapability,
    ModuleEnablement as DomainModuleEnablement, ModuleHealthReason as DomainModuleHealthReason,
    ModuleHealthState as DomainModuleHealthState, ThemePreference as DomainThemePreference,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CorrelationId, HealthReason, HealthState, ModuleEnablement, ModuleId};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "ModuleCapability.ts")]
pub enum ModuleCapability {
    TaskManagerRead,
    SystemdUserRead,
    SystemdSystemRead,
    LauncherRead,
    LauncherWrite,
    LauncherExecute,
}

impl From<DomainModuleCapability> for ModuleCapability {
    fn from(value: DomainModuleCapability) -> Self {
        match value {
            DomainModuleCapability::TaskManagerRead => Self::TaskManagerRead,
            DomainModuleCapability::SystemdUserRead => Self::SystemdUserRead,
            DomainModuleCapability::SystemdSystemRead => Self::SystemdSystemRead,
            DomainModuleCapability::LauncherRead => Self::LauncherRead,
            DomainModuleCapability::LauncherWrite => Self::LauncherWrite,
            DomainModuleCapability::LauncherExecute => Self::LauncherExecute,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "ModuleManifestView.ts")]
pub struct ModuleManifestView {
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "EffectiveModule.ts")]
pub struct EffectiveModule {
    pub manifest: ModuleManifestView,
    pub enablement: ModuleEnablement,
    pub order: u16,
    pub health: HealthState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_reason: Option<HealthReason>,
}

impl From<&DomainEffectiveModule> for EffectiveModule {
    fn from(value: &DomainEffectiveModule) -> Self {
        Self {
            manifest: ModuleManifestView {
                id: (&value.manifest.id).into(),
                display_name: value.manifest.display_name.clone(),
                description: value.manifest.description.clone(),
                version: value.manifest.version.clone(),
                route: value.manifest.route.clone(),
                default_order: value.manifest.default_order,
                default_enabled: value.manifest.default_enabled,
                capabilities: value
                    .manifest
                    .capabilities
                    .iter()
                    .copied()
                    .map(Into::into)
                    .collect(),
                dependencies: value.manifest.dependencies.iter().map(Into::into).collect(),
                linux_required: value.manifest.linux_required,
            },
            enablement: match value.enablement {
                DomainModuleEnablement::Enabled => ModuleEnablement::Enabled,
                DomainModuleEnablement::Disabled => ModuleEnablement::Disabled,
            },
            order: value.order,
            health: match value.health.state {
                DomainModuleHealthState::Available => HealthState::Available,
                DomainModuleHealthState::Unavailable => HealthState::Unavailable,
                DomainModuleHealthState::Degraded => HealthState::Degraded,
                DomainModuleHealthState::Error => HealthState::Error,
            },
            health_reason: value.health.reason.as_ref().map(|reason| match reason {
                DomainModuleHealthReason::PlatformUnavailable { message } => {
                    HealthReason::PlatformUnavailable {
                        message: message.clone(),
                    }
                }
                DomainModuleHealthReason::Dependency { module_id, message } => {
                    HealthReason::Dependency {
                        module_id: module_id.into(),
                        message: message.clone(),
                    }
                }
                DomainModuleHealthReason::Internal { correlation_id } => HealthReason::Internal {
                    correlation_id: CorrelationId::from(*correlation_id),
                },
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "ListModulesResponse.ts")]
pub struct ListModulesResponse {
    pub modules: Vec<EffectiveModule>,
    pub unknown_preference_ids: Vec<ModuleId>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "ThemePreference.ts")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

impl From<DomainThemePreference> for ThemePreference {
    fn from(value: DomainThemePreference) -> Self {
        match value {
            DomainThemePreference::System => Self::System,
            DomainThemePreference::Light => Self::Light,
            DomainThemePreference::Dark => Self::Dark,
        }
    }
}

impl From<ThemePreference> for DomainThemePreference {
    fn from(value: ThemePreference) -> Self {
        match value {
            ThemePreference::System => Self::System,
            ThemePreference::Light => Self::Light,
            ThemePreference::Dark => Self::Dark,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "BootstrapSettings.ts")]
pub struct BootstrapSettings {
    pub theme: ThemePreference,
    pub theme_warning: bool,
    pub production_data_warning: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "SetThemeRequest.ts")]
pub struct SetThemeRequest {
    pub theme: ThemePreference,
}
