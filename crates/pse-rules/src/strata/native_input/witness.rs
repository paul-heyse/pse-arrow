// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Witness membership and support-map projection are native typed joins.

use super::{NativeWitness, require_empty};
use crate::{
    RuleError,
    errmap::{engine, internal},
    strata::{RuleInputLocation, admission, relational},
};
use datafusion::{arrow::array::RecordBatch, functions_aggregate::expr_fn::count};
use datafusion_common::{Column, JoinType, NullEquality, ScalarValue};
use datafusion_expr::{Expr, LogicalPlan, LogicalPlanBuilder, col, lit};
use pse_catalog::session::{SnapshotSession, scalar};
use pse_ids::CancellationToken;
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::RelationSpec;
use std::{collections::BTreeMap, sync::Arc};

impl NativeWitness {
    pub(super) fn validate(
        &self,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<(), RuleError> {
        if self.port.is_empty() {
            return Err(internal("native witness port is empty"));
        }
        let spec = session
            .registry()
            .relation(&self.input.relation.qualified_name())
            .filter(|spec| spec.key == self.input.relation)
            .ok_or_else(|| internal("native source declaration absent"))?;
        if !self.key_columns.is_empty() && self.key_columns.len() != spec.primary_key.len() {
            return Err(internal(
                "native witness does not carry the complete source key",
            ));
        }
        match &self.input.location {
            RuleInputLocation::Facts(_) => {
                admission::validate_immutable_owner(&self.input, session.registry())
            }
            RuleInputLocation::Completed(input) => input.validate_owner(spec.key, session, cancel),
            RuleInputLocation::Native(input) => input.validate_owner(spec.key, session, cancel),
            RuleInputLocation::Workspace => Err(internal(
                "native witnesses require retained immutable source owners",
            )),
        }
    }
    fn checked(&self) -> Result<FieldCheckedBatch, RuleError> {
        match &self.input.location {
            RuleInputLocation::Facts(facts) => Ok(facts.checked().clone()),
            RuleInputLocation::Completed(input) => Ok(input.checked().clone()),
            RuleInputLocation::Native(input) => Ok(input.checked().clone()),
            RuleInputLocation::Workspace => {
                Err(internal("native witness has no immutable checked fields"))
            }
        }
    }
}

pub(super) fn duplicate_keys(
    plan: LogicalPlan,
    target: &RelationSpec,
) -> Result<LogicalPlan, RuleError> {
    if target.primary_key.is_empty() {
        return Err(internal("native source needs a declared key"));
    }
    let alias = unused_name(
        plan.schema()
            .fields()
            .iter()
            .map(|field| field.name().as_str()),
        "__native_key_count",
    );
    LogicalPlanBuilder::from(plan)
        .aggregate(
            target.primary_key.iter().map(|name| col(*name)),
            vec![count(lit(1_u64)).alias(&alias)],
        )
        .map_err(engine)?
        .filter(col(&alias).gt(lit(1_i64)))
        .map_err(engine)?
        .build()
        .map_err(engine)
}

fn unused_name<'a>(names: impl Iterator<Item = &'a str>, prefix: &str) -> String {
    let names = names.collect::<std::collections::BTreeSet<_>>();
    let mut value = prefix.to_owned();
    while names.contains(value.as_str()) {
        value.push('_');
    }
    value
}

fn require_key_values(
    plan: LogicalPlan,
    source_plan: &LogicalPlan,
    witness: &NativeWitness,
    source: &RelationSpec,
    session: &SnapshotSession,
) -> Result<LogicalPlan, RuleError> {
    let mut required = std::collections::BTreeSet::new();
    for (actual, expected) in witness.key_columns.iter().zip(&source.primary_key) {
        if plan
            .schema()
            .field_with_unqualified_name(actual)
            .map_err(engine)?
            .is_nullable()
            && !source_plan
                .schema()
                .field_with_unqualified_name(expected)
                .map_err(engine)?
                .is_nullable()
        {
            required.insert(actual.as_str());
        }
    }
    if required.is_empty() {
        return Ok(plan);
    }
    let function = session.scalar_function("pse_require_nonnull")?;
    let expressions = plan
        .schema()
        .columns()
        .into_iter()
        .map(|column| {
            if required.contains(column.name.as_str()) {
                function
                    .call(vec![Expr::Column(column.clone())])
                    .alias(column.name)
            } else {
                Expr::Column(column)
            }
        })
        .collect::<Vec<_>>();
    LogicalPlanBuilder::from(plan)
        .project(expressions)
        .map_err(engine)?
        .build()
        .map_err(engine)
}

