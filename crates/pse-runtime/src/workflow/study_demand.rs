// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit occurrence detail. Original completion is mandatory in every mode.

use super::{WorkflowError, contract};

/// Detail selected independently of the original completion and assessment.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StudyOutputDemand {
    /// Retain the complete original occurrence report and its public relations.
    Full {},
    /// Retain mandatory summaries and only the explicitly bounded selected detail.
    Compact {
        /// Selected detail for usable occurrences, including requested KPI relations.
        detail: StudyDetailDemand,
        /// Independently bounded detail for failed, flagged or cancelled occurrences.
        failed_detail: StudyDetailDemand,
    },
}

/// Exact named relations with a finite per-occurrence materialization allowance.
/// Exceeding the allowance refuses detail capture; it never silently truncates rows.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct StudyDetailDemand {
    /// Registry-owned result relations; an empty list requests no detail.
    pub relations: Vec<String>,
    /// Exact scientific members. Selected rows retain every original column.
    pub members: Vec<StudyMemberDemand>,
    /// Maximum total rows across these relations, including failed detail.
    pub maximum_rows: u32,
    /// Maximum retained physical detail bytes across these relations.
    pub maximum_bytes: u64,
}

/// Registry-typed scientific keys within an occurrence. Run identity is supplied
/// by the original occurrence rather than accepted from a detail selector.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StudyMemberDemand {
    /// One algebraic variable at its original absolute solve step.
    Variable {
        /// Original absolute solve step.
        step: i64,
        /// Authored variable identity.
        symbol_id: pse_ids::SemanticId,
    },
    /// One algebraic constraint at its original absolute solve step.
    Constraint {
        /// Original absolute solve step.
        step: i64,
        /// Authored constraint identity.
        row_id: pse_ids::SemanticId,
    },
    /// One fitted parameter.
    FitParameter {
        /// Authored fitted parameter identity.
        parameter_id: pse_ids::SemanticId,
    },
    /// One observation in its original experiment.
    FitObservation {
        /// Original experiment instance identity.
        experiment_id: pse_model::generated::identities::InstanceId,
        /// Authored observation identity within that experiment.
        observation_id: pse_ids::SemanticId,
    },
    /// The original samples of one trajectory symbol.
    Trajectory {
        /// Authored trajectory symbol identity.
        symbol_id: pse_ids::SemanticId,
    },
}
impl StudyMemberDemand {
    fn relation_name(&self) -> &'static str {
        match self {
            Self::Variable { .. } => "runtime.solve_variables",
            Self::Constraint { .. } => "runtime.solve_constraints",
            Self::FitParameter { .. } => "runtime.fit_parameters",
            Self::FitObservation { .. } => "runtime.fit_observations",
            Self::Trajectory { .. } => "runtime.simulation_samples",
        }
    }
    fn original_key(
        table: &pse_relations::columnar::FieldCheckedBatch,
        index: usize,
    ) -> Result<Self, WorkflowError> {
        use pse_relations::columnar::ArrowValue;
        use pse_relations::generated::runtime::*;
        let id = table.relation_id();
        Ok(if id == solve_variables::RELATION_ID {
            let view = solve_variables::View::from_checked(table).map_err(super::relation)?;
            Self::Variable {
                step: view.step_column().value(index),
                symbol_id: pse_ids::SemanticId::read(view.symbol_id_column(), index)
                    .map_err(super::relation)?,
            }
        } else if id == solve_constraints::RELATION_ID {
            let view = solve_constraints::View::from_checked(table).map_err(super::relation)?;
            Self::Constraint {
                step: view.step_column().value(index),
                row_id: pse_ids::SemanticId::read(view.row_id_column(), index)
                    .map_err(super::relation)?,
            }
        } else if id == fit_parameters::RELATION_ID {
            let view = fit_parameters::View::from_checked(table).map_err(super::relation)?;
            Self::FitParameter {
                parameter_id: pse_ids::SemanticId::read(view.parameter_id_column(), index)
                    .map_err(super::relation)?,
            }
        } else if id == fit_observations::RELATION_ID {
            let view = fit_observations::View::from_checked(table).map_err(super::relation)?;
            Self::FitObservation {
                experiment_id: pse_model::generated::identities::InstanceId::read(
                    view.experiment_id_column(),
                    index,
                )
                .map_err(super::relation)?,
                observation_id: pse_ids::SemanticId::read(view.observation_id_column(), index)
                    .map_err(super::relation)?,
            }
        } else if id == simulation_samples::RELATION_ID {
            let view = simulation_samples::View::from_checked(table).map_err(super::relation)?;
            Self::Trajectory {
                symbol_id: pse_ids::SemanticId::read(view.symbol_id_column(), index)
                    .map_err(super::relation)?,
            }
        } else {
            return Err(contract("relation has no declared scientific member key"));
        })
    }
}

