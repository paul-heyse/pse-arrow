// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Sum-of-squares bounds on polynomial programs (Plan 22 N5; I5): the Lasserre moment
//! relaxation of `pounce-convex` over the polynomial projection of a factorable program.
//!
//! **Projection.** A program is polynomial when its objective and every row are exact
//! compositions of variables, constants, sums, products and powers with non-negative integer
//! exponents, with no auxiliary, implicit block, native form or validity obligation. The
//! projection expands each function into its monomials; a row `l ≤ g ≤ u` becomes
//! `g − l ≥ 0` and `u − g ≥ 0` (or `g − l = 0` when `l = u`), and a finite variable bound a
//! linear inequality. A maximization bounds `−f`.
//!
//! **Assurance.** The relaxation solves a semidefinite program in floating point. Its value
//! bounds the optimum only up to the SDP's own accuracy, and no rational certificate is
//! extracted, so the bound is labelled `sos_bound_nonrigorous`: never a certified global
//! bound, never the basis of an objective-bound check (ADR-0106 §9).
use crate::{ProblemError, solve::Assurance};
use pse_math::{binding::ObjectiveSense, factorable::FactorableProgram};
use std::collections::BTreeMap;

/// Exponent vector → coefficient.
type Terms = BTreeMap<Vec<u32>, f64>;

/// Bounds on the size of a projection: its degree, its terms per function, and the moment
/// basis of the relaxation it implies.
const MAX_DEGREE: u32 = 8;
const MAX_TERMS: usize = 4096;
const MAX_BASIS: usize = 256;

/// A polynomial program: the objective to minimize, inequalities `gᵢ ≥ 0` and equalities
/// `hⱼ = 0` over `variables` coordinates, as exponent vectors and coefficients.
#[derive(Clone, Debug, PartialEq)]
pub struct Polynomial {
    /// Coordinates, the program's free columns in order.
    pub variables: usize,
    /// The objective in the minimization sense.
    pub objective: Vec<(Vec<u32>, f64)>,
    /// Inequalities `g ≥ 0`.
    pub inequalities: Vec<Vec<(Vec<u32>, f64)>>,
    /// Equalities `h = 0`.
    pub equalities: Vec<Vec<(Vec<u32>, f64)>>,
    /// The authored orientation, which maps the bound back.
    pub sense: ObjectiveSense,
}
impl Polynomial {
    /// The largest total degree of any function.
    pub fn degree(&self) -> u32 {
        std::iter::once(&self.objective)
            .chain(&self.inequalities)
            .chain(&self.equalities)
            .flat_map(|f| f.iter().map(|(e, _)| e.iter().sum::<u32>()))
            .max()
            .unwrap_or(0)
    }
}

/// A bound from the moment relaxation, in the authored sense: a lower bound on a
/// minimization's optimum, an upper bound on a maximization's.
#[derive(Clone, Debug, PartialEq)]
pub struct SosBound {
    /// The bound.
    pub bound: f64,
    /// The relaxation order: half the degree of the moment basis products.
    pub order: usize,
    /// The native SDP status.
    pub status: String,
    /// `sos_bound_nonrigorous` when the SDP solved to its tolerance; none otherwise.
    pub assurance: Assurance,
}

fn add(into: &mut Terms, from: &Terms, scale: f64) {
    for (e, c) in from {
        *into.entry(e.clone()).or_insert(0.0) += scale * c;
    }
    into.retain(|_, c| *c != 0.0);
}
fn multiply(a: &Terms, b: &Terms) -> Result<Terms, ProblemError> {
    let mut out = Terms::new();
    for (ea, ca) in a {
        for (eb, cb) in b {
            let e: Vec<u32> = ea.iter().zip(eb).map(|(x, y)| x + y).collect();
            if e.iter().sum::<u32>() > MAX_DEGREE {
                return Err(too_large("degree"));
            }
            *out.entry(e).or_insert(0.0) += ca * cb;
        }
    }
    out.retain(|_, c| *c != 0.0);
    if out.len() > MAX_TERMS {
        return Err(too_large("terms"));
    }
    Ok(out)
}
fn too_large(what: &str) -> ProblemError {
    ProblemError::Unsupported(format!(
        "the polynomial projection exceeds its {what} bound (degree {MAX_DEGREE}, {MAX_TERMS} terms)"
    ))
}
fn constant(n: usize, value: f64) -> Terms {
    let mut t = Terms::new();
    if value != 0.0 {
        t.insert(vec![0; n], value);
    }
    t
}
fn listed(t: Terms) -> Vec<(Vec<u32>, f64)> {
    t.into_iter().collect()
}

