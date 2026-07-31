//! Thin Tauri composition boundary for Argos.

use argos_application::{ApplicationError, BoundaryProofService, SystemIdentityService};
use argos_contracts::{AppError, BoundaryProof, EventEnvelope, SystemIdentity};
use argos_domain::{ActorContext, CorrelationId, SystemIdentityReader};
use argos_platform_linux::LinuxSystemIdentityReader;
use tauri::Emitter;

const BOUNDARY_PROOF_MESSAGE: &str = "Argos typed boundary is ready.";
const BOUNDARY_PROOF_EVENT: &str = "core://boundary-proof";

#[derive(Default)]
struct AppState {
    boundary_proof: BoundaryProofService,
    system_identity: SystemIdentityService<LinuxSystemIdentityReader>,
}

fn boundary_proof_contract(state: &AppState, actor: &ActorContext) -> BoundaryProof {
    let result = state.boundary_proof.execute(actor);
    BoundaryProof {
        message: BOUNDARY_PROOF_MESSAGE.to_owned(),
        correlation_id: result.correlation_id().into(),
    }
}

fn public_error_contract(error: &ApplicationError) -> AppError {
    let public = error.public();
    AppError::from_safe_parts(
        public.code(),
        public.details(),
        public.retryable(),
        public.correlation_id(),
    )
}

fn system_identity_contract<R: SystemIdentityReader>(
    service: &SystemIdentityService<R>,
    actor: &ActorContext,
) -> Result<SystemIdentity, ApplicationError> {
    service.execute(actor).map(|result| SystemIdentity {
        hostname: result.hostname().as_str().to_owned(),
    })
}

#[tauri::command]
fn core_boundary_proof<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, AppState>,
) -> Result<BoundaryProof, AppError> {
    let correlation_id = CorrelationId::new();
    let actor = ActorContext::local_human(correlation_id);
    let proof = boundary_proof_contract(&state, &actor);
    let event = EventEnvelope {
        schema_version: 1,
        correlation_id: correlation_id.into(),
        payload: proof.clone(),
    };

    app.emit(BOUNDARY_PROOF_EVENT, event).map_err(|cause| {
        public_error_contract(&ApplicationError::internal(correlation_id, cause))
    })?;

    Ok(proof)
}

#[tauri::command]
fn core_get_system_identity(state: tauri::State<'_, AppState>) -> Result<SystemIdentity, AppError> {
    let correlation_id = CorrelationId::new();
    let actor = ActorContext::local_human(correlation_id);

    system_identity_contract(&state.system_identity, &actor)
        .map_err(|error| public_error_contract(&error))
}

/// Runs the desktop host without placing application behavior in Tauri.
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            core_boundary_proof,
            core_get_system_identity
        ])
        .run(tauri::generate_context!())
}

#[cfg(test)]
mod tests {
    use std::io;

    use argos_contracts::AppErrorCode;
    use argos_domain::{Hostname, HostnameError};
    use tauri::{WebviewWindowBuilder, webview::InvokeRequest};

    use super::*;

    struct FixedIdentityReader;

    impl SystemIdentityReader for FixedIdentityReader {
        fn read_hostname(&self) -> Result<Hostname, HostnameError> {
            Hostname::parse("argos-workstation")
        }
    }

    #[test]
    fn tauri_translation_uses_application_output_and_contract_types() {
        let correlation_id = CorrelationId::new();
        let actor = ActorContext::local_human(correlation_id);
        let proof = boundary_proof_contract(&AppState::default(), &actor);

        assert_eq!(proof.message, BOUNDARY_PROOF_MESSAGE);
        assert_eq!(proof.correlation_id.as_str(), correlation_id.to_string());
    }

    #[test]
    fn tauri_error_translation_does_not_disclose_internal_causes() {
        let sensitive = "private database path and token";
        let error = ApplicationError::internal(CorrelationId::new(), io::Error::other(sensitive));
        let contract = public_error_contract(&error);
        let exposed = format!("{contract:?}");

        assert_eq!(contract.code, AppErrorCode::CoreInternal);
        assert!(!exposed.contains(sensitive));
    }

    #[test]
    fn tauri_translates_application_identity_to_the_generated_contract()
    -> Result<(), Box<dyn std::error::Error>> {
        let actor = ActorContext::local_human(CorrelationId::new());
        let service = SystemIdentityService::new(FixedIdentityReader);
        let identity = system_identity_contract(&service, &actor)?;

        assert_eq!(identity.hostname, "argos-workstation");
        Ok(())
    }

    #[test]
    fn registered_ipc_command_round_trips_the_generated_contract()
    -> Result<(), Box<dyn std::error::Error>> {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .invoke_handler(tauri::generate_handler![
                core_boundary_proof,
                core_get_system_identity
            ])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))?;
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default()).build()?;
        let response = tauri::test::get_ipc_response(
            &webview,
            InvokeRequest {
                cmd: "core_boundary_proof".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "tauri://localhost".parse()?,
                body: tauri::ipc::InvokeBody::default(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_owned(),
            },
        );
        let body = match response {
            Ok(body) => body,
            Err(error) => return Err(format!("IPC command returned an error: {error}").into()),
        };
        let proof = body.deserialize::<BoundaryProof>()?;

        assert_eq!(proof.message, BOUNDARY_PROOF_MESSAGE);
        assert_eq!(proof.correlation_id.as_str().len(), 36);
        Ok(())
    }

    #[test]
    fn registered_identity_ipc_reads_a_bounded_kernel_hostname()
    -> Result<(), Box<dyn std::error::Error>> {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .invoke_handler(tauri::generate_handler![
                core_boundary_proof,
                core_get_system_identity
            ])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))?;
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default()).build()?;
        let response = tauri::test::get_ipc_response(
            &webview,
            InvokeRequest {
                cmd: "core_get_system_identity".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "tauri://localhost".parse()?,
                body: tauri::ipc::InvokeBody::default(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_owned(),
            },
        );
        let body = match response {
            Ok(body) => body,
            Err(error) => return Err(format!("IPC command returned an error: {error}").into()),
        };
        let identity = body.deserialize::<SystemIdentity>()?;

        assert!(!identity.hostname.is_empty());
        assert!(identity.hostname.len() <= Hostname::MAX_BYTES);
        Ok(())
    }
}
