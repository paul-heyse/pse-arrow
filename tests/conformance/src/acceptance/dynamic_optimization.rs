// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Simultaneous dynamic optimization of authored fixtures (Plan 22 Y5a): the seed
//! `SaturatedProcess` and `SwitchedProcess` definitions of `control_fixtures`, a first-order
//! lag `lag·dx/dt = u − x`, x(0) = 0, over 2 s, tracking a target with a bounded input. The
//! same `SaturatedProcess` definition also integrates with its input scheduled.
use super::fixtures::*;
use pse_backend_native::solve::{Backend, Qualification, SolutionStatus, SolverSelection};
use pse_ids::SemanticId;
use pse_relations::generated::enums::NumericalTarget;
use pse_runtime::{
    CancelSource,
    math::solves::{Outcome, SolverProfile},
    workflow::{ModelingPackage, ModelingResult, RunResult},
};

/// `test saturated_simultaneous`, `saturated_integrated` and `switched_simultaneous`.
const SATURATED: &str = "f33e2cf83c7e4f1b8df6da7f66212bd5";
const INTEGRATED: &str = "50b334c432114f34ade0f9e24ff460e5";
const SWITCHED: &str = "bbf77b04ed344205a780917531a693f4";
/// ∫₀² (x − 1)² dt at the saturated input u = 1, x = 1 − e^{−t}.
fn saturated_tracking() -> f64 {
    (1.0 - (-4.0f64).exp()) / 2.0
}
fn optimize(selection: SolverSelection) -> SolverProfile {
    let mut solver = profile(Backend::Ipopt, true);
    solver.selection = selection;
    solver.controls.time_limit = std::time::Duration::from_secs(300);
    solver
}
/// The holdup at the element boundaries 0.5 s and 1 s and at the horizon.
const HOLDUPS: [(f64, &str); 3] = [
    (0.5, "root.holdup[0.5{s}]"),
    (1.0, "root.holdup[1{s}]"),
    (2.0, "root.holdup[2{s}]"),
];
/// Solve an authored simultaneous fixture, with `target` replacing the authored target;
/// returns the run and the identities of the demanded `paths`.
async fn simultaneous(
    package: &ModelingPackage,
    fixture: &str,
    solver: SolverProfile,
    target: Option<f64>,
    paths: &[&str],
) -> (std::sync::Arc<RunResult>, Vec<SemanticId>) {
    let cancel = CancelSource::new();
    let mut analysis = package
        .declared_execution(
            SemanticId::parse_hex(fixture).unwrap().into(),
            compiler(),
            solver,
            Default::default(),
            seed_limits(),
            &cancel,
        )
        .await
        .unwrap()
        .analysis;
    if let Some(target) = target {
        analysis.case.values.insert("root.target".into(), target);
    }
    analysis
        .bindings
        .demand
        .extend(paths.iter().map(|p| (*p).to_owned()));
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let known = &prepared.model.model.compiled().model.paths;
    let ids = paths
        .iter()
        .map(|p| *known.get(*p).unwrap_or_else(|| panic!("{p}")))
        .collect();
    (prepared.start().unwrap().wait().await.unwrap(), ids)
}
fn tracking(result: &ModelingResult) -> f64 {
    result
        .reports
        .iter()
        .find(|r| r.label == "tracking")
        .unwrap()
        .value
}
/// Every solved value of the indexed member `name`, one per mesh coordinate.
fn values(result: &ModelingResult, name: &str) -> Vec<(SemanticId, f64)> {
    let model = &result.prepared.model.model.compiled().model;
    let suffix = format!("root.{name}");
    let found = result
        .values
        .scalars
        .iter()
        .filter(|(id, _)| {
            model
                .symbols
                .get(id)
                .is_some_and(|s| s.lineage.path.ends_with(&suffix))
        })
        .map(|(id, v)| (*id, *v))
        .collect::<Vec<_>>();
    assert!(!found.is_empty(), "no member {name}");
    found
}
/// Free variables less equations, as the fixture declares them.
fn degrees_of_freedom(result: &ModelingResult) -> usize {
    let facts = &result.prepared.model.case.compiled().facts;
    facts.variables - facts.rows
}
fn native(result: &ModelingResult) -> &pse_backend_native::solve::SolveReport {
    match &result.outcome {
        Outcome::Native(native) => native,
        other => panic!("{other:?}"),
    }
}
/// Qualify the actual candidate against every original physical row and bound.
/// The tracking objective does not specify a unique vector of collocation inputs.
fn physical_premises(result: &ModelingResult) {
    let native = native(result);
    assert!(native.quality.as_ref().unwrap().feasible());
    let observation = native.observation.as_ref().unwrap();
    let plan = &result.prepared.model.case.compiled().plan;
    let numerics = result.prepared.solve.numerics();
    let rows = plan.structure().rows();
    assert_eq!(observation.values.len(), rows.len());
    assert_eq!(observation.bounds.len(), rows.len());
    for ((row, value), (lower, upper)) in rows
        .iter()
        .zip(&observation.values)
        .zip(&observation.bounds)
    {
        let allowance = resolved_allowance(numerics, NumericalTarget::Row, row.id);
        assert!(
            value.is_finite() && *value >= lower - allowance && *value <= upper + allowance,
            "original row {}: {value} outside ({lower}, {upper}) with allowance {allowance}",
            row.id
        );
    }
    for variable in plan.structure().variables() {
        let id = variable.port.id;
        let value = result.values.scalars[&id];
        let allowance = if variable.fixed {
            assert_eq!(value, result.prepared.model.values.scalars[&id]);
            0.0
        } else {
            resolved_allowance(numerics, NumericalTarget::Variable, id)
        };
        assert!(value.is_finite());
        assert!(
            variable
                .lower
                .is_none_or(|lower| value >= lower - allowance)
        );
        assert!(
            variable
                .upper
                .is_none_or(|upper| value <= upper + allowance)
        );
    }
}

