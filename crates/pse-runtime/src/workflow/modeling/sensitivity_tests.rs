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
            solve_variables,
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

pub(in crate::workflow) fn package(text: &str) -> (ModelingPackage, DeclarationId) {
    let physical = fixture::physical();
    let mut names = fixture::discrete_names();
    names.insert(
        "Scalar".into(),
        physical.quantities.neutral_dimensionless().unwrap(),
    );
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
        .modeling_package(rows, physical, names)
        .unwrap();
    (package, root)
}
pub(in crate::workflow) fn analysis(root: DeclarationId, selection: SolverSelection) -> ModelingAnalysis {
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
) -> (std::sync::Arc<crate::workflow::RunResult>, [SemanticId; 3]) {
    let (result, ids) = solve_propagating(text, selection, &["x"], |_, _| None).await;
    (result, [ids[0], ids[1], ids[2]])
}
/// Solve `text` with a sensitivity request over its parameters `a` and `b` and the
/// propagation `propagation` builds from the identities of `a`, `b` and `outputs`;
/// returns the result and those identities.
pub(in crate::workflow) async fn solve_propagating(
    text: &str,
    selection: SolverSelection,
    outputs: &[&str],
    propagation: impl FnOnce(&[SemanticId], &[SemanticId]) -> Option<crate::math::settings::Propagation>,
) -> (std::sync::Arc<crate::workflow::RunResult>, Vec<SemanticId>) {
    let (package, root) = package(text);
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
        propagation: propagation(&ids[..2], &ids[2..]),
    });
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    (result, ids)
}
pub(in crate::workflow) fn rows<R: RelationRow>(result: &crate::workflow::RunResult, name: &str) -> Vec<R> {
    R::rows(&result.table(name).unwrap()).unwrap()
}

#[tokio::test]
async fn sensitivities_published_with_local_validity() {
    let (result, [a, b, x]) = solve(QUADRATIC, SolverSelection::Explicit(Backend::Ipopt)).await;
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
            (v.licq, v.strict_complementarity, v.second_order, v.weakly_active),
            (Some(true), Some(true), Some(true), Some(0))
        );
        assert!(v.residual.is_some_and(|r| r < 1e-10), "{v:?}");
    }
    // dx/da = 2/3, dx/db = 1/3 and df*/d(a, b) = (−λ, −x) = (−2/3, −4/3).
    let sensitivities: Vec<parametric_sensitivities::Row> =
        rows(&result, "runtime.parametric_sensitivities");
    let find = |parameter, kind, target| {
        sensitivities
            .iter()
            .find(|r| r.parameter_id == parameter && r.target_kind == kind && r.target_id == target)
            .unwrap()
    };
    let close = |actual: Option<f64>, expected: f64| {
        actual.is_some_and(|v| (v - expected).abs() < 1e-6)
    };
    assert!(close(find(a, NumericalTarget::Variable, x).primal, 2.0 / 3.0));
    assert!(close(find(b, NumericalTarget::Variable, x).primal, 1.0 / 3.0));
    let objective = find(a, NumericalTarget::Objective, SemanticId::NIL);
    assert!(close(objective.primal, -2.0 / 3.0) && objective.dual.is_none());
    assert!(close(
        find(b, NumericalTarget::Objective, SemanticId::NIL).primal,
        -4.0 / 3.0
    ));
    // The coupling row's multiplier λ = 2(b − a)/3 moves by (−2/3, 2/3).
    let row = sensitivities
        .iter()
        .find(|r| r.parameter_id == a && r.target_kind == NumericalTarget::Row)
        .unwrap();
    assert!(row.primal.is_none() && close(row.dual, -2.0 / 3.0), "{row:?}");
    let hessians: Vec<reduced_hessians::Row> = rows(&result, "runtime.reduced_hessians");
    assert_eq!(hessians.len(), 1);
    assert_eq!(hessians[0].parameters, vec![a, b]);
    for (actual, expected) in hessians[0]
        .values
        .iter()
        .zip([2.0 / 3.0, -2.0 / 3.0, -2.0 / 3.0, -1.0 / 3.0])
    {
        assert!((actual - expected).abs() < 1e-6, "{:?}", hessians[0]);
    }
    assert!(hessians[0].eigenvalues[0] < 0.0 && hessians[0].eigenvalues[1] > 0.0);
    // The multipliers of a certified point are sensitivity certified.
    let variables: Vec<solve_variables::Row> = rows(&result, "runtime.solve_variables");
    let row = variables.iter().find(|r| r.symbol_id == x).unwrap();
    assert_eq!(
        row.dual_qualification,
        DualQualification::SensitivityCertified
    );
    // Each parameter's coordinate scale is resolved, with its source recorded.
    let numerics: Vec<resolved_numerics::Row> = rows(&result, "runtime.resolved_numerics");
    for parameter in [a, b] {
        let target = numerics.iter().find(|r| r.target_id == parameter).unwrap();
        assert!(target.coordinate_scale > 0.0 && !target.provenance.is_empty());
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
