// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Layout and library-generated composition, never a project differentiation engine.
use crate::{
    MathError,
    library::{self, Optimization},
};
use pse_kernels::DerivativeOrder;
use std::sync::{Arc, atomic::AtomicBool};
use symbolica::{
    atom::{Atom, AtomCore, Indeterminate},
    evaluate::ExpressionEvaluator,
};

/// Explicit resource allowance for one compiled body and its worker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EvaluationLimits {
    /// Maximum Taylor components per scalar.
    pub derivative_components: usize,
    /// Maximum scalar arithmetic operations per compiled demand.
    pub operations: usize,
    /// Maximum owned numeric scratch bytes per worker.
    pub scratch_bytes: usize,
    /// Maximum provider calls in the admitted control schedule.
    pub provider_calls: usize,
}
impl Default for EvaluationLimits {
    fn default() -> Self {
        Self {
            derivative_components: 4096,
            operations: 1_000_000,
            scratch_bytes: 64 * 1024 * 1024,
            provider_calls: 4096,
        }
    }
}
impl EvaluationLimits {
    pub(crate) fn check(&self) -> Result<(), MathError> {
        if self.derivative_components == 0
            || self.operations == 0
            || self.scratch_bytes == 0
            || self.provider_calls == 0
        {
            return Err(MathError::Limit("zero evaluation allowance"));
        }
        Ok(())
    }
    pub(crate) fn allocation(&self, entries: usize) -> Result<(), MathError> {
        if entries
            .checked_mul(size_of::<f64>())
            .is_none_or(|b| b > self.scratch_bytes)
        {
            return Err(MathError::Limit("worker scratch bytes"));
        }
        Ok(())
    }
}

/// Downward-closed value/gradient/symmetric-Hessian Taylor coefficient layout.
#[derive(Clone, Debug)]
pub struct JetLayout {
    pub(crate) coordinates: Vec<usize>,
    pub(crate) shape: Vec<Vec<usize>>,
    pub(crate) pairs: Vec<(usize, usize)>,
    pub(crate) order: DerivativeOrder,
}
impl JetLayout {
    /// Create a checked layout; coordinates are unique formal input slots.
    pub fn new(
        coordinates: Vec<usize>,
        order: DerivativeOrder,
        limits: EvaluationLimits,
    ) -> Result<Self, MathError> {
        limits.check()?;
        let n = coordinates.len();
        if coordinates
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != n
        {
            return Err(MathError::Contract(
                "duplicate derivative coordinates".into(),
            ));
        }
        let first = if order >= DerivativeOrder::First {
            n
        } else {
            0
        };
        let second = if order >= DerivativeOrder::Second {
            n.checked_mul(n + 1).map(|k| k / 2)
        } else {
            Some(0)
        };
        let count = second
            .and_then(|s| s.checked_add(first))
            .and_then(|s| s.checked_add(1))
            .ok_or(MathError::Limit("derivative components"))?;
        if count > limits.derivative_components {
            return Err(MathError::Limit("derivative components"));
        }
        // Shape metadata is bounded as well as the numeric frame.
        limits.allocation(
            count
                .checked_mul(n.max(1))
                .ok_or(MathError::Limit("derivative shape"))?,
        )?;
        let mut shape = vec![vec![0; n]];
        let mut pairs = vec![];
        if order >= DerivativeOrder::First {
            for i in 0..n {
                let mut term = vec![0; n];
                term[i] = 1;
                shape.push(term);
            }
        }
        if order >= DerivativeOrder::Second {
            for i in 0..n {
                for j in i..n {
                    let mut term = vec![0; n];
                    term[i] += 1;
                    term[j] += 1;
                    shape.push(term);
                    pairs.push((i, j));
                }
            }
        }
        Ok(Self {
            coordinates,
            shape,
            pairs,
            order,
        })
    }
    /// Number of coefficients carried by each scalar.
    pub fn width(&self) -> usize {
        self.shape.len()
    }
    /// Formal inputs with respect to which derivatives are taken.
    pub fn coordinates(&self) -> &[usize] {
        &self.coordinates
    }
    /// Raw partial = coefficient * alpha factorial (2 for diagonal second terms).
    pub fn raw_factor(&self, component: usize) -> f64 {
        if self.shape[component].contains(&2) {
            2.0
        } else {
            1.0
        }
    }
    pub(crate) fn differentiate(
        &self,
        atom: &Atom,
        variables: &[Atom],
        component: usize,
    ) -> Result<Atom, MathError> {
        let mut result = atom.clone();
        for (var, &degree) in variables.iter().zip(&self.shape[component]) {
            for _ in 0..degree {
                result = result.derivative(
                    Indeterminate::try_from(var.clone())
                        .map_err(|e| MathError::Library(e.clone()))?,
                );
            }
        }
        Ok(result)
    }
}

