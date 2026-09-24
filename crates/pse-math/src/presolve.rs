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
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::atom::{Atom, AtomCore, AtomView, Indeterminate};

/// One proved affine row, including its original constant.
#[derive(Clone, Debug, PartialEq)]
pub struct AffineRow {
    /// Canonical global column coefficients; exact represented zeros are omitted.
    pub entries: BTreeMap<usize, f64>,
    /// Value at zero coordinates.
    pub constant: f64,
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
    /// Row tapes in the same original coordinates as the callbacks.
    pub tapes: Vec<FbbtTape>,
    /// Whether each tape covers every subexpression.
    pub complete: Vec<bool>,
    /// Variables with proved affine objective dependence; unknown is false.
    pub objective_linear: Vec<bool>,
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
            tapes: (0..rows.len())
                .map(|_| FbbtTape {
                    ops: vec![Op::Const(0.0)],
                })
                .collect(),
            complete: vec![true; rows.len()],
            objective_linear: vec![true; columns.len()],
        };
        let mut remaining = limit
            .checked_sub(rows.len())
            .ok_or(MathError::Limit("presolve rows"))?;
        for b in self.structure().instances() {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let body = &self.bodies()[&b.body];
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
                    let tape = &mut facts.tapes[r];
                    let previous = tape.ops.len() - 1;
                    let mut emitter = Emitter {
                        ops: &mut tape.ops,
                        bindings: &bindings,
                        remaining: &mut remaining,
                        cancel,
                    };
                    let v = match &expression {
                        Some(a) => emitter.atom(a.as_view(), 0)?,
                        None => emitter.push(Op::Opaque)?,
                    };
                    let scale = emitter.push(Op::Const(c.scale))?;
                    let v = emitter.push(Op::Mul(v, scale))?;
                    emitter.push(Op::Add(previous, v))?;
                    if body.has_obligations() || expression.is_none() {
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
                    for (formal, col, _, _) in &bindings {
                        if let Some(col) = col {
                            let proved = expression.as_ref().is_some_and(|a| {
                                derivative(a, formal)
                                    .is_ok_and(|d| d.get_all_symbols(false).is_empty())
                            });
                            facts.objective_linear[*col] &= proved && !body.has_obligations();
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
        let mut h = FramedHasher::new("pse.math.presolve-facts.v1");
        h.hash(&facts.structure)
            .str("pounce-nlp-0.12.0;projection-v1");
        for (id, b) in &facts.values {
            h.str(&id.to_string()).u64(*b);
        }
        facts.key = h.finish_hash();
        Ok(facts)
    }
}
fn derivative(a: &Atom, f: &Atom) -> Result<Atom, MathError> {
    Ok(
        a.derivative(
            Indeterminate::try_from(f.clone()).map_err(|e| MathError::Library(e.clone()))?,
        ),
    )
}
fn number(a: &Atom, c: &Arc<AtomicBool>) -> Result<f64, MathError> {
    crate::coefficients::number(a, library::Optimization::default(), c)
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
            if !d.get_all_symbols(false).is_empty() {
                return Ok(None);
            }
            let v = number(&d, c)? * scale;
            *result.entries.entry(*col).or_default() += v * s;
            result.constant += v * o;
            zero = zero.replace(f.clone()).with(Atom::num(0));
        }
    }
    if !zero.get_all_symbols(false).is_empty() {
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
    #[expect(
        clippy::many_single_char_names,
        reason = "local symbolic nodes and affine scale/offset coordinates"
    )]
    fn atom(&mut self, a: AtomView<'_>, depth: usize) -> Result<usize, MathError> {
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
                let exponent = v.get_exp().to_owned();
                if !exponent.get_all_symbols(false).is_empty() {
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
                let name = symbol.get_name();
                let op: Option<fn(usize) -> Op> = match name {
                    "exp" => Some(Op::Exp),
                    "log" => Some(Op::Ln),
                    "sin" => Some(Op::Sin),
                    "cos" => Some(Op::Cos),
                    "abs" => Some(Op::Abs),
                    "sqrt" => Some(Op::Sqrt),
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
