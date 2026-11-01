// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original metadata and admitted units are independently verified, then explicitly associated.
use super::*;
use crate::{
    compiler_investigation::{self as compiler, artifact},
    correctness::CompilerUnit,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Serialize, JsonSchema)]
pub(in crate::cli) struct MetadataPremises {
    pub path: query::NativePath,
    pub sha256: String,
    pub original: ResultReference,
    pub workspace_root: PathBuf,
    pub association: String,
    pub limitations: Vec<String>,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(in crate::cli) struct ContextInspection {
    pub original: Envelope<compiler::UnitContextObservation>,
    pub reference: ResultReference,
    pub metadata: MetadataPremises,
}
fn metadata(
    path: &Path,
    expected: Option<&str>,
) -> Result<(cargo_metadata::Metadata, MetadataPremises), HostError> {
    let path = path.canonicalize()?;
    let bytes = std::fs::read(&path)?;
    use sha2::{Digest as _, Sha256};
    let sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    if expected.is_some_and(|expected| expected != sha256) {
        return Err(query::Error::Stale("original selected metadata bytes changed".into()).into());
    }
    let metadata: cargo_metadata::Metadata =
        serde_json::from_slice(&bytes).map_err(query::Error::from)?;
    let premises=MetadataPremises {path:path.clone().into(),sha256:sha256.clone(),original:ResultReference::new(Locator::Source{path:path.clone().into(),source:query::SourceKind::Working,range:None,guard:Some(query::SourceGuard{root:path.parent().map(|p|p.to_path_buf().into()),sha256:Some(sha256),length:Some(bytes.len() as u64),..Default::default()})}),workspace_root:metadata.workspace_root.as_std_path().to_owned(),association:"user_selected_independent_metadata; common acquisition not established".into(),limitations:vec!["Raw JSON bytes remain inspectable through the exact original reference; typed defaults and unsupported artifact-dependency fields do not replace those bytes.".into(),"This compiler context does not bind a Cargo metadata digest. A verified metadata file and authenticated original units do not establish that they came from the same acquisition.".into()]};
    Ok((metadata, premises))
}
pub(in crate::cli) fn context_inspection(
    root: &Path,
    invocation: &str,
    path: &Path,
) -> Result<ContextInspection, HostError> {
    let state = crate::paths::CheckoutState::resolve(root, &std::env::vars_os().collect())?;
    let original = compiler::unit_context(root, &state, invocation)?;
    if !original.payload.admission_available {
        return Err(query::Error::Unavailable("the original operation never recorded admitted compiler contexts; original report inspection remains available".into()).into());
    }
    let (_, metadata) = metadata(path, None)?;
    let reference = ResultReference::new(Locator::Context {
        metadata: metadata.path.clone(),
        metadata_sha256: metadata.sha256.clone(),
        compiler_invocation: original.payload.original_invocation.as_str().into(),
        compiler_context_sha256: original.payload.context_sha256.clone(),
    });
    reference.validate()?;
    Ok(ContextInspection {
        original,
        reference,
        metadata,
    })
}
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub(super) struct Package {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    manifest: PathBuf,
    /// Unknown without a resolve graph; metadata union is not selected unit activation.
    features: Option<BTreeSet<String>>,
    dependencies: Option<Vec<Edge>>,
    targets: Vec<Target>,
    selected_units: Vec<usize>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, JsonSchema)]
