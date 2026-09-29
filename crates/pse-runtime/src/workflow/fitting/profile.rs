// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Profile-likelihood intervals by adaptive pin chains (Plan 22 S3; ADR-0118 items 8 and 9;
//! PS-11, PS-12).
//!
//! **Chains.** Each free parameter has two chains, one per end. A point pins the parameter
//! with [`native::transform::Pinned`] on the fit prepared once and re-solves the fit over
//! the other coordinates through the one NLP runner; its seed is the solution of the
//! chain's latest accepted point below the threshold, or the fit estimate, and is recorded
//! as the point's input (PS-11). Chains run in parallel, as many at once as the fit's job
//! admitted cores for, each pinned fit on the fit's own threads.
//!
//! **Statistic.** With `f = ½·χ²` the objective, the signed root `r = √(2(f − f*))` of the
//! likelihood-ratio statistic is compared with `r* = √χ²₁(level)`. On a linear model `r`
//! is linear in the pinned value, `|θ − θ̂|/σ`, so the end is `θ̂ ± r*·σ`, the Wald end.
//!
//! **Step control** (homotopy style). The predictor extrapolates `r` along the secant of
//! the last two accepted points to reach `r*`, at most doubling the step; a failed pinned
//! fit halves it toward the last accepted point. Once a point passes the threshold, the
//! end is bracketed and the next pin is the secant root in `r` between the bracket's
//! points, exact on a linear model. A chain ends at a point within the tolerance of `r*`
//! (`threshold`), at the parameter's declared bound below it (`bound`), or stops: a failure
//! at the smallest step, the point budget, the fit's deadline or cancellation, or a pinned
//! fit whose objective falls below the estimate's (`stopped`).
use super::*;
use super::{covariance::IntervalBound, oracle::FitOracle};
use native::{
    ProblemError,
    kkt::Withheld,
    solve::{Backend, Compatibility, Execution, Qualification},
};
use pse_relations::generated::enums::{IntervalEnd, IntervalOutcome};
use statrs::distribution::{ChiSquared, ContinuousCDF};
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

/// One pinned fit of a profile chain.
#[derive(Clone, Debug, PartialEq)]
pub struct ProfilePoint {
    /// The pinned parameter value, in the parameter's unit.
    pub value: f64,
    /// The chain point whose solution seeded this fit; `None` for the fit estimate (PS-11).
    pub seed: Option<usize>,
    /// The pinned fit's qualification, when the runner returned a report.
    pub qualification: Option<Qualification>,
    /// The pinned fit's objective, half the weighted residual sum of squares.
    pub objective: Option<f64>,
    /// `√(2·max(f − f*, 0))`.
    pub statistic: Option<f64>,
    /// Qualified stationary or better and feasible, and used by the chain.
    pub accepted: bool,
    /// Why the pinned fit failed or was not used.
    pub detail: Option<String>,
}
/// The chain of one free parameter toward one end.
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileChain {
    /// The parameter.
    pub parameter: SemanticId,
    /// The end the chain moves toward.
    pub end: IntervalEnd,
    /// The fit estimate of the parameter.
    pub estimate: f64,
    /// The end reached.
    pub bound: IntervalBound,
    /// Why the chain stopped.
    pub detail: Option<String>,
    /// Every pinned fit, in solve order.
    pub points: Vec<ProfilePoint>,
}
impl ProfileChain {
    /// Retained bytes.
    pub(super) fn bytes(&self) -> usize {
        size_of::<Self>()
            + self.detail.as_ref().map_or(0, String::len)
            + self
                .points
                .iter()
                .map(|p| size_of::<ProfilePoint>() + p.detail.as_ref().map_or(0, String::len))
                .sum::<usize>()
    }
}

