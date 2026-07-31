use argos_domain::{ActionClassification, ActorContext, Hostname, SystemIdentityReader};

use crate::ApplicationError;

/// Current machine identity returned by the application use case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemIdentityResult {
    hostname: Hostname,
}

impl SystemIdentityResult {
    #[must_use]
    pub fn hostname(&self) -> &Hostname {
        &self.hostname
    }
}

/// Read-only use case for the local machine hostname.
#[derive(Clone, Debug)]
pub struct SystemIdentityService<R> {
    reader: R,
}

impl<R> SystemIdentityService<R> {
    #[must_use]
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    #[must_use]
    pub const fn classification() -> ActionClassification {
        ActionClassification::Read
    }
}

impl<R> Default for SystemIdentityService<R>
where
    R: Default,
{
    fn default() -> Self {
        Self::new(R::default())
    }
}

impl<R> SystemIdentityService<R>
where
    R: SystemIdentityReader,
{
    pub fn execute(&self, actor: &ActorContext) -> Result<SystemIdentityResult, ApplicationError> {
        self.reader
            .read_hostname()
            .map(|hostname| SystemIdentityResult { hostname })
            .map_err(|cause| ApplicationError::internal(actor.correlation_id(), cause))
    }
}

#[cfg(test)]
mod tests {
    use argos_domain::{CorrelationId, HostnameError};

    use super::*;

    struct FakeIdentityReader(Result<Hostname, HostnameError>);

    impl SystemIdentityReader for FakeIdentityReader {
        fn read_hostname(&self) -> Result<Hostname, HostnameError> {
            self.0.clone()
        }
    }

    #[test]
    fn reads_hostname_with_trusted_request_context() -> Result<(), Box<dyn std::error::Error>> {
        let actor = ActorContext::local_human(CorrelationId::new());
        let service =
            SystemIdentityService::new(FakeIdentityReader(Hostname::parse("argos-workstation")));
        let result = service.execute(&actor)?;

        assert_eq!(result.hostname().as_str(), "argos-workstation");
        assert_eq!(
            SystemIdentityService::<FakeIdentityReader>::classification(),
            ActionClassification::Read
        );
        Ok(())
    }

    #[test]
    fn maps_platform_failure_without_exposing_a_hostname() {
        let correlation_id = CorrelationId::new();
        let actor = ActorContext::local_human(correlation_id);
        let service = SystemIdentityService::new(FakeIdentityReader(Err(HostnameError)));
        let error = match service.execute(&actor) {
            Ok(_result) => panic!("failure fixture unexpectedly succeeded"),
            Err(error) => error,
        };
        let public = error.public();

        assert_eq!(public.message(), "Argos could not complete the request.");
        assert_eq!(public.correlation_id(), correlation_id);
        assert!(!format!("{public:?}").contains("hostname"));
    }
}
