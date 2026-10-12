// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Recognized polynomial acceleration uses Symbolica's coefficient extraction and certified roots.
use super::*;
use symbolica::{
    atom::{Atom, AtomCore},
    domains::rational::{Q, Rational},
    poly::univariate::UnivariatePolynomial,
};
/// A bounded accelerator for one scalar residual polynomial of degree at most three.
/// Original residual guards and branch intervals remain the authority for acceptance.
#[derive(Debug)]
pub struct CubicRoots {
    coefficients: symbolica::evaluate::ExpressionEvaluator<f64>,
    parameters: usize,
    bytes: usize,
}
#[derive(Debug)]
struct CubicWorker {
    coefficients: symbolica::evaluate::ExpressionEvaluator<f64>,
    parameters: usize,
}
impl CubicRoots {
    /// Admit only an exact polynomial in the first (unknown) coordinate.
    pub fn new(
        body: &crate::guarded::PreparedBody,
        limits: crate::jets::EvaluationLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        if body.output_count() != 1 || body.input_count() == 0 {
            return Err(MathError::Contract(
                "cubic accelerator requires one residual and one unknown".into(),
            ));
        }
        let expression = body.expression(0).ok_or_else(|| {
            MathError::Contract("cubic accelerator requires a library polynomial projection".into())
        })?;
        let x = crate::library::formal(0)?;
        let powers = (0..=3)
            .map(|i| x.clone().pow(Atom::num(i)))
            .collect::<Vec<_>>();
        let mut coefficients = vec![Atom::num(0); 4];
        for (power, coefficient) in expression.coefficient_list::<u16>(std::slice::from_ref(&x)) {
            let Some(index) = powers.iter().position(|p| *p == power) else {
                return Err(MathError::Contract(
                    "unrecognized polynomial degree for cubic accelerator".into(),
                ));
            };
            coefficients[index] += coefficient;
        }
        let formals = (1..body.input_count())
            .map(crate::library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        let layout = crate::jets::JetLayout::new(vec![], DerivativeOrder::Value, limits)?;
        let evaluator = crate::library::bounded_evaluator(
            SemanticId::NIL,
            &coefficients,
            &formals,
            &layout,
            &[],
            crate::library::Optimization::default(),
            cancel,
            limits,
            limits.operations,
            0,
            &mut Default::default(),
        )?;
        let storage = crate::library::storage(&evaluator)?;
        let bytes = storage
            .numeric_entries
            .checked_mul(size_of::<f64>())
            .and_then(|bytes| bytes.checked_add(storage.instruction_bytes))
            .and_then(|bytes| bytes.checked_add(size_of::<CubicWorker>()))
            .ok_or(MathError::Limit("cubic coefficient storage extent"))?;
        Ok(Self {
            coefficients: evaluator,
            parameters: formals.len(),
            bytes,
        })
    }
}
impl InnerSolver for CubicRoots {
    fn minimum_order(&self) -> DerivativeOrder {
        DerivativeOrder::Value
    }
    fn identity(&self) -> ContentHash {
        solver_identity("symbolica.cubic-roots.v2")
    }
    fn worker(&self) -> Box<dyn InnerWorker> {
        Box::new(CubicWorker {
            coefficients: self.coefficients.clone(),
            parameters: self.parameters,
        })
    }
    fn retained_bytes(&self) -> usize {
        self.bytes
    }
    fn worker_bytes(&self) -> usize {
        self.bytes
    }
}
impl InnerWorker for CubicWorker {
    fn solve(
        &mut self,
        problem: Arc<Problem>,
        parameters: &[f64],
        options: &Options,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Vec<f64>, MathError> {
        problem.validate_options(options)?;
        if parameters.len() != self.parameters
            || problem.unknowns.len() != 1
            || parameters.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract("cubic input contract".into()));
        }
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        let started = std::time::Instant::now();
        let mut coefficients = [0.0; 4];
        self.coefficients.evaluate(parameters, &mut coefficients);
        if coefficients.iter().any(|v| !v.is_finite()) {
            return Err(MathError::Domain {
                source_id: problem.id,
                requirement: "finite cubic coefficients",
            });
        }
        let mut coefficients = coefficients
            .into_iter()
            .map(|v| Rational::try_from(v).map_err(|e| MathError::Library(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?;
        while coefficients.last().is_some_and(Rational::is_zero) {
            coefficients.pop();
        }
        if coefficients.len() <= 1 {
            return Err(MathError::Domain {
                source_id: problem.id,
                requirement: "nondegenerate polynomial root",
            });
        }
        let formal = crate::library::formal(0)?;
        let symbolica::atom::AtomView::Var(x) = formal.as_view() else {
            return Err(MathError::Contract("root formal".into()));
        };
        let polynomial = UnivariatePolynomial::from_coefficients(
            &Q,
            coefficients,
            Arc::new(symbolica::poly::PolyVariable::Symbol(x.get_symbol())),
        );
        let roots = real_points(&polynomial, cancel)?;
        if started.elapsed() > options.time_limit {
            return Err(MathError::Limit("cubic root isolation time"));
        }
        let mut accepted = Vec::new();
        for root in roots {
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            if started.elapsed() > options.time_limit {
                return Err(MathError::Limit("cubic root isolation time"));
            }
            let point = vec![root];
            match problem.verify(parameters, &point, options, cancel) {
                Ok(()) => accepted.push(point),
                Err(MathError::Domain { .. }) => {}
                Err(error) => return Err(error),
            }
        }
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        if started.elapsed() > options.time_limit {
            return Err(MathError::Limit("cubic root isolation time"));
        }
        if accepted.len() != 1 {
            return Err(MathError::Domain {
                source_id: problem.id,
                requirement: "exactly one original-residual-verified root in the declared branch interval",
            });
        }
        accepted
            .pop()
            .ok_or_else(|| MathError::Contract("selected cubic root absent".into()))
    }
}

/// Real intervals are complete for the exact rational polynomial. The public
/// refiner requires non-root endpoints; near-critical separation can violate
/// that premise, so those intervals use Symbolica's certified general fallback.
fn real_points(
    polynomial: &UnivariatePolynomial<Q>,
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<f64>, MathError> {
    if cancel.load(Ordering::Acquire) {
        return Err(MathError::Cancelled);
    }
    let intervals = polynomial.isolate_real_root_intervals();
    if cancel.load(Ordering::Acquire) {
        return Err(MathError::Cancelled);
    }
    let precision = Rational::from((1_i64, 1_i64 << 54));
    let mut points = Vec::with_capacity(intervals.len());
    let mut fallback = false;
    for (a, b, _) in intervals {
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        if a == b {
            points.push(a.to_f64());
        } else if polynomial.evaluate(&a).is_zero() || polynomial.evaluate(&b).is_zero() {
            fallback = true;
            break;
        } else {
            let (lower, upper) = polynomial.refine_root_interval((a, b), &precision);
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            if lower.to_f64() != upper.to_f64() {
                fallback = true;
                break;
            }
            points.push(((lower + upper) / Rational::from(2)).to_f64());
        }
    }
    if fallback || points.iter().any(|point| !point.is_finite()) {
        points.clear();
        let roots = polynomial.isolate_real_roots();
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        for (root, _) in roots {
            points.push(root.refined(&precision).enclosure().center().re.to_f64());
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
        }
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn polynomial(coefficients: Vec<Rational>) -> UnivariatePolynomial<Q> {
        crate::initialize().unwrap();
        let formal = crate::library::formal(0).unwrap();
        let symbolica::atom::AtomView::Var(x) = formal.as_view() else {
            unreachable!()
        };
        UnivariatePolynomial::from_coefficients(
            &Q,
            coefficients,
            Arc::new(symbolica::poly::PolyVariable::Symbol(x.get_symbol())),
        )
    }

    #[test]
    fn real_interval_route_matches_certified_roots_including_near_critical_endpoints() {
        let fixtures = [
            vec![(-6).into(), 11.into(), (-6).into(), 1.into()],
            vec![(-2).into(), 0.into(), 1.into()],
            vec![(-2).into(), 1.into()],
            vec![0.into(), 0.into(), 0.into(), 1.into()],
            vec![
                0.into(),
                Rational::from((-1, 1_i64 << 60)),
                0.into(),
                1.into(),
            ],
            vec![(-1).into(), 3.into(), (-3).into(), 1.into()],
        ];
        let cancel = Arc::new(AtomicBool::new(false));
        let precision = Rational::from((1_i64, 1_i64 << 54));
        for coefficients in fixtures {
            let poly = polynomial(coefficients);
            let mut actual = real_points(&poly, &cancel).unwrap();
            let mut reference = poly
                .isolate_real_roots()
                .into_iter()
                .map(|(root, _)| root.refined(&precision).enclosure().center().re.to_f64())
                .collect::<Vec<_>>();
            actual.sort_by(f64::total_cmp);
            reference.sort_by(f64::total_cmp);
            assert_eq!(actual.len(), reference.len());
            for (actual, reference) in actual.into_iter().zip(reference) {
                assert!((actual - reference).abs() <= 1e-14 * reference.abs().max(1.0));
            }
        }
    }

    fn admitted() -> (CubicRoots, Arc<Problem>, Options, Arc<AtomicBool>) {
        use crate::guarded::{PreparedBody, Stage};
        let cancel = Arc::new(AtomicBool::new(false));
        crate::initialize().unwrap();
        let body = PreparedBody::new(
            2,
            3,
            vec![2],
            vec![Stage::Block {
                expressions: vec![
                    crate::library::formal(0).unwrap().pow(Atom::num(2))
                        - crate::library::formal(1).unwrap(),
                ],
                outputs: vec![2],
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let solver =
            CubicRoots::new(&body, crate::jets::EvaluationLimits::default(), &cancel).unwrap();
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
                ContentHash::from_bytes([137; 32]),
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
            variable_tolerance: vec![1e-10],
            residual_tolerance: vec![1e-10],
            iterations: 10,
            time_limit: Duration::from_secs(2),
            derivative_tolerance: 1e-10,
        };
        (solver, problem, options, cancel)
    }

    #[test]
    fn private_cubic_workers_preserve_original_branch_residual_and_cancellation() {
        let (solver, problem, options, cancel) = admitted();
        assert!(solver.worker_bytes() > size_of::<CubicWorker>());
        assert!(matches!(problem.factor.lock().unwrap().parallel, Par::Seq));
        let mut first = solver.worker();
        let mut second = solver.worker();
        assert_eq!(
            first
                .solve(problem.clone(), &[4.0], &options, &cancel)
                .unwrap(),
            [2.0]
        );
        assert_eq!(
            second
                .solve(problem.clone(), &[1.0], &options, &cancel)
                .unwrap(),
            [1.0]
        );
        assert_eq!(
            first
                .solve(problem.clone(), &[4.0], &options, &cancel)
                .unwrap(),
            [2.0]
        );
        assert!(
            second
                .solve(problem.clone(), &[-1.0], &options, &cancel)
                .is_err()
        );
        assert_eq!(
            first
                .solve(problem.clone(), &[0.0], &options, &cancel)
                .unwrap(),
            [0.0]
        );
        assert!(
            problem
                .derivatives(&[0.0], &[0.0], DerivativeOrder::First, &options, &cancel)
                .is_err()
        );
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            first.solve(problem, &[4.0], &options, &cancel),
            Err(MathError::Cancelled)
        ));
    }
}
