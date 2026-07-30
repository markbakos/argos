use argos_domain::{ActorContext, CorrelationId};

/// Side-effect-free proof that a trusted actor reached the application layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundaryProofResult {
    correlation_id: CorrelationId,
}

impl BoundaryProofResult {
    #[must_use]
    pub fn correlation_id(&self) -> CorrelationId {
        self.correlation_id
    }
}

/// Minimal application service used to prove adapter composition without Tauri coupling.
#[derive(Clone, Copy, Debug, Default)]
pub struct BoundaryProofService;

impl BoundaryProofService {
    #[must_use]
    pub fn execute(&self, actor: &ActorContext) -> BoundaryProofResult {
        BoundaryProofResult {
            correlation_id: actor.correlation_id(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_uses_the_trusted_actor_correlation() {
        let correlation_id = CorrelationId::new();
        let actor = ActorContext::local_human(correlation_id);

        assert_eq!(
            BoundaryProofService.execute(&actor).correlation_id(),
            correlation_id
        );
    }
}
