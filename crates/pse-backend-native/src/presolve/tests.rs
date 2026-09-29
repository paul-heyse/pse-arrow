// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    NlpOracle, OracleContract, Variable,
    kkt::{Analysis, Budget},
    quality::Tolerances,
    solve::*,
};
use pounce_nlp::expression_provider::{FbbtOp as Op, FbbtTape};
use pse_ids::{ContentHash, SemanticId};
use pse_math::{
    binding::ObjectiveSense,
    index::{Entry, OriginalCol, OriginalRow},
    presolve::{AffineRow, Facts},
    sparse::AssemblyMatrix,
};

/// The budgets of a recovery that requests no local analysis; nothing reads them.
const UNUSED: Budget = Budget {
    dual: 0.0,
    limit: 0,
};
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
/// A source Jacobian over `n` original rows and columns.
fn jacobian(n: usize, entries: &[(usize, usize)]) -> AssemblyMatrix {
    let entries: Vec<_> = entries
        .iter()
        .map(|&(r, c)| Entry::new(OriginalRow::new(r), OriginalCol::new(c)))
        .collect();
    AssemblyMatrix::new(n, n, &entries, 100).unwrap()
}
/// A source lower-triangle Hessian over `n` original columns.
fn hessian(n: usize, entries: &[(usize, usize)]) -> AssemblyMatrix {
    let entries: Vec<_> = entries
        .iter()
        .map(|&(r, c)| Entry::new(OriginalCol::new(r), OriginalCol::new(c)))
        .collect();
    AssemblyMatrix::hessian(n, &entries, 100).unwrap()
}
fn dimensions(
    original_columns: usize,
    original_rows: usize,
    presolved_columns: usize,
    presolved_rows: usize,
) -> Dimensions {
    Dimensions {
        original_columns,
        original_rows,
        presolved_columns,
        presolved_rows,
    }
}
#[derive(Debug)]
struct Mixed {
    contract: OracleContract,
    facts: Facts,
    j: AssemblyMatrix,
    h: AssemblyMatrix,
    bounds: Vec<(f64, f64)>,
    fail: bool,
    fail_domain: bool,
    sign: f64,
    linear_second: bool,
    normalization: Option<pse_math::normalization::Normalization>,
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
                obligations: BTreeMap::new(),
                signs: BTreeMap::new(),
                has_guards: false,
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
                row_sources: vec![vec![]; 2],
                objective_degree: None,
                objective_linear: vec![false, false],
            },
            j: jacobian(2, &[(0, 0), (1, 0), (0, 1), (1, 1)]),
            h: hessian(2, &[(0, 0), (1, 1)]),
            bounds: vec![(7.0, 7.0), (0.0, 100.0)],
            fail: false,
            fail_domain: false,
            sign: 1.0,
            linear_second: false,
            normalization: None,
        }
    }
}
impl NlpOracle for Mixed {
    fn normalization(&self) -> Option<&pse_math::normalization::Normalization> {
        self.normalization.as_ref()
    }
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
        if self.fail_domain {
            return Err(pse_math::MathError::Domain {
                source_id: id(3),
                requirement: "synthetic envelope",
            }
            .into());
        }
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
        profile: ContentHash::from_bytes([4; 32]),
        data: ContentHash::from_bytes([3; 32]),
        backend: Backend::Ipopt,
    }
}
fn execution() -> Execution {
    Execution::new(Default::default(), &Controls::default())
}
#[test]
fn kernel_presolve_retains_the_typed_failed_trial_witness() {
    let mut oracle = Mixed::new();
    oracle.fail_domain = true;
    let error = Pipeline::new(
        Box::new(oracle),
        &[1., 3.],
        &Policy::Auto,
        &tolerances(),
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap_err();
    assert!(
        matches!(error, ProblemError::Math(pse_math::MathError::Domain {
        source_id, requirement: "synthetic envelope"
    }) if source_id == id(3)),
        "{error:?}"
    );
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
            execution(),
            warm,
            compatibility,
            1000,
        )
        .unwrap();
        let mut oracle = pipeline.take_oracle().unwrap();
        let controls = Controls::default();
        let accuracy = ResolvedAccuracy {
            feasibility: 1e-10,
            stationarity: 1e-10,
            complementarity: 1e-10,
            ..ResolvedAccuracy::nominal()
        };
        let report = match backend {
            Backend::Ipopt => crate::ipopt::Session::new().solve(
                &mut oracle,
                pipeline.initial(),
                ObjectiveSense::Minimize,
                &controls,
                &accuracy,
                &Default::default(),
                execution(),
                &pipeline.tolerances(&tolerances()),
                pipeline.warm(),
                pipeline.native_compatibility().clone(),
            ),
            Backend::Pounce => crate::pounce::Session::new().solve(
                Box::new(oracle),
                pipeline.initial(),
                ObjectiveSense::Minimize,
                &controls,
                &accuracy,
                &Default::default(),
                execution(),
                &pipeline.tolerances(&tolerances()),
                pipeline.warm(),
                pipeline.native_compatibility().clone(),
            ),
            _ => panic!("test helper supports only the two shared NLP adapters"),
        }
        .unwrap();
        let (mut report, _) = pipeline.finish(
            report,
            &tolerances(),
            ObjectiveSense::Minimize,
            &Analysis::NONE,
            UNUSED,
        );
        crate::quality::record_kkt(
            &mut report,
            &pse_math::normalization::Normalization::identity(2, 2),
            &accuracy,
        );
        crate::quality::qualify(&mut report, &accuracy);
        report
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
                        dimensions(2, 2, 1, 1)
                    } else {
                        dimensions(2, 2, 2, 2)
                    }
                );
                // Library reports are serde JSON records, never Rust `Debug` text (F30).
                let diagnostics = &report.preprocessing.as_ref().unwrap().diagnostics;
                let library = [
                    "bounds",
                    "fbbt",
                    "rank",
                    "auxiliary",
                    "affine",
                    "warm.rows",
                    "warm.affine",
                ];
                let provenance = ["warm.diagnostics", "crossover", "feral.effective"];
                let records = library
                    .iter()
                    .filter_map(|key| diagnostics.get(*key).map(|text| (*key, text)))
                    .chain(
                        provenance
                            .iter()
                            .filter_map(|key| report.provenance.get(*key).map(|text| (*key, text))),
                    )
                    .collect::<Vec<_>>();
                if matches!(policy, Policy::Auto) {
                    assert!(
                        records.iter().any(|(key, _)| library.contains(key)),
                        "{diagnostics:?}"
                    );
                }
                for (key, text) in records {
                    assert!(
                        serde_json::from_str::<serde_json::Value>(text).is_ok(),
                        "{key}: {text}"
                    );
                }
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
        kind: CandidateKind::FinalIterate,
        primal: vec![2.0],
        objective: Some(8.0),
        row_dual: Some(vec![0.0]),
        bound_dual: Some((vec![0.0], vec![0.0])),
        reduced_costs: None,
        slacks: None,
    });
    let (report, _) = p.finish(
        report,
        &tolerances(),
        ObjectiveSense::Minimize,
        &Analysis::NONE,
        UNUSED,
    );
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
    assert_eq!(
        report.preprocessing.unwrap().dimensions,
        dimensions(2, 2, 1, 1)
    );
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
fn off_is_identity_and_only_tape_edits_invalidate_native_reuse() {
    let mut a = Pipeline::new(
        Box::new(Mixed::new()),
        &[1.0, 3.0],
        &Policy::Off,
        &tolerances(),
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
    let off = |oracle: Mixed| {
        Pipeline::new(
            Box::new(oracle),
            &[1.0, 3.0],
            &Policy::Off,
            &tolerances(),
            execution(),
            None,
            stamp(),
            1000,
        )
        .unwrap()
        .native_compatibility()
        .layout
    };
    // A tape edit changes the facts' structure and invalidates native reuse.
    let mut edited = Mixed::new();
    edited.facts.key = ContentHash::from_bytes([7; 32]);
    edited.facts.structure = ContentHash::from_bytes([7; 32]);
    assert_ne!(a.native_compatibility().layout, off(edited));
    // Values the facts consumed change their identity, but without an applied pass they
    // shape no native coordinate: a value-only rebind keeps native reuse (Plan 22 Y5c).
    let mut rebound = Mixed::new();
    rebound.facts.key = ContentHash::from_bytes([8; 32]);
    rebound.facts.values = BTreeMap::from([(id(9), 2.0f64.to_bits())]);
    assert_eq!(a.native_compatibility().layout, off(rebound));
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
        kind: CandidateKind::FinalIterate,
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
    assert!(report.validation_failure().is_some());
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
fn maximization_and_original_warm_seed_preserve_conventions() {
    let mut source = Mixed::new();
    source.sign = -1.0;
    let warm = WarmStart {
        origin: None,
        compatibility: stamp(),
        payload: WarmPayload::Nlp {
            primal: vec![2.0, 2.0],
            bounds: Some((vec![0.0; 2], vec![0.0; 2])),
            rows: Some(vec![4.0, 0.0]),
            barrier: None,
            working: None,
        },
    };
    let mut pipeline = Pipeline::new(
        Box::new(source),
        &[1.0, 3.0],
        &Policy::Off,
        &tolerances(),
        execution(),
        Some(&warm),
        stamp(),
        1000,
    )
    .unwrap();
    assert!(pipeline.warm().is_some());
    let transport = pipeline.take_oracle().unwrap();
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
        kind: CandidateKind::FinalIterate,
        primal: vec![2.0, 2.0],
        objective: Some(8.0),
        row_dual: Some(vec![4.0, 0.0]),
        bound_dual: Some((vec![0.0; 2], vec![0.0; 2])),
        reduced_costs: None,
        slacks: None,
    });
    let (report, _) = pipeline.finish(
        report,
        &tolerances(),
        ObjectiveSense::Maximize,
        &Analysis::NONE,
        UNUSED,
    );
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

#[test]
fn normalization_callbacks_and_original_duals_round_trip() {
    let mut source = Mixed::new();
    source.normalization = Some(pse_math::normalization::Normalization {
        variables: vec![2.0, 4.0],
        rows: vec![10.0, 5.0],
        objective: 8.0,
    });
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
    let mut pipeline = Pipeline::new(
        Box::new(source),
        &[2.0, 2.0],
        &Policy::Off,
        &tolerances(),
        execution(),
        Some(&warm),
        stamp(),
        1000,
    )
    .unwrap();
    assert_eq!(pipeline.initial(), &[1.0, 0.5]);
    let mut oracle = pipeline.take_oracle().unwrap();
    assert_eq!(oracle.objective(&[1.0, 0.5]).unwrap(), 1.0);
    let mut v = vec![0.0; 2];
    oracle.gradient(&[1.0, 0.5], &mut v).unwrap();
    assert_eq!(v, vec![1.0, 2.0]);
    oracle.constraints(&[1.0, 0.5], &mut v).unwrap();
    assert_eq!(v, vec![0.7, 1.6]);
    let mut j = vec![0.0; 4];
    oracle.jacobian(&[1.0, 0.5], &mut j).unwrap();
    assert_eq!(j, vec![0.2, 1.6, 0.4, 3.2]);
    oracle
        .hessian(&[1.0, 0.5], 3.0, &[5.0, 7.0], &mut v)
        .unwrap();
    assert!((v[0] - 14.2).abs() < 1e-12 && (v[1] - 56.8).abs() < 1e-12);
    let mut report = SolveReport::new(
        Backend::Ipopt,
        oracle.contract(),
        NativeTermination {
            code: 0,
            name: "coordinate-unit-test".into(),
            message: None,
            category: Termination::Success,
            assurance: Assurance::LocalStationary,
        },
        &execution(),
    );
    report.candidate = Some(Candidate {
        kind: CandidateKind::FinalIterate,
        primal: vec![1.0, 0.5],
        objective: Some(1.0),
        row_dual: Some(vec![-5.0, 0.0]),
        bound_dual: Some((vec![0.0; 2], vec![0.0; 2])),
        reduced_costs: None,
        slacks: None,
    });
    let (report, _) = pipeline.finish(
        report,
        &tolerances(),
        ObjectiveSense::Minimize,
        &Analysis::NONE,
        UNUSED,
    );
    let c = report.candidate.unwrap();
    assert_eq!(c.primal, vec![2.0, 2.0]);
    assert_eq!(c.objective, Some(8.0));
    assert_eq!(c.row_dual, Some(vec![-4.0, 0.0]));
    assert!(report.quality.unwrap().feasible());
    assert_eq!(
        report.observation.unwrap().stationarity,
        Some(vec![0.0, 0.0])
    );
}
#[test]
fn presolve_certificate_respects_each_bound_budget() {
    let source = || {
        let mut s = Mixed::new();
        s.linear_second = true;
        s.bounds[1] = (100.0, 100.0);
        s.facts.affine[1] = Some(AffineRow {
            entries: BTreeMap::from([(0, 1.0), (1, -1.0)]),
            constant: 0.0,
        });
        s
    };
    let mut allowed = tolerances();
    allowed.rows[1] = 500.0;
    let p = Pipeline::new(
        Box::new(source()),
        &[2.0, 2.0],
        &Policy::Auto,
        &allowed,
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    assert!(p.report().proof.is_none());
    assert!(!p.report().effective.enabled);
    let mut distinct = tolerances();
    distinct.rows[0] = 500.0;
    let p = Pipeline::new(
        Box::new(source()),
        &[2.0, 2.0],
        &Policy::Auto,
        &distinct,
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    assert!(p.report().proof.is_some());
    assert_eq!(p.report().proof.as_ref().unwrap().rows, vec![id(3), id(4)]);
    assert_eq!(
        p.terminal_report(ObjectiveSense::Minimize)
            .unwrap()
            .unwrap()
            .termination
            .category,
        Termination::Infeasible
    );
}

#[derive(Debug)]
struct PropagationFixedRow(Mixed);
impl NlpOracle for PropagationFixedRow {
    fn contract(&self) -> &OracleContract {
        &self.0.contract
    }
    fn presolve_facts(&self) -> Option<&Facts> {
        Some(&self.0.facts)
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.0.bounds
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.0.j.matrix().symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        Some(self.0.h.matrix().symbolic())
    }
    fn objective(&mut self, _: &[f64]) -> Result<f64, ProblemError> {
        Ok(0.0)
    }
    fn gradient(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.fill(0.0);
        Ok(())
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.copy_from_slice(&[x[0] * x[0], x[1] - x[2], x[1] * x[1] + x[2] * x[2]]);
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.copy_from_slice(&[2.0 * x[0], 1.0, 2.0 * x[1], -1.0, 2.0 * x[2]]);
        Ok(())
    }
    fn hessian(
        &mut self,
        _: &[f64],
        _: f64,
        l: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out.copy_from_slice(&[2.0 * l[0], 2.0 * l[2], 2.0 * l[2]]);
        Ok(())
    }
}
fn propagation_fixed_row() -> PropagationFixedRow {
    let mut m = Mixed::new();
    m.contract.variables = (0..3)
        .map(|i| Variable {
            id: id(i + 1),
            lower: 1.0,
            upper: 3.0,
        })
        .collect();
    m.contract.rows = vec![id(4), id(5), id(6)];
    m.bounds = vec![(4.0, 4.0), (0.0, 0.0), (8.0, 8.0)];
    m.j = jacobian(3, &[(0, 0), (1, 1), (2, 1), (1, 2), (2, 2)]);
    m.h = hessian(3, &[(0, 0), (1, 1), (2, 2)]);
    m.facts.affine = vec![
        None,
        Some(AffineRow {
            entries: BTreeMap::from([(1, 1.0), (2, -1.0)]),
            constant: 0.0,
        }),
        None,
    ];
    m.facts.tapes = vec![
        FbbtTape {
            ops: vec![Op::Var(0), Op::PowInt(0, 2)],
        },
        FbbtTape {
            ops: vec![Op::Var(1), Op::Var(2), Op::Sub(0, 1)],
        },
        FbbtTape {
            ops: vec![Op::Opaque],
        },
    ];
    m.facts.complete = vec![true, true, false];
    m.facts.row_sources = vec![vec![]; 3];
    m.facts.objective_linear = vec![true; 3];
    PropagationFixedRow(m)
}
#[test]
fn automatic_presolve_retains_original_when_propagation_leaves_constant_nonlinear_rows() {
    let tolerance = Tolerances {
        variables: vec![1e-8; 3],
        rows: vec![1e-8; 3],
        integrality: 1e-8,
    };
    let mut pipeline = Pipeline::new(
        Box::new(propagation_fixed_row()),
        &[1.5, 1.5, 1.5],
        &Policy::Auto,
        &tolerance,
        execution(),
        None,
        stamp(),
        1000,
    )
    .unwrap();
    assert_eq!(pipeline.report().dimensions, Dimensions::identity(3, 3));
    assert!(
        pipeline
            .report()
            .diagnostics
            .contains_key("structure.declined")
    );
    assert!(pipeline.report().passes.values().all(|p| !p.applied));
    let mut oracle = pipeline.take_oracle().unwrap();
    crate::validate_nlp(&oracle, pse_kernels::DerivativeOrder::Second).unwrap();
    let mut values = vec![0.0; 3];
    oracle.constraints(&[2.0, 2.0, 2.0], &mut values).unwrap();
    assert_eq!(values, vec![4.0, 0.0, 8.0]);
    let required = Policy::Explicit {
        options: PresolveOptions {
            enabled: true,
            linear_eq_reduction: true,
            fbbt: true,
            ..PresolveOptions::defaults()
        },
        required: BTreeSet::from([Pass::AffineElimination, Pass::Fbbt]),
    };
    assert!(matches!(
        Pipeline::new(
            Box::new(propagation_fixed_row()),
            &[1.5, 1.5, 1.5],
            &required,
            &tolerance,
            execution(),
            None,
            stamp(),
            1000
        ),
        Err(ProblemError::Structural { .. })
    ));
}
