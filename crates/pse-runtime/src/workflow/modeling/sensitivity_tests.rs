// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Parametric sensitivities of an authored case through the real pipeline (Plan 22 S1):
//! the request travels in the solver profile, the view's parametric program is prepared
//! and cached with it, and the results writer publishes every requested quantity with its
//! `LocalValidity` row.
use super::*;
use crate::math::{
    settings::SensitivityRequest,
    solves::{NumericalInputs, SolverProfile},
};
use crate::workflow::tests as fixture;
use pse_backend_native::solve::{Backend, SolveIntent, SolverSelection};
use pse_compiler::workspace::ModelingCaseBindings;
use pse_kernels::DerivativeOrder;
use pse_relations::{
    columnar::RelationRow,
    generated::{
        enums::{DerivedQuantity, DualQualification, NumericalTarget, WithheldReason},
        runtime::{
            local_validity, parametric_sensitivities, reduced_hessians, resolved_numerics,
            solve_constraints, solve_variables,
        },
    },
};

/// `min ½x² + y² − b·x + ½z² + a·z  s.t.  x + y = a`, z ∈ [0, 10], at (a, b) = (1, 2):
/// x = (b + 2a)/3, y = (a − b)/3, z = 0 with its bound multiplier a, and
/// d²f*/d(a, b)² = [[2/3, −2/3], [−2/3, −1/3]].
pub(in crate::workflow) const QUADRATIC: &str = "package p {
    def Root {
      param a: Scalar = 1; param b: Scalar = 2;
      var x: Scalar; var y: Scalar; var z: Scalar;
      eq coupling: x + y == a;
      let cost: Scalar = 0.5*x*x + y*y - b*x + 0.5*z*z + a*z;
      annotation objective cost(minimize);
      annotation bounds z(0, 10);
      annotation start x(0.5); annotation start y(0.5); annotation start z(0.5); } }";
/// `min b·x  s.t.  x >= a`: a linear program the coefficient route solves.
pub(in crate::workflow) const LINEAR: &str = "package p {
    def Root {
      param a: Scalar = 1; param b: Scalar = 2;
      var x: Scalar;
      eq floor: x >= a;
      let cost: Scalar = b*x;
      annotation objective cost(minimize);
      annotation bounds x(-10, 10);
      annotation start x(0); } }";

pub(in crate::workflow) async fn package(text: &str) -> (ModelingPackage, DeclarationId) {
    let physical = fixture::physical();
    let rows = pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = fixture::runtime_with(16 << 20, 1 << 20, 1 << 30)
        .modeling_package(rows, physical)
        .await
        .unwrap();
    (package, root)
}
pub(in crate::workflow) fn analysis(
    root: DeclarationId,
    selection: SolverSelection,
) -> ModelingAnalysis {
    let mut solver: SolverProfile = fixture::profile();
    solver.intent = SolveIntent::Optimize;
    solver.selection = selection;
    ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: ModelingCaseBindings::default(),
        order: DerivativeOrder::Second,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    }
}
/// Solve `text` with a sensitivity request over its parameters `a` and `b`; returns the
/// result and the identities of `a`, `b` and `x`.
pub(in crate::workflow) async fn solve(
    text: &str,
    selection: SolverSelection,
) -> (Arc<crate::workflow::RunResult>, [SemanticId; 3]) {
    let (result, ids) = solve_propagating(text, selection, &["x"], |_, _, _| None).await;
    (result, [ids[0], ids[1], ids[2]])
}
/// Solve `text` with a sensitivity request over its parameters `a` and `b` and the
/// propagation `propagation` builds from the identities of `a`, `b` and `outputs`;
/// returns the result and those identities.
pub(in crate::workflow) async fn solve_propagating(
    text: &str,
    selection: SolverSelection,
    outputs: &[&str],
    propagation: impl FnOnce(
        &[SemanticId],
        &[SemanticId],
        &[pse_kernels::Port],
    ) -> Option<crate::math::settings::Propagation>,
) -> (Arc<crate::workflow::RunResult>, Vec<SemanticId>) {
    let (package, root) = package(text).await;
    let cancel = crate::CancelSource::new();
    let mut analysis = analysis(root, selection);
    analysis.bindings.demand = ["a", "b"]
        .iter()
        .chain(outputs)
        .map(|p| (*p).into())
        .collect();
    let plain = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let paths = &plain.model.model.compiled().model.paths;
    let ids: Vec<SemanticId> = ["a", "b"]
        .iter()
        .chain(outputs)
        .map(|p| paths[*p])
        .collect();
    analysis.solver.sensitivity = Some(SensitivityRequest {
        parameters: vec![ids[0], ids[1]],
        reduced_hessian: true,
        propagation: propagation(
            &ids[..2],
            &ids[2..],
            plain.model.case.compiled().plan.structure().parameters(),
        ),
    });
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    (result, ids)
}
pub(in crate::workflow) fn rows<R: RelationRow>(
    result: &crate::workflow::RunResult,
    name: &str,
) -> Vec<R> {
    R::rows(&result.table(name).unwrap()).unwrap()
}

