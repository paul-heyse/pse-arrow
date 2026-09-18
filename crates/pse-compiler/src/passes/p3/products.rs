// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native finite products, exact source witnesses and Cartesian candidates.
mod ancestry;
pub(crate) mod expansion;
use super::plans as native;
mod shapes;

use crate::{CompilerError, passes::native_outputs::Sources};
use datafusion::logical_expr::col;
use native::{Plans, project};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::columnar::FieldCheckedBatch;
use pse_rules::strata::native_input::NativeInput;
use pse_schema::model::{AlgorithmSpec, RelationKey};
use std::{collections::BTreeMap, sync::Arc};

/// All outputs retain the actual native completion and original row correspondences.
/// Candidate tuples claim Cartesian membership only; P5 decides semantic eligibility.
pub(crate) async fn emit(
    inputs: &BTreeMap<RelationKey, FieldCheckedBatch>,
    sources: &Sources,
    pass: &AlgorithmSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    let mut plans = Plans::new(inputs, sources, pass, session, cancel)?;
    let shapes = shapes::build(&mut plans).await?;
    let (parents, bindings) = ancestry::build(&mut plans, shapes).await?;
    let subsets = expansion::subsets(&mut plans, parents).await?;
    let labels = expansion::labels(&mut plans, subsets.clone()).await?;
    let subsets = expansion::labelled(subsets, labels.clone(), "factors", "product_id")?;
    let subsets = expansion::labelled(
        subsets,
        labels.clone(),
        "parent_factors",
        "parent_product_id",
    )?;
    let subsets = plans.retain(subsets).await?;
    let mut output = BTreeMap::new();
    let products = project(
        subsets.clone(),
        [
            col("product_id"),
            col("factors").alias("domain_ids"),
            col("supports"),
        ],
    )?;
    let product = plans
        .output("normalized.domain_products", products, false)
        .await?;
    output.insert(plans.key("normalized.domain_products")?, product);
    let bindings = expansion::labelled(bindings, labels, "factors", "product_id")?;
    let bindings = project(
        bindings,
        [
            col("instance_id"),
            col("product_id"),
            col("index"),
            col("supports"),
        ],
    )?;
    let bindings = plans
        .output("normalized.instance_binding_products", bindings, false)
        .await?;
    output.insert(plans.key("normalized.instance_binding_products")?, bindings);
    let projections = expansion::projection(&plans, subsets.clone())?;
    let projections = plans
        .output("normalized.domain_product_projections", projections, false)
        .await?;
    output.insert(
        plans.key("normalized.domain_product_projections")?,
        projections,
    );
    let sources = project(subsets, [col("product_id"), col("supports")])?;
    let sources = plans
        .output("normalized.domain_product_sources", sources, true)
        .await?;
    output.insert(plans.key("normalized.domain_product_sources")?, sources);
    let tuples = expansion::tuples(&mut plans).await?;
    let members = expansion::members(tuples.clone())?;
    let tuples = plans
        .output(
            "normalized.candidate_index_tuples",
            project(tuples, [col("product_id"), col("tuple"), col("supports")])?,
            false,
        )
        .await?;
    output.insert(plans.key("normalized.candidate_index_tuples")?, tuples);
    let members = plans
        .output("normalized.candidate_index_members", members, false)
        .await?;
    output.insert(plans.key("normalized.candidate_index_members")?, members);
    Ok(output)
}
