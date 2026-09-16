// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact pre/post rows are staged under distinct operation-role ports.

use super::{AuthoredReader, ChangeSet, StagedMember, contract};
use crate::{AuthoringError, document::Batches};
use datafusion::arrow::array::Array;
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::columnar::FieldCheckedBatch;
use pse_relations::generated::{authored, enums::ChangeOpKind};
use pse_schema::{
    Registry,
    model::{Cell, RelationSpec},
};
use std::collections::BTreeMap;

impl ChangeSet {
    /// Construct an empty operation envelope from its generated header.
    pub fn new(header: authored::change_sets::Row) -> Self {
        Self::from_parts(header, Vec::new(), BTreeMap::new())
    }
    /// Stage one insert whose key and payload share the same admitted row.
    /// # Errors
    /// Invalid schema, values or operation count.
    pub fn insert(
        &mut self,
        registry: &Registry,
        relation: &RelationSpec,
        row: &[Cell],
    ) -> Result<(), AuthoringError> {
        self.push(registry, relation, ChangeOpKind::Insert, row, Some(row))
    }
    /// Stage an exact base preimage and a replacement with the same declared key.
    /// # Errors
    /// Invalid values, changed key or operation count.
    pub fn update(
        &mut self,
        registry: &Registry,
        relation: &RelationSpec,
        before: &[Cell],
        after: &[Cell],
    ) -> Result<(), AuthoringError> {
        self.push(
            registry,
            relation,
            ChangeOpKind::Update,
            before,
            Some(after),
        )
    }
    /// Stage the complete expected base row; delete has no replacement.
    /// # Errors
    /// Invalid schema, values or operation count.
    pub fn delete(
        &mut self,
        registry: &Registry,
        relation: &RelationSpec,
        before: &[Cell],
    ) -> Result<(), AuthoringError> {
        self.push(registry, relation, ChangeOpKind::Delete, before, None)
    }
    fn push(
        &mut self,
        registry: &Registry,
        relation: &RelationSpec,
        op: ChangeOpKind,
        before: &[Cell],
        after: Option<&[Cell]>,
    ) -> Result<(), AuthoringError> {
        let ordinal = i64::try_from(self.ops.len()).map_err(|_| contract("too many operations"))?;
        let key_port = format!("operation/{ordinal}/key");
        let row_port = if op == ChangeOpKind::Insert {
            key_port.clone()
        } else {
            format!("operation/{ordinal}/row")
        };
        let key = member(registry, relation, before)?;
        let replacement = after
            .map(|row| member(registry, relation, row))
            .transpose()?;
        if let Some(after) = after
            && op == ChangeOpKind::Update
            && super::apply::key(relation, before)? != super::apply::key(relation, after)?
        {
            return Err(contract(
                "update cannot change a primary key; use delete plus insert",
            ));
        }
        self.data_mut().staged.insert(key_port.clone(), key);
        if let Some(replacement) = replacement {
            self.data_mut().staged.insert(row_port.clone(), replacement);
        }
        let change_set_id = self.header.change_set_id;
        self.data_mut().ops.push(authored::change_ops::Row {
            change_set_id,
            ordinal,
            op,
            relation_id: relation.id,
            row_key: authored::change_ops::AuthoredChangeOpsFieldRowKey {
                staged_port: key_port,
                staged_ordinal: 0,
            },
            row: after.map(|_| authored::change_ops::AuthoredChangeOpsFieldRow {
                staged_port: row_port,
                staged_ordinal: 0,
            }),
            precondition: None,
        });
        Ok(())
    }
}
fn member(
    registry: &Registry,
    relation: &RelationSpec,
    row: &[Cell],
) -> Result<StagedMember, AuthoringError> {
    Ok(StagedMember {
        relation_id: relation.id,
        batch: pse_relations::cells::batch_from_cells(registry, relation, &[row.to_vec()])
            .map_err(|error| contract(&error.to_string()))?,
    })
}

