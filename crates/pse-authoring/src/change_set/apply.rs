// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ordered current-format application through native joins and anti joins.

use super::{AuthoredReader, CandidateSnapshot, ChangeSet, contract, plans};
use crate::AuthoringError;
use datafusion::logical_expr::{JoinType, LogicalPlan, LogicalPlanBuilder};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{authored, enums::ChangeOpKind},
};
use pse_schema::{
    Registry,
    model::{Authority, Cell, RelationSpec},
};
use std::collections::{BTreeMap, BTreeSet};

/// Apply an external envelope in exact ordinal order. Each before-image reads the
/// result of prior operations, so repeated keys cannot collapse to last-write-wins.
/// # Errors
/// Invalid envelope, missing/colliding key, stale before-image, undeclared rename,
/// source/candidate disagreement, cancellation or native execution failure.
pub async fn apply_owned(
    base: &dyn AuthoredReader,
    changes: &ChangeSet,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<super::OwnedCandidateSnapshot, AuthoringError> {
    if base.revision_id() != changes.header.base_revision_id {
        return Err(contract("change-set base differs from reader revision"));
    }
    let registry = session.registry();
    let mut work = session.reserver().open("authoring:change-application");
    work.try_grow(crate::work::mul(super::allocation::envelope(changes)?, 4)?)?;
    validate_envelope(changes, registry)?;
    let execution = plans::session(session)?;
    let mut relations = super::base::checked(base, session)?.into_owned();
    let mut completed = Vec::new();
    for operation in &changes.ops {
        apply_operation(
            operation,
            changes,
            &mut relations,
            &execution,
            cancel,
            &mut completed,
        )
        .await?;
    }
    if let Some(source) = changes.source() {
        // This is external envelope admission, not replay of a local constructor.
        // Establish actual supplied before-images against the retained parsed output.
        for (id, expected) in &source.candidate {
            let spec = relation(registry, *id)?;
            let actual = match relations.entry(*id) {
                std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::btree_map::Entry::Vacant(entry) => {
                    // An absent reader member denotes the declared empty relation.
                    entry.insert(FieldCheckedBatch::concat(registry, spec, &[])?)
                }
            };
            let bound = plans::roles(&execution, actual.clone(), expected.clone(), cancel)?;
            if row_count(
                &bound,
                plans::difference(&bound, spec)?,
                cancel,
                &mut completed,
            )
            .await?
                != 0
            {
                return Err(contract(
                    "applied operations differ from retained parsed source output",
                ));
            }
        }
    }
    let raw = crate::p1::raw_batches(&relations);
    work.try_grow(super::allocation::candidate(&raw, changes)?)?;
    let mut retained = changes.clone();
    let prior = changes.completed.iter().cloned().collect::<Vec<_>>();
    retained.retain_completions(prior.into_iter().chain(completed).collect());
    Ok(super::OwnedCandidateSnapshot::new(
        CandidateSnapshot {
            relations: raw,
            base_revision_id: base.revision_id(),
            changes: retained,
        },
        relations,
        work,
    ))
}

async fn apply_operation(
    operation: &super::ChangeOp,
    changes: &ChangeSet,
    relations: &mut crate::document::Batches,
    execution: &SnapshotSession,
    cancel: &CancellationToken,
    completed: &mut plans::Completions,
) -> Result<(), AuthoringError> {
    let registry = execution.registry();
    cancel.checkpoint()?;
    let spec = relation(registry, operation.relation_id)?;
    if operation.op == ChangeOpKind::Rename && changes.source().is_none() {
        return Err(contract(
            "rename requires the retained identity-bound source constructor",
        ));
    }
    let current = relations
        .get(&spec.id)
        .cloned()
        .map_or_else(|| FieldCheckedBatch::concat(registry, spec, &[]), Ok)?;
    let before = staged(
        changes,
        registry,
        spec,
        &operation.row_key.staged_port,
        operation.row_key.staged_ordinal,
    )?;
    let bound = plans::roles(execution, current, before.clone(), cancel)?;
    let left = bound.scan_role("change_before")?;
    let right = bound.scan_role("change_after")?;
    let matches = plans::join(left.clone(), right.clone(), spec, JoinType::Inner)?
        .project(plans::project(spec, "before"))
        .and_then(LogicalPlanBuilder::build)
        .map_err(plans::engine)?;
    let count = row_count(&bound, matches, cancel, completed).await?;
    if operation.op == ChangeOpKind::Insert {
        if count != 0 {
            return Err(contract("insert primary key already exists"));
        }
    } else {
        if count != 1 {
            return Err(AuthoringError::UnknownRowKey {
                relation: spec.key.qualified_name(),
                row_key: format!(
                    "{}:{}",
                    operation.row_key.staged_port, operation.row_key.staged_ordinal
                ),
            });
        }
        let exact = plans::join(left.clone(), right.clone(), spec, JoinType::Inner)?
            .filter(plans::row_equal(spec))
            .and_then(|plan| plan.project(plans::project(spec, "before")))
            .and_then(LogicalPlanBuilder::build)
            .map_err(plans::engine)?;
        if row_count(&bound, exact, cancel, completed).await? != 1 {
            return Err(contract(
                "actual row differs from the complete staged before-image",
            ));
        }
    }
    let retained = if operation.op == ChangeOpKind::Insert {
        left
    } else {
        plans::join(left, right, spec, JoinType::LeftAnti)?
            .project(plans::project(spec, "before"))
            .and_then(LogicalPlanBuilder::build)
            .map_err(plans::engine)?
    };
    let output = if let Some(after) = &operation.row {
        let replacement = staged(
            changes,
            registry,
            spec,
            &after.staged_port,
            after.staged_ordinal,
        )?;
        let keys = plans::roles(execution, before, replacement.clone(), cancel)?;
        let matching = plans::join(
            keys.scan_role("change_before")?,
            keys.scan_role("change_after")?,
            spec,
            JoinType::Inner,
        )?
        .project(plans::project(spec, "before"))
        .and_then(LogicalPlanBuilder::build)
        .map_err(plans::engine)?;
        if row_count(&keys, matching, cancel, completed).await? != 1 {
            return Err(contract("replacement changes the declared primary key"));
        }
        let fork = bound.with_checked_role_inputs(
            BTreeMap::from([("replacement".to_owned(), replacement)]),
            cancel,
        )?;
        let plan = LogicalPlanBuilder::from(retained)
            .union(fork.scan_role("replacement")?)
            .and_then(LogicalPlanBuilder::build)
            .map_err(plans::engine)?;
        plans::relation(&fork, plan, spec, cancel, completed).await?
    } else {
        plans::relation(&bound, retained, spec, cancel, completed).await?
    };
    relations.insert(spec.id, output);
    Ok(())
}

async fn row_count(
    session: &SnapshotSession,
    plan: LogicalPlan,
    cancel: &CancellationToken,
    completed: &mut plans::Completions,
) -> Result<usize, AuthoringError> {
    plans::execute_recorded(session, plan, cancel, completed)
        .await?
        .iter()
        .try_fold(0_usize, |count, batch| {
            count
                .checked_add(batch.num_rows())
                .ok_or_else(|| contract("native result row count overflow"))
        })
}

pub(super) fn validate_envelope(
    changes: &ChangeSet,
    registry: &Registry,
) -> Result<(), AuthoringError> {
    let mut header = authored::change_sets::Builder::with_registry(registry, 1)?;
    header.push(changes.header.clone())?;
    header.finish()?;
    let mut operations = authored::change_ops::Builder::with_registry(registry, changes.ops.len())?;
    let mut referenced = BTreeMap::<&str, BTreeSet<i64>>::new();
    for (ordinal, operation) in changes.ops.iter().enumerate() {
        if usize::try_from(operation.ordinal).ok() != Some(ordinal)
            || operation.change_set_id != changes.header.change_set_id
        {
            return Err(contract(
                "operation identity/ordinal differs from its envelope",
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
                "before-images, not text preconditions, define current-format changes",
            ));
        }
        operations.push(operation.clone())?;
        referenced
            .entry(&operation.row_key.staged_port)
            .or_default()
            .insert(operation.row_key.staged_ordinal);
        staged(
            changes,
            registry,
            spec,
            &operation.row_key.staged_port,
            operation.row_key.staged_ordinal,
        )?;
        match (operation.op, &operation.row) {
            (ChangeOpKind::Delete, None) => {}
            (ChangeOpKind::Delete, Some(_)) | (_, None) => {
                return Err(contract("delete alone has no replacement"));
            }
            (_, Some(after)) => {
                referenced
                    .entry(&after.staged_port)
                    .or_default()
                    .insert(after.staged_ordinal);
                staged(
                    changes,
                    registry,
                    spec,
                    &after.staged_port,
                    after.staged_ordinal,
                )?;
            }
        }
    }
    operations.finish()?;
    if referenced.len() != changes.staged.len() {
        return Err(contract("unreferenced staged port"));
    }
    for (port, member) in &changes.staged {
        let indices = referenced
            .get(port.as_str())
            .ok_or_else(|| contract("staged port absent"))?;
        if indices.len() != member.batch.num_rows() {
            return Err(contract("staged batch contains unreferenced rows"));
        }
    }
    Ok(())
}
fn staged(
    changes: &ChangeSet,
    registry: &Registry,
    spec: &RelationSpec,
    port: &str,
    ordinal: i64,
) -> Result<FieldCheckedBatch, AuthoringError> {
    let member = changes
        .staged
        .get(port)
        .ok_or_else(|| contract("staged member absent"))?;
    let index = usize::try_from(ordinal).map_err(|_| contract("staged ordinal overflow"))?;
    if member.relation_id != spec.id || index >= member.batch.num_rows() {
        return Err(contract(
            "staged reference is outside its declared relation",
        ));
    }
    Ok(FieldCheckedBatch::admit(
        registry,
        spec,
        member.batch.slice(index, 1),
    )?)
}
fn relation(registry: &Registry, id: SemanticId) -> Result<&RelationSpec, AuthoringError> {
    registry
        .relation_by_id(id)
        .ok_or_else(|| contract("operation relation absent"))
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
                .position(|column| column.name() == *name)
                .map(|index| row[index].literal_spec())
                .ok_or_else(|| contract("unregistered primary key"))
        })
        .collect()
}
