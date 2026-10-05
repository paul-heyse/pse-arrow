// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Parametric sensitivities over compiled case plans (Plan 22 S1): the parametric plan of
//! a case with declared parameters ([`CasePlan::parametric`]) differentiated at candidates
//! of Ipopt, POUNCE and the SCIP fixed-assignment re-solve, with and without presolve.
use super::*;
use crate::{
    execution::ResolveSensitivity,
    kkt::{Parametric, Sensitivity},
    presolve::Policy,
    transform::Relaxed,
};

/// A compiled case with declared parameters: its solve plan and its parametric plan.
struct Parameterized {
    solve: Arc<CaseAssembly>,
    parametric: Arc<CaseAssembly>,
    values: CaseValues,
    parameters: Vec<(SemanticId, f64)>,
}
/// Columns `id(i + 1)` and parameters `id(30 + k)` bound to the body's inputs in that
/// order; outputs are the rows `id(101 + r)` and, last, the minimized objective.
fn parameterized(
    registry: &QuantityRegistry,
    body: pse_math::guarded::PreparedBody,
    columns: &[(ModelingVariableDomain, Option<f64>, Option<f64>, f64)],
    parameters: &[f64],
    rows: &[(f64, f64)],
) -> Parameterized {
    let key = ContentHash::from_bytes([9; 32]);
    let variable_port = |i: usize| port(registry, u8::try_from(i + 1).unwrap());
    let parameter_port = |k: usize| port(registry, 30 + u8::try_from(k).unwrap());
    let variables = columns
        .iter()
        .enumerate()
        .map(|(i, (domain, lower, upper, _))| Variable {
            port: variable_port(i),
            fixed: false,
            domain: *domain,
            lower: *lower,
            upper: *upper,
        })
        .collect();
    let ports: Vec<Port> = (0..columns.len())
        .map(variable_port)
        .chain((0..parameters.len()).map(parameter_port))
        .collect();
    let slots = ports
        .iter()
        .map(|p| SlotBinding::new(p, p, registry).unwrap())
        .collect();
    let contributions = (0..=rows.len())
        .map(|k| Contribution {
            output: k,
            target: if k == rows.len() {
                Target::PRIMARY
            } else {
                Target::Row(id(101 + u8::try_from(k).unwrap()))
            },
            scale: 1.0,
        })
        .collect();
    let structure = Arc::new(
        CaseStructure::new(
            variables,
            (0..parameters.len()).map(parameter_port).collect(),
            vec![InstanceBinding {
                checked_members: Default::default(),
                instance: id(9),
                body: key,
                slots,
                contributions,
            }],
            rows.iter()
                .enumerate()
                .map(|(r, (lower, upper))| Row {
                    id: id(101 + u8::try_from(r).unwrap()),
                    quantity: neutral(),
                    lower: *lower,
                    upper: *upper,
                })
                .collect(),
            Some(Objective {
                quantity: neutral(),
                sense: ObjectiveSense::Minimize,
            }),
            CaseLimits::default(),
        )
        .unwrap(),
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let plan = CasePlan::prepare(
        structure,
        BTreeMap::from([(key, Arc::new(body))]),
        registry,
        DerivativeOrder::Second,
        AssemblyLimits::default(),
        &cancel,
    )
    .unwrap();
    let ids: Vec<SemanticId> = (0..parameters.len())
        .map(|k| parameter_port(k).id)
        .collect();
    let parametric = plan
        .parametric(&ids, DerivativeOrder::Second, registry, &cancel)
        .unwrap();
    let compile = |plan: CasePlan| {
        Arc::new(
            Arc::new(plan)
                .compile(
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &cancel,
                )
                .unwrap(),
        )
    };
    Parameterized {
        solve: compile(plan),
        parametric: compile(parametric),
        values: CaseValues {
            scalars: ports
                .iter()
                .zip(
                    columns
                        .iter()
                        .map(|c| c.3)
                        .chain(parameters.iter().copied()),
                )
                .map(|(p, v)| (p.id, v))
                .collect(),
        },
        parameters: ids.into_iter().zip(parameters.iter().copied()).collect(),
    }
}
impl Parameterized {
    fn initial(&self) -> Vec<f64> {
        self.solve
            .columns()
            .iter()
            .map(|c| self.values.scalars[c])
            .collect()
    }
    fn rows(&self) -> usize {
        self.solve.structure().rows().len()
    }
    /// Callbacks over `assembly` with identity normalization, and the compiler's presolve
    /// facts for the solve plan on request.
    fn oracle(
        &self,
        assembly: &Arc<CaseAssembly>,
        facts: bool,
    ) -> Result<Box<dyn NlpOracle>, ProblemError> {
        let worker = assembly.worker(BTreeMap::new(), Arc::new(AtomicBool::new(false)));
        let mut oracle = crate::assembled::AlgebraicOracle::new(worker, self.values.clone())?
            .with_normalization(Normalization::identity(
                assembly.columns().len(),
                self.rows(),
            ))?;
        if facts {
            let cancel = Arc::new(AtomicBool::new(false));
            oracle = oracle.with_presolve_facts(
                Arc::new(assembly.presolve_facts(&self.values, 100_000, &cancel)?).into(),
            )?;
        }
        Ok(Box::new(oracle))
    }
    /// Callbacks over `assembly` with every column, discrete ones included, for a re-solve
    /// to pin.
    fn relaxed(&self, assembly: &Arc<CaseAssembly>) -> Result<Relaxed, ProblemError> {
        let worker = assembly.worker(BTreeMap::new(), Arc::new(AtomicBool::new(false)));
        crate::assembled::AlgebraicOracle::relaxation(
            worker,
            self.values.clone(),
            Normalization::identity(assembly.columns().len(), self.rows()),
        )
    }
    fn request(&self) -> Sensitivity {
        Sensitivity {
            source: None,
            oracle: self.oracle(&self.parametric, false).unwrap(),
            parameters: self.parameters.clone(),
            reduced_hessian: true,
            retain: false,
        }
    }
}

/// One NLP solve through the one runner, with the request's sensitivities.
fn nlp(case: &Parameterized, backend: Backend, presolve: &Policy) -> SolveReport {
    let (n, m) = (case.solve.columns().len(), case.rows());
    let controls = Controls::default();
    let accuracy = ResolvedAccuracy::from_policy(&Default::default(), 1e-9).unwrap();
    let tolerances = tolerances(n, m);
    let normalization = Normalization::identity(n, m);
    execution::nlp(
        Step {
            snapshot: &execution::Snapshot::observe(&execution::LINKED),
            structure: None,
            adapter: execution::adapter(backend),
            settings: &BackendSettings::Default,
            controls: &controls,
            accuracy: &accuracy,
            execution: execution(false),
            tolerances: &tolerances,
            normalization: &normalization,
            compatibility: stamp(backend),
            warm: None,
        },
        &mut Retained::default(),
        execution::Nlp {
            oracle: case
                .oracle(&case.solve, matches!(presolve, Policy::Auto))
                .unwrap(),
            initial: &case.initial(),
            presolve,
            intent: SolveIntent::Optimize,
            sense: ObjectiveSense::Minimize,
            limit: 100_000,
            analysis: execution::Analysis {
                second_order: true,
                sensitivity: Some(case.request()),
                inverse_reduced_hessian: None,
                output_accuracy: None,
            },
        },
    )
    .unwrap()
}
/// The global route with its fixed-assignment re-solve and the re-solve's sensitivities.
fn scip(case: &Parameterized) -> SolveReport {
    let program = case
        .solve
        .factorable_program(
            &case.values,
            &FactorableRequest::default(),
            100_000,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let (n, m) = (program.variables.len(), program.rows.len());
    let accuracy = ResolvedAccuracy::verification();
    let tolerances = tolerances(n, m);
    let normalization = Normalization::identity(n, m);
    let mut original = Evaluated(case);
    let mut relaxed = || case.relaxed(&case.solve);
    let mut parametric = || case.relaxed(&case.parametric);
    let initial = case.initial();
    execution::factorable(
        Step {
            snapshot: &execution::Snapshot::observe(&execution::LINKED),
            structure: None,
            adapter: execution::adapter(Backend::Scip),
            settings: &BackendSettings::Scip(ScipSettings::default()),
            controls: &Controls::default(),
            accuracy: &accuracy,
            execution: execution(false),
            tolerances: &tolerances,
            normalization: &normalization,
            compatibility: stamp(Backend::Scip),
            warm: None,
        },
        &mut Retained::default(),
        Factorable {
            program: &program,
            initial: &initial,
            intent: SolveIntent::Optimize,
            original: &mut original,
            resolve: Some(Resolve {
                oracle: &mut relaxed,
                presolve: &Policy::Auto,
                limit: 100_000,
                sensitivity: Some(ResolveSensitivity {
                    oracle: &mut parametric,
                    parameters: case.parameters.clone(),
                    reduced_hessian: true,
                }),
            }),
        },
    )
    .unwrap()
}
/// The original compiled case, evaluated fresh at a candidate.
struct Evaluated<'c>(&'c Parameterized);
impl OriginalModel for Evaluated<'_> {
    fn evaluate(&mut self, primal: &[f64]) -> Result<Evaluation, ProblemError> {
        let case = self.0;
        let mut worker = case
            .solve
            .worker(BTreeMap::new(), Arc::new(AtomicBool::new(false)));
        let mut values = case.values.clone();
        for (c, v) in case.solve.columns().iter().zip(primal) {
            values.scalars.insert(*c, *v);
        }
        Ok(Evaluation {
            constraints: worker.constraints(&values)?,
            objective: Some(worker.objective(&values)?),
            sources: worker.constraint_sources()?,
        })
    }
}
fn certified(report: &SolveReport) -> &Parametric {
    let parametric = report.evidence.sensitivity.as_ref().unwrap();
    assert!(parametric.sensitivities.is_ok(), "{parametric:?}");
    assert!(
        matches!(parametric.reduced_hessian, Some(Ok(_))),
        "{parametric:?}"
    );
    parametric
}
/// Every quantity of `a` within `tolerance` of `b`.
fn agree(a: &Parametric, b: &Parametric, tolerance: f64) {
    let (sa, sb) = (
        a.sensitivities.as_ref().unwrap(),
        b.sensitivities.as_ref().unwrap(),
    );
    let (ha, hb) = (
        a.reduced_hessian.as_ref().unwrap().as_ref().unwrap(),
        b.reduced_hessian.as_ref().unwrap().as_ref().unwrap(),
    );
    let pairs = sa
        .primal
        .iter()
        .flatten()
        .zip(sb.primal.iter().flatten())
        .chain(sa.rows.iter().flatten().zip(sb.rows.iter().flatten()))
        .chain(sa.bounds.iter().flatten().zip(sb.bounds.iter().flatten()))
        .chain(sa.objective.iter().zip(&sb.objective))
        .chain(ha.values.iter().zip(&hb.values));
    for (x, y) in pairs {
        assert!(
            (x - y).abs() <= tolerance * (1.0 + y.abs()),
            "{sa:?} {sb:?} {ha:?} {hb:?}"
        );
    }
}

