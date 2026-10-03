// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Disciplined-convex curvature and cone recognition over a factorable program
//! (ADR-0121 Outcome 1).
//!
//! The pass reads the [`FactorableProgram`] preparation already builds, with node signs and
//! ranges from the library's outward-rounded interval arithmetic over the variable box. It
//! proves each node's curvature by DCP composition rules and records the atom that proves
//! it:
//! - `exp` and `log`, entropy `x·log x` and relative entropy `x·log(x/y)` map to the
//!   exponential cone;
//! - `xᵖ` maps to the power cone: convex for even `p`, for `p > 1` on a nonnegative base
//!   and for `p < 0` on a positive base, concave for `0 < p < 1` on a nonnegative base;
//! - `|a|` of an affine argument maps to two linear rows;
//! - `√(Σ wᵢ aᵢ² + c)` over affine `aᵢ` with `w, c ≥ 0` maps to a second-order cone;
//! - a node DCP cannot classify that is a polynomial of degree at most two is decided by an
//!   exact rational LDLᵀ of its quadratic form ([`crate::convexity`]); a certified one is
//!   a sum of weighted squares and maps to a second-order cone.
//!
//! Only programs whose rows and objective are `Exact`, with no auxiliaries, implicit
//! residuals or native forms, whose columns are continuous, and whose retained obligations
//! follow from the box, are recognized. The recognized program's epigraph form
//! ([`Epigraph`]) is the library-neutral cone representation a conic adapter lowers.
use crate::{
    MathError,
    binding::ObjectiveSense,
    convexity::{ConeSummary, Convexity, ConvexityClass, GramFactors, Ldlt, Unrecognized, ldlt},
    factorable::{Constant, FactorableProgram, Fidelity, Node, NodeId, ObligationKind},
    guarded::Condition,
};
use pounce_presolve::fbbt::Interval;
use pse_ids::FramedHasher;
use pse_model::generated::enums::ModelingVariableDomain;
use std::{
    collections::{BTreeMap, HashMap},
    sync::atomic::{AtomicBool, Ordering},
};
use symbolica::domains::rational::Rational;

/// Curvature of a node's function over the variable box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curvature {
    /// Independent of every column.
    Constant,
    /// Affine in the columns.
    Affine,
    /// Convex and not known to be affine.
    Convex,
    /// Concave and not known to be affine.
    Concave,
    /// No curvature was proved.
    Unknown,
}
impl Curvature {
    /// Convex, including affine and constant.
    pub const fn convex(self) -> bool {
        matches!(self, Self::Constant | Self::Affine | Self::Convex)
    }
    /// Concave, including affine and constant.
    pub const fn concave(self) -> bool {
        matches!(self, Self::Constant | Self::Affine | Self::Concave)
    }
    const fn negate(self) -> Self {
        match self {
            Self::Convex => Self::Concave,
            Self::Concave => Self::Convex,
            other => other,
        }
    }
    const fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::Constant, c) | (c, Self::Constant) => c,
            (Self::Affine, c) | (c, Self::Affine) => c,
            (Self::Convex, Self::Convex) => Self::Convex,
            (Self::Concave, Self::Concave) => Self::Concave,
            _ => Self::Unknown,
        }
    }
}

