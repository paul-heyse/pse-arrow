// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Admitted document tables are tracked queries (ADR-0125, review F10): the workspace is
//! their one reuse owner.
use super::*;
use pse_authoring::{
    ParseBudget,
    language::{IdentityPolicy, parse},
};
use pse_modeling::document::{DocumentColumn, RowSet, Values};
use std::sync::atomic::AtomicUsize;

const SOURCE: SemanticId = SemanticId::from_bytes([81; 16]);
const PACKAGE: SemanticId = SemanticId::from_bytes([82; 16]);

const TEXT: &str = r#"package p {
 entity kind source provenance { attribute title: Text; }
 enum role { given }
 entity source s { title = "document test" }
 identifier scheme cas;
 entity kind item { attribute cas: Id<cas>?; }
 entity item a { cas = Id<cas>("71-43-2") }
 entity item b { cas = Id<cas>("108-88-3") }
 table first[j: item by cas]: Energy storage {kJ} missing optional;
 table second[j: item by cas]: Energy storage {J} missing optional;
 dataset f: first provenance(s, role.given) from "data/first.parquet";
 dataset g: second provenance(s, role.given) from "data/second.parquet";
}"#;

fn rows(text: &str) -> Vec<Declaration> {
    parse(text, SOURCE, IdentityPolicy::Named, ParseBudget::default()).unwrap()
}
fn document(path: &str, magnitudes: &[f64]) -> Arc<DataDocument> {
    let bytes = format!("{path}:{magnitudes:?}");
    Arc::new(DataDocument {
        id: pse_ids::named_id(PACKAGE, path),
        path: path.into(),
        content_hash: pse_ids::encoding_checksum(bytes.as_bytes()).content_hash(),
        rows: RowSet::new(vec![
            DocumentColumn {
                name: "j".into(),
                unit: None,
                values: Values::Text(
                    ["71-43-2", "108-88-3"]
                        .iter()
                        .take(magnitudes.len())
                        .map(|v| Some((*v).to_owned()))
                        .collect(),
                ),
            },
            DocumentColumn {
                name: "value".into(),
                unit: None,
                values: Values::Magnitude(magnitudes.iter().copied().map(Some).collect()),
            },
        ])
        .unwrap(),
    })
}
fn inventory(first: &[f64], second: &[f64]) -> Arc<DocumentInventory> {
    Arc::new(DocumentInventory {
        packages: [(SOURCE, PACKAGE)].into(),
        documents: [
            document("data/first.parquet", first),
            document("data/second.parquet", second),
        ]
        .into_iter()
        .map(|d| (d.id, d))
        .collect(),
    })
}
fn value(revision: &ModelingRevision, table: &str, item: &str) -> f64 {
    let p = &revision.checked;
    let id = p.entry(table).unwrap();
    let key = vec![pse_modeling::specialize::Value::Entity {
        id: p.entry(item).unwrap(),
        kind: p.entry("p.item").unwrap(),
    }];
    let pse_modeling::specialize::Value::Number { bits, .. } =
        p.table(id).unwrap().rows[&key].cells[0]
    else {
        panic!("a quantity value")
    };
    f64::from_bits(bits)
}

/// Editing one data document re-admits only the tables it supplies; an unchanged document
/// under an unchanged plan reuses its admitted rows, and the incremental revision equals a
/// clean workspace's.
#[test]
fn unchanged_dataset_reuses_admitted_table() {
    let executions = Arc::new(AtomicUsize::new(0));
    let observed = executions.clone();
    let mut workspace = CompilerWorkspace::with_events(
        super::super::tests::inputs(),
        WorkspaceLimits::default(),
        Some(Box::new(move |event| {
            if let salsa::EventKind::WillExecute { .. } = event.kind
                && format!("{:?}", event.kind).contains("document_table")
            {
                observed.fetch_add(1, Ordering::Relaxed);
            }
        })),
    )
    .unwrap();
    let scope = PhysicalScope::default();
    let original = workspace
        .publish_modeling_with(
            rows(TEXT),
            scope.clone(),
            inventory(&[1.0, 2.0], &[3.0, 4.0]),
        )
        .unwrap();
    assert_eq!(executions.load(Ordering::Relaxed), 2);
    assert_eq!(value(&original, "p.first", "p.a"), 1000.0);
    assert_eq!(value(&original, "p.second", "p.b"), 4.0);
    // One document's bytes change: only its table is admitted again.
    let edited = inventory(&[1.0, 2.0], &[3.0, 5.0]);
    let incremental = workspace
        .publish_modeling_with(rows(TEXT), scope.clone(), edited.clone())
        .unwrap();
    assert_eq!(executions.load(Ordering::Relaxed), 3);
    assert_eq!(value(&incremental, "p.second", "p.b"), 5.0);
    let clean = CompilerWorkspace::new(super::super::tests::inputs(), WorkspaceLimits::default())
        .unwrap()
        .publish_modeling_with(rows(TEXT), scope.clone(), edited.clone())
        .unwrap();
    assert_eq!(incremental.checked, clean.checked);
    // A declaration edit that leaves the first table's plan unchanged reuses its rows; one
    // that changes its storage unit admits them again.
    let renamed = TEXT.replace("dataset g:", "dataset renamed_g:");
    workspace
        .publish_modeling_with(rows(&renamed), scope.clone(), edited.clone())
        .unwrap();
    assert_eq!(executions.load(Ordering::Relaxed), 4);
    let restored = TEXT.replace("Energy storage {kJ}", "Energy storage {J}");
    let replanned = workspace
        .publish_modeling_with(rows(&restored), scope, edited)
        .unwrap();
    assert_eq!(executions.load(Ordering::Relaxed), 5);
    assert_eq!(value(&replanned, "p.first", "p.a"), 1.0);
}

