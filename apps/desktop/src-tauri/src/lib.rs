//! Thin Tauri composition boundary for Argos.

use std::{io, path::Path};

use argos_application::{
    ApplicationError, BoundaryProofService, ConfigurationService, ModuleRegistry,
    SystemIdentityService, TaskManagerService, compiled_module_registry,
};
use argos_contracts::{
    AppError, BootstrapSettings, BoundaryProof, BuildInfo, EffectiveModule, EventEnvelope,
    ListModulesResponse, RuntimeProfile, SetThemeRequest, SystemIdentity,
    TaskManagerProcessDetails, TaskManagerProcessIdentity, TaskManagerSnapshot,
    TaskManagerSnapshotRequest,
};
use argos_domain::{
    ActorContext, BootstrapConfig, CorrelationId, RuntimeProfile as DomainRuntimeProfile,
    SystemIdentityReader, TaskManagerReader,
};
use argos_platform_linux::{
    LinuxConfigStore, LinuxSystemIdentityReader, LinuxTaskManagerReader, PathEnvironment,
    ResolvedPaths, resolve_paths,
};
use tauri::{Emitter, Manager};

const BOUNDARY_PROOF_MESSAGE: &str = "Argos typed boundary is ready.";
const BOUNDARY_PROOF_EVENT: &str = "core://boundary-proof";

fn build_info_contract() -> BuildInfo {
    BuildInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        build: env!("ARGOS_BUILD_KIND").to_owned(),
        profile: match env!("ARGOS_RUNTIME_PROFILE") {
            "production" => RuntimeProfile::Production,
            _ => RuntimeProfile::Development,
        },
    }
}

struct AppState {
    boundary_proof: BoundaryProofService,
    configuration: ConfigurationService<LinuxConfigStore>,
    module_registry: ModuleRegistry,
    production_data_warning: bool,
    system_identity: SystemIdentityService<LinuxSystemIdentityReader>,
    task_manager: TaskManagerService<LinuxTaskManagerReader>,
}

impl AppState {
    fn runtime() -> Result<Self, ApplicationError> {
        let correlation_id = CorrelationId::new();
        let embedded_profile = match env!("ARGOS_RUNTIME_PROFILE") {
            "production" => DomainRuntimeProfile::Production,
            _ => DomainRuntimeProfile::Development,
        };
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(3);
        let paths = resolve_paths(
            embedded_profile,
            &PathEnvironment::from_process(),
            repository_root,
        )
        .map_err(|error| ApplicationError::from_domain(error, correlation_id, false))?;
        Self::from_paths(paths, correlation_id)
    }

