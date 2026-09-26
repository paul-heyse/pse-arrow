// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Joined fitting observations use existing generated contracts.
use super::*;
use crate::workflow::{RunReport, RunRequest, RunResult, relation};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{
        authored::computation_models,
        runtime::{
            computation_runs, fit_constraints, fit_observations, fit_parameters, fit_variables,
            response_sensitivities, solve_metrics,
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
                for v in s.case.assembly.structure().variables() {
                    let value = if v.fixed {
                        s.values.scalars.get(&v.port.id).copied()
                    } else {
                        s.coordinates
                            .iter()
                            .find(|(id, _)| *id == v.port.id)
                            .and_then(|(_, col)| {
                                report
                                    .and_then(|r| r.candidate.as_ref())
                                    .and_then(|c| c.get(*col).copied())
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
                    let row = &s.case.assembly.structure().rows()[local];
                    constraints
                        .push(fit_constraints::Row {
                            run_id: self.run_id,
                            experiment_id: experiment,
                            row_id: row.id,
                            quantity_id: row.quantity.as_id(),
                            unit_id: p
                                .revision
                                .0
                                .physical
                                .quantities
                                .quantity_type(row.quantity)
                                .map_err(math)?
                                .canonical_unit
                                .as_id(),
                            value: report.and_then(|r| r.constraint_values.get(global).copied()),
                            lower: row.lower.is_finite().then_some(row.lower),
                            upper: row.upper.is_finite().then_some(row.upper),
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
                    .and_then(|v| p.parameter_columns[i].and_then(|c| v.get(c).copied()))
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
                                <= p.parameter_columns[i].map_or(0.0, |c| p.tolerances.variables[c])
                        }) || param.upper.is_some_and(|b| {
                            (v - b).abs()
                                <= p.parameter_columns[i].map_or(0.0, |c| p.tolerances.variables[c])
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
                &id.to_hex(),
                Metric::Text(native::dynamics::profile_json(profile).to_string()),
            )?;
        }
        for (i, experiment) in p.experiments.iter().enumerate() {
            if let Experiment::Transient(s) = experiment {
                metric(
                    "effective.simulation",
                    &p.declaration.experiments[i].experiment_id.to_hex(),
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
                metric("local_response", "unavailable", Metric::Text(d.clone()))?;
            }
            for (i, s) in r.singular_values.iter().enumerate() {
                metric(
                    "local_response.singular_value",
                    &i.to_string(),
                    Metric::Real(*s),
                )?;
            }
            if let Some(s) = &r.solve {
                metric("native", "status", Metric::Text(s.termination.name.clone()))?;
                metric("native", "code", Metric::Integer(s.termination.code))?;
                if let Some(q) = &s.quality {
                    metric("physical", "feasible", Metric::Bool(q.feasible()))?;
                }
                if let Some(e) = &s.validation_error {
                    metric("physical", "validation_error", Metric::Text(e.clone()))?;
                }
            }
        }
        if let Some(s) = report.and_then(|r| r.solve.as_ref()) {
            crate::workflow::results::push_native_metrics(&mut metrics, self.run_id, 0, s)?;
        }
        let mut models =
            computation_models::Builder::with_registry(registry, 1).map_err(relation)?;
        models.push(p.revision.0.row.clone()).map_err(relation)?;
        let mut batches = BTreeMap::from([
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
                computation_models::RELATION_ID,
                models.finish().map_err(relation)?,
            ),
        ]);
        self.retain_sources(&mut batches)?;
        Ok(batches)
    }
}
