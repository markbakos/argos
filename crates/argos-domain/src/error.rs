use std::fmt;

/// Maximum number of Unicode scalar values in one public error detail.
pub const MAX_ERROR_DETAIL_CHARACTERS: usize = 200;

/// Stable error-code namespace used for ownership and UI fallback handling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorNamespace {
    Core,
    Config,
    Storage,
    Module,
    Systemd,
    Process,
    Launcher,
    Permission,
    Validation,
}

/// Stable foundation error codes shared by domain and application logic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    CoreInternal,
    CoreCancelled,
    ConfigHomeUnavailable,
    ConfigInvalid,
    ConfigPathUnsafe,
    ConfigProductionAckRequired,
    StorageUnavailable,
    StorageBusy,
    StorageIntegrityFailed,
    StorageMigrationFailed,
    StorageAuditFailed,
    ModuleDuplicate,
    ModuleDependencyInvalid,
    ModuleDisabled,
    ModuleUnavailable,
    SystemdBusUnavailable,
    SystemdManagerUnavailable,
    SystemdPermissionDenied,
    SystemdTimeout,
    SystemdUnitNotFound,
    SystemdJournalUnavailable,
    SystemdJournalParseFailed,
    ProcessTargetNotFound,
    ProcessNotExecutable,
    ProcessOpenFailed,
    ProcessSpawnFailed,
    LauncherNotFound,
    LauncherConflict,
    LauncherKindUnsupported,
    PermissionDenied,
    ValidationRequired,
    ValidationInvalidFormat,
    ValidationOutOfRange,
}

impl ErrorCode {
    pub const ALL: [Self; 33] = [
        Self::CoreInternal,
        Self::CoreCancelled,
        Self::ConfigHomeUnavailable,
        Self::ConfigInvalid,
        Self::ConfigPathUnsafe,
        Self::ConfigProductionAckRequired,
        Self::StorageUnavailable,
        Self::StorageBusy,
        Self::StorageIntegrityFailed,
        Self::StorageMigrationFailed,
        Self::StorageAuditFailed,
        Self::ModuleDuplicate,
        Self::ModuleDependencyInvalid,
        Self::ModuleDisabled,
        Self::ModuleUnavailable,
        Self::SystemdBusUnavailable,
        Self::SystemdManagerUnavailable,
        Self::SystemdPermissionDenied,
        Self::SystemdTimeout,
        Self::SystemdUnitNotFound,
        Self::SystemdJournalUnavailable,
        Self::SystemdJournalParseFailed,
        Self::ProcessTargetNotFound,
        Self::ProcessNotExecutable,
        Self::ProcessOpenFailed,
        Self::ProcessSpawnFailed,
        Self::LauncherNotFound,
        Self::LauncherConflict,
        Self::LauncherKindUnsupported,
        Self::PermissionDenied,
        Self::ValidationRequired,
        Self::ValidationInvalidFormat,
        Self::ValidationOutOfRange,
    ];

