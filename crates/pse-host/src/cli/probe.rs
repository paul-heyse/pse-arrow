// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Scratch workspace effects, reusing the managed native compiler invocation owner.
use super::{GenerationIntent, HostError, ManagedCommand, OutputOptions};
use crate::{envelope::Envelope, generation::EntryGuard, paths::CheckoutState, probe};
use clap::Subcommand;
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Retain an authored standalone workspace seeded from exact locked dependencies.
    New {
        name: String,
        /// Unambiguous locked dependency names, name@version or source-qualified package IDs.
        #[arg(long, value_delimiter = ',')]
        deps: Vec<String>,
    },
    /// Resolve explicitly and run the scratch binary through correctness admission.
    Run {
        name: String,
        #[arg(last = true)]
        arguments: Vec<OsString>,
    },
    /// Delete the authored probe only when no original owner or live process uses it.
    Rm { name: String },
    /// Preview or release only regenerable Cargo output from an owned probe.
    Clean {
        name: String,
        /// Release the exact output identity returned by preview.
        #[arg(long)]
        release: Option<String>,
    },
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Clean { release: None, .. } => GenerationIntent::Passive,
            _ => GenerationIntent::NewWork,
        }
    }
}
#[derive(Debug, Serialize, JsonSchema)]
struct ActualResolution {
    probe: probe::Probe,
    metadata_sha256: String,
    metadata_directory: PathBuf,
    target_directory: PathBuf,
    packages: Vec<crate::dependency::PackageIdentity>,
    features: Vec<ResolvedFeatures>,
    rustc_identity: String,
    cargo_identity: String,
    limitations: Vec<&'static str>,
}
#[derive(Debug, Serialize, JsonSchema)]
struct ResolvedFeatures {
    package_id: String,
    features: Vec<String>,
}
fn error(error: probe::ProbeError) -> HostError {
    HostError::Probe(Box::new(error))
}
pub(super) fn dispatch(
    action: Action,
    root: &Path,
    output: &OutputOptions,
    entry: &EntryGuard,
) -> Result<i32, HostError> {
    let environment = std::env::vars_os().collect();
    let store = CheckoutState::resolve(root, &environment)?.root;
    match action {
        Action::New { name, deps } => {
            let snapshot = crate::dependency::capture(root, &environment)?;
            let probe = probe::new(root, &store, &name, &deps, &snapshot).map_err(error)?;
            Ok(output.emit(Envelope::response(
                "probe new",
                "authored scratch workspace retained; edit src/main.rs before running",
                probe,
            ))?)
        }
        Action::Rm { name } => {
            let path = super::runtime()?
                .block_on(probe::rm(root, &store, &name, &environment))
                .map_err(error)?;
            Ok(output.emit(Envelope::response(
                "probe rm",
                "selected probe removed after original recorded native roots drained",
                path,
            ))?)
        }
        Action::Clean { name, release } => {
            let result = super::runtime()?
                .block_on(probe::clean_outputs(
                    root,
                    &store,
                    &name,
                    &environment,
                    release.as_deref(),
                ))
                .map_err(error)?;
            Ok(output.emit(Envelope::response(
                "probe clean",
                "observed regenerable Cargo output under the original probe lease",
                result,
            ))?)
        }
        Action::Run { name, arguments } => {
            let control=entry.prepared().ok_or_else(||HostError::Ownership("probe run requires explicitly prepared native host tools; run scripts/bootstrap-host".into()))?;
            let control_checkout = entry.checkout().ok_or_else(|| {
                HostError::Ownership("original probe control checkout is absent".into())
            })?;
            let run =
                probe::prepare_run(root, &store, &name, &environment, &arguments).map_err(error)?;
            let features = run
                .resolution
                .metadata
                .resolve
                .as_ref()
                .map(|resolve| {
                    resolve
                        .nodes
                        .iter()
                        .map(|node| ResolvedFeatures {
                            package_id: node.id.to_string(),
                            features: node.features.iter().map(ToString::to_string).collect(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            let resolution = ActualResolution {
                probe: run.probe.clone(),
                metadata_sha256: run.resolution.digest.clone(),
                metadata_directory: run.resolution_directory.clone(),
                target_directory: run
                    .resolution
                    .metadata
                    .target_directory
                    .as_std_path()
                    .to_owned(),
                packages: run
                    .resolution
                    .metadata
                    .packages
                    .iter()
                    .map(crate::dependency::PackageIdentity::from)
                    .collect(),
                features,
                rustc_identity: run.rustc_identity.clone(),
                cargo_identity: run.cargo_identity.clone(),
                limitations: vec![
                    "copied workspace lock is a resolution seed; this is Cargo's actual scratch graph",
                    "metadata features are not compiler-unit admission or compile success",
                ],
            };
            let bytes = serde_json::to_vec_pretty(&resolution).map_err(std::io::Error::other)?;
            let path = run.resolution_directory.join("resolution.json");
            std::fs::write(&path, &bytes)?;
            let artifacts = vec![crate::envelope::ArtifactReference {
                kind: "probe_actual_resolution".into(),
                location: path.display().to_string(),
                digest: Some(probe::digest(&bytes)),
            }];
            run.record_managed(
                root,
                &run.environment,
                probe::ManagedRun {
                    invocation: output.invocation_id.clone(),
                    command_name: "probe run".into(),
                    command: run.command.clone(),
                    control: control.clone(),
                    control_checkout,
                },
            )
            .map_err(error)?;
            let status = super::launch_managed_with_artifacts(
                ManagedCommand {
                    parent: None,
                    root: root.to_owned(),
                    cwd: run.probe.directory.clone(),
                    environment: run.environment,
                    command: run.command,
                    detach: false,
                    session: None,
                    resource_class: None,
                    verb: "probe run",
                    tool: None,
                    workflow: None,
                },
                entry,
                output,
                artifacts,
            );
            drop(run.lease);
            status
        }
    }
}
