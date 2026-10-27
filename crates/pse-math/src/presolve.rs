// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Loss-aware projections into the library's presolve vocabulary, never an evaluator.
use crate::{
    MathError,
    assembly::CasePlan,
    binding::CaseValues,
    factorable::{
        Constraint, FactorableError, FactorableProgram, FactorableRequest, Node, NodeId,
        ObligationKind, ObligationScope, ProjectedObligation,
    },
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

pub use crate::class_evidence::{
    ClassDependency, ClassEvidence, ClassRequest, ClassStatus, ClassWitness,
};

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
impl ObligationStatus {
    /// Stable report spelling, independent of Rust debug output.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discharged => "discharged",
            Self::Violated => "violated",
            Self::Unestablished => "unestablished",
        }
    }
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
    /// Assessed coefficient-class meaning; absent optional fields are never a negative proof.
    pub class_status: ClassStatus,
    /// Finite proof allowance left by the base projection, continued by class demands.
    pub proof_remaining: usize,
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
    /// The curvature pass's convexity fact over the projected program (ADR-0121), for a
    /// program that is not a coefficient program; `None` when it is one, whose class the
    /// coefficient snapshot establishes.
    pub curvature: Option<crate::convexity::Convexity>,
    /// Bound on the polynomial degree of every later lexicographic objective (ADR-0111);
    /// absence means unestablished. A coefficient model needs them linear.
    pub lexicographic_degree: Option<u8>,
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
            + match &self.class_status {
                ClassStatus::Pending(dependencies) => {
                    dependencies.capacity() * size_of::<ClassDependency>()
                }
                _ => 0,
            }
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
    pub fn presolve_facts(
        &self,
        values: &CaseValues,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Facts, MathError> {
        let base = self.presolve_domain_facts(values, limit, cancel)?;
        crate::class_evidence::shape_facts(self, values, &base, limit, cancel)
    }
    /// Prepare value/domain/tape evidence without affine or objective derivative proof.
    pub fn presolve_domain_facts(
        &self,
        values: &CaseValues,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Facts, MathError> {
        if limit == 0 {
            return Err(MathError::Limit("presolve tape extent"));
        }
        let topology = self.projection_topology();
        let columns = &topology.columns;
        let rows = &topology.rows;
        let mut remaining = limit
            .checked_sub(rows.len())
            .ok_or(MathError::Limit("presolve rows"))?;
        // The default request exports every branch through the auxiliary policy, so the only
        // failures are mathematical ones.
        let program = self
            .factorable_program(values, &FactorableRequest::default(), limit, cancel)
            .map_err(|e| match e {
                FactorableError::Math(e) => e,
                other @ (FactorableError::DisjunctiveBranch { .. }
                | FactorableError::ExactRequired { .. }) => MathError::Contract(other.to_string()),
            })?;
        // Literal population is already node-budgeted. Prepay one peak scratch
        // extent before any finite, non-iterative exact conversion, rather than
        // growing the constructor once for every row's repeated literal.
        let mut constant_scratch=0;
        for node in &program.nodes {
            if cancel.load(Ordering::Relaxed) {return Err(MathError::Cancelled);}
            if let Node::Const(value)=node {constant_scratch=constant_scratch.max(value.enclosure_bytes()?);}
        }
        crate::construction::additional(constant_scratch)?;
        let lower: Vec<_> = program.variables.iter().map(|v| v.lower).collect();
        let upper: Vec<_> = program.variables.iter().map(|v| v.upper).collect();
        let mut facts = Facts {
            class_status: ClassStatus::Unassessed,
            proof_remaining: 0,
            key: self.structure().key(),
            structure: self.structure().key(),
            values: program.values.clone(),
            affine: vec![None; rows.len()],
            row_sources: topology.row_sources.clone(),
            tapes: Vec::with_capacity(rows.len()),
            complete: Vec::with_capacity(rows.len()),
            objective_linear: vec![false; columns.len()],
            objective_degree: None,
            lexicographic_degree: None,
            obligations: BTreeMap::new(),
            has_guards: false,
            signs: BTreeMap::new(),
            curvature: None,
        };
        for b in self.structure().instances() {
            facts
                .obligations
                .insert(b.instance, ObligationStatus::Discharged);
        }
        for obligation in &program.obligations {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let status = admission(&program, obligation, &lower, &upper, &mut remaining, cancel)?;
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
                && let Some((column, positive)) = single_column_sign(&program, argument)
            {
                use crate::guarded::Condition;
                let sign = match condition {
                    Condition::Positive => Some(GuardSign {
                        positive,
                        strict: true,
                    }),
                    Condition::Nonnegative => Some(GuardSign {
                        positive,
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
                Some(root) => optional_tape(tape(&program, root, &mut remaining, true, cancel))?,
                None => None,
            };
            let tape = tape.unwrap_or_else(|| FbbtTape {
                ops: vec![Op::Opaque],
            });
            facts.complete.push(!tape.ops.contains(&Op::Opaque));
            facts.tapes.push(tape);
        }
        for binding in self.structure().instances() {
            facts.has_guards |= self.bodies()[&binding.body].has_obligations();
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
        // The curvature pass reads the program built above (ADR-0121); a coefficient
        // program's class belongs to its coefficient snapshot.
        if !facts.coefficient_eligible() {
            facts.curvature = Some(crate::curvature::fact(
                &program,
                crate::convexity::EXACT_OPERATIONS,
                cancel,
            )?);
        }
        let mut h = FramedHasher::new(pse_ids::Frame::MathBoundFactsV3);
        h.hash(&facts.structure)
            .str("pounce-nlp-0.12.0;projection-v5;constant-enclosures;exact-exponents;value-domain;explicit-class-proof;aggregate-coefficients;curvature-v1");
        for (id, b) in &facts.values {
            h.str(&id.to_string()).u64(*b);
        }
        facts.key = h.finish_hash();
        facts.proof_remaining = remaining;
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
    cancel: &AtomicBool,
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
        let Some(tape) = optional_tape(tape(program, root, remaining, false, cancel))? else {
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
fn single_column_sign(program: &FactorableProgram, node: NodeId) -> Option<(usize, bool)> {
    match program.nodes.get(node)? {
        Node::Var(c) => Some((*c, true)),
        Node::Product(children) if children.len() == 2 => {
            match (
                program.nodes.get(children[0])?,
                program.nodes.get(children[1])?,
            ) {
                (Node::Const(k), Node::Var(c)) | (Node::Var(c), Node::Const(k)) => {
                    k.shared_sign().map(|positive|(*c,positive))
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
    cancel: &AtomicBool,
) -> Result<FbbtTape, MathError> {
    let mut prepaid = prepaid;
    let mut reachable = BTreeSet::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {return Err(MathError::Cancelled);}
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
        if cancel.load(Ordering::Relaxed) {return Err(MathError::Cancelled);}
        let emitted = match &program.nodes[n] {
            Node::Var(c) => push(&mut ops, Op::Var(*c))?,
            Node::Aux(_) => push(&mut ops, Op::Opaque)?,
            Node::Const(c) => {
                let (lo,hi)=c.enclosure()?;
                push(&mut ops,Op::ConstInterval {lo,hi})?
            }
            Node::Sum(children) | Node::Product(children) => {
                let sum = matches!(program.nodes[n], Node::Sum(_));
                let (first, rest) = children
                    .split_first()
                    .ok_or_else(|| MathError::Contract("empty projected operation".into()))?;
                let mut acc = slot(&slots, first)?;
                for child in rest {
                    if cancel.load(Ordering::Relaxed) {return Err(MathError::Cancelled);}
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
                if exponent.is_exact_half() {
                    push(&mut ops, Op::Sqrt(b))?
                } else if let Some(e)=exponent.small_integer() {
                    let p = push(&mut ops, Op::PowInt(b,e.unsigned_abs()))?;
                    if e<0 {
                        let one=push(&mut ops,Op::Const(1.0))?;
                        push(&mut ops,Op::Div(one,p))?
                    } else {p}
                } else {
                    push(&mut ops,Op::Opaque)?
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

#[cfg(test)]
mod tape_construction_tests {
    use super::*;
    #[test]
    fn late_tape_refusal_keeps_prior_tape_and_visits_full_candidate_population() {
        let mut nodes = (0..16).map(Node::Var).collect::<Vec<_>>();
        nodes.push(Node::Sum((0..16).collect()));
        nodes.push(Node::Pow {
            base: 16,
            exponent: crate::factorable::Constant::Rational(
                symbolica::domains::rational::Rational::from(-2),
            ),
        });
        let program = crate::factorable::tape_construction_fixture(nodes);
        let mut remaining = 35;
        let retained = tape(&program, 17, &mut remaining, false, &AtomicBool::new(false)).unwrap();
        assert_eq!(retained.ops.len(), 34); // sixteen leaves, fifteen adds, three negative-power ops
        assert!(retained.first_invalid_slot().is_none());
        assert_eq!(remaining, 1);
        assert!(matches!(
            tape(&program, 17, &mut remaining, true, &AtomicBool::new(false)),
            Err(MathError::Limit("presolve tape extent"))
        ));
        // The second call builds its full reachable-node set, initial operation
        // capacity and slot map even with only one operation of budget left.
        assert_eq!(remaining, 0);
        assert_eq!(retained.ops.len(), 34);
    }
    #[test]
    fn nary_alias_tape_population_counts_operand_edges_beyond_reachable_nodes() {
        let program = crate::factorable::tape_construction_fixture(vec![
            Node::Var(0),
            Node::Sum(vec![0; 256]),
        ]);
        let mut remaining = 256;
        let complete = tape(&program, 1, &mut remaining, false, &AtomicBool::new(false)).unwrap();
        assert_eq!(complete.ops.len(), 256);
        assert!(complete.first_invalid_slot().is_none());
        let mut insufficient = 255;
        assert!(matches!(
            tape(&program, 1, &mut insufficient, false, &AtomicBool::new(false)),
            Err(MathError::Limit("presolve tape extent"))
        ));
        assert_eq!(insufficient, 0);
    }
}

#[cfg(test)]
mod rational_interval_tests {
    use super::*;
    use crate::factorable::{Constant,Rational,tape_construction_fixture};
    fn emit(nodes:Vec<Node>,root:usize)->FbbtTape {
        let program=tape_construction_fixture(nodes);
        tape(&program,root,&mut 4096,false,&AtomicBool::new(false)).unwrap()
    }
    #[test]
    fn exact_rational_leaf_prevents_amplified_cancellation_from_shrinking_enclosure() {
        // d=6004799503160661/2^54 is the nearest binary64 BELOW 1/3.
        // Therefore (1/3-d)*2^100 = 2^46/3, independently of this tape.
        let d=6004799503160661.0/18014398509481984.0;
        let tape=emit(vec![Node::Var(0),Node::Const(Constant::Rational(Rational::new(1,3))),Node::Sum(vec![0,1]),Node::Const(Constant::Float(2.0_f64.powi(100))),Node::Product(vec![2,3])],4);
        let values=pounce_presolve::fbbt::forward_pass(&tape,&[-d],&[-d]).unwrap();
        let range=pounce_presolve::fbbt::forward_result(&values);
        let exact=Rational::from(1_i64<<46)/Rational::from(3);
        assert!(Rational::try_from(range.lo).unwrap()<=exact);
        assert!(Rational::try_from(range.hi).unwrap()>=exact);
        assert!(range.hi>1.0e13);
    }
    #[test]
    fn constant_interval_contains_exact_source_and_existing_double_rounded_projection() {
        // Numerator rounds up to2^53+4, denominator down to2^53, so
        // the existing two-conversion quotient is1+2ulp. The exact ratio is
        // strictly between1 and1+1ulp, with nearest image1+1ulp.
        let exact=Rational::from((9007199254740995_i64,9007199254740993_i64));
        let constant=Constant::Rational(exact.clone());
        assert_eq!(constant.value().to_bits(),1.0_f64.to_bits()+2);
        // This is the same source-owned map_coeff conversion consumed by
        // scalar, defined-scalar and Taylor floating evaluator construction.
        use symbolica::atom::{Atom,AtomCore};
        crate::initialize().unwrap();
        let expression=Atom::num(exact.clone());
        let mut numerical=expression.evaluator::<Atom>(&[]).build().unwrap().map_coeff(&|value|value.re.to_f64());
        assert_eq!(numerical.evaluate_single(&[]).to_bits(),constant.value().to_bits());
        assert_eq!(crate::rational_binary64::nearest(&exact).unwrap().to_bits(),1.0_f64.to_bits()+1);
        let (lo,hi)=constant.enclosure().unwrap();
        assert!(Rational::try_from(lo).unwrap()<=exact && exact<=Rational::try_from(hi).unwrap());
        assert!(lo<=constant.value() && constant.value()<=hi);
    }
    #[test]
    fn exact_exponent_dispatch_never_rounds_near_integer_or_half_into_supported_power() {
        let epsilon=Rational::from((1_i64,1_i64<<54));
        for exponent in [Rational::one()+&epsilon,Rational::new(1,2)+Rational::from((1_i64,1_i64<<56))] {
            let tape=emit(vec![Node::Var(0),Node::Pow {base:0,exponent:Constant::Rational(exponent)}],1);
            assert_eq!(tape.ops.last(),Some(&Op::Opaque));
        }
        for (exponent,expected) in [(Rational::from(2),Op::PowInt(0,2)),(Rational::new(1,2),Op::Sqrt(0))] {
            let tape=emit(vec![Node::Var(0),Node::Pow {base:0,exponent:Constant::Rational(exponent)}],1);
            assert_eq!(tape.ops.last(),Some(&expected));
        }
    }
    #[test]
    fn underflowed_coefficient_cannot_fabricate_a_negative_hard_sign() {
        use symbolica::domains::integer::Integer;
        let tiny=Rational::from((Integer::one(),Integer::from(2).pow(1200)));
        let constant=Constant::Rational(tiny);
        assert_eq!(constant.value(),0.0);
        let program=tape_construction_fixture(vec![Node::Var(0),Node::Const(constant),Node::Product(vec![0,1])]);
        assert_eq!(single_column_sign(&program,2),None);
    }
}
