// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! SCIP adapter units (Plan 22 G1 binding, G3). Test problems are compiled case plans
//! built directly from typed bodies; the neutral program is projected from them exactly
//! as preparation projects an authored case.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "test fixtures report a broken precondition by panicking; the panic policy governs library code"
)]
use crate::{
    NlpOracle, ProblemError,
    execution::{
        self, BackendSettings, Evaluation, Factorable, IpoptLinearSolver,
        OriginalModel, Refusal, Resolve, Retained, ScipSettings, Step,
    },
    quality::Tolerances,
    scip::{self, Status},
    solve::{
        Assurance, Backend, BoundSource, Controls, Execution, OptionValue, PrimalSource,
        Qualification, SolveIntent, SolveReport, Termination, WarmCapability,
    },
    solver_tests::stamp,
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{
    AdmittedProvider, DerivativeOrder, EvaluationContext, Port, Provider, ProviderError,
    ProviderKey, ProviderRequest, ProviderSpec, ProviderValues,
};
use pse_math::{
    Function,
    assembly::{AssemblyLimits, CaseAssembly, CasePlan},
    binding::{
        CaseLimits, CaseStructure, CaseValues, Contribution, InstanceBinding, Objective,
        ObjectiveSense, Row, SlotBinding, Target, Variable, VariableDomain,
    },
    factorable::{BoundOwner, FactorableProgram, FactorableRequest, Fidelity},
    jets::EvaluationLimits,
    library::Optimization,
    normalization::Normalization,
    typed::{Binary, BodyBuilder, BodyLimits, TypedValue},
};
use pse_quantity::{
    IndexSet, QuantityRegistry, QuantityTypeId,
    literal::LiteralContext,
    standard::{StandardInvariantChecker, ids, standard_registry},
};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

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
/// A typed body over `inputs` neutral inputs.
struct Body<'r> {
    b: BodyBuilder<'r>,
    registry: &'r QuantityRegistry,
    x: Vec<TypedValue>,
}
impl<'r> Body<'r> {
    fn new(registry: &'r QuantityRegistry, inputs: usize) -> Self {
        let mut b = BodyBuilder::new(
            pse_math::initialize().unwrap(),
            registry,
            &StandardInvariantChecker,
            inputs,
            BodyLimits::default(),
        )
        .unwrap();
        let x = (0..inputs)
            .map(|i| b.input(i, neutral(), IndexSet::new(), id(200)).unwrap())
            .collect();
        Self { b, registry, x }
    }
    fn c(&mut self, value: f64) -> TypedValue {
        let unit = self
            .registry
            .quantity_type(neutral())
            .unwrap()
            .canonical_unit;
        self.b
            .literal(
                value,
                unit,
                LiteralContext::Explicit {
                    quantity_type: neutral(),
                },
                id(201),
            )
            .unwrap()
    }
    fn op(&mut self, op: Binary, l: &TypedValue, r: &TypedValue) -> TypedValue {
        self.b
            .binary(op, l.clone(), r.clone(), None, id(202))
            .unwrap()
    }
    fn f(&mut self, f: Function, v: &TypedValue) -> TypedValue {
        self.b.unary(f, v.clone(), id(203)).unwrap()
    }
}
/// Columns `id(i + 1)` with declared domains and boxes; output `k` contributes to row
/// `id(100 + k)`, or to the objective when it is `objective`.
struct Case {
    assembly: Arc<CaseAssembly>,
    values: CaseValues,
    providers: fn() -> BTreeMap<ProviderKey, Box<dyn Provider>>,
}
fn no_providers() -> BTreeMap<ProviderKey, Box<dyn Provider>> {
    BTreeMap::new()
}
fn case(
    registry: &QuantityRegistry,
    body: pse_math::guarded::PreparedBody,
    columns: &[(VariableDomain, Option<f64>, Option<f64>, f64)],
    rows: &[(f64, f64)],
    objective: Option<(usize, ObjectiveSense)>,
    order: DerivativeOrder,
) -> Case {
    let key = ContentHash::from_bytes([7; 32]);
    let variables = columns
        .iter()
        .enumerate()
        .map(|(i, (domain, lower, upper, _))| Variable {
            port: port(registry, u8::try_from(i + 1).unwrap()),
            fixed: false,
            domain: *domain,
            lower: *lower,
            upper: *upper,
        })
        .collect();
    let slots = (0..columns.len())
        .map(|i| {
            let p = port(registry, u8::try_from(i + 1).unwrap());
            SlotBinding::new(&p, &p, registry).unwrap()
        })
        .collect();
    let mut row = 0;
    let mut contributions = Vec::new();
    for k in 0..body.output_count() {
        let target = if objective.is_some_and(|(o, _)| o == k) {
            Target::Objective
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
                instance: id(9),
                body: key,
                slots,
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
    let assembly = Arc::new(
        Arc::new(
            CasePlan::prepare(
                structure,
                BTreeMap::from([(key, Arc::new(body))]),
                registry,
                order,
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
    );
    let values = CaseValues {
        scalars: columns
            .iter()
            .enumerate()
            .map(|(i, c)| (id(u8::try_from(i + 1).unwrap()), c.3))
            .collect(),
    };
    Case {
        assembly,
        values,
        providers: no_providers,
    }
}
impl Case {
    fn program(&self, request: &FactorableRequest) -> FactorableProgram {
        self.assembly
            .factorable_program(
                &self.values,
                request,
                100_000,
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap()
    }
    fn initial(&self) -> Vec<f64> {
        self.assembly
            .columns()
            .iter()
            .map(|c| self.values.scalars[c])
            .collect()
    }
    fn continuous_oracle(&self) -> Box<dyn NlpOracle> {
        let worker = self
            .assembly
            .worker((self.providers)(), Arc::new(AtomicBool::new(false)));
        Box::new(
            crate::assembled::AlgebraicOracle::new(worker, self.values.clone())
                .unwrap()
                .with_normalization(Normalization::identity(
                    self.assembly.columns().len(),
                    self.assembly.structure().rows().len(),
                ))
                .unwrap(),
        )
    }
    fn fixed_oracle(
        &self,
        assignment: &BTreeMap<usize, f64>,
    ) -> Result<Box<dyn NlpOracle>, ProblemError> {
        let worker = self
            .assembly
            .worker((self.providers)(), Arc::new(AtomicBool::new(false)));
        let columns = self.assembly.columns();
        let fixed = assignment.iter().map(|(i, v)| (columns[*i], *v)).collect();
        Ok(Box::new(
            crate::assembled::AlgebraicOracle::with_fixed_assignment(
                worker,
                self.values.clone(),
                &fixed,
            )?
            .with_normalization(Normalization::identity(
                columns.len(),
                self.assembly.structure().rows().len(),
            ))?,
        ))
    }
}
/// The original compiled model, evaluated fresh at a candidate.
struct Original<'c>(&'c Case);
impl OriginalModel for Original<'_> {
    fn evaluate(&mut self, primal: &[f64]) -> Result<Evaluation, ProblemError> {
        let case = self.0;
        let mut worker = case
            .assembly
            .worker((case.providers)(), Arc::new(AtomicBool::new(false)));
        let mut values = case.values.clone();
        for (c, v) in case.assembly.columns().iter().zip(primal) {
            values.scalars.insert(*c, *v);
        }
        let constraints = worker.constraints(&values)?;
        let objective = case
            .assembly
            .structure()
            .objective()
            .map(|o| worker.objective(&values).map(|v| v * o.sense.sign()))
            .transpose()?;
        Ok(Evaluation {
            constraints,
            objective,
            sources: worker.constraint_sources()?,
        })
    }
}
fn tolerances(n: usize, m: usize) -> Tolerances {
    Tolerances {
        variables: vec![1e-7; n],
        rows: vec![1e-7; m],
        integrality: 1e-7,
    }
}
fn execution(cancel: bool) -> Execution {
    let mut execution = Execution::new(Arc::new(AtomicBool::new(cancel)), &Controls::default());
    execution.memory = Some(256 << 20);
    execution
}
/// Run the factorable runner on SCIP, optionally with the fixed-assignment re-solve.
fn run(
    case: &Case,
    program: &FactorableProgram,
    intent: SolveIntent,
    resolve: bool,
    cancel: bool,
) -> Result<SolveReport, ProblemError> {
    let n = program.variables.len();
    let m = program.rows.len();
    let controls = Controls::default();
    let tolerances = tolerances(n, m);
    let normalization = Normalization::identity(n, m);
    let mut original = Original(case);
    let mut fixed = |a: &BTreeMap<usize, f64>| case.fixed_oracle(a);
    let presolve = crate::presolve::Policy::Auto;
    let initial = case.initial();
    execution::factorable(
        Step {
            adapter: execution::adapter(Backend::Scip),
            settings: &BackendSettings::Default,
            controls: &controls,
            execution: execution(cancel),
            tolerances: &tolerances,
            normalization: &normalization,
            compatibility: stamp(Backend::Scip),
            warm: None,
        },
        &mut Retained::default(),
        Factorable {
            program,
            initial: &initial,
            intent,
            original: &mut original,
            resolve: resolve.then_some(Resolve {
                oracle: &mut fixed,
                presolve: &presolve,
                limit: 100_000,
            }),
        },
    )
}

/// min y² − 3y + x  s.t.  y − x² = 0,  x ∈ [−2, 2], y ∈ [0, 4]: the quartic
/// x⁴ − 3x² + x with global minimum near x = −1.3008 and a local one near x = 1.1309.
fn quartic(registry: &QuantityRegistry, x_box: (Option<f64>, Option<f64>)) -> Case {
    let mut b = Body::new(registry, 2);
    let (x, y) = (b.x[0].clone(), b.x[1].clone());
    let xx = b.op(Binary::Mul, &x, &x);
    let row = b.op(Binary::Sub, &y, &xx);
    let yy = b.op(Binary::Mul, &y, &y);
    let three = b.c(3.0);
    let three_y = b.op(Binary::Mul, &three, &y);
    let objective = b.op(Binary::Sub, &yy, &three_y);
    let objective = b.op(Binary::Add, &objective, &x);
    let body = b.b.prepare(&[row, objective]).unwrap();
    case(
        registry,
        body,
        &[
            (VariableDomain::Continuous, x_box.0, x_box.1, 1.0),
            (VariableDomain::Continuous, Some(0.0), Some(4.0), 1.0),
        ],
        &[(0.0, 0.0)],
        Some((1, ObjectiveSense::Minimize)),
        DerivativeOrder::Second,
    )
}
/// The quartic's stationary points: roots of 4x³ − 6x + 1 by Newton from a start.
fn quartic_root(mut x: f64) -> f64 {
    for _ in 0..64 {
        x -= (4.0 * x * x * x - 6.0 * x + 1.0) / (12.0 * x * x - 6.0);
    }
    x
}
fn quartic_value(x: f64) -> f64 {
    x.powi(4) - 3.0 * x * x + x
}
/// Superstructure synthesis: build a unit (binary y) whose product x·y meets a demand of
/// 3 together with purchases s. min 2y + x²/2 + 4s  s.t.  x·y + s ≥ 3, x ∈ [0, 4],
/// s ∈ [0, 3]. Building is optimal: y = 1, x = 3, s = 0, cost 6.5 (without it, 12).
fn synthesis(registry: &QuantityRegistry) -> Case {
    let mut b = Body::new(registry, 3);
    let (x, s, y) = (b.x[0].clone(), b.x[1].clone(), b.x[2].clone());
    let xy = b.op(Binary::Mul, &x, &y);
    let row = b.op(Binary::Add, &xy, &s);
    let two = b.c(2.0);
    let half = b.c(0.5);
    let four = b.c(4.0);
    let build = b.op(Binary::Mul, &two, &y);
    let xx = b.op(Binary::Mul, &x, &x);
    let feed = b.op(Binary::Mul, &half, &xx);
    let buy = b.op(Binary::Mul, &four, &s);
    let objective = b.op(Binary::Add, &build, &feed);
    let objective = b.op(Binary::Add, &objective, &buy);
    let body = b.b.prepare(&[row, objective]).unwrap();
    case(
        registry,
        body,
        &[
            (VariableDomain::Continuous, Some(0.0), Some(4.0), 0.0),
            (VariableDomain::Continuous, Some(0.0), Some(3.0), 3.0),
            (VariableDomain::Binary, None, None, 0.0),
        ],
        &[(3.0, f64::INFINITY)],
        Some((1, ObjectiveSense::Minimize)),
        DerivativeOrder::Second,
    )
}

#[test]
fn scip_abi_matches_image() {
    let abi = scip::abi().unwrap();
    assert_eq!(abi.library, (10, 0, 2));
    assert_eq!(abi.api, scip::API_VERSION);
    assert_eq!(scip::API_VERSION, 156);
    // The adapter is linked and publishes a certifying capability record.
    let adapter = execution::adapter(Backend::Scip);
    assert!(adapter.linked());
    assert_eq!(
        adapter.representation(),
        execution::Representation::Factorable
    );
    let row = adapter.capability().row(Backend::Scip);
    assert!(row.certifies && !row.parallel);
    assert_eq!(row.warm, WarmCapability::Primal);
    assert!(
        execution::LINKED
            .published()
            .iter()
            .any(|r| r.backend == Backend::Scip)
    );
}

#[test]
fn scip_status_map_exhaustive() {
    use scip_sys as ffi;
    let raw = [
        ffi::SCIP_Status_SCIP_STATUS_UNKNOWN,
        ffi::SCIP_Status_SCIP_STATUS_OPTIMAL,
        ffi::SCIP_Status_SCIP_STATUS_INFEASIBLE,
        ffi::SCIP_Status_SCIP_STATUS_UNBOUNDED,
        ffi::SCIP_Status_SCIP_STATUS_INFORUNBD,
        ffi::SCIP_Status_SCIP_STATUS_USERINTERRUPT,
        ffi::SCIP_Status_SCIP_STATUS_TERMINATE,
        ffi::SCIP_Status_SCIP_STATUS_NODELIMIT,
        ffi::SCIP_Status_SCIP_STATUS_TOTALNODELIMIT,
        ffi::SCIP_Status_SCIP_STATUS_STALLNODELIMIT,
        ffi::SCIP_Status_SCIP_STATUS_TIMELIMIT,
        ffi::SCIP_Status_SCIP_STATUS_MEMLIMIT,
        ffi::SCIP_Status_SCIP_STATUS_GAPLIMIT,
        ffi::SCIP_Status_SCIP_STATUS_PRIMALLIMIT,
        ffi::SCIP_Status_SCIP_STATUS_DUALLIMIT,
        ffi::SCIP_Status_SCIP_STATUS_SOLLIMIT,
        ffi::SCIP_Status_SCIP_STATUS_BESTSOLLIMIT,
        ffi::SCIP_Status_SCIP_STATUS_RESTARTLIMIT,
    ];
    // Every header status has exactly one typed status, and the map covers each.
    assert_eq!(raw.len(), Status::ALL.len());
    for (code, status) in raw.iter().zip(Status::ALL) {
        assert_eq!(Status::from_raw(*code), Some(status));
        assert_eq!(status.raw(), *code);
        let t = scip::termination(status);
        assert_eq!(t.code, i64::from(*code));
        assert_eq!(t.name, status.name());
        assert!(t.name.starts_with("SCIP_STATUS_"));
    }
    // Codes outside the 10.0.2 ABI are not guessed.
    for code in 0..64 {
        assert_eq!(Status::from_raw(code).is_some(), raw.contains(&code));
    }
    // Pessimistic categories: only a closed or requested gap is a successful stop, and
    // only it or a proof of infeasibility carries a native assurance.
    let category = |s| scip::termination(s).category;
    let assurance = |s| scip::termination(s).assurance;
    for s in [Status::Optimal, Status::GapLimit] {
        assert_eq!(category(s), Termination::Success);
        assert_eq!(assurance(s), Assurance::GlobalBound);
    }
    assert_eq!(category(Status::Infeasible), Termination::Infeasible);
    assert_eq!(assurance(Status::Infeasible), Assurance::ProvenInfeasible);
    assert_eq!(category(Status::UserInterrupt), Termination::Cancelled);
    assert_eq!(category(Status::TimeLimit), Termination::TimeLimit);
    assert_eq!(
        category(Status::MemoryLimit),
        Termination::ResourceExhausted
    );
    assert_eq!(category(Status::NodeLimit), Termination::Limit);
    assert_eq!(category(Status::SolutionLimit), Termination::SolutionLimit);
    assert_eq!(category(Status::DualLimit), Termination::ObjectiveLimit);
    assert_eq!(category(Status::Unknown), Termination::Inconclusive);
    for s in Status::ALL {
        if !matches!(s, Status::Optimal | Status::GapLimit | Status::Infeasible) {
            assert_eq!(assurance(s), Assurance::None, "{s:?}");
        }
    }
}

#[test]
fn scip_interrupt_via_event_handler() {
    let registry = standard_registry().unwrap();
    let case = synthesis(&registry);
    let program = case.program(&FactorableRequest::default());
    // The attempt's flag is raised before the solve; the handler interrupts at its first
    // event on the owning thread.
    let (status, events, interrupted) =
        scip::testing::raw_status(&program, SolveIntent::Optimize, &execution(true)).unwrap();
    assert_eq!(status, Status::UserInterrupt);
    assert!(events >= 1 && interrupted);
    // Without the flag the same model solves.
    let (status, _, interrupted) =
        scip::testing::raw_status(&program, SolveIntent::Optimize, &execution(false)).unwrap();
    assert!(matches!(status, Status::Optimal | Status::GapLimit));
    assert!(!interrupted);
    // Through the runner, a cancelled attempt is typed and claims nothing.
    let report = run(&case, &program, SolveIntent::Optimize, false, true).unwrap();
    assert_eq!(report.termination.category, Termination::Cancelled);
    assert_eq!(report.termination.name, "SCIP_STATUS_USERINTERRUPT");
    assert_ne!(report.qualification, Qualification::GapQualified);
    assert_ne!(report.termination.assurance, Assurance::GlobalBound);
}

#[test]
fn scip_export_readback_equivalent() {
    let registry = standard_registry().unwrap();
    let mut b = Body::new(&registry, 3);
    let (x, y, z) = (b.x[0].clone(), b.x[1].clone(), b.x[2].clone());
    // A nonlinear row over the whole factorable vocabulary.
    let e = b.f(Function::Exp, &x);
    let l = b.f(Function::Log, &y);
    let el = b.op(Binary::Mul, &e, &l);
    let s = b.f(Function::Sin, &x);
    let c = b.f(Function::Cos, &y);
    let root = b.f(Function::Sqrt, &y);
    let quotient = b.op(Binary::Div, &root, &x);
    let gap = b.op(Binary::Sub, &x, &y);
    let magnitude = b.f(Function::Abs, &gap);
    let low = b.b.extremum(true, x.clone(), y.clone(), id(205)).unwrap();
    let nonlinear = b.op(Binary::Add, &el, &s);
    let nonlinear = b.op(Binary::Sub, &nonlinear, &c);
    let nonlinear = b.op(Binary::Add, &nonlinear, &quotient);
    let nonlinear = b.op(Binary::Add, &nonlinear, &magnitude);
    let nonlinear = b.op(Binary::Add, &nonlinear, &low);
    // An affine row with a constant.
    let two = b.c(2.0);
    let seven = b.c(7.0);
    let twice = b.op(Binary::Mul, &two, &x);
    let affine = b.op(Binary::Sub, &twice, &z);
    let affine = b.op(Binary::Add, &affine, &seven);
    // A nonlinear objective, exported through its epigraph.
    let objective = b.op(Binary::Mul, &x, &z);
    let body = b.b.prepare(&[nonlinear, affine, objective]).unwrap();
    let boxes = [(0.5, 3.0), (0.5, 3.0), (-1.0, 2.0)];
    let columns: Vec<_> = boxes
        .iter()
        .map(|(l, u)| (VariableDomain::Continuous, Some(*l), Some(*u), *l))
        .collect();
    let case = case(
        &registry,
        body,
        &columns,
        &[(f64::NEG_INFINITY, 10.0), (1.0, 5.0)],
        Some((2, ObjectiveSense::Minimize)),
        DerivativeOrder::Value,
    );
    let program = case.program(&FactorableRequest::default());
    assert_eq!(program.fidelity(), Fidelity::Exact);
    let mut state = 7_u64;
    let mut next = |(l, u): (f64, f64)| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        l + (u - l) * ((state >> 11) as f64 / (1_u64 << 53) as f64)
    };
    let points: Vec<_> = (0..32)
        .map(|_| (boxes.iter().map(|b| next(*b)).collect::<Vec<_>>(), vec![]))
        .collect();
    let (deviation, linear, nonlinear) =
        scip::testing::readback_at(&program, SolveIntent::Optimize, &points).unwrap();
    assert!(deviation <= scip::READBACK_TOLERANCE, "{deviation:e}");
    // The affine row is a linear constraint; the rest, obligations and the epigraph are
    // nonlinear constraints.
    assert!(linear >= 1 && nonlinear >= 2, "{linear} {nonlinear}");
    // Negative control: a program with a changed constant reads back as different.
    let mut b = Body::new(&registry, 3);
    let (x, _, z) = (b.x[0].clone(), b.x[1].clone(), b.x[2].clone());
    let three = b.c(3.0);
    let seven = b.c(7.0);
    let thrice = b.op(Binary::Mul, &three, &x);
    let affine = b.op(Binary::Sub, &thrice, &z);
    let affine = b.op(Binary::Add, &affine, &seven);
    let body = b.b.prepare(&[affine]).unwrap();
    let changed = crate::scip_tests::case(
        &registry,
        body,
        &columns,
        &[(1.0, 5.0)],
        None,
        DerivativeOrder::Value,
    )
    .program(&FactorableRequest::default());
    let mut b = Body::new(&registry, 3);
    let (x, _, z) = (b.x[0].clone(), b.x[1].clone(), b.x[2].clone());
    let two = b.c(2.0);
    let seven = b.c(7.0);
    let twice = b.op(Binary::Mul, &two, &x);
    let affine = b.op(Binary::Sub, &twice, &z);
    let affine = b.op(Binary::Add, &affine, &seven);
    let body = b.b.prepare(&[affine]).unwrap();
    let original = crate::scip_tests::case(
        &registry,
        body,
        &columns,
        &[(1.0, 5.0)],
        None,
        DerivativeOrder::Value,
    )
    .program(&FactorableRequest::default());
    assert_eq!(original.nodes.len(), changed.nodes.len());
    let deviation = scip::testing::readback_against(&original, &changed, &points[..4]).unwrap();
    assert!(deviation > 1e-3, "{deviation:e}");
}

#[test]
fn scip_solves_small_nonconvex_nlp_globally() {
    let registry = standard_registry().unwrap();
    let case = quartic(&registry, (Some(-2.0), Some(2.0)));
    let program = case.program(&FactorableRequest::default());
    assert_eq!(program.fidelity(), Fidelity::Exact);
    let global = quartic_root(-1.3);
    let local = quartic_root(1.1);
    assert!(quartic_value(global) < quartic_value(local) - 2.0);
    // A local solve from the start point stays in the local basin.
    let controls = Controls::default();
    let tolerances = tolerances(2, 1);
    let normalization = Normalization::identity(2, 1);
    let ipopt = execution::nlp(
        Step {
            adapter: execution::adapter(Backend::Ipopt),
            settings: &BackendSettings::Default,
            controls: &controls,
            execution: execution(false),
            tolerances: &tolerances,
            normalization: &normalization,
            compatibility: stamp(Backend::Ipopt),
            warm: None,
        },
        &mut Retained::default(),
        execution::Nlp {
            oracle: case.continuous_oracle(),
            initial: &case.initial(),
            presolve: &crate::presolve::Policy::Auto,
            intent: SolveIntent::Optimize,
            sense: ObjectiveSense::Minimize,
            limit: 100_000,
        },
    )
    .unwrap();
    let x = ipopt.candidate.as_ref().unwrap().primal[0];
    assert!((x - local).abs() < 1e-5, "Ipopt {x}");
    assert_eq!(ipopt.qualification, Qualification::Stationary);
    // The certify route reaches the global optimum with a gap claim.
    let report = run(&case, &program, SolveIntent::Certify, true, false).unwrap();
    assert_eq!(report.backend, Backend::Scip);
    assert_eq!(report.termination.category, Termination::Success);
    let candidate = report.candidate.as_ref().unwrap();
    assert!(
        (candidate.primal[0] - global).abs() < 1e-4,
        "{:?}",
        candidate.primal
    );
    let g = report.evidence.global.unwrap();
    assert!(g.readback);
    assert_eq!(g.fidelity, Fidelity::Exact);
    assert_eq!(g.dual, BoundSource::ExactExport);
    assert!(matches!(
        g.primal,
        PrimalSource::Backend | PrimalSource::FixedAssignment
    ));
    let bound = g.dual_bound.unwrap();
    assert!((bound - quartic_value(global)).abs() < 1e-3, "{bound}");
    assert_eq!(report.qualification, Qualification::GapQualified);
    assert_eq!(report.termination.assurance, Assurance::GlobalBound);
    // Tolerances and the box are recorded with the claim.
    assert!(g.feasibility > 0.0 && g.gap_relative > 0.0 && g.gap_absolute > 0.0);
    assert!(g.nodes >= 1);
    assert_eq!(report.options["misc/catchctrlc"], OptionValue::Bool(false));
}

#[test]
fn scip_minlp_small_synthesis() {
    let registry = standard_registry().unwrap();
    let case = synthesis(&registry);
    let program = case.program(&FactorableRequest::default());
    assert_eq!(program.fidelity(), Fidelity::Exact);
    let report = run(&case, &program, SolveIntent::Optimize, true, false).unwrap();
    assert_eq!(report.termination.category, Termination::Success);
    let x = &report.candidate.as_ref().unwrap().primal;
    assert!((x[2] - 1.0).abs() < 1e-9, "{x:?}");
    assert!((x[0] - 3.0).abs() < 1e-5 && x[1].abs() < 1e-5, "{x:?}");
    let objective = report.observation.as_ref().unwrap().objective.unwrap();
    assert!((objective - 6.5).abs() < 1e-5, "{objective}");
    let g = report.evidence.global.unwrap();
    assert!((g.dual_bound.unwrap() - 6.5).abs() < 1e-3);
    assert_eq!(report.qualification, Qualification::GapQualified);
    assert_eq!(report.termination.assurance, Assurance::GlobalBound);
    // The integrality of the binary column was checked in original coordinates.
    assert_eq!(report.quality.as_ref().unwrap().integrality.len(), 1);
}

#[test]
fn minlp_candidate_from_fixed_assignment_resolve() {
    let registry = standard_registry().unwrap();
    let case = synthesis(&registry);
    let program = case.program(&FactorableRequest::default());
    let report = run(&case, &program, SolveIntent::Optimize, true, false).unwrap();
    let g = report.evidence.global.unwrap();
    // SCIP's incumbent was an assignment proposal; the candidate is the continuous
    // re-solve with y fixed, through the one NLP runner.
    assert_eq!(g.primal, PrimalSource::FixedAssignment);
    assert_eq!(g.dual, BoundSource::ExactExport);
    assert!(matches!(
        &report.metrics["resolve.backend"],
        crate::solve::Metric::Text(b) if b == "ipopt" || b == "pounce"
    ));
    assert!(report.metrics.contains_key("scip.incumbent.objective"));
    // The re-solve's multipliers are conditional on the assignment and recorded.
    assert!(report.evidence.kkt.is_some());
    let o = report.observation.as_ref().unwrap();
    assert!(o.stationarity.is_some());
    assert_eq!(report.qualification, Qualification::GapQualified);
    // Without the re-solve, the exact export's incumbent is the candidate.
    let direct = run(&case, &program, SolveIntent::Optimize, false, false).unwrap();
    assert_eq!(
        direct.evidence.global.unwrap().primal,
        PrimalSource::Backend
    );
    assert!(direct.evidence.kkt.is_none());
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
fn cube_spec(registry: &QuantityRegistry) -> ProviderSpec {
    ProviderSpec {
        shapes: pse_kernels::ProviderShapes::default(),
        derivative_source: pse_kernels::DerivativeSource::Analytic,
        id: id(40),
        revision: ContentHash::from_bytes([40; 32]),
        data: ContentHash::from_bytes([2; 32]),
        inputs: vec![port(registry, 41)],
        outputs: vec![port(registry, 42)],
        derivatives: DerivativeOrder::Value,
        smoothness: DerivativeOrder::Value,
    }
}
fn cube_providers() -> BTreeMap<ProviderKey, Box<dyn Provider>> {
    let registry = standard_registry().unwrap();
    let spec = cube_spec(&registry);
    let key = spec.key();
    let provider: Box<dyn Provider> = Box::new(Cube(spec));
    BTreeMap::from([(key, provider)])
}

#[test]
fn relaxed_export_bound_only() {
    // min cube(x) + y over x ∈ [0.5, 3], y ∈ [−1, 1], with the opaque provider exported
    // as an auxiliary inside its enforced envelope [0, 27]: a sound relaxation.
    let registry = standard_registry().unwrap();
    let spec = cube_spec(&registry);
    let admitted = AdmittedProvider::new(spec.clone(), &registry).unwrap();
    let mut b = Body::new(&registry, 2);
    let (x, y) = (b.x[0].clone(), b.x[1].clone());
    let cube =
        b.b.provider(&admitted, std::slice::from_ref(&x), id(210))
            .unwrap()
            .remove(0);
    let objective = b.op(Binary::Add, &cube, &y);
    let body = b.b.prepare(&[objective]).unwrap();
    let mut case = case(
        &registry,
        body,
        &[
            (VariableDomain::Continuous, Some(0.5), Some(3.0), 2.0),
            (VariableDomain::Continuous, Some(-1.0), Some(1.0), 0.0),
        ],
        &[],
        Some((0, ObjectiveSense::Minimize)),
        DerivativeOrder::Value,
    );
    case.providers = cube_providers;
    // Without an enforced envelope the objective yields no bound and is refused.
    let unbounded = case.program(&FactorableRequest::default());
    assert!(
        execution::admit_program(&unbounded, SolveIntent::Certify)
            .contains(&Refusal::ObjectiveUnavailable)
    );
    let request = FactorableRequest {
        envelopes: BTreeMap::from([(spec.key(), vec![(0.0, 27.0)])]),
        ..Default::default()
    };
    let program = case.program(&request);
    assert_eq!(program.fidelity(), Fidelity::Relaxed);
    let report = run(&case, &program, SolveIntent::Certify, false, false).unwrap();
    let g = report.evidence.global.unwrap();
    assert_eq!(g.fidelity, Fidelity::Relaxed);
    assert_eq!(g.dual, BoundSource::RelaxedExport);
    assert_eq!(g.primal, PrimalSource::RelaxedIncumbent);
    // The relaxation's bound is valid for the true optimum 0.125 − 1 ...
    let optimum = 0.5_f64.powi(3) - 1.0;
    let bound = g.dual_bound.unwrap();
    assert!(bound <= optimum + 1e-9, "{bound}");
    // ... and is the only claim: never optimality, never a gap from the relaxed point.
    assert_eq!(report.termination.assurance, Assurance::GlobalBound);
    assert!(!matches!(
        report.qualification,
        Qualification::GapQualified | Qualification::OptimalWithinTolerance
    ));
}

#[test]
fn unbounded_variable_refused_for_spatial_branching() {
    let registry = standard_registry().unwrap();
    let case = quartic(&registry, (None, Some(2.0)));
    let program = case.program(&FactorableRequest::default());
    let refusals = execution::admit_program(&program, SolveIntent::Certify);
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    let Refusal::UnboundedNonlinear(missing) = &refusals[0] else {
        panic!("{refusals:?}");
    };
    assert_eq!(missing.owner, BoundOwner::Variable(id(1)));
    assert!(missing.lower && !missing.upper);
    assert!(refusals[0].to_string().contains("spatial branching"));
    // The runner refuses before any native instance exists.
    assert!(matches!(
        run(&case, &program, SolveIntent::Certify, false, false),
        Err(ProblemError::Unsupported(m)) if m.contains("spatial branching")
    ));
    // A variable that enters only linearly may stay unbounded.
    let mut b = Body::new(&registry, 2);
    let (x, t) = (b.x[0].clone(), b.x[1].clone());
    let xx = b.op(Binary::Mul, &x, &x);
    let row = b.op(Binary::Add, &t, &xx);
    let body = b.b.prepare(&[row, t.clone()]).unwrap();
    let linear = crate::scip_tests::case(
        &registry,
        body,
        &[
            (VariableDomain::Continuous, Some(-1.0), Some(1.0), 0.0),
            (VariableDomain::Continuous, None, None, 0.0),
        ],
        &[(0.0, f64::INFINITY)],
        Some((1, ObjectiveSense::Minimize)),
        DerivativeOrder::Value,
    )
    .program(&FactorableRequest::default());
    assert!(execution::admit_program(&linear, SolveIntent::Certify).is_empty());
}

#[test]
fn scip_internal_ipopt_uses_typed_linear_solver() {
    let controls = Controls::default();
    // The nested Ipopt's linear solver comes from the typed setting, default MUMPS.
    let defaults = scip::testing::configured(&ScipSettings::default(), &controls).unwrap();
    assert_eq!(
        defaults["nlpi/ipopt/linear_solver"],
        OptionValue::Text("mumps".into())
    );
    let pardiso = ScipSettings {
        nlp_linear_solver: IpoptLinearSolver::Pardisomkl,
        seed: 3,
        nodes: Some(1000),
    };
    let options = scip::testing::configured(&pardiso, &controls).unwrap();
    assert_eq!(
        options["nlpi/ipopt/linear_solver"],
        OptionValue::Text("pardisomkl".into())
    );
    assert_eq!(
        options["randomization/randomseedshift"],
        OptionValue::Integer(3)
    );
    // Every reserved option is read back from the instance.
    assert_eq!(options["misc/catchctrlc"], OptionValue::Bool(false));
    assert_eq!(options["limits/memory"], OptionValue::Real(256.0));
    assert_eq!(options["lp/threads"], OptionValue::Integer(1));
    assert_eq!(
        options["limits/gap"],
        OptionValue::Real(controls.accuracy.mip_relative_gap)
    );
    // Free-form options cannot reach reserved or unadmitted native settings.
    for key in [
        "nlpi/ipopt/linear_solver",
        "nlpi/ipopt/hsllib",
        "nlpi/ipopt/pardisolib",
        "misc/catchctrlc",
        "parallel/maxnthreads",
    ] {
        let mut controls = Controls::default();
        controls
            .options
            .insert(key.into(), OptionValue::Text("x".into()));
        assert!(
            matches!(
                scip::testing::configured(&ScipSettings::default(), &controls),
                Err(ProblemError::Contract(_))
            ),
            "{key}"
        );
    }
    // SPRAL is admitted only with its process precondition.
    let spral = ScipSettings {
        nlp_linear_solver: IpoptLinearSolver::Spral,
        ..ScipSettings::default()
    };
    let cancellation =
        std::env::var("OMP_CANCELLATION").is_ok_and(|v| v.eq_ignore_ascii_case("true"));
    assert_eq!(spral.admit().is_ok(), cancellation);
    // Foreign settings and extra threads are refused by the adapter.
    let adapter = execution::adapter(Backend::Scip);
    assert!(
        adapter
            .admit_settings(&BackendSettings::Scip(pardiso), &Controls::default())
            .is_ok()
    );
    assert!(
        adapter
            .admit_settings(
                &BackendSettings::Default,
                &Controls {
                    threads: 2,
                    ..Controls::default()
                }
            )
            .is_err()
    );
}

#[test]
fn scip_cancel_flag_is_not_ambient() {
    // The flag belongs to the attempt: a second attempt with a fresh flag is unaffected.
    let flag = Arc::new(AtomicBool::new(false));
    let mut first = execution(false);
    first.cancel = flag.clone();
    flag.store(true, Ordering::Release);
    let registry = standard_registry().unwrap();
    let case = synthesis(&registry);
    let program = case.program(&FactorableRequest::default());
    let (status, ..) = scip::testing::raw_status(&program, SolveIntent::Optimize, &first).unwrap();
    assert_eq!(status, Status::UserInterrupt);
    let (status, ..) =
        scip::testing::raw_status(&program, SolveIntent::Optimize, &execution(false)).unwrap();
    assert_ne!(status, Status::UserInterrupt);
}
