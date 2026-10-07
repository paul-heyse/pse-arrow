// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-established scalar graph forms and attempt-bound selection enforcement.
use super::*;
use symbolica::atom::{Atom, AtomCore, AtomView, Indeterminate};

/// Bounded graph equivalence evidence; no claim about unsupported nonlinear systems.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionEquivalence {
    /// Nonzero constant linear coefficient, so the residual has at most one root.
    NondegenerateAffine,
    /// Nonzero constant square coefficient and no linear term, restricted to one sign.
    RestrictedSquareRoot,
    /// No supported equivalence evidence.
    Unestablished,
}
/// Establish only the initial exact scalar controls using Symbolica coefficient extraction.
pub fn graph_equivalence(
    body: &crate::guarded::PreparedBody,
    unknowns: usize,
    sign: Option<bool>,
) -> Result<SelectionEquivalence, MathError> {
    if unknowns != 1 || body.output_count() != 1 {
        return Ok(SelectionEquivalence::Unestablished);
    }
    let expression = match body.expression(0) {
        Some(e) => e.clone(),
        None => {
            let Some(e) = independent_call_expression(body)? else {
                return Ok(SelectionEquivalence::Unestablished);
            };
            e
        }
    };
    let x = crate::library::formal(0)?;
    let mut coefficients = [Atom::num(0), Atom::num(0), Atom::num(0)];
    for (power, coefficient) in expression.coefficient_list::<u16>(std::slice::from_ref(&x)) {
        let Some(index) = (0..3).find(|i| x.clone().pow(Atom::num(*i)) == power) else {
            return Ok(SelectionEquivalence::Unestablished);
        };
        coefficients[index] += coefficient;
    }
    let constant_nonzero = |a: &Atom| matches!(a.as_view(), AtomView::Num(_)) && *a != Atom::num(0);
    if coefficients[2] == Atom::num(0) && constant_nonzero(&coefficients[1]) {
        Ok(SelectionEquivalence::NondegenerateAffine)
    } else if sign.is_some()
        && coefficients[1] == Atom::num(0)
        && constant_nonzero(&coefficients[2])
    {
        Ok(SelectionEquivalence::RestrictedSquareRoot)
    } else {
        Ok(SelectionEquivalence::Unestablished)
    }
}
// Only substitute straight library blocks and calls independent of the scalar unknown.
// This is coefficient evidence, never an evaluator or a provider replacement.
fn independent_call_expression(
    body: &crate::guarded::PreparedBody,
) -> Result<Option<Atom>, MathError> {
    use crate::guarded::Stage;
    let formals = (0..body.slot_count())
        .map(crate::library::formal)
        .collect::<Result<Vec<_>, _>>()?;
    let x = &formals[0];
    let mut values = (0..body.slot_count())
        .map(|i| (i < body.input_count()).then(|| formals[i].clone()))
        .collect::<Vec<_>>();
    let substitute = |atom: &Atom, values: &[Option<Atom>]| -> Option<Atom> {
        let mut result = atom.clone();
        for (i, value) in values.iter().enumerate().rev() {
            if let Some(value) = value {
                if result
                    .as_view()
                    .get_byte_size()
                    .checked_mul(value.as_view().get_byte_size().checked_add(1)?)
                    .is_none_or(|bytes| bytes > 1024 * 1024)
                {
                    return None;
                }
                result = result.replace(formals[i].clone()).with(value.clone());
            }
        }
        Some(result)
    };
    let variable = Indeterminate::try_from(x.clone()).map_err(MathError::Library)?;
    for stage in body.demanded_stages(&[0])? {
        match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                let Some(expanded) = expressions
                    .iter()
                    .map(|e| substitute(e, &values))
                    .collect::<Option<Vec<_>>>()
                else {
                    return Ok(None);
                };
                for (slot, expression) in outputs.into_iter().zip(expanded) {
                    values[slot] = Some(expression);
                }
            }
            Stage::Provider {
                inputs, outputs, ..
            } => {
                if inputs.iter().any(|i| {
                    values[*i]
                        .as_ref()
                        .is_none_or(|a| a.derivative(variable.clone()) != Atom::num(0))
                }) {
                    return Ok(None);
                }
                for slot in outputs {
                    values[slot] = Some(formals[slot].clone());
                }
            }
            Stage::Require { .. } => {}
            Stage::Domain { token, .. } | Stage::Applicability { token, .. } => {
                values[token] = Some(Atom::num(0));
            }
            Stage::Branch { .. } => return Ok(None),
        }
    }
    Ok(values[body.outputs[0]].clone())
}
/// Compiler-issued selection transport. A semantic anchor is separate from numerical hints.
#[derive(Clone, Debug, Default)]
pub struct Selection {
    /// Semantic operational anchor program, evaluated from independent inputs.
    pub anchor: Option<Arc<CompiledBody>>,
    /// Checked algorithm/settings reference for operational selection.
    pub settings: Option<String>,
    /// Admitted branch/neighborhood predicate encoded as a scalar indicator.
    pub restriction: Option<Arc<CompiledBody>>,
    /// Supported scalar sign restriction; true is positive, false is negative.
    pub sign: Option<bool>,
}
#[derive(Debug)]
pub(super) struct SelectionWorker {
    anchor: Option<Worker>,
    restriction: Option<Worker>,
    sign: Option<bool>,
}
impl SelectionWorker {
    pub(super) fn new(selection: &Selection) -> Self {
        Self {
            anchor: selection.anchor.as_ref().map(|b| b.worker()),
            restriction: selection.restriction.as_ref().map(|b| b.worker()),
            sign: selection.sign,
        }
    }
    pub(super) fn anchor(
        &mut self,
        problem: &Problem,
        inputs: &[f64],
        cancel: &Arc<AtomicBool>,
    ) -> Result<Option<Vec<f64>>, MathError> {
        if let Some(anchor) = &mut self.anchor {
            let unknowns = vec![0.; problem.unknowns.len()];
            let coordinates = selected_inputs(anchor, &unknowns, inputs)?;
            return Ok(Some(
                anchor
                    .evaluate(
                        &coordinates,
                        DerivativeOrder::Value,
                        &mut *problem.providers.lock().map_err(|_| {
                            MathError::Library("implicit anchor provider lock poisoned".into())
                        })?,
                        cancel,
                    )?
                    .values,
            ));
        }
        Ok(None)
    }
    pub(super) fn configure(
        &mut self,
        problem: &mut Arc<Problem>,
        options: &Options,
    ) -> Result<(), MathError> {
        if let Some(positive) = self.sign {
            let problem = Arc::get_mut(problem)
                .ok_or_else(|| MathError::Contract("selection problem is borrowed".into()))?;
            if problem.unknowns.len() != 1 {
                return Err(MathError::Contract("scalar branch extent".into()));
            }
            if positive {
                problem.unknowns[0].lower = problem.unknowns[0].lower.max(0.);
            } else {
                problem.unknowns[0].upper = problem.unknowns[0].upper.min(0.);
            }
        }
        problem.validate_options(options)
    }
    pub(super) fn verify(
        &mut self,
        problem: &Problem,
        inputs: &[f64],
        point: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), MathError> {
        if let Some(restriction) = &mut self.restriction {
            let coordinates = selected_inputs(restriction, point, inputs)?;
            let jet = restriction.evaluate(
                &coordinates,
                if order > DerivativeOrder::Value {
                    DerivativeOrder::First
                } else {
                    DerivativeOrder::Value
                },
                &mut *problem.providers.lock().map_err(|_| {
                    MathError::Library("implicit selection provider lock poisoned".into())
                })?,
                cancel,
            )?;
            if jet.values != [1.] {
                return Err(MathError::Domain {
                    source_id: problem.id,
                    requirement: "implicit selected branch restriction",
                });
            }
        }
        Ok(())
    }
}
