use argos_domain::{
    CorrelationId as DomainCorrelationId, ErrorCode as DomainErrorCode,
    ErrorDetails as DomainErrorDetails,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::CorrelationId;

/// Stable frontend error-code union.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export_to = "AppErrorCode.ts")]
pub enum AppErrorCode {
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

impl AppErrorCode {
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
}

impl From<DomainErrorCode> for AppErrorCode {
    fn from(value: DomainErrorCode) -> Self {
        match value {
            DomainErrorCode::CoreInternal => Self::CoreInternal,
            DomainErrorCode::CoreCancelled => Self::CoreCancelled,
            DomainErrorCode::ConfigHomeUnavailable => Self::ConfigHomeUnavailable,
            DomainErrorCode::ConfigInvalid => Self::ConfigInvalid,
            DomainErrorCode::ConfigPathUnsafe => Self::ConfigPathUnsafe,
            DomainErrorCode::ConfigProductionAckRequired => Self::ConfigProductionAckRequired,
            DomainErrorCode::StorageUnavailable => Self::StorageUnavailable,
            DomainErrorCode::StorageBusy => Self::StorageBusy,
            DomainErrorCode::StorageIntegrityFailed => Self::StorageIntegrityFailed,
            DomainErrorCode::StorageMigrationFailed => Self::StorageMigrationFailed,
            DomainErrorCode::StorageAuditFailed => Self::StorageAuditFailed,
            DomainErrorCode::ModuleDuplicate => Self::ModuleDuplicate,
            DomainErrorCode::ModuleDependencyInvalid => Self::ModuleDependencyInvalid,
            DomainErrorCode::ModuleDisabled => Self::ModuleDisabled,
            DomainErrorCode::ModuleUnavailable => Self::ModuleUnavailable,
            DomainErrorCode::SystemdBusUnavailable => Self::SystemdBusUnavailable,
            DomainErrorCode::SystemdManagerUnavailable => Self::SystemdManagerUnavailable,
            DomainErrorCode::SystemdPermissionDenied => Self::SystemdPermissionDenied,
            DomainErrorCode::SystemdTimeout => Self::SystemdTimeout,
            DomainErrorCode::SystemdUnitNotFound => Self::SystemdUnitNotFound,
            DomainErrorCode::SystemdJournalUnavailable => Self::SystemdJournalUnavailable,
            DomainErrorCode::SystemdJournalParseFailed => Self::SystemdJournalParseFailed,
            DomainErrorCode::ProcessTargetNotFound => Self::ProcessTargetNotFound,
            DomainErrorCode::ProcessNotExecutable => Self::ProcessNotExecutable,
            DomainErrorCode::ProcessOpenFailed => Self::ProcessOpenFailed,
            DomainErrorCode::ProcessSpawnFailed => Self::ProcessSpawnFailed,
            DomainErrorCode::LauncherNotFound => Self::LauncherNotFound,
            DomainErrorCode::LauncherConflict => Self::LauncherConflict,
            DomainErrorCode::LauncherKindUnsupported => Self::LauncherKindUnsupported,
            DomainErrorCode::PermissionDenied => Self::PermissionDenied,
            DomainErrorCode::ValidationRequired => Self::ValidationRequired,
            DomainErrorCode::ValidationInvalidFormat => Self::ValidationInvalidFormat,
            DomainErrorCode::ValidationOutOfRange => Self::ValidationOutOfRange,
        }
    }
}

/// Allowlisted optional fields for code-specific public error context.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "AppErrorDetails.ts")]
pub struct AppErrorDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_display: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side_effect_may_have_occurred: Option<bool>,
}

impl From<&DomainErrorDetails> for AppErrorDetails {
    fn from(value: &DomainErrorDetails) -> Self {
        Self {
            field: value.field().map(str::to_owned),
            scope: value.scope().map(str::to_owned),
            module_id: value.module_id().map(str::to_owned),
            target_display: value.target_display().map(str::to_owned),
            side_effect_may_have_occurred: value.side_effect_may_have_occurred(),
        }
    }
}

/// Stable, safe error contract returned by every frontend-facing failure.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[ts(export_to = "AppError.ts")]
pub struct AppError {
    pub code: AppErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Box<AppErrorDetails>>,
    pub retryable: bool,
    pub correlation_id: CorrelationId,
}

impl AppError {
    /// Builds the wire contract only from the safe application-facing parts.
    #[must_use]
    pub fn from_safe_parts(
        code: DomainErrorCode,
        details: Option<&DomainErrorDetails>,
        retryable: bool,
        correlation_id: DomainCorrelationId,
    ) -> Self {
        Self {
            code: code.into(),
            message: code.safe_message().to_owned(),
            details: details.map(|value| Box::new(AppErrorDetails::from(value))),
            retryable,
            correlation_id: correlation_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use argos_domain::{ErrorDetailError, ErrorDetails};

    use super::*;

    #[test]
    fn every_error_code_round_trips_with_the_stable_wire_name() -> Result<(), serde_json::Error> {
        for (domain, contract) in DomainErrorCode::ALL.into_iter().zip(AppErrorCode::ALL) {
            assert_eq!(AppErrorCode::from(domain), contract);

            let json = serde_json::to_string(&contract)?;
            assert_eq!(json, format!("\"{}\"", domain.as_str()));
            assert_eq!(serde_json::from_str::<AppErrorCode>(&json)?, contract);
        }

        Ok(())
    }

    #[test]
    fn every_public_error_detail_shape_round_trips() -> Result<(), Box<dyn std::error::Error>> {
        let details = [
            None,
            Some(ErrorDetails::for_field("title")?),
            Some(ErrorDetails::for_scope("user")?),
            Some(ErrorDetails::for_module("systemd")?),
            Some(ErrorDetails::for_target_display("Example")?),
            Some(ErrorDetails::for_side_effect(true)),
        ];

        for detail in &details {
            let error = AppError::from_safe_parts(
                DomainErrorCode::ValidationInvalidFormat,
                detail.as_ref(),
                false,
                DomainCorrelationId::new(),
            );
            let json = serde_json::to_string(&error)?;
            let decoded = serde_json::from_str::<AppError>(&json)?;

            assert_eq!(decoded, error);
            assert_eq!(decoded.message, "A value has an invalid format.");
        }

        Ok(())
    }

    #[test]
    fn detail_validation_error_remains_safe() {
        let Err(error) = ErrorDetails::for_field("") else {
            panic!("empty detail must be rejected");
        };
        assert_eq!(error, ErrorDetailError);
        assert_eq!(error.to_string(), "public error detail is invalid");
    }
}