/// The finite, continuous-variable QP's evidence must meet the admitted policy.
/// These normalized stationarity and objective-error budgets qualify the tracking
/// decision; physical variable allowances do not bound distance to the exact optimizer.
fn optimality_premises(result: &ModelingResult) {
    let native = native(result);
    assert_eq!(native.backend, Backend::Highs);
    assert_eq!(native.qualification, Qualification::OptimalWithinTolerance);
    let evidence = native.evidence.coefficient.as_ref().unwrap();
    let policy = result.prepared.solve.accuracy();
    assert!(evidence.upload_equivalent && !evidence.discrete);
    assert_eq!(evidence.primal, SolutionStatus::Feasible);
    assert_eq!(evidence.dual, SolutionStatus::Feasible);
    assert!(
        evidence.max_dual_infeasibility.is_some_and(|value| {
            value.is_finite() && value >= 0.0 && value <= policy.stationarity
        }),
        "{evidence:?}; stationarity budget={}",
        policy.stationarity
    );
    assert!(
        evidence.primal_dual_objective_error.is_some_and(|value| {
            value.is_finite() && value >= 0.0 && value <= policy.gap_relative
        }),
        "{evidence:?}; objective-error budget={}",
        policy.gap_relative
    );
}

/// Independently evaluate the original finite program's c + gᵀx + ½xᵀHx.
/// Demand the existing compiler coefficient artifact under its retained class budget;
/// native readback/optimality evidence is checked separately for each actual backend.
fn discrete_objective_premises(result: &ModelingResult) {
    let native = native(result);
    assert!(native.validation_failure().is_none());
    // Route-specific class products are separate from the shared compiled case.
    // Use the same production owner to obtain the original semantic coefficients,
    // including for SCIP's factorable export and fixed-assignment re-solve.
    let prepared = result
        .prepared
        .model
        .case
        .compiled()
        .prepare_class(
            &result.prepared.model.values,
            &std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        )
        .unwrap();
    let coefficients = prepared
        .coefficients
        .as_ref()
        .expect("tracking QP coefficients");
    assert!(coefficients.matches_values(&result.prepared.model.values));
    let columns = prepared.plan.columns();
    assert_eq!(native.variables.as_slice(), columns);
    let candidate = native.candidate.as_ref().unwrap();
    assert_eq!(candidate.primal.len(), columns.len());
    for (id, value) in columns.iter().zip(&candidate.primal) {
        assert_eq!(
            result.values.scalars[id], *value,
            "full candidate publication {id}"
        );
    }
    let point = &candidate.primal;
    assert_eq!(coefficients.objective.len(), point.len());
    let mut objective = coefficients.objective_constant;
    for (linear, value) in coefficients.objective.iter().zip(point) {
        objective += linear * value;
    }
    let hessian = &coefficients.hessian;
    assert_eq!(hessian.nrows(), point.len());
    assert_eq!(hessian.ncols(), point.len());
    for (column, range) in hessian.symbolic().col_ptr().windows(2).enumerate() {
        for entry in range[0]..range[1] {
            objective +=
                0.5 * point[hessian.row_idx()[entry]] * hessian.val()[entry] * point[column];
        }
    }
    let allowance = result.prepared.solve.objective_accuracy();
    assert!(objective.is_finite() && allowance.is_finite() && allowance > 0.0);
    if let Some(evidence) = native.evidence.coefficient.as_ref() {
        let objective_scale = result
            .prepared
            .solve
            .numerics()
            .targets
            .iter()
            .find(|target| target.kind == NumericalTarget::Objective)
            .unwrap()
            .coordinate_scale;
        near(
            evidence.objective.unwrap() * objective_scale,
            objective,
            allowance,
        );
    }
    near(candidate.objective.unwrap(), objective, allowance);
    near(
        native.observation.as_ref().unwrap().objective.unwrap(),
        objective,
        allowance,
    );
    near(tracking(result), objective, allowance);
}

