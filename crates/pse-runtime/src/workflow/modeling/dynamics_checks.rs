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
            .chain(self.coordinates.parameters.iter().zip(&parameters))
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
            let create_worker =
                |mode: &SimulationMode, program: &Option<Arc<crate::math::ExecutableCase>>| {
                    program
                        .as_ref()
                        .map(|p| {
                            let providers = mode
                                .context
                                .providers
                                .values()
                                .map(|r| {
                                    r.worker_scoped(cancel.clone())
                                        .map(|w| (r.spec().key(), w))
                                        .map_err(|e| contract(e.to_string()))
                                })
                                .collect::<Result<_, _>>()?;
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
            for (index, sample) in report.samples.iter().enumerate() {
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
                self.update_sample_values(sample, parameters, point)?;
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
                let observed = match (program, active_worker) {
                    (Some(program), Some(worker)) => program
                        .assembly
                        .structure()
                        .rows()
                        .iter()
                        .map(|r| r.id)
                        .zip(
                            worker
                                .constraints(point)
                                .map_err(super::super::super::math)?,
                        )
                        .collect::<BTreeMap<_, _>>(),
                    _ => BTreeMap::new(),
                };
                let (mut sample_checks, reports) = results::assess_observations(
                    run_id,
                    product,
                    point,
                    &observed,
                    &mode.numerics,
                    &self.quantities,
                    terminal,
                    if terminal {
                        None
                    } else {
                        mode.sample_scope.as_ref()
                    },
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
                result.complete = report.samples.len() == self.profile.samples.len()
                    && report
                        .samples
                        .iter()
                        .zip(&self.profile.samples)
                        .all(|(s, t)| s.time == *t);
            }
            Err(error) => result.error = Some(error.boundary_diagnostic()),
        }
        result
    }
}
