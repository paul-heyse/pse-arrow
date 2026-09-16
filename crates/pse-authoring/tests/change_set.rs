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
    change_set::{AuthoredReader, ChangeSet, apply_owned},
};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
mod support;
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
#[tokio::test]
async fn composite_text_key_updates_and_deletes_use_complete_actual_preimages() {
    let registry_owner = support::registry();
    let registry = registry_owner.as_ref();
    let session = support::session(
        std::sync::Arc::clone(&registry_owner),
        FixedBudget::new(512 << 20),
    );
    let cancel = CancellationToken::new();
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
    let candidate = apply_owned(&base, &changes, &session, &cancel)
        .await
        .unwrap();
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
    assert!(apply_owned(&base, &wrong, &session, &cancel).await.is_err());
    let mut delete = ChangeSet::new(header());
    delete.delete(registry, spec, &before).unwrap();
    assert!(delete.ops[0].row.is_none());
    assert_eq!(
        apply_owned(&base, &delete, &session, &cancel)
            .await
            .unwrap()
            .relations[&spec.id]
            .num_rows(),
        0
    );
    assert!(
        changes
            .update(registry, spec, &before, &row("renamed-key", "2"))
            .is_err()
    );
}
#[tokio::test]
async fn malformed_staging_unknown_keys_and_stale_bases_never_mutate_the_base() {
    let registry_owner = support::registry();
    let registry = registry_owner.as_ref();
    let session = support::session(
        std::sync::Arc::clone(&registry_owner),
        FixedBudget::new(512 << 20),
    );
    let cancel = CancellationToken::new();
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
        apply_owned(&base, &changes, &session, &cancel)
            .await
            .unwrap()
            .relations[&spec.id]
            .num_rows(),
        1
    );
    let mut extra = changes.staged.clone();
    extra.insert(
        "undeclared".to_owned(),
        extra.values().next().unwrap().clone(),
    );
    assert!(
        ChangeSet::from_staged(changes.header.clone(), changes.ops.clone(), extra, registry)
            .is_err()
    );
    let mut wrong = changes.ops.clone();
    wrong[0].row_key.staged_ordinal = 1;
    assert!(
        ChangeSet::from_staged(
            changes.header.clone(),
            wrong,
            changes.staged.clone(),
            registry
        )
        .is_err()
    );
    let mut stale = changes.header.clone();
    stale.base_revision_id = id(9);
    let stale =
        ChangeSet::from_staged(stale, changes.ops.clone(), changes.staged.clone(), registry)
            .unwrap();
    assert!(apply_owned(&base, &stale, &session, &cancel).await.is_err());
    let mut missing = ChangeSet::new(header());
    missing.delete(registry, spec, &before).unwrap();
    assert!(
        apply_owned(&base, &missing, &session, &cancel)
            .await
            .is_err()
    );
    assert!(base.rows.is_empty());
}

#[tokio::test]
async fn repeated_key_operations_follow_explicit_ordinal_preimages() {
    let registry_owner = support::registry();
    let registry = registry_owner.as_ref();
    let session = support::session(
        std::sync::Arc::clone(&registry_owner),
        FixedBudget::new(512 << 20),
    );
    let cancel = CancellationToken::new();
    let spec = registry.relation("authored.template_params").unwrap();
    let before = row("pressure", "1");
    let middle = row("pressure", "2");
    let after = row("pressure", "3");
    let base = Base {
        revision: id(2),
        rows: BTreeMap::from([(
            spec.id,
            pse_relations::cells::batch_from_cells(registry, spec, std::slice::from_ref(&before))
                .unwrap(),
        )]),
    };
    let mut changes = ChangeSet::new(header());
    changes.update(registry, spec, &before, &middle).unwrap();
    changes.update(registry, spec, &middle, &after).unwrap();
    let candidate = apply_owned(&base, &changes, &session, &cancel)
        .await
        .unwrap();
    assert_eq!(
        pse_relations::cells::cells_from_batch(registry, spec, &candidate.relations[&spec.id])
            .unwrap(),
        vec![after.clone()]
    );
    let mut stale = ChangeSet::new(header());
    stale.update(registry, spec, &before, &middle).unwrap();
    stale.update(registry, spec, &before, &after).unwrap();
    assert!(apply_owned(&base, &stale, &session, &cancel).await.is_err());
}
