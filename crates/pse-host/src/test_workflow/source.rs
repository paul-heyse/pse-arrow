// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit source selection resolves before the pure Cargo declaration normalizer.
use super::{
    WorkflowError,
    execute::Action,
    explanation::{AffectedSelection, OriginalVerdict},
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
    /// Passive original completeness, not a reconstructed native NextestSummary.
    complete: Option<bool>,
}

/// Map the original completed failing test observations to their admitted compiler roots.
/// Historical images are only references and are never opened here.
fn failed_cases(
    evidence: &crate::test_inventory::CapturedEvidence,
) -> Result<Vec<RerunCase>, WorkflowError> {
    if evidence.observation.action != Action::Test {
        return Err(WorkflowError::Invalid(
            "original performing report identity differs".into(),
        ));
    }
    if !evidence.observation.required_capture_complete {
        return Err(WorkflowError::Invalid("original failed selection has incomplete required capture; inspect its evidence and select current tests explicitly".into()));
    }
    let historical: HistoricalReport =
        serde_json::from_slice(&evidence.report).map_err(crate::discovery::DiscoveryError::from)?;
    let mut cases = Vec::new();
    for group in historical.observations {
        let HistoricalGroup::Test { observation } = group else {
            continue;
        };
        if observation.summary.complete != Some(true) {
            return Err(WorkflowError::Invalid("original failed selection has unavailable or incomplete native test reconciliation; select current tests explicitly".into()));
        }
        for failure in observation.summary.failed {
            let captured = evidence.inventory.test_record(&failure)?;
            let root = evidence
                .inventory
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
                    WorkflowError::Invalid("failed identity has no original compiler root".into())
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
            let case = RerunCase {
                package: root.package_name().into(),
                target: selection::TargetIntent {
                    name: captured.artifact.cargo.target.name.to_string(),
                    kind: Some(kind),
                },
                features: selection::Features {
                    defaults: false,
                    all: false,
                    enabled: captured.artifact.cargo.features.iter().cloned().collect(),
                    absent: root.features().absent.clone(),
                },
                binary_id: failure.binary_id,
                test_name: failure.test_name,
            };
            if !cases.contains(&case) {
                cases.push(case);
            }
        }
    }
    Ok(cases)
}

pub(super) struct Selection {
    pub plan: Plan,
    pub profile: String,
    pub evidence: Evidence,
}

#[derive(Default)]
pub(super) struct Evidence {
    pub affected: Option<AffectedSelection>,
    pub original_verdict: Option<OriginalVerdict>,
}

pub(super) fn resolve(
    metadata: &cargo_metadata::Metadata,
    mut request: Request,
    profile: String,
    environment: &Environment,
) -> Result<Selection, WorkflowError> {
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
            Ok(Selection {
                plan: plan.source(basis, Vec::new()),
                profile,
                evidence: Evidence {
                    affected: Some(AffectedSelection::from(preview)),
                    ..Evidence::default()
                },
            })
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
            let captured_evidence = crate::test_inventory::captured(&store, id.as_str(), None)?;
            let cases = failed_cases(&captured_evidence)?;
            let original_verdict = OriginalVerdict::of(&captured_evidence.observation);
            let original = crate::managed::workflow::retained_input(
                &store,
                &id,
                Action::Test,
                &captured_evidence.observation,
            )?;
            let inventory = captured_evidence.inventory;
            let mut packages = Vec::new();
            for case in &cases {
                let package = selection::PackageIntent {
                    package: case.package.clone(),
                    target: Some(case.target.clone()),
                    features: case.features.clone(),
                };
                if !packages.contains(&package) {
                    packages.push(package);
                }
            }
            request.scope = Scope::Packages(packages);
            request.platform = original.request.platform;
            request.non_incremental = original.request.non_incremental;
            request.filters = inventory.selection.filters().clone();
            request.ignored = inventory.selection.ignored();
            let plan = if cases.is_empty() {
                selection::empty(&request)
            } else {
                selection::plan(metadata, request)?
            };
            Ok(Selection {
                plan: plan.source(SourceBasis::RerunFailed { invocation: id }, cases),
                profile: inventory.runner_profile,
                evidence: Evidence {
                    original_verdict,
                    ..Evidence::default()
                },
            })
        }
        _ => {
            request.scope = scope_targets(request.scope);
            Ok(Selection {
                plan: selection::plan(metadata, request)?,
                profile,
                evidence: Evidence::default(),
            })
        }
    }
}

pub(super) fn verify_rerun(inventory: &crate::discovery::Inventory) -> Result<(), WorkflowError> {
    let cases = inventory.selection.rerun_cases();
    if cases.is_empty() {
        return Ok(());
    }
    let mut mapped = BTreeSet::new();
    let mut verified = inventory.verified_tests();
    for identity in inventory.selected_tests_iter() {
        let captured = verified.test(&identity)?;
        let root = inventory
            .selection
            .groups()
            .get(identity.group)
            .and_then(|group| {
                group
                    .roots()
                    .iter()
                    .find(|root| root.package_id() == identity.package_id)
            })
            .ok_or_else(|| {
                WorkflowError::Invalid("current rerun identity has no compiler root".into())
            })?;
        let features: BTreeSet<_> = captured.artifact.cargo.features.iter().cloned().collect();
        let mut matching = cases.iter().enumerate().filter(|(_, case)| {
            root.package_name() == case.package
                && root.features().absent == case.features.absent
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
                && features == case.features.enabled
        });
        let Some((index, _)) = matching.next() else {
            return Err(WorkflowError::Invalid("current native listing selects a test outside the original failed identities; select current tests explicitly".into()));
        };
        if matching.next().is_some() || !mapped.insert(index) {
            return Err(WorkflowError::Invalid("original failed test identity has an ambiguous current native match; select current tests explicitly".into()));
        }
    }
    if mapped.len() != cases.len() {
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
            non_incremental: false,
            ignored: selection::IgnoredPolicy::Default,
        };
        let empty = resolve(
            &metadata,
            request.clone(),
            "default".into(),
            &Environment::new(),
        )
        .unwrap();
        assert!(empty.plan.groups().is_empty());
        assert!(empty.evidence.affected.unwrap().selected_packages.is_empty());
        std::fs::write(
            root.join("crates/alpha/src/lib.rs"),
            "pub fn changed() {}\n",
        )
        .unwrap();
        let selected =
            resolve(&metadata, request, "default".into(), &Environment::new()).unwrap();
        let affected = selected.evidence.affected.unwrap();
        assert_eq!(affected.changed_packages, [id]);
        assert_eq!(affected.selected_packages[0].id, id);
        assert_eq!(selected.plan.groups().len(), 1);
        assert_eq!(
            selected.plan.groups()[0].roots()[0].target().unwrap().name,
            "actual_library"
        );
        let serialized = serde_json::to_value(selected.plan).unwrap();
        assert_eq!(serialized["source"]["kind"], "changed");
        assert_eq!(serialized["source"]["base"], "HEAD");
    }
}
