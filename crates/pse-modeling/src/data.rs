// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed immutable table admission, independent of any selected model instance.
use crate::specialize::value::{Environment, Evaluator, Value};
use crate::{CheckedPackage, DeclarationId, Result, Type, TypeContext, invalid};
use pse_ids::SemanticId;
use std::collections::{BTreeMap, BTreeSet};
/// Derived table data. Authored declarations remain the durable authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    /// Ordered complete key types.
    pub keys: Vec<Type>,
    /// Whole scalar or heterogeneous row type.
    pub result: Type,
    /// Named heterogeneous columns in source order.
    pub columns: Vec<(String, Type)>,
    /// Unique admitted keys and complete values.
    pub rows: BTreeMap<Vec<Value>, Value>,
    /// Declared fallback, evaluated at admission.
    pub default: Option<Value>,
    /// Missing values remain explicit.
    pub optional: bool,
    /// Dataset declaration supplying each row.
    pub origins: BTreeMap<Vec<Value>, DeclarationId>,
}
pub(crate) fn admit(p: &mut CheckedPackage, c: &TypeContext<'_>) -> Result<()> {
    let env = Environment::new();
    let mut pending = p
        .declarations
        .values()
        .filter(|r| r.value.table.is_some())
        .map(|r| r.declaration_id)
        .collect::<BTreeSet<_>>();
    while !pending.is_empty() {
        let mut progress = false;
        let mut failure = None;
        for id in pending.clone() {
            match table(p, c, &env, id) {
                Ok(value) => {
                    p.tables.insert(id, value);
                    pending.remove(&id);
                    progress = true;
                }
                Err(error) => failure = Some(error),
            }
        }
        if !progress {
            return Err(failure.unwrap_or_else(|| invalid(SemanticId::NIL, "cyclic table data")));
        }
    }
    // A dataset supplies a table's rows here or a keyed kind's in `entity::admit`; only a
    // keyed kind's dataset binds keys (ADR-0123 Outcome 2).
    for row in p.declarations.values() {
        if let Some(dataset) = &row.value.dataset {
            let target = p
                .resolve(row.declaration_id, &dataset.target)
                .ok_or_else(|| invalid(row.declaration_id, "unknown dataset target"))?;
            if p.tables.contains_key(&target) {
                if !dataset.bindings.is_empty() {
                    return Err(invalid(
                        row.declaration_id,
                        "only a dataset of a keyed kind binds keys",
                    ));
                }
            } else if !p.kinds.contains_key(&target) {
                return Err(invalid(
                    row.declaration_id,
                    "dataset target is neither a table nor a keyed entity kind",
                ));
            }
        }
    }
    Ok(())
}
fn evaluator<'a, 'b>(
    p: &'a CheckedPackage,
    c: &'a TypeContext<'b>,
    env: &'a Environment,
    at: DeclarationId,
) -> Evaluator<'a, 'b> {
    Evaluator {
        package: p,
        physical: c,
        at,
        env,
        limit: 100_000,
        stack: vec![],
    }
}
fn table(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    env: &Environment,
    id: DeclarationId,
) -> Result<Table> {
    let contract = p.declarations[&id]
        .value
        .table
        .as_ref()
        .ok_or_else(|| invalid(id, "table contract"))?;
    let names = p.named_types(id);
    let variables = BTreeSet::new();
    let mut key_names = BTreeSet::new();
    let keys = contract
        .keys
        .iter()
        .map(|key| {
            if !key_names.insert(&key.name) {
                return Err(invalid(id, "duplicate table key name"));
            }
            c.resolve(&key.r#type, &variables, &names, id)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut column_names = BTreeSet::new();
    let columns = contract
        .columns
        .iter()
        .map(|column| {
            if !column_names.insert(&column.name) {
                return Err(invalid(id, "duplicate table column"));
            }
            Ok((
                column.name.clone(),
                c.resolve(&column.r#type, &variables, &names, id)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let result = match &contract.value_type {
        Some(value) if columns.is_empty() => c.resolve(value, &variables, &names, id)?,
        None if !columns.is_empty() => Type::Row(id),
        _ => {
            return Err(invalid(
                id,
                "a table declares a value type exactly when it has no columns",
            ));
        }
    };
    let default = contract
        .default_value
        .as_ref()
        .map(|source| evaluator(p, c, env, id).text(source, Some(&result)))
        .transpose()?;
    let mut table = Table {
        keys,
        result,
        columns,
        rows: BTreeMap::new(),
        origins: BTreeMap::new(),
        default,
        optional: contract.missing_policy
            == pse_model::generated::enums::ModelingMissingPolicy::Optional,
    };
    for row in p.declarations.values() {
        let Some(data) = &row.value.dataset else {
            continue;
        };
        if p.resolve(row.declaration_id, &data.target) != Some(id) {
            continue;
        }
        if data.source.is_empty() {
            return Err(invalid(row.declaration_id, "dataset provenance is empty"));
        }
        for entry in &data.rows {
            if entry.keys.len() != table.keys.len() {
                return Err(invalid(row.declaration_id, "dataset key arity"));
            }
            // Cells are typed once, never evaluated (ADR-0123 Outcome 1).
            let cell = |cell: &pse_authoring::language::Cell, ty: &Type| -> Result<Value> {
                let typed = crate::entity::typed(p, c, row.declaration_id, cell, ty)?;
                if typed.uncertainty.is_some() {
                    return Err(invalid(
                        row.declaration_id,
                        "a table row carries no uncertainty",
                    ));
                }
                Ok(typed.value)
            };
            let keys = entry
                .keys
                .iter()
                .zip(&table.keys)
                .map(|(source, ty)| cell(source, ty))
                .collect::<Result<Vec<_>>>()?;
            if table.rows.contains_key(&keys) {
                return Err(invalid(row.declaration_id, "duplicate dataset key"));
            }
            let value = if table.columns.is_empty() {
                if entry.values.len() != 1 {
                    return Err(invalid(row.declaration_id, "scalar table row arity"));
                }
                cell(&entry.values[0], &table.result)?
            } else {
                if entry.values.len() != table.columns.len() {
                    return Err(invalid(row.declaration_id, "heterogeneous table row arity"));
                }
                Value::Row {
                    table: id,
                    fields: entry
                        .values
                        .iter()
                        .zip(&table.columns)
                        .map(|(source, (name, ty))| Ok((name.clone(), cell(source, ty)?)))
                        .collect::<Result<_>>()?,
                }
            };
            table.origins.insert(keys.clone(), row.declaration_id);
            table.rows.insert(keys, value);
        }
    }
    Ok(table)
}
