// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public Root request, physical response publication and independent withholding.
use super::*;
use crate::math::{settings::SensitivityRequest, solves::NumericalInputs};
use crate::workflow::tests as fixture;
use pse_backend_native::solve::{Backend, SolveIntent, SolverSelection};
use pse_kernels::DerivativeOrder;
use pse_relations::{
    columnar::RelationRow,
    generated::{
        enums::NumericalTarget,
        runtime::{local_validity, parametric_sensitivities, solve_runs},
    },
};
async fn solve_with_backend(
    boundary: bool,
    backend: Backend,
) -> std::sync::Arc<crate::workflow::RunResult> {
    let text = "package p { def Root { param p: Scalar=4; var x: Scalar; eq root: x*x==p; annotation start x(2); } }";
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
    let package = fixture::runtime_with(16 << 20, 16 << 20, 1 << 30)
        .modeling_package(rows, fixture::physical())
        .unwrap();
    let mut solver = fixture::profile();
    solver.intent = SolveIntent::Root;
    solver.selection = SolverSelection::Explicit(backend);
    let order = DerivativeOrder::First.max(solver.derivative_order());
    let mut analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    };
    analysis.bindings.demand.push("p".into());
    analysis.case.variables.insert(
        "x".into(),
        pse_compiler::workspace::ModelingVariableState {
            lower: Some(Some(if boundary { 2. } else { 0. })),
            upper: Some(if backend == Backend::Scip {
                Some(10.)
            } else {
                None
            }),
            fixed: None,
        },
    );
    let cancel = crate::CancelSource::new();
    let plain = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let p = plain.model.model.compiled().model.paths["p"];
    analysis.solver.sensitivity = Some(SensitivityRequest {
        parameters: vec![p],
        reduced_hessian: false,
        propagation: None,
    });
    package
        .prepare_analysis(&analysis, &cancel)
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap()
}
#[tokio::test]
async fn root_response_publication_has_physical_primal_and_no_kkt_fields() {
    let result = solve_with_backend(false, Backend::Kinsol).await;
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    assert_eq!(validity.len(), 1);
    let v = &validity[0].validity;
    assert!(v.certified, "{v:?}");
    assert_eq!(v.root_rank, Some(1));
    assert!(v.root_backward_error.is_some());
    assert!(v.root_neighborhood.is_some());
    assert!(v.licq.is_none() && v.strict_complementarity.is_none() && v.second_order.is_none());
    let response = parametric_sensitivities::Row::rows(
        &result.table("runtime.parametric_sensitivities").unwrap(),
    )
    .unwrap();
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].target_kind, NumericalTarget::Variable);
    assert!(response[0].dual.is_none());
    assert!((response[0].primal.unwrap() - 0.25).abs() < 1e-10);
    let h = 1e-5;
    let independently = (4.0_f64 + h).sqrt() - (4.0_f64 - h).sqrt();
    assert!((response[0].primal.unwrap() - independently / (2. * h)).abs() < 1e-9);
}
#[tokio::test]
async fn root_response_withheld_at_active_bound_keeps_base_solution() {
    let result = solve_with_backend(true, Backend::Kinsol).await;
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    assert_eq!(validity.len(), 1);
    assert!(!validity[0].validity.certified);
    assert_eq!(
        validity[0].validity.reason,
        Some(pse_relations::generated::enums::WithheldReason::NeighborhoodUnavailable)
    );
    assert!(
        parametric_sensitivities::Row::rows(
            &result.table("runtime.parametric_sensitivities").unwrap()
        )
        .unwrap()
        .is_empty()
    );
    let runs = solve_runs::Row::rows(&result.table("runtime.solve_runs").unwrap()).unwrap();
    assert!(!runs.is_empty());
    assert!(
        runs.iter()
            .any(|r| r.qualification
                == pse_relations::generated::enums::NativeQualification::Feasible)
    );
}

#[cfg(any(
    feature = "solver-ipopt",
    feature = "solver-pounce",
    feature = "solver-scip"
))]
async fn assert_root_response(backend: Backend, boundary: bool) {
    let result = solve_with_backend(boundary, backend).await;
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    assert_eq!(validity.len(), 1);
    let v = &validity[0].validity;
    let response = parametric_sensitivities::Row::rows(
        &result.table("runtime.parametric_sensitivities").unwrap(),
    )
    .unwrap();
    assert!(v.licq.is_none() && v.strict_complementarity.is_none() && v.second_order.is_none());
    if boundary {
        assert!(!v.certified, "{backend:?}: {v:?}");
        assert_eq!(
            v.reason,
            Some(pse_relations::generated::enums::WithheldReason::NeighborhoodUnavailable),
            "{backend:?}: {v:?}"
        );
        assert!(response.is_empty());
    } else {
        assert!(v.certified, "{backend:?}: {v:?}");
        assert_eq!(v.root_rank, Some(1));
        assert_eq!(response.len(), 1);
        assert!(response[0].dual.is_none());
        assert!(
            (response[0].primal.unwrap() - 0.25).abs() < 1e-9,
            "{backend:?}: {:?}",
            response[0]
        );
    }
    let runs = solve_runs::Row::rows(&result.table("runtime.solve_runs").unwrap()).unwrap();
    assert!(
        runs.iter().any(|r| r.backend == Some(backend)
            && r.qualification == pse_relations::generated::enums::NativeQualification::Feasible),
        "{backend:?}: {runs:?}"
    );
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn explicit_ipopt_root_response_uses_shared_square_analysis() {
    assert_root_response(Backend::Ipopt, false).await;
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn explicit_ipopt_root_active_bound_withholds_response_and_keeps_base() {
    assert_root_response(Backend::Ipopt, true).await;
}
#[cfg(feature = "solver-pounce")]
#[tokio::test]
async fn explicit_pounce_root_response_uses_shared_square_analysis() {
    assert_root_response(Backend::Pounce, false).await;
}
#[cfg(feature = "solver-pounce")]
#[tokio::test]
async fn explicit_pounce_root_active_bound_withholds_response_and_keeps_base() {
    assert_root_response(Backend::Pounce, true).await;
}

#[cfg(feature = "solver-scip")]
#[tokio::test]
async fn explicit_scip_root_response_uses_shared_square_analysis() {
    assert_root_response(Backend::Scip, false).await;
}
#[cfg(feature = "solver-scip")]
#[tokio::test]
async fn explicit_scip_root_active_bound_withholds_response_and_keeps_base() {
    assert_root_response(Backend::Scip, true).await;
}
