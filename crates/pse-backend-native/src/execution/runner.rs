// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared representation runners. Each builds its adapter's native representation from
//! original-coordinate inputs, executes the selected adapter, recovers original
//! coordinates and qualifies the report. Workflows choose a runner by representation and
//! the adapter by table lookup; neither step names a backend.
use super::{BackendExecution, BackendSettings, Budgets, Input, Problem, Representation, Retained};
pub use crate::kkt::Analysis;
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
    /// Immutable observation retained by contextual admission.
    pub snapshot: &'a super::Snapshot,
    /// The admitted interpretation of the original retained structural witness.
    pub structure: Option<&'a crate::structural::Assessment>,
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

impl Step<'_> {
    /// Final observation/settings validation before protected preparation or effects.
    pub(crate) fn preflight(&self) -> Result<(), ProblemError> {
        self.adapter
            .validate_snapshot(self.settings, self.snapshot)?;
        self.adapter
            .admit_settings(self.settings, self.controls, self.snapshot)
    }
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
    /// The local analysis the caller requests at the candidate.
    pub analysis: Analysis,
}
/// The one NLP runner (F28): presolve pipeline, native solve (or the pipeline's terminal
/// report), library recovery with independent original observation, the requested local
/// analysis, KKT evidence, qualification and the requested sensitivities. Solve sequences,
/// initialization and fitting all run through it.
///
/// # Errors
/// A second-order or sensitivity analysis requested for a feasibility purpose, an
/// inconsistent sensitivity request, pipeline admission, or native execution failed before
/// a report existed.
pub fn nlp(
    step: Step<'_>,
    retained: &mut Retained,
    mut run: Nlp<'_>,
) -> Result<SolveReport, ProblemError> {
    step.preflight()?;
    if let Some(structure) = step.structure {
        crate::structural::check(
            run.oracle.contract(),
            run.oracle.jacobian_pattern(),
            run.oracle.constraint_bounds(),
            structure.mode,
            Some(&structure.witness),
        )?;
        run.oracle = crate::structural::retain_nlp(run.oracle, structure.witness.clone());
    }
    // Feasibility purposes solve a constant objective, whose curvature says nothing.
    if (run.analysis.second_order
        || run.analysis.sensitivity.is_some()
        || run.analysis.output_accuracy.is_some())
        && run.intent != SolveIntent::Optimize
    {
        return Err(ProblemError::Contract(format!(
            "a second-order or sensitivity analysis needs an optimizing intent, not {}",
            run.intent.as_str()
        )));
    }
    let sensitivity = run.analysis.sensitivity.take();
    if run.analysis.output_accuracy.is_some() && !run.analysis.second_order {
        return Err(ProblemError::Contract(
            "KKT output accuracy requires original exact-Hessian local analysis".into(),
        ));
    }
    if let Some(observer) = &run.analysis.output_accuracy {
        let source = observer.source();
        let goals = observer.goals();
        let distinct = goals
            .iter()
            .map(|goal| goal.declaration.goal_id)
            .collect::<std::collections::BTreeSet<_>>();
        if goals.is_empty()
            || distinct.len() != goals.len()
            || source.normalization != Some(step.normalization.key())
            || goals.iter().any(|goal| {
                goal.source != source
                    || pse_model::engineering_accuracy::validate_goal(&goal.declaration).is_err()
            })
        {
            return Err(ProblemError::Contract("KKT output accuracy needs distinct valid goals bound to its actual original context and normalization".into()));
        }
    }
    if let Some(request) = &sensitivity {
        request.admit(run.oracle.contract())?;
    }
    // An inverse reduced Hessian is read from the step's own analysis, over distinct
    // columns of the solve.
    if let Some(columns) = &run.analysis.inverse_reduced_hessian {
        let n = run.oracle.contract().variables.len();
        let distinct: std::collections::BTreeSet<_> = columns.iter().collect();
        if !run.analysis.second_order
            || columns.is_empty()
            || distinct.len() != columns.len()
            || columns.iter().any(|c| c.get() >= n)
        {
            return Err(ProblemError::Contract(
                "an inverse reduced Hessian names distinct columns of the solve and needs its second-order analysis".into(),
            ));
        }
    }
    // Presolve tightens bounds and removes rows on the premise that every row holds; a
    // method that relaxes the rows would then minimize violation over a domain those rows
    // already restricted. `Auto` lets the system choose, and it chooses no pass, recorded
    // with its reason; an explicitly requested pass is refused.
    let (policy, resolution) = match (step.settings.relaxes_rows(), run.presolve) {
        (false, requested) => (requested.clone(), None),
        (true, presolve::Policy::Off) => (presolve::Policy::Off, None),
        (true, presolve::Policy::Auto) => (
            presolve::Policy::Off,
            Some(presolve::Resolution::RelaxedRows),
        ),
        (true, presolve::Policy::Explicit { .. }) => {
            return Err(ProblemError::Contract(
                "the ℓ1 exact penalty relaxes every row; explicit presolve passes assume they hold"
                    .into(),
            ));
        }
    };
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
        &policy,
        step.tolerances,
        step.accuracy,
        step.execution.clone(),
        step.warm,
        step.compatibility,
        run.limit,
    )?;
    if let Some(resolution) = resolution {
        pipeline.resolved(run.presolve, resolution);
    }
    let report = match pipeline.terminal_report(run.sense)? {
        Some(report) => report,
        None => {
            let transport = pipeline.take_oracle()?;
            let oracle: Box<dyn NlpOracle> = match step.structure {
                Some(structure)
                    if crate::structural::same_inventory(structure, transport.contract()) =>
                {
                    crate::structural::retain_nlp(Box::new(transport), structure.witness.clone())
                }
                _ => Box::new(transport),
            };
            let tolerances = pipeline.tolerances(step.tolerances);
            let mut transported_execution = step.execution.clone();
            transported_execution.callback_work_owner = false;
            step.adapter.execute(
                retained,
                Input {
                    problem: Problem::Nlp {
                        oracle,
                        initial: pipeline.initial(),
                        sense: run.sense,
                    },
                    controls: step.controls,
                    accuracy: step.accuracy,
                    settings: step.settings,
                    snapshot: step.snapshot,
                    structure: step.structure,
                    execution: transported_execution,
                    tolerances: &tolerances,
                    warm: pipeline.warm(),
                    compatibility: pipeline.native_compatibility().clone(),
                },
            )?
        }
    };
    // The requested local analysis (L-N6) runs at the recovered candidate: multipliers
    // within the dual stationarity budget are weak, and the run's ceiling bounds the KKT.
    let budget = crate::kkt::Budget {
        dual: step.accuracy.stationarity,
        limit: run.limit,
    };
    let (mut report, factor) =
        pipeline.finish(report, step.tolerances, run.sense, &run.analysis, budget);
    quality::record_kkt(&mut report, step.normalization, step.accuracy);
    quality::qualify(&mut report, step.accuracy);
    report.least_infeasible = quality::least_infeasible(&report);
    if let Some(mut observer) = run.analysis.output_accuracy.take() {
        // The clone only shares existing factor storage. Output work is deferred
        // until the workflow completes its original scientific/model checks.
        report.evidence.output_accuracy = crate::engineering_accuracy::defer_kkt(
            observer.as_mut(),
            factor.clone(),
            &report,
            &step.execution,
        );
    }
    // The requested sensitivities read the qualified report; their factor lives for this
    // step only, unless the request keeps it for an advanced step (ADR-0118 item 12).
    if let Some(request) = sensitivity
        && let Some(advance) =
            crate::kkt::derive(&mut report, request, step.tolerances, run.sense, budget)
    {
        retained.keep(advance);
    }
    if let Some(columns) = run.analysis.inverse_reduced_hessian.take() {
        crate::kkt::invert(&mut report, factor, &columns, run.sense);
    }
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
            .field("analysis", &self.analysis)
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
    step.preflight()?;
    let contract = run.oracle.contract().clone();
    let original = if let Some(structure) = step.structure {
        crate::structural::check(
            run.oracle.contract(),
            run.oracle.jacobian_pattern(),
            &vec![(0.0, 0.0); contract.rows.len()],
            structure.mode,
            Some(&structure.witness),
        )?;
        crate::structural::retain_roots(run.oracle, structure.witness.clone())
    } else {
        run.oracle
    };
    let oracle = transport::Roots::new(original, step.normalization.clone())?;
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
                    accuracy: step.accuracy,
                    tolerances: step.tolerances,
                    normalization: step.normalization,
                },
                owner: run.owner,
            },
            controls: step.controls,
            accuracy: step.accuracy,
            settings: step.settings,
            snapshot: step.snapshot,
            structure: step.structure,
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
    /// Complete original lowering admitted during contextual preparation, when selected.
    pub lowered: Option<&'a crate::conic::Lowered>,
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
/// A cone adapter receives the problem lowered to cone form
/// ([`ConicProblem::from_coefficients`]) and its report is raised back to the coefficient
/// rows before recovery. A certificate is verified against the original coefficients.
///
/// # Errors
/// Transport, lowering or native execution failed before a report existed.
pub fn coefficients(
    step: Step<'_>,
    retained: &mut Retained,
    mut run: Coefficients<'_>,
) -> Result<SolveReport, ProblemError> {
    step.preflight()?;
    let transported = Transported::new(&step, &run)?;
    let report = step
        .adapter
        .execute(retained, transported.input(&step, run.row_constants))?;
    transported.finish(&step, &mut run, report)
}
/// The independent coefficient steps of one batch on one adapter (Plan 22 N5): each is
/// transported and lowered as [`coefficients`] does, the adapter solves them as one batch
/// ([`BackendExecution::execute_batch`]), and each report is recovered, re-checked against
/// its original model and qualified on its own. One result per step, in order.
///
/// # Errors
/// Per step, as [`coefficients`]; steps on different adapters are refused.
pub fn coefficients_batch(
    retained: &mut Retained,
    mut steps: Vec<(Step<'_>, Coefficients<'_>)>,
) -> Vec<Result<SolveReport, ProblemError>> {
    let Some(adapter) = steps.first().map(|(step, _)| step.adapter) else {
        return Vec::new();
    };
    if steps
        .iter()
        .any(|(step, _)| step.adapter.backend() != adapter.backend())
    {
        return steps
            .iter()
            .map(|_| Err(ProblemError::Contract("a batch runs on one adapter".into())))
            .collect();
    }
    let transported: Vec<Result<Transported, ProblemError>> = steps
        .iter()
        .map(|(step, run)| {
            step.preflight()?;
            Transported::new(step, run)
        })
        .collect();
    let inputs: Vec<Input<'_>> = transported
        .iter()
        .zip(&steps)
        .filter_map(|(t, (step, run))| t.as_ref().ok().map(|t| t.input(step, run.row_constants)))
        .collect();
    let mut reports = adapter.execute_batch(retained, inputs).into_iter();
    transported
        .into_iter()
        .zip(&mut steps)
        .map(|(t, (step, run))| {
            let t = t?;
            let report = reports
                .next()
                .unwrap_or_else(|| Err(ProblemError::Internal("batch report missing".into())))?;
            t.finish(step, run, report)
        })
        .collect()
}
/// One coefficient step's normalized transport: the problem, its evidence and budgets, the
/// warm start, and for a cone adapter the lowered cone form with its row budgets.
struct Transported {
    problem: CoefficientProblem,
    evidence: Option<pse_math::convexity::TransportedEvidence>,
    tolerances: Tolerances,
    warm: Option<WarmStart>,
    lowered: Option<(crate::conic::Lowered, Tolerances)>,
}
impl Transported {
    fn new(step: &Step<'_>, run: &Coefficients<'_>) -> Result<Self, ProblemError> {
        if let Some(structure) = step.structure {
            crate::structural::validate_assessment(
                structure,
                &run.problem.contract,
                &run.row_bounds,
            )?;
        }
        let (problem, evidence) =
            transport::coefficients(run.problem, step.normalization, run.certificate)?;
        let tolerances = step.tolerances.normalized(step.normalization)?;
        let warm = step
            .warm
            .map(|w| transport::warm(w, step.normalization, true))
            .transpose()?;
        let lowered = if step.adapter.representation() == Representation::Cone {
            let certificate = evidence.as_ref().map(|p| -> &dyn QuadraticEvidence { p });
            let lowered = match run.lowered {
                Some(lowered) => lowered.transport(step.normalization)?,
                None => ConicProblem::from_coefficients(&problem, certificate)?,
            };
            let budgets = lowered.tolerances(&tolerances);
            Some((lowered, budgets))
        } else {
            None
        };
        Ok(Self {
            problem,
            evidence,
            tolerances,
            warm,
            lowered,
        })
    }
    /// The adapter input: the lowered cone form for a cone adapter, the coefficients
    /// otherwise.
    fn input<'a>(&'a self, step: &'a Step<'_>, row_constants: &'a [f64]) -> Input<'a> {
        let problem = match &self.lowered {
            Some((lowered, _)) => Problem::Cone {
                problem: &lowered.problem,
                certificate: &lowered.evidence,
            },
            None => Problem::Coefficients {
                problem: &self.problem,
                certificate: self
                    .evidence
                    .as_ref()
                    .map(|p| -> &dyn QuadraticEvidence { p }),
                normalization: step.normalization,
                row_constants,
            },
        };
        Input {
            problem,
            controls: step.controls,
            accuracy: step.accuracy,
            settings: step.settings,
            snapshot: step.snapshot,
            structure: step.structure,
            execution: step.execution.clone(),
            tolerances: match &self.lowered {
                Some((_, budgets)) => budgets,
                None => &self.tolerances,
            },
            warm: self.warm.as_ref(),
            compatibility: step.compatibility.clone(),
        }
    }
    /// Raise a cone adapter's report to the coefficient rows, recover original coordinates,
    /// verify a certificate against the original coefficients, re-check the candidate
    /// against the original model and qualify.
    fn finish(
        &self,
        step: &Step<'_>,
        run: &mut Coefficients<'_>,
        mut report: SolveReport,
    ) -> Result<SolveReport, ProblemError> {
        if let Some((lowered, _)) = &self.lowered {
            lowered.raise(&mut report, &self.problem, &self.tolerances)?;
        }
        transport::recover(&mut report, step.normalization, &run.problem.contract)?;
        if let Some(c) = &mut report.certificate {
            crate::certificate::verify_coefficients(
                c,
                run.problem,
                step.tolerances,
                step.accuracy.feasibility,
            );
        }
        if let Some(candidate) = &report.candidate {
            match run.problem.observation(
                &candidate.primal,
                run.row_constants,
                run.row_bounds.clone(),
            ) {
                Ok(o) => report.observation = Some(o),
                Err(e) => report.record_validation_failure(e),
            }
        }
        reobserve(&mut report, run.problem, run.original, step)?;
        if let Some(evidence) = report.evidence.coefficient
            && evidence.upload_equivalent
            && evidence.discrete
            && let Some(value) = evidence.mip_dual_bound.filter(|v| v.is_finite())
        {
            let value = value * step.normalization.objective;
            let absolute_tolerance = step.accuracy.mip_absolute_gap * step.normalization.objective;
            if value.is_finite() && absolute_tolerance.is_finite() {
                report.evidence.original_bound = Some(crate::solve::OriginalObjectiveBound {
                    origin: pse_model::generated::enums::CandidateBoundOrigin::MixedInteger,
                    sense: run.problem.sense,
                    value,
                    absolute_tolerance,
                    relative_tolerance: step.accuracy.mip_relative_gap,
                });
            }
        }
        quality::qualify(&mut report, step.accuracy);
        Ok(report)
    }
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
    let validation = quality::contained(|| -> Result<_, ProblemError> {
        let fresh = step
            .execution
            .counted(crate::callback::evaluation_unit(), || {
                original.evaluate(&candidate.primal)
            })?;
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
        let mut observed = Observation::from_values(
            fresh.objective,
            fresh.constraints,
            observation.bounds.clone(),
        )?;
        observed.sources = fresh.sources;
        // Semi-continuous/semi-integer zero branches belong to the coefficient
        // domain, not the enclosing bound interval. Recompute that domain's checks
        // rather than carrying observations an earlier validation may have withdrawn.
        let domain = problem.quality(&candidate.primal, step.tolerances)?;
        Ok((
            Quality::new(quality.rows, domain.bounds, domain.integrality)?,
            observed,
        ))
    });
    match validation {
        Ok((q, observed)) => {
            report.observation = Some(observed);
            report.quality = Some(q);
            report.clear_validation_failure();
        }
        Err(e) => report.record_validation_failure(e),
    }
    Ok(())
}

