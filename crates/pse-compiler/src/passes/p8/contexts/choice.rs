// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Literal, feature and selected-state default balance choices use exact source joins.
use super::native::{Sources, append, c, error, join, require};
use crate::CompilerError;
use datafusion::{
    functions::{core::expr_fn::get_field, string::expr_fn::starts_with, unicode::expr_fn::substr},
    functions_aggregate::expr_fn::count_distinct,
    functions_nested::expr_fn::array_length,
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, col, lit, when},
};
use pse_catalog::session::scalar::array_element;
use pse_ids::CancellationToken;
use std::ops::Not;

pub(super) async fn apply(
    base: LogicalPlan,
    sources: &mut Sources<'_>,
    cancel: &CancellationToken,
) -> Result<LogicalPlan, CompilerError> {
    let option = array_element(c("law", "options"), lit(1_i64));
    require(
        &base,
        array_length(c("law", "options"))
            .eq(lit(1_u64))
            .and(get_field(option.clone(), "key").eq(lit("balance_type"))),
        "law requires exactly its declared balance_type option",
        sources.session,
        cancel,
    )
    .await?;
    require(
        &base,
        c("contract", "default_state_child")
            .is_null()
            .eq(c("contract", "default_feature_name").is_null()),
        "law default child and feature must be declared together",
        sources.session,
        cancel,
    )
    .await?;
    let base = append(base, [get_field(option, "value").alias("balance_option")])?;
    let is_feature = starts_with(col("balance_option"), lit("self."));
    let features = sources.scan("inferred.instance_features", "feature")?;
    let base = join(
        base,
        features,
        JoinType::Left,
        [
            is_feature.clone(),
            c("feature", "instance_id").eq(c("instance", "instance_id")),
            c("feature", "name").eq(substr(col("balance_option"), lit(6_i64))),
        ],
    )?;
    require(
        &base,
        is_feature.clone().not().or(feature_contract("feature")),
        "law balance feature has no settled value in its declared dictionary",
        sources.session,
        cancel,
    )
    .await?;
    let member = when(
        is_feature,
        get_field(get_field(c("feature", "value"), "enumeration"), "member"),
    )
    .otherwise(col("balance_option"))
    .map_err(error)?;
    let base = append(base, [member.alias("initial_balance")])?;

    // Prospective child rows are joined to actual instances before joining the parent.
    // Disabled children therefore contribute neither values nor positive row support.
    let children = sources.scan("normalized.instance_bindings", "child")?;
    let actual = sources.scan("inferred.instances", "state")?;
    let children = join(
        children,
        actual,
        JoinType::Inner,
        [c("child", "instance_id").eq(c("state", "instance_id"))],
    )?;
    let base = join(
        base,
        children,
        JoinType::Left,
        [
            col("initial_balance").eq(lit("useDefault")),
            c("child", "parent_instance_id").eq(c("instance", "instance_id")),
            c("child", "submodel_name").eq(c("contract", "default_state_child")),
        ],
    )?;
    let packages = sources.scan("normalized.property_packages", "package")?;
    let base = join(
        base,
        packages,
        JoinType::Left,
        [c("child", "property_package_id").eq(c("package", "property_package_id"))],
    )?;
    let definitions = sources.scan("reference.method_specs", "definition")?;
    let base = join(
        base,
        definitions,
        JoinType::Left,
        [c("package", "state_definition_method_id").eq(c("definition", "method_id"))],
    )?;
    let default_feature = sources.scan("inferred.instance_features", "default_feature")?;
    let base = join(
        base,
        default_feature,
        JoinType::Left,
        [
            c("state", "instance_id").eq(c("default_feature", "instance_id")),
            c("default_feature", "name").eq(c("contract", "default_feature_name")),
        ],
    )?;
    let is_default = col("initial_balance").eq(lit("useDefault"));
    let default_value = get_field(
        get_field(c("default_feature", "value"), "enumeration"),
        "member",
    );
    let valid = c("state", "instance_id")
        .is_not_null()
        .and(c("definition", "family").eq(lit("state_definition")))
        .and(c("definition", "template_id").eq(c("state", "template_id")))
        .and(feature_contract("default_feature"))
        .and(default_value.clone().not_eq(lit("useDefault")));
    require(
        &base,
        is_default.clone().not().or(valid),
        "law default requires an actual selected state and a settled nonrecursive balance feature",
        sources.session,
        cancel,
    )
    .await?;
    let value = when(is_default, default_value)
        .otherwise(col("initial_balance"))
        .map_err(error)?;
    let base = append(base, [value.alias("balance_member")])?;
    let settled = LogicalPlanBuilder::from(base.clone())
        .aggregate(
            [
                c("instance", "instance_id"),
                c("law", "law_instance_decl_id"),
            ],
            [count_distinct(col("balance_member")).alias("choice_count")],
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    require(
        &settled,
        col("choice_count").eq(lit(1_i64)),
        "law state collection does not have one common balance choice",
        sources.session,
        cancel,
    )
    .await?;
    let dictionary = sources.scan("reference.schema_enums", "dictionary")?;
    let base = join(
        base,
        dictionary,
        JoinType::Left,
        [
            c("dictionary", "enum_id").eq(c("contract", "balance_enum_id")),
            c("dictionary", "member").eq(col("balance_member")),
        ],
    )?;
    require(
        &base,
        c("dictionary", "enum_id")
            .is_not_null()
            .and(c("dictionary", "deprecated").eq(lit(false))),
        "law balance choice is missing or deprecated in its exact dictionary",
        sources.session,
        cancel,
    )
    .await?;
    Ok(base)
}
fn feature_contract(alias: &str) -> Expr {
    let value = c(alias, "value");
    get_field(value.clone(), "kind")
        .eq(lit("enum"))
        .and(
            get_field(get_field(value.clone(), "enumeration"), "enum_id")
                .eq(c("contract", "balance_enum_id")),
        )
        .and(get_field(get_field(value, "enumeration"), "member").is_not_null())
}
