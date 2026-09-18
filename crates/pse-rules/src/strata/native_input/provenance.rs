// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native support grouping shared with inference derivation materialization.

use super::{
    BTreeMap, CancellationToken, ColumnRole, FieldCheckedBatch, LogicalPlanBuilder, RuleError, col,
    declare_relation_output, derivation_id, engine, internal, relational,
};
use crate::strata::native_materialize;
use datafusion_common::{Column, JoinType, NullEquality, ScalarValue};

/// Materialize support using the current native invocation; retain only the result buffers.
pub(super) async fn materialize(
    checked: &FieldCheckedBatch,
    support_mapping: &FieldCheckedBatch,
    relation: pse_schema::model::RelationKey,
    pass_id: pse_ids::SemanticId,
    owner: &pse_catalog::session::SnapshotSession,
    cancel: &CancellationToken,
) -> Result<FieldCheckedBatch, RuleError> {
    let registry = owner.registry();
    let source = registry
        .relation(&relation.qualified_name())
        .ok_or_else(|| internal("native derivation source absent"))?;
    let target = registry
        .relation("provenance.derivations")
        .ok_or_else(|| internal("native derivation contract absent"))?;
    let session = owner.with_checked_role_inputs(
        BTreeMap::from([
            ("__native_derivation_values".to_owned(), checked.clone()),
            (
                "__native_derivation_support".to_owned(),
                support_mapping.clone(),
            ),
        ]),
        cancel,
    )?;
    let raw = session.scan_role("__native_derivation_values")?;
    let derivation =
        if source.columns.iter().any(|column| {
            column.role() == ColumnRole::Provenance && column.name() == "derivation_id"
        }) {
            col("derivation_id")
        } else {
            derivation_id(relational::key(source), pass_id, registry)?.alias("derivation_id")
        };
    let null_rule = relational::declared_literal(
        registry,
        target
            .column("rule_id")
            .ok_or_else(|| internal("rule field absent"))?,
        ScalarValue::FixedSizeBinary(16, None),
    )?;
    let heads = LogicalPlanBuilder::from(raw)
        .project(vec![
            derivation,
            relational::id(source.id, registry)?.alias("relation_id"),
            relational::key(source).alias("row_key"),
            null_rule.alias("rule_id"),
        ])
        .map_err(engine)?
        .alias("__native_heads")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let mapping = LogicalPlanBuilder::from(session.scan_role("__native_derivation_support")?)
        .alias("__native_map")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let support = LogicalPlanBuilder::from(heads.clone())
        .join_detailed(
            mapping,
            JoinType::Inner,
            (
                vec![Column::new(Some("__native_heads"), "row_key")],
                vec![Column::new(Some("__native_map"), "output_key")],
            ),
            None,
            NullEquality::NullEqualsNull,
        )
        .map_err(engine)?
        .project(vec![
            col("__native_heads.derivation_id").alias("assertion_id"),
            col("__native_map.input_relation_id"),
            col("__native_map.input_key"),
        ])
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let plan = native_materialize::derivation_rows(
        heads,
        support,
        target,
        registry,
        &session,
        Some(pass_id),
        cancel,
    )?;
    let plan = declare_relation_output(plan, registry, target).map_err(engine)?;
    Ok(session
        .prepare_rule_plan(plan, cancel)?
        .execute(cancel)
        .await?
        .into_checked_relation(registry, target, cancel)?)
}
