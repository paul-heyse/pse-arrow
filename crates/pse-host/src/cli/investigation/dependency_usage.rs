// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive native census adapter; session preparation stays explicit.
use crate::{
    HostError,
    envelope::{Envelope, OutputOptions},
    semantic,
};
use std::path::Path;

#[derive(Debug, clap::Args)]
pub(in crate::cli::investigation) struct Args {
    /// Source-qualified Cargo IDs (use inspect dependency to resolve names first).
    #[arg(required = true, num_args = 1..)]
    package_ids: Vec<String>,
    #[arg(long, value_enum, default_value = "native-snapshot")]
    backend: semantic::Backend,
    #[arg(long, value_delimiter = ',')]
    features: Vec<String>,
    #[arg(long)]
    no_default_features: bool,
    #[arg(long)]
    target: Option<String>,
    #[arg(long)]
    test_context: bool,
    #[arg(long)]
    build_macros: bool,
}

pub(super) fn inspect(args: Args, root: &Path, output: &OutputOptions) -> Result<i32, HostError> {
    let environment = std::env::vars_os().collect();
    let error = |error: semantic::Error| HostError::Ownership(error.to_string());
    let expected = semantic::Configuration {
        backend: args.backend,
        project: root.to_path_buf(),
        features: args.features,
        no_default_features: args.no_default_features,
        target: args.target,
        build_macros: args.build_macros,
        test_context: args.test_context,
    }
    .resolve()
    .map_err(error)?;
    if expected.backend != semantic::Backend::NativeSnapshot {
        return Err(HostError::Ownership(
            "dependency usage requires native-snapshot".into(),
        ));
    }
    let record = semantic::load(root, &environment)
        .map_err(error)?
        .ok_or_else(|| {
            HostError::Ownership(
                "explicitly prepare a native snapshot before dependency usage inspection".into(),
            )
        })?;
    if record.configuration != expected {
        return Err(HostError::Ownership(
            "prepared native configuration differs from requested dependency usage context".into(),
        ));
    }
    let answer = super::super::super::runtime()?
        .block_on(semantic::client(
            root,
            &environment,
            &record,
            &semantic::Request::DependencyUsage {
                package_ids: args.package_ids,
            },
        ))
        .and_then(semantic::Response::into_dependency_usage)
        .map_err(error)?;
    let mut response = Envelope::response(
        "inspect dependency-usage",
        "observed written-token references; not runtime calls or exhaustive capability usage",
        answer,
    );
    response.observation.completeness = crate::envelope::Completeness::Partial;
    Ok(output.emit(response)?)
}
