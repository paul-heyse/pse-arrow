// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! POUNCE-convex adapter (Plan 22 N5; I5): the `pounce-convex` interior-point method for
//! linear, convex quadratic and continuous cone programs in the pse cone form, selected
//! explicitly only, and the one adapter that solves a batch of independent programs in
//! parallel on the admitted threads.
//!
//! **Form.** The cone form `A x + s = b, s ∈ K` with a variable box maps to POUNCE's
//! standard form: zero-cone rows become equalities `A_eq x = b_eq`, every other block a
//! block of inequality rows `G x ⪯_K h` with its `ConeSpec`, and the box stays first class.
//! The power cone is stored norm-first in POUNCE (`|x₁| ≤ x₂^α x₃^{1−α}`) and last in the
//! pse form, and a PSD block is POUNCE's lower triangle column by column against the pse
//! upper triangle; both are row permutations, mapped back for the multipliers. The
//! generalized power cone has no POUNCE counterpart and is refused.
//!
//! **Batch.** A batch of programs whose cones are all zero or nonnegative (linear and
//! quadratic programs) runs through `solve_qp_batch_parallel_warm`, each instance factored
//! serially on one of the admitted threads; any other batch solves its instances in
//! parallel through `solve_socp_ipm_warm`. Each report is built and later qualified on its
//! own. The previous solutions of the same layout warm-start the next call.
//!
//! **Cancellation.** POUNCE-convex exposes a solve-wide deadline and no interrupt: a stop
//! request is honoured before a solve or batch starts, and a running solve stops at the
//! step's deadline, overshooting it by at most one factorization.
use super::{BackendExecution, BackendSettings, Capability, Input, Representation, Retained};
use crate::{
    ProblemError,
    solve::{Backend, Controls, DerivativeCapability, ProblemClass, SolveReport, WarmCapability},
};

#[derive(Debug)]
pub(super) struct PounceConvex;
pub(super) static ADAPTER: PounceConvex = PounceConvex;
static CAPABILITY: Capability = Capability {
    structural: crate::structural::Policy::Equalities,
    lexicographic_degradation: crate::routing::DegradationSupport::Max,
    classes: &[
        ProblemClass::Linear,
        ProblemClass::ConvexQuadratic,
        ProblemClass::ContinuousCone,
    ],
    // Explicit only: HiGHS owns linear and quadratic programs automatically and Clarabel
    // continuous cone programs; POUNCE-convex serves a selection that asks for it, such as
    // a batched study.
    automatic_classes: &[],
    derivatives: DerivativeCapability::Coefficients,
    warm: WarmCapability::None,
    general_bounds: true,
    sign_bounds: true,
    parallel: true,
    certifies: false,
    native_forms: &[],
    requirements: &[],
    lexicographic: &[],
    batch: true,
    sensitivities: false,
    reuse: "the previous solutions of the same layout warm-start the interior point; no native state is retained",
    cancellation: "solve-wide deadline only: a stop request is honoured before a solve or batch starts; a running solve stops at the deadline, overshooting by at most one factorization",
    diagnostics: "native status, iterations and objective; primal, dual and gap residuals of the cone form",
};
impl BackendExecution for PounceConvex {
    fn backend(&self) -> Backend {
        Backend::PounceConvex
    }
    fn capability(&self) -> &'static Capability {
        &CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Cone
    }
    fn linked(&self) -> bool {
        cfg!(feature = "pounce")
    }
    fn automatic(&self) -> Option<u8> {
        None
    }
    fn admit_settings(
        &self,
        settings: &BackendSettings,
        _controls: &Controls,
    ) -> Result<(), ProblemError> {
        match settings {
            BackendSettings::Default | BackendSettings::PounceConvex(_) => Ok(()),
            _ => Err(super::foreign(Backend::PounceConvex)),
        }
    }
    fn scope(
        &self,
        threads: usize,
        stack: usize,
        work: &mut (dyn FnMut() + Send),
    ) -> Result<(), ProblemError> {
        #[cfg(feature = "pounce")]
        {
            crate::pounce::with_threads(threads, stack, move || {
                work();
                Ok::<(), ProblemError>(())
            })
        }
        #[cfg(not(feature = "pounce"))]
        {
            let _ = (threads, stack);
            work();
            Ok(())
        }
    }
    fn execute(
        &self,
        retained: &mut Retained,
        input: Input<'_>,
    ) -> Result<SolveReport, ProblemError> {
        self.execute_batch(retained, vec![input])
            .pop()
            .unwrap_or_else(|| Err(ProblemError::Internal("empty POUNCE-convex batch".into())))
    }
    fn execute_batch(
        &self,
        retained: &mut Retained,
        inputs: Vec<Input<'_>>,
    ) -> Vec<Result<SolveReport, ProblemError>> {
        #[cfg(feature = "pounce")]
        {
            native::solve(retained, inputs)
        }
        #[cfg(not(feature = "pounce"))]
        {
            let _ = retained;
            inputs
                .iter()
                .map(|_| {
                    Err(ProblemError::Unsupported(
                        "POUNCE-convex is not linked in this build".into(),
                    ))
                })
                .collect()
        }
    }
}

