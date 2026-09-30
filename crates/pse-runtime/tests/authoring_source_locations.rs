// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Parser locations remain attached to actual rows through native transformations.
#![allow(
    clippy::unwrap_used,
    reason = "source fixture and native result assertions"
)]
use datafusion::logical_expr::{LogicalPlanBuilder, col, lit};
use pse_authoring::{ParseBudget, SourceSpan};
use pse_columnar::CancellationToken;
use pse_ids::SemanticId;
use pse_relations::{columnar::FieldCheckedBatch, generated::authored::observations};
use pse_runtime::authoring_driver::document::load_package_documents;
use std::{collections::BTreeMap, fmt::Write};
#[path = "authoring_support/mod.rs"]
mod support;

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}

/// Text documents as the exact bytes the loader reads.
fn bytes(texts: BTreeMap<String, String>) -> BTreeMap<String, Vec<u8>> {
    texts
        .into_iter()
        .map(|(path, text)| (path, text.into_bytes()))
        .collect()
}
fn texts() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "package.toml".into(),
            format!(
                "[package]\nid='{}'\nname='locations'\nversion='1'\nkind='model'\nid_policy='explicit'\ndependencies=[]\ndoc=''\n",
                id(1)
            ),
        ),
        (
            "cases/first.yaml".into(),
            format!(
                "observations:\n- id: '{}'\n  dataset_id: '{}'\n  target: first.x\n  value: 1\n  unit_id: '00000000000000000000000000000011'\n",
                id(2),
                id(4)
            ),
        ),
        (
            "cases/second.yaml".into(),
            format!(
                "observations:\n- id: '{}'\n  dataset_id: '{}'\n  target: second.y\n  value: 2\n  unit_id: '00000000000000000000000000000011'\n",
                id(3),
                id(4)
            ),
        ),
    ])
}

#[tokio::test]
async fn target_spans_follow_values_after_sort_filter_union_and_source_release() {
    let registry = support::registry();
    let bundle = load_package_documents(bytes(texts()), &registry, ParseBudget::default()).unwrap();
    let originals = bundle
        .documents
        .iter()
        .map(|document| (document.id, document.text().unwrap().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let spec = observations::spec(&registry).unwrap();
    let cancel = CancellationToken::new();
    let session = support::session(
        registry.clone(),
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 20)),
    )
    .with_checked_workspace(
        BTreeMap::from([(spec.key, bundle.batches[&spec.id].clone())]),
        &cancel,
    )
    .unwrap();
    drop(bundle);
    let source = LogicalPlanBuilder::scan(
        session.table_reference(&spec.key).unwrap(),
        session.table_source(&spec.key).unwrap(),
        None,
    )
    .unwrap()
    .build()
    .unwrap();
    let filtered = LogicalPlanBuilder::from(source.clone())
        .filter(col("value").eq(lit(1.0_f64)))
        .unwrap()
        .build()
        .unwrap();
    let plan = LogicalPlanBuilder::from(source)
        .union(filtered)
        .unwrap()
        .sort([col("value").sort(false, false)])
        .unwrap()
        .build()
        .unwrap();
    let completed = session
        .prepare_rule_plan(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let mut identities = Vec::new();
    for owned in completed.batches() {
        let checked = FieldCheckedBatch::admit_owned_projection(
            &registry,
            spec,
            owned,
            &(0..spec.columns.len()).collect::<Vec<_>>(),
        )
        .unwrap();
        for row in observations::View::from_checked(&checked)
            .unwrap()
            .rows()
            .unwrap()
        {
            let span = SourceSpan::try_from(row.source_span).unwrap();
            let original = &originals[&span.document_id];
            let excerpt = &original[span.start as usize..span.end as usize];
            assert!(
                excerpt.contains(&row.observation_id.to_string()),
                "{excerpt}"
            );
            assert!(excerpt.contains(&row.target), "{excerpt}");
            identities.push(row.observation_id);
        }
    }
    assert_eq!(identities, [id(3), id(2), id(2)]);
}

#[test]
fn authored_span_override_is_rejected() {
    let registry = support::registry();
    let mut input = texts();
    writeln!(
        input.get_mut("cases/first.yaml").unwrap(),
        "  source_span: {{document_id: '{}', start: 0, end: 0}}",
        id(9)
    )
    .unwrap();
    let error = load_package_documents(bytes(input), &registry, ParseBudget::default()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("source spans come from the original parser")
    );
}
