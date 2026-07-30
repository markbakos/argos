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
  const violations = [...frontendViolations, ...cargoViolations];

  if (violations.length) {
    console.error(`Boundary violations:\n${violations.map((value) => `- ${value}`).join("\n")}`);
    process.exitCode = 1;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}

