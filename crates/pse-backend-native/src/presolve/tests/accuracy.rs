// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Per-row physical budgets are native coordinates, not changed scientific criteria.
use super::*;
use pse_math::normalization::Normalization;

fn declared() -> Normalization {
    Normalization {
        variables: vec![2.0, 4.0],
        rows: vec![3.0, 7.0],
        objective: 8.0,
    }
}
fn prepare(rows: Vec<f64>) -> Pipeline {
    let mut source = Mixed::new();
    source.normalization = Some(declared());
    let tolerances = Tolerances {
        variables: vec![1e-8; 2],
        rows,
        integrality: 1e-8,
    };
    Pipeline::new(
        Box::new(source),
        &[2.0, 2.0],
        &Policy::Off,
        &tolerances,
        &ResolvedAccuracy::verification(),
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap()
}

#[test]
fn heterogeneous_row_budgets_transport_callbacks_bounds_and_original_duals() {
    let mut source = Mixed::new();
    source.normalization = Some(declared());
    let tolerances = Tolerances {
        variables: vec![1e-8; 2],
        rows: vec![1e-8, 1e-4],
        integrality: 1e-8,
    };
    let warm = WarmStart {
        origin: None,
        compatibility: stamp(),
        payload: WarmPayload::Nlp {
            primal: vec![2.0, 2.0],
            bounds: Some((vec![0.0; 2], vec![0.0; 2])),
            rows: Some(vec![-4.0, 0.0]),
            barrier: None,
            working: None,
        },
    };
    let accuracy = ResolvedAccuracy::verification();
    let mut pipeline = Pipeline::new(
        Box::new(source),
        &[2.0, 2.0],
        &Policy::Off,
        &tolerances,
        &accuracy,
        execution(),
        Some(&warm),
        stamp(),
        1000,
    )
    .unwrap();
    assert_eq!(pipeline.initial(), &[1.0, 0.5]);
    let WarmPayload::Nlp {
        rows: Some(rows), ..
    } = &pipeline.warm().unwrap().payload
    else {
        panic!("native warm multipliers absent")
    };
    assert_eq!(rows, &[-0.5, 0.0]);
    let projected = pipeline.tolerances(&tolerances);
    assert_eq!(projected.rows, vec![accuracy.feasibility; 2]);
    let normalization: serde_json::Value =
        serde_json::from_str(&pipeline.report().diagnostics["normalization.native"]).unwrap();
    assert_eq!(normalization["kind"], "physical-row-budget");
    assert_eq!(normalization["feasibility"], accuracy.feasibility);
    assert_eq!(normalization["declared"], declared().key().to_string());
    assert_eq!(
        normalization["native"],
        declared()
            .with_row_budgets(&tolerances.rows, accuracy.feasibility)
            .unwrap()
            .key()
            .to_string()
    );
    let mut native = pipeline.take_oracle().unwrap();
    assert_eq!(native.contract().variables[0].upper, 5.0);
    assert_eq!(native.contract().variables[1].upper, 2.5);
    assert_eq!(native.constraint_bounds(), &[(7.0, 7.0), (0.0, 0.01)]);
    let mut values = [0.0; 2];
    native.constraints(&[1.0, 0.5], &mut values).unwrap();
    assert_eq!(values, [7.0, 0.0008]);
    let mut jacobian = [0.0; 4];
    native.jacobian(&[1.0, 0.5], &mut jacobian).unwrap();
    assert_eq!(jacobian, [2.0, 0.0008, 4.0, 0.0016]);
    native
        .hessian(&[1.0, 0.5], 3.0, &[5.0, 7.0], &mut values)
        .unwrap();
    assert!((values[0] - 3.0056).abs() < 1e-12 && (values[1] - 12.0224).abs() < 1e-12);
    let mut report = SolveReport::new(
        Backend::Ipopt,
        native.contract(),
        NativeTermination {
            code: 0,
            name: "transport-control".into(),
            message: None,
            category: Termination::Success,
            assurance: Assurance::None,
        },
        &execution(),
    );
    report.candidate = Some(Candidate {
        kind: CandidateKind::FinalIterate,
        primal: vec![1.0, 0.5],
        objective: Some(1.0),
        row_dual: Some(vec![-0.5, 0.0]),
        bound_dual: Some((vec![0.0; 2], vec![0.0; 2])),
        reduced_costs: None,
        slacks: None,
        commitment: None,
    });
    let (report, _) = pipeline.finish(
        report,
        &tolerances,
        ObjectiveSense::Minimize,
        &Analysis::NONE,
        UNUSED,
    );
    assert!(report.quality.as_ref().unwrap().feasible());
    assert_eq!(
        report.observation.unwrap().stationarity,
        Some(vec![0.0, 0.0])
    );
    let candidate = report.candidate.unwrap();
    assert_eq!(candidate.primal, vec![2.0, 2.0]);
    assert_eq!(candidate.row_dual, Some(vec![-4.0, 0.0]));
    assert_eq!(candidate.objective, Some(8.0));
}

#[test]
fn changed_row_budget_invalidates_native_layout_without_changing_declared_nominals() {
    let original = declared();
    let first = prepare(vec![1e-8, 1e-4]);
    let same = prepare(vec![1e-8, 1e-4]);
    let changed = prepare(vec![1e-8, 2e-4]);
    assert_eq!(
        first.native_compatibility().layout,
        same.native_compatibility().layout
    );
    assert_ne!(
        first.native_compatibility().layout,
        changed.native_compatibility().layout
    );
    assert_eq!(
        first.native_compatibility().profile,
        changed.native_compatibility().profile
    );
    assert_eq!(declared(), original);
}

#[test]
fn affine_pass_eligibility_consumes_actual_native_row_scales() {
    let source = Mixed::new();
    let mut tolerance = tolerances();
    tolerance.rows[0] = 1e8;
    let native = Normalization::identity(2, 2)
        .with_row_budgets(&tolerance.rows, 1e-8)
        .unwrap();
    let report = Policy::Auto.qualify(&source, &tolerance, &native).unwrap();
    assert!(!report.passes[&Pass::AffineElimination].eligible);
    assert!(!report.passes[&Pass::AffineElimination].applied);
}

#[test]
fn native_pipeline_rejects_invalid_physical_row_budget_projection() {
    for rows in [
        vec![1e-8],
        vec![1e-8, f64::NAN],
        vec![1e-8, 0.0],
        vec![1e-8, f64::MAX],
    ] {
        let source = Mixed::new();
        let tolerance = Tolerances {
            variables: vec![1e-8; 2],
            rows,
            integrality: 1e-8,
        };
        assert!(
            Pipeline::new(
                Box::new(source),
                &[2.0, 2.0],
                &Policy::Off,
                &tolerance,
                &ResolvedAccuracy::verification(),
                execution(),
                None,
                stamp(),
                1000
            )
            .is_err()
        );
    }
}

#[test]
fn native_row_maps_refuse_an_insufficient_admitted_attempt_allowance() {
    let maps = 2 * (4 * size_of::<f64>() + size_of::<Normalization>());
    let mut admitted = execution();
    admitted.memory = Some(maps - 1);
    let error = Pipeline::new(
        Box::new(Mixed::new()),
        &[2.0, 2.0],
        &Policy::Off,
        &tolerances(),
        &ResolvedAccuracy::verification(),
        admitted,
        None,
        stamp(),
        1000,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ProblemError::Limit {
            kind: crate::LimitKind::Memory,
            ..
        }
    ));
}