/// Stage native full outer differences for explicitly supplied complete relations.
/// Operation pre/post ports are whole columnar batches with exact row ordinals.
/// Unspecified relations remain part of the unchanged base.
/// # Errors
/// Duplicate keys, invalid source fields, stale base, cancellation or native execution.
pub async fn stage_batches(
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    batches: &Batches,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<super::OwnedChangeSet, AuthoringError> {
    let prior = super::base::checked(base, session)?;
    stage_checked(base.revision_id(), header, batches, &prior, session, cancel).await
}
pub(crate) async fn stage_checked(
    base_revision: pse_ids::SemanticId,
    header: authored::change_sets::Row,
    batches: &Batches,
    prior: &Batches,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<super::OwnedChangeSet, AuthoringError> {
    if header.base_revision_id != base_revision {
        return Err(contract("change-set base differs from reader revision"));
    }
    let registry = session.registry();
    let execution = super::plans::session(session)?;
    let mut work = session.reserver().open("authoring:change-envelope");
    work.try_grow(crate::work::mul(batches.len(), 4096)?)?;
    let mut changes = ChangeSet::new(header);
    let mut completed = Vec::new();
    for (id, next) in batches {
        cancel.checkpoint()?;
        let spec = registry
            .relation_by_id(*id)
            .ok_or_else(|| contract("unknown staged relation"))?;
        let before = prior
            .get(id)
            .cloned()
            .map_or_else(|| FieldCheckedBatch::concat(registry, spec, &[]), Ok)?;
        if before.batch().num_rows() == 0 && next.batch().num_rows() == 0 {
            continue;
        }
        let bound = super::plans::roles(&execution, before, next.clone(), cancel)?;
        for role in ["change_before", "change_after"] {
            completed.push(unique(&bound, role, spec, cancel).await?);
        }
        let difference = super::plans::difference(&bound, spec)?;
        let delta_batches =
            super::plans::execute_recorded(&bound, difference, cancel, &mut completed).await?;
        for (partition, batch) in delta_batches.iter().enumerate() {
            work.try_grow(crate::work::mul(batch.num_rows(), 1024)?)?;
            append(&mut changes, &bound, spec, partition, batch, cancel)?;
        }
    }
    cancel.checkpoint()?;
    changes.retain_completions(completed);
    Ok(super::OwnedChangeSet::new(changes, work))
}

pub(crate) async fn unique(
    session: &SnapshotSession,
    role: &str,
    spec: &RelationSpec,
    cancel: &CancellationToken,
) -> Result<std::sync::Arc<pse_catalog::session::CompletedComputation>, AuthoringError> {
    use datafusion::logical_expr::{LogicalPlanBuilder, col, lit};
    let keys = spec
        .primary_key
        .iter()
        .map(|name| col(*name))
        .collect::<Vec<_>>();
    if keys.is_empty() {
        return Err(contract("change relation has no primary key"));
    }
    let plan = LogicalPlanBuilder::from(session.scan_role(role)?)
        .aggregate(
            keys,
            vec![
                datafusion::functions_aggregate::expr_fn::count(lit(1_i64)).alias("__multiplicity"),
            ],
        )
        .and_then(|plan| plan.filter(col("__multiplicity").gt(lit(1_i64))))
        .and_then(|plan| plan.limit(0, Some(1)))
        .and_then(LogicalPlanBuilder::build)
        .map_err(super::plans::engine)?;
    let completed = std::sync::Arc::new(
        session
            .prepare_rule_plan(plan, cancel)?
            .execute(cancel)
            .await?,
    );
    if completed
        .batches()
        .iter()
        .any(|batch| batch.num_rows() != 0)
    {
        return Err(contract("duplicate primary key in change input"));
    }
    Ok(completed)
}

fn append(
    changes: &mut ChangeSet,
    session: &SnapshotSession,
    spec: &RelationSpec,
    partition: usize,
    batch: &datafusion::arrow::array::RecordBatch,
    cancel: &CancellationToken,
) -> Result<(), AuthoringError> {
    let before = super::plans::presence(batch, 0)?;
    let after = super::plans::presence(batch, 1)?;
    let prefix = format!("relation/{}/{partition}", spec.id);
    let before_port = format!("{prefix}/before");
    let after_port = format!("{prefix}/after");
    let before_rows = super::plans::side(batch, before, 2, spec, session, cancel)?;
    let after_rows =
        super::plans::side(batch, after, 2 + spec.columns.len(), spec, session, cancel)?;
    if before_rows.batch().num_rows() != 0 {
        changes.data_mut().staged.insert(
            before_port.clone(),
            StagedMember {
                relation_id: spec.id,
                batch: before_rows.into_batch(),
            },
        );
    }
    if after_rows.batch().num_rows() != 0 {
        changes.data_mut().staged.insert(
            after_port.clone(),
            StagedMember {
                relation_id: spec.id,
                batch: after_rows.into_batch(),
            },
        );
    }
    let mut before_ordinal = 0;
    let mut after_ordinal = 0;
    for row in 0..batch.num_rows() {
        let has_before = before.is_valid(row) && before.value(row);
        let has_after = after.is_valid(row) && after.value(row);
        let op = match (has_before, has_after) {
            (true, true) => ChangeOpKind::Update,
            (true, false) => ChangeOpKind::Delete,
            (false, true) => ChangeOpKind::Insert,
            (false, false) => return Err(contract("native difference has no source side")),
        };
        let ordinal =
            i64::try_from(changes.ops.len()).map_err(|_| contract("operation ordinal overflow"))?;
        let change_set_id = changes.header.change_set_id;
        changes.data_mut().ops.push(authored::change_ops::Row {
            change_set_id,
            ordinal,
            op,
            relation_id: spec.id,
            row_key: authored::change_ops::AuthoredChangeOpsFieldRowKey {
                staged_port: if has_before {
                    before_port.clone()
                } else {
                    after_port.clone()
                },
                staged_ordinal: if has_before {
                    before_ordinal
                } else {
                    after_ordinal
                },
            },
            row: has_after.then(|| authored::change_ops::AuthoredChangeOpsFieldRow {
                staged_port: after_port.clone(),
                staged_ordinal: after_ordinal,
            }),
            precondition: None,
        });
        before_ordinal += i64::from(has_before);
        after_ordinal += i64::from(has_after);
    }
    Ok(())
}