/// One chain's task: the parameter's declaration index and column, its end and the first
/// step.
#[derive(Clone, Copy, Debug)]
struct Task {
    parameter: usize,
    column: OriginalCol,
    end: IntervalEnd,
    step: f64,
}
/// An accepted pinned fit.
struct Solution {
    objective: f64,
    primal: Vec<f64>,
    qualification: Qualification,
}
/// What every chain of one fit shares.
struct Chains<'a> {
    problem: &'a Arc<FitProblem>,
    backend: Backend,
    execution: &'a Execution,
    estimate: &'a [f64],
    /// The estimate's objective `f*`.
    optimum: f64,
    /// `r* = √χ²₁(level)`.
    threshold: f64,
    tolerance: f64,
    points: usize,
}
impl Chains<'_> {
    /// One pinned fit at `value`, seeded from `seed`.
    fn solve(
        &self,
        column: OriginalCol,
        value: f64,
        seed: &[f64],
    ) -> Result<Solution, (Option<Qualification>, String)> {
        if let Some(stop) = self.execution.stopped() {
            return Err((None, format!("the fit stopped: {}", stop.as_str())));
        }
        let p = self.problem;
        let mut controls = p.profile.solver.controls.clone();
        controls.time_limit = self
            .execution
            .time_limit
            .saturating_sub(self.execution.started.elapsed());
        let failed = |e: ProblemError| (None, e.to_string());
        let oracle = FitOracle::new(p.clone(), self.execution.clone()).map_err(failed)?;
        let pinned = native::transform::Pinned::new(Box::new(oracle), &[(column.get(), value)])
            .map_err(failed)?;
        let mut initial = seed.to_vec();
        initial[column.get()] = value;
        let report = native::execution::nlp(
            native::execution::Step {
                adapter: native::execution::adapter(self.backend),
                settings: &p.profile.solver.backend,
                controls: &controls,
                accuracy: &p.accuracy,
                execution: self.execution.clone(),
                tolerances: &p.tolerances,
                normalization: &p.normalization,
                compatibility: Compatibility {
                    layout: p.key,
                    profile: p.profile_key,
                    data: p.source_identity,
                    backend: self.backend,
                },
                warm: None,
            },
            &mut native::execution::Retained::default(),
            native::execution::Nlp {
                oracle: Box::new(pinned),
                initial: &initial,
                presolve: &p.profile.solver.presolve,
                intent: SolveIntent::Optimize,
                sense: pse_math::binding::ObjectiveSense::Minimize,
                limit: p.profile.max_cells,
                analysis: native::execution::Analysis::NONE,
            },
        )
        .map_err(failed)?;
        let qualification = report.qualification;
        if !matches!(
            qualification,
            Qualification::Stationary
                | Qualification::OptimalWithinTolerance
                | Qualification::GapQualified
        ) || !report
            .quality
            .as_ref()
            .is_some_and(native::quality::Quality::feasible)
        {
            return Err((
                Some(qualification),
                format!(
                    "the pinned fit is qualified {} ({})",
                    qualification.as_str(),
                    report.termination.name
                ),
            ));
        }
        let objective = report
            .observation
            .as_ref()
            .and_then(|o| o.objective)
            .filter(|f| f.is_finite());
        match (report.candidate, objective) {
            (Some(candidate), Some(objective)) => Ok(Solution {
                objective,
                primal: candidate.primal,
                qualification,
            }),
            _ => Err((
                Some(qualification),
                "the pinned fit observed no objective".into(),
            )),
        }
    }
    /// Run one chain to its end.
    fn run(&self, task: Task) -> ProfileChain {
        let p = self.problem;
        let column = task.column.get();
        let direction = if task.end == IntervalEnd::Lower { -1.0 } else { 1.0 };
        let estimate = self.estimate[column];
        let variable = &p.contract.variables[column];
        let limit = if task.end == IntervalEnd::Lower {
            variable.lower
        } else {
            variable.upper
        };
        let tolerance = p.tolerances.variables[column];
        let scale = p.declaration.parameters[task.parameter].scale;
        let smallest = 1e-9 * scale.max(estimate.abs());
        let mut chain = ProfileChain {
            parameter: p.declaration.parameters[task.parameter].symbol_id,
            end: task.end,
            estimate,
            bound: IntervalBound {
                value: None,
                outcome: IntervalOutcome::Stopped,
            },
            detail: None,
            points: Vec::new(),
        };
        let end = |mut chain: ProfileChain, value: Option<f64>, outcome, detail| {
            chain.bound = IntervalBound { value, outcome };
            chain.detail = detail;
            chain
        };
        if limit.is_finite() && (limit - estimate).abs() <= tolerance {
            return end(chain, Some(limit), IntervalOutcome::Bound, None);
        }
        // The latest accepted point below the threshold: value, statistic, solution and
        // chain index (`None` for the estimate).
        let mut base = (estimate, 0.0, self.estimate.to_vec(), None::<usize>);
        let mut over: Option<(f64, f64)> = None;
        let mut ceiling: Option<f64> = None;
        let mut step = task.step.max(smallest);
        // An objective this far below the estimate's is a lower minimum, not noise.
        let lower = self.tolerance * self.threshold * self.threshold / 2.0;
        while chain.points.len() < self.points {
            let (at, statistic) = (base.0, base.1);
            let value = match over {
                // The secant root in r between the bracket's points.
                Some((beyond, above)) => {
                    let fraction = ((self.threshold - statistic) / (above - statistic))
                        .clamp(0.01, 0.99);
                    at + fraction * (beyond - at)
                }
                None => {
                    let mut value = at + direction * step;
                    if let Some(ceiling) = ceiling
                        && direction * (value - ceiling) >= 0.0
                    {
                        value = 0.5 * (at + ceiling);
                    }
                    if limit.is_finite() && direction * (value - limit) > 0.0 {
                        value = limit;
                    }
                    value
                }
            };
            let index = chain.points.len();
            let seed = base.3;
            match self.solve(task.column, value, &base.2) {
                Err((qualification, detail)) => {
                    chain.points.push(ProfilePoint {
                        value,
                        seed,
                        qualification,
                        objective: None,
                        statistic: None,
                        accepted: false,
                        detail: Some(detail.clone()),
                    });
                    // Halve the step toward the last accepted point.
                    let gap = (value - at).abs() / 2.0;
                    if gap < smallest || self.execution.stopped().is_some() {
                        return end(chain, None, IntervalOutcome::Stopped, Some(detail));
                    }
                    over = None;
                    ceiling = Some(value);
                    step = gap;
                }
                Ok(solution) => {
                    let increase = solution.objective - self.optimum;
                    let r = (2.0 * increase.max(0.0)).sqrt();
                    let below = increase < -lower;
                    chain.points.push(ProfilePoint {
                        value,
                        seed,
                        qualification: Some(solution.qualification),
                        objective: Some(solution.objective),
                        statistic: Some(r),
                        accepted: !below,
                        detail: below
                            .then(|| "the pinned fit's objective is below the estimate's".into()),
                    });
                    if below {
                        let detail = format!(
                            "a pinned fit at {value} reached an objective {} below the estimate's",
                            -increase
                        );
                        return end(chain, None, IntervalOutcome::Stopped, Some(detail));
                    }
                    if (r - self.threshold).abs() <= self.tolerance * self.threshold {
                        return end(chain, Some(value), IntervalOutcome::Threshold, None);
                    }
                    if r > self.threshold {
                        over = Some((value, r));
                        continue;
                    }
                    let advanced = (value - at).abs();
                    let slope = (r - statistic) / advanced;
                    base = (value, r, solution.primal, Some(index));
                    if limit.is_finite() && value == limit {
                        return end(chain, Some(limit), IntervalOutcome::Bound, None);
                    }
                    if over.is_none() {
                        let predicted = if slope > 0.0 {
                            ((self.threshold - r) / slope).min(2.0 * advanced)
                        } else {
                            2.0 * advanced
                        };
                        step = predicted.max(smallest);
                    }
                }
            }
        }
        let detail = format!("the chain's budget of {} pinned fits ran out", self.points);
        end(chain, None, IntervalOutcome::Stopped, Some(detail))
    }
}

