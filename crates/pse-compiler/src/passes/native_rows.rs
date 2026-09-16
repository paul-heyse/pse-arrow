// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit native-plan to typed algorithm input boundary for semantic realization.

use std::collections::BTreeMap;

use datafusion::{
    arrow::array::{Array, StringArray},
    common::{Column, DataFusionError, NullEquality},
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, col},
};
use pse_catalog::session::{
    CompletedComputation, SnapshotSession,
    output::{declare_relation_output, declare_relation_projection},
    scalar,
};
use pse_ids::{CancellationToken, MemoryReserver, Reservation};
use pse_relations::columnar::{FieldCheckedBatch, RelationRow};
use pse_schema::{Registry, model::RelationSpec};

use crate::CompilerError;

/// Accounts for decoded algorithm arguments. Queries and their predecessor plans
/// are released when their owned values cross this explicit algorithm boundary.
#[derive(Debug)]
pub(crate) struct AlgorithmInputs {
    work: Box<dyn Reservation>,
}

/// Select the current input inventory, then bind actual checked outputs without
/// copying or re-admitting their field values. Earlier plans retain their providers.
pub(crate) fn workspace(
    session: &SnapshotSession,
    outputs: &BTreeMap<pse_schema::model::RelationKey, FieldCheckedBatch>,
    cancel: &CancellationToken,
) -> Result<SnapshotSession, CompilerError> {
    let selected = session
        .input_keys()
        .filter(|key| !outputs.contains_key(key))
        .collect();
    Ok(session
        .select_inputs(&selected, cancel)?
        .with_checked_workspace(outputs.clone(), cancel)?)
}

impl AlgorithmInputs {
    pub(crate) async fn execute(
        &mut self,
        plan: LogicalPlan,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<CompletedComputation, CompilerError> {
        let completed = session
            .prepare_rule_plan(plan, cancel)?
            .execute(cancel)
            .await?;
        let extent = completed.batches().iter().try_fold(0_usize, |sum, batch| {
            sum.checked_add(pse_ids::validation_extent(batch)?)
                .ok_or_else(|| invalid("algorithm projection extent overflow"))
        })?;
        self.reserve(
            extent
                .checked_mul(2)
                .and_then(|size| size.checked_add(size_of::<CompletedComputation>() * 2))
                .ok_or_else(|| invalid("algorithm projection extent overflow"))?,
        )?;
        Ok(completed)
    }
    pub(crate) fn reserve(&mut self, bytes: usize) -> Result<(), CompilerError> {
        self.work
            .try_grow(bytes)
            .map_err(pse_ids::CanonError::from)?;
        Ok(())
    }
    pub(crate) fn new(reserver: &dyn MemoryReserver, owner: &str) -> Self {
        Self {
            work: reserver.open(owner),
        }
    }

    pub(crate) async fn retain_plan(
        &mut self,
        plan: LogicalPlan,
        session: &SnapshotSession,
        role: &str,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CompilerError> {
        let completed = session
            .prepare_rule_plan(plan, cancel)?
            .execute(cancel)
            .await?;
        self.work
            .try_grow(size_of::<SnapshotSession>() * 2)
            .map_err(pse_ids::CanonError::from)?;
        let retained = session
            .with_computation_roles(BTreeMap::from([(role.to_owned(), completed)]), cancel)?;
        Ok(retained)
    }

    pub(crate) async fn rows<T: RelationRow>(
        &mut self,
        plan: LogicalPlan,
        session: &SnapshotSession,
        registry: &Registry,
        cancel: &CancellationToken,
    ) -> Result<Vec<T>, CompilerError> {
        let spec = T::relation(registry)?;
        let plan = declare_relation_output(plan, registry, spec).map_err(engine)?;
        let completed = session
            .prepare_rule_plan(plan, cancel)?
            .execute(cancel)
            .await?;
        let checked = completed.checked_relation(registry, spec, cancel)?;
        let bytes = pse_ids::validation_extent(checked.batch())?
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(size_of::<CompletedComputation>() * 2))
            .ok_or_else(|| invalid("typed algorithm input extent overflow"))?;
        self.work
            .try_grow(bytes)
            .map_err(pse_ids::CanonError::from)?;
        cancel.checkpoint()?;
        let rows = T::rows(&checked)?;
        Ok(rows)
    }

    pub(crate) async fn strings(
        &mut self,
        plan: LogicalPlan,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<Vec<Option<String>>, CompilerError> {
        let completed = session
            .prepare_rule_plan(plan, cancel)?
            .execute(cancel)
            .await?;
        let extent = completed.batches().iter().try_fold(0_usize, |sum, batch| {
            sum.checked_add(pse_ids::validation_extent(batch)?)
                .ok_or_else(|| invalid("algorithm text projection extent overflow"))
        })?;
        self.work
            .try_grow(
                extent
                    .checked_mul(2)
                    .ok_or_else(|| invalid("algorithm text projection extent overflow"))?,
            )
            .map_err(pse_ids::CanonError::from)?;
        let mut values = Vec::new();
        for batch in completed.batches() {
            cancel.checkpoint()?;
            if batch.num_columns() != 1 {
                return Err(invalid("algorithm text projection must have one column"));
            }
            let column = batch
                .column(0)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| invalid("algorithm text projection is not Utf8"))?;
            values.extend(
                (0..column.len())
                    .map(|index| (!column.is_null(index)).then(|| column.value(index).to_owned())),
            );
        }
        Ok(values)
    }
}

pub(crate) async fn reject(
    violation: LogicalPlan,
    session: &SnapshotSession,
    cancel: &CancellationToken,
    reason: &str,
) -> Result<(), CompilerError> {
    let plan = LogicalPlanBuilder::from(violation)
        .limit(0, Some(1))
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let completed = session
        .prepare_rule_plan(plan, cancel)?
        .execute(cancel)
        .await?;
    if completed
        .batches()
        .iter()
        .any(|batch| batch.num_rows() != 0)
    {
        return Err(invalid(reason));
    }
    Ok(())
}

pub(crate) fn scan(
    session: &SnapshotSession,
    spec: &RelationSpec,
    alias: &str,
) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::scan(
        session.table_reference(&spec.key)?,
        session.table_source(&spec.key)?,
        None,
    )
    .map_err(engine)?
    .alias(alias)
    .map_err(engine)?
    .build()
    .map_err(engine)
}

