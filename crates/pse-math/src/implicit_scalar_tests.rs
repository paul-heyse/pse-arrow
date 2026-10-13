// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::guarded::{PreparedBody, Stage};
use symbolica::atom::Atom;

fn run(
    lower: f64,
    upper: f64,
    tolerance: f64,
    iterations: usize,
    evaluations: usize,
    residual: impl FnMut(f64) -> Result<f64, MathError>,
) -> Result<f64, MathError> {
    bracketed(
        SemanticId::NIL,
        lower,
        upper,
        tolerance,
        1e-10,
        iterations,
        evaluations,
        Duration::from_secs(1),
        &Arc::new(AtomicBool::new(false)),
        residual,
    )
}

#[test]
fn endpoints_are_exact_and_reversed_or_tiny_same_sign_intervals_refuse() {
    for expected in [0.0, 1.0] {
        let mut calls = 0;
        assert_eq!(
            run(0.0, 1.0, 1e-8, 10, 100, |x| {
                calls += 1;
                Ok(x - expected)
            })
            .unwrap(),
            expected
        );
        assert_eq!(calls, 3); // Two preflight endpoints and independent final acceptance.
    }
    let mut calls = 0;
    assert!(matches!(
        run(1.0, 0.0, 1e-8, 10, 100, |_| {
            calls += 1;
            Ok(0.0)
        }),
        Err(MathError::Contract(_))
    ));
    assert_eq!(calls, 0);
    assert!(matches!(
        run(1.0, 1.0 + f64::EPSILON, 1e-8, 10, 100, |_| Ok(1.0)),
        Err(MathError::Domain { .. })
    ));
    assert_eq!(run(1.0, 1.0, 1e-8, 10, 100, |_| Ok(0.0)).unwrap(), 1.0);
}

#[test]
fn nonfinite_endpoints_values_and_domain_holes_keep_typed_refusals() {
    assert!(matches!(
        run(f64::NEG_INFINITY, 1.0, 1e-8, 10, 100, Ok),
        Err(MathError::Contract(_))
    ));
    for value in [f64::NAN, f64::INFINITY] {
        assert!(matches!(
            run(0.0, 1.0, 1e-8, 10, 100, |_| Ok(value)),
            Err(MathError::Domain { .. })
        ));
    }
    let witness = SemanticId::from_bytes([179; 16]);
    let result = run(-1.0, 1.0, 1e-8, 10, 100, |x| {
        if x.abs() < 0.5 {
            Err(MathError::Domain {
                source_id: witness,
                requirement: "authored hole",
            })
        } else {
            Ok(x)
        }
    });
    assert!(
        matches!(result, Err(MathError::Domain { source_id, requirement: "authored hole" }) if source_id == witness)
    );
}

#[test]
fn cancellation_deadline_and_actual_evaluation_allowance_stop_callbacks() {
    let cancel = Arc::new(AtomicBool::new(true));
    let mut calls = 0;
    assert!(matches!(
        bracketed(
            SemanticId::NIL,
            0.0,
            1.0,
            1e-8,
            1e-8,
            10,
            100,
            Duration::from_secs(1),
            &cancel,
            |_| {
                calls += 1;
                Ok(0.0)
            }
        ),
        Err(MathError::Cancelled)
    ));
    assert_eq!(calls, 0);
    cancel.store(false, Ordering::Release);
    assert!(matches!(
        bracketed(
            SemanticId::NIL,
            0.0,
            1.0,
            1e-8,
            1e-8,
            10,
            100,
            Duration::from_secs(1),
            &cancel,
            |_| {
                calls += 1;
                cancel.store(true, Ordering::Release);
                Ok(0.0)
            }
        ),
        Err(MathError::Cancelled)
    ));
    assert_eq!(calls, 1);
    assert!(matches!(
        run(0.0, 1.0, 1e-8, 10, 1, |_| {
            calls += 1;
            Ok(1.0)
        }),
        Err(MathError::Limit("scalar residual evaluations"))
    ));
    assert_eq!(calls, 2);
    let cancel = Arc::new(AtomicBool::new(false));
    assert!(matches!(
        bracketed(
            SemanticId::NIL,
            0.0,
            1.0,
            1e-8,
            1e-8,
            10,
            100,
            Duration::from_millis(1),
            &cancel,
            |_| {
                std::thread::sleep(Duration::from_millis(3));
                Ok(0.0)
            }
        ),
        Err(MathError::Scope(pse_kernels::ProviderError::Deadline))
    ));
}

