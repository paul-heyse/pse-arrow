// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent analytic oracles exercised through actual foreign solver iterations.
use super::fixtures::near;
use faer::sparse::{SparseColMat, SymbolicSparseColMatRef, Triplet};
use pse_backend_native::{
    NlpOracle, OracleContract, ProblemError, Variable, ipopt, pounce, quality::Tolerances, solve::*,
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::DerivativeOrder;
use pse_math::binding::ObjectiveSense;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Debug, Clone, Copy)]
enum Fault {
    None,
    Trial,
    Fatal,
    Panic,
    Cancel,
}
#[derive(Debug)]
struct Quadratic {
    contract: OracleContract,
    jacobian: SparseColMat<usize, f64>,
    hessian: SparseColMat<usize, f64>,
    fault: Fault,
    fired: Arc<AtomicBool>,
    cancel: Arc<AtomicBool>,
}
impl Quadratic {
    fn new(fault: Fault, cancel: Arc<AtomicBool>, fired: Arc<AtomicBool>) -> Self {
        Self {
            contract: OracleContract {
                identity: ContentHash::from_bytes([9; 32]),
                variables: vec![Variable {
                    id: SemanticId::from_bytes([1; 16]),
                    lower: -100.,
                    upper: 100.,
                }],
                rows: vec![],
                derivatives: DerivativeOrder::Second,
                smoothness: DerivativeOrder::Second,
            },
            jacobian: SparseColMat::try_new_from_triplets(0, 1, &[]).unwrap(),
            hessian: SparseColMat::try_new_from_triplets(1, 1, &[Triplet::new(0, 0, 2.)]).unwrap(),
            fault,
            fired,
            cancel,
        }
    }
}
impl NlpOracle for Quadratic {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> SymbolicSparseColMatRef<'_, usize> {
        self.jacobian.symbolic()
    }
    fn hessian_pattern(&self) -> Option<SymbolicSparseColMatRef<'_, usize>> {
        Some(self.hessian.symbolic())
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &[]
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        // A failure after the valid initial point proves native trial recovery.
        if !matches!(self.fault, Fault::None)
            && (x[0] - 10.).abs() > 1e-9
            && !self.fired.swap(true, Ordering::SeqCst)
        {
            match self.fault {
                Fault::Trial => {
                    return Err(pse_math::MathError::Domain {
                        source_id: self.contract.variables[0].id,
                        requirement: "injected recoverable trial",
                    }
                    .into());
                }
                Fault::Fatal => {
                    return Err(ProblemError::Contract(
                        "injected terminal provider failure".into(),
                    ));
                }
                Fault::Panic => panic!("injected contained native callback panic"),
                Fault::Cancel => {
                    self.cancel.store(true, Ordering::SeqCst);
                    return Err(pse_math::MathError::Cancelled.into());
                }
                Fault::None => {}
            }
        }
        Ok((x[0] - 2.).powi(2))
    }
    fn constraints(&mut self, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
        Ok(())
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out[0] = 2. * (x[0] - 2.);
        Ok(())
    }
    fn jacobian(&mut self, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
        Ok(())
    }
    fn hessian(
        &mut self,
        _: &[f64],
        weight: f64,
        _: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out[0] = 2. * weight;
        Ok(())
    }
}
fn solve(backend: Backend, fault: Fault, controls: &Controls) -> (SolveReport, bool) {
    let cancel = Arc::new(AtomicBool::new(false));
    let fired = Arc::new(AtomicBool::new(false));
    let mut oracle = Quadratic::new(fault, cancel.clone(), fired.clone());
    let execution = Execution::new(cancel, controls);
    let compatibility = Compatibility {
        layout: ContentHash::from_bytes([9; 32]),
        data: ContentHash::from_bytes([8; 32]),
        backend,
    };
    let tolerances = Tolerances {
        variables: vec![1e-6],
        rows: vec![],
        integrality: 1e-8,
    };
    let report = match backend {
        Backend::Ipopt => ipopt::Session::new().solve(
            &mut oracle,
            &[10.],
            ObjectiveSense::Minimize,
            controls,
            execution,
            &tolerances,
            None,
            None,
            compatibility,
        ),
        Backend::Pounce => pounce::Session::new().solve(
            Box::new(oracle),
            &[10.],
            ObjectiveSense::Minimize,
            controls,
            pounce::Method::InteriorPoint,
            Default::default(),
            execution,
            &tolerances,
            None,
            compatibility,
        ),
        _ => unreachable!(),
    }
    .unwrap();
    (report, fired.load(Ordering::SeqCst))
}
#[test]
fn native_nlp_recovers_trial_and_proves_analytic_optimum() {
    for backend in [Backend::Ipopt, Backend::Pounce] {
        for hessian in [HessianMode::Exact, HessianMode::LimitedMemory] {
            let controls = Controls {
                hessian,
                ..Default::default()
            };
            for fault in [Fault::None, Fault::Trial] {
                let (report, fired) = solve(backend, fault, &controls);
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
                assert!(report.quality.as_ref().unwrap().feasible());
                near(report.candidate.as_ref().unwrap().primal[0], 2., 1e-5);
                if matches!(fault, Fault::Trial) {
                    assert!(fired);
                    assert!(
                        report
                            .events
                            .iter()
                            .any(|event| event.values.get("recoverable")
                                == Some(&Metric::Bool(true)))
                    );
                }
            }
        }
    }
}
#[test]
fn native_nlp_terminal_failure_panic_limit_and_after_entry_stop() {
    for backend in [Backend::Ipopt, Backend::Pounce] {
        for (fault, expected) in [
            (Fault::Fatal, Termination::Evaluation),
            (Fault::Panic, Termination::Panic),
            (Fault::Cancel, Termination::Cancelled),
        ] {
            let (report, fired) = solve(backend, fault, &Controls::default());
            assert!(fired, "native trial was never entered");
            assert_eq!(report.termination.category, expected, "{report:?}");
            assert_eq!(report.termination.assurance, Assurance::None, "{report:?}");
        }
        let (report, _) = solve(
            backend,
            Fault::None,
            &Controls {
                iterations: 1,
                ..Default::default()
            },
        );
        assert_eq!(
            report.termination.category,
            Termination::Limit,
            "{report:?}"
        );
        assert_eq!(report.termination.assurance, Assurance::None);
    }
}
