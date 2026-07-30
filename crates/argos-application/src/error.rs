use std::{error::Error, fmt};

use argos_domain::{CorrelationId, DomainError, ErrorCode, ErrorDetails};

type InternalCause = Box<dyn Error + Send + Sync + 'static>;

/// Application failure that retains a private cause for later redacted diagnostics.
#[derive(Debug)]
pub struct ApplicationError {
    code: ErrorCode,
    details: Option<ErrorDetails>,
    retryable: bool,
    correlation_id: CorrelationId,
    internal_cause: Option<InternalCause>,
}

impl ApplicationError {
    /// Attaches request context and retry policy to a safe domain failure.
    #[must_use]
    pub fn from_domain(error: DomainError, correlation_id: CorrelationId, retryable: bool) -> Self {
        let code = error.code();
        Self {
            code,
            details: error.into_details(),
            retryable,
            correlation_id,
            internal_cause: None,
        }
    }

    /// Converts an unexpected internal cause to the fixed public internal error.
    pub fn internal(
        correlation_id: CorrelationId,
        cause: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            code: ErrorCode::CoreInternal,
            details: None,
            retryable: false,
            correlation_id,
            internal_cause: Some(Box::new(cause)),
        }
    }

    /// Produces the only representation allowed to cross a public adapter boundary.
    #[must_use]
    pub fn public(&self) -> PublicError {
        PublicError {
            code: self.code,
            message: self.code.safe_message(),
            details: self.details.clone(),
            retryable: self.retryable,
            correlation_id: self.correlation_id,
        }
    }
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code.safe_message())
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.internal_cause
            .as_deref()
            .map(|cause| cause as &(dyn Error + 'static))
    }
}

/// Safe, transport-independent error view produced by the application boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicError {
    code: ErrorCode,
    message: &'static str,
    details: Option<ErrorDetails>,
    retryable: bool,
    correlation_id: CorrelationId,
}

impl PublicError {
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &'static str {
        self.message
    }

    #[must_use]
    pub fn details(&self) -> Option<&ErrorDetails> {
        self.details.as_ref()
    }

    #[must_use]
    pub fn retryable(&self) -> bool {
        self.retryable
    }

    #[must_use]
    pub fn correlation_id(&self) -> CorrelationId {
        self.correlation_id
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    #[test]
    fn domain_failures_keep_typed_public_context() -> Result<(), argos_domain::ErrorDetailError> {
        let correlation_id = CorrelationId::new();
        let domain_error = DomainError::new(
            ErrorCode::SystemdTimeout,
            Some(ErrorDetails::for_scope("user")?),
        );
        let public = ApplicationError::from_domain(domain_error, correlation_id, true).public();

        assert_eq!(public.code(), ErrorCode::SystemdTimeout);
        assert_eq!(public.message(), "The systemd request timed out.");
        assert_eq!(public.details().and_then(ErrorDetails::scope), Some("user"));
        assert!(public.retryable());
        assert_eq!(public.correlation_id(), correlation_id);

        Ok(())
    }

    #[test]
    fn unknown_internal_failures_never_cross_the_public_boundary() {
        let sensitive = "sqlite error at /home/user/private.db: token=secret";
        let error = ApplicationError::internal(CorrelationId::new(), io::Error::other(sensitive));
        let public = error.public();
        let exposed = format!("{public:?} {error}");

        assert_eq!(public.code(), ErrorCode::CoreInternal);
        assert_eq!(public.message(), "Argos could not complete the request.");
        assert!(!public.retryable());
        assert!(!exposed.contains(sensitive));
        assert!(!exposed.contains("private.db"));
        assert!(error.source().is_some());
    }
}