/// `min ½x₁² + x₂² − p₂x₁ (+ 3y)  s.t.  x₁ + x₂ − p₁ = 0` at p = (1, 2), with the
/// closed-form sensitivities of the analytic unit: `dx/dp`, `df*/dp = (−λ, −x₁)` and
/// `d²f*/dp² = [[2/3, −2/3], [−2/3, −1/3]]`. With `committed`, a binary y joins at cost 3:
/// a MINLP whose optimal assignment is y = 0 and whose continuous problem under it is the
/// continuous case.
fn quadratic(registry: &QuantityRegistry, committed: bool) -> Parameterized {
    let inputs = if committed { 5 } else { 4 };
    let mut b = Body::new(registry, inputs);
    let (x1, x2) = (b.x[0].clone(), b.x[1].clone());
    let (p1, p2) = (b.x[inputs - 2].clone(), b.x[inputs - 1].clone());
    let sum = b.op(Binary::Add, &x1, &x2);
    let row = b.op(Binary::Sub, &sum, &p1);
    let half = b.c(0.5);
    let x1x1 = b.op(Binary::Mul, &x1, &x1);
    let first = b.op(Binary::Mul, &half, &x1x1);
    let second = b.op(Binary::Mul, &x2, &x2);
    let linear = b.op(Binary::Mul, &p2, &x1);
    let objective = b.op(Binary::Add, &first, &second);
    let mut objective = b.op(Binary::Sub, &objective, &linear);
    let mut columns = vec![
        (
            ModelingVariableDomain::Continuous,
            Some(-10.0),
            Some(10.0),
            0.5,
        ),
        (
            ModelingVariableDomain::Continuous,
            Some(-10.0),
            Some(10.0),
            0.5,
        ),
    ];
    if committed {
        let three = b.c(3.0);
        let y = b.x[2].clone();
        let build = b.op(Binary::Mul, &three, &y);
        objective = b.op(Binary::Add, &objective, &build);
        columns.push((ModelingVariableDomain::Binary, None, None, 0.0));
    }
    let body = b.b.prepare(&[row, objective]).unwrap();
    parameterized(registry, body, &columns, &[1.0, 2.0], &[(0.0, 0.0)])
}
const DX: [[f64; 2]; 2] = [[2.0 / 3.0, 1.0 / 3.0], [1.0 / 3.0, -1.0 / 3.0]];
const DF: [f64; 2] = [-2.0 / 3.0, -4.0 / 3.0];
const H: [f64; 4] = [2.0 / 3.0, -2.0 / 3.0, -2.0 / 3.0, -1.0 / 3.0];
/// The closed form of [`quadratic`] on the continuous columns.
fn analytic(parametric: &Parametric) {
    let s = parametric.sensitivities.as_ref().unwrap();
    for k in 0..2 {
        for j in 0..2 {
            assert!((s.primal[k][j] - DX[k][j]).abs() < 1e-6, "{s:?}");
        }
        assert!((s.objective[k] - DF[k]).abs() < 1e-6, "{s:?}");
    }
    let h = parametric
        .reduced_hessian
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    for (actual, expected) in h.values.iter().zip(H) {
        assert!((actual - expected).abs() < 1e-6, "{h:?}");
    }
}

