// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Materialize only actual result fields, witness keys and witness conditions.
//! Native projection retains every input/condition until common admission and
//! lets the native optimizer prune intermediate join payloads before allocation.

use super::{
    Arc, ColumnRole, Expr, LogicalPlan, NativeWitness, RelationSpec, RuleError, col, engine,
};
use datafusion_common::Column;
use datafusion_expr::Projection;
use std::collections::BTreeSet;

pub(super) fn inputs(
    plan: LogicalPlan,
    target: &RelationSpec,
    columns: &[(String, String)],
    witnesses: &mut [NativeWitness],
) -> Result<LogicalPlan, RuleError> {
    let mut names = columns
        .iter()
        .zip(&target.columns)
        .filter(|((name, _), field)| {
            field.name() != "derivation_id"
                || field.role() != ColumnRole::Provenance
                || target.primary_key.iter().any(|key| *key == name)
        })
        .map(|((_, source), _)| source.as_str())
        .collect::<BTreeSet<_>>();
    for witness in witnesses.iter() {
        names.extend(witness.key_columns.iter().flatten().map(String::as_str));
    }
    let mut expressions = names
        .into_iter()
        .map(|name| {
            let (qualifier, field) = plan
                .schema()
                .qualified_field_with_unqualified_name(name)
                .map_err(engine)?;
            Ok(Expr::Column(Column::new(qualifier.cloned(), field.name())))
        })
        .collect::<Result<Vec<_>, RuleError>>()?;
    // Evaluate a witness condition beside its actual row, including subqueries
    // or volatile expressions. Subsequent membership/support queries consume
    // that same boolean instead of evaluating it against another execution.
    let mut occupied = plan
        .schema()
        .fields()
        .iter()
        .map(|field| field.name().clone())
        .collect::<BTreeSet<_>>();
    for (index, witness) in witnesses.iter_mut().enumerate() {
        if let Some(condition) = witness.when.take() {
            let mut name = format!("__native_witness_condition_{index}");
            while occupied.contains(&name) {
                name.push('_');
            }
            occupied.insert(name.clone());
            expressions.push(condition.alias(&name));
            witness.when = Some(col(&name));
        }
    }
    Ok(LogicalPlan::Projection(
        Projection::try_new(expressions, Arc::new(plan)).map_err(engine)?,
    ))
}