/// SCIP's original-unit dual bound qualifies the actual finite-program candidate.
/// The relative branch uses the smaller same-sign magnitude, as production does.
fn global_optimality_premises(result: &ModelingResult) {
    use pse_backend_native::solve::{Assurance, PrimalSource, Termination};
    let native = native(result);
    let evidence = native.evidence.global.as_ref().unwrap();
    assert!(evidence.readback && !evidence.infeasible, "{evidence:?}");
    assert_eq!(evidence.primal, PrimalSource::FixedAssignment);
    assert_eq!(native.termination.category, Termination::Success);
    assert!(matches!(
        native.termination.assurance,
        Assurance::GlobalBound | Assurance::ExactCertificate
    ));
    let policy = result.prepared.solve.accuracy();
    let objective_scale = result
        .prepared
        .solve
        .numerics()
        .targets
        .iter()
        .find(|target| target.kind == NumericalTarget::Objective)
        .unwrap()
        .coordinate_scale;
    assert_eq!(
        evidence.gap_absolute,
        policy.mip_absolute_gap * objective_scale
    );
    assert_eq!(evidence.gap_relative, policy.mip_relative_gap);
    let objective = native.observation.as_ref().unwrap().objective.unwrap();
    let bound = evidence.dual_bound.unwrap();
    assert!(objective.is_finite() && bound.is_finite());
    let relative = if objective.signum() == bound.signum() {
        evidence.gap_relative * objective.abs().min(bound.abs())
    } else {
        0.0
    };
    assert!(
        (objective - bound).abs() <= evidence.gap_absolute.max(relative),
        "original objective={objective}, bound={bound}; {evidence:?}"
    );
    assert_eq!(
        evidence.sense,
        result
            .prepared
            .model
            .case
            .compiled()
            .plan
            .structure()
            .objective()
            .unwrap()
            .sense
    );
}

