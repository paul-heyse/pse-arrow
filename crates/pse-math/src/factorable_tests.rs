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
use pse_model::generated::enums::ModelingVariableDomain;
use pse_quantity::{
    IndexSet, QuantityRegistry, QuantityTypeId,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, atomic::AtomicBool},
};
use symbolica::atom::{Atom, AtomCore};

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
        quantity: pse_quantity::ResolvedPhysicalContract::named(
            neutral(),
            IndexSet::new(),
            &standard_registry().unwrap(),
        )
        .unwrap(),
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
            domain: ModelingVariableDomain::Continuous,
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
                Target::PRIMARY
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
                checked_members: Default::default(),
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
            exponent: Constant::Rational(r), .. } if r == &Rational::new(1, 3)
    )));
    assert!(program.nodes.iter().any(|n| matches!(
        n,
        Node::Pow {
            exponent: Constant::Rational(r), .. } if r == &Rational::new(3, 2)
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
    // Binary64 coefficients, as authored decimal literals carry.
    let scaled = op(&mut b, Binary::Mul, &number(Atom::num(-562.2)), x);
    let shifted = op(&mut b, Binary::Add, y, &number(Atom::num(0.1)));
    let decimal = b.extremum(false, scaled, shifted, id(209)).unwrap();
    let body = b.prepare(&[low, high, magnitude, nested, decimal]).unwrap();
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
}

fn assert_projection_budget_refusal(objective: bool) {
    let registry = standard_registry().unwrap();
    let mut b = builder(&registry, 1);
    let x = inputs(&mut b, 1).remove(0);
    let square = op(&mut b, Binary::Mul, &x, &x);
    let expression = op(&mut b, Binary::Add, &square, &x);
    let assembly = case(
        &registry,
        b.prepare(&[expression]).unwrap(),
        &[(0.0, 3.0)],
        if objective { &[0] } else { &[] },
    );
    let values = CaseValues::default();
    let request = FactorableRequest::default();
    let cancel = Arc::new(AtomicBool::new(false));
    let complete = assembly
        .factorable_program(&values, &request, 100, &cancel)
        .unwrap();
    assert_eq!(complete.fidelity(), Fidelity::Exact);
    let root = if objective {
        assert!(complete.rows.is_empty());
        complete.objective.as_ref().unwrap().expression.unwrap()
    } else {
        assert!(complete.objective.is_none());
        complete.rows[0].expression.unwrap()
    };
    assert_eq!(complete.evaluate(&[2.0], &[]).unwrap()[root], 6.0);
    assert!(matches!(
        assembly.factorable_program(&values, &request, 2, &cancel),
        Err(FactorableError::Math(MathError::Limit(
            "factorable projection extent"
        )))
    ));
    // Presolve must propagate the same extraction failure before it can publish facts.
    assert!(matches!(
        assembly.presolve_domain_facts(&values, 2, &cancel),
        Err(MathError::Limit("factorable projection extent"))
    ));
    assert!(
        assembly
            .presolve_domain_facts(&values, 100, &cancel)
            .is_ok()
    );
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(matches!(
        assembly.factorable_program(&values, &request, 100, &cancel),
        Err(FactorableError::Math(MathError::Cancelled))
    ));
}

#[test]
fn mandatory_row_projection_budget_is_a_resource_refusal() {
    assert_projection_budget_refusal(false);
}

#[test]
fn selected_objective_projection_budget_is_a_resource_refusal() {
    assert_projection_budget_refusal(true);
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
        selection: SelectedGraph::RestrictedSquareRoot {
            positive: true,
            strict: false,
        },
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
    assert_eq!(block.bounds.len(), 2);
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
    // An operational selection without graph equivalence exposes both residual roots;
    // selected evaluator values remain admissible, but exact-only consumers refuse it.
    let mut operational = request.clone();
    operational.implicit.get_mut(&root.key()).unwrap().selection = SelectedGraph::Unestablished {
        meaning: "operational:native.kinsol.v1".into(),
    };
    operational.implicit.get_mut(&root.key()).unwrap().bounds = None;
    operational.implicit.get_mut(&root.key()).unwrap().unknowns = vec![(-10.0, 10.0)];
    let graph = project(&assembly, &operational);
    assert_eq!(graph.implicit[0].fidelity, Fidelity::Relaxed);
    assert_eq!(graph.rows[0].fidelity, Fidelity::Relaxed);
    assert_ne!(graph.key, program.key);
    let nodes = graph.evaluate(&[4.0, 0.0], &[-2.0]).unwrap();
    assert_eq!(nodes[graph.implicit[0].residuals[0]], 0.0);
    assert_rows_match(
        &assembly,
        &graph,
        &[vec![1.0, 0.0]],
        || registered(&root, Box::new(Root(root.clone()))),
        |_| vec![1.0],
    );
    let wrong = graph.evaluate(&[1.0, 0.0], &[-1.0]).unwrap();
    let selected = program.evaluate(&[1.0, 0.0], &[1.0]).unwrap();
    assert_ne!(
        wrong[graph.rows[0].expression.unwrap()],
        selected[program.rows[0].expression.unwrap()]
    );
    operational.require_exact = true;
    assert!(matches!(
        assembly.factorable_program(
            &CaseValues::default(),
            &operational,
            10000,
            &Arc::new(AtomicBool::new(false))
        ),
        Err(FactorableError::ExactRequired {
            fidelity: Fidelity::Relaxed
        })
    ));
    // A caller cannot fabricate an affine equivalence witness for this square relation.
    let mut fabricated = request.clone();
    fabricated.implicit.get_mut(&root.key()).unwrap().selection =
        SelectedGraph::NondegenerateAffine {
            sign: None,
            strict: false,
        };
    assert_eq!(
        project(&assembly, &fabricated).fidelity(),
        Fidelity::Relaxed
    );
    fabricated.require_exact = true;
    assert!(matches!(
        assembly.factorable_program(
            &CaseValues::default(),
            &fabricated,
            10000,
            &Arc::new(AtomicBool::new(false))
        ),
        Err(FactorableError::ExactRequired { .. })
    ));
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
        .domain(form_lineage(id(230)), |b| {
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
    assert!(matches!(
        refused,
        Err(FactorableError::DisjunctiveBranch { instance }) if instance == id(9)
    ));
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
        .presolve_domain_facts(
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
#[path = "curvature_tests.rs"]
mod curvature_tests;

/// The lineage of a form-layer predicate stated by `source`, reading no parameter set.
fn form_lineage(source: SemanticId) -> Arc<pse_model::diagnostic::ValidityLineage> {
    Arc::new(pse_model::diagnostic::ValidityLineage {
        layer: pse_model::generated::enums::ModelingValidityLayer::Form,
        source,
        form: Some(source),
        sets: Vec::new(),
        variables: Vec::new(),
        members: Vec::new(),
    })
}

#[test]
fn arbitrary_precision_rational_coefficients_survive_factorable_transport() {
    let registry = standard_registry().unwrap();
    let huge = Rational::from(i64::MAX).pow(4);
    let ratio = &huge / &Rational::new(3, 7);
    let mut b = builder(&registry, 1);
    let x = inputs(&mut b, 1).remove(0);
    let coefficient = number(Atom::num(ratio.clone()));
    let value = op(&mut b, Binary::Mul, &coefficient, &x);
    let body = b.prepare(&[value]).unwrap();
    let assembly = case(&registry, body, &[(0.0, 1.0)], &[]);
    let program = project(&assembly, &FactorableRequest::default());
    assert!(program.nodes.iter().any(|node| matches!(node,
        Node::Const(Constant::Rational(q)) if q == &ratio)));
    assert!(
        !program
            .nodes
            .iter()
            .any(|node| matches!(node, Node::Const(Constant::Float(_))))
    );
    assert_eq!(program.fidelity(), Fidelity::Exact);
}

fn isolation_body(
    inputs: usize,
    slots: usize,
    outputs: Vec<usize>,
    stages: Vec<Stage>,
) -> PreparedBody {
    crate::initialize().unwrap();
    PreparedBody::new(inputs, slots, outputs, stages, DerivativeOrder::Second).unwrap()
}
fn isolation_true(inputs: usize) -> PreparedBody {
    isolation_body(
        inputs,
        inputs + 1,
        vec![inputs],
        vec![Stage::Block {
            expressions: vec![Atom::num(1)],
            outputs: vec![inputs],
            source: id(240),
        }],
    )
}
fn isolation_criterion(inputs: usize) -> PreparedBody {
    isolation_body(
        inputs,
        inputs + 2,
        vec![inputs, inputs + 1],
        vec![Stage::Block {
            expressions: vec![Atom::num(0), Atom::num(0)],
            outputs: vec![inputs, inputs + 1],
            source: id(246),
        }],
    )
}
fn isolation_project(
    residual: &PreparedBody,
    eligibility: &PreparedBody,
) -> Option<RootIsolationProgram> {
    root_isolation_program(
        id(240),
        residual,
        eligibility,
        &isolation_criterion(residual.input_count()),
        &Arc::new(AtomicBool::new(false)),
        10_000,
    )
    .unwrap()
}

#[test]
fn root_isolation_predicate_conjunction_preserves_original_coordinates_and_exact_bounds() {
    let registry = standard_registry().unwrap();
    let residual = isolation_body(2, 2, vec![0, 1], vec![]);
    let mut b = builder(&registry, 2);
    let v = inputs(&mut b, 2);
    let zero = number(Atom::num(0));
    let one = number(Atom::num(1));
    let third = number(Atom::num(1) / Atom::num(3));
    let predicate = select(
        &mut b,
        Comparison::Lt,
        &zero,
        &v[0],
        &|b| {
            select(
                b,
                Comparison::Le,
                &v[1],
                &third,
                &|_| Ok(one.clone()),
                &zero,
            )
        },
        &zero,
    )
    .unwrap();
    let eligibility = b.prepare(&[predicate]).unwrap();
    let p = isolation_project(&residual, &eligibility).unwrap();
    assert_eq!(p.inputs, 2);
    assert!(matches!(p.nodes[p.residuals[0]], Node::Var(0)));
    assert!(matches!(p.nodes[p.residuals[1]], Node::Var(1)));
    assert_eq!(p.eligibility.len(), 2);
    assert_eq!(p.eligibility.iter().filter(|c| c.strict).count(), 1);
    assert!(
        p.eligibility
            .iter()
            .all(|c| c.upper == 0.0 && c.lower == f64::NEG_INFINITY)
    );
    assert!(
        p.nodes
            .iter()
            .any(|n| matches!(n, Node::Const(Constant::Rational(r)) if r == &Rational::new(1, 3)))
    );
    assert!(
        !p.nodes
            .iter()
            .any(|n| matches!(n, Node::Aux(_) | Node::Abs(_)))
    );
}

#[test]
fn root_isolation_retains_strict_nonzero_and_derivative_order_guards() {
    use crate::guarded::Condition;
    let require = |condition, order, argument| Stage::Require {
        argument,
        condition,
        order,
        source: id(241),
        lineage: None,
    };
    let residual = isolation_body(
        2,
        2,
        vec![0],
        vec![
            require(Condition::Nonnegative, DerivativeOrder::Value, 0),
            require(Condition::Nonzero, DerivativeOrder::Value, 1),
            require(Condition::Positive, DerivativeOrder::First, 0),
            require(Condition::Nonzero, DerivativeOrder::Second, 1),
        ],
    );
    let p = isolation_project(&residual, &isolation_true(2)).unwrap();
    assert_eq!(p.obligations.len(), 2);
    assert_eq!(p.derivative_obligations.len(), 2);
    let nonnegative = &p.obligations[0];
    assert_eq!(
        nonnegative.kind,
        ObligationKind::Require(Condition::Nonnegative)
    );
    assert!(!nonnegative.constraints[0].strict);
    let nonzero = &p.obligations[1];
    assert_eq!(nonzero.kind, ObligationKind::Require(Condition::Nonzero));
    assert!(nonzero.argument.is_some());
    assert!(nonzero.constraints.is_empty());
    assert!(nonzero.represented);
    assert_eq!(p.derivative_obligations[0].0, DerivativeOrder::First);
    assert!(p.derivative_obligations[0].1.constraints[0].strict);
    assert_eq!(p.derivative_obligations[1].0, DerivativeOrder::Second);
    assert_eq!(p.derivative_obligations[1].1.argument, nonzero.argument);
    assert!(
        p.obligations
            .iter()
            .chain(p.derivative_obligations.iter().map(|(_, o)| o))
            .all(|o| o.fidelity == Fidelity::Exact && o.scope == ObligationScope::Unconditional)
    );
}

#[test]
fn root_isolation_refuses_conditional_guards_even_when_the_result_is_constant() {
    use crate::guarded::Condition;
    let guarded = vec![
        Stage::Require {
            argument: 0,
            condition: Condition::Nonzero,
            order: DerivativeOrder::Value,
            source: id(242),
            lineage: None,
        },
        Stage::Block {
            expressions: vec![Atom::num(1)],
            outputs: vec![2],
            source: id(242),
        },
    ];
    let eligibility = isolation_body(
        1,
        3,
        vec![2],
        vec![
            Stage::Block {
                expressions: vec![Atom::num(0)],
                outputs: vec![1],
                source: id(242),
            },
            Stage::Branch {
                continuity: DerivativeOrder::Value,
                comparison: Comparison::Lt,
                left: 1,
                right: 0,
                then: guarded,
                otherwise: vec![Stage::Block {
                    expressions: vec![Atom::num(1)],
                    outputs: vec![2],
                    source: id(242),
                }],
            },
        ],
    );
    let residual = isolation_body(1, 1, vec![0], vec![]);
    assert!(isolation_project(&residual, &eligibility).is_none());
}

#[test]
fn root_isolation_refuses_nonzero_eligibility_disjunction() {
    let registry = standard_registry().unwrap();
    let mut b = builder(&registry, 1);
    let v = inputs(&mut b, 1);
    let zero = number(Atom::num(0));
    let one = number(Atom::num(1));
    let predicate = select(
        &mut b,
        Comparison::Ne,
        &v[0],
        &zero,
        &|_| Ok(one.clone()),
        &zero,
    )
    .unwrap();
    let eligibility = b.prepare(&[predicate]).unwrap();
    let residual = isolation_body(1, 1, vec![0], vec![]);
    assert!(isolation_project(&residual, &eligibility).is_none());
}

#[test]
fn root_isolation_refuses_provider_opacity_and_nonsmooth_residuals() {
    let registry = standard_registry().unwrap();
    let provider = AdmittedProvider::new(spec(&registry, 40), &registry).unwrap();
    let mut b = builder(&registry, 1);
    let v = inputs(&mut b, 1);
    let value = b.provider(&provider, &v, id(243)).unwrap().remove(0);
    let residual = b.prepare(&[value]).unwrap();
    assert!(isolation_project(&residual, &isolation_true(1)).is_none());
    let mut b = builder(&registry, 1);
    let v = inputs(&mut b, 1);
    let value = unary(&mut b, Function::Abs, &v[0]);
    let residual = b.prepare(&[value]).unwrap();
    assert!(isolation_project(&residual, &isolation_true(1)).is_none());
}

#[test]
fn root_isolation_preserves_stored_binary_coefficients_and_exact_constant_comparisons() {
    let registry = standard_registry().unwrap();
    let binary = 0.1_f64;
    let exact = Rational::try_from(binary).unwrap();
    let mut b = builder(&registry, 1);
    let x = inputs(&mut b, 1).remove(0);
    let residual = op(&mut b, Binary::Mul, &number(Atom::num(binary)), &x);
    let residual = b.prepare(&[residual]).unwrap();
    let p = isolation_project(&residual, &isolation_true(1)).unwrap();
    assert!(
        p.nodes
            .iter()
            .any(|n| matches!(n, Node::Const(Constant::Rational(r)) if r == &exact))
    );
    assert!(
        !p.nodes
            .iter()
            .any(|n| matches!(n, Node::Const(Constant::Float(_))))
    );
    // A positive rational below binary64's subnormal range remains positive.
    let tiny = Rational::one() / Rational::from(2).pow(1200);
    let eligibility = isolation_body(
        1,
        2,
        vec![1],
        vec![Stage::Block {
            expressions: vec![Atom::num(tiny)],
            outputs: vec![1],
            source: id(244),
        }],
    );
    assert!(
        isolation_project(&residual, &eligibility)
            .unwrap()
            .eligibility
            .is_empty()
    );
}

#[test]
fn root_isolation_projection_budget_and_cancellation_are_errors() {
    let residual = isolation_body(2, 2, vec![0, 1], vec![]);
    let eligibility = isolation_true(2);
    for limit in [0, 1] {
        assert!(matches!(
            root_isolation_program(
                id(240),
                &residual,
                &eligibility,
                &isolation_criterion(2),
                &Arc::new(AtomicBool::new(false)),
                limit
            ),
            Err(FactorableError::Math(MathError::Limit(_)))
        ));
    }
    assert!(matches!(
        root_isolation_program(
            id(240),
            &residual,
            &eligibility,
            &isolation_criterion(2),
            &Arc::new(AtomicBool::new(true)),
            10_000
        ),
        Err(FactorableError::Math(MathError::Cancelled))
    ));
}

#[test]
fn root_isolation_refuses_unrepresented_domain_obligations() {
    let registry = standard_registry().unwrap();
    let mut b = builder(&registry, 1);
    let x = inputs(&mut b, 1).remove(0);
    let zero = number(Atom::num(0));
    let one = number(Atom::num(1));
    let predicate = b
        .domain(form_lineage(id(245)), |b| {
            select(b, Comparison::Ne, &x, &zero, &|_| Ok(one.clone()), &zero)
        })
        .unwrap();
    let output = b.with_assumption(x, &predicate);
    let residual = b.prepare(&[output]).unwrap();
    assert!(isolation_project(&residual, &isolation_true(1)).is_none());
}

#[test]
fn root_isolation_projects_shared_dag_without_a_flattened_expression() {
    let registry = standard_registry().unwrap();
    let mut b = builder(&registry, 2);
    let v = inputs(&mut b, 2);
    let mut value = v[0].clone();
    for _ in 0..16 {
        let s = unary(&mut b, Function::Sin, &value);
        let c = unary(&mut b, Function::Cos, &value);
        let weighted = op(&mut b, Binary::Mul, &s, &v[1]);
        let next = op(&mut b, Binary::Add, &weighted, &c);
        value = b.bind(next).unwrap();
    }
    let residual = b.prepare(&[value]).unwrap();
    assert!(residual.expression(0).is_none());
    let p = isolation_project(&residual, &isolation_true(2)).unwrap();
    assert!(p.nodes.len() < 128);
    assert_eq!(p.residuals.len(), 1);
}

#[test]
fn root_isolation_criterion_preserves_score_tolerance_order_and_exact_shared_nodes() {
    let residual = isolation_body(2, 2, vec![1, 0], vec![]);
    let tiny = Rational::one() / Rational::from(2).pow(1200);
    let criterion = isolation_body(
        2,
        3,
        vec![0, 2],
        vec![Stage::Block {
            expressions: vec![Atom::num(tiny.clone())],
            outputs: vec![2],
            source: id(246),
        }],
    );
    let p = root_isolation_program(
        id(240),
        &residual,
        &isolation_true(2),
        &criterion,
        &Arc::new(AtomicBool::new(false)),
        10_000,
    )
    .unwrap()
    .unwrap();
    assert_eq!(p.criterion[0], p.residuals[1]);
    assert!(matches!(&p.nodes[p.criterion[1]], Node::Const(Constant::Rational(r)) if r == &tiny));
    assert!(
        !p.nodes
            .iter()
            .any(|node| matches!(node, Node::Const(Constant::Float(_))))
    );
}

#[test]
fn root_isolation_criterion_retains_value_and_derivative_guards() {
    use crate::guarded::Condition;
    let residual = isolation_body(2, 2, vec![0], vec![]);
    let criterion = isolation_body(
        2,
        2,
        vec![0, 1],
        vec![
            Stage::Require {
                argument: 1,
                condition: Condition::Nonnegative,
                order: DerivativeOrder::Value,
                source: id(246),
                lineage: None,
            },
            Stage::Require {
                argument: 0,
                condition: Condition::Positive,
                order: DerivativeOrder::First,
                source: id(246),
                lineage: None,
            },
            Stage::Require {
                argument: 1,
                condition: Condition::Nonzero,
                order: DerivativeOrder::Second,
                source: id(246),
                lineage: None,
            },
        ],
    );
    let p = root_isolation_program(
        id(240),
        &residual,
        &isolation_true(2),
        &criterion,
        &Arc::new(AtomicBool::new(false)),
        10_000,
    )
    .unwrap()
    .unwrap();
    assert_eq!(p.obligations.len(), 1);
    assert_eq!(p.obligations[0].argument, Some(p.criterion[1]));
    assert_eq!(
        p.obligations[0].kind,
        ObligationKind::Require(Condition::Nonnegative)
    );
    assert_eq!(p.derivative_obligations.len(), 2);
    assert_eq!(p.derivative_obligations[0].0, DerivativeOrder::First);
    assert_eq!(p.derivative_obligations[0].1.argument, Some(p.criterion[0]));
    assert!(p.derivative_obligations[0].1.constraints[0].strict);
    assert_eq!(p.derivative_obligations[1].0, DerivativeOrder::Second);
    assert_eq!(p.derivative_obligations[1].1.argument, Some(p.criterion[1]));
    assert!(p.derivative_obligations[1].1.represented);
    assert!(p.derivative_obligations[1].1.constraints.is_empty());
}

#[test]
fn root_isolation_refuses_opaque_nonsmooth_or_unrepresented_criteria() {
    let registry = standard_registry().unwrap();
    let residual = isolation_body(1, 1, vec![0], vec![]);
    let provider = AdmittedProvider::new(spec(&registry, 40), &registry).unwrap();
    let mut b = builder(&registry, 1);
    let v = inputs(&mut b, 1);
    let value = b.provider(&provider, &v, id(246)).unwrap().remove(0);
    let opaque = b.prepare(&[value, number(Atom::num(0))]).unwrap();
    let mut b = builder(&registry, 1);
    let x = inputs(&mut b, 1).remove(0);
    let value = unary(&mut b, Function::Abs, &x);
    let nonsmooth = b.prepare(&[value, number(Atom::num(0))]).unwrap();
    let mut b = builder(&registry, 1);
    let x = inputs(&mut b, 1).remove(0);
    let zero = number(Atom::num(0));
    let one = number(Atom::num(1));
    let predicate = b
        .domain(form_lineage(id(246)), |b| {
            select(b, Comparison::Ne, &x, &zero, &|_| Ok(one.clone()), &zero)
        })
        .unwrap();
    let value = b.with_assumption(x, &predicate);
    let unrepresented = b.prepare(&[value, zero]).unwrap();
    for criterion in [opaque, nonsmooth, unrepresented] {
        assert!(
            root_isolation_program(
                id(240),
                &residual,
                &isolation_true(1),
                &criterion,
                &Arc::new(AtomicBool::new(false)),
                10_000,
            )
            .unwrap()
            .is_none()
        );
    }
}

#[test]
fn root_isolation_refuses_conditional_criterion_guards_with_constant_score() {
    use crate::guarded::Condition;
    let criterion = isolation_body(
        1,
        3,
        vec![2, 1],
        vec![
            Stage::Block {
                expressions: vec![Atom::num(0)],
                outputs: vec![1],
                source: id(246),
            },
            Stage::Branch {
                continuity: DerivativeOrder::Second,
                comparison: Comparison::Lt,
                left: 1,
                right: 0,
                then: vec![
                    Stage::Require {
                        argument: 0,
                        condition: Condition::Nonzero,
                        order: DerivativeOrder::Second,
                        source: id(246),
                        lineage: None,
                    },
                    Stage::Block {
                        expressions: vec![Atom::num(1)],
                        outputs: vec![2],
                        source: id(246),
                    },
                ],
                otherwise: vec![Stage::Block {
                    expressions: vec![Atom::num(1)],
                    outputs: vec![2],
                    source: id(246),
                }],
            },
        ],
    );
    assert!(
        root_isolation_program(
            id(240),
            &isolation_body(1, 1, vec![0], vec![]),
            &isolation_true(1),
            &criterion,
            &Arc::new(AtomicBool::new(false)),
            10_000,
        )
        .unwrap()
        .is_none()
    );
}

fn covered_guard_criterion(cover: Option<Stage>, guard: Stage) -> PreparedBody {
    let mut stages = vec![Stage::Block {
        expressions: vec![Atom::num(0)],
        outputs: vec![2],
        source: id(247),
    }];
    stages.extend(cover);
    stages.push(Stage::Branch {
        continuity: DerivativeOrder::Second,
        comparison: Comparison::Lt,
        left: 2,
        right: 1,
        then: vec![
            guard,
            Stage::Block {
                expressions: vec![Atom::num(1)],
                outputs: vec![4],
                source: id(248),
            },
        ],
        otherwise: vec![Stage::Block {
            expressions: vec![Atom::num(1)],
            outputs: vec![4],
            source: id(248),
        }],
    });
    isolation_body(2, 7, vec![4, 2], stages)
}
fn project_guard_criterion(criterion: &PreparedBody) -> Option<RootIsolationProgram> {
    root_isolation_program(
        id(240),
        &isolation_body(2, 2, vec![0], vec![]),
        &isolation_true(2),
        criterion,
        &Arc::new(AtomicBool::new(false)),
        10_000,
    )
    .unwrap()
}
fn guard_domain(argument: usize, token: usize, source: SemanticId) -> Stage {
    Stage::Domain {
        stages: vec![],
        argument,
        token,
        lineage: form_lineage(source),
    }
}
fn guard_require(
    argument: usize,
    condition: crate::guarded::Condition,
    order: DerivativeOrder,
    source: SemanticId,
) -> Stage {
    Stage::Require {
        argument,
        condition,
        order,
        source,
        lineage: None,
    }
}

#[test]
fn root_isolation_retains_unconditionally_covered_branch_domain_attribution() {
    let criterion = covered_guard_criterion(
        Some(guard_domain(0, 3, id(247))),
        guard_domain(0, 6, id(248)),
    );
    let program = project_guard_criterion(&criterion).unwrap();
    assert_eq!(program.obligations.len(), 2);
    assert_eq!(program.obligations[0].source, id(247));
    assert_eq!(program.obligations[1].source, id(248));
    assert_eq!(
        program.obligations[0].constraints,
        program.obligations[1].constraints
    );
    assert_eq!(program.obligations[1].constraints.len(), 1);
    assert!(
        program
            .obligations
            .iter()
            .all(|o| o.scope == ObligationScope::Unconditional
                && o.represented
                && o.fidelity == Fidelity::Exact
                && o.instance == id(240))
    );
}

#[test]
fn root_isolation_refuses_branch_guards_without_the_same_unconditional_predicate() {
    use crate::guarded::Condition;
    for criterion in [
        covered_guard_criterion(None, guard_domain(0, 6, id(248))),
        covered_guard_criterion(
            Some(guard_domain(1, 3, id(247))),
            guard_domain(0, 6, id(248)),
        ),
        covered_guard_criterion(
            Some(guard_require(
                0,
                Condition::Nonnegative,
                DerivativeOrder::Value,
                id(247),
            )),
            guard_require(0, Condition::Positive, DerivativeOrder::Value, id(248)),
        ),
        // Nonzero guards have identical empty closed constraints; their actual arguments
        // still differ, so the unconditional guard cannot cover this branch obligation.
        covered_guard_criterion(
            Some(guard_require(
                1,
                Condition::Nonzero,
                DerivativeOrder::Value,
                id(247),
            )),
            guard_require(0, Condition::Nonzero, DerivativeOrder::Value, id(248)),
        ),
        covered_guard_criterion(
            Some(guard_require(
                0,
                Condition::Nonzero,
                DerivativeOrder::Value,
                id(247),
            )),
            guard_domain(0, 6, id(248)),
        ),
    ] {
        assert!(project_guard_criterion(&criterion).is_none());
    }
}

#[test]
fn root_isolation_guard_coverage_respects_minimum_derivative_order() {
    use crate::guarded::Condition;
    let criterion = |cover, branch| {
        covered_guard_criterion(
            Some(guard_require(0, Condition::Nonzero, cover, id(247))),
            guard_require(0, Condition::Nonzero, branch, id(248)),
        )
    };
    assert!(
        project_guard_criterion(&criterion(DerivativeOrder::Second, DerivativeOrder::First))
            .is_none()
    );
    let program =
        project_guard_criterion(&criterion(DerivativeOrder::First, DerivativeOrder::Second))
            .unwrap();
    assert_eq!(program.derivative_obligations.len(), 2);
    assert_eq!(program.derivative_obligations[0].0, DerivativeOrder::First);
    assert_eq!(program.derivative_obligations[1].0, DerivativeOrder::Second);
    assert_eq!(program.derivative_obligations[1].1.source, id(248));
    assert_eq!(
        program.derivative_obligations[1].1.scope,
        ObligationScope::Unconditional
    );
}

#[test]
fn root_isolation_does_not_promote_matching_unrepresented_domain_guards() {
    let domain = |source, token| Stage::Domain {
        stages: vec![Stage::Branch {
            continuity: DerivativeOrder::Value,
            comparison: Comparison::Ne,
            left: 0,
            right: 2,
            then: vec![Stage::Block {
                expressions: vec![Atom::num(1)],
                outputs: vec![5],
                source,
            }],
            otherwise: vec![Stage::Block {
                expressions: vec![Atom::num(0)],
                outputs: vec![5],
                source,
            }],
        }],
        argument: 5,
        token,
        lineage: form_lineage(source),
    };
    // The nonzero predicate is a disjunction. Identical incomplete records do not
    // establish the required complete original validity domain.
    let criterion = covered_guard_criterion(Some(domain(id(247), 3)), domain(id(248), 6));
    assert!(project_guard_criterion(&criterion).is_none());
}

fn rational_power_body(exponent: &Rational, positive: bool) -> PreparedBody {
    crate::initialize().unwrap();
    let mut stages = vec![];
    if positive {
        stages.push(Stage::Require {
            argument: 0,
            condition: crate::guarded::Condition::Positive,
            order: DerivativeOrder::Value,
            source: id(249),
            lineage: None,
        });
    }
    stages.push(Stage::Block {
        expressions: vec![library::formal(0).unwrap().pow(Atom::num(exponent.clone()))],
        outputs: vec![1],
        source: id(250),
    });
    isolation_body(1, 2, vec![1], stages)
}
fn native_power_vocabulary(program: &RootIsolationProgram) -> bool {
    program.nodes.iter().all(|node| match node {
        Node::Pow {
            exponent: Constant::Rational(r),
            ..
        } => r == &Rational::new(1, 2) || r.is_integer() && i32::try_from(r.numerator()).is_ok(),
        Node::Pow { .. } => false,
        _ => true,
    })
}

#[test]
fn root_isolation_projects_general_rational_powers_under_original_positive_guards() {
    let cancel = Arc::new(AtomicBool::new(false));
    for exponent in [
        Rational::new(3, 2),
        Rational::new(1, 3),
        Rational::new(5, 8),
        Rational::new(-2, 3),
        Rational::new(-3, 2),
    ] {
        let body = rational_power_body(&exponent, true);
        let program = isolation_project(&body, &isolation_true(1)).unwrap();
        assert!(native_power_vocabulary(&program));
        assert!(
            program
                .nodes
                .iter()
                .any(|node| matches!(node, Node::Exp(_)))
        );
        assert!(
            program
                .nodes
                .iter()
                .any(|node| matches!(node, Node::Log(_)))
        );
        let guard = program
            .obligations
            .iter()
            .find(|guard| guard.source == id(249))
            .unwrap();
        assert_eq!(
            guard.kind,
            ObligationKind::Require(crate::guarded::Condition::Positive)
        );
        assert_eq!(guard.scope, ObligationScope::Unconditional);
        assert!(guard.represented);
        assert!(guard.constraints[0].strict);
        assert!(matches!(
            program.nodes[guard.argument.unwrap()],
            Node::Var(0)
        ));
        let mut worker = body
            .compile(
                &[0],
                &[],
                DerivativeOrder::Value,
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap()
            .worker();
        let original = worker
            .evaluate(
                &[8.0],
                DerivativeOrder::Value,
                &mut BTreeMap::new(),
                &cancel,
            )
            .unwrap();
        assert!((original.values[0] - 8.0_f64.powf(exponent.to_f64())).abs() < 1e-10);
        for invalid in [0.0, -1.0] {
            assert!(
                matches!(worker.evaluate(&[invalid], DerivativeOrder::Value, &mut BTreeMap::new(), &cancel),
                Err(MathError::Domain { source_id, .. }) if source_id == id(249))
            );
        }
    }
}

#[test]
fn root_isolation_raw_dyadic_powers_preserve_zero_and_negative_power_poles() {
    let cancel = Arc::new(AtomicBool::new(false));
    for exponent in [
        Rational::new(3, 2),
        Rational::new(5, 8),
        Rational::new(-3, 2),
    ] {
        let body = rational_power_body(&exponent, false);
        let program = isolation_project(&body, &isolation_true(1)).unwrap();
        assert!(native_power_vocabulary(&program));
        assert!(program.nodes.iter().any(|node| matches!(node,
            Node::Pow { exponent: Constant::Rational(r), .. } if r == &Rational::new(1, 2))));
        assert!(
            !program
                .nodes
                .iter()
                .any(|node| matches!(node, Node::Log(_) | Node::Exp(_)))
        );
        assert!(program.obligations.is_empty());
        let mut worker = body
            .compile(
                &[0],
                &[],
                DerivativeOrder::Value,
                Optimization::default(),
                EvaluationLimits::default(),
                &cancel,
            )
            .unwrap()
            .worker();
        if exponent > 0 {
            assert_eq!(
                worker
                    .evaluate(
                        &[0.0],
                        DerivativeOrder::Value,
                        &mut BTreeMap::new(),
                        &cancel
                    )
                    .unwrap()
                    .values,
                vec![0.0]
            );
        } else {
            assert!(
                worker
                    .evaluate(
                        &[0.0],
                        DerivativeOrder::Value,
                        &mut BTreeMap::new(),
                        &cancel
                    )
                    .is_err()
            );
            assert!(program.nodes.iter().any(|node| matches!(node,
                Node::Pow { exponent: Constant::Rational(r), .. } if r.is_integer() && r < &Rational::from(0))));
        }
        assert!(
            worker
                .evaluate(
                    &[-1.0],
                    DerivativeOrder::Value,
                    &mut BTreeMap::new(),
                    &cancel
                )
                .is_err()
        );
    }
    // No positive-domain authority exists for an arbitrary unguarded rational power.
    // Keep it outside the native vocabulary rather than introducing a new log domain.
    let raw = isolation_project(
        &rational_power_body(&Rational::new(1, 3), false),
        &isolation_true(1),
    )
    .unwrap();
    assert!(!native_power_vocabulary(&raw));
    assert!(
        !raw.nodes
            .iter()
            .any(|node| matches!(node, Node::Log(_) | Node::Exp(_)))
    );
}

#[test]
fn root_isolation_integer_negative_base_domain_is_unchanged() {
    let cancel = Arc::new(AtomicBool::new(false));
    let body = rational_power_body(&Rational::from(-3), false);
    let program = isolation_project(&body, &isolation_true(1)).unwrap();
    assert!(native_power_vocabulary(&program));
    assert!(
        !program
            .nodes
            .iter()
            .any(|node| matches!(node, Node::Log(_) | Node::Exp(_)))
    );
    let mut worker = body
        .compile(
            &[0],
            &[],
            DerivativeOrder::Value,
            Optimization::default(),
            EvaluationLimits::default(),
            &cancel,
        )
        .unwrap()
        .worker();
    assert_eq!(
        worker
            .evaluate(
                &[-2.0],
                DerivativeOrder::Value,
                &mut BTreeMap::new(),
                &cancel
            )
            .unwrap()
            .values,
        vec![-0.125]
    );
    assert!(
        worker
            .evaluate(
                &[0.0],
                DerivativeOrder::Value,
                &mut BTreeMap::new(),
                &cancel
            )
            .is_err()
    );
}

#[test]
fn root_isolation_criterion_layout_must_match_original_inputs_and_two_outputs() {
    let residual = isolation_body(2, 2, vec![0], vec![]);
    for criterion in [
        isolation_criterion(1),
        isolation_true(2),
        isolation_body(2, 2, vec![0, 1, 0], vec![]),
    ] {
        assert!(matches!(
            root_isolation_program(
                id(240),
                &residual,
                &isolation_true(2),
                &criterion,
                &Arc::new(AtomicBool::new(false)),
                10_000,
            ),
            Err(FactorableError::Math(MathError::Contract(_)))
        ));
    }
}
