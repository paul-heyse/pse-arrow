// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry declarations become one native union of exact diagnostic projections.

mod native;
#[cfg(test)]
mod tests;

use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion::arrow::array::builder::{ListBuilder, make_builder};
use datafusion::arrow::datatypes::DataType;
use datafusion::common::ScalarValue;
use datafusion::logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, Projection, Sort, col, lit};
use pse_columnar::CancellationToken;
use pse_engine::session::{
    EngineSession,
    output::{checked_literal, declare_relation_output},
    scalar,
};
use pse_schema::{
    Registry,
    model::{InvariantSpec, RelationKey, RelationSpec, SnapshotClass},
};
use std::{collections::BTreeSet, sync::Arc};

use super::InvariantScope;

pub(super) async fn compile(
    candidates: &BTreeSet<RelationKey>,
    session: &EngineSession,
    registry: &Registry,
    scope: InvariantScope<'_>,
    cancel: &CancellationToken,
) -> Result<(Option<LogicalPlan>, usize), RuleError> {
    let branches = compile_individual(candidates, session, registry, scope, cancel).await?;
    let count = branches.len();
    Ok((
        combine_findings(
            branches.into_iter().map(|(_, plan)| plan).collect(),
            registry,
        )?,
        count,
    ))
}

/// Keep each obligation's identity while sharing native query/field admission.
pub(super) async fn compile_individual(
    candidates: &BTreeSet<RelationKey>,
    session: &EngineSession,
    registry: &Registry,
    scope: InvariantScope<'_>,
    cancel: &CancellationToken,
) -> Result<Vec<(pse_ids::SemanticId, LogicalPlan)>, RuleError> {
    let mut branches = native::compile(candidates, session, registry, scope, cancel)?;
    let mut declared = Vec::new();
    for invariant in registry.invariants() {
        if let InvariantScope::Required(selected) = scope
            && !selected.contains(&invariant.id)
        {
            continue;
        }
        cancel
            .checkpoint()
            .map_err(pse_relations::RelationError::from)?;
        let target = registry
            .relation(&invariant.relation)
            .ok_or_else(|| internal("invariant target absent"))?;
        if !candidates.contains(&target.key) || !applies(scope, target) {
            continue;
        }
        if let InvariantScope::Affected(changed) = scope
            && !invariant.inputs.iter().any(|name| {
                registry
                    .relation(name)
                    .is_some_and(|spec| changed.contains(&spec.key))
            })
        {
            continue;
        }
        if scope == InvariantScope::SidecarRelation
            && invariant
                .inputs
                .iter()
                .any(|name| name != &invariant.relation)
        {
            continue;
        }
        let inputs = invariant
            .inputs
            .iter()
            .map(|name| {
                let spec = registry
                    .relation(name)
                    .ok_or_else(|| internal("invariant dependency undeclared"))?;
                if !candidates.contains(&spec.key) {
                    return Err(internal(format!(
                        "invariant {} lacks explicit dependency {}",
                        invariant.qualified_name(),
                        spec.key
                    )));
                }
                Ok(spec.key)
            })
            .collect::<Result<Vec<_>, RuleError>>()?;
        declared.push((invariant, target, inputs));
    }
    let queries = declared
        .iter()
        .map(|(invariant, _, inputs)| (invariant.query.as_str(), inputs.as_slice()))
        .collect::<Vec<_>>();
    let plans = session.bind_declared_queries(&queries, cancel).await?;
    for ((invariant, target, _), plan) in declared.into_iter().zip(plans) {
        let keys = project_query_keys(plan, invariant, target, registry)?;
        branches.push((
            invariant.id,
            finding(keys, invariant.into(), "violation", invariant.doc, registry)?,
        ));
    }
    if let InvariantScope::Required(selected) = scope
        && (branches.len() != selected.len()
            || branches.iter().map(|(id, _)| *id).collect::<BTreeSet<_>>() != *selected)
    {
        return Err(internal(
            "required invariant declaration or target binding is absent",
        ));
    }
    // All findings have the same declared result and exact immutable source scope.
    // Derive their shared producers together instead of rewalking the complete
    // compiler graph for every individual check and diagnostic projection.
    if branches.is_empty() {
        return Ok(Vec::new());
    }
    let target = registry
        .relation("runtime.diagnostics_findings")
        .ok_or_else(|| internal("diagnostic output is undeclared"))?;
    let (ids, plans): (Vec<_>, Vec<_>) = branches.into_iter().unzip();
    ids.into_iter()
        .zip(session.derive_plan_fields_many(&plans, cancel)?)
        .map(|(id, plan)| {
            Ok((
                id,
                declare_relation_output(plan, registry, target).map_err(engine)?,
            ))
        })
        .collect()
}

