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
    coefficients: Mutex<symbolica::evaluate::ExpressionEvaluator<f64>>,
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
            crate::library::Optimization::default(),
            cancel,
            limits,
            limits.operations,
            0,
        )?;
        Ok(Self {
            coefficients: Mutex::new(evaluator),
            parameters: formals.len(),
        })
    }
}
impl InnerSolver for CubicRoots {
    fn minimum_order(&self) -> DerivativeOrder {
        DerivativeOrder::Value
    }
    fn identity(&self) -> ContentHash {
        solver_identity("symbolica.cubic-roots.v1")
    }
    fn solve(
        &self,
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
        self.coefficients
            .lock()
            .map_err(|_| MathError::Library("cubic coefficient lock".into()))?
            .evaluate(parameters, &mut coefficients);
        if coefficients.iter().any(|v| !v.is_finite()) {
            return Err(MathError::Domain {
                source_id: problem.id,
                requirement: "finite cubic coefficients",
            });
        }
        let coefficients = coefficients
            .into_iter()
            .map(|v| Rational::try_from(v).map_err(|e| MathError::Library(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?;
        if coefficients.iter().skip(1).all(Rational::is_zero) {
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
        let roots = polynomial.isolate_real_roots();
        let precision = Rational::from((1_i64, 1_i64 << 54));
        let mut accepted = Vec::new();
        for (root, _) in roots {
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            if started.elapsed() > options.time_limit {
                return Err(MathError::Limit("cubic root isolation time"));
            }
            let point = vec![root.refined(&precision).enclosure().center().re.to_f64()];
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
