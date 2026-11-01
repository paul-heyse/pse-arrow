// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact Cargo dependency facts. Metadata features are not compiler-unit admission.
use crate::{
    catalog::MetadataSnapshot,
    correctness::CorrectnessError,
    envelope::{Envelope, SubjectState},
    environment::Environment,
};
use cargo_metadata::{Metadata, Package};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// An exact source-qualified identity; equal names/versions are not interchangeable.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PackageIdentity {
    /// Cargo's opaque package identifier.
    pub id: String,
    /// Package name.
    pub name: String,
    /// Locked resolved version.
    pub version: String,
    /// Cargo's opaque source, or local path identity in the package ID.
    pub source: Option<String>,
    /// Original manifest location for source inspection.
    pub manifest: PathBuf,
}
impl From<&Package> for PackageIdentity {
    fn from(package: &Package) -> Self {
        Self {
            id: package.id.to_string(),
            name: package.name.to_string(),
            version: package.version.to_string(),
            source: package.source.as_ref().map(ToString::to_string),
            manifest: package.manifest_path.as_std_path().to_owned(),
        }
    }
}
/// A resolved direct workspace edge, preserving renamed dependencies and conditional kinds.
#[derive(Debug, Serialize, JsonSchema)]
pub struct WorkspaceUse {
    /// Original workspace source.
    pub package_id: String,
    /// Original package name.
    pub package: String,
    /// Effective library dependency name, including any rename.
    pub dependency_name: String,
    /// Each native edge kind and cfg/platform expression.
    pub kinds: Vec<DependencyKind>,
    /// Joined authored requests, not activated features. Empty means no unambiguous
    /// request/source join was established, rather than no authored declaration.
    pub declarations: Vec<Declaration>,
}
/// One authored incoming dependency declaration, preserving conditional and feature policy.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Declaration {
    /// Authored version requirement, separate from the lock resolution.
    pub requirement: String,
    /// An optional edge need not participate in the selected build.
    pub optional: bool,
    /// Whether this request enables the supplier's defaults.
    pub uses_default_features: bool,
    /// Features explicitly requested by this declaration.
    pub features: Vec<String>,
    /// Native dependency kind.
    pub kind: String,
    /// Unevaluated native platform condition.
    pub target: Option<String>,
}
/// A shortest package inclusion path in the inspected metadata graph.
#[derive(Debug, Serialize, JsonSchema)]
pub struct InclusionPath {
    /// Actual workspace root from which the supplier is reachable.
    pub workspace_member: String,
    /// Source-qualified IDs in dependency order, including both endpoints.
    pub packages: Vec<String>,
}
/// Authored inverse feature links. No feature simulation or activation is inferred.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FeatureUse {
    /// Source-qualified owner of the authored feature declaration.
    pub package: String,
    /// Feature whose declared expansion contains this exact member spelling.
    pub feature: String,
    /// Original expansion member (`dep:foo`, `foo?/bar`, etc.).
    pub member: String,
}
/// Cargo's actual edge context, not an evaluated cross-platform availability assertion.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DependencyKind {
    /// Cargo's kind spelling, including unknown future kinds.
    pub kind: String,
    /// Native target expression when present.
    pub target: Option<String>,
}
/// Facts from exactly one source-qualified metadata package.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PackageFacts {
    /// Locked package identity.
    pub package: PackageIdentity,
    /// Actual Cargo targets, distinct from discovered tests and analyzer commands.
    pub targets: Vec<PackageTarget>,
    /// Enabled features in this metadata graph, or unknown if --no-deps omitted the graph.
    pub metadata_features: Option<Vec<String>>,
    /// Authored feature declarations, not a promise that every feature is enabled.
    pub declared_features: BTreeMap<String, Vec<String>>,
    /// Direct resolved edges from workspace members; unknown without a resolve graph.
    pub workspace_uses: Option<Vec<WorkspaceUse>>,
    /// The single workspace metadata hold, if one is authored for this package name.
    pub pin_reason: Option<String>,
    /// Exact filename-matched library overviews. Combined overviews may require explicit lookup.
    pub overview_candidates: Vec<PathBuf>,
}
/// Authored target eligibility from the selected native metadata.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PackageTarget {
    /// Native target name.
    pub name: String,
    /// Native target roles, retaining future kinds.
    pub kinds: Vec<String>,
    /// Original source identity.
    pub source: PathBuf,
    /// Authored target edition.
    pub edition: String,
    /// Required features; metadata membership does not imply their activation.
    pub required_features: Vec<String>,
    /// Participation in Cargo's test selection.
    pub test: bool,
    /// Benchmark kind is retained above; metadata omits authored `bench` eligibility.
    pub bench: Option<bool>,
    /// Participation in doctest selection, not a transformation or execution verdict.
    pub doctest: bool,
}
/// Name lookup preserves ambiguity instead of selecting the first locked source/version.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "selection", rename_all = "snake_case")]
pub enum Selection {
    /// One exact package was selected.
    Exact {
        /// Facts for the selected source-qualified package.
        facts: Box<PackageFacts>,
    },
    /// Select a source-qualified ID or an unambiguous name@version.
    Ambiguous {
        /// All matching locked identities; none was silently preferred.
        candidates: Vec<PackageIdentity>,
    },
    /// This metadata snapshot contains no matching package.
    NoMatch,
}
/// Read-only dependency answer with explicit metadata context and limitations.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    /// Original query spelling.
    pub query: String,
    /// Exact bytes of the inspected Cargo metadata response.
    pub metadata_digest: String,
    /// Explicit retained snapshot or live locked offline metadata collection.
    pub source: String,
    /// The metadata workspace; all-platform graph facts are not compiler-unit proofs.
    pub workspace: PathBuf,
    /// Exact result or preserved ambiguity.
    pub selected: Selection,
    /// Requested package reach; unknown when Cargo omitted the graph.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inclusion_paths: Option<Vec<InclusionPath>>,
    /// Explicit feature member inverse lookup, without evaluating a feature expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_uses: Option<Vec<FeatureUse>>,
}

