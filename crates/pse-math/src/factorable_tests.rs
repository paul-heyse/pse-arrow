// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::{
    Function, MathError,
    assembly::*,
    binding::*,
    factorable::*,
    guarded::{Comparison, PreparedBody, Stage},
    jets::EvaluationLimits,
    library::{self, Optimization},
    presolve::ObligationStatus,
    typed::{Binary, BodyBuilder, BodyLimits, TypedValue},
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{
    AdmittedProvider, DerivativeOrder, EvaluationContext, Port, Provider, ProviderError,
    ProviderKey, ProviderRequest, ProviderSpec, ProviderValues,
};
use pse_quantity::{
    IndexSet, QuantityRegistry, QuantityTypeId,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, atomic::AtomicBool},
};
use symbolica::atom::Atom;

fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn neutral() -> QuantityTypeId {
    ids::quantity("neutral")
}
fn port(registry: &QuantityRegistry, n: u8) -> Port {
    Port {
        id: id(n),
        quantity: neutral(),
        unit: registry.quantity_type(neutral()).unwrap().canonical_unit,
    }
}
fn builder(registry: &QuantityRegistry, inputs: usize) -> BodyBuilder<'_> {
    BodyBuilder::new(
        crate::initialize().unwrap(),
        registry,
        &StandardInvariantChecker,
        inputs,
        BodyLimits::default(),
    )
    .unwrap()
}
fn number(value: Atom) -> TypedValue {
    TypedValue {
        effects: BTreeSet::new(),
        atom: value,
        quantity: neutral(),
        indices: IndexSet::new(),
        source: id(200),
    }
}
fn inputs(b: &mut BodyBuilder<'_>, n: usize) -> Vec<TypedValue> {
    (0..n)
        .map(|i| b.input(i, neutral(), IndexSet::new(), id(200)).unwrap())
        .collect()
}
fn op(b: &mut BodyBuilder<'_>, op: Binary, l: &TypedValue, r: &TypedValue) -> TypedValue {
    b.binary(op, l.clone(), r.clone(), None, id(201)).unwrap()
}
fn unary(b: &mut BodyBuilder<'_>, f: Function, v: &TypedValue) -> TypedValue {
    b.unary(f, v.clone(), id(202)).unwrap()
}

