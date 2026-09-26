// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Loss-aware projections into the library's presolve vocabulary, never an evaluator.
use crate::{
    MathError,
    assembly::CasePlan,
    binding::{CaseValues, Target},
    library,
};
use pounce_nlp::expression_provider::{FbbtOp as Op, FbbtTape};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::atom::{Atom, AtomCore, AtomView, Indeterminate, Symbol};

/// One proved affine row, including its original constant.
#[derive(Clone, Debug, PartialEq)]
pub struct AffineRow {
    /// Canonical global column coefficients; exact represented zeros are omitted.
    pub entries: BTreeMap<usize, f64>,
    /// Value at zero coordinates.
    pub constant: f64,
}
/// A domain obligation established over the complete selected variable box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObligationStatus {
    /// Every retained obligation holds throughout the admitted box.
    Discharged,
    /// An obligation fails throughout the admitted box.
    Violated,
    /// The projection cannot establish validity throughout the box.
    Unestablished,
}
/// A hard sign domain implied by a retained original obligation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuardSign {
    /// Positive versus negative side of zero.
    pub positive: bool,
    /// Zero itself is excluded.
    pub strict: bool,
}
/// Immutable source-derived view. Tapes are owned by pounce-nlp.
#[derive(Clone, Debug)]
pub struct Facts {
    /// Complete structure and consumed assumption identity.
    pub key: ContentHash,
    /// Original case layout.
    pub structure: ContentHash,
    /// Values whose change invalidates this projection.
    pub values: BTreeMap<SemanticId, u64>,
    /// Independent row proofs, including in mixed nonlinear models.
    pub affine: Vec<Option<AffineRow>>,
    /// Original instance/output contributions for each row, including duplicate occurrences.
    pub row_sources: Vec<Vec<(SemanticId, usize)>>,
    /// Row tapes in the same original coordinates as the callbacks.
    pub tapes: Vec<FbbtTape>,
    /// Whether each tape covers every subexpression.
    pub complete: Vec<bool>,
    /// Original guards exist even when normalization erases their arithmetic.
    pub has_guards: bool,
    /// Source variables whose hard sign domain can be represented without relaxing a guard.
    pub signs: BTreeMap<SemanticId, GuardSign>,
    /// Per-instance domain admission shared by all mathematical consumers.
    pub obligations: BTreeMap<SemanticId, ObligationStatus>,
    /// Variables with proved affine objective dependence; unknown is false.
    pub objective_linear: Vec<bool>,
    /// Bound on polynomial objective degree under these assumptions; absence means unestablished.
    pub objective_degree: Option<u8>,
}
impl Facts {
    /// Refuse stale numeric assumptions without including free trial values.
    pub fn matches(&self, plan: &CasePlan, values: &CaseValues) -> bool {
        self.structure == plan.structure().key()
            && self
                .values
                .iter()
                .all(|(id, b)| values.scalars.get(id).is_some_and(|v| v.to_bits() == *b))
    }
    /// Bounded accounting for the principal owned allocations.
    pub fn bytes(&self) -> usize {
        self.tapes
            .iter()
            .map(|t| t.ops.capacity() * size_of::<Op>())
            .sum::<usize>()
            + self
                .affine
                .iter()
                .flatten()
                .map(|r| r.entries.len() * 64)
                .sum::<usize>()
            + self.row_sources.capacity() * size_of::<Vec<(SemanticId, usize)>>()
            + self
                .row_sources
                .iter()
                .map(|r| r.capacity() * size_of::<(SemanticId, usize)>())
                .sum::<usize>()
            + self.signs.len() * 64
            + self.obligations.len() * 64
            + self.tapes.capacity() * size_of::<FbbtTape>()
            + self.affine.capacity() * size_of::<Option<AffineRow>>()
            + self.values.len() * 64
            + self.objective_linear.capacity()
            + self.complete.capacity()
    }
}
impl CasePlan {
    /// Build independently useful affine proofs and native FBBT tapes.
    ///
    /// # Errors
    /// Rejects missing or nonfinite consumed values, exhausted tape allowance,
    /// cancellation, or a failed symbolic projection.
    #[expect(
        clippy::too_many_lines,
        reason = "one bounded traversal accumulates row proofs and native tapes together"
    )]
    pub fn presolve_facts(
        &self,
        values: &CaseValues,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Facts, MathError> {
        if limit == 0 {
            return Err(MathError::Limit("presolve tape extent"));
        }
        let columns: BTreeMap<_, _> = self
            .columns()
            .iter()
            .enumerate()
            .map(|(i, id)| (*id, i))
            .collect();
        let rows: BTreeMap<_, _> = self
            .structure()
            .rows()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect();
        let mut facts = Facts {
            key: self.structure().key(),
            structure: self.structure().key(),
            values: BTreeMap::new(),
            affine: vec![
                Some(AffineRow {
                    entries: BTreeMap::new(),
                    constant: 0.0
                });
                rows.len()
            ],
            row_sources: vec![vec![]; rows.len()],
            tapes: (0..rows.len())
                .map(|_| FbbtTape {
                    ops: vec![Op::Const(0.0)],
                })
                .collect(),
            complete: vec![true; rows.len()],
            objective_linear: vec![true; columns.len()],
            objective_degree: Some(0),
            obligations: BTreeMap::new(),
            has_guards: false,
            signs: BTreeMap::new(),
        };
        let mut remaining = limit
            .checked_sub(rows.len())
            .ok_or(MathError::Limit("presolve rows"))?;
        let bounds: Vec<_> = self
            .structure()
            .variables()
            .iter()
            .filter(|v| !v.fixed)
            .map(|v| {
                (
                    if v.domain.is_semi() {
                        0.0
                    } else {
                        v.lower.unwrap_or(f64::NEG_INFINITY)
                    },
                    v.upper.unwrap_or(f64::INFINITY),
                )
            })
            .collect();
        let lower: Vec<_> = bounds.iter().map(|b| b.0).collect();
        let upper: Vec<_> = bounds.iter().map(|b| b.1).collect();
        let mut row_memos: HashMap<(SemanticId, usize), HashMap<Atom, usize>> = HashMap::new();
        for b in self.structure().instances() {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let body = &self.bodies()[&b.body];
            facts.has_guards |= body.has_obligations();
            let mut bindings = Vec::new();
            for (i, s) in b.slots.iter().enumerate() {
                let formal = library::formal(i)?;
                if let Some(&column) = columns.get(&s.source()) {
                    bindings.push((formal, Some(column), s.scale(), s.offset()));
                } else {
                    let v = *values
                        .scalars
                        .get(&s.source())
                        .ok_or_else(|| MathError::Contract("missing presolve parameter".into()))?;
                    if !v.is_finite() {
                        return Err(MathError::Contract("nonfinite presolve parameter".into()));
                    }
                    facts.values.insert(s.source(), v.to_bits());
                    bindings.push((formal, None, 0.0, s.scale() * v + s.offset()));
                }
            }
            let mut admitted = ObligationStatus::Discharged;
            for (expression, condition) in &body.obligations {
                let Some(expression) = expression else {
                    admitted = ObligationStatus::Unestablished;
                    continue;
                };
                use crate::guarded::Condition;
                if let Some(row) = affine(expression, &bindings, 1.0, cancel)?
                    && row.constant == 0.0
                    && row.entries.len() == 1
                    && let Some((&column, &coefficient)) = row.entries.first_key_value()
                {
                    let sign = match condition {
                        Condition::Positive => Some(GuardSign {
                            positive: coefficient > 0.0,
                            strict: true,
                        }),
                        Condition::Nonnegative => Some(GuardSign {
                            positive: coefficient > 0.0,
                            strict: false,
                        }),
                        Condition::Nonzero if lower[column] >= 0.0 => Some(GuardSign {
                            positive: true,
                            strict: true,
                        }),
                        Condition::Nonzero if upper[column] <= 0.0 => Some(GuardSign {
                            positive: false,
                            strict: true,
                        }),
                        _ => None,
                    };
                    if let Some(mut sign) = sign {
                        let id = self.columns()[column];
                        if let Some(old) = facts.signs.get(&id) {
                            if old.positive != sign.positive {
                                return Err(MathError::Contract(format!(
                                    "conflicting original sign guards for {id}"
                                )));
                            }
                            sign.strict |= old.strict;
                        }
                        facts.signs.insert(id, sign);
                    }
                }
                let mut ops = Vec::new();
                let mut emitter = Emitter {
                    ops: &mut ops,
                    bindings: &bindings,
                    remaining: &mut remaining,
                    cancel,
                    memo: HashMap::new(),
                };
                emitter.atom(expression.as_view(), 0)?;
                let Ok(intervals) =
                    pounce_presolve::fbbt::forward_pass(&FbbtTape { ops }, &lower, &upper)
                else {
                    admitted = ObligationStatus::Unestablished;
                    continue;
                };
                let interval = pounce_presolve::fbbt::forward_result(&intervals);
                let valid = interval.lo <= interval.hi
                    && match condition {
                        Condition::Positive => interval.lo > 0.0,
                        Condition::Nonnegative => interval.lo >= 0.0,
                        Condition::Nonzero => interval.lo > 0.0 || interval.hi < 0.0,
                    };
                let violated = interval.lo > interval.hi
                    || match condition {
                        Condition::Positive => interval.hi <= 0.0,
                        Condition::Nonnegative => interval.hi < 0.0,
                        Condition::Nonzero => interval.lo == 0.0 && interval.hi == 0.0,
                    };
                if violated {
                    admitted = ObligationStatus::Violated;
                    break;
                }
                if !valid {
                    admitted = ObligationStatus::Unestablished;
                }
            }
            facts.obligations.insert(b.instance, admitted);
            for c in &b.contributions {
                let expression = body.expression(c.output).map(|a| {
                    bindings
                        .iter()
                        .filter(|(_, col, _, _)| col.is_none())
                        .fold(a.clone(), |a, (f, _, _, v)| {
                            a.replace(f.clone()).with(Atom::num(*v))
                        })
                });
                if let Target::Row(id) = c.target {
                    let r = rows[&id];
                    facts.row_sources[r].push((b.instance, c.output));
                    let tape = &mut facts.tapes[r];
                    let previous = tape.ops.len() - 1;
                    let mut emitter = Emitter {
                        ops: &mut tape.ops,
                        bindings: &bindings,
                        remaining: &mut remaining,
                        cancel,
                        memo: row_memos.remove(&(b.instance, r)).unwrap_or_default(),
                    };
                    let v = match &expression {
                        Some(a) => emitter.atom(a.as_view(), 0)?,
                        None => emitter.push(Op::Opaque)?,
                    };
                    let scale = emitter.push(Op::Const(c.scale))?;
                    let v = emitter.push(Op::Mul(v, scale))?;
                    emitter.push(Op::Add(previous, v))?;
                    row_memos.insert((b.instance, r), std::mem::take(&mut emitter.memo));
                    if admitted != ObligationStatus::Discharged || expression.is_none() {
                        facts.affine[r] = None;
                    }
                    if let (Some(target), Some(a)) = (&mut facts.affine[r], &expression) {
                        if let Some(local) = affine(a, &bindings, c.scale, cancel)? {
                            target.constant += local.constant;
                            for (col, v) in local.entries {
                                *target.entries.entry(col).or_default() += v;
                            }
                        } else {
                            facts.affine[r] = None;
                        }
                    }
                } else {
                    let formals: Vec<_> = bindings
                        .iter()
                        .filter(|(_, c, _, _)| c.is_some())
                        .map(|(f, _, _, _)| f.clone())
                        .collect();
                    let degree = if admitted == ObligationStatus::Discharged {
                        expression
                            .as_ref()
                            .map(|a| degree_bound(a, &formals, &mut remaining, cancel))
                            .transpose()?
                            .flatten()
                    } else {
                        None
                    };
                    facts.objective_degree =
                        facts.objective_degree.zip(degree).map(|(a, b)| a.max(b));
                    for (formal, col, _, _) in &bindings {
                        if let Some(col) = col {
                            let proved = expression.as_ref().is_some_and(|a| {
                                derivative(a, formal).is_ok_and(|d| d.is_constant())
                            });
                            facts.objective_linear[*col] &=
                                proved && admitted == ObligationStatus::Discharged;
                        }
                    }
                }
            }
        }
        for (r, tape) in facts.tapes.iter().enumerate() {
            facts.complete[r] = !tape.ops.contains(&Op::Opaque);
            if tape.first_invalid_slot().is_some() {
                return Err(MathError::Contract("invalid derived FBBT tape".into()));
            }
        }
        for r in facts.affine.iter_mut().flatten() {
            r.entries.retain(|_, v| *v != 0.0);
            if !r.constant.is_finite() || r.entries.values().any(|v| !v.is_finite()) {
                return Err(MathError::Contract("nonfinite affine projection".into()));
            }
        }
        let mut h = FramedHasher::new("pse.math.bound-facts.v2");
        h.hash(&facts.structure)
            .str("pounce-nlp-0.12.0;projection-v2");
        for (id, b) in &facts.values {
            h.str(&id.to_string()).u64(*b);
        }
        facts.key = h.finish_hash();
        Ok(facts)
    }
}
/// Symbolica establishes a low-degree bound once for every consumer of the admitted objective.
fn degree_bound(
    a: &Atom,
    formals: &[Atom],
    remaining: &mut usize,
    cancel: &Arc<AtomicBool>,
) -> Result<Option<u8>, MathError> {
    let cost = formals
        .len()
        .checked_mul(formals.len() + 1)
        .and_then(|v| v.checked_div(2));
    let Some(cost) = cost.filter(|c| *c <= *remaining) else {
        return Ok(None);
    };
    *remaining -= cost;
    let mut degree = 0;
    for (i, f) in formals.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let first = derivative(a, f)?;
        if !first.is_zero() {
            degree = degree.max(1);
        }
        for other in &formals[i..] {
            let second = derivative(&first, other)?;
            if !second.is_constant() {
                return Ok(None);
            }
            if !second.is_zero() {
                degree = 2;
            }
        }
    }
    Ok(Some(degree))
}
fn derivative(a: &Atom, f: &Atom) -> Result<Atom, MathError> {
    Ok(
        a.derivative(
            Indeterminate::try_from(f.clone()).map_err(|e| MathError::Library(e.clone()))?,
        ),
    )
}
fn number(a: &Atom, c: &Arc<AtomicBool>) -> Result<f64, MathError> {
    crate::coefficients::number(a, c)
}
fn affine(
    a: &Atom,
    bindings: &[(Atom, Option<usize>, f64, f64)],
    scale: f64,
    c: &Arc<AtomicBool>,
) -> Result<Option<AffineRow>, MathError> {
    let mut result = AffineRow {
        entries: BTreeMap::new(),
        constant: 0.0,
    };
    let mut zero = a.clone();
    for (f, col, s, o) in bindings {
        if let Some(col) = col {
            let d = derivative(a, f)?;
            if !d.is_constant() {
                return Ok(None);
            }
            let v = number(&d, c)? * scale;
            *result.entries.entry(*col).or_default() += v * s;
            result.constant += v * o;
            zero = zero.replace(f.clone()).with(Atom::num(0));
        }
    }
    if !zero.is_constant() {
        return Ok(None);
    }
    result.constant += number(&zero, c)? * scale;
    Ok(Some(result))
}
struct Emitter<'a> {
    ops: &'a mut Vec<Op>,
    bindings: &'a [(Atom, Option<usize>, f64, f64)],
    remaining: &'a mut usize,
    cancel: &'a Arc<AtomicBool>,
    memo: HashMap<Atom, usize>,
}
impl Emitter<'_> {
    fn push(&mut self, op: Op) -> Result<usize, MathError> {
        if *self.remaining == 0 {
            return Err(MathError::Limit("presolve tape extent"));
        }
        *self.remaining -= 1;
        let i = self.ops.len();
        self.ops.push(op);
        Ok(i)
    }
    fn atom(&mut self, a: AtomView<'_>, depth: usize) -> Result<usize, MathError> {
        if let Some(slot) = self.memo.get(&a.to_owned()) {
            return Ok(*slot);
        }
        let slot = self.emit(a, depth)?;
        self.memo.insert(a.to_owned(), slot);
        Ok(slot)
    }
    fn emit(&mut self, a: AtomView<'_>, depth: usize) -> Result<usize, MathError> {
        if depth > 128 {
            return self.push(Op::Opaque);
        }
        if self.cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        if let Some((_, col, s, o)) = self.bindings.iter().find(|(f, _, _, _)| f.as_view() == a) {
            let (col, s, o) = (*col, *s, *o);
            return match col {
                None => self.push(Op::Const(o)),
                Some(col) => {
                    let v = self.push(Op::Var(col))?;
                    let s = self.push(Op::Const(s))?;
                    let v = self.push(Op::Mul(v, s))?;
                    let o = self.push(Op::Const(o))?;
                    self.push(Op::Add(v, o))
                }
            };
        }
        match a {
            AtomView::Num(_) => self.push(Op::Const(number(&a.to_owned(), self.cancel)?)),
            AtomView::Add(v) => {
                let mut result = self.push(Op::Const(0.0))?;
                for x in v {
                    let k = self.atom(x, depth + 1)?;
                    result = self.push(Op::Add(result, k))?;
                }
                Ok(result)
            }
            AtomView::Mul(v) => {
                let mut result = self.push(Op::Const(1.0))?;
                for x in v {
                    let k = self.atom(x, depth + 1)?;
                    result = self.push(Op::Mul(result, k))?;
                }
                Ok(result)
            }
            AtomView::Pow(v) => {
                if matches!(v.get_base(), AtomView::Var(b) if b.get_symbol() == Symbol::E) {
                    let argument = self.atom(v.get_exp(), depth + 1)?;
                    return self.push(Op::Exp(argument));
                }
                let exponent = v.get_exp().to_owned();
                if !exponent.is_constant() {
                    return self.push(Op::Opaque);
                }
                let e = number(&exponent, self.cancel)?;
                let b = self.atom(v.get_base(), depth + 1)?;
                if e.to_bits() == 0.5_f64.to_bits() {
                    self.push(Op::Sqrt(b))
                } else if e.fract() == 0.0 && e.abs() <= 64.0 {
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "the branch proved an integer absolute exponent at most 64"
                    )]
                    let power = e.abs() as u32;
                    let p = self.push(Op::PowInt(b, power))?;
                    if e < 0.0 {
                        let one = self.push(Op::Const(1.0))?;
                        self.push(Op::Div(one, p))
                    } else {
                        Ok(p)
                    }
                } else {
                    self.push(Op::Opaque)
                }
            }
            AtomView::Fun(v) if v.get_nargs() == 1 => {
                let symbol = v.get_symbol();
                let op: Option<fn(usize) -> Op> = match symbol {
                    s if s == Symbol::EXP => Some(Op::Exp),
                    s if s == Symbol::LOG => Some(Op::Ln),
                    s if s == Symbol::SIN => Some(Op::Sin),
                    s if s == Symbol::COS => Some(Op::Cos),
                    s if s == Symbol::ABS => Some(Op::Abs),
                    _ => None,
                };
                if let Some(op) = op {
                    let k = self.atom(v.get(0), depth + 1)?;
                    self.push(op(k))
                } else {
                    self.push(Op::Opaque)
                }
            }
            _ => self.push(Op::Opaque),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fbbt_projection_normalized_transcendentals_and_shared_nodes() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let expressions = [x.exp(), x.log(), x.sin(), x.cos(), x.abs()];
        for expression in expressions {
            let mut ops = Vec::new();
            let mut remaining = 100;
            let cancel = Arc::new(AtomicBool::new(false));
            let bindings = [(x.clone(), Some(0), 1.0, 0.0)];
            let mut emitter = Emitter {
                ops: &mut ops,
                bindings: &bindings,
                remaining: &mut remaining,
                cancel: &cancel,
                memo: HashMap::new(),
            };
            let first = emitter.atom(expression.as_view(), 0).unwrap();
            let count = emitter.ops.len();
            assert_eq!(emitter.atom(expression.as_view(), 0).unwrap(), first);
            assert_eq!(emitter.ops.len(), count);
            let tape = FbbtTape { ops };
            assert!(!tape.ops.contains(&Op::Opaque), "{expression}");
            assert!(tape.first_invalid_slot().is_none());
            let range = pounce_presolve::fbbt::forward_result(
                &pounce_presolve::fbbt::forward_pass(&tape, &[1.0], &[2.0]).unwrap(),
            );
            assert!(range.lo.is_finite() && range.hi.is_finite());
        }
    }
}