#[test]
fn document_reuse_decoded_interpretation_changes_match_clean_admission() {
    let mut workspace =
        CompilerWorkspace::new(super::super::tests::inputs(), WorkspaceLimits::default()).unwrap();
    let old_inventory = inventory(&[1., 2.], &[3., 4.]);
    let original = workspace
        .publish_modeling_with(rows(TEXT), PhysicalScope::default(), old_inventory.clone())
        .unwrap();
    let mut edited = (*inventory(&[1., 2.], &[3., 5.])).clone();
    let id = pse_ids::named_id(PACKAGE, "data/second.parquet");
    Arc::make_mut(edited.documents.get_mut(&id).unwrap()).content_hash =
        old_inventory.documents[&id].content_hash;
    let edited = Arc::new(edited);
    let incremental = workspace
        .publish_modeling_with(rows(TEXT), PhysicalScope::default(), edited.clone())
        .unwrap();
    let clean = CompilerWorkspace::new(super::super::tests::inputs(), WorkspaceLimits::default())
        .unwrap()
        .publish_modeling_with(rows(TEXT), PhysicalScope::default(), edited)
        .unwrap();
    assert_eq!(value(&original, "p.second", "p.b"), 4.);
    assert_eq!(value(&incremental, "p.second", "p.b"), 5.);
    assert_eq!(incremental.checked, clean.checked);
}

/// A document's decoded rows and its admitted table are charged to the workspace's input
/// limit: the same declarations admit a small document and refuse a large one.
#[test]
fn document_rows_are_charged_to_the_workspace_limits() {
    let text = format!(
        "{} table big[n: Integer]: Energy storage {{J}} missing optional;\n dataset bg: big provenance(s, role.given) from \"data/big.parquet\";\n}}",
        TEXT.trim_end_matches('}')
    );
    let big = |rows: i64| {
        let mut inventory = inventory(&[1.0], &[1.0]).as_ref().clone();
        let document = Arc::new(DataDocument {
            id: pse_ids::named_id(PACKAGE, "data/big.parquet"),
            path: "data/big.parquet".into(),
            content_hash: pse_ids::encoding_checksum(&rows.to_le_bytes()).content_hash(),
            rows: RowSet::new(vec![
                DocumentColumn {
                    name: "n".into(),
                    unit: None,
                    values: Values::Integer((0..rows).map(Some).collect()),
                },
                DocumentColumn {
                    name: "value".into(),
                    unit: None,
                    values: Values::Magnitude((0..rows).map(|n| Some(n as f64)).collect()),
                },
            ])
            .unwrap(),
        });
        inventory.documents.insert(document.id, document);
        Arc::new(inventory)
    };
    let mut workspace = CompilerWorkspace::new(
        super::super::tests::inputs(),
        WorkspaceLimits {
            input_bytes: 32 << 20,
            ..WorkspaceLimits::default()
        },
    )
    .unwrap();
    let small = workspace
        .publish_modeling_with(rows(&text), PhysicalScope::default(), big(10))
        .unwrap();
    assert!(small.retained_bytes() > small.documents().retained_bytes());
    assert_eq!(
        small
            .checked
            .table(small.checked.entry("p.big").unwrap())
            .unwrap()
            .rows
            .len(),
        10
    );
    let refused =
        workspace.publish_modeling_with(rows(&text), PhysicalScope::default(), big(200_000));
    assert!(
        matches!(
            refused,
            Err(CompileError::Modeling(pse_modeling::ModelingError::Budget(
                _
            )))
        ),
        "{refused:?}"
    );
}
