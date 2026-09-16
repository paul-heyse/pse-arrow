// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Target declaration and finite member products execute as native logical plans.
use super::{IndexSelector, TargetPath, TargetRow};
use crate::{AuthoringError, document::Batches};
use datafusion::{
    arrow::{
        array::{FixedSizeBinaryArray, RecordBatch},
        compute::concat_batches,
    },
    common::{Column, ScalarValue},
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::{
    SnapshotSession,
    output::{checked_literal, declare_relation_output},
};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{authored, enums::TargetKind},
};
use pse_schema::model::FieldContract;
use std::collections::BTreeSet;

fn contract(at: Option<crate::SourceSpan>, reason: &str) -> AuthoringError {
    AuthoringError::Contract {
        at,
        reason: reason.to_owned(),
    }
}
fn engine(error: datafusion::common::DataFusionError) -> AuthoringError {
    crate::change_set::plans::engine(error)
}
fn field(alias: &str, name: &str) -> Expr {
    Expr::Column(Column::new(Some(alias), name))
}
fn identity(session: &SnapshotSession, id: SemanticId) -> Result<Expr, AuthoringError> {
    checked_literal(
        session.registry(),
        &FieldContract::payload("identity", FieldContract::id(), "Actual identity."),
        ScalarValue::FixedSizeBinary(16, Some(id.as_bytes().to_vec())),
    )
    .map_err(engine)
}
fn scan(session: &SnapshotSession, id: SemanticId) -> Result<LogicalPlan, AuthoringError> {
    Ok(session.scan_role(&format!("target_{id}"))?)
}
async fn selected(
    session: &SnapshotSession,
    relation: SemanticId,
    condition: Expr,
    cancel: &CancellationToken,
    completed: &mut crate::change_set::plans::Completions,
) -> Result<FieldCheckedBatch, AuthoringError> {
    let plan = LogicalPlanBuilder::from(scan(session, relation)?)
        .filter(condition)
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    materialize(session, relation, plan, cancel, completed).await
}
async fn materialize(
    session: &SnapshotSession,
    relation: SemanticId,
    plan: LogicalPlan,
    cancel: &CancellationToken,
    completed: &mut crate::change_set::plans::Completions,
) -> Result<FieldCheckedBatch, AuthoringError> {
    let spec = session
        .registry()
        .relation_by_id(relation)
        .ok_or_else(|| contract(None, "target relation absent"))?;
    let plan = declare_relation_output(plan, session.registry(), spec).map_err(engine)?;
    let batches =
        crate::change_set::plans::execute_recorded(session, plan, cancel, completed).await?;
    let mut work = session.reserver().open("authoring:target-native-output");
    work.try_grow(batches.iter().try_fold(0_usize, |sum, batch| {
        crate::work::add(sum, pse_ids::validation_extent(batch)?)
    })?)?;
    let schema = std::sync::Arc::new(
        pse_schema::arrow::relation_schema(session.registry(), spec)
            .map_err(pse_relations::RelationError::from)?,
    );
    Ok(FieldCheckedBatch::admit(
        session.registry(),
        spec,
        concat_batches(&schema, &batches).map_err(|error| engine(error.into()))?,
    )?
    .retained(session.reserver(), cancel)?)
}

