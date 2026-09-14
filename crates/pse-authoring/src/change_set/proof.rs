// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Private exact-input evidence for a fully checked rename, never a digest certificate.

use super::{AuthoredReader, ChangeSet, contract};
use crate::{
    AuthoringError,
    document::{DocumentEdit, Rows},
};
use pse_ids::SemanticId;
use pse_schema::{Registry, model::Cell};
use std::collections::BTreeMap;

type StagedRows = BTreeMap<String, (SemanticId, Vec<Vec<Cell>>)>;

#[derive(Clone, Debug)]
pub(super) struct RenameProof {
    base: Rows,
    registry: Vec<(pse_schema::model::RelationKey, Vec<Vec<Cell>>)>,
    documents: BTreeMap<SemanticId, String>,
    header: Vec<Cell>,
    ops: Vec<Vec<Cell>>,
    staged: StagedRows,
    pub(super) edits: Vec<DocumentEdit>,
}

pub(crate) fn rows(base: &dyn AuthoredReader, registry: &Registry) -> Result<Rows, AuthoringError> {
    base.relations()?
        .into_iter()
        .map(|(id, batch)| {
            let spec = registry
                .relations()
                .iter()
                .find(|relation| relation.id == id)
                .ok_or_else(|| contract("unknown base relation"))?;
            Ok((
                id,
                pse_relations::cells::cells_from_batch(registry, spec, &batch)
                    .map_err(|error| contract(&error.to_string()))?,
            ))
        })
        .collect()
}
fn staged(changes: &ChangeSet, registry: &Registry) -> Result<StagedRows, AuthoringError> {
    changes
        .staged
        .iter()
        .map(|(port, member)| {
            let spec = registry
                .relations()
                .iter()
                .find(|relation| relation.id == member.relation_id)
                .ok_or_else(|| contract("unknown staged relation"))?;
            Ok((
                port.clone(),
                (
                    member.relation_id,
                    pse_relations::cells::cells_from_batch(registry, spec, &member.batch)
                        .map_err(|error| contract(&error.to_string()))?,
                ),
            ))
        })
        .collect()
}

pub(crate) fn capture(
    changes: &mut ChangeSet,
    base: &dyn AuthoredReader,
    registry: &Registry,
    edits: Vec<DocumentEdit>,
) -> Result<(), AuthoringError> {
    changes.rename_proof = Some(RenameProof {
        base: rows(base, registry)?,
        registry: registry.schema_rows(),
        documents: base.source_documents()?,
        header: changes.header.clone().into_cells(),
        ops: changes
            .ops
            .iter()
            .cloned()
            .map(super::ChangeOp::into_cells)
            .collect(),
        staged: staged(changes, registry)?,
        edits,
    });
    Ok(())
}

pub(crate) fn verify(
    changes: &ChangeSet,
    base: &dyn AuthoredReader,
    registry: &Registry,
) -> Result<bool, AuthoringError> {
    let Some(proof) = &changes.rename_proof else {
        return Ok(false);
    };
    let current_ops = changes
        .ops
        .iter()
        .cloned()
        .map(super::ChangeOp::into_cells)
        .collect::<Vec<_>>();
    let current_staged = staged(changes, registry)?;
    let current_base = rows(base, registry)?;
    let same_base = proof.base.len() == current_base.len()
        && proof.base.iter().all(|(id, rows)| {
            current_base
                .get(id)
                .is_some_and(|current| equal_rows(rows, current))
        });
    let same_staged = proof.staged.len() == current_staged.len()
        && proof.staged.iter().all(|(port, (id, rows))| {
            current_staged
                .get(port)
                .is_some_and(|(current_id, current)| id == current_id && equal_rows(rows, current))
        });
    let current_registry = registry.schema_rows_ref();
    let same_registry = proof.registry.len() == current_registry.len()
        && proof.registry.iter().zip(current_registry).all(
            |((key, rows), (current_key, current_rows))| {
                key == current_key && equal_rows(rows, current_rows)
            },
        );
    if !same_registry
        || !same_base
        || !same_staged
        || proof.documents != base.source_documents()?
        || !super::apply::equal(&proof.header, &changes.header.clone().into_cells())
        || !equal_rows(&proof.ops, &current_ops)
    {
        return Err(contract(
            "rename's complete actual base, source bytes or operation envelope changed after semantic validation",
        ));
    }
    Ok(true)
}
fn equal_rows(first: &[Vec<Cell>], second: &[Vec<Cell>]) -> bool {
    first.len() == second.len()
        && first
            .iter()
            .zip(second)
            .all(|(first, second)| super::apply::equal(first, second))
}

/// Allocation bound for the complete retained proof clone. No decoded base,
/// expanded validation cells, or temporary comparison vectors survive in it.
pub(super) fn clone_extent(changes: &ChangeSet) -> Result<usize, AuthoringError> {
    use super::allocation::{cells, map, rows};
    use crate::work::{add, mul};
    let Some(proof) = &changes.rename_proof else {
        return Ok(0);
    };
    let mut size = map::<SemanticId, Vec<Vec<Cell>>>(proof.base.len())?;
    for value in proof.base.values() {
        size = add(size, rows(value)?)?;
    }
    size = add(
        size,
        mul(
            proof.registry.len(),
            size_of::<(pse_schema::model::RelationKey, Vec<Vec<Cell>>)>(),
        )?,
    )?;
    for (_, value) in &proof.registry {
        size = add(size, rows(value)?)?;
    }
    size = add(size, map::<SemanticId, String>(proof.documents.len())?)?;
    for text in proof.documents.values() {
        size = add(size, text.len())?;
    }
    size = add(size, add(cells(&proof.header)?, rows(&proof.ops)?)?)?;
    size = add(
        size,
        map::<String, (SemanticId, Vec<Vec<Cell>>)>(proof.staged.len())?,
    )?;
    for (port, (_, value)) in &proof.staged {
        size = add(size, add(port.len(), rows(value)?)?)?;
    }
    size = add(size, mul(proof.edits.len(), size_of::<DocumentEdit>())?)?;
    for edit in &proof.edits {
        size = add(
            size,
            add(edit.path.len(), add(edit.before.len(), edit.after.len())?)?,
        )?;
    }
    Ok(size)
}

/// Extra copies needed by actual proof comparison and the retained candidate copy.
pub(crate) fn extent(
    changes: &ChangeSet,
    base: &dyn AuthoredReader,
    registry: &Registry,
) -> Result<usize, AuthoringError> {
    let Some(proof) = &changes.rename_proof else {
        return Ok(0);
    };
    let mut size = crate::work::rows(&proof.base)?;
    for (_, rows) in registry.schema_rows_ref() {
        for row in rows {
            for value in row {
                size = crate::work::add(size, crate::work::cell(value)?)?;
            }
        }
    }
    for text in proof.documents.values() {
        size = crate::work::add(size, text.len())?;
    }
    for edit in &proof.edits {
        size = crate::work::add(size, crate::work::add(edit.before.len(), edit.after.len())?)?;
    }
    for (_, batch) in base.relations()? {
        size = crate::work::add(size, pse_ids::validation_extent(&batch)?)?;
    }
    crate::work::mul(size, 4)
}
