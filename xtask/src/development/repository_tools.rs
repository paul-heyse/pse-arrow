// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Solver pin projections and Git-selected source snapshots for the pinned REUSE CLI.

use anyhow::{Context, Result, ensure};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
    process::{Command, ExitStatus},
};

const MANIFEST: &str = ".github/setup/solver-images.json";
const PREFIX: &str = "ghcr.io/paul-heyse/pse-solvers:";
const OVERRIDE: &str = "PSE_SOLVER_IMAGE";
type Pins = BTreeMap<String, String>;

/// Solver image stage declared by the single authored pin manifest.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum ImageStage {
    /// CI build image.
    Ci,
    /// Local development/runtime image.
    Dev,
}
impl ImageStage {
    fn name(self) -> &'static str {
        match self {
            Self::Ci => "ci",
            Self::Dev => "dev",
        }
    }
}

/// Explicit solver pin reads and workflow/devcontainer projection publication.
#[derive(Clone, Debug, clap::Subcommand)]
pub enum SolverImagesCommand {
    /// Report stale projections without modifying files.
    Check,
    /// Synchronize projections to the already authored pins.
    Sync,
    /// Print the declared pin without applying a runtime override.
    Ref {
        /// Stage selected from the authored manifest.
        #[arg(value_enum)]
        stage: ImageStage,
    },
    /// Print the immutable runtime override, or the selected authored pin.
    Runtime {
        /// Stage used when no override is selected.
        #[arg(value_enum)]
        stage: ImageStage,
    },
    /// Validate and publish both pins with matching recipe-tree tags and digests.
    Update {
        /// Complete immutable CI image reference.
        #[arg(long)]
        ci: String,
        /// Complete immutable development image reference.
        #[arg(long)]
        dev: String,
    },
}

fn validate(pins: &Pins) -> Result<()> {
    ensure!(
        pins.keys().map(String::as_str).collect::<BTreeSet<_>>() == BTreeSet::from(["ci", "dev"]),
        "image manifest must contain ci and dev"
    );
    let mut trees = BTreeSet::new();
    for (stage, reference) in pins {
        let pattern = Regex::new(&format!(
            r"^{}{}-([0-9a-f]{{12}})@sha256:[0-9a-f]{{64}}$",
            regex::escape(PREFIX),
            stage
        ))?;
        let matched = pattern
            .captures(reference)
            .with_context(|| format!("{stage}: expected a recipe tree tag and immutable digest"))?;
        trees.insert(matched[1].to_owned());
    }
    ensure!(
        trees.len() == 1,
        "ci and dev must come from the same recipe tree"
    );
    Ok(())
}

fn runtime(stage: ImageStage, pins: &Pins, supplied: Option<&str>) -> Result<String> {
    let value = supplied.unwrap_or("");
    if value.is_empty() {
        return pins
            .get(stage.name())
            .cloned()
            .context("selected solver stage is absent");
    }
    let immutable =
        Regex::new(r"^(?:sha256:[0-9a-f]{64}|[A-Za-z0-9][A-Za-z0-9._/:-]*@sha256:[0-9a-f]{64})$")?;
    ensure!(
        immutable.is_match(value),
        "{OVERRIDE}={value:?} is not immutable; use a local image ID (docker image inspect --format '{{{{.Id}}}}' <image>) or name@sha256:<digest>"
    );
    Ok(value.to_owned())
}

fn projections(root: &Path, pins: &Pins) -> Result<BTreeMap<PathBuf, String>> {
    let directory = root.join(".github/workflows");
    let mut targets = Vec::new();
    if directory.exists() {
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path.extension().is_some_and(|extension| extension == "yml") {
                targets.push(path);
            }
        }
    }
    targets.sort();
    targets.push(root.join(".devcontainer/devcontainer.json"));
    let mut changed = BTreeMap::new();
    for path in targets {
        let original =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let mut updated = original.clone();
        for (stage, reference) in pins {
            let pattern = Regex::new(&format!(
                r"{}{}-[0-9a-zA-Z.\-]+(?:@sha256:[0-9a-f]{{64}})?",
                regex::escape(PREFIX),
                stage
            ))?;
            updated = pattern
                .replace_all(&updated, regex::NoExpand(reference))
                .into_owned();
        }
        if updated != original {
            changed.insert(path, updated);
        }
    }
    Ok(changed)
}