/// One run of a recognized convex program (ADR-0121 Outcome 6).
pub struct Recognized<'a> {
    /// Retained exact cone-space proof from contextual preparation.
    pub certificate: Option<&'a dyn QuadraticEvidence>,
    /// The program's cone form, lowered from the preparation's recognition.
    pub lowered: &'a crate::conic::Recognized,
    /// The original compiled model the candidate is re-evaluated against.
    pub original: &'a mut dyn OriginalModel,
}
impl std::fmt::Debug for Recognized<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Recognized")
            .field("lowered", &self.lowered)
            .finish_non_exhaustive()
    }
}
/// A recognized convex program on a cone adapter: the cone form transported into the
/// case's coordinates (auxiliary columns and atom cones at unit scale), the native solve,
/// recovery, verification of a certificate against the cone form's original data, and the
/// report raised to the program's columns and rows with the original model re-evaluated at
/// the candidate, before qualification.
///
/// # Errors
/// Transport or native execution failed before a report existed.
pub fn recognized(
    step: Step<'_>,
    retained: &mut Retained,
    run: Recognized<'_>,
) -> Result<SolveReport, ProblemError> {
    step.preflight()?;
    let lowered = run.lowered;
    if let Some(structure) = step.structure {
        crate::structural::validate_assessment(structure, &lowered.original, &lowered.bounds)?;
    }
    let normalization = lowered.normalization(step.normalization)?;
    let composed;
    let certificate = match run.certificate {
        Some(certificate) => certificate,
        None => {
            composed = crate::conic::zero_certificate(
                lowered.problem.contract.variables.len(),
                &step.execution.cancel,
            )?;
            &composed
        }
    };
    let (problem, certificate) = transport::conic(&lowered.problem, &normalization, certificate)?;
    let original = lowered.tolerances(step.tolerances, step.accuracy.feasibility)?;
    let tolerances = original.normalized(&normalization)?;
    let mut report = step.adapter.execute(
        retained,
        Input {
            problem: Problem::Cone {
                problem: &problem,
                certificate: &certificate,
            },
            controls: step.controls,
            accuracy: step.accuracy,
            settings: step.settings,
            snapshot: step.snapshot,
            structure: step.structure,
            execution: step.execution,
            tolerances: &tolerances,
            warm: None,
            compatibility: step.compatibility,
        },
    )?;
    transport::recover(&mut report, &normalization, &lowered.problem.contract)?;
    if let Some(c) = &mut report.certificate {
        crate::certificate::verify(c, &lowered.problem, &original, step.accuracy.feasibility);
    }
    lowered.raise(&mut report, run.original, step.tolerances)?;
    quality::qualify(&mut report, step.accuracy);
    Ok(report)
}