/// An affine function over the epigraph coordinates: the program's columns first, then the
/// auxiliary columns in creation order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Linear {
    /// Coefficients by coordinate; exact zeros are omitted.
    pub terms: BTreeMap<usize, f64>,
    /// Constant term.
    pub constant: f64,
}
impl Linear {
    fn constant(value: f64) -> Self {
        Self {
            terms: BTreeMap::new(),
            constant: value,
        }
    }
    fn coordinate(index: usize) -> Self {
        Self {
            terms: BTreeMap::from([(index, 1.0)]),
            constant: 0.0,
        }
    }
    fn scaled(&self, factor: f64) -> Self {
        Self {
            terms: self
                .terms
                .iter()
                .map(|(k, v)| (*k, factor * v))
                .filter(|(_, v)| *v != 0.0)
                .collect(),
            constant: factor * self.constant,
        }
    }
    fn plus(mut self, other: &Self, factor: f64) -> Self {
        for (k, v) in &other.terms {
            let entry = self.terms.entry(*k).or_insert(0.0);
            *entry += factor * v;
            if *entry == 0.0 {
                self.terms.remove(k);
            }
        }
        self.constant += factor * other.constant;
        self
    }
    /// Value at a point of the epigraph coordinates.
    pub fn at(&self, point: &[f64]) -> f64 {
        self.terms
            .iter()
            .map(|(k, v)| v * point.get(*k).copied().unwrap_or(f64::NAN))
            .sum::<f64>()
            + self.constant
    }
    fn finite(&self) -> bool {
        self.constant.is_finite() && self.terms.values().all(|v| v.is_finite())
    }
}
/// One cone membership of the epigraph form.
#[derive(Clone, Debug, PartialEq)]
pub enum ConeConstraint {
    /// The member is nonnegative.
    Nonnegative(Linear),
    /// `(x, y, z)` in the closed exponential cone: `y·exp(x/y) ≤ z`, `y > 0`.
    Exponential([Linear; 3]),
    /// `(x, y, z)` in the power cone: `x^α·y^(1−α) ≥ |z|`, `x, y ≥ 0`.
    Power {
        /// Exponent in (0, 1).
        alpha: f64,
        /// Members.
        members: [Linear; 3],
    },
    /// `members[0] ≥ ‖members[1..]‖₂`.
    SecondOrder(Vec<Linear>),
}
/// The side of an original row an epigraph row states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// `form = bound`.
    Equal,
    /// `form ≤ bound`.
    Upper,
    /// `form ≥ bound`.
    Lower,
}
/// One side of an original row in the epigraph form.
#[derive(Clone, Debug, PartialEq)]
pub struct EpigraphRow {
    /// The program row.
    pub row: usize,
    /// The side stated.
    pub side: Side,
    /// The row's canonical affine form over the epigraph coordinates.
    pub form: Linear,
    /// The row's bound on that side.
    pub bound: f64,
}
/// A recognized program in epigraph form: minimize `objective` over the epigraph
/// coordinates subject to `rows`, `cones` and the program's variable box. At every
/// feasible point of the program there are auxiliary values meeting every constraint with
/// the objective equal to the program's, so the forms are equivalent.
#[derive(Clone, Debug, PartialEq)]
pub struct Epigraph {
    /// Program columns.
    pub columns: usize,
    /// The node each auxiliary column bounds, in creation order: with every auxiliary at
    /// its node's value, every cone membership holds and every form equals its function.
    pub auxiliaries: Vec<NodeId>,
    /// The minimization objective (the authored objective times its sense), when declared.
    pub objective: Option<Linear>,
    /// The rows, in program row order, equality sides first within a row.
    pub rows: Vec<EpigraphRow>,
    /// The atoms' cone memberships.
    pub cones: Vec<ConeConstraint>,
}
impl Epigraph {
    /// The cones this form lowers to.
    pub fn summary(&self) -> ConeSummary {
        let mut s = ConeSummary {
            auxiliaries: self.auxiliaries.len(),
            ..ConeSummary::default()
        };
        for cone in &self.cones {
            match cone {
                ConeConstraint::Nonnegative(_) => s.nonnegative += 1,
                ConeConstraint::Exponential(_) => s.exponential += 1,
                ConeConstraint::Power { .. } => s.power += 1,
                ConeConstraint::SecondOrder(_) => s.second_order += 1,
            }
        }
        s
    }
}
/// What the pass established about one program.
#[derive(Clone, Debug)]
pub enum Recognition {
    /// Every row and the objective are convex in their required directions, with this
    /// epigraph form.
    Cone(Epigraph),
    /// Not recognized, with the first reason.
    Unrecognized(Unrecognized),
    /// The exact allowance was exhausted before recognition.
    Inconclusive,
}
/// Recognize `program` within `limit` exact multiply-adds.
///
/// # Errors
/// Cancellation.
pub fn recognize(
    program: &FactorableProgram,
    limit: usize,
    cancel: &AtomicBool,
) -> Result<Recognition, MathError> {
    Pass::new(program, limit, cancel).run()
}
/// The curvature the pass proves for one node of `program` over its variable box, by the
/// same rules recognition uses.
///
/// # Errors
/// A node outside the program, or cancellation.
pub fn node_curvature(
    program: &FactorableProgram,
    node: NodeId,
    limit: usize,
    cancel: &AtomicBool,
) -> Result<Curvature, MathError> {
    Ok(Pass::new(program, limit, cancel).info(node)?.curvature)
}
/// The convexity fact of a program that is not a coefficient program: its recognition,
/// identified by the program (which carries its consumed values) and the pass version.
///
/// # Errors
/// Cancellation.
pub fn fact(
    program: &FactorableProgram,
    limit: usize,
    cancel: &AtomicBool,
) -> Result<Convexity, MathError> {
    let class = match recognize(program, limit, cancel)? {
        Recognition::Cone(form) => ConvexityClass::Cone(form.summary()),
        Recognition::Unrecognized(reason) => ConvexityClass::Unrecognized(reason),
        Recognition::Inconclusive => ConvexityClass::Inconclusive,
    };
    let mut h = FramedHasher::new(pse_ids::Frame::MathConvexityFactV1);
    h.hash(&program.key).str("curvature-v1").u64(limit as u64);
    class.hash_into(&mut h);
    Ok(Convexity {
        key: h.finish_hash(),
        class,
    })
}