    fn from_paths(
        paths: ResolvedPaths,
        correlation_id: CorrelationId,
    ) -> Result<Self, ApplicationError> {
        let production_data_warning = paths.production_data_warning;
        let module_registry = compiled_module_registry()
            .map_err(|error| ApplicationError::from_domain(error, correlation_id, false))?;
        Ok(Self {
            boundary_proof: BoundaryProofService,
            configuration: ConfigurationService::new(LinuxConfigStore::new(paths)),
            module_registry,
            production_data_warning,
            system_identity: SystemIdentityService::new(LinuxSystemIdentityReader),
            task_manager: TaskManagerService::new(LinuxTaskManagerReader::default()),
        })
    }
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

fn config_contract(config: &BootstrapConfig, production_data_warning: bool) -> BootstrapSettings {
    BootstrapSettings {
        theme: config.theme.into(),
        theme_warning: config.theme_warning,
        production_data_warning,
    }
}

fn system_identity_contract<R: SystemIdentityReader>(
    service: &SystemIdentityService<R>,
    actor: &ActorContext,
) -> Result<SystemIdentity, ApplicationError> {
    service.execute(actor).map(|result| SystemIdentity {
        hostname: result.hostname().as_str().to_owned(),
    })
}

fn task_manager_snapshot_contract<R: TaskManagerReader>(
    service: &TaskManagerService<R>,
    actor: &ActorContext,
    request: TaskManagerSnapshotRequest,
) -> Result<TaskManagerSnapshot, ApplicationError> {
    let fresh_baseline = request.fresh_baseline;
    let query = request
        .try_into()
        .map_err(|error| ApplicationError::from_domain(error, actor.correlation_id(), false))?;
    service
        .snapshot(actor, &query, fresh_baseline)
        .map(|snapshot| TaskManagerSnapshot::from(&snapshot))
}

fn task_manager_details_contract<R: TaskManagerReader>(
    service: &TaskManagerService<R>,
    actor: &ActorContext,
    identity: TaskManagerProcessIdentity,
) -> Result<TaskManagerProcessDetails, ApplicationError> {
    service
        .process_details(actor, identity.into())
        .map(|details| TaskManagerProcessDetails::from(&details))
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

#[tauri::command]
fn core_get_build_info() -> BuildInfo {
    build_info_contract()
}

#[tauri::command]
fn core_list_modules(state: tauri::State<'_, AppState>) -> Result<ListModulesResponse, AppError> {
    let correlation_id = CorrelationId::new();
    let result = state.module_registry.effective(&[]).map_err(|error| {
        public_error_contract(&ApplicationError::from_domain(error, correlation_id, false))
    })?;
    Ok(ListModulesResponse {
        modules: result.modules.iter().map(EffectiveModule::from).collect(),
        unknown_preference_ids: result
            .unknown_preference_ids
            .iter()
            .map(Into::into)
            .collect(),
    })
}

#[tauri::command]
fn core_get_settings(state: tauri::State<'_, AppState>) -> Result<BootstrapSettings, AppError> {
    let actor = ActorContext::local_human(CorrelationId::new());
    state
        .configuration
        .read(&actor)
        .map(|config| config_contract(&config, state.production_data_warning))
        .map_err(|error| public_error_contract(&error))
}

#[tauri::command]
fn core_set_theme(
    state: tauri::State<'_, AppState>,
    request: SetThemeRequest,
) -> Result<BootstrapSettings, AppError> {
    let actor = ActorContext::local_human(CorrelationId::new());
    state
        .configuration
        .set_theme(&actor, request.theme.into())
        .map(|config| config_contract(&config, state.production_data_warning))
        .map_err(|error| public_error_contract(&error))
}

#[tauri::command]
fn task_manager_snapshot(
    state: tauri::State<'_, AppState>,
    request: TaskManagerSnapshotRequest,
) -> Result<TaskManagerSnapshot, AppError> {
    let actor = ActorContext::local_human(CorrelationId::new());
    task_manager_snapshot_contract(&state.task_manager, &actor, request)
        .map_err(|error| public_error_contract(&error))
}

#[tauri::command]
fn task_manager_process_details(
    state: tauri::State<'_, AppState>,
    identity: TaskManagerProcessIdentity,
) -> Result<TaskManagerProcessDetails, AppError> {
    let actor = ActorContext::local_human(CorrelationId::new());
    task_manager_details_contract(&state.task_manager, &actor, identity)
        .map_err(|error| public_error_contract(&error))
}

/// Runs the desktop host without placing application behavior in Tauri.
pub fn run() -> tauri::Result<()> {
    let state = AppState::runtime().map_err(|_cause| {
        tauri::Error::Io(io::Error::other(
            "Argos runtime configuration is unavailable.",
        ))
    })?;
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            core_boundary_proof,
            core_get_build_info,
            core_get_settings,
            core_get_system_identity,
            core_list_modules,
            core_set_theme,
            task_manager_process_details,
            task_manager_snapshot,
        ])
        .on_window_event(|window, event| {
            if window.label() == "main"
                && matches!(event, tauri::WindowEvent::CloseRequested { .. })
            {
                window.app_handle().exit(0);
            }
        })
        .run(tauri::generate_context!())
}

#[cfg(test)]
mod tests {
    use std::{io, time::Duration};