/// Raw local partial or raw incoming derivative bound to a generated library parameter.
#[derive(Clone, Debug)]
pub(crate) enum LiftInput {
    Value,
    First(usize),
    Second(usize, usize),
    Argument(usize, usize),
}
#[derive(Clone)]
pub(crate) struct ProviderLift {
    pub evaluator: ExpressionEvaluator<f64>,
    pub inputs: Vec<LiftInput>,
    pub scratch: Vec<f64>,
    pub output: Vec<f64>,
}
impl ProviderLift {
    /// Symbolica differentiates an abstract composition `f(a_0(z)`, ..., `a_n(z)`).
    pub(crate) fn compile(
        arity: usize,
        layout: &JetLayout,
        options: Optimization,
        limits: EvaluationLimits,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        let n = layout.coordinates.len();
        if arity + n >= library::MAX_FORMAL_SYMBOLS {
            return Err(MathError::Limit("provider composition symbols"));
        }
        let estimate = arity
            .checked_mul(layout.width())
            .and_then(|k| k.checked_add(arity.checked_mul(arity)?))
            .ok_or(MathError::Limit("provider composition width"))?;
        limits.allocation(estimate)?;
        let variables = (0..n).map(library::formal).collect::<Result<Vec<_>, _>>()?;
        let local = (n..n + arity)
            .map(library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        let arguments = (0..arity)
            .map(|i| library::function(i + 1, &variables))
            .collect::<Result<Vec<_>, _>>()?;
        let generic = library::function(0, &local)?;
        let composed = library::function(0, &arguments)?;
        let substitute = |value: Atom| {
            local.iter().zip(&arguments).fold(value, |value, (u, a)| {
                value.replace(u.clone()).with(a.clone())
            })
        };
        let mut parameters = vec![composed.clone()];
        let mut inputs = vec![LiftInput::Value];
        if layout.order >= DerivativeOrder::First {
            for i in 0..arity {
                let d = generic.derivative(
                    Indeterminate::try_from(local[i].clone())
                        .map_err(|e| MathError::Library(e.clone()))?,
                );
                parameters.push(substitute(d.clone()));
                inputs.push(LiftInput::First(i));
                if layout.order >= DerivativeOrder::Second {
                    for (j, atom) in local.iter().enumerate().skip(i) {
                        parameters.push(substitute(
                            d.derivative(
                                Indeterminate::try_from(atom.clone())
                                    .map_err(|e| MathError::Library(e.clone()))?,
                            ),
                        ));
                        inputs.push(LiftInput::Second(i, j));
                    }
                }
            }
            for (i, argument) in arguments.iter().enumerate() {
                for component in 1..layout.width() {
                    parameters.push(layout.differentiate(argument, &variables, component)?);
                    inputs.push(LiftInput::Argument(i, component));
                }
            }
        }
        let expressions = (0..layout.width())
            .map(|component| {
                Ok(layout.differentiate(&composed, &variables, component)?
                    / Atom::num(layout.raw_factor(component)))
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let evaluator = library::evaluator(&expressions, &parameters, options, cancelled)?;
        Ok(Self {
            scratch: vec![0.0; inputs.len()],
            output: vec![0.0; layout.width()],
            evaluator,
            inputs,
        })
    }
}
