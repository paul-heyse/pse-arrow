// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Conservative source reach. Preview never builds, lists tests or prepares capabilities.
use crate::{
    ownership::{self, OwnershipError, OwnershipMap},
    repository::{self, GitName, HeadState, RepositoryError, RepositoryObservation},
    selection,
};
use cargo_metadata::Metadata;
use gix::bstr::ByteSlice as _;
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    os::unix::ffi::OsStrExt as _,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Original input failure, separate from a conservative widening decision.
#[derive(Debug, thiserror::Error)]
pub enum AffectedError {
    /// Native Git source failure.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    /// Native base/object lookup failure.
    #[error("affected Git comparison: {0}")]
    Git(#[from] gix::Error),
    /// Original map/path source failure.
    #[error(transparent)]
    Ownership(#[from] OwnershipError),
    /// A source cannot establish the requested selection.
    #[error("affected source reach: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    AffectedError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::HostOwnership) },
    forward(this) { match this { Self::Repository(error) => Some(error), Self::Ownership(error) => Some(error), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn invalid(message: impl Into<String>) -> AffectedError {
    AffectedError::Invalid(message.into())
}

/// Native base-to-HEAD comparison plus independently observed index/worktree layers.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ChangeSet {
    /// Caller-selected revision spelling, never a command-line option.
    pub base: String,
    /// Exact commit resolved by Gitoxide.
    pub base_commit: String,
    /// Original live owner observation, including HEAD before/after and byte paths.
    pub live: RepositoryObservation,
    /// Paths changed between the selected base tree and the observed HEAD tree.
    pub committed_paths: Vec<GitName>,
}
impl ChangeSet {
    /// Stable byte-preserving union, including both endpoints of any native rewrite.
    pub fn paths(&self) -> Vec<GitName> {
        let mut paths = BTreeMap::new();
        for path in
            self.committed_paths
                .iter()
                .chain(self.live.changes.iter().flat_map(|change| {
                    std::iter::once(&change.path).chain(change.previous_path.iter())
                }))
        {
            paths.insert(path.as_bytes().to_vec(), path.clone());
        }
        paths.into_values().collect()
    }
}

/// Observe through Gitoxide only; no porcelain parser, index refresh or Git mutation.
pub fn observe(
    root: &Path,
    base: &str,
    interrupt: Arc<AtomicBool>,
) -> Result<ChangeSet, AffectedError> {
    if base.is_empty() {
        return Err(invalid("comparison base is empty"));
    }
    let live = repository::observe(root, interrupt.clone())?;
    if live.head_before != live.head_after {
        return Err(invalid(
            "HEAD changed during source observation; observe again",
        ));
    }
    let mut repo = gix::open(root)?;
    repo.make_paths_absolute()?;
    let base_object = repo
        .rev_parse_single(format!("{base}^{{commit}}").as_str())?
        .object()?
        .peel_to_commit()?;
    let base_commit = base_object.id.to_string();
    let base_tree = base_object.tree()?;
    let head_commit = match &live.head_after {
        HeadState::Attached { commit, .. } | HeadState::Detached { commit } => commit,
        HeadState::Unborn { .. } => {
            return Err(invalid(
                "selected comparison requires an existing HEAD commit",
            ));
        }
    };
    let head_tree = repo
        .rev_parse_single(head_commit.as_str())?
        .object()?
        .peel_to_commit()?
        .tree()?;
    let mut committed_paths = BTreeMap::new();
    base_tree
        .changes()?
        .options(|options| {
            options.track_path().track_rewrites(None);
        })
        .for_each_to_obtain_tree(&head_tree, |change| {
            use gix::object::tree::diff::{Action, Change};
            let (location, source) = match change {
                Change::Addition { location, .. }
                | Change::Deletion { location, .. }
                | Change::Modification { location, .. } => (location, None),
                Change::Rewrite {
                    location,
                    source_location,
                    ..
                } => (location, Some(source_location)),
            };
            for location in std::iter::once(location).chain(source) {
                committed_paths.insert(
                    location.as_bytes().to_vec(),
                    GitName::new(location.as_bytes()),
                );
            }
            Ok(if interrupt.load(Ordering::Relaxed) {
                Action::Break(())
            } else {
                Action::Continue(())
            })
        })?;
    if interrupt.load(Ordering::Relaxed) {
        return Err(invalid(
            "base comparison was cancelled before complete observation",
        ));
    }
    let after = repo.head_id()?.to_string();
    if &after != head_commit {
        return Err(invalid(
            "HEAD changed during base comparison; observe again",
        ));
    }
    Ok(ChangeSet {
        base: base.into(),
        base_commit,
        live,
        committed_paths: committed_paths.into_values().collect(),
    })
}

/// Exact member plus declared integration targets; no runnable test identity is inferred.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SelectedPackage {
    /// Opaque Cargo source identity.
    pub id: String,
    /// Human-readable captured package name.
    pub name: String,
    /// Actual Cargo integration-target declarations, independent of feature activation.
    pub integration_targets: Vec<selection::TargetIntent>,
}
/// Captured graph availability, not compiler admission or source freshness.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Graph {
    /// Existing declarations were interpreted.
    Available,
    /// No changed paths require a graph.
    NotNeeded,
    /// A graph was missing or invalid; no executable scope can be established.
    Unavailable {
        /// Original unavailable premise.
        reason: String,
    },
}
/// Why exact source reach is unknown and selection conservatively widens to all members.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Widening {
    /// Workspace-level Cargo or command configuration can change every compiler context.
    WorkspaceContext {
        /// Exact selected context path.
        path: GitName,
    },
    /// A selected changed source lacks a declared containing package/input consumer.
    UnknownSource {
        /// Exact changed path.
        path: GitName,
    },
    /// Existing aggregate ownership data cannot establish generator input reach.
    OwnershipUnavailable {
        /// Original failure text; original error retained separately.
        reason: String,
    },
    /// A generator output does not overlap any captured package source scope.
    UnknownGeneratedConsumer {
        /// Declared generator identity.
        producer: String,
        /// Original exact output root.
        output: String,
    },
    /// Declaration identity cannot prove whether a local member is a consumer.
    UnknownDependency {
        /// Exact declaring member.
        package_id: String,
        /// Authored dependency name, not a resolved source ID.
        dependency: String,
    },
}
/// An actual declared input match; a map is not a freshness or regeneration receipt.
#[derive(Debug, Serialize, JsonSchema)]
pub struct GeneratorReach {
    /// Changed source input.
    pub path: GitName,
    /// Declared producer.
    pub producer: String,
    /// Generator's original narrow regeneration argv.
    pub regenerate: Vec<String>,
}
/// Source-only selection. Capability/fixture readiness remains with actual admission owners.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Preview {
    /// Caller-selected Git comparison spelling.
    pub base: String,
    /// Exact resolved base commit.
    pub base_commit: String,
    /// HEAD identity observed around status traversal.
    pub head: HeadState,
    /// Byte-preserving changed-path union.
    pub changes: Vec<GitName>,
    /// Exact direct source-owner/input-consumer seeds.
    pub changed_packages: Vec<String>,
    /// Declared reverse dependency closure, or all members after explicit widening.
    pub selected_packages: Vec<SelectedPackage>,
    /// Independent graph availability.
    pub graph: Graph,
    /// Explicit conservative widening; no native package exclusion exists.
    pub widening: Vec<Widening>,
    /// Existing generated ownership input matches.
    pub generators: Vec<GeneratorReach>,
    /// Scope and qualification limits.
    pub coverage_limits: Vec<String>,
    /// Original independent read/domain errors for outer diagnostic projection.
    #[serde(skip)]
    #[schemars(skip)]
    pub failures: Vec<AffectedError>,
}
impl Preview {
    /// Feed the same admitted selection owner for test/check. Empty changes select no work.
    /// Explicit integration targets derive their own required features during normalization.
    pub fn scope(
        &self,
        features: selection::Features,
    ) -> Result<Option<selection::Scope>, AffectedError> {
        if let Graph::Unavailable { reason } = &self.graph {
            return Err(invalid(reason.clone()));
        }
        if self.selected_packages.is_empty() {
            return Ok(None);
        }
        let mut packages = Vec::new();
        for package in &self.selected_packages {
            packages.push(selection::PackageIntent {
                package: package.id.clone(),
                target: None,
                features: features.clone(),
            });
            packages.extend(package.integration_targets.iter().map(|target| {
                selection::PackageIntent {
                    package: package.id.clone(),
                    target: Some(target.clone()),
                    features: features.clone(),
                }
            }));
        }
        Ok(Some(selection::Scope::Packages(packages)))
    }
}

