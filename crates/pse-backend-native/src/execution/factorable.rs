// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The factorable representation and its runner (ADR-0105). A global backend receives the
//! neutral [`FactorableProgram`] over original case columns; the runner re-qualifies every
//! candidate against the original compiled model and, where the relaxation-soundness rule
//! requires it, takes the candidate from a continuous re-solve with the backend's discrete
//! assignment fixed, run through the one NLP runner.
//!
//! Admission and the export plan are library-neutral and testable without a backend:
//! - every exported function is a row with a projection, an unconditional obligation
//!   (strict bounds closed by [`STRICT_MARGIN`]), or an implicit residual (`== 0`) or bound;
//! - rows without a projection are dropped, which keeps the export a sound relaxation;
//! - an unavailable objective, a variable or auxiliary of a nonlinear term without a finite
//!   box, a semi domain or a nonfinite constant refuses with a typed [`Refusal`].
use super::{
    BackendExecution, BackendSettings, Input, LINKED, Nlp, OriginalModel, Problem, Retained, Step,
    nlp,
};
use crate::{
    NlpOracle, ProblemError, presolve,
    quality::{self, Observation, Quality, Violation},
    routing::{self, Requirements, Route},
    solve::{
        BoundSource, Compatibility, Controls, Metric, Options, PrimalSource, SolveIntent,
        SolveReport, SolverSelection, WarmPayload, WarmStart,
    },
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_math::{
    binding::ObjectiveSense,
    factorable::{BoundOwner, Constraint, FactorableProgram, Fidelity, MissingBound, Node, NodeId},
};
use pse_model::generated::enums::ModelingVariableDomain;
use std::collections::BTreeMap;

/// Relative margin that closes a strict obligation bound: `x > b` exports as
/// `x >= b + STRICT_MARGIN * max(1, |b|)`, and `x < b` symmetrically.
pub(crate) const STRICT_MARGIN: f64 = 1e-9;
/// Largest affine form tracked for one node; larger forms export as expressions.
const AFFINE_TERMS: usize = 4096;

/// Why a factorable program cannot be exported to a global backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The objective has no projection, or depends on an unbounded opaque auxiliary, so no
    /// bound exists (ADR-0105 §2).
    ObjectiveUnavailable,
    /// A variable or auxiliary inside a nonlinear term has no finite box; spatial
    /// branch-and-bound needs one.
    UnboundedNonlinear(MissingBound),
    /// Semicontinuous and semi-integer domains need a declared lowering (Plan 22 M3).
    SemiDomain(SemanticId),
    /// A constant or exponent outside finite representation.
    Constant(NodeId),
}
impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObjectiveUnavailable => {
                f.write_str("the objective has no factorable projection, so no bound exists")
            }
            Self::UnboundedNonlinear(m) => {
                match m.owner {
                    BoundOwner::Variable(id) => write!(f, "variable {id}")?,
                    BoundOwner::Auxiliary(k) => write!(f, "auxiliary {k}")?,
                }
                let side = match (m.lower, m.upper) {
                    (true, true) => "lower and upper bounds",
                    (true, false) => "a lower bound",
                    (false, _) => "an upper bound",
                };
                write!(
                    f,
                    " enters a nonlinear term without {side}; spatial branching needs a finite box"
                )
            }
            Self::SemiDomain(id) => write!(
                f,
                "variable {id} has a semi domain, which needs a declared lowering"
            ),
            Self::Constant(node) => write!(f, "node {node} holds a nonfinite constant"),
        }
    }
}

