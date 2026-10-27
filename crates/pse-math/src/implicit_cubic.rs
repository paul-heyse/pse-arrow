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
    limits: crate::jets::EvaluationLimits,
    construction_owner: Option<Arc<dyn crate::construction::ConstructionAdmission>>,
}
#[derive(Debug)]
struct CubicWorker {
    construction_owner: Option<Arc<dyn crate::construction::ConstructionAdmission>>,
    reserved: usize,
    limits: crate::jets::EvaluationLimits,
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
        let inspected = crate::library::bounded_evaluator(
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
        let storage = inspected.facts.storage;
        let bytes = storage
            .numeric_entries
            .checked_mul(size_of::<f64>())
            .and_then(|bytes| bytes.checked_add(storage.instruction_bytes))
            .and_then(|bytes| bytes.checked_add(size_of::<CubicWorker>()))
            .ok_or(MathError::Limit("cubic coefficient storage extent"))?;
        let count = inspected.facts.operations;
        let coefficient_operations = count.additions.checked_add(count.multiplications)
            .and_then(|n| n.checked_add(count.inversions))
            .and_then(|n| n.checked_add(count.function_calls))
            .ok_or(MathError::Limit("cubic coefficient work extent"))?;
        let limits = crate::jets::EvaluationLimits {
            operations: limits.operations.checked_sub(coefficient_operations)
                .ok_or(MathError::Limit("cubic coefficient operations"))?,
            scratch_bytes: limits.scratch_bytes.checked_sub(bytes)
                .ok_or(MathError::Limit("cubic coefficient scratch bytes"))?,
            ..limits
        };
        Ok(Self {
            coefficients: inspected.evaluator,
            parameters: formals.len(),
            bytes,
            limits,
            construction_owner: crate::construction::current(),
        })
    }
}
impl InnerSolver for CubicRoots {
    fn minimum_order(&self) -> DerivativeOrder {
        DerivativeOrder::Value
    }
    fn identity(&self) -> ContentHash {
        solver_identity("symbolica.cubic-roots.v3")
    }
    fn worker(&self) -> Box<dyn InnerWorker> {
        Box::new(CubicWorker {
            limits: self.limits,
            construction_owner: self.construction_owner.clone(),
            reserved: 0,
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
impl CubicWorker {
    fn reserve(&mut self, bytes: usize) -> Result<(), MathError> {
        if bytes > self.limits.scratch_bytes {
            return Err(MathError::Limit("cubic isolation scratch bytes"));
        }
        if bytes > self.reserved {
            if let Some(owner) = &self.construction_owner {
                owner.try_grow(bytes - self.reserved)?;
            }
            self.reserved = bytes;
        }
        Ok(())
    }
}
impl InnerWorker for CubicWorker {
    fn solve(
        &mut self,
        problem: Arc<Problem>,
        parameters: &[f64],
        options: &Options,
        cancel: &pse_diagnostics::Cancellation,
    ) -> Result<Vec<f64>, MathError> {
        let control = cancel;
        let cancel = &control.flag();
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
        // Four finite binary64 dyadics have <=1075-bit rational limbs.
        // Eight limb payloads, Rational owners, vector capacity overlap and
        // construction temporaries fit in this fixed source-derived extent.
        self.reserve(8192)?;
        let limits = crate::jets::EvaluationLimits {
            operations: self.limits.operations.checked_sub(4)
                .ok_or(MathError::Limit("cubic coefficient conversion operations"))?,
            ..self.limits
        };
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
        let mut reserve = |bytes| self.reserve(bytes);
        let roots = real_points(&polynomial, cancel, started, options.time_limit, limits, &mut reserve)?;
        if started.elapsed() > options.time_limit {
            return Err(MathError::Limit("cubic root isolation time"));
        }
        let mut accepted = Vec::new();
        for candidates in roots {
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            if started.elapsed() > options.time_limit {
                return Err(MathError::Limit("cubic root isolation time"));
            }
            for root in candidates {
                check_control(cancel, started, options.time_limit)?;
                let point = vec![root];
                match problem.verify(parameters, &point, options, cancel) {
                    Ok(()) => {
                        check_control(cancel, started, options.time_limit)?;
                        accepted.push(point);
                        break;
                    }
                    Err(error) if matches!(error.instance_cause(), MathError::Domain { .. }) => {}
                    Err(error) => return Err(error),
                }
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

fn check_control(
    cancel: &Arc<AtomicBool>,
    started: std::time::Instant,
    deadline: Duration,
) -> Result<(), MathError> {
    if cancel.load(Ordering::Acquire) {
        return Err(MathError::Cancelled);
    }
    if started.elapsed() >= deadline {
        return Err(MathError::Limit("cubic root isolation time"));
    }
    Ok(())
}

/// Admission for local exact bisection, conversion and temporary rational
/// payload, after the supplier's separately admitted isolation stages.
#[derive(Debug)]
struct BracketEnvelope {
    steps: usize,
    operations: usize,
    bytes: usize,
}
impl BracketEnvelope {
    fn new(polynomial: &UnivariatePolynomial<Q>, a: &Rational, b: &Rational) -> Result<Self, MathError> {
        let checked = || MathError::Limit("cubic bracket extent");
        let endpoint = crate::rational_binary64::bits(a)?.max(crate::rational_binary64::bits(b)?);
        let height = polynomial.coefficients().iter().try_fold(1, |height, coefficient| {
            Ok::<_, MathError>(height.max(crate::rational_binary64::bits(coefficient)?))
        })?;
        // |b-a| < 2^(endpoint+1). After endpoint+1077 bisections
        // width < 2^-1076, strictly less than the minimum distance
        // 2^-1074 between two binary64 nearest-rounding boundaries.
        let steps = endpoint.checked_add(1077).ok_or_else(checked)?;
        // A common denominator da*db followed by k halvings bounds every
        // retained endpoint numerator/denominator by 2*endpoint+k+2 bits.
        let bits = endpoint.checked_mul(2).and_then(|v| v.checked_add(steps))
            .and_then(|v| v.checked_add(2)).ok_or_else(checked)?;
        // Horner at degree <=3 needs at most 4*(height+bits+2)
        // bits in either rational limb before reduction. Extra copies cover
        // midpoint addition, derivative signs and two directed MPFR values.
        let limb = height.checked_add(bits).and_then(|v| v.checked_add(2))
            .and_then(|v| v.checked_mul(8)).ok_or_else(checked)?;
        let bytes = limb.checked_add(7).and_then(|v| v.checked_div(8))
            .and_then(|v| v.checked_mul(128)).and_then(|v| v.checked_add(4096)).ok_or_else(checked)?;
        let operations = steps.checked_mul(48).and_then(|v| v.checked_add(256)).ok_or_else(checked)?;
        Ok(Self { steps, operations, bytes })
    }
}

fn finite_root_candidate(value: f64) -> Result<Vec<f64>, MathError> {
    if !value.is_finite() {
        return Err(MathError::Limit("finite binary64 cubic root representative"));
    }
    Ok(vec![value])
}

fn bracket_candidates(
    polynomial: &UnivariatePolynomial<Q>,
    mut a: Rational,
    mut b: Rational,
    steps: usize,
    cancel: &Arc<AtomicBool>,
    started: std::time::Instant,
    deadline: Duration,
) -> Result<Vec<f64>, MathError> {
    check_control(cancel, started, deadline)?;
    if a == b {
        let value = crate::rational_binary64::nearest(&a)?;
        return finite_root_candidate(value);
    }
    let value = polynomial.evaluate(&a);
    // A different root may occupy a bracket endpoint. Its one-sided sign,
    // rather than treating it as the designated root, retains the open interior.
    let negative = if value.is_zero() {
        polynomial.derivative().evaluate(&a).is_negative()
    } else {
        value.is_negative()
    };
    for _ in 0..=steps {
        check_control(cancel, started, deadline)?;
        let lower = crate::rational_binary64::nearest(&a)?;
        let upper = crate::rational_binary64::nearest(&b)?;
        let midpoint = (&a + &b) / Rational::from(2);
        if lower == upper || lower.next_up() == upper {
            let middle = crate::rational_binary64::nearest(&midpoint)?;
            let mut candidates = Vec::with_capacity(3);
            for point in [middle, lower, upper] {
                if point.is_finite() && !candidates.contains(&point) {
                    candidates.push(point);
                }
            }
            check_control(cancel, started, deadline)?;
            if candidates.is_empty() {
                return Err(MathError::Limit("finite binary64 cubic root representative"));
            }
            return Ok(candidates);
        }
        let value = polynomial.evaluate(&midpoint);
        if value.is_zero() {
            let point = crate::rational_binary64::nearest(&midpoint)?;
            return finite_root_candidate(point);
        }
        if value.is_negative() == negative {
            a = midpoint;
        } else {
            b = midpoint;
        }
    }
    Err(MathError::Contract("exact cubic bisection exceeded derived extent".into()))
}

/// Each returned vector belongs to one distinct exact root. Equal binary64
/// images of different roots retain separate associations for uniqueness.
fn real_points(
    polynomial: &UnivariatePolynomial<Q>,
    cancel: &Arc<AtomicBool>,
    started: std::time::Instant,
    deadline: Duration,
    limits: crate::jets::EvaluationLimits,
    reserve: &mut dyn FnMut(usize) -> Result<(), MathError>,
) -> Result<Vec<Vec<f64>>, MathError> {
    check_control(cancel, started, deadline)?;
    limits.check()?;
    if polynomial.degree() == 0 || polynomial.degree() > 3 {
        return Err(MathError::Contract("cubic isolation degree".into()));
    }
    let mut remaining = limits.operations;
    let mut admit = |step: symbolica::poly::univariate::RealRootIsolationStep| {
        check_control(cancel, started, deadline)?;
        if step.operations > remaining {
            return Err(MathError::Limit("cubic isolation operations"));
        }
        if step.bytes > limits.scratch_bytes {
            return Err(MathError::Limit("cubic isolation scratch bytes"));
        }
        reserve(step.bytes)?;
        remaining -= step.operations;
        Ok(())
    };
    let supplier_error = |error| match error {
        symbolica::poly::univariate::BoundedRootIsolationError::Control(error) => error,
        symbolica::poly::univariate::BoundedRootIsolationError::Extent => MathError::Limit("cubic isolation extent"),
    };
    // Source-owned hooks admit each exact quotient and VAS transformation.
    // Already square-free input bypasses repeated Yun/preferred-GCD work.
    let square_free = polynomial.square_free_part_bounded(&mut admit).map_err(supplier_error)?;
    let intervals = square_free.isolate_real_root_intervals_square_free_bounded(&mut admit).map_err(supplier_error)?;
    if intervals.len() > square_free.degree() {
        return Err(MathError::Contract("cubic isolation root population".into()));
    }
    check_control(cancel, started, deadline)?;
    let envelopes = intervals.iter().map(|(a,b)| BracketEnvelope::new(&square_free,a,b))
        .collect::<Result<Vec<_>, _>>()?;
    let operations = envelopes.iter().try_fold(0_usize, |sum, envelope| sum.checked_add(envelope.operations))
        .ok_or(MathError::Limit("cubic bracket operations"))?;
    if operations > remaining {
        return Err(MathError::Limit("cubic bracket operations"));
    }
    let bytes = envelopes.iter().map(|e| e.bytes).max().unwrap_or(0);
    if bytes > limits.scratch_bytes {
        return Err(MathError::Limit("cubic bracket scratch bytes"));
    }
    reserve(bytes)?;
    let mut points = Vec::with_capacity(intervals.len());
    for ((a,b), envelope) in intervals.into_iter().zip(envelopes) {
        points.push(bracket_candidates(&square_free,a,b,envelope.steps,cancel,started,deadline)?);
    }
    check_control(cancel, started, deadline)?;
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn polynomial(coefficients: Vec<Rational>) -> UnivariatePolynomial<Q> {
        crate::initialize().unwrap();
        let formal = crate::library::formal(0).unwrap();
        let symbolica::atom::AtomView::Var(x) = formal.as_view() else {
            panic!("implicit cubic fixture requires its declared root expression")
        };
        UnivariatePolynomial::from_coefficients(
            &Q,
            coefficients,
            Arc::new(symbolica::poly::PolyVariable::Symbol(x.get_symbol())),
        )
    }

    #[test]
    fn real_interval_route_matches_known_roots_including_near_critical_endpoints() {
        // Explicit factorizations supply the root counts and values independently
        // of the isolation/refinement route used by the production fallback.
        let near_zero = 1.0 / 1_073_741_824.0;
        let fixtures = [
            (
                vec![(-6).into(), 11.into(), (-6).into(), 1.into()],
                vec![1.0, 2.0, 3.0],
            ),
            (
                vec![(-2).into(), 0.into(), 1.into()],
                vec![-2.0_f64.sqrt(), 2.0_f64.sqrt()],
            ),
            (vec![(-2).into(), 1.into()], vec![2.0]),
            (vec![0.into(), 0.into(), 0.into(), 1.into()], vec![0.0]),
            (
                vec![
                    0.into(),
                    Rational::from((-1, 1_i64 << 60)),
                    0.into(),
                    1.into(),
                ],
                vec![-near_zero, 0.0, near_zero],
            ),
            (
                vec![(-1).into(), 3.into(), (-3).into(), 1.into()],
                vec![1.0],
            ),
        ];
        let cancel = Arc::new(AtomicBool::new(false));
        for (coefficients, expected) in fixtures {
            let poly = polynomial(coefficients);
            let mut actual = real_points(&poly, &cancel, std::time::Instant::now(), Duration::from_secs(2), crate::jets::EvaluationLimits::default(), &mut |_| Ok(())).unwrap().into_iter().map(|points| points[0]).collect::<Vec<_>>();
            actual.sort_by(f64::total_cmp);
            assert_eq!(actual.len(), expected.len());
            for (actual, expected) in actual.into_iter().zip(expected) {
                assert!((actual - expected).abs() <= 1e-14 * expected.abs().max(1.0), "actual={actual:?} expected={expected:?}");
            }
        }
    }

    #[test]
    fn cubic_square_free_containment_selects_each_original_branch() {
        crate::initialize().unwrap();
        let x = crate::library::formal(0).unwrap();
        // Independent exact factorization: (3x-8)(2x-3)^2.
        let expression = Atom::num(12) * x.clone().pow(Atom::num(3))
            - Atom::num(68) * x.clone().pow(Atom::num(2))
            + Atom::num(123) * x - Atom::num(72);
        for (lower, upper, expected) in [(1.0, 2.0, 1.5), (2.5, 3.0, 8.0 / 3.0)] {
            let (solver, problem, mut options, control) = admitted_expression(expression.clone(), lower, upper);
            options.start[0] = (lower+upper)/2.0;
            assert_eq!(solver.worker().solve(problem, &[0.0], &options, &control).unwrap(), [expected]);
        }
    }

    #[test]
    fn cubic_bracket_endpoint_neighbours_and_subnormal_progress() {
        let cancel = Arc::new(AtomicBool::new(false));
        let fixtures = [
            (polynomial(vec![(-6).into(),11.into(),(-6).into(),1.into()]), Rational::zero(), Rational::from(2), 1.0),
            (polynomial(vec![(-6).into(),11.into(),(-6).into(),1.into()]), Rational::one(), Rational::from((5,2)), 2.0),
            (polynomial(vec![1.into(),(-4).into(),(-1).into(),4.into()]), Rational::zero(), Rational::one(), 0.25),
            (polynomial(vec![0.into(),(-1).into(),0.into(),1.into()]), Rational::from((-1,2)), Rational::from((1,2)), 0.0),
            (polynomial(vec![-Rational::try_from(f64::from_bits(1)).unwrap(),1.into()]), Rational::zero(), Rational::one(), f64::from_bits(1)),
        ];
        for (poly, a, b, expected) in fixtures {
            let envelope = BracketEnvelope::new(&poly,&a,&b).unwrap();
            let points = bracket_candidates(&poly,a,b,envelope.steps,&cancel,std::time::Instant::now(),Duration::from_secs(10)).unwrap();
            assert_eq!(points[0].to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn cubic_all_or_error_work_bytes_control_and_equal_images() {
        let cancel = Arc::new(AtomicBool::new(false));
        let poly = polynomial(vec![(-2).into(),0.into(),1.into()]);
        for limits in [
            crate::jets::EvaluationLimits { operations: 1, ..Default::default() },
            crate::jets::EvaluationLimits { scratch_bytes: 1, ..Default::default() },
        ] {
            assert!(matches!(real_points(&poly,&cancel,std::time::Instant::now(),Duration::from_secs(10),limits,&mut |_| Ok(())), Err(MathError::Limit(_))));
        }
        assert!(matches!(real_points(&poly,&cancel,std::time::Instant::now(),Duration::ZERO,Default::default(),&mut |_| Ok(())), Err(MathError::Limit("cubic root isolation time"))));
        cancel.store(true,Ordering::Release);
        assert!(matches!(real_points(&poly,&cancel,std::time::Instant::now(),Duration::from_secs(10),Default::default(),&mut |_| Ok(())), Err(MathError::Cancelled)));
        cancel.store(false,Ordering::Release);
        let delta = Rational::from((1_i64,1_i64 << 54));
        // Roots 1 and 1+2^-54 have the same nearest binary64 image.
        let poly = polynomial(vec![Rational::one()+&delta,-(Rational::from(2)+delta),Rational::one()]);
        let points = real_points(&poly,&cancel,std::time::Instant::now(),Duration::from_secs(10),Default::default(),&mut |_| Ok(())).unwrap();
        assert_eq!(points.len(),2);
        assert_eq!(points[0][0],1.0);
        assert!(points[1].contains(&1.0));
        assert!(matches!(real_points(&poly,&cancel,std::time::Instant::now(),Duration::from_secs(10),Default::default(),&mut |_| Err(MathError::Limit("test cubic reservation"))), Err(MathError::Limit("test cubic reservation"))));
    }

    #[test]
    fn cubic_supplier_callbacks_refuse_during_exact_reduction_and_isolation() {
        use symbolica::poly::univariate::BoundedRootIsolationError;
        let poly = polynomial(vec![(-72).into(),123.into(),(-68).into(),12.into()]);
        let mut stages = 0;
        let reduced = poly.square_free_part_bounded(|step| {
            assert!(step.operations>0 && step.bytes>0);
            stages+=1;
            Ok::<_,MathError>(())
        }).unwrap();
        assert_eq!(reduced.degree(),2);
        assert!(stages>1);
        let mut stages = 0;
        let refusal = poly.square_free_part_bounded(|_| {
            stages+=1;
            if stages==2 { Err(MathError::Cancelled) } else { Ok(()) }
        });
        assert!(matches!(refusal,Err(BoundedRootIsolationError::Control(MathError::Cancelled))));
        let mut stages = 0;
        let refusal = reduced.isolate_real_root_intervals_square_free_bounded(|_| {
            stages+=1;
            if stages==3 { Err(MathError::Limit("test supplier deadline")) } else { Ok(()) }
        });
        assert!(matches!(refusal,Err(BoundedRootIsolationError::Control(MathError::Limit("test supplier deadline")))));
        let roots = reduced.isolate_real_root_intervals_square_free_bounded(|_| Ok::<_,MathError>(())).unwrap();
        // These exact rational roots follow from the authored factorization.
        for expected in [Rational::from((3,2)),Rational::from((8,3))] {
            assert_eq!(roots.iter().filter(|(a,b)| (a==b && a==&expected) || (a<&expected && &expected<b)).count(),1);
        }
    }

    #[test]
    fn cubic_unrepresentable_root_refuses_complete_enumeration() {
        let cancel = Arc::new(AtomicBool::new(false));
        let tiny = Rational::try_from(f64::MIN_POSITIVE).unwrap();
        // All coefficients are finite binary64 values. The first polynomial
        // has exact root 4/2^-1022 = 2^1024; the second additionally has root0.
        // Returning only0 would incorrectly turn incomplete enumeration into
        // a successful unique finite association.
        for coefficients in [vec![(-4).into(),tiny.clone()],vec![0.into(),(-4).into(),tiny]] {
            let poly = polynomial(coefficients);
            assert!(matches!(real_points(&poly,&cancel,std::time::Instant::now(),Duration::from_secs(10),Default::default(),&mut |_| Ok(())), Err(MathError::Limit("finite binary64 cubic root representative"))));
        }
    }

    fn admitted() -> (
        CubicRoots,
        Arc<Problem>,
        Options,
        pse_diagnostics::Cancellation,
    ) {
        crate::initialize().unwrap();
        admitted_expression(
            crate::library::formal(0).unwrap().pow(Atom::num(2))
                - crate::library::formal(1).unwrap(),
            0.0,
            3.0,
        )
    }

    fn admitted_expression(
        expression: Atom,
        lower: f64,
        upper: f64,
    ) -> (
        CubicRoots,
        Arc<Problem>,
        Options,
        pse_diagnostics::Cancellation,
    ) {
        use crate::guarded::{PreparedBody, Stage};
        let control = pse_diagnostics::Cancellation::default();
        let cancel = control.flag();
        crate::initialize().unwrap();
        let instance = pse_ids::named_id(SemanticId::NIL, "cubic original residual fixture");
        let body = PreparedBody::new(
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
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let problem = Arc::new(
            Problem::new(
                instance,
                ContentHash::from_bytes([137; 32]),
                vec![Unknown {
                    id: SemanticId::NIL,
                    lower,
                    upper,
                }],
                vec![SemanticId::NIL],
                1,
                residual_bodies_for_test(source_body_for_test(instance, &body, Arc::new(compiled))),
                1024,
            )
            .unwrap(),
        );
        problem
            .prepare_complete(
                DerivativeOrder::First,
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        problem
            .prepare_complete(
                DerivativeOrder::Second,
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let options = Options {
            start: vec![1.0],
            variable_nominals: vec![1.0],
            variable_tolerance: vec![1e-10],
            residual_tolerance: vec![1e-10],
            iterations: 10,
            time_limit: Duration::from_secs(2),
            derivative_tolerance: 1e-10,
        };
        (solver, problem, options, control)
    }

    #[test]
    fn cubic_parameter_degree_changes_degeneracy_and_multiple_branch_roots_refuse() {
        crate::initialize().unwrap();
        let x = crate::library::formal(0).unwrap();
        let p = crate::library::formal(1).unwrap();
        let (solver, problem, options, cancel) = admitted_expression(
            p.clone() * x.clone().pow(Atom::num(3)) + x.clone() - Atom::num(1),
            0.0,
            3.0,
        );
        let mut worker = solver.worker();
        for parameter in [1.0, 0.0, 1.0] {
            let point = worker
                .solve(problem.clone(), &[parameter], &options, &cancel)
                .unwrap();
            problem
                .verify(&[parameter], &point, &options, &cancel.flag())
                .unwrap();
            if parameter == 0.0 {
                assert_eq!(point, [1.0]);
            }
        }
        let (solver, problem, options, cancel) =
            admitted_expression(p * x.clone().pow(Atom::num(2)), 0.0, 3.0);
        assert!(matches!(
            solver.worker().solve(problem, &[0.0], &options, &cancel),
            Err(MathError::Domain {
                requirement: "nondegenerate polynomial root",
                ..
            })
        ));
        let (solver, problem, options, cancel) =
            admitted_expression(x.pow(Atom::num(2)) - Atom::num(1), -3.0, 3.0);
        assert!(matches!(
            solver.worker().solve(problem, &[0.0], &options, &cancel),
            Err(MathError::Domain {
                requirement: "exactly one original-residual-verified root in the declared branch interval",
                ..
            })
        ));
    }

    #[test]
    fn repeated_cubic_root_is_a_value_but_never_a_regular_ift_jet() {
        crate::initialize().unwrap();
        let (solver, problem, options, cancel) = admitted_expression(
            (crate::library::formal(0).unwrap() - crate::library::formal(1).unwrap())
                .pow(Atom::num(3)),
            0.0,
            3.0,
        );
        let point = solver
            .worker()
            .solve(problem.clone(), &[1.0], &options, &cancel)
            .unwrap();
        assert_eq!(point, [1.0]);
        assert!(matches!(
            problem.derivatives(
                &[1.0],
                &point,
                DerivativeOrder::First,
                &options,
                &cancel.flag()
            ),
            Err(MathError::Domain {
                requirement: "implicit derivative residual verification failed",
                ..
            })
        ));
    }

    #[test]
    fn private_cubic_workers_preserve_original_branch_residual_and_cancellation() {
        let (solver, mut problem, options, cancel) = admitted();
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
        // Test the zero Jacobian in an interior neighborhood, independently of
        // the interval-endpoint refusal already covered by the common IFT tests.
        Arc::get_mut(&mut problem).unwrap().unknowns[0].lower = -1.0;
        assert!(matches!(
            problem.derivatives(
                &[0.0],
                &[0.0],
                DerivativeOrder::First,
                &options,
                &cancel.flag()
            ),
            Err(MathError::Domain {
                requirement: "implicit derivative residual verification failed",
                ..
            })
        ));
        cancel.cancel();
        assert!(matches!(
            first.solve(problem, &[4.0], &options, &cancel),
            Err(MathError::Cancelled)
        ));
    }
}
