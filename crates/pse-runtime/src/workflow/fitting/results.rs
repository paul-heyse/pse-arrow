// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Joined fitting observations use existing generated contracts.
use super::*;
use crate::workflow::{RunReport, RunRequest, RunResult, relation};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{
        enums::{DerivedQuantity, IntervalEnd, IntervalMethod},
        identities::RunId,
        runtime::{
            computation_runs, fit_constraints, fit_observations, fit_parameters, fit_variables,
            infeasibility_certificates, local_validity, parameter_covariances, parameter_intervals,
            profile_points, propagated_covariances, response_directions, response_sensitivities,
            solve_metrics,
        },
    },
};
impl RunResult {
    pub(crate) fn encode_fit(
        &self,
    ) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
        let RunRequest::Fit(p) = &self.request else {
            return Err(contract("fit request mismatch"));
        };
        let source = &p.source;
        let p = &p.problem;
        let report = match &self.report {
            Ok(RunReport::Fit(r)) => Some(r),
            Err(_) => None,
            _ => return Err(contract("fit report mismatch")),
        };
        let registry = &self.runtime.registry;
        let staging = pse_columnar::MemoryConsumer::new("workflow:fit-encoding")
            .register(&self.runtime.shared.pool());
        staging
            .try_grow(
                p.bytes
                    .checked_mul(4)
                    .ok_or_else(|| contract("fit encoding extent"))?,
            )
            .map_err(|e| WorkflowError::Math(e.into()))?;
        let mut header = computation_runs::Builder::with_registry(registry, 1).map_err(relation)?;
        header
            .push(
                self.completion()
                    .map_err(|e| contract(e.to_string()))?
                    .computation
                    .clone()
                    .ok_or_else(|| contract("missing completed computation"))?,
            )
            .map_err(relation)?;
        let mut parameters =
            fit_parameters::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut observations =
            fit_observations::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut responses =
            response_sensitivities::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut metrics = solve_metrics::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut states = fit_variables::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut constraints =
            fit_constraints::Builder::with_registry(registry, 0).map_err(relation)?;
        for (ei, e) in p.experiments.iter().enumerate() {
            if let Experiment::Steady(s) = e {
                let experiment = p.declaration.experiments[ei].experiment_id;
                for v in &s.variables {
                    let value = if v.fixed {
                        s.values.scalars.get(&v.port.id).copied()
                    } else {
                        s.coordinates
                            .iter()
                            .find(|(id, _)| *id == v.port.id)
                            .and_then(|(_, col)| {
                                report
                                    .and_then(|r| r.candidate.as_ref())
                                    .and_then(|c| c.get(col.get()).copied())
                            })
                    };
                    states
                        .push(fit_variables::Row {
                            run_id: self.run_id,
                            experiment_id: experiment,
                            symbol_id: v.port.id,
                            quantity_id: v.port.quantity.as_id(),
                            unit_id: v.port.unit.as_id(),
                            fixed: v.fixed,
                            value,
                            lower: v.lower,
                            upper: v.upper,
                        })
                        .map_err(relation)?;
                }
                for &(local, global) in &s.constraints {
                    let (local, global) = (local.get(), global.get());
                    let row = &s.case.assembly.structure().rows()[local];
                    constraints
                        .push(fit_constraints::Row {
                            run_id: self.run_id,
                            experiment_id: experiment,
                            row_id: row.id,
                            quantity_id: row.quantity.as_id(),
                            unit_id: p
                                .quantities
                                .quantity_type(row.quantity)
                                .map_err(math)?
                                .canonical_unit
                                .as_id(),
                            value: report.and_then(|r| r.constraint_values.get(global).copied()),
                            lower: p.bounds[global].0.is_finite().then_some(p.bounds[global].0),
                            upper: p.bounds[global].1.is_finite().then_some(p.bounds[global].1),
                            tolerance: p.tolerances.rows[global],
                        })
                        .map_err(relation)?;
                }
            }
        }
        for (i, param) in p.declaration.parameters.iter().enumerate() {
            let value = if param.fixed {
                Some(param.value)
            } else {
                report
                    .and_then(|r| r.candidate.as_ref())
                    .and_then(|v| p.parameter_columns[i].and_then(|c| v.get(c.get()).copied()))
            };
            parameters
                .push(fit_parameters::Row {
                    run_id: self.run_id,
                    parameter_id: param.symbol_id,
                    fixed: param.fixed,
                    value,
                    unit_id: p.parameter_ports[i].unit.as_id(),
                    scale: param.scale,
                    at_bound: value.map(|v| {
                        param.lower.is_some_and(|b| {
                            (v - b).abs()
                                <= p.parameter_columns[i]
                                    .map_or(0.0, |c| p.tolerances.variables[c.get()])
                        }) || param.upper.is_some_and(|b| {
                            (v - b).abs()
                                <= p.parameter_columns[i]
                                    .map_or(0.0, |c| p.tolerances.variables[c.get()])
                        })
                    }),
                })
                .map_err(relation)?;
        }
        for (i, o) in p.measurements.iter().enumerate() {
            let prediction = report.and_then(|r| r.predictions.get(i).copied().flatten());
            let residual = prediction.zip(o.value).map(|(p, v)| p - v);
            let standardized = residual.zip(o.sigma).map(|(r, s)| r / s);
            observations
                .push(fit_observations::Row {
                    run_id: self.run_id,
                    observation_id: o.id,
                    experiment_id: p.declaration.experiments[o.experiment].experiment_id,
                    included: o.included,
                    prediction,
                    residual,
                    standardized_residual: standardized,
                    objective_contribution: if o.included {
                        standardized.map(|r| 0.5 * o.importance * r * r)
                    } else {
                        None
                    },
                    unit_id: o.port.unit.as_id(),
                })
                .map_err(relation)?;
            if let Some(jac) = report
                .filter(|_| o.included)
                .and_then(|r| r.responses.as_ref())
            {
                for (j, (pi, _)) in p
                    .parameter_columns
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_some())
                    .enumerate()
                {
                    responses
                        .push(response_sensitivities::Row {
                            run_id: self.run_id,
                            experiment_id: p.declaration.experiments[o.experiment].experiment_id,
                            sample: i as i64,
                            time: o.time,
                            output_id: o.port.id,
                            parameter_id: p.declaration.parameters[pi].symbol_id,
                            output_unit_id: o.port.unit.as_id(),
                            parameter_unit_id: p.parameter_ports[pi].unit.as_id(),
                            value: jac[(i, j)],
                        })
                        .map_err(relation)?;
                }
            }
        }
        use native::solve::Metric;
        let mut metric = |ns: &str, name: &str, v: Metric| {
            crate::workflow::results::push_metric(&mut metrics, self.run_id, 0, ns, name, &v)
        };
        metric(
            "profile",
            "solver_identity",
            Metric::Text(
                crate::math::solves::profile_key(&p.profile.solver)
                    .map_err(crate::math::MathRuntimeError::from)?
                    .to_prefixed(),
            ),
        )?;
        metric(
            "profile",
            "rank_tolerance",
            Metric::Real(p.profile.rank_tolerance),
        )?;
        metric(
            "profile",
            "max_cells",
            Metric::Integer(p.profile.max_cells as i64),
        )?;
        for (id, profile) in &p.profile.simulations {
            metric(
                "profile.simulation",
                &id.as_id().to_hex(),
                Metric::Text(native::dynamics::profile_json(profile).to_string()),
            )?;
        }
        for (i, experiment) in p.experiments.iter().enumerate() {
            if let Experiment::Transient(s) = experiment {
                metric(
                    "effective.simulation",
                    &p.declaration.experiments[i].experiment_id.as_id().to_hex(),
                    Metric::Text(native::dynamics::profile_json(&s.profile).to_string()),
                )?;
            }
        }
        if let Some(r) = report {
            if let Some(objective) = r.objective {
                metric("physical", "fit_objective", Metric::Real(objective))?;
            }
            if let Some(q) = &r.quality {
                metric("physical", "fit_feasible", Metric::Bool(q.feasible()))?;
            }
            if let Some(rank) = r.rank {
                metric("local_response", "rank", Metric::Integer(rank as i64))?;
            }
            // The derivative sources of the native gradient and Hessian (PS-07).
            metric(
                "derivatives",
                "hessian",
                Metric::Text(r.hessian.as_str().into()),
            )?;
            metric(
                "derivatives",
                "gradient",
                Metric::Text(r.derivatives.as_str().into()),
            )?;
            // An exact Hessian's transient curvature comes from IDAS forward-over-adjoint
            // second-order sensitivities (ADR-0110 item 4).
            if r.hessian == HessianMode::Exact
                && p.experiments
                    .iter()
                    .any(|e| matches!(e, Experiment::Transient(_)))
            {
                metric(
                    "derivatives",
                    "transient_hessian",
                    Metric::Text("idas_forward_over_adjoint".into()),
                )?;
            }
            metric(
                "estimate",
                "qualified",
                Metric::Bool(r.estimate_qualified()),
            )?;
            metric(
                "local_response",
                "available",
                Metric::Bool(r.responses.is_some()),
            )?;
            if let Some(condition) = r.response_condition() {
                metric("local_response", "condition", Metric::Real(condition))?;
            }
            if let Some(d) = &r.diagnostic {
                metric(
                    "local_response",
                    "unavailable",
                    Metric::Text(d.rule.as_str().into()),
                )?;
            }
            if let Some(s) = &r.solve {
                metric("native", "status", Metric::Text(s.termination.name.clone()))?;
                metric("native", "code", Metric::Integer(s.termination.code))?;
                if let Some(q) = &s.quality {
                    metric("physical", "feasible", Metric::Bool(q.feasible()))?;
                }
                if let Some(e) = s.validation_failure() {
                    metric("physical", "validation_error", Metric::Text(e.to_string()))?;
                }
            }
        }
        if let Some(s) = report.and_then(|r| r.solve.as_ref()) {
            let stored = self.stored_events(0)?;
            let events = stored.as_deref().map_or(
                crate::workflow::results::StepEvents::Retained,
                crate::workflow::results::StepEvents::Stored,
            );
            crate::workflow::results::push_native_metrics(&mut metrics, self.run_id, 0, s, events)?;
        }
        let mut certificates =
            infeasibility_certificates::Builder::with_registry(registry, 0).map_err(relation)?;
        if let Some(row) = report
            .and_then(|r| r.solve.as_ref())
            .and_then(|s| crate::workflow::results::certificate_row(self.run_id, 0, s))
        {
            certificates.push(row).map_err(relation)?;
        }
        let mut batches = uncertainty(registry, self.run_id, p, report.map(|r| &**r))?;
        batches.extend([
            (
                fit_variables::RELATION_ID,
                states.finish().map_err(relation)?,
            ),
            (
                fit_constraints::RELATION_ID,
                constraints.finish().map_err(relation)?,
            ),
            (
                computation_runs::RELATION_ID,
                header.finish().map_err(relation)?,
            ),
            (
                fit_parameters::RELATION_ID,
                parameters.finish().map_err(relation)?,
            ),
            (
                fit_observations::RELATION_ID,
                observations.finish().map_err(relation)?,
            ),
            (
                response_sensitivities::RELATION_ID,
                responses.finish().map_err(relation)?,
            ),
            (
                solve_metrics::RELATION_ID,
                metrics.finish().map_err(relation)?,
            ),
            (
                infeasibility_certificates::RELATION_ID,
                certificates.finish().map_err(relation)?,
            ),
        ]);
        batches.extend(source.fit_declarations.tables(registry)?);
        use pse_relations::generated::runtime::{modeling_checks, modeling_reports};
        let mut checks = modeling_checks::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut reports =
            modeling_reports::Builder::with_registry(registry, 0).map_err(relation)?;
        if let Some(report) = report {
            for row in &report.checks {
                checks.push(row.clone()).map_err(relation)?;
            }
            for row in &report.reports {
                reports.push(row.clone()).map_err(relation)?;
            }
        }
        batches.insert(
            modeling_checks::RELATION_ID,
            checks.finish().map_err(relation)?,
        );
        batches.insert(
            modeling_reports::RELATION_ID,
            reports.finish().map_err(relation)?,
        );
        self.retain_sources(&mut batches)?;
        batches.extend(source.source_tables()?);
        Ok(batches)
    }
}