/// One instance of `body`, input `i` bound to variable `id(i + 1)` with box `boxes[i]`;
/// output `k` contributes to row `id(100 + k)`, or to the objective when listed.
fn case(
    registry: &QuantityRegistry,
    body: PreparedBody,
    boxes: &[(f64, f64)],
    objective: &[usize],
) -> Arc<CaseAssembly> {
    let key = ContentHash::from_bytes([7; 32]);
    let outputs = body.output_count();
    let variables = boxes
        .iter()
        .enumerate()
        .map(|(i, (lower, upper))| Variable {
            port: port(registry, u8::try_from(i + 1).unwrap()),
            fixed: false,
            domain: VariableDomain::Continuous,
            lower: Some(*lower),
            upper: Some(*upper),
        })
        .collect();
    let slots = (0..boxes.len())
        .map(|i| {
            let p = port(registry, u8::try_from(i + 1).unwrap());
            SlotBinding::new(&p, &p, registry).unwrap()
        })
        .collect();
    let contributions = (0..outputs)
        .map(|k| Contribution {
            output: k,
            target: if objective.contains(&k) {
                Target::Objective
            } else {
                Target::Row(id(100 + u8::try_from(k).unwrap()))
            },
            scale: 1.0,
        })
        .collect();
    let rows = (0..outputs)
        .filter(|k| !objective.contains(k))
        .map(|k| Row {
            id: id(100 + u8::try_from(k).unwrap()),
            quantity: neutral(),
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
        })
        .collect();
    let structure = Arc::new(
        CaseStructure::new(
            variables,
            vec![],
            vec![InstanceBinding {
                instance: id(9),
                body: key,
                slots,
                contributions,
            }],
            rows,
            (!objective.is_empty()).then_some(Objective {
                quantity: neutral(),
                sense: ObjectiveSense::Minimize,
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
fn project(assembly: &CaseAssembly, request: &FactorableRequest) -> FactorableProgram {
    assembly
        .factorable_program(
            &CaseValues::default(),
            request,
            100_000,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
}
/// Deterministic points in a box (splitmix64); no test dependency.
struct Points(u64);
impl Points {
    fn next(&mut self, (lower, upper): (f64, f64)) -> f64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        lower + (upper - lower) * ((z >> 11) as f64 / (1_u64 << 53) as f64)
    }
}
fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}
/// Compare every exact row with the compiled evaluator at the supplied points.
fn assert_rows_match(
    assembly: &Arc<CaseAssembly>,
    program: &FactorableProgram,
    points: &[Vec<f64>],
    providers: impl Fn() -> BTreeMap<ProviderKey, Box<dyn Provider>>,
    auxiliary: impl Fn(&[f64]) -> Vec<f64>,
) {
    for point in points {
        let mut worker = assembly.worker(providers(), Arc::new(AtomicBool::new(false)));
        let values = CaseValues {
            scalars: point
                .iter()
                .enumerate()
                .map(|(i, v)| (id(u8::try_from(i + 1).unwrap()), *v))
                .collect(),
        };
        let evaluated = worker.constraints(&values).unwrap();
        let columns: Vec<f64> = program
            .variables
            .iter()
            .map(|v| values.scalars[&v.id])
            .collect();
        let nodes = program.evaluate(&columns, &auxiliary(point)).unwrap();
        for (row, expected) in program.rows.iter().zip(&evaluated) {
            let projected = nodes[row.expression.unwrap()];
            assert!(
                close(projected, *expected),
                "row {} at {point:?}: projected {projected}, evaluator {expected}",
                row.id
            );
        }
    }
}
fn sample(boxes: &[(f64, f64)], n: usize, seed: u64) -> Vec<Vec<f64>> {
    let mut points = Points(seed);
    (0..n)
        .map(|_| boxes.iter().map(|b| points.next(*b)).collect())
        .collect()
}

#[test]
fn projection_exact_rows_match_evaluator() {
    let registry = standard_registry().unwrap();
    let mut b = builder(&registry, 2);
    let v = inputs(&mut b, 2);
    let (x, y) = (&v[0], &v[1]);
    let three = number(Atom::num(3));
    let two = number(Atom::num(2));
    // Polynomial with a shared product.
    let xy = op(&mut b, Binary::Mul, x, y);
    let xx = op(&mut b, Binary::Mul, x, x);
    let poly = op(&mut b, Binary::Add, &xy, &xx);
    let poly = op(&mut b, Binary::Sub, &poly, &three);
    // Transcendentals with their logarithm obligation.
    let e = unary(&mut b, Function::Exp, x);
    let l = unary(&mut b, Function::Log, y);
    let el = op(&mut b, Binary::Mul, &e, &l);
    let s = unary(&mut b, Function::Sin, x);
    let c = unary(&mut b, Function::Cos, y);
    let trig = op(&mut b, Binary::Add, &el, &s);
    let trig = op(&mut b, Binary::Sub, &trig, &c);
    // Square root and a quotient with its nonzero obligation.
    let root = unary(&mut b, Function::Sqrt, x);
    let quotient = op(&mut b, Binary::Div, &root, y);
    // Exact rational exponents.
    let third = number(Atom::num(1) / Atom::num(3));
    let cube_root = b
        .binary(
            Binary::Pow,
            x.clone(),
            third,
            Some(pse_quantity::Ratio::new(1, 3).unwrap()),
            id(203),
        )
        .unwrap();
    let three_halves = number(Atom::num(3) / Atom::num(2));
    let power = b
        .binary(
            Binary::Pow,
            y.clone(),
            three_halves,
            Some(pse_quantity::Ratio::new(3, 2).unwrap()),
            id(204),
        )
        .unwrap();
    let powers = op(&mut b, Binary::Mul, &cube_root, &power);
    // Absolute value, minimum and maximum through their admitted branches.
    let gap = op(&mut b, Binary::Sub, x, y);
    let magnitude = unary(&mut b, Function::Abs, &gap);
    let twice = op(&mut b, Binary::Mul, &two, y);
    let low = b.extremum(true, x.clone(), y.clone(), id(205)).unwrap();
    let high = b.extremum(false, x.clone(), twice, id(206)).unwrap();
    let extrema = op(&mut b, Binary::Add, &low, &high);
    let body = b
        .prepare(&[poly, trig, quotient, powers, magnitude, extrema])
        .unwrap();
    let boxes = [(0.5, 3.0), (0.5, 3.0)];
    let assembly = case(&registry, body, &boxes, &[]);
    let program = project(&assembly, &FactorableRequest::default());
    assert_eq!(program.row_counts().exact, 6);
    assert!(program.auxiliaries.is_empty());
    assert!(program.rows.iter().all(|r| r.fidelity == Fidelity::Exact));
    assert_eq!(program.fidelity(), Fidelity::Exact);
    assert!(program.missing_bounds().is_empty());
    assert!(program.nodes.iter().any(|n| matches!(
        n,
        Node::Pow {
            exponent: Constant::Rational(Rational {
                numerator: 1,
                denominator: 3
            }),
            ..
        }
    )));
    assert!(program.nodes.iter().any(|n| matches!(
        n,
        Node::Pow {
            exponent: Constant::Rational(Rational {
                numerator: 3,
                denominator: 2
            }),
            ..
        }
    )));
    // The logarithm, square root, quotient and powers keep closed obligations.
    assert!(
        program
            .obligations
            .iter()
            .all(|o| o.scope == ObligationScope::Unconditional)
    );
    assert!(program.obligations.iter().any(|o| matches!(
        o.kind,
        ObligationKind::Require(crate::guarded::Condition::Positive)
    ) && o.constraints.len() == 1
        && o.constraints[0].strict
        && o.constraints[0].lower == 0.0));
    assert_rows_match(
        &assembly,
        &program,
        &sample(&boxes, 64, 7),
        BTreeMap::new,
        |_| vec![],
    );
}

#[test]
fn min_max_exported_exactly() {
    let registry = standard_registry().unwrap();
    let mut b = builder(&registry, 2);
    let v = inputs(&mut b, 2);
    let (x, y) = (&v[0], &v[1]);
    let one = number(Atom::num(1));
    let low = b.extremum(true, x.clone(), y.clone(), id(205)).unwrap();
    let high = b.extremum(false, x.clone(), y.clone(), id(206)).unwrap();
    let gap = op(&mut b, Binary::Sub, x, y);
    let magnitude = unary(&mut b, Function::Abs, &gap);
    let inner = b.extremum(false, y.clone(), one, id(207)).unwrap();
    let nested = b.extremum(true, x.clone(), inner, id(208)).unwrap();
    let body = b.prepare(&[low, high, magnitude, nested]).unwrap();
    let boxes = [(-2.0, 2.0), (-2.0, 2.0)];
    let assembly = case(&registry, body, &boxes, &[]);
    let program = project(&assembly, &FactorableRequest::default());
    assert!(program.auxiliaries.is_empty(), "{:?}", program.auxiliaries);
    assert!(program.rows.iter().all(|r| r.fidelity == Fidelity::Exact));
    assert!(program.nodes.iter().any(|n| matches!(n, Node::Abs(_))));
    // Both sides of every switch, and the switch itself.
    let mut points = sample(&boxes, 48, 11);
    points.extend([
        vec![0.5, 0.5],
        vec![1.0, 1.5],
        vec![-1.0, 1.0],
        vec![1.5, 1.0],
    ]);
    assert_rows_match(&assembly, &program, &points, BTreeMap::new, |_| vec![]);
}

#[derive(Debug)]
struct Cube(ProviderSpec);
impl Provider for Cube {
    fn spec(&self) -> &ProviderSpec {
        &self.0
    }
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &ProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        request.validate(&self.0, context)?;
        let x = inputs[0];
        Ok(ProviderValues {
            values: request.outputs.iter().map(|_| x * x * x).collect(),
            jacobian: vec![],
            hessians: vec![],
        })
    }
}
#[derive(Debug)]
struct Root(ProviderSpec);
impl Provider for Root {
    fn spec(&self) -> &ProviderSpec {
        &self.0
    }
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &ProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        request.validate(&self.0, context)?;
        Ok(ProviderValues {
            values: request.outputs.iter().map(|_| inputs[0].sqrt()).collect(),
            jacobian: vec![],
            hessians: vec![],
        })
    }
}
fn registered(
    spec: &ProviderSpec,
    provider: Box<dyn Provider>,
) -> BTreeMap<ProviderKey, Box<dyn Provider>> {
    BTreeMap::from([(spec.key(), provider)])
}
fn spec(registry: &QuantityRegistry, n: u8) -> ProviderSpec {
    ProviderSpec {
        shapes: pse_kernels::ProviderShapes::default(),
        derivative_source: pse_kernels::DerivativeSource::Analytic,
        id: id(n),
        revision: ContentHash::from_bytes([n; 32]),
        data: ContentHash::from_bytes([2; 32]),
        inputs: vec![port(registry, n + 1)],
        outputs: vec![port(registry, n + 2)],
        derivatives: DerivativeOrder::Value,
        smoothness: DerivativeOrder::Value,
    }
}
/// `provider(x) + y`, optionally as the objective.
fn provider_case(
    registry: &QuantityRegistry,
    spec: &ProviderSpec,
    objective: bool,
) -> Arc<CaseAssembly> {
    let admitted = AdmittedProvider::new(spec.clone(), registry).unwrap();
    let mut b = builder(registry, 2);
    let v = inputs(&mut b, 2);
    let value = b
        .provider(&admitted, std::slice::from_ref(&v[0]), id(210))
        .unwrap()
        .remove(0);
    let out = op(&mut b, Binary::Add, &value, &v[1]);
    let body = b.prepare(&[out]).unwrap();
    case(
        registry,
        body,
        &[(0.5, 3.0), (-1.0, 1.0)],
        if objective { &[0] } else { &[] },
    )
}

#[test]
fn relaxed_rows_enclose_evaluator() {
    let registry = standard_registry().unwrap();
    let cube = spec(&registry, 40);
    let assembly = provider_case(&registry, &cube, false);
    let program = project(&assembly, &FactorableRequest::default());
    assert_eq!(program.auxiliaries.len(), 1);
    let auxiliary = &program.auxiliaries[0];
    assert!(matches!(
        auxiliary.role,
        AuxiliaryRole::Opaque(Opacity::Provider { output: 0, .. })
    ));
    assert_eq!(program.rows[0].fidelity, Fidelity::Relaxed);
    assert_eq!(program.fidelity(), Fidelity::Relaxed);
    // A free auxiliary is reported for spatial branch-and-bound.
    assert!(program.missing_bounds().contains(&MissingBound {
        owner: BoundOwner::Auxiliary(0),
        lower: true,
        upper: true,
    }));
    let providers = || registered(&cube, Box::new(Cube(cube.clone())));
    let points = sample(&[(0.5, 3.0), (-1.0, 1.0)], 16, 3);
    // The relaxation admits the evaluator's value: the auxiliary at the provider output.
    assert_rows_match(&assembly, &program, &points, providers, |p| {
        vec![p[0] * p[0] * p[0]]
    });
    // Negative control: any other auxiliary value is also admitted, so the row is not exact.
    let row = program.rows[0].expression.unwrap();
    let shifted = program.evaluate(&[2.0, 0.0], &[9.0]).unwrap()[row];
    assert!(!close(shifted, 8.0));
    // An enforced envelope bounds the auxiliary; the row stays a relaxation.
    let envelope = FactorableRequest {
        envelopes: BTreeMap::from([(cube.key(), vec![(0.0, 27.0)])]),
        ..Default::default()
    };
    let bounded = project(&assembly, &envelope);
    assert_eq!(
        (bounded.auxiliaries[0].lower, bounded.auxiliaries[0].upper),
        (0.0, 27.0)
    );
    assert_eq!(bounded.rows[0].fidelity, Fidelity::Relaxed);
    assert!(bounded.missing_bounds().is_empty());
}

#[test]
fn unavailable_objective_reported() {
    let registry = standard_registry().unwrap();
    let cube = spec(&registry, 50);
    let assembly = provider_case(&registry, &cube, true);
    let program = project(&assembly, &FactorableRequest::default());
    let objective = program.objective.as_ref().unwrap();
    assert!(objective.expression.is_some());
    assert_eq!(objective.fidelity, Fidelity::Unavailable);
    assert_eq!(program.fidelity(), Fidelity::Unavailable);
    // An enforced envelope makes the objective a bounded relaxation.
    let request = FactorableRequest {
        envelopes: BTreeMap::from([(cube.key(), vec![(0.125, 27.0)])]),
        ..Default::default()
    };
    let bounded = project(&assembly, &request);
    assert_eq!(
        bounded.objective.as_ref().unwrap().fidelity,
        Fidelity::Relaxed
    );
    // A node budget that cannot hold the objective leaves it unavailable, never partial.
    let limited = assembly
        .factorable_program(
            &CaseValues::default(),
            &FactorableRequest::default(),
            2,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(limited.incomplete, vec![id(9)]);
    assert_eq!(
        limited.objective.as_ref().unwrap().fidelity,
        Fidelity::Unavailable
    );
    assert!(limited.objective.as_ref().unwrap().expression.is_none());
}

#[test]
fn implicit_residual_exported_exactly() {
    let registry = standard_registry().unwrap();
    let root = spec(&registry, 60);
    // Residual y*y - p = 0 over formals (y, p), unknown first.
    let mut b = builder(&registry, 2);
    let v = inputs(&mut b, 2);
    let square = op(&mut b, Binary::Mul, &v[0], &v[0]);
    let residual = op(&mut b, Binary::Sub, &square, &v[1]);
    let residual = Arc::new(b.prepare(&[residual]).unwrap());
    // Declared bounds over the same formals: [0, p + 1].
    let mut b = builder(&registry, 2);
    let v = inputs(&mut b, 2);
    let one = number(Atom::num(1));
    let upper = op(&mut b, Binary::Add, &v[1], &one);
    let zero = number(Atom::num(0));
    let bounds = Arc::new(b.prepare(&[zero, upper]).unwrap());
    let definition = ImplicitDefinition {
        residual,
        unknowns: vec![(f64::NEG_INFINITY, 10.0)],
        bounds: Some(ImplicitBounds {
            body: bounds,
            lower: vec![Some(0)],
            upper: vec![Some(1)],
        }),
    };
    let request = FactorableRequest {
        implicit: BTreeMap::from([(root.key(), definition)]),
        ..Default::default()
    };
    let assembly = provider_case(&registry, &root, false);
    let program = project(&assembly, &request);
    assert_eq!(program.implicit.len(), 1);
    let block = &program.implicit[0];
    assert_eq!(block.fidelity, Fidelity::Exact);
    assert_eq!(block.residuals.len(), 1);
    assert_eq!(block.bounds.len(), 1);
    let unknown = &program.auxiliaries[block.unknowns[0]];
    assert!(matches!(
        unknown.role,
        AuxiliaryRole::Implicit { unknown: 0, .. }
    ));
    assert_eq!((unknown.lower, unknown.upper), (0.0, 10.0));
    assert_eq!(unknown.fidelity, Fidelity::Exact);
    assert_eq!(program.rows[0].fidelity, Fidelity::Exact);
    assert_eq!(program.fidelity(), Fidelity::Exact);
    let providers = || registered(&root, Box::new(Root(root.clone())));
    let points = sample(&[(0.5, 3.0), (-1.0, 1.0)], 16, 5);
    assert_rows_match(&assembly, &program, &points, providers, |p| {
        vec![p[0].sqrt()]
    });
    // The root satisfies the exported residual and its input-dependent bound.
    let nodes = program.evaluate(&[4.0, 0.0], &[2.0]).unwrap();
    assert_eq!(nodes[block.residuals[0]], 0.0);
    assert!(nodes[block.bounds[0].expression] <= block.bounds[0].upper);
    // Without the definition the same provider output is only a relaxation.
    let relaxed = project(&assembly, &FactorableRequest::default());
    assert_eq!(relaxed.rows[0].fidelity, Fidelity::Relaxed);
    assert!(relaxed.implicit.is_empty());
}

type Region<'r> = &'r dyn Fn(&mut BodyBuilder<'_>) -> Result<TypedValue, MathError>;
/// `if left cmp right then region else otherwise`, as the compiler lowers a conditional.
fn select(
    b: &mut BodyBuilder<'_>,
    comparison: Comparison,
    left: &TypedValue,
    right: &TypedValue,
    then: Region<'_>,
    otherwise: &TypedValue,
) -> Result<TypedValue, MathError> {
    let guard = b.compare(comparison, left, right, id(220))?;
    let otherwise = otherwise.clone();
    b.conditional(guard, |b| then(b), move |_| Ok(otherwise), id(221))
}
fn sum(b: &mut BodyBuilder<'_>, terms: Vec<TypedValue>) -> TypedValue {
    let mut terms = terms.into_iter();
    let first = terms.next().unwrap();
    terms.fold(first, |a, c| op(b, Binary::Add, &a, &c))
}
/// A packing-fraction-like moment: r * sum(k n_k) / sum(n_k) / 1000.
fn moment(
    b: &mut BodyBuilder<'_>,
    n: &[TypedValue],
    r: &TypedValue,
) -> Result<TypedValue, MathError> {
    let mut weighted = vec![];
    for (k, n) in (1_i64..).zip(n) {
        weighted.push(op(b, Binary::Mul, &number(Atom::num(k)), n));
    }
    let weighted = sum(b, weighted);
    let total = sum(b, n.to_vec());
    let ratio = op(b, Binary::Div, &weighted, &total);
    let scaled = op(b, Binary::Mul, &ratio, r);
    Ok(op(
        b,
        Binary::Mul,
        &scaled,
        &number(Atom::num(1) / Atom::num(1000)),
    ))
}
/// The PC-SAFT pattern: a `valid(...)` guard lowered as a domain predicate over nested
/// conditionals (a per-species `if n > 0 then 0 else 1` summed and compared with zero),
/// wrapped around a large factorable body whose flattened form exceeds the flattening
/// bound. The census attributed these rows to the guard; the cause is the flattening bound.
#[test]
fn pcsaft_valid_guard_projects_exactly() {
    let registry = standard_registry().unwrap();
    let mut b = builder(&registry, 5);
    let v = inputs(&mut b, 5);
    let (t, r) = (v[0].clone(), v[1].clone());
    let n = v[2..].to_vec();
    let zero = number(Atom::num(0));
    let one = number(Atom::num(1));
    let hundredth = number(Atom::num(1) / Atom::num(100));
    // valid(t > 0 and r > 0 and sum(j | if n[j] > 0 then 0 else 1) == 0 and moment < 1)
    let predicate = b
        .domain(id(230), |b| {
            select(
                b,
                Comparison::Lt,
                &zero,
                &t,
                &|b| {
                    select(
                        b,
                        Comparison::Lt,
                        &zero,
                        &r,
                        &|b| {
                            let mut flags = vec![];
                            for n in &n {
                                flags.push(select(
                                    b,
                                    Comparison::Lt,
                                    &zero,
                                    n,
                                    &|_| Ok(zero.clone()),
                                    &one,
                                )?);
                            }
                            let count = sum(b, flags);
                            select(
                                b,
                                Comparison::Eq,
                                &count,
                                &zero,
                                &|b| {
                                    let m = moment(b, &n, &r)?;
                                    select(b, Comparison::Lt, &m, &one, &|_| Ok(one.clone()), &zero)
                                },
                                &zero,
                            )
                        },
                        &zero,
                    )
                },
                &zero,
            )
        })
        .unwrap();
    // A shared chain: each level reads the previous one twice, so its flattened tree
    // doubles per level while the stage program grows linearly.
    let mut h = op(&mut b, Binary::Mul, &t, &hundredth);
    for _ in 0..16 {
        let s = unary(&mut b, Function::Sin, &h);
        let c = unary(&mut b, Function::Cos, &h);
        let a = op(&mut b, Binary::Mul, &s, &r);
        let level = op(&mut b, Binary::Add, &a, &c);
        h = b.bind(level).unwrap();
    }
    let output = b.with_assumption(h, &predicate);
    let body = b.prepare(&[output]).unwrap();
    // The flattened expression is absent: this, not the guard, made the census rows opaque.
    assert!(body.expression(0).is_none());
    let boxes = [
        (200.0, 400.0),
        (1.0, 10.0),
        (0.1, 1.0),
        (0.1, 1.0),
        (0.1, 1.0),
    ];
    let assembly = case(&registry, body, &boxes, &[]);
    let program = project(&assembly, &FactorableRequest::default());
    assert_eq!(program.rows[0].fidelity, Fidelity::Exact);
    assert!(program.auxiliaries.is_empty());
    // The validity guard is one unconditional obligation, represented by its closed
    // conjunction: t >= 0, r >= 0, n_j >= 0 (all strict) and moment <= 1 (strict).
    let domain: Vec<_> = program
        .obligations
        .iter()
        .filter(|o| o.kind == ObligationKind::Domain)
        .collect();
    assert_eq!(domain.len(), 1);
    let guard = domain[0];
    assert_eq!(guard.scope, ObligationScope::Unconditional);
    assert!(guard.represented);
    assert_eq!(guard.fidelity, Fidelity::Exact);
    assert_eq!(guard.constraints.len(), 6);
    assert!(guard.constraints.iter().all(|c| c.strict));
    assert_eq!(
        guard
            .constraints
            .iter()
            .filter(|c| c.lower == 0.0 && c.upper == f64::INFINITY)
            .count(),
        5
    );
    assert!(
        guard
            .constraints
            .iter()
            .any(|c| c.lower == f64::NEG_INFINITY && c.upper == 1.0)
    );
    // The guard is enforced by the evaluator, and the export matches it where it holds.
    assert_rows_match(
        &assembly,
        &program,
        &sample(&boxes, 16, 13),
        BTreeMap::new,
        |_| vec![],
    );
    let mut worker = assembly.worker(BTreeMap::new(), Arc::new(AtomicBool::new(false)));
    let outside = CaseValues {
        scalars: BTreeMap::from([
            (id(1), 300.0),
            (id(2), 5.0),
            (id(3), 0.5),
            (id(4), -0.5),
            (id(5), 0.5),
        ]),
    };
    assert!(worker.constraints(&outside).is_err());
    // FBBT presolve now has a complete tape and discharges the guard over the box.
    let facts = assembly
        .presolve_facts(
            &CaseValues::default(),
            100_000,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(facts.complete, vec![true]);
    assert_eq!(facts.obligations[&id(9)], ObligationStatus::Discharged);
}

#[test]
fn disjunctive_branch_policy_is_a_typed_refusal() {
    crate::initialize().unwrap();
    let registry = standard_registry().unwrap();
    let x = library::formal(0).unwrap();
    // if x < 1 then x^2 else 2x - 1: continuous with a proven first derivative, but not an
    // absolute-value identity.
    let mut body = PreparedBody::new(
        1,
        4,
        vec![3],
        vec![
            Stage::Block {
                expressions: vec![x.clone(), Atom::num(1)],
                outputs: vec![1, 2],
                source: id(1),
            },
            Stage::Branch {
                continuity: DerivativeOrder::First,
                comparison: Comparison::Lt,
                left: 1,
                right: 2,
                then: vec![Stage::Block {
                    expressions: vec![&x * &x],
                    outputs: vec![3],
                    source: id(1),
                }],
                otherwise: vec![Stage::Block {
                    expressions: vec![Atom::num(2) * &x - Atom::num(1)],
                    outputs: vec![3],
                    source: id(1),
                }],
            },
        ],
        DerivativeOrder::Second,
    )
    .unwrap();
    body.set_quantities(vec![Some(neutral())], vec![neutral()]);
    let assembly = case(&registry, body, &[(0.0, 2.0)], &[]);
    let relaxed = project(&assembly, &FactorableRequest::default());
    assert_eq!(relaxed.rows[0].fidelity, Fidelity::Relaxed);
    assert!(matches!(
        relaxed.auxiliaries[0].role,
        AuxiliaryRole::Opaque(Opacity::Branch)
    ));
    let refused = assembly.factorable_program(
        &CaseValues::default(),
        &FactorableRequest {
            branches: BranchPolicy::Disjunctive,
            ..Default::default()
        },
        100_000,
        &Arc::new(AtomicBool::new(false)),
    );
    assert!(matches!(refused, Err(MathError::Unsupported(_))));
}

#[test]
fn fbbt_tapes_follow_the_shared_stage_projection() {
    use pounce_nlp::expression_provider::FbbtOp as Op;
    let registry = standard_registry().unwrap();
    let mut b = builder(&registry, 1);
    let x = inputs(&mut b, 1).remove(0);
    let e = unary(&mut b, Function::Exp, &x);
    let l = unary(&mut b, Function::Log, &x);
    let s = unary(&mut b, Function::Sin, &x);
    let c = unary(&mut b, Function::Cos, &x);
    let a = unary(&mut b, Function::Abs, &x);
    let shared = op(&mut b, Binary::Mul, &e, &l);
    let shared = b.bind(shared).unwrap();
    let twice = op(&mut b, Binary::Add, &shared, &shared);
    let body = b.prepare(&[e, l, s, c, a, twice]).unwrap();
    let assembly = case(&registry, body, &[(1.0, 2.0)], &[]);
    let facts = assembly
        .presolve_facts(
            &CaseValues::default(),
            1000,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    assert!(facts.complete.iter().all(|v| *v));
    for (tape, expected) in
        facts
            .tapes
            .iter()
            .zip([Op::Exp(0), Op::Ln(0), Op::Sin(0), Op::Cos(0), Op::Abs(0)])
    {
        assert!(tape.first_invalid_slot().is_none());
        assert!(
            tape.ops
                .iter()
                .any(|op| std::mem::discriminant(op) == std::mem::discriminant(&expected)),
            "{tape:?}"
        );
        let range = pounce_presolve::fbbt::forward_result(
            &pounce_presolve::fbbt::forward_pass(tape, &[1.0], &[2.0]).unwrap(),
        );
        assert!(range.lo.is_finite() && range.hi.is_finite());
    }
    // The shared product is emitted once in the row that reads it twice.
    let last = facts.tapes.last().unwrap();
    assert_eq!(
        last.ops
            .iter()
            .filter(|op| matches!(op, Op::Exp(_)))
            .count(),
        1
    );
    assert_eq!(
        facts.signs[&id(1)],
        crate::presolve::GuardSign {
            positive: true,
            strict: true
        }
    );
}
