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

/// A compatible Cargo invocation. Independent contexts are never feature-unioned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Group {
    package_id: String,
    package_name: String,
    target: Option<DeclaredTarget>,
    features: Features,
    platform: Option<String>,
    arguments: Vec<String>,
}
impl Group {
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
        let roots: Vec<_> = resolution.units.iter().filter(|unit| unit.root).collect();
        if roots.is_empty() || roots.iter().any(|unit| unit.package_id != self.package_id) {
            return Err(invalid(
                "admitted Cargo roots differ from normalized package selection",
            ));
        }
        for unit in roots {
            if let Some(target) = &self.target
                && (unit.target_name != target.name
                    || unit.target_kinds != target.kinds
                    || unit.source_path != target.source_path)
            {
                return Err(invalid(
                    "admitted Cargo target differs from explicit target intent",
                ));
            }
            if !unit.features.is_disjoint(&self.features.absent) {
                return Err(invalid(format!(
                    "{} activates a feature required absent",
                    self.package_name
                )));
            }
        }
        Ok(())
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
    /// Compatible compiler selections, with independent feature contexts retained.
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
        self.resolve_with(correctness::resolve)
    }

    /// Inject the existing admission owner for independent focused tests.
    pub fn resolve_with<F>(self, mut admit: F) -> Result<Resolved, CorrectnessError>
    where
        F: FnMut(&[OsString]) -> Result<correctness::Resolution, CorrectnessError>,
    {
        let mut admitted = Vec::with_capacity(self.groups.len());
        for group in &self.groups {
            let arguments: Vec<_> = group.arguments.iter().map(OsString::from).collect();
            let resolution = admit(&arguments)?;
            group.validate_resolution(&resolution)?;
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
    /// The unchanged normalized intent, including filters and ignored policy.
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
    let mut groups = Vec::new();
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
        let mut arguments = vec![
            match request.operation {
                Operation::Test => "test",
                Operation::Check => "check",
            }
            .into(),
            "--locked".into(),
            "-p".into(),
            package.id.repr.clone(),
        ];
        if let Some(target) = &target {
            let flag = target_flag(&target.kinds)?;
            arguments.push(flag.into());
            if flag != "--lib" {
                arguments.push(target.name.clone());
            }
        }
        if let Some(platform) = &request.platform {
            arguments.extend(["--target".into(), platform.clone()]);
        }
        if !features.defaults {
            arguments.push("--no-default-features".into());
        }
        if features.all {
            arguments.push("--all-features".into());
        }
        if !features.enabled.is_empty() {
            arguments.extend([
                "--features".into(),
                features
                    .enabled
                    .iter()
                    .map(|feature| {
                        if feature.contains('/') {
                            feature.clone()
                        } else {
                            format!("{}/{feature}", package.name)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(","),
            ]);
        }
        let group = Group {
            package_id: package.id.repr.clone(),
            package_name: package.name.to_string(),
            target,
            features,
            platform: request.platform.clone(),
            arguments,
        };
        if !groups.contains(&group) {
            groups.push(group);
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
