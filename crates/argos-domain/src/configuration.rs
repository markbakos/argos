use crate::{ActionClassification, DomainError};

pub const BOOTSTRAP_CONFIG_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RuntimeProfile {
    Production,
    #[default]
    Development,
    Test,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapConfig {
    pub version: u16,
    pub theme: ThemePreference,
    pub theme_warning: bool,
    pub executable_search_paths: Vec<String>,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            version: BOOTSTRAP_CONFIG_VERSION,
            theme: ThemePreference::System,
            theme_warning: false,
            executable_search_paths: Vec::new(),
        }
    }
}

pub trait BootstrapConfigRepository: Send + Sync {
    fn read(&self) -> Result<BootstrapConfig, DomainError>;

    fn write(&self, config: &BootstrapConfig) -> Result<(), DomainError>;
}

impl ThemePreference {
    #[must_use]
    pub const fn classification() -> ActionClassification {
        ActionClassification::Write
    }
}
