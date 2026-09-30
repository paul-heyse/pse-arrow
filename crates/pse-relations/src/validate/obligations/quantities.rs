// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Occurrence-preserving anti joins against admitted selected physical compatibility.
use super::QuantityCompatibility;
use crate::native::{
    arrow::datatypes::Field,
    common::{Column, Result},
    functions::core::expr_fn::get_field,
    logical_expr::{JoinType, LogicalPlan, LogicalPlanBuilder, lit},
};

pub(super) fn plans(
    input: &LogicalPlan,
    compatibility: Option<&QuantityCompatibility>,
) -> Result<Vec<LogicalPlan>> {
    super::nested_values::occurrences(
        input, input.schema().fields().iter().map(AsRef::as_ref), is_quantity,
    )?.into_iter().map(|occurrence| {
        let value = occurrence.value()?;
        let values = super::nested_values::append(occurrence.input, vec![
            get_field(value.clone(), "quantity_type_id").alias("quantity"),
            get_field(value, "unit_id").alias("unit"),
        ])?;
        let invalid = if let Some(compatibility) = compatibility {
            LogicalPlanBuilder::from(values).join(
                compatibility.compatible.clone(), JoinType::LeftAnti,
                (vec![Column::from_name("quantity"), Column::from_name("unit")],
                 vec![Column::from_name("admitted_quantity"), Column::from_name("admitted_unit")]), None,
            )?.build()?
        } else { values };
        super::nested_values::append(invalid, vec![lit(format!(
            "quantity {} requires selected admitted physical definitions and a compatible scalar representation",
            occurrence.path.join(".")
        )).alias("violation")])
    }).collect()
}
pub(super) fn is_quantity(field: &Field) -> bool {
    field
        .metadata()
        .get(pse_schema::arrow::KEY_EXTENSION_NAME)
        .is_some_and(|name| name == "pse.quantity_value")
}
#[cfg(test)]
mod tests;