impl StudyOutputDemand {
    pub(super) fn validate(&self, runtime: &super::Runtime) -> Result<(), WorkflowError> {
        match self {
            Self::Full {} => Ok(()),
            Self::Compact {
                detail,
                failed_detail,
            } => {
                detail.validate(runtime)?;
                failed_detail.validate(runtime)
            }
        }
    }

    pub(super) fn detail(&self, usable: bool) -> Option<&StudyDetailDemand> {
        match self {
            Self::Full {} => None,
            Self::Compact {
                detail,
                failed_detail,
            } => Some(if usable { detail } else { failed_detail }),
        }
    }
}

impl StudyDetailDemand {
    fn validate(&self, runtime: &super::Runtime) -> Result<(), WorkflowError> {
        if self.relations.is_empty() && self.members.is_empty() {
            if self.maximum_rows != 0 || self.maximum_bytes != 0 {
                return Err(contract(
                    "empty study detail has no materialization allowance",
                ));
            }
            return Ok(());
        }
        if self.maximum_rows == 0 || self.maximum_bytes == 0 {
            return Err(contract(
                "selected study detail needs positive finite allowances",
            ));
        }
        let extent = self
            .relations
            .len()
            .checked_add(self.members.len())
            .and_then(|n| n.checked_mul(128))
            .ok_or_else(|| contract("study demand validation extent"))?;
        let _owner = runtime
            .shared
            .math()
            .reserve("study:detail-key-validation", extent)?;
        let mut names = std::collections::BTreeSet::new();
        for name in &self.relations {
            if name.len() > 256
                || !names.insert(name)
                || runtime.registry().relation(name).is_none()
            {
                return Err(contract("unknown or duplicate study detail relation"));
            }
        }
        let mut members = std::collections::BTreeSet::new();
        for member in &self.members {
            if !members.insert(member)
                || matches!(member, StudyMemberDemand::Variable { step, .. } | StudyMemberDemand::Constraint { step, .. } if *step < 0)
            {
                return Err(contract("duplicate or invalid study member key"));
            }
        }
        Ok(())
    }
    pub(super) fn relation_names<'a>(&'a self) -> impl Iterator<Item = &'a str> + 'a {
        self.relations.iter().map(String::as_str).chain(
            self.members
                .iter()
                .map(|member| -> &'a str { member.relation_name() }),
        )
    }
    pub(super) fn selector(
        &self,
        name: &str,
        id: pse_ids::SemanticId,
        pool: std::sync::Arc<dyn pse_columnar::MemoryPool>,
    ) -> Result<StudyMemberSelection, WorkflowError> {
        if self.relations.iter().any(|whole| whole == name) {
            return Ok(StudyMemberSelection {
                id,
                keys: None,
                pool,
                _owner: None,
            });
        }
        if name_for_id(id) != name {
            return Err(contract("study member relation identity differs"));
        }
        let reservation =
            pse_columnar::MemoryConsumer::new("study:member-selection-keys").register(&pool);
        reservation
            .try_grow(
                self.members
                    .len()
                    .checked_mul(size_of::<StudyMemberDemand>() + 64)
                    .ok_or_else(|| contract("study member key extent"))?,
            )
            .map_err(pse_engine::EngineError::from)?;
        let keys = self
            .members
            .iter()
            .filter(|member| member.relation_name() == name)
            .cloned()
            .collect();
        Ok(StudyMemberSelection {
            id,
            keys: Some(keys),
            pool,
            _owner: Some(pse_columnar::AllocationLease::new(reservation)),
        })
    }
    /// Filter original checked rows before materialization and allowance accounting.
    /// A whole-relation request subsumes its member requests without duplicating rows.
    pub(super) fn select_cursor<'a>(
        &self,
        name: &str,
        id: pse_ids::SemanticId,
        mut cursor: super::ResultCursor<'a>,
        pool: std::sync::Arc<dyn pse_columnar::MemoryPool>,
    ) -> Result<super::ResultCursor<'a>, WorkflowError> {
        if self.relations.iter().any(|whole| whole == name) {
            return Ok(cursor);
        }
        let schema = cursor.schema();
        let selector = self.selector(name, id, pool)?;
        super::ResultCursor::new(
            schema,
            id,
            1024,
            super::ResultOrder::Public,
            move |request| async move {
                while let Some(batch) = cursor.next_batch().await? {
                    let selected = selector
                        .select(batch, &request.cancel)
                        .map_err(std::sync::Arc::new)?;
                    request.send_checked(selected).await?;
                }
                if !cursor.complete() {
                    return Err(std::sync::Arc::new(contract(
                        "member detail producer did not complete",
                    )));
                }
                Ok(())
            },
        )
    }
}

