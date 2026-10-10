// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Single and multiple shooting (ADR-0110 Outcome 5) against an analytic optimum, with
//! continuity closing from perturbed nodes and path bounds at the samples.
use super::*;
use crate::workflow::tests::{compiler_profile, id};
use native::solve::{Backend, Controls, SolverSelection};
use pse_backend_native::dynamics::{Method, ScheduledInput};
use pse_modeling::{Bindings, Limits};
use std::time::Instant;

/// A tracking problem: `x' = (u − x)/1 s` from `x(0) = 0` with the algebraic `y = x²`
/// (the states are ordered y, x),
/// the control `u` scheduled on [0, 1) and [1, 2], the integral cost `∫ (x − 1)² dt/1 s`
/// and the reported terminal miss `(x − 1.5)²`.
async fn tracking(method: Method) -> crate::workflow::ModelingSimulation {
    let mut physical = crate::workflow::tests::physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    let rows = pse_authoring::language::parse(
        "package p { def Tracking { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param u: Scalar = 0.5; var x[i in t]: Scalar; var y[i in t]: Scalar; eq rate[i in t]: d(x[i])/di == (u - x[i])/1{s}; eq square[i in t]: y[i] == x[i]*x[i]; eq initial: x[0{s}] == 0; annotation start x(0); annotation start y(0); let cost: Scalar = integral(i in t | (x[i]-1)*(x[i]-1)/1{s}); annotation check cost(cost >= 0); let miss[i in t]: Scalar = (x[i]-1.5)*(x[i]-1.5); annotation report miss(\"terminal miss\"); } }",
        id(30),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Tracking")
        .unwrap()
        .declaration_id;
    let package = crate::workflow::tests::runtime_with_workspace(32 << 20)
        .modeling_package(rows, physical)
        .await
        .unwrap();
    let allowance = pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
    let profile = native::dynamics::Profile {
        method,
        end: 2.,
        samples: vec![0., 0.5, 1., 1.5, 2.],
        rtol: allowance,
        atol: vec![allowance; 2],
        out_rtol: Some(allowance),
        out_atol: vec![allowance],
        initial_step: 1e-4,
        parameter_scales: vec![1.],
        schedule: vec![ScheduledInput {
            parameter: 0,
            times: vec![1.],
        }],
        ..Default::default()
    };
    package
        .prepare_simulation(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            Default::default(),
            compiler_profile(),
            profile,
            DerivativeOrder::First,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap()
}
fn methods() -> Vec<Method> {
    let mut methods = vec![Method::Diffsol];
    if cfg!(feature = "solver-idas") {
        methods.push(Method::Idas);
    }
    methods
}
fn solver() -> SolverProfile {
    SolverProfile {
        selection: SolverSelection::Explicit(Backend::Ipopt),
        controls: Controls {
            hessian: HessianMode::LimitedMemory,
            ..Default::default()
        },
        presolve: native::presolve::Policy::Off,
        ..Default::default()
    }
}
/// The shooting request on the tracking problem: the terminal miss and the integral cost
/// with unit weights.
fn request(
    simulation: &crate::workflow::ModelingSimulation,
    method: ShootingMethod,
    nodes: Vec<f64>,
    controlled: bool,
) -> ShootingProfile {
    let c = simulation.contract();
    ShootingProfile {
        method,
        nodes,
        controls: if controlled {
            vec![ShootingControl {
                input: c.parameters[0],
                lower: Some(0.),
                upper: Some(5.),
            }]
        } else {
            vec![]
        },
        path: vec![],
        objective: ShootingObjective {
            // The outputs are the states, then the reported miss.
            terminal: BTreeMap::from([(c.outputs[2], 1.)]),
            integral: BTreeMap::from([(c.quadratures[0], 1.)]),
        },
        solver: solver(),
    }
}
/// The analytic optimum of J(u₁, u₂) = ∫₀² (x − 1)² dt + (x(2) − 1.5)², with x linear in
/// the piecewise-constant control: x = u₁(1 − e^{−t}) on [0, 1] and, with s = t − 1,
/// x = c·e^{−s}·u₁ + (1 − e^{−s})·u₂ on [1, 2], c = 1 − e^{−1}. The normal equations
/// of the quadratic give the minimizer; J* = 4.25 − rᵀA⁻¹r.
fn quadratic() -> ([f64; 3], [f64; 2]) {
    let (e1, e2) = ((-1f64).exp(), (-2f64).exp());
    let c = 1. - e1;
    let square = 1. - 2. * c + (1. - e2) / 2.;
    let (ta, tb) = (c * e1, 1. - e1);
    let a11 = square + c * c * (1. - e2) / 2. + ta * ta;
    let a12 = c * (c - (1. - e2) / 2.) + ta * tb;
    let a22 = square + tb * tb;
    let r1 = e1 + c * c + 1.5 * ta;
    let r2 = e1 + 1.5 * tb;
    ([a11, a12, a22], [r1, r2])
}
fn original_objective(u: &[f64]) -> f64 {
    assert_eq!(u.len(), 2);
    let ([a11, a12, a22], [r1, r2]) = quadratic();
    4.25 - 2. * (r1 * u[0] + r2 * u[1])
        + a11 * u[0] * u[0]
        + 2. * a12 * u[0] * u[1]
        + a22 * u[1] * u[1]
}
fn analytic() -> ([f64; 2], f64) {
    let ([a11, a12, a22], [r1, r2]) = quadratic();
    let det = a11 * a22 - a12 * a12;
    let u = [(r1 * a22 - r2 * a12) / det, (a11 * r2 - a12 * r1) / det];
    (u, original_objective(&u))
}
fn objective_budget(problem: &ShootingProblem) -> f64 {
    crate::workflow::tests::engineering_target(
        problem.numerics(),
        NumericalTarget::Objective,
        SemanticId::NIL,
    )
    .budget
}
fn assert_constraints(report: &ShootingReport, bounds: &[(f64, f64)], budgets: &[f64]) {
    let values = report.constraint_values.as_ref().unwrap();
    assert_eq!(values.len(), bounds.len());
    assert_eq!(values.len(), budgets.len());
    for ((value, (lower, upper)), budget) in values.iter().zip(bounds).zip(budgets) {
        assert!(
            value.is_finite() && *value >= lower - budget && *value <= upper + budget,
            "{value} outside [{lower}, {upper}] with {budget}"
        );
    }
    assert!(report.quality.as_ref().unwrap().feasible());
}
fn assert_tracking_publication(
    report: &ShootingReport,
    simulation: &crate::workflow::ModelingSimulation,
) {
    let contract = simulation.contract();
    assert_eq!(contract.quadratures.len(), 1);
    assert_eq!(contract.outputs.len(), 3);
    let state_outputs = contract
        .states
        .iter()
        .map(|state| pse_compiler::workspace::ModelingOutput::Member(*state).row_id())
        .collect::<Vec<_>>();
    assert_eq!(
        &contract.outputs[..contract.states.len()],
        state_outputs.as_slice()
    );
    assert_eq!(contract.differential.iter().filter(|d| **d).count(), 1);
    let x = contract.differential.iter().position(|d| *d).unwrap();
    let trajectory = report.trajectory.as_ref().unwrap();
    assert_eq!(trajectory.requested_initial.len(), contract.states.len());
    assert_eq!(trajectory.samples.len(), simulation.profile().samples.len());
    for (sample, time) in trajectory.samples.iter().zip(&simulation.profile().samples) {
        assert_eq!(sample.time, *time);
    }
    let last = trajectory.samples.last().unwrap();
    assert_eq!(last.time, simulation.profile().end);
    assert_eq!(trajectory.completed_time, last.time);
    assert_eq!(last.integrals.len(), contract.quadratures.len());
    let miss = last.outputs[x] - 1.5;
    let terminal = miss * miss;
    // A manufactured arithmetic identity checks the authored terminal expression;
    // this is unrelated to integration convergence or forward solution accuracy.
    let roundoff = 16. * f64::EPSILON * terminal.abs().max(last.outputs[2].abs()).max(1.);
    assert!((last.outputs[2] - terminal).abs() <= roundoff);
    // Production publishes the sum of the retained native cumulative quadrature
    // and terminal output. Local integration controls do not promise that this
    // assembled observation has the NLP Objective target's forward accuracy.
    let published = last.integrals[0] + last.outputs[2];
    // The evaluator adds terminal then window integrals; stitching accumulates
    // the integrals first. Allow floating-point reassociation, not integration error.
    let composition_roundoff =
        16. * f64::EPSILON * (last.integrals[0].abs() + last.outputs[2].abs()).max(1.);
    assert!((report.objective.unwrap() - published).abs() <= composition_roundoff);
}
fn assert_original_objective(
    report: &ShootingReport,
    optimum: f64,
    budget: f64,
    simulation: &crate::workflow::ModelingSimulation,
) {
    let controls = report.controls.values().next().unwrap();
    let original = original_objective(controls);
    assert!(original.is_finite());
    // This is an empirical check of the decision quantity, not a forward-control
    // error guarantee inferred from stationarity or local integration tolerances.
    assert!(
        (original - optimum).abs() <= budget,
        "{original} vs {optimum}; budget={budget}"
    );
    assert_tracking_publication(report, simulation);
}
async fn solve(
    problem: ShootingProblem,
    initial: Option<&[f64]>,
) -> Arc<crate::workflow::RunResult> {
    let problem = Arc::new(problem);
    let handle = match initial {
        Some(initial) => problem.start_with_initial(initial.to_vec()),
        None => problem.start(),
    }
    .unwrap();
    handle.wait().await.unwrap()
}
fn report_of(result: &crate::workflow::RunResult) -> &ShootingReport {
    let crate::workflow::RunReport::Shooting(report) = result.report().unwrap() else {
        panic!("shooting report expected");
    };
    let trace = report
        .strategy
        .as_ref()
        .expect("joined shooting strategy trace");
    assert!(trace.owner.is_some());
    assert!(trace.events.iter().all(|event| event.decision.is_some()));
    use pse_relations::columnar::RelationRow;
    let rows = pse_relations::generated::runtime::solve_strategy_events::Row::rows(
        &result.table("runtime.solve_strategy_events").unwrap(),
    )
    .unwrap();
    assert_eq!(rows.len(), trace.events.len());
    assert_eq!(
        rows.iter()
            .filter(|row| row.kind == pse_model::generated::enums::NumericalEventKind::Started)
            .count(),
        1
    );
    let final_row = rows.last().unwrap();
    assert_eq!(final_row.phase, pse_model::strategy::Phase::Assessment);
    assert_eq!(final_row.attempts, Some(1));
    assert_eq!(final_row.evaluations, None);
    assert_eq!(final_row.factorizations, None);
    report
}

#[tokio::test]
async fn scientific_shooting_strict_composed_work_refuses_before_integration() {
    let simulation = tracking(Method::Diffsol).await;
    let mut requested = request(&simulation, ShootingMethod::Single, vec![], false);
    let limits = pse_model::strategy::WorkLimits {
        attempts: 1,
        evaluations: Some(1),
        iterations: None,
        factorizations: None,
        proof_steps: None,
    };
    requested.solver.composition.limits = Some(limits);
    let problem = Arc::new(simulation.shooting(requested).unwrap());
    let error = problem
        .solve(
            id(99).into(),
            pse_kernels::ExecutionScope::new(Arc::default(), None),
            Arc::new(native::solve::Progress::new(16)),
            None,
            &problem.solver,
        )
        .unwrap_err();
    let trace = error.strategy_trace().unwrap();
    assert_eq!(trace.declaration.limits, limits);
    assert!(
        trace
            .events
            .iter()
            .all(|event| event.kind != pse_model::generated::enums::NumericalEventKind::Started)
    );
    assert!(
        trace
            .events
            .iter()
            .any(|event| matches!(event.cause.as_deref(), Some(ProblemError::Unsupported(_))))
    );
    assert!(trace.events.iter().all(|event| event.decision.is_some()));
    // An explicit vector cannot acquire entry permission from optional recovery.
    let mut requested = request(&simulation, ShootingMethod::Single, vec![], false);
    requested.solver.composition.recovery = vec![pse_model::strategy::StartOrigin::Auxiliary];
    let problem = Arc::new(simulation.shooting(requested).unwrap());
    let error = problem
        .solve(
            id(100).into(),
            pse_kernels::ExecutionScope::new(Arc::default(), None),
            Arc::new(native::solve::Progress::new(16)),
            Some(&[]),
            &problem.solver,
        )
        .unwrap_err();
    let trace = error.strategy_trace().unwrap();
    assert!(
        trace
            .events
            .iter()
            .all(|event| event.kind != pse_model::generated::enums::NumericalEventKind::Started)
    );
    assert!(
        trace
            .events
            .iter()
            .any(|event| matches!(event.cause.as_deref(), Some(ProblemError::Contract(_))))
    );
}

/// ADR-0110 Outcome 5: single shooting and multiple shooting (nodes at 0.5, 1 and 1.5 s)
/// reach the analytic optimum of the tracking problem, whose control grid is its schedule
/// and whose objective has an integral part (adjoint gradient) and a terminal part
/// (forward sensitivities), on Diffsol and IDAS using the production numerical policy.
/// The exact original objective at the returned controls and the original constraints
/// use their frozen engineering allowances. Native publication composition is checked
/// separately from this independent control decision quantity.
/// Simultaneous collocation of the same
/// definition is Y5a's; the comparison here is the optimum of the piecewise-constant
/// control that both routes discretize.
#[tokio::test]
async fn shooting_matches_simultaneous_optimum() {
    let (optimum, value) = analytic();
    assert!(optimum.iter().all(|u| *u > 0. && *u < 5.), "{optimum:?}");
    for method in methods() {
        let simulation = tracking(method).await;
        for (shooting, nodes) in [
            (ShootingMethod::Single, vec![]),
            (ShootingMethod::Multiple, vec![0.5, 1., 1.5]),
        ] {
            let case = format!("{method:?} {shooting:?}");
            let problem = simulation
                .shooting(request(&simulation, shooting, nodes.clone(), true))
                .unwrap();
            assert_eq!(problem.controls(), 2);
            assert_eq!(problem.contract().variables.len(), 2 + nodes.len());
            let budget = objective_budget(&problem);
            let bounds = problem.constraint_bounds().to_vec();
            let row_budgets = problem.tolerances().rows.clone();
            let result = solve(problem, None).await;
            let report = report_of(&result);
            assert!(
                result.usable(),
                "{:?}; complete {}; error {:?}; checks {:?}",
                result.assessments(),
                report.checks_complete,
                report.validation_error,
                report.checks
            );
            let header = result.table("runtime.computation_runs").unwrap();
            let rows =
                pse_relations::generated::runtime::computation_runs::View::from_checked(&header)
                    .unwrap();
            assert_eq!(
                rows.row(0).unwrap().kind,
                pse_model::generated::enums::ComputationKind::Shooting
            );
            assert_eq!(
                result
                    .table("runtime.solve_variables")
                    .unwrap()
                    .batch()
                    .num_rows(),
                report.candidate.as_ref().unwrap().len()
            );
            let solve = report.solve.as_ref().unwrap();
            assert_eq!(
                solve.termination.category,
                native::solve::Termination::Success,
                "{case}: {:?}",
                solve.termination
            );
            assert_original_objective(report, value, budget, &simulation);
            assert_constraints(report, &bounds, &row_budgets);
            assert_eq!(report.nodes.len(), 1 + nodes.len());
            // The model's own checks hold on the stitched trajectory.
            assert!(
                report.checks_complete
                    && !report.checks.is_empty()
                    && report.checks.iter().all(|c| c.satisfied),
                "{case}: {:?}",
                report.checks
            );
        }
    }
}

#[tokio::test]
async fn shooting_trajectory_projection_retains_completion_diagnostic_and_lease() {
    let simulation = tracking(Method::Diffsol).await;
    let mut profile = request(&simulation, ShootingMethod::Single, vec![], false);
    // No controls can satisfy this path: retain the actual refused joined candidate.
    profile.path.push(PathBound {
        output: simulation.contract().outputs[0],
        lower: Some(10.),
        upper: None,
    });
    let problem = simulation.shooting(profile).unwrap();
    let result = solve(problem, None).await;
    let report = report_of(&result);
    assert!(!report.completion.permits_use());
    let native = report.trajectory.as_ref().unwrap();
    let owner = result._owner.as_ref().unwrap();
    let owners_before = Arc::strong_count(owner);
    let header = result.completion().unwrap().computation.clone().unwrap();
    let projected = simulation
        .completed_trajectory(
            report,
            header.clone(),
            result.assessments[0].clone(),
            owner.clone(),
        )
        .unwrap();
    assert!(std::ptr::eq(projected.report(), native.as_ref()));
    assert_eq!(projected.completion(), &report.completion);
    assert!(!projected.accepted());
    assert!(projected.diagnostic().is_some());
    assert_eq!(projected.checks_complete(), report.checks_complete);
    assert_eq!(projected.header(), &header);
    assert_eq!(projected.assessment(), &result.assessments[0]);
    assert_eq!(
        header.kind,
        pse_model::generated::enums::ComputationKind::Shooting
    );
    assert_eq!(
        header.qualification,
        pse_model::generated::enums::NativeQualification::Unqualified
    );
    let cloned = projected.clone();
    assert!(std::ptr::eq(cloned.report(), projected.report()));
    assert_eq!(Arc::strong_count(owner), owners_before + 1);
    drop(cloned);
    drop(projected);
    assert_eq!(Arc::strong_count(owner), owners_before);

    assert_shooting_sample_projection(&result, native, simulation.contract().outputs.len());
    assert_eq!(
        result.completion().unwrap().computation.as_ref(),
        Some(&header)
    );
    assert!(!report.completion.permits_use());

    // A separate actual joined completion makes the range checks nonempty;
    // no contradictory path bound or fabricated report supplies its samples.
    let successful = solve(
        simulation
            .shooting(request(&simulation, ShootingMethod::Single, vec![], false))
            .unwrap(),
        None,
    )
    .await;
    let successful_report = report_of(&successful);
    assert!(successful.usable(), "{:?}", successful.assessments());
    assert!(successful_report.completion.permits_use());
    let successful_native = successful_report.trajectory.as_ref().unwrap();
    assert!(successful_native.samples.len() >= 3);
    assert_shooting_sample_projection(
        &successful,
        successful_native,
        simulation.contract().outputs.len(),
    );
}

fn assert_shooting_sample_projection(
    result: &crate::workflow::RunResult,
    native: &native::dynamics::Report,
    width: usize,
) {
    use pse_relations::{columnar::RelationRow, generated::runtime::simulation_samples};
    fn samples(
        mut cursor: crate::workflow::ResultCursor<'_>,
        bound: usize,
    ) -> Vec<simulation_samples::Row> {
        let mut rows = Vec::new();
        let mut empty_batches = 0;
        while let Some(chunk) = cursor.next_chunk().unwrap() {
            assert_eq!(chunk.batch().schema(), cursor.schema());
            assert!(chunk.batch().num_rows() <= bound);
            empty_batches += usize::from(chunk.batch().num_rows() == 0);
            rows.extend(simulation_samples::Row::rows(&chunk).unwrap());
        }
        // Empty retained relations/windows carry their exact declared schema once.
        assert_eq!(empty_batches, usize::from(rows.is_empty()));
        assert!(cursor.complete());
        assert!(cursor.next_chunk().unwrap().is_none());
        rows
    }
    // Exercise the joined shooting adapter directly. When samples are retained,
    // chunk boundaries and the selected window cross native sample boundaries.
    let name = "runtime.simulation_samples";
    let all = samples(result.table_cursor(name, 2).unwrap(), 2);
    assert_eq!(all, samples(result.table_cursor(name, 4).unwrap(), 4));
    assert_eq!(all.len(), native.samples.len() * width);
    assert!(width >= 2);
    for (index, row) in all.iter().enumerate() {
        let sample = index / width;
        let column = index % width;
        assert_eq!(row.run_id, result.run_id);
        assert_eq!(row.sample, sample as i64);
        assert_eq!(row.time.to_bits(), native.samples[sample].time.to_bits());
        assert_eq!(row.symbol_id, all[column].symbol_id);
        assert_eq!(
            row.value.to_bits(),
            native.samples[sample].outputs[column].to_bits()
        );
    }
    // Refused completions may lawfully retain fewer samples, including none;
    // export only the samples actually retained, never fill the requested horizon.
    let range = (width - 1).min(all.len())..(2 * width + 1).min(all.len());
    assert_eq!(
        samples(result.table_range(name, range.clone(), 2).unwrap(), 2),
        all[range].to_vec()
    );
    for empty in [0..0, all.len()..all.len(), (all.len() + 1)..(all.len() + 3)] {
        assert!(samples(result.table_range(name, empty, 1).unwrap(), 1).is_empty());
    }
}

/// Multiple shooting's continuity rows close from nodes perturbed away from the chained
/// integration, with and without free controls: the stitched trajectory then equals one
/// integration of the horizon at the candidate's controls. Continuity is assessed using
/// the original rows and their frozen production budgets; reintegration remains an
/// independent observation, with accumulated admitted node jumps included explicitly.
/// Invalid requests are refused before any integration.
#[tokio::test]
async fn multiple_shooting_continuity_closes() {
    for method in methods() {
        let simulation = tracking(method).await;
        for controlled in [false, true] {
            let case = format!("{method:?} controlled={controlled}");
            let problem = Arc::new(
                simulation
                    .shooting(request(
                        &simulation,
                        ShootingMethod::Multiple,
                        vec![0.5, 1., 1.5],
                        controlled,
                    ))
                    .unwrap(),
            );
            let execution = Execution::new(Arc::new(AtomicBool::new(false)), &solver().controls);
            let mut initial = problem.initial_point(&execution).unwrap();
            let controls = problem.controls();
            for node in &mut initial[controls..] {
                *node += 0.3;
            }
            let result = problem
                .start_with_initial(initial)
                .unwrap()
                .wait()
                .await
                .unwrap();
            let report = report_of(&result);
            assert!(
                result.usable(),
                "{:?}; complete {}; error {:?}; checks {:?}",
                result.assessments(),
                report.checks_complete,
                report.validation_error,
                report.checks
            );
            let solve = report.solve.as_ref().unwrap();
            assert_eq!(
                solve.termination.category,
                native::solve::Termination::Success,
                "{case}: {:?}",
                solve.termination
            );
            assert_constraints(
                report,
                problem.constraint_bounds(),
                &problem.tolerances().rows,
            );
            // One integration of the horizon at the candidate's controls.
            let mut profile = simulation.profile().clone();
            profile.sensitivity = DynamicSensitivity::None;
            let mut worker = simulation
                .worker(pse_kernels::ExecutionScope::new(
                    Arc::new(AtomicBool::new(false)),
                    Some(Instant::now().checked_add(profile.time_limit).unwrap()),
                ))
                .unwrap();
            let horizon = native::dynamics::integrate(
                &mut worker,
                &profile,
                report.parameters.as_ref().unwrap(),
                Arc::default(),
            )
            .unwrap();
            let stitched = report.trajectory.as_ref().unwrap();
            assert_eq!(stitched.samples.len(), horizon.samples.len());
            let x = simulation
                .contract()
                .differential
                .iter()
                .position(|d| *d)
                .unwrap();
            let state_target = crate::workflow::tests::engineering_target(
                simulation.numerics(),
                NumericalTarget::Variable,
                simulation.contract().states[x],
            );
            let model = &simulation.model().compiled().model;
            let x_lineage = &model.symbols[&simulation.contract().states[x]].lineage;
            let instance = &model.instances[&x_lineage.instance];
            let square_declaration = instance.members["square"];
            let mut square_rows = model.equations.iter().filter(|row| {
                row.lineage.instance == instance.id && row.lineage.declaration == square_declaration
            });
            let square = square_rows.next().unwrap();
            assert!(square_rows.next().is_none());
            let square_target = crate::workflow::tests::engineering_target(
                simulation.numerics(),
                NumericalTarget::Row,
                square.id,
            );
            assert_eq!(square_target.quantity, state_target.quantity);
            assert_eq!(square_target.unit, state_target.unit);
            let algebraic_budget = square_target.budget;
            // The exact flow is contractive in its initial x. Every admitted
            // continuity jump may contribute, while two integrations each have
            // the same empirical physical state allowance.
            let comparison_budget = problem.tolerances().rows.iter().sum::<f64>()
                * state_target.coordinate_scale
                + 2. * state_target.budget;
            for (a, b) in stitched.samples.iter().zip(&horizon.samples) {
                assert_eq!(a.time, b.time);
                assert!(
                    (a.outputs[x] - b.outputs[x]).abs() <= comparison_budget,
                    "{case}: t={} {} vs {}",
                    a.time,
                    a.outputs[x],
                    b.outputs[x]
                );
                // The algebraic y=x² is checked at each actual trajectory point,
                // rather than assigning x's allowance directly to its nonlinear image.
                let y = 1 - x;
                for point in [a, b] {
                    assert!(
                        (point.outputs[y] - point.outputs[x].powi(2)).abs() <= algebraic_budget
                    );
                }
            }
            for (node, t) in report.nodes.iter().skip(1).zip([0.5, 1., 1.5]) {
                let sample = horizon.samples.iter().find(|s| s.time == t).unwrap();
                assert!(
                    (node[0] - sample.state[x]).abs()
                        <= comparison_budget / state_target.coordinate_scale,
                    "{case}: node at {t}"
                );
            }
            if controlled {
                assert_original_objective(
                    report,
                    analytic().1,
                    objective_budget(&problem),
                    &simulation,
                );
            } else {
                assert_eq!(report.parameters.as_deref(), Some([0.5, 0.5].as_slice()));
                assert_tracking_publication(report, &simulation);
            }
        }
    }
    let simulation = tracking(Method::Diffsol).await;
    let c = simulation.contract().clone();
    let base = request(&simulation, ShootingMethod::Multiple, vec![0.5], true);
    for invalid in [
        ShootingProfile {
            nodes: vec![],
            ..base.clone()
        },
        ShootingProfile {
            nodes: vec![2.],
            ..base.clone()
        },
        ShootingProfile {
            nodes: vec![1., 0.5],
            ..base.clone()
        },
        ShootingProfile {
            method: ShootingMethod::Single,
            ..base.clone()
        },
        ShootingProfile {
            controls: vec![ShootingControl {
                input: c.states[0],
                lower: None,
                upper: None,
            }],
            ..base.clone()
        },
        ShootingProfile {
            objective: ShootingObjective {
                terminal: BTreeMap::from([(c.quadratures[0], 1.)]),
                integral: BTreeMap::new(),
            },
            ..base.clone()
        },
        ShootingProfile {
            solver: SolverProfile {
                controls: Controls {
                    hessian: HessianMode::Exact,
                    ..Default::default()
                },
                ..solver()
            },
            ..base.clone()
        },
    ] {
        assert!(matches!(
            simulation.shooting(invalid),
            Err(WorkflowError::Input(_))
        ));
    }
}

/// Path bounds hold at every sample under their original physical budgets. Both routes
/// independently attain the constrained quadratic objective, allowing the bound-row
/// uncertainty rather than demanding nearly identical control coordinates.
#[tokio::test]
async fn shooting_path_bounds_hold_at_samples() {
    let simulation = tracking(Method::Diffsol).await;
    let c = simulation.contract().clone();
    // The outputs begin with the states; x is the differential one.
    let x = c.differential.iter().position(|d| *d).unwrap();
    let c1 = 1. - (-1f64).exp();
    let optimum = [0.9 / c1, 0.9];
    let ([a11, a12, a22], [r1, r2]) = quadratic();
    let end_multiplier = -2. * (a12 * optimum[0] + a22 * optimum[1] - r2) / c1;
    let mid_multiplier = (-2. * (a11 * optimum[0] + a12 * optimum[1] - r1)
        - c1 * (-1f64).exp() * end_multiplier)
        / c1;
    assert!(a11 > 0. && a11 * a22 - a12 * a12 > 0. && end_multiplier >= 0. && mid_multiplier >= 0.);
    for (shooting, nodes) in [
        (ShootingMethod::Single, vec![]),
        (ShootingMethod::Multiple, vec![1.]),
    ] {
        let mut profile = request(&simulation, shooting, nodes, true);
        profile.path = vec![PathBound {
            output: c.outputs[x],
            lower: None,
            upper: Some(0.9),
        }];
        let problem = simulation.shooting(profile).unwrap();
        let budget = objective_budget(&problem);
        let bounds = problem.constraint_bounds().to_vec();
        let row_budgets = problem.tolerances().rows.clone();
        let path_budget = crate::workflow::tests::engineering_target(
            problem.numerics(),
            NumericalTarget::Row,
            pse_ids::named_id(c.outputs[x], "shooting.path.0"),
        )
        .budget;
        let continuity_count = row_budgets.len() - simulation.profile().samples.len();
        let state_scale = crate::workflow::tests::engineering_target(
            simulation.numerics(),
            NumericalTarget::Variable,
            c.states[x],
        )
        .coordinate_scale;
        let admitted_jumps = row_budgets[..continuity_count].iter().sum::<f64>() * state_scale;
        let result = solve(problem, None).await;
        let report = report_of(&result);
        assert!(
            result.usable(),
            "{:?}; complete {}; error {:?}; checks {:?}",
            result.assessments(),
            report.checks_complete,
            report.validation_error,
            report.checks
        );
        let trajectory = report.trajectory.as_ref().unwrap();
        assert_constraints(report, &bounds, &row_budgets);
        assert!(
            trajectory
                .samples
                .iter()
                .all(|s| s.outputs[x] <= 0.9 + path_budget)
        );
        assert!(
            trajectory
                .samples
                .iter()
                .any(|s| s.outputs[x] >= 0.9 - path_budget),
            "the bound is active"
        );
        let actual = original_objective(report.controls.values().next().unwrap());
        // Reintegrating the controls removes multiple shooting's admitted node
        // jumps. Their exact contractive state propagation can enlarge the
        // sampled path allowance of that independent continuous trajectory.
        let relaxed = 0.9 + path_budget + admitted_jumps;
        let lower = original_objective(&[relaxed / c1, relaxed]);
        assert!(
            actual >= lower - budget && actual <= original_objective(&optimum) + budget,
            "{actual} vs [{lower}, {}]",
            original_objective(&optimum)
        );
        assert!((report.objective.unwrap() - actual).abs() <= budget);
    }
}

/// The physical context of the authored shooting fixtures: Scalar and Time.
fn physical() -> crate::workflow::PhysicalContext {
    let mut physical = crate::workflow::tests::physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    physical
}
fn parse(
    text: &str,
) -> Result<Vec<pse_authoring::language::Declaration>, pse_authoring::AuthoringError> {
    pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
}
/// A `route integrated; procedure shooting` fixture declares its method, its controls (schedules held free) and
/// its integration controls; the package admission refuses each incomplete or misplaced
/// declaration before any integration.
#[tokio::test]
async fn shooting_fixture_needs_authored_controls() {
    let physical = physical();
    let runtime = crate::workflow::tests::runtime();
    let def = "def Root {domain t:Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); param u:Scalar=1; var x[i in t]:Scalar; eq rate[i in t]:d(x[i])/di==(u-x[i])/1{s}; eq initial:x[0{s}]==0;}";
    let allowance = pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
    let integrate = format!(
        "integrate samples(0{{s}},1{{s}}) relative({allowance}) normalized_absolute({allowance}) step(1e-4{{s}});"
    );
    let free = "schedule root.u at(0.5{s}) values(1, 1) free lower(0) upper(2);";
    let fixed = "schedule root.u at(0.5{s}) values(1, 1);";
    for (fixture, refusal) in [
        // No integration controls.
        (
            "route integrated; procedure shooting; shoot single;".to_owned(),
            "fixture procedure metadata disagrees with its temporal route",
        ),
        // No controls, or no method.
        (
            format!("route integrated; procedure shooting; {integrate} shoot single;"),
            "a shooting fixture declares its method",
        ),
        (
            format!("route integrated; procedure shooting; {integrate} shoot single; {fixed}"),
            "a shooting fixture declares its method",
        ),
        (
            format!("route integrated; procedure shooting; {integrate} {free}"),
            "a shooting fixture declares its method",
        ),
        // Multiple shooting needs its inner nodes; single shooting has none.
        (
            format!("route integrated; procedure shooting; {integrate} shoot multiple; {free}"),
            "a shooting fixture declares its method",
        ),
        (
            format!(
                "route integrated; procedure shooting; {integrate} shoot single nodes(0.5{{s}}); {free}"
            ),
            "a shooting fixture declares its method",
        ),
        // Only a shooting fixture holds a schedule free.
        (
            format!("route integrated; procedure integrate; {integrate} {free}"),
            "only a shooting fixture holds a schedule free",
        ),
    ] {
        let text = format!(
            "package p {{ {def} test shot fixture {{dof 0; {fixture}}} {{child root:Root=Root();}} }}"
        );
        let declarations = parse(&text).unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "shot")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, physical.clone())
            .await
            .unwrap();
        let error = package
            .declared_execution(
                root,
                compiler_profile(),
                solver(),
                Default::default(),
                Limits::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains(refusal), "{fixture}: {error}");
    }
}

