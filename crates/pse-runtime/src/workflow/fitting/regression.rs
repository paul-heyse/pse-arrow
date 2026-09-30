// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A weighted linear regression `yᵢ = a + b·xᵢ` fit, whose covariance is `(XᵀWX)⁻¹`
//! exactly: the fixture of the covariance, interval and propagation units (Plan 22 S3,
//! S4).
use super::*;
use crate::workflow::{
    RunReport, RunResult,
    tests::{compiler_profile, id, physical, runtime},
};
use native::solve::Backend;
use pse_relations::{columnar::RelationRow, generated::runtime::local_validity};

pub(in crate::workflow) const X: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
pub(in crate::workflow) const Y: [f64; 4] = [3.1, 4.9, 7.2, 8.8];
pub(in crate::workflow) const SIGMA: [f64; 4] = [0.5, 1.0, 0.5, 2.0];
/// The parameters `a`, `b` and the offset `d`, and the declared scales of `a` and `b`.
pub(in crate::workflow) const A: u8 = 1;
pub(in crate::workflow) const B: u8 = 2;
pub(in crate::workflow) const D: u8 = 3;
pub(in crate::workflow) const SCALES: [f64; 2] = [1.0, 2.0];

/// `(XᵀWX)⁻¹` row-major and the weighted least-squares estimate.
pub(in crate::workflow) fn analytic() -> ([f64; 4], [f64; 2]) {
    let (mut s00, mut s01, mut s11, mut r0, mut r1) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..4 {
        let w = 1.0 / (SIGMA[i] * SIGMA[i]);
        s00 += w;
        s01 += w * X[i];
        s11 += w * X[i] * X[i];
        r0 += w * Y[i];
        r1 += w * X[i] * Y[i];
    }
    let det = s00 * s11 - s01 * s01;
    let inverse = [s11 / det, -s01 / det, -s01 / det, s00 / det];
    let estimate = [
        inverse[0] * r0 + inverse[1] * r1,
        inverse[2] * r0 + inverse[3] * r1,
    ];
    (inverse, estimate)
}

/// The regression package: one steady experiment observing `y₁…y₄`, with `sigma` and
/// `importance` per observation.
pub(in crate::workflow) fn package(
    sigma: [Option<f64>; 4],
    importance: [f64; 4],
) -> crate::workflow::ModelingPackage {
    package_with(false, sigma, importance)
}
/// With `offset`, a third parameter `d` enters every output only through `a + d`, so the
/// two are not separately identifiable.
pub(in crate::workflow) fn package_with(
    offset: bool,
    sigma: [Option<f64>; 4],
    importance: [f64; 4],
) -> crate::workflow::ModelingPackage {
    let mut physical = physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    let intercept = if offset { "(a + d)" } else { "a" };
    let outputs = (1..=4)
        .map(|i| format!("let y{i}: Scalar = {intercept} + b*{i};"))
        .collect::<String>();
    let mut rows = pse_authoring::language::parse(
        &format!(
            "package p {{ def Root {{ param a: Scalar = 1; param b: Scalar = 1; param d: Scalar = 0; {outputs} }} }}"
        ),
        id(20),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    rows.extend(measured_rows(
        &(0..4)
            .map(|i| (id(40 + i as u8), "Scalar", Some(Y[i]), sigma[i]))
            .collect::<Vec<_>>(),
    ));
    let mut data = FitDeclarations::default();
    let mut observations = Vec::new();
    for (i, weight) in importance.iter().enumerate() {
        let observation = id(40 + i as u8);
        observations.push(serde_json::json!({"value_attribute":"value","standard_deviation_attribute":"sigma","observation_id":observation,"experiment_id":id(33),"output_path":format!("y{}", i + 1),"time":null,"included":true,"importance":weight}));
    }
    let mut parameters = vec![
        serde_json::json!({"symbol_id":id(A),"fixed":false,"value":1.0,"lower":-100.0,"upper":100.0,"scale":SCALES[0]}),
        serde_json::json!({"symbol_id":id(B),"fixed":false,"value":1.0,"lower":-100.0,"upper":100.0,"scale":SCALES[1]}),
    ];
    let mut bindings = vec![
        serde_json::json!({"parameter_id":id(A),"path":"a"}),
        serde_json::json!({"parameter_id":id(B),"path":"b"}),
    ];
    if offset {
        parameters.push(serde_json::json!({"symbol_id":id(D),"fixed":false,"value":0.0,"lower":-100.0,"upper":100.0,"scale":1.0}));
        bindings.push(serde_json::json!({"parameter_id":id(D),"path":"d"}));
    }
    data.fits.push(serde_json::from_value(serde_json::json!({"fit_id":id(32),"parameters":parameters,
        "experiments":[{"experiment_id":id(33),"case_id":root,"route":"steady","bindings":bindings}],
        "observations":observations})).unwrap());
    runtime()
        .modeling_package(rows, physical)
        .unwrap()
        .with_fit_declarations(data)
        .unwrap()
}
pub(in crate::workflow) fn declared() -> [Option<f64>; 4] {
    SIGMA.map(Some)
}
pub(in crate::workflow) fn profile(
    hessian: HessianMode,
    uncertainty: Option<FitUncertainty>,
) -> FitProfile {
    FitProfile {
        solver: SolverProfile {
            presolve: Default::default(),
            numerics: Default::default(),
            convexity: Default::default(),
            intent: SolveIntent::Optimize,
            selection: native::solve::SolverSelection::Explicit(Backend::Ipopt),
            controls: native::solve::Controls {
                hessian,
                ..Default::default()
            },
            backend: native::execution::BackendSettings::Default,
            sensitivity: None,
        },
        simulations: BTreeMap::new(),
        rank_tolerance: 1e-8,
        max_cells: 100_000,
        derivatives: FitDerivatives::Responses,
        uncertainty,
    }
}
pub(in crate::workflow) async fn fit(
    package: &crate::workflow::ModelingPackage,
    profile: FitProfile,
) -> Arc<RunResult> {
    package
        .prepare_fit(
            id(32).into(),
            profile,
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap()
}
pub(in crate::workflow) fn report(result: &RunResult) -> &FitReport {
    let RunReport::Fit(report) = result.report().unwrap() else {
        panic!("fit report")
    };
    report
}
pub(in crate::workflow) fn close(actual: f64, expected: f64, relative: f64) -> bool {
    (actual - expected).abs() <= relative * (1.0 + expected.abs())
}
/// The fit's validity rows by quantity.
pub(in crate::workflow) fn validity(result: &RunResult) -> Vec<local_validity::Row> {
    local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap()
}
