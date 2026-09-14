// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Atomic in-memory application compares exact preimages before candidate construction.

use super::{AuthoredReader, CandidateSnapshot, ChangeSet, contract};
use crate::AuthoringError;
use pse_ids::SemanticId;
use pse_relations::generated::{
    authored,
    enums::{ChangeOpKind, IdPolicy},
};
use pse_schema::{
    Registry,
    model::{Authority, Cell, RelationSpec},
};
use std::collections::{BTreeMap, BTreeSet};

/// Validate the entire envelope and apply ordered operations to an unpublished copy.
///
/// # Errors
/// Rejects stale bases, unknown/derived writes, malformed staging, absent/duplicate keys,
/// unequal actual preimages, key-changing updates, and rename without a complete inventory.
pub fn apply(
    base: &dyn AuthoredReader,
    changes: &ChangeSet,
    registry: &Registry,
) -> Result<CandidateSnapshot, AuthoringError> {
    if base.revision_id() != changes.header.base_revision_id {
        return Err(contract(
            "change-set base differs from exact reader revision",
        ));
    }
    validate_envelope(changes, registry)?;
    let verified_rename = super::proof::verify(changes, base, registry)?;
    let batches = base.relations()?;
    let mut rows = BTreeMap::new();
    for (id, batch) in &batches {
        let spec = relation(registry, *id)?;
        rows.insert(
            *id,
            pse_relations::cells::cells_from_batch(registry, spec, batch)
                .map_err(|error| contract(&error.to_string()))?,
        );
    }
    for operation in &changes.ops {
        let spec = relation(registry, operation.relation_id)?;
        let before = staged(
            changes,
            registry,
            spec,
            &operation.row_key.staged_port,
            operation.row_key.staged_ordinal,
        )?;
        let after = operation
            .row
            .as_ref()
            .map(|row| {
                staged(
                    changes,
                    registry,
                    spec,
                    &row.staged_port,
                    row.staged_ordinal,
                )
            })
            .transpose()?;
        if operation.op == ChangeOpKind::Rename && !verified_rename {
            return rename_refusal(&rows, &before);
        }
        apply_row(
            rows.entry(spec.id).or_default(),
            spec,
            operation.op,
            &before,
            after.as_deref(),
        )?;
    }
    let relations = rows
        .into_iter()
        .map(|(id, rows)| {
            let batch =
                pse_relations::cells::batch_from_cells(registry, relation(registry, id)?, &rows)
                    .map_err(|error| contract(&error.to_string()))?;
            Ok((id, batch))
        })
        .collect::<Result<_, AuthoringError>>()?;
    Ok(CandidateSnapshot {
        relations,
        base_revision_id: base.revision_id(),
        changes: changes.clone(),
    })
}

/// Apply exact operations while sharing unchanged admitted buffers and owning new ones.
/// # Errors
/// The same preimage/proof failures as `apply`, cancellation or shared reservation refusal.
pub fn apply_owned(
    base: &dyn AuthoredReader,
    changes: &ChangeSet,
    registry: &Registry,
    reserver: &dyn pse_ids::MemoryReserver,
    cancel: &pse_ids::CancellationToken,
) -> Result<super::OwnedCandidateSnapshot, AuthoringError> {
    cancel.checkpoint()?;
    if base.revision_id() != changes.header.base_revision_id {
        return Err(contract(
            "change-set base differs from exact reader revision",
        ));
    }
    let mut work = reserver.open("authoring:apply");
    work.try_grow(8192)?;
    for staged in changes.staged.values() {
        work.try_grow(pse_ids::validation_extent(&staged.batch)?)?;
    }
    work.try_grow(crate::work::mul(changes.ops.len(), 8192)?)?;
    work.try_grow(crate::work::mul(registry.relations().len(), 2048)?)?;
    // Header/operation DTO clones, Cells and validation builders coexist. Their
    // variable-size strings are additional to the fixed per-operation slots.
    work.try_grow(crate::work::mul(super::allocation::envelope(changes)?, 8)?)?;
    cancel.checkpoint()?;
    validate_envelope(changes, registry)?;
    let verified_rename = verify_owned(changes, base, registry, reserver, cancel)?;
    let mut relations = base.relations()?;
    let mut rows = BTreeMap::new();
    for operation in &changes.ops {
        cancel.checkpoint()?;
        let spec = relation(registry, operation.relation_id)?;
        if let std::collections::btree_map::Entry::Vacant(entry) = rows.entry(spec.id) {
            let decoded = if let Some(batch) = relations.get(&spec.id) {
                work.try_grow(pse_ids::validation_extent(batch)?)?;
                pse_relations::cells::cells_from_batch(registry, spec, batch)
                    .map_err(|error| contract(&error.to_string()))?
            } else {
                Vec::new()
            };
            entry.insert(decoded);
        }
        let before = staged(
            changes,
            registry,
            spec,
            &operation.row_key.staged_port,
            operation.row_key.staged_ordinal,
        )?;
        let after = operation
            .row
            .as_ref()
            .map(|row| {
                staged(
                    changes,
                    registry,
                    spec,
                    &row.staged_port,
                    row.staged_ordinal,
                )
            })
            .transpose()?;
        if operation.op == ChangeOpKind::Rename && !verified_rename {
            return rename_refusal(&rows, &before);
        }
        apply_row(
            rows.entry(spec.id).or_default(),
            spec,
            operation.op,
            &before,
            after.as_deref(),
        )?;
    }
    for (id, values) in rows {
        cancel.checkpoint()?;
        let batch = pse_relations::cells::batch_from_cells_owned(
            registry,
            relation(registry, id)?,
            &values,
            reserver,
            cancel,
        )
        .map_err(|error| contract(&error.to_string()))?;
        relations.insert(id, batch);
    }
    retain_candidate(relations, base.revision_id(), changes, work, cancel)
}

