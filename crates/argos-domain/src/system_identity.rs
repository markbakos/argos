use std::fmt;

/// Validated current Linux kernel hostname.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Hostname(String);

impl Hostname {
    pub const MAX_BYTES: usize = 64;

    /// Validates a hostname received from the platform boundary.
    pub fn parse(value: impl Into<String>) -> Result<Self, HostnameError> {
        let value = value.into();

        if value.is_empty() || value.len() > Self::MAX_BYTES || value.chars().any(char::is_control)
        {
            return Err(HostnameError);
        }

        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Hostname {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Safe error for an empty, oversized, non-UTF-8, or control-bearing hostname.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostnameError;

impl fmt::Display for HostnameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("system hostname is invalid")
    }
}

impl std::error::Error for HostnameError {}

/// Port for the one narrow local machine identity read.
pub trait SystemIdentityReader: Send + Sync {
    fn read_hostname(&self) -> Result<Hostname, HostnameError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostname_accepts_bounded_visible_utf8() {
        let hostname = Hostname::parse("argos-workstation");

        assert_eq!(
            hostname.as_ref().map(Hostname::as_str),
            Ok("argos-workstation")
        );
        assert!(Hostname::parse("živa-machine").is_ok());
    }

    #[test]
    fn hostname_rejects_empty_oversized_and_control_values() {
        assert!(Hostname::parse("").is_err());
        assert!(Hostname::parse("a".repeat(Hostname::MAX_BYTES + 1)).is_err());
        assert!(Hostname::parse("argos\nprivate").is_err());
    }

    #[test]
    fn hostname_limit_is_measured_in_utf8_bytes() {
        assert!(Hostname::parse("ž".repeat(Hostname::MAX_BYTES / 2)).is_ok());
        assert!(Hostname::parse("ž".repeat(Hostname::MAX_BYTES / 2 + 1)).is_err());
    }
}
