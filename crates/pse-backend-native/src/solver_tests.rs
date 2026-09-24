// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use solve::*;
use std::sync::{Arc, atomic::AtomicBool};
pub(crate) fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
pub(crate) fn contract() -> OracleContract {
    OracleContract {
        identity: ContentHash::from_bytes([1; 32]),
        variables: vec![Variable {
            id: id(1),
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
        }],
        rows: vec![id(2)],
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    }
}
#[cfg(any(feature = "ipopt", feature = "pounce", feature = "kinsol"))]
pub(crate) fn execution() -> Execution {
    Execution::new(Arc::new(AtomicBool::new(false)), &Controls::default())
}
pub(crate) fn stamp(backend: Backend) -> Compatibility {
    Compatibility {
        layout: ContentHash::from_bytes([2; 32]),
        data: ContentHash::from_bytes([3; 32]),
        backend,
    }
}
#[derive(Debug)]
pub(crate) struct Polynomial {
    pub(super) c: OracleContract,
    pub(super) matrix: faer::sparse::SparseColMat<usize, f64>,
    pub(super) bounds: Vec<(f64, f64)>,
    pub(super) fail: bool,
    pub(super) panic: bool,
}
impl Polynomial {
    pub(super) fn new() -> Self {
        Self {
            c: contract(),
            matrix: faer::sparse::SparseColMat::try_new_from_triplets(
                1,
                1,
                &[faer::sparse::Triplet::new(0, 0, 1.0)],
            )
            .unwrap(),
            bounds: vec![(1.0, 1.0)],
            fail: false,
            panic: false,
        }
    }
}
impl NlpOracle for Polynomial {
    fn contract(&self) -> &OracleContract {
        &self.c
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.matrix.symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        Some(self.matrix.symbolic())
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        if self.panic {
            panic!("callback test panic")
        };
        Ok(x[0] * x[0])
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out[0] = 2.0 * x[0];
        if self.fail {
            return Err(ProblemError::Contract("intentional partial failure".into()));
        }
        Ok(())
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out[0] = x[0] * x[0] * x[0];
        Ok(())
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out[0] = 3.0 * x[0] * x[0];
        Ok(())
    }
    fn hessian(
        &mut self,
        x: &[f64],
        w: f64,
        l: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out[0] = 2.0 * w + 6.0 * x[0] * l[0];
        Ok(())
    }
}
impl NleOracle for Polynomial {
    fn contract(&self) -> &OracleContract {
        &self.c
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        out[0] = x[0] * x[0] * x[0] - 1.0;
        Ok(())
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.matrix.symbolic()
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        NlpOracle::jacobian(self, x, out)
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        v: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        out[0] = 3.0 * x[0] * x[0] * v[0];
        Ok(())
    }
}
#[test]
fn feasibility_wrapper_has_constant_objective_and_exact_constraint_hessian() {
    let mut o = assembled::FeasibilityOracle(Box::new(Polynomial::new()));
    assert_eq!(o.objective(&[2.0]).unwrap(), 0.0);
    let mut v = [9.0];
    o.gradient(&[2.0], &mut v).unwrap();
    assert_eq!(v, [0.0]);
    o.hessian(&[2.0], 100.0, &[3.0], &mut v).unwrap();
    assert_eq!(v, [36.0]);
}
#[test]
fn physical_quality_keeps_original_units_and_normalizes_only_ratios() {
    let q = quality::Quality::new(
        vec![
            quality::Violation {
                id: id(1),
                physical: 1e5,
                tolerance: 2e5,
            },
            quality::Violation {
                id: id(2),
                physical: 0.02,
                tolerance: 0.01,
            },
        ],
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(q.normalized_max, 2.0);
    assert!(!q.feasible());
    assert_eq!(q.rows[0].physical, 1e5);
}
#[test]
fn warm_compatibility_is_semantic_and_backend_specific() {
    let mut target = stamp(Backend::Kinsol);
    let start = WarmStart {
        compatibility: target.clone(),
        payload: WarmPayload::Root(vec![1.0]),
    };
    target.data = ContentHash::from_bytes([4; 32]);
    assert!(start.validate(&target).is_ok());
    target.backend = Backend::Ipopt;
    assert!(start.validate(&target).is_err());
    target = stamp(Backend::Kinsol);
    target.layout = ContentHash::from_bytes([5; 32]);
    assert!(start.validate(&target).is_err());
}
#[test]
fn history_is_bounded_and_deadline_is_distinct_from_cancellation() {
    let mut c = Controls::default();
    c.history = 2;
    let e = Execution::new(Arc::new(AtomicBool::new(false)), &c);
    for _ in 0..5 {
        e.progress.push(Event {
            phase: "unit".into(),
            elapsed: std::time::Duration::ZERO,
            values: Default::default(),
        });
    }
    let (v, d) = e.progress.snapshot();
    assert_eq!((v.len(), d), (2, 3));
    let mut timed = e.clone();
    timed.started = std::time::Instant::now() - c.time_limit;
    assert_eq!(timed.stopped(), Some(Termination::TimeLimit));
    e.cancel.store(true, std::sync::atomic::Ordering::Release);
    assert_eq!(e.stopped(), Some(Termination::Cancelled));
}
#[test]
fn semi_domains_include_zero_but_never_fill_the_gap() {
    use pse_math::binding::VariableDomain as D;
    for d in [D::SemiContinuous, D::SemiInteger] {
        assert!(d.contains(0.0, 2.0, 5.0));
        assert!(!d.contains(1.0, 2.0, 5.0));
        assert!(d.contains(3.0, 2.0, 5.0));
        assert!(!d.contains(6.0, 2.0, 5.0));
    }
    assert!(D::SemiContinuous.contains(2.5, 2.0, 5.0));
    assert!(!D::SemiInteger.contains(2.5, 2.0, 5.0));
    assert!(!D::Binary.contains(2.0, -10.0, 10.0));
}

#[cfg(any(feature = "ipopt", feature = "pounce", feature = "highs"))]
#[test]
fn native_extensions_cannot_override_nested_semantics_or_unbound_history_strings() {
    use std::collections::BTreeMap;
    let options = BTreeMap::from([("resto.jac_c_constant".into(), OptionValue::Bool(true))]);
    assert!(reject_reserved(&options, &["jac_c_constant"]).is_err());
    let progress = Progress::new(1);
    progress.push(Event {
        phase: "x".repeat(8192),
        elapsed: std::time::Duration::ZERO,
        values: BTreeMap::new(),
    });
    assert_eq!(progress.snapshot().1, 1);
    assert!(progress.snapshot().0.is_empty());
    let mut controls = Controls::default();
    let before = controls.report_allowance().unwrap();
    controls
        .options
        .insert("option".into(), OptionValue::Text("x".repeat(4000)));
    assert!(controls.report_allowance().unwrap() > before);
    assert!(controls.validate().is_ok());
    controls
        .options
        .insert("option".into(), OptionValue::Text("x".repeat(5000)));
    assert!(controls.validate().is_err());
}
