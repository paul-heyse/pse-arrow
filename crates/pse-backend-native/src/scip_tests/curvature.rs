// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! SCIP as the differential oracle of the curvature pass (ADR-0121 Outcome 5).
use super::*;
use crate::scip::ScipCurvature;
use pse_math::{
    convexity::EXACT_OPERATIONS,
    curvature::{Curvature, node_curvature},
};
use pse_quantity::Ratio;

type Build = fn(&mut Body<'_>) -> TypedValue;
fn pow(body: &mut Body<'_>, base: &TypedValue, exponent: i32) -> TypedValue {
    let e = body.c(f64::from(exponent));
    body.b
        .binary(
            Binary::Pow,
            base.clone(),
            e,
            Some(Ratio::new(exponent, 1).unwrap()),
            id(204),
        )
        .unwrap()
}
fn real_pow(body: &mut Body<'_>, base: &TypedValue, exponent: f64) -> TypedValue {
    let e = body.c(exponent);
    body.b
        .binary(Binary::Pow, base.clone(), e, None, id(205))
        .unwrap()
}
/// The corpus over a positive box `x, y ∈ [0.5, 2]`, each with the curvature the pass
/// proves.
fn positive() -> Vec<(&'static str, Build, Curvature)> {
    use Curvature::{Concave, Convex, Unknown};
    vec![
        ("exp(x)", |b| { let x = b.x[0].clone(); b.f(Function::Exp, &x) }, Convex),
        ("exp(x + 2y)", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let two = b.c(2.0);
            let t = b.op(Binary::Mul, &two, &y);
            let s = b.op(Binary::Add, &x, &t);
            b.f(Function::Exp, &s)
        }, Convex),
        ("log(x)", |b| { let x = b.x[0].clone(); b.f(Function::Log, &x) }, Concave),
        ("log(x) + log(y)", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let (lx, ly) = (b.f(Function::Log, &x), b.f(Function::Log, &y));
            b.op(Binary::Add, &lx, &ly)
        }, Concave),
        ("x·log(x)", |b| {
            let x = b.x[0].clone();
            let l = b.f(Function::Log, &x);
            b.op(Binary::Mul, &x, &l)
        }, Convex),
        ("x·log(x/y)", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let r = b.op(Binary::Div, &x, &y);
            let l = b.f(Function::Log, &r);
            b.op(Binary::Mul, &x, &l)
        }, Convex),
        ("x^2.5", |b| { let x = b.x[0].clone(); real_pow(b, &x, 2.5) }, Convex),
        ("sqrt(x)", |b| { let x = b.x[0].clone(); b.f(Function::Sqrt, &x) }, Concave),
        ("1/x", |b| {
            let x = b.x[0].clone();
            let one = b.c(1.0);
            b.op(Binary::Div, &one, &x)
        }, Convex),
        ("x^-2", |b| { let x = b.x[0].clone(); pow(b, &x, -2) }, Convex),
        ("sqrt(x² + y²)", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let (x2, y2) = (pow(b, &x, 2), pow(b, &y, 2));
            let s = b.op(Binary::Add, &x2, &y2);
            b.f(Function::Sqrt, &s)
        }, Convex),
        ("x² + xy + y²", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let (x2, y2) = (pow(b, &x, 2), pow(b, &y, 2));
            let xy = b.op(Binary::Mul, &x, &y);
            let s = b.op(Binary::Add, &x2, &xy);
            b.op(Binary::Add, &s, &y2)
        }, Convex),
        ("xy − x² − y²", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let (x2, y2) = (pow(b, &x, 2), pow(b, &y, 2));
            let xy = b.op(Binary::Mul, &x, &y);
            let s = b.op(Binary::Sub, &xy, &x2);
            b.op(Binary::Sub, &s, &y2)
        }, Concave),
        ("exp(x² + y²)", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let (x2, y2) = (pow(b, &x, 2), pow(b, &y, 2));
            let s = b.op(Binary::Add, &x2, &y2);
            b.f(Function::Exp, &s)
        }, Convex),
        ("log(sqrt(x) + y)", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let r = b.f(Function::Sqrt, &x);
            let s = b.op(Binary::Add, &r, &y);
            b.f(Function::Log, &s)
        }, Concave),
        ("|x − y|", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let d = b.op(Binary::Sub, &x, &y);
            b.f(Function::Abs, &d)
        }, Convex),
        ("3·exp(x) − 2·log(y)", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let (three, two) = (b.c(3.0), b.c(2.0));
            let e = b.f(Function::Exp, &x);
            let l = b.f(Function::Log, &y);
            let (e3, l2) = (b.op(Binary::Mul, &three, &e), b.op(Binary::Mul, &two, &l));
            b.op(Binary::Sub, &e3, &l2)
        }, Convex),
        ("(x + y)²", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let s = b.op(Binary::Add, &x, &y);
            pow(b, &s, 2)
        }, Convex),
        ("−exp(x)", |b| {
            let x = b.x[0].clone();
            let e = b.f(Function::Exp, &x);
            let zero = b.c(0.0);
            b.op(Binary::Sub, &zero, &e)
        }, Concave),
        ("x·y", |b| { let (x, y) = (b.x[0].clone(), b.x[1].clone()); b.op(Binary::Mul, &x, &y) }, Unknown),
        ("sin(x)", |b| { let x = b.x[0].clone(); b.f(Function::Sin, &x) }, Unknown),
        ("x² − y²", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let (x2, y2) = (pow(b, &x, 2), pow(b, &y, 2));
            b.op(Binary::Sub, &x2, &y2)
        }, Unknown),
        ("exp(x)·log(y)", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let e = b.f(Function::Exp, &x);
            let l = b.f(Function::Log, &y);
            b.op(Binary::Mul, &e, &l)
        }, Unknown),
    ]
}
/// The corpus over a box across zero, `x, y ∈ [−1, 2]`.
fn signed() -> Vec<(&'static str, Build, Curvature)> {
    use Curvature::{Convex, Unknown};
    vec![
        ("x²", |b| { let x = b.x[0].clone(); pow(b, &x, 2) }, Convex),
        ("x³", |b| { let x = b.x[0].clone(); pow(b, &x, 3) }, Unknown),
        ("|x|", |b| { let x = b.x[0].clone(); b.f(Function::Abs, &x) }, Convex),
        ("exp(−x)", |b| {
            let x = b.x[0].clone();
            let zero = b.c(0.0);
            let n = b.op(Binary::Sub, &zero, &x);
            b.f(Function::Exp, &n)
        }, Convex),
        ("(x − y)⁴", |b| {
            let (x, y) = (b.x[0].clone(), b.x[1].clone());
            let d = b.op(Binary::Sub, &x, &y);
            pow(b, &d, 4)
        }, Convex),
        ("x·y", |b| { let (x, y) = (b.x[0].clone(), b.x[1].clone()); b.op(Binary::Mul, &x, &y) }, Unknown),
    ]
}
/// Midpoint convexity (or concavity) of a node at sampled pairs of the box: the numerical
/// check of a claim SCIP's rules leave undecided.
fn midpoint(program: &FactorableProgram, node: usize, sign: f64, boxes: &[(f64, f64)]) -> bool {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = |(lo, hi): (f64, f64)| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        lo + (hi - lo) * ((state >> 11) as f64 / (1_u64 << 53) as f64)
    };
    let value = |p: &[f64]| program.evaluate(p, &[]).unwrap()[node];
    for _ in 0..200 {
        let p: Vec<f64> = boxes.iter().map(|b| next(*b)).collect();
        let q: Vec<f64> = boxes.iter().map(|b| next(*b)).collect();
        for lambda in [0.25, 0.5, 0.75] {
            let m: Vec<f64> = p
                .iter()
                .zip(&q)
                .map(|(a, b)| lambda * a + (1.0 - lambda) * b)
                .collect();
            let chord = lambda * value(&p) + (1.0 - lambda) * value(&q);
            if sign * (value(&m) - chord) > 1e-9 * chord.abs().max(1.0) {
                return false;
            }
        }
    }
    true
}

