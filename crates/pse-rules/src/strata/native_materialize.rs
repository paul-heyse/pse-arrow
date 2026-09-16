// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native outcome and provenance projections over settled typed assertions.

use super::{
    native_state::{State, role, spec},
    relational,
};
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion::arrow::{
    array::{ListArray, RecordBatch, new_empty_array},
    buffer::OffsetBuffer,
    datatypes::DataType,
};
use datafusion::functions_aggregate::expr_fn::{array_agg, max};
use datafusion_common::{Column, JoinType, NullEquality, ScalarValue};
use datafusion_expr::{Expr, LogicalPlan, LogicalPlanBuilder, col, lit, when};
use pse_catalog::session::{SnapshotSession, scalar};
use pse_ids::CancellationToken;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    Registry,
    model::{RelationKey, RelationSpec},
};
use std::{collections::BTreeMap, sync::Arc};

impl State {
    pub(super) async fn materialize(
        &mut self,
        registry: &Registry,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<(BTreeMap<RelationKey, FieldCheckedBatch>, Vec<RecordBatch>), RuleError> {
        let current = self.workspace(session, cancel)?;
        let mut output = self
            .facts
            .iter()
            .map(|(key, batch)| (*key, batch.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut outcomes = vec![];
        let mut derivations = vec![];
        for (head_key, assertion_key) in &self.heads {
            let head = spec(registry, *head_key)?;
            let assertions = current.scan_role(&role("assertions", *head_key))?;
            outcomes.push(outcome(assertions.clone(), head, registry)?);
            derivations.push(derivation_heads(assertions, head, registry)?);
            output.insert(*assertion_key, self.assertions[head_key].clone());
        }
        let target = registry
            .relation("inferred.rule_outcomes")
            .ok_or_else(|| internal("outcome relation absent"))?;
        let outcomes = self
            .execute_relation(
                relational::union(outcomes)?,
                target,
                &current,
                registry,
                cancel,
            )
            .await?;
        output.insert(target.key, outcomes);
        let support = registry
            .relation("provenance.rule_support_edges")
            .ok_or_else(|| internal("support relation absent"))?;
        output.insert(support.key, self.support.clone());
        let mut provenance = vec![];
        if let Some(target) = registry.relation("provenance.derivations") {
            let plan = derivation_rows(
                relational::union(derivations)?,
                current.scan_role("__pse_support")?,
                target,
                registry,
                &current,
                None,
            )?;
            provenance.push(
                self.execute_relation(plan, target, &current, registry, cancel)
                    .await?
                    .into_batch(),
            );
        }
        Ok((output, provenance))
    }
}

fn outcome(
    assertions: LogicalPlan,
    head: &RelationSpec,
    registry: &Registry,
) -> Result<LogicalPlan, RuleError> {
    let conflicts = relational::conflicts(assertions.clone(), head, registry)?;
    let mut keys = vec![col("rule_id")];
    keys.extend(head.primary_key.iter().map(|name| col(*name)));
    let grouped = LogicalPlanBuilder::from(assertions)
        .aggregate(
            keys,
            vec![
                max(when(col("truth").eq(lit("true")), lit(1_i64))
                    .otherwise(lit(0_i64))
                    .map_err(engine)?)
                .alias("__yes"),
                max(when(col("truth").eq(lit("false")), lit(1_i64))
                    .otherwise(lit(0_i64))
                    .map_err(engine)?)
                .alias("__no"),
            ],
        )
        .map_err(engine)?
        .alias("__outcome")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let mut expressions = conflicts
        .schema()
        .columns()
        .into_iter()
        .map(Expr::Column)
        .collect::<Vec<_>>();
    expressions.push(lit(true).alias("__conflict"));
    let conflicts = LogicalPlanBuilder::from(conflicts)
        .project(expressions)
        .map_err(engine)?
        .alias("__conflicts")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let plan = LogicalPlanBuilder::from(grouped)
        .join_detailed(
            conflicts,
            JoinType::Left,
            (
                head.primary_key
                    .iter()
                    .map(|name| Column::new(Some("__outcome"), *name))
                    .collect::<Vec<_>>(),
                head.primary_key
                    .iter()
                    .map(|name| Column::new(Some("__conflicts"), *name))
                    .collect::<Vec<_>>(),
            ),
            None,
            NullEquality::NullEqualsNull,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let target = registry
        .relation("inferred.rule_outcomes")
        .ok_or_else(|| internal("outcome relation absent"))?;
    let truth = decision(
        ["conflict", "true", "false", "unknown"],
        target
            .column("truth")
            .ok_or_else(|| internal("truth declaration absent"))?,
        plan.schema(),
        registry,
    )?;
    let reason = decision(
        [
            "value_conflict",
            "asserted",
            "predicate_false",
            "predicate_unknown",
        ],
        target
            .column("reason")
            .ok_or_else(|| internal("reason declaration absent"))?,
        plan.schema(),
        registry,
    )?;
    relational::declared_projection(
        plan,
        target,
        vec![
            col("__outcome.rule_id"),
            relational::id(head.id, registry)?,
            scalar::key(
                head.primary_key
                    .iter()
                    .map(|name| (*name, Expr::Column(Column::new(Some("__outcome"), *name))))
                    .collect(),
            ),
            truth,
            reason,
        ],
        registry,
    )
}

fn decision(
    values: [&str; 4],
    column: &pse_schema::model::FieldContract,
    schema: &datafusion_common::DFSchema,
    registry: &Registry,
) -> Result<Expr, RuleError> {
    let literal = |value: &str| {
        relational::declared_literal(registry, column, ScalarValue::Utf8(Some(value.to_owned())))
    };
    let conditions = [
        col("__conflicts.__conflict").is_true(),
        col("__outcome.__yes").gt(lit(0_i64)),
        col("__outcome.__no").gt(lit(0_i64)),
    ];
    let mut expression = literal(values[3])?;
    for (condition, value) in conditions.into_iter().zip(values).rev() {
        expression = pse_catalog::session::output::same_field_case(
            schema,
            condition,
            literal(value)?,
            expression,
        )
        .map_err(engine)?;
    }
    Ok(expression)
}

fn derivation_heads(
    assertions: LogicalPlan,
    head: &RelationSpec,
    registry: &Registry,
) -> Result<LogicalPlan, RuleError> {
    LogicalPlanBuilder::from(assertions)
        .project(vec![
            col("assertion_id").alias("derivation_id"),
            relational::id(head.id, registry)?.alias("relation_id"),
            relational::key(head).alias("row_key"),
            col("rule_id"),
        ])
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}

pub(super) fn derivation_rows(
    heads: LogicalPlan,
    support: LogicalPlan,
    target: &RelationSpec,
    registry: &Registry,
    session: &SnapshotSession,
    pass: Option<pse_ids::SemanticId>,
) -> Result<LogicalPlan, RuleError> {
    let members = derivation_members(support, session)?;
    let heads = LogicalPlanBuilder::from(heads)
        .alias("__heads")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let plan = LogicalPlanBuilder::from(heads)
        .join_detailed(
            members,
            JoinType::Left,
            (
                vec![Column::new(Some("__heads"), "derivation_id")],
                vec![Column::new(Some("__members"), "assertion_id")],
            ),
            None,
            NullEquality::NullEqualsNull,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let schema = pse_schema::arrow::relation_schema(registry, target)
        .map_err(|error| internal(error.to_string()))?;
    let field = schema
        .field_with_name("supporting")
        .map_err(|error| internal(error.to_string()))?;
    let DataType::List(child) = field.data_type() else {
        return Err(internal("derivation support must be a declared list"));
    };
    let empty = ListArray::try_new(
        Arc::clone(child),
        OffsetBuffer::new(vec![0_i32, 0].into()),
        new_empty_array(child.data_type()),
        None,
    )
    .map_err(|error| internal(error.to_string()))?;
    let mut branches = Vec::new();
    for present in [true, false] {
        let branch = LogicalPlanBuilder::from(plan.clone())
            .filter(if present {
                col("__members.supporting").is_not_null()
            } else {
                col("__members.supporting").is_null()
            })
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        let branch = scalar::refine_filtered_fields(branch).map_err(engine)?;
        let template = relational::declared_literal(
            registry,
            target
                .column("supporting")
                .ok_or_else(|| internal("support field absent"))?,
            ScalarValue::List(Arc::new(empty.clone())),
        )?;
        let supporting = if present {
            session
                .scalar_function("pse_list_field")?
                .call(vec![col("__members.supporting"), template])
        } else {
            template
        };
        // The explicit native scalar invokes Arrow's checked list constructor;
        // DataFusion's type-only CAST eligibility cannot establish this residual.
        branches.push(relational::declared_projection(
            branch,
            target,
            vec![
                col("__heads.derivation_id"),
                col("__heads.relation_id"),
                col("__heads.row_key"),
                col("__heads.rule_id"),
                lit(ScalarValue::FixedSizeBinary(
                    16,
                    pass.map(|id| id.as_bytes().to_vec()),
                )),
                supporting,
                lit(ScalarValue::FixedSizeBinary(32, None)),
                lit(ScalarValue::FixedSizeBinary(32, None)),
            ],
            registry,
        )?);
    }
    relational::union(branches)
}

fn derivation_members(
    support: LogicalPlan,
    session: &SnapshotSession,
) -> Result<LogicalPlan, RuleError> {
    let support = LogicalPlanBuilder::from(support)
        .filter(col("input_key").is_not_null())
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let support = scalar::refine_filtered_fields(support).map_err(engine)?;
    let members = LogicalPlanBuilder::from(support)
        .project(vec![
            col("assertion_id"),
            col("input_relation_id"),
            col("input_key"),
        ])
        .map_err(engine)?
        .distinct()
        .map_err(engine)?
        .aggregate(
            vec![col("assertion_id")],
            vec![
                array_agg(scalar::named_fields(vec![
                    lit("relation_id"),
                    col("input_relation_id"),
                    lit("row_key"),
                    col("input_key"),
                ]))
                .alias("supporting"),
            ],
        )
        .map_err(engine)?
        // Sorting each completed list preserves the declared lexicographic support
        // order without compacting a nested scalar for every ordered-aggregate row.
        .project(vec![
            col("assertion_id"),
            session.scalar_function("array_sort")?.call(vec![
                col("supporting"), lit("ASC"), lit("NULLS FIRST"),
            ]).alias("supporting"),
        ])
        .map_err(engine)?
        .alias("__members")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    Ok(members)
}
