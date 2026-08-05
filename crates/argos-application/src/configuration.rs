use argos_domain::{
    ActionClassification, ActorContext, BootstrapConfig, BootstrapConfigRepository, DomainError,
    ThemePreference,
};

use crate::ApplicationError;

pub struct ConfigurationService<R> {
    repository: R,
}

impl<R> ConfigurationService<R> {
    #[must_use]
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }

    #[must_use]
    pub const fn read_classification() -> ActionClassification {
        ActionClassification::Read
    }

    #[must_use]
    pub const fn write_classification() -> ActionClassification {
        ActionClassification::Write
    }
}

impl<R> ConfigurationService<R>
where
    R: BootstrapConfigRepository,
{
    pub fn read(&self, actor: &ActorContext) -> Result<BootstrapConfig, ApplicationError> {
        self.repository
            .read()
            .map_err(|error| config_error(error, actor))
    }

    pub fn set_theme(
        &self,
        actor: &ActorContext,
        theme: ThemePreference,
    ) -> Result<BootstrapConfig, ApplicationError> {
        let mut config = self
            .repository
            .read()
            .map_err(|error| config_error(error, actor))?;
        config.theme = theme;
        config.theme_warning = false;
        self.repository
            .write(&config)
            .map_err(|error| config_error(error, actor))?;
        Ok(config)
    }
}

fn config_error(error: DomainError, actor: &ActorContext) -> ApplicationError {
    ApplicationError::from_domain(error, actor.correlation_id(), false)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use argos_domain::{BootstrapConfigRepository, CorrelationId, DomainError, RuntimeProfile};

    use super::*;

    struct MemoryConfig(Mutex<BootstrapConfig>);

    impl BootstrapConfigRepository for MemoryConfig {
        fn read(&self) -> Result<BootstrapConfig, DomainError> {
            self.0
                .lock()
                .map(|config| config.clone())
                .map_err(|_cause| DomainError::new(argos_domain::ErrorCode::ConfigInvalid, None))
        }

        fn write(&self, config: &BootstrapConfig) -> Result<(), DomainError> {
            *self.0.lock().map_err(|_cause| {
                DomainError::new(argos_domain::ErrorCode::ConfigInvalid, None)
            })? = config.clone();
            Ok(())
        }
    }

    #[test]
    fn theme_update_preserves_other_bootstrap_values() -> Result<(), Box<dyn std::error::Error>> {
        let service = ConfigurationService::new(MemoryConfig(Mutex::new(BootstrapConfig {
            version: 1,
            theme: ThemePreference::System,
            theme_warning: true,
            executable_search_paths: vec!["/usr/bin".to_owned()],
        })));
        let actor = ActorContext::local_human(CorrelationId::new());

        let result = service.set_theme(&actor, ThemePreference::Dark)?;

        assert_eq!(result.theme, ThemePreference::Dark);
        assert!(!result.theme_warning);
        assert_eq!(result.executable_search_paths, ["/usr/bin"]);
        assert_eq!(RuntimeProfile::default(), RuntimeProfile::Development);
        assert_eq!(
            ConfigurationService::<MemoryConfig>::write_classification(),
            ActionClassification::Write
        );
        Ok(())
    }
}
