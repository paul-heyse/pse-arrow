// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit source selection resolves before the pure Cargo declaration normalizer.
use super::{
    WorkflowError,
    execute::{Action, Input},
};
use crate::{
    environment::Environment,
    selection::{self, Plan, Request, RerunCase, Scope, SourceBasis},
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    sync::{Arc, atomic::AtomicBool},
};

#[derive(Deserialize)]
struct HistoricalReport {
    version: u32,
    invocation: crate::envelope::CliInvocationId,
    action: Action,
    observations: Vec<HistoricalGroup>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum HistoricalGroup {
    Test {
        observation: HistoricalObservation,
    },
    #[serde(other)]
    Other,
}
#[derive(Deserialize)]
struct HistoricalObservation {
    summary: HistoricalSummary,
}
#[derive(Deserialize)]
struct HistoricalSummary {
    failed: Vec<crate::discovery::TestIdentity>,
}

pub(super) fn resolve(
    metadata: &cargo_metadata::Metadata,
    mut request: Request,
    profile: String,
    environment: &Environment,
) -> Result<(Plan, String), WorkflowError> {
    let root = metadata.workspace_root.as_std_path();
    let target_selector = if let Scope::Targets { scope, selector } = request.scope.clone() {
        request.scope = *scope;
        Some(selector)
    } else {
        None
    };
    let scope_targets = |scope| match &target_selector {
        Some(selector) => Scope::Targets {
            scope: Box::new(scope),
            selector: selector.clone(),
        },
        None => scope,
    };
    match request.scope.clone() {
        Scope::Changed { base, features } => {
            let changes = crate::affected::observe(root, &base, Arc::new(AtomicBool::new(false)))
                .map_err(crate::HostError::from)?;
            let preview = crate::affected::preview(root, &changes, Some(metadata))
                .map_err(crate::HostError::from)?;
            let basis = SourceBasis::Changed {
                base,
                base_commit: preview.base_commit.clone(),
            };
            let plan = match preview.scope(features).map_err(crate::HostError::from)? {
                Some(scope) => {
                    request.scope = scope_targets(scope);
                    selection::plan(metadata, request)?
                }
                None => selection::empty(&request),
            };
            Ok((plan.source(basis, Vec::new()), profile))
        }
        Scope::RerunFailed { invocation } => {
            if target_selector.is_some() || request.operation != selection::Operation::Test {
                return Err(WorkflowError::Invalid(
                    "failed-test rerun requires a test operation".into(),
                ));
            }
            let store = crate::invocation::InvocationStore::new(
                crate::paths::CheckoutState::resolve(root, environment)
                    .map_err(crate::HostError::from)?,
            );
            let id = store
                .resolve(&invocation, None)?
                .ok_or_else(|| WorkflowError::Invalid("rerun invocation is absent".into()))?;
            let directory = store.directory(&id)?;
            let original: Input = directory
                .evidence(crate::invocation::InvocationEvidence::WorkflowInput)?
                .ok_or_else(|| {
                    WorkflowError::Invalid("original performing test input is absent".into())
                })?;
            if original.action != Action::Test || original.invocation != id {
                return Err(WorkflowError::Invalid(
                    "rerun input is not the selected original test operation".into(),
                ));
            }
            let captured_evidence = crate::test_inventory::captured(&store, id.as_str(), None)?;
            let historical: HistoricalReport = serde_json::from_slice(&captured_evidence.report)
                .map_err(crate::discovery::DiscoveryError::from)?;
            if historical.version != 1
                || historical.invocation != id
                || historical.action != Action::Test
            {
                return Err(WorkflowError::Invalid(
                    "original performing report identity differs".into(),
                ));
            }
            let inventory = captured_evidence.inventory;
            let mut cases = Vec::new();
            let mut packages = Vec::new();
            for failure in historical
                .observations
                .into_iter()
                .flat_map(|group| match group {
                    HistoricalGroup::Test { observation } => observation.summary.failed,
                    HistoricalGroup::Other => Vec::new(),
                })
            {
                let captured = inventory.test(&failure)?;
                let root = inventory
                    .selection
                    .groups()
                    .get(failure.group)
                    .and_then(|group| {
                        group
                            .roots()
                            .iter()
                            .find(|root| root.package_id() == failure.package_id)
                    })
                    .ok_or_else(|| {
                        WorkflowError::Invalid(
                            "failed identity has no original compiler root".into(),
                        )
                    })?;
                let kind = captured
                    .artifact
                    .cargo
                    .target
                    .kind
                    .first()
                    .ok_or_else(|| {
                        WorkflowError::Invalid("failed identity has no native target kind".into())
                    })?
                    .to_string();
                let target = selection::TargetIntent {
                    name: captured.artifact.cargo.target.name.to_string(),
                    kind: Some(kind),
                };
                let features = selection::Features {
                    defaults: false,
                    all: false,
                    enabled: captured.artifact.cargo.features.iter().cloned().collect(),
                    absent: root.features().absent.clone(),
                };
                let case = RerunCase {
                    package: root.package_name().into(),
                    target: target.clone(),
                    features: features.clone(),
                    binary_id: failure.binary_id,
                    test_name: failure.test_name,
                };
                if !cases.contains(&case) {
                    cases.push(case);
                }
                let package = selection::PackageIntent {
                    package: root.package_name().into(),
                    target: Some(target),
                    features,
                };
                if !packages.contains(&package) {
                    packages.push(package);
                }
            }
            request.scope = Scope::Packages(packages);
            request.platform = original.request.platform;
            request.filters = inventory.selection.filters().clone();
            request.ignored = inventory.selection.ignored();
            let plan = if cases.is_empty() {
                selection::empty(&request)
            } else {
                selection::plan(metadata, request)?
            };
            Ok((
                plan.source(SourceBasis::RerunFailed { invocation: id }, cases),
                inventory.runner_profile,
            ))
        }
        _ => {
            request.scope = scope_targets(request.scope);
            Ok((selection::plan(metadata, request)?, profile))
        }
    }
}

pub(super) fn verify_rerun(inventory: &crate::discovery::Inventory) -> Result<(), WorkflowError> {
    let mut mapped = BTreeSet::new();
    for identity in inventory.selected_tests_iter() {
        let captured = inventory.test(&identity)?;
        for (index, case) in inventory.selection.rerun_cases().iter().enumerate() {
            let original_package = inventory
                .selection
                .groups()
                .get(identity.group)
                .is_some_and(|group| {
                    group.roots().iter().any(|root| {
                        root.package_id() == identity.package_id
                            && root.package_name() == case.package
                    })
                });
            if original_package
                && identity.binary_id == case.binary_id
                && identity.test_name == case.test_name
                && captured.artifact.cargo.target.name.as_str() == case.target.name
                && case.target.kind.as_ref().is_none_or(|kind| {
                    captured
                        .artifact
                        .cargo
                        .target
                        .kind
                        .iter()
                        .any(|actual| actual.to_string() == *kind)
                })
                && captured
                    .artifact
                    .cargo
                    .features
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>()
                    == case.features.enabled
            {
                mapped.insert(index);
            }
        }
    }
    if mapped.len() != inventory.selection.rerun_cases().len() {
        return Err(WorkflowError::Invalid("original failed test identity or compiler feature context is absent from current native listing; select current tests explicitly".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn changed_source_is_resolved_before_native_target_and_feature_normalization() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path();
        std::fs::create_dir_all(root.join("crates/alpha/src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers=['crates/alpha']\n",
        )
        .unwrap();
        std::fs::write(
            root.join("crates/alpha/src/lib.rs"),
            "pub fn original() {}\n",
        )
        .unwrap();
        for args in [
            vec!["init", "-q"],
            vec!["add", "."],
            vec![
                "-c",
                "user.name=fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "initial",
            ],
        ] {
            assert!(
                std::process::Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        }
        let id = "path+file:///fixture/alpha#0.1.0";
        let metadata:cargo_metadata::Metadata=serde_json::from_value(json!({"version":1,"workspace_root":root,"target_directory":root.join("target"),"workspace_members":[id],"workspace_default_members":[id],"packages":[{"name":"alpha","version":"0.1.0","id":id,"manifest_path":root.join("crates/alpha/Cargo.toml"),"dependencies":[],"features":{"default":[]},"targets":[{"name":"actual_library","kind":["lib"],"crate_types":["lib"],"src_path":root.join("crates/alpha/src/lib.rs"),"edition":"2024"}]}]})).unwrap();
        let request = Request {
            operation: selection::Operation::Test,
            scope: Scope::Targets {
                scope: Box::new(Scope::Changed {
                    base: "HEAD".into(),
                    features: selection::Features::default(),
                }),
                selector: selection::CargoTargets::Library,
            },
            platform: None,
            filters: selection::Filters::default(),
            ignored: selection::IgnoredPolicy::Default,
        };
        let (empty, _) = resolve(
            &metadata,
            request.clone(),
            "default".into(),
            &Environment::new(),
        )
        .unwrap();
        assert!(empty.groups().is_empty());
        std::fs::write(
            root.join("crates/alpha/src/lib.rs"),
            "pub fn changed() {}\n",
        )
        .unwrap();
        let (selected, _) =
            resolve(&metadata, request, "default".into(), &Environment::new()).unwrap();
        assert_eq!(selected.groups().len(), 1);
        assert_eq!(
            selected.groups()[0].roots()[0].target().unwrap().name,
            "actual_library"
        );
        let serialized = serde_json::to_value(selected).unwrap();
        assert_eq!(serialized["source"]["kind"], "changed");
        assert_eq!(serialized["source"]["base"], "HEAD");
    }
}
