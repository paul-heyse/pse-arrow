// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared native SQL predicates become typed diagnostics without a rule AST.

use super::{Check, InvariantScope, applies, engine, finding, internal};
use crate::RuleError;
use datafusion::logical_expr::{LogicalPlan, LogicalPlanBuilder, Projection, col};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_schema::{
    Registry,
    model::{RelationKey, Severity},
};
use std::{collections::BTreeSet, sync::Arc};

pub(super) fn compile(
    candidates: &BTreeSet<RelationKey>,
    session: &SnapshotSession,
    registry: &Registry,
    scope: InvariantScope<'_>,
    cancel: &CancellationToken,
) -> Result<(Vec<LogicalPlan>, usize), RuleError> {
    let mut branches = Vec::new();
    let mut count = 0;
    for key in candidates {
        cancel
            .checkpoint()
            .map_err(pse_catalog::CatalogError::from)?;
        let spec = registry
            .relation_by_key(*key)
            .ok_or_else(|| internal("native check relation absent"))?;
        if spec.checks.is_empty()
            || !applies(scope, spec)
            || matches!(scope, InvariantScope::Affected(changed) if !changed.contains(key))
        {
            continue;
        }
        let selected: BTreeSet<_> = spec.checks.keys().filter(|name| {
            !matches!(scope, InvariantScope::Required(ids) if !spec.row_check_id(name).is_some_and(|id| ids.contains(&id)))
        }).cloned().collect();
        if selected.is_empty() {
            continue;
        }
        let input = LogicalPlanBuilder::scan(
            session.table_reference(key)?,
            session.table_source(key)?,
            None,
        )
        .map_err(engine)?
        .build()
        .map_err(engine)?;
        for (name, predicate) in session.row_check_expressions(*key)? {
            if !selected.contains(&name) {
                continue;
            }
            let id = spec
                .row_check_id(&name)
                .ok_or_else(|| internal("native check is undeclared"))?;
            let violations = LogicalPlan::Filter(
                datafusion::logical_expr::Filter::try_new(
                    predicate.is_not_true(),
                    Arc::new(input.clone()),
                )
                .map_err(engine)?,
            );
            let violations = LogicalPlan::Projection(
                Projection::try_new(
                    spec.primary_key.iter().map(|name| col(*name)).collect(),
                    Arc::new(violations),
                )
                .map_err(engine)?,
            );
            branches.push(finding(
                violations,
                Check {
                    id,
                    relation: &spec.qualified_name(),
                    severity: Severity::Error,
                },
                "violation",
                &format!(
                    "{}: native check {name} requires SQL true",
                    spec.qualified_name()
                ),
                registry,
            )?);
            count += 1;
        }
    }
    Ok((branches, count))
}