/// One admitted exact member-key set shared by cursor and immutable projection
/// consumers. It owns no scientific evaluation or producer restart permission.
pub(super) struct StudyMemberSelection {
    id: pse_ids::SemanticId,
    keys: Option<std::collections::BTreeSet<StudyMemberDemand>>,
    pool: std::sync::Arc<dyn pse_columnar::MemoryPool>,
    _owner: Option<std::sync::Arc<pse_columnar::AllocationLease>>,
}
impl StudyMemberSelection {
    pub(super) fn select(
        &self,
        batch: pse_relations::columnar::FieldCheckedBatch,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        if batch.relation_id() != self.id {
            return Err(contract("study member chunk relation differs"));
        }
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        let Some(keys) = &self.keys else {
            return Ok(batch);
        };
        let reservation =
            pse_columnar::MemoryConsumer::new("study:member-selection-mask").register(&self.pool);
        reservation
            .try_grow(
                batch
                    .batch()
                    .num_rows()
                    .checked_mul(16)
                    .ok_or_else(|| contract("study member mask extent"))?,
            )
            .map_err(pse_engine::EngineError::from)?;
        let mut mask =
            datafusion::arrow::array::BooleanBuilder::with_capacity(batch.batch().num_rows());
        for index in 0..batch.batch().num_rows() {
            mask.append_value(keys.contains(&StudyMemberDemand::original_key(&batch, index)?));
        }
        batch
            .filter_reserved(&mask.finish(), &self.pool, cancel)
            .map_err(super::relation)
    }
}

fn name_for_id(id: pse_ids::SemanticId) -> &'static str {
    use pse_relations::generated::runtime::*;
    if id == solve_variables::RELATION_ID {
        "runtime.solve_variables"
    } else if id == solve_constraints::RELATION_ID {
        "runtime.solve_constraints"
    } else if id == fit_parameters::RELATION_ID {
        "runtime.fit_parameters"
    } else if id == fit_observations::RELATION_ID {
        "runtime.fit_observations"
    } else if id == simulation_samples::RELATION_ID {
        "runtime.simulation_samples"
    } else {
        ""
    }
}

