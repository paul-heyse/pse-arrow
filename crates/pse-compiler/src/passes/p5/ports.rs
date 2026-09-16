// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native port traversal and ordered state/member construction.
mod guard;
mod walk;
use crate::{
    CompilerError,
    passes::{
        native_construction::{
            Plans, append, c, concat, error, filter, join, prefix, project, union,
        },
        native_outputs::Sources,
    },
};
use datafusion::{
    common::ScalarValue,
    functions_nested::expr_fn::array_length,
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::{SnapshotSession, scalar};
use pse_ids::CancellationToken;
use pse_relations::columnar::FieldCheckedBatch;
use pse_rules::strata::native_input::NativeInput;
use pse_schema::model::{PassSpec, RelationKey};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) async fn construct(
    inputs: &BTreeMap<RelationKey, FieldCheckedBatch>,
    sources: &Sources,
    pass: &PassSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    let mut plans = Plans::new(inputs, sources, pass, session, cancel)?;
    let owners = plans.scan("normalized.instance_bindings", "owner")?;
    let ports = plans.scan("normalized.template_ports", "port")?;
    let base = join(
        owners,
        ports,
        JoinType::Inner,
        [c("owner", "template_id").eq(c("port", "template_id"))],
    )?;
    let lengths = plans.scan("normalized.port_binding_lengths", "length")?;
    let base = join(
        base,
        lengths,
        JoinType::Left,
        [
            c("port", "template_id").eq(c("length", "template_id")),
            c("port", "name").eq(c("length", "name")),
        ],
    )?;
    plans
        .require(
            &base,
            c("length", "length").is_not_null(),
            "port has no actual parsed path length",
        )
        .await?;
    let base = project(
        base,
        [
            scalar::named_id(
                c("owner", "instance_id"),
                concat(lit("port:"), c("port", "name")),
            )
            .alias("port_id"),
            c("owner", "instance_id").alias("owner_instance_id"),
            c("owner", "template_id").alias("owner_template_id"),
            c("port", "name").alias("name"),
            c("port", "kind").alias("kind"),
            c("port", "direction").alias("direction"),
            c("port", "guard_id").alias("guard_id"),
            plans.present(c("length", "length"))?.alias("length"),
            plans
                .lists(vec![
                    c("owner", "support"),
                    c("port", "support"),
                    c("length", "support"),
                ])?
                .alias("supports"),
        ],
    )?;
    let base = guard::settle(
        &mut plans,
        base,
        col("owner_instance_id"),
        col("owner_template_id"),
        col("guard_id"),
        "guard_outcome",
    )
    .await?;
    let base = plans.retain(base).await?;
    let enabled = filter(base.clone(), col("guard_outcome").eq(lit("true")))?;
    let targets = walk::resolve(&mut plans, enabled).await?;
    let products = plans.scan("normalized.domain_products", "state_product")?;
    let targets = join(
        targets,
        products,
        JoinType::Left,
        [col("domain_ids").eq(c("state_product", "domain_ids"))],
    )?;
    plans
        .require(
            &targets,
            c("state_product", "product_id").is_not_null(),
            "port state has no declared ordered domain product",
        )
        .await?;
    let target_fields = [
        "port_id",
        "owner_instance_id",
        "owner_template_id",
        "name",
        "kind",
        "direction",
        "guard_outcome",
        "state_instance_id",
        "state_template_id",
        "index",
        "domain_ids",
    ];
    let mut fields = target_fields.into_iter().map(col).collect::<Vec<_>>();
    fields.push(
        plans
            .present(c("state_product", "product_id"))?
            .alias("product_id"),
    );
    fields.push(
        plans
            .lists(vec![col("supports"), c("state_product", "support")])?
            .alias("supports"),
    );
    let targets = plans.retain(project(targets, fields)?).await?;
    // A single port must select one state template and one ordered domain vector.
    let left = prefix(targets.clone(), "left_state")?;
    let right = prefix(targets.clone(), "right_state")?;
    let pairs = join(
        left,
        right,
        JoinType::Inner,
        [c("left_state", "port_id").eq(c("right_state", "port_id"))],
    )?;
    plans
        .require(
            &pairs,
            c("left_state", "state_template_id")
                .eq(c("right_state", "state_template_id"))
                .and(c("left_state", "domain_ids").eq(c("right_state", "domain_ids"))),
            "port collection has competing state templates or ordered domains",
        )
        .await?;
    let missing_state = pse_catalog::session::output::checked_literal(
        session.registry(),
        &pse_schema::model::FieldContract::payload(
            "state_instance_id",
            pse_schema::model::FieldContract::id(),
            "Absent state target",
        )
        .optional(),
        ScalarValue::FixedSizeBinary(16, None),
    )
    .map_err(error)?;
    let enabled_ports = project(
        targets.clone(),
        [
            col("port_id"),
            col("owner_instance_id").alias("instance_id"),
            col("name"),
            col("kind"),
            col("direction"),
            pse_catalog::session::output::same_field_case(
                targets.schema(),
                array_length(col("domain_ids")).eq(lit(0_u64)),
                col("state_instance_id"),
                missing_state.clone(),
            )
            .map_err(error)?
            .alias("state_instance_id"),
            col("guard_outcome"),
            col("supports"),
        ],
    )?;
    let disabled_ports = project(
        filter(base, col("guard_outcome").eq(lit("false")))?,
        [
            col("port_id"),
            col("owner_instance_id").alias("instance_id"),
            col("name"),
            col("kind"),
            col("direction"),
            missing_state.alias("state_instance_id"),
            col("guard_outcome"),
            col("supports"),
        ],
    )?;
    let mut outputs = BTreeMap::new();
    let ports = plans
        .output(
            "inferred.port_candidates",
            union(vec![enabled_ports, disabled_ports])?,
            false,
        )
        .await?;
    outputs.insert(
        pse_relations::generated::inferred::port_candidates::RELATION_KEY,
        ports,
    );
    let states = plans
        .output(
            "inferred.port_state_candidates",
            project(
                targets.clone(),
                [
                    col("port_id"),
                    col("index").alias("state_index"),
                    col("state_instance_id"),
                    col("supports"),
                ],
            )?,
            false,
        )
        .await?;
    outputs.insert(
        pse_relations::generated::inferred::port_state_candidates::RELATION_KEY,
        states,
    );
    let domains = plans
        .output(
            "inferred.port_state_domain_candidates",
            project(
                targets.clone(),
                [
                    col("port_id"),
                    col("domain_ids"),
                    col("product_id"),
                    col("supports"),
                ],
            )?,
            false,
        )
        .await?;
    outputs.insert(
        pse_relations::generated::inferred::port_state_domain_candidates::RELATION_KEY,
        domains,
    );
    members(&mut plans, targets, &mut outputs).await?;
    Ok(outputs)
}