/// The authored eight-element, order-three Radau program is qualified against its
/// actual coefficients and original physical constraints. Its finite state/input
/// parameterization and quadrature do not promise the continuous-control optimum,
/// including the off-mesh switch at t = ln 2 for target 0.5. Integrating the same
/// definition with the saturated input scheduled independently checks the continuous
/// analytic trajectory and integral at the declared integration controls.
#[tokio::test]
async fn simultaneous_dynamic_optimization_matches_analytic() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let paths = HOLDUPS.map(|(_, p)| p);
    let (saturated, _holdups) = simultaneous(
        &package,
        SATURATED,
        optimize(SolverSelection::Auto),
        None,
        &paths,
    )
    .await;
    let result = authored_success(&saturated);
    // A linear lag with a quadratic tracking integral is a convex QP; the fixture's
    // declared degrees of freedom are its free inputs.
    assert_eq!(native(result).backend, Backend::Highs);
    assert_eq!(degrees_of_freedom(result), 24);
    let objective_allowance = resolved_allowance(
        result.prepared.solve.numerics(),
        NumericalTarget::Objective,
        SemanticId::NIL,
    );
    physical_premises(result);
    optimality_premises(result);
    discrete_objective_premises(result);
    let inputs = values(result, "u");
    // One input per mesh node, the fixed left boundary included.
    assert_eq!(inputs.len(), 25);
    for (id, u) in &inputs {
        let variable = result
            .prepared
            .model
            .case
            .compiled()
            .plan
            .structure()
            .variables()
            .iter()
            .find(|variable| variable.port.id == *id)
            .unwrap();
        if variable.fixed {
            assert_eq!(*u, 1., "authored fixed left input");
        }
    }
    // Track half the reachable range in the same finite control parameterization.
    let (half, _holdups) = simultaneous(
        &package,
        SATURATED,
        optimize(SolverSelection::Auto),
        Some(0.5),
        &paths,
    )
    .await;
    let half = authored_success(&half);
    physical_premises(half);
    optimality_premises(half);
    discrete_objective_premises(half);
    assert_eq!(values(half, "u").len(), 25);

    // Integrated, with the input scheduled in two intervals at its saturated optimum.
    let cancel = CancelSource::new();
    let integrated = SemanticId::parse_hex(INTEGRATED).unwrap().into();
    let declared = package
        .declared_simulation(integrated, compiler(), None, seed_limits(), &cancel)
        .await
        .unwrap();
    let contract = declared.contract();
    let symbols = &declared.model().compiled().model.symbols;
    let input = contract
        .parameters
        .iter()
        .position(|id| symbols[id].lineage.path.contains(".u"))
        .unwrap();
    let mut schedule = declared.profile().clone();
    schedule
        .schedule
        .push(pse_backend_native::dynamics::ScheduledInput {
            parameter: input,
            times: vec![std::f64::consts::LN_2],
        });
    let simulation = package
        .declared_simulation(
            integrated,
            compiler(),
            Some(schedule),
            seed_limits(),
            &cancel,
        )
        .await
        .unwrap();
    let trajectory = simulation.run(&cancel).await.unwrap();
    assert_eq!(
        trajectory.report().termination,
        pse_backend_native::dynamics::Termination::Completed,
        "{:?}",
        trajectory.report().error
    );
    let state = simulation.contract().states[0];
    let output = simulation
        .contract()
        .outputs
        .iter()
        .position(|id| *id == state);
    let integrated_allowance =
        resolved_allowance(simulation.numerics(), NumericalTarget::Variable, state);
    assert_eq!(trajectory.report().samples.len(), HOLDUPS.len());
    for (sample, (time, _)) in trajectory.report().samples.iter().zip(HOLDUPS) {
        assert_eq!(sample.time, time);
        let integrated = output.map_or(sample.state[0], |o| sample.outputs[o]);
        near(integrated, 1.0 - (-sample.time).exp(), integrated_allowance);
    }
    near(
        trajectory.report().samples.last().unwrap().integrals[0],
        saturated_tracking(),
        objective_allowance,
    );
}

/// An on/off input at every collocation node makes the problem a mixed-integer program
/// that SCIP certifies. Its candidate is the continuous re-solve under the node
/// assignment, which the step states as its commitment (Plan 22 M2c). Its original
/// tracking objective must satisfy the retained bound for that finite program.
#[tokio::test]
async fn simultaneous_dynamic_optimization_with_discrete_decision() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let (run, _) = simultaneous(
        &package,
        SWITCHED,
        optimize(SolverSelection::Auto),
        None,
        &[],
    )
    .await;
    let result = authored_success(&run);
    assert_eq!(degrees_of_freedom(result), 8);
    let native = native(result);
    assert_eq!(native.backend, Backend::Scip);
    assert!(
        matches!(
            native.qualification,
            Qualification::GapQualified | Qualification::OptimalWithinTolerance
        ),
        "{:?}",
        native.qualification
    );
    physical_premises(result);
    discrete_objective_premises(result);
    global_optimality_premises(result);
    // The switched inputs are the committed assignment: binary, and both on and off.
    let commitment = native
        .candidate
        .as_ref()
        .unwrap()
        .commitment
        .as_ref()
        .unwrap_or_else(|| {
            let resolve = native
                .metrics
                .iter()
                .filter(|(key, _)| key.starts_with("resolve."))
                .collect::<Vec<_>>();
            panic!("a mixed-integer candidate states its commitment; re-solve: {resolve:?}")
        });
    assert_eq!(commitment.columns.len(), 8);
    assert!(
        commitment
            .columns
            .iter()
            .all(|(_, (l, u))| l == u && (*l == 0.0 || *l == 1.0))
    );
    assert!(commitment.columns.iter().any(|(_, (v, _))| *v == 0.0));
    assert!(commitment.columns.iter().any(|(_, (v, _))| *v == 1.0));
    let on = values(result, "on");
    // Every switch but the one fixed at the start is committed at its solved value.
    assert_eq!(on.len(), 9);
    for (id, (committed, _)) in &commitment.columns {
        let solved = on.iter().find(|(o, _)| o == id).unwrap().1;
        assert_eq!(solved, *committed);
    }
    let runs = pse_relations::generated::runtime::solve_runs::View::from_checked(
        &run.table("runtime.solve_runs").unwrap(),
    )
    .unwrap()
    .rows()
    .unwrap();
    assert_eq!(runs[0].commitment.as_ref().map(Vec::len), Some(8));
}
