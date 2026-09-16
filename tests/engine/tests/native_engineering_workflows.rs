// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Engineering declarations enter through source admission and the current native pipeline.
#[path = "../../support/engineering_expectations.rs"]
mod engineering_expectations;
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;

#[path = "../../support/engineering_sources.rs"]
mod engineering_sources;
use engineering_sources::{FCTP, FTPX, HEATER, MIXER, sources};
use pse_ids::SemanticId;
use pse_relations::generated::normalized;
use std::path::Path;

async fn engineering_workflow(unit: &str, state: &str, expected_states: usize) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("pse_compiler=info,pse_rules=debug")
        .with_ansi(false)
        .try_init();
    let mut fixture = native_pipeline::Fixture::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let documents = sources(root, &fixture.registry, unit, state);
    let committed = fixture.commit(documents).await;
    let report = fixture.run(committed, "P10").await.unwrap_or_else(|error| {
        let findings = if let pse_compiler::CompilerError::AttemptFailed { record, .. } = &error {
            pse_relations::generated::provenance::pass_records::View::from_checked(
                record.relation().checked(),
            )
            .unwrap()
            .rows()
            .unwrap()
            .into_iter()
            .flat_map(|row| row.findings)
            .map(|finding| {
                format!(
                    "{:?}: {} — {}",
                    finding.check_id, finding.message, finding.values
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
        } else {
            String::new()
        };
        panic!("engineering source-to-P10 failed: {error}\n{findings}");
    });
    let configured = &report.stages[0].snapshot;
    let instances = normalized::instance_bindings::View::from_checked(
        configured
            .relation("normalized", "instance_bindings")
            .unwrap()
            .checked(),
    )
    .unwrap()
    .rows()
    .unwrap();
    let state_template = SemanticId::parse_hex(if state == FTPX {
        "7c6cfa99af1e4477a136a4ca9d5562ec"
    } else {
        "88ad610582c04b999ec786ac9f3aebba"
    })
    .unwrap();
    assert_eq!(
        instances
            .iter()
            .filter(|row| row.template_id == state_template)
            .count(),
        expected_states,
        "heater has two states; a two-inlet mixer has three"
    );
    let canonical = &report.stages.last().unwrap().snapshot;
    engineering_expectations::check(
        canonical,
        &instances,
        state_template,
        state == FCTP,
        unit == HEATER,
    );
}

#[tokio::test]
async fn heater_ftpx_source_to_canonical_graph() {
    engineering_workflow(HEATER, FTPX, 2).await;
}
#[tokio::test]
async fn heater_fctp_source_to_canonical_graph() {
    engineering_workflow(HEATER, FCTP, 2).await;
}
#[tokio::test]
async fn mixer_ftpx_source_to_canonical_graph() {
    engineering_workflow(MIXER, FTPX, 3).await;
}
#[tokio::test]
async fn mixer_fctp_source_to_canonical_graph() {
    engineering_workflow(MIXER, FCTP, 3).await;
}
