// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ordered named-axis binding, shared by products and port/member construction.
use super::{
    CompilerError, Expr, JoinType, LogicalPlan, LogicalPlanBuilder, Plans, append, array_element,
    array_length, c, coalesce, col, error, explode, join, lit, prefix, project, scalar,
};
use datafusion::{functions_nested::expr_fn::flatten, logical_expr::ExprFunctionExt};
use pse_catalog::session::aggregate::array_agg;

impl Plans<'_> {
    /// Bind every ordered axis name against its actual owner's domain relation.
    /// The retained source tokens carry positive axis evidence through aggregation.
    /// Keys must uniquely identify a row in the caller's native construction.
    pub(crate) async fn named_axes(
        &mut self,
        base: LogicalPlan,
        keys: &[&str],
        owner: &str,
        names: &str,
        factors: &str,
    ) -> Result<LogicalPlan, CompilerError> {
        let expanded = explode(base.clone(), col(names), "axis_position")?;
        let binding = self.scan("normalized.instance_domain_bindings", "axis_binding")?;
        let expanded = join(
            expanded,
            binding,
            JoinType::Left,
            [
                col(owner).eq(c("axis_binding", "instance_id")),
                array_element(col(names), col("axis_position") + lit(1_i64))
                    .eq(c("axis_binding", "domain_name")),
            ],
        )?;
        self.require(
            &expanded,
            c("axis_binding", "domain_id").is_not_null(),
            "declared axis has no actual owner/name domain binding",
        )
        .await?;
        let expanded = append(
            expanded,
            [self
                .present(c("axis_binding", "domain_id"))?
                .alias("axis_domain")],
        )?;
        let aggregate = LogicalPlanBuilder::from(expanded)
            .aggregate(
                keys.iter().map(|key| col(*key)),
                [
                    array_agg(col("axis_domain"))
                        .order_by(vec![col("axis_position").sort(true, false)])
                        .build()
                        .map_err(error)?
                        .alias("ordered_domains"),
                    array_agg(c("axis_binding", "support")).alias("axis_supports"),
                ],
            )
            .and_then(LogicalPlanBuilder::build)
            .map_err(error)?;
        let aggregate = prefix(aggregate, "bound_axes")?;
        let joined = join(
            base.clone(),
            aggregate,
            JoinType::Left,
            keys.iter().map(|key| col(*key).eq(c("bound_axes", key))),
        )?;
        let domains = coalesce(vec![
            c("bound_axes", "ordered_domains"),
            scalar::id_list(vec![]),
        ]);
        self.require(
            &joined,
            array_length(domains.clone()).eq(array_length(col(names))),
            "ordered axis binding is missing or ambiguous",
        )
        .await?;
        let mut columns = base
            .schema()
            .columns()
            .into_iter()
            .filter(|field| field.name != "supports")
            .map(Expr::Column)
            .collect::<Vec<_>>();
        columns.push(domains.alias(factors));
        columns.push(
            self.lists(vec![
                col("supports"),
                flatten(c("bound_axes", "axis_supports")),
            ])?
            .alias("supports"),
        );
        project(joined, columns)
    }
}
