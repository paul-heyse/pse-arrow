// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive offline schema inspection; no helper or registry is acquired.
use crate::{
    HostError,
    envelope::{Envelope, OutputOptions},
    schema::Bundle,
};
use clap::Args as ClapArgs;
use std::path::{Path, PathBuf};

#[derive(Debug, ClapArgs)]
pub(in crate::cli) struct Args {
    /// Exact schema declaration name or returned descriptor address; omission lists files.
    declaration: Option<String>,
    /// Complete descriptor set including imports.
    #[arg(long)]
    descriptor: PathBuf,
    /// Compare the selected current bundle with an explicit original baseline.
    #[arg(long, conflicts_with = "declaration")]
    compare_with: Option<PathBuf>,
}
pub(super) fn inspect(args: Args, root: &Path, output: &OutputOptions) -> Result<i32, HostError> {
    let bundle = Bundle::load(&root.join(args.descriptor))?;
    if let Some(baseline) = args.compare_with {
        let baseline = Bundle::load(&root.join(baseline))?;
        let mut report = crate::schema::compare::compare(&bundle, &baseline)?;
        report.native = super::super::super::runtime()?
            .block_on(crate::schema::compare::classify(&bundle, &baseline))?;
        report.associations = crate::schema::compare::associations(root, &bundle)?
            .into_iter()
            .filter(|association| {
                report.differences.iter().any(|difference| {
                    difference.declaration == association.declaration
                        || difference
                            .declaration
                            .split_once('#')
                            .is_some_and(|(parent, _)| parent == association.declaration)
                })
            })
            .collect();
        let mut response = Envelope::response(
            "inspect schema compare",
            "qualified contract differences",
            report,
        );
        for classification in &response.payload.native {
            if matches!(classification.outcome.as_str(), "failed" | "unavailable") {
                let error = crate::schema::SchemaError::Tool(format!(
                    "Buf {} classification {} (exit {:?}): {}",
                    classification.category,
                    classification.outcome,
                    classification.exit_code,
                    String::from_utf8_lossy(&classification.stderr)
                ));
                response
                    .failures
                    .push(crate::envelope::ErrorRecord::from_typed(
                        &error,
                        &crate::envelope::RedactionPolicy::from_environment(),
                    ));
            }
        }
        if !response.failures.is_empty() {
            response.outcome = crate::envelope::CommandOutcome::Failure;
            response.observation.completeness = crate::envelope::Completeness::Partial;
            response.observation.limitations.push("Structural comparison is available; native compatibility classification is incomplete.".into());
        }
        return output.emit(response).map_err(Into::into);
    }
    output
        .emit(Envelope::response(
            "inspect schema",
            "offline descriptor-qualified schema inspection",
            bundle
                .inspect(args.declaration.as_deref())?
                .with_associations(root, &bundle)?,
        ))
        .map_err(Into::into)
}
