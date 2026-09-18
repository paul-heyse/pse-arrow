// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native law-context framing over complete admitted stage inputs.
mod axes;
mod choice;
mod guards;
mod native;

use crate::{AlgorithmContext, AlgorithmInputs, CompilerError};
use datafusion::{
    arrow::datatypes::DataType,
    functions::encoding::expr_fn::encode,
    logical_expr::{Expr, JoinType, Operator, col, lit},
};
use native::{Sources, append, c, filter, invalid, join, require};
use pse_catalog::session::scalar;
use pse_rules::strata::native_input::NativeInput;
use pse_schema::model::{AlgorithmSpec, RelationKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[expect(
    clippy::too_many_lines,
    reason = "build keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) async fn build(
    spec: &AlgorithmSpec,
    ctx: &AlgorithmContext<'_>,
    inputs: &AlgorithmInputs,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    inputs.validate(spec, ctx.registry)?;
    let session = ctx
        .session
        .select_inputs(&BTreeSet::new(), ctx.cancel)?
        .with_checked_workspace(inputs.checked_rows(ctx.registry)?, ctx.cancel)?;
    let mut sources = Sources::new(&session, inputs, spec);
    let instances = sources.scan("inferred.instances", "instance")?;
    let laws = sources.scan("normalized.template_law_instances", "law")?;
    let base = join(
        instances,
        laws,
        JoinType::Inner,
        [c("instance", "template_id").eq(c("law", "template_id"))],
    )?;
    let base = guards::apply(base, &mut sources, ctx.cancel).await?;
    let contracts = sources.scan("normalized.template_law_contracts", "contract")?;
    let base = join(
        base,
        contracts,
        JoinType::Left,
        [c("law", "law_instance_decl_id").eq(c("contract", "law_instance_decl_id"))],
    )?;
    require(
        &base,
        c("contract", "law_instance_decl_id").is_not_null(),
        "law lacks its exact declared contract",
        &session,
        ctx.cancel,
    )
    .await?;
    let base = choice::apply(base, &mut sources, ctx.cancel).await?;
    let declarations = sources.scan("normalized.template_scopes", "scope_declaration")?;
    let base = join(
        base,
        declarations,
        JoinType::Left,
        [
            c("instance", "template_id").eq(c("scope_declaration", "template_id")),
            c("law", "scope").eq(c("scope_declaration", "name")),
        ],
    )?;
    let scopes = sources.scan("inferred.scope_bindings", "scope")?;
    let base = join(
        base,
        scopes,
        JoinType::Left,
        [
            c("instance", "instance_id").eq(c("scope", "owner_instance_id")),
            c("scope_declaration", "scope_id").eq(c("scope", "scope_decl_id")),
        ],
    )?;
    require(
        &base,
        c("scope", "scope_id").is_not_null(),
        "law scope lacks an actual instance binding",
        &session,
        ctx.cancel,
    )
    .await?;
    let quantities = sources.scan("reference.quantity_types", "quantity")?;
    let base = join(
        base,
        quantities,
        JoinType::Left,
        [c("contract", "quantity_type_id").eq(c("quantity", "quantity_type_id"))],
    )?;
    require(
        &base,
        c("quantity", "quantity_type_id").is_not_null(),
        "law has no actual complete quantity contract",
        &session,
        ctx.cancel,
    )
    .await?;
    let base = axes::apply(base, &mut sources, ctx.cancel).await?;
    let application = scalar::named_id(
        c("instance", "instance_id"),
        concat(
            lit("pse:law-application:v1:"),
            encode(c("law", "law_instance_decl_id"), lit("hex")),
        ),
    );
    let base = append(base, [application.alias("application_id")])?;
    let mut values = BTreeMap::from([
        ("application_id", col("application_id")),
        ("law_instance_decl_id", c("law", "law_instance_decl_id")),
        ("law_template_id", c("law", "law_template_id")),
        ("owner_instance_id", c("instance", "instance_id")),
        ("scope_id", c("scope", "scope_id")),
        ("product_id", c("product", "product_id")),
        ("quantity_type_id", c("contract", "quantity_type_id")),
        ("basis_id", c("quantity", "basis_id")),
        ("balance_enum_id", c("contract", "balance_enum_id")),
        ("balance_member", col("balance_member")),
        (
            "derivation_id",
            scalar::named_id(col("application_id"), lit("P8:context")),
        ),
    ]);
    values.insert("coordinates", c("contract", "coordinates"));
    let context = ctx
        .registry
        .relation("inferred.law_contexts")
        .ok_or_else(|| invalid("law contexts undeclared"))?;
    let expressions = context
        .columns
        .iter()
        .map(|field| {
            let value = values
                .remove(field.name())
                .ok_or_else(|| invalid(format!("law context column {} omitted", field.name())))?;
            let value = if field.nullable() {
                value
            } else {
                session
                    .scalar_function("pse_require_nonnull")?
                    .call(vec![value])
            };
            Ok(value.alias(format!("output:{}", field.name())))
        })
        .collect::<Result<Vec<_>, CompilerError>>()?;
    if !values.is_empty() {
        return Err(invalid("law context has undeclared result fields"));
    }
    let base = append(base, expressions)?;
    // Both declared output projections consume this one observed native completion.
    let witnesses = sources.witnesses(&base);
    let complete = session
        .prepare_rule_plan(base, ctx.cancel)?
        .execute(ctx.cancel)
        .await?;
    let owner = session.with_computation_roles(
        BTreeMap::from([("p8-law-framing".to_owned(), complete)]),
        ctx.cancel,
    )?;
    let base = owner.scan_computation_role("p8-law-framing")?;
    let columns = context
        .columns
        .iter()
        .map(|field| (field.name().to_owned(), format!("output:{}", field.name())))
        .collect();
    let contexts = NativeInput::build(
        base.clone(),
        context.key,
        spec.id,
        columns,
        witnesses.clone(),
        &owner,
        ctx.cancel,
    )
    .await?;
    let axis = ctx
        .registry
        .relation("inferred.law_axes")
        .ok_or_else(|| invalid("law axes undeclared"))?;
    let axis_plan = filter(base, col("axis_position").is_not_null())?;
    let ordinal = Expr::Cast(datafusion::logical_expr::expr::Cast::new(
        Box::new(col("axis_position")),
        DataType::Int64,
    ));
    let ordinal = owner
        .scalar_function("pse_require_nonnull")?
        .call(vec![ordinal]);
    let ordinal_text = Expr::Cast(datafusion::logical_expr::expr::Cast::new(
        Box::new(ordinal.clone()),
        DataType::Utf8,
    ));
    let axis_plan = append(
        axis_plan,
        [
            ordinal.alias("position"),
            owner
                .scalar_function("pse_require_nonnull")?
                .call(vec![col("axis_domain")])
                .alias("domain_id"),
            owner
                .scalar_function("pse_require_nonnull")?
                .call(vec![scalar::named_id(
                    col("output:application_id"),
                    concat(lit("pse:law-axis:v1:"), ordinal_text),
                )])
                .alias("bound_index_id"),
        ],
    )?;
    let columns = axis
        .columns
        .iter()
        .map(|field| {
            (
                field.name().to_owned(),
                if matches!(field.name(), "application_id" | "derivation_id") {
                    format!("output:{}", field.name())
                } else {
                    field.name().to_owned()
                },
            )
        })
        .collect();
    let axis_output = NativeInput::build(
        axis_plan, axis.key, spec.id, columns, witnesses, &owner, ctx.cancel,
    )
    .await?;
    Ok(BTreeMap::from([
        (context.key, contexts),
        (axis.key, axis_output),
    ]))
}
fn concat(left: Expr, right: Expr) -> Expr {
    Expr::BinaryExpr(datafusion::logical_expr::expr::BinaryExpr::new(
        Box::new(left),
        Operator::StringConcat,
        Box::new(right),
    ))
}