/// Where an exported function comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Origin {
    /// A selected row.
    Row(usize),
    /// An unconditional obligation's closed constraint.
    Obligation(usize, usize),
    /// An implicit block's residual, constrained to zero.
    Residual(usize, usize),
    /// An implicit block's declared bound.
    ImplicitBound(usize, usize),
}
/// One exported constraint `lower <= node <= upper`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Function {
    pub node: NodeId,
    pub lower: f64,
    pub upper: f64,
    pub origin: Origin,
}
/// An affine form over the combined coordinates: columns first, then auxiliaries.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Affine {
    pub terms: Vec<(usize, f64)>,
    pub constant: f64,
}
/// The library-neutral export of one admitted program.
#[derive(Debug)]
pub(crate) struct Plan<'p> {
    pub program: &'p FactorableProgram,
    /// Exported constraints, in a deterministic order.
    pub constraints: Vec<Function>,
    /// Exported objective, when the intent optimizes and one is declared.
    pub objective: Option<(NodeId, ObjectiveSense)>,
    /// Rows without a projection, dropped from the export.
    pub dropped: usize,
    /// Worst fidelity of the export, counting dropped rows as a relaxation.
    pub fidelity: Fidelity,
    /// Closed boxes of the columns and then the auxiliaries; a binary column is `[0, 1]`
    /// intersected with its declaration.
    pub boxes: Vec<(f64, f64)>,
    /// Affine form of every node, when it is affine and small enough.
    pub affine: Vec<Option<Affine>>,
    /// Identity of the declared box and domains the backend branches over.
    pub domain: ContentHash,
}
impl Plan<'_> {
    /// Some exported function or the objective is not affine.
    pub(crate) fn nonlinear(&self) -> bool {
        self.constraints
            .iter()
            .map(|c| c.node)
            .chain(self.objective.map(|o| o.0))
            .any(|n| self.affine[n].is_none())
    }
    /// A column has an integer domain.
    pub(crate) fn discrete(&self) -> bool {
        self.program.variables.iter().any(|v| v.domain.is_integer())
    }
}

fn close(c: &Constraint) -> (f64, f64) {
    let margin = |b: f64| STRICT_MARGIN * b.abs().max(1.0);
    let lower = if c.strict && c.lower.is_finite() {
        c.lower + margin(c.lower)
    } else {
        c.lower
    };
    let upper = if c.strict && c.upper.is_finite() {
        c.upper - margin(c.upper)
    } else {
        c.upper
    };
    (lower, upper)
}
fn column_box(domain: ModelingVariableDomain, lower: f64, upper: f64) -> (f64, f64) {
    if domain == ModelingVariableDomain::Binary {
        (lower.max(0.0), upper.min(1.0))
    } else {
        (lower, upper)
    }
}
/// Operands of a node, children first in the DAG.
pub(crate) fn children(node: &Node) -> &[NodeId] {
    match node {
        Node::Var(_) | Node::Aux(_) | Node::Const(_) => &[],
        Node::Sum(c) | Node::Product(c) => c,
        Node::Pow { base, .. } => std::slice::from_ref(base),
        Node::Exp(i) | Node::Log(i) | Node::Abs(i) | Node::Sin(i) | Node::Cos(i) => {
            std::slice::from_ref(i)
        }
    }
}
fn constant(form: &Affine) -> Option<f64> {
    form.terms.is_empty().then_some(form.constant)
}
fn scaled(form: &Affine, scale: f64) -> Affine {
    Affine {
        terms: form
            .terms
            .iter()
            .map(|(i, c)| (*i, c * scale))
            .filter(|(_, c)| *c != 0.0)
            .collect(),
        constant: form.constant * scale,
    }
}
/// Affine forms of every node, children first; `None` for a nonlinear node.
fn affine_forms(program: &FactorableProgram) -> Vec<Option<Affine>> {
    let columns = program.variables.len();
    let mut forms: Vec<Option<Affine>> = Vec::with_capacity(program.nodes.len());
    for node in &program.nodes {
        let unary = |i: &NodeId, f: fn(f64) -> f64, forms: &[Option<Affine>]| {
            forms[*i].as_ref().and_then(constant).map(|v| Affine {
                terms: vec![],
                constant: f(v),
            })
        };
        let form = match node {
            Node::Var(c) => Some(Affine {
                terms: vec![(*c, 1.0)],
                constant: 0.0,
            }),
            Node::Aux(k) => Some(Affine {
                terms: vec![(columns + k, 1.0)],
                constant: 0.0,
            }),
            Node::Const(c) => Some(Affine {
                terms: vec![],
                constant: c.value(),
            }),
            Node::Sum(c) => c
                .iter()
                .try_fold(BTreeMap::<usize, f64>::new(), |mut acc, i| {
                    let form = forms[*i].as_ref()?;
                    for (j, v) in &form.terms {
                        *acc.entry(*j).or_insert(0.0) += v;
                    }
                    (acc.len() <= AFFINE_TERMS).then_some(acc)
                })
                .map(|terms| Affine {
                    terms: terms.into_iter().filter(|(_, v)| *v != 0.0).collect(),
                    constant: c
                        .iter()
                        .filter_map(|i| forms[*i].as_ref().map(|f| f.constant))
                        .sum(),
                }),
            Node::Product(c) => {
                let mut factor = 1.0;
                let mut varying: Option<&Affine> = None;
                let mut linear = true;
                for i in c {
                    match forms[*i].as_ref() {
                        Some(f) if f.terms.is_empty() => factor *= f.constant,
                        Some(f) if varying.is_none() => varying = Some(f),
                        _ => linear = false,
                    }
                }
                if linear {
                    Some(varying.map_or(
                        Affine {
                            terms: vec![],
                            constant: factor,
                        },
                        |f| scaled(f, factor),
                    ))
                } else {
                    None
                }
            }
            Node::Pow { base, exponent } => match forms[*base].as_ref() {
                Some(f) if f.terms.is_empty() => Some(Affine {
                    terms: vec![],
                    constant: f.constant.powf(exponent.value()),
                }),
                Some(f) if exponent.value() == 1.0 => Some(f.clone()),
                _ => None,
            },
            Node::Exp(i) => unary(i, f64::exp, &forms),
            Node::Log(i) => unary(i, f64::ln, &forms),
            Node::Abs(i) => unary(i, f64::abs, &forms),
            Node::Sin(i) => unary(i, f64::sin, &forms),
            Node::Cos(i) => unary(i, f64::cos, &forms),
        };
        forms.push(form.filter(|f| f.constant.is_finite()));
    }
    forms
}