    use argos_contracts::{AppErrorCode, TaskManagerSort, TaskManagerSortDirection};
    use argos_domain::{
        Hostname, HostnameError, LoadUsage, ProcessDetails, ProcessIdentity, RawCpuSnapshot,
        RawMemorySnapshot, RawTaskManagerSnapshot, TaskManagerReadError,
    };
    use tauri::{WebviewWindowBuilder, webview::InvokeRequest};

    use super::*;

    struct FixedIdentityReader;

    struct FixedTaskManagerReader;

    impl SystemIdentityReader for FixedIdentityReader {
        fn read_hostname(&self) -> Result<Hostname, HostnameError> {
            Hostname::parse("argos-workstation")
        }
    }

    impl TaskManagerReader for FixedTaskManagerReader {
        fn read_snapshot(&self) -> Result<RawTaskManagerSnapshot, TaskManagerReadError> {
            Ok(RawTaskManagerSnapshot {
                captured_at: Duration::from_secs(1),
                cpu: RawCpuSnapshot::default(),
                memory: RawMemorySnapshot {
                    total_bytes: 1,
                    available_bytes: 1,
                    ..RawMemorySnapshot::default()
                },
                load: LoadUsage::default(),
                cpu_pressure: None,
                memory_pressure: None,
                io_pressure: None,
                block_devices: Vec::new(),
                network_interfaces: Vec::new(),
                processes: Vec::new(),
                observed_process_count: 0,
                partial_reason: None,
            })
        }

        fn read_process_details(
            &self,
            _identity: ProcessIdentity,
        ) -> Result<ProcessDetails, TaskManagerReadError> {
            Err(TaskManagerReadError::ProcessGone)
        }
    }

    fn test_state() -> Result<AppState, ApplicationError> {
        let correlation_id = CorrelationId::new();
        let paths = ResolvedPaths::for_test(Path::new("/tmp/argos-tauri-tests"))
            .map_err(|error| ApplicationError::from_domain(error, correlation_id, false))?;
        AppState::from_paths(paths, correlation_id)
    }

    #[test]
    fn tauri_translation_uses_application_output_and_contract_types()
    -> Result<(), Box<dyn std::error::Error>> {
        let correlation_id = CorrelationId::new();
        let actor = ActorContext::local_human(correlation_id);
        let proof = boundary_proof_contract(&test_state()?, &actor);

        assert_eq!(proof.message, BOUNDARY_PROOF_MESSAGE);
        assert_eq!(proof.correlation_id.as_str(), correlation_id.to_string());
        Ok(())
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
            .manage(test_state()?)
            .invoke_handler(tauri::generate_handler![
                core_boundary_proof,
                core_get_build_info,
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
            .manage(test_state()?)
            .invoke_handler(tauri::generate_handler![
                core_boundary_proof,
                core_get_build_info,
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

    #[test]
    fn build_info_reports_the_embedded_runtime_profile() {
        let info = build_info_contract();
        let expected = match env!("ARGOS_RUNTIME_PROFILE") {
            "production" => RuntimeProfile::Production,
            _ => RuntimeProfile::Development,
        };

        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.profile, expected);
        assert!(!info.build.is_empty());
    }

    #[test]
    fn task_manager_handler_translates_validated_requests_and_contracts()
    -> Result<(), Box<dyn std::error::Error>> {
        let actor = ActorContext::local_human(CorrelationId::new());
        let request = TaskManagerSnapshotRequest {
            fresh_baseline: true,
            search: None,
            sort: TaskManagerSort::Cpu,
            direction: TaskManagerSortDirection::Descending,
            limit: 200,
        };
        let contract = task_manager_snapshot_contract(
            &TaskManagerService::new(FixedTaskManagerReader),
            &actor,
            request,
        )?;

        assert!(contract.is_baseline);
        assert_eq!(contract.observed_process_count, 0);
        assert_eq!(contract.memory.total_bytes, 1);
        Ok(())
    }
}
