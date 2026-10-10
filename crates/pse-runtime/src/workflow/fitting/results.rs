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
    pub(in crate::workflow) async fn encode_fit(
        &self,
        request: &crate::workflow::result_export::Projection,
    ) -> Result<(), WorkflowError> {
        let RunRequest::Fit(p) = &self.request else {
            return Err(contract("fit request mismatch"));
        };
        let p = &p.problem;
        let report = match &self.report {
            Ok(RunReport::Fit(r)) => Some(r),
            Err(_) => None,
            _ => return Err(contract("fit report mismatch")),
        };
        let registry = &self.runtime.registry;
        let validation = self.runtime.validation_context()?;
        let pool = self.runtime.shared.pool();
        let cancel = request.cancel.clone();
        let mut header = crate::workflow::result_export::Rows::<computation_runs::Row>::new(
            request,
            registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        if request.wants(computation_runs::RELATION_ID) {
            header
                .push(
                    self.completion()
                        .map_err(|e| contract(e.to_string()))?
                        .computation
                        .clone()
                        .ok_or_else(|| contract("missing completed computation"))?,
                )
                .await
                .map_err(relation)?;
        }
        let mut parameters = crate::workflow::result_export::Rows::<fit_parameters::Row>::new(
            request,
            registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        let mut observations = crate::workflow::result_export::Rows::<fit_observations::Row>::new(
            request,
            registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        let mut responses =
            crate::workflow::result_export::Rows::<response_sensitivities::Row>::new(
                request,
                registry,
                &pool,
                &cancel,
                &validation,
            )
            .map_err(relation)?;
        let mut metrics = crate::workflow::result_export::Rows::<solve_metrics::Row>::new(
            request,
            registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        let mut states = crate::workflow::result_export::Rows::<fit_variables::Row>::new(
            request,
            registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        let mut constraints = crate::workflow::result_export::Rows::<fit_constraints::Row>::new(
            request,
            registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        if request.wants(fit_variables::RELATION_ID) || request.wants(fit_constraints::RELATION_ID)
        {
            for (ei, e) in p.experiments.iter().enumerate() {
                if let Experiment::Steady(s) = e {
                    let experiment = p.declaration.experiments[ei].experiment_id;
                    if request.wants(fit_variables::RELATION_ID) {
                        for v in &s.variables {
                            if states.skip_next() {
                                continue;
                            }
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
                                .await
                                .map_err(relation)?;
                        }
                    }

                    if request.wants(fit_constraints::RELATION_ID) {
                        for &(local, global) in &s.constraints {
                            if constraints.skip_next() {
                                continue;
                            }
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
                                    value: report
                                        .and_then(|r| r.constraint_values.get(global).copied()),
                                    lower: p.bounds[global]
                                        .0
                                        .is_finite()
                                        .then_some(p.bounds[global].0),
                                    upper: p.bounds[global]
                                        .1
                                        .is_finite()
                                        .then_some(p.bounds[global].1),
                                    tolerance: p.tolerances.rows[global],
                                })
                                .await
                                .map_err(relation)?;
                        }
                    }
                }
            }
        }

        if request.wants(fit_parameters::RELATION_ID) {
            for (i, param) in p.declaration.parameters.iter().enumerate() {
                if parameters.skip_next() {
                    continue;
                }
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
                    .await
                    .map_err(relation)?;
            }
        }

        if request.wants(fit_observations::RELATION_ID)
            || request.wants(response_sensitivities::RELATION_ID)
        {
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
                    .await
                    .map_err(relation)?;
                if request.wants(response_sensitivities::RELATION_ID)
                    && let Some(jac) = report
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
                        if responses.skip_next() {
                            continue;
                        }
                        responses
                            .push(response_sensitivities::Row {
                                run_id: self.run_id,
                                experiment_id: p.declaration.experiments[o.experiment]
                                    .experiment_id,
                                sample: i as i64,
                                time: o.time,
                                output_id: o.port.id,
                                parameter_id: p.declaration.parameters[pi].symbol_id,
                                output_unit_id: o.port.unit.as_id(),
                                parameter_unit_id: p.parameter_ports[pi].unit.as_id(),
                                value: jac[(i, j)],
                            })
                            .await
                            .map_err(relation)?;
                    }
                }
            }
        }

        if request.wants(solve_metrics::RELATION_ID) {
            use native::solve::Metric;
            let mut metric = async |ns: &str, name: &str, v: Metric| {
                crate::workflow::results::push_metric(&mut metrics, self.run_id, 0, ns, name, &v)
                    .await
            };
            metric(
                "profile",
                "solver_identity",
                Metric::Text(
                    crate::math::solves::profile_key(&p.profile.solver)
                        .map_err(crate::math::MathRuntimeError::from)?
                        .to_prefixed(),
                ),
            )
            .await?;
            metric(
                "profile",
                "rank_tolerance",
                Metric::Real(p.profile.rank_tolerance),
            )
            .await?;
            metric(
                "profile",
                "max_cells",
                Metric::Integer(p.profile.max_cells as i64),
            )
            .await?;
            for (id, profile) in &p.profile.simulations {
                metric(
                    "profile.simulation",
                    &id.as_id().to_hex(),
                    Metric::Text(native::dynamics::profile_json(profile).to_string()),
                )
                .await?;
            }
            for (i, experiment) in p.experiments.iter().enumerate() {
                if let Experiment::Transient(s) = experiment {
                    metric(
                        "effective.simulation",
                        &p.declaration.experiments[i].experiment_id.as_id().to_hex(),
                        Metric::Text(native::dynamics::profile_json(&s.profile).to_string()),
                    )
                    .await?;
                }
            }
            if let Some(r) = report {
                if let Some(objective) = r.objective {
                    metric("physical", "fit_objective", Metric::Real(objective)).await?;
                }
                if let Some(q) = &r.quality {
                    metric("physical", "fit_feasible", Metric::Bool(q.feasible())).await?;
                }
                if let Some(rank) = r.rank {
                    metric("local_response", "rank", Metric::Integer(rank as i64)).await?;
                }
                // The derivative sources of the native gradient and Hessian (PS-07).
                metric(
                    "derivatives",
                    "hessian",
                    Metric::Text(r.hessian.as_str().into()),
                )
                .await?;
                metric(
                    "derivatives",
                    "gradient",
                    Metric::Text(r.derivatives.as_str().into()),
                )
                .await?;
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
                    )
                    .await?;
                }
                metric(
                    "estimate",
                    "qualified",
                    Metric::Bool(r.estimate_qualified()),
                )
                .await?;
                metric(
                    "local_response",
                    "available",
                    Metric::Bool(r.responses.is_some()),
                )
                .await?;
                if let Some(condition) = r.response_condition() {
                    metric("local_response", "condition", Metric::Real(condition)).await?;
                }
                if let Some(d) = &r.diagnostic {
                    metric(
                        "local_response",
                        "unavailable",
                        Metric::Text(d.rule.as_str().into()),
                    )
                    .await?;
                }
                if let Some(s) = &r.solve {
                    metric("native", "status", Metric::Text(s.termination.name.clone())).await?;
                    metric("native", "code", Metric::Integer(s.termination.code)).await?;
                    if let Some(q) = &s.quality {
                        metric("physical", "feasible", Metric::Bool(q.feasible())).await?;
                    }
                    if let Some(e) = s.validation_failure() {
                        metric("physical", "validation_error", Metric::Text(e.to_string())).await?;
                    }
                }
            }
            if let Some(s) = report.and_then(|r| r.solve.as_ref()) {
                crate::workflow::results::push_native_metrics(&mut metrics, self.run_id, 0, s)
                    .await?;
            }
        }
        let mut certificates = crate::workflow::result_export::Rows::<
            infeasibility_certificates::Row,
        >::new(request, registry, &pool, &cancel, &validation)
        .map_err(relation)?;
        if let Some(native) = report.and_then(|r| r.solve.as_ref()) {
            crate::workflow::results::push_certificate(
                &mut certificates,
                self.run_id,
                0,
                native.backend,
                native.certificate.as_ref(),
            )
            .await?;
        }
        if [
            local_validity::RELATION_ID,
            parameter_covariances::RELATION_ID,
            parameter_intervals::RELATION_ID,
            profile_points::RELATION_ID,
            response_directions::RELATION_ID,
            propagated_covariances::RELATION_ID,
        ]
        .contains(&request.relation)
        {
            uncertainty(
                request,
                registry,
                &pool,
                &cancel,
                &validation,
                self.run_id,
                p,
                report.map(|r| &**r),
            )
            .await?;
        }
        header.finish().await.map_err(relation)?;
        parameters.finish().await.map_err(relation)?;
        observations.finish().await.map_err(relation)?;
        responses.finish().await.map_err(relation)?;
        metrics.finish().await.map_err(relation)?;
        states.finish().await.map_err(relation)?;
        constraints.finish().await.map_err(relation)?;
        certificates.finish().await.map_err(relation)?;
        use pse_relations::generated::runtime::{modeling_checks, modeling_reports};
        let mut checks = crate::workflow::result_export::Rows::<modeling_checks::Row>::new(
            request,
            registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        let mut reports = crate::workflow::result_export::Rows::<modeling_reports::Row>::new(
            request,
            registry,
            &pool,
            &cancel,
            &validation,
        )
        .map_err(relation)?;
        if let Some(report) = report {
            if request.wants(modeling_checks::RELATION_ID) {
                for row in &report.checks {
                    if checks.skip_next() {
                        continue;
                    }
                    checks.push_ref(row).await.map_err(relation)?;
                }
            }

            if request.wants(modeling_reports::RELATION_ID) {
                for row in &report.reports {
                    if reports.skip_next() {
                        continue;
                    }
                    reports.push_ref(row).await.map_err(relation)?;
                }
            }
        }
        checks.finish().await.map_err(relation)?;
        reports.finish().await.map_err(relation)?;
        let trace = report
            .and_then(|r| r.strategy.as_ref())
            .or_else(|| match &self.report {
                Err(error) => error.strategy_trace(),
                _ => None,
            });
        use pse_relations::generated::runtime::solve_strategy_events;
        let mut strategy_events =
            crate::workflow::result_export::Rows::<solve_strategy_events::Row>::new(
                request,
                registry,
                &pool,
                &cancel,
                &validation,
            )
            .map_err(relation)?;
        if let Some(trace) = trace
            && request.wants(solve_strategy_events::RELATION_ID)
        {
            strategy_events
                .strategy_events(trace, self.run_id, 0)
                .await?;
        }
        strategy_events.finish().await.map_err(relation)?;
        use pse_relations::generated::runtime::solve_strategy_products;
        let mut products =
            crate::workflow::result_export::Rows::<solve_strategy_products::Row>::new(
                request,
                registry,
                &pool,
                &cancel,
                &validation,
            )
            .map_err(relation)?;
        if let Some(trace) = trace
            && request.wants(solve_strategy_products::RELATION_ID)
        {
            products.strategy_products(trace, self.run_id, 0).await?;
        }
        products.finish().await.map_err(relation)?;
        Ok(())
    }
}

