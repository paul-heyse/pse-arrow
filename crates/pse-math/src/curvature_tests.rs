// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Curvature pass units (ADR-0121 Outcome 1): programs projected from typed bodies exactly
//! as preparation projects an authored case.
use super::*;
use crate::{
    convexity::{ConvexityClass, EXACT_OPERATIONS, Unrecognized},
    curvature::{self, ConeConstraint, Epigraph, Linear, Recognition, Side},
};
use pse_quantity::Ratio;

/// Columns `id(i + 1)` with `domains[i]` and `boxes[i]`; output `k` contributes to the
/// objective when it is `objective`, otherwise to the next row, bounded by `rows`.
fn bounded(
    registry: &QuantityRegistry,
    body: PreparedBody,
    boxes: &[(f64, f64)],
    domains: &[ModelingVariableDomain],
    rows: &[(f64, f64)],
    objective: Option<(usize, ObjectiveSense)>,
) -> Arc<CaseAssembly> {
    let key = ContentHash::from_bytes([7; 32]);
    let variables = boxes
        .iter()
        .enumerate()
        .map(|(i, (lower, upper))| Variable {
            port: port(registry, u8::try_from(i + 1).unwrap()),
            fixed: false,
            domain: domains
                .get(i)
                .copied()
                .unwrap_or(ModelingVariableDomain::Continuous),
            lower: Some(*lower),
            upper: Some(*upper),
        })
        .collect();
    let slots: Vec<_> = (0..boxes.len())
        .map(|i| {
            let p = port(registry, u8::try_from(i + 1).unwrap());
            SlotBinding::new(&p, &p, registry).unwrap()
        })
        .collect();
    let mut row = 0;
    let mut contributions = Vec::new();
    for k in 0..body.output_count() {
        let target = if objective.is_some_and(|(o, _)| o == k) {
            Target::Objective(0)
        } else {
            row += 1;
            Target::Row(id(100 + u8::try_from(row).unwrap()))
        };
        contributions.push(Contribution {
            output: k,
            target,
            scale: 1.0,
        });
    }
    let rows = rows
        .iter()
        .enumerate()
        .map(|(r, (lower, upper))| Row {
            id: id(101 + u8::try_from(r).unwrap()),
            quantity: neutral(),
            lower: *lower,
            upper: *upper,
        })
        .collect();
    let structure = Arc::new(
        CaseStructure::new(
            variables,
            vec![],
            vec![InstanceBinding {
                checked_members: Default::default(),
                instance: id(9),
                body: key,
                slots: slots.into(),
                contributions,
            }],
            rows,
            objective.map(|(_, sense)| Objective {
                quantity: neutral(),
                sense,
            }),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    Arc::new(
        Arc::new(
            CasePlan::prepare(
                structure,
                BTreeMap::from([(key, Arc::new(body))]),
                registry,
                DerivativeOrder::Value,
                AssemblyLimits::default(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap(),
        )
        .compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap(),
    )
}
fn recognize(program: &FactorableProgram) -> Recognition {
    curvature::recognize(program, EXACT_OPERATIONS, &AtomicBool::new(false)).unwrap()
}
fn power(
    b: &mut BodyBuilder<'_>,
    base: &TypedValue,
    numerator: i32,
    denominator: i32,
) -> TypedValue {
    let exponent = number(Atom::num(i64::from(numerator)) / Atom::num(i64::from(denominator)));
    b.binary(
        Binary::Pow,
        base.clone(),
        exponent,
        Some(Ratio::new(numerator, denominator).unwrap()),
        id(210),
    )
    .unwrap()
}
fn constant(value: i64) -> TypedValue {
    number(Atom::num(value))
}
/// Whether one cone membership holds at `z` within `tolerance`.
fn holds(cone: &ConeConstraint, z: &[f64], tolerance: f64) -> bool {
    let at = |l: &Linear| l.at(z);
    match cone {
        ConeConstraint::Nonnegative(f) => at(f) >= -tolerance,
        ConeConstraint::Exponential([x, y, w]) => {
            let (x, y, w) = (at(x), at(y), at(w));
            if y > 0.0 {
                y * (x / y).exp() <= w + tolerance * w.abs().max(1.0)
            } else {
                y >= -tolerance && x <= tolerance && w >= -tolerance
            }
        }
        ConeConstraint::Power { alpha, members } => {
            let (x, y, w) = (at(&members[0]), at(&members[1]), at(&members[2]));
            x >= -tolerance
                && y >= -tolerance
                && x.max(0.0).powf(*alpha) * y.max(0.0).powf(1.0 - alpha)
                    >= w.abs() - tolerance * w.abs().max(1.0)
        }
        ConeConstraint::SecondOrder(members) => {
            let head = at(&members[0]);
            let norm = members[1..]
                .iter()
                .map(|m| at(m).powi(2))
                .sum::<f64>()
                .sqrt();
            head >= norm - tolerance * head.abs().max(1.0)
        }
    }
}
/// The epigraph form is tight: with every auxiliary at its node's value, every cone
/// membership holds and every row form equals its row function, at every point.
fn assert_tight(program: &FactorableProgram, form: &Epigraph, points: &[Vec<f64>]) {
    for point in points {
        let values = program.evaluate(point, &[]).unwrap();
        let mut z = point.clone();
        z.extend(form.auxiliaries.iter().map(|n| values[*n]));
        for cone in &form.cones {
            assert!(holds(cone, &z, 1e-9), "{cone:?} at {point:?}");
        }
        for row in &form.rows {
            let expected = values[program.rows[row.row].expression.unwrap()];
            assert!(
                close(row.form.at(&z), expected),
                "row {} {:?}: {} vs {expected}",
                row.row,
                row.side,
                row.form.at(&z)
            );
        }
        if let (Some(objective), Some(o)) = (&form.objective, &program.objective) {
            let expected = o.sense.sign() * values[o.expression.unwrap()];
            assert!(close(objective.at(&z), expected), "objective at {point:?}");
        }
    }
}
fn sample(boxes: &[(f64, f64)], count: usize, seed: u64) -> Vec<Vec<f64>> {
    let mut points = Points(seed);
    (0..count)
        .map(|_| boxes.iter().map(|b| points.next(*b)).collect())
        .collect()
}

/// Every DCP atom of the pass, each in the direction its row bound requires, is recognized
/// with its cone; the epigraph form is tight at sampled points; and the fact records the
/// cones.
#[test]
fn curvature_recognizes_dcp_atoms() {
    let registry = standard_registry().unwrap();
    let mut b = builder(&registry, 3);
    let v = inputs(&mut b, 3);
    let (x, y, z) = (&v[0], &v[1], &v[2]);
    // exp(x) + exp(y) ≤ 10
    let ex = unary(&mut b, Function::Exp, x);
    let ey = unary(&mut b, Function::Exp, y);
    let exps = op(&mut b, Binary::Add, &ex, &ey);
    // log(x) ≥ −1
    let lx = unary(&mut b, Function::Log, x);
    // x·log x + x·log(x/y) ≤ 5
    let entropy = op(&mut b, Binary::Mul, x, &lx);
    let ratio = op(&mut b, Binary::Div, x, y);
    let lr = unary(&mut b, Function::Log, &ratio);
    let relative = op(&mut b, Binary::Mul, x, &lr);
    let entropies = op(&mut b, Binary::Add, &entropy, &relative);
    // x^(5/2) ≤ 10, √y ≥ 0.1, 1/x ≤ 3
    let p52 = power(&mut b, x, 5, 2);
    let root = unary(&mut b, Function::Sqrt, y);
    let one = constant(1);
    let reciprocal = op(&mut b, Binary::Div, &one, x);
    // |z| ≤ 0.5
    let magnitude = unary(&mut b, Function::Abs, z);
    // √(x² + y²) ≤ 3
    let x2 = power(&mut b, x, 2, 1);
    let y2 = power(&mut b, y, 2, 1);
    let squares = op(&mut b, Binary::Add, &x2, &y2);
    let norm = unary(&mut b, Function::Sqrt, &squares);
    // x² + x·y + y² ≤ 5: DCP cannot read the bilinear term; the exact Gram certificate can.
    let xy = op(&mut b, Binary::Mul, x, y);
    let quadratic = op(&mut b, Binary::Add, &squares, &xy);
    // Minimize z − log(y): a convex objective.
    let ly = unary(&mut b, Function::Log, y);
    let objective = op(&mut b, Binary::Sub, z, &ly);
    let body = b
        .prepare(&[
            exps, lx, entropies, p52, root, reciprocal, magnitude, norm, quadratic, objective,
        ])
        .unwrap();
    let boxes = [(0.5, 2.0), (0.5, 2.0), (-1.0, 1.0)];
    let inf = f64::INFINITY;
    let assembly = bounded(
        &registry,
        body,
        &boxes,
        &[],
        &[
            (-inf, 10.0),
            (-1.0, inf),
            (-inf, 5.0),
            (-inf, 10.0),
            (0.1, inf),
            (-inf, 3.0),
            (-inf, 0.5),
            (-inf, 3.0),
            (-inf, 5.0),
        ],
        Some((9, ObjectiveSense::Minimize)),
    );
    let program = project(&assembly, &FactorableRequest::default());
    let Recognition::Cone(form) = recognize(&program) else {
        panic!("recognized: {:?}", recognize(&program))
    };
    let summary = form.summary();
    // exp ×2, log x, entropy, relative entropy, log y.
    assert_eq!(summary.exponential, 6, "{form:#?}");
    // x^(5/2), √y, 1/x.
    assert_eq!(summary.power, 3, "{form:#?}");
    // The norm and the certified quadratic.
    assert_eq!(summary.second_order, 2, "{form:#?}");
    assert_eq!(summary.nonnegative, 2, "{form:#?}");
    assert_eq!(form.rows.len(), 9);
    assert!(form.rows.iter().enumerate().all(|(i, r)| r.row == i));
    assert_eq!(form.rows[1].side, Side::Lower);
    assert_eq!(form.rows[4].side, Side::Lower);
    assert_tight(&program, &form, &sample(&boxes, 32, 3));
    // The fact records the class and changes with the program.
    let fact = curvature::fact(&program, EXACT_OPERATIONS, &AtomicBool::new(false)).unwrap();
    assert_eq!(fact.class, ConvexityClass::Cone(summary));
    assert!(fact.cone());
}

/// ADR-0121 Outcome 2 in a row: a degree-two row with a cross term is certified by the
/// exact rational LDLᵀ and lowered to a second-order cone that is tight at the row's value
/// and violated below it; a maximized convex quadratic is refused.
#[test]
fn gram_certificate_yields_soc() {
    let registry = standard_registry().unwrap();
    let mut b = builder(&registry, 2);
    let v = inputs(&mut b, 2);
    let (x, y) = (&v[0], &v[1]);
    let x2 = power(&mut b, x, 2, 1);
    let y2 = power(&mut b, y, 2, 1);
    let xy = op(&mut b, Binary::Mul, x, y);
    let two = constant(2);
    let twice = op(&mut b, Binary::Mul, &two, &y2);
    let sum = op(&mut b, Binary::Add, &x2, &twice);
    let quadratic = op(&mut b, Binary::Sub, &sum, &xy);
    let objective = op(&mut b, Binary::Add, x, y);
    let body = b.prepare(&[quadratic, objective]).unwrap();
    let boxes = [(-3.0, 3.0), (-3.0, 3.0)];
    let assembly = bounded(
        &registry,
        body.clone(),
        &boxes,
        &[],
        &[(f64::NEG_INFINITY, 1.0)],
        Some((1, ObjectiveSense::Minimize)),
    );
    let program = project(&assembly, &FactorableRequest::default());
    let Recognition::Cone(form) = recognize(&program) else {
        panic!("x² − xy + 2y² ≤ 1 is convex")
    };
    let [ConeConstraint::SecondOrder(members)] = form.cones.as_slice() else {
        panic!("one second-order cone: {form:#?}")
    };
    // Rotated cone over the two Gram squares: 1 + u, 1 − u, and one member per square.
    assert_eq!(members.len(), 4);
    assert_eq!(form.summary().second_order, 1);
    let points = sample(&boxes, 32, 7);
    assert_tight(&program, &form, &points);
    for point in &points {
        let values = program.evaluate(point, &[]).unwrap();
        let q = values[program.rows[0].expression.unwrap()];
        let mut below = point.clone();
        below.push(q - 0.5);
        assert!(!holds(&form.cones[0], &below, 1e-9), "{point:?}");
    }
    // Maximizing the convex quadratic as the objective is not a convex program.
    let mut b = builder(&registry, 2);
    let v = inputs(&mut b, 2);
    let x2 = power(&mut b, &v[0], 2, 1);
    let xy = op(&mut b, Binary::Mul, &v[0], &v[1]);
    let objective = op(&mut b, Binary::Add, &x2, &xy);
    let ex = unary(&mut b, Function::Exp, &v[1]);
    let body = b.prepare(&[ex, objective]).unwrap();
    let assembly = bounded(
        &registry,
        body,
        &boxes,
        &[],
        &[(f64::NEG_INFINITY, 2.0)],
        Some((1, ObjectiveSense::Maximize)),
    );
    let program = project(&assembly, &FactorableRequest::default());
    assert!(matches!(
        recognize(&program),
        Recognition::Unrecognized(Unrecognized::Curvature { row: None })
    ));
}

/// Negative controls: every function in a direction its curvature does not support, a
/// nonconvex polynomial, a transcendental outside the vocabulary, an odd power across zero,
/// a nonlinear equality, a logarithm whose domain the box does not establish, and a
/// discrete column, are each refused with a reason, never recognized.
#[test]
fn curvature_refuses_unsupported_directions() {
    let registry = standard_registry().unwrap();
    let inf = f64::INFINITY;
    type Build = fn(&mut BodyBuilder<'_>, &[TypedValue]) -> TypedValue;
    /// A case: its label, body, variable box, row bounds and the expected refusal.
    type Case = (&'static str, Build, (f64, f64), (f64, f64), Unrecognized);
    let cases: Vec<Case> = vec![
        (
            "exp(x) ≥ 1",
            |b, v| unary(b, Function::Exp, &v[0]),
            (0.5, 2.0),
            (1.0, inf),
            Unrecognized::Curvature { row: Some(0) },
        ),
        (
            "x·y ≤ 1",
            |b, v| op(b, Binary::Mul, &v[0], &v[1]),
            (0.5, 2.0),
            (-inf, 1.0),
            Unrecognized::Curvature { row: Some(0) },
        ),
        (
            "sin(x) ≤ 0.5",
            |b, v| unary(b, Function::Sin, &v[0]),
            (0.5, 2.0),
            (-inf, 0.5),
            Unrecognized::Curvature { row: Some(0) },
        ),
        (
            "x³ ≤ 1 across zero",
            |b, v| power(b, &v[0], 3, 1),
            (-1.0, 1.0),
            (-inf, 1.0),
            Unrecognized::Curvature { row: Some(0) },
        ),
        (
            "exp(x) = 2",
            |b, v| unary(b, Function::Exp, &v[0]),
            (0.5, 2.0),
            (2.0, 2.0),
            Unrecognized::Curvature { row: Some(0) },
        ),
        (
            "log(x) ≥ 0 with x possibly nonpositive",
            |b, v| unary(b, Function::Log, &v[0]),
            (-1.0, 2.0),
            (0.0, inf),
            Unrecognized::Curvature { row: Some(0) },
        ),
    ];
    for (name, build, x_box, row, reason) in cases {
        let mut b = builder(&registry, 2);
        let v = inputs(&mut b, 2);
        let output = build(&mut b, &v);
        let objective = op(&mut b, Binary::Add, &v[0], &v[1]);
        let body = b.prepare(&[output, objective]).unwrap();
        let assembly = bounded(
            &registry,
            body,
            &[x_box, (0.5, 2.0)],
            &[],
            &[row],
            Some((1, ObjectiveSense::Minimize)),
        );
        let program = project(&assembly, &FactorableRequest::default());
        match recognize(&program) {
            Recognition::Unrecognized(r) => assert_eq!(r, reason, "{name}"),
            other => panic!("{name}: {other:?}"),
        }
    }
    // A discrete column is never a cone program.
    let mut b = builder(&registry, 1);
    let v = inputs(&mut b, 1);
    let ex = unary(&mut b, Function::Exp, &v[0]);
    let body = b.prepare(&[ex, v[0].clone()]).unwrap();
    let assembly = bounded(
        &registry,
        body,
        &[(0.0, 3.0)],
        &[ModelingVariableDomain::Integer],
        &[(-inf, 10.0)],
        Some((1, ObjectiveSense::Minimize)),
    );
    let program = project(&assembly, &FactorableRequest::default());
    assert!(matches!(
        recognize(&program),
        Recognition::Unrecognized(Unrecognized::Discrete)
    ));
}

/// An exhausted exact allowance is inconclusive, never recognized, and the curvature a
/// node query reports is the pass's.
#[test]
fn curvature_allowance_is_inconclusive() {
    let registry = standard_registry().unwrap();
    let mut b = builder(&registry, 2);
    let v = inputs(&mut b, 2);
    let x2 = power(&mut b, &v[0], 2, 1);
    let y2 = power(&mut b, &v[1], 2, 1);
    let xy = op(&mut b, Binary::Mul, &v[0], &v[1]);
    let sum = op(&mut b, Binary::Add, &x2, &y2);
    let quadratic = op(&mut b, Binary::Add, &sum, &xy);
    let objective = op(&mut b, Binary::Add, &v[0], &v[1]);
    let body = b.prepare(&[quadratic, objective]).unwrap();
    let assembly = bounded(
        &registry,
        body,
        &[(-1.0, 1.0), (-1.0, 1.0)],
        &[],
        &[(f64::NEG_INFINITY, 1.0)],
        Some((1, ObjectiveSense::Minimize)),
    );
    let program = project(&assembly, &FactorableRequest::default());
    let never = AtomicBool::new(false);
    assert!(matches!(
        curvature::recognize(&program, 1, &never).unwrap(),
        Recognition::Inconclusive
    ));
    assert!(matches!(
        curvature::fact(&program, 1, &never).unwrap().class,
        ConvexityClass::Inconclusive
    ));
    let root = program.rows[0].expression.unwrap();
    assert_eq!(
        curvature::node_curvature(&program, root, EXACT_OPERATIONS, &never).unwrap(),
        curvature::Curvature::Convex
    );
}
