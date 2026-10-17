// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-owned scalar bracket iteration with charged fallible physical evaluations.
use super::*;
use symbolica::atom::AtomCore;

/// TOMS748 for an admitted guard-free polynomial and explicit physical interval.
/// A bracket is not a uniqueness certificate; authored selection remains independent.
#[derive(Clone, Debug)]
pub struct BracketedRoot {
    parameters: usize,
}
impl BracketedRoot {
    /// Establish continuity from a polynomial in actual formals, with no hidden guards.
    /// Providers, lazy branches and domain obligations retain their existing solver route.
    pub fn new(
        body: &crate::guarded::PreparedBody,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        if body.input_count() == 0 || body.output_count() != 1 || body.has_obligations() {
            return Err(MathError::Contract(
                "bracketed scalar continuity/domain admission".into(),
            ));
        }
        let expression = body.expression(0).ok_or_else(|| {
            MathError::Contract("bracketed scalar requires a polynomial projection".into())
        })?;
        let formals = (0..body.input_count())
            .map(crate::library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        // Symbolica may treat a function or reciprocal as an independent polynomial
        // variable. Only the actual formal symbols establish the intended continuity.
        let variables = expression.is_polynomial(true, false).ok_or_else(|| {
            MathError::Contract("bracketed scalar requires polynomial arithmetic".into())
        })?;
        if variables
            .iter()
            .any(|variable| !formals.iter().any(|formal| formal.as_view() == *variable))
        {
            return Err(MathError::Contract(
                "bracketed scalar has nonpolynomial variables".into(),
            ));
        }
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        Ok(Self {
            parameters: body.input_count() - 1,
        })
    }
}
impl InnerSolver for BracketedRoot {
    fn minimum_order(&self) -> DerivativeOrder {
        DerivativeOrder::Value
    }
    fn identity(&self) -> ContentHash {
        solver_identity("root1d.0.4.1.toms748.polynomial.v1")
    }
    fn worker(&self) -> Box<dyn InnerWorker> {
        Box::new(self.clone())
    }
}
impl InnerWorker for BracketedRoot {
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
        if problem.unknowns.len() != 1
            || parameters.len() != self.parameters
            || parameters.iter().any(|value| !value.is_finite())
        {
            return Err(MathError::Contract(
                "bracketed scalar input contract".into(),
            ));
        }
        // TOMS748 uses at most six new evaluations per iteration. This finite ceiling
        // includes preflight, repeated library endpoints and independent final acceptance.
        let evaluations = usize::try_from(options.iterations)
            .ok()
            .and_then(|n| n.checked_mul(6))
            .and_then(|n| n.checked_add(7))
            .ok_or(MathError::Limit("scalar evaluation allowance extent"))?;
        let unknown = &problem.unknowns[0];
        let point = bracketed(
            problem.id,
            unknown.lower,
            unknown.upper,
            options.variable_tolerance[0],
            options.residual_tolerance[0],
            options.iterations as usize,
            evaluations,
            options.time_limit,
            cancel,
            |point| {
                let evaluation = problem.evaluate_unknown(
                    parameters,
                    &[point],
                    DerivativeOrder::Value,
                    cancel,
                )?;
                evaluation
                    .values
                    .first()
                    .copied()
                    .ok_or_else(|| MathError::Contract("scalar residual extent".into()))
            },
        )?;
        Ok(vec![point])
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "all original physical and finite work controls are explicit at the library boundary"
)]
fn bracketed(
    source_id: SemanticId,
    lower: f64,
    upper: f64,
    variable_tolerance: f64,
    residual_tolerance: f64,
    iterations: usize,
    evaluations: usize,
    time_limit: Duration,
    cancel: &Arc<AtomicBool>,
    mut residual: impl FnMut(f64) -> Result<f64, MathError>,
) -> Result<f64, MathError> {
    if !lower.is_finite()
        || !upper.is_finite()
        || lower > upper
        || iterations == 0
        || evaluations == 0
        || time_limit.is_zero()
        || !variable_tolerance.is_finite()
        || variable_tolerance <= 0.0
        || !residual_tolerance.is_finite()
        || residual_tolerance <= 0.0
    {
        return Err(MathError::Contract(
            "scalar interval or work controls".into(),
        ));
    }
    let started = std::time::Instant::now();
    let check = || {
        if cancel.load(Ordering::Acquire) {
            Err(MathError::Cancelled)
        } else if started.elapsed() >= time_limit {
            Err(MathError::Scope(pse_kernels::ProviderError::Deadline))
        } else {
            Ok(())
        }
    };
    let mut remaining = evaluations;
    let mut evaluate = |point: f64| -> Result<f64, MathError> {
        check()?;
        if remaining == 0 {
            return Err(MathError::Limit("scalar residual evaluations"));
        }
        if !point.is_finite() || point < lower || point > upper {
            return Err(MathError::Domain {
                source_id,
                requirement: "scalar trial outside original interval",
            });
        }
        remaining -= 1;
        let result = residual(point);
        check()?;
        let value = result?;
        if !value.is_finite() {
            return Err(MathError::Domain {
                source_id,
                requirement: "finite original scalar residual",
            });
        }
        Ok(value)
    };
    // root1d checks width before signs and returns the midpoint on that path.
    // Perform every physical bracket check before allowing that library shortcut.
    let left = evaluate(lower)?;
    let right = evaluate(upper)?;
    let point = if left == 0.0 {
        lower
    } else if right == 0.0 {
        upper
    } else {
        if lower == upper || left.is_sign_negative() == right.is_sign_negative() {
            return Err(MathError::Domain {
                source_id,
                requirement: "opposite original scalar endpoint signs",
            });
        }
        root1d::toms748(&mut evaluate, lower, upper)
            .maxiter(iterations)
            .maxiter_err(true)
            .terminate(|a: &f64, b: &f64, _fa: &f64, _fb: &f64| {
                b * 0.5 - a * 0.5 <= variable_tolerance * 0.5
            })
            .root()
            .map_err(|error| match error {
                root1d::Error::Fun { err, .. } => err,
                root1d::Error::MaxIter => MathError::Limit("scalar root iterations"),
                root1d::Error::NotFinite { .. } => MathError::Domain {
                    source_id,
                    requirement: "finite scalar library trial",
                },
                root1d::Error::NoSignChange { .. } => MathError::Domain {
                    source_id,
                    requirement: "opposite original scalar endpoint signs",
                },
            })?
    };
    // The library candidate may never have been evaluated. Width does not discharge
    // original physical residual acceptance, even for a steep or discontinuous function.
    if evaluate(point)?.abs() > residual_tolerance {
        return Err(MathError::Domain {
            source_id,
            requirement: "scalar root did not satisfy original residual budget",
        });
    }
    check()?;
    Ok(point)
}

#[cfg(test)]
#[path = "implicit_scalar_tests.rs"]
mod tests;