fn retain_candidate(
    relations: BTreeMap<SemanticId, arrow_array::RecordBatch>,
    base_revision_id: SemanticId,
    changes: &ChangeSet,
    mut work: Box<dyn pse_ids::Reservation>,
    cancel: &pse_ids::CancellationToken,
) -> Result<super::OwnedCandidateSnapshot, AuthoringError> {
    // All decoded operation/base rows have been consumed and dropped. New Arrow
    // arrays retain their own construction leases; unchanged arrays retain their
    // original owners. Only metadata and the complete proof clone remain here.
    let retained = super::allocation::candidate(&relations, changes)?;
    if retained > work.size() {
        work.try_grow(retained - work.size())?;
    }
    cancel.checkpoint()?;
    let candidate = CandidateSnapshot {
        relations,
        base_revision_id,
        changes: changes.clone(),
    };
    work.shrink(work.size() - retained);
    cancel.checkpoint()?;
    Ok(super::OwnedCandidateSnapshot::new(candidate, work))
}

fn verify_owned(
    changes: &ChangeSet,
    base: &dyn AuthoredReader,
    registry: &Registry,
    reserver: &dyn pse_ids::MemoryReserver,
    cancel: &pse_ids::CancellationToken,
) -> Result<bool, AuthoringError> {
    let mut proof = reserver.open("authoring:apply-proof");
    proof.try_grow(super::proof::extent(changes, base, registry)?)?;
    cancel.checkpoint()?;
    // Verification allocates decoded copies solely for actual-value comparison.
    // Its reservation ends with those copies, before any retained candidate clone.
    let verified = super::proof::verify(changes, base, registry)?;
    cancel.checkpoint()?;
    Ok(verified)
}

fn validate_envelope(changes: &ChangeSet, registry: &Registry) -> Result<(), AuthoringError> {
    for (name, rows) in [
        (
            "authored.change_sets",
            vec![changes.header.clone().into_cells()],
        ),
        (
            "authored.change_ops",
            changes
                .ops
                .iter()
                .cloned()
                .map(authored::change_ops::Row::into_cells)
                .collect(),
        ),
    ] {
        let spec = registry
            .relation(name)
            .ok_or_else(|| contract("missing change-set schema"))?;
        pse_relations::cells::batch_from_cells(registry, spec, &rows)
            .map_err(|error| contract(&error.to_string()))?;
    }
    let mut expected = BTreeSet::new();
    for (ordinal, operation) in changes.ops.iter().enumerate() {
        if usize::try_from(operation.ordinal).ok() != Some(ordinal)
            || operation.change_set_id != changes.header.change_set_id
        {
            return Err(contract(
                "operation identity or ordinal differs from its envelope",
            ));
        }
        let spec = relation(registry, operation.relation_id)?;
        if spec.authority == Authority::Derived {
            return Err(AuthoringError::DerivedWrite {
                relation: spec.key.qualified_name(),
            });
        }
        if operation.precondition.is_some() {
            return Err(contract(
                "text preconditions are unsupported; the complete staged preimage is required",
            ));
        }
        let key = format!("operation/{ordinal}/key");
        if operation.row_key.staged_port != key || operation.row_key.staged_ordinal != 0 {
            return Err(contract("invalid operation key port"));
        }
        expected.insert(key.clone());
        match (operation.op, &operation.row) {
            (ChangeOpKind::Delete, None) => {}
            (ChangeOpKind::Delete, Some(_)) | (_, None) => {
                return Err(contract(
                    "delete has no replacement; every other operation requires one",
                ));
            }
            (_, Some(row)) => {
                let port = if operation.op == ChangeOpKind::Insert {
                    key
                } else {
                    format!("operation/{ordinal}/row")
                };
                if row.staged_port != port || row.staged_ordinal != 0 {
                    return Err(contract("invalid operation replacement port"));
                }
                expected.insert(port);
            }
        }
    }
    if expected != changes.staged.keys().cloned().collect() {
        return Err(contract(
            "staging inventory differs from the complete expected operation ports",
        ));
    }
    Ok(())
}

