// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native graph ordinal projection shared by indexed graph consumers.
use super::native_construction::{error, project};
use crate::CompilerError;
use datafusion::arrow::array::{Array, FixedSizeBinaryArray, Int64Array};
use datafusion::logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, lit};
use pse_ids::SemanticId;
use pse_mathir::NodeId;
use pse_relations::columnar::FieldCheckedBatch;
use std::collections::BTreeMap;

pub(crate) fn root_field(key: &pse_schema::model::RelationKey) -> Option<&'static str> {
    match key.qualified_name().as_str() {
        "compiled.symbol_expressions"
        | "compiled.expression_roots"
        | "compiled.element_projection_coefficients"
        | "compiled.predicate_mask_members" => Some("node_id"),
        "compiled.contributions" => Some("expression_root"),
        _ => None,
    }
}
pub(crate) fn node_mapping(
    mapping: &BTreeMap<NodeId, NodeId>,
) -> Result<LogicalPlan, CompilerError> {
    if mapping.is_empty() {
        return LogicalPlanBuilder::empty(false)
            .project([lit(0_i64).alias("old_node"), lit(0_i64).alias("new_node")])
            .and_then(LogicalPlanBuilder::build)
            .map_err(error);
    }
    let plan = LogicalPlanBuilder::values(
        mapping
            .iter()
            .map(|(old, new)| vec![lit(old.0), lit(new.0)])
            .collect(),
    )
    .and_then(LogicalPlanBuilder::build)
    .map_err(error)?;
    let columns = plan.schema().columns();
    project(
        plan,
        [
            Expr::Column(columns[0].clone()).alias("old_node"),
            Expr::Column(columns[1].clone()).alias("new_node"),
        ],
    )
}

pub(crate) fn ordinal(
    input: &FieldCheckedBatch,
    name: &str,
    row: usize,
) -> Result<Option<i64>, CompilerError> {
    let Some(array) = input.batch().column_by_name(name) else {
        return Ok(None);
    };
    let array = array
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| graph_error("graph ordinal has unexpected storage"))?;
    if array.is_null(row) {
        return Ok(None);
    }
    let value = array.value(row);
    if value < 0 {
        return Err(graph_error("graph ordinal is negative"));
    }
    Ok(Some(value))
}
pub(crate) fn identity(
    input: &FieldCheckedBatch,
    name: &str,
    row: usize,
) -> Result<Option<SemanticId>, CompilerError> {
    let Some(array) = input.batch().column_by_name(name) else {
        return Ok(None);
    };
    let array = array
        .as_any()
        .downcast_ref::<FixedSizeBinaryArray>()
        .ok_or_else(|| graph_error("graph identity has unexpected storage"))?;
    if array.is_null(row) {
        return Ok(None);
    }
    Ok(Some(SemanticId::try_from_slice(array.value(row)).map_err(
        |_| graph_error("graph identity width differs"),
    )?))
}

fn graph_error(detail: &str) -> CompilerError {
    pse_templates::TemplateError::Binding {
        instance: SemanticId::NIL,
        detail: detail.to_owned(),
    }
    .into()
}