fn project_query_keys(
    plan: LogicalPlan,
    invariant: &InvariantSpec,
    target: &RelationSpec,
    registry: &Registry,
) -> Result<LogicalPlan, RuleError> {
    let mut projection = Vec::with_capacity(invariant.key_columns.len());
    for name in &invariant.key_columns {
        let expected = target
            .column(name)
            .ok_or_else(|| internal("invariant key is not declared"))?;
        let (_, actual) = plan
            .schema()
            .qualified_field_with_unqualified_name(name)
            .map_err(engine)?;
        let expected = pse_schema::arrow::field_for(registry, expected)
            .map_err(pse_relations::RelationError::from)?;
        if actual.data_type() != expected.data_type() {
            return Err(internal(format!(
                "invariant {} has an incompatible key field {name}",
                invariant.qualified_name()
            )));
        }
        projection.push(col(*name));
    }
    Ok(LogicalPlan::Projection(
        Projection::try_new(projection, Arc::new(plan)).map_err(engine)?,
    ))
}

fn combine_findings(
    mut branches: Vec<LogicalPlan>,
    registry: &Registry,
) -> Result<Option<LogicalPlan>, RuleError> {
    if branches.is_empty() {
        return Ok(None);
    }
    if branches.len() == 1 {
        return Ok(branches.pop());
    }
    let plan = LogicalPlan::Union(
        datafusion::logical_expr::Union::try_new_with_loose_types(
            branches.into_iter().map(Arc::new).collect(),
        )
        .map_err(engine)?,
    );
    let spec = registry
        .relation("runtime.diagnostics_findings")
        .ok_or_else(|| internal("diagnostic relation absent"))?;
    Ok(Some(
        declare_relation_output(plan, registry, spec).map_err(engine)?,
    ))
}

/// Full reports alone promise unique findings in identity order.
pub(super) fn full_report(plan: LogicalPlan) -> Result<LogicalPlan, RuleError> {
    let plan = LogicalPlanBuilder::from(plan)
        .distinct()
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let finding_id = plan
        .schema()
        .qualified_field_with_unqualified_name("finding_id")
        .map_err(engine)?;
    let ordering = Expr::Column(datafusion::common::Column::new(
        finding_id.0.cloned(),
        finding_id.1.name(),
    ));
    Ok(LogicalPlan::Sort(Sort {
        expr: vec![ordering.sort(true, false)],
        input: Arc::new(plan),
        fetch: None,
    }))
}

fn applies(scope: InvariantScope<'_>, target: &RelationSpec) -> bool {
    match scope {
        InvariantScope::Candidate | InvariantScope::Affected(_) | InvariantScope::Required(_) => {
            true
        }
        InvariantScope::SidecarRelation => target.snapshot_class == SnapshotClass::Sidecar,
        InvariantScope::Model => target.snapshot_class == SnapshotClass::Model,
        InvariantScope::Case => target.snapshot_class == SnapshotClass::Case,
        InvariantScope::Registry => target.key.namespace == pse_schema::model::Namespace::Reference,
    }
}

#[derive(Clone, Copy)]
struct Check<'a> {
    id: pse_ids::SemanticId,
    relation: &'a str,
    severity: pse_schema::model::Severity,
}
impl<'a> From<&'a InvariantSpec> for Check<'a> {
    fn from(value: &'a InvariantSpec) -> Self {
        Self {
            id: value.id,
            relation: &value.relation,
            severity: value.severity,
        }
    }
}