#[test]
fn native_pipeline_refuses_layout_before_normalization_allocation() {
    for (initial, limit) in [
        (&[2.0, 2.0][..], 1),
        (&[2.0][..], 2),
        (&[f64::NAN, 2.0][..], 2),
    ] {
        let mut source = Mixed::new();
        source.normalization = Some(Normalization {
            variables: vec![f64::NAN; 2],
            rows: vec![1.0; 2],
            objective: 1.0,
        });
        let mut admitted = execution();
        admitted.memory = Some(0);
        let error = Pipeline::new(
            Box::new(source),
            initial,
            &Policy::Off,
            &tolerances(),
            &ResolvedAccuracy::verification(),
            admitted,
            None,
            stamp(),
            limit,
        )
        .unwrap_err();
        assert!(
            matches!(error, ProblemError::Contract(detail) if detail == "presolve dimensions/start/cap")
        );
    }
}

#[cfg(feature = "ipopt")]
#[derive(Debug)]
struct PressureRoot {
    contract: OracleContract,
    bounds: Vec<(f64, f64)>,
    j: AssemblyMatrix,
    h: AssemblyMatrix,
}
#[cfg(feature = "ipopt")]
impl PressureRoot {
    fn new() -> Self {
        let mut contract = Mixed::new().contract;
        contract.variables[0].lower = 0.01;
        contract.variables[0].upper = 1.0;
        contract.variables[1].lower = 5e4;
        contract.variables[1].upper = 2e5;
        Self {
            contract,
            bounds: vec![(0.25, 0.25), (1e5, 1e5)],
            j: jacobian(2, &[(0, 0), (1, 1)]),
            h: hessian(2, &[(0, 0), (1, 1)]),
        }
    }
}
#[cfg(feature = "ipopt")]
impl NlpOracle for PressureRoot {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.j.matrix().symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        Some(self.h.matrix().symbolic())
    }
    fn objective(&mut self, _: &[f64]) -> Result<f64, ProblemError> {
        Ok(0.0)
    }
    fn gradient(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.fill(0.0);
        Ok(())
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.copy_from_slice(&[x[0] * x[0], x[1] * x[1] / 1e5]);
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.copy_from_slice(&[2.0 * x[0], 2.0 * x[1] / 1e5]);
        Ok(())
    }
    fn hessian(
        &mut self,
        _: &[f64],
        _: f64,
        lambda: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out.copy_from_slice(&[2.0 * lambda[0], 2.0 * lambda[1] / 1e5]);
        Ok(())
    }
}

