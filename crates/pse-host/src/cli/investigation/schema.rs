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
    /// Original file to assess semantically; repeat for a bounded explicit selection.
    #[arg(long, requires = "compare_with")]
    semantic_file: Vec<String>,
    /// Available native semantic policy; acquired options cannot nominate native coverage.
    #[arg(long, value_enum, requires = "compare_with")]
    semantic_policy: Option<crate::schema::compare::semantics::Policy>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;
    #[derive(Debug, clap::Parser)]
    struct Parsed {
        #[command(flatten)]
        args: Args,
    }
    #[test]
    fn schema_semantics_cli_report_retains_known_policy_beside_independent_buf_disposition() {
        let directory = tempfile::tempdir().unwrap();
        let descriptors = crate::developer_evidence_contracts::DESCRIPTORS;
        std::fs::write(directory.path().join("old.bin"), descriptors).unwrap();
        std::fs::write(directory.path().join("new.bin"), descriptors).unwrap();
        let args = Parsed::try_parse_from([
            "schema",
            "--descriptor",
            "new.bin",
            "--compare-with",
            "old.bin",
            "--semantic-file",
            "developer_evidence.proto",
            "--semantic-policy",
            "developer-evidence",
        ])
        .unwrap()
        .args;
        let invocation = crate::envelope::CliInvocationId::new();
        let output = OutputOptions {
            format: crate::envelope::OutputFormat::Json,
            presentation: crate::envelope::Presentation {
                fields: Some(vec!["semantic".into()]),
                budget: None,
            },
            invocation_id: invocation.clone(),
            artifact_directory: Some(directory.path().join("retained")),
        };
        let code = inspect(args, directory.path(), &output).unwrap();
        let report: serde_json::Value = serde_json::from_slice(
            &std::fs::read(
                directory
                    .path()
                    .join("retained")
                    .join(format!("{invocation}.json")),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(report["payload"]["semantic"]["disposition"], "compatible");
        assert_eq!(
            report["payload"]["semantic"]["policy"],
            "pse.developer.evidence.v1/native"
        );
        assert_eq!(
            report["payload"]["semantic"]["native_coverage_available"],
            true
        );
        assert_eq!(report["payload"]["native"].as_array().unwrap().len(), 3);
        assert_eq!(report["payload"]["differences"], serde_json::json!([]));
        // A Buf refusal of the original artifact remains an independent command failure;
        // it neither erases nor upgrades an available native semantic assessment.
        if code == 1 {
            assert_eq!(report["outcome"], "failure");
            assert!(!report["failures"].as_array().unwrap().is_empty());
            assert!(report["payload"]["native"].as_array().unwrap().iter().any(
                |category| matches!(category["outcome"].as_str(), Some("failed" | "unavailable"))
            ));
        } else {
            assert_eq!(code, 0);
            assert_eq!(report["outcome"], "success");
            assert!(
                report["payload"]["native"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|category| category["outcome"] == "compatible")
            );
        }
        assert!(Parsed::try_parse_from(["schema", "--descriptor", "new.bin"]).is_ok());
        assert!(
            Parsed::try_parse_from([
                "schema",
                "--descriptor",
                "new.bin",
                "--semantic-file",
                "developer_evidence.proto"
            ])
            .is_err()
        );
        assert!(
            Parsed::try_parse_from([
                "schema",
                "--descriptor",
                "new.bin",
                "--semantic-policy",
                "developer-evidence"
            ])
            .is_err()
        );
    }
}
pub(super) fn inspect(args: Args, root: &Path, output: &OutputOptions) -> Result<i32, HostError> {
    let bundle = Bundle::load(&root.join(args.descriptor))?;
    if let Some(baseline) = args.compare_with {
        let baseline = Bundle::load(&root.join(baseline))?;
        let mut report = crate::schema::compare::compare(&bundle, &baseline)?;
        report.semantic = crate::schema::compare::semantics::assess(
            &bundle,
            &baseline,
            &args.semantic_file,
            args.semantic_policy.unwrap_or_default(),
        )?;
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
