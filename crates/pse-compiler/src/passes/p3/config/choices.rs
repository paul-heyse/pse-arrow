// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared parameter/feature rows join their explicit overrides before text parsing.
use super::{
    Configuration, invalid,
    native::{self, Source, engine},
};
use crate::CompilerError;
use datafusion::{
    common::{Column, NullHandling, UnnestOptions},
    functions::core::expr_fn::get_field,
    functions_aggregate::expr_fn::count,
    logical_expr::{JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::SnapshotSession;
use pse_relations::{
    columnar::{Collection, RelationRow},
    generated::authored,
};
use std::collections::BTreeMap;

pub(super) async fn select<T: RelationRow>(
    config: &mut Configuration<'_>,
    instance: &authored::instances::Row,
    assignments: &str,
) -> Result<Vec<(Source<T>, Option<String>)>, CompilerError> {
    let spec = T::relation(config.registry)?;
    let source = config
        .binding_batches
        .get(&spec.id)
        .ok_or_else(|| invalid("configuration declaration absent"))?
        .clone();
    let mut columns = Collection::new(config.registry, config.reserver, config.cancel);
    columns.push(instance.clone())?;
    let instance_batch = columns
        .finish()?
        .remove(&authored::instances::RELATION_KEY)
        .ok_or_else(|| invalid("configuration instance argument absent"))?;
    let session = config.session.with_checked_role_inputs(
        BTreeMap::from([
            ("configuration_declaration".to_owned(), source),
            ("configuration_instance".to_owned(), instance_batch),
        ]),
        config.cancel,
    )?;
    let declaration = LogicalPlanBuilder::from(session.scan_role("configuration_declaration")?)
        .filter(col("template_id").eq(native::identity(
            config.registry,
            spec,
            "template_id",
            instance.template_id,
        )?))
        .map_err(engine)?
        .alias("d")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let overrides = LogicalPlanBuilder::from(session.scan_role("configuration_instance")?)
        .project([col(assignments)])
        .map_err(engine)?
        .unnest_column_with_options(
            Column::from_name(assignments),
            UnnestOptions::new().with_null_handling(NullHandling::Drop),
        )
        .map_err(engine)?
        .project([
            get_field(col(assignments), "name").alias("assignment_name"),
            get_field(col(assignments), "value").alias("assignment_value"),
        ])
        .map_err(engine)?
        .alias("a")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    reject_assignments(config, &session, &overrides, &declaration).await?;
    let joined = native::join(
        declaration,
        overrides,
        JoinType::Left,
        &[("d.name", "a.assignment_name")],
    )?;
    let mut fields = spec
        .columns
        .iter()
        .map(|field| native::column("d", field.name()).alias(field.name()))
        .collect::<Vec<_>>();
    fields.push(col("a.assignment_value").alias("configuration_override"));
    let joined = LogicalPlanBuilder::from(joined)
        .project(fields)
        .map_err(engine)?
        .sort(
            spec.primary_key
                .iter()
                .map(|name| col(*name).sort(true, false)),
        )
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    crate::passes::native_rows::keyed_rows_with_text::<T>(
        &mut config.arguments,
        joined,
        Some(col("configuration_override")),
        &session,
        config.registry,
        config.cancel,
    )
    .await
}

async fn reject_assignments(
    config: &mut Configuration<'_>,
    session: &SnapshotSession,
    overrides: &LogicalPlan,
    declaration: &LogicalPlan,
) -> Result<(), CompilerError> {
    let duplicate = LogicalPlanBuilder::from(overrides.clone())
        .aggregate(
            [col("a.assignment_name")],
            [count(lit(1_i64)).alias("assignment_count")],
        )
        .map_err(engine)?
        .filter(col("assignment_count").not_eq(lit(1_i64)))
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    crate::passes::native_rows::reject(
        duplicate,
        session,
        config.cancel,
        "duplicate instance configuration assignment",
    )
    .await?;
    let unknown = native::join(
        overrides.clone(),
        declaration.clone(),
        JoinType::LeftAnti,
        &[("a.assignment_name", "d.name")],
    )?;
    crate::passes::native_rows::reject(
        unknown,
        session,
        config.cancel,
        "undeclared instance configuration assignment",
    )
    .await?;
    Ok(())
}
