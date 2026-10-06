// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One occurrence projection for both executors; artifact owners attach available members.
use super::{
    OperationRequest, PointOutcome, StudyId, StudyPointDefinition, StudyReport, WorkflowError,
    diagnostic_rows,
};
use pse_model::generated::runtime::study_outcomes::*;
use pse_model::study::StartProvenance;
use pse_relations::columnar::{Collection, FieldCheckedBatch, RelationRow};

macro_rules! start_row {
    ($ctor:ident, $start:expr) => {{
        let start = $start;
        let (predecessor, role, seed_id, unavailable) = match start {
            StartProvenance::Fresh | StartProvenance::NotNeeded => (None, None, None, None),
            StartProvenance::Continuation {
                predecessor,
                role,
                seed,
            } => (
                Some(i64::from(predecessor.0)),
                Some(*role),
                Some(*seed),
                None,
            ),
            StartProvenance::Explicit { role, seed } => (None, Some(*role), Some(*seed), None),
            StartProvenance::FreshFallback {
                predecessor,
                role,
                reason,
            } => (
                Some(i64::from(predecessor.0)),
                Some(*role),
                None,
                Some(*reason),
            ),
        };
        $ctor {
            kind: start.kind(),
            predecessor,
            role,
            seed_id,
            unavailable,
        }
    }};
}

/// Project existing facts without inferring scientific success or artifact presence.
pub(in crate::workflow) fn outcome_row(
    study_id: StudyId,
    point: &StudyPointDefinition,
    outcome: &PointOutcome,
) -> RuntimeStudyOutcomesRow {
    let latest = outcome.attempts.last();
    RuntimeStudyOutcomesRow {
        study_id,
        point_index: i64::from(outcome.key.0),
        case_id: match &point.operation.operation {
            OperationRequest::DeclaredCase(case) => Some(case.case),
            OperationRequest::Simulation(simulation) => Some(simulation.case),
            OperationRequest::Fit(_) => None,
            OperationRequest::Horizon(horizon) => Some(horizon.plant.case),
        },
        binding_hash: point.binding_hash.as_id(),
        state: outcome.lifecycle,
        attempt_id: latest.and_then(|attempt| attempt.attempt_id),
        attempt_state: latest.and_then(|attempt| attempt.lifecycle),
        result_id: None,
        usable: outcome.scientific.usable,
        seed_permission: outcome.scientific.seed_permission,
        candidate_use: outcome.scientific.candidate_use,
        effect: outcome.effect,
        start: outcome
            .start
            .as_ref()
            .map(|start| start_row!(RuntimeStudyOutcomesFieldStart, start)),
        diagnostic: outcome
            .diagnostic
            .as_ref()
            .map(diagnostic_rows::project_study_diagnostic),
        attempts: outcome
            .attempts
            .iter()
            .map(|attempt| RuntimeStudyOutcomesFieldAttemptsItem {
                attempt_id: attempt.attempt_id,
                lifecycle: attempt.lifecycle,
                usable: attempt.scientific.usable,
                seed_permission: attempt.scientific.seed_permission,
                candidate_use: attempt.scientific.candidate_use,
                effect: attempt.effect,
                start: attempt
                    .start
                    .as_ref()
                    .map(|start| start_row!(RuntimeStudyOutcomesFieldAttemptsItemStart, start)),
                diagnostic: attempt
                    .diagnostic
                    .as_ref()
                    .map(diagnostic_rows::project_study_attempt_diagnostic),
            })
            .collect(),
    }
}

impl StudyReport {
    /// The same occurrence relation used by durable finalization, including unattempted points.
    pub fn table(&self) -> Result<FieldCheckedBatch, WorkflowError> {
        let study_id = StudyId::from_id(self.run_id.as_id());
        if self.outcomes.len() != self.definition.points.len()
            || self.results.len() != self.outcomes.len()
        {
            return Err(WorkflowError::Internal(
                "study occurrence extent changed after execution".into(),
            ));
        }
        let rows = self
            .definition
            .points
            .iter()
            .zip(&self.outcomes)
            .zip(&self.results)
            .map(|((point, outcome), result)| {
                let mut row = outcome_row(study_id, point, outcome);
                row.result_id = result.as_ref().map(|result| result.run_id());
                row
            })
            .collect();
        export::<RuntimeStudyOutcomesRow>(&self.runtime, rows)
    }
    /// Detailed diagnostics are projected independently of scientific member availability.
    pub fn findings_table(&self) -> Result<FieldCheckedBatch, WorkflowError> {
        let rows = self
            .outcomes
            .iter()
            .enumerate()
            .filter_map(|(index, outcome)| {
                outcome.diagnostic.as_ref().map(|diagnostic| {
                    diagnostic_rows::project_finding(self.run_id, index as i64, diagnostic)
                })
            })
            .collect();
        export::<pse_model::generated::runtime::modeling_findings::Row>(&self.runtime, rows)
    }
}
pub(super) fn export<T: RelationRow + pse_model::HeapUsage>(
    runtime: &super::Runtime,
    rows: Vec<T>,
) -> Result<FieldCheckedBatch, WorkflowError> {
    let bytes = rows
        .iter()
        .map(pse_model::HeapUsage::owned_bytes)
        .sum::<usize>();
    let lease = runtime.shared.math().reserve(
        "study:outcome-row-copy",
        bytes
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(4096))
            .ok_or_else(|| WorkflowError::Internal("study projection extent overflow".into()))?,
    )?;
    let pool = runtime.shared.pool();
    let cancel = pse_columnar::CancellationToken::new();
    let validation = runtime.validation_context()?;
    let mut columns = Collection::new(&runtime.registry, &pool, &cancel, &validation);
    columns.ensure::<T>().map_err(super::relation)?;
    for row in rows {
        columns.push(row).map_err(super::relation)?;
    }
    let batch = columns
        .finish()
        .map_err(super::relation)?
        .pop_first()
        .map(|(_, batch)| batch)
        .ok_or_else(|| WorkflowError::Internal("study outcome relation absent".into()))?;
    drop(lease);
    Ok(batch)
}
