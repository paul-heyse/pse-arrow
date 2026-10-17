// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The same original-model assessments at explicit trajectory samples.
use super::*;
use pse_model::generated::identities::RunId;

/// Merge retained points with their original ordered coverage in one pass. Compact
/// reports may retain a subset; a separately retained endpoint keeps its live inputs.
fn check_points<'a>(
    samples: &'a [native::Sample],
    coverage: &'a [(f64, usize)],
    endpoint: Option<&'a native::TrajectoryEndpoint>,
    include_samples: bool,
) -> impl Iterator<Item = (usize, &'a native::Sample, Option<&'a [f64]>)> {
    let endpoint_index = endpoint.map(|end| {
        coverage
            .iter()
            .position(|p| p.0 == end.point.time)
            .unwrap_or(coverage.len())
    });
    let endpoint_in_samples = include_samples
        && endpoint.is_some_and(|end| samples.iter().any(|s| s.time == end.point.time));
    let mut points = coverage.iter().enumerate().peekable();
    let retained = samples
        .iter()
        .filter(move |_| include_samples)
        .map(move |sample| {
            while points.peek().is_some_and(|(_, p)| p.0 < sample.time) {
                points.next();
            }
            let index = points
                .peek()
                .filter(|(_, p)| p.0 == sample.time)
                .map_or(coverage.len(), |(i, _)| *i);
            match endpoint.filter(|end| end.point.time == sample.time) {
                Some(end) => (index, &end.point, Some(end.inputs.as_slice())),
                None => (index, sample, None),
            }
        });
    let tail = endpoint.filter(|_| !endpoint_in_samples).map(move |end| {
        (
            endpoint_index.unwrap_or(coverage.len()),
            &end.point,
            Some(end.inputs.as_slice()),
        )
    });
    retained.chain(tail)
}

