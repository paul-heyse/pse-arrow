// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Output contracts are checked native expressions, before physical execution.
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion::arrow::datatypes::DataType;
use datafusion_common::{Column, DFSchema};
use datafusion_expr::{Expr, LogicalPlan, LogicalPlanBuilder};
use pse_catalog::session::SnapshotSession;
use pse_schema::{Registry, model::RelationSpec};
use std::sync::Arc;

pub(super) fn prepare(
    plan: LogicalPlan,
    target: &RelationSpec,
    session: &SnapshotSession,
    registry: &Registry,
    keep_support: bool,
) -> Result<LogicalPlan, RuleError> {
    check_names(&plan, target, keep_support)?;
    let checked = session.scalar_function("pse_checked_value")?;
    let mut expressions = Vec::new();
    for field in &target.columns {
        let (qualifier, source) = plan
            .schema()
            .qualified_field_with_unqualified_name(field.name())
            .map_err(engine)?;
        let declared = pse_schema::arrow::field_for(registry, field)
            .map_err(|error| internal(error.to_string()))?;
        let mut value = Expr::Column(Column::new(qualifier.cloned(), source.name()));
        let actual = source.data_type();
        let wanted = declared.data_type();
        // Conversion is explicit in the native plan. CheckedValue then establishes
        // dictionary membership, bounds, alternatives and totality from actual values.
        if actual != wanted
            && ((actual.is_integer() && wanted.is_integer())
                || matches!(
                    (actual, wanted),
                    (DataType::Utf8, DataType::Dictionary(_, _))
                ))
        {
            value = Expr::Cast(datafusion_expr::expr::Cast::new(
                Box::new(value),
                wanted.clone(),
            ));
        }
        expressions.push(
            checked
                .call(vec![
                    value,
                    datafusion_expr::lit(target.key.qualified_name()),
                    datafusion_expr::lit(field.name().to_owned()),
                ])
                .alias(field.name()),
        );
    }
    if keep_support {
        expressions.extend(
            plan.schema()
                .iter()
                .filter(|(_, field)| field.name().starts_with("__pse_support_"))
                .map(|(qualifier, field)| {
                    Expr::Column(Column::new(qualifier.cloned(), field.name()))
                }),
        );
    }
    let projected = LogicalPlanBuilder::from(plan)
        .project(expressions)
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let projected = session.derive_plan_fields(projected, &pse_ids::CancellationToken::new())?;
    let mut schema = pse_schema::arrow::relation_schema(registry, target)
        .map_err(|error| internal(error.to_string()))?;
    if keep_support {
        let fields = schema
            .fields()
            .iter()
            .cloned()
            .chain(
                projected
                    .schema()
                    .fields()
                    .iter()
                    .filter(|field| field.name().starts_with("__pse_support_"))
                    .cloned(),
            )
            .collect::<Vec<_>>();
        schema = datafusion::arrow::datatypes::Schema::new_with_metadata(
            fields,
            schema.metadata().clone(),
        );
    }
    let expressions = projected
        .schema()
        .columns()
        .into_iter()
        .map(Expr::Column)
        .collect();
    Ok(LogicalPlan::Projection(
        datafusion_expr::Projection::try_new_with_schema(
            expressions,
            Arc::new(projected),
            Arc::new(DFSchema::try_from(schema).map_err(engine)?),
        )
        .map_err(engine)?,
    ))
}

fn check_names(
    plan: &LogicalPlan,
    target: &RelationSpec,
    keep_support: bool,
) -> Result<(), RuleError> {
    let expected = target
        .columns
        .iter()
        .map(pse_schema::model::FieldContract::name)
        .collect::<std::collections::BTreeSet<_>>();
    let actual = plan
        .schema()
        .fields()
        .iter()
        .filter(|field| !(keep_support && field.name().starts_with("__pse_support_")))
        .map(|field| field.name().as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if actual != expected {
        return Err(internal(format!(
            "native rule output fields differ from {}",
            target.key.qualified_name()
        )));
    }
    Ok(())
}