/// Run one solver pin operation against the selected checkout's single manifest.
/// # Errors
/// Invalid pin declarations, inaccessible projection files or publication failure.
/// Invalid runtime overrides retain their original command status 2.
pub fn solver_images(root: &Path, command: SolverImagesCommand) -> Result<i32> {
    let authored: serde_json::Value = serde_json::from_slice(&fs::read(root.join(MANIFEST))?)
        .context("decoding solver image manifest")?;
    let pins: Pins = if let SolverImagesCommand::Update { ci, dev } = &command {
        BTreeMap::from([("ci".into(), ci.clone()), ("dev".into(), dev.clone())])
    } else {
        serde_json::from_value(authored)
            .context("solver image manifest requires stage/reference pairs")?
    };
    validate(&pins)?;
    match command {
        SolverImagesCommand::Ref { stage } => {
            writeln!(std::io::stdout().lock(), "{}", pins[stage.name()])?;
            Ok(0)
        }
        SolverImagesCommand::Runtime { stage } => {
            let value =
                std::env::var(OVERRIDE).context("runtime solver image override must be UTF-8");
            let supplied = match value {
                Ok(value) => Some(value),
                Err(_) if std::env::var_os(OVERRIDE).is_none() => None,
                Err(error) => return Err(error),
            };
            match runtime(stage, &pins, supplied.as_deref()) {
                Ok(value) => {
                    writeln!(std::io::stdout().lock(), "{value}")?;
                    Ok(0)
                }
                Err(error) => {
                    writeln!(std::io::stderr().lock(), "solver-images: {error}")?;
                    Ok(2)
                }
            }
        }
        command => {
            let changes = projections(root, &pins)?;
            if matches!(command, SolverImagesCommand::Check) {
                for path in changes.keys() {
                    writeln!(
                        std::io::stderr().lock(),
                        "solver pin drift: {}",
                        path.strip_prefix(root)?.display()
                    )?;
                }
                return Ok(i32::from(!changes.is_empty()));
            }
            if matches!(command, SolverImagesCommand::Update { .. }) {
                let mut encoded = serde_json::to_vec_pretty(&pins)?;
                encoded.push(b'\n');
                fs::write(root.join(MANIFEST), encoded)?;
            }
            for (path, content) in changes {
                fs::write(&path, content)?;
                writeln!(
                    std::io::stdout().lock(),
                    "updated {}",
                    path.strip_prefix(root)?.display()
                )?;
            }
            Ok(0)
        }
    }
}

fn source_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let result = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(root)
        .output()
        .context("enumerating Git-eligible source for REUSE")?;
    ensure!(
        result.status.success(),
        "Git source enumeration failed ({}): {}",
        result.status,
        String::from_utf8_lossy(&result.stderr)
    );
    ensure!(
        result.stdout.is_empty() || result.stdout.last() == Some(&0),
        "Git source inventory is not NUL-terminated"
    );
    let mut paths = BTreeSet::new();
    for field in result
        .stdout
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
    {
        let name = std::str::from_utf8(field).context("Git source path is not UTF-8")?;
        let path = PathBuf::from(name);
        ensure!(
            !path.components().any(|part| matches!(
                part,
                Component::RootDir | Component::ParentDir | Component::Prefix(_)
            )),
            "Git source path escapes the checkout"
        );
        paths.insert(path);
    }
    Ok(paths.into_iter().collect())
}

fn copy_sources(root: &Path, snapshot: &Path, paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        let mut source = root.to_owned();
        let mut eligible = true;
        for part in path.components() {
            source.push(part);
            match fs::symlink_metadata(&source) {
                Ok(metadata) if metadata.is_symlink() => {
                    eligible = false;
                    break;
                }
                Ok(_) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                    ) =>
                {
                    eligible = false;
                    break;
                }
                Err(error) => {
                    return Err(error).with_context(|| format!("observing {}", source.display()));
                }
            }
        }
        if !eligible || !source.is_file() {
            continue;
        }
        let destination = snapshot.join(path);
        fs::create_dir_all(
            destination
                .parent()
                .context("snapshot destination has no parent")?,
        )?;
        fs::copy(&source, &destination)
            .with_context(|| format!("copying Git-eligible source {}", path.display()))?;
    }
    Ok(())
}

/// Run the pinned third-party REUSE CLI against a private Git-eligible source snapshot.
/// Tracked files and eligible untracked files are copied; ignored acquisition trees,
/// deleted files and symlinks are not traversed. The temporary snapshot is reclaimed
/// after the original tool exits, and its actual exit/signal is returned.
/// # Errors
/// Source enumeration/copy or actual CLI launch failed.
pub fn reuse_lint(root: &Path) -> Result<ExitStatus> {
    let paths = source_paths(root)?;
    let snapshot = tempfile::Builder::new().prefix("pse-reuse-").tempdir()?;
    copy_sources(root, snapshot.path(), &paths)?;
    Command::new(root.join(".venv/bin/reuse"))
        .arg("--root")
        .arg(snapshot.path())
        .arg("lint")
        .current_dir(root)
        .status()
        .context("running pinned REUSE CLI over the source snapshot")
}

#[cfg(test)]
#[path = "repository_tools_tests.rs"]
mod tests;
