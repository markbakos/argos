use std::{
    collections::HashSet,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use argos_domain::{
    BOOTSTRAP_CONFIG_VERSION, BootstrapConfig, BootstrapConfigRepository, DomainError, ErrorCode,
    RuntimeProfile, ThemePreference,
};
use serde::{Deserialize, Serialize};

const MAX_CONFIG_BYTES: u64 = 64 * 1_024;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PathEnvironment {
    pub home: Option<PathBuf>,
    pub xdg_config_home: Option<PathBuf>,
    pub xdg_data_home: Option<PathBuf>,
    pub xdg_state_home: Option<PathBuf>,
    pub xdg_cache_home: Option<PathBuf>,
    pub xdg_runtime_dir: Option<PathBuf>,
    pub argos_home: Option<PathBuf>,
    pub argos_profile: Option<OsString>,
    pub production_acknowledgement: Option<OsString>,
}

impl PathEnvironment {
    #[must_use]
    pub fn from_process() -> Self {
        Self {
            home: std::env::var_os("HOME").map(PathBuf::from),
            xdg_config_home: std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
            xdg_data_home: std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
            xdg_state_home: std::env::var_os("XDG_STATE_HOME").map(PathBuf::from),
            xdg_cache_home: std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from),
            xdg_runtime_dir: std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from),
            argos_home: std::env::var_os("ARGOS_HOME").map(PathBuf::from),
            argos_profile: std::env::var_os("ARGOS_PROFILE"),
            production_acknowledgement: std::env::var_os("ARGOS_ACKNOWLEDGE_PRODUCTION_DATA"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedPaths {
    pub profile: RuntimeProfile,
    pub config: PathBuf,
    pub data: PathBuf,
    pub state: PathBuf,
    pub cache: PathBuf,
    pub runtime: Option<PathBuf>,
    pub production_data_warning: bool,
    repository_root: Option<PathBuf>,
}

impl ResolvedPaths {
    pub fn for_test(root: &Path) -> Result<Self, DomainError> {
        validate_absolute(root)?;
        Ok(Self {
            profile: RuntimeProfile::Test,
            config: root.join("config"),
            data: root.join("data"),
            state: root.join("state"),
            cache: root.join("cache"),
            runtime: Some(root.join("runtime")),
            production_data_warning: false,
            repository_root: None,
        })
    }
}

pub fn resolve_paths(
    embedded_profile: RuntimeProfile,
    environment: &PathEnvironment,
    repository_root: Option<&Path>,
) -> Result<ResolvedPaths, DomainError> {
    if embedded_profile == RuntimeProfile::Test {
        return Err(config_error(ErrorCode::ConfigPathUnsafe));
    }

    let requested_production = environment.argos_profile.as_deref() == Some("production".as_ref());
    let acknowledged =
        environment.production_acknowledgement.as_deref() == Some("argos-production".as_ref());
    if requested_production != acknowledged {
        return Err(config_error(ErrorCode::ConfigProductionAckRequired));
    }
    if environment
        .argos_profile
        .as_deref()
        .is_some_and(|profile| profile != "production")
    {
        return Err(config_error(ErrorCode::ConfigInvalid));
    }

    let profile = if embedded_profile == RuntimeProfile::Production || requested_production {
        RuntimeProfile::Production
    } else {
        RuntimeProfile::Development
    };
    if environment.argos_home.is_some()
        && (profile != RuntimeProfile::Development || requested_production)
    {
        return Err(config_error(ErrorCode::ConfigPathUnsafe));
    }

    let mut paths = if let Some(root) = environment.argos_home.as_deref() {
        validate_absolute(root)?;
        ResolvedPaths {
            profile,
            config: root.join("config"),
            data: root.join("data"),
            state: root.join("state"),
            cache: root.join("cache"),
            runtime: Some(root.join("runtime")),
            production_data_warning: false,
            repository_root: repository_root.map(Path::to_path_buf),
        }
    } else {
        let namespace = match profile {
            RuntimeProfile::Production => "argos",
            RuntimeProfile::Development => "argos-dev",
            RuntimeProfile::Test => return Err(config_error(ErrorCode::ConfigPathUnsafe)),
        };
        let home = environment.home.as_deref();
        let config = xdg_category(
            environment.xdg_config_home.as_deref(),
            home,
            ".config",
            namespace,
        )?;
        let data = xdg_category(
            environment.xdg_data_home.as_deref(),
            home,
            ".local/share",
            namespace,
        )?;
        let state = xdg_category(
            environment.xdg_state_home.as_deref(),
            home,
            ".local/state",
            namespace,
        )?;
        let cache = xdg_category(
            environment.xdg_cache_home.as_deref(),
            home,
            ".cache",
            namespace,
        )?;
        let runtime = environment
            .xdg_runtime_dir
            .as_deref()
            .filter(|path| is_safe_absolute(path))
            .map(|path| path.join(namespace));
        ResolvedPaths {
            profile,
            config,
            data,
            state,
            cache,
            runtime,
            production_data_warning: embedded_profile == RuntimeProfile::Development
                && profile == RuntimeProfile::Production,
            repository_root: repository_root.map(Path::to_path_buf),
        }
    };

    validate_distinct(&paths)?;
    if let Some(repository_root) = repository_root {
        for path in [&paths.config, &paths.data, &paths.state, &paths.cache] {
            reject_repository_path(path, repository_root)?;
        }
        if let Some(runtime) = &paths.runtime {
            reject_repository_path(runtime, repository_root)?;
        }
    }
    paths.runtime = paths.runtime.filter(|path| is_safe_absolute(path));
    Ok(paths)
}

fn xdg_category(
    configured: Option<&Path>,
    home: Option<&Path>,
    fallback: &str,
    namespace: &str,
) -> Result<PathBuf, DomainError> {
    if let Some(path) = configured.filter(|path| is_safe_absolute(path)) {
        return Ok(path.join(namespace));
    }
    let home = home.ok_or_else(|| config_error(ErrorCode::ConfigHomeUnavailable))?;
    validate_absolute(home)?;
    Ok(home.join(fallback).join(namespace))
}

fn validate_distinct(paths: &ResolvedPaths) -> Result<(), DomainError> {
    let mut categories = HashSet::new();
    for path in [&paths.config, &paths.data, &paths.state, &paths.cache] {
        if !categories.insert(path) {
            return Err(config_error(ErrorCode::ConfigPathUnsafe));
        }
    }
    if paths
        .runtime
        .as_ref()
        .is_some_and(|runtime| categories.contains(runtime))
    {
        return Err(config_error(ErrorCode::ConfigPathUnsafe));
    }
    Ok(())
}

fn reject_repository_path(path: &Path, repository_root: &Path) -> Result<(), DomainError> {
    validate_absolute(repository_root)?;
    if path == repository_root || path.starts_with(repository_root) {
        return Err(config_error(ErrorCode::ConfigPathUnsafe));
    }
    Ok(())
}

fn validate_absolute(path: &Path) -> Result<(), DomainError> {
    if !is_safe_absolute(path) {
        return Err(config_error(ErrorCode::ConfigPathUnsafe));
    }
    Ok(())
}

fn is_safe_absolute(path: &Path) -> bool {
    path.is_absolute()
        && !path.as_os_str().is_empty()
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

#[derive(Debug)]
pub struct LinuxConfigStore {
    paths: ResolvedPaths,
}

impl LinuxConfigStore {
    #[must_use]
    pub const fn new(paths: ResolvedPaths) -> Self {
        Self { paths }
    }

    #[must_use]
    pub fn paths(&self) -> &ResolvedPaths {
        &self.paths
    }

    fn config_path(&self) -> PathBuf {
        self.paths.config.join("config.toml")
    }
}

impl BootstrapConfigRepository for LinuxConfigStore {
    fn read(&self) -> Result<BootstrapConfig, DomainError> {
        validate_config_location(&self.paths)?;
        let path = self.config_path();
        reject_unsafe_existing_file(&path)?;
        let mut bytes = Vec::new();
        match File::open(&path) {
            Ok(file) => {
                file.take(MAX_CONFIG_BYTES.saturating_add(1))
                    .read_to_end(&mut bytes)
                    .map_err(|_cause| config_error(ErrorCode::ConfigInvalid))?;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(BootstrapConfig::default());
            }
            Err(_cause) => return Err(config_error(ErrorCode::ConfigInvalid)),
        }
        if bytes.len() > usize::try_from(MAX_CONFIG_BYTES).unwrap_or(usize::MAX) {
            return Err(config_error(ErrorCode::ConfigInvalid));
        }
        let stored: StoredConfig =
            toml::from_slice(&bytes).map_err(|_cause| config_error(ErrorCode::ConfigInvalid))?;
        stored.try_into()
    }

    fn write(&self, config: &BootstrapConfig) -> Result<(), DomainError> {
        validate_config_location(&self.paths)?;
        let stored = StoredConfig::try_from(config)?;
        let contents = toml::to_string_pretty(&stored)
            .map_err(|_cause| config_error(ErrorCode::ConfigInvalid))?;
        ensure_private_directory(&self.paths.config)?;
        let target = self.config_path();
        reject_unsafe_existing_file(&target)?;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_cause| config_error(ErrorCode::ConfigInvalid))?
            .as_nanos();
        let temporary = self
            .paths
            .config
            .join(format!(".config.toml.{}-{unique}.tmp", std::process::id()));
        let result = (|| -> io::Result<()> {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&temporary)?;
            file.write_all(contents.as_bytes())?;
            file.sync_all()?;
            fs::rename(&temporary, &target)?;
            File::open(&self.paths.config)?.sync_all()
        })();
        if result.is_err() {
            let _ignored = fs::remove_file(&temporary);
            return Err(config_error(ErrorCode::ConfigInvalid));
        }
        Ok(())
    }
}

fn validate_config_location(paths: &ResolvedPaths) -> Result<(), DomainError> {
    if let Some(repository_root) = paths.repository_root.as_deref() {
        let repository_root = fs::canonicalize(repository_root)
            .map_err(|_cause| config_error(ErrorCode::ConfigPathUnsafe))?;
        let mut existing = paths.config.as_path();
        while fs::symlink_metadata(existing).is_err() {
            existing = existing
                .parent()
                .ok_or_else(|| config_error(ErrorCode::ConfigPathUnsafe))?;
        }
        let resolved = fs::canonicalize(existing)
            .map_err(|_cause| config_error(ErrorCode::ConfigPathUnsafe))?;
        if resolved == repository_root || resolved.starts_with(repository_root) {
            return Err(config_error(ErrorCode::ConfigPathUnsafe));
        }
    }
    match fs::symlink_metadata(&paths.config) {
        Ok(metadata)
            if metadata.file_type().is_symlink()
                || !metadata.is_dir()
                || metadata.uid() != rustix::process::getuid().as_raw()
                || metadata.permissions().mode() & 0o077 != 0 =>
        {
            Err(config_error(ErrorCode::ConfigPathUnsafe))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_cause) => Err(config_error(ErrorCode::ConfigPathUnsafe)),
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct StoredConfig {
    version: u16,
    theme: String,
    #[serde(default)]
    executable_search_paths: Vec<String>,
}

impl TryFrom<StoredConfig> for BootstrapConfig {
    type Error = DomainError;

    fn try_from(value: StoredConfig) -> Result<Self, Self::Error> {
        if value.version != BOOTSTRAP_CONFIG_VERSION {
            return Err(config_error(ErrorCode::ConfigInvalid));
        }
        let (theme, theme_warning) = match value.theme.as_str() {
            "system" => (ThemePreference::System, false),
            "light" => (ThemePreference::Light, false),
            "dark" => (ThemePreference::Dark, false),
            _ => (ThemePreference::System, true),
        };
        let executable_search_paths = validate_executable_paths(value.executable_search_paths)?;
        Ok(Self {
            version: value.version,
            theme,
            theme_warning,
            executable_search_paths,
        })
    }
}

impl TryFrom<&BootstrapConfig> for StoredConfig {
    type Error = DomainError;

    fn try_from(value: &BootstrapConfig) -> Result<Self, Self::Error> {
        if value.version != BOOTSTRAP_CONFIG_VERSION {
            return Err(config_error(ErrorCode::ConfigInvalid));
        }
        Ok(Self {
            version: value.version,
            theme: match value.theme {
                ThemePreference::System => "system",
                ThemePreference::Light => "light",
                ThemePreference::Dark => "dark",
            }
            .to_owned(),
            executable_search_paths: validate_executable_paths(
                value.executable_search_paths.clone(),
            )?,
        })
    }
}

fn validate_executable_paths(paths: Vec<String>) -> Result<Vec<String>, DomainError> {
    let mut seen = HashSet::new();
    let mut validated = Vec::with_capacity(paths.len());
    for value in paths {
        let path = Path::new(&value);
        validate_absolute(path)?;
        if !path.is_dir() {
            return Err(config_error(ErrorCode::ConfigInvalid));
        }
        if seen.insert(value.clone()) {
            validated.push(value);
        }
    }
    Ok(validated)
}

fn ensure_private_directory(path: &Path) -> Result<(), DomainError> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink()
            || !metadata.is_dir()
            || metadata.uid() != rustix::process::getuid().as_raw()
        {
            return Err(config_error(ErrorCode::ConfigPathUnsafe));
        }
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_cause| config_error(ErrorCode::ConfigPathUnsafe))?;
        return Ok(());
    }

    fs::create_dir_all(path).map_err(|_cause| config_error(ErrorCode::ConfigPathUnsafe))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_cause| config_error(ErrorCode::ConfigPathUnsafe))?;
    Ok(())
}

