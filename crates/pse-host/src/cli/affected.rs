// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A source-reach preview is independent of test listing, fixture/provider preparation and run.
use super::{HostError, OutputOptions, checkout_root, source_path};
use crate::{
    catalog::MetadataSnapshot,
    envelope::{Completeness, Envelope, ErrorRecord, RedactionPolicy},
};
use clap::Args;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
};

#[derive(Debug, Args)]
pub(super) struct Selection {
    /// Explicit Git revision for base-to-HEAD plus observed index/worktree/untracked changes.
    #[arg(long, default_value = "HEAD")]
    base: String,
    /// Existing metadata snapshot; absence queries locked offline Cargo metadata without a build.
    #[arg(long)]
    metadata: Option<PathBuf>,
}
pub(super) fn dispatch(
    args: Selection,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let root = checkout_root(selected)?;
    let changes = crate::affected::observe(&root, &args.base, Arc::new(AtomicBool::new(false)))?;
    let mut metadata_failure = None;
    let metadata = match args.metadata {
        Some(path) => Some(MetadataSnapshot::read(&source_path(selected, path)?)?),
        None => match crate::dependency::capture(&root, &std::env::vars_os().collect()) {
            Ok(metadata) => Some(metadata),
            Err(error) => {
                metadata_failure = Some(error);
                None
            }
        },
    };
    let preview = crate::affected::preview(
        &root,
        &changes,
        metadata.as_ref().map(|snapshot| &snapshot.metadata),
    )?;
    let mut response = Envelope::response(
        "affected",
        "conservative source reach; no build, test or preparation performed",
        preview,
    );
    let policy = RedactionPolicy::from_environment();
    response.failures.extend(
        response
            .payload
            .failures
            .iter()
            .map(|failure| ErrorRecord::from_typed(failure, &policy)),
    );
    if let Some(error) = metadata_failure {
        response
            .failures
            .push(ErrorRecord::from_typed(&error, &policy));
    }
    response.observation.scope = "git_layers_and_declared_workspace_dependency_reach".into();
    response.observation.limitations = response.payload.coverage_limits.clone();
    if !response.failures.is_empty() {
        response.observation.completeness = Completeness::Partial;
    }
    Ok(output.emit(response)?)
}
