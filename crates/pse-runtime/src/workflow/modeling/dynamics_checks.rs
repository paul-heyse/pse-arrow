// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The same original-model assessments at explicit trajectory samples.
use super::*;
use pse_model::generated::identities::RunId;
use std::sync::atomic::{AtomicBool, Ordering};

pub(in crate::workflow) struct SampleChecks {
    pub rows: Vec<ModelingCheck>,
    pub reports: Vec<ModelingReport>,
    pub complete: bool,
    pub error: Option<pse_model::diagnostic::BoundaryDiagnostic>,
}
impl ModelingSimulation {
    pub(in crate::workflow) fn update_sample_values(
        &self,
        sample: &native::Sample,
        parameters: &[f64],
        point: &mut CaseValues,
    ) -> Result<(), WorkflowError> {
        let parameters = self.profile.parameters_at(parameters, sample.time);
        self.update_point_values(sample, &parameters, point)
    }
    fn update_point_values(
        &self,
        sample: &native::Sample,
        parameters: &[f64],
        point: &mut CaseValues,
    ) -> Result<(), WorkflowError> {
        if sample.state.len() != self.coordinates.state.len()
            || parameters.len() != self.coordinates.parameters.len()
        {
            return Err(contract("trajectory check coordinate extent"));
        }
        point.scalars.insert(
            self.coordinates.time,
            (sample.time - self.coordinates.time_origin) / self.coordinates.time_scale,
        );
        for (coordinate, value) in self
            .coordinates
            .state
            .iter()
            .zip(&sample.state)
            .chain(self.coordinates.parameters.iter().zip(parameters))
        {
            point
                .scalars
                .insert(coordinate.id, value * coordinate.scale + coordinate.offset);
        }
        Ok(())
    }
    pub(in crate::workflow) fn check_samples(
        &self,
        run_id: RunId,
        report: &native::Report,
        parameters: &[f64],
        cancel: &Arc<AtomicBool>,
        started: std::time::Instant,
    ) -> SampleChecks {
        let mut result = SampleChecks {
            rows: Vec::new(),
            reports: Vec::new(),
            complete: false,
            error: None,
        };
        let mut run = || -> Result<(), WorkflowError> {
            for obligation in &self.modes[0].original_initial_conditions {
                let actual = report
                    .consistent_initial
                    .get(obligation.coordinate)
                    .ok_or_else(|| contract("original initial consistency state absent"))?;
                let coordinate = &self.coordinates.state[obligation.coordinate];
                let residual = actual * coordinate.scale + coordinate.offset - obligation.expected;
                if !residual.is_finite() {
                    return Err(contract("nonfinite original initial consistency residual"));
                }
                result.rows.push(results::temporal_initial_check(
                    run_id,
                    obligation.source,
                    obligation.row,
                    self.profile.start,
                    residual,
                    obligation.tolerance,
                ));
            }
            for (index, point) in report.conservation.iter().enumerate() {
                let mode = self
                    .modes
                    .get(point.mode)
                    .ok_or_else(|| contract("conservation point mode absent"))?;
                if point.defects.len() != self.contract.balances.len() {
                    return Err(contract("conservation point descriptor extent"));
                }
                for (balance, residual) in self.contract.balances.iter().zip(&point.defects) {
                    let descriptor = mode
                        .model
                        .compiled()
                        .model
                        .inventory_balances
                        .get(&balance.id)
                        .ok_or_else(|| contract("conserved subject absent in active mode"))?;
                    if !residual.is_finite() {
                        return Err(contract("nonfinite physical conservation residual"));
                    }
                    result.rows.push(results::temporal_closure_check(
                        run_id,
                        descriptor.lineage.declaration,
                        balance.id,
                        index,
                        point.time,
                        *residual,
                        balance.tolerance,
                    ));
                }
            }
            let create_worker =
                |mode: &SimulationMode, program: &Option<Arc<crate::math::ExecutableCase>>| {
                    program
                        .as_ref()
                        .map(|p| {
                            let providers =
                                crate::math::attempt_providers(&mode.context.providers, cancel)
                                    .map_err(|e| contract(e.to_string()))?;
                            Ok::<_, WorkflowError>(p.assembly.worker(providers, cancel.clone()))
                        })
                        .transpose()
                };
            let mut workers = self
                .modes
                .iter()
                .map(|mode| {
                    Ok::<_, WorkflowError>((
                        create_worker(mode, &mode.check_program)?,
                        create_worker(mode, &mode.terminal_check_program)?,
                        mode.context.values.clone(),
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let endpoint = report.endpoint.as_ref();
            let mut points = report.samples.iter().map(|s| (s, None)).collect::<Vec<_>>();
            if let Some(end) = endpoint {
                // Replace a coincident grid point's inputs; otherwise evaluate the separate endpoint.
                if let Some(point) = points.iter_mut().find(|(s, _)| s.time == end.point.time) {
                    *point = (&end.point, Some(end.inputs.as_slice()));
                } else {
                    points.push((&end.point, Some(end.inputs.as_slice())));
                }
            }
            for (index, (sample, endpoint_inputs)) in points.into_iter().enumerate() {
                let mode = self
                    .modes
                    .get(sample.mode)
                    .ok_or_else(|| contract("trajectory sample mode absent"))?;
                let product = mode.model.compiled();
                let (worker, terminal_worker, point) = workers
                    .get_mut(sample.mode)
                    .ok_or_else(|| contract("trajectory check mode absent"))?;
                if cancel.load(Ordering::Acquire) {
                    return Err(crate::math::MathRuntimeError::Cancelled.into());
                }
                if started.elapsed() > self.profile.time_limit {
                    return Err(crate::math::MathRuntimeError::Limit(
                        "trajectory check time limit",
                    )
                    .into());
                }
                if let Some(inputs) = endpoint_inputs {
                    self.update_point_values(sample, inputs, point)?;
                } else {
                    self.update_sample_values(sample, parameters, point)?;
                }
                let terminal = !product.model.integrals.is_empty()
                    && sample.time == self.profile.end
                    && report.termination == native::Termination::Completed;
                if terminal {
                    if sample.integrals.len() != self.contract.quadratures.len() {
                        return Err(contract("terminal quadrature extent"));
                    }
                    for (id, value) in self.contract.quadratures.iter().zip(&sample.integrals) {
                        point.scalars.insert(*id, *value);
                    }
                }
                let (program, active_worker) = if terminal {
                    (&mode.terminal_check_program, terminal_worker)
                } else {
                    (&mode.check_program, worker)
                };
                let mut applicability = Vec::new();
                let observed = match (program, active_worker) {
                    (Some(program), Some(worker)) => {
                        let evaluated = worker
                            .constraints(point)
                            .map_err(super::super::super::math)?;
                        applicability = worker.applicability_observations();
                        program
                            .assembly
                            .structure()
                            .rows()
                            .iter()
                            .map(|r| r.id)
                            .zip(evaluated)
                            .collect::<BTreeMap<_, _>>()
                    }
                    _ => BTreeMap::new(),
                };
                let (mut sample_checks, reports) = results::assess_observations(
                    run_id,
                    product,
                    point,
                    &observed,
                    &applicability,
                    &mode.numerics,
                    &self.quantities,
                    terminal,
                    if terminal {
                        None
                    } else {
                        mode.sample_scope.as_ref()
                    },
                    // An integrated sample has no certified objective bound.
                    None,
                )?;
                result.reports.extend(reports);
                for target in mode
                    .numerics
                    .targets
                    .iter()
                    .filter(|t| t.kind == NumericalTarget::Row)
                {
                    let value = observed
                        .get(&target.id)
                        .copied()
                        .ok_or_else(|| contract("trajectory algebraic residual absent"))?;
                    let row = product
                        .model
                        .equations
                        .iter()
                        .find(|r| r.id == target.id)
                        .ok_or_else(|| contract("trajectory algebraic source absent"))?;
                    sample_checks.push(ModelingCheck {
                        step: 0,
                        run_id,
                        sample_index: 0,
                        time: None,
                        target_id: target.id,
                        source_id: row.lineage.declaration,
                        kind: pse_model::generated::enums::ModelingCheckKind::OriginalEquation,
                        value: value.abs(),
                        tolerance: Some(target.budget),
                        satisfied: value.abs() <= target.budget,
                        within_validity: None,
                        extrapolation_allowed: None,
                        basis: pse_model::generated::enums::ModelingCheckBasis::Point,
                        layer: None,
                        claim_id: None,
                        claim_owner: None,
                        claim_owner_lineage: Vec::new(),
                        coverage_id: None,
                        evidence_id: None,
                        form_id: None,
                        call_id: None,
                        selected_records: Vec::new(),
                        dependencies: Vec::new(),
                        input_values: Vec::new(),
                        applicability_outcome: None,
                        applicability_basis: None,
                        permission_ids: Vec::new(),
                        unknown_allowed: None,
                        observation_instance: None,
                        applicability_required: None,
                        applicability_reason: None,
                        applicability_permissions: Vec::new(),
                    });
                }
                for check in &mut sample_checks {
                    check.sample_index = index as i64;
                    check.time = Some(sample.time);
                }
                result.rows.extend(sample_checks);
            }
            if cancel.load(Ordering::Acquire) {
                return Err(crate::math::MathRuntimeError::Cancelled.into());
            }
            if started.elapsed() > self.profile.time_limit {
                return Err(
                    crate::math::MathRuntimeError::Limit("trajectory check time limit").into(),
                );
            }
            Ok(())
        };
        match run() {
            Ok(()) => {
                let coverage = report.assess_endpoint(&self.profile);
                let fixed_integrals_complete = self.modes.iter().all(|m| {
                    let model = &m.model.compiled().model;
                    model
                        .integrals
                        .keys()
                        .all(|id| model.inventory_balances.values().any(|b| b.flux_id == *id))
                }) || report.completed_time == self.profile.end
                    && report.termination == native::Termination::Completed;
                result.complete = coverage.satisfied
                    && coverage.prefix_complete
                    && fixed_integrals_complete
                    && (self.contract.balances.is_empty()
                        || report
                            .conservation
                            .first()
                            .is_some_and(|p| p.time == self.profile.start)
                            && report
                                .conservation
                                .last()
                                .is_some_and(|p| p.time == report.completed_time));
            }
            Err(error) => result.error = Some(error.boundary_diagnostic()),
        }
        result
    }
}
