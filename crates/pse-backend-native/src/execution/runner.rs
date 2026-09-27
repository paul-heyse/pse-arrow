// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared representation runners. Each builds its adapter's native representation from
//! original-coordinate inputs, executes the selected adapter, recovers original
//! coordinates and qualifies the report. Workflows choose a runner by representation and
//! the adapter by table lookup; neither step names a backend.
use super::{BackendExecution, BackendSettings, Budgets, Input, Problem, Retained};
use crate::{
    CoefficientProblem, ConicProblem, NleOracle, NlpOracle, ProblemError,
    assembled::FeasibilityOracle,
    presolve::{self, Pipeline},
    quality::{self, Observation, Quality, Tolerances},
    solve::{Compatibility, Controls, ResolvedAccuracy, SolveIntent, SolveReport, WarmStart},
    transport,
};
use pse_math::{
    binding::ObjectiveSense, convexity::QuadraticEvidence, normalization::Normalization,
};
use std::any::Any;

/// Inputs common to every runner: the selected adapter, typed settings and controls, and
/// the step's original-coordinate acceptance and reuse contracts.
pub struct Step<'a> {
    /// Selected adapter.
    pub adapter: &'a dyn BackendExecution,
    /// Typed settings admitted for `adapter`.
    pub settings: &'a BackendSettings,
    /// Finite shared controls chosen by the caller.
    pub controls: &'a Controls,
    /// Stopping budgets resolved from the numerical policy for this step.
    pub accuracy: &'a ResolvedAccuracy,
    /// Cancellation, deadline and bounded progress of this attempt.
    pub execution: crate::solve::Execution,
    /// Original physical acceptance budgets.
    pub tolerances: &'a Tolerances,
    /// Model coordinate transport.
    pub normalization: &'a Normalization,
    /// Semantic reuse identity of this step.
    pub compatibility: Compatibility,
    /// Submitted seed in original coordinates.
    pub warm: Option<&'a WarmStart>,
}

impl std::fmt::Debug for Step<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Step")
            .field("backend", &self.adapter.backend())
            .field("settings", &self.settings)
            .field("compatibility", &self.compatibility)
            .finish_non_exhaustive()
    }
}
/// One NLP run.
pub struct Nlp<'a> {
    /// Original-coordinate callbacks, carrying their resolved normalization.
    pub oracle: Box<dyn NlpOracle>,
    /// Original start.
    pub initial: &'a [f64],
    /// Qualified library preprocessing policy.
    pub presolve: &'a presolve::Policy,
    /// Mathematical purpose; feasibility purposes solve with a constant objective.
    pub intent: SolveIntent,
    /// Authored objective sense.
    pub sense: ObjectiveSense,
    /// Presolve dimension ceiling.
    pub limit: usize,
}
/// The one NLP runner (F28): presolve pipeline, native solve (or the pipeline's terminal
/// report), library recovery with independent original observation, KKT evidence and
/// qualification. Solve sequences, initialization and fitting all run through it.
///
/// # Errors
/// Pipeline admission or native execution failed before a report existed.
pub fn nlp(
    step: Step<'_>,
    retained: &mut Retained,
    run: Nlp<'_>,
) -> Result<SolveReport, ProblemError> {
    let oracle: Box<dyn NlpOracle> = if matches!(
        run.intent,
        SolveIntent::FeasiblePoint | SolveIntent::Root | SolveIntent::Initialize
    ) {
        Box::new(FeasibilityOracle(run.oracle))
    } else {
        run.oracle
    };
    let mut pipeline = Pipeline::new(
        oracle,
        run.initial,
        run.presolve,
        step.tolerances,
        step.execution.clone(),
        step.warm,
        step.compatibility,
        run.limit,
    )?;
    let report = match pipeline.terminal_report(run.sense)? {
        Some(report) => report,
        None => {
            let transport = pipeline.take_oracle()?;
            let tolerances = pipeline.tolerances(step.tolerances);
            step.adapter.execute(
                retained,
                Input {
                    problem: Problem::Nlp {
                        oracle: Box::new(transport),
                        initial: pipeline.initial(),
                        sense: run.sense,
                    },
                    controls: step.controls,
                    accuracy: step.accuracy,
                    settings: step.settings,
                    execution: step.execution,
                    tolerances: &tolerances,
                    warm: pipeline.warm(),
                    compatibility: pipeline.native_compatibility().clone(),
                },
            )?
        }
    };
    let mut report = pipeline.finish(report, step.tolerances, run.sense);
    quality::record_kkt(&mut report, step.normalization, step.accuracy);
    quality::qualify(&mut report, step.accuracy);
    Ok(report)
}