struct Member {
    selected: SelectedPackage,
    directory: PathBuf,
}
struct Workspace {
    members: BTreeMap<String, Member>,
    reverse: BTreeMap<String, BTreeSet<String>>,
    uncertain: Vec<Widening>,
}
impl Workspace {
    fn read(root: &Path, metadata: &Metadata) -> Result<Self, AffectedError> {
        if !metadata.workspace_root.is_absolute() {
            return Err(invalid("captured metadata workspace root is not absolute"));
        }
        if ownership::resolve(metadata.workspace_root.as_std_path())? != root {
            return Err(invalid("captured metadata belongs to another checkout"));
        }
        let mut members = BTreeMap::new();
        let mut directories = BTreeSet::new();
        for id in &metadata.workspace_members {
            let package = metadata
                .packages
                .iter()
                .find(|package| &package.id == id)
                .ok_or_else(|| invalid("workspace member package is absent from metadata"))?;
            if !package.manifest_path.is_absolute() {
                return Err(invalid("captured member manifest is not absolute"));
            }
            let manifest = ownership::resolve(package.manifest_path.as_std_path())?;
            let directory = manifest
                .parent()
                .ok_or_else(|| invalid("member manifest parent absent"))?
                .to_owned();
            if !directories.insert(directory.clone()) {
                return Err(invalid("workspace member manifest scopes are ambiguous"));
            }
            let integration_targets = package
                .targets
                .iter()
                .filter(|target| target.kind.iter().any(|kind| kind.to_string() == "test"))
                .map(|target| selection::TargetIntent {
                    name: target.name.clone(),
                    kind: Some("test".into()),
                })
                .collect();
            members.insert(
                id.to_string(),
                Member {
                    selected: SelectedPackage {
                        id: id.to_string(),
                        name: package.name.to_string(),
                        integration_targets,
                    },
                    directory,
                },
            );
        }
        if members.is_empty() {
            return Err(invalid("captured metadata declares no workspace members"));
        }
        let mut reverse: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut uncertain = Vec::new();
        for package in metadata.workspace_packages() {
            for dependency in &package.dependencies {
                if let Some(path) = &dependency.path {
                    let path = ownership::resolve(path.as_std_path())?;
                    for (id, member) in &members {
                        if member.directory == path {
                            reverse
                                .entry(id.clone())
                                .or_default()
                                .insert(package.id.to_string());
                        }
                    }
                } else {
                    let candidates: Vec<_> = metadata
                        .workspace_packages()
                        .into_iter()
                        .filter(|candidate| {
                            candidate.name == dependency.name
                                && dependency.req.matches(&candidate.version)
                        })
                        .collect();
                    if candidates.is_empty() {
                        continue;
                    }
                    let resolved = metadata.resolve.as_ref().and_then(|resolve| {
                        resolve.nodes.iter().find(|node| node.id == package.id)
                    });
                    if let Some(node) = resolved {
                        for candidate in &candidates {
                            if node.deps.iter().any(|edge| edge.pkg == candidate.id) {
                                reverse
                                    .entry(candidate.id.to_string())
                                    .or_default()
                                    .insert(package.id.to_string());
                            }
                        }
                    }
                    // Optional/target declarations omitted by this resolve context cannot prove absence.
                    if dependency.optional
                        || resolved.is_none()
                        || candidates.iter().any(|candidate| {
                            !resolved.is_some_and(|node| {
                                node.deps.iter().any(|edge| edge.pkg == candidate.id)
                            })
                        })
                    {
                        uncertain.push(Widening::UnknownDependency {
                            package_id: package.id.to_string(),
                            dependency: dependency.name.clone(),
                        });
                    }
                }
            }
        }
        Ok(Self {
            members,
            reverse,
            uncertain,
        })
    }
    fn owner(&self, path: &Path) -> Option<&str> {
        self.members
            .iter()
            .filter(|(_, member)| path.starts_with(&member.directory))
            .max_by_key(|(_, member)| member.directory.components().count())
            .map(|(id, _)| id.as_str())
    }
    fn closure(&self, seeds: &BTreeSet<String>) -> BTreeSet<String> {
        let mut found = seeds.clone();
        let mut pending: Vec<_> = seeds.iter().cloned().collect();
        while let Some(id) = pending.pop() {
            if let Some(reverse) = self.reverse.get(&id) {
                for id in reverse {
                    if found.insert(id.clone()) {
                        pending.push(id.clone());
                    }
                }
            }
        }
        found
    }
}