impl FitProblem {
    /// The two profile-likelihood chains of every free parameter, or why they are withheld:
    /// they need the estimate's own conditions, not full rank, which a flat profile shows.
    pub(super) fn profiles(
        self: &Arc<Self>,
        report: &FitReport,
        level: f64,
        controls: &ProfileControls,
        (route, workers): (native::routing::Route, usize),
        execution: &Execution,
    ) -> Result<Vec<ProfileChain>, FitWithheld> {
        self.estimate_valid(report)?;
        let (Some(estimate), Some(optimum)) = (report.candidate.as_deref(), report.objective)
        else {
            return Err(FitWithheld::Local(Withheld::NoCandidate));
        };
        let native::routing::Route::Native(backend) = route else {
            return Err(FitWithheld::Local(Withheld::NoCandidate));
        };
        let threshold = ChiSquared::new(1.0)
            .map(|c| c.inverse_cdf(level).sqrt())
            .ok()
            .filter(|r| r.is_finite() && *r > 0.0)
            .ok_or(FitWithheld::Local(Withheld::Backsolve))?;
        // A certified covariance sizes the first step at half the Wald half-width; without
        // one, a tenth of the parameter's declared scale.
        let errors = report
            .covariance
            .as_ref()
            .and_then(Covariance::standard_errors);
        let tasks: Vec<Task> = self
            .free()
            .enumerate()
            .flat_map(|(j, (parameter, column))| {
                let scale = self.declaration.parameters[parameter].scale;
                let step = errors
                    .as_ref()
                    .map(|e| 0.5 * threshold * e[j])
                    .filter(|s| s.is_finite() && *s > 0.0)
                    .unwrap_or(0.1 * scale);
                [IntervalEnd::Lower, IntervalEnd::Upper].map(|end| Task {
                    parameter,
                    column,
                    end,
                    step,
                })
            })
            .collect();
        let threads = self.profile.solver.controls.threads;
        let workers = workers.min(tasks.len()).max(1);
        let chains = Chains {
            problem: self,
            backend,
            execution,
            estimate,
            optimum,
            threshold,
            tolerance: controls.tolerance.into_inner(),
            points: controls.points.into_inner(),
        };
        if workers == 1 {
            return Ok(tasks.into_iter().map(|t| chains.run(t)).collect());
        }
        let next = AtomicUsize::new(0);
        let results: Mutex<Vec<Option<ProfileChain>>> = Mutex::new(vec![None; tasks.len()]);
        let stack = self.runtime.native().stack_bytes();
        std::thread::scope(|scope| {
            let work = || {
                native::execution::scoped(
                    &[native::execution::adapter(backend)],
                    threads,
                    stack,
                    || -> Result<(), ProblemError> {
                        loop {
                            let index = next.fetch_add(1, Ordering::Relaxed);
                            let Some(task) = tasks.get(index) else {
                                return Ok(());
                            };
                            let chain = chains.run(*task);
                            if let Ok(mut results) = results.lock() {
                                results[index] = Some(chain);
                            }
                        }
                    },
                )
            };
            let handles: Vec<_> = (0..workers)
                .filter_map(|_| {
                    std::thread::Builder::new()
                        .name("pse-profile".into())
                        .stack_size(stack)
                        .spawn_scoped(scope, work)
                        .ok()
                })
                .collect();
            for handle in handles {
                let _ = handle.join();
            }
        });
        // A chain no worker completed, because a worker could not start or failed, runs
        // here, inside the fit's own execution scope.
        let results = results
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(tasks
            .iter()
            .zip(results)
            .map(|(task, chain)| chain.unwrap_or_else(|| chains.run(*task)))
            .collect())
    }
}
