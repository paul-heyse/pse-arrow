// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Symbolic boundary proofs for physically checked lazy functions.
use super::*;
use std::collections::BTreeMap;
use symbolica::atom::Indeterminate;
fn order(stages: &[Stage], unproved: DerivativeOrder) -> DerivativeOrder {
    stages
        .iter()
        .fold(DerivativeOrder::Second, |order, s| match s {
            Stage::Branch {
                continuity,
                then,
                otherwise,
                ..
            } => order
                .min(if *continuity == DerivativeOrder::Value {
                    unproved
                } else {
                    *continuity
                })
                .min(self::order(then, unproved))
                .min(self::order(otherwise, unproved)),
            _ => order,
        })
}
pub(super) fn branch_order(stages: &[Stage]) -> DerivativeOrder {
    order(stages, DerivativeOrder::Value)
}
fn bounded(atom: &Atom, limit: usize) -> Result<(), MathError> {
    let n = atom.count_operations();
    if n.additions
        .saturating_add(n.multiplications)
        .saturating_add(n.inversions)
        .saturating_add(n.function_calls)
        > limit
    {
        return Err(MathError::Limit("piecewise symbolic proof"));
    }
    Ok(())
}
fn expand(atom: &Atom, values: &BTreeMap<usize, Atom>, limit: usize) -> Result<Atom, MathError> {
    let mut result = atom.clone();
    for (slot, value) in values.iter().rev() {
        result = result.replace(library::formal(*slot)?).with(value.clone());
        bounded(&result, limit)?;
    }
    Ok(result)
}
// Select the already-proved branch adjacent to another breakpoint. Guard ordering
// must reduce to an exact real coefficient; numerical sampling is never proof.
fn boundary_values(
    stages: &[Stage],
    mut values: BTreeMap<usize, Atom>,
    axis: &Atom,
    boundary: &Atom,
    limit: usize,
) -> Result<BTreeMap<usize, Atom>, MathError> {
    for stage in stages {
        match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                for (e, s) in expressions.iter().zip(outputs) {
                    values.insert(*s, expand(e, &values, limit)?);
                }
            }
            Stage::Require { .. } | Stage::Domain { .. } | Stage::Applicability { .. } => {}
            Stage::Provider { .. } => {
                return Err(MathError::Contract("opaque piecewise boundary".into()));
            }
            Stage::Branch {
                comparison,
                left,
                right,
                then,
                otherwise,
                ..
            } => {
                let a = values
                    .get(left)
                    .ok_or_else(|| MathError::Contract("piecewise boundary guard".into()))?;
                let b = values
                    .get(right)
                    .ok_or_else(|| MathError::Contract("piecewise boundary guard".into()))?;
                let delta = (a - b)
                    .replace(axis.clone())
                    .with(boundary.clone())
                    .expand();
                bounded(&delta, limit)?;
                let symbolica::atom::AtomView::Num(n) = delta.as_view() else {
                    return Err(MathError::Contract(
                        "piecewise breakpoint ordering is not symbolically established".into(),
                    ));
                };
                use symbolica::coefficient::CoefficientView;
                let coefficient = n.get_coeff_view();
                if !matches!(coefficient,CoefficientView::Natural(_,_,0,_)|CoefficientView::Large(_,_) if coefficient.is_real())
                {
                    return Err(MathError::Contract(
                        "piecewise ordering requires an exact real breakpoint difference".into(),
                    ));
                }
                let take = coefficient.to_owned().is_negative()
                    || (*comparison == Comparison::Le && coefficient.is_zero());
                values = boundary_values(
                    if take { then } else { otherwise },
                    values,
                    axis,
                    boundary,
                    limit,
                )?;
            }
        }
    }
    Ok(values)
}
fn prove(
    stages: &mut [Stage],
    values: &mut BTreeMap<usize, Atom>,
    arguments: &[Atom],
    requested: DerivativeOrder,
    limit: usize,
) -> Result<usize, MathError> {
    let mut proofs = 0;
    for stage in stages {
        match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                for (expression, slot) in expressions.iter().zip(outputs) {
                    values.insert(*slot, expand(expression, values, limit)?);
                }
            }
            Stage::Require { .. } | Stage::Domain { .. } | Stage::Applicability { .. } => {}
            Stage::Provider { .. } => {
                return Err(MathError::Contract(
                    "opaque functions cannot establish symbolic boundary agreement".into(),
                ));
            }
            Stage::Branch {
                comparison,
                left,
                right,
                then,
                otherwise,
                continuity,
            } => {
                if !matches!(comparison, Comparison::Lt | Comparison::Le) {
                    return Err(MathError::Contract(
                        "piecewise requires ordered breakpoint guards".into(),
                    ));
                }
                let a = values
                    .get(left)
                    .ok_or_else(|| MathError::Contract("piecewise left guard expression".into()))?;
                let b = values.get(right).ok_or_else(|| {
                    MathError::Contract("piecewise right guard expression".into())
                })?;
                let (axis, boundary) = if let Some(axis) = arguments.iter().find(|v| *v == a) {
                    (axis, b)
                } else if let Some(axis) = arguments.iter().find(|v| *v == b) {
                    (axis, a)
                } else {
                    return Err(MathError::Contract(
                        "piecewise breakpoint must compare an explicit argument".into(),
                    ));
                };
                let variable = Indeterminate::try_from(axis.clone())
                    .map_err(|e| MathError::Library(e.clone()))?;
                if !boundary.derivative(variable).is_zero() {
                    return Err(MathError::Contract(
                        "piecewise breakpoint depends on its selected argument".into(),
                    ));
                }
                let axis = axis.clone();
                let boundary = boundary.clone();
                let mut x = values.clone();
                let mut y = values.clone();
                proofs += prove(then, &mut x, arguments, requested, limit)?;
                proofs += prove(otherwise, &mut y, arguments, requested, limit)?;
                let x = boundary_values(then, values.clone(), &axis, &boundary, limit)?;
                let y = boundary_values(otherwise, values.clone(), &axis, &boundary, limit)?;
                let mut outputs = 0;
                for (slot, a) in &x {
                    let Some(b) = y.get(slot) else {
                        continue;
                    };
                    if values.get(slot) == Some(a) && a == b {
                        continue;
                    }
                    outputs += 1;
                    let delta = a - b;
                    let check = |value: Atom| -> Result<(), MathError> {
                        bounded(&value, limit)?;
                        let evaluated = value.replace(axis.clone()).with(boundary.clone()).expand();
                        bounded(&evaluated, limit)?;
                        if !evaluated.is_zero() {
                            return Err(MathError::Contract(
                                "piecewise continuity claim is not symbolically established".into(),
                            ));
                        }
                        Ok(())
                    };
                    check(delta.clone())?;
                    if requested >= DerivativeOrder::First {
                        for (i, arg) in arguments.iter().enumerate() {
                            let derivative = delta.derivative(
                                Indeterminate::try_from(arg.clone())
                                    .map_err(|e| MathError::Library(e.clone()))?,
                            );
                            check(derivative.clone())?;
                            if requested >= DerivativeOrder::Second {
                                for other in &arguments[..=i] {
                                    check(
                                        derivative.derivative(
                                            Indeterminate::try_from(other.clone())
                                                .map_err(|e| MathError::Library(e.clone()))?,
                                        ),
                                    )?;
                                }
                            }
                        }
                    }
                }
                if outputs == 0 {
                    return Err(MathError::Contract(
                        "piecewise has no comparable branch output".into(),
                    ));
                }
                let keys = x.keys().chain(y.keys()).copied().collect::<BTreeSet<_>>();
                for key in keys {
                    match (x.get(&key), y.get(&key)) {
                        (Some(a), Some(b)) if a == b => {
                            values.insert(key, a.clone());
                        }
                        _ => {
                            values.remove(&key);
                        }
                    }
                }
                *continuity = requested;
                proofs += 1;
            }
        }
    }
    Ok(proofs)
}
impl BodyBuilder<'_> {
    /// Prove values and all explicit-argument partials through the claimed order at every breakpoint.
    /// Unproved or opaque transitions remain errors; no numerical samples authorize smoothness.
    pub fn verify_piecewise(
        &mut self,
        scope: FunctionScope,
        arguments: &[TypedValue],
        order: DerivativeOrder,
    ) -> Result<(), MathError> {
        if self.physical_only {
            return Ok(());
        }
        if scope.0 > self.stages.len() || arguments.len() > 64 {
            return Err(MathError::Limit("piecewise proof scope or argument extent"));
        }
        let mut stages = self.stages[scope.0..].to_vec();
        let arguments = arguments.iter().map(|a| a.atom.clone()).collect::<Vec<_>>();
        let count = prove(
            &mut stages,
            &mut BTreeMap::new(),
            &arguments,
            order,
            self.limits.occurrences,
        )?;
        if count == 0 {
            return Err(MathError::Contract(
                "piecewise declaration requires an explicit breakpoint".into(),
            ));
        }
        self.stages.truncate(scope.0);
        self.stages.extend(stages);
        Ok(())
    }
}