pub(in crate::workflow) struct SampleChecks {
    pub rows: Vec<ModelingCheck>,
    pub reports: Vec<ModelingReport>,
    pub complete: bool,
    pub error: Option<pse_model::diagnostic::BoundaryDiagnostic>,
}
impl ModelingSimulation {
    pub(super) fn sample_check_worker<'a>(
        &'a self,
        run_id: RunId,
        parameters: &'a [f64],
        scope: &'a pse_kernels::ExecutionScope,
    ) -> Result<SampleCheckWorker<'a>, WorkflowError> {
        let create_worker =
            |mode: &SimulationMode, program: &Option<Arc<crate::math::ExecutableCase>>| {
                program
                    .as_ref()
                    .map(|p| {
                        let providers =
                            crate::math::attempt_providers(&mode.context.providers, scope)
                                .map_err(|e| {
                                    WorkflowError::Math(crate::math::MathRuntimeError::Solve(
                                        ProblemError::Provider(e),
                                    ))
                                })?;
                        Ok::<_, WorkflowError>(p.assembly.worker_scoped(providers, scope.clone()))
                    })
                    .transpose()
            };
        let workers = self
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
        Ok(SampleCheckWorker {
            simulation: self,
            run_id,
            parameters,
            scope,
            workers,
        })
    }

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
        scope: &pse_kernels::ExecutionScope,
    ) -> SampleChecks {
        self.check_samples_impl(run_id, report, parameters, scope, true)
    }
    /// Initial, closure and endpoint checks complement the already checked live samples.
    #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
    pub(super) fn finish_live_checks(
        &self,
        run_id: RunId,
        report: &native::Report,
        parameters: &[f64],
        scope: &pse_kernels::ExecutionScope,
    ) -> SampleChecks {
        self.check_samples_impl(run_id, report, parameters, scope, false)
    }
    fn check_samples_impl(
        &self,
        run_id: RunId,
        report: &native::Report,
        parameters: &[f64],
        scope: &pse_kernels::ExecutionScope,
        include_samples: bool,
    ) -> SampleChecks {
        let interval = crate::invocation::stage("dynamic.original_physical_checks");
        let mut result = SampleChecks {
            rows: Vec::new(),
            reports: Vec::new(),
            complete: false,
            error: None,
        };
        let checkpoint = || {
            scope.check().map_err(|error| {
                WorkflowError::from(crate::math::MathRuntimeError::from(ProblemError::Provider(
                    error,
                )))
            })
        };
        let mut run = || -> Result<(), WorkflowError> {
            checkpoint()?;
            for obligation in &self.modes[0].original_initial_conditions {
                checkpoint()?;
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
                checkpoint()?;
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
            let mut workers = self.sample_check_worker(run_id, parameters, scope)?;
            for (index, sample, endpoint_inputs) in check_points(
                &report.samples,
                &report.coverage.points,
                report.endpoint.as_ref(),
                include_samples,
            ) {
                let terminal = sample.time == self.profile.end
                    && report.termination == native::Termination::Completed;
                workers.check(index, sample, endpoint_inputs, terminal, &mut result)?;
            }
            checkpoint()?;
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
        interval.finish(if result.error.is_none() && result.complete {
            crate::invocation::InvocationOutcome::Completed
        } else {
            crate::invocation::InvocationOutcome::Failed
        });
        result
    }
}

/// Original-space check scratch stays on the live native owner's thread.
pub(super) struct SampleCheckWorker<'a> {
    simulation: &'a ModelingSimulation,
    run_id: RunId,
    parameters: &'a [f64],
    scope: &'a pse_kernels::ExecutionScope,
    workers: Vec<(
        Option<pse_math::assembly::CaseWorker>,
        Option<pse_math::assembly::CaseWorker>,
        CaseValues,
    )>,
}
impl SampleCheckWorker<'_> {
    pub(super) fn check(
        &mut self,
        index: usize,
        sample: &native::Sample,
        endpoint_inputs: Option<&[f64]>,
        terminal: bool,
        result: &mut SampleChecks,
    ) -> Result<(), WorkflowError> {
        let simulation = self.simulation;
        let run_id = self.run_id;
        let parameters = self.parameters;
        let checkpoint = || {
            self.scope.check().map_err(|error| {
                WorkflowError::from(crate::math::MathRuntimeError::from(ProblemError::Provider(
                    error,
                )))
            })
        };
        let mode = simulation
            .modes
            .get(sample.mode)
            .ok_or_else(|| contract("trajectory sample mode absent"))?;
        let product = mode.model.compiled();
        let terminal = terminal && !product.model.integrals.is_empty();
        let (worker, terminal_worker, point) = self
            .workers
            .get_mut(sample.mode)
            .ok_or_else(|| contract("trajectory check mode absent"))?;
        checkpoint()?;
        if let Some(inputs) = endpoint_inputs {
            simulation.update_point_values(sample, inputs, point)?;
        } else {
            simulation.update_sample_values(sample, parameters, point)?;
        }
        if terminal {
            if sample.integrals.len() != simulation.contract.quadratures.len() {
                return Err(contract("terminal quadrature extent"));
            }
            for (id, value) in simulation
                .contract
                .quadratures
                .iter()
                .zip(&sample.integrals)
            {
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
                let evaluated = worker.constraints(&*point).map_err(math)?;
                let observed = program
                    .assembly
                    .structure()
                    .rows()
                    .iter()
                    .map(|r| r.id)
                    .zip(evaluated.iter().copied())
                    .collect::<BTreeMap<_, _>>();
                applicability = worker.applicability_observations();
                observed
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
            &simulation.quantities,
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
        Ok(())
    }
}

#[cfg(test)]
mod ordered_check_tests {
    use super::*;
    fn sample(time: f64, mode: usize) -> native::Sample {
        native::Sample {
            time,
            mode,
            integrals: vec![],
            state: vec![],
            outputs: vec![],
            state_sensitivities: vec![],
            output_sensitivities: vec![],
        }
    }
    #[test]
    fn ordered_sample_checks_preserve_sparse_ordinals_and_endpoint_inputs() {
        let samples = vec![sample(0., 0), sample(2., 0), sample(4., 1)];
        let coverage = [(0., 0), (1., 0), (2., 0), (3., 1), (4., 1)];
        let endpoint = native::TrajectoryEndpoint {
            point: sample(4., 2),
            event: None,
            input_columns: vec![0],
            inputs: vec![7.],
        };
        let points = check_points(&samples, &coverage, Some(&endpoint), true).collect::<Vec<_>>();
        assert_eq!(
            points.iter().map(|p| p.0).collect::<Vec<_>>(),
            vec![0, 2, 4]
        );
        assert_eq!(points[2].1.mode, 2);
        assert_eq!(points[2].2, Some([7.].as_slice()));
        let compact = check_points(&samples, &coverage, Some(&endpoint), false).collect::<Vec<_>>();
        assert_eq!(compact.len(), 1);
        assert_eq!(compact[0].0, 4);
    }
    #[test]
    fn ordered_sample_checks_keep_off_grid_endpoint_after_partial_prefix() {
        let samples = vec![sample(0., 0), sample(1., 0)];
        let coverage = [(0., 0), (1., 0)];
        let endpoint = native::TrajectoryEndpoint {
            point: sample(1.5, 1),
            event: None,
            input_columns: vec![],
            inputs: vec![],
        };
        let points = check_points(&samples, &coverage, Some(&endpoint), true).collect::<Vec<_>>();
        assert_eq!(
            points.iter().map(|p| (p.0, p.1.time)).collect::<Vec<_>>(),
            vec![(0, 0.), (1, 1.), (2, 1.5)]
        );
        assert_eq!(check_points(&samples, &coverage, None, false).count(), 0);
    }
}