/// How a node's curvature was proved; the epigraph construction follows it.
#[derive(Clone, Debug)]
enum Shape {
    /// A program column.
    Variable(usize),
    /// A constant value.
    Constant(f64),
    /// A sum of children.
    Sum(Vec<NodeId>),
    /// A constant multiple of one child.
    Scale(f64, NodeId),
    /// `exp(a)`.
    Exp(NodeId),
    /// `log(a)`.
    Log(NodeId),
    /// `baseᵖ`.
    Power(NodeId, f64),
    /// `|a|` of an affine `a`.
    Abs(NodeId),
    /// `factor · x·log x` over affine `x`, with a nonzero factor.
    Entropy(f64, NodeId),
    /// `factor · x·log(x/y)` over affine `x`, `y`, with a nonzero factor.
    RelativeEntropy(f64, NodeId, NodeId),
    /// `√(Σ wᵢ aᵢ² + c)` over affine `aᵢ`, `w, c ≥ 0`.
    Norm(Vec<(f64, NodeId)>, f64),
    /// A polynomial of degree at most two decided exactly.
    Quadratic(Box<Quadratic>),
    /// No proof.
    Unknown,
}
/// `sign · Σₖ wₖ (rₖ·x)² + b·x + c`, with the factors of `sign · Q`.
#[derive(Clone, Debug)]
struct Quadratic {
    sign: f64,
    factors: GramFactors,
    linear: BTreeMap<usize, Rational>,
    constant: Rational,
}
#[derive(Clone, Debug)]
struct Info {
    interval: Interval,
    curvature: Curvature,
    shape: Shape,
    value: Option<f64>,
}
/// A polynomial of degree at most two: monomial `(i, j)` with `i ≤ j`, where
/// [`usize::MAX`] marks an absent factor.
type Polynomial = BTreeMap<(usize, usize), Rational>;
const NONE: usize = usize::MAX;
/// The epigraph construction's use of a node: an upper bound of a convex node, a lower
/// bound of a concave one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Sense {
    Up,
    Down,
}
impl Sense {
    const fn flip(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}
struct Pass<'a> {
    program: &'a FactorableProgram,
    info: Vec<Option<Info>>,
    polynomials: HashMap<NodeId, Option<Polynomial>>,
    remaining: usize,
    exhausted: bool,
    cancel: &'a AtomicBool,
    // Epigraph construction.
    auxiliaries: Vec<NodeId>,
    cones: Vec<ConeConstraint>,
    forms: HashMap<(NodeId, Option<Sense>), Linear>,
}
/// A recognition failure inside the pass: no recognized form (an exhausted allowance is
/// recorded on the pass), or a mathematical error.
enum Failure {
    No,
    Math(MathError),
}
impl From<MathError> for Failure {
    fn from(e: MathError) -> Self {
        Self::Math(e)
    }
}
impl<'a> Pass<'a> {
    fn new(program: &'a FactorableProgram, limit: usize, cancel: &'a AtomicBool) -> Self {
        Self {
            program,
            info: vec![None; program.nodes.len()],
            polynomials: HashMap::new(),
            remaining: limit,
            exhausted: false,
            cancel,
            auxiliaries: Vec::new(),
            cones: Vec::new(),
            forms: HashMap::new(),
        }
    }
    fn run(mut self) -> Result<Recognition, MathError> {
        let p = self.program;
        if p.variables
            .iter()
            .any(|v| v.domain != ModelingVariableDomain::Continuous)
        {
            return Ok(Recognition::Unrecognized(Unrecognized::Discrete));
        }
        if !p.auxiliaries.is_empty()
            || !p.implicit.is_empty()
            || !p.native.is_empty()
            || p.rows
                .iter()
                .any(|r| r.fidelity != Fidelity::Exact || r.expression.is_none())
            || p.objective
                .as_ref()
                .is_some_and(|o| o.fidelity != Fidelity::Exact || o.expression.is_none())
        {
            return Ok(Recognition::Unrecognized(Unrecognized::Inexact));
        }
        let unrecognized = |this: &Self, reason| {
            Ok(if this.exhausted {
                Recognition::Inconclusive
            } else {
                Recognition::Unrecognized(reason)
            })
        };
        let mut rows = Vec::new();
        for (index, row) in p.rows.iter().enumerate() {
            let Some(root) = row.expression else {
                return Ok(Recognition::Unrecognized(Unrecognized::Inexact));
            };
            match self.row(index, root, row.lower, row.upper) {
                Ok(sides) => rows.extend(sides),
                Err(Failure::No) => {
                    return unrecognized(&self, Unrecognized::Curvature { row: Some(index) });
                }
                Err(Failure::Math(e)) => return Err(e),
            }
        }
        let objective = match &p.objective {
            None => None,
            Some(o) => {
                let root = o.expression.ok_or_else(|| {
                    MathError::Contract("exact objective without an expression".into())
                })?;
                let sense = match o.sense {
                    ObjectiveSense::Minimize => Sense::Up,
                    ObjectiveSense::Maximize => Sense::Down,
                };
                match self.bounded(root, sense) {
                    Ok(form) => Some(form.scaled(o.sense.sign())),
                    Err(Failure::No) => {
                        return unrecognized(&self, Unrecognized::Curvature { row: None });
                    }
                    Err(Failure::Math(e)) => return Err(e),
                }
            }
        };
        // Rows first: most programs fail there, before any obligation is enclosed.
        if !self.domains()? {
            return Ok(Recognition::Unrecognized(Unrecognized::Domain));
        }
        if self.cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        Ok(Recognition::Cone(Epigraph {
            columns: p.variables.len(),
            auxiliaries: self.auxiliaries,
            objective,
            rows,
            cones: self.cones,
        }))
    }
    /// Every retained obligation, conditional or not, holds throughout the box: a
    /// requirement through its exact condition on its argument, a domain predicate through
    /// its complete closed conjunction.
    fn domains(&mut self) -> Result<bool, MathError> {
        for o in &self.program.obligations {
            match (o.kind, o.argument) {
                (ObligationKind::Require(condition), Some(argument)) => {
                    let range = self.info(argument)?.interval;
                    let holds = !range.is_empty()
                        && match condition {
                            Condition::Positive => range.lo > 0.0,
                            Condition::Nonnegative => range.lo >= 0.0,
                            Condition::Nonzero => range.lo > 0.0 || range.hi < 0.0,
                        };
                    if !holds {
                        return Ok(false);
                    }
                }
                _ => {
                    if !o.represented {
                        return Ok(false);
                    }
                    for c in &o.constraints {
                        let range = self.info(c.expression)?.interval;
                        let holds = !range.is_empty()
                            && if c.strict {
                                range.lo > c.lower && range.hi < c.upper
                            } else {
                                range.lo >= c.lower && range.hi <= c.upper
                            };
                        if !holds {
                            return Ok(false);
                        }
                    }
                }
            }
        }
        Ok(true)
    }
    /// The epigraph rows of one program row: an equality is affine, a finite upper bound
    /// needs a convex row and a finite lower bound a concave one.
    fn row(
        &mut self,
        index: usize,
        root: NodeId,
        lower: f64,
        upper: f64,
    ) -> Result<Vec<EpigraphRow>, Failure> {
        let mut sides = Vec::new();
        if lower == upper {
            let form = self.exact(root)?;
            sides.push(EpigraphRow {
                row: index,
                side: Side::Equal,
                form,
                bound: lower,
            });
            return Ok(sides);
        }
        if upper.is_finite() {
            let form = self.bounded(root, Sense::Up)?;
            sides.push(EpigraphRow {
                row: index,
                side: Side::Upper,
                form,
                bound: upper,
            });
        }
        if lower.is_finite() {
            let form = self.bounded(root, Sense::Down)?;
            sides.push(EpigraphRow {
                row: index,
                side: Side::Lower,
                form,
                bound: lower,
            });
        }
        Ok(sides)
    }

    // ---- Analysis: intervals, curvature and shapes, bottom-up in node order. ----

