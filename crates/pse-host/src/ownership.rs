// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive generated-output routing over the aggregate generator's actual declarations.
#[path = "../../../scripts/generated-outputs.rs"]
pub mod protocol;

use protocol::{
    GeneratorOwner, MAP_PATH, MAP_PRODUCER, OutputMap, RelationshipDocument, Scope, ScopeKind,
    SourceRelationship, VERSION, normal_relative,
};
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    collections::{BTreeSet, VecDeque},
    ffi::OsString,
    fs,
    path::{Component, Path, PathBuf},
};

/// Original map/sidecar read or domain refusal; readers never regenerate missing evidence.
#[derive(Debug, thiserror::Error)]
pub enum OwnershipError {
    /// Native original filesystem cause.
    #[error("source ownership I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Original generated record cannot be decoded.
    #[error("source ownership JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// A record or path cannot establish its declared routing contract.
    #[error("source ownership: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    OwnershipError,
    code(this) { Some(match this { Self::Io(_) => pse_diagnostics::DiagnosticCode::HostIo, _ => pse_diagnostics::DiagnosticCode::HostOwnership }) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn invalid(message: impl Into<String>) -> OwnershipError {
    OwnershipError::Invalid(message.into())
}

/// Checked last-aggregate declaration snapshot; no dirty-source freshness is inferred.
#[derive(Debug)]
pub struct OwnershipMap {
    record: OutputMap,
}
impl OwnershipMap {
    /// Read and check one existing generated map. This starts no generator or compiler.
    pub fn read(root: &Path) -> Result<Self, OwnershipError> {
        let record: OutputMap = serde_json::from_slice(&fs::read(root.join(MAP_PATH))?)?;
        if record.schema_version != VERSION {
            return Err(invalid("unsupported generated output map version"));
        }
        let mut producers = BTreeSet::new();
        let mut outputs = BTreeSet::new();
        let mut self_owned = false;
        for owner in &record.owners {
            if owner.producer.is_empty()
                || !producers.insert(&owner.producer)
                || owner.outputs.is_empty()
                || owner.regenerate.is_empty()
                || owner.regenerate.iter().any(String::is_empty)
            {
                return Err(invalid(
                    "generated output map has an empty or duplicate producer",
                ));
            }
            for scope in owner.outputs.iter().chain(&owner.inputs) {
                if !normal_relative(Path::new(&scope.path)) {
                    return Err(invalid("generated output map has a non-relative scope"));
                }
            }
            for scope in &owner.outputs {
                if !outputs.insert(PathBuf::from(&scope.path)) {
                    return Err(invalid("generated output map repeats an output root"));
                }
                if owner.producer == MAP_PRODUCER
                    && scope.path == MAP_PATH
                    && scope.kind == ScopeKind::File
                {
                    self_owned = true;
                }
            }
            if let Some(path) = &owner.contributors
                && !normal_relative(Path::new(path))
            {
                return Err(invalid("contributor sidecar is not repository relative"));
            }
        }
        if !self_owned {
            return Err(invalid(
                "generated output map omits its own exact file owner",
            ));
        }
        Ok(Self { record })
    }

    /// Prefer the most specific actual output scope, including nested generated schema owners.
    pub fn owner(
        &self,
        relative: &Path,
    ) -> Result<Option<(&GeneratorOwner, &Scope)>, OwnershipError> {
        if !normal_relative(relative) {
            return Err(invalid(
                "ownership lookup requires a normalized repository-relative path",
            ));
        }
        Ok(self
            .record
            .owners
            .iter()
            .flat_map(|owner| owner.outputs.iter().map(move |scope| (owner, scope)))
            .filter(|(_, scope)| scope.contains(relative))
            .max_by_key(|(_, scope)| Path::new(&scope.path).components().count()))
    }

    /// Conservative declared input reach, distinct from exact contributor lineage.
    pub fn input_owners(&self, relative: &Path) -> Result<Vec<&GeneratorOwner>, OwnershipError> {
        if !normal_relative(relative) {
            return Err(invalid(
                "input lookup requires a normalized repository-relative path",
            ));
        }
        Ok(self
            .record
            .owners
            .iter()
            .filter(|owner| owner.inputs.iter().any(|scope| scope.contains(relative)))
            .collect())
    }
    /// Original map's assurance limits.
    pub fn coverage_limits(&self) -> &[String] {
        &self.record.coverage_limits
    }
}

/// A declared output owner or an honest absence from the selected map.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OutputOwnership {
    /// The most specific declared generator scope matches.
    Declared {
        /// Actual producer/regeneration and conservative source scopes.
        owner: GeneratorOwner,
        /// Exact file or enclosing generated tree that matched.
        scope: Scope,
    },
    /// No output declaration matches; this does not establish source/consumer completeness.
    Unmapped,
}
/// Original contributor evidence is independent of the output map's protection route.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Contributors {
    /// Selected exact output's records from its existing producer sidecar.
    Available {
        /// Existing original record address.
        sidecar: String,
        /// Producer-declared contributors; absent source locations remain absent.
        records: Vec<SourceRelationship>,
        /// Original producer's independent coverage/identity limits.
        coverage_limits: Vec<String>,
    },
    /// This producer declares no contributor sidecar.
    NotDeclared,
    /// Output routing is known, but the independent contributor source could not be read.
    Unavailable {
        /// Existing original record address.
        sidecar: String,
        /// Explicit source failure, never an empty successful lineage result.
        reason: String,
    },
}
/// One exact output-to-owner/source route; no syntax query, editing or regeneration occurs.
#[derive(Debug, Serialize, JsonSchema)]
pub struct OwnerQuery {
    /// Original normalized native operand, including non-UTF8 platform bytes.
    pub path: OsString,
    /// Declaration-bound output ownership.
    pub ownership: OutputOwnership,
    /// Existing contributor source, independent of protection and regeneration.
    pub contributors: Contributors,
    /// Selected aggregate map's exact evidence limits.
    pub coverage_limits: Vec<String>,
    /// Independent original contributor failures for the outer diagnostic projection.
    #[serde(skip)]
    #[schemars(skip)]
    pub failures: Vec<OwnershipError>,
}