#[tokio::test]
async fn sensitivities_published_with_local_validity() {
    let (result, ids) = solve_propagating(
        QUADRATIC,
        SolverSelection::Explicit(Backend::Ipopt),
        &["x", "z"],
        |_, _, _| None,
    )
    .await;
    crate::workflow::review_qualification_tests::assert_review_transport(
        &result,
        &[
            ("/steps/0/constructor", "replay-case-constructor.v2"),
            ("/steps/0/context", "scientific-coordinates.v1"),
            ("/steps/0/runtime", "native-execution-snapshot.v1"),
        ],
    )
    .await;
    let (a, b, x, z) = (ids[0], ids[1], ids[2], ids[3]);
    let policy = fixture::profile().numerics;
    let numerics: Vec<resolved_numerics::Row> = rows(&result, "runtime.resolved_numerics");
    let coordinate_scale = |kind, id| {
        numerics
            .iter()
            .find(|row| row.target_kind == kind && row.target_id == id)
            .unwrap()
            .coordinate_scale
    };
    let engineering_allowance = |kind, id| {
        numerics
            .iter()
            .find(|row| row.target_kind == kind && row.target_id == id)
            .unwrap()
            .engineering
            .as_ref()
            .unwrap()
            .budget
    };
    let parameter_scales = [
        coordinate_scale(NumericalTarget::Variable, a),
        coordinate_scale(NumericalTarget::Variable, b),
    ];
    let variable_scale = coordinate_scale(NumericalTarget::Variable, x);
    let objective_scale = coordinate_scale(NumericalTarget::Objective, SemanticId::NIL);
    let variable_allowance = engineering_allowance(NumericalTarget::Variable, x) / variable_scale;
    let objective_allowance =
        engineering_allowance(NumericalTarget::Objective, SemanticId::NIL) / objective_scale;
    // Compare responses to one frozen normalized parameter step against the
    // published physical allowance. The analytic answer never selects the budget.
    let close = |actual: Option<f64>, expected: f64, scale: f64, allowance: f64| {
        actual.is_some_and(|value| {
            value.is_finite() && ((value - expected) * scale).abs() <= allowance
        })
    };
    // One validity row per requested quantity, both certified at a point with independent
    // active gradients, strict complementarity and second-order sufficiency.
    let validity: Vec<local_validity::Row> = rows(&result, "runtime.local_validity");
    assert_eq!(validity.len(), 2, "{validity:?}");
    for quantity in [
        DerivedQuantity::ParametricSensitivity,
        DerivedQuantity::ReducedHessian,
    ] {
        let row = validity.iter().find(|r| r.quantity == quantity).unwrap();
        let v = &row.validity;
        assert!(v.certified && v.reason.is_none(), "{v:?}");
        assert!(!v.conditional);
        assert_eq!(
            (
                v.licq,
                v.strict_complementarity,
                v.second_order,
                v.weakly_active
            ),
            (Some(true), Some(true), Some(true), Some(0))
        );
        // The factor's backward error is an observed diagnostic, not an extra
        // optimization stopping or sensitivity-certification requirement.
        assert!(
            v.residual.is_some_and(|r| r.is_finite() && r >= 0.0),
            "{v:?}"
        );
    }
    let variables: Vec<solve_variables::Row> = rows(&result, "runtime.solve_variables");
    let value = |id| {
        variables
            .iter()
            .find(|row| row.symbol_id == id)
            .unwrap()
            .value
            .unwrap()
    };
    let constraints: Vec<solve_constraints::Row> = rows(&result, "runtime.solve_constraints");
    assert_eq!(constraints.len(), 1);
    let coupling = &constraints[0];
    assert!(coupling.equality_residual.unwrap().abs() <= coupling.tolerance.unwrap());
    // The envelope derivative is evaluated at the actual qualified candidate,
    // whose active-bound slack need not equal zero at the production tolerance.
    let objective_expected = [value(z) - coupling.dual.unwrap(), -value(x)];
    // The primal sensitivity and Hessian of this quadratic are constant over the
    // qualified active set, independently of the candidate's remaining solve error.
    let sensitivities: Vec<parametric_sensitivities::Row> =
        rows(&result, "runtime.parametric_sensitivities");
    let find = |parameter, kind, target| {
        sensitivities
            .iter()
            .find(|r| r.parameter_id == parameter && r.target_kind == kind && r.target_id == target)
            .unwrap()
    };
    assert!(close(
        find(a, NumericalTarget::Variable, x).primal,
        2.0 / 3.0,
        parameter_scales[0] / variable_scale,
        variable_allowance,
    ));
    assert!(close(
        find(b, NumericalTarget::Variable, x).primal,
        1.0 / 3.0,
        parameter_scales[1] / variable_scale,
        variable_allowance,
    ));
    let objective = find(a, NumericalTarget::Objective, SemanticId::NIL);
    assert!(
        close(
            objective.primal,
            objective_expected[0],
            parameter_scales[0] / objective_scale,
            objective_allowance
        ) && objective.dual.is_none()
    );
    assert!(close(
        find(b, NumericalTarget::Objective, SemanticId::NIL).primal,
        objective_expected[1],
        parameter_scales[1] / objective_scale,
        objective_allowance,
    ));
    // The coupling row's multiplier λ = 2(b − a)/3 moves by (−2/3, 2/3).
    let row = sensitivities
        .iter()
        .find(|r| r.parameter_id == a && r.target_kind == NumericalTarget::Row)
        .unwrap();
    assert_eq!(row.target_id, coupling.row_id);
    assert!(
        row.primal.is_none()
            && close(
                row.dual,
                -2.0 / 3.0,
                coordinate_scale(NumericalTarget::Row, coupling.row_id) * parameter_scales[0]
                    / objective_scale,
                policy.kkt.stationarity
            ),
        "{row:?}"
    );
    let hessians: Vec<reduced_hessians::Row> = rows(&result, "runtime.reduced_hessians");
    assert_eq!(hessians.len(), 1);
    assert_eq!(hessians[0].parameters, vec![a, b]);
    for (index, (actual, expected)) in hessians[0]
        .values
        .iter()
        .zip([2.0 / 3.0, -2.0 / 3.0, -2.0 / 3.0, -1.0 / 3.0])
        .enumerate()
    {
        assert!(
            close(
                Some(*actual),
                expected,
                parameter_scales[index / 2] * parameter_scales[index % 2] / objective_scale,
                objective_allowance
            ),
            "{:?}",
            hessians[0]
        );
    }
    assert!(hessians[0].eigenvalues[0] < 0.0 && hessians[0].eigenvalues[1] > 0.0);
    // The multipliers of a certified point are sensitivity certified.
    let row = variables.iter().find(|r| r.symbol_id == x).unwrap();
    assert_eq!(
        row.dual_qualification,
        DualQualification::SensitivityCertified
    );
    // Each parameter's coordinate scale is resolved, with its source recorded.
    for parameter in [a, b] {
        let target = numerics.iter().find(|r| r.target_id == parameter).unwrap();
        assert!(target.coordinate_scale > 0.0 && !target.provenance.is_empty());
        assert_eq!(
            target.engineering.as_ref().unwrap().relative_fraction,
            policy.engineering_relative_fraction
        );
    }
}

