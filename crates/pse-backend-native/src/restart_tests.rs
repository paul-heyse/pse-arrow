// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! NLP warm restarts through the shared runner (L-N3, L-N4, F07): the interior-point restart
//! profile, the final barrier value carried by a seed, and the POUNCE working set passed
//! through the presolve pipeline under its own transformation.
use crate::{
    NlpOracle, OracleContract, ProblemError, Variable,
    execution::{self, BackendSettings, LINKED, Nlp, Retained, Step},
    presolve::Policy,
    quality::Tolerances,
    solve::*,
    solver_tests::id,
};
use pse_ids::ContentHash;
use pse_kernels::DerivativeOrder;
use pse_math::{binding::ObjectiveSense, normalization::Normalization};

/// Euclidean projection of `target` onto the unit simplex: minimize ‖x − t‖² subject to
/// Σx = 1 and 0 ≤ x ≤ `upper`. Several bounds are active at the optimum.
#[derive(Debug)]
pub(crate) struct Simplex {
    contract: OracleContract,
    target: Vec<f64>,
    bounds: Vec<(f64, f64)>,
    jacobian: faer::sparse::SparseColMat<usize, f64>,
    hessian: faer::sparse::SparseColMat<usize, f64>,
    normalization: Normalization,
}
impl Simplex {
    pub(crate) fn new(target: &[f64], upper: f64, objective_scale: f64) -> Self {
        let n = target.len();
        let variables = (0..n)
            .map(|i| Variable {
                id: id(10 + u8::try_from(i).unwrap()),
                lower: 0.0,
                upper,
            })
            .collect();
        Self {
            contract: OracleContract {
                identity: ContentHash::from_bytes([7; 32]),
                variables,
                rows: vec![id(9)],
                derivatives: DerivativeOrder::Second,
                smoothness: DerivativeOrder::Second,
            },
            target: target.to_vec(),
            bounds: vec![(1.0, 1.0)],
            jacobian: faer::sparse::SparseColMat::try_new_from_triplets(
                1,
                n,
                &(0..n)
                    .map(|j| faer::sparse::Triplet::new(0, j, 1.0))
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
            hessian: faer::sparse::SparseColMat::try_new_from_triplets(
                n,
                n,
                &(0..n)
                    .map(|j| faer::sparse::Triplet::new(j, j, 1.0))
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
            normalization: Normalization {
                variables: vec![1.0; n],
                rows: vec![1.0],
                objective: objective_scale,
            },
        }
    }
}
impl NlpOracle for Simplex {
    fn normalization(&self) -> Option<&Normalization> {
        Some(&self.normalization)
    }
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
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        Ok(x.iter()
            .zip(&self.target)
            .map(|(x, t)| (x - t) * (x - t))
            .sum())
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out[0] = x.iter().sum();
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        for ((o, x), t) in out.iter_mut().zip(x).zip(&self.target) {
            *o = 2.0 * (x - t);
        }
        Ok(())
    }
    fn jacobian(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out.fill(1.0);
        Ok(())
    }
    fn hessian(
        &mut self,
        _: &[f64],
        weight: f64,
        _: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out.fill(2.0 * weight);
        Ok(())
    }
}

pub(crate) const TARGET: [f64; 6] = [0.9, 0.5, -0.3, -0.2, 0.25, -0.5];
pub(crate) const PERTURBED: [f64; 6] = [0.92, 0.47, -0.31, -0.18, 0.27, -0.52];

pub(crate) fn tolerances(n: usize) -> Tolerances {
    Tolerances {
        variables: vec![1e-8; n],
        rows: vec![1e-8],
        integrality: 1e-8,
    }
}
/// One run of `backend` through the shared NLP runner.
pub(crate) fn run(
    backend: Backend,
    settings: &BackendSettings,
    oracle: Simplex,
    warm: Option<&WarmStart>,
    presolve: &Policy,
) -> SolveReport {
    let n = oracle.contract.variables.len();
    let controls = Controls {
        reuse: ReusePolicy::Fresh,
        start: if warm.is_some() {
            StartPolicy::Explicit
        } else {
            StartPolicy::NoPriorStart
        },
        ..Controls::default()
    };
    // Cold and warm solutions are compared at 1e-6; use the explicit
    // verification budgets rather than ordinary engineering allowances.
    let accuracy = ResolvedAccuracy::verification();
    let normalization = oracle.normalization.clone();
    let tolerances = tolerances(n);
    let initial = vec![1.0 / n as f64; n];
    execution::nlp(
        Step {
            snapshot: &execution::Snapshot::observe(&LINKED),
            structure: None,
            adapter: LINKED.get(backend).unwrap(),
            settings,
            controls: &controls,
            accuracy: &accuracy,
            execution: Execution::new(Default::default(), &controls),
            tolerances: &tolerances,
            normalization: &normalization,
            compatibility: Compatibility {
                layout: ContentHash::from_bytes([2; 32]),
                profile: ContentHash::from_bytes([4; 32]),
                data: ContentHash::from_bytes([3; 32]),
                backend,
            },
            warm,
        },
        &mut Retained::default(),
        Nlp {
            oracle: Box::new(oracle),
            initial: &initial,
            presolve,
            intent: SolveIntent::Optimize,
            sense: ObjectiveSense::Minimize,
            limit: 1 << 20,
            analysis: execution::Analysis::for_intent(SolveIntent::Optimize),
        },
    )
    .unwrap()
}
/// Native iterations of a report.
#[cfg(all(feature = "ipopt", feature = "pounce"))]
pub(crate) fn iterations(report: &SolveReport) -> i64 {
    #[cfg(feature = "pounce")]
    if let Some(statistics) = &report.pounce_statistics {
        return i64::from(statistics.iteration_count);
    }
    report
        .events
        .iter()
        .filter_map(|e| match e.values.get("iteration") {
            Some(Metric::Integer(i)) => Some(*i),
            _ => None,
        })
        .max()
        .unwrap()
}
fn solved(report: &SolveReport) -> bool {
    report.termination.category == Termination::Success
        && report.quality.as_ref().is_some_and(|q| q.feasible())
}

#[cfg(feature = "ipopt")]
#[test]
fn interior_point_warm_profile_recorded() {
    let settings = BackendSettings::Ipopt(Default::default());
    // An objective scale of 4 exercises the barrier's transport through normalization.
    let cold = run(
        Backend::Ipopt,
        &settings,
        Simplex::new(&TARGET, 1.0, 4.0),
        None,
        &Policy::Auto,
    );
    assert!(solved(&cold), "{:?}", cold.termination);
    assert!(cold.evidence.restart.is_none());
    let seed = cold.warm_start.clone().unwrap();
    let WarmPayload::Nlp {
        barrier: Some(barrier),
        bounds: Some(_),
        rows: Some(_),
        ..
    } = &seed.payload
    else {
        panic!("{:?}", seed.payload);
    };
    // The seed carries the last iteration's barrier, observed in the intermediate callback,
    // in authored objective units.
    let native = cold
        .events
        .iter()
        .rev()
        .find(|e| e.phase == "ipopt.iteration")
        .and_then(|e| match e.values.get("barrier") {
            Some(Metric::Real(mu)) => Some(*mu),
            _ => None,
        })
        .unwrap();
    assert!((barrier - native * 4.0).abs() <= 1e-12 * barrier.abs());
    let warm = run(
        Backend::Ipopt,
        &settings,
        Simplex::new(&PERTURBED, 1.0, 4.0),
        Some(&seed),
        &Policy::Auto,
    );
    assert!(solved(&warm), "{:?}", warm.termination);
    let applied = warm.evidence.restart.unwrap();
    assert_eq!(applied.profile, WarmRestart::default());
    let mu = applied.mu_init.unwrap();
    assert!((mu - native).abs() <= 1e-12 * native, "{mu} {native}");
    assert_eq!(warm.options["mu_init"], OptionValue::Real(mu));
    assert_eq!(
        warm.options["warm_start_bound_push"],
        OptionValue::Real(1e-9)
    );
    assert_eq!(
        warm.options["warm_start_mult_bound_push"],
        OptionValue::Real(1e-9)
    );
    assert_eq!(
        warm.options["warm_start_init_point"],
        OptionValue::Bool(true)
    );
    // The start receipt records the restart among the seed's transformations.
    let mut receipt = StartReceipt {
        previous_attempt: Some(0),
        seed: Some(seed.clone()),
        sparse_seed: None,
        transformations: vec![],
        submitted: false,
    };
    receipt.record(
        &warm,
        Simplex::new(&PERTURBED, 1.0, 4.0).normalization.key(),
    );
    assert!(receipt.submitted);
    assert!(
        receipt
            .transformations
            .contains(&SeedTransformation::InteriorRestart(applied)),
        "{:?}",
        receipt.transformations
    );
    assert!(
        receipt.snapshot()["transformations"]
            .to_string()
            .contains("interior_restart")
    );
    // A stated barrier replaces the seed's, and the raw options cannot bypass the profile.
    let stated = BackendSettings::Ipopt(crate::ipopt::Settings {
        restart: WarmRestart {
            barrier: RestartBarrier::Value {
                value: pse_model::scalars::Tolerance::try_new(1e-3).unwrap(),
            },
            ..WarmRestart::default()
        },
        ..Default::default()
    });
    let explicit = run(
        Backend::Ipopt,
        &stated,
        Simplex::new(&PERTURBED, 1.0, 4.0),
        Some(&seed),
        &Policy::Auto,
    );
    assert_eq!(explicit.evidence.restart.unwrap().mu_init, Some(1e-3));
    // A primal-only seed is not an interior-point restart.
    let primal = WarmStart {
        payload: WarmPayload::primal(cold.candidate.as_ref().unwrap().primal.clone()),
        ..seed
    };
    let plain = run(
        Backend::Ipopt,
        &settings,
        Simplex::new(&PERTURBED, 1.0, 4.0),
        Some(&primal),
        &Policy::Auto,
    );
    assert!(plain.evidence.restart.is_none());
    assert!(!plain.options.contains_key("mu_init"));
}

#[cfg(all(feature = "ipopt", feature = "pounce"))]
#[test]
fn warm_restart_reduces_iterations_on_perturbed_case() {
    for (backend, settings) in [
        (Backend::Ipopt, BackendSettings::Ipopt(Default::default())),
        (Backend::Pounce, BackendSettings::Pounce(Default::default())),
    ] {
        let seed = run(
            backend,
            &settings,
            Simplex::new(&TARGET, 1.0, 1.0),
            None,
            &Policy::Auto,
        )
        .warm_start
        .unwrap();
        let cold = run(
            backend,
            &settings,
            Simplex::new(&PERTURBED, 1.0, 1.0),
            None,
            &Policy::Auto,
        );
        let warm = run(
            backend,
            &settings,
            Simplex::new(&PERTURBED, 1.0, 1.0),
            Some(&seed),
            &Policy::Auto,
        );
        for report in [&cold, &warm] {
            assert!(solved(report), "{backend:?} {:?}", report.termination);
        }
        let (a, b) = (
            &cold.candidate.as_ref().unwrap().primal,
            &warm.candidate.as_ref().unwrap().primal,
        );
        assert!(
            a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-6),
            "{a:?} {b:?}"
        );
        assert!(warm.evidence.restart.is_some(), "{backend:?}");
        assert!(
            iterations(&warm) < iterations(&cold),
            "{backend:?}: warm {} cold {}",
            iterations(&warm),
            iterations(&cold)
        );
    }
}

