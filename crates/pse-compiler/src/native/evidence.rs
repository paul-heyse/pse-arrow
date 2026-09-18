// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Repeated native rule invocations can justify the same head through different
//! sources. A derivation owns the union of those source keys, not one row per pass.
use crate::{
    CompilerError,
    passes::{invalid, native_rows::engine},
};
use datafusion::{
    arrow::{
        array::{ListArray, new_empty_array},
        buffer::OffsetBuffer,
        datatypes::DataType,
    },
    common::{Column, ScalarValue},
    functions::core::expr_fn::coalesce,
    functions_aggregate::expr_fn::array_agg,
    functions_nested::expr_fn::{array_distinct, array_sort, flatten},
    logical_expr::{Aggregate, Expr, LogicalPlan, Projection, Union, col, lit},
};
use pse_catalog::session::{SnapshotSession, output::declare_relation_output};
use pse_ids::CancellationToken;
use pse_relations::generated::provenance::derivations;
use pse_schema::model::RelationKey;
use std::sync::Arc;

#[cfg(test)]
mod tests;

pub(super) fn combine(
    key: RelationKey,
    mut plans: Vec<LogicalPlan>,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<LogicalPlan, CompilerError> {
    let plan = if plans.len() == 1 {
        plans.remove(0)
    } else {
        LogicalPlan::Union(
            Union::try_new_with_loose_types(plans.into_iter().map(Arc::new).collect())
                .map_err(engine)?,
        )
    };
    if key != derivations::RELATION_KEY {
        return Ok(plan);
    }
    // Group the complete head, not just its ID: inconsistent meanings for one ID
    // remain separate rows, so ordinary primary-key admission still refuses them.
    let groups = plan
        .schema()
        .columns()
        .into_iter()
        .filter(|column| column.name != "supporting")
        .map(Expr::Column)
        .collect();
    let (qualifier, field) = plan
        .schema()
        .qualified_field_with_unqualified_name("supporting")
        .map_err(engine)?;
    let supporting = Expr::Column(Column::new(qualifier.cloned(), field.name()));
    let grouped = LogicalPlan::Aggregate(
        Aggregate::try_new(
            Arc::new(plan),
            groups,
            vec![array_agg(supporting).alias("__support_groups")],
        )
        .map_err(engine)?,
    );
    let mut columns = grouped
        .schema()
        .columns()
        .into_iter()
        .filter(|column| column.name != "__support_groups")
        .map(Expr::Column)
        .collect::<Vec<_>>();
    columns.push(
        coalesce(vec![
            array_sort(
                array_distinct(flatten(col("__support_groups"))),
                lit("ASC"),
                lit("NULLS FIRST"),
            ),
            empty_support(session)?,
        ])
        .alias("supporting"),
    );
    let plan =
        LogicalPlan::Projection(Projection::try_new(columns, Arc::new(grouped)).map_err(engine)?);
    let plan = session.derive_plan_fields(plan, cancel)?;
    declare_relation_output(
        plan,
        session.registry(),
        derivations::spec(session.registry())?,
    )
    .map_err(engine)
}

fn empty_support(session: &SnapshotSession) -> Result<Expr, CompilerError> {
    let schema = pse_schema::arrow::relation_schema(
        session.registry(),
        derivations::spec(session.registry())?,
    )
    .map_err(|error| invalid(error.to_string()))?;
    let DataType::List(child) = schema
        .field_with_name("supporting")
        .map_err(|error| invalid(error.to_string()))?
        .data_type()
    else {
        return Err(invalid("derivation support declaration is not a list"));
    };
    let list = ListArray::try_new(
        Arc::clone(child),
        OffsetBuffer::new(vec![0_i32, 0].into()),
        new_empty_array(child.data_type()),
        None,
    )
    .map_err(|error| invalid(error.to_string()))?;
    Ok(lit(ScalarValue::List(Arc::new(list))))
}