fn finding(
    input: LogicalPlan,
    check: Check<'_>,
    status: &str,
    message: &str,
    registry: &Registry,
) -> Result<LogicalPlan, RuleError> {
    let target = registry
        .relation("runtime.diagnostics_findings")
        .ok_or_else(|| internal("diagnostic output is undeclared"))?;
    let relation = registry
        .relation(check.relation)
        .ok_or_else(|| internal("finding relation absent"))?
        .id;
    let (key, subjects) = finding_values(&input, registry, target, relation)?;
    let check_id = constant(
        registry,
        target,
        "check_id",
        ScalarValue::FixedSizeBinary(16, Some(check.id.as_bytes().to_vec())),
    )?;
    let finding_id = scalar::named_id(scalar::named_id(check_id.clone(), lit(status)), key.clone());
    let output = vec![
        finding_id.alias("finding_id"),
        constant(
            registry,
            target,
            "run_id",
            ScalarValue::FixedSizeBinary(16, None),
        )?
        .alias("run_id"),
        check_id.alias("check_id"),
        constant(
            registry,
            target,
            "severity",
            ScalarValue::Utf8(Some(check.severity.as_str().to_owned())),
        )?
        .alias("severity"),
        subjects.alias("subjects"),
        row_evidence(registry, target, relation, key)?.alias("evidence"),
        constant(
            registry,
            target,
            "message",
            ScalarValue::Utf8(Some(message.to_owned())),
        )?
        .alias("message"),
        empty_list(registry, target, "next_steps")?.alias("next_steps"),
    ];
    Ok(LogicalPlan::Projection(
        Projection::try_new(output, Arc::new(input)).map_err(engine)?,
    ))
}

fn row_evidence(
    registry: &Registry,
    target: &RelationSpec,
    relation: pse_ids::SemanticId,
    key: Expr,
) -> Result<Expr, RuleError> {
    let evidence = target
        .column("evidence")
        .ok_or_else(|| internal("diagnostic evidence absent"))?;
    let execution = evidence
        .children()
        .into_iter()
        .find(|field| field.name() == "execution")
        .ok_or_else(|| internal("execution evidence arm absent"))?;
    let field = pse_schema::arrow::field_for(registry, &execution)
        .map_err(pse_relations::RelationError::from)?;
    let absent = checked_literal(
        registry,
        &execution,
        ScalarValue::try_from(field.data_type()).map_err(engine)?,
    )
    .map_err(engine)?;
    let identity = checked_literal(
        registry,
        &pse_schema::model::FieldContract::id(),
        ScalarValue::FixedSizeBinary(16, Some(relation.as_bytes().to_vec())),
    )
    .map_err(engine)?;
    Ok(datafusion::functions::core::expr_fn::named_struct(vec![
        lit("kind"),
        lit("row"),
        lit("row"),
        datafusion::functions::core::expr_fn::named_struct(vec![
            lit("relation_id"),
            identity,
            lit("row_key"),
            key,
        ]),
        lit("execution"),
        absent,
    ]))
}

fn finding_values(
    input: &LogicalPlan,
    registry: &Registry,
    target: &RelationSpec,
    relation: pse_ids::SemanticId,
) -> Result<(Expr, Expr), RuleError> {
    let columns = input.schema().columns();
    let names = columns
        .iter()
        .map(|column| column.name().to_owned())
        .collect::<Vec<_>>();
    let expressions = input
        .schema()
        .columns()
        .into_iter()
        .map(Expr::Column)
        .collect::<Vec<_>>();
    let key = pse_relations::identity::key(
        relation,
        names
            .iter()
            .zip(&expressions)
            .map(|(name, expression)| (name.as_str(), expression.clone()))
            .collect(),
    );
    let ids = input
        .schema()
        .fields()
        .iter()
        .zip(&expressions)
        .filter(|(field, _)| {
            field
                .metadata()
                .get(pse_schema::arrow::KEY_EXTENSION_NAME)
                .is_some_and(|name| name == "pse.semantic_id")
        })
        .map(|(_, expression)| expression.clone())
        .collect::<Vec<_>>();
    let subjects = if ids.is_empty() {
        empty_list(registry, target, "subjects")?
    } else {
        scalar::id_list(ids)
    };
    Ok((key, subjects))
}

fn constant(
    registry: &Registry,
    spec: &RelationSpec,
    name: &str,
    value: ScalarValue,
) -> Result<Expr, RuleError> {
    let column = spec
        .column(name)
        .ok_or_else(|| internal("diagnostic field absent"))?;
    checked_literal(registry, column, value).map_err(engine)
}

fn empty_list(registry: &Registry, spec: &RelationSpec, name: &str) -> Result<Expr, RuleError> {
    let column = spec
        .column(name)
        .ok_or_else(|| internal("diagnostic list field absent"))?;
    let field = pse_schema::arrow::field_for(registry, column)
        .map_err(pse_relations::RelationError::from)?;
    let DataType::List(child) = field.data_type() else {
        return Err(internal("diagnostic collection field is not List"));
    };
    let mut list =
        ListBuilder::new(make_builder(child.data_type(), 0)).with_field(Arc::clone(child));
    list.append(true);
    checked_literal(registry, column, ScalarValue::List(Arc::new(list.finish()))).map_err(engine)
}