/// Query the exact retained metadata document, never build, fetch or prepare a dependency.
pub fn inspect(snapshot: &MetadataSnapshot, query: &str, source: &str) -> Envelope<Report> {
    inspect_details(snapshot, query, source, false, None)
}
/// Expand only selected reach or exact authored feature relationships, without compilation.
pub fn inspect_details(
    snapshot: &MetadataSnapshot,
    query: &str,
    source: &str,
    why: bool,
    feature_member: Option<&str>,
) -> Envelope<Report> {
    let metadata = &snapshot.metadata;
    let exact = metadata
        .packages
        .iter()
        .find(|package| package.id.repr == query);
    let candidates: Vec<_> = if let Some(exact) = exact {
        vec![exact]
    } else {
        let (name, version) = query
            .rsplit_once('@')
            .map_or((query, None), |(name, version)| (name, Some(version)));
        metadata
            .packages
            .iter()
            .filter(|package| {
                package.name == name
                    && version.is_none_or(|version| package.version.to_string() == version)
            })
            .collect()
    };
    let selected = match candidates.as_slice() {
        [] => Selection::NoMatch,
        [package] => Selection::Exact {
            facts: Box::new(facts(metadata, package)),
        },
        _ => Selection::Ambiguous {
            candidates: candidates.into_iter().map(PackageIdentity::from).collect(),
        },
    };
    let inclusion_paths = if why {
        match &selected {
            Selection::Exact { facts } => inclusion_paths(metadata, &facts.package.id),
            _ => None,
        }
    } else {
        None
    };
    let feature_uses = feature_member.map(|member| {
        metadata
            .packages
            .iter()
            .flat_map(|package| {
                package.features.iter().flat_map(move |(feature, members)| {
                    members
                        .iter()
                        .filter(move |value| value.as_str() == member)
                        .map(move |value| FeatureUse {
                            package: package.id.to_string(),
                            feature: feature.to_string(),
                            member: value.to_string(),
                        })
                })
            })
            .collect()
    });
    let mut response = Envelope::response(
        "inspect dependency",
        match &selected {
            Selection::Exact { .. } => "exact locked dependency facts",
            Selection::Ambiguous { .. } => {
                "ambiguous dependency; select a source-qualified package ID"
            }
            Selection::NoMatch => "no matching dependency in this metadata context",
        },
        Report {
            query: query.into(),
            metadata_digest: snapshot.digest.clone(),
            source: source.into(),
            workspace: metadata.workspace_root.as_std_path().to_owned(),
            selected,
            inclusion_paths,
            feature_uses,
        },
    );
    response.subject_state = match &response.payload.selected {
        Selection::NoMatch => SubjectState::NoMatch,
        Selection::Ambiguous { .. } => SubjectState::Unknown,
        Selection::Exact { .. } => SubjectState::Available,
    };
    response.observation.scope = "captured_cargo_metadata_dependency_context".into();
    response.observation.limitations.extend([
        "metadata features aggregate package contexts; selected compiler host/target units require correctness admission".into(),
        "metadata graph is not ordinary-build API availability, compilation success or source freshness proof".into(),
        "overview candidates match exact filenames; combined library documents may not appear".into(),
        "incoming authored requests join only when alias, version, native kind/target and explicit source/path agree; unmatched request details remain unknown".into(),
    ]);
    if why {
        response.observation.limitations.push("one shortest path per reachable workspace member in Cargo's package graph; target conditions, optional edges and feature unions do not prove selected-unit or test reach".into());
    }
    if feature_member.is_some() {
        response.observation.limitations.push("feature member lookup reports exact authored links across the inspected metadata; it does not resolve or activate an expression and does not simulate feature configurations".into());
    }
    response
}
fn inclusion_paths(metadata: &Metadata, selected: &str) -> Option<Vec<InclusionPath>> {
    use petgraph::{algo::astar, graphmap::DiGraphMap};
    let resolve = metadata.resolve.as_ref()?;
    let mut graph = DiGraphMap::<&str, ()>::new();
    for node in &resolve.nodes {
        graph.add_node(&node.id.repr);
        for edge in &node.deps {
            graph.add_edge(node.id.repr.as_str(), edge.pkg.repr.as_str(), ());
        }
    }
    Some(
        metadata
            .workspace_members
            .iter()
            .filter_map(|root| {
                astar(
                    &graph,
                    root.repr.as_str(),
                    |node| node == selected,
                    |_| 1usize,
                    |_| 0,
                )
                .map(|(_, path)| InclusionPath {
                    workspace_member: root.to_string(),
                    packages: path.into_iter().map(str::to_owned).collect(),
                })
            })
            .collect(),
    )
}
fn facts(metadata: &Metadata, package: &Package) -> PackageFacts {
    let node = metadata
        .resolve
        .as_ref()
        .and_then(|resolve| resolve.nodes.iter().find(|node| node.id == package.id));
    let workspace_uses = metadata.resolve.as_ref().map(|resolve| {
        let mut uses = Vec::new();
        for node in &resolve.nodes {
            if !metadata.workspace_members.contains(&node.id) {
                continue;
            }
            let Some(owner) = metadata
                .packages
                .iter()
                .find(|package| package.id == node.id)
            else {
                continue;
            };
            for edge in node.deps.iter().filter(|edge| edge.pkg == package.id) {
                uses.push(WorkspaceUse {
                    package_id: node.id.to_string(),
                    package: owner.name.to_string(),
                    dependency_name: edge.name.clone(),
                    kinds: edge
                        .dep_kinds
                        .iter()
                        .map(|kind| DependencyKind {
                            kind: kind.kind.to_string(),
                            target: kind.target.as_ref().map(ToString::to_string),
                        })
                        .collect(),
                    declarations: owner
                        .dependencies
                        .iter()
                        .filter(|dependency| {
                            dependency
                                .rename
                                .as_deref()
                                .unwrap_or(&dependency.name)
                                .replace('-', "_")
                                == edge.name
                                && dependency.name == package.name.as_str()
                                && dependency.req.matches(&package.version)
                                && edge.dep_kinds.iter().any(|kind| {
                                    kind.kind == dependency.kind && kind.target == dependency.target
                                })
                                && match (&dependency.path, &dependency.source, &package.source) {
                                    (Some(path), None, None) => {
                                        package.manifest_path.parent() == Some(path.as_path())
                                    }
                                    (None, Some(requested), Some(resolved)) => {
                                        requested == resolved
                                    }
                                    _ => false,
                                }
                        })
                        .map(|dependency| Declaration {
                            requirement: dependency.req.to_string(),
                            optional: dependency.optional,
                            uses_default_features: dependency.uses_default_features,
                            features: dependency.features.clone(),
                            kind: dependency.kind.to_string(),
                            target: dependency.target.as_ref().map(ToString::to_string),
                        })
                        .collect(),
                });
            }
        }
        uses.sort_by(|left, right| {
            (&left.package_id, &left.dependency_name)
                .cmp(&(&right.package_id, &right.dependency_name))
        });
        uses
    });
    let pin_reason = metadata
        .workspace_metadata
        .pointer("/pse/pins")
        .and_then(|pins| pins.get(package.name.as_str()))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    let overview = metadata
        .workspace_root
        .join("docs/libraries")
        .join(format!("{}.md", package.name));
    PackageFacts {
        package: package.into(),
        targets: package
            .targets
            .iter()
            .map(|target| PackageTarget {
                name: target.name.clone(),
                kinds: target.kind.iter().map(ToString::to_string).collect(),
                source: target.src_path.as_std_path().to_owned(),
                edition: target.edition.to_string(),
                required_features: target.required_features.clone(),
                test: target.test,
                bench: None,
                doctest: target.doctest,
            })
            .collect(),
        metadata_features: node.map(|node| node.features.iter().map(ToString::to_string).collect()),
        declared_features: package
            .features
            .iter()
            .map(|(name, members)| {
                (
                    name.to_string(),
                    members.iter().map(ToString::to_string).collect(),
                )
            })
            .collect(),
        workspace_uses,
        pin_reason,
        overview_candidates: if overview.is_file() {
            vec![overview.as_std_path().to_owned()]
        } else {
            Vec::new()
        },
    }
}