/// The export plan of an admitted program, or every typed refusal.
pub(crate) fn plan(
    program: &FactorableProgram,
    intent: SolveIntent,
) -> Result<Plan<'_>, Vec<Refusal>> {
    let mut refusals = Vec::new();
    let optimizes = matches!(intent, SolveIntent::Optimize | SolveIntent::Certify);
    let objective = match (&program.objective, optimizes) {
        (Some(o), true) => match o.expression {
            Some(node) if o.fidelity != Fidelity::Unavailable => Some((node, o.sense)),
            _ => {
                refusals.push(Refusal::ObjectiveUnavailable);
                None
            }
        },
        _ => None,
    };
    let mut constraints = Vec::new();
    let mut dropped = 0;
    let mut fidelity = Fidelity::Exact;
    for (r, row) in program.rows.iter().enumerate() {
        match row.expression {
            Some(node) => {
                fidelity = fidelity.max(row.fidelity);
                constraints.push(Function {
                    node,
                    lower: row.lower,
                    upper: row.upper,
                    origin: Origin::Row(r),
                });
            }
            None => dropped += 1,
        }
    }
    if dropped > 0 {
        fidelity = fidelity.max(Fidelity::Relaxed);
    }
    for (o, obligation) in program.obligations.iter().enumerate() {
        if obligation.scope != pse_math::factorable::ObligationScope::Unconditional {
            continue;
        }
        fidelity = fidelity.max(obligation.fidelity);
        for (k, c) in obligation.constraints.iter().enumerate() {
            let (lower, upper) = close(c);
            constraints.push(Function {
                node: c.expression,
                lower,
                upper,
                origin: Origin::Obligation(o, k),
            });
        }
    }
    for (b, block) in program.implicit.iter().enumerate() {
        fidelity = fidelity.max(block.fidelity);
        for (k, residual) in block.residuals.iter().enumerate() {
            constraints.push(Function {
                node: *residual,
                lower: 0.0,
                upper: 0.0,
                origin: Origin::Residual(b, k),
            });
        }
        for (k, c) in block.bounds.iter().enumerate() {
            let (lower, upper) = close(c);
            constraints.push(Function {
                node: c.expression,
                lower,
                upper,
                origin: Origin::ImplicitBound(b, k),
            });
        }
    }
    if let (Some(o), Some(_)) = (&program.objective, objective) {
        fidelity = fidelity.max(o.fidelity);
    }
    for v in &program.variables {
        if v.domain.is_semi() {
            refusals.push(Refusal::SemiDomain(v.id));
        }
    }
    let columns = program.variables.len();
    let boxes: Vec<(f64, f64)> = program
        .variables
        .iter()
        .map(|v| column_box(v.domain, v.lower, v.upper))
        .chain(program.auxiliaries.iter().map(|a| (a.lower, a.upper)))
        .collect();
    // Reachable nodes, and those under a nonlinear operator, in one reverse pass each.
    let nodes = &program.nodes;
    let affine = affine_forms(program);
    let mut reachable = vec![false; nodes.len()];
    for root in constraints
        .iter()
        .map(|c| c.node)
        .chain(objective.map(|o| o.0))
    {
        reachable[root] = true;
    }
    let mut nonlinear = vec![false; nodes.len()];
    for i in (0..nodes.len()).rev() {
        if !reachable[i] {
            continue;
        }
        let node = &nodes[i];
        if let Node::Const(c) = node
            && !c.value().is_finite()
        {
            refusals.push(Refusal::Constant(i));
        }
        if let Node::Pow { exponent, .. } = node
            && !exponent.value().is_finite()
        {
            refusals.push(Refusal::Constant(i));
        }
        // Sums and products with one varying factor pass linearity through; every other
        // operator over a varying operand makes its operands nonlinear.
        let varying = |c: &NodeId| affine[*c].as_ref().is_none_or(|f| !f.terms.is_empty());
        let operator = match node {
            Node::Var(_) | Node::Aux(_) | Node::Const(_) | Node::Sum(_) => false,
            Node::Product(c) => c.iter().filter(|c| varying(c)).count() > 1,
            Node::Pow { base, exponent } => exponent.value() != 1.0 && varying(base),
            Node::Exp(c) | Node::Log(c) | Node::Abs(c) | Node::Sin(c) | Node::Cos(c) => varying(c),
        };
        let inner = nonlinear[i] || operator;
        for &child in children(node) {
            reachable[child] = true;
            nonlinear[child] |= inner;
        }
    }
    let mut unbounded = BTreeMap::new();
    for (i, node) in nodes.iter().enumerate() {
        if !(reachable[i] && nonlinear[i]) {
            continue;
        }
        let (owner, index) = match node {
            Node::Var(c) => (BoundOwner::Variable(program.variables[*c].id), *c),
            Node::Aux(k) => (BoundOwner::Auxiliary(*k), columns + k),
            _ => continue,
        };
        let (lower, upper) = boxes[index];
        if !lower.is_finite() || !upper.is_finite() {
            unbounded.insert(
                index,
                MissingBound {
                    owner,
                    lower: !lower.is_finite(),
                    upper: !upper.is_finite(),
                },
            );
        }
    }
    refusals.extend(unbounded.into_values().map(Refusal::UnboundedNonlinear));
    if !refusals.is_empty() {
        return Err(refusals);
    }
    let mut h = FramedHasher::new("pse.factorable.domain.v1");
    h.hash(&program.key);
    for (v, (lower, upper)) in program.variables.iter().zip(&boxes) {
        h.id(&v.id)
            .u64(v.domain as u64)
            .u64(lower.to_bits())
            .u64(upper.to_bits());
    }
    for (lower, upper) in &boxes[columns..] {
        h.u64(lower.to_bits()).u64(upper.to_bits());
    }
    Ok(Plan {
        program,
        constraints,
        objective,
        dropped,
        fidelity,
        boxes,
        affine,
        domain: h.finish_hash(),
    })
}
/// Every typed refusal of exporting `program` for `intent`; empty means admitted.
pub fn admit_program(program: &FactorableProgram, intent: SolveIntent) -> Vec<Refusal> {
    plan(program, intent).err().unwrap_or_default()
}
fn refused(refusals: &[Refusal]) -> ProblemError {
    let reasons: Vec<String> = refusals.iter().map(ToString::to_string).collect();
    ProblemError::Unsupported(format!("factorable export refused: {}", reasons.join("; ")))
}