/// Resolve a parsed path with native name/ownership/domain joins and cross products.
/// # Errors
/// Missing or ambiguous declarations, invalid selectors, cancellation or resource refusal.
pub async fn resolve_native(
    path: &TargetPath,
    batches: &Batches,
    session: &SnapshotSession,
    source_id: SemanticId,
    work: &mut dyn pse_ids::Reservation,
    completed: &mut Vec<std::sync::Arc<pse_catalog::session::CompletedComputation>>,
    cancel: &CancellationToken,
) -> Result<Vec<TargetRow>, AuthoringError> {
    if path.names.is_empty() || (path.instance_wildcard && !path.indices.is_empty()) {
        return Err(contract(
            Some(path.at),
            "target path has no names or an indexed instance wildcard",
        ));
    }
    let roles = batches
        .iter()
        .map(|(id, batch)| (format!("target_{id}"), batch.clone()))
        .collect();
    let session = session.with_checked_role_inputs(roles, cancel)?;
    let session = crate::change_set::plans::session(&session)?;
    let instance = instance(path, &session, cancel, completed).await?;
    let mut target = TargetRow {
        spec_id: source_id,
        ordinal: 0,
        instance_id: instance.instance_id,
        member_kind: TargetKind::InstanceWildcard,
        symbol_decl_id: None,
        equation_decl_id: None,
        port_template_id: None,
        port_name: None,
        index: None,
        wildcard: path.instance_wildcard,
    };
    if path.instance_wildcard {
        return Ok(vec![target]);
    }
    let axes = select_member(path, &session, &instance, &mut target, cancel, completed).await?;
    if axes.is_empty() {
        if !path.indices.is_empty() {
            return Err(contract(
                Some(path.at),
                "scalar or port target cannot carry indices",
            ));
        }
        return Ok(vec![target]);
    }
    if axes.iter().collect::<BTreeSet<_>>().len() != axes.len()
        || (!path.indices.is_empty() && axes.len() != path.indices.len())
    {
        return Err(contract(
            Some(path.at),
            "duplicate axes or mismatched target index arity",
        ));
    }
    let plan = target_product(path, &session, &instance, &axes, cancel, completed).await?;
    let tuples =
        crate::change_set::plans::execute_recorded(&session, plan, cancel, completed).await?;
    let count = tuples.iter().map(RecordBatch::num_rows).sum::<usize>();
    if count > usize::from(u16::MAX) + 1 {
        return Err(contract(
            Some(path.at),
            "target ordinal exceeds declared UInt16",
        ));
    }
    work.try_grow(crate::work::mul(
        count,
        crate::work::add(
            size_of::<TargetRow>(),
            crate::work::mul(axes.len(), size_of::<SemanticId>())?,
        )?,
    )?)?;
    let mut output = Vec::with_capacity(count);
    for batch in tuples {
        for row in 0..batch.num_rows() {
            let mut result = target.clone();
            result.ordinal = u16::try_from(output.len())
                .map_err(|_| contract(Some(path.at), "target ordinal overflow"))?;
            result.index = Some(
                batch
                    .columns()
                    .iter()
                    .map(|array| {
                        let array = array
                            .as_any()
                            .downcast_ref::<FixedSizeBinaryArray>()
                            .ok_or_else(|| contract(None, "target member identity storage"))?;
                        Ok(SemanticId::from_bytes(
                            array
                                .value(row)
                                .try_into()
                                .map_err(|_| contract(None, "target member identity width"))?,
                        ))
                    })
                    .collect::<Result<Vec<_>, AuthoringError>>()?,
            );
            output.push(result);
        }
    }
    Ok(output)
}
async fn instance(
    path: &TargetPath,
    session: &SnapshotSession,
    cancel: &CancellationToken,
    completed: &mut crate::change_set::plans::Completions,
) -> Result<authored::instances::Row, AuthoringError> {
    let count = path.names.len() - usize::from(!path.instance_wildcard);
    let name = path.names[..count].join(".");
    if name.is_empty() {
        return Err(contract(Some(path.at), "target must name its instance"));
    }
    let instances = LogicalPlanBuilder::from(scan(session, authored::instances::RELATION_ID)?)
        .alias("instance")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let entities = LogicalPlanBuilder::from(scan(session, authored::entities::RELATION_ID)?)
        .alias("entity")
        .map_err(engine)?;
    let condition = field("entity", "qualified_name").eq(lit(name.clone())).or(
        datafusion::functions::string::expr_fn::ends_with(
            field("entity", "qualified_name"),
            lit(format!(".{name}")),
        ),
    );
    let spec = session
        .registry()
        .relation_by_id(authored::instances::RELATION_ID)
        .ok_or_else(|| contract(None, "instances declaration absent"))?;
    let plan = entities
        .filter(condition)
        .and_then(|plan| {
            plan.join_on(
                instances,
                JoinType::Inner,
                [field("entity", "entity_id").eq(field("instance", "instance_id"))],
            )
        })
        .and_then(|plan| {
            plan.project(
                spec.columns
                    .iter()
                    .map(|column| field("instance", column.name()))
                    .collect::<Vec<_>>(),
            )
        })
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let rows = materialize(session, spec.id, plan, cancel, completed).await?;
    let mut rows = authored::instances::View::from_checked(&rows)?.rows()?;
    if rows.len() != 1 {
        return Err(contract(
            Some(path.at),
            "target instance missing or ambiguous",
        ));
    }
    rows.pop()
        .ok_or_else(|| contract(Some(path.at), "target instance absent"))
}
async fn domain(
    path: &TargetPath,
    session: &SnapshotSession,
    instance: &authored::instances::Row,
    name: &str,
    cancel: &CancellationToken,
    completed: &mut crate::change_set::plans::Completions,
) -> Result<authored::domains::Row, AuthoringError> {
    let declaration = selected(
        session,
        authored::template_domains::RELATION_ID,
        col("template_id")
            .eq(identity(session, instance.template_id)?)
            .and(col("name").eq(lit(name))),
        cancel,
        completed,
    )
    .await?;
    let declaration = authored::template_domains::View::from_checked(&declaration)?.rows()?;
    let binding = selected(
        session,
        authored::instance_domain_bindings::RELATION_ID,
        col("instance_id")
            .eq(identity(session, instance.instance_id)?)
            .and(col("domain_name").eq(lit(name))),
        cancel,
        completed,
    )
    .await?;
    let binding = authored::instance_domain_bindings::View::from_checked(&binding)?.rows()?;
    let ([declaration], [binding]) = (declaration.as_slice(), binding.as_slice()) else {
        return Err(contract(
            Some(path.at),
            "domain declaration or binding missing/ambiguous",
        ));
    };
    let actual = selected(
        session,
        authored::domains::RELATION_ID,
        col("domain_id").eq(identity(session, binding.domain_id)?),
        cancel,
        completed,
    )
    .await?;
    let mut actual = authored::domains::View::from_checked(&actual)?.rows()?;
    if actual.len() != 1 {
        return Err(contract(Some(path.at), "bound domain absent or ambiguous"));
    }
    let actual = actual
        .pop()
        .ok_or_else(|| contract(Some(path.at), "bound domain absent"))?;
    if declaration.kind != actual.kind || declaration.continuous != actual.continuous {
        return Err(contract(
            Some(path.at),
            "bound domain kind/continuity mismatch",
        ));
    }
    let mut current = Some(instance.instance_id);
    let mut seen = BTreeSet::new();
    while let Some(owner) = current {
        if !seen.insert(owner) {
            return Err(contract(Some(path.at), "instance containment cycle"));
        }
        if owner == actual.owner_entity_id {
            return Ok(actual);
        }
        let parent = selected(
            session,
            authored::instances::RELATION_ID,
            col("instance_id").eq(identity(session, owner)?),
            cancel,
            completed,
        )
        .await?;
        let parents = authored::instances::View::from_checked(&parent)?.rows()?;
        let [parent] = parents.as_slice() else {
            return Err(contract(
                Some(path.at),
                "domain owner ancestor missing/ambiguous",
            ));
        };
        current = parent.parent_instance_id;
    }
    Err(contract(
        Some(path.at),
        "domain owner is outside target instance ancestry",
    ))
}
fn selector_condition(
    session: &SnapshotSession,
    selector: &IndexSelector,
) -> Result<Expr, AuthoringError> {
    let (text, quoted) = match selector {
        IndexSelector::Label(text) => (text, true),
        IndexSelector::Value(text) => (text, false),
        IndexSelector::Wildcard => return Ok(lit(true)),
    };
    let mut condition = col("label").eq(lit(text.clone()));
    if !quoted {
        if let Ok(id) = SemanticId::parse_hex(text) {
            condition = condition.or(col("member_id").eq(identity(session, id)?));
        }
        if let Some(number) = text.parse::<f64>().ok().filter(|number| number.is_finite()) {
            let spec = session
                .registry()
                .relation_by_id(authored::domain_members::RELATION_ID)
                .ok_or_else(|| contract(None, "member declaration absent"))?;
            let coordinate = spec
                .columns
                .iter()
                .find(|column| column.name() == "coordinate")
                .ok_or_else(|| contract(None, "coordinate declaration absent"))?;
            let value = checked_literal(
                session.registry(),
                coordinate,
                ScalarValue::Float64(Some(number)),
            )
            .map_err(engine)?;
            condition = condition.or(crate::change_set::exact::equal(col("coordinate"), value));
        }
    }
    Ok(condition)
}

