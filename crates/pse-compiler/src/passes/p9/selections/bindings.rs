// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Required state-parameter declarations are matched before typed instance construction.

use datafusion::{
    common::ScalarValue,
    functions_aggregate::expr_fn::count,
    logical_expr::{JoinType, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::{SnapshotSession, output::checked_literal};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::{normalized, reference};
use pse_schema::Registry;
use std::collections::BTreeMap;

use crate::{
    CompilerError,
    passes::native_rows::{AlgorithmInputs, column, engine, join, project, scan},
};

pub(super) async fn state_parameters(
    owner: &mut AlgorithmInputs,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<BTreeMap<SemanticId, String>, CompilerError> {
    let selected = LogicalPlanBuilder::from(super::native::mappings(session, registry)?)
        .alias("a")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let joined = join(
        selected.clone(),
        scan(session, reference::method_specs::spec(registry)?, "m")?,
        JoinType::Left,
        &[("a.method_id", "m.method_id")],
    )?;
    let joined = join(
        joined,
        scan(session, normalized::template_params::spec(registry)?, "v")?,
        JoinType::Left,
        &[
            ("m.template_id", "v.template_id"),
            ("a.parameter_name", "v.name"),
        ],
    )?;
    let parameters = normalized::template_params::spec(registry)?;
    let kind = parameters
        .columns
        .iter()
        .find(|column| column.name() == "logical_type_id")
        .ok_or_else(|| super::invalid("template parameter logical type field absent"))?;
    let semantic_id = registry
        .logical_type("semantic_id")
        .ok_or_else(|| super::invalid("SemanticId logical type absent"))?
        .id;
    let semantic_id = checked_literal(
        registry,
        kind,
        ScalarValue::FixedSizeBinary(16, Some(semantic_id.as_bytes().to_vec())),
    )
    .map_err(engine)?;
    let bad = column("v", "template_id")
        .is_null()
        .or(column("v", "logical_type_id").not_eq(semantic_id))
        .or(column("v", "enum_id").is_not_null())
        .or(column("v", "required").is_not_true());
    crate::passes::native_rows::reject(
        LogicalPlanBuilder::from(joined.clone())
            .filter(bad)
            .map_err(engine)?
            .build()
            .map_err(engine)?,
        session,
        cancel,
        "method state mapping must name an actual required SemanticId parameter",
    )
    .await?;
    let count = LogicalPlanBuilder::from(joined)
        .aggregate(
            [column("a", "method_id")],
            [count(lit(1_i64)).alias("matches")],
        )
        .map_err(engine)?
        .filter(col("matches").not_eq(lit(1_i64)))
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    crate::passes::native_rows::reject(
        count,
        session,
        cancel,
        "method state parameter mapping is ambiguous",
    )
    .await?;
    let rows: Vec<reference::method_state_parameters::Row> = owner
        .rows(
            project(
                selected,
                reference::method_state_parameters::spec(registry)?,
                "a",
            )?,
            session,
            registry,
            cancel,
        )
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.method_id, row.parameter_name))
        .collect())
}