/// Builds original-coordinate NLP callbacks over every column, with the given column
/// indices fixed at the given integral values.
pub type FixedOracle<'a> =
    dyn FnMut(&BTreeMap<usize, f64>) -> Result<Box<dyn NlpOracle>, ProblemError> + 'a;
/// A continuous re-solve with the discrete columns fixed (ADR-0105 §2, T07).
pub struct Resolve<'a> {
    /// The fixed-assignment callbacks.
    pub oracle: &'a mut FixedOracle<'a>,
    /// Qualified library preprocessing policy of the re-solve.
    pub presolve: &'a presolve::Policy,
    /// Presolve dimension ceiling.
    pub limit: usize,
}
impl std::fmt::Debug for Resolve<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resolve")
            .field("limit", &self.limit)
            .finish_non_exhaustive()
    }
}
/// One factorable run.
pub struct Factorable<'a> {
    /// Program projected from the prepared case under the step's values.
    pub program: &'a FactorableProgram,
    /// Original start in program column order.
    pub initial: &'a [f64],
    /// Mathematical purpose; only optimization intents export the objective.
    pub intent: SolveIntent,
    /// The original compiled model every candidate is re-checked against.
    pub original: &'a mut dyn OriginalModel,
    /// The fixed-assignment re-solve; without it, a relaxed or unqualified incumbent
    /// remains an observation.
    pub resolve: Option<Resolve<'a>>,
}
impl std::fmt::Debug for Factorable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Factorable")
            .field("intent", &self.intent)
            .field("resolve", &self.resolve)
            .finish_non_exhaustive()
    }
}

