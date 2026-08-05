import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { extname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repositoryRoot = resolve(fileURLToPath(new URL("..", import.meta.url)));
const frontendRoot = join(repositoryRoot, "apps/desktop/src");
const allowedTauriTransport = "apps/desktop/src/api/transport/tauri.ts";
const tauriImport = /["']@tauri-apps\/(?:api(?:\/[^"']*)?|plugin-[^"']*)["']/;

function sourceFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory()
      ? sourceFiles(path)
      : [".ts", ".tsx"].includes(extname(entry.name))
        ? [path]
        : [];
  });
}

export function forbiddenFrontendImports(files) {
  return files.flatMap(({ path, source }) =>
    path !== allowedTauriTransport && tauriImport.test(source) ? [path] : [],
  );
}

export function invalidCargoDependencies(packages) {
  const forbidden = {
    "argos-domain": new Set(["tauri", "rusqlite", "zbus"]),
    "argos-application": new Set(["tauri", "rusqlite", "zbus"]),
    "argos-contracts": new Set(["tauri", "rusqlite", "zbus"]),
    "argos-platform-linux": new Set(["tauri", "rusqlite", "zbus"]),
    "argos-storage-sqlite": new Set(["tauri", "zbus"]),
    "argos-systemd": new Set(["tauri", "rusqlite"]),
  };

  return packages.flatMap((pkg) =>
    pkg.dependencies
      .filter((dependency) => forbidden[pkg.name]?.has(dependency.name))
      .map((dependency) => `${pkg.name} -> ${dependency.name}`),
  );
}

function stringArray(source, name) {
  const escapedName = name.replaceAll(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const body = source.match(new RegExp(`${escapedName}[^=]*=\\s*\\[([\\s\\S]*?)\\]`))?.[1] ?? "";
  return [...body.matchAll(/["']([^"']+)["']/g)].map((match) => match[1]);
}

export function moduleRegistryMismatch(backendSource, frontendSource) {
  const backend = stringArray(backendSource, "COMPILED_MODULE_IDS").sort();
  const frontend = stringArray(frontendSource, "FRONTEND_MODULE_IDS").sort();
  return JSON.stringify(backend) === JSON.stringify(frontend)
    ? []
    : [`module registry mismatch: backend=${backend.join(",")} frontend=${frontend.join(",")}`];
}

export function invalidTaskManagerCapability(source) {
  const capability = JSON.parse(source);
  const permissions = [...(capability.permissions ?? [])].sort();
  const expected = [
    "allow-task-manager-process-details",
    "allow-task-manager-snapshot",
  ];
  return capability.identifier === "task-manager-read" &&
    JSON.stringify(permissions) === JSON.stringify(expected)
    ? []
    : ["Task Manager capability must contain only its two narrow reads"];
}

export function taskManagerPrivacyViolations(files) {
  const forbiddenSink = /localStorage|sessionStorage|console\.|tracing::|log::|rusqlite|config\.toml|audit/i;
  return files.flatMap(({ path, source }) =>
    forbiddenSink.test(source) ? [path] : [],
  );
}

export function broadCapabilityViolations(files) {
  const broadAuthority = /["'](?:shell|fs|process|autostart):|systemd[^"']*write/i;
  return files.flatMap(({ path, source }) =>
    broadAuthority.test(source) ? [path] : [],
  );
}

function main() {
  const frontendViolations = forbiddenFrontendImports(
    sourceFiles(frontendRoot).map((path) => ({
      path: relative(repositoryRoot, path),
      source: readFileSync(path, "utf8"),
    })),
  );
  const metadata = JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: repositoryRoot,
      encoding: "utf8",
    }),
  );
  const cargoViolations = invalidCargoDependencies(metadata.packages);
  const registryViolations = moduleRegistryMismatch(
    readFileSync(join(repositoryRoot, "crates/argos-application/src/modules.rs"), "utf8"),
    readFileSync(join(frontendRoot, "modules/registry.tsx"), "utf8"),
  );
  const capabilityViolations = invalidTaskManagerCapability(
    readFileSync(
      join(repositoryRoot, "apps/desktop/src-tauri/capabilities/task-manager-read.json"),
      "utf8",
    ),
  );
  const broadCapabilityViolationsFound = broadCapabilityViolations(
    readdirSync(join(repositoryRoot, "apps/desktop/src-tauri/capabilities"))
      .filter((name) => name.endsWith(".json"))
      .map((name) => ({
        path: `apps/desktop/src-tauri/capabilities/${name}`,
        source: readFileSync(
          join(repositoryRoot, "apps/desktop/src-tauri/capabilities", name),
          "utf8",
        ),
      })),
  );
  const taskManagerFiles = [
    "crates/argos-domain/src/task_manager.rs",
    "crates/argos-application/src/task_manager.rs",
    "crates/argos-contracts/src/task_manager.rs",
    "crates/argos-platform-linux/src/task_manager.rs",
    "apps/desktop/src/api/taskManager.ts",
    ...sourceFiles(join(frontendRoot, "modules/task-manager")).map((path) =>
      relative(repositoryRoot, path),
    ),
  ].map((path) => ({ path, source: readFileSync(join(repositoryRoot, path), "utf8") }));
  const privacyViolations = taskManagerPrivacyViolations(taskManagerFiles);
  const violations = [
    ...frontendViolations,
    ...cargoViolations,
    ...registryViolations,
    ...capabilityViolations,
    ...broadCapabilityViolationsFound,
    ...privacyViolations,
  ];

  if (violations.length) {
    console.error(`Boundary violations:\n${violations.map((value) => `- ${value}`).join("\n")}`);
    process.exitCode = 1;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
