// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The ℓ1 exact-penalty route (ADR-0109, L-N5) through the shared NLP runner.
use crate::{
    NlpOracle, OracleContract, ProblemError, Variable,
    execution::{self, BackendSettings, LINKED, Nlp, Retained, Step},
    pounce::{Method, Settings},
    presolve::Policy,
    quality::Tolerances,
    solve::*,
    solver_tests::id,
};
use pse_ids::ContentHash;
use pse_kernels::DerivativeOrder;
use pse_math::{binding::ObjectiveSense, normalization::Normalization};

/// Rows `x² ∈ bounds[0]` and `x² ∈ bounds[1]` in one variable `x ∈ [-10, 10]`, or with
/// `linear` rows `x + y ∈ bounds[i]` in two variables.
#[derive(Debug)]
struct Rows {
    contract: OracleContract,
    linear: bool,
    bounds: Vec<(f64, f64)>,
    jacobian: faer::sparse::SparseColMat<usize, f64>,
    hessian: faer::sparse::SparseColMat<usize, f64>,
}
impl Rows {
    fn new(linear: bool, bounds: [(f64, f64); 2]) -> Self {
        let n = if linear { 2 } else { 1 };
        let triplets: Vec<_> = (0..2)
            .flat_map(|r| (0..n).map(move |c| faer::sparse::Triplet::new(r, c, 1.0)))
            .collect();
        Self {
            contract: OracleContract {
                identity: ContentHash::from_bytes([5; 32]),
                variables: (0..n)
                    .map(|i| Variable {
                        id: id(20 + u8::try_from(i).unwrap()),
                        lower: -10.0,
                        upper: 10.0,
                    })
                    .collect(),
                rows: vec![id(30), id(31)],
                derivatives: DerivativeOrder::Second,
                smoothness: DerivativeOrder::Second,
            },
            linear,
            bounds: bounds.to_vec(),
            jacobian: faer::sparse::SparseColMat::try_new_from_triplets(2, n, &triplets).unwrap(),
            hessian: faer::sparse::SparseColMat::try_new_from_triplets(
                n,
                n,
                &(0..n)
                    .map(|j| faer::sparse::Triplet::new(j, j, 1.0))
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        }
    }
    fn value(&self, x: &[f64]) -> f64 {
        if self.linear {
            x[0] + x[1]
        } else {
            x[0] * x[0]
        }
    }
}
impl NlpOracle for Rows {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.jacobian.symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        Some(self.hessian.symbolic())
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn objective(&mut self, _: &[f64]) -> Result<f64, ProblemError> {
        Ok(0.0)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.fill(self.value(x));
        Ok(())
    }
    fn gradient(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.fill(0.0);
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if self.linear {
            out.fill(1.0);
        } else {
            out.fill(2.0 * x[0]);
        }
        Ok(())
    }
    fn hessian(
        &mut self,
        _: &[f64],
        _: f64,
        l: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out.fill(if self.linear {
            0.0
        } else {
            2.0 * (l[0] + l[1])
        });
        Ok(())
    }
}
fn run(
    oracle: Rows,
    settings: &BackendSettings,
    options: Options,
) -> Result<SolveReport, ProblemError> {
    run_with(oracle, settings, options, &Policy::Off)
}
fn run_with(
    oracle: Rows,
    settings: &BackendSettings,
    options: Options,
    presolve: &Policy,
) -> Result<SolveReport, ProblemError> {
    let n = oracle.contract.variables.len();
    let controls = Controls {
        options,
        ..Controls::default()
    };
    let accuracy = ResolvedAccuracy::from_policy(&Default::default(), 1e-8)?;
    let tolerances = Tolerances {
        variables: vec![1e-8; n],
        rows: vec![1e-8; 2],
        integrality: 1e-8,
    };
    let initial = vec![1.5; n];
    execution::nlp(
        Step {
            adapter: LINKED.get(Backend::Pounce).unwrap(),
            settings,
            controls: &controls,
            accuracy: &accuracy,
            execution: Execution::new(Default::default(), &controls),
            tolerances: &tolerances,
            normalization: &Normalization::identity(n, 2),
            compatibility: Compatibility {
                layout: ContentHash::from_bytes([2; 32]),
                profile: ContentHash::from_bytes([4; 32]),
                data: ContentHash::from_bytes([3; 32]),
                backend: Backend::Pounce,
            },
            warm: None,
        },
        &mut Retained::default(),
        Nlp {
            oracle: Box::new(oracle),
            initial: &initial,
            presolve,
            intent: SolveIntent::FeasiblePoint,
            sense: ObjectiveSense::Minimize,
            limit: 1 << 20,
        },
    )
}
fn l1() -> BackendSettings {
    BackendSettings::Pounce(Settings {
        method: Method::L1ExactPenalty,
        ..Settings::default()
    })
}

#[test]
fn l1_route_returns_labelled_least_infeasible_point() {
    let infinity = f64::INFINITY;
    // Inequalities no point satisfies, x² ≥ 4 and x² ≤ 1: every point with 1 ≤ x² ≤ 4 has the
    // least ℓ1 violation, 3. Equalities x + y = 1 and x + y = 3: least violation 2.
    for (oracle, least) in [
        (Rows::new(false, [(4.0, infinity), (-infinity, 1.0)]), 3.0),
        (Rows::new(true, [(1.0, 1.0), (3.0, 3.0)]), 2.0),
    ] {
        let rows = oracle.contract.rows.clone();
        let report = run(oracle, &l1(), Options::new()).unwrap();
        assert_eq!(
            report.termination.category,
            Termination::Infeasible,
            "{:?}",
            report.termination
        );
        assert_eq!(
            report.options["l1_exact_penalty_barrier"],
            OptionValue::Bool(true)
        );
        assert!(!report.quality.as_ref().unwrap().feasible());
        let label = report.least_infeasible.as_ref().unwrap();
        assert!(!label.violated.is_empty());
        assert!(label.violated.iter().all(|v| rows.contains(&v.id)));
        let total: f64 = report
            .quality
            .as_ref()
            .unwrap()
            .rows
            .iter()
            .map(|v| v.physical)
            .sum();
        assert!((total - least).abs() < 1e-5, "{total} {least}");
        assert_eq!(
            label.violated.iter().map(|v| v.physical).sum::<f64>(),
            total
        );
    }
    // A feasible model: the exact penalty returns a feasible, unlabelled point.
    let report = run(
        Rows::new(false, [(4.0, infinity), (-infinity, 9.0)]),
        &l1(),
        Options::new(),
    )
    .unwrap();
    assert_eq!(report.termination.category, Termination::Success);
    assert!(report.quality.as_ref().unwrap().feasible());
    assert!(report.least_infeasible.is_none());
    let x = report.candidate.as_ref().unwrap().primal[0];
    assert!((4.0 - 1e-6..=9.0 + 1e-6).contains(&(x * x)), "{x}");
}

#[test]
fn l1_auto_presolve_resolves_off_and_is_recorded() {
    let infinity = f64::INFINITY;
    let infeasible = || Rows::new(false, [(4.0, infinity), (-infinity, 1.0)]);
    // `Auto` lets the system choose: under the relaxed rows it chooses no pass. Were a pass
    // run, interval propagation would certify this model infeasible instead of reaching the
    // least-infeasible point.
    let report = run_with(infeasible(), &l1(), Options::new(), &Policy::Auto).unwrap();
    assert_eq!(report.termination.category, Termination::Infeasible);
    assert!(report.least_infeasible.is_some());
    let record = report.preprocessing.as_ref().unwrap();
    assert!(matches!(record.requested, Policy::Auto));
    assert_eq!(
        record.resolution,
        Some(crate::presolve::Resolution::RelaxedRows)
    );
    assert!(!record.effective.enabled);
    assert!(record.passes.values().all(|p| !p.applied));
    assert!(record.proof.is_none());
    // `Off` is the policy that runs, so nothing is resolved; the interior point keeps `Auto`.
    let off = run_with(infeasible(), &l1(), Options::new(), &Policy::Off).unwrap();
    assert!(off.preprocessing.as_ref().unwrap().resolution.is_none());
    let interior = run_with(
        infeasible(),
        &BackendSettings::Pounce(Settings::default()),
        Options::new(),
        &Policy::Auto,
    )
    .unwrap();
    let record = interior.preprocessing.as_ref().unwrap();
    assert!(record.resolution.is_none() && record.effective.enabled);
    // Explicitly requested passes assume the rows hold and are refused.
    let explicit = Policy::from_native_options(&Options::new(), Default::default()).unwrap();
    let error = run_with(infeasible(), &l1(), Options::new(), &explicit).unwrap_err();
    assert!(matches!(error, ProblemError::Contract(_)), "{error:?}");
}

#[test]
fn l1_never_automatic() {
    // Automatic routing runs POUNCE with its default settings: the interior point.
    assert_eq!(Settings::default().method, Method::InteriorPoint);
    let infeasible = || Rows::new(false, [(4.0, f64::INFINITY), (-f64::INFINITY, 1.0)]);
    for settings in [
        BackendSettings::Default,
        BackendSettings::Pounce(Settings::default()),
    ] {
        let report = run(infeasible(), &settings, Options::new()).unwrap();
        // The interior point ends at its own verdict; no ℓ1 attempt followed it.
        assert_ne!(report.termination.category, Termination::Success);
        for key in [
            "l1_exact_penalty_barrier",
            "l1_fallback_on_restoration_failure",
        ] {
            assert_eq!(report.options[key], OptionValue::Bool(false), "{key}");
        }
    }
    // Raw options can neither switch the penalty on nor enable the automatic ℓ1 retry.
    for key in [
        "l1_exact_penalty_barrier",
        "l1_fallback_on_restoration_failure",
    ] {
        for settings in [BackendSettings::Default, l1()] {
            let error = run(
                infeasible(),
                &settings,
                Options::from([(key.into(), OptionValue::Bool(true))]),
            )
            .unwrap_err();
            assert!(
                matches!(error, ProblemError::Contract(_)),
                "{key}: {error:?}"
            );
        }
    }
    // The explicit method still never enables the retry.
    let report = run(infeasible(), &l1(), Options::new()).unwrap();
    assert_eq!(
        report.options["l1_fallback_on_restoration_failure"],
        OptionValue::Bool(false)
    );
}
