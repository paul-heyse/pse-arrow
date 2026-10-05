// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual native application controls for the observed Schur bridge.
use super::*;
use crate::solve::{NativeStorageScope, WorkAdmission, WorkEvidence};
use std::sync::atomic::{AtomicU64, Ordering};
#[derive(Clone, Copy, Debug)]
enum ProbeFailure {
    Domain,
    Work,
}

#[derive(Debug)]
struct Quadratic {
    contract: crate::OracleContract,
    jac: faer::sparse::SparseColMat<usize, f64>,
    hess: faer::sparse::SparseColMat<usize, f64>,
    separator: crate::SolveSeparator,
    singular_ff: bool,
    hessian_calls: Arc<AtomicU64>,
    gradient_calls: Arc<AtomicU64>,
    probe_calls: Arc<AtomicU64>,
    fail_probe: Option<ProbeFailure>,
}
impl Quadratic {
    fn new(mode: HessianMode, singular_ff: bool, fixed: bool) -> Self {
        Self::sized(mode, singular_ff, fixed, 4)
    }
    fn sized(mode: HessianMode, singular_ff: bool, fixed: bool, n: usize) -> Self {
        let mut variables: Vec<_> = (0..n)
            .map(|index| crate::Variable {
                id: crate::solver_tests::id(u8::try_from(index + 1).unwrap()),
                lower: -1.,
                upper: 1.,
            })
            .collect();
        if singular_ff {
            variables[0].lower = f64::NEG_INFINITY;
            variables[0].upper = f64::INFINITY;
        }
        if fixed {
            variables[0].lower = 0.;
            variables[0].upper = 0.;
        }
        Self {
            contract: crate::OracleContract {
                identity: pse_ids::ContentHash::from_bytes([91; 32]),
                variables,
                rows: vec![crate::solver_tests::id(u8::try_from(n + 1).unwrap())],
                derivatives: if mode == HessianMode::Exact {
                    pse_kernels::DerivativeOrder::Second
                } else {
                    pse_kernels::DerivativeOrder::First
                },
                smoothness: pse_kernels::DerivativeOrder::Second,
            },
            jac: faer::sparse::SparseColMat::try_new_from_triplets(
                1,
                n,
                &(0..n)
                    .map(|col| faer::sparse::Triplet::new(0, col, 1.))
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
            hess: faer::sparse::SparseColMat::try_new_from_triplets(
                n,
                n,
                &(0..n)
                    .map(|col| faer::sparse::Triplet::new(col, col, 1.))
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
            separator: crate::SolveSeparator {
                variables: vec![],
                rows: vec![0],
            },
            singular_ff,
            hessian_calls: Arc::new(AtomicU64::new(0)),
            gradient_calls: Arc::new(AtomicU64::new(0)),
            probe_calls: Arc::new(AtomicU64::new(0)),
            fail_probe: None,
        }
    }
}
impl NlpOracle for Quadratic {
    fn contract(&self) -> &crate::OracleContract {
        &self.contract
    }
    fn solve_separator(&self) -> Option<&crate::SolveSeparator> {
        Some(&self.separator)
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.jac.symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        (self.contract.derivatives == pse_kernels::DerivativeOrder::Second)
            .then(|| self.hess.symbolic())
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &[(1., 1.)]
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        Ok(x.iter()
            .enumerate()
            .filter(|(col, _)| !self.singular_ff || *col != 0)
            .map(|(_, value)| value * value * 0.5)
            .sum())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.gradient_calls.fetch_add(1, Ordering::SeqCst);
        if x.windows(2).any(|pair| (pair[0] - pair[1]).abs() > 1e-12) {
            self.probe_calls.fetch_add(1, Ordering::SeqCst);
            if let Some(failure) = self.fail_probe {
                return Err(match failure {
                    ProbeFailure::Domain => pse_math::MathError::Domain {
                        source_id: crate::solver_tests::id(1),
                        requirement: "finite difference probe envelope",
                    }
                    .into(),
                    ProbeFailure::Work => ProblemError::Limit {
                        kind: crate::LimitKind::Work,
                        detail: "finite difference probe allowance".into(),
                    },
                });
            }
        }
        out.copy_from_slice(x);
        if self.singular_ff {
            out[0] = 0.;
        }
        Ok(())
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out[0] = if self.singular_ff {
            x[0]
        } else {
            x.iter().sum()
        };
        Ok(())
    }
    fn jacobian(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.fill(if self.singular_ff { 0. } else { 1. });
        out[0] = 1.;
        Ok(())
    }
    fn hessian(
        &mut self,
        _: &[f64],
        weight: f64,
        _: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        assert_eq!(
            self.contract.derivatives,
            pse_kernels::DerivativeOrder::Second
        );
        self.hessian_calls.fetch_add(1, Ordering::SeqCst);
        out.fill(weight);
        if self.singular_ff {
            out[0] = 0.;
        }
        Ok(())
    }
}
fn bounded() -> Settings {
    let mut settings = Settings::default();
    settings.linear.bounded_storage_max_dimension = None;
    settings.linear.scaling = feral::scaling::ScalingStrategy::Mc64Symmetric;
    settings.linear.ordering = feral::symbolic::OrderingMethod::Amd;
    settings
}
fn run(
    session: &mut Session,
    oracle: Quadratic,
    mode: HessianMode,
    settings: &Settings,
    mut controls: Controls,
    execution: Execution,
) -> SolveReport {
    controls.hessian = mode;
    controls.foreign_bytes = Some(64 << 20);
    let n = oracle.contract.variables.len();
    session
        .solve(
            Box::new(oracle),
            &vec![0.; n],
            ObjectiveSense::Minimize,
            &controls,
            &ResolvedAccuracy::verification(),
            settings,
            execution,
            &Tolerances {
                variables: vec![1e-8; n],
                rows: vec![1e-8],
                integrality: 1e-8,
            },
            None,
            crate::solver_tests::stamp(Backend::Pounce),
        )
        .unwrap()
}
fn integer(report: &SolveReport, name: &str) -> i64 {
    let Metric::Integer(value) = report.metrics[name] else {
        panic!("integer native metric {name}");
    };
    value
}
#[test]
fn actual_schur_assembled_profiles_and_typed_mc64_quality_reach_factors() {
    for mode in [
        HessianMode::Exact,
        HessianMode::Partitioned,
        HessianMode::FiniteDifference,
    ] {
        let oracle = Quadratic::sized(mode, false, false, 12);
        let hessian_calls = oracle.hessian_calls.clone();
        let report = run(
            &mut Session::new(),
            oracle,
            mode,
            &bounded(),
            Controls::default(),
            crate::solver_tests::execution(),
        );
        assert_eq!(
            report.termination.category,
            Termination::Success,
            "{mode:?}: {report:?}"
        );
        assert_eq!(
            report.metrics["linear.schur.actual_use"],
            Metric::Bool(true)
        );
        assert!(integer(&report, "linear.observed.schur.f.factor_attempts") > 0);
        assert!(integer(&report, "linear.observed.schur.s.factor_attempts") > 0);
        assert_eq!(
            hessian_calls.load(Ordering::SeqCst) > 0,
            mode == HessianMode::Exact
        );
        assert_eq!(
            report.metrics["linear.storage.complete_extent"],
            Metric::Bool(true)
        );
    }
    let mut settings = bounded();
    settings.linear.scaling = feral::scaling::ScalingStrategy::Mc64Symmetric;
    settings.linear.increase_quality = false;
    settings.linear.refine = false;
    let report = run(
        &mut Session::new(),
        Quadratic::new(HessianMode::Exact, false, false),
        HessianMode::Exact,
        &settings,
        Controls::default(),
        crate::solver_tests::execution(),
    );
    assert_eq!(
        report.termination.category,
        Termination::Success,
        "{report:?}"
    );
    assert_eq!(
        report.metrics["linear.schur.actual_use"],
        Metric::Bool(true)
    );
    assert!(
        integer(&report, "linear.observed.schur.f.factor_attempts") > 0
            && integer(&report, "linear.observed.schur.s.factor_attempts") > 0
    );
    let profile: serde_json::Value =
        serde_json::from_str(&report.provenance["feral.effective"]).unwrap();
    assert_eq!(profile["scaling"], "mc64_symmetric");
    assert_eq!(profile["increase_quality"], false);
    assert_eq!(profile["refine"], false);
}
#[test]
fn actual_schur_layout_consumes_fixed_removal_and_explicit_relax_bounds() {
    for relax in [false, true] {
        let mut controls = Controls::default();
        controls.options.insert(
            "fixed_variable_treatment".into(),
            OptionValue::Text(
                if relax {
                    "relax_bounds"
                } else {
                    "make_parameter"
                }
                .into(),
            ),
        );
        let report = run(
            &mut Session::new(),
            Quadratic::new(HessianMode::Exact, false, true),
            HessianMode::Exact,
            &bounded(),
            controls,
            crate::solver_tests::execution(),
        );
        assert_eq!(
            report.termination.category,
            Termination::Success,
            "{report:?}"
        );
        assert!(report.callback_failure().is_none());
        assert!(!report.evidence.callback.terminal_failure);
        let layout: serde_json::Value =
            serde_json::from_str(&report.provenance["linear.native_layout"]).unwrap();
        assert_eq!(layout["relax_bounds"], relax);
        assert_eq!(
            layout["fixed_removed"],
            if relax {
                serde_json::json!([])
            } else {
                serde_json::json!([0])
            }
        );
        assert_eq!(
            layout["x_original"].as_array().unwrap().len(),
            if relax { 4 } else { 3 }
        );
        assert_eq!(
            report.metrics["linear.schur.actual_use"],
            Metric::Bool(true)
        );
    }
}
#[test]
fn actual_schur_unsuitable_separator_and_failed_ff_use_monolithic_disjointly() {
    let mut unsuitable = Quadratic::new(HessianMode::Exact, false, false);
    unsuitable.separator.variables = vec![0, 1, 2, 3];
    let report = run(
        &mut Session::new(),
        unsuitable,
        HessianMode::Exact,
        &bounded(),
        Controls::default(),
        crate::solver_tests::execution(),
    );
    assert_eq!(
        report.termination.category,
        Termination::Success,
        "{report:?}"
    );
    assert_eq!(
        report.metrics["linear.schur.actual_use"],
        Metric::Bool(false)
    );
    assert_eq!(
        integer(&report, "linear.observed.schur.f.factor_attempts"),
        0
    );
    assert!(integer(&report, "linear.observed.monolithic.factor_attempts") > 0);
    let failed = run(
        &mut Session::new(),
        Quadratic::new(HessianMode::Exact, true, false),
        HessianMode::Exact,
        &bounded(),
        Controls::default(),
        crate::solver_tests::execution(),
    );
    assert_eq!(
        failed.termination.category,
        Termination::Success,
        "{failed:?}"
    );
    assert!(
        integer(&failed, "linear.observed.schur.f.failed_factor_attempts") > 0,
        "{failed:?}"
    );
    assert!(integer(&failed, "linear.observed.monolithic.factor_attempts") > 0);
    assert_eq!(
        failed.evidence.work.factorizations,
        Some(
            u64::try_from(
                integer(&failed, "linear.observed.schur.f.factor_attempts")
                    + integer(&failed, "linear.observed.schur.s.factor_attempts")
                    + integer(&failed, "linear.observed.monolithic.factor_attempts")
            )
            .unwrap()
        )
    );
    assert_eq!(
        failed.metrics["linear.schur.path_reason"],
        Metric::Text("schur-numerical-failure".into())
    );
}
#[derive(Clone, Copy, Debug)]
enum Gate {
    Allow,
    Storage,
    Cancel,
}
#[derive(Debug)]
struct Admission {
    gate: Gate,
    allocation: Arc<()>,
    storage: AtomicU64,
    factors: AtomicU64,
    observed: AtomicU64,
    evaluations: AtomicU64,
    observed_evaluations: AtomicU64,
}
impl Admission {
    fn new(gate: Gate) -> Arc<Self> {
        Arc::new(Self {
            gate,
            allocation: Arc::new(()),
            storage: 0.into(),
            factors: 0.into(),
            observed: 0.into(),
            evaluations: 0.into(),
            observed_evaluations: 0.into(),
        })
    }
}
impl WorkAdmission for Admission {
    fn retain_storage(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        Some(self.allocation.clone())
    }
    fn admit(&self, work: WorkEvidence) -> Result<(), ProblemError> {
        self.factors
            .fetch_add(work.factorizations.unwrap(), Ordering::SeqCst);
        self.evaluations
            .fetch_add(work.evaluations.unwrap(), Ordering::SeqCst);
        Ok(())
    }
    fn observe(&self, work: WorkEvidence) -> Result<(), ProblemError> {
        self.observed
            .fetch_add(work.factorizations.unwrap(), Ordering::SeqCst);
        self.observed_evaluations
            .fetch_add(work.evaluations.unwrap(), Ordering::SeqCst);
        Ok(())
    }
    fn admit_storage(
        &self,
        scope: NativeStorageScope,
        _: &str,
        _: usize,
        _: bool,
    ) -> Result<(), ProblemError> {
        if scope == NativeStorageScope::Linear {
            self.storage.fetch_add(1, Ordering::SeqCst);
            match self.gate {
                Gate::Allow => {}
                Gate::Storage => {
                    return Err(ProblemError::memory(
                        "native pool refused before allocation",
                    ));
                }
                Gate::Cancel => return Err(ProblemError::Cancelled),
            }
        }
        Ok(())
    }
}
#[test]
fn actual_schur_storage_cancel_and_contract_abort_without_monolithic_fallback() {
    for gate in [Gate::Storage, Gate::Cancel] {
        let admission = Admission::new(gate);
        let mut execution = crate::solver_tests::execution();
        execution.work_admission = Some(admission.clone());
        let report = run(
            &mut Session::new(),
            Quadratic::new(HessianMode::Exact, false, false),
            HessianMode::Exact,
            &bounded(),
            Controls::default(),
            execution,
        );
        assert_ne!(report.termination.category, Termination::Success);
        assert!(report.validation_failure().is_some());
        assert!(admission.storage.load(Ordering::SeqCst) > 0);
        assert_eq!(admission.factors.load(Ordering::SeqCst), 0);
        assert_eq!(admission.observed.load(Ordering::SeqCst), 0);
        assert_eq!(
            integer(&report, "linear.observed.monolithic.factor_attempts"),
            0
        );
        assert!(
            second_opinion_profiles(&Controls::default(), &bounded(), &report, true)
                .unwrap()
                .is_empty()
        );
    }
    let mut invalid = Quadratic::new(HessianMode::Exact, false, false);
    invalid.separator.variables = vec![99];
    let report = run(
        &mut Session::new(),
        invalid,
        HessianMode::Exact,
        &bounded(),
        Controls::default(),
        crate::solver_tests::execution(),
    );
    assert!(matches!(
        report.validation_failure(),
        Some(ProblemError::Contract(_))
    ));
    assert_eq!(
        integer(&report, "linear.observed.monolithic.factor_attempts"),
        0
    );
}
#[test]
fn actual_native_storage_owner_is_deduplicated_and_retained_through_teardown() {
    let mut session = Session::new();
    let mut previous = None;
    for _ in 0..8 {
        let admission = Admission::new(Gate::Allow);
        let ledger = Arc::downgrade(&admission);
        let token = Arc::downgrade(&admission.allocation);
        let mut execution = crate::solver_tests::execution();
        execution.work_admission = Some(admission.clone());
        let report = run(
            &mut session,
            Quadratic::new(HessianMode::Exact, false, false),
            HessianMode::Exact,
            &bounded(),
            Controls {
                reuse: ReusePolicy::AllowRebuild,
                ..Default::default()
            },
            execution,
        );
        assert_eq!(
            report.termination.category,
            Termination::Success,
            "{report:?}"
        );
        drop(admission);
        assert!(
            ledger.upgrade().is_none(),
            "completed task ledger must be released"
        );
        assert!(token.upgrade().is_some(), "actual allocation remains owned");
        if let Some(old) = previous {
            assert!(
                std::sync::Weak::<()>::upgrade(&old).is_none(),
                "replaced allocation owner must release"
            );
        }
        previous = Some(token);
    }
    assert_eq!(session.retained_foreign_allowance(), Some(64 << 20));
    let last = previous.unwrap();
    drop(session);
    assert!(last.upgrade().is_none());
}

#[test]
fn actual_partitioned_width_degradation_and_fd_reuse_consume_first_only_oracle() {
    let mut settings = Settings::default();
    settings.partitioned.max_element = 1;
    let oracle = Quadratic::new(HessianMode::Partitioned, false, false);
    let hessian = oracle.hessian_calls.clone();
    let report = run(
        &mut Session::new(),
        oracle,
        HessianMode::Partitioned,
        &settings,
        Controls::default(),
        crate::solver_tests::execution(),
    );
    assert_eq!(
        report.termination.category,
        Termination::Success,
        "{report:?}"
    );
    let statistics = report.pounce_statistics.as_ref().unwrap();
    assert!(statistics.partitioned_diagonal_elements > 0);
    assert_eq!(statistics.partitioned_dense_elements, 0);
    assert_eq!(hessian.load(Ordering::SeqCst), 0);
    let mut observations = Vec::new();
    for reuse in [0., 1.] {
        let mut settings = Settings::default();
        settings.finite_difference.reuse_tolerance = reuse;
        let oracle = Quadratic::new(HessianMode::FiniteDifference, false, false);
        let probes = oracle.probe_calls.clone();
        let gradients = oracle.gradient_calls.clone();
        let report = run(
            &mut Session::new(),
            oracle,
            HessianMode::FiniteDifference,
            &settings,
            Controls::default(),
            crate::solver_tests::execution(),
        );
        assert_eq!(
            report.termination.category,
            Termination::Success,
            "{report:?}"
        );
        assert_eq!(
            integer(&report, "callback.gradient.calls"),
            i64::try_from(gradients.load(Ordering::SeqCst)).unwrap()
        );
        assert!(probes.load(Ordering::SeqCst) > 0);
        observations.push(probes.load(Ordering::SeqCst));
    }
    assert!(
        observations[1] < observations[0],
        "actual native reuse must suppress derivative probes: {observations:?}"
    );
}
#[test]
fn actual_fd_probe_failure_never_substitutes_a_cached_derivative() {
    let mut oracle = Quadratic::new(HessianMode::FiniteDifference, false, false);
    oracle.fail_probe = Some(ProbeFailure::Domain);
    let probes = oracle.probe_calls.clone();
    let gradients = oracle.gradient_calls.clone();
    let hessian = oracle.hessian_calls.clone();
    let admission = Admission::new(Gate::Allow);
    let mut execution = crate::solver_tests::execution();
    execution.work_admission = Some(admission.clone());
    let report = run(
        &mut Session::new(),
        oracle,
        HessianMode::FiniteDifference,
        &Settings::default(),
        Controls::default(),
        execution,
    );
    assert_eq!(
        report.termination.category,
        Termination::Panic,
        "{report:?}"
    );
    assert_eq!(report.termination.name, "rust.unwind");
    assert_eq!(report.termination.code, i64::MIN);
    assert!(matches!(
        report.callback_failure(),
        Some(ProblemError::Math(pse_math::MathError::Domain {
            requirement: "finite difference probe envelope",
            ..
        }))
    ));
    assert!(report.validation_failure().is_some());
    assert!(report.candidate.is_none());
    assert!(report.warm_start.is_none());
    assert!(
        second_opinion_profiles(
            &Controls {
                hessian: HessianMode::FiniteDifference,
                ..Default::default()
            },
            &Settings::default(),
            &report,
            true
        )
        .unwrap()
        .is_empty()
    );
    assert!(probes.load(Ordering::SeqCst) > 0);
    assert!(report.evidence.callback.trial_rejections > 0);
    assert_eq!(
        integer(&report, "callback.gradient.calls"),
        i64::try_from(gradients.load(Ordering::SeqCst)).unwrap()
    );
    assert_eq!(hessian.load(Ordering::SeqCst), 0);
    assert_eq!(
        admission.evaluations.load(Ordering::SeqCst),
        admission.observed_evaluations.load(Ordering::SeqCst)
    );
    assert_eq!(
        report.evidence.work.evaluations,
        Some(admission.observed_evaluations.load(Ordering::SeqCst))
    );
}
#[test]
fn actual_fd_terminal_probe_stop_preserves_original_limit() {
    let mut oracle = Quadratic::new(HessianMode::FiniteDifference, false, false);
    oracle.fail_probe = Some(ProbeFailure::Work);
    let probes = oracle.probe_calls.clone();
    let admission = Admission::new(Gate::Allow);
    let mut execution = crate::solver_tests::execution();
    execution.work_admission = Some(admission.clone());
    let report = run(
        &mut Session::new(),
        oracle,
        HessianMode::FiniteDifference,
        &Settings::default(),
        Controls::default(),
        execution,
    );
    assert_eq!(report.termination.category, Termination::Limit);
    assert!(
        matches!(report.callback_failure(),Some(ProblemError::Limit {kind:crate::LimitKind::Work,detail}) if detail=="finite difference probe allowance")
    );
    assert!(probes.load(Ordering::SeqCst) > 0);
    assert!(report.candidate.is_none());
    assert!(report.warm_start.is_none());
    assert!(
        second_opinion_profiles(
            &Controls {
                hessian: HessianMode::FiniteDifference,
                ..Default::default()
            },
            &Settings::default(),
            &report,
            true
        )
        .unwrap()
        .is_empty()
    );
    assert_eq!(
        admission.evaluations.load(Ordering::SeqCst),
        admission.observed_evaluations.load(Ordering::SeqCst)
    );
    assert_eq!(
        report.evidence.work.evaluations,
        Some(admission.observed_evaluations.load(Ordering::SeqCst))
    );
}
