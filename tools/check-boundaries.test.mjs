import assert from "node:assert/strict";
import test from "node:test";

import {
  forbiddenFrontendImports,
  broadCapabilityViolations,
  invalidCargoDependencies,
  invalidTaskManagerCapability,
  moduleRegistryMismatch,
  taskManagerPrivacyViolations,
} from "./check-boundaries.mjs";

test("rejects Tauri imports outside the transport", () => {
  assert.deepEqual(
    forbiddenFrontendImports([
      { path: "apps/desktop/src/app/bad.ts", source: 'import { invoke } from "@tauri-apps/api/core";' },
      { path: "apps/desktop/src/modules/bad.ts", source: 'import { listen } from "@tauri-apps/api/event";' },
      {
        path: "apps/desktop/src/api/transport/tauri.ts",
        source: 'import { invoke } from "@tauri-apps/api/core";',
      },
    ]),
    ["apps/desktop/src/app/bad.ts", "apps/desktop/src/modules/bad.ts"],
  );
});

test("rejects adapter dependencies in the core", () => {
  assert.deepEqual(
    invalidCargoDependencies([
      { name: "argos-domain", dependencies: [{ name: "rusqlite" }] },
      { name: "argos-storage-sqlite", dependencies: [{ name: "rusqlite" }] },
    ]),
    ["argos-domain -> rusqlite"],
  );
});

test("requires matching backend and frontend module registries", () => {
  assert.deepEqual(
    moduleRegistryMismatch(
      'const COMPILED_MODULE_IDS = ["task-manager", "systemd"]',
      'const FRONTEND_MODULE_IDS = ["task-manager", "launcher"]',
    ),
    ["module registry mismatch: backend=systemd,task-manager frontend=launcher,task-manager"],
  );
  assert.deepEqual(
    moduleRegistryMismatch(
      'const COMPILED_MODULE_IDS = ["task-manager"]',
      'const FRONTEND_MODULE_IDS = ["task-manager"]',
    ),
    [],
  );
});

test("allows only narrow Task Manager reads and no secondary data sinks", () => {
  assert.deepEqual(
    invalidTaskManagerCapability(
      JSON.stringify({
        identifier: "task-manager-read",
        permissions: [
          "allow-task-manager-snapshot",
          "allow-task-manager-process-details",
        ],
      }),
    ),
    [],
  );
  assert.deepEqual(
    invalidTaskManagerCapability(
      JSON.stringify({
        identifier: "task-manager-read",
        permissions: ["allow-task-manager-snapshot", "allow-task-manager-kill"],
      }),
    ),
    ["Task Manager capability must contain only its two narrow reads"],
  );
  assert.deepEqual(
    taskManagerPrivacyViolations([
      { path: "safe.ts", source: "render(snapshot)" },
      { path: "bad.ts", source: "localStorage.setItem('process', name)" },
    ]),
    ["bad.ts"],
  );
  assert.deepEqual(
    broadCapabilityViolations([
      { path: "core.json", source: '"core:default"' },
      { path: "bad.json", source: '"shell:allow-execute"' },
    ]),
    ["bad.json"],
  );
});