struct Edge {
    name: String,
    package: String,
    kinds: Vec<(String, Option<String>)>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
struct Target {
    name: String,
    kinds: Vec<String>,
    source: PathBuf,
    required_features: Vec<String>,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(in crate::cli) struct Side {
    reference: ResultReference,
    metadata: MetadataPremises,
    compiler_invocation: String,
    compiler_context_sha256: String,
    original_outcome: crate::envelope::CommandOutcome,
    admission_groups: Vec<AdmissionGroup>,
    #[serde(skip)]
    #[schemars(skip)]
    units: Vec<CompilerUnit>,
    #[serde(skip)]
    #[schemars(skip)]
    packages: Vec<Package>,
    /// Indexes always address the original retained context, including beyond this view.
    unit_records: Vec<IndexedUnit>,
    package_records: Vec<IndexedPackage>,
    omitted_units: usize,
    omitted_packages: usize,
    native_world_requested: bool,
    native_profiles: Observation<Vec<IndexedNativeProfile>>,
    omitted_native_profiles: usize,
    native_joins: Vec<crate::cli::dependency::WorldJoin>,
    source_premise_issues: Vec<String>,
    /// The original full context remains independently inspectable under its invocation owner.
    original_context: OriginalContext,
}
#[derive(Debug, Serialize, JsonSchema)]
struct IndexedUnit {
    index: usize,
    group: usize,
    group_unit: usize,
    unit: CompilerUnit,
}
#[derive(Debug, Serialize, JsonSchema)]
struct AdmissionGroup {
    group: usize,
    arguments: Vec<std::ffi::OsString>,
    packages: Vec<String>,
    unit_start: usize,
    unit_end: usize,
}
#[derive(Debug, Serialize, JsonSchema)]
struct IndexedPackage {
    index: usize,
    package: Package,
}
#[derive(Debug, Serialize, JsonSchema)]
struct IndexedNativeProfile {
    original_index: usize,
    profile: crate::semantic::NativePackageContext,
}
#[derive(Debug, Serialize, JsonSchema)]
struct OriginalContext {
    invocation: String,
    context_sha256: String,
    origin: compiler::UnitContextOrigin,
    full_context: Option<artifact::ArtifactFile>,
    total_units: usize,
    root_units: usize,
}
#[derive(Debug, Serialize, JsonSchema)]
struct Alignment {
    before: Vec<usize>,
    after: Vec<usize>,
    correspondence: String,
    basis: String,
    differences: Vec<String>,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(in crate::cli) struct Comparison {
    before: Observation<Side>,
    after: Observation<Side>,
    packages: Vec<Alignment>,
    units: Vec<Alignment>,
    omitted_alignments: usize,
    limitations: Vec<String>,
}
impl Side {
    pub(super) fn detail(&self, budget: Option<usize>) -> crate::envelope::DetailView {
        let omitted_records = self.omitted_units + self.omitted_packages;
        crate::envelope::DetailView {
            budget,
            omitted_records,
            truncated: omitted_records > 0,
            ..Default::default()
        }
    }
    pub(super) fn partial(&self) -> bool {
        self.omitted_packages > 0
            || self.omitted_units > 0
            || !self.source_premise_issues.is_empty()
            || self.native_world_requested
                && (self.native_profiles.value().is_none()
                    || self.native_joins.iter().any(|join| {
                        !matches!(
                            join.state,
                            crate::cli::dependency::JoinState::UniqueCandidate
                        )
                    }))
    }
}
impl Comparison {
    pub(super) fn partial(&self) -> bool {
        self.omitted_alignments > 0
            || [&self.before, &self.after]
                .into_iter()
                .any(|side| side.value().is_none_or(Side::partial))
            || self
                .packages
                .iter()
                .chain(&self.units)
                .any(|pair| pair.correspondence != "qualified_candidate")
    }
}
fn profile_projection(
    units: &[CompilerUnit],
    profiles: Observation<Vec<crate::semantic::NativePackageContext>>,
) -> (
    Observation<Vec<IndexedNativeProfile>>,
    Vec<crate::cli::dependency::WorldJoin>,
    usize,
) {
    let joins = crate::cli::dependency::profile_joins(units, profiles.value().map(Vec::as_slice));
    let required = joins
        .iter()
        .flat_map(|join| join.profiles.iter().copied())
        .collect::<BTreeSet<_>>();
    match profiles {
        Observation::Available { value } => {
            let omitted = value.len().saturating_sub(required.len());
            let records = value
                .into_iter()
                .enumerate()
                .filter_map(|(original_index, profile)| {
                    required
                        .contains(&original_index)
                        .then_some(IndexedNativeProfile {
                            original_index,
                            profile,
                        })
                })
                .collect();
            (Observation::Available { value: records }, joins, omitted)
        }
        Observation::Unavailable { diagnostic } => {
            (Observation::Unavailable { diagnostic }, joins, 0)
        }
    }
}
fn packages(metadata: &cargo_metadata::Metadata, units: &[CompilerUnit]) -> Vec<Package> {
    let nodes = metadata.resolve.as_ref().map(|resolve| {
        resolve
            .nodes
            .iter()
            .map(|node| (&node.id, node))
            .collect::<BTreeMap<_, _>>()
    });
    let mut selected = BTreeMap::<&str, Vec<usize>>::new();
    for (index, unit) in units.iter().enumerate() {
        selected.entry(&unit.package_id).or_default().push(index);
    }
    metadata
        .packages
        .iter()
        .map(|package| {
            let node = nodes
                .as_ref()
                .and_then(|nodes| nodes.get(&package.id))
                .copied();
            let mut dependencies = node.map(|node| {
                node.deps
                    .iter()
                    .map(|dep| {
                        let mut kinds = dep
                            .dep_kinds
                            .iter()
                            .map(|kind| {
                                (
                                    kind.kind.to_string(),
                                    kind.target.as_ref().map(ToString::to_string),
                                )
                            })
                            .collect::<Vec<_>>();
                        kinds.sort();
                        Edge {
                            name: dep.name.clone(),
                            package: dep.pkg.to_string(),
                            kinds,
                        }
                    })
                    .collect::<Vec<_>>()
            });
            if let Some(edges) = &mut dependencies {
                edges.sort();
            }
            Package {
                id: package.id.to_string(),
                name: package.name.to_string(),
                version: package.version.to_string(),
                source: package.source.as_ref().map(ToString::to_string),
                manifest: package.manifest_path.as_std_path().to_owned(),
                features: node.map(|node| node.features.iter().map(ToString::to_string).collect()),
                dependencies,
                targets: package
                    .targets
                    .iter()
                    .map(|t| Target {
                        name: t.name.clone(),
                        kinds: t.kind.iter().map(ToString::to_string).collect(),
                        source: t.src_path.as_std_path().to_owned(),
                        required_features: t.required_features.clone(),
                    })
                    .collect(),
                selected_units: selected
                    .get(package.id.to_string().as_str())
                    .cloned()
                    .unwrap_or_default(),
            }
        })
        .collect()
}
fn source_issues(metadata: &cargo_metadata::Metadata, units: &[CompilerUnit]) -> Vec<String> {
    let mut issues = Vec::new();
    let packages = metadata
        .packages
        .iter()
        .map(|package| (package.id.to_string(), package))
        .collect::<BTreeMap<_, _>>();
    for unit in units {
        let Some(package) = packages.get(&unit.package_id) else {
            issues.push(format!(
                "original unit package {} is absent from selected metadata",
                unit.package_id
            ));
            continue;
        };
        if !package.targets.iter().any(|target| {
            target.name == unit.target_name
                && target.src_path.as_std_path() == Path::new(&unit.source_path)
                && target
                    .kind
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    == unit.target_kinds
        }) {
            issues.push(format!(
                "original unit {} has incompatible metadata target/source premises",
                unit.target_name
            ));
        }
    }
    issues
}
async fn read(
    root: &Path,
    reference: &ResultReference,
    world: Option<&ResultReference>,
    limit: usize,
    token: &tokio_util::sync::CancellationToken,
) -> Result<Side, HostError> {
    if token.is_cancelled() {
        return Err(query::Error::Cancelled.into());
    }
    let Locator::Context {
        metadata: path,
        metadata_sha256,
        compiler_invocation,
        compiler_context_sha256,
    } = &reference.locator
    else {
        return Err(invalid(
            "context comparison requires original context locators",
        ));
    };
    let state = crate::paths::CheckoutState::resolve(root, &std::env::vars_os().collect())?;
    let original = compiler::unit_context(root, &state, compiler_invocation)?;
    if original.payload.context_sha256 != *compiler_context_sha256 {
        return Err(query::Error::Stale(
            "original compiler context no longer matches the selected address".into(),
        )
        .into());
    }
    if !original.payload.admission_available {
        return Err(query::Error::Unavailable("the selected original workflow did not record admitted compiler contexts; no current planning was substituted".into()).into());
    }
    let (metadata, premises) = metadata(&path.to_path_buf(), Some(metadata_sha256))?;
    let mut units = Vec::new();
    let mut positions = Vec::new();
    let mut admission_groups = Vec::new();
    for (group, admitted) in original.payload.admission.iter().enumerate() {
        let unit_start = units.len();
        for (group_unit, unit) in admitted.units.iter().enumerate() {
            positions.push((group, group_unit));
            units.push(unit.clone());
        }
        admission_groups.push(AdmissionGroup {
            group,
            arguments: admitted.arguments.clone(),
            packages: admitted.packages.clone(),
            unit_start,
            unit_end: units.len(),
        });
    }
    let source_premise_issues = source_issues(&metadata, &units);
    let packages = packages(&metadata, &units);
    let omitted_packages = packages.len().saturating_sub(limit);
    let omitted_units = units.len().saturating_sub(limit);
    let package_records = packages
        .iter()
        .take(limit)
        .cloned()
        .enumerate()
        .map(|(index, package)| IndexedPackage { index, package })
        .collect();
    let unit_records = units
        .iter()
        .take(limit)
        .cloned()
        .enumerate()
        .map(|(index, unit)| IndexedUnit {
            index,
            group: positions[index].0,
            group_unit: positions[index].1,
            unit,
        })
        .collect();
    let visible_units = &units[..units.len().min(limit)];
    let native_profiles = Observation::from_result(if let Some(world) = world {
        let ids = visible_units.iter().map(|u| u.package_id.clone()).collect();
        semantic::package_contexts(root, world, &ids, token).await
    } else {
        Err(query::Error::Unavailable("no selected original native world; retained metadata/unit comparison remains independently available".into()).into())
    });
    let (native_profiles, native_joins, omitted_native_profiles) =
        profile_projection(visible_units, native_profiles);
    let original_context = OriginalContext {
        invocation: compiler_invocation.clone(),
        context_sha256: compiler_context_sha256.clone(),
        origin: original.payload.origin,
        full_context: original.payload.full_context,
        total_units: units.len(),
        root_units: units.iter().filter(|unit| unit.root).count(),
    };
    Ok(Side {
        reference: reference.clone(),
        metadata: premises,
        compiler_invocation: compiler_invocation.clone(),
        compiler_context_sha256: compiler_context_sha256.clone(),
        original_outcome: original.payload.original_outcome,
        admission_groups,
        units,
        packages,
        unit_records,
        package_records,
        omitted_units,
        omitted_packages,
        native_world_requested: world.is_some(),
        native_profiles,
        omitted_native_profiles,
        native_joins,
        source_premise_issues,
        original_context,
    })
}
pub(super) async fn inspect_reference(
    root: &Path,
    reference: &ResultReference,
    limit: usize,
) -> Result<Side, HostError> {
    read(
        root,
        reference,
        None,
        limit,
        &tokio_util::sync::CancellationToken::new(),
    )
    .await
}
fn package_pairs(a: &[Package], b: &[Package]) -> Vec<Alignment> {
    let mut result = Vec::new();
    let mut used = BTreeSet::new();
    for (i, old) in a.iter().enumerate() {
        let mut after = b
            .iter()
            .enumerate()
            .filter_map(|(j, new)| (old.id == new.id).then_some(j))
            .collect::<Vec<_>>();
        let exact = !after.is_empty();
        if !exact {
            after = b
                .iter()
                .enumerate()
                .filter_map(|(j, new)| (old.name == new.name).then_some(j))
                .collect();
        }
        used.extend(after.iter().copied());
        let mut differences = Vec::new();
        if let [j] = after.as_slice() {
            let new = &b[*j];
            if old.id != new.id {
                differences.push("source_qualified_package_identity".into());
            }
            if old.features != new.features {
                differences.push("metadata_features".into());
            }
            if old.dependencies != new.dependencies {
                differences.push("dependency_alias_package_kind_or_target".into());
            }
            if old.targets != new.targets {
                differences.push("authored_targets_or_source_membership".into());
            }
            if old.selected_units.is_empty() != new.selected_units.is_empty() {
                differences.push("selected_unit_inclusion".into());
            }
        }
        let correspondence = if after.is_empty() {
            "unmatched"
        } else if after.len() > 1 {
            "ambiguous"
        } else if exact {
            "qualified_candidate"
        } else {
            "candidate_package_upgrade_or_moved_root"
        };
        result.push(Alignment {
            before: vec![i],
            after,
            correspondence: correspondence.into(),
            basis: if exact {
                "exact original Cargo package ID"
            } else {
                "same package name nominates upgrade/source/root correspondence only"
            }
            .into(),
            differences,
        });
    }
    for j in 0..b.len() {
        if !used.contains(&j) {
            result.push(Alignment {
                before: vec![],
                after: vec![j],
                correspondence: "unmatched".into(),
                basis: "no selected before package".into(),
                differences: vec!["inclusion".into()],
            });
        }
    }
    result
}
fn unit_key(
    unit: &CompilerUnit,
    package: Option<&Package>,
) -> (String, String, Vec<String>, PathBuf, Option<String>, String) {
    let source = Path::new(&unit.source_path);
    let source = package
        .and_then(|p| p.manifest.parent())
        .and_then(|root| source.strip_prefix(root).ok())
        .unwrap_or(source)
        .to_owned();
    (
        unit.package_name.clone(),
        unit.target_name.clone(),
        unit.target_kinds.clone(),
        source,
        unit.platform.clone(),
        unit.mode.clone(),
    )
}
fn unit_pairs(
    a_units: &[CompilerUnit],
    a_packages: &[Package],
    b_units: &[CompilerUnit],
    b_packages: &[Package],
) -> Vec<Alignment> {
    let mut groups = BTreeMap::<_, (Vec<usize>, Vec<usize>)>::new();
    let a_packages = a_packages
        .iter()
        .map(|package| (package.id.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let b_packages = b_packages
        .iter()
        .map(|package| (package.id.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    for (i, unit) in a_units.iter().enumerate() {
        groups
            .entry(unit_key(
                unit,
                a_packages.get(unit.package_id.as_str()).copied(),
            ))
            .or_default()
            .0
            .push(i);
    }
    for (i, unit) in b_units.iter().enumerate() {
        groups
            .entry(unit_key(
                unit,
                b_packages.get(unit.package_id.as_str()).copied(),
            ))
            .or_default()
            .1
            .push(i);
    }
    groups.into_values().map(|(before, after)| {
        let mut differences = Vec::new();
        let correspondence = if before.is_empty() || after.is_empty() { "unmatched" }
            else if before.len() != 1 || after.len() != 1 { "ambiguous" }
            else if a_units[before[0]].package_id != b_units[after[0]].package_id { "candidate_package_upgrade_or_moved_root" }
            else { "qualified_candidate" };
        if let ([i], [j]) = (before.as_slice(), after.as_slice()) {
            let old = &a_units[*i];
            let new = &b_units[*j];
            if old.features != new.features { differences.push("unit_features".into()); }
            if old.root != new.root { differences.push("root_selection".into()); }
            if old.package_id != new.package_id { differences.push("source_qualified_package_identity".into()); }
            if old.source_path != new.source_path { differences.push("source_path".into()); }
        }
        Alignment { before, after, correspondence: correspondence.into(), basis: "target name/kinds, package source correspondence, relative source, host/target platform and compiler mode; features remain differences".into(), differences }
    }).collect()
}
pub(super) async fn inspect(
    request: &Request,
    before_root: &Path,
    after_root: &Path,
    token: &tokio_util::sync::CancellationToken,
) -> Result<Comparison, HostError> {
    let before = Observation::from_result(
        read(
            before_root,
            &request.before,
            request.before_world.as_ref(),
            request.max_records,
            token,
        )
        .await,
    );
    let after = Observation::from_result(
        read(
            after_root,
            &request.after,
            request.after_world.as_ref(),
            request.max_records,
            token,
        )
        .await,
    );
    if token.is_cancelled() {
        return Err(query::Error::Cancelled.into());
    }
    let (mut packages, mut units) = match (before.value(), after.value()) {
        (Some(a), Some(b)) => (
            package_pairs(&a.packages, &b.packages),
            unit_pairs(&a.units, &a.packages, &b.units, &b.packages),
        ),
        _ => (Vec::new(), Vec::new()),
    };
    let omitted_alignments = packages.len().saturating_sub(request.max_records)
        + units.len().saturating_sub(request.max_records);
    packages.truncate(request.max_records);
    units.truncate(request.max_records);
    Ok(Comparison{before,after,packages,units,omitted_alignments,limitations:vec!["Original metadata package membership, admitted compiler selection and native profiles are separate observations; none establishes executed builds, tests or exact compiler equivalence.".into(),"Each metadata snapshot is read once. Original JSON remains addressed for field presence and unsupported experimental fields. Retained Cargo events, cached build scripts, modeled feature simulations and unselected units were not examined.".into(),"Selected metadata and compiler contexts are independently supplied and verified; matching package/target dimensions establish association candidates, not a common acquisition.".into()]})
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn sample() -> cargo_metadata::Metadata {
        serde_json::from_value(json!({"packages":[{"name":"dep","version":"1.0.0","id":"path+file:///fixture/dep#1.0.0","source":null,"dependencies":[],"targets":[{"name":"dep","kind":["lib"],"crate_types":["lib"],"src_path":"/fixture/dep/src/lib.rs","edition":"2024","doctest":true,"test":true,"doc":true}],"features":{},"manifest_path":"/fixture/dep/Cargo.toml"},{"name":"only-metadata","version":"1.0.0","id":"path+file:///fixture/only#1.0.0","source":null,"dependencies":[],"targets":[],"features":{},"manifest_path":"/fixture/only/Cargo.toml"}],"workspace_members":[],"workspace_default_members":[],"resolve":{"nodes":[{"id":"path+file:///fixture/dep#1.0.0","dependencies":[],"deps":[{"name":"alias","pkg":"registry+dep#2.0.0","dep_kinds":[{"kind":null,"target":"cfg(unix)"}]}],"features":["old"]}],"root":null},"target_directory":"/fixture/target","workspace_root":"/fixture","metadata":null,"version":1})).unwrap()
    }
    #[test]
    fn comparison_metadata_keeps_alias_target_features_and_packages_absent_from_units() {
        let a = sample();
        let mut b = sample();
        let node = &mut b.resolve.as_mut().unwrap().nodes[0];
        node.deps[0].name = "new_alias".into();
        node.features = serde_json::from_value(json!(["new"])).unwrap();
        let before = packages(&a, &[]);
        let after = packages(&b, &[]);
        let pairs = package_pairs(&before, &after);
        assert!(
            pairs[0]
                .differences
                .contains(&"dependency_alias_package_kind_or_target".into())
        );
        assert!(pairs[0].differences.contains(&"metadata_features".into()));
        assert_eq!(before.len(), 2);
        assert!(before.iter().all(|p| p.selected_units.is_empty()));
    }
    #[test]
    fn comparison_native_profile_projection_keeps_original_indices_and_all_ambiguous_candidates() {
        let profile=serde_json::from_value::<crate::semantic::NativePackageContext>(json!({"package_id":"exact","manifest":"/fixture/Cargo.toml","source_root":"/fixture","workspace_root":"/fixture","library_root":"/fixture/src/lib.rs","features":[],"target":"target","role":"lib","edition":"2024","cfg":[],"source_hashes":{},"dependencies":[],"dependency_issues":[],"dependency_exclusions":[]})).unwrap();
        let unit = CompilerUnit {
            package_id: "exact".into(),
            package_name: "dep".into(),
            target_name: "dep".into(),
            target_kinds: vec!["lib".into()],
            source_path: "/fixture/src/lib.rs".into(),
            features: BTreeSet::new(),
            platform: Some("target".into()),
            mode: "check".into(),
            root: true,
        };
        let mut unrelated = profile.clone();
        unrelated.package_id = "unrelated".into();
        let (projected, joins, omitted) = profile_projection(
            std::slice::from_ref(&unit),
            Observation::Available {
                value: vec![unrelated, profile.clone()],
            },
        );
        assert_eq!(omitted, 1);
        assert_eq!(joins[0].profiles, [1]);
        assert_eq!(projected.value().unwrap()[0].original_index, 1);
        let (projected, joins, omitted) = profile_projection(
            &[unit],
            Observation::Available {
                value: vec![profile; 120],
            },
        );
        assert_eq!(omitted, 0);
        assert_eq!(projected.value().unwrap().len(), 120);
        assert_eq!(joins[0].profiles.len(), 120);
        for index in &joins[0].profiles {
            assert!(
                projected
                    .value()
                    .unwrap()
                    .iter()
                    .any(|profile| profile.original_index == *index)
            );
        }
    }
    #[test]
    fn comparison_metadata_raw_snapshot_guard_refuses_replaced_file() {
        let t = tempfile::tempdir().unwrap();
        let path = t.path().join("metadata.json");
        std::fs::write(&path, serde_json::to_vec(&sample()).unwrap()).unwrap();
        let (_, original) = metadata(&path, None).unwrap();
        std::fs::write(&path, b"{}").unwrap();
        assert!(metadata(&path, Some(&original.sha256)).is_err());
    }
    #[test]
    fn comparison_unit_pair_key_retains_host_target_test_and_feature_changes() {
        let unit = CompilerUnit {
            package_id: "exact".into(),
            package_name: "dep".into(),
            target_name: "dep".into(),
            target_kinds: vec!["lib".into()],
            source_path: "/fixture/src/lib.rs".into(),
            features: ["before".into()].into(),
            platform: Some("target".into()),
            mode: "check".into(),
            root: true,
        };
        let mut changed = unit.clone();
        changed.features = ["after".into()].into();
        assert_eq!(unit_key(&unit, None), unit_key(&changed, None));
        let paired = unit_pairs(
            std::slice::from_ref(&unit),
            &[],
            std::slice::from_ref(&changed),
            &[],
        );
        assert_eq!(paired[0].differences, ["unit_features"]);
        assert_eq!(paired[0].correspondence, "qualified_candidate");
        changed.platform = None;
        assert_ne!(unit_key(&unit, None), unit_key(&changed, None));
        changed = unit.clone();
        changed.mode = "test".into();
        assert_ne!(unit_key(&unit, None), unit_key(&changed, None));
        let unmatched = unit_pairs(&[unit], &[], &[changed], &[]);
        assert!(
            unmatched
                .iter()
                .all(|pair| pair.correspondence == "unmatched")
        );
    }
}