/// Export, native global solve, readback, original re-qualification and the candidate
/// rule of ADR-0105 §2:
/// - an exact export's incumbent is the candidate when it meets the original tolerances
///   and the program is not mixed-integer nonlinear;
/// - otherwise the candidate comes from the fixed-assignment continuous re-solve through
///   the one NLP runner, started at the incumbent, and the backend's incumbent stays an
///   assignment proposal;
/// - without a re-solve, a relaxed export's incumbent is an observation only.
///
/// The dual bound and the candidate are recorded with their sources; `quality` grants a
/// gap only from both.
///
/// # Errors
/// A typed export refusal, or native execution failed before a report existed.
pub fn factorable(
    step: Step<'_>,
    retained: &mut Retained,
    run: Factorable<'_>,
) -> Result<SolveReport, ProblemError> {
    let plan = plan(run.program, run.intent).map_err(|r| refused(&r))?;
    if run.initial.len() != run.program.variables.len() {
        return Err(ProblemError::Contract("factorable start dimensions".into()));
    }
    let mut report = step.adapter.execute(
        retained,
        Input {
            problem: Problem::Factorable {
                program: run.program,
                initial: run.initial,
                intent: run.intent,
                normalization: step.normalization,
            },
            controls: step.controls,
            accuracy: step.accuracy,
            settings: step.settings,
            execution: step.execution.clone(),
            tolerances: step.tolerances,
            warm: step.warm,
            compatibility: step.compatibility.clone(),
        },
    )?;
    observe(&mut report, &plan, run.original, step.tolerances);
    report.metrics.insert(
        "export.rows.dropped".into(),
        Metric::Integer(i64::try_from(plan.dropped).unwrap_or(i64::MAX)),
    );
    if let Some(mut global) = report.evidence.global {
        // The runner's plan is the authority for what was exported.
        global.fidelity = plan.fidelity;
        global.domain = plan.domain;
        let feasible = report.validation_failure().is_none()
            && report.quality.as_ref().is_some_and(Quality::feasible);
        let proposal =
            global.fidelity != Fidelity::Exact || plan.discrete() && plan.nonlinear() || !feasible;
        global.primal = if global.fidelity == Fidelity::Exact {
            PrimalSource::Backend
        } else {
            PrimalSource::RelaxedIncumbent
        };
        if report.candidate.is_some()
            && proposal
            && let Some(resolve) = run.resolve
        {
            let objective = plan.objective.is_some();
            match fixed_assignment(&step, retained, &report, run.program, resolve, objective) {
                Ok(resolved) => {
                    if adopt(&mut report, resolved, &plan, run.original, &step) {
                        global.primal = PrimalSource::FixedAssignment;
                    }
                }
                Err(e) => {
                    report
                        .metrics
                        .insert("resolve.refused".into(), Metric::Text(e.to_string()));
                }
            }
        }
        global.dual = if global.fidelity == Fidelity::Exact {
            BoundSource::ExactExport
        } else {
            BoundSource::RelaxedExport
        };
        report.evidence.global = Some(global);
    }
    quality::qualify(&mut report, step.accuracy);
    Ok(report)
}