#[test]
fn sensitivity_backend_independent() {
    let registry = standard_registry().unwrap();
    // The global route's candidate is the continuous re-solve under its assignment y = 0;
    // its sensitivities are conditional on the assignment, and the committed y is pinned.
    let global = scip(&quadratic(&registry, true));
    assert_eq!(
        global.evidence.global.unwrap().primal,
        PrimalSource::FixedAssignment
    );
    let reference = certified(&global);
    analytic(reference);
    let s = reference.sensitivities.as_ref().unwrap();
    assert!(s.primal.iter().all(|column| column[2] == 0.0), "{s:?}");
    // The local routes differentiate the same continuous problem.
    let case = quadratic(&registry, false);
    for backend in [Backend::Ipopt, Backend::Pounce] {
        let report = nlp(&case, backend, &Policy::Off);
        assert_eq!(
            report.qualification,
            Qualification::Stationary,
            "{backend:?}"
        );
        analytic(certified(&report));
    }
}

#[test]
fn sensitivity_survives_presolve() {
    // Presolve eliminates the coupling row and one column; postsolve recovers the row's
    // multiplier, and the analysis differentiates the original model whatever presolve
    // removed.
    let registry = standard_registry().unwrap();
    let case = quadratic(&registry, false);
    for backend in [Backend::Ipopt, Backend::Pounce] {
        let off = nlp(&case, backend, &Policy::Off);
        let auto = nlp(&case, backend, &Policy::Auto);
        let dimensions = &auto.preprocessing.as_ref().unwrap().dimensions;
        assert!(
            dimensions.presolved_rows < dimensions.original_rows,
            "{backend:?} {dimensions:?}"
        );
        for report in [&off, &auto] {
            assert_eq!(
                report.qualification,
                Qualification::Stationary,
                "{backend:?}"
            );
            analytic(certified(report));
        }
        agree(certified(&auto), certified(&off), 1e-6);
    }
}

