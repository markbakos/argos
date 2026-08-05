use std::collections::{BTreeMap, HashMap, HashSet};

use argos_domain::{
    DomainError, EffectiveModule, ErrorCode, ModuleCapability, ModuleEnablement, ModuleHealth,
    ModuleHealthReason, ModuleHealthState, ModuleId, ModuleManifest, ModulePreference,
};

pub const TASK_MANAGER_MODULE_ID: &str = "task-manager";
pub const COMPILED_MODULE_IDS: [&str; 3] = ["task-manager", "systemd", "launcher"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveModules {
    pub modules: Vec<EffectiveModule>,
    pub unknown_preference_ids: Vec<ModuleId>,
}

#[derive(Clone, Debug)]
pub struct ModuleRegistry {
    manifests: Vec<ModuleManifest>,
}

impl ModuleRegistry {
    pub fn new(manifests: Vec<ModuleManifest>) -> Result<Self, DomainError> {
        validate_manifests(&manifests)?;
        Ok(Self { manifests })
    }

    pub fn effective(
        &self,
        preferences: &[ModulePreference],
    ) -> Result<EffectiveModules, DomainError> {
        let preferences_by_id = preferences
            .iter()
            .map(|preference| (&preference.module_id, preference))
            .collect::<HashMap<_, _>>();
        let manifest_ids = self
            .manifests
            .iter()
            .map(|manifest| manifest.id.clone())
            .collect::<HashSet<_>>();
        let mut modules = self
            .manifests
            .iter()
            .map(|manifest| {
                let preference = preferences_by_id.get(&manifest.id).copied();
                EffectiveModule {
                    manifest: manifest.clone(),
                    enablement: if preference
                        .and_then(|value| value.enabled)
                        .unwrap_or(manifest.default_enabled)
                    {
                        ModuleEnablement::Enabled
                    } else {
                        ModuleEnablement::Disabled
                    },
                    order: preference
                        .and_then(|value| value.order)
                        .unwrap_or(manifest.default_order),
                    health: manifest.health.clone(),
                }
            })
            .collect::<Vec<_>>();

        let state_by_id = modules
            .iter()
            .map(|module| {
                (
                    module.manifest.id.clone(),
                    (module.enablement, module.health.state),
                )
            })
            .collect::<HashMap<_, _>>();
        for module in &mut modules {
            if module.enablement == ModuleEnablement::Disabled {
                continue;
            }
            if let Some(dependency) = module.manifest.dependencies.iter().find(|dependency| {
                state_by_id
                    .get(*dependency)
                    .is_some_and(|(enablement, health)| {
                        *enablement == ModuleEnablement::Disabled
                            || *health != ModuleHealthState::Available
                    })
            }) {
                module.health = ModuleHealth {
                    state: ModuleHealthState::Unavailable,
                    reason: Some(ModuleHealthReason::Dependency {
                        module_id: dependency.clone(),
                        message: "A required module is disabled or unavailable.".to_owned(),
                    }),
                };
            }
        }
        modules.sort_by(|left, right| {
            left.order
                .cmp(&right.order)
                .then_with(|| {
                    left.manifest
                        .default_order
                        .cmp(&right.manifest.default_order)
                })
                .then_with(|| left.manifest.id.cmp(&right.manifest.id))
        });

        let mut unknown_preference_ids = preferences
            .iter()
            .filter(|preference| !manifest_ids.contains(&preference.module_id))
            .map(|preference| preference.module_id.clone())
            .collect::<Vec<_>>();
        unknown_preference_ids.sort();
        Ok(EffectiveModules {
            modules,
            unknown_preference_ids,
        })
    }

    #[must_use]
    pub fn manifests(&self) -> &[ModuleManifest] {
        &self.manifests
    }
}

pub fn compiled_module_registry() -> Result<ModuleRegistry, DomainError> {
    ModuleRegistry::new(vec![
        ModuleManifest {
            id: ModuleId::parse(COMPILED_MODULE_IDS[0])?,
            display_name: "Task Manager".to_owned(),
            description: "Inspect current system and process resource usage.".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            route: "/task-manager".to_owned(),
            default_order: 100,
            default_enabled: true,
            capabilities: vec![ModuleCapability::TaskManagerRead],
            dependencies: vec![],
            linux_required: true,
            health: ModuleHealth::available(),
        },
        ModuleManifest {
            id: ModuleId::parse(COMPILED_MODULE_IDS[1])?,
            display_name: "Systemd".to_owned(),
            description: "Inspect user and system services and timers.".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            route: "/systemd".to_owned(),
            default_order: 200,
            default_enabled: true,
            capabilities: vec![
                ModuleCapability::SystemdUserRead,
                ModuleCapability::SystemdSystemRead,
            ],
            dependencies: vec![],
            linux_required: true,
            health: ModuleHealth::unavailable("The systemd module is not implemented yet."),
        },
        ModuleManifest {
            id: ModuleId::parse(COMPILED_MODULE_IDS[2])?,
            display_name: "Launcher".to_owned(),
            description: "Open saved local resources and applications.".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            route: "/launcher".to_owned(),
            default_order: 300,
            default_enabled: true,
            capabilities: vec![
                ModuleCapability::LauncherRead,
                ModuleCapability::LauncherWrite,
                ModuleCapability::LauncherExecute,
            ],
            dependencies: vec![],
            linux_required: true,
            health: ModuleHealth::unavailable("The launcher module is not implemented yet."),
        },
    ])
}

fn validate_manifests(manifests: &[ModuleManifest]) -> Result<(), DomainError> {
    let mut ids = HashSet::new();
    let mut routes = HashSet::new();
    for manifest in manifests {
        if !ids.insert(manifest.id.clone()) || !routes.insert(manifest.route.as_str()) {
            return Err(DomainError::new(ErrorCode::ModuleDuplicate, None));
        }
        if !manifest.route.starts_with('/')
            || manifest.route == "/"
            || manifest.capabilities.is_empty()
        {
            return Err(DomainError::new(ErrorCode::ModuleDependencyInvalid, None));
        }
    }
    for manifest in manifests {
        if manifest
            .dependencies
            .iter()
            .any(|dependency| dependency == &manifest.id || !ids.contains(dependency))
        {
            return Err(DomainError::new(ErrorCode::ModuleDependencyInvalid, None));
        }
    }

    let graph = manifests
        .iter()
        .map(|manifest| (&manifest.id, manifest.dependencies.as_slice()))
        .collect::<BTreeMap<_, _>>();
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for id in graph.keys() {
        visit(id, &graph, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn visit<'a>(
    id: &'a ModuleId,
    graph: &BTreeMap<&'a ModuleId, &'a [ModuleId]>,
    visiting: &mut HashSet<&'a ModuleId>,
    visited: &mut HashSet<&'a ModuleId>,
) -> Result<(), DomainError> {
    if visited.contains(id) {
        return Ok(());
    }
    if !visiting.insert(id) {
        return Err(DomainError::new(ErrorCode::ModuleDependencyInvalid, None));
    }
    for dependency in graph.get(id).copied().unwrap_or_default() {
        visit(dependency, graph, visiting, visited)?;
    }
    visiting.remove(id);
    visited.insert(id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(
        id: &str,
        route: &str,
        dependencies: &[&str],
    ) -> Result<ModuleManifest, DomainError> {
        Ok(ModuleManifest {
            id: ModuleId::parse(id)?,
            display_name: id.to_owned(),
            description: id.to_owned(),
            version: "1".to_owned(),
            route: route.to_owned(),
            default_order: 100,
            default_enabled: true,
            capabilities: vec![ModuleCapability::TaskManagerRead],
            dependencies: dependencies
                .iter()
                .map(|dependency| ModuleId::parse(*dependency))
                .collect::<Result<Vec<_>, _>>()?,
            linux_required: true,
            health: ModuleHealth::available(),
        })
    }

    #[test]
    fn registry_rejects_duplicates_missing_dependencies_and_cycles() -> Result<(), DomainError> {
        let duplicate = manifest("one", "/one", &[])?;
        assert!(ModuleRegistry::new(vec![duplicate.clone(), duplicate]).is_err());
        assert!(ModuleRegistry::new(vec![manifest("one", "/one", &["missing"])?]).is_err());
        assert!(
            ModuleRegistry::new(vec![
                manifest("one", "/one", &["two"])?,
                manifest("two", "/two", &["one"])?,
            ])
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn preferences_order_deterministically_and_dependency_health_stays_separate()
    -> Result<(), DomainError> {
        let registry = ModuleRegistry::new(vec![
            manifest("one", "/one", &[])?,
            manifest("two", "/two", &["one"])?,
        ])?;
        let result = registry.effective(&[
            ModulePreference {
                module_id: ModuleId::parse("one")?,
                enabled: Some(false),
                order: Some(20),
            },
            ModulePreference {
                module_id: ModuleId::parse("two")?,
                enabled: None,
                order: Some(10),
            },
            ModulePreference {
                module_id: ModuleId::parse("unknown")?,
                enabled: Some(true),
                order: None,
            },
        ])?;

        assert_eq!(result.modules[0].manifest.id.as_str(), "two");
        assert_eq!(
            result.modules[0].health.state,
            ModuleHealthState::Unavailable
        );
        assert_eq!(result.modules[1].enablement, ModuleEnablement::Disabled);
        assert_eq!(result.unknown_preference_ids[0].as_str(), "unknown");
        Ok(())
    }
}
