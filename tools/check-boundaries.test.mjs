import assert from "node:assert/strict";
import test from "node:test";

import { forbiddenFrontendImports, invalidCargoDependencies } from "./check-boundaries.mjs";

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
