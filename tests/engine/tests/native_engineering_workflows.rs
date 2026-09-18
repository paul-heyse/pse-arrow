// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    reason = "test fixture construction and exact independent value assertions"
)]
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
    use pse_relations::generated::authored;
    use std::collections::BTreeSet;

    let fixture = native_pipeline::Fixture::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let committed = fixture.source(sources(root, &fixture.registry, HEATER, FTPX));
    let report = fixture.evaluate(committed, "P3").await.unwrap();
    let stage = &report;
    let members = normalized::material_domain_members::View::from_checked(
        native_pipeline::relation(stage, "normalized", "material_domain_members").unwrap(),
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
        native_pipeline::relation(stage, "normalized", "domain_members").unwrap(),
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
    let evidence = pse_relations::generated::provenance::derivations::View::from_checked(
        native_pipeline::relation(stage, "provenance", "derivations").unwrap(),
    )
    .unwrap()
    .rows()
    .unwrap()
    .into_iter()
    .filter(|derivation| derivation.relation_id == normalized::material_domain_members::RELATION_ID)
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
    let fixture = native_pipeline::Fixture::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let documents = sources(root, &fixture.registry, unit, state);
    let committed = fixture.source(documents);
    let report = fixture
        .evaluate(committed, "P10")
        .await
        .unwrap_or_else(|error| {
            let mut causes = vec![error.to_string()];
            let mut source = std::error::Error::source(&error);
            while let Some(cause) = source {
                causes.push(cause.to_string());
                source = cause.source();
            }
            panic!("engineering source-to-P10 failed: {}", causes.join(": "));
        });
    let configured = &report;
    let instances = normalized::instance_bindings::View::from_checked(
        native_pipeline::relation(configured, "normalized", "instance_bindings").unwrap(),
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
    let canonical = &report;
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