pub(crate) fn join(
    left: LogicalPlan,
    right: LogicalPlan,
    kind: JoinType,
    keys: &[(&str, &str)],
) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::from(left)
        .join_detailed(
            right,
            kind,
            (
                keys.iter()
                    .map(|(left, _)| Column::from_qualified_name(*left))
                    .collect::<Vec<_>>(),
                keys.iter()
                    .map(|(_, right)| Column::from_qualified_name(*right))
                    .collect::<Vec<_>>(),
            ),
            None,
            NullEquality::NullEqualsNothing,
        )
        .map_err(engine)?
        .build()
        .map_err(engine)
}

pub(crate) fn project(
    input: LogicalPlan,
    spec: &RelationSpec,
    alias: &str,
) -> Result<LogicalPlan, CompilerError> {
    LogicalPlanBuilder::from(input)
        .project(
            spec.columns
                .iter()
                .map(|field| column(alias, field.name()).alias(field.name())),
        )
        .map_err(engine)?
        .build()
        .map_err(engine)
}

pub(crate) fn column(alias: &str, name: &str) -> Expr {
    Expr::Column(Column::new(Some(alias), name))
}

pub(crate) fn engine(error: DataFusionError) -> CompilerError {
    pse_rules::errmap::classify(error, pse_catalog::PlanOrigin::RuleCompiler).into()
}

fn invalid(detail: &str) -> CompilerError {
    pse_templates::TemplateError::Binding {
        instance: pse_ids::SemanticId::NIL,
        detail: detail.to_owned(),
    }
    .into()
}

#[derive(Clone, Debug)]
pub(crate) struct Keyed<T> {
    pub(crate) row: T,
    pub(crate) key: String,
}