/// Native solve over a cone model normalized at preparation, recovery, verification of a
/// certificate against the `original` cone data, and qualification.
///
/// # Errors
/// Native execution failed before a report existed.
pub fn cone(
    step: Step<'_>,
    retained: &mut Retained,
    problem: &ConicProblem,
    original: &ConicProblem,
    certificate: &dyn QuadraticEvidence,
) -> Result<SolveReport, ProblemError> {
    step.preflight()?;
    if let Some(structure) = step.structure {
        crate::structural::validate_assessment(
            structure,
            &original.contract,
            &crate::structural::conic_bounds(original)?,
        )?;
    }
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
            snapshot: step.snapshot,
            structure: step.structure,
            execution: step.execution,
            tolerances: &tolerances,
            warm: None,
            compatibility: step.compatibility,
        },
    )?;
    transport::recover(&mut report, step.normalization, &problem.contract)?;
    if let Some(c) = &mut report.certificate {
        crate::certificate::verify(c, original, step.tolerances, step.accuracy.feasibility);
    }
    quality::qualify(&mut report, step.accuracy);
    Ok(report)
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use crate::{
        clarabel_tests::{Affine, budgets, mixed_lp},
        solve::{Assurance, Backend, Qualification, Termination},
        solver_tests::stamp,
    };

    /// Only original-model revalidation fails; Clarabel receives unchanged coefficients.
    struct Original<'a> {
        affine: Affine<'a>,
        evaluations: usize,
        source: pse_ids::SemanticId,
        panic_next: bool,
    }
    impl OriginalModel for Original<'_> {
        fn evaluate(&mut self, primal: &[f64]) -> Result<Evaluation, ProblemError> {
            self.evaluations += 1;
            assert!(primal.iter().all(|v| v.is_finite()));
            assert!(!self.panic_next, "test original-model callback panic");
            if self.evaluations == 1 {
                return Err(pse_math::MathError::Domain {
                    source_id: self.source,
                    requirement: "test original-model revalidation",
                }
                .into());
            }
            self.affine.evaluate(primal)
        }
    }

    #[test]
    fn linked_coefficient_revalidation_withdraws_stale_quality_and_recovers() {
        let problem = mixed_lp();
        let backend = Backend::Clarabel;
        let controls = Controls::default();
        let accuracy = ResolvedAccuracy::verification();
        let tolerances = budgets(&problem, 1e-7);
        let normalization = Normalization::identity(
            problem.contract.variables.len(),
            problem.contract.rows.len(),
        );
        let snapshot = super::super::Snapshot::observe(&super::super::LINKED);
        let step = || Step {
            adapter: super::super::adapter(backend),
            settings: &BackendSettings::Default,
            snapshot: &snapshot,
            structure: None,
            controls: &controls,
            accuracy: &accuracy,
            execution: crate::solve::Execution::new(std::sync::Arc::default(), &controls),
            tolerances: &tolerances,
            normalization: &normalization,
            compatibility: stamp(backend),
            warm: None,
        };
        let source = pse_ids::SemanticId::from_bytes([17; 16]);
        let mut original = Original {
            affine: Affine(&problem),
            evaluations: 0,
            source,
            panic_next: false,
        };
        let constants = vec![0.0; problem.bounds.len()];
        let mut report = coefficients(
            step(),
            &mut Retained::default(),
            Coefficients {
                lowered: None,
                problem: &problem,
                certificate: None,
                row_constants: &constants,
                row_bounds: problem.bounds.clone(),
                original: &mut original,
            },
        )
        .unwrap();
        assert_eq!(original.evaluations, 1);
        assert!(report.candidate.is_some());
        assert_eq!(report.termination.category, Termination::Success);
        assert!(report.quality.is_none());
        assert!(report.observation.is_none());
        assert_eq!(report.qualification, Qualification::Unqualified);
        assert_eq!(report.termination.assurance, Assurance::None);
        assert!(matches!(report.validation_failure(),
            Some(ProblemError::Math(pse_math::MathError::Domain { source_id, .. }))
                if *source_id == source));
        assert!(report.evidence.conic.is_some());
        assert!(!report.metrics.is_empty());
        assert!(report.provenance["native"].contains("Clarabel"));

        // Reconstruct the coefficient observation before another independent original
        // evaluation of this same native candidate; no native solve or failure switch.
        let candidate = report.candidate.as_ref().unwrap();
        report.observation = Some(
            problem
                .observation(&candidate.primal, &constants, problem.bounds.clone())
                .unwrap(),
        );
        reobserve(&mut report, &problem, &mut original, &step()).unwrap();
        assert_eq!(original.evaluations, 2);
        assert!(report.validation_failure().is_none());
        assert!(report.quality.as_ref().unwrap().feasible());
        assert!(report.observation.is_some());
        quality::qualify(&mut report, &accuracy);
        assert_eq!(report.qualification, Qualification::OptimalWithinTolerance);

        original.panic_next = true;
        reobserve(&mut report, &problem, &mut original, &step()).unwrap();
        assert!(matches!(
            report.validation_failure(),
            Some(ProblemError::Internal(_))
        ));
        assert!(report.candidate.is_some());
        assert!(report.quality.is_none());
        assert!(report.observation.is_none());
        assert_eq!(report.qualification, Qualification::Unqualified);
        assert_eq!(report.termination.category, Termination::Success);
        assert!(report.evidence.conic.is_some());
    }
}