/// The fit's derived quantities (ADR-0118 items 8 and 10; PS-12): a `local_validity` row at
/// step 0 per quantity the fit derives, certified or withheld with its reason, and the data
/// rows of the certified ones; the response directions whenever the responses exist.
fn uncertainty(
    registry: &pse_schema::Registry,
    run_id: RunId,
    p: &FitProblem,
    report: Option<&FitReport>,
) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
    let mut validity = local_validity::Builder::with_registry(registry, 0).map_err(relation)?;
    let mut covariances =
        parameter_covariances::Builder::with_registry(registry, 0).map_err(relation)?;
    let mut intervals =
        parameter_intervals::Builder::with_registry(registry, 0).map_err(relation)?;
    let mut points = profile_points::Builder::with_registry(registry, 0).map_err(relation)?;
    let mut directions =
        response_directions::Builder::with_registry(registry, 0).map_err(relation)?;
    let mut propagated =
        propagated_covariances::Builder::with_registry(registry, 0).map_err(relation)?;
    let predictions = p
        .profile
        .uncertainty
        .as_ref()
        .is_some_and(|u| u.predictions);
    let free: Vec<(usize, SemanticId)> = p
        .free()
        .map(|(k, _)| (k, p.declaration.parameters[k].symbol_id))
        .collect();
    let unit = |id: SemanticId| {
        free.iter()
            .find(|(_, f)| *f == id)
            .map(|(k, _)| p.parameter_ports[*k].unit.as_id())
            .ok_or_else(|| contract(format!("no free fit parameter {id}")))
    };
    let mut row = |quantity, outcome: Result<(), &FitWithheld>, point| {
        validity
            .push(local_validity::Row {
                run_id,
                step: 0,
                quantity,
                validity: crate::workflow::local_analysis::record(
                    outcome
                        .map(|()| &())
                        .map_err(|w| (w.reason(), w.to_string())),
                    point,
                    false,
                ),
            })
            .map_err(relation)
    };
    // A run without a report withholds every quantity it would have derived.
    let absent = FitWithheld::Local(pse_backend_native::kkt::Withheld::NoCandidate);
    let Some(report) = report else {
        if !free.is_empty() {
            row(DerivedQuantity::ParameterCovariance, Err(&absent), None)?;
            if let Some(u) = &p.profile.uncertainty {
                row(DerivedQuantity::WaldInterval, Err(&absent), None)?;
                if u.profile.is_some() {
                    row(DerivedQuantity::ProfileInterval, Err(&absent), None)?;
                }
            }
            if predictions {
                let upstream = FitWithheld::Upstream(DerivedQuantity::ParameterCovariance);
                row(DerivedQuantity::PropagatedCovariance, Err(&upstream), None)?;
            }
        }
        return finish(
            validity,
            covariances,
            intervals,
            points,
            directions,
            propagated,
        );
    };
    if let Some(covariance) = &report.covariance {
        // The exact covariance is read from the fit's KKT point, whose verdicts it states.
        let point = (covariance.approximation
            == pse_relations::generated::enums::CovarianceApproximation::Exact)
            .then_some(report.solve.as_ref())
            .flatten()
            .and_then(|s| s.evidence.local.as_ref())
            .and_then(|l| l.as_ref().ok());
        row(
            DerivedQuantity::ParameterCovariance,
            covariance.values.as_ref().map(|_| ()),
            point,
        )?;
        if let Ok(values) = &covariance.values {
            covariances
                .push(parameter_covariances::Row {
                    run_id,
                    approximation: covariance.approximation,
                    parameters: covariance.parameters.clone(),
                    parameter_units: covariance
                        .parameters
                        .iter()
                        .map(|id| unit(*id))
                        .collect::<Result<_, _>>()?,
                    values: values.clone(),
                })
                .map_err(relation)?;
        }
    }
    let level = p
        .profile
        .uncertainty
        .as_ref()
        .map_or(0.0, |u| u.level.into_inner());
    let mut interval = |parameter: SemanticId,
                        method,
                        end,
                        estimate,
                        bound: &IntervalBound,
                        count: usize,
                        detail: Option<String>| {
        intervals
            .push(parameter_intervals::Row {
                run_id,
                parameter_id: parameter,
                method,
                end,
                unit_id: unit(parameter)?,
                level,
                estimate,
                value: bound.value,
                outcome: bound.outcome,
                points: i64::try_from(count).map_err(|_| contract("profile chain length"))?,
                detail,
            })
            .map_err(relation)
    };
    if let Some(wald) = &report.wald {
        row(
            DerivedQuantity::WaldInterval,
            wald.as_ref().map(|_| ()),
            None,
        )?;
        for i in wald.iter().flatten() {
            for (end, bound) in [
                (IntervalEnd::Lower, &i.lower),
                (IntervalEnd::Upper, &i.upper),
            ] {
                interval(
                    i.parameter,
                    IntervalMethod::Wald,
                    end,
                    i.estimate,
                    bound,
                    0,
                    None,
                )?;
            }
        }
    }
    if let Some(profiles) = &report.profiles {
        row(
            DerivedQuantity::ProfileInterval,
            profiles.as_ref().map(|_| ()),
            None,
        )?;
        for chain in profiles.iter().flatten() {
            interval(
                chain.parameter,
                IntervalMethod::ProfileLikelihood,
                chain.end,
                chain.estimate,
                &chain.bound,
                chain.points.len(),
                chain.detail.clone(),
            )?;
            for (k, point) in chain.points.iter().enumerate() {
                let ordinal =
                    |k: usize| i64::try_from(k).map_err(|_| contract("profile point ordinal"));
                points
                    .push(profile_points::Row {
                        run_id,
                        parameter_id: chain.parameter,
                        end: chain.end,
                        point: ordinal(k)?,
                        value: point.value,
                        seed: point.seed.map(ordinal).transpose()?,
                        qualification: point.qualification,
                        objective: point.objective,
                        statistic: point.statistic,
                        accepted: point.accepted,
                        detail: point.detail.clone(),
                    })
                    .map_err(relation)?;
            }
        }
    }
    // The covariance propagated to the included predictions through the responses (S4).
    if let Some(covariance) = report.covariance.as_ref().filter(|_| predictions) {
        let included: Vec<(usize, &Measurement)> = p
            .measurements
            .iter()
            .enumerate()
            .filter(|(_, o)| o.included)
            .collect();
        let jacobian = report
            .responses
            .as_ref()
            .map(|r| crate::workflow::uncertainty::Jacobian {
                outputs: included.iter().map(|(_, o)| o.id).collect(),
                parameters: covariance.parameters.clone(),
                values: included
                    .iter()
                    .flat_map(|(i, _)| (0..r.ncols()).map(move |j| r[(*i, j)]))
                    .collect(),
            });
        let jacobian = jacobian.ok_or_else(|| crate::workflow::uncertainty::Upstream {
            quantity: DerivedQuantity::ParameterCovariance,
            reason: pse_relations::generated::enums::WithheldReason::ResponsesUnavailable,
            detail: "the fit's responses are unavailable".into(),
        });
        let input = covariance.propagation_input(run_id);
        let result = crate::workflow::uncertainty::propagate(
            input.as_ref().map_err(Clone::clone),
            jacobian.as_ref().map_err(Clone::clone),
        )?;
        validity
            .push(local_validity::Row {
                run_id,
                step: 0,
                quantity: DerivedQuantity::PropagatedCovariance,
                validity: crate::workflow::local_analysis::record(
                    result.as_ref().map_err(|u| {
                        (
                            pse_relations::generated::enums::WithheldReason::UpstreamWithheld,
                            u.to_string(),
                        )
                    }),
                    None,
                    false,
                ),
            })
            .map_err(relation)?;
        if let Ok(result) = result {
            propagated
                .push(propagated_covariances::Row {
                    run_id,
                    step: 0,
                    covariance_run_id: result.covariance_run,
                    parameters: result.parameters,
                    output_units: included.iter().map(|(_, o)| o.port.unit.as_id()).collect(),
                    outputs: result.outputs,
                    values: result.values,
                })
                .map_err(relation)?;
        }
    }
    if let (Some(basis), Some(rank)) = (&report.directions, report.rank) {
        for k in 0..basis.ncols() {
            for (j, (_, parameter)) in free.iter().enumerate() {
                directions
                    .push(response_directions::Row {
                        run_id,
                        direction: i64::try_from(k).map_err(|_| contract("direction ordinal"))?,
                        parameter_id: *parameter,
                        singular_value: report.singular_values.get(k).copied().unwrap_or(0.0),
                        identifiable: k < rank,
                        component: basis[(j, k)],
                    })
                    .map_err(relation)?;
            }
        }
    }
    finish(
        validity,
        covariances,
        intervals,
        points,
        directions,
        propagated,
    )
}
fn finish(
    validity: local_validity::Builder,
    covariances: parameter_covariances::Builder,
    intervals: parameter_intervals::Builder,
    points: profile_points::Builder,
    directions: response_directions::Builder,
    propagated: propagated_covariances::Builder,
) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
    Ok(BTreeMap::from([
        (
            propagated_covariances::RELATION_ID,
            propagated.finish().map_err(relation)?,
        ),
        (
            local_validity::RELATION_ID,
            validity.finish().map_err(relation)?,
        ),
        (
            parameter_covariances::RELATION_ID,
            covariances.finish().map_err(relation)?,
        ),
        (
            parameter_intervals::RELATION_ID,
            intervals.finish().map_err(relation)?,
        ),
        (
            profile_points::RELATION_ID,
            points.finish().map_err(relation)?,
        ),
        (
            response_directions::RELATION_ID,
            directions.finish().map_err(relation)?,
        ),
    ]))
}