/// Original completion and selected checked chunks, without the numerical report owner.
#[derive(Debug)]
pub struct CompactStudyResult {
    /// Original occurrence identity.
    pub run_id: pse_model::generated::identities::RunId,
    completion: Result<super::Completion, std::sync::Arc<WorkflowError>>,
    detail:
        Result<std::collections::BTreeMap<String, CompactRelation>, std::sync::Arc<WorkflowError>>,
    _owner: std::sync::Arc<pse_columnar::AllocationLease>,
}
#[derive(Debug)]
struct CompactRelation {
    schema: datafusion::arrow::datatypes::SchemaRef,
    id: pse_ids::SemanticId,
    batches: Vec<pse_relations::columnar::FieldCheckedBatch>,
}
impl CompactStudyResult {
    /// Original scientific completion. Detail capture never recomputes or changes it.
    pub fn completion(&self) -> Result<&super::Completion, &WorkflowError> {
        self.completion.as_ref().map_err(AsRef::as_ref)
    }
    pub(super) fn completion_shared(
        &self,
    ) -> Result<&super::Completion, std::sync::Arc<WorkflowError>> {
        self.completion.as_ref().map_err(Clone::clone)
    }
    /// Selected detail names, or the separate materialization refusal.
    pub fn table_names(&self) -> Result<Vec<&str>, &WorkflowError> {
        self.detail
            .as_ref()
            .map(|detail| detail.keys().map(String::as_str).collect())
            .map_err(AsRef::as_ref)
    }
    /// Bounded original checked chunks. An empty selected relation retains its schema.
    pub fn table_cursor(
        &self,
        name: &str,
    ) -> Result<super::ResultCursor<'_>, std::sync::Arc<WorkflowError>> {
        let detail = self.detail.as_ref().map_err(Clone::clone)?;
        let relation = detail
            .get(name)
            .ok_or_else(|| std::sync::Arc::new(contract("study detail was not selected")))?;
        super::ResultCursor::from_retained(relation.schema.clone(), relation.id, &relation.batches)
            .map_err(std::sync::Arc::new)
    }
    /// Owning selected stream for a foreign-language consumer.
    pub fn into_table_cursor(
        self: std::sync::Arc<Self>,
        name: &str,
    ) -> Result<super::ResultCursor<'static>, std::sync::Arc<WorkflowError>> {
        let detail = self.detail.as_ref().map_err(Clone::clone)?;
        let relation = detail
            .get(name)
            .ok_or_else(|| std::sync::Arc::new(contract("study detail was not selected")))?;
        let id = relation.id;
        let schema = relation.schema.clone();
        super::ResultCursor::new(
            schema,
            id,
            1024,
            super::ResultOrder::Public,
            move |request| async move {
                let detail = self.detail.as_ref().map_err(Clone::clone)?;
                let relation = detail
                    .values()
                    .find(|relation| relation.id == id)
                    .ok_or_else(|| {
                        std::sync::Arc::new(contract("selected relation identity absent"))
                    })?;
                for batch in &relation.batches {
                    request.send_checked(batch.clone()).await?;
                }
                Ok(())
            },
        )
        .map_err(std::sync::Arc::new)
    }
    pub(super) async fn capture(
        result: &super::RunResult,
        demand: &StudyDetailDemand,
    ) -> Result<Self, WorkflowError> {
        // Reserve the owned summary before its clone. Arrow buffers already carry
        // their original pool owners; the selection envelope also has a finite grant.
        let summary_owner = super::durable::completion_retention_owner(
            &result.runtime.shared.pool(),
            &result.completion.as_ref().ok(),
        )?;
        let detail = capture_detail(result, demand)
            .await
            .map_err(std::sync::Arc::new);
        Ok(Self {
            run_id: result.run_id,
            completion: result.completion.clone(),
            detail,
            _owner: summary_owner,
        })
    }
}

async fn capture_detail(
    result: &super::RunResult,
    demand: &StudyDetailDemand,
) -> Result<std::collections::BTreeMap<String, CompactRelation>, WorkflowError> {
    let rows =
        usize::try_from(demand.maximum_rows).map_err(|_| contract("study detail row extent"))?;
    usize::try_from(demand.maximum_bytes).map_err(|_| contract("study detail byte extent"))?;
    let envelope = rows
        .checked_mul(size_of::<pse_relations::columnar::FieldCheckedBatch>() + 128)
        .and_then(|n| n.checked_add(demand.relation_names().count().checked_mul(1024)?))
        .ok_or_else(|| contract("study detail envelope extent"))?;
    let owner = result
        .runtime
        .shared
        .math()
        .reserve("study:selected-detail-envelope", envelope)?;
    let mut allowance = DetailAllowance::new(demand);
    let mut selected = std::collections::BTreeMap::new();
    // The declared physical allowance is checked against every original chunk;
    // buffer allocation itself remains charged by the existing result producer.
    for name in demand
        .relation_names()
        .collect::<std::collections::BTreeSet<_>>()
    {
        let cursor = result
            .table_cursor(name, 1024)
            .map_err(WorkflowError::Shared)?;
        let id = result
            .runtime
            .registry()
            .relation(name)
            .ok_or_else(|| contract("study detail relation disappeared"))?
            .id;
        #[cfg(test)]
        let cursor = controlled_capture_cursor(cursor, id)?;
        let mut cursor = demand.select_cursor(name, id, cursor, result.runtime.shared.pool())?;
        let mut relation = CompactRelation {
            schema: cursor.schema(),
            id,
            batches: Vec::new(),
        };
        while let Some(batch) = cursor.next_batch().await.map_err(WorkflowError::Shared)? {
            // The relation already retains its exact schema. Empty filtered
            // prefixes carry no selected members and must not grow the retained
            // chunk vector independently of the requested row allowance.
            if batch.batch().num_rows() == 0 {
                continue;
            }
            allowance.accept(&batch)?;
            relation
                .batches
                .push(batch.with_export_owner(owner.clone()));
        }
        if !cursor.complete() {
            return Err(contract("study detail producer did not complete"));
        }
        selected.insert(name.to_owned(), relation);
    }
    Ok(selected)
}