#[cfg(feature = "pounce")]
mod native {
    use super::super::{BackendSettings, Input, Problem, Retained};
    use crate::{
        ConicProblem, ProblemError,
        conic::{self, Cone},
        solve::{
            Assurance, Backend, Candidate, CandidateKind, CertificateAccuracy, CertificateKind,
            ConicEvidence, Event, InfeasibilityCertificate, Metric, NativeTermination,
            RayCoordinate, RayEntry, SolveReport, Termination,
        },
    };
    use pounce_rs::{
        convex::{
            ConeSpec, QpOptions, QpProblem, QpSolution, QpStatus, QpWarmStart, Triplet,
            solve_qp_batch_parallel_warm, solve_socp_ipm_warm,
        },
        linsol::{FeralSolverInterface, SparseSymLinearSolverInterface},
    };
    use rayon::prelude::*;
    use std::collections::BTreeMap;

    type Settings = crate::settings::pounce_convex::Settings;

    /// Where an original cone-form row lives in POUNCE's standard form.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Target {
        Equality(usize),
        Inequality(usize),
    }
    /// One program's standard form and the map back to its rows.
    #[derive(Debug)]
    struct Form {
        problem: QpProblem,
        specs: Vec<ConeSpec>,
        rows: Vec<Target>,
        /// Only zero and nonnegative cones: a quadratic program.
        quadratic: bool,
    }
    /// The layout a warm start must match: coordinates, row and cone shapes.
    #[derive(Clone, Debug, PartialEq)]
    struct Layout {
        coordinates: pse_ids::ContentHash,
        n: usize,
        rows: Vec<Target>,
        specs: Vec<ConeSpec>,
    }
    /// The previous call's solutions, by position, as warm starts of the next.
    #[derive(Debug, Default)]
    struct Session {
        warm: Vec<Option<(Layout, QpWarmStart)>>,
    }
    /// No warm start: the dimension mismatch makes POUNCE start the instance cold.
    fn cold() -> QpWarmStart {
        QpWarmStart {
            x: vec![],
            y: vec![],
            z: vec![],
            z_lb: vec![],
            z_ub: vec![],
        }
    }

    /// Rows of the pse PSD triangle (upper, column by column) in POUNCE's order (lower,
    /// column by column): entry `q` is the pse row of POUNCE row `q`. Both scale
    /// off-diagonals by `√2`.
    fn psd_rows(order: usize) -> Vec<usize> {
        let mut rows = Vec::with_capacity(order * (order + 1) / 2);
        for c in 0..order {
            for r in c..order {
                // Symmetric entry (c, r) with c ≤ r in the pse upper triangle.
                rows.push(r * (r + 1) / 2 + c);
            }
        }
        rows
    }

    fn form(p: &ConicProblem) -> Result<Form, ProblemError> {
        let m = p.rhs.len();
        let mut rows = vec![Target::Equality(0); m];
        let (mut equalities, mut inequalities) = (0, 0);
        let mut specs = Vec::new();
        let mut quadratic = true;
        let mut start = 0;
        for cone in &p.cones {
            let d = cone.dim();
            let order: Vec<usize> = match cone {
                Cone::Zero { .. } => {
                    for row in &mut rows[start..start + d] {
                        *row = Target::Equality(equalities);
                        equalities += 1;
                    }
                    start += d;
                    continue;
                }
                Cone::Nonnegative { dimension } => {
                    specs.push(ConeSpec::Nonneg(*dimension));
                    (0..d).collect()
                }
                Cone::SecondOrder { dimension } => {
                    quadratic = false;
                    specs.push(ConeSpec::SecondOrder(*dimension));
                    (0..d).collect()
                }
                Cone::Exponential => {
                    quadratic = false;
                    specs.push(ConeSpec::Exponential);
                    (0..3).collect()
                }
                Cone::Power { alpha } => {
                    quadratic = false;
                    specs.push(ConeSpec::Power(*alpha));
                    vec![2, 0, 1]
                }
                Cone::PsdTriangle { order } => {
                    quadratic = false;
                    specs.push(ConeSpec::Psd(*order));
                    psd_rows(*order)
                }
                Cone::GeneralizedPower { .. } => {
                    return Err(ProblemError::Unsupported(
                        "POUNCE-convex has no generalized power cone".into(),
                    ));
                }
            };
            for offset in order {
                rows[start + offset] = Target::Inequality(inequalities);
                inequalities += 1;
            }
            start += d;
        }
        let (mut a, mut g) = (Vec::new(), Vec::new());
        let (mut b, mut h) = (vec![0.0; equalities], vec![0.0; inequalities]);
        for (r, target) in rows.iter().enumerate() {
            match target {
                Target::Equality(k) => b[*k] = p.rhs[r],
                Target::Inequality(k) => h[*k] = p.rhs[r],
            }
        }
        let c = &p.constraints;
        for column in 0..c.columns {
            for k in c.column(column) {
                let value = c.values[k];
                match rows[c.row_indices[k]] {
                    Target::Equality(row) => a.push(Triplet::new(row, column, value)),
                    Target::Inequality(row) => g.push(Triplet::new(row, column, value)),
                }
            }
        }
        // The pse objective matrix is the upper triangle; POUNCE takes the lower one.
        let q = &p.quadratic;
        let mut p_lower = Vec::with_capacity(q.values.len());
        for column in 0..q.columns {
            for k in q.column(column) {
                p_lower.push(Triplet::new(column, q.row_indices[k], q.values[k]));
            }
        }
        let variables = &p.contract.variables;
        Ok(Form {
            problem: QpProblem {
                n: variables.len(),
                p_lower,
                c: p.objective.clone(),
                a,
                b,
                g,
                h,
                lb: variables.iter().map(|v| v.lower).collect(),
                ub: variables.iter().map(|v| v.upper).collect(),
            },
            specs,
            rows,
            quadratic,
        })
    }

    /// One admitted batch member: its cone form, settings, standard form and options.
    struct Member<'a> {
        input: Input<'a>,
        problem: &'a ConicProblem,
        settings: Settings,
        form: Form,
        options: QpOptions,
        layout: Layout,
    }

    fn admit(input: Input<'_>) -> Result<Member<'_>, ProblemError> {
        let Problem::Cone {
            problem,
            certificate,
        } = input.problem
        else {
            return Err(super::super::representation(Backend::PounceConvex));
        };
        problem.validate(certificate)?;
        input.tolerances.validate(
            problem.contract.variables.len(),
            problem.contract.rows.len(),
        )?;
        input.controls.validate()?;
        let settings = match input.settings {
            BackendSettings::Default => Settings::default(),
            BackendSettings::PounceConvex(settings) => settings.clone(),
            #[allow(
                unreachable_patterns,
                reason = "other adapters' variants exist only when their features are linked"
            )]
            _ => return Err(super::super::foreign(Backend::PounceConvex)),
        };
        let form = form(problem)?;
        let execution = &input.execution;
        let options = QpOptions {
            time_limit: Some(
                execution
                    .time_limit
                    .saturating_sub(execution.started.elapsed()),
            ),
            // The interior point stops a decade inside every budget the report is later
            // qualified against: residuals, stationarity and the duality gap.
            tol: (0.1
                * input
                    .accuracy
                    .stationarity
                    .min(input.accuracy.feasibility)
                    .min(input.accuracy.gap_relative)
                    .min(input.accuracy.gap_absolute))
            .max(f64::EPSILON),
            max_iter: usize::try_from(input.controls.iterations).unwrap_or(usize::MAX),
            use_hsde: settings.self_dual,
            equilibrate: settings.equilibrate,
            crossover: settings.crossover,
            ..QpOptions::default()
        };
        let layout = Layout {
            coordinates: input.compatibility.layout,
            n: form.problem.n,
            rows: form.rows.clone(),
            specs: form.specs.clone(),
        };
        Ok(Member {
            input,
            problem,
            settings,
            form,
            options,
            layout,
        })
    }

    /// Solve every input: a stopped member reports its stop, the others run as one
    /// parallel batch, each warm-started from the previous call's solution at its position
    /// when the layouts agree.
    pub(super) fn solve(
        retained: &mut Retained,
        inputs: Vec<Input<'_>>,
    ) -> Vec<Result<SolveReport, ProblemError>> {
        let count = inputs.len();
        let mut members: Vec<Option<Member<'_>>> = Vec::with_capacity(count);
        let mut results: Vec<Option<Result<SolveReport, ProblemError>>> =
            (0..count).map(|_| None).collect();
        for (i, input) in inputs.into_iter().enumerate() {
            match admit(input) {
                Ok(member) => match member.input.execution.stopped() {
                    Some(stop) => {
                        results[i] = Some(Ok(stopped(&member, stop)));
                        members.push(None);
                    }
                    None => members.push(Some(member)),
                },
                Err(error) => {
                    results[i] = Some(Err(error));
                    members.push(None);
                }
            }
        }
        let Some(first) = members.iter().flatten().next() else {
            return results.into_iter().flatten().collect();
        };
        let policy = first.input.controls.reuse;
        let session = match retained.session(
            Backend::PounceConvex,
            policy,
            |_: &mut Session| Ok(true),
            || Ok(Session::default()),
        ) {
            Ok((session, _)) => session,
            Err(error) => {
                let error = std::sync::Arc::new(error);
                return (0..count)
                    .map(|i| {
                        results[i].take().unwrap_or_else(|| {
                            Err(ProblemError::Internal(format!(
                                "POUNCE-convex session: {error}"
                            )))
                        })
                    })
                    .collect();
            }
        };
        let warm: Vec<QpWarmStart> = members
            .iter()
            .enumerate()
            .map(|(i, member)| {
                member
                    .as_ref()
                    .and_then(|m| {
                        session
                            .warm
                            .get(i)
                            .and_then(Option::as_ref)
                            .filter(|(layout, _)| *layout == m.layout)
                            .map(|(_, w)| w.clone())
                    })
                    .unwrap_or_else(cold)
            })
            .collect();
        let active: Vec<usize> = (0..count).filter(|i| members[*i].is_some()).collect();
        let solutions = solve_members(&members, &warm, &active);
        session.warm = (0..count)
            .map(|i| {
                let solved = active
                    .iter()
                    .position(|a| *a == i)
                    .and_then(|k| solutions[k].as_ref());
                match (&members[i], solved) {
                    (Some(m), Some(solution)) if solution.x.iter().all(|v| v.is_finite()) => {
                        Some((m.layout.clone(), QpWarmStart::from_solution(solution)))
                    }
                    _ => None,
                }
            })
            .collect();
        let batch = active.len();
        for (k, i) in active.into_iter().enumerate() {
            let (Some(member), Some(solution)) = (&members[i], &solutions[k]) else {
                continue;
            };
            results[i] = Some(report(member, solution, batch, !warm[i].x.is_empty()));
        }
        results
            .into_iter()
            .map(|r| {
                r.unwrap_or_else(|| {
                    Err(ProblemError::Internal(
                        "POUNCE-convex batch member unsolved".into(),
                    ))
                })
            })
            .collect()
    }

    /// Solve the active members in parallel on the admitted pool: quadratic programs with
    /// one options set through the library's warm batch, the others instance by instance.
    /// A single member factors on the worker's threads; a batch factors each instance
    /// serially, its parallelism across instances.
    fn solve_members(
        members: &[Option<Member<'_>>],
        warm: &[QpWarmStart],
        active: &[usize],
    ) -> Vec<Option<QpSolution>> {
        struct Job<'a> {
            problem: &'a QpProblem,
            specs: &'a [ConeSpec],
            quadratic: bool,
            warm: &'a QpWarmStart,
            options: &'a QpOptions,
            linear: crate::settings::pounce::LinearSettings,
        }
        let serial = active.len() > 1;
        let jobs: Vec<Job<'_>> = active
            .iter()
            .filter_map(|i| {
                members[*i].as_ref().map(|m| {
                    let mut linear = m.settings.linear.clone();
                    linear.parallel = Some(!serial && m.input.controls.threads > 1);
                    linear.fma = false;
                    Job {
                        problem: &m.form.problem,
                        specs: &m.form.specs,
                        quadratic: m.form.quadratic,
                        warm: &warm[*i],
                        options: &m.options,
                        linear,
                    }
                })
            })
            .collect();
        let backend = |linear: &crate::settings::pounce::LinearSettings| {
            let linear = linear.clone();
            move || -> Box<dyn SparseSymLinearSolverInterface> {
                Box::new(FeralSolverInterface::with_config(linear.clone()))
            }
        };
        let uniform = jobs.first().is_some_and(|f| {
            jobs.iter().all(|j| {
                j.quadratic
                    && format!("{:?}", j.options) == format!("{:?}", f.options)
                    && format!("{:?}", j.linear) == format!("{:?}", f.linear)
            })
        });
        if uniform && let Some(first) = jobs.first() {
            let problems: Vec<QpProblem> = jobs.iter().map(|j| j.problem.clone()).collect();
            let warms: Vec<QpWarmStart> = jobs.iter().map(|j| j.warm.clone()).collect();
            return solve_qp_batch_parallel_warm(
                &problems,
                &warms,
                first.options,
                backend(&first.linear),
            )
            .into_iter()
            .map(Some)
            .collect();
        }
        jobs.par_iter()
            .map(|j| {
                Some(solve_socp_ipm_warm(
                    j.problem,
                    j.specs,
                    j.warm,
                    j.options,
                    backend(&j.linear),
                ))
            })
            .collect()
    }

    fn status_name(status: QpStatus) -> &'static str {
        match status {
            QpStatus::Optimal => "optimal",
            QpStatus::OptimalInaccurate => "optimal_inaccurate",
            QpStatus::PrimalInfeasible => "primal_infeasible",
            QpStatus::DualInfeasible => "dual_infeasible",
            QpStatus::IterationLimit => "iteration_limit",
            QpStatus::TimeLimit => "time_limit",
            QpStatus::NumericalFailure => "numerical_failure",
        }
    }
    fn termination(status: QpStatus) -> NativeTermination {
        let (code, category) = match status {
            QpStatus::Optimal => (0, Termination::Success),
            QpStatus::OptimalInaccurate => (1, Termination::Acceptable),
            QpStatus::PrimalInfeasible => (2, Termination::Infeasible),
            QpStatus::DualInfeasible => (3, Termination::Unbounded),
            QpStatus::IterationLimit => (4, Termination::IterationLimit),
            QpStatus::TimeLimit => (5, Termination::TimeLimit),
            QpStatus::NumericalFailure => (6, Termination::Numerical),
        };
        NativeTermination {
            code,
            name: status_name(status).into(),
            message: None,
            category,
            assurance: Assurance::None,
        }
    }

    fn stopped(member: &Member<'_>, stop: Termination) -> SolveReport {
        let mut report = SolveReport::new(
            Backend::PounceConvex,
            &member.problem.contract,
            NativeTermination {
                code: -1,
                name: "not_started".into(),
                message: None,
                category: stop,
                assurance: Assurance::None,
            },
            &member.input.execution,
        );
        report.termination.category = stop;
        report
    }

    /// The report of one member's solution in the pse cone form: multipliers mapped back to
    /// its rows, residual evidence, a certificate for an infeasibility status, and the
    /// candidate's quality.
    fn report(
        member: &Member<'_>,
        solution: &QpSolution,
        batch: usize,
        warm: bool,
    ) -> Result<SolveReport, ProblemError> {
        let p = member.problem;
        let execution = &member.input.execution;
        let mut report = SolveReport::new(
            Backend::PounceConvex,
            &p.contract,
            termination(solution.status),
            execution,
        );
        if solution.status == QpStatus::TimeLimit
            && execution.stopped() == Some(Termination::Cancelled)
        {
            report.termination.category = Termination::Cancelled;
        }
        let iterations = i64::try_from(solution.iters).unwrap_or(i64::MAX);
        report.metrics = BTreeMap::from([
            ("iterations".into(), Metric::Integer(iterations)),
            ("objective".into(), Metric::Real(solution.obj)),
            (
                "status".into(),
                Metric::Text(status_name(solution.status).into()),
            ),
            (
                "batch".into(),
                Metric::Integer(i64::try_from(batch).unwrap_or(i64::MAX)),
            ),
            ("warm_started".into(), Metric::Bool(warm)),
        ]);
        execution.progress.push(Event {
            phase: "pounce_convex.solve".into(),
            elapsed: execution.started.elapsed(),
            values: report.metrics.clone(),
            incumbent: None,
        });
        report.provenance.insert(
            "native".into(),
            format!(
                "pounce-convex 0.12.0 over FERAL 0.18.0; {}; {}",
                if member.settings.self_dual {
                    "homogeneous self-dual embedding"
                } else {
                    "direct infeasible-start interior point"
                },
                if member.form.quadratic {
                    "quadratic program"
                } else {
                    "cone program"
                }
            ),
        );
        report.provenance.insert(
            "settings.effective".into(),
            serde_json::to_string(&member.settings)
                .map_err(|e| ProblemError::Internal(e.to_string()))?,
        );
        let n = p.contract.variables.len();
        let m = p.rhs.len();
        if solution.x.len() != n
            || solution.z_lb.len() != n
            || solution.z_ub.len() != n
            || member.form.rows.iter().any(|t| match t {
                Target::Equality(k) => *k >= solution.y.len(),
                Target::Inequality(k) => *k >= solution.z.len(),
            })
        {
            report.termination.assurance = Assurance::None;
            return Ok(report);
        }
        let row_dual: Vec<f64> = member
            .form
            .rows
            .iter()
            .map(|t| match t {
                Target::Equality(k) => solution.y[*k],
                Target::Inequality(k) => solution.z[*k],
            })
            .collect();
        if let Some(certificate) = certificate(p, solution, &row_dual) {
            report.certificate = Some(certificate);
            return Ok(report);
        }
        let x = &solution.x;
        if !matches!(
            solution.status,
            QpStatus::Optimal
                | QpStatus::OptimalInaccurate
                | QpStatus::IterationLimit
                | QpStatus::TimeLimit
        ) || !x.iter().all(|v| v.is_finite())
            || !solution.obj.is_finite()
        {
            report.termination.assurance = Assurance::None;
            return Ok(report);
        }
        let slacks = slacks(p, x);
        report.evidence.conic = Some(evidence(p, x, &row_dual, &slacks, solution)?);
        report.candidate = Some(Candidate {
            kind: CandidateKind::FinalIterate,
            primal: x.clone(),
            objective: Some(solution.obj + p.objective_constant),
            row_dual: Some(row_dual),
            bound_dual: Some((solution.z_lb.clone(), solution.z_ub.clone())),
            reduced_costs: None,
            slacks: Some(slacks),
            commitment: None,
        });
        debug_assert_eq!(m, p.contract.rows.len());
        match crate::quality::contained(|| conic::quality(p, x, member.input.tolerances)) {
            Ok(q) => {
                if !q.feasible() {
                    report.termination.assurance = Assurance::None;
                }
                report.quality = Some(q);
            }
            Err(e) => {
                report.record_validation_failure(e);
                report.termination.assurance = Assurance::None;
            }
        }
        Ok(report)
    }

    /// `s = b − A x` over the cone rows.
    fn slacks(p: &ConicProblem, x: &[f64]) -> Vec<f64> {
        let mut s = p.rhs.clone();
        for (column, value) in x.iter().enumerate() {
            for k in p.constraints.column(column) {
                s[p.constraints.row_indices[k]] -= p.constraints.values[k] * value;
            }
        }
        s
    }

    /// Primal, dual and gap residuals of the cone form at the solution, in its coordinates:
    /// the cone and box violations, `‖P x + c + Aᵀ z − z_lb + z_ub‖∞`, and the difference of
    /// the primal and dual objectives.
    fn evidence(
        p: &ConicProblem,
        x: &[f64],
        z: &[f64],
        s: &[f64],
        solution: &QpSolution,
    ) -> Result<ConicEvidence, ProblemError> {
        let mut primal: f64 = 0.0;
        let mut start = 0;
        for cone in &p.cones {
            let end = start + cone.dim();
            primal = primal.max(conic::cone_violation(cone, &s[start..end])?);
            start = end;
        }
        for (v, value) in p.contract.variables.iter().zip(x) {
            primal = primal.max(v.lower - value).max(value - v.upper);
        }
        let mut gradient = p.objective.clone();
        let q = &p.quadratic;
        let mut curvature = 0.0;
        for column in 0..q.columns {
            for k in q.column(column) {
                let (row, value) = (q.row_indices[k], q.values[k]);
                gradient[row] += value * x[column];
                if row != column {
                    gradient[column] += value * x[row];
                    curvature += 2.0 * value * x[row] * x[column];
                } else {
                    curvature += value * x[row] * x[column];
                }
            }
        }
        let linear: f64 = p.objective.iter().zip(x).map(|(c, x)| c * x).sum();
        for (column, g) in gradient.iter_mut().enumerate() {
            for k in p.constraints.column(column) {
                *g += p.constraints.values[k] * z[p.constraints.row_indices[k]];
            }
            *g += solution.z_ub[column] - solution.z_lb[column];
        }
        let dual = gradient.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        let primal_objective = 0.5 * curvature + linear;
        let mut dual_objective =
            -0.5 * curvature - p.rhs.iter().zip(z).map(|(b, z)| b * z).sum::<f64>();
        for (j, v) in p.contract.variables.iter().enumerate() {
            if v.lower.is_finite() {
                dual_objective += v.lower * solution.z_lb[j];
            }
            if v.upper.is_finite() {
                dual_objective -= v.upper * solution.z_ub[j];
            }
        }
        let gap = (primal_objective - dual_objective).abs();
        Ok(ConicEvidence {
            primal_residual: primal.max(0.0),
            dual_residual: dual,
            gap_absolute: gap,
            gap_relative: gap / primal_objective.abs().min(dual_objective.abs()).max(1.0),
        })
    }

    /// The ray of an infeasibility status in the cone form's certificate layout: over the
    /// cone rows then the finite bounds in variable order, lower before upper
    /// ([`conic::bound_rows`]); or a recession direction over the variables.
    fn certificate(
        p: &ConicProblem,
        solution: &QpSolution,
        row_dual: &[f64],
    ) -> Option<InfeasibilityCertificate> {
        let variables = &p.contract.variables;
        let (kind, ray) =
            match solution.status {
                QpStatus::PrimalInfeasible => (
                    CertificateKind::PrimalInfeasible,
                    p.contract
                        .rows
                        .iter()
                        .zip(row_dual)
                        .map(|(id, value)| RayEntry {
                            coordinate: RayCoordinate::Row,
                            id: *id,
                            value: *value,
                        })
                        .chain(conic::bound_rows(variables).into_iter().map(|(i, lower)| {
                            RayEntry {
                                coordinate: if lower {
                                    RayCoordinate::VariableLower
                                } else {
                                    RayCoordinate::VariableUpper
                                },
                                id: variables[i].id,
                                value: if lower {
                                    solution.z_lb[i]
                                } else {
                                    solution.z_ub[i]
                                },
                            }
                        }))
                        .collect(),
                ),
                QpStatus::DualInfeasible => (
                    CertificateKind::DualInfeasible,
                    variables
                        .iter()
                        .zip(&solution.x)
                        .map(|(v, value)| RayEntry {
                            coordinate: RayCoordinate::Variable,
                            id: v.id,
                            value: *value,
                        })
                        .collect(),
                ),
                _ => return None,
            };
        Some(InfeasibilityCertificate {
            kind,
            accuracy: CertificateAccuracy::Full,
            ray,
            verification: None,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::psd_rows;

        /// The order-3 PSD triangle: POUNCE's lower triangle column by column is
        /// `(0,0),(1,0),(2,0),(1,1),(2,1),(2,2)`, the pse upper triangle
        /// `(0,0),(0,1),(1,1),(0,2),(1,2),(2,2)`.
        #[test]
        fn psd_rows_map_lower_to_upper_triangle() {
            assert_eq!(psd_rows(3), vec![0, 1, 3, 2, 4, 5]);
        }
    }
}