#[cfg(feature = "ipopt")]
#[test]
fn native_pressure_like_root_stops_with_heterogeneous_original_row_budgets() {
    let tolerance = Tolerances {
        variables: vec![1e-8, 1e-4],
        rows: vec![1e-8, 1e-4],
        integrality: 1e-8,
    };
    let accuracy = ResolvedAccuracy::verification();
    let mut pipeline = Pipeline::new(
        Box::new(PressureRoot::new()),
        &[0.6, 1.1e5],
        &Policy::Off,
        &tolerance,
        &accuracy,
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    let mut native = pipeline.take_oracle().unwrap();
    assert_eq!(native.constraint_bounds(), &[(0.25, 0.25), (10.0, 10.0)]);
    assert_eq!(native.contract().variables[1].lower, 5e4);
    assert_eq!(native.contract().variables[1].upper, 2e5);
    let controls = Controls {
        iterations: 100,
        ..Controls::default()
    };
    let report = crate::ipopt::Session::new()
        .solve(
            &mut native,
            pipeline.initial(),
            ObjectiveSense::Minimize,
            &controls,
            &accuracy,
            &Default::default(),
            execution(),
            &pipeline.tolerances(&tolerance),
            pipeline.warm(),
            pipeline.native_compatibility().clone(),
        )
        .unwrap();
    assert_eq!(
        report.termination.category,
        Termination::Success,
        "{report:?}"
    );
    let (report, _) = pipeline.finish(
        report,
        &tolerance,
        ObjectiveSense::Minimize,
        &Analysis::NONE,
        UNUSED,
    );
    let quality = report.quality.unwrap();
    assert!(quality.feasible(), "{quality:?}");
    assert!(quality.rows.iter().all(|v| v.physical <= v.tolerance));
    let point = report.candidate.unwrap().primal;
    assert!((point[0] - 0.5).abs() < 1e-8);
    assert!((point[1] - 1e5).abs() < 1e-4);
}