#[cfg(test)]
#[derive(Default)]
pub(super) struct CaptureControl {
    pub(super) pause_first: bool,
    pub(super) fail_after_last: bool,
    pub(super) claimed: std::sync::atomic::AtomicBool,
    pub(super) chunks: std::sync::atomic::AtomicUsize,
    pub(super) completed_siblings: std::sync::atomic::AtomicUsize,
    pub(super) entered: tokio::sync::Notify,
    pub(super) release: tokio::sync::Notify,
    pub(super) sibling_complete: tokio::sync::Notify,
}
#[cfg(test)]
tokio::task_local! { pub(super) static CAPTURE_CONTROL: std::sync::Arc<CaptureControl>; }
#[cfg(test)]
fn controlled_capture_cursor<'a>(
    mut cursor: super::ResultCursor<'a>,
    id: pse_ids::SemanticId,
) -> Result<super::ResultCursor<'a>, WorkflowError> {
    use std::sync::{Arc, atomic::Ordering};
    let Some(control) = CAPTURE_CONTROL.try_with(Arc::clone).ok() else {
        return Ok(cursor);
    };
    let first = !control.claimed.swap(true, Ordering::AcqRel);
    super::ResultCursor::new(
        cursor.schema(),
        id,
        1024,
        super::ResultOrder::Public,
        move |request| async move {
            let mut sent = false;
            while let Some(batch) = cursor.next_batch().await? {
                let nonempty = batch.batch().num_rows() != 0;
                request.send_checked(batch).await?;
                if nonempty {
                    control.chunks.fetch_add(1, Ordering::AcqRel);
                    if first && !sent && control.pause_first {
                        control.entered.notify_one();
                        control.release.notified().await;
                    }
                    sent = true;
                }
            }
            if !cursor.complete() {
                return Err(Arc::new(contract("controlled source did not complete")));
            }
            if control.fail_after_last {
                return Err(Arc::new(contract(
                    "controlled selected producer failed after last chunk",
                )));
            }
            if !first {
                control.completed_siblings.fetch_add(1, Ordering::AcqRel);
                control.sibling_complete.notify_one();
            }
            Ok(())
        },
    )
}