async fn select_member(
    path: &TargetPath,
    session: &SnapshotSession,
    instance: &authored::instances::Row,
    target: &mut TargetRow,
    cancel: &CancellationToken,
    completed: &mut crate::change_set::plans::Completions,
) -> Result<Vec<String>, AuthoringError> {
    let name = path
        .names
        .last()
        .ok_or_else(|| contract(Some(path.at), "target member absent"))?;
    let condition = || {
        Ok::<_, AuthoringError>(
            col("template_id")
                .eq(identity(session, instance.template_id)?)
                .and(col("name").eq(lit(name.clone()))),
        )
    };
    macro_rules! members {
        ($relation:ident) => {{
            let values = selected(
                session,
                authored::$relation::RELATION_ID,
                condition()?,
                cancel,
                completed,
            )
            .await?;
            authored::$relation::View::from_checked(&values)?.rows()?
        }};
    }
    let symbols = members!(template_symbols);
    let equations = members!(template_equations);
    let ports = members!(template_ports);
    if symbols.len() + equations.len() + ports.len() != 1 {
        return Err(contract(
            Some(path.at),
            "target member missing or ambiguous",
        ));
    }
    let axes = if let Some(symbol) = symbols.into_iter().next() {
        target.member_kind = TargetKind::Symbol;
        target.symbol_decl_id = Some(symbol.symbol_decl_id);
        symbol.indexed_by
    } else if let Some(equation) = equations.into_iter().next() {
        target.member_kind = TargetKind::Equation;
        target.equation_decl_id = Some(equation.equation_decl_id);
        equation.indexed_by
    } else {
        let port = ports
            .into_iter()
            .next()
            .ok_or_else(|| contract(Some(path.at), "target port absent"))?;
        target.member_kind = TargetKind::Port;
        target.port_template_id = Some(port.template_id);
        target.port_name = Some(port.name);
        Vec::new()
    };
    Ok(axes)
}

