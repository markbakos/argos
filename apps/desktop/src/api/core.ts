import type {
  BoundaryProof,
  BootstrapSettings,
  BuildInfo,
  EffectiveModule,
  EventEnvelope,
  HealthReason,
  ListModulesResponse,
  ModuleCapability,
  SetThemeRequest,
  SystemIdentity,
} from "../generated";
import { normalizeAppError } from "./errors";
import type { Transport, Unlisten } from "./transport/tauri";

const BOUNDARY_PROOF_COMMAND = "core_boundary_proof";
const BOUNDARY_PROOF_EVENT = "core://boundary-proof";
const SYSTEM_IDENTITY_COMMAND = "core_get_system_identity";
const BUILD_INFO_COMMAND = "core_get_build_info";
const SETTINGS_COMMAND = "core_get_settings";
const SET_THEME_COMMAND = "core_set_theme";
const MODULES_COMMAND = "core_list_modules";
const HOSTNAME_MAX_BYTES = 64;

function containsControlCharacter(value: string) {
  return Array.from(value).some((character) => {
    const codePoint = character.codePointAt(0);
    return (
      codePoint !== undefined &&
      (codePoint <= 0x1f || (codePoint >= 0x7f && codePoint <= 0x9f))
    );
  });
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function decodeBoundaryProof(value: unknown): BoundaryProof {
  if (
    !isRecord(value) ||
    typeof value["message"] !== "string" ||
    typeof value["correlation_id"] !== "string"
  ) {
    throw new Error("The boundary proof response is invalid.");
  }

  return {
    message: value["message"],
    correlation_id: value["correlation_id"],
  };
}

function decodeSystemIdentity(value: unknown): SystemIdentity {
  if (
    !isRecord(value) ||
    typeof value["hostname"] !== "string" ||
    value["hostname"].length === 0 ||
    new TextEncoder().encode(value["hostname"]).length > HOSTNAME_MAX_BYTES ||
    containsControlCharacter(value["hostname"])
  ) {
    throw new Error("The system identity response is invalid.");
  }

  return { hostname: value["hostname"] };
}

function decodeBuildInfo(value: unknown): BuildInfo {
  if (
    !isRecord(value) ||
    typeof value["version"] !== "string" ||
    typeof value["build"] !== "string" ||
    !["production", "development", "test"].includes(String(value["profile"]))
  ) {
    throw new Error("The build information response is invalid.");
  }

  return {
    version: value["version"],
    build: value["build"],
    profile: value["profile"] as BuildInfo["profile"],
  };
}

const MODULE_CAPABILITIES: readonly ModuleCapability[] = [
  "task_manager_read",
  "systemd_user_read",
  "systemd_system_read",
  "launcher_read",
  "launcher_write",
  "launcher_execute",
];

function decodeHealthReason(value: unknown): HealthReason | undefined {
  if (!isRecord(value) || typeof value["kind"] !== "string") {
    return undefined;
  }
  if (
    ["platform_unavailable", "permission_denied"].includes(value["kind"]) &&
    typeof value["message"] === "string"
  ) {
    return value as HealthReason;
  }
  if (
    value["kind"] === "dependency" &&
    typeof value["module_id"] === "string" &&
    typeof value["message"] === "string"
  ) {
    return value as HealthReason;
  }
  if (
    value["kind"] === "internal" &&
    typeof value["correlation_id"] === "string"
  ) {
    return value as HealthReason;
  }
  return undefined;
}

function decodeModule(value: unknown): EffectiveModule {
  if (!isRecord(value) || !isRecord(value["manifest"])) {
    throw new Error("The module response is invalid.");
  }
  const manifest = value["manifest"];
  const capabilities = manifest["capabilities"];
  const dependencies = manifest["dependencies"];
  if (
    typeof manifest["id"] !== "string" ||
    typeof manifest["display_name"] !== "string" ||
    typeof manifest["description"] !== "string" ||
    typeof manifest["version"] !== "string" ||
    typeof manifest["route"] !== "string" ||
    typeof manifest["default_order"] !== "number" ||
    typeof manifest["default_enabled"] !== "boolean" ||
    !Array.isArray(capabilities) ||
    !capabilities.every(
      (capability) =>
        typeof capability === "string" &&
        MODULE_CAPABILITIES.includes(capability as ModuleCapability),
    ) ||
    !Array.isArray(dependencies) ||
    !dependencies.every((dependency) => typeof dependency === "string") ||
    typeof manifest["linux_required"] !== "boolean" ||
    !["enabled", "disabled"].includes(String(value["enablement"])) ||
    typeof value["order"] !== "number" ||
    !["available", "unavailable", "degraded", "error"].includes(
      String(value["health"]),
    )
  ) {
    throw new Error("The module response is invalid.");
  }
  const healthReason = decodeHealthReason(value["health_reason"]);
  if (value["health_reason"] != null && healthReason === undefined) {
    throw new Error("The module health response is invalid.");
  }
  return {
    manifest: manifest as EffectiveModule["manifest"],
    enablement: value["enablement"] as EffectiveModule["enablement"],
    order: value["order"],
    health: value["health"] as EffectiveModule["health"],
    ...(healthReason ? { health_reason: healthReason } : {}),
  };
}

function decodeModules(value: unknown): ListModulesResponse {
  if (
    !isRecord(value) ||
    !Array.isArray(value["modules"]) ||
    !Array.isArray(value["unknown_preference_ids"]) ||
    !value["unknown_preference_ids"].every(
      (moduleId) => typeof moduleId === "string",
    )
  ) {
    throw new Error("The module list response is invalid.");
  }
  return {
    modules: value["modules"].map(decodeModule),
    unknown_preference_ids: value["unknown_preference_ids"],
  };
}

function decodeSettings(value: unknown): BootstrapSettings {
  if (
    !isRecord(value) ||
    !["system", "light", "dark"].includes(String(value["theme"])) ||
    typeof value["theme_warning"] !== "boolean" ||
    typeof value["production_data_warning"] !== "boolean"
  ) {
    throw new Error("The settings response is invalid.");
  }
  return value as BootstrapSettings;
}

function decodeBoundaryProofEvent(
  value: unknown,
): EventEnvelope<BoundaryProof> {
  if (
    !isRecord(value) ||
    typeof value["schema_version"] !== "number" ||
    typeof value["correlation_id"] !== "string"
  ) {
    throw new Error("The boundary proof event is invalid.");
  }

  return {
    schema_version: value["schema_version"],
    correlation_id: value["correlation_id"],
    payload: decodeBoundaryProof(value["payload"]),
  };
}

function normalizeFailure<T>(promise: Promise<T>): Promise<T> {
  return promise.catch((error: unknown) =>
    Promise.reject(normalizeAppError(error)),
  );
}

export interface CoreApi {
  getBuildInfo(): Promise<BuildInfo>;
  getSettings(): Promise<BootstrapSettings>;
  listModules(): Promise<ListModulesResponse>;
  setTheme(request: SetThemeRequest): Promise<BootstrapSettings>;
  getSystemIdentity(): Promise<SystemIdentity>;
  proveBoundary(): Promise<BoundaryProof>;
  onBoundaryProof(
    handler: (event: EventEnvelope<BoundaryProof>) => void,
  ): Promise<Unlisten>;
}

export function createCoreApi(transport: Transport): CoreApi {
  return {
    getBuildInfo() {
      return normalizeFailure(
        transport.invoke(BUILD_INFO_COMMAND, decodeBuildInfo),
      );
    },
    getSettings() {
      return normalizeFailure(
        transport.invoke(SETTINGS_COMMAND, decodeSettings),
      );
    },
    listModules() {
      return normalizeFailure(transport.invoke(MODULES_COMMAND, decodeModules));
    },
    setTheme(request) {
      return normalizeFailure(
        transport.invoke(SET_THEME_COMMAND, decodeSettings, { request }),
      );
    },
    getSystemIdentity() {
      return normalizeFailure(
        transport.invoke(SYSTEM_IDENTITY_COMMAND, decodeSystemIdentity),
      );
    },
    proveBoundary() {
      return normalizeFailure(
        transport.invoke(BOUNDARY_PROOF_COMMAND, decodeBoundaryProof),
      );
    },
    onBoundaryProof(handler) {
      return normalizeFailure(
        transport.listen<EventEnvelope<BoundaryProof>>(
          BOUNDARY_PROOF_EVENT,
          decodeBoundaryProofEvent,
          handler,
        ),
      );
    },
  };
}
