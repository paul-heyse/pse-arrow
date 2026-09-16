// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native comparisons at external reader admission and rename-preservation boundaries.
use crate::{
    AuthoringError,
    document::{Batches, load::contract},
};
use datafusion::{
    common::Column,
    logical_expr::{Expr, JoinType, LogicalPlanBuilder, lit},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::generated::authored;
use std::ops::Not;

pub(super) async fn correspondence(
    expected: &Batches,
    actual: &Batches,
    session: &SnapshotSession,
    cancel: &CancellationToken,
    completed: &mut crate::change_set::plans::Completions,
) -> Result<(), AuthoringError> {
    let execution = crate::change_set::plans::session(session)?;
    for (id, expected) in expected {
        let Some(actual) = actual.get(id) else {
            // AuthoredReader explicitly represents an absent relation as empty.
            // A projected nonempty source relation still requires actual rows.
            if expected.batch().num_rows() == 0 {
                continue;
            }
            return Err(contract(
                None,
                &format!("reader omits nonempty source relation {id}"),
            ));
        };
        if expected.batch().num_rows() == 0 && actual.batch().num_rows() == 0 {
            continue;
        }
        let spec = session
            .registry()
            .relation_by_id(*id)
            .ok_or_else(|| contract(None, "source relation absent"))?;
        let bound =
            crate::change_set::plans::roles(&execution, actual.clone(), expected.clone(), cancel)?;
        for role in ["change_before", "change_after"] {
            completed.push(crate::change_set::stage::unique(&bound, role, spec, cancel).await?);
        }
        let plan = crate::change_set::plans::difference(&bound, spec)?;
        if crate::change_set::plans::execute_recorded(&bound, plan, cancel, completed)
            .await?
            .iter()
            .any(|batch| batch.num_rows() != 0)
        {
            return Err(contract(
                None,
                "reader rows differ from their exact original sources",
            ));
        }
    }
    Ok(())
}
pub(super) async fn preserved(
    before: &Batches,
    after: &Batches,
    names: bool,
    session: &SnapshotSession,
    cancel: &CancellationToken,
    completed: &mut crate::change_set::plans::Completions,
) -> Result<(), AuthoringError> {
    let targets = [
        authored::case_spec_targets::RELATION_ID,
        authored::case_activation_targets::RELATION_ID,
        authored::observation_targets::RELATION_ID,
    ];
    let expected = before
        .iter()
        .filter(|(id, _)| targets.contains(id))
        .map(|(id, batch)| (*id, batch.clone()))
        .collect();
    correspondence(&expected, after, session, cancel, completed).await?;
    if !names {
        return Ok(());
    }
    let id = authored::entities::RELATION_ID;
    let before = before
        .get(&id)
        .ok_or_else(|| contract(None, "prior entities absent"))?;
    let after = after
        .get(&id)
        .ok_or_else(|| contract(None, "updated entities absent"))?;
    let execution = crate::change_set::plans::session(session)?;
    let bound = crate::change_set::plans::roles(&execution, before.clone(), after.clone(), cancel)?;
    let spec = session
        .registry()
        .relation_by_id(id)
        .ok_or_else(|| contract(None, "entity declaration absent"))?;
    let columns = spec
        .columns
        .iter()
        .filter(|column| column.name() != "source_span")
        .map(pse_schema::model::FieldContract::name)
        .collect::<Vec<_>>();
    let side = |role: &str, alias: &str| -> Result<_, AuthoringError> {
        LogicalPlanBuilder::from(bound.scan_role(role)?)
            .project(
                columns
                    .iter()
                    .map(|name| datafusion::logical_expr::col(*name))
                    .chain([lit(true).alias("present")])
                    .collect::<Vec<_>>(),
            )
            .and_then(|plan| plan.alias(alias))
            .and_then(LogicalPlanBuilder::build)
            .map_err(crate::change_set::plans::engine)
    };
    let field = |alias, name| Expr::Column(Column::new(Some(alias), name));
    let plan = LogicalPlanBuilder::from(side("change_before", "prior")?)
        .join_on(
            side("change_after", "next")?,
            JoinType::Full,
            [field("prior", "entity_id").eq(field("next", "entity_id"))],
        )
        .and_then(|plan| {
            plan.filter(
                field("prior", "present")
                    .is_null()
                    .or(field("next", "present").is_null())
                    .or(columns
                        .iter()
                        .map(|name| {
                            crate::change_set::exact::equal(
                                field("prior", name),
                                field("next", name),
                            )
                        })
                        .reduce(Expr::and)
                        .unwrap_or_else(|| lit(true))
                        .not()),
            )
        })
        .and_then(LogicalPlanBuilder::build)
        .map_err(crate::change_set::plans::engine)?;
    if crate::change_set::plans::execute_recorded(&bound, plan, cancel, completed)
        .await?
        .iter()
        .any(|batch| batch.num_rows() != 0)
    {
        return Err(contract(
            None,
            "additional source edits change an entity identity or name",
        ));
    }
    Ok(())
}