async fn target_product(
    path: &TargetPath,
    session: &SnapshotSession,
    instance: &authored::instances::Row,
    axes: &[String],
    cancel: &CancellationToken,
    completed: &mut crate::change_set::plans::Completions,
) -> Result<LogicalPlan, AuthoringError> {
    let mut product = None;
    for (position, name) in axes.iter().enumerate() {
        let plan = axis_members(path, session, instance, name, position, cancel, completed).await?;
        product = Some(match product {
            None => plan,
            Some(prior) => LogicalPlanBuilder::from(prior)
                .cross_join(plan)
                .and_then(LogicalPlanBuilder::build)
                .map_err(engine)?,
        });
    }
    let plan = product.ok_or_else(|| contract(Some(path.at), "target product absent"))?;
    let order = (0..axes.len())
        .flat_map(|position| {
            [
                col(format!("order_{position}")).sort(true, false),
                col(format!("member_{position}")).sort(true, false),
            ]
        })
        .collect::<Vec<_>>();
    let plan = LogicalPlanBuilder::from(plan)
        .sort(order)
        .and_then(|plan| {
            plan.project(
                (0..axes.len())
                    .map(|position| col(format!("member_{position}")))
                    .collect::<Vec<_>>(),
            )
        })
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    Ok(plan)
}

async fn axis_members(
    path: &TargetPath,
    session: &SnapshotSession,
    instance: &authored::instances::Row,
    name: &str,
    position: usize,
    cancel: &CancellationToken,
    completed: &mut crate::change_set::plans::Completions,
) -> Result<LogicalPlan, AuthoringError> {
    let domain = domain(path, session, instance, name, cancel, completed).await?;
    let mut plan = LogicalPlanBuilder::from(scan(session, authored::domain_members::RELATION_ID)?)
        .filter(col("domain_id").eq(identity(session, domain.domain_id)?))
        .map_err(engine)?;
    if let Some(selector) = path
        .indices
        .get(position)
        .filter(|value| **value != IndexSelector::Wildcard)
    {
        plan = plan
            .filter(selector_condition(session, selector)?)
            .map_err(engine)?;
    }
    let plan = plan
        .project(vec![
            col("member_id").alias(format!("member_{position}")),
            col("ordinal").alias(format!("order_{position}")),
        ])
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let duplicate = LogicalPlanBuilder::from(plan.clone())
        .aggregate(
            vec![col(format!("member_{position}"))],
            vec![datafusion::functions_aggregate::expr_fn::count(lit(1_i64)).alias("multiplicity")],
        )
        .and_then(|plan| plan.filter(col("multiplicity").gt(lit(1_i64))))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    if crate::change_set::plans::execute_recorded(session, duplicate, cancel, completed)
        .await?
        .iter()
        .any(|batch| batch.num_rows() > 0)
    {
        return Err(contract(
            Some(path.at),
            "bound domain repeats a member identity",
        ));
    }
    let selected =
        crate::change_set::plans::execute_recorded(session, plan.clone(), cancel, completed)
            .await?;
    let count = selected.iter().map(RecordBatch::num_rows).sum::<usize>();
    if count == 0
        || (path
            .indices
            .get(position)
            .is_some_and(|selector| *selector != IndexSelector::Wildcard)
            && count != 1)
    {
        return Err(contract(
            Some(path.at),
            "selector must match its actual finite domain without ambiguity",
        ));
    }
    Ok(plan)
}