#[cfg(feature = "pounce")]
#[test]
fn sqp_working_set_restart_reaches_runtime() {
    let settings = BackendSettings::Pounce(crate::pounce::Settings {
        method: crate::pounce::Method::ActiveSetSqp,
        ..Default::default()
    });
    let first = run(
        Backend::Pounce,
        &settings,
        Simplex::new(&TARGET, 1.0, 1.0),
        None,
        &Policy::Auto,
    );
    assert!(solved(&first), "{:?}", first.termination);
    let seed = first.warm_start.clone().unwrap();
    let WarmPayload::Nlp {
        working: Some(working),
        barrier,
        ..
    } = &seed.payload
    else {
        panic!("{:?}", seed.payload);
    };
    assert!(
        barrier.is_none(),
        "an active-set iterate has no barrier value"
    );
    let transformation = first.preprocessing.as_ref().unwrap().transformation;
    assert_eq!(working.transformation, transformation);
    // The published seed names each status in snake_case, never by Rust `Debug` (F30).
    let snapshot = serde_json::to_value(seed.snapshot()).unwrap();
    let active = &snapshot["payload"]["working_set"]["active"];
    let statuses = active["bounds"]
        .as_array()
        .unwrap()
        .iter()
        .chain(active["constraints"].as_array().unwrap())
        .collect::<Vec<_>>();
    assert!(!statuses.is_empty());
    for status in statuses {
        assert!(
            ["inactive", "at_lower", "at_upper", "fixed", "equality"]
                .contains(&status.as_str().unwrap()),
            "{status}"
        );
    }
    // The same transformation: the pipeline passes the working set to the native SQP.
    let second = run(
        Backend::Pounce,
        &settings,
        Simplex::new(&PERTURBED, 1.0, 1.0),
        Some(&seed),
        &Policy::Auto,
    );
    assert!(solved(&second), "{:?}", second.termination);
    assert!(second.evidence.start_submitted && second.evidence.working_set_submitted);
    let transfer = WorkingSetTransfer {
        transformation,
        retained: true,
    };
    assert_eq!(
        second.preprocessing.as_ref().unwrap().working_set,
        Some(transfer)
    );
    let mut receipt = StartReceipt {
        previous_attempt: Some(0),
        seed: Some(seed.clone()),
        sparse_seed: None,
        transformations: vec![],
        submitted: false,
    };
    receipt.record(&second, Normalization::identity(6, 1).key());
    assert!(
        receipt
            .transformations
            .contains(&SeedTransformation::WorkingSet(transfer)),
        "{:?}",
        receipt.transformations
    );
    // The chained seed keeps a working set for the step after.
    assert!(matches!(
        &second.warm_start.as_ref().unwrap().payload,
        WarmPayload::Nlp {
            working: Some(_),
            ..
        }
    ));
    // Another transformation (a changed bound): the working set is recorded as not
    // retained and never reaches the native solver; the portable primal-dual seed still does.
    let moved = run(
        Backend::Pounce,
        &settings,
        Simplex::new(&PERTURBED, 0.95, 1.0),
        Some(&seed),
        &Policy::Auto,
    );
    assert!(solved(&moved), "{:?}", moved.termination);
    assert!(moved.evidence.start_submitted && !moved.evidence.working_set_submitted);
    assert_eq!(
        moved.preprocessing.as_ref().unwrap().working_set,
        Some(WorkingSetTransfer {
            transformation,
            retained: false
        })
    );
}
