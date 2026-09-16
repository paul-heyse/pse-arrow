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

#[tokio::test]
async fn material_configuration_keeps_generated_members_and_native_support_together() {
    use pse_relations::generated::{authored, provenance::pass_records};
    use std::collections::BTreeSet;

    let mut fixture = native_pipeline::Fixture::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let committed = fixture
        .commit(sources(root, &fixture.registry, HEATER, FTPX))
        .await;
    let report = fixture.run(committed, "P3").await.unwrap();
    let stage = report.stages.last().unwrap();
    let members = normalized::material_domain_members::View::from_checked(
        stage
            .snapshot
            .relation("normalized", "material_domain_members")
            .unwrap()
            .checked(),
    )
    .unwrap()
    .rows()
    .unwrap();
    let phase = SemanticId::parse_hex("436ec3e1a25947f6924123e50715927e").unwrap();
    let species = SemanticId::parse_hex("4578f9c197334d6a96955114fe9ccd4d").unwrap();
    let element = SemanticId::parse_hex("fc9744e59f95042db7f36df279602748").unwrap();
    let system = SemanticId::parse_hex("30a37480208b4591b2126f24e5b6bd3d").unwrap();
    // The heater's child control volume declares an element domain as well.
    assert_eq!(members.len(), 3);
    assert!(members.iter().all(|row| row.material_system_id == system));
    assert_eq!(
        members
            .iter()
            .map(|row| (row.phase_id, row.species_id, row.element_id))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            (Some(phase), None, None),
            (None, Some(species), None),
            (None, None, Some(element)),
        ])
    );
    let domains = normalized::domain_members::View::from_checked(
        stage
            .snapshot
            .relation("normalized", "domain_members")
            .unwrap()
            .checked(),
    )
    .unwrap()
    .rows()
    .unwrap();
    for member in &members {
        assert!(
            domains
                .iter()
                .any(|row| row.domain_id == member.domain_id && row.member_id == member.member_id)
        );
    }
    let records = pass_records::View::from_checked(stage.record.relation().checked())
        .unwrap()
        .rows()
        .unwrap();
    let evidence = records
        .iter()
        .flat_map(|record| &record.derivations)
        .filter(|derivation| {
            derivation.relation_id == normalized::material_domain_members::RELATION_ID
        })
        .collect::<Vec<_>>();
    assert!(!evidence.is_empty());
    let supporting = evidence
        .iter()
        .flat_map(|derivation| &derivation.supporting)
        .map(|source| source.relation_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        supporting,
        BTreeSet::from([
            authored::material_systems::RELATION_ID,
            authored::species::RELATION_ID,
            authored::phases::RELATION_ID,
            authored::species_elements::RELATION_ID,
            pse_relations::generated::reference::elements::RELATION_ID,
        ])
    );
}

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
                    "{:?}: {} — {:?}",
                    finding.check_id, finding.message, finding.evidence
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
        } else {
            String::new()
        };
        let mut causes = vec![error.to_string()];
        let mut source = std::error::Error::source(&error);
        while let Some(cause) = source {
            causes.push(cause.to_string());
            source = cause.source();
        }
        panic!(
            "engineering source-to-P10 failed: {}\n{findings}",
            causes.join(": ")
        );
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