impl RunResult {
    /// Explicitly export canonical typed parameter values from this run's qualified fit.
    /// Retains producing identities; publication into an authored bank is a separate action.
    pub fn export_fit_parameters(&self) -> Result<FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::fitted_parameter_cells as cells;
        let (RunRequest::Fit(prepared), Ok(RunReport::Fit(report))) = (&self.request, &self.report)
        else {
            return Err(contract(
                "parameter publication requires a completed fitting run",
            ));
        };
        if !report.estimate_qualified() {
            return Err(contract(
                "parameter publication requires a freshly qualified, locally identifiable fit",
            ));
        }
        let problem = &prepared.problem;
        let count = problem.declaration.parameters.len();
        let bytes = count
            .checked_mul(512)
            .and_then(|n| n.checked_add(4096))
            .ok_or_else(|| contract("fitted parameter export extent"))?;
        let _scratch = self
            .runtime
            .shared
            .math()
            .reserve("fit:parameter-export", bytes)?;
        let pool = self.runtime.shared.pool();
        let cancel = pse_columnar::CancellationToken::default();
        let mut builder =
            pse_relations::columnar::Collection::new(&self.runtime.registry, &pool, &cancel);
        builder.ensure::<cells::Row>().map_err(relation)?;
        for (i, parameter) in problem.declaration.parameters.iter().enumerate() {
            let value = if parameter.fixed {
                parameter.value
            } else {
                report
                    .candidate
                    .as_ref()
                    .and_then(|values| {
                        problem.parameter_columns[i]
                            .and_then(|column| values.get(column.get()).copied())
                    })
                    .ok_or_else(|| contract("qualified fitting candidate absent"))?
            };
            builder
                .push(cells::Row {
                    run_id: self.run_id,
                    fit_id: problem.declaration.fit_id,
                    source_revision: prepared.source.revision.identity(),
                    fit_source: problem.source_identity,
                    parameter_id: parameter.symbol_id,
                    quantity_type_id: problem.parameter_ports[i].quantity.as_id(),
                    unit_id: problem.parameter_ports[i].unit.as_id(),
                    value,
                })
                .map_err(relation)?;
        }
        builder
            .finish()
            .map_err(relation)?
            .into_values()
            .next()
            .ok_or_else(|| contract("fitted parameter export relation absent"))
    }
}