pub(crate) async fn keyed_rows<T: RelationRow>(
    arguments: &mut AlgorithmInputs,
    plan: LogicalPlan,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<Vec<Keyed<T>>, CompilerError> {
    Ok(
        keyed_rows_with_text::<T>(arguments, plan, None, session, registry, cancel)
            .await?
            .into_iter()
            .map(|(row, _)| row)
            .collect(),
    )
}

/// Read a typed source, its native key and an optional selected text value from
/// one execution. Nullable overrides retain their association with that exact row.
pub(crate) async fn keyed_rows_with_text<T: RelationRow>(
    arguments: &mut AlgorithmInputs,
    plan: LogicalPlan,
    value: Option<Expr>,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<Vec<(Keyed<T>, Option<String>)>, CompilerError> {
    let spec = T::relation(registry)?;
    let mut extras = vec![
        scalar::key(
            spec.primary_key
                .iter()
                .map(|name| (*name, col(*name)))
                .collect(),
        )
        .alias("configuration_source_key"),
    ];
    let has_value = value.is_some();
    if let Some(value) = value {
        extras.push(value.alias("configuration_selected_value"));
    }
    let plan = declare_relation_projection(plan, registry, spec, extras).map_err(engine)?;
    let plan = LogicalPlanBuilder::from(plan)
        .sort(
            spec.primary_key
                .iter()
                .map(|name| col(*name).sort(true, false)),
        )
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let completed = arguments.execute(plan, session, cancel).await?;
    let positions = (0..spec.columns.len()).collect::<Vec<_>>();
    let mut result = Vec::new();
    for batch in completed.batches() {
        cancel.checkpoint()?;
        let keys = batch
            .column(spec.columns.len())
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| invalid("algorithm source key is not Utf8"))?;
        let checked = FieldCheckedBatch::admit_owned_projection(registry, spec, batch, &positions)?;
        let values = if has_value {
            Some(
                batch
                    .column(spec.columns.len() + 1)
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or_else(|| invalid("algorithm selected value is not Utf8"))?,
            )
        } else {
            None
        };
        // Both values and keys are read from this exact completed batch. No independent
        // scan, sort, or execution order is used to reconstruct their correspondence.
        for (index, row) in T::rows(&checked)?.into_iter().enumerate() {
            if keys.is_null(index) {
                return Err(invalid("algorithm source key is null"));
            }
            result.push((
                Keyed {
                    row,
                    key: keys.value(index).to_owned(),
                },
                values.and_then(|values| {
                    (!values.is_null(index)).then(|| values.value(index).to_owned())
                }),
            ));
        }
    }
    Ok(result)
}

/// A typed algorithm argument coupled to the exact native source key and role.
#[derive(Clone, Debug)]
pub(crate) struct Located<T> {
    pub(crate) row: T,
    pub(crate) source: super::native_outputs::SourceKey,
}
impl<T> std::ops::Deref for Located<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.row
    }
}
impl<T> Located<T> {
    pub(crate) fn support(
        &self,
        support: &mut std::collections::BTreeSet<super::native_outputs::SourceKey>,
    ) {
        support.insert(self.source.clone());
    }
}
pub(crate) async fn located_input<T: RelationRow>(
    arguments: &mut AlgorithmInputs,
    session: &SnapshotSession,
    sources: &super::native_outputs::Sources,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<Vec<Located<T>>, CompilerError> {
    let spec = T::relation(registry)?;
    let (port, _) = sources.get(&spec.key).ok_or_else(|| {
        invalid(&format!(
            "typed input {} has no exact bound source role",
            spec.key
        ))
    })?;
    Ok(keyed_rows::<T>(
        arguments,
        scan(session, spec, "argument")?,
        session,
        registry,
        cancel,
    )
    .await?
    .into_iter()
    .map(|row| Located {
        row: row.row,
        source: super::native_outputs::SourceKey {
            relation: spec.key,
            port: port.clone(),
            key: row.key,
        },
    })
    .collect())
}
