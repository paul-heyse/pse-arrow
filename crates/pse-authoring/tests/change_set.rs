// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact typed preimages and staging inventory protect unpublished candidates.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixture assertions"
)]

use arrow_array::RecordBatch;
use pse_authoring::{
    AuthoringError,
    change_set::{AuthoredReader, ChangeSet, apply},
};
use pse_ids::SemanticId;
use pse_relations::generated::authored;
use pse_schema::model::Cell;
use std::collections::BTreeMap;

struct Base {
    revision: SemanticId,
    rows: BTreeMap<SemanticId, RecordBatch>,
}
impl AuthoredReader for Base {
    fn revision_id(&self) -> SemanticId {
        self.revision
    }
    fn relations(&self) -> Result<BTreeMap<SemanticId, RecordBatch>, AuthoringError> {
        Ok(self.rows.clone())
    }
}
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn header() -> authored::change_sets::Row {
    authored::change_sets::Row {
        change_set_id: id(1),
        base_revision_id: id(2),
        author: "fixture".to_owned(),
        message: "typed operations".to_owned(),
        created_at: 0,
    }
}
fn row(name: &str, default: &str) -> Vec<Cell> {
    authored::template_params::Row {
        template_id: id(3),
        name: name.to_owned(),
        logical_type_id: id(4),
        enum_id: None,
        default: Some(default.to_owned()),
        required: true,
        domain_spec: None,
        doc: "Parameter.".to_owned(),
    }
    .into_cells()
}
#[test]
fn composite_text_key_updates_and_deletes_use_complete_actual_preimages() {
    let registry = pse_schema::registry().unwrap();
    let spec = registry.relation("authored.template_params").unwrap();
    let before = row("pressure", "1");
    let after = row("pressure", "2");
    let batch =
        pse_relations::cells::batch_from_cells(registry, spec, std::slice::from_ref(&before))
            .unwrap();
    let base = Base {
        revision: id(2),
        rows: BTreeMap::from([(spec.id, batch)]),
    };
    let mut changes = ChangeSet::new(header());
    changes.update(registry, spec, &before, &after).unwrap();
    let candidate = apply(&base, &changes, registry).unwrap();
    assert_eq!(
        pse_relations::cells::cells_from_batch(registry, spec, &candidate.relations[&spec.id])
            .unwrap(),
        vec![after.clone()]
    );
    assert_eq!(
        pse_relations::cells::cells_from_batch(registry, spec, &base.rows[&spec.id]).unwrap(),
        vec![before.clone()]
    );
    let mut wrong = ChangeSet::new(header());
    wrong
        .update(registry, spec, &row("pressure", "stale"), &after)
        .unwrap();
    assert!(apply(&base, &wrong, registry).is_err());
    let mut delete = ChangeSet::new(header());
    delete.delete(registry, spec, &before).unwrap();
    assert!(delete.ops[0].row.is_none());
    assert_eq!(
        apply(&base, &delete, registry).unwrap().relations[&spec.id].num_rows(),
        0
    );
    assert!(
        changes
            .update(registry, spec, &before, &row("renamed-key", "2"))
            .is_err()
    );
}
#[test]
fn malformed_staging_unknown_keys_and_stale_bases_never_mutate_the_base() {
    let registry = pse_schema::registry().unwrap();
    let spec = registry.relation("authored.template_params").unwrap();
    let before = row("pressure", "1");
    let base = Base {
        revision: id(2),
        rows: BTreeMap::new(),
    };
    let mut changes = ChangeSet::new(header());
    changes.insert(registry, spec, &before).unwrap();
    assert_eq!(changes.staged.len(), 1);
    assert_eq!(
        apply(&base, &changes, registry).unwrap().relations[&spec.id].num_rows(),
        1
    );
    let mut extra = changes.clone();
    extra.staged.insert(
        "undeclared".to_owned(),
        extra.staged.values().next().unwrap().clone(),
    );
    assert!(apply(&base, &extra, registry).is_err());
    let mut wrong = changes.clone();
    wrong.ops[0].row_key.staged_ordinal = 1;
    assert!(apply(&base, &wrong, registry).is_err());
    let mut stale = changes.clone();
    stale.header.base_revision_id = id(9);
    assert!(apply(&base, &stale, registry).is_err());
    let mut missing = ChangeSet::new(header());
    missing.delete(registry, spec, &before).unwrap();
    assert!(apply(&base, &missing, registry).is_err());
    assert!(base.rows.is_empty());
}
