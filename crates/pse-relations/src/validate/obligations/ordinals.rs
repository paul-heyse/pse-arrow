// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordinal references bind to the actual selected target cardinality, including empty targets.
use crate::native::{
    common::{Column, DataFusionError, Result},
    functions_aggregate::expr_fn::count,
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, lit},
};
use pse_schema::{
    Registry,
    model::{EXTENSION_TYPES, RelationSpec},
};
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
        let text = occurrence
            .field
            .metadata()
            .get(pse_schema::arrow::KEY_EXTENSION_METADATA)
            .ok_or_else(|| DataFusionError::Plan("ordinal metadata absent".into()))?;
        let target = crate::ext::ExtMetadata::parse(&EXTENSION_TYPES[6], text)?
            .target_relation_id
            .ok_or_else(|| DataFusionError::Plan("ordinal target absent".into()))?;
        if registry.relation_by_id(target).is_none() {
            return Err(DataFusionError::Plan("ordinal target not declared".into()));
        }
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
