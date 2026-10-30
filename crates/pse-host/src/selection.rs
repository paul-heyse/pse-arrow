// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared test/check intent and admitted Cargo compiler contexts.
//!
//! Metadata declares targets. Only the correctness adapter establishes selected compiler
//! units; neither source invents a built binary or a runnable test population.

use crate::correctness::{self, CorrectnessError};
use cargo_metadata::Metadata;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, ffi::OsString};

/// Compiler action shared by test, check, discovery and explain consumers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    /// Source-built tests; listing is a separate explicit preparation step.
    Test,
    /// Check the same selected source context.
    Check,
}

/// Ignored tests remain part of selection identity, including exact reruns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum IgnoredPolicy {
    /// Run ordinary tests only.
    #[default]
    Default,
    /// Run ignored tests only.
    Only,
    /// Run ordinary and ignored tests.
    All,
}

/// Literal filter inputs. The test runner owns filter grammar and matching.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Filters {
    /// Bare test-name words, retained independently of full filtersets.
    pub words: Vec<String>,
    /// Explicit nextest filtersets.
    pub expressions: Vec<String>,
}

/// One intended feature context. Absence controls must survive grouping and admission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Features {
    /// Whether Cargo's default features are enabled.
    pub defaults: bool,
    /// Explicit all-features selection.
    pub all: bool,
    /// Package-local features, or qualified Cargo feature inputs.
    pub enabled: BTreeSet<String>,
    /// Package-local features that must remain absent in selected root units.
    pub absent: BTreeSet<String>,
}
impl Default for Features {
    fn default() -> Self {
        Self {
            defaults: true,
            all: false,
            enabled: BTreeSet::new(),
            absent: BTreeSet::new(),
        }
    }
}

/// Explicit target intent; kind disambiguates a same-named bin and integration target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TargetIntent {
    /// Cargo target name.
    pub name: String,
    /// Cargo target kind, when specified.
    pub kind: Option<String>,
}

/// One independently meaningful package/target and feature context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PackageIntent {
    /// Exact Cargo package ID or unambiguous workspace package name.
    pub package: String,
    /// No target means Cargo's ordinary package selection; no required-feature union occurs.
    pub target: Option<TargetIntent>,
    /// This intent's feature policy.
    pub features: Features,
}

/// Package scope before normalization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Explicit package/target intentions, each preserving its feature context.
    Packages(Vec<PackageIntent>),
    /// Workspace members except the named or ID-qualified exclusions.
    Workspace {
        /// Exact workspace names or package IDs to exclude.
        exclude: Vec<String>,
        /// Feature policy preserved independently for each member.
        features: Features,
    },
    /// Cargo metadata's default workspace members.
    DefaultMembers {
        /// Feature policy preserved independently for each default member.
        features: Features,
    },
}

/// Parser-independent request. CLI adapters do not own selection normalization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    /// Selected compiler action.
    pub operation: Operation,
    /// Package and target intent.
    pub scope: Scope,
    /// Explicit target triple, or Cargo's configured native/default selection.
    pub platform: Option<String>,
    /// Test matching inputs.
    pub filters: Filters,
    /// Ignored-test policy.
    pub ignored: IgnoredPolicy,
}

/// Declared target identity; this is not a binary identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DeclaredTarget {
    /// Target name.
    pub name: String,
    /// All declared target kinds.
    pub kinds: Vec<String>,
    /// Source path supplied by Cargo metadata.
    pub source_path: String,
    /// Required features activated by this explicit target intent.
    pub required_features: BTreeSet<String>,
}

/// One intended root, retaining its independent target and feature policy inside a group.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Root {
    package_id: String,
    package_name: String,
    target: Option<DeclaredTarget>,
    features: Features,
}
impl Root {
    /// Exact package identity.
    pub fn package_id(&self) -> &str {
        &self.package_id
    }
    /// Package name for presentation.
    pub fn package_name(&self) -> &str {
        &self.package_name
    }
    /// Explicit declared target, if any.
    pub fn target(&self) -> Option<&DeclaredTarget> {
        self.target.as_ref()
    }
    /// Effective requested features, including this explicit target's requirements.
    pub fn features(&self) -> &Features {
        &self.features
    }
    fn selector(&self) -> Result<Vec<String>, CorrectnessError> {
        let Some(target) = &self.target else {
            return Ok(Vec::new());
        };
        let flag = target_flag(&target.kinds)?;
        Ok(if flag == "--lib" {
            vec![flag.into()]
        } else {
            vec![flag.into(), target.name.clone()]
        })
    }
}

/// A real root set with shared Cargo selectors; admission checks feature compatibility.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Group {
    roots: Vec<Root>,
    platform: Option<String>,
    arguments: Vec<String>,
}

