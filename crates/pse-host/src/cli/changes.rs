// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Focused changed-source facts composed from native Git and declared ownership.
use super::{HostError, OutputOptions, checkout_root, source_path};
use crate::{
    catalog::MetadataSnapshot,
    envelope::{Envelope, NextAction},
};
use clap::Args;
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    os::unix::ffi::OsStringExt as _,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

#[derive(Debug, Args)]
pub(super) struct Selection {
    /// Direct base-to-HEAD comparison, plus separately observed staged/working changes.
    #[arg(long, default_value = "HEAD")]
    base: String,
    /// Existing exact Cargo metadata; omission collects locked offline metadata without building.
    #[arg(long)]
    metadata: Option<PathBuf>,
}
#[derive(Debug, Serialize, JsonSchema)]
struct ChangedPath {
    source: crate::ownership::SourceOwnerQuery,
    generated: Option<crate::ownership::OwnerQuery>,
    applicable_rules: Option<Vec<crate::applicable_rules::Match>>,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct Report {
    observed: crate::affected::ChangeSet,
    paths: Vec<ChangedPath>,
    groups: Vec<PackageGroup>,
    reach: crate::affected::Preview,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
enum GroupOwner {
    WorkspacePackage { package_id: String },
    Unmapped,
    MetadataUnavailable,
}
#[derive(Debug, Serialize, JsonSchema)]
struct PackageGroup {
    owner: GroupOwner,
    path_indices: Vec<usize>,
}
pub(super) fn dispatch(
    args: Selection,
    selected: Option<&std::path::Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let snapshot = match args.metadata {
        Some(path) => MetadataSnapshot::read(&source_path(selected, path)?)?,
        None => crate::dependency::capture(&root, &std::env::vars_os().collect())?,
    };
    Ok(output.emit(inspect(&root, &snapshot, &args.base)?)?)
}
fn inspect(
    root: &std::path::Path,
    snapshot: &MetadataSnapshot,
    base: &str,
) -> Result<Envelope<Report>, HostError> {
    let observed = crate::affected::observe(root, base, Arc::new(AtomicBool::new(false)))?;
    let mut owner_failures = Vec::new();
    let rules = match crate::applicable_rules::Catalog::load(root) {
        Ok(rules) => Some(rules),
        Err(error) => {
            owner_failures.push(error);
            None
        }
    };
    let paths = observed
        .paths()
        .into_iter()
        .map(|path| {
            let path = PathBuf::from(std::ffi::OsString::from_vec(path.as_bytes().to_vec()));
            let generated = match crate::ownership::query(&root, &path) {
                Ok(owner) => Some(owner),
                Err(error) => {
                    owner_failures.push(HostError::from(error));
                    None
                }
            };
            Ok(ChangedPath {
                applicable_rules: rules.as_ref().map(|rules| rules.matching(&path)),
                source: crate::ownership::source_owner(&root, &path, Some(&snapshot))?,
                generated,
            })
        })
        .collect::<Result<Vec<_>, HostError>>()?;
    let reach = crate::affected::preview(&root, &observed, Some(&snapshot.metadata))?;
    let mut groups = std::collections::BTreeMap::new();
    for (index, path) in paths.iter().enumerate() {
        let (key, owner) = match &path.source.package {
            crate::ownership::PackageOwnership::WorkspacePackage { package_id, .. } => (
                format!("workspace:{package_id}"),
                GroupOwner::WorkspacePackage {
                    package_id: package_id.clone(),
                },
            ),
            crate::ownership::PackageOwnership::Unmapped { .. } => {
                ("unmapped".into(), GroupOwner::Unmapped)
            }
            crate::ownership::PackageOwnership::Unavailable { .. } => {
                ("unavailable".into(), GroupOwner::MetadataUnavailable)
            }
        };
        groups
            .entry(key)
            .or_insert_with(|| PackageGroup {
                owner,
                path_indices: vec![],
            })
            .path_indices
            .push(index);
    }
    let mut response = Envelope::response(
        "changes",
        "changed paths with declared owners and conservative test reach",
        Report {
            observed,
            paths,
            groups: groups.into_values().collect(),
            reach,
        },
    );
    response.observation.scope = "direct_base_to_head_plus_separate_live_git_layers".into();
    response.observation.limitations = response.payload.reach.coverage_limits.clone();
    response.observation.limitations.push("Shared-rule scope matches are advisory navigation from current authored YAML globs, not an edit authorization or replacement for AGENTS and native guards.".into());
    let policy = crate::envelope::RedactionPolicy::from_environment();
    response.failures.extend(
        owner_failures
            .iter()
            .map(|failure| crate::envelope::ErrorRecord::from_typed(failure, &policy)),
    );
    response.failures.extend(
        response
            .payload
            .paths
            .iter()
            .filter_map(|path| path.generated.as_ref())
            .flat_map(|owner| owner.failures.iter())
            .map(|failure| crate::envelope::ErrorRecord::from_typed(failure, &policy)),
    );
    response.failures.extend(
        response
            .payload
            .reach
            .failures
            .iter()
            .map(|failure| crate::envelope::ErrorRecord::from_typed(failure, &policy)),
    );
    if !response.failures.is_empty() {
        response.observation.completeness = crate::envelope::Completeness::Partial;
    }
    response.next.push(NextAction::Command {
        argv: vec![
            "pse".into(),
            "test".into(),
            "--changed".into(),
            "--base".into(),
            base.into(),
        ],
        reason:
            "select tests through the original conservative correctness owner; no tests have run"
                .into(),
    });
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_layers_and_declared_package_survive_unavailable_generator_map() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\n[package]\nname='changed-fixture'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(
            root.join("Cargo.lock"),
            "version = 4\n\n[[package]]\nname = \"changed-fixture\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::create_dir(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn original() {}\n").unwrap();
        for args in [
            vec!["init", "-q"],
            vec!["add", "."],
            vec![
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "original",
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
        let snapshot = crate::dependency::capture(root, &std::env::vars_os().collect()).unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn changed() {}\n").unwrap();
        let report = inspect(root, &snapshot, "HEAD").unwrap();
        assert!(
            report
                .payload
                .observed
                .live
                .changes
                .iter()
                .any(|change| change.path.as_bytes() == b"src/lib.rs"
                    && change.layer == crate::repository::ChangeLayer::Worktree)
        );
        let package = snapshot
            .metadata
            .workspace_packages()
            .into_iter()
            .find(|package| package.name.as_str() == "changed-fixture")
            .unwrap();
        assert!(
            report
                .payload
                .reach
                .changed_packages
                .contains(&package.id.to_string())
        );
        assert_eq!(
            report.observation.completeness,
            crate::envelope::Completeness::Partial
        );
        assert!(!report.failures.is_empty());
        assert!(
            report
                .payload
                .paths
                .iter()
                .any(|path| path.generated.is_none())
        );
    }
}