/// Collect locked, offline metadata from an explicit checkout and complete captured environment.
/// Cargo may refuse an uncached graph; this route neither downloads nor compiles it.
pub fn capture(
    root: &Path,
    environment: &Environment,
) -> Result<MetadataSnapshot, CorrectnessError> {
    capture_original(root, environment).map(|(snapshot, _)| snapshot)
}
/// Preserve the original response alongside its typed view without reconstruction.
pub fn capture_original(
    root: &Path,
    environment: &Environment,
) -> Result<(MetadataSnapshot, Vec<u8>), CorrectnessError> {
    let mut builder = cargo_metadata::MetadataCommand::new();
    builder
        .cargo_path(crate::correctness::real_cargo_in(environment)?)
        .current_dir(root)
        .other_options(vec!["--locked".into(), "--offline".into()]);
    let mut command = builder.cargo_command();
    let output = command.env_clear().envs(environment).output()?;
    if !output.status.success() {
        return Err(CorrectnessError::Invalid(format!(
            "locked offline dependency metadata exited {}; {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let digest = Sha256::digest(&output.stdout)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok((
        MetadataSnapshot {
            metadata: serde_json::from_slice(&output.stdout)?,
            digest,
        },
        output.stdout,
    ))
}

#[cfg(test)]
mod tests;
