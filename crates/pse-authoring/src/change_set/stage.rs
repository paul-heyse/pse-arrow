// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact pre/post rows are staged under distinct operation-role ports.

use super::{AuthoredReader, ChangeSet, StagedMember, contract};
use crate::{AuthoringError, document::Rows};
use pse_relations::generated::{authored, enums::ChangeOpKind};
use pse_schema::{
    Registry,
    model::{Cell, RelationSpec},
};
use std::collections::BTreeMap;

impl ChangeSet {
    /// Construct an empty operation envelope from its generated header.
    pub fn new(header: authored::change_sets::Row) -> Self {
        Self {
            header,
            ops: Vec::new(),
            staged: BTreeMap::new(),
            rename_proof: None,
        }
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
        let ordinal = u32::try_from(self.ops.len()).map_err(|_| contract("too many operations"))?;
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
        self.staged.insert(key_port.clone(), key);
        if let Some(replacement) = replacement {
            self.staged.insert(row_port.clone(), replacement);
        }
        self.ops.push(authored::change_ops::Row {
            change_set_id: self.header.change_set_id,
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

/// Stage the exact difference for every explicitly supplied complete relation.
/// Unspecified base relations are retained. Equality compares actual lossless cells.
/// # Errors
/// Invalid base/source rows, duplicate keys, mismatched base or schema failures.
pub fn stage_rows(
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    rows: &Rows,
    registry: &Registry,
) -> Result<ChangeSet, AuthoringError> {
    if header.base_revision_id != base.revision_id() {
        return Err(contract("change-set base differs from reader revision"));
    }
    let prior = base.relations()?;
    let mut changes = ChangeSet::new(header);
    for (id, next) in rows {
        let spec = registry
            .relations()
            .iter()
            .find(|spec| spec.id == *id)
            .ok_or_else(|| contract("unknown staged relation"))?;
        let previous = prior
            .get(id)
            .map(|batch| pse_relations::cells::cells_from_batch(registry, spec, batch))
            .transpose()
            .map_err(|error| contract(&error.to_string()))?
            .unwrap_or_default();
        stage_relation(&mut changes, registry, spec, &previous, next)?;
    }
    Ok(changes)
}
fn stage_relation(
    changes: &mut ChangeSet,
    registry: &Registry,
    spec: &RelationSpec,
    previous: &[Vec<Cell>],
    next: &[Vec<Cell>],
) -> Result<(), AuthoringError> {
    let previous = keyed(spec, previous)?;
    let next = keyed(spec, next)?;
    for (key, before) in &previous {
        if let Some(after) = next.get(key) {
            if !super::apply::equal(before, after) {
                changes.update(registry, spec, before, after)?;
            }
        } else {
            changes.delete(registry, spec, before)?;
        }
    }
    for (key, after) in next {
        if !previous.contains_key(&key) {
            changes.insert(registry, spec, after)?;
        }
    }
    Ok(())
}
fn keyed<'a>(
    spec: &RelationSpec,
    rows: &'a [Vec<Cell>],
) -> Result<BTreeMap<Vec<String>, &'a [Cell]>, AuthoringError> {
    let mut map = BTreeMap::new();
    for row in rows {
        if map
            .insert(super::apply::key(spec, row)?, row.as_slice())
            .is_some()
        {
            return Err(contract("duplicate staged primary key"));
        }
    }
    Ok(map)
}
