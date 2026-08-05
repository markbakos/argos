//! Linux path, configuration, filesystem, opener, and process adapters.

mod configuration;
mod task_manager;

use argos_domain::{Hostname, HostnameError, SystemIdentityReader};

pub use configuration::{LinuxConfigStore, PathEnvironment, ResolvedPaths, resolve_paths};
pub use task_manager::LinuxTaskManagerReader;

/// Safe adapter for the current Linux kernel hostname value.
#[derive(Clone, Copy, Debug, Default)]
pub struct LinuxSystemIdentityReader;

impl SystemIdentityReader for LinuxSystemIdentityReader {
    fn read_hostname(&self) -> Result<Hostname, HostnameError> {
        hostname_from_bytes(rustix::system::uname().nodename().to_bytes())
    }
}

fn hostname_from_bytes(value: &[u8]) -> Result<Hostname, HostnameError> {
    let value = std::str::from_utf8(value).map_err(|_cause| HostnameError)?;
    Hostname::parse(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_hostname_bytes_are_validated_without_lossy_conversion() {
        assert_eq!(
            hostname_from_bytes(b"argos-workstation").map(|value| value.to_string()),
            Ok("argos-workstation".to_owned())
        );
        assert!(hostname_from_bytes(b"").is_err());
        assert!(hostname_from_bytes(b"argos\nprivate").is_err());
        assert!(hostname_from_bytes(&[0xff]).is_err());
    }
}
