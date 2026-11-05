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
    /// Analyze a separately prepared project while retaining this checkout's session owner.
    #[arg(long)]
    project: Option<std::path::PathBuf>,
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
    let error = |error: semantic::Error| HostError::Semantic(Box::new(error));
    let expected = semantic::Configuration {
        backend: args.backend,
        project: args.project.unwrap_or_else(|| root.to_path_buf()),
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
    Ok(output.emit(report(answer))?)
}

fn report(answer: semantic::DependencyUsageAnswer) -> Envelope<semantic::DependencyUsageAnswer> {
    let incomplete = !answer.unavailable_package_ids.is_empty() || answer.unmapped_definitions != 0;
    let mut response = Envelope::response(
        "inspect dependency-usage",
        "observed written-token references; not runtime calls or exhaustive capability usage",
        answer,
    );
    response.observation.scope = "selected_native_world_resolved_written_token_references".into();
    response.observation.limitations = response.payload.limitations.clone();
    if incomplete {
        response.observation.completeness = crate::envelope::Completeness::Partial;
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    fn answer() -> semantic::DependencyUsageAnswer {
        semantic::DependencyUsageAnswer {
            session: semantic::AnalysisContext {
                backend: "native_snapshot".into(),
                configuration: semantic::Configuration {
                    backend: semantic::Backend::NativeSnapshot,
                    project: "/snapshot".into(),
                    features: vec![],
                    no_default_features: false,
                    target: None,
                    build_macros: false,
                    test_context: false,
                },
                position_encoding: "utf-8".into(),
                backend_status: None,
                preparation: "restricted immutable snapshot".into(),
            },
            generation: 1,
            package_ids: vec!["supplier".into()],
            unavailable_package_ids: vec![],
            references: vec![],
            indexed_files: 1,
            unmapped_definitions: 0,
            limitations: vec!["No runtime or exhaustive source-usage proof".into()],
        }
    }
    #[test]
    fn dependency_usage_delivery_completion_preserves_scope_and_missing_evidence() {
        let complete = report(answer());
        assert_eq!(complete.exit_code(), 0);
        assert!(
            complete
                .observation
                .scope
                .starts_with("selected_native_world")
        );
        assert_eq!(
            complete.observation.limitations,
            complete.payload.limitations
        );
        let mut missing = answer();
        missing.unavailable_package_ids.push("missing".into());
        assert_eq!(report(missing).exit_code(), 1);
        let mut unmapped = answer();
        unmapped.unmapped_definitions = 1;
        assert_eq!(report(unmapped).exit_code(), 1);
    }
}
