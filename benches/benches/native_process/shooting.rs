// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One complete IDAS shooting operation with the native tracking journey's original oracle.
use super::*;
use pse_backend_native::{self as native, solve::HessianMode};
use pse_ids::{SemanticId, named_id};
use pse_model::generated::enums::NumericalTarget;
use pse_relations::columnar::RelationRow;
use pse_runtime::workflow;
use std::sync::Arc;

/// Independent normal equations for the piecewise-constant tracking controls.
fn quadratic() -> ([f64; 3], [f64; 2]) {
    let (e1, e2) = ((-1f64).exp(), (-2f64).exp());
    let c = 1. - e1;
    let square = 1. - 2. * c + (1. - e2) / 2.;
    let (ta, tb) = (c * e1, 1. - e1);
    (
        [
            square + c * c * (1. - e2) / 2. + ta * ta,
            c * (c - (1. - e2) / 2.) + ta * tb,
            square + tb * tb,
        ],
        [e1 + c * c + 1.5 * ta, e1 + 1.5 * tb],
    )
}
fn original_objective(u: &[f64]) -> f64 {
    assert_eq!(u.len(), 2);
    let ([a11, a12, a22], [r1, r2]) = quadratic();
    4.25 - 2. * (r1 * u[0] + r2 * u[1])
        + a11 * u[0] * u[0]
        + 2. * a12 * u[0] * u[1]
        + a22 * u[1] * u[1]
}
fn optimum() -> f64 {
    let ([a11, a12, a22], [r1, r2]) = quadratic();
    let det = a11 * a22 - a12 * a12;
    let u = [(r1 * a22 - r2 * a12) / det, (a11 * r2 - a12 * r1) / det];
    assert!(u.iter().all(|u| *u > 0. && *u < 5.));
    original_objective(&u)
}
pub(super) async fn run(owner: &WorkflowRuntime, observations: &mut observations::Observations) {
    let physical = physical(owner).await;
    let rows = pse_authoring::language::parse(
        "package tracking { def Tracking { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param u: Scalar = 0.5; var x[i in t]: Scalar; var y[i in t]: Scalar; eq rate[i in t]: d(x[i])/di == (u - x[i])/1{s}; eq square[i in t]: y[i] == x[i]*x[i]; eq initial: x[0{s}] == 0; annotation start x(0); annotation start y(0); let cost: Scalar = integral(i in t | (x[i]-1)*(x[i]-1)/1{s}); annotation check cost(cost >= 0); let miss[i in t]: Scalar = (x[i]-1.5)*(x[i]-1.5); annotation report miss(\"terminal miss\"); } }",
        named_id(SemanticId::NIL, "process-cost.shooting"),
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    ).unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Tracking")
        .unwrap()
        .declaration_id;
    let package = runtime(owner)
        .modeling_package(rows, physical.clone())
        .await
        .unwrap();
    let allowance = pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
    let integration = native::dynamics::Profile {
        method: native::dynamics::Method::Idas,
        end: 2.,
        samples: vec![0., 0.5, 1., 1.5, 2.],
        rtol: allowance,
        atol: vec![allowance; 2],
        out_rtol: Some(allowance),
        out_atol: vec![allowance],
        initial_step: 1e-4,
        parameter_scales: vec![1.],
        schedule: vec![native::dynamics::ScheduledInput {
            parameter: 0,
            times: vec![1.],
        }],
        ..Default::default()
    };
    let simulation = package
        .prepare_simulation(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            seed_limits(),
            Default::default(),
            compiler(),
            integration,
            pse_kernels::DerivativeOrder::First,
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let contract = simulation.contract();
    assert_eq!(contract.states.len(), 2);
    assert_eq!(contract.parameters.len(), 1);
    assert_eq!(contract.outputs.len(), 3);
    assert_eq!(contract.quadratures.len(), 1);
    assert_eq!(contract.differential.iter().filter(|d| **d).count(), 1);
    let state_outputs = contract
        .states
        .iter()
        .map(|state| pse_compiler::workspace::ModelingOutput::Member(*state).row_id())
        .collect::<Vec<_>>();
    assert_eq!(&contract.outputs[..2], state_outputs.as_slice());
    let x = contract.differential.iter().position(|d| *d).unwrap();
    let mut solver = profile(Backend::Ipopt, true);
    solver.controls.hessian = HessianMode::LimitedMemory;
    solver.controls.threads = 1;
    solver.presolve = native::presolve::Policy::Off;
    // Shooting's numerical-policy accessor is private. Resolve this independent
    // dimensionless decision budget through the same public physical-policy owner.
    let quantity = physical.quantities().neutral_dimensionless().unwrap();
    let unit = physical
        .quantities()
        .quantity_type(quantity)
        .unwrap()
        .canonical_unit;
    let objective_numerics = pse_math::numerics::resolve(
        physical.quantities(),
        physical.preconditions(),
        &[pse_math::numerics::TargetSpec {
            id: SemanticId::NIL,
            kind: NumericalTarget::Objective,
            quantity,
            unit,
            integer: false,
            declared_tolerance: None,
        }],
        &[],
        &solver.numerics,
    )
    .unwrap();
    let objective_budget = objective_numerics
        .targets
        .iter()
        .find(|target| target.kind == NumericalTarget::Objective && target.id == SemanticId::NIL)
        .unwrap()
        .budget;
    assert!(objective_budget.is_finite() && objective_budget > 0.);
    let problem = simulation
        .shooting(workflow::ShootingProfile {
            method: workflow::ShootingMethod::Single,
            nodes: vec![],
            controls: vec![workflow::ShootingControl {
                input: contract.parameters[0],
                lower: Some(0.),
                upper: Some(5.),
            }],
            path: vec![],
            objective: workflow::ShootingObjective {
                terminal: BTreeMap::from([(contract.outputs[2], 1.)]),
                integral: BTreeMap::from([(contract.quadratures[0], 1.)]),
            },
            solver,
        })
        .unwrap();
    assert_eq!(problem.controls(), 2);
    assert_eq!(problem.contract().variables.len(), 2);
    let control_numerics = pse_math::numerics::resolve(
        physical.quantities(),
        physical.preconditions(),
        &problem
            .contract()
            .variables
            .iter()
            .map(|variable| pse_math::numerics::TargetSpec {
                id: variable.id,
                kind: NumericalTarget::Variable,
                quantity,
                unit,
                integer: false,
                declared_tolerance: None,
            })
            .collect::<Vec<_>>(),
        &[],
        &objective_numerics.policy,
    )
    .unwrap();
    let result = Arc::new(problem).start().unwrap().wait().await.unwrap();
    assert!(result.usable(), "{:?}", result.assessments());
    let RunReport::Shooting(report) = result.report().unwrap() else {
        panic!("shooting report expected")
    };
    let solve = report.solve.as_ref().unwrap();
    assert_eq!(solve.termination.category, Termination::Success);
    assert!(report.quality.as_ref().unwrap().feasible());
    // Single shooting has no continuity or path rows; control bounds remain original.
    assert!(report.constraint_values.as_ref().unwrap().is_empty());
    assert_eq!(report.nodes.len(), 1);
    assert!(
        report.checks_complete
            && !report.checks.is_empty()
            && report.checks.iter().all(|check| check.satisfied)
    );
    assert!(report.validation_error.is_none());
    let controls = report.controls.values().next().unwrap();
    assert_eq!(report.controls.len(), 1);
    assert_eq!(controls.len(), 2);
    assert_eq!(report.candidate.as_ref().unwrap(), controls);
    for (value, target) in controls.iter().zip(&control_numerics.targets) {
        assert!(value.is_finite() && *value >= -target.budget && *value <= 5. + target.budget);
    }
    let original = original_objective(controls);
    assert!(original.is_finite());
    assert!((original - optimum()).abs() <= objective_budget);
    let trajectory = report.trajectory.as_ref().unwrap();
    assert_eq!(trajectory.requested_initial.len(), contract.states.len());
    assert_eq!(trajectory.samples.len(), simulation.profile().samples.len());
    assert_eq!(trajectory.completed_time, simulation.profile().end);
    for (sample, time) in trajectory.samples.iter().zip(&simulation.profile().samples) {
        assert_eq!(sample.time, *time);
        assert_eq!(sample.outputs.len(), contract.outputs.len());
        assert_eq!(sample.integrals.len(), contract.quadratures.len());
        assert!(
            sample
                .outputs
                .iter()
                .chain(&sample.integrals)
                .all(|value| value.is_finite())
        );
    }
    let last = trajectory.samples.last().unwrap();
    assert_eq!(last.time, simulation.profile().end);
    assert_eq!(last.integrals.len(), contract.quadratures.len());
    let terminal = (last.outputs[x] - 1.5).powi(2);
    // These are the original journey's publication identities. The local
    // integration controls do not promise all-sample forward accuracy under
    // Variable budgets, nor assembled objective accuracy under the NLP budget.
    let roundoff = 16. * f64::EPSILON * terminal.abs().max(last.outputs[2].abs()).max(1.);
    assert!((last.outputs[2] - terminal).abs() <= roundoff);
    let published = last.integrals[0] + last.outputs[2];
    let composition_roundoff =
        16. * f64::EPSILON * (last.integrals[0].abs() + last.outputs[2].abs()).max(1.);
    assert!((report.objective.unwrap() - published).abs() <= composition_roundoff);
    let header = result.table("runtime.computation_runs").unwrap();
    let view =
        pse_relations::generated::runtime::computation_runs::View::from_checked(&header).unwrap();
    assert_eq!(
        view.row(0).unwrap().kind,
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
    let trace = report.strategy.as_ref().unwrap();
    let rows = pse_relations::generated::runtime::solve_strategy_events::Row::rows(
        &result.table("runtime.solve_strategy_events").unwrap(),
    )
    .unwrap();
    assert!(
        rows.iter()
            .zip(
                trace
                    .rows(
                        result.run_id,
                        0,
                        owner.runtime.pool(),
                        0..trace.event_count()
                    )
                    .unwrap()
            )
            .all(|(row, projected)| row == &projected.unwrap())
    );
    assert_eq!(rows.len(), trace.event_count());
    assert!(rows.iter().any(|row| row.charging_owner.is_some()));
    assert!(rows.iter().all(|row| row.decision_identity.is_some()));
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
    observations.rows(&rows);
    observations.native(solve);
    observations.trajectory(trajectory);
    std::hint::black_box((result.assessments(), controls, trajectory));
}