pub(super) struct DetailAllowance {
    rows: u64,
    bytes: u64,
}
impl DetailAllowance {
    pub(super) fn new(demand: &StudyDetailDemand) -> Self {
        Self {
            rows: u64::from(demand.maximum_rows),
            bytes: demand.maximum_bytes,
        }
    }
    pub(super) fn accept(
        &mut self,
        batch: &pse_relations::columnar::FieldCheckedBatch,
    ) -> Result<(), WorkflowError> {
        let rows = batch.batch().num_rows() as u64;
        let bytes = batch.batch().get_array_memory_size() as u64;
        let remaining_rows = self
            .rows
            .checked_sub(rows)
            .ok_or_else(|| contract("selected study detail exceeds requested row allowance"))?;
        let remaining_bytes = self
            .bytes
            .checked_sub(bytes)
            .ok_or_else(|| contract("selected study detail exceeds requested byte allowance"))?;
        self.rows = remaining_rows;
        self.bytes = remaining_bytes;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn study_member_detail_preserves_original_step_keys_bits_and_empty_schema() {
        use pse_relations::generated::{enums::DualQualification, runtime::solve_variables};
        let registry = pse_schema::registry().unwrap();
        let validation = pse_relations::validate::ValidationContext::new(
            registry,
            pse_engine::validation::NativeValidation(
                datafusion::prelude::SessionContext::new().state(),
            ),
        );
        let run_id = pse_operations::mint_id();
        let symbol_id = pse_ids::SemanticId::from_bytes([7; 16]);
        let precise = f64::from_bits(1.0_f64.to_bits() + 1);
        let mut builder =
            solve_variables::Builder::with_registry(registry, 3, &validation).unwrap();
        for (step, value) in [(0, Some(1.0)), (7, Some(-0.0)), (8, Some(precise))] {
            builder
                .push(solve_variables::Row {
                    run_id,
                    step,
                    symbol_id,
                    quantity_id: None,
                    unit_id: None,
                    fixed: false,
                    parameter: false,
                    domain: None,
                    value,
                    lower: None,
                    upper: None,
                    lower_violation: None,
                    upper_violation: None,
                    tolerance: None,
                    lower_dual: None,
                    upper_dual: None,
                    reduced_cost: None,
                    stationarity: None,
                    dual_qualification: DualQualification::Unavailable,
                })
                .unwrap();
        }
        let original = builder.finish().unwrap();
        let original_batches = [original];
        let pool: std::sync::Arc<dyn pse_columnar::MemoryPool> = std::sync::Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(1 << 20),
        );
        let demand = StudyDetailDemand {
            relations: Vec::new(),
            members: vec![
                StudyMemberDemand::Variable { step: 7, symbol_id },
                StudyMemberDemand::Variable { step: 8, symbol_id },
            ],
            maximum_rows: 2,
            maximum_bytes: 1 << 20,
        };
        let cursor = super::super::ResultCursor::from_retained(
            original_batches[0].batch().schema(),
            solve_variables::RELATION_ID,
            &original_batches,
        )
        .unwrap();
        let mut selected = demand
            .select_cursor(
                "runtime.solve_variables",
                solve_variables::RELATION_ID,
                cursor,
                pool.clone(),
            )
            .unwrap();
        let batch = selected.next_batch().await.unwrap().unwrap();
        let view = solve_variables::View::from_checked(&batch).unwrap();
        assert_eq!(view.len(), 2);
        assert_eq!(view.row(0).unwrap().step, 7);
        assert_eq!(
            view.row(0).unwrap().value.unwrap().to_bits(),
            (-0.0_f64).to_bits()
        );
        assert_eq!(
            view.row(1).unwrap().value.unwrap().to_bits(),
            precise.to_bits()
        );
        assert_eq!(batch.batch().schema(), original_batches[0].batch().schema());
        let mut allowance = DetailAllowance::new(&demand);
        allowance.accept(&batch).unwrap();
        assert!(selected.next_batch().await.unwrap().is_none());
        assert!(selected.complete());
        let missing = StudyDetailDemand {
            members: vec![StudyMemberDemand::Variable { step: 9, symbol_id }],
            ..demand.clone()
        };
        let cursor = super::super::ResultCursor::from_retained(
            original_batches[0].batch().schema(),
            solve_variables::RELATION_ID,
            &original_batches,
        )
        .unwrap();
        let mut selected = missing
            .select_cursor(
                "runtime.solve_variables",
                solve_variables::RELATION_ID,
                cursor,
                pool.clone(),
            )
            .unwrap();
        assert_eq!(selected.schema(), original_batches[0].batch().schema());
        assert_eq!(
            selected
                .next_batch()
                .await
                .unwrap()
                .unwrap()
                .batch()
                .num_rows(),
            0
        );
        assert!(selected.next_batch().await.unwrap().is_none());
        assert!(selected.complete());
        let whole = StudyDetailDemand {
            relations: vec!["runtime.solve_variables".into()],
            ..demand
        };
        let cursor = super::super::ResultCursor::from_retained(
            original_batches[0].batch().schema(),
            solve_variables::RELATION_ID,
            &original_batches,
        )
        .unwrap();
        let mut selected = whole
            .select_cursor(
                "runtime.solve_variables",
                solve_variables::RELATION_ID,
                cursor,
                pool,
            )
            .unwrap();
        assert_eq!(
            selected
                .next_batch()
                .await
                .unwrap()
                .unwrap()
                .batch()
                .num_rows(),
            3
        );
    }
    #[test]
    fn study_output_requires_explicit_complete_success_and_failure_contracts() {
        let full = StudyOutputDemand::Full {};
        assert_eq!(
            serde_json::to_value(&full).unwrap(),
            serde_json::json!({"kind":"full"})
        );
        assert_eq!(
            serde_json::from_value::<StudyOutputDemand>(serde_json::json!({"kind":"full"}))
                .unwrap(),
            full
        );
        assert!(serde_json::from_str::<StudyOutputDemand>(r#"{}"#).is_err());
        assert!(
            serde_json::from_str::<StudyOutputDemand>(
                r#"{"kind":"compact","detail":{"relations":[],"maximum_rows":0,"maximum_bytes":0}}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<StudyOutputDemand>(r#"{"kind":"full","detail":{}}"#).is_err()
        );
        let demand = StudyOutputDemand::Compact {
            detail: StudyDetailDemand {
                relations: vec!["runtime.solve_variables".into()],
                members: Vec::new(),
                maximum_rows: 8,
                maximum_bytes: 4096,
            },
            failed_detail: Default::default(),
        };
        let decoded =
            serde_json::from_slice::<StudyOutputDemand>(&serde_json::to_vec(&demand).unwrap())
                .unwrap();
        assert_eq!(demand, decoded);
        assert_eq!(decoded.detail(false).unwrap().maximum_rows, 0);
        assert!(StudyOutputDemand::Full {}.detail(true).is_none());
    }
}