/// Fresh original values, the plan's declared boxes and integrality at the reported
/// candidate.
fn observe(
    report: &mut SolveReport,
    plan: &Plan<'_>,
    original: &mut dyn OriginalModel,
    tolerances: &quality::Tolerances,
) {
    let program = plan.program;
    let Some(candidate) = report.candidate.as_ref() else {
        return;
    };
    let result = quality::contained(|| {
        let x = &candidate.primal;
        tolerances.validate(program.variables.len(), program.rows.len())?;
        if x.len() != program.variables.len() || x.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("invalid factorable candidate"));
        }
        let fresh = original.evaluate(x)?;
        let limits: Vec<(f64, f64)> = program.rows.iter().map(|r| (r.lower, r.upper)).collect();
        let mut observation = Observation::from_values(fresh.objective, fresh.constraints, limits)?;
        observation.sources = fresh.sources;
        let rows = program
            .rows
            .iter()
            .zip(&observation.values)
            .zip(&tolerances.rows)
            .map(|((r, v), t)| Violation {
                id: r.id,
                physical: quality::interval(*v, r.lower, r.upper),
                tolerance: *t,
            })
            .collect();
        let mut bounds = Vec::with_capacity(x.len());
        let mut integrality = Vec::new();
        for (((v, x), t), (lower, upper)) in program
            .variables
            .iter()
            .zip(x)
            .zip(&tolerances.variables)
            .zip(&plan.boxes)
        {
            bounds.push(Violation {
                id: v.id,
                physical: quality::interval(*x, *lower, *upper),
                tolerance: *t,
            });
            if v.domain.is_integer() {
                integrality.push(Violation {
                    id: v.id,
                    physical: (x - x.round()).abs(),
                    tolerance: tolerances.integrality,
                });
            }
        }
        Ok((Quality::new(rows, bounds, integrality)?, observation))
    });
    match result {
        Ok((q, o)) => {
            report.quality = Some(q);
            report.observation = Some(o);
            report.clear_validation_failure();
        }
        Err(e) => {
            report.quality = None;
            report.observation = None;
            report.record_validation_failure(e);
        }
    }
}