/// Interpret already observed changes and optional captured metadata. Reads only the existing map.
pub fn preview(
    root: &Path,
    changes: &ChangeSet,
    metadata: Option<&Metadata>,
) -> Result<Preview, AffectedError> {
    let paths = changes.paths();
    for path in &paths {
        let path = Path::new(std::ffi::OsStr::from_bytes(path.as_bytes()));
        if !ownership::protocol::normal_relative(path) || path.as_os_str().as_bytes().contains(&0) {
            return Err(invalid(
                "changed paths must remain normalized inside the selected checkout",
            ));
        }
    }
    let mut result = Preview { base: changes.base.clone(), base_commit: changes.base_commit.clone(), head: changes.live.head_after.clone(), changes: paths,
        changed_packages: Vec::new(), selected_packages: Vec::new(), graph: Graph::NotNeeded, widening: Vec::new(), generators: Vec::new(), failures: Vec::new(),
        coverage_limits: vec!["Conservative source/declaration reach, including optional/build/dev/target declarations; not exhaustive behavior coverage or test execution.".into(),
            "Base-to-observed-HEAD and staged/worktree/untracked paths are unioned conservatively; these observations are not an atomic filesystem snapshot.".into(),
            "Integration required features, native capabilities, fixture readiness and force-validation are established only by the existing selection/admission owners.".into(),
            "Existing aggregate generated declarations have unknown dirty-source freshness; no regeneration or contributor completeness is inferred.".into()] };
    if result.changes.is_empty() {
        return Ok(result);
    }
    let root = ownership::resolve(root)?;
    if ownership::resolve(Path::new(std::ffi::OsStr::from_bytes(
        changes.live.worktree.as_bytes(),
    )))? != root
    {
        return Err(invalid(
            "changed source observation belongs to another checkout",
        ));
    }
    let workspace = metadata
        .ok_or_else(|| {
            invalid("captured Cargo metadata was not supplied; preview never runs Cargo")
        })
        .and_then(|metadata| Workspace::read(&root, metadata));
    let map = match OwnershipMap::read(&root) {
        Ok(map) => Some(map),
        Err(error) => {
            result.widening.push(Widening::OwnershipUnavailable {
                reason: error.to_string(),
            });
            result.failures.push(error.into());
            None
        }
    };
    let mut workspace = match workspace {
        Ok(workspace) => workspace,
        Err(error) => {
            result.graph = Graph::Unavailable {
                reason: error.to_string(),
            };
            result.failures.push(error);
            return Ok(result);
        }
    };
    result.graph = Graph::Available;
    result.widening.append(&mut workspace.uncertain);
    let mut seeds = BTreeSet::new();
    for path in &result.changes {
        let relative = Path::new(std::ffi::OsStr::from_bytes(path.as_bytes()));
        // Workspace-level authored context is shared even when the root also declares a package.
        // Directory scopes deliberately stay broader than any operation's captured-input list.
        if [
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain",
            "rust-toolchain.toml",
        ]
        .iter()
        .any(|context| relative == Path::new(context))
            || relative.starts_with(".cargo")
            || relative.starts_with(".config")
        {
            result
                .widening
                .push(Widening::WorkspaceContext { path: path.clone() });
        }
        let absolute = root.join(relative);
        let direct = workspace.owner(&absolute);
        if let Some(id) = direct {
            seeds.insert(id.to_owned());
        }
        let owners = map
            .as_ref()
            .map(|map| map.input_owners(relative))
            .transpose()?
            .unwrap_or_default();
        if direct.is_none() && owners.is_empty() {
            result
                .widening
                .push(Widening::UnknownSource { path: path.clone() });
        }
        for owner in owners {
            result.generators.push(GeneratorReach {
                path: path.clone(),
                producer: owner.producer.clone(),
                regenerate: owner.regenerate.clone(),
            });
            for output in &owner.outputs {
                let output_path = root.join(&output.path);
                let consumers: Vec<_> = workspace
                    .members
                    .iter()
                    .filter(|(_, member)| {
                        output_path.starts_with(&member.directory)
                            || (output.kind == ownership::protocol::ScopeKind::Tree
                                && member.directory.starts_with(&output_path))
                    })
                    .map(|(id, _)| id.clone())
                    .collect();
                if consumers.is_empty() {
                    result.widening.push(Widening::UnknownGeneratedConsumer {
                        producer: owner.producer.clone(),
                        output: output.path.clone(),
                    });
                }
                seeds.extend(consumers);
            }
        }
    }
    let selected = if result.widening.is_empty() {
        workspace.closure(&seeds)
    } else {
        workspace.members.keys().cloned().collect()
    };
    result.changed_packages = seeds.into_iter().collect();
    result.selected_packages = selected
        .into_iter()
        .filter_map(|id| workspace.members.remove(&id).map(|member| member.selected))
        .collect();
    Ok(result)
}

#[cfg(test)]
mod tests;