pub(super) async fn mapping(
    raw: LogicalPlan,
    output: &RelationSpec,
    columns: &[(String, String)],
    witnesses: &[NativeWitness],
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<FieldCheckedBatch, RuleError> {
    let registry = session.registry();
    let target = registry
        .relation("provenance.constructed_supports")
        .ok_or_else(|| internal("native support mapping schema absent"))?;
    let mut results = Vec::new();
    for (position, witness) in witnesses.iter().enumerate() {
        cancel
            .checkpoint()
            .map_err(pse_catalog::CatalogError::from)?;
        let role = format!("__native_witness:{}:{position}", output.id);
        let mut session = session.with_checked_role_inputs(
            BTreeMap::from([(role.clone(), witness.checked()?)]),
            cancel,
        )?;
        if let RuleInputLocation::Native(input) = &witness.input.location {
            session = session.with_checked_role_inputs(
                BTreeMap::from([(
                    "__native_nested_support".to_owned(),
                    input.support_mapping().clone(),
                )]),
                cancel,
            )?;
        }
        let source = registry
            .relation(&witness.input.relation.qualified_name())
            .ok_or_else(|| internal("native witness schema absent"))?;
        let source_plan = session.scan_role(&role)?;
        let plan = select_witness(&raw, witness, source_plan, source, &session, cancel).await?;
        let direct = direct_mapping(plan, output, columns, witness, source, registry, false)?;
        let direct = if witness.key_columns.is_empty() {
            let global = direct_mapping(
                LogicalPlanBuilder::empty(true).build().map_err(engine)?,
                output,
                columns,
                witness,
                source,
                registry,
                true,
            )?;
            relational::union(vec![direct, global])?
        } else {
            direct
        };
        let mapped = if matches!(&witness.input.location, RuleInputLocation::Native(_)) {
            expand_native(direct, target, &session)?
        } else {
            direct
        };
        let mapped =
            pse_catalog::session::output::declare_relation_output(mapped, registry, target)
                .map_err(engine)?;
        let collisions = relational::identity_collisions(mapped.clone(), "mapping_id")?;
        require_empty(
            collisions,
            &session,
            cancel,
            "native support identity collision",
        )
        .await?;
        let complete = session
            .prepare_rule_plan(mapped, cancel)?
            .execute(cancel)
            .await?;
        results.push(complete.checked_relation(registry, target, cancel)?);
    }
    if results.is_empty() {
        return Ok(FieldCheckedBatch::admit(
            registry,
            target,
            RecordBatch::new_empty(Arc::new(
                pse_schema::arrow::relation_schema(registry, target)
                    .map_err(pse_relations::RelationError::from)?,
            )),
        )?);
    }
    Ok(FieldCheckedBatch::concat_reserved(
        registry,
        target,
        &results,
        session.reserver(),
        cancel,
    )?)
}

async fn select_witness(
    raw: &LogicalPlan,
    witness: &NativeWitness,
    source_plan: LogicalPlan,
    source: &RelationSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<LogicalPlan, RuleError> {
    let selected = if let Some(predicate) = &witness.when {
        let filtered = LogicalPlanBuilder::from(raw.clone())
            .filter(predicate.clone())
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        scalar::refine_filtered_fields(filtered).map_err(engine)?
    } else {
        raw.clone()
    };
    let plan = if witness.key_columns.is_empty() {
        LogicalPlanBuilder::from(selected)
            .alias("__native_values")
            .map_err(engine)?
            .build()
            .map_err(engine)?
    } else {
        let selected = require_key_values(selected, &source_plan, witness, source, session)?;
        for (actual, expected) in witness.key_columns.iter().zip(&source.primary_key) {
            let actual = selected
                .schema()
                .field_with_unqualified_name(actual)
                .map_err(engine)?;
            let expected = source_plan
                .schema()
                .field_with_unqualified_name(expected)
                .map_err(engine)?;
            pse_catalog::session::output::check_field_output(actual, expected).map_err(engine)?;
        }
        let raw = LogicalPlanBuilder::from(selected)
            .alias("__native_values")
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        let source_plan = LogicalPlanBuilder::from(source_plan)
            .alias("__native_source")
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        let keys = (
            witness
                .key_columns
                .iter()
                .map(|name| Column::new(Some("__native_values"), name))
                .collect::<Vec<_>>(),
            source
                .primary_key
                .iter()
                .map(|name| Column::new(Some("__native_source"), *name))
                .collect::<Vec<_>>(),
        );
        let missing = LogicalPlanBuilder::from(raw.clone())
            .join_detailed(
                source_plan,
                JoinType::LeftAnti,
                keys,
                None,
                NullEquality::NullEqualsNull,
            )
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        require_empty(
            missing,
            session,
            cancel,
            "native witness key is absent from its actual immutable source",
        )
        .await?;
        raw
    };
    Ok(plan)
}

fn direct_mapping(
    plan: LogicalPlan,
    output: &RelationSpec,
    columns: &[(String, String)],
    witness: &NativeWitness,
    source: &RelationSpec,
    registry: &pse_schema::Registry,
    global: bool,
) -> Result<LogicalPlan, RuleError> {
    let target = registry
        .relation("provenance.constructed_supports")
        .ok_or_else(|| internal("native support mapping schema absent"))?;
    let scope = witness.key_columns.is_empty();
    let key = |names: Vec<(&str, Expr)>| {
        if scope {
            lit(ScalarValue::Utf8(None))
        } else {
            scalar::key(names)
        }
    };
    let output_key = if global {
        lit(ScalarValue::Utf8(None))
    } else {
        scalar::key(
            output
                .primary_key
                .iter()
                .map(|name| {
                    let source = columns
                        .iter()
                        .find(|(target, _)| target == name)
                        .map(|(_, source)| source.as_str())
                        .ok_or_else(|| internal("native output key projection absent"))?;
                    Ok((
                        *name,
                        Expr::Column(Column::new(Some("__native_values"), source)),
                    ))
                })
                .collect::<Result<Vec<_>, RuleError>>()?,
        )
    };
    let input_key = key(source
        .primary_key
        .iter()
        .zip(&witness.key_columns)
        .map(|(name, source)| {
            (
                *name,
                Expr::Column(Column::new(Some("__native_values"), source)),
            )
        })
        .collect());
    let kind = super::super::native_support::kind(&witness.input.location);
    let mut values = vec![
        relational::id(output.id, registry)?,
        output_key,
        lit(witness.port.clone()),
        relational::id(source.id, registry)?,
        super::super::native_support::selection(&witness.input.location, target, registry)?,
        relational::declared_literal(
            registry,
            target
                .column("support_kind")
                .ok_or_else(|| internal("support kind absent"))?,
            ScalarValue::Utf8(Some(if scope { "absence" } else { kind }.to_owned())),
        )?,
        input_key,
    ];
    let id = scalar::named_id(
        relational::id(output.id, registry)?,
        relational::frame(values.clone()),
    );
    values.insert(0, id);
    let plan = relational::declared_projection(plan, target, values, registry)?;
    LogicalPlanBuilder::from(plan)
        .distinct()
        .map_err(engine)?
        .build()
        .map_err(engine)
}

fn expand_native(
    direct: LogicalPlan,
    target: &RelationSpec,
    session: &SnapshotSession,
) -> Result<LogicalPlan, RuleError> {
    let role = "__native_nested_support".to_owned();
    let left = LogicalPlanBuilder::from(direct)
        .alias("__direct")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let right = LogicalPlanBuilder::from(session.scan_role(&role)?)
        .alias("__nested")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let plan = LogicalPlanBuilder::from(left)
        .join_detailed(
            right,
            JoinType::Inner,
            (
                vec![Column::new(Some("__direct"), "input_key")],
                vec![Column::new(Some("__nested"), "output_key")],
            ),
            None,
            NullEquality::NullEqualsNull,
        )
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let mut values = vec![
        col("__direct.output_relation_id"),
        col("__direct.output_key"),
        col("__nested.input_port"),
        col("__nested.input_relation_id"),
        col("__nested.input_selection"),
        col("__nested.support_kind"),
        col("__nested.input_key"),
    ];
    let id = scalar::named_id(
        col("__direct.output_relation_id"),
        relational::frame(values.clone()),
    );
    values.insert(0, id);
    relational::declared_projection(plan, target, values, session.registry())
}