enum AdmissionMismatch {
    Invalid(CorrectnessError),
    FeatureAbsence(BTreeSet<usize>),
}
impl From<CorrectnessError> for AdmissionMismatch {
    fn from(error: CorrectnessError) -> Self {
        Self::Invalid(error)
    }
}

impl Group {
    /// Exact intended roots, each with its own feature-absence controls.
    pub fn roots(&self) -> &[Root] {
        &self.roots
    }
    /// Requested target platform; actual host/target contexts come from admission.
    pub fn platform(&self) -> Option<&str> {
        self.platform.as_deref()
    }
    /// Cargo input shared by explain and admission, before Arrow feature propagation.
    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub(crate) fn validate_resolution(
        &self,
        resolution: &correctness::Resolution,
    ) -> Result<(), CorrectnessError> {
        self.check_resolution(resolution)
            .map_err(|error| match error {
                AdmissionMismatch::Invalid(error) => error,
                AdmissionMismatch::FeatureAbsence(roots) => invalid(format!(
                    "selected Cargo roots activate features required absent: {}",
                    roots
                        .iter()
                        .map(|&index| self.roots[index].package_name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            })
    }

    fn check_resolution(
        &self,
        resolution: &correctness::Resolution,
    ) -> Result<(), AdmissionMismatch> {
        let mut found = BTreeSet::new();
        let mut absent = BTreeSet::new();
        for unit in resolution.units.iter().filter(|unit| unit.root) {
            let (index, root) = self
                .roots
                .iter()
                .enumerate()
                .find(|(_, root)| root.package_id == unit.package_id)
                .ok_or_else(|| {
                    invalid("admitted Cargo root is outside normalized package selection")
                })?;
            found.insert(index);
            if unit.package_name != root.package_name {
                return Err(invalid("admitted Cargo package identity/name disagree").into());
            }
            if let Some(target) = &root.target
                && (unit.target_name != target.name
                    || unit.target_kinds != target.kinds
                    || unit.source_path != target.source_path)
            {
                return Err(
                    invalid("admitted Cargo target differs from explicit target intent").into(),
                );
            }
            if let Some(platform) = &self.platform
                && unit.platform.as_ref() != Some(platform)
                && !(unit.platform.is_none()
                    && unit.target_kinds.iter().any(|kind| kind == "proc-macro"))
            {
                return Err(
                    invalid("admitted Cargo root differs from selected target platform").into(),
                );
            }
            if !unit.features.is_disjoint(&root.features.absent) {
                absent.insert(index);
            }
            for feature in &root.features.enabled {
                let qualified = format!("{}/", root.package_name);
                let local = feature.strip_prefix(&qualified).unwrap_or(feature);
                if !local.contains('/') && !unit.features.contains(local) {
                    return Err(invalid(
                        "admitted Cargo root omits an explicitly required feature",
                    )
                    .into());
                }
            }
        }
        if found.len() != self.roots.len() || found.is_empty() {
            return Err(
                invalid("admitted Cargo graph omits an intended package/target root").into(),
            );
        }
        if !absent.is_empty() {
            return Err(AdmissionMismatch::FeatureAbsence(absent));
        }
        Ok(())
    }

    fn new(
        operation: Operation,
        roots: Vec<Root>,
        platform: Option<String>,
    ) -> Result<Self, CorrectnessError> {
        let first = roots
            .first()
            .ok_or_else(|| invalid("compiler group has no roots"))?;
        let mut arguments = vec![
            match operation {
                Operation::Test => "test",
                Operation::Check => "check",
            }
            .into(),
            "--locked".into(),
        ];
        for root in &roots {
            arguments.extend(["-p".into(), root.package_id.clone()]);
        }
        arguments.extend(first.selector()?);
        if let Some(platform) = &platform {
            arguments.extend(["--target".into(), platform.clone()]);
        }
        if !first.features.defaults {
            arguments.push("--no-default-features".into());
        }
        if first.features.all {
            arguments.push("--all-features".into());
        }
        let enabled: BTreeSet<_> = roots
            .iter()
            .flat_map(|root| {
                root.features.enabled.iter().map(|feature| {
                    if feature.contains('/') {
                        feature.clone()
                    } else {
                        format!("{}/{feature}", root.package_name)
                    }
                })
            })
            .collect();
        if !enabled.is_empty() {
            arguments.extend([
                "--features".into(),
                enabled.into_iter().collect::<Vec<_>>().join(","),
            ]);
        }
        Ok(Self {
            roots,
            platform,
            arguments,
        })
    }

    fn accepts(&self, root: &Root) -> Result<bool, CorrectnessError> {
        let first = &self.roots[0];
        Ok(!self
            .roots
            .iter()
            .any(|existing| existing.package_id == root.package_id)
            && first.features.defaults == root.features.defaults
            && first.features.all == root.features.all
            && first.selector()? == root.selector()?)
    }
}

/// Pure normalized intent. Explain can render this without building or listing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Plan {
    operation: Operation,
    groups: Vec<Group>,
    filters: Filters,
    ignored: IgnoredPolicy,
}
impl Plan {
    /// Candidate compiler groups; actual admission can split feature-incompatible sets.
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }
    /// Matching inputs retained for discovery and execution.
    pub fn filters(&self) -> &Filters {
        &self.filters
    }
    /// Ignored-test policy retained for discovery and execution.
    pub fn ignored(&self) -> IgnoredPolicy {
        self.ignored
    }
    /// Selected action.
    pub fn operation(&self) -> Operation {
        self.operation
    }

    /// Query real Cargo graphs and verify Arrow admission, without building or preparing.
    pub fn resolve(self) -> Result<Resolved, CorrectnessError> {
        let cargo = correctness::real_cargo()?;
        let mut context = correctness::AdmissionContext::default();
        self.resolve_with(|arguments| {
            context.resolve_with(arguments, |arguments| {
                correctness::capture(&cargo, arguments)
            })
        })
    }

    /// Inject the existing admission owner for independent focused tests.
    pub fn resolve_with<F>(mut self, mut admit: F) -> Result<Resolved, CorrectnessError>
    where
        F: FnMut(&[OsString]) -> Result<correctness::Resolution, CorrectnessError>,
    {
        let mut admitted = Vec::with_capacity(self.groups.len());
        let mut pending: std::collections::VecDeque<_> = std::mem::take(&mut self.groups).into();
        while let Some(group) = pending.pop_front() {
            let arguments: Vec<_> = group.arguments.iter().map(OsString::from).collect();
            let resolution = admit(&arguments)?;
            match group.check_resolution(&resolution) {
                Ok(()) => {}
                Err(AdmissionMismatch::FeatureAbsence(violating)) if group.roots.len() > 1 => {
                    // Split only on owned feature-absence evidence. A singleton that still
                    // violates its policy fails; no failed admission ever launches work.
                    let split = *violating
                        .iter()
                        .next()
                        .ok_or_else(|| invalid("missing incompatible root"))?;
                    let mut remaining = group.roots;
                    let root = remaining.remove(split);
                    let separate = Group::new(self.operation, vec![root], group.platform.clone())?;
                    let remainder = Group::new(self.operation, remaining, group.platform)?;
                    pending.push_front(remainder);
                    pending.push_front(separate);
                    continue;
                }
                Err(_) => {
                    group.validate_resolution(&resolution)?;
                }
            }
            self.groups.push(group);
            admitted.push(resolution);
        }
        Ok(Resolved {
            plan: self,
            admitted,
        })
    }
}

/// One admitted operation for explain, capability/fixture resolution and execution.
/// There are deliberately no binary paths or test IDs until explicit build/list preparation.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Resolved {
    plan: Plan,
    admitted: Vec<correctness::Resolution>,
}
impl Resolved {
    /// Original root policies and filters, with the grouping established by admission.
    pub fn plan(&self) -> &Plan {
        &self.plan
    }
    /// Admitted compiler groups in plan order, including the actual Arrow feature splice.
    pub fn admitted(&self) -> &[correctness::Resolution] {
        &self.admitted
    }
    /// Capabilities required by the actual selected backend compiler units.
    pub fn native_capabilities(&self) -> BTreeSet<String> {
        self.admitted
            .iter()
            .flat_map(|group| group.native_capabilities.iter().cloned())
            .collect()
    }
    /// A test selection remains pending actual binary/test discovery until explicitly built/listed.
    pub fn needs_binary_listing(&self) -> bool {
        self.plan.operation == Operation::Test
    }
}