/// ADR-0110 Outcome 5 from authored data (ADR-0119): a `route integrated; procedure shooting` fixture declares the
/// method and nodes and holds the control schedule free within its bounds; the model's
/// objective annotations name the integral cost and the terminal miss. The declared
/// shooting problem reaches the tracking problem's analytic optimum by single and multiple
/// shooting under the same production numerical policy,
/// and conformance solves the fixture and passes its model checks.
#[tokio::test]
async fn authored_shooting_fixture_solves() {
    let physical = physical();
    let def = "def Tracking { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param u: Scalar = 0.5; var x[i in t]: Scalar; var y[i in t]: Scalar; eq rate[i in t]: d(x[i])/di == (u - x[i])/1{s}; eq square[i in t]: y[i] == x[i]*x[i]; eq initial: x[0{s}] == 0; annotation start x(0); annotation start y(0); let cost: Scalar = integral(i in t | (x[i]-1)*(x[i]-1)/1{s}); annotation check cost(cost >= 0); let miss[i in t]: Scalar = (x[i]-1.5)*(x[i]-1.5); annotation report miss(\"terminal miss\"); annotation objective cost(minimize, weight = 1); annotation objective miss(minimize, weight = 1); }";
    let fixture = |name: &str, shoot: &str| {
        let allowance =
            pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
        format!(
            "test {name} fixture {{ dof 0; route integrated; procedure shooting; integrate samples(0{{s}}, 0.5{{s}}, 1{{s}}, 1.5{{s}}, 2{{s}}) relative({allowance}) normalized_absolute({allowance}) step(1e-4{{s}}) quadrature_relative({allowance}) quadrature_absolute(root.cost = {allowance}); schedule root.u at(1{{s}}) values(0.5, 0.5) free lower(0) upper(5); {shoot} }} {{ child root: Tracking = Tracking(); }}"
        )
    };
    let text = format!(
        "package p {{ {def} {} {} }}",
        fixture("single", "shoot single;"),
        fixture("multiple", "shoot multiple nodes(0.5{s}, 1{s}, 1.5{s});")
    );
    let rows = parse(&text).unwrap();
    let rendered = pse_authoring::language::render(&rows).unwrap();
    assert!(
        rendered.contains("schedule root.u at(1{s}) values(0.5, 0.5) free lower(0) upper(5);")
            && rendered.contains("shoot multiple nodes(0.5{s}, 1{s}, 1.5{s});"),
        "{rendered}"
    );
    assert_eq!(
        rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
        parse(&rendered)
            .unwrap()
            .iter()
            .map(|r| &r.value)
            .collect::<Vec<_>>()
    );
    let root = |name| rows.iter().find(|r| r.name == name).unwrap().declaration_id;
    let fixtures = [root("single"), root("multiple")];
    let package = crate::workflow::tests::runtime_with_workspace(32 << 20)
        .modeling_package(rows, physical)
        .await
        .unwrap();
    let cancel = crate::CancelSource::new();
    let (_, value) = analytic();
    for (fixture, nodes) in fixtures.into_iter().zip([0, 3]) {
        let refusal = package
            .declared_simulation(
                fixture,
                compiler_profile(),
                None,
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap_err();
        assert!(
            refusal
                .to_string()
                .contains("requires the authored integration procedure"),
            "{refusal}"
        );
        let problem = package
            .declared_shooting(
                fixture,
                compiler_profile(),
                None,
                solver(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(problem.controls(), 2);
        assert_eq!(problem.contract().variables.len(), 2 + nodes);
        let budget = objective_budget(&problem);
        let simulation = problem.simulation.clone();
        let bounds = problem.constraint_bounds().to_vec();
        let row_budgets = problem.tolerances().rows.clone();
        let result = solve(problem, None).await;
        let report = report_of(&result);
        assert!(
            result.usable(),
            "{:?}; complete {}; error {:?}; checks {:?}",
            result.assessments(),
            report.checks_complete,
            report.validation_error,
            report.checks
        );
        let solve = report.solve.as_ref().unwrap();
        assert_eq!(
            solve.termination.category,
            native::solve::Termination::Success,
            "{:?}",
            solve.termination
        );
        assert_original_objective(report, value, budget, &simulation);
        assert_constraints(report, &bounds, &row_budgets);
        assert!(report.checks_complete && report.checks.iter().all(|c| c.satisfied));
    }
    let policy = crate::workflow::ModelingConformancePolicy {
        compiler: compiler_profile(),
        solver: solver(),
        numerical: Default::default(),
        limits: Limits::default(),
        maximum_fixtures: 4,
        maximum_checks: 64,
        fixtures: Default::default(),
        diagnostics: None,
        derivatives: native::derivative_diagnostics::Policy {
            perturbation: 1e-6,
            relative_tolerance: pse_model::numerics::NumericalPolicy::default()
                .supplier_action_accuracy,
            maximum_cells: 100,
        },
    };
    let report = package.conform(policy, &cancel).await.unwrap();
    assert!(report.passed(), "{:?}", report.checks);
    assert_eq!(
        report
            .checks
            .iter()
            .filter(|c| c.kind == pse_model::generated::enums::ModelingConformanceKind::StartToSolve)
            .count(),
        2
    );
}