fn staged(
    changes: &ChangeSet,
    registry: &Registry,
    spec: &RelationSpec,
    port: &str,
    ordinal: u64,
) -> Result<Vec<Cell>, AuthoringError> {
    let member = changes
        .staged
        .get(port)
        .ok_or_else(|| contract("missing staged operation member"))?;
    if member.relation_id != spec.id || ordinal != 0 || member.batch.num_rows() != 1 {
        return Err(contract(
            "staged reference must name one row of the operation's exact relation",
        ));
    }
    pse_relations::cells::cells_from_batch(registry, spec, &member.batch)
        .map_err(|error| contract(&error.to_string()))?
        .pop()
        .ok_or_else(|| contract("empty staged member"))
}

fn apply_row(
    rows: &mut Vec<Vec<Cell>>,
    spec: &RelationSpec,
    op: ChangeOpKind,
    before: &[Cell],
    after: Option<&[Cell]>,
) -> Result<(), AuthoringError> {
    let target = key(spec, before)?;
    let positions = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| match key(spec, row) {
            Ok(key) if key == target => Some(Ok(index)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if op == ChangeOpKind::Insert {
        if !positions.is_empty() {
            return Err(contract("insert primary key already exists"));
        }
        rows.push(
            after
                .ok_or_else(|| contract("insert replacement absent"))?
                .to_vec(),
        );
        return Ok(());
    }
    let [position] = positions.as_slice() else {
        return Err(AuthoringError::UnknownRowKey {
            relation: spec.key.qualified_name(),
            row_key: target.join(","),
        });
    };
    if !equal(&rows[*position], before) {
        return Err(contract(
            "actual base row differs from the complete staged preimage",
        ));
    }
    if op == ChangeOpKind::Delete {
        rows.remove(*position);
    } else {
        let after = after.ok_or_else(|| contract("update replacement absent"))?;
        if key(spec, after)? != target {
            return Err(contract(
                "update changes primary key; use delete plus insert",
            ));
        }
        rows[*position] = after.to_vec();
    }
    Ok(())
}

fn relation(registry: &Registry, id: SemanticId) -> Result<&RelationSpec, AuthoringError> {
    registry
        .relations()
        .iter()
        .find(|relation| relation.id == id)
        .ok_or_else(|| contract("operation references an unknown relation"))
}
/// Lossless value comparison includes signed zero and every nested field.
pub(super) fn equal(first: &[Cell], second: &[Cell]) -> bool {
    first.len() == second.len()
        && first
            .iter()
            .zip(second)
            .all(|(first, second)| first.literal_spec() == second.literal_spec())
}
pub(super) fn key(spec: &RelationSpec, row: &[Cell]) -> Result<Vec<String>, AuthoringError> {
    if row.len() != spec.columns.len() {
        return Err(contract("row width differs from declared columns"));
    }
    spec.primary_key
        .iter()
        .map(|name| {
            spec.columns
                .iter()
                .position(|column| column.name == *name)
                .map(|index| row[index].literal_spec())
                .ok_or_else(|| contract("unregistered primary key"))
        })
        .collect()
}

fn rename_refusal<T>(
    rows: &BTreeMap<SemanticId, Vec<Vec<Cell>>>,
    before: &[Cell],
) -> Result<T, AuthoringError> {
    let entity = authored::entities::Row::from_cells(before.to_vec())
        .map_err(|error| contract(&error.to_string()))?;
    let packages = rows
        .get(&authored::packages::RELATION_ID)
        .ok_or_else(|| contract("rename requires package policy"))?;
    for row in packages {
        let package = authored::packages::Row::from_cells(row.clone())
            .map_err(|error| contract(&error.to_string()))?;
        if package.package_id == entity.package_id && package.id_policy == IdPolicy::Named {
            return Err(AuthoringError::RenameNamed {
                entity_id: entity.entity_id,
                qualified_name: entity.qualified_name,
            });
        }
    }
    Err(contract(
        "rename requires complete identity-bound document and expression inventory",
    ))
}