fn reject_unsafe_existing_file(path: &Path) -> Result<(), DomainError> {
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.file_type().is_symlink()
                || !metadata.is_file()
                || metadata.uid() != rustix::process::getuid().as_raw()
                || metadata.permissions().mode() & 0o077 != 0 =>
        {
            Err(config_error(ErrorCode::ConfigPathUnsafe))
        }
        Ok(_) | Err(_) => Ok(()),
    }
}

fn config_error(code: ErrorCode) -> DomainError {
    DomainError::new(code, None)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use super::*;

    fn temporary_root() -> io::Result<PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("argos-config-{}-{unique}", std::process::id()));
        fs::create_dir_all(&root)?;
        Ok(root)
    }

    #[test]
    fn development_paths_use_xdg_and_home_fallbacks_without_runtime_fallback()
    -> Result<(), DomainError> {
        let environment = PathEnvironment {
            home: Some(PathBuf::from("/home/synthetic")),
            xdg_config_home: Some(PathBuf::from("relative")),
            xdg_data_home: Some(PathBuf::from("/data")),
            ..PathEnvironment::default()
        };

        let paths = resolve_paths(
            RuntimeProfile::Development,
            &environment,
            Some(Path::new("/workspace/argos")),
        )?;

        assert_eq!(paths.config, Path::new("/home/synthetic/.config/argos-dev"));
        assert_eq!(paths.data, Path::new("/data/argos-dev"));
        assert_eq!(
            paths.state,
            Path::new("/home/synthetic/.local/state/argos-dev")
        );
        assert_eq!(paths.runtime, None);
        Ok(())
    }

    #[test]
    fn production_override_requires_both_exact_values_and_rejects_repository_roots() {
        let base = PathEnvironment {
            home: Some(PathBuf::from("/home/synthetic")),
            argos_profile: Some("production".into()),
            ..PathEnvironment::default()
        };
        assert!(
            resolve_paths(
                RuntimeProfile::Development,
                &base,
                Some(Path::new("/workspace"))
            )
            .is_err()
        );

        let acknowledged = PathEnvironment {
            production_acknowledgement: Some("argos-production".into()),
            ..base
        };
        let paths = resolve_paths(
            RuntimeProfile::Development,
            &acknowledged,
            Some(Path::new("/workspace")),
        );
        assert!(paths.is_ok_and(|paths| paths.production_data_warning));

        let unsafe_home = PathEnvironment {
            home: Some(PathBuf::from("/home/synthetic")),
            argos_home: Some(PathBuf::from("/workspace/argos-data")),
            ..PathEnvironment::default()
        };
        assert!(
            resolve_paths(
                RuntimeProfile::Development,
                &unsafe_home,
                Some(Path::new("/workspace")),
            )
            .is_err()
        );
    }

    #[test]
    fn config_round_trips_atomically_and_rejects_unsafe_executable_paths()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temporary_root()?;
        let paths = ResolvedPaths::for_test(&root)?;
        let store = LinuxConfigStore::new(paths);
        let valid_path = root.join("bin");
        fs::create_dir(&valid_path)?;
        let config = BootstrapConfig {
            version: BOOTSTRAP_CONFIG_VERSION,
            theme: ThemePreference::Dark,
            theme_warning: false,
            executable_search_paths: vec![valid_path.to_string_lossy().into_owned()],
        };

        store.write(&config)?;

        assert_eq!(store.read()?, config);
        assert_eq!(
            fs::metadata(store.config_path())?.permissions().mode() & 0o777,
            0o600
        );
        assert!(
            store
                .write(&BootstrapConfig {
                    executable_search_paths: vec!["relative".to_owned()],
                    ..BootstrapConfig::default()
                })
                .is_err()
        );
        assert_eq!(store.read()?, config);
        fs::write(
            store.config_path(),
            "version = 1\ntheme = \"not-a-theme\"\n",
        )?;
        let recovered = store.read()?;
        assert_eq!(recovered.theme, ThemePreference::System);
        assert!(recovered.theme_warning);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn config_rejects_a_category_symlink_into_the_repository()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temporary_root()?;
        let repository = root.join("repository");
        let xdg_config = root.join("outside-config");
        fs::create_dir_all(&repository)?;
        fs::create_dir_all(&xdg_config)?;
        symlink(&repository, xdg_config.join("argos-dev"))?;
        let paths = resolve_paths(
            RuntimeProfile::Development,
            &PathEnvironment {
                home: Some(root.join("home")),
                xdg_config_home: Some(xdg_config),
                xdg_data_home: Some(root.join("data")),
                xdg_state_home: Some(root.join("state")),
                xdg_cache_home: Some(root.join("cache")),
                ..PathEnvironment::default()
            },
            Some(&repository),
        )?;

        assert_eq!(
            LinuxConfigStore::new(paths).read().map(|_| ()),
            Err(config_error(ErrorCode::ConfigPathUnsafe))
        );
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