/// The fit's derived quantities (ADR-0118 items 8 and 10; PS-12): a `local_validity` row at
/// step 0 per quantity the fit derives, certified or withheld with its reason, and the data
/// rows of the certified ones; the response directions whenever the responses exist.
#[allow(
    clippy::too_many_arguments,
    reason = "selected uncertainty projection borrows request delivery resources and immutable fit completion without transferring their ownership"
)]
async fn uncertainty(
    request: &crate::workflow::result_export::Projection,
    registry: &pse_schema::Registry,
    pool: &Arc<dyn pse_columnar::MemoryPool>,
    cancel: &pse_columnar::CancellationToken,
    validation: &pse_relations::validate::ValidationContext,
    run_id: RunId,
    p: &FitProblem,
    report: Option<&FitReport>,
) -> Result<(), WorkflowError> {
    let mut validity = crate::workflow::result_export::Rows::<local_validity::Row>::new(
        request, registry, pool, cancel, validation,
    )
    .map_err(relation)?;
    let mut covariances = crate::workflow::result_export::Rows::<parameter_covariances::Row>::new(
        request, registry, pool, cancel, validation,
    )
    .map_err(relation)?;
    let mut intervals = crate::workflow::result_export::Rows::<parameter_intervals::Row>::new(
        request, registry, pool, cancel, validation,
    )
    .map_err(relation)?;
    let mut points = crate::workflow::result_export::Rows::<profile_points::Row>::new(
        request, registry, pool, cancel, validation,
    )
    .map_err(relation)?;
    let mut directions = crate::workflow::result_export::Rows::<response_directions::Row>::new(
        request, registry, pool, cancel, validation,
    )
    .map_err(relation)?;
    let mut propagated = crate::workflow::result_export::Rows::<propagated_covariances::Row>::new(
        request, registry, pool, cancel, validation,
    )
    .map_err(relation)?;
    let predictions = p
        .profile
        .uncertainty
        .as_ref()
        .is_some_and(|u| u.predictions);
    let free = || {
        p.free()
            .map(|(k, _)| (k, p.declaration.parameters[k].symbol_id))
    };
    let unit = |id: SemanticId| {
        free()
            .find(|(_, f)| *f == id)
            .map(|(k, _)| p.parameter_ports[k].unit.as_id())
            .ok_or_else(|| contract(format!("no free fit parameter {id}")))
    };
    let mut row = async |quantity, outcome: Result<(), &FitWithheld>, point| {
        if !validity.wanted() || validity.skip_next() {
            return Ok(());
        }
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
            .await
            .map_err(relation)
    };
    // A run without a report withholds every quantity it would have derived.
    let absent = FitWithheld::Local(pse_backend_native::kkt::Withheld::NoCandidate);
    let Some(report) = report else {
        if free().next().is_some() {
            row(DerivedQuantity::ParameterCovariance, Err(&absent), None).await?;
            if let Some(u) = &p.profile.uncertainty {
                row(DerivedQuantity::WaldInterval, Err(&absent), None).await?;
                if u.profile.is_some() {
                    row(DerivedQuantity::ProfileInterval, Err(&absent), None).await?;
                }
            }
            if predictions {
                let upstream = FitWithheld::Upstream(DerivedQuantity::ParameterCovariance);
                row(DerivedQuantity::PropagatedCovariance, Err(&upstream), None).await?;
            }
        }
        return finish(
            validity,
            covariances,
            intervals,
            points,
            directions,
            propagated,
        )
        .await;
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
        )
        .await?;
        if request.wants(parameter_covariances::RELATION_ID)
            && let Ok(values) = &covariance.values
            && !covariances.skip_next()
        {
            let _copy = crate::workflow::result_export::working(
                pool,
                "result:fit-covariance-copy",
                &[(covariance.parameters.len(), 32), (values.len(), 8)],
            )?;
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
                .await
                .map_err(relation)?;
        }
    }
    let level = p
        .profile
        .uncertainty
        .as_ref()
        .map_or(0.0, |u| u.level.into_inner());
    let mut interval = async |parameter: SemanticId,
                              method,
                              end,
                              estimate,
                              bound: &IntervalBound,
                              count: usize,
                              detail: Option<String>| {
        if !intervals.wanted() || intervals.skip_next() {
            return Ok(());
        }
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
            .await
            .map_err(relation)
    };
    if let Some(wald) = &report.wald {
        row(
            DerivedQuantity::WaldInterval,
            wald.as_ref().map(|_| ()),
            None,
        )
        .await?;
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
                )
                .await?;
            }
        }
    }
    if let Some(profiles) = &report.profiles {
        row(
            DerivedQuantity::ProfileInterval,
            profiles.as_ref().map(|_| ()),
            None,
        )
        .await?;
        for chain in profiles.iter().flatten() {
            interval(
                chain.parameter,
                IntervalMethod::ProfileLikelihood,
                chain.end,
                chain.estimate,
                &chain.bound,
                chain.points.len(),
                chain.detail.clone(),
            )
            .await?;
            if request.wants(profile_points::RELATION_ID) {
                for (k, point) in chain.points.iter().enumerate() {
                    if points.skip_next() {
                        continue;
                    }
                    let detail = point.detail.as_ref().map_or(0, String::len);
                    let _copy = points.working(&[(detail, 1), (point.failures.len(), 8192)])?;
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
                            termination: point.termination,
                            callback_terminal_failure: Some(point.callback_terminal_failure),
                            failures: point
                                .failures
                                .iter()
                                .map(|failure| {
                                    crate::workflow::diagnostic_rows::profile_failure(
                                        &failure.diagnostic(),
                                    )
                                })
                                .collect(),
                            objective: point.objective,
                            statistic: point.statistic,
                            accepted: point.accepted,
                            detail: point.detail.clone(),
                        })
                        .await
                        .map_err(relation)?;
                }
            }
        }
    }
    // The covariance propagated to the included predictions through the responses (S4).
    if request.wants(local_validity::RELATION_ID)
        && let Some(covariance) = report.covariance.as_ref().filter(|_| predictions)
    {
        let result = covariance.propagation_verdict().and_then(|()| {
            if report.responses.is_some() {
                Ok(())
            } else {
                Err(crate::workflow::uncertainty::Upstream {
                    quantity: DerivedQuantity::ParameterCovariance,
                    reason: pse_relations::generated::enums::WithheldReason::ResponsesUnavailable,
                    detail: "the fit's responses are unavailable".into(),
                })
            }
        });
        validity
            .push(local_validity::Row {
                run_id,
                step: 0,
                quantity: DerivedQuantity::PropagatedCovariance,
                validity: crate::workflow::local_analysis::record(
                    result.as_ref().map_err(|upstream| {
                        (
                            pse_relations::generated::enums::WithheldReason::UpstreamWithheld,
                            upstream.to_string(),
                        )
                    }),
                    None,
                    false,
                ),
            })
            .await
            .map_err(relation)?;
    }
    if request.wants(propagated_covariances::RELATION_ID)
        && !propagated.skip_next()
        && let Some(covariance) = report.covariance.as_ref().filter(|_| predictions)
    {
        let included_count = p.measurements.iter().filter(|o| o.included).count();
        let response_columns = report.responses.as_ref().map_or(0, |r| r.ncols());
        let jacobian_cells = included_count
            .checked_mul(response_columns)
            .ok_or_else(|| contract("fit response projection extent"))?;
        let covariance_cells = covariance.values.as_ref().map_or(0, |values| values.len());
        let _copy = crate::workflow::result_export::working(
            pool,
            "result:fit-propagation-copy",
            &[
                (included_count, 64),
                (jacobian_cells, 8),
                (
                    covariance_cells,
                    size_of::<pse_model::scalars::FiniteBound>(),
                ),
                (covariance.parameters.len(), 32),
            ],
        )?;

        let included: Vec<(usize, &Measurement)> = p
            .measurements
            .iter()
            .enumerate()
            .filter(|(_, o)| o.included)
            .collect();
        if report
            .responses
            .as_ref()
            .is_some_and(|r| r.nrows() != p.measurements.len())
        {
            return Err(contract("fit response row extent"));
        }
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
        let (result, _derived) = crate::workflow::uncertainty::propagate_reserved(
            input.as_ref().map_err(Clone::clone),
            jacobian.as_ref().map_err(Clone::clone),
            pool,
            cancel,
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
            .await
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
                .await
                .map_err(relation)?;
        }
    }

    if request.wants(response_directions::RELATION_ID)
        && let (Some(basis), Some(rank)) = (&report.directions, report.rank)
    {
        for k in 0..basis.ncols() {
            for (j, (_, parameter)) in free().enumerate() {
                if directions.skip_next() {
                    continue;
                }
                directions
                    .push(response_directions::Row {
                        run_id,
                        direction: i64::try_from(k).map_err(|_| contract("direction ordinal"))?,
                        parameter_id: parameter,
                        singular_value: report.singular_values.get(k).copied().unwrap_or(0.0),
                        identifiable: k < rank,
                        component: basis[(j, k)],
                    })
                    .await
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
    .await
}
async fn finish(
    validity: crate::workflow::result_export::Rows<'_, local_validity::Row>,
    covariances: crate::workflow::result_export::Rows<'_, parameter_covariances::Row>,
    intervals: crate::workflow::result_export::Rows<'_, parameter_intervals::Row>,
    points: crate::workflow::result_export::Rows<'_, profile_points::Row>,
    directions: crate::workflow::result_export::Rows<'_, response_directions::Row>,
    propagated: crate::workflow::result_export::Rows<'_, propagated_covariances::Row>,
) -> Result<(), WorkflowError> {
    validity.finish().await.map_err(relation)?;
    covariances.finish().await.map_err(relation)?;
    intervals.finish().await.map_err(relation)?;
    points.finish().await.map_err(relation)?;
    directions.finish().await.map_err(relation)?;
    propagated.finish().await.map_err(relation)?;
    Ok(())
}

impl RunResult {
    /// Explicit qualified parameter publication through the same bounded projection.
    pub fn export_fit_parameters(&self) -> Result<FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::fitted_parameter_cells as cells;
        crate::workflow::result_export::collect(
            self.export_fit_parameters_cursor(1024)?,
            &self.runtime,
            cells::RELATION_ID,
        )
        .map_err(WorkflowError::Shared)
    }
    /// Bounded parameter transport preserves the original qualification obligation.
    pub fn export_fit_parameters_cursor(
        &self,
        rows: usize,
    ) -> Result<crate::workflow::ResultCursor<'_>, WorkflowError> {
        use pse_relations::generated::runtime::fitted_parameter_cells as cells;
        self.require_fit_publication()?;
        if let Some(error) = self.encodings.failure(cells::RELATION_ID) {
            return Err(WorkflowError::Shared(error));
        }
        let spec = self
            .runtime
            .registry
            .relation_by_id(cells::RELATION_ID)
            .ok_or_else(|| contract("fitted parameter declaration absent"))?;
        let schema = pse_schema::arrow::relation_schema_ref(&self.runtime.registry, spec)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        crate::workflow::ResultCursor::new(
            schema,
            cells::RELATION_ID,
            rows,
            crate::workflow::ResultOrder::Public,
            |request| async move {
                self.project_fit_parameters(&request)
                    .await
                    .map_err(|error| self.encodings.record(cells::RELATION_ID, error))
            },
        )
    }
    /// Owning one-consumption parameter stream for foreign-language delivery.
    pub fn into_fit_parameters_cursor(
        self: Arc<Self>,
        rows: usize,
    ) -> Result<crate::workflow::ResultCursor<'static>, WorkflowError> {
        use pse_relations::generated::runtime::fitted_parameter_cells as cells;
        self.require_fit_publication()?;
        if let Some(error) = self.encodings.failure(cells::RELATION_ID) {
            return Err(WorkflowError::Shared(error));
        }
        let spec = self
            .runtime
            .registry
            .relation_by_id(cells::RELATION_ID)
            .ok_or_else(|| contract("fitted parameter declaration absent"))?;
        let schema = pse_schema::arrow::relation_schema_ref(&self.runtime.registry, spec)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        crate::workflow::ResultCursor::new(
            schema,
            cells::RELATION_ID,
            rows,
            crate::workflow::ResultOrder::Public,
            |request| async move {
                self.project_fit_parameters(&request)
                    .await
                    .map_err(|error| self.encodings.record(cells::RELATION_ID, error))
            },
        )
    }
    fn require_fit_publication(&self) -> Result<(), WorkflowError> {
        match &self.report {
            Ok(RunReport::Fit(report)) if report.estimate_qualified() => Ok(()),
            _ => Err(contract(
                "parameter publication requires a completed freshly qualified, locally identifiable fit",
            )),
        }
    }
    async fn project_fit_parameters(
        &self,
        request: &crate::workflow::result_export::Projection,
    ) -> Result<(), WorkflowError> {
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
        let pool = self.runtime.shared.pool();
        let cancel = request.cancel.clone();
        let validation = self.runtime.validation_context()?;
        let mut builder = crate::workflow::result_export::SelectedCollection::new(
            request,
            &self.runtime.registry,
            &pool,
            &cancel,
            &validation,
        );
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
                    source_revision: prepared.source.revision.identity().as_id(),
                    fit_source: problem.source_identity,
                    parameter_id: parameter.symbol_id,
                    quantity_type_id: problem.parameter_ports[i].quantity.as_id(),
                    unit_id: problem.parameter_ports[i].unit.as_id(),
                    value,
                })
                .await
                .map_err(relation)?;
        }
        builder.finish().await.map_err(relation)
    }
}