async fn members(
    plans: &mut Plans<'_>,
    targets: LogicalPlan,
    outputs: &mut BTreeMap<RelationKey, Arc<NativeInput>>,
) -> Result<(), CompilerError> {
    let declarations = plans.scan("normalized.template_port_members", "member")?;
    let members = join(
        targets,
        declarations,
        JoinType::Left,
        [col("state_template_id").eq(c("member", "template_id"))],
    )?;
    plans
        .require(
            &members,
            c("member", "symbol_decl_id").is_not_null(),
            "bound state has no explicit port member",
        )
        .await?;
    let symbols = plans.scan("normalized.template_symbols", "symbol")?;
    let members = join(
        members,
        symbols,
        JoinType::Left,
        [
            c("member", "symbol_decl_id").eq(c("symbol", "symbol_decl_id")),
            col("state_template_id").eq(c("symbol", "template_id")),
        ],
    )?;
    plans
        .require(
            &members,
            c("symbol", "symbol_decl_id").is_not_null(),
            "port member belongs to a different or absent state template",
        )
        .await?;
    let quantities = plans.scan("reference.quantity_types", "quantity")?;
    let members = join(
        members,
        quantities,
        JoinType::Left,
        [c("symbol", "quantity_type_id").eq(c("quantity", "quantity_type_id"))],
    )?;
    plans
        .require(
            &members,
            c("quantity", "quantity_type_id").is_not_null(),
            "port member has no declared quantity type",
        )
        .await?;
    let members = project(
        members,
        [
            col("port_id"),
            col("state_instance_id"),
            col("state_template_id"),
            col("domain_ids"),
            plans.present(c("member", "ordinal"))?.alias("ordinal"),
            plans
                .present(c("member", "symbol_group"))?
                .alias("symbol_group"),
            plans
                .present(c("symbol", "symbol_decl_id"))?
                .alias("symbol_decl_id"),
            plans
                .present(c("symbol", "quantity_type_id"))?
                .alias("quantity_type_id"),
            plans
                .present(c("symbol", "indexed_by"))?
                .alias("axis_names"),
            c("symbol", "guard_id").alias("member_guard"),
            plans
                .lists(vec![
                    col("supports"),
                    c("member", "support"),
                    c("symbol", "support"),
                    c("quantity", "support"),
                ])?
                .alias("supports"),
        ],
    )?;
    let members = guard::settle(
        plans,
        members,
        col("state_instance_id"),
        col("state_template_id"),
        col("member_guard"),
        "member_outcome",
    )
    .await?;
    plans
        .require(
            &members,
            col("member_outcome").eq(lit("true")),
            "state port member is absent in an actual state context",
        )
        .await?;
    let members = plans
        .named_axes(
            members,
            &["port_id", "state_instance_id", "ordinal"],
            "state_instance_id",
            "axis_names",
            "member_domains",
        )
        .await?;
    let fields = [
        "port_id",
        "state_instance_id",
        "ordinal",
        "symbol_group",
        "symbol_decl_id",
        "quantity_type_id",
    ];
    let mut columns = fields.into_iter().map(col).collect::<Vec<_>>();
    columns.push(
        plans
            .ids(vec![col("domain_ids"), col("member_domains")])?
            .alias("domain_ids"),
    );
    columns.push(col("supports"));
    let members = project(members, columns)?;
    let products = plans.scan("normalized.domain_products", "member_product")?;
    let members = join(
        members,
        products,
        JoinType::Left,
        [col("domain_ids").eq(c("member_product", "domain_ids"))],
    )?;
    plans
        .require(
            &members,
            c("member_product", "product_id").is_not_null(),
            "port member has no complete ordered domain product",
        )
        .await?;
    let mut columns = fields.into_iter().map(col).collect::<Vec<_>>();
    columns.extend([
        col("domain_ids"),
        plans
            .present(c("member_product", "product_id"))?
            .alias("product_id"),
        plans
            .lists(vec![col("supports"), c("member_product", "support")])?
            .alias("supports"),
    ]);
    let members = plans.retain(project(members, columns)?).await?;
    // The unique output key detects conflicting ordinal/group, declaration,
    // quantity and domain bindings across states. Equal values retain every source.
    let values = plans
        .output(
            "inferred.port_member_candidates",
            project(
                members.clone(),
                [
                    col("port_id"),
                    col("ordinal"),
                    col("symbol_group"),
                    col("symbol_decl_id"),
                    col("quantity_type_id"),
                    col("supports"),
                ],
            )?,
            false,
        )
        .await?;
    outputs.insert(
        pse_relations::generated::inferred::port_member_candidates::RELATION_KEY,
        values,
    );
    let domains = plans
        .output(
            "inferred.port_member_domain_candidates",
            project(
                members,
                [
                    col("port_id"),
                    col("ordinal"),
                    col("domain_ids"),
                    col("product_id"),
                    col("supports"),
                ],
            )?,
            false,
        )
        .await?;
    outputs.insert(
        pse_relations::generated::inferred::port_member_domain_candidates::RELATION_KEY,
        domains,
    );
    Ok(())
}