#[test]
fn iteration_exhaustion_is_an_error_and_width_never_grants_residual_acceptance() {
    assert!(matches!(
        run(0.0, 2.0, 1e-15, 1, 100, |x| Ok(x * x - 2.0)),
        Err(MathError::Limit("scalar root iterations"))
    ));
    assert!(matches!(
        run(-1.0, 1.0, 1e-8, 100, 1000, |x| Ok(if x < 0.2 {
            -1.0
        } else {
            1.0
        })),
        Err(MathError::Domain {
            requirement: "scalar root did not satisfy original residual budget",
            ..
        })
    ));
    let mut visited = Vec::new();
    assert!(matches!(
        run(0.0, 1.0, 2.0, 10, 100, |x| {
            visited.push(x);
            Ok(1e20 * (x - 0.2))
        }),
        Err(MathError::Domain {
            requirement: "scalar root did not satisfy original residual budget",
            ..
        })
    ));
    assert_eq!(visited, [0.0, 1.0, 0.0, 1.0, 0.5]); // Unevaluated midpoint is checked independently.
    let root = run(-1.0, 1.0, 1e-12, 100, 1000, |x| Ok((x - 0.3).powi(3))).unwrap();
    assert!((root - 0.3).abs() < 1e-10);
}

fn prepared(expression: Atom) -> PreparedBody {
    PreparedBody::new(
        2,
        3,
        vec![2],
        vec![Stage::Block {
            expressions: vec![expression],
            outputs: vec![2],
            source: SemanticId::NIL,
        }],
        DerivativeOrder::Second,
    )
    .unwrap()
}

#[test]
fn admission_requires_actual_polynomial_formals_and_preserves_original_guards() {
    crate::initialize().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let x = crate::library::formal(0).unwrap();
    let p = crate::library::formal(1).unwrap();
    assert!(BracketedRoot::new(&prepared(x.clone().pow(Atom::num(5)) - p), &cancel).is_ok());
    assert!(BracketedRoot::new(&prepared(x.clone().sin()), &cancel).is_err());
    assert!(BracketedRoot::new(&prepared(x.clone().pow(Atom::num(-1))), &cancel).is_err());
    let guarded = PreparedBody::new(
        2,
        3,
        vec![2],
        vec![
            Stage::Require {
                argument: 0,
                condition: crate::guarded::Condition::Nonzero,
                order: DerivativeOrder::Value,
                source: SemanticId::NIL,
                lineage: None,
            },
            Stage::Block {
                expressions: vec![x],
                outputs: vec![2],
                source: SemanticId::NIL,
            },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    assert!(BracketedRoot::new(&guarded, &cancel).is_err());
}

#[test]
fn real_worker_uses_original_residual_and_derivatives_and_does_not_certify_uniqueness() {
    crate::initialize().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let x = crate::library::formal(0).unwrap();
    let body = prepared(x.clone().pow(Atom::num(2)) - crate::library::formal(1).unwrap());
    let solver = BracketedRoot::new(&body, &cancel).unwrap();
    let compiled = body
        .compile(
            &[0],
            &[0, 1],
            DerivativeOrder::Second,
            crate::library::Optimization::default(),
            crate::jets::EvaluationLimits::default(),
            &cancel,
        )
        .unwrap();
    let problem = Arc::new(
        Problem::new(
            SemanticId::NIL,
            ContentHash::from_bytes([178; 32]),
            vec![Unknown {
                id: SemanticId::NIL,
                lower: 0.0,
                upper: 3.0,
            }],
            vec![SemanticId::NIL],
            1,
            Arc::new(compiled),
            1024,
        )
        .unwrap(),
    );
    let options = Options {
        start: vec![1.0],
        variable_nominals: vec![1.0],
        variable_tolerance: vec![1e-12],
        residual_tolerance: vec![1e-10],
        iterations: 100,
        time_limit: Duration::from_secs(2),
        derivative_tolerance: 1e-10,
    };
    let mut worker = solver.worker();
    for p in [4.0, 1.0, 4.0] {
        let root = worker
            .solve(problem.clone(), &[p], &options, &cancel)
            .unwrap();
        assert!((root[0] - p.sqrt()).abs() < 1e-10);
        let jet = problem
            .derivatives(&[p], &root, DerivativeOrder::Second, &options, &cancel)
            .unwrap();
        assert!((jet.jacobian[0] - 0.5 / p.sqrt()).abs() < 1e-10);
    }
    let root = worker
        .solve(problem.clone(), &[0.0], &options, &cancel)
        .unwrap();
    assert!(
        problem
            .derivatives(&[0.0], &root, DerivativeOrder::First, &options, &cancel)
            .is_err()
    );
    // A sign-changing interval may contain several roots. It provides a checked proposal,
    // never an exactly-one-root assertion; the common selector must still establish choice.
    let proposal = run(-2.0, 2.0, 1e-12, 100, 1000, |x| {
        Ok(x * (x - 1.0) * (x + 1.0))
    })
    .unwrap();
    assert!((proposal * (proposal - 1.0) * (proposal + 1.0)).abs() < 1e-10);
}