#[tokio::test]
async fn sensitivity_withheld_without_local_analysis() {
    // The coefficient route runs no KKT-point analysis: the validity rows say so and no
    // data row exists.
    let (result, _) = solve(LINEAR, SolverSelection::Explicit(Backend::Highs)).await;
    let validity: Vec<local_validity::Row> = rows(&result, "runtime.local_validity");
    assert_eq!(validity.len(), 2);
    for row in &validity {
        assert!(!row.validity.certified);
        assert_eq!(row.validity.reason, Some(WithheldReason::NoLocalAnalysis));
        assert!(row.validity.licq.is_none());
    }
    assert!(
        rows::<parametric_sensitivities::Row>(&result, "runtime.parametric_sensitivities")
            .is_empty()
    );
    assert!(rows::<reduced_hessians::Row>(&result, "runtime.reduced_hessians").is_empty());
}

/// The same nonlinear state relation used by the covariance witness. `b` supplies
/// an independent keyed parameter axis: the physical forcing is a + 2*b.
#[cfg(all(feature = "canonical-tests", feature = "native-solvers"))]
const NONLINEAR_CURVATURE: &str = "package p { def Root {
    param a: Scalar = 3; param b: Scalar = 0;
    var y: Scalar; var x: Scalar;
    eq coupling: y - x*x == 0;
    let cost: Scalar = 0.5*(x-a-2*b)*(x-a-2*b) + 0.5*y*y;
    annotation objective cost(minimize);
    annotation start x(0.8); annotation start y(0.7); } }";

