// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native finite demand keys plus the shared typed source-coordinate algorithm.
mod dependencies;
mod indices;
mod reads;
use crate::{
    CompilerError, PassContext,
    passes::{
        native_construction::{
            Plans, append, c, concat, distinct, error, explode, filter, invalid, join, prefix,
            project, union,
        },
        native_outputs::Sources,
    },
};
use datafusion::{
    common::ScalarValue,
    functions::core::expr_fn::coalesce,
    functions_nested::expr_fn::array_length,
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, col, lit, when},
};
use pse_catalog::session::scalar;
use pse_catalog::session::scalar::array_element;
use pse_relations::columnar::FieldCheckedBatch;
use pse_rules::strata::native_input::NativeInput;
use pse_schema::model::{PassSpec, RelationKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Not,
    sync::Arc,
};
type Inputs = BTreeMap<RelationKey, FieldCheckedBatch>;
type Outputs = BTreeMap<RelationKey, Arc<NativeInput>>;

pub(super) async fn construct(
    pass: &PassSpec,
    inputs: &Inputs,
    sources: &Sources,
    ctx: &PassContext<'_>,
) -> Result<Outputs, CompilerError> {
    let base = ctx.session;
    let mut plans = Plans::new(inputs, sources, pass, base, ctx.cancel)?;
    let mut output = Outputs::new();
    let indices = indices::build(&mut plans, &mut output).await?;
    dependencies::build(&mut plans, &indices, &mut output).await?;
    Box::pin(reads::build(&mut plans, inputs, ctx, &mut output)).await?;
    Ok(output)
}
async fn emit(
    plans: &mut Plans<'_>,
    outputs: &mut Outputs,
    name: &str,
    values: LogicalPlan,
) -> Result<(), CompilerError> {
    let name = format!("inferred.{name}");
    let output = plans.output(&name, values, false).await?;
    outputs.insert(plans.key(&name)?, output);
    Ok(())
}