/// One square root-system run.
pub struct Roots<'a> {
    /// Original-coordinate residual equations.
    pub oracle: Box<dyn NleOracle>,
    /// Original start.
    pub initial: &'a [f64],
    /// Owner of the evaluators behind `oracle`, kept alive as long as native state retains it.
    pub owner: Option<Box<dyn Any>>,
}
impl std::fmt::Debug for Nlp<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Nlp")
            .field("intent", &self.intent)
            .field("sense", &self.sense)
            .finish_non_exhaustive()
    }
}
impl std::fmt::Debug for Roots<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Roots")
            .field("oracle", &self.oracle)
            .finish_non_exhaustive()
    }
}
/// Normalized residual transport, native solve, original recovery and qualification.
///
/// # Errors
/// Transport or native execution failed before a report existed.
pub fn roots(
    step: Step<'_>,
    retained: &mut Retained,
    run: Roots<'_>,
) -> Result<SolveReport, ProblemError> {
    let contract = run.oracle.contract().clone();
    let oracle = transport::Roots::new(run.oracle, step.normalization.clone())?;
    let initial = step.normalization.normalized_point(run.initial)?;
    let warm = step
        .warm
        .map(|w| transport::warm(w, step.normalization, true))
        .transpose()?;
    let tolerances = step.tolerances.normalized(step.normalization)?;
    let mut report = step.adapter.execute(
        retained,
        Input {
            problem: Problem::Roots {
                oracle: Box::new(oracle),
                initial: &initial,
                budgets: Budgets {
                    tolerances: step.tolerances,
                    normalization: step.normalization,
                    feasibility: step.accuracy.feasibility,
                },
                owner: run.owner,
            },
            controls: step.controls,
            accuracy: step.accuracy,
            settings: step.settings,
            execution: step.execution,
            tolerances: &tolerances,
            warm: warm.as_ref(),
            compatibility: step.compatibility,
        },
    )?;
    transport::recover(&mut report, step.normalization, &contract)?;
    quality::record_kkt(&mut report, step.normalization, step.accuracy);
    quality::qualify(&mut report, step.accuracy);
    Ok(report)
}

