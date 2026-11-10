// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read original physical member products; unavailable coverage never executes science.
use super::super::{Completion, StudyOccurrenceResult, StudyReport, WorkflowError, contract};
use super::*;
use pse_model::generated::enums::{NativeBoundaryClass, StudyPointState};
use pse_relations::columnar::{FieldCheckedBatch, RelationRow};

fn physical_failure(outcome: &pse_model::study::PointOutcome) -> Option<bool> {
    if outcome.scientific.usable {
        return Some(false);
    }
    if outcome.lifecycle == StudyPointState::Cancelled {
        return None;
    }
    match outcome
        .diagnostic
        .as_ref()
        .map(|diagnostic| diagnostic.class)
    {
        Some(
            NativeBoundaryClass::InvalidModel
            | NativeBoundaryClass::TrialRejected
            | NativeBoundaryClass::Nonfinite
            | NativeBoundaryClass::Numerical,
        ) => Some(true),
        Some(_) => None,
        None if outcome.scientific.candidate_use.is_some() => Some(true),
        None => None,
    }
}

async fn original_rows(
    report: &StudyReport,
    result: &StudyOccurrenceResult,
    name: &str,
    cancel: &crate::CancelSource,
) -> Result<(Option<Completion>, Vec<FieldCheckedBatch>), WorkflowError> {
    let mut batches = Vec::new();
    match result {
        StudyOccurrenceResult::Retained { run, attempt, .. } => {
            let data = report.services().retained_data_services();
            let completion = data
                .stored_completion(run, attempt)
                .await?
                .completion
                .clone();
            if !completion
                .as_ref()
                .and_then(|completion| completion.permission.as_ref())
                .is_some_and(super::super::ScientificPermission::permits_result)
            {
                return Ok((completion, batches));
            }
            let selected = data.canonical_store()?.read_results(run, attempt).await?;
            if !selected.sets().iter().any(|set| set.name == name) {
                return Ok((completion, batches));
            }
            let mut reader = data
                .results(run, attempt, name, 0, u64::MAX, cancel.token())
                .await?;
            while let Some(batch) = reader.next_relation_batch().await? {
                batches.push(batch);
            }
            Ok((completion, batches))
        }
        StudyOccurrenceResult::Ephemeral(result) => {
            let completion = result.completion().ok().cloned();
            if !completion
                .as_ref()
                .and_then(|completion| completion.permission.as_ref())
                .is_some_and(super::super::ScientificPermission::permits_result)
            {
                return Ok((completion, batches));
            }
            if !result.table_names().iter().any(|relation| relation == name) {
                return Ok((completion, batches));
            }
            let mut reader = result
                .table_cursor(name, 1024)
                .map_err(WorkflowError::Shared)?;
            while let Some(batch) = reader.next_chunk().map_err(WorkflowError::Shared)? {
                batches.push(batch);
            }
            Ok((completion, batches))
        }
        StudyOccurrenceResult::Compact(result) => {
            let completion = result.completion().ok().cloned();
            if !completion
                .as_ref()
                .and_then(|completion| completion.permission.as_ref())
                .is_some_and(super::super::ScientificPermission::permits_result)
            {
                return Ok((completion, batches));
            }
            if !result
                .table_names()
                .ok()
                .is_some_and(|names| names.contains(&name))
            {
                return Ok((completion, batches));
            }
            let mut reader = result.table_cursor(name).map_err(WorkflowError::Shared)?;
            while let Some(batch) = reader.next_chunk().map_err(WorkflowError::Shared)? {
                batches.push(batch);
            }
            Ok((completion, batches))
        }
    }
}

