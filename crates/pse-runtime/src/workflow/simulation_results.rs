// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical trajectories and exact execution provenance use generated owned relations.
use super::{RunReport, RunRequest, RunResult, WorkflowError, contract, relation};
use pse_ids::SemanticId;
use pse_relations::{columnar::FieldCheckedBatch, generated::runtime::computation_runs};
use std::collections::BTreeMap;
impl RunResult {
    pub(super) fn encode_simulation(
        &self,
    ) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
        let RunRequest::Simulation(p) = &self.request else {
            return Err(contract("simulation request mismatch"));
        };
        let mut batches = match &self.report {
            Ok(RunReport::Simulation(r)) => r.tables()?,
            _ => BTreeMap::new(),
        };
        let registry = &self.runtime.registry;
        let mut header = computation_runs::Builder::with_registry(registry, 1).map_err(relation)?;
        header
            .push(
                self.completion()
                    .map_err(|e| contract(e.to_string()))?
                    .computation
                    .clone()
                    .ok_or_else(|| contract("simulation completion absent"))?,
            )
            .map_err(relation)?;
        batches.insert(
            computation_runs::RELATION_ID,
            header.finish().map_err(relation)?,
        );
        batches.extend(p.source.source_tables()?);
        self.retain_sources(&mut batches)?;
        Ok(batches)
    }
    #[cfg(feature = "solver-diffsol")]
    pub(super) fn encode_shooting(
        &self,
    ) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
        let RunRequest::Shooting { problem, .. } = &self.request else {
            return Err(contract("shooting request mismatch"));
        };
        let report = match &self.report {
            Ok(RunReport::Shooting(report)) => Some(report.as_ref()),
            _ => None,
        };
        let staging = pse_columnar::MemoryConsumer::new("workflow:shooting-encoding")
            .register(&self.runtime.shared.pool());
        staging
            .try_grow(problem.encoding_bytes(report)?)
            .map_err(|e| WorkflowError::Math(e.into()))?;
        let mut batches = BTreeMap::new();
        if let Ok(RunReport::Shooting(report)) = &self.report
            && let Some(trajectory) = &report.trajectory
        {
            // Share the native trajectory; qualification is copied
            // from the joined shooting result and never recomputed by this exporter.
            let owner = self
                ._owner
                .clone()
                .ok_or_else(|| contract("shooting result ownership absent"))?;
            batches = problem
                .simulation
                .completed_trajectory(
                    self.run_id,
                    trajectory.clone(),
                    super::modeling::dynamics::checks::SampleChecks {
                        rows: report.checks.clone(),
                        reports: report.reports.clone(),
                        complete: report.checks_complete,
                        error: report.validation_error.clone(),
                    },
                    report.completion.clone(),
                    owner,
                )
                .tables_for_kind(pse_model::generated::enums::ComputationKind::Shooting)?;
        }
        use pse_model::generated::enums::{
            DualQualification, ModelingVariableDomain, NumericalTarget,
        };
        use pse_relations::generated::runtime::{
            solve_constraints, solve_metrics, solve_variables,
        };
        let native = report.and_then(|r| r.solve.as_ref());
        let candidate = report.and_then(|r| r.candidate.as_ref());
        let policy = problem.numerics();
        let metadata = |id, kind| {
            policy
                .targets
                .iter()
                .find(|t| t.id == id && t.kind == kind)
                .ok_or_else(|| contract("shooting coordinate numerical metadata absent"))
        };
        let mut variables = solve_variables::Builder::with_registry(
            &self.runtime.registry,
            problem.contract().variables.len(),
        )
        .map_err(relation)?;
        for (i, variable) in problem.contract().variables.iter().enumerate() {
            let target = metadata(variable.id, NumericalTarget::Variable)?;
            variables
                .push(solve_variables::Row {
                    run_id: self.run_id,
                    step: 0,
                    symbol_id: variable.id,
                    quantity_id: Some(target.quantity),
                    unit_id: Some(target.unit),
                    fixed: false,
                    parameter: false,
                    domain: Some(ModelingVariableDomain::Continuous),
                    value: candidate.and_then(|c| c.get(i).copied()),
                    lower: variable.lower.is_finite().then_some(variable.lower),
                    upper: variable.upper.is_finite().then_some(variable.upper),
                    lower_violation: None,
                    upper_violation: None,
                    tolerance: Some(problem.tolerances().variables[i]),
                    lower_dual: None,
                    upper_dual: None,
                    reduced_cost: None,
                    stationarity: None,
                    dual_qualification: DualQualification::Unavailable,
                })
                .map_err(relation)?;
        }
        batches.insert(
            solve_variables::RELATION_ID,
            variables.finish().map_err(relation)?,
        );
        let mut constraints = solve_constraints::Builder::with_registry(
            &self.runtime.registry,
            problem.contract().rows.len(),
        )
        .map_err(relation)?;
        for (i, id) in problem.contract().rows.iter().enumerate() {
            let target = metadata(*id, NumericalTarget::Row)?;
            let (lower, upper) = problem.constraint_bounds()[i];
            constraints
                .push(solve_constraints::Row {
                    run_id: self.run_id,
                    step: 0,
                    row_id: *id,
                    quantity_id: Some(target.quantity),
                    unit_id: Some(target.unit),
                    value: report
                        .and_then(|r| r.constraint_values.as_ref())
                        .and_then(|v| v.get(i).copied()),
                    lower: lower.is_finite().then_some(lower),
                    upper: upper.is_finite().then_some(upper),
                    equality_residual: None,
                    lower_violation: None,
                    upper_violation: None,
                    tolerance: Some(problem.tolerances().rows[i]),
                    dual: None,
                    dual_qualification: DualQualification::Unavailable,
                })
                .map_err(relation)?;
        }
        batches.insert(
            solve_constraints::RELATION_ID,
            constraints.finish().map_err(relation)?,
        );
        let mut metrics =
            solve_metrics::Builder::with_registry(&self.runtime.registry, 0).map_err(relation)?;
        if let Some(native) = native {
            let stored = self.stored_events(0)?;
            let events = stored
                .as_ref()
                .map_or(super::results::StepEvents::Retained, |events| {
                    super::results::StepEvents::Stored(events)
                });
            super::results::push_native_metrics(&mut metrics, self.run_id, 0, native, events)?;
        }
        if let Some(report) = report {
            for (name, value) in [
                ("objective", report.objective),
                ("continuity", report.continuity),
            ] {
                if let Some(value) = value {
                    super::results::push_metric(
                        &mut metrics,
                        self.run_id,
                        0,
                        "shooting",
                        name,
                        &pse_backend_native::solve::Metric::Real(value),
                    )?;
                }
            }
        }
        batches.insert(
            solve_metrics::RELATION_ID,
            metrics.finish().map_err(relation)?,
        );
        let mut header = computation_runs::Builder::with_registry(&self.runtime.registry, 1)
            .map_err(relation)?;
        header
            .push(
                self.completion()
                    .map_err(|e| contract(e.to_string()))?
                    .computation
                    .clone()
                    .ok_or_else(|| contract("shooting completion absent"))?,
            )
            .map_err(relation)?;
        batches.insert(
            computation_runs::RELATION_ID,
            header.finish().map_err(relation)?,
        );
        batches.extend(problem.simulation.source.source_tables()?);
        self.retain_sources(&mut batches)?;
        Ok(batches)
    }
    pub(super) fn retain_sources(
        &self,
        batches: &mut BTreeMap<SemanticId, FieldCheckedBatch>,
    ) -> Result<(), WorkflowError> {
        self.numerical_tables(batches)?;
        use pse_relations::generated::runtime::run_lineage;
        let completion = self.completion().map_err(|e| contract(e.to_string()))?;
        let mut lineage =
            run_lineage::Builder::with_registry(&self.runtime.registry, completion.lineage.len())
                .map_err(relation)?;
        for row in &completion.lineage {
            lineage.push(row.clone()).map_err(relation)?;
        }
        batches.insert(
            run_lineage::RELATION_ID,
            lineage.finish().map_err(relation)?,
        );
        let cancel = pse_columnar::CancellationToken::new();
        for batch in batches.values_mut() {
            *batch = batch
                .retained(&self.runtime.shared.pool(), &cancel)
                .map_err(relation)?;
        }
        Ok(())
    }
}