fn invalid(message: impl Into<String>) -> CorrectnessError {
    CorrectnessError::Invalid(message.into())
}

fn package<'a>(
    metadata: &'a Metadata,
    selector: &str,
) -> Result<&'a cargo_metadata::Package, CorrectnessError> {
    let mut matches = metadata
        .workspace_packages()
        .into_iter()
        .filter(|package| package.id.repr == selector || package.name.as_str() == selector);
    let selected = matches
        .next()
        .ok_or_else(|| invalid(format!("unknown workspace package {selector}")))?;
    if matches.next().is_some() {
        return Err(invalid(format!("ambiguous workspace package {selector}")));
    }
    Ok(selected)
}

fn target_flag(kinds: &[String]) -> Result<&'static str, CorrectnessError> {
    for kind in kinds {
        match kind.as_str() {
            "lib" | "rlib" | "dylib" | "cdylib" | "staticlib" | "proc-macro" => return Ok("--lib"),
            "test" => return Ok("--test"),
            "bin" => return Ok("--bin"),
            "example" => return Ok("--example"),
            "bench" => return Ok("--bench"),
            _ => {}
        }
    }
    Err(invalid("target kind has no supported compiler selection"))
}

/// Normalize against Cargo declarations only. This performs no I/O or preparation.
pub fn plan(metadata: &Metadata, request: Request) -> Result<Plan, CorrectnessError> {
    if request.operation == Operation::Check
        && (request.filters != Filters::default() || request.ignored != IgnoredPolicy::Default)
    {
        return Err(invalid(
            "check does not accept test filters or ignored-test policy",
        ));
    }
    let intents = match request.scope {
        Scope::Packages(intents) => intents,
        Scope::Workspace { exclude, features } => {
            let excluded: BTreeSet<_> = exclude
                .iter()
                .map(|name| package(metadata, name).map(|package| package.id.clone()))
                .collect::<Result<_, _>>()?;
            metadata
                .workspace_packages()
                .into_iter()
                .filter(|package| !excluded.contains(&package.id))
                .map(|package| PackageIntent {
                    package: package.id.repr.clone(),
                    target: None,
                    features: features.clone(),
                })
                .collect()
        }
        Scope::DefaultMembers { features } => {
            if metadata.workspace_default_members.is_missing() {
                return Err(invalid("Cargo default workspace members unavailable"));
            }
            metadata
                .workspace_default_packages()
                .into_iter()
                .map(|package| PackageIntent {
                    package: package.id.repr.clone(),
                    target: None,
                    features: features.clone(),
                })
                .collect()
        }
    };
    if intents.is_empty() {
        return Err(invalid("package selection is empty"));
    }
    let mut roots = Vec::new();
    for intent in intents {
        let package = package(metadata, &intent.package)?;
        let mut features = intent.features;
        if features.all && !features.absent.is_empty() {
            return Err(invalid(
                "all-features conflicts with feature-absence controls",
            ));
        }
        let target = if let Some(intent) = intent.target {
            let mut matches = package.targets.iter().filter(|target| {
                target.name == intent.name
                    && intent.kind.as_ref().is_none_or(|kind| {
                        target.kind.iter().any(|value| value.to_string() == *kind)
                    })
            });
            let selected = matches.next().ok_or_else(|| {
                invalid(format!("unknown target {}:{}", package.name, intent.name))
            })?;
            if matches.next().is_some() {
                return Err(invalid(format!(
                    "ambiguous target {}:{}; specify kind",
                    package.name, intent.name
                )));
            }
            features
                .enabled
                .extend(selected.required_features.iter().cloned());
            Some(DeclaredTarget {
                name: selected.name.clone(),
                kinds: selected.kind.iter().map(ToString::to_string).collect(),
                source_path: selected.src_path.to_string(),
                required_features: selected.required_features.iter().cloned().collect(),
            })
        } else {
            None
        };
        for feature in &features.enabled {
            let local = feature
                .strip_prefix(&format!("{}/", package.name))
                .unwrap_or(feature);
            if features.absent.contains(local) {
                return Err(invalid(format!(
                    "feature {local} is both enabled and required absent"
                )));
            }
            if !feature.contains('/')
                && !package.features.keys().any(|name| name.as_str() == feature)
            {
                return Err(invalid(format!(
                    "unknown feature {}:{feature}",
                    package.name
                )));
            }
        }
        let root = Root {
            package_id: package.id.repr.clone(),
            package_name: package.name.to_string(),
            target,
            features,
        };
        if !roots.contains(&root) {
            roots.push(root);
        }
    }
    let mut groups: Vec<Group> = Vec::new();
    for root in roots {
        let mut compatible = None;
        for (index, group) in groups.iter().enumerate() {
            if group.accepts(&root)? {
                compatible = Some(index);
                break;
            }
        }
        if let Some(index) = compatible {
            let mut roots = std::mem::take(&mut groups[index].roots);
            roots.push(root);
            groups[index] = Group::new(request.operation, roots, request.platform.clone())?;
        } else {
            groups.push(Group::new(
                request.operation,
                vec![root],
                request.platform.clone(),
            )?);
        }
    }
    Ok(Plan {
        operation: request.operation,
        groups,
        filters: request.filters,
        ignored: request.ignored,
    })
}

#[cfg(test)]
mod tests;
