// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{NlpOracle, OracleContract, Variable, quality::Tolerances, solve::*};
use pounce_nlp::expression_provider::{FbbtOp as Op, FbbtTape};
use pse_ids::{ContentHash, SemanticId};
use pse_math::{
    binding::ObjectiveSense,
    presolve::{AffineRow, Facts},
    sparse::AssemblyMatrix,
};
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
#[derive(Debug)]
struct Mixed {
    contract: OracleContract,
    facts: Facts,
    j: AssemblyMatrix,
    h: AssemblyMatrix,
    bounds: Vec<(f64, f64)>,
    fail: bool,
    sign: f64,
    linear_second: bool,
}
impl Mixed {
    fn new() -> Self {
        let key = ContentHash::from_bytes([1; 32]);
        Self {
            contract: OracleContract {
                identity: key,
                variables: vec![
                    Variable {
                        id: id(1),
                        lower: 0.0,
                        upper: 10.0,
                    },
                    Variable {
                        id: id(2),
                        lower: 0.0,
                        upper: 10.0,
                    },
                ],
                rows: vec![id(3), id(4)],
                derivatives: pse_kernels::DerivativeOrder::Second,
                smoothness: pse_kernels::DerivativeOrder::Second,
            },
            facts: Facts {
                key,
                structure: key,
                values: BTreeMap::new(),
                affine: vec![
                    Some(AffineRow {
                        entries: BTreeMap::from([(0, 1.0), (1, 1.0)]),
                        constant: 3.0,
                    }),
                    None,
                ],
                tapes: vec![
                    FbbtTape {
                        ops: vec![
                            Op::Var(0),
                            Op::Var(1),
                            Op::Add(0, 1),
                            Op::Const(3.0),
                            Op::Add(2, 3),
                        ],
                    },
                    FbbtTape {
                        ops: vec![Op::Opaque],
                    },
                ],
                complete: vec![true, false],
                objective_linear: vec![false, false],
            },
            j: AssemblyMatrix::new(2, 2, &[(0, 0), (1, 0), (0, 1), (1, 1)], 100).unwrap(),
            h: AssemblyMatrix::new(2, 2, &[(0, 0), (1, 1)], 100).unwrap(),
            bounds: vec![(7.0, 7.0), (0.0, 100.0)],
            fail: false,
            sign: 1.0,
            linear_second: false,
        }
    }
}
impl NlpOracle for Mixed {
    fn presolve_facts(&self) -> Option<&Facts> {
        Some(&self.facts)
    }
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
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        Ok(self.sign * (x[0] * x[0] + x[1] * x[1]))
    }
    fn constraints(&mut self, x: &[f64], g: &mut [f64]) -> Result<(), ProblemError> {
        if self.fail {
            return Err(ProblemError::Contract("failed observation".into()));
        }
        g.copy_from_slice(&[
            x[0] + x[1] + 3.0,
            if self.linear_second {
                x[0] - x[1]
            } else {
                x[0] * x[0] + x[1] * x[1]
            },
        ]);
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], g: &mut [f64]) -> Result<(), ProblemError> {
        g.copy_from_slice(&[self.sign * 2.0 * x[0], self.sign * 2.0 * x[1]]);
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], g: &mut [f64]) -> Result<(), ProblemError> {
        g.copy_from_slice(&[
            1.0,
            if self.linear_second { 1.0 } else { 2.0 * x[0] },
            1.0,
            if self.linear_second { -1.0 } else { 2.0 * x[1] },
        ]);
        Ok(())
    }
    fn hessian(&mut self, _: &[f64], w: f64, l: &[f64], g: &mut [f64]) -> Result<(), ProblemError> {
        g.fill(2.0 * (self.sign * w + if self.linear_second { 0.0 } else { l[1] }));
        Ok(())
    }
}
fn tolerances() -> Tolerances {
    Tolerances {
        variables: vec![1e-8; 2],
        rows: vec![1e-8; 2],
        integrality: 1e-8,
    }
}
fn stamp() -> Compatibility {
    Compatibility {
        layout: ContentHash::from_bytes([2; 32]),
        data: ContentHash::from_bytes([3; 32]),
        backend: Backend::Ipopt,
    }
}
fn execution() -> Execution {
    Execution::new(Default::default(), &Controls::default())
}
#[cfg(all(feature = "ipopt", feature = "pounce"))]
#[test]
fn native_presolve_recovers_optimum_duals_and_compatible_warm_start() {
    fn run(backend: Backend, policy: &Policy, warm: Option<&WarmStart>) -> SolveReport {
        let mut compatibility = stamp();
        compatibility.backend = backend;
        let mut pipeline = Pipeline::new(
            Box::new(Mixed::new()),
            &[1., 3.],
            policy,
            &tolerances(),
            Some(&Scaling {
                objective: 0.5,
                variables: vec![2., 3.],
                constraints: vec![4., 5.],
            }),
            execution(),
            warm,
            compatibility,
            1000,
        )
        .unwrap();
        let mut oracle = pipeline.take_oracle().unwrap();
        let controls = Controls {
            tolerance: 1e-10,
            ..Default::default()
        };
        let report = match backend {
            Backend::Ipopt => crate::ipopt::Session::new().solve(
                &mut oracle,
                pipeline.initial(),
                ObjectiveSense::Minimize,
                &controls,
                execution(),
                &pipeline.tolerances(&tolerances()),
                None,
                pipeline.warm(),
                pipeline.native_compatibility().clone(),
            ),
            Backend::Pounce => crate::pounce::Session::new().solve(
                Box::new(oracle),
                pipeline.initial(),
                ObjectiveSense::Minimize,
                &controls,
                crate::pounce::Method::InteriorPoint,
                Default::default(),
                execution(),
                &pipeline.tolerances(&tolerances()),
                pipeline.warm(),
                pipeline.native_compatibility().clone(),
            ),
            _ => unreachable!(),
        }
        .unwrap();
        pipeline.finish(report, &tolerances(), ObjectiveSense::Minimize)
    }
    for backend in [Backend::Ipopt, Backend::Pounce] {
        for policy in [Policy::Off, Policy::Auto] {
            let cold = run(backend, &policy, None);
            for report in [&cold, &run(backend, &policy, cold.warm_start.as_ref())] {
                assert!(
                    matches!(
                        report.termination.category,
                        Termination::Success | Termination::Acceptable
                    ),
                    "{report:?}"
                );
                assert_eq!(
                    report.termination.assurance,
                    Assurance::LocalStationary,
                    "{report:?}"
                );
                let candidate = report.candidate.as_ref().unwrap();
                assert!(
                    candidate.primal.iter().all(|v| (v - 2.).abs() < 1e-6),
                    "{report:?}"
                );
                assert!((candidate.objective.unwrap() - 8.).abs() < 1e-6);
                assert!(
                    (candidate.row_dual.as_ref().unwrap()[0] + 4.).abs() < 1e-5,
                    "{report:?}"
                );
                assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
                assert!(
                    report
                        .observation
                        .as_ref()
                        .unwrap()
                        .stationarity
                        .as_ref()
                        .unwrap()
                        .iter()
                        .all(|v| v.abs() < 1e-5),
                    "{report:?}"
                );
                assert_eq!(
                    report.preprocessing.as_ref().unwrap().dimensions,
                    if matches!(policy, Policy::Auto) {
                        (2, 2, 1, 1)
                    } else {
                        (2, 2, 2, 2)
                    }
                );
            }
            let mut incompatible = cold.warm_start.unwrap();
            incompatible.compatibility.layout = ContentHash::from_bytes([99; 32]);
            let mut compatibility = stamp();
            compatibility.backend = backend;
            assert!(
                Pipeline::new(
                    Box::new(Mixed::new()),
                    &[1., 3.],
                    &policy,
                    &tolerances(),
                    None,
                    execution(),
                    Some(&incompatible),
                    compatibility,
                    1000
                )
                .is_err()
            );
        }
    }
}
#[test]
fn shared_affine_transport_recovers_original_values_and_kkt() {
    let mut p = Pipeline::new(
        Box::new(Mixed::new()),
        &[1.0, 3.0],
        &Policy::Auto,
        &tolerances(),
        None,
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    let mut oracle = p.take_oracle().unwrap();
    assert_eq!(oracle.contract().variables.len(), 1);
    let mut gradient = vec![0.0; 1];
    oracle.gradient(&[2.0], &mut gradient).unwrap();
    assert_eq!(gradient, vec![0.0]);
    let mut h = vec![0.0; 1];
    oracle.hessian(&[2.0], 1.0, &[0.0], &mut h).unwrap();
    assert_eq!(h, vec![4.0]);
    let mut report = SolveReport::new(
        Backend::Ipopt,
        oracle.contract(),
        NativeTermination {
            code: 0,
            name: "unit-finalization".into(),
            message: None,
            category: Termination::Success,
            assurance: Assurance::LocalStationary,
        },
        &execution(),
    );
    report.candidate = Some(Candidate {
        primal: vec![2.0],
        objective: Some(8.0),
        row_dual: Some(vec![0.0]),
        bound_dual: Some((vec![0.0], vec![0.0])),
        reduced_costs: None,
        slacks: None,
    });
    let report = p.finish(report, &tolerances(), ObjectiveSense::Minimize);
    assert_eq!(report.candidate.as_ref().unwrap().primal, vec![2.0, 2.0]);
    let observation = report.observation.unwrap();
    assert_eq!(observation.values, vec![7.0, 8.0]);
    assert_eq!(observation.equality_residuals, vec![Some(0.0), None]);
    assert!(
        observation
            .stationarity
            .unwrap()
            .iter()
            .all(|v| v.abs() < 1e-12)
    );
    assert!(report.quality.unwrap().feasible());
    assert_eq!(report.preprocessing.unwrap().dimensions, (2, 2, 1, 1));
}
#[test]
fn qualification_declines_tiny_support_and_narrow_intervals() {
    let mut oracle = Mixed::new();
    oracle.facts.affine[0]
        .as_mut()
        .unwrap()
        .entries
        .insert(1, 1e-15);
    let r = Policy::Auto.qualify(&oracle, &tolerances()).unwrap();
    assert!(!r.passes[&Pass::AffineElimination].applied);
    let required = Policy::Explicit {
        options: PresolveOptions {
            enabled: true,
            linear_eq_reduction: true,
            ..PresolveOptions::defaults()
        },
        required: BTreeSet::from([Pass::AffineElimination]),
    };
    assert!(required.qualify(&oracle, &tolerances()).is_err());
    oracle.facts.affine[0]
        .as_mut()
        .unwrap()
        .entries
        .insert(1, 1.0);
    oracle.bounds[0] = (7.0, 7.0 + 1e-13);
    assert!(
        !Policy::Auto
            .qualify(&oracle, &tolerances())
            .unwrap()
            .effective
            .linear_eq_reduction
    );
}
#[test]
fn off_is_identity_and_tape_edits_invalidate_native_reuse() {
    let mut a = Pipeline::new(
        Box::new(Mixed::new()),
        &[1.0, 3.0],
        &Policy::Off,
        &tolerances(),
        None,
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    let mut oracle = a.take_oracle().unwrap();
    let mut g = vec![0.0; 2];
    oracle.constraints(&[1.0, 3.0], &mut g).unwrap();
    assert_eq!(g, vec![7.0, 10.0]);
    let mut edited = Mixed::new();
    edited.facts.key = ContentHash::from_bytes([7; 32]);
    let b = Pipeline::new(
        Box::new(edited),
        &[1.0, 3.0],
        &Policy::Off,
        &tolerances(),
        None,
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    assert_ne!(a.native_compatibility().layout, b.native_compatibility().layout);
}
#[test]
fn failed_observation_preserves_native_status_and_candidate() {
    let mut oracle = Mixed::new();
    oracle.fail = true;
    let mut report = SolveReport::new(
        Backend::Ipopt,
        oracle.contract(),
        NativeTermination {
            code: 42,
            name: "native-return".into(),
            message: None,
            category: Termination::Success,
            assurance: Assurance::LocalStationary,
        },
        &execution(),
    );
    report.candidate = Some(Candidate {
        primal: vec![2.0, 2.0],
        objective: Some(8.0),
        row_dual: None,
        bound_dual: None,
        reduced_costs: None,
        slacks: None,
    });
    crate::quality::attach_nlp(
        &mut report,
        &mut oracle,
        &tolerances(),
        ObjectiveSense::Minimize,
    );
    assert_eq!(report.termination.code, 42);
    assert!(report.candidate.is_some());
    assert!(report.validation_error.is_some());
    assert!(report.observation.is_none());
    assert_eq!(report.termination.assurance, Assurance::None);
}

#[test]
fn explicit_options_are_library_validated_and_required_passes_are_not_noops() {
    let policy = Policy::from_native_options(
        &Options::from([("presolve_fbbt".into(), OptionValue::Bool(true))]),
        BTreeSet::from([Pass::Fbbt]),
    )
    .unwrap();
    assert!(policy.qualify(&Mixed::new(), &tolerances()).unwrap().passes[&Pass::Fbbt].applied);
    assert!(
        Policy::from_native_options(
            &Options::from([("invented".into(), OptionValue::Bool(true))]),
            BTreeSet::new()
        )
        .is_err()
    );
    let mut opaque = Mixed::new();
    opaque.facts.complete.fill(false);
    assert!(policy.qualify(&opaque, &tolerances()).is_err());
}
#[test]
fn maximization_scales_and_original_warm_seed_preserve_conventions() {
    let mut source = Mixed::new();
    source.sign = -1.0;
    let warm = WarmStart {
        compatibility: stamp(),
        payload: WarmPayload::Nlp {
            primal: vec![2.0, 2.0],
            bounds: Some((vec![0.0; 2], vec![0.0; 2])),
            rows: Some(vec![4.0, 0.0]),
        },
    };
    let scales = Scaling {
        objective: 2.0,
        variables: vec![3.0, 5.0],
        constraints: vec![7.0, 11.0],
    };
    let mut pipeline = Pipeline::new(
        Box::new(source),
        &[1.0, 3.0],
        &Policy::Off,
        &tolerances(),
        Some(&scales),
        execution(),
        Some(&warm),
        stamp(),
        1000,
    )
    .unwrap();
    assert!(pipeline.warm().is_some());
    let transport = pipeline.take_oracle().unwrap();
    assert_eq!(transport.scaling().unwrap().objective, 2.0);
    let mut report = SolveReport::new(
        Backend::Ipopt,
        transport.contract(),
        NativeTermination {
            code: 0,
            name: "unit-finalization".into(),
            message: None,
            category: Termination::Success,
            assurance: Assurance::LocalStationary,
        },
        &execution(),
    );
    report.candidate = Some(Candidate {
        primal: vec![2.0, 2.0],
        objective: Some(8.0),
        row_dual: Some(vec![4.0, 0.0]),
        bound_dual: Some((vec![0.0; 2], vec![0.0; 2])),
        reduced_costs: None,
        slacks: None,
    });
    let report = pipeline.finish(report, &tolerances(), ObjectiveSense::Maximize);
    let observed = report.observation.unwrap();
    assert_eq!(observed.objective, Some(8.0));
    assert_eq!(observed.stationarity, Some(vec![0.0, 0.0]));
    assert!(report.warm_start.is_some());
}
#[test]
fn fully_determined_library_standdown_preserves_the_original_problem() {
    let mut source = Mixed::new();
    source.linear_second = true;
    source.bounds[1] = (0.0, 0.0);
    source.facts.affine[1] = Some(AffineRow {
        entries: BTreeMap::from([(0, 1.0), (1, -1.0)]),
        constant: 0.0,
    });
    let policy = Policy::Explicit {
        options: PresolveOptions {
            enabled: true,
            bound_tightening: false,
            redundant_constraint_removal: false,
            linear_eq_reduction: true,
            licq_check: false,
            ..PresolveOptions::defaults()
        },
        required: BTreeSet::from([Pass::AffineElimination]),
    };
    let pipeline = Pipeline::new(
        Box::new(source),
        &[2.0, 2.0],
        &policy,
        &tolerances(),
        None,
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    // Pinned POUNCE deliberately stands down when every variable would disappear.
    // Do not fabricate a zero-dimensional success or a bespoke constant solver.
    assert!(
        pipeline
            .terminal_report(ObjectiveSense::Minimize)
            .unwrap()
            .is_none()
    );
    let mut pipeline = pipeline;
    let mut oracle = pipeline.take_oracle().unwrap();
    assert_eq!(oracle.contract().variables.len(), 2);
    let mut g = vec![0.0; 2];
    oracle.constraints(&[2.0, 2.0], &mut g).unwrap();
    assert_eq!(g, vec![4.0, 0.0]); // normalized affine constant, same feasible set
}