/// `min ½x₁² + x₂² − p₂x₁ + p₂x₃  s.t.  x₁ + x₂ − p₁ = 0, x₃ − p₁ = 0`: presolve fixes x₃
/// by the singleton row and removes both.
fn singleton(registry: &QuantityRegistry) -> Parameterized {
    let mut b = Body::new(registry, 5);
    let (x1, x2, x3, p1, p2) = (
        b.x[0].clone(),
        b.x[1].clone(),
        b.x[2].clone(),
        b.x[3].clone(),
        b.x[4].clone(),
    );
    let sum = b.op(Binary::Add, &x1, &x2);
    let coupled = b.op(Binary::Sub, &sum, &p1);
    let fixed = b.op(Binary::Sub, &x3, &p1);
    let half = b.c(0.5);
    let x1x1 = b.op(Binary::Mul, &x1, &x1);
    let first = b.op(Binary::Mul, &half, &x1x1);
    let second = b.op(Binary::Mul, &x2, &x2);
    let linear = b.op(Binary::Mul, &p2, &x1);
    let price = b.op(Binary::Mul, &p2, &x3);
    let objective = b.op(Binary::Add, &first, &second);
    let objective = b.op(Binary::Sub, &objective, &linear);
    let objective = b.op(Binary::Add, &objective, &price);
    let body = b.b.prepare(&[coupled, fixed, objective]).unwrap();
    parameterized(
        registry,
        body,
        &[
            (
                ModelingVariableDomain::Continuous,
                Some(-10.0),
                Some(10.0),
                0.5,
            ),
            (
                ModelingVariableDomain::Continuous,
                Some(-10.0),
                Some(10.0),
                0.5,
            ),
            (
                ModelingVariableDomain::Continuous,
                Some(-10.0),
                Some(10.0),
                0.5,
            ),
        ],
        &[1.0, 2.0],
        &[(0.0, 0.0), (0.0, 0.0)],
    )
}

