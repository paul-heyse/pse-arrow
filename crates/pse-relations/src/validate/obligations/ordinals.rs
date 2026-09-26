// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordinal references bind to the actual selected target cardinality, including empty targets.
use crate::native::{
    common::{Column, DataFusionError, Result},
    functions_aggregate::expr_fn::count,
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, lit},
};
use pse_schema::{Registry, model::RelationSpec};
pub(super) fn plans(
    input: &LogicalPlan,
    spec: &RelationSpec,
    inputs: &super::RelationInputs,
    registry: &Registry,
) -> Result<Vec<LogicalPlan>> {
    let mut checks = Vec::new();
    for occurrence in super::nested_values::occurrences(
        input,
        input.schema().fields().iter().map(AsRef::as_ref),
        |field| {
            field
                .metadata()
                .get(pse_schema::arrow::KEY_EXTENSION_NAME)
                .is_some_and(|name| name == "pse.ordinal_ref")
        },
    )? {
        let declaration = registry
            .obligations(spec.key)
            .map_err(pse_columnar::external)?
            .ordinals
            .iter()
            .find(|obligation| obligation.path == occurrence.path)
            .ok_or_else(|| DataFusionError::Plan("missing compiled ordinal occurrence".into()))?;
        let target = registry
            .relation(&declaration.target)
            .ok_or_else(|| DataFusionError::Plan("ordinal target not declared".into()))?
            .id;
        let value = occurrence.value()?;
        let invalid = if let Some(target) = inputs.get(&target) {
            let mut count_name = "__pse_ordinal_count".to_owned();
            while occurrence
                .input
                .schema()
                .fields()
                .iter()
                .any(|field| field.name() == &count_name)
            {
                count_name.push('_');
            }
            let counts = LogicalPlanBuilder::from(target.clone())
                .aggregate(
                    Vec::<Expr>::new(),
                    vec![count(lit(1_i64)).alias(&count_name)],
                )?
                .build()?;
            LogicalPlanBuilder::from(occurrence.input)
                .cross_join(counts)?
                .filter(
                    value
                        .clone()
                        .lt(lit(0_i64))
                        .or(value.gt_eq(Expr::Column(Column::from_name(count_name)))),
                )?
                .build()?
        } else {
            occurrence.input
        };
        let invalid = super::nested_values::append(
            invalid,
            vec![
                Expr::Column(Column::from_name(occurrence.path_column))
                    .alias("__pse_occurrence_path"),
            ],
        )?;
        checks.push(super::violation(
            invalid,
            spec,
            "ordinal reference outside selected target",
        )?);
    }
    Ok(checks)
}
