// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One native anti-join path for scalar and correlated nested references.

use datafusion::{
    common::{Column, DataFusionError, Result},
    execution::context::SessionContext,
    functions::core::expr_fn::get_field,
    functions_aggregate::expr_fn::count,
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, lit},
};
use pse_relations::generated::runtime::publications;
use pse_schema::{
    Registry,
    model::{FieldContract, ReferenceContract, ReferenceNullPolicy, RelationSpec},
};

pub(super) async fn plans(
    input: &LogicalPlan,
    spec: &RelationSpec,
    record: &publications::Row,
    registry: &Registry,
    catalog: &str,
    context: &SessionContext,
) -> Result<Vec<LogicalPlan>> {
    let mut plans = vec![];
    for occurrence in super::nested_values::occurrences(
        input,
        spec.columns.iter().map(FieldContract::field),
        |field| {
            field
                .metadata()
                .contains_key(pse_schema::model::reference::KEY_REFERENCE)
                || FieldContract::from_field(field.clone()).fk().is_some()
        },
    )? {
        let value = occurrence.value()?;
        let reference =
            ReferenceContract::for_contract(&FieldContract::from_field(occurrence.field))
                .map_err(external)?
                .ok_or_else(|| DataFusionError::Plan("missing reference contract".into()))?;
        let local = components(&reference, &value);
        let present = all(local.iter().cloned().map(Expr::is_not_null));
        let source = LogicalPlanBuilder::from(occurrence.input)
            .filter(present)?
            .project(
                local
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| value.alias(key(i)))
                    .collect::<Vec<_>>(),
            )?
            .build()?;
        let target_spec = registry
            .relation(&reference.relation)
            .ok_or_else(|| DataFusionError::Plan("unknown reference relation".into()))?;
        let selected =
            super::admission::selected_table(record, target_spec.id, catalog, context).await?;
        let missing = if let Some(target) = selected {
            let target = LogicalPlanBuilder::from(target)
                .project(
                    reference
                        .columns
                        .iter()
                        .enumerate()
                        .map(|(i, mapping)| column(&mapping.target).alias(key(i)))
                        .collect::<Vec<_>>(),
                )?
                .build()?;
            let keys = (0..reference.columns.len())
                .map(|i| column(&key(i)))
                .collect::<Vec<_>>();
            let target = LogicalPlanBuilder::from(target)
                .filter(all(keys.iter().cloned().map(Expr::is_not_null)))?
                .build()?;
            let duplicates = LogicalPlanBuilder::from(target.clone())
                .aggregate(keys, vec![count(lit(1_i64)).alias("key_count")])?
                .filter(column("key_count").gt(lit(1_i64)))?
                .build()?;
            plans.push(super::admission::violation(
                duplicates,
                spec,
                &format!("ambiguous reference target {}", reference.relation),
            )?);
            LogicalPlanBuilder::from(source)
                .alias("candidate")?
                .join(
                    LogicalPlanBuilder::from(target)
                        .alias("reference")?
                        .build()?,
                    JoinType::LeftAnti,
                    (
                        (0..reference.columns.len())
                            .map(|i| Column::new(Some("candidate"), key(i)))
                            .collect::<Vec<_>>(),
                        (0..reference.columns.len())
                            .map(|i| Column::new(Some("reference"), key(i)))
                            .collect::<Vec<_>>(),
                    ),
                    None,
                )?
                .build()?
        } else {
            source
        };
        plans.push(super::admission::violation(
            missing,
            spec,
            &format!("missing reference {}", occurrence.path.join(".")),
        )?);
    }
    Ok(plans)
}

/// Row-local presence obligation, also lowered into the table's durable CHECK.
pub(super) fn presence(reference: &ReferenceContract, value: &Expr) -> Expr {
    let components = components(reference, value);
    let present = all(components.iter().cloned().map(Expr::is_not_null));
    match reference.null_policy {
        ReferenceNullPolicy::Required => present,
        ReferenceNullPolicy::AllOrNone => {
            present.or(all(components.into_iter().map(Expr::is_null)))
        }
    }
}

fn components(reference: &ReferenceContract, value: &Expr) -> Vec<Expr> {
    reference
        .columns
        .iter()
        .map(|mapping| mapping.source.iter().fold(value.clone(), get_field))
        .collect()
}
fn all(values: impl IntoIterator<Item = Expr>) -> Expr {
    values
        .into_iter()
        .reduce(Expr::and)
        .unwrap_or_else(|| lit(true))
}
fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}
fn key(index: usize) -> String {
    format!("key_{index}")
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
