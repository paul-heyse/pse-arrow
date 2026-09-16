// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry declarations become one native union of exact diagnostic projections.

use crate::{
    RuleError,
    errmap::{engine, internal},
    plan::{PortBinding, compile as compile_rule},
};
use datafusion::arrow::array::builder::{ListBuilder, make_builder};
use datafusion::arrow::datatypes::DataType;
use datafusion::common::ScalarValue;
use datafusion::logical_expr::{
    Expr, LogicalPlan, LogicalPlanBuilder, Operator, col, expr::BinaryExpr, lit,
};
use pse_catalog::session::{
    SnapshotSession,
    output::{checked_literal, declare_relation_output},
    scalar,
};
use pse_ids::{CancellationToken, SnapshotId};
use pse_schema::{
    Registry,
    model::{InvariantSpec, RelationKey, RelationSpec, SnapshotClass},
};
use std::{collections::BTreeSet, sync::Arc};

use super::InvariantScope;

pub(super) fn compile(
    candidates: &BTreeSet<RelationKey>,
    session: &SnapshotSession,
    registry: &Registry,
    scope: InvariantScope<'_>,
    subject: Option<SnapshotId>,
    cancel: &CancellationToken,
) -> Result<(Option<LogicalPlan>, usize), RuleError> {
    let mut branches = Vec::new();
    let mut count = 0;
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
        let rule = registry
            .rule(&invariant.rule)
            .ok_or_else(|| internal("invariant rule absent"))?;
        let dependencies = rule.plan.dependencies();
        if let InvariantScope::Affected(changed) = scope
            && !dependencies.iter().any(|(name, _, _)| {
                registry
                    .relation(name)
                    .is_some_and(|spec| changed.contains(&spec.key))
            })
        {
            continue;
        }
        if scope == InvariantScope::SidecarRelation
            && dependencies
                .iter()
                .any(|(name, _, _)| *name != target.key.qualified_name())
        {
            continue;
        }
        let mut binding = PortBinding::default();
        for (relation, port, _) in dependencies {
            let spec = registry
                .relation(relation)
                .ok_or_else(|| internal("invariant dependency undeclared"))?;
            if !candidates.contains(&spec.key) {
                return Err(internal(format!(
                    "invariant {} lacks explicit dependency {}",
                    invariant.qualified_name(),
                    spec.key
                )));
            }
            if binding
                .ports
                .insert(port.to_owned(), spec.key)
                .is_some_and(|prior| prior != spec.key)
            {
                return Err(internal(
                    "one invariant port is bound to different relations",
                ));
            }
        }
        let compiled = compile_rule(rule, &binding, session, registry)?;
        branches.push(finding(
            strip_order(compiled.plan),
            invariant,
            "violation",
            invariant.doc,
            subject,
            registry,
        )?);
        if let Some(plan) = compiled.undecided {
            branches.push(finding(
                strip_order(plan),
                invariant,
                "unknown",
                &format!("{}: invariant predicate is unknown", invariant.doc),
                subject,
                registry,
            )?);
        }
        for (plan, reason) in compiled.checks {
            branches.push(finding(
                plan,
                invariant,
                "precondition",
                &reason,
                subject,
                registry,
            )?);
        }
        count += 1;
    }
    if let InvariantScope::Required(selected) = scope
        && count != selected.len()
    {
        return Err(internal(
            "required invariant declaration or target binding is absent",
        ));
    }
    Ok((combine_findings(branches, registry)?, count))
}

fn combine_findings(
    branches: Vec<LogicalPlan>,
    registry: &Registry,
) -> Result<Option<LogicalPlan>, RuleError> {
    let mut plans = branches.into_iter();
    let Some(first) = plans.next() else {
        return Ok(None);
    };
    let mut plan = first;
    for next in plans {
        plan = LogicalPlanBuilder::from(plan)
            .union(next)
            .map_err(engine)?
            .build()
            .map_err(engine)?;
    }
    let plan = LogicalPlanBuilder::from(plan)
        .distinct()
        .map_err(engine)?
        .sort([col("finding_id").sort(true, false)])
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let spec = registry
        .relation("runtime.diagnostics_findings")
        .ok_or_else(|| internal("diagnostic relation absent"))?;
    Ok(Some(
        declare_relation_output(plan, registry, spec).map_err(engine)?,
    ))
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

fn strip_order(plan: LogicalPlan) -> LogicalPlan {
    if let LogicalPlan::Sort(sort) = plan {
        sort.input.as_ref().clone()
    } else {
        plan
    }
}

fn finding(
    input: LogicalPlan,
    invariant: &InvariantSpec,
    status: &str,
    message: &str,
    subject: Option<SnapshotId>,
    registry: &Registry,
) -> Result<LogicalPlan, RuleError> {
    let target = registry
        .relation("runtime.diagnostics_findings")
        .ok_or_else(|| internal("diagnostic output is undeclared"))?;
    let (key, subjects) = finding_values(&input, registry, target)?;
    let check_id = constant(
        registry,
        target,
        "check_id",
        ScalarValue::FixedSizeBinary(16, Some(invariant.id.as_bytes().to_vec())),
    )?;
    let finding_id = scalar::named_id(
        check_id.clone(),
        // Native concatenation is total for these two non-null operands. The
        // general CONCAT UDF conservatively declares nullable output at this pin.
        Expr::BinaryExpr(BinaryExpr::new(
            Box::new(lit(format!("{status}:"))),
            Operator::StringConcat,
            Box::new(key.clone()),
        )),
    );
    let output = vec![
        finding_id.alias("finding_id"),
        constant(
            registry,
            target,
            "subject_snapshot",
            ScalarValue::FixedSizeBinary(
                32,
                subject.map(|id| id.content_hash().as_bytes().to_vec()),
            ),
        )?
        .alias("subject_snapshot"),
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
            ScalarValue::Utf8(Some(invariant.severity.as_str().to_owned())),
        )?
        .alias("severity"),
        subjects.alias("subjects"),
        key.alias("values"),
        constant(
            registry,
            target,
            "message",
            ScalarValue::Utf8(Some(message.to_owned())),
        )?
        .alias("message"),
        empty_list(registry, target, "next_steps")?.alias("next_steps"),
    ];
    let plan = LogicalPlanBuilder::from(input)
        .project(output)
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    declare_relation_output(plan, registry, target).map_err(engine)
}

fn finding_values(
    input: &LogicalPlan,
    registry: &Registry,
    target: &RelationSpec,
) -> Result<(Expr, Expr), RuleError> {
    let columns = input.schema().columns();
    let names = columns
        .iter()
        .map(datafusion::common::Column::flat_name)
        .collect::<Vec<_>>();
    let expressions = input
        .schema()
        .columns()
        .into_iter()
        .map(Expr::Column)
        .collect::<Vec<_>>();
    let key = scalar::key(
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