/// The continuous problem with every integer column fixed at its rounded incumbent value,
/// solved through the one NLP runner by the automatic NLP route and seeded there.
fn fixed_assignment(
    step: &Step<'_>,
    retained: &mut Retained,
    report: &SolveReport,
    program: &FactorableProgram,
    resolve: Resolve<'_>,
    objective: bool,
) -> Result<SolveReport, ProblemError> {
    let incumbent = report
        .candidate
        .as_ref()
        .ok_or_else(|| ProblemError::Internal("fixed assignment without an incumbent".into()))?;
    let assignment: BTreeMap<usize, f64> = program
        .variables
        .iter()
        .enumerate()
        .filter(|(_, v)| v.domain.is_integer())
        .map(|(i, _)| (i, incumbent.primal[i].round()))
        .collect();
    let mut start = incumbent.primal.clone();
    for (i, v) in &assignment {
        start[*i] = *v;
    }
    let oracle = (resolve.oracle)(&assignment)?;
    let equalities = oracle
        .constraint_bounds()
        .iter()
        .all(|(l, u)| l.is_finite() && l == u);
    let facts = routing::oracle_facts(oracle.contract(), objective, equalities);
    let intent = if objective {
        SolveIntent::Optimize
    } else {
        SolveIntent::FeasiblePoint
    };
    // Native options belong to the global backend; the re-solve uses NLP defaults.
    let controls = Controls {
        options: Options::new(),
        threads: 1,
        ..step.controls.clone()
    };
    let Route::Native(backend) = (Requirements {
        table: &LINKED,
        facts: &facts,
        intent,
        convex: false,
        controls: &controls,
    })
    .select(SolverSelection::Auto)?
    else {
        return Err(ProblemError::Internal(
            "fixed-assignment re-solve has no free column".into(),
        ));
    };
    let adapter: &dyn BackendExecution = super::adapter(backend);
    if adapter.representation() != super::Representation::Nlp {
        return Err(ProblemError::Unsupported(format!(
            "fixed-assignment re-solve routed to {}, which is not an NLP adapter",
            backend.as_str()
        )));
    }
    let mut layout = FramedHasher::new("pse.factorable.fixed-assignment.v1");
    layout.hash(&step.compatibility.layout);
    for (i, v) in &assignment {
        layout.u64(*i as u64).u64(v.to_bits());
    }
    // The re-solve's native profile is its own: default settings of its NLP adapter under
    // the step's profile.
    let mut profile = FramedHasher::new("pse.factorable.fixed-assignment.profile.v1");
    profile
        .hash(&step.compatibility.profile)
        .str(backend.as_str());
    let compatibility = Compatibility {
        layout: layout.finish_hash(),
        profile: profile.finish_hash(),
        data: step.compatibility.data,
        backend,
    };
    let seed = WarmStart {
        origin: None,
        compatibility: compatibility.clone(),
        payload: adapter.primal_start(start.clone())?,
    };
    nlp(
        Step {
            adapter,
            settings: &BackendSettings::Default,
            controls: &controls,
            accuracy: step.accuracy,
            execution: step.execution.clone(),
            tolerances: step.tolerances,
            normalization: step.normalization,
            compatibility,
            warm: Some(&seed),
        },
        retained,
        Nlp {
            oracle,
            initial: &start,
            presolve: resolve.presolve,
            intent,
            sense: program
                .objective
                .as_ref()
                .map_or(ObjectiveSense::Minimize, |o| o.sense),
            limit: resolve.limit,
        },
    )
}
/// Adopt the re-solve's candidate when it met its tolerances, re-observed against the
/// original model with the declared box and integrality; returns whether it was adopted.
/// The backend's incumbent objective is kept as a metric.
fn adopt(
    report: &mut SolveReport,
    resolved: SolveReport,
    plan: &Plan<'_>,
    original: &mut dyn OriginalModel,
    step: &Step<'_>,
) -> bool {
    report.metrics.insert(
        "resolve.backend".into(),
        Metric::Text(resolved.backend.as_str().into()),
    );
    report.metrics.insert(
        "resolve.termination".into(),
        Metric::Text(resolved.termination.category.as_str().into()),
    );
    report.metrics.insert(
        "resolve.qualification".into(),
        Metric::Text(resolved.qualification.as_str().into()),
    );
    let usable = resolved.validation_failure().is_none()
        && resolved.quality.as_ref().is_some_and(Quality::feasible)
        && resolved.candidate.is_some();
    if !usable {
        return false;
    }
    if let Some(incumbent) = &report.candidate
        && let Some(value) = incumbent.objective
    {
        report
            .metrics
            .insert("scip.incumbent.objective".into(), Metric::Real(value));
    }
    let local = resolved.observation.clone();
    report.candidate = resolved.candidate;
    // A later global step is seeded with the adopted candidate, not the proposal.
    if let (Some(seed), Some(candidate)) = (report.warm_start.as_mut(), &report.candidate) {
        seed.payload = WarmPayload::Nlp {
            primal: candidate.primal.clone(),
            bounds: None,
            rows: None,
        };
    }
    report.evidence.kkt = resolved.evidence.kkt;
    report.preprocessing = resolved.preprocessing;
    observe(report, plan, original, step.tolerances);
    // Multipliers of the fixed-assignment problem are conditional on the assignment.
    if let (Some(observation), Some(local)) = (report.observation.as_mut(), local) {
        observation.stationarity = local.stationarity;
        observation.complementarity = local.complementarity;
        observation.dual_error = local.dual_error;
    }
    true
}