impl ExpectationIteration {
    /// Select outputs from the original terminal finite child, including failed/unattempted members.
    /// Durable results are decoded through the protected canonical reader after producer release.
    pub async fn read_observations(
        &self,
        admitted: &AdmittedExpectation,
        report: &StudyReport,
        cancel: &crate::CancelSource,
    ) -> Result<Vec<MemberObservation>, WorkflowError> {
        if report
            .canonical_study_id()
            .is_some_and(|child| child != self.child)
            || report.outcomes.len() != self.samples.len()
            || report.results.len() != self.samples.len()
            || report.definition.point_count() != self.samples.len()
            || admitted.request.source.check(&report._source).is_err()
        {
            return Err(contract(
                "expectation terminal report is not its original finite child",
            ));
        }
        let mut observations = Vec::with_capacity(self.samples.len());
        for (index, sample) in self.samples.iter().enumerate() {
            cancel
                .token()
                .checkpoint()
                .map_err(pse_engine::EngineError::from)?;
            let outcome = &report.outcomes[index];
            if outcome.key.0 as usize != index
                || matches!(
                    outcome.lifecycle,
                    StudyPointState::Pending | StudyPointState::Assigned
                )
            {
                return Err(contract(
                    "expectation original child occurrence is not settled",
                ));
            }
            let scientific_failure = physical_failure(outcome);
            let mut observation = MemberObservation {
                child: self.child,
                ordinal: sample.ordinal,
                values: vec![None; admitted.outputs.len()],
                scientific_failure,
                operational_incomplete: scientific_failure.is_none(),
            };
            if outcome.scientific.usable
                && let Some(result) = &report.results[index]
            {
                match &admitted.request.member {
                    ExpectationMember::Observation(_) => {
                        let (_, batches) =
                            original_rows(report, result, "runtime.selected_observations", cancel)
                                .await?;
                        for batch in batches {
                            let rows =
                                pse_model::generated::runtime::selected_observations::Row::rows(
                                    &batch,
                                )
                                .map_err(super::super::relation)?;
                            for row in rows {
                                if row.run_id != result.run_id() {
                                    return Err(contract(
                                        "expectation selected output original run differs",
                                    ));
                                }
                                if let Some(output) = admitted
                                    .outputs
                                    .iter()
                                    .position(|output| output.request.output == row.member_id)
                                {
                                    if row.quantity_id.as_id()
                                        != admitted.outputs[output].request.quantity.as_id()
                                        || observation.values[output].is_some()
                                        || !row.canonical.is_finite()
                                    {
                                        return Err(contract(
                                            "expectation selected physical output quantity, coverage or value differs",
                                        ));
                                    }
                                    observation.values[output] = Some(row.canonical);
                                }
                            }
                        }
                    }
                    ExpectationMember::Case(_) => {
                        let (completion, batches) =
                            original_rows(report, result, "runtime.solve_variables", cancel)
                                .await?;
                        let Some(completion) = completion else {
                            observations.push(observation);
                            continue;
                        };
                        let step = completion.solves.iter().map(|solve| solve.step).max();
                        let permitted = step.is_some_and(|step| {
                            completion.step_permissions.iter().any(|permission| {
                                permission.step == Some(step) && permission.permits_result()
                            })
                        });
                        if permitted {
                            for batch in batches {
                                let rows =
                                    pse_model::generated::runtime::solve_variables::Row::rows(
                                        &batch,
                                    )
                                    .map_err(super::super::relation)?;
                                for row in rows {
                                    if Some(row.step) != step {
                                        continue;
                                    }
                                    if row.run_id != result.run_id() {
                                        return Err(contract(
                                            "expectation solve output original run differs",
                                        ));
                                    }
                                    if let Some(output) = admitted
                                        .outputs
                                        .iter()
                                        .position(|output| output.request.output == row.symbol_id)
                                    {
                                        let expected = &admitted.outputs[output].request;
                                        if row.quantity_id != Some(expected.quantity.as_id())
                                            || observation.values[output].is_some()
                                        {
                                            return Err(contract(
                                                "expectation original solve physical output differs",
                                            ));
                                        }
                                        if let (Some(value), Some(unit)) = (row.value, row.unit_id)
                                        {
                                            let plan =
                                                pse_quantity::CanonicalConversionPlan::registered(
                                                    &admitted.original.quantities,
                                                    expected.quantity,
                                                    pse_quantity::UnitId::from_id(unit),
                                                )
                                                .map_err(|error| {
                                                    super::super::math(
                                                        pse_math::MathError::Quantity(error),
                                                    )
                                                })?;
                                            observation.values[output] = Some(
                                                plan.apply(value)
                                                    .map_err(|error| {
                                                        super::super::math(
                                                            pse_math::MathError::Quantity(error),
                                                        )
                                                    })?
                                                    .value(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            observations.push(observation);
        }
        Ok(observations)
    }
}
