// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Loss-aware projections into the library's presolve vocabulary, never an evaluator.
use crate::{
    MathError,
    assembly::CasePlan,
    binding::{CaseValues, Target},
    factorable::{
        Constraint, FactorableError, FactorableProgram, FactorableRequest, Node, NodeId,
        ObligationKind,
        ObligationScope, ProjectedObligation,
    },
    library,
};
use pounce_nlp::expression_provider::{FbbtOp as Op, FbbtTape};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::atom::{Atom, AtomCore, Indeterminate};

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
    /// The outward-rounded FBBT enclosure of row `index` over a free-column box, from the
    /// library's forward pass. `None` when the row's tape does not cover every subexpression.
    /// # Errors
    /// A box whose length differs from the tape's coordinates.
    pub fn row_enclosure(
        &self,
        index: usize,
        lower: &[f64],
        upper: &[f64],
    ) -> Result<Option<(f64, f64)>, MathError> {
        if !self.complete.get(index).copied().unwrap_or(false) {
            return Ok(None);
        }
        let tape = self
            .tapes
            .get(index)
            .ok_or_else(|| MathError::Contract("row enclosure outside the projection".into()))?;
        let values = pounce_presolve::fbbt::forward_pass(tape, lower, upper)
            .map_err(|e| MathError::Contract(format!("row enclosure: {e:?}")))?;
        let interval = pounce_presolve::fbbt::forward_result(&values);
        Ok(Some((interval.lo, interval.hi)))
    }
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
    /// Tapes and obligation admission come from the shared stage projection
    /// ([`crate::factorable`]): a large factorable body keeps a complete tape, and a
    /// validity predicate is checked through its closed conjunction. Affine proofs and the
    /// objective degree use the optional flattened expressions.
    ///
    /// # Errors
    /// Rejects missing or nonfinite consumed values, insufficient row storage,
    /// cancellation, or a failed symbolic projection. Exhausted optional tape
    /// construction leaves an opaque row or an unestablished obligation.
    #[expect(
        clippy::too_many_lines,
        reason = "one bounded traversal accumulates row proofs, admission and native tapes together"
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
        let mut remaining = limit
            .checked_sub(rows.len())
            .ok_or(MathError::Limit("presolve rows"))?;
        // The default request exports every branch through the auxiliary policy, so the only
        // failures are mathematical ones.
        let program = self
            .factorable_program(values, &FactorableRequest::default(), limit, cancel)
            .map_err(|e| match e {
                FactorableError::Math(e) => e,
                other @ FactorableError::DisjunctiveBranch { .. } => {
                    MathError::Contract(other.to_string())
                }
            })?;
        let lower: Vec<_> = program.variables.iter().map(|v| v.lower).collect();
        let upper: Vec<_> = program.variables.iter().map(|v| v.upper).collect();
        let mut facts = Facts {
            key: self.structure().key(),
            structure: self.structure().key(),
            values: program.values.clone(),
            affine: vec![
                Some(AffineRow {
                    entries: BTreeMap::new(),
                    constant: 0.0
                });
                rows.len()
            ],
            row_sources: vec![vec![]; rows.len()],
            tapes: Vec::with_capacity(rows.len()),
            complete: Vec::with_capacity(rows.len()),
            objective_linear: vec![true; columns.len()],
            objective_degree: Some(0),
            obligations: BTreeMap::new(),
            has_guards: false,
            signs: BTreeMap::new(),
        };
        for b in self.structure().instances() {
            facts
                .obligations
                .insert(b.instance, ObligationStatus::Discharged);
        }
        // An instance whose projection exhausted the budget is admitted only when its body
        // retains no obligation at all.
        for b in self.structure().instances() {
            if program.incomplete.contains(&b.instance)
                && !self.bodies()[&b.body].obligations.is_empty()
            {
                facts
                    .obligations
                    .insert(b.instance, ObligationStatus::Unestablished);
            }
        }
        for obligation in &program.obligations {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let status = admission(&program, obligation, &lower, &upper, &mut remaining)?;
            let current = facts
                .obligations
                .entry(obligation.instance)
                .or_insert(ObligationStatus::Discharged);
            *current = match (*current, status) {
                (ObligationStatus::Violated, _) | (_, ObligationStatus::Violated) => {
                    ObligationStatus::Violated
                }
                (ObligationStatus::Unestablished, _) | (_, ObligationStatus::Unestablished) => {
                    ObligationStatus::Unestablished
                }
                _ => ObligationStatus::Discharged,
            };
            // Only an obligation enforced at every evaluation is a hard sign domain.
            if let (
                ObligationKind::Require(condition),
                ObligationScope::Unconditional,
                Some(argument),
            ) = (obligation.kind, obligation.scope, obligation.argument)
                && let Some((column, coefficient)) = single_column(&program, argument)
            {
                use crate::guarded::Condition;
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
                    Condition::Nonzero => None,
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
        }
        for row in &program.rows {
            let tape = match row.expression {
                // The original expression remains executable. A partial native tape
                // cannot support any interval conclusion.
                Some(root) => optional_tape(tape(&program, root, &mut remaining, true))?,
                None => None,
            };
            let tape = tape.unwrap_or_else(|| FbbtTape {
                ops: vec![Op::Opaque],
            });
            facts.complete.push(!tape.ops.contains(&Op::Opaque));
            facts.tapes.push(tape);
        }
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
                    bindings.push((formal, None, 0.0, s.scale() * v + s.offset()));
                }
            }
            let admitted = facts.obligations[&b.instance];
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
        for tape in &facts.tapes {
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
            .str("pounce-nlp-0.12.0;projection-v3;stage-dag");
        for (id, b) in &facts.values {
            h.str(&id.to_string()).u64(*b);
        }
        facts.key = h.finish_hash();
        Ok(facts)
    }
}
/// Admission of one retained obligation over the complete selected variable box.
fn admission(
    program: &FactorableProgram,
    obligation: &ProjectedObligation,
    lower: &[f64],
    upper: &[f64],
    remaining: &mut usize,
) -> Result<ObligationStatus, MathError> {
    use crate::guarded::Condition;
    // A requirement is checked through its exact condition on the argument; a domain
    // predicate through its closed conjunction with strictness, which must be complete.
    let (mut status, checks): (_, Vec<(NodeId, Check)>) =
        match (obligation.kind, obligation.argument) {
            (ObligationKind::Require(condition), Some(argument)) => (
                ObligationStatus::Discharged,
                vec![(argument, Check::Condition(condition))],
            ),
            _ => (
                if obligation.represented {
                    ObligationStatus::Discharged
                } else {
                    ObligationStatus::Unestablished
                },
                obligation
                    .constraints
                    .iter()
                    .map(|c| (c.expression, Check::Closed(*c)))
                    .collect(),
            ),
        };
    for (root, check) in checks {
        let Some(tape) = optional_tape(tape(program, root, remaining, false))? else {
            status = ObligationStatus::Unestablished;
            continue;
        };
        if tape.ops.contains(&Op::Opaque) {
            status = ObligationStatus::Unestablished;
            continue;
        }
        let Ok(intervals) = pounce_presolve::fbbt::forward_pass(&tape, lower, upper) else {
            status = ObligationStatus::Unestablished;
            continue;
        };
        let range = pounce_presolve::fbbt::forward_result(&intervals);
        let (lo, hi) = (range.lo, range.hi);
        let (valid, violated) = match check {
            Check::Condition(condition) => (
                lo <= hi
                    && match condition {
                        Condition::Positive => lo > 0.0,
                        Condition::Nonnegative => lo >= 0.0,
                        Condition::Nonzero => lo > 0.0 || hi < 0.0,
                    },
                lo > hi
                    || match condition {
                        Condition::Positive => hi <= 0.0,
                        Condition::Nonnegative => hi < 0.0,
                        Condition::Nonzero => lo == 0.0 && hi == 0.0,
                    },
            ),
            Check::Closed(c) => (
                lo <= hi
                    && (if c.strict {
                        lo > c.lower
                    } else {
                        lo >= c.lower
                    })
                    && (if c.strict {
                        hi < c.upper
                    } else {
                        hi <= c.upper
                    }),
                lo > hi
                    || (if c.strict {
                        hi <= c.lower
                    } else {
                        hi < c.lower
                    })
                    || (if c.strict {
                        lo >= c.upper
                    } else {
                        lo > c.upper
                    }),
            ),
        };
        if violated {
            // A region-local obligation constrains only points where its region is taken.
            return Ok(if obligation.scope == ObligationScope::Unconditional {
                ObligationStatus::Violated
            } else {
                ObligationStatus::Unestablished
            });
        }
        if !valid {
            status = ObligationStatus::Unestablished;
        }
    }
    Ok(status)
}
#[derive(Clone, Copy, Debug)]
enum Check {
    Condition(crate::guarded::Condition),
    Closed(Constraint),
}
/// `coefficient * column` with no offset, as a hard sign domain requires.
fn single_column(program: &FactorableProgram, node: NodeId) -> Option<(usize, f64)> {
    match program.nodes.get(node)? {
        Node::Var(c) => Some((*c, 1.0)),
        Node::Product(children) if children.len() == 2 => {
            match (
                program.nodes.get(children[0])?,
                program.nodes.get(children[1])?,
            ) {
                (Node::Const(k), Node::Var(c)) | (Node::Var(c), Node::Const(k)) => {
                    Some((*c, k.value()))
                }
                _ => None,
            }
        }
        _ => None,
    }
}
/// Native FBBT tape of one projected node, emitting every reachable node once. A row's
/// first operation was charged when the row inventory was admitted (`prepaid`).
fn tape(
    program: &FactorableProgram,
    root: NodeId,
    remaining: &mut usize,
    prepaid: bool,
) -> Result<FbbtTape, MathError> {
    let mut prepaid = prepaid;
    let mut reachable = BTreeSet::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if !reachable.insert(n) {
            continue;
        }
        match program.nodes.get(n) {
            Some(Node::Sum(c) | Node::Product(c)) => stack.extend(c),
            Some(
                Node::Pow { base: i, .. }
                | Node::Exp(i)
                | Node::Log(i)
                | Node::Abs(i)
                | Node::Sin(i)
                | Node::Cos(i),
            ) => stack.push(*i),
            Some(Node::Var(_) | Node::Aux(_) | Node::Const(_)) => {}
            None => return Err(MathError::Contract("projected node out of range".into())),
        }
    }
    let mut ops = Vec::with_capacity(reachable.len());
    let mut push = |ops: &mut Vec<Op>, op: Op| -> Result<usize, MathError> {
        if prepaid {
            prepaid = false;
        } else if *remaining == 0 {
            return Err(MathError::Limit("presolve tape extent"));
        } else {
            *remaining -= 1;
        }
        ops.push(op);
        Ok(ops.len() - 1)
    };
    let mut slots: HashMap<NodeId, usize> = HashMap::with_capacity(reachable.len());
    let slot = |slots: &HashMap<NodeId, usize>, n: &NodeId| {
        slots
            .get(n)
            .copied()
            .ok_or_else(|| MathError::Contract("projected node order".into()))
    };
    // Node identities are topological: children precede their parents, and the root is
    // the largest reachable identity, so it is the tape result.
    for n in reachable {
        let emitted = match &program.nodes[n] {
            Node::Var(c) => push(&mut ops, Op::Var(*c))?,
            Node::Aux(_) => push(&mut ops, Op::Opaque)?,
            Node::Const(c) => push(&mut ops, Op::Const(c.value()))?,
            Node::Sum(children) | Node::Product(children) => {
                let sum = matches!(program.nodes[n], Node::Sum(_));
                let (first, rest) = children
                    .split_first()
                    .ok_or_else(|| MathError::Contract("empty projected operation".into()))?;
                let mut acc = slot(&slots, first)?;
                for child in rest {
                    let next = slot(&slots, child)?;
                    acc = push(
                        &mut ops,
                        if sum {
                            Op::Add(acc, next)
                        } else {
                            Op::Mul(acc, next)
                        },
                    )?;
                }
                acc
            }
            Node::Pow { base, exponent } => {
                let b = slot(&slots, base)?;
                let e = exponent.value();
                if e.to_bits() == 0.5_f64.to_bits() {
                    push(&mut ops, Op::Sqrt(b))?
                } else if e.fract() == 0.0 && e.abs() <= 64.0 {
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "the branch proved an integer absolute exponent at most 64"
                    )]
                    let power = e.abs() as u32;
                    let p = push(&mut ops, Op::PowInt(b, power))?;
                    if e < 0.0 {
                        let one = push(&mut ops, Op::Const(1.0))?;
                        push(&mut ops, Op::Div(one, p))?
                    } else {
                        p
                    }
                } else {
                    push(&mut ops, Op::Opaque)?
                }
            }
            Node::Exp(i) => push(&mut ops, Op::Exp(slot(&slots, i)?))?,
            Node::Log(i) => push(&mut ops, Op::Ln(slot(&slots, i)?))?,
            Node::Abs(i) => push(&mut ops, Op::Abs(slot(&slots, i)?))?,
            Node::Sin(i) => push(&mut ops, Op::Sin(slot(&slots, i)?))?,
            Node::Cos(i) => push(&mut ops, Op::Cos(slot(&slots, i)?))?,
        };
        slots.insert(n, emitted);
    }
    if slot(&slots, &root)? + 1 != ops.len() {
        return Err(MathError::Contract("projected tape result".into()));
    }
    Ok(FbbtTape { ops })
}
fn optional_tape<T>(result: Result<T, MathError>) -> Result<Option<T>, MathError> {
    match result {
        Err(MathError::Limit("presolve tape extent")) => Ok(None),
        other => other.map(Some),
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
    match affine_candidate(a, bindings, scale, c) {
        Err(MathError::CoefficientRange) => Ok(None),
        other => other,
    }
}
fn affine_candidate(
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