/// The polynomial projection of `program`.
///
/// # Errors
/// A function that is not an exact polynomial of the variables, an auxiliary, implicit
/// block, native form or obligation, a missing objective, or a projection beyond its size
/// bounds.
pub fn polynomial(program: &FactorableProgram) -> Result<Polynomial, ProblemError> {
    use pse_math::factorable::{Fidelity, Node};
    let refuse = |why: &str| {
        Err(ProblemError::Unsupported(format!(
            "no polynomial projection: {why}"
        )))
    };
    if !program.auxiliaries.is_empty()
        || !program.implicit.is_empty()
        || !program.native.is_empty()
        || !program.incomplete.is_empty()
    {
        return refuse("auxiliary, implicit, native or incomplete structure");
    }
    if !program.obligations.is_empty() {
        return refuse("validity obligations restrict the domain");
    }
    let Some(objective) = &program.objective else {
        return refuse("no objective");
    };
    let n = program.variables.len();
    let mut nodes: Vec<Option<Terms>> = Vec::with_capacity(program.nodes.len());
    for node in &program.nodes {
        let terms = match node {
            Node::Var(j) => {
                let mut e = vec![0; n];
                e[*j] = 1;
                Some(Terms::from([(e, 1.0)]))
            }
            Node::Const(c) => Some(constant(n, c.value())),
            Node::Sum(children) => children.iter().try_fold(Terms::new(), |mut acc, c| {
                let child = nodes.get(*c).and_then(Option::as_ref)?;
                add(&mut acc, child, 1.0);
                Some(acc)
            }),
            Node::Product(children) => {
                let mut acc = Some(constant(n, 1.0));
                for c in children {
                    acc = match (acc, nodes.get(*c).and_then(Option::as_ref)) {
                        (Some(a), Some(child)) => Some(multiply(&a, child)?),
                        _ => None,
                    };
                }
                acc
            }
            Node::Pow { base, exponent } => {
                let e = exponent.value();
                match nodes.get(*base).and_then(Option::as_ref) {
                    Some(b) if e >= 0.0 && e.fract() == 0.0 && e <= f64::from(MAX_DEGREE) => {
                        let mut acc = constant(n, 1.0);
                        for _ in 0..(e as u32) {
                            acc = multiply(&acc, b)?;
                        }
                        Some(acc)
                    }
                    _ => None,
                }
            }
            Node::Aux(_)
            | Node::Exp(_)
            | Node::Log(_)
            | Node::Abs(_)
            | Node::Sin(_)
            | Node::Cos(_) => None,
        };
        nodes.push(terms);
    }
    let function = |node: Option<pse_math::factorable::NodeId>, fidelity: Fidelity| {
        if fidelity != Fidelity::Exact {
            return None;
        }
        nodes.get(node?).cloned().flatten()
    };
    let Some(f) = function(objective.expression, objective.fidelity) else {
        return refuse("the objective is not an exact polynomial");
    };
    let sign = objective.sense.sign();
    let mut inequalities = Vec::new();
    let mut equalities = Vec::new();
    for row in &program.rows {
        let Some(g) = function(row.expression, row.fidelity) else {
            return refuse("a row is not an exact polynomial");
        };
        if row.lower == row.upper {
            let mut h = g;
            add(&mut h, &constant(n, -row.lower), 1.0);
            equalities.push(listed(h));
            continue;
        }
        if row.lower.is_finite() {
            let mut h = g.clone();
            add(&mut h, &constant(n, -row.lower), 1.0);
            inequalities.push(listed(h));
        }
        if row.upper.is_finite() {
            let mut h = constant(n, row.upper);
            add(&mut h, &g, -1.0);
            inequalities.push(listed(h));
        }
    }
    for (j, v) in program.variables.iter().enumerate() {
        let mut x = vec![0; n];
        x[j] = 1;
        if v.lower.is_finite() {
            inequalities.push(listed(Terms::from([
                (x.clone(), 1.0),
                (vec![0; n], -v.lower),
            ])));
        }
        if v.upper.is_finite() {
            inequalities.push(listed(Terms::from([(x, -1.0), (vec![0; n], v.upper)])));
        }
    }
    let mut minimized = Terms::new();
    add(&mut minimized, &f, sign);
    let problem = Polynomial {
        variables: n,
        objective: listed(minimized),
        inequalities,
        equalities,
        sense: objective.sense,
    };
    // The moment basis of the default order has C(n + d, d) monomials.
    let d = problem.degree().div_ceil(2) as usize;
    let basis = (1..=d).try_fold(1usize, |acc, k| acc.checked_mul(n + k).map(|v| v / k));
    if basis.is_none_or(|b| b > MAX_BASIS) {
        return Err(ProblemError::Unsupported(format!(
            "the moment relaxation of {n} variables at order {d} exceeds {MAX_BASIS} basis monomials"
        )));
    }
    Ok(problem)
}

