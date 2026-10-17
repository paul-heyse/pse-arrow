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

/// Mathematical output actually consumed by an admitted evaluator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProductKind {
    /// Selected values and coordinate partials through the requested order.
    SelectedPartials,
    /// One first directional action with runtime coordinate seeds.
    JacobianAction,
    /// Lower sparse Hessian of a runtime-weighted output sum.
    WeightedHessian,
    /// Hessian of a runtime-weighted output sum applied to a runtime direction.
    WeightedHessianAction,
}
/// Runtime bindings in the immutable product's selected output/coordinate order.
#[derive(Clone, Copy, Debug)]
pub enum ProductValues<'a> {
    /// No weights or direction are consumed.
    SelectedPartials,
    /// Direction entries correspond to selected derivative coordinates.
    JacobianAction {
        /// Ordered derivative-coordinate seeds.
        direction: &'a [f64] },
    /// Weights correspond to selected output ordinals.
    WeightedHessian {
        /// Ordered selected-output coefficients.
        weights: &'a [f64] },
    /// Weights and direction are independent runtime bindings.
    WeightedHessianAction {
        /// Ordered selected-output coefficients.
        weights: &'a [f64],
        /// Ordered derivative-coordinate seeds.
        direction: &'a [f64] },
}
impl ProductValues<'_> {
    /// Product identity, independent of the numerical binding values.
    pub fn kind(self) -> ProductKind {
        match self {
            Self::SelectedPartials => ProductKind::SelectedPartials,
            Self::JacobianAction { .. } => ProductKind::JacobianAction,
            Self::WeightedHessian { .. } => ProductKind::WeightedHessian,
            Self::WeightedHessianAction { .. } => ProductKind::WeightedHessianAction,
        }
    }
}

/// Explicit resource allowance for one compiled body and its worker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
            scratch_bytes: 512 * 1024 * 1024,
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
        scalars: &mut library::ScalarCache,
    ) -> Result<Self, MathError> {
        let n = layout.coordinates.len();
        let estimate = arity
            .checked_mul(layout.width())
            .and_then(|k| k.checked_add(arity.checked_mul(arity)?))
            .ok_or(MathError::Limit("provider composition width"))?;
        limits.allocation(estimate)?;
        // Second-order chain rule has <=arity²+arity terms per coefficient,
        // with at most three scalar operations per term and one factorial scale.
        // Admit the symbolic tree before differentiating the abstract composition.
        let symbolic_work = arity
            .checked_mul(arity)
            .and_then(|v| v.checked_add(arity))
            .and_then(|v| v.checked_mul(3))
            .and_then(|v| v.checked_add(1))
            .and_then(|v| v.checked_mul(layout.width()))
            .ok_or(MathError::Limit("provider composition operations"))?;
        if symbolic_work > limits.operations {
            return Err(MathError::Limit("provider composition operations"));
        }
        limits.allocation(symbolic_work)?;
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
        let evaluator = library::bounded_scalar_evaluator(
            &expressions,
            &parameters,
            options,
            cancelled,
            limits,
            scalars,
        )?;
        Ok(Self {
            scratch: vec![0.0; inputs.len()],
            output: vec![0.0; layout.width()],
            evaluator,
            inputs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_provider_lifts_share_scalar_preparation_with_distinct_order_keys() {
        crate::initialize().unwrap();
        let limits = EvaluationLimits::default();
        let cancel = Arc::new(AtomicBool::new(false));
        let mut scalars = library::ScalarCache::default();
        let first_layout = JetLayout::new(vec![0], DerivativeOrder::First, limits).unwrap();
        let second_layout = JetLayout::new(vec![0], DerivativeOrder::Second, limits).unwrap();
        let compile = |layout: &JetLayout, scalars: &mut library::ScalarCache| {
            ProviderLift::compile(1, layout, Optimization::default(), limits, &cancel, scalars)
                .unwrap()
        };
        let mut first = compile(&first_layout, &mut scalars);
        let mut another_first = compile(&first_layout, &mut scalars);
        assert_eq!(scalars.len(), 1);
        let mut second = compile(&second_layout, &mut scalars);
        let _another_second = compile(&second_layout, &mut scalars);
        assert_eq!(
            scalars.len(),
            2,
            "order-specific expressions and ordered parameters remain distinct"
        );
        let refill = |lift: &mut ProviderLift, argument_first: f64| {
            for (input, value) in lift.inputs.iter().zip(&mut lift.scratch) {
                *value = match input {
                    LiftInput::Value => 9.,
                    LiftInput::First(0) => 6.,
                    LiftInput::Second(0, 0) => 2.,
                    LiftInput::Argument(0, 1) => argument_first,
                    LiftInput::Argument(0, 2) => 4.,
                    _ => panic!("unexpected unary lift input"),
                };
            }
            lift.evaluator
                .try_evaluate(&lift.scratch, &mut lift.output)
                .unwrap();
        };
        refill(&mut first, 2.);
        assert_eq!(first.output, vec![9., 12.]);
        refill(&mut another_first, 3.);
        assert_eq!(another_first.output, vec![9., 18.]);
        assert_eq!(first.output, vec![9., 12.]);
        refill(&mut second, 2.);
        assert_eq!(second.output, vec![9., 12., 16.]);
        refill(&mut first, 2.);
        assert_eq!(first.output, vec![9., 12.]);
        assert_eq!(scalars.len(), 2);
    }
}
