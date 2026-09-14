// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Claimed source artifacts must reproduce their actual authored row projections.
use crate::{
    CompilerError,
    driver::{BaseReader, inputs},
    passes::dag::invalid,
};
use pse_catalog::{Catalog, store::membership::AdmissionContext};
use pse_ids::{CancellationToken, SemanticId, SnapshotKind};
use pse_relations::{RecordBatch, generated::authored};
use pse_schema::model::{RelationKey, SnapshotClass};
use std::collections::BTreeMap;

pub(super) async fn validate(
    catalog: &Catalog,
    kind: SnapshotKind,
    context: &AdmissionContext,
    candidates: &BTreeMap<RelationKey, RecordBatch>,
    cancel: &CancellationToken,
) -> Result<(), CompilerError> {
    let registry = catalog.registry();
    let mut rows = candidates.clone();
    for parent in context.parents.values() {
        for (key, input) in inputs::inventory(parent, registry)? {
            if let Some(existing) = rows.get(&key) {
                if existing != input.relation().batch() {
                    return Err(invalid("source admission has ambiguous actual parent rows"));
                }
            } else {
                rows.insert(key, input.relation().batch().clone());
            }
        }
    }
    let documents = inputs::documents(catalog, &rows, cancel).await?;
    pse_authoring::p0::resolve(documents.bundles(), registry)?;
    let base = BaseReader {
        revision: SemanticId::NIL,
        rows: inputs::primitive_rows(&rows, registry),
        documents: documents.clone(),
    };
    let changes = pse_authoring::p1::stage_owned(
        &documents,
        &base,
        authored::change_sets::Row {
            change_set_id: pse_authoring::ids::uuid_v7(),
            base_revision_id: SemanticId::NIL,
            author: "source-admission".to_owned(),
            message: "compare exact source projection".to_owned(),
            created_at: 0,
        },
        registry,
        catalog.reserver().as_ref(),
        cancel,
    )?;
    let class = match kind {
        SnapshotKind::Model => SnapshotClass::Model,
        SnapshotKind::Case => SnapshotClass::Case,
        _ => {
            return Err(invalid(
                "source projection admission requires Model or Case",
            ));
        }
    };
    for operation in &changes.ops {
        let relation = registry
            .relation_by_id(operation.relation_id)
            .ok_or_else(|| invalid("source projection relation is undeclared"))?;
        if relation.snapshot_class == class {
            return Err(invalid(format!(
                "actual {} rows differ from exact source projection",
                relation.key
            )));
        }
    }
    cancel.checkpoint()?;
    Ok(())
}