    fn info(&mut self, root: NodeId) -> Result<&Info, MathError> {
        if root >= self.info.len() {
            return Err(MathError::Contract("curvature node out of range".into()));
        }
        if self.info[root].is_none() {
            let mut pending = vec![root];
            let mut order = Vec::new();
            let mut seen = std::collections::HashSet::new();
            while let Some(n) = pending.pop() {
                if self.info[n].is_some() || !seen.insert(n) {
                    continue;
                }
                order.push(n);
                pending.extend(children(&self.program.nodes[n]).iter().copied());
            }
            // Node identities are topological: children precede their parents.
            order.sort_unstable();
            for n in order {
                if self.cancel.load(Ordering::Relaxed) {
                    return Err(MathError::Cancelled);
                }
                let info = self.classify(n);
                self.info[n] = Some(info);
            }
        }
        self.info[root]
            .as_ref()
            .ok_or_else(|| MathError::Contract("curvature node order".into()))
    }
    /// A classified node. Children precede their parents, so every child is classified;
    /// a node that were not would read as unknown over the entire line, which certifies
    /// nothing.
    fn get(&self, n: NodeId) -> &Info {
        const UNCLASSIFIED: Info = Info {
            interval: Interval::ENTIRE,
            curvature: Curvature::Unknown,
            shape: Shape::Unknown,
            value: None,
        };
        self.info[n].as_ref().unwrap_or(&UNCLASSIFIED)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive dispatch of the DCP composition rules over the node vocabulary"
    )]
    fn classify(&mut self, n: NodeId) -> Info {
        let node = &self.program.nodes[n];
        let constant = |interval: Interval, value: f64| Info {
            interval,
            curvature: Curvature::Constant,
            shape: Shape::Constant(value),
            value: Some(value),
        };
        let unknown = |interval| Info {
            interval,
            curvature: Curvature::Unknown,
            shape: Shape::Unknown,
            value: None,
        };
        // A node whose children are all constant is a constant.
        let kids = children(node);
        if !matches!(node, Node::Var(_) | Node::Aux(_) | Node::Const(_))
            && kids.iter().all(|c| self.get(*c).value.is_some())
        {
            let interval = self.interval(n);
            return match self.constant_value(n) {
                Some(v) if v.is_finite() => constant(interval, v),
                _ => unknown(interval),
            };
        }
        let interval = self.interval(n);
        let info = match node {
            Node::Var(c) => Info {
                interval,
                curvature: Curvature::Affine,
                shape: Shape::Variable(*c),
                value: None,
            },
            Node::Aux(_) => unknown(interval),
            Node::Const(c) => constant(interval, c.value()),
            Node::Sum(children) => {
                let curvature = children.iter().fold(Curvature::Constant, |acc, c| {
                    acc.add(self.get(*c).curvature)
                });
                Info {
                    interval,
                    curvature,
                    shape: Shape::Sum(children.clone()),
                    value: None,
                }
            }
            Node::Product(children) => {
                let (constants, factors): (Vec<NodeId>, Vec<NodeId>) =
                    children.iter().partition(|c| self.get(**c).value.is_some());
                let factor: f64 = constants
                    .iter()
                    .map(|c| self.get(*c).value.unwrap_or(f64::NAN))
                    .product();
                let range = constants.iter().fold(Interval::point(1.0), |acc, c| {
                    acc.mul(self.get(*c).interval)
                });
                // The sign of the constant factor must be certain.
                let sign = if range.lo > 0.0 {
                    1.0
                } else if range.hi < 0.0 {
                    -1.0
                } else {
                    0.0
                };
                if sign == 0.0 || !factor.is_finite() || factor == 0.0 {
                    unknown(interval)
                } else if let [child] = factors.as_slice() {
                    let c = self.get(*child).curvature;
                    Info {
                        interval,
                        curvature: if sign > 0.0 { c } else { c.negate() },
                        shape: Shape::Scale(factor, *child),
                        value: None,
                    }
                } else if let [a, b] = factors.as_slice() {
                    match self.entropy(*a, *b).or_else(|| self.entropy(*b, *a)) {
                        Some(shape) => {
                            let shape = match shape {
                                Shape::Entropy(_, x) => Shape::Entropy(factor, x),
                                Shape::RelativeEntropy(_, x, y) => {
                                    Shape::RelativeEntropy(factor, x, y)
                                }
                                other => other,
                            };
                            Info {
                                interval,
                                curvature: if sign > 0.0 {
                                    Curvature::Convex
                                } else {
                                    Curvature::Concave
                                },
                                shape,
                                value: None,
                            }
                        }
                        None => unknown(interval),
                    }
                } else {
                    unknown(interval)
                }
            }
            Node::Pow { base, exponent } => self.power(*base, exponent, interval),
            Node::Exp(a) => {
                if self.get(*a).curvature.convex() {
                    Info {
                        interval,
                        curvature: Curvature::Convex,
                        shape: Shape::Exp(*a),
                        value: None,
                    }
                } else {
                    unknown(interval)
                }
            }
            Node::Log(a) => {
                let arg = self.get(*a);
                if arg.interval.lo > 0.0 && arg.curvature.concave() {
                    Info {
                        interval,
                        curvature: Curvature::Concave,
                        shape: Shape::Log(*a),
                        value: None,
                    }
                } else {
                    unknown(interval)
                }
            }
            Node::Abs(a) => {
                let arg = self.get(*a);
                // On a signed argument the absolute value is the argument or its negation.
                if arg.interval.lo >= 0.0 {
                    Info {
                        interval,
                        curvature: arg.curvature,
                        shape: Shape::Scale(1.0, *a),
                        value: None,
                    }
                } else if arg.interval.hi <= 0.0 {
                    Info {
                        interval,
                        curvature: arg.curvature.negate(),
                        shape: Shape::Scale(-1.0, *a),
                        value: None,
                    }
                } else if arg.curvature == Curvature::Affine {
                    Info {
                        interval,
                        curvature: Curvature::Convex,
                        shape: Shape::Abs(*a),
                        value: None,
                    }
                } else {
                    unknown(interval)
                }
            }
            Node::Sin(_) | Node::Cos(_) => unknown(interval),
        };
        // A polynomial of degree two that DCP cannot classify is decided exactly.
        if info.curvature == Curvature::Unknown
            && matches!(node, Node::Sum(_) | Node::Product(_) | Node::Pow { .. })
            && let Some(quadratic) = self.quadratic(n)
        {
            return Info {
                interval,
                curvature: quadratic.0,
                shape: quadratic.1,
                value: None,
            };
        }
        info
    }
    /// `x·log x` or `x·log(x/y)` (also `x·(log x − log y)`) over affine `x`, `y`, with
    /// `x ≥ 0` and `y > 0` on the box.
    fn entropy(&self, x: NodeId, log: NodeId) -> Option<Shape> {
        let nodes = &self.program.nodes;
        let affine = |n: NodeId| self.get(n).curvature == Curvature::Affine;
        let positive = |n: NodeId| self.get(n).interval.lo > 0.0;
        if !affine(x) || self.get(x).interval.lo < 0.0 {
            return None;
        }
        let Node::Log(argument) = &nodes[log] else {
            return None;
        };
        if *argument == x {
            return Some(Shape::Entropy(1.0, x));
        }
        // log(x · y⁻¹)
        if let Node::Product(members) = &nodes[*argument]
            && let [a, b] = members.as_slice()
        {
            for (numerator, reciprocal) in [(*a, *b), (*b, *a)] {
                if numerator == x
                    && let Node::Pow { base: y, exponent } = &nodes[reciprocal]
                    && exponent.value() == -1.0
                    && affine(*y)
                    && positive(*y)
                {
                    return Some(Shape::RelativeEntropy(1.0, x, *y));
                }
            }
        }
        None
    }
    fn power(&self, base: NodeId, exponent: &Constant, interval: Interval) -> Info {
        let p = exponent.value();
        let b = self.get(base);
        let result = |curvature, shape| Info {
            interval,
            curvature,
            shape,
            value: None,
        };
        let unknown = || result(Curvature::Unknown, Shape::Unknown);
        if !p.is_finite() {
            return unknown();
        }
        if p == 1.0 {
            return result(b.curvature, Shape::Scale(1.0, base));
        }
        if p == 0.5
            && let Some(norm) = self.norm(base)
        {
            return result(Curvature::Convex, norm);
        }
        let even = p >= 2.0 && p.fract() == 0.0 && (p / 2.0).fract() == 0.0;
        let curvature = if even {
            match b.curvature {
                Curvature::Affine => Curvature::Convex,
                Curvature::Convex if b.interval.lo >= 0.0 => Curvature::Convex,
                Curvature::Concave if b.interval.hi <= 0.0 => Curvature::Convex,
                _ => Curvature::Unknown,
            }
        } else if p > 1.0 {
            if b.interval.lo >= 0.0 && b.curvature.convex() {
                Curvature::Convex
            } else {
                Curvature::Unknown
            }
        } else if p > 0.0 {
            if b.interval.lo >= 0.0 && b.curvature.concave() {
                Curvature::Concave
            } else {
                Curvature::Unknown
            }
        } else if p < 0.0 {
            if b.interval.lo > 0.0 && b.curvature.concave() {
                Curvature::Convex
            } else {
                Curvature::Unknown
            }
        } else {
            Curvature::Unknown
        };
        if curvature == Curvature::Unknown {
            unknown()
        } else {
            result(curvature, Shape::Power(base, p))
        }
    }
    /// `Σ wᵢ aᵢ² + c` with affine `aᵢ` and `w, c ≥ 0`, as the members of a norm.
    fn norm(&self, sum: NodeId) -> Option<Shape> {
        let nodes = &self.program.nodes;
        let square = |n: NodeId| -> Option<NodeId> {
            match &nodes[n] {
                Node::Pow { base, exponent }
                    if exponent.value() == 2.0
                        && self.get(*base).curvature == Curvature::Affine =>
                {
                    Some(*base)
                }
                _ => None,
            }
        };
        let term = |n: NodeId| -> Option<(f64, NodeId)> {
            if let Some(a) = square(n) {
                return Some((1.0, a));
            }
            let Node::Product(members) = &nodes[n] else {
                return None;
            };
            let [x, y] = members.as_slice() else {
                return None;
            };
            for (k, s) in [(*x, *y), (*y, *x)] {
                let k = self.get(k);
                if let (Some(w), Some(a)) = (k.value, square(s))
                    && k.interval.lo >= 0.0
                    && w >= 0.0
                {
                    return Some((w, a));
                }
            }
            None
        };
        let members: Vec<NodeId> = match &nodes[sum] {
            Node::Sum(members) => members.clone(),
            _ => vec![sum],
        };
        let mut terms = Vec::new();
        let mut constant = 0.0;
        for m in members {
            let info = self.get(m);
            if let Some(v) = info.value {
                if info.interval.lo < 0.0 || v < 0.0 {
                    return None;
                }
                constant += v;
            } else {
                terms.push(term(m)?);
            }
        }
        (!terms.is_empty()).then_some(Shape::Norm(terms, constant))
    }
    /// The outward-rounded enclosure of one node over the box, from its children's.
    fn interval(&self, n: NodeId) -> Interval {
        let get = |c: &NodeId| self.get(*c).interval;
        match &self.program.nodes[n] {
            Node::Var(c) => self
                .program
                .variables
                .get(*c)
                .map_or(Interval::ENTIRE, |v| Interval::new(v.lower, v.upper)),
            Node::Aux(k) => self
                .program
                .auxiliaries
                .get(*k)
                .map_or(Interval::ENTIRE, |a| Interval::new(a.lower, a.upper)),
            Node::Const(c) => constant_interval(c),
            Node::Sum(children) => children
                .iter()
                .fold(Interval::point(0.0), |acc, c| acc.add(get(c))),
            Node::Product(children) => children
                .iter()
                .fold(Interval::point(1.0), |acc, c| acc.mul(get(c))),
            Node::Pow { base, exponent } => power_interval(get(base), exponent),
            Node::Exp(a) => get(a).exp(),
            Node::Log(a) => get(a).ln(),
            Node::Abs(a) => get(a).abs(),
            Node::Sin(a) => get(a).sin(),
            Node::Cos(a) => get(a).cos(),
        }
    }
    /// The binary64 value of a node whose children are all constant, as the evaluator
    /// computes it.
    fn constant_value(&self, n: NodeId) -> Option<f64> {
        let value = |c: &NodeId| self.get(*c).value;
        Some(match &self.program.nodes[n] {
            Node::Var(_) | Node::Aux(_) => return None,
            Node::Const(c) => c.value(),
            Node::Sum(children) => children.iter().map(value).sum::<Option<f64>>()?,
            Node::Product(children) => children.iter().map(value).product::<Option<f64>>()?,
            Node::Pow { base, exponent } => crate::factorable::power(value(base)?, exponent),
            Node::Exp(a) => value(a)?.exp(),
            Node::Log(a) => value(a)?.ln(),
            Node::Abs(a) => value(a)?.abs(),
            Node::Sin(a) => value(a)?.sin(),
            Node::Cos(a) => value(a)?.cos(),
        })
    }

    // ---- Exact quadratic forms. ----

    /// The exact curvature of a node that is a polynomial of degree at most two: affine,
    /// or convex or concave with the Gram factors of `±Q`.
    fn quadratic(&mut self, n: NodeId) -> Option<(Curvature, Shape)> {
        let polynomial = self.polynomial(n)?;
        let mut rows: BTreeMap<usize, BTreeMap<usize, Rational>> = BTreeMap::new();
        let mut linear = BTreeMap::new();
        let mut constant = Rational::zero();
        for ((i, j), v) in &polynomial {
            match (*i, *j) {
                (NONE, NONE) => constant = v.clone(),
                (i, NONE) => {
                    linear.insert(i, v.clone());
                }
                (i, j) if i == j => {
                    rows.entry(i).or_default().insert(i, v.clone());
                }
                (i, j) => {
                    let half = v / &Rational::from(2);
                    rows.entry(i).or_default().insert(j, half.clone());
                    rows.entry(j).or_default().insert(i, half);
                }
            }
        }
        if rows.is_empty() {
            return Some((
                Curvature::Affine,
                Shape::Quadratic(Box::new(Quadratic {
                    sign: 1.0,
                    factors: GramFactors::default(),
                    linear,
                    constant,
                })),
            ));
        }
        // Dense local indices over the columns the quadratic form touches.
        let columns: Vec<usize> = rows.keys().copied().collect();
        let local: BTreeMap<usize, usize> =
            columns.iter().enumerate().map(|(k, c)| (*c, k)).collect();
        for sign in [1.0, -1.0] {
            let matrix: Vec<BTreeMap<usize, Rational>> = columns
                .iter()
                .map(|c| {
                    rows[c]
                        .iter()
                        .map(|(j, v)| (local[j], if sign > 0.0 { v.clone() } else { -v.clone() }))
                        .collect()
                })
                .collect();
            match ldlt(matrix, &mut self.remaining, self.cancel) {
                Ok(Ldlt::Psd(factors)) => {
                    let factors = factors.relabel(&columns);
                    return Some((
                        if sign > 0.0 {
                            Curvature::Convex
                        } else {
                            Curvature::Concave
                        },
                        Shape::Quadratic(Box::new(Quadratic {
                            sign,
                            factors,
                            linear,
                            constant,
                        })),
                    ));
                }
                Ok(Ldlt::Indefinite) => {}
                Ok(Ldlt::Exhausted) | Err(_) => {
                    self.exhausted = true;
                    return None;
                }
            }
        }
        None
    }
    /// The exact polynomial of a node of degree at most two, when it is one: columns,
    /// exact constants, sums, products and squares. A transcendental constant is not
    /// exact, so it is not a polynomial coefficient.
    fn polynomial(&mut self, root: NodeId) -> Option<Polynomial> {
        if let Some(known) = self.polynomials.get(&root) {
            return known.clone();
        }
        let mut pending = vec![root];
        let mut order = Vec::new();
        let mut seen = std::collections::HashSet::new();
        while let Some(n) = pending.pop() {
            if self.polynomials.contains_key(&n) || !seen.insert(n) {
                continue;
            }
            order.push(n);
            pending.extend(children(&self.program.nodes[n]).iter().copied());
        }
        order.sort_unstable();
        for n in order {
            let result = self.expand(n);
            self.polynomials.insert(n, result);
            if self.exhausted {
                break;
            }
        }
        self.polynomials.get(&root).cloned().flatten()
    }
    fn expand(&mut self, n: NodeId) -> Option<Polynomial> {
        let program = self.program;
        let known = &self.polynomials;
        let child = |c: &NodeId| known.get(c).cloned().flatten();
        match &program.nodes[n] {
            Node::Var(c) => Some(Polynomial::from([((*c, NONE), Rational::one())])),
            Node::Const(c) => Some(Polynomial::from([((NONE, NONE), exact(c)?)])),
            Node::Sum(members) => {
                let mut sum = Polynomial::new();
                for m in members {
                    for (k, v) in child(m)? {
                        let entry = sum.entry(k).or_insert_with(Rational::zero);
                        *entry += &v;
                    }
                }
                sum.retain(|_, v| !v.is_zero());
                Some(sum)
            }
            Node::Product(members) => {
                let factors: Vec<Polynomial> = members.iter().map(child).collect::<Option<_>>()?;
                let mut product = Polynomial::from([((NONE, NONE), Rational::one())]);
                for factor in &factors {
                    product = self.multiply(&product, factor)?;
                }
                Some(product)
            }
            Node::Pow { base, exponent } => {
                let square = match exponent {
                    Constant::Rational(r) if r.is_zero() => {
                        return Some(Polynomial::from([((NONE, NONE), Rational::one())]));
                    }
                    Constant::Rational(r) if r.is_one() => {
                        return child(base);
                    }
                    Constant::Rational(r) => r == &Rational::from(2),
                    Constant::Float(v) => *v == 2.0,
                };
                if !square {
                    return None;
                }
                let b = child(base)?;
                self.multiply(&b, &b)
            }
            _ => None,
        }
    }
    fn multiply(&mut self, a: &Polynomial, b: &Polynomial) -> Option<Polynomial> {
        let cost = a.len().saturating_mul(b.len());
        if cost > self.remaining {
            self.exhausted = true;
            return None;
        }
        self.remaining -= cost;
        let mut out = Polynomial::new();
        for ((i, j), u) in a {
            for ((k, l), v) in b {
                let mut factors: Vec<usize> = [*i, *j, *k, *l]
                    .into_iter()
                    .filter(|f| *f != NONE)
                    .collect();
                if factors.len() > 2 {
                    return None;
                }
                factors.sort_unstable();
                let key = match factors.as_slice() {
                    [] => (NONE, NONE),
                    [a] => (*a, NONE),
                    [a, b] => (*a, *b),
                    _ => return None,
                };
                let entry = out.entry(key).or_insert_with(Rational::zero);
                *entry += &(u * v);
            }
        }
        out.retain(|_, v| !v.is_zero());
        Some(out)
    }

    // ---- Epigraph construction. ----

    /// The exact affine form of an affine or constant node.
    fn exact(&mut self, n: NodeId) -> Result<Linear, Failure> {
        let curvature = self.info(n)?.curvature;
        if !matches!(curvature, Curvature::Affine | Curvature::Constant) {
            return Err(Failure::No);
        }
        self.form(n, None)
    }
    /// An affine form bounding `n` from above (`Up`, for a convex node) or below (`Down`,
    /// for a concave one), equal to `n` for some auxiliary values.
    fn bounded(&mut self, n: NodeId, sense: Sense) -> Result<Linear, Failure> {
        let curvature = self.info(n)?.curvature;
        match (curvature, sense) {
            (Curvature::Affine | Curvature::Constant, _) => self.form(n, None),
            (Curvature::Convex, Sense::Up) | (Curvature::Concave, Sense::Down) => {
                self.form(n, Some(sense))
            }
            _ => Err(Failure::No),
        }
    }
    fn auxiliary(&mut self, n: NodeId) -> (usize, Linear) {
        let index = self.program.variables.len() + self.auxiliaries.len();
        self.auxiliaries.push(n);
        (index, Linear::coordinate(index))
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive dispatch of the epigraph atoms"
    )]
    fn form(&mut self, n: NodeId, sense: Option<Sense>) -> Result<Linear, Failure> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(Failure::Math(MathError::Cancelled));
        }
        if let Some(known) = self.forms.get(&(n, sense)) {
            return Ok(known.clone());
        }
        let shape = self.get(n).shape.clone();
        let form = match shape {
            Shape::Variable(c) => Linear::coordinate(c),
            Shape::Constant(v) => Linear::constant(v),
            Shape::Sum(children) => {
                let mut sum = Linear::default();
                for c in children {
                    let term = match sense {
                        None => self.exact(c)?,
                        Some(s) => self.bounded(c, s)?,
                    };
                    sum = sum.plus(&term, 1.0);
                }
                sum
            }
            Shape::Scale(factor, child) => {
                let term = match sense {
                    None => self.exact(child)?,
                    Some(s) => self.bounded(child, if factor >= 0.0 { s } else { s.flip() })?,
                };
                term.scaled(factor)
            }
            Shape::Exp(a) => {
                // exp(A) ≤ t: (A, 1, t) ∈ K_exp.
                let arg = self.bounded(a, Sense::Up)?;
                let (_, t) = self.auxiliary(n);
                self.cones.push(ConeConstraint::Exponential([
                    arg,
                    Linear::constant(1.0),
                    t.clone(),
                ]));
                t
            }
            Shape::Log(a) => {
                // t ≤ log(A): (t, 1, A) ∈ K_exp.
                let arg = self.bounded(a, Sense::Down)?;
                let (_, t) = self.auxiliary(n);
                self.cones.push(ConeConstraint::Exponential([
                    t.clone(),
                    Linear::constant(1.0),
                    arg,
                ]));
                t
            }
            Shape::Entropy(factor, x) => {
                // x·log x ≤ t/f, which bounds the node f·x·log x above by t for f > 0 and
                // below for f < 0: (−t/f, x, 1) ∈ K_exp.
                let x = self.exact(x)?;
                let (_, t) = self.auxiliary(n);
                self.cones.push(ConeConstraint::Exponential([
                    t.scaled(-1.0 / factor),
                    x,
                    Linear::constant(1.0),
                ]));
                t
            }
            Shape::RelativeEntropy(factor, x, y) => {
                // x·log(x/y) ≤ t/f: (−t/f, x, y) ∈ K_exp.
                let x = self.exact(x)?;
                let y = self.exact(y)?;
                let (_, t) = self.auxiliary(n);
                self.cones
                    .push(ConeConstraint::Exponential([t.scaled(-1.0 / factor), x, y]));
                t
            }
            Shape::Power(base, p) => {
                let curvature = self.get(n).curvature;
                let b = self.get(base).clone();
                let (_, t) = self.auxiliary(n);
                if curvature == Curvature::Convex && p > 0.0 {
                    // |B| ≤ t^(1/p): (t, 1, B) ∈ K_pow(1/p), with B bounding the base
                    // toward larger magnitude.
                    let bound = match b.curvature {
                        Curvature::Affine | Curvature::Constant => self.exact(base)?,
                        Curvature::Convex => self.bounded(base, Sense::Up)?,
                        Curvature::Concave => self.bounded(base, Sense::Down)?,
                        Curvature::Unknown => return Err(Failure::No),
                    };
                    self.cones.push(ConeConstraint::Power {
                        alpha: 1.0 / p,
                        members: [t.clone(), Linear::constant(1.0), bound],
                    });
                } else if curvature == Curvature::Concave {
                    // t ≤ B^p with 0 < p < 1: (B, 1, t) ∈ K_pow(p).
                    let bound = self.bounded(base, Sense::Down)?;
                    self.cones.push(ConeConstraint::Power {
                        alpha: p,
                        members: [bound, Linear::constant(1.0), t.clone()],
                    });
                } else if curvature == Curvature::Convex && p < 0.0 {
                    // B^p ≤ t: (t, B, 1) ∈ K_pow(1/(1−p)), with B a lower bound of the base.
                    let bound = self.bounded(base, Sense::Down)?;
                    self.cones.push(ConeConstraint::Power {
                        alpha: 1.0 / (1.0 - p),
                        members: [t.clone(), bound, Linear::constant(1.0)],
                    });
                } else {
                    return Err(Failure::No);
                }
                t
            }
            Shape::Abs(a) => {
                // |A| ≤ t: t − A ≥ 0 and t + A ≥ 0.
                let a = self.exact(a)?;
                let (_, t) = self.auxiliary(n);
                self.cones
                    .push(ConeConstraint::Nonnegative(t.clone().plus(&a, -1.0)));
                self.cones
                    .push(ConeConstraint::Nonnegative(t.clone().plus(&a, 1.0)));
                t
            }
            Shape::Norm(terms, constant) => {
                // ‖(√wᵢ Aᵢ, √c)‖ ≤ t.
                let (_, t) = self.auxiliary(n);
                let mut members = vec![t.clone()];
                for (w, a) in terms {
                    members.push(self.exact(a)?.scaled(w.sqrt()));
                }
                if constant > 0.0 {
                    members.push(Linear::constant(constant.sqrt()));
                }
                self.cones.push(ConeConstraint::SecondOrder(members));
                t
            }
            Shape::Quadratic(q) => {
                let mut linear = Linear::constant(q.constant.to_f64());
                for (c, v) in &q.linear {
                    linear.terms.insert(*c, v.to_f64());
                }
                linear.terms.retain(|_, v| *v != 0.0);
                if q.factors.rank() == 0 {
                    linear
                } else {
                    // sign·Σ wₖ yₖ² + L ≤ t (convex) or ≥ t (concave), i.e.
                    // Σ wₖ yₖ² ≤ u = sign·(t − L), as the rotated cone
                    // ‖(2√wₖ yₖ, 1 − u)‖ ≤ 1 + u.
                    let (_, t) = self.auxiliary(n);
                    let u = t.clone().plus(&linear, -1.0).scaled(q.sign);
                    let mut members = vec![
                        u.clone().plus(&Linear::constant(1.0), 1.0),
                        Linear::constant(1.0).plus(&u, -1.0),
                    ];
                    for (w, row) in q.factors.terms() {
                        let scale = 2.0 * w.to_f64().sqrt();
                        let mut y = Linear::default();
                        for (c, v) in row {
                            y.terms.insert(*c, scale * v.to_f64());
                        }
                        y.terms.retain(|_, v| *v != 0.0);
                        members.push(y);
                    }
                    self.cones.push(ConeConstraint::SecondOrder(members));
                    t
                }
            }
            Shape::Unknown => return Err(Failure::No),
        };
        if !form.finite() {
            return Err(Failure::No);
        }
        self.forms.insert((n, sense), form.clone());
        Ok(form)
    }
}
fn children(node: &Node) -> &[NodeId] {
    match node {
        Node::Sum(c) | Node::Product(c) => c,
        Node::Pow { base, .. } => std::slice::from_ref(base),
        Node::Exp(i) | Node::Log(i) | Node::Abs(i) | Node::Sin(i) | Node::Cos(i) => {
            std::slice::from_ref(i)
        }
        Node::Var(_) | Node::Aux(_) | Node::Const(_) => &[],
    }
}
/// The exact rational of a constant: a library rational, or a binary64 value, which is a
/// dyadic rational.
fn exact(c: &Constant) -> Option<Rational> {
    match c {
        Constant::Rational(r) => Some(r.clone()),
        Constant::Float(v) => Rational::try_from(*v).ok(),
    }
}
/// A constant's enclosure: its binary64 value, widened by one unit in the last place when
/// a rational is not exactly that value.
fn constant_interval(c: &Constant) -> Interval {
    let v = c.value();
    match c {
        Constant::Float(_) => Interval::point(v),
        Constant::Rational(_) => {
            if exact(c).is_some_and(|q| Rational::try_from(v).is_ok_and(|w| w == q)) {
                Interval::point(v)
            } else {
                Interval::new(v.next_down(), v.next_up())
            }
        }
    }
}
/// `baseᵖ` over an enclosure: integer powers through the library's operations, a real
/// power of a nonnegative base as `exp(p·log base)`.
fn power_interval(base: Interval, exponent: &Constant) -> Interval {
    let p = exponent.value();
    if !p.is_finite() || base.is_empty() {
        return Interval::ENTIRE;
    }
    if p.fract() == 0.0 && p.abs() <= 64.0 {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the branch proved an integer absolute exponent at most 64"
        )]
        let n = p.abs() as u32;
        let magnitude = base.pow_uint(n);
        return if p < 0.0 {
            Interval::point(1.0).div(magnitude)
        } else {
            magnitude
        };
    }
    if p == 0.5 {
        return base.sqrt();
    }
    if base.lo < 0.0 {
        return Interval::ENTIRE;
    }
    let result = Interval::point(p).mul(base.ln()).exp();
    // A zero base with a positive exponent reaches zero.
    if base.lo == 0.0 && p > 0.0 {
        Interval::new(0.0, result.hi)
    } else {
        result
    }
}