#[test]
fn automatic_presolve_preserves_original_multiplier_and_sensitivity() {
    // The supported affine reduction recovers the removed singleton row's multiplier
    // x₃ = p₁ as −p₂ = −2. Untracked interval bounds must not replace that source owner.
    // Both the original problem and the qualified reduction have the analytic response:
    // dx₃/dp = (1, 0), dλ₂/dp = (0, −1), df*/dp = (−λ₁ + p₂, −x₁ + x₃) = (4/3, −1/3) and
    // d²f*/dp² = [[2/3, 1/3], [1/3, −1/3]].
    let registry = standard_registry().unwrap();
    let case = singleton(&registry);
    for policy in [Policy::Off, Policy::Auto] {
        let report = nlp(&case, Backend::Ipopt, &policy);
        assert!(
            (report
                .candidate
                .as_ref()
                .unwrap()
                .row_dual
                .as_ref()
                .unwrap()[1]
                + 2.0)
                .abs()
                < 1e-6,
            "{report:?}"
        );
        assert_eq!(report.evidence.kkt.unwrap().stationarity, Some(true));
        if matches!(policy, Policy::Auto) {
            assert_eq!(
                report
                    .preprocessing
                    .as_ref()
                    .unwrap()
                    .dimensions
                    .presolved_rows,
                0
            );
        }
        let certified = certified(&report);
        let s = certified.sensitivities.as_ref().unwrap();
        assert!(
            (s.primal[0][2] - 1.0).abs() < 1e-6 && s.primal[1][2].abs() < 1e-6,
            "{s:?}"
        );
        assert!(
            s.rows[0][1].abs() < 1e-6 && (s.rows[1][1] + 1.0).abs() < 1e-6,
            "{s:?}"
        );
        assert!((s.objective[0] - 4.0 / 3.0).abs() < 1e-6, "{s:?}");
        assert!((s.objective[1] + 1.0 / 3.0).abs() < 1e-6, "{s:?}");
        let h = certified
            .reduced_hessian
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap();
        for (actual, expected) in h
            .values
            .iter()
            .zip([2.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0, -1.0 / 3.0])
        {
            assert!((actual - expected).abs() < 1e-6, "{h:?}");
        }
    }
}
