// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Executes explicitly declared schema migrations, admitting both endpoints.

use crate::RelationError;
use arrow_array::RecordBatch;
use pse_schema::Registry;
use pse_schema::model::{Cell, MigrationSpec, MigrationStep, RelationSpec};

/// Executes one migration declared by `reg`. The source schema and values are admitted
/// before transformation and the target is rebuilt under its exact declaration.
///
/// # Errors
/// Unknown migrations, invalid source batches, inconsistent steps or invalid target values.
pub fn migrate(
    reg: &Registry,
    plan: &MigrationSpec,
    batch: &RecordBatch,
) -> Result<RecordBatch, RelationError> {
    if !reg.migrations().contains(plan) {
        return Err(error(
            plan,
            "migration is not declared in the bound registry",
        ));
    }
    let find = |version| {
        reg.relations()
            .iter()
            .find(|spec| spec.qualified_name() == plan.relation && spec.key.version == version)
    };
    let source =
        find(plan.from_version).ok_or_else(|| error(plan, "source version is not declared"))?;
    let target =
        find(plan.to_version).ok_or_else(|| error(plan, "target version is not declared"))?;
    let mut rows = crate::cells::cells_from_batch(reg, source, batch)?;
    let mut columns = source
        .columns
        .iter()
        .map(|column| column.name)
        .collect::<Vec<_>>();
    for step in &plan.steps {
        apply(plan, step, target, &mut columns, &mut rows)?;
    }
    let order = target
        .columns
        .iter()
        .map(|column| {
            columns
                .iter()
                .position(|name| *name == column.name)
                .ok_or_else(|| error(plan, "target column is missing after migration"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if columns.len() != order.len() {
        return Err(error(plan, "migration left undeclared columns"));
    }
    let rows = rows
        .into_iter()
        .map(|row| order.iter().map(|index| row[*index].clone()).collect())
        .collect::<Vec<_>>();
    crate::cells::batch_from_cells(reg, target, &rows)
}

fn error(plan: &MigrationSpec, reason: &str) -> RelationError {
    RelationError::Contract {
        relation: plan.qualified_name(),
        reason: reason.to_owned(),
    }
}
fn apply(
    plan: &MigrationSpec,
    step: &MigrationStep,
    target: &RelationSpec,
    columns: &mut Vec<&'static str>,
    rows: &mut [Vec<Cell>],
) -> Result<(), RelationError> {
    match step {
        MigrationStep::AddColumn { name, default } => {
            if columns.contains(name) || target.column(name).is_none() {
                return Err(error(plan, "added column is duplicate or absent in target"));
            }
            columns.push(name);
            for row in rows {
                row.push(default.clone());
            }
        }
        MigrationStep::DropColumn(name) => {
            let index = columns
                .iter()
                .position(|column| column == name)
                .ok_or_else(|| error(plan, "dropped column is missing"))?;
            columns.remove(index);
            for row in rows {
                row.remove(index);
            }
        }
        MigrationStep::RenameColumn { from, to } => {
            if columns.contains(to) {
                return Err(error(plan, "rename target already exists"));
            }
            let index = columns
                .iter()
                .position(|column| column == from)
                .ok_or_else(|| error(plan, "rename source is missing"))?;
            columns[index] = to;
        }
        MigrationStep::ChangeNullable { name, nullable } => {
            if !columns.contains(name)
                || target
                    .column(name)
                    .is_none_or(|column| column.nullable != *nullable)
            {
                return Err(error(
                    plan,
                    "nullability step differs from target declaration",
                ));
            }
        }
    }
    Ok(())
}

/// Reads an admitted source version through a declared path to the registry's current
/// version. Paths are chosen by fewest declared migrations, then declaration order.
///
/// # Errors
/// Unknown source identity, invalid source rows, missing migration paths or invalid outputs.
pub fn migrate_to_current(
    reg: &Registry,
    batch: &RecordBatch,
) -> Result<RecordBatch, RelationError> {
    let source_id = batch
        .schema()
        .metadata()
        .get(pse_schema::arrow::KEY_CONTRACT_ID)
        .and_then(|text| pse_ids::SemanticId::parse_hex(text).ok())
        .ok_or_else(|| RelationError::UnknownRegistry {
            relation: "missing source relation identity".to_owned(),
        })?;
    let source = reg
        .relation_by_id(source_id)
        .ok_or_else(|| RelationError::UnknownRegistry {
            relation: source_id.to_hex(),
        })?;
    crate::validate::validate_batch(reg, source, batch)
        .map_err(|errors| RelationError::Validation { errors })?;
    let target =
        reg.relation(&source.qualified_name())
            .ok_or_else(|| RelationError::UnknownRegistry {
                relation: source.qualified_name(),
            })?;
    let mut queue = std::collections::VecDeque::from([(source.key.version, Vec::<usize>::new())]);
    let mut seen = std::collections::BTreeSet::from([source.key.version]);
    while let Some((version, path)) = queue.pop_front() {
        if version == target.key.version {
            let mut output = batch.clone();
            for index in path {
                output = migrate(reg, &reg.migrations()[index], &output)?;
            }
            return Ok(output);
        }
        for (index, plan) in reg.migrations().iter().enumerate() {
            if plan.relation == source.qualified_name()
                && plan.from_version == version
                && seen.insert(plan.to_version)
            {
                let mut next = path.clone();
                next.push(index);
                queue.push_back((plan.to_version, next));
            }
        }
    }
    Err(RelationError::Contract {
        relation: source.qualified_name(),
        reason: format!(
            "no declared migration path from {} to {}",
            source.key.version, target.key.version
        ),
    })
}