/// ADR-0121 Outcome 5: over a corpus of expressions on two boxes, the pass's curvature is
/// the one expected, and whenever the pass proves a curvature SCIP's own detection agrees
/// or leaves the expression undecided; an undecided claim must then hold numerically at
/// sampled midpoints. SCIP never proves the opposite of a claim, and it confirms most.
#[test]
fn curvature_sound_against_scip_oracle() {
    let registry = standard_registry().unwrap();
    let mut confirmed = 0;
    let mut undecided = Vec::new();
    for (corpus, (lo, hi)) in [(positive(), (0.5, 2.0)), (signed(), (-1.0, 2.0))] {
        let mut body = Body::new(&registry, 2);
        let outputs: Vec<TypedValue> = corpus.iter().map(|(_, build, _)| build(&mut body)).collect();
        let prepared = body.b.prepare(&outputs).unwrap();
        let column = (ModelingVariableDomain::Continuous, Some(lo), Some(hi), (lo + hi) / 2.0);
        let rows = vec![(f64::NEG_INFINITY, f64::INFINITY); outputs.len()];
        let case = case(
            &registry,
            prepared,
            &[column, column],
            &rows,
            None,
            DerivativeOrder::Value,
        );
        let program = case.program(&FactorableRequest::default());
        let roots: Vec<usize> = program
            .rows
            .iter()
            .map(|r| r.expression.unwrap())
            .collect();
        let scip = scip::curvature(&program, &roots).unwrap();
        let never = AtomicBool::new(false);
        for (((name, _, expected), root), oracle) in corpus.iter().zip(&roots).zip(&scip) {
            let pse = node_curvature(&program, *root, EXACT_OPERATIONS, &never).unwrap();
            assert_eq!(pse, *expected, "{name}: pse {pse:?}, SCIP {oracle:?}");
            let sign = match pse {
                Curvature::Convex => 1.0,
                Curvature::Concave => -1.0,
                _ => continue,
            };
            match (sign > 0.0, oracle) {
                (true, ScipCurvature::Convex | ScipCurvature::Linear)
                | (false, ScipCurvature::Concave | ScipCurvature::Linear) => confirmed += 1,
                (_, ScipCurvature::Unknown) => {
                    assert!(
                        midpoint(&program, *root, sign, &[(lo, hi), (lo, hi)]),
                        "{name}: {pse:?} fails at sampled midpoints"
                    );
                    undecided.push(*name);
                }
                _ => panic!("{name}: the pass proves {pse:?}, SCIP proves {oracle:?}"),
            }
        }
    }
    assert!(confirmed >= 12, "SCIP confirmed {confirmed}; undecided {undecided:?}");
    eprintln!("SCIP confirmed {confirmed} claims; undecided by SCIP: {undecided:?}");
}