/// The moment relaxation's bound on `problem`, at `order` or the least order its degree
/// admits, within `time_limit` and to `tolerance`.
///
/// # Errors
/// This build does not link POUNCE-convex.
pub fn bound(
    problem: &Polynomial,
    order: Option<usize>,
    tolerance: f64,
    time_limit: std::time::Duration,
) -> Result<SosBound, ProblemError> {
    #[cfg(feature = "pounce")]
    {
        use pounce_rs::{
            convex::{
                PolyProblem, Polynomial as Terms, QpOptions, QpStatus,
                sos_constrained_lower_bound_opts, sos_opts,
            },
            linsol::{FeralSolverInterface, SparseSymLinearSolverInterface},
        };
        let poly = |terms: &[(Vec<u32>, f64)]| {
            Terms::new(
                problem.variables,
                terms
                    .iter()
                    .map(|(e, c)| (e.iter().map(|v| *v as usize).collect(), *c))
                    .collect(),
            )
        };
        let mut sdp = PolyProblem::new(poly(&problem.objective));
        for g in &problem.inequalities {
            sdp = sdp.ge(poly(g));
        }
        for h in &problem.equalities {
            sdp = sdp.eq(poly(h));
        }
        let order = order.unwrap_or_else(|| problem.degree().div_ceil(2).max(1) as usize);
        let options = QpOptions {
            tol: tolerance,
            time_limit: Some(time_limit),
            ..sos_opts()
        };
        let result = sos_constrained_lower_bound_opts(
            &sdp,
            Some(order),
            &options,
            || -> Box<dyn SparseSymLinearSolverInterface> {
                let config = crate::settings::pounce::LinearSettings {
                    fma: false,
                    ..Default::default()
                };
                Box::new(FeralSolverInterface::with_config(config))
            },
        );
        let solved = result.status == QpStatus::Optimal && result.lower_bound.is_finite();
        Ok(SosBound {
            bound: problem.sense.sign() * result.lower_bound,
            order,
            status: format!("{:?}", result.status),
            assurance: if solved {
                Assurance::SosBoundNonrigorous
            } else {
                Assurance::None
            },
        })
    }
    #[cfg(not(feature = "pounce"))]
    {
        let _ = (problem, order, tolerance, time_limit);
        Err(ProblemError::Unsupported(
            "SOS bounds need POUNCE-convex, which this build does not link".into(),
        ))
    }
}