    /// Returns the stable wire identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CoreInternal => "CORE_INTERNAL",
            Self::CoreCancelled => "CORE_CANCELLED",
            Self::ConfigHomeUnavailable => "CONFIG_HOME_UNAVAILABLE",
            Self::ConfigInvalid => "CONFIG_INVALID",
            Self::ConfigPathUnsafe => "CONFIG_PATH_UNSAFE",
            Self::ConfigProductionAckRequired => "CONFIG_PRODUCTION_ACK_REQUIRED",
            Self::StorageUnavailable => "STORAGE_UNAVAILABLE",
            Self::StorageBusy => "STORAGE_BUSY",
            Self::StorageIntegrityFailed => "STORAGE_INTEGRITY_FAILED",
            Self::StorageMigrationFailed => "STORAGE_MIGRATION_FAILED",
            Self::StorageAuditFailed => "STORAGE_AUDIT_FAILED",
            Self::ModuleDuplicate => "MODULE_DUPLICATE",
            Self::ModuleDependencyInvalid => "MODULE_DEPENDENCY_INVALID",
            Self::ModuleDisabled => "MODULE_DISABLED",
            Self::ModuleUnavailable => "MODULE_UNAVAILABLE",
            Self::SystemdBusUnavailable => "SYSTEMD_BUS_UNAVAILABLE",
            Self::SystemdManagerUnavailable => "SYSTEMD_MANAGER_UNAVAILABLE",
            Self::SystemdPermissionDenied => "SYSTEMD_PERMISSION_DENIED",
            Self::SystemdTimeout => "SYSTEMD_TIMEOUT",
            Self::SystemdUnitNotFound => "SYSTEMD_UNIT_NOT_FOUND",
            Self::SystemdJournalUnavailable => "SYSTEMD_JOURNAL_UNAVAILABLE",
            Self::SystemdJournalParseFailed => "SYSTEMD_JOURNAL_PARSE_FAILED",
            Self::ProcessTargetNotFound => "PROCESS_TARGET_NOT_FOUND",
            Self::ProcessNotExecutable => "PROCESS_NOT_EXECUTABLE",
            Self::ProcessOpenFailed => "PROCESS_OPEN_FAILED",
            Self::ProcessSpawnFailed => "PROCESS_SPAWN_FAILED",
            Self::LauncherNotFound => "LAUNCHER_NOT_FOUND",
            Self::LauncherConflict => "LAUNCHER_CONFLICT",
            Self::LauncherKindUnsupported => "LAUNCHER_KIND_UNSUPPORTED",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::ValidationRequired => "VALIDATION_REQUIRED",
            Self::ValidationInvalidFormat => "VALIDATION_INVALID_FORMAT",
            Self::ValidationOutOfRange => "VALIDATION_OUT_OF_RANGE",
        }
    }

    #[must_use]
    pub const fn namespace(self) -> ErrorNamespace {
        match self {
            Self::CoreInternal | Self::CoreCancelled => ErrorNamespace::Core,
            Self::ConfigHomeUnavailable
            | Self::ConfigInvalid
            | Self::ConfigPathUnsafe
            | Self::ConfigProductionAckRequired => ErrorNamespace::Config,
            Self::StorageUnavailable
            | Self::StorageBusy
            | Self::StorageIntegrityFailed
            | Self::StorageMigrationFailed
            | Self::StorageAuditFailed => ErrorNamespace::Storage,
            Self::ModuleDuplicate
            | Self::ModuleDependencyInvalid
            | Self::ModuleDisabled
            | Self::ModuleUnavailable => ErrorNamespace::Module,
            Self::SystemdBusUnavailable
            | Self::SystemdManagerUnavailable
            | Self::SystemdPermissionDenied
            | Self::SystemdTimeout
            | Self::SystemdUnitNotFound
            | Self::SystemdJournalUnavailable
            | Self::SystemdJournalParseFailed => ErrorNamespace::Systemd,
            Self::ProcessTargetNotFound
            | Self::ProcessNotExecutable
            | Self::ProcessOpenFailed
            | Self::ProcessSpawnFailed => ErrorNamespace::Process,
            Self::LauncherNotFound | Self::LauncherConflict | Self::LauncherKindUnsupported => {
                ErrorNamespace::Launcher
            }
            Self::PermissionDenied => ErrorNamespace::Permission,
            Self::ValidationRequired
            | Self::ValidationInvalidFormat
            | Self::ValidationOutOfRange => ErrorNamespace::Validation,
        }
    }

    /// Returns a fixed user-safe summary. Internal causes never influence it.
    #[must_use]
    pub const fn safe_message(self) -> &'static str {
        match self {
            Self::CoreInternal => "Argos could not complete the request.",
            Self::CoreCancelled => "The request was cancelled.",
            Self::ConfigHomeUnavailable => "The application home directory is unavailable.",
            Self::ConfigInvalid => "The application configuration is invalid.",
            Self::ConfigPathUnsafe => "The selected application path is unsafe.",
            Self::ConfigProductionAckRequired => {
                "Production data access requires explicit acknowledgement."
            }
            Self::StorageUnavailable => "Application storage is unavailable.",
            Self::StorageBusy => "Application storage is busy.",
            Self::StorageIntegrityFailed => "Application storage failed an integrity check.",
            Self::StorageMigrationFailed => "Application storage could not be upgraded safely.",
            Self::StorageAuditFailed => "The required audit record could not be stored.",
            Self::ModuleDuplicate => "A module is registered more than once.",
            Self::ModuleDependencyInvalid => "A module dependency is invalid.",
            Self::ModuleDisabled => "The requested module is disabled.",
            Self::ModuleUnavailable => "The requested module is unavailable.",
            Self::SystemdBusUnavailable => "The systemd bus is unavailable.",
            Self::SystemdManagerUnavailable => "The systemd manager is unavailable.",
            Self::SystemdPermissionDenied => "Permission to read systemd data was denied.",
            Self::SystemdTimeout => "The systemd request timed out.",
            Self::SystemdUnitNotFound => "The requested systemd unit was not found.",
            Self::SystemdJournalUnavailable => "The system journal is unavailable.",
            Self::SystemdJournalParseFailed => "Recent journal entries could not be read safely.",
            Self::ProcessTargetNotFound => "The requested target was not found.",
            Self::ProcessNotExecutable => "The requested target is not executable.",
            Self::ProcessOpenFailed => "The requested target could not be opened.",
            Self::ProcessSpawnFailed => "The requested executable could not be started.",
            Self::LauncherNotFound => "The launcher item was not found.",
            Self::LauncherConflict => "The launcher item changed before this request completed.",
            Self::LauncherKindUnsupported => "The launcher item kind is unsupported.",
            Self::PermissionDenied => "Permission to complete the request was denied.",
            Self::ValidationRequired => "A required value is missing.",
            Self::ValidationInvalidFormat => "A value has an invalid format.",
            Self::ValidationOutOfRange => "A value is outside the allowed range.",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct BoundedErrorDetail(Option<String>);

impl BoundedErrorDetail {
    fn new(value: impl Into<String>) -> Result<Self, ErrorDetailError> {
        let value = value.into();
        let count = value.chars().count();

        if value.is_empty()
            || count > MAX_ERROR_DETAIL_CHARACTERS
            || value.chars().any(char::is_control)
        {
            return Err(ErrorDetailError);
        }

        Ok(Self(Some(value)))
    }

    fn as_deref(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

/// Allowlisted, bounded values that may cross a public error boundary.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ErrorDetails {
    field: BoundedErrorDetail,
    scope: BoundedErrorDetail,
    module_id: BoundedErrorDetail,
    target_display: BoundedErrorDetail,
    side_effect_may_have_occurred: Option<bool>,
}

impl ErrorDetails {
    pub fn for_field(value: impl Into<String>) -> Result<Self, ErrorDetailError> {
        Ok(Self {
            field: BoundedErrorDetail::new(value)?,
            ..Self::default()
        })
    }

    pub fn for_scope(value: impl Into<String>) -> Result<Self, ErrorDetailError> {
        Ok(Self {
            scope: BoundedErrorDetail::new(value)?,
            ..Self::default()
        })
    }

    pub fn for_module(value: impl Into<String>) -> Result<Self, ErrorDetailError> {
        Ok(Self {
            module_id: BoundedErrorDetail::new(value)?,
            ..Self::default()
        })
    }

    pub fn for_target_display(value: impl Into<String>) -> Result<Self, ErrorDetailError> {
        Ok(Self {
            target_display: BoundedErrorDetail::new(value)?,
            ..Self::default()
        })
    }

    #[must_use]
    pub fn for_side_effect(may_have_occurred: bool) -> Self {
        Self {
            side_effect_may_have_occurred: Some(may_have_occurred),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn field(&self) -> Option<&str> {
        self.field.as_deref()
    }

    #[must_use]
    pub fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }

    #[must_use]
    pub fn module_id(&self) -> Option<&str> {
        self.module_id.as_deref()
    }

    #[must_use]
    pub fn target_display(&self) -> Option<&str> {
        self.target_display.as_deref()
    }

    #[must_use]
    pub fn side_effect_may_have_occurred(&self) -> Option<bool> {
        self.side_effect_may_have_occurred
    }
}

/// Safe failure returned when an error detail is empty, oversized, or contains controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ErrorDetailError;

impl fmt::Display for ErrorDetailError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("public error detail is invalid")
    }
}

impl std::error::Error for ErrorDetailError {}

/// A typed domain failure containing only public-safe domain context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainError {
    code: ErrorCode,
    details: Option<ErrorDetails>,
}