/// Expand bounded pure control paths, and ask Symbolica to differentiate each leaf.
/// Conditions keep their original slots and domain guards execute in the original schedule.
#[expect(
    clippy::too_many_arguments,
    reason = "path expansion threads its work and slot counters beside the fixed stage schedule, arguments and limits"
)]
pub(super) fn partial_paths(
    stages: &[Stage],
    mut values: BTreeMap<usize, Atom>,
    value: &Atom,
    arguments: &[TypedValue],
    output: usize,
    source: SemanticId,
    limit: usize,
    work: &mut usize,
    next_slot: &mut usize,
    slot_limit: usize,
) -> Result<Vec<Stage>, MathError> {
    for (index, stage) in stages.iter().enumerate() {
        *work = work
            .checked_add(1)
            .ok_or(MathError::Limit("piecewise partial paths"))?;
        if *work > limit {
            return Err(MathError::Limit("piecewise partial paths"));
        }
        match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                for (e, s) in expressions.iter().zip(outputs) {
                    values.insert(*s, expand(e, &values, limit)?);
                }
            }
            Stage::Require { .. } | Stage::Domain { .. } | Stage::Applicability { .. } => {}
            Stage::Provider { .. } => {
                return Err(MathError::Contract(
                    "explicit partial requires external derivative output capability".into(),
                ));
            }
            Stage::Branch {
                comparison,
                left,
                right,
                then,
                otherwise,
                continuity,
            } => {
                let remaining = (*continuity as usize)
                    .checked_sub(arguments.len())
                    .ok_or_else(|| {
                        MathError::Contract(
                            "piecewise partial exceeds its proved continuity order".into(),
                        )
                    })?;
                let continuity = match remaining {
                    0 => DerivativeOrder::Value,
                    1 => DerivativeOrder::First,
                    _ => DerivativeOrder::Second,
                };
                let tail = &stages[index + 1..];
                if then
                    .len()
                    .saturating_add(otherwise.len())
                    .saturating_add(tail.len() * 2)
                    > limit
                {
                    return Err(MathError::Limit("piecewise partial continuation"));
                }
                let mut a = then.clone();
                a.extend_from_slice(tail);
                let mut b = otherwise.clone();
                b.extend_from_slice(tail);
                let left_atom = values.get(left).cloned().unwrap_or(library::formal(*left)?);
                let right_atom = values
                    .get(right)
                    .cloned()
                    .unwrap_or(library::formal(*right)?);
                if next_slot.saturating_add(2) > slot_limit {
                    return Err(MathError::SlotLimit {
                        required: next_slot.saturating_add(2),
                        available: slot_limit,
                    });
                }
                let left = *next_slot;
                let right = left + 1;
                *next_slot += 2;
                return Ok(vec![
                    Stage::Block {
                        expressions: vec![left_atom, right_atom],
                        outputs: vec![left, right],
                        source,
                    },
                    Stage::Branch {
                        comparison: *comparison,
                        left,
                        right,
                        continuity,
                        then: partial_paths(
                            &a,
                            values.clone(),
                            value,
                            arguments,
                            output,
                            source,
                            limit,
                            work,
                            next_slot,
                            slot_limit,
                        )?,
                        otherwise: partial_paths(
                            &b, values, value, arguments, output, source, limit, work, next_slot,
                            slot_limit,
                        )?,
                    },
                ]);
            }
        }
    }
    let mut atom = expand(value, &values, limit)?;
    for argument in arguments {
        atom = atom.derivative(
            Indeterminate::try_from(argument.atom.clone()).map_err(MathError::Library)?,
        );
        bounded(&atom, limit)?;
    }
    Ok(vec![Stage::Block {
        expressions: vec![atom],
        outputs: vec![output],
        source,
    }])
}
pub(super) fn strengthen_guards(stages: &mut [Stage], order: usize) {
    for stage in stages {
        match stage {
            Stage::Require { order: o, .. } => {
                *o = match (*o, order) {
                    (DerivativeOrder::Second, 1) => DerivativeOrder::First,
                    _ => DerivativeOrder::Value,
                }
            }
            Stage::Branch {
                then, otherwise, ..
            } => {
                strengthen_guards(then, order);
                strengthen_guards(otherwise, order);
            }
            _ => {}
        }
    }
}