fn contributors(
    root: &Path,
    owner: &GeneratorOwner,
    relative: &Path,
) -> Result<Contributors, OwnershipError> {
    let Some(sidecar) = &owner.contributors else {
        return Ok(Contributors::NotDeclared);
    };
    let document: RelationshipDocument = serde_json::from_slice(&fs::read(root.join(sidecar))?)?;
    if document.schema_version != 1 || document.producer != owner.producer {
        return Err(invalid(
            "contributor source producer/version differs from output routing",
        ));
    }
    for record in &document.relationships {
        if !normal_relative(Path::new(&record.output_path))
            || !normal_relative(Path::new(&record.generator_path))
            || record
                .source_path
                .as_ref()
                .is_some_and(|path| !normal_relative(Path::new(path)))
            || !owner
                .outputs
                .iter()
                .any(|scope| scope.contains(Path::new(&record.output_path)))
        {
            return Err(invalid(
                "contributor record is outside its declared source/output scope",
            ));
        }
    }
    Ok(Contributors::Available {
        sidecar: sidecar.clone(),
        records: document
            .relationships
            .into_iter()
            .filter(|record| Path::new(&record.output_path) == relative)
            .collect(),
        coverage_limits: document.coverage_limits,
    })
}

/// Read an exact path's routing and existing contributor evidence without preparing anything.
pub fn query(root: &Path, path: &Path) -> Result<OwnerQuery, OwnershipError> {
    let root = resolve(root)?;
    let target = resolve(&root.join(path))?;
    let map = OwnershipMap::read(&root)?;
    let matched = target
        .strip_prefix(&root)
        .ok()
        .filter(|relative| !relative.as_os_str().is_empty())
        .map(|relative| map.owner(relative))
        .transpose()?
        .flatten();
    let mut failures = Vec::new();
    let (ownership, contributors) = match matched {
        Some((owner, scope)) => {
            let relative = target
                .strip_prefix(&root)
                .map_err(|_| invalid("selected owner lies outside checkout"))?;
            let contributors = match contributors(&root, owner, relative) {
                Ok(value) => value,
                Err(error) => {
                    let unavailable = Contributors::Unavailable {
                        sidecar: owner.contributors.clone().unwrap_or_default(),
                        reason: error.to_string(),
                    };
                    failures.push(error);
                    unavailable
                }
            };
            (
                OutputOwnership::Declared {
                    owner: owner.clone(),
                    scope: scope.clone(),
                },
                contributors,
            )
        }
        None => (OutputOwnership::Unmapped, Contributors::NotDeclared),
    };
    Ok(OwnerQuery {
        path: target.into_os_string(),
        ownership,
        contributors,
        coverage_limits: map.record.coverage_limits,
        failures,
    })
}

// Resolve existing symlinks even when the final target does not exist. Shared by queries/hooks.
pub(crate) fn resolve(path: &Path) -> Result<PathBuf, OwnershipError> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut pending = absolute
        .components()
        .map(|component| component.as_os_str().to_owned())
        .collect::<VecDeque<_>>();
    let mut resolved = PathBuf::new();
    let mut links = 0;
    while let Some(part) = pending.pop_front() {
        match Path::new(&part).components().next() {
            Some(Component::CurDir) => {}
            Some(Component::ParentDir) => {
                resolved.pop();
            }
            Some(Component::RootDir | Component::Prefix(_)) => resolved.push(part),
            Some(Component::Normal(_)) => {
                let next = resolved.join(&part);
                match fs::symlink_metadata(&next) {
                    Ok(metadata) if metadata.is_symlink() => {
                        links += 1;
                        if links > 40 {
                            return Err(invalid(
                                "edit path has a symlink cycle or excessive indirection",
                            ));
                        }
                        let target = fs::read_link(next)?;
                        if target.is_absolute() {
                            resolved.clear();
                        }
                        for component in target.components().rev() {
                            pending.push_front(component.as_os_str().to_owned());
                        }
                    }
                    Ok(_) => resolved.push(part),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        resolved.push(part)
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            None => {}
        }
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests;
