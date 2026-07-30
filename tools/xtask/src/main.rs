use std::{
    collections::BTreeMap,
    error::Error,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::{SystemTime, UNIX_EPOCH},
};

use argos_contracts::{
    ActionClassification, ActorId, ActorKind, ActorRef, AppError, AppErrorCode, AppErrorDetails,
    Availability, BoundaryProof, BuildInfo, CoreEvent, CorrelationId, Cursor, EventEnvelope,
    HealthReason, HealthState, ModuleEnablement, ModuleId, Page, PageRequest, RuntimeProfile,
    SettingsCategory,
};
use ts_rs::{Config, TS};

type XtaskResult<T> = Result<T, Box<dyn Error>>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> XtaskResult<()> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [group, command] if group == "contracts" && command == "generate" => {
            generate_contracts(&repository_root()?)
        }
        [group, command] if group == "contracts" && command == "check" => {
            check_contracts(&repository_root()?)
        }
        _ => Err("usage: cargo run -p xtask -- contracts <generate|check>".into()),
    }
}

fn repository_root() -> XtaskResult<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}

fn generated_directory(repository_root: &Path) -> PathBuf {
    repository_root.join("apps/desktop/src/generated")
}

fn unique_sibling(parent: &Path, purpose: &str) -> XtaskResult<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(parent.join(format!(
        ".argos-generated-{purpose}-{}-{nanos}",
        std::process::id()
    )))
}

fn generate_contracts(repository_root: &Path) -> XtaskResult<()> {
    let output = generated_directory(repository_root);
    let parent = output
        .parent()
        .ok_or("generated directory must have a parent")?;
    let temporary = unique_sibling(parent, "new")?;
    let backup = unique_sibling(parent, "old")?;

    let result = (|| -> XtaskResult<()> {
        generate_tree(&temporary)?;
        format_tree(repository_root, &temporary)?;

        if output.exists() {
            fs::rename(&output, &backup)?;
        }

        if let Err(error) = fs::rename(&temporary, &output) {
            if backup.exists() {
                fs::rename(&backup, &output)?;
            }
            return Err(error.into());
        }

        if backup.exists() {
            fs::remove_dir_all(&backup)?;
        }

        Ok(())
    })();

    if temporary.exists() {
        fs::remove_dir_all(&temporary)?;
    }

    result
}

fn check_contracts(repository_root: &Path) -> XtaskResult<()> {
    let expected = generated_directory(repository_root);
    let parent = expected
        .parent()
        .ok_or("generated directory must have a parent")?;
    let temporary = unique_sibling(parent, "check")?;

    let result = (|| -> XtaskResult<()> {
        generate_tree(&temporary)?;
        format_tree(repository_root, &temporary)?;

        let mismatches = tree_mismatches(&expected, &temporary)?;
        if mismatches.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "generated contracts are stale: {}. Run `pnpm contracts:generate`.",
                mismatches.join(", ")
            )
            .into())
        }
    })();

    if temporary.exists() {
        fs::remove_dir_all(&temporary)?;
    }

    result
}

fn generate_tree(output: &Path) -> XtaskResult<()> {
    fs::create_dir(output)?;
    let config = Config::new().with_out_dir(output);

    ActionClassification::export(&config)?;
    ActorId::export(&config)?;
    ActorKind::export(&config)?;
    ActorRef::export(&config)?;
    AppError::export(&config)?;
    AppErrorCode::export(&config)?;
    AppErrorDetails::export(&config)?;
    Availability::export(&config)?;
    BoundaryProof::export(&config)?;
    BuildInfo::export(&config)?;
    CoreEvent::export(&config)?;
    CorrelationId::export(&config)?;
    Cursor::export(&config)?;
    EventEnvelope::<String>::export(&config)?;
    HealthReason::export(&config)?;
    HealthState::export(&config)?;
    ModuleEnablement::export(&config)?;
    ModuleId::export(&config)?;
    Page::<String>::export(&config)?;
    PageRequest::export(&config)?;
    RuntimeProfile::export(&config)?;
    SettingsCategory::export(&config)?;

    fs::write(output.join("index.ts"), generated_index())?;
    Ok(())
}

fn generated_index() -> String {
    let names = [
        "ActionClassification",
        "ActorId",
        "ActorKind",
        "ActorRef",
        "AppError",
        "AppErrorCode",
        "AppErrorDetails",
        "Availability",
        "BoundaryProof",
        "BuildInfo",
        "CoreEvent",
        "CorrelationId",
        "Cursor",
        "EventEnvelope",
        "HealthReason",
        "HealthState",
        "ModuleEnablement",
        "ModuleId",
        "Page",
        "PageRequest",
        "RuntimeProfile",
        "SettingsCategory",
    ];
    let exports = names
        .into_iter()
        .map(|name| format!("export type {{ {name} }} from \"./{name}\";"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "// This file was generated by Argos from Rust contracts. Do not edit manually.\n{exports}\n"
    )
}

fn format_tree(repository_root: &Path, output: &Path) -> XtaskResult<()> {
    let status = Command::new("pnpm")
        .args(["--filter", "@argos/desktop", "exec", "prettier", "--write"])
        .arg(output)
        .current_dir(repository_root)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("Prettier failed while formatting generated contracts".into())
    }
}

fn tree_mismatches(expected: &Path, actual: &Path) -> XtaskResult<Vec<String>> {
    let expected_files = read_tree(expected)?;
    let actual_files = read_tree(actual)?;
    let mut paths = expected_files
        .keys()
        .chain(actual_files.keys())
        .cloned()
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();

    Ok(paths
        .into_iter()
        .filter(|path| expected_files.get(path) != actual_files.get(path))
        .map(|path| path.to_string_lossy().into_owned())
        .collect())
}

fn read_tree(root: &Path) -> XtaskResult<BTreeMap<PathBuf, Vec<u8>>> {
    if !root.exists() {
        return Ok(BTreeMap::new());
    }

    fn visit(
        root: &Path,
        directory: &Path,
        files: &mut BTreeMap<PathBuf, Vec<u8>>,
    ) -> XtaskResult<()> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files)?;
            } else if path.extension() == Some(OsStr::new("ts")) {
                files.insert(path.strip_prefix(root)?.to_owned(), fs::read(path)?);
            }
        }
        Ok(())
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files)?;
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TemporaryDirectory(PathBuf);

    impl TemporaryDirectory {
        fn new(purpose: &str) -> XtaskResult<Self> {
            let root = std::env::temp_dir();
            let path = unique_sibling(&root, purpose)?;
            fs::create_dir(&path)?;
            Ok(Self(path))
        }
    }

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _ignored = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn two_generation_runs_are_byte_identical() -> XtaskResult<()> {
        let root = TemporaryDirectory::new("determinism")?;
        let first = root.0.join("first");
        let second = root.0.join("second");

        generate_tree(&first)?;
        generate_tree(&second)?;

        assert!(tree_mismatches(&first, &second)?.is_empty());
        Ok(())
    }

    #[test]
    fn stale_generated_file_is_rejected() -> XtaskResult<()> {
        let root = TemporaryDirectory::new("stale")?;
        let expected = root.0.join("expected");
        let stale = root.0.join("stale");

        generate_tree(&expected)?;
        generate_tree(&stale)?;
        fs::write(stale.join("AppError.ts"), "stale fixture")?;

        assert_eq!(tree_mismatches(&expected, &stale)?, ["AppError.ts"]);
        Ok(())
    }
}