/// Explicit conditioning and accuracy requests are inputs, independent of the
/// analytic answer. Unequal scales expose normalized/physical transport errors.
#[cfg(all(feature = "canonical-tests", feature = "native-solvers"))]
pub(in crate::workflow) fn curvature_requirement(
    kind: NumericalTarget,
    target: SemanticId,
    nominal: f64,
) -> pse_model::numerics::NumericalRequirement {
    pse_model::numerics::NumericalRequirement {
        requirement_id: pse_ids::named_id(target, &format!("curvature:{kind:?}")),
        model_id: None,
        case_id: None,
        instance_id: None,
        fit_id: None,
        target_id: target,
        target_kind: kind,
        nominal: Some(nominal),
        scaling_factor: None,
        absolute_tolerance: Some(1e-7),
        relative_tolerance: Some(0.),
        shared_engineering_allowance: None,
        engineering_rule_id: None,
        unit_id: None,
        coordinates: pse_model::generated::enums::NumericalCoordinates::Physical,
        priority: 100,
        required: true,
        provenance: "independent nonlinear curvature witness".into(),
    }
}

#[cfg(all(feature = "canonical-tests", feature = "native-solvers"))]
pub(in crate::workflow) async fn reopened_rows<R: RelationRow>(
    runtime: &Runtime,
    result: &crate::workflow::RunResult,
    name: &str,
) -> Vec<R> {
    let mut stream = runtime
        .results(
            result.canonical_run_key().unwrap(),
            result.canonical_attempt_key().unwrap(),
            name,
            0,
            u64::MAX,
            pse_columnar::CancellationToken::new(),
        )
        .await
        .unwrap();
    let mut reopened = Vec::new();
    while let Some(batch) = stream.next_batch().await.unwrap() {
        let checked = pse_relations::columnar::FieldCheckedBatch::admit(
            runtime.registry(),
            R::relation(runtime.registry()).unwrap(),
            batch,
            &runtime.validation_context(),
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap();
        reopened.extend(R::rows(&checked).unwrap());
    }
    reopened
}

/// L=f+lambda*(y-x²), so at x=s, y=s², a=s+2s³ the multiplier
/// is -s². The tangent denominator is 1+6s², not the 1+4s² from
/// omitting multiplier curvature. d²f*/da²=1-1/(1+6s²).
#[cfg(all(feature = "canonical-tests", feature = "native-solvers"))]
#[tokio::test]
async fn nonlinear_constraint_curvature_sensitivity_and_value_hessian_reopen() {
    let runtime = crate::workflow::durable_tests::durable_runtime();
    let (package, mut request) =
        crate::workflow::durable_tests::package_on(&runtime, NONLINEAR_CURVATURE).await;
    request.solver.intent = SolveIntent::Optimize;
    request.solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    request.bindings.demand = ["a", "b", "x", "y"].map(String::from).to_vec();
    let cancel = crate::CancelSource::new();
    let plain = package.prepare_analysis(&request, &cancel).await.unwrap();
    let model = &plain.model.model.compiled().model;
    let [a, b, x, y] = ["a", "b", "x", "y"].map(|path| model.paths[path]);
    let coupling = model
        .equations
        .iter()
        .find(|row| row.lineage.path.ends_with(".coupling"))
        .unwrap()
        .id;
    request.solver.numerics.requirements = [
        (NumericalTarget::Variable, a, 3.),
        (NumericalTarget::Variable, b, 5.),
        (NumericalTarget::Variable, x, 2.),
        (NumericalTarget::Variable, y, 7.),
        (NumericalTarget::Row, coupling, 11.),
        (NumericalTarget::Objective, SemanticId::NIL, 13.),
    ]
    .map(|(kind, id, scale)| curvature_requirement(kind, id, scale))
    .to_vec();
    request.solver.numerics.kkt.stationarity = 1e-9;
    request.solver.numerics.kkt.complementarity = 1e-9;
    request.solver.numerics.linear_backward_error = 1e-9;
    request.solver.numerics.supplier_action_accuracy = 1e-9;
    for (forcing, s, parameters) in [(3., 1., vec![b, a]), (18., 2., vec![a, b])] {
        request.case.values.insert("a".into(), forcing);
        request.solver.sensitivity = Some(SensitivityRequest {
            parameters: parameters.clone(),
            reduced_hessian: true,
            propagation: None,
        });
        let prepared = package.prepare_analysis(&request, &cancel).await.unwrap();
        let result = prepared.start().unwrap().wait().await.unwrap();
        assert!(result.usable(), "{:?}", result.report());
        let variables: Vec<solve_variables::Row> = rows(&result, "runtime.solve_variables");
        for (target, expected) in [(x, s), (y, s * s)] {
            let row = variables
                .iter()
                .find(|row| row.symbol_id == target)
                .unwrap();
            assert!((row.value.unwrap() - expected).abs() < 1e-5, "{row:?}");
            assert_eq!(
                row.dual_qualification,
                DualQualification::SensitivityCertified
            );
        }
        let constraints: Vec<solve_constraints::Row> = rows(&result, "runtime.solve_constraints");
        let constraint = constraints
            .iter()
            .find(|row| row.row_id == coupling)
            .unwrap();
        assert!(
            (constraint.dual.unwrap() + s * s).abs() < 1e-5,
            "{constraint:?}"
        );
        assert!(constraint.equality_residual.unwrap().abs() <= constraint.tolerance.unwrap());
        let denominator = 1. + 6. * s * s;
        let sensitivities: Vec<parametric_sensitivities::Row> =
            rows(&result, "runtime.parametric_sensitivities");
        for parameter in &parameters {
            let axis = if *parameter == a { 1. } else { 2. };
            for (kind, target, expected, dual) in [
                (NumericalTarget::Variable, x, axis / denominator, false),
                (
                    NumericalTarget::Variable,
                    y,
                    axis * 2. * s / denominator,
                    false,
                ),
                (
                    NumericalTarget::Row,
                    coupling,
                    -axis * 2. * s / denominator,
                    true,
                ),
                (
                    NumericalTarget::Objective,
                    SemanticId::NIL,
                    axis * (forcing - s),
                    false,
                ),
            ] {
                let row = sensitivities
                    .iter()
                    .find(|row| {
                        row.parameter_id == *parameter
                            && row.target_kind == kind
                            && row.target_id == target
                    })
                    .unwrap();
                let actual = if dual { row.dual } else { row.primal }.unwrap();
                assert!(
                    (actual - expected).abs() < 1e-5,
                    "{forcing}: {row:?}, expected {expected}"
                );
            }
        }
        let hessians: Vec<reduced_hessians::Row> = rows(&result, "runtime.reduced_hessians");
        assert_eq!(hessians.len(), 1);
        let hessian = &hessians[0];
        // Interpret every matrix cell by its published identity rather than request position.
        assert_eq!(
            hessian
                .parameters
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>(),
            [a, b].into_iter().collect()
        );
        assert_eq!(hessian.values.len(), 4);
        for (i, left) in hessian.parameters.iter().enumerate() {
            for (j, right) in hessian.parameters.iter().enumerate() {
                let weight = if *left == a { 1. } else { 2. } * if *right == a { 1. } else { 2. };
                let expected = weight * (1. - 1. / denominator);
                assert!(
                    (hessian.values[2 * i + j] - expected).abs() < 1e-5,
                    "{hessian:?}"
                );
            }
        }
        let validity: Vec<local_validity::Row> = rows(&result, "runtime.local_validity");
        for quantity in [
            DerivedQuantity::ParametricSensitivity,
            DerivedQuantity::ReducedHessian,
        ] {
            let row = validity
                .iter()
                .find(|row| row.quantity == quantity)
                .unwrap();
            assert!(
                row.validity.certified && row.validity.reason.is_none(),
                "{row:?}"
            );
            assert_eq!(
                (row.validity.licq, row.validity.second_order),
                (Some(true), Some(true))
            );
        }
        let numerics: Vec<resolved_numerics::Row> = rows(&result, "runtime.resolved_numerics");
        for (kind, target, expected) in [
            (NumericalTarget::Variable, x, 2.),
            (NumericalTarget::Variable, y, 7.),
            (NumericalTarget::Row, coupling, 11.),
            (NumericalTarget::Objective, SemanticId::NIL, 13.),
        ] {
            assert_eq!(
                numerics
                    .iter()
                    .find(|row| row.target_kind == kind && row.target_id == target)
                    .unwrap()
                    .coordinate_scale,
                expected
            );
        }
        // Lifecycle assertions match declared semantic primary keys, independently
        // of query order. Every identity, unit, value and validity field remains exact.
        let mut sensitivities = sensitivities;
        let mut reopened_sensitivities = reopened_rows::<parametric_sensitivities::Row>(
            &runtime, &result, "runtime.parametric_sensitivities",
        ).await;
        sensitivities.sort_by_key(|row| (
            row.run_id, row.step, row.parameter_id, row.target_kind.as_str(), row.target_id,
        ));
        reopened_sensitivities.sort_by_key(|row| (
            row.run_id, row.step, row.parameter_id, row.target_kind.as_str(), row.target_id,
        ));
        assert_eq!(reopened_sensitivities, sensitivities);
        let mut hessians = hessians;
        let mut reopened_hessians = reopened_rows::<reduced_hessians::Row>(
            &runtime, &result, "runtime.reduced_hessians",
        ).await;
        hessians.sort_by_key(|row| (row.run_id, row.step));
        reopened_hessians.sort_by_key(|row| (row.run_id, row.step));
        assert_eq!(reopened_hessians, hessians);
        let mut validity = validity;
        let mut reopened_validity = reopened_rows::<local_validity::Row>(
            &runtime, &result, "runtime.local_validity",
        ).await;
        validity.sort_by_key(|row| (row.run_id, row.step, row.quantity.as_str()));
        reopened_validity.sort_by_key(|row| (row.run_id, row.step, row.quantity.as_str()));
        assert_eq!(reopened_validity, validity);
    }
}