impl DomainError {
    #[must_use]
    pub fn new(code: ErrorCode, details: Option<ErrorDetails>) -> Self {
        Self { code, details }
    }

    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    #[must_use]
    pub fn details(&self) -> Option<&ErrorDetails> {
        self.details.as_ref()
    }

    #[must_use]
    pub fn into_details(self) -> Option<ErrorDetails> {
        self.details
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code.safe_message())
    }
}

impl std::error::Error for DomainError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_has_a_matching_namespace_and_safe_message() {
        for code in ErrorCode::ALL {
            let expected_prefix = match code.namespace() {
                ErrorNamespace::Core => "CORE_",
                ErrorNamespace::Config => "CONFIG_",
                ErrorNamespace::Storage => "STORAGE_",
                ErrorNamespace::Module => "MODULE_",
                ErrorNamespace::Systemd => "SYSTEMD_",
                ErrorNamespace::Process => "PROCESS_",
                ErrorNamespace::Launcher => "LAUNCHER_",
                ErrorNamespace::Permission => "PERMISSION_",
                ErrorNamespace::Validation => "VALIDATION_",
            };

            assert!(code.as_str().starts_with(expected_prefix));
            assert!(!code.safe_message().is_empty());
        }
    }

    #[test]
    fn public_details_are_allowlisted_and_bounded() -> Result<(), ErrorDetailError> {
        let details = ErrorDetails::for_field("title")?;
        assert_eq!(details.field(), Some("title"));
        assert_eq!(details.scope(), None);

        assert!(ErrorDetails::for_field("").is_err());
        assert!(ErrorDetails::for_field("x".repeat(MAX_ERROR_DETAIL_CHARACTERS + 1)).is_err());
        assert!(ErrorDetails::for_field("title\nsecret").is_err());

        Ok(())
    }

    #[test]
    fn domain_error_display_is_fixed_by_code() -> Result<(), ErrorDetailError> {
        let error = DomainError::new(
            ErrorCode::ValidationInvalidFormat,
            Some(ErrorDetails::for_field("target")?),
        );

        assert_eq!(error.to_string(), "A value has an invalid format.");
        assert_eq!(error.code(), ErrorCode::ValidationInvalidFormat);

        Ok(())
    }
}