/// Fresh evaluation of the original compiled model, independent of any solver projection.
pub trait OriginalModel {
    /// Constraint values, authored-sense objective and source terms at `primal`.
    ///
    /// # Errors
    /// The original model could not be evaluated.
    fn evaluate(&mut self, primal: &[f64]) -> Result<Evaluation, ProblemError>;
}
/// One fresh original-model evaluation.
#[derive(Debug)]
pub struct Evaluation {
    /// Constraint values in original row order.
    pub constraints: Vec<f64>,
    /// Authored-sense objective, when declared.
    pub objective: Option<f64>,
    /// Original source terms of the constraint rows.
    pub sources: Vec<pse_math::assembly::OutputValue>,
}
/// One coefficient-model run.
pub struct Coefficients<'a> {
    /// Original-coordinate coefficient projection.
    pub problem: &'a CoefficientProblem,
    /// Convexity evidence for a quadratic objective.
    pub certificate: Option<&'a dyn QuadraticEvidence>,
    /// Original row constants removed by the projection.
    pub row_constants: &'a [f64],
    /// Authored row bounds, before the projection shifted them.
    pub row_bounds: Vec<(f64, f64)>,
    /// The original compiled model the projection must agree with.
    pub original: &'a mut dyn OriginalModel,
}
impl std::fmt::Debug for Coefficients<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Coefficients")
            .field("problem", &self.problem)
            .finish_non_exhaustive()
    }
}
/// Normalized coefficient transport, native solve and recovery, then an independent
/// re-check of the candidate against the original compiled model before qualification.
///
/// # Errors
/// Transport or native execution failed before a report existed.
pub fn coefficients(
    step: Step<'_>,
    retained: &mut Retained,
    run: Coefficients<'_>,
) -> Result<SolveReport, ProblemError> {
    let (problem, transported) =
        transport::coefficients(run.problem, step.normalization, run.certificate)?;
    let certificate = transported
        .as_ref()
        .map(|p| -> &dyn QuadraticEvidence { p });
    let tolerances = step.tolerances.normalized(step.normalization)?;
    let warm = step
        .warm
        .map(|w| transport::warm(w, step.normalization, true))
        .transpose()?;
    let mut report = step.adapter.execute(
        retained,
        Input {
            problem: Problem::Coefficients {
                problem: &problem,
                certificate,
                normalization: step.normalization,
                row_constants: run.row_constants,
            },
            controls: step.controls,
            accuracy: step.accuracy,
            settings: step.settings,
            execution: step.execution.clone(),
            tolerances: &tolerances,
            warm: warm.as_ref(),
            compatibility: step.compatibility.clone(),
        },
    )?;
    transport::recover(&mut report, step.normalization, &run.problem.contract)?;
    if let Some(candidate) = &report.candidate {
        match run
            .problem
            .observation(&candidate.primal, run.row_constants, run.row_bounds)
        {
            Ok(o) => report.observation = Some(o),
            Err(e) => report.record_validation_failure(e),
        }
    }
    reobserve(&mut report, run.problem, run.original, &step)?;
    quality::qualify(&mut report, step.accuracy);
    Ok(report)
}
/// The coefficient projection must reproduce the original model's rows and objective at
/// the candidate; the fresh original values then replace the projected observation.
fn reobserve(
    report: &mut SolveReport,
    problem: &CoefficientProblem,
    original: &mut dyn OriginalModel,
    step: &Step<'_>,
) -> Result<(), ProblemError> {
    let (Some(candidate), Some(observation)) = (&report.candidate, &mut report.observation) else {
        return Ok(());
    };
    let validation = (|| -> Result<_, ProblemError> {
        let fresh = original.evaluate(&candidate.primal)?;
        if fresh
            .constraints
            .iter()
            .zip(&observation.values)
            .zip(&step.tolerances.rows)
            .any(|((a, b), t)| (a - b).abs() > *t)
            || fresh
                .objective
                .zip(candidate.objective)
                .is_some_and(|(a, b)| {
                    (a - b).abs() > step.normalization.objective * step.accuracy.gap_absolute
                })
        {
            return Err(ProblemError::numerical(
                "native coefficient projection disagrees with the original model",
            ));
        }
        let quality = quality::observed(
            &problem.contract,
            &observation.bounds,
            &candidate.primal,
            &fresh.constraints,
            step.tolerances,
        )?;
        Ok((quality, fresh))
    })();
    match validation {
        Ok((q, fresh)) => {
            let mut observed = Observation::from_values(
                fresh.objective,
                fresh.constraints,
                observation.bounds.clone(),
            )?;
            observed.sources = fresh.sources;
            *observation = observed;
            let integrality = report
                .quality
                .as_ref()
                .map_or_else(Vec::new, |q| q.integrality.clone());
            // Semi-continuous/semi-integer zero branches belong to the coefficient
            // domain, not the enclosing bound interval.
            let bounds = report
                .quality
                .as_ref()
                .map_or(q.bounds, |prior| prior.bounds.clone());
            report.quality = Some(Quality::new(q.rows, bounds, integrality)?);
        }
        Err(e) => report.record_validation_failure(e),
    }
    Ok(())
}

/// Native solve over a cone model normalized at preparation, recovery and qualification.
///
/// # Errors
/// Native execution failed before a report existed.
pub fn cone(
    step: Step<'_>,
    retained: &mut Retained,
    problem: &ConicProblem,
    certificate: &dyn QuadraticEvidence,
) -> Result<SolveReport, ProblemError> {
    let tolerances = step.tolerances.normalized(step.normalization)?;
    let mut report = step.adapter.execute(
        retained,
        Input {
            problem: Problem::Cone {
                problem,
                certificate,
            },
            controls: step.controls,
            accuracy: step.accuracy,
            settings: step.settings,
            execution: step.execution,
            tolerances: &tolerances,
            warm: None,
            compatibility: step.compatibility,
        },
    )?;
    transport::recover(&mut report, step.normalization, &problem.contract)?;
    quality::qualify(&mut report, step.accuracy);
    Ok(report)
}
