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
        self, BackendSettings, Evaluation, Factorable, OriginalModel, Refusal, Resolve, Retained,
        ScipSettings, Step,
    },
    quality::Tolerances,
    scip::{self, Status},
    solve::{
        Assurance, Backend, BoundSource, Controls, Execution, IisMember, IpoptLinearSolver,
        OptionValue, PrimalSource, Qualification, ResolvedAccuracy, SolveIntent, SolveReport,
        Termination, WarmCapability,
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
        ObjectiveSense, Row, SlotBinding, Target, Variable,
    },
    factorable::{BoundOwner, FactorableProgram, FactorableRequest, Fidelity},
    jets::EvaluationLimits,
    library::Optimization,
    normalization::Normalization,
    typed::{Binary, BodyBuilder, BodyLimits, TypedValue},
};
use pse_model::{
    forms::{LogicOperand, NativeConstraint},
    generated::enums::{ModelingVariableDomain, NativeConstraintForm},
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
    columns: &[(ModelingVariableDomain, Option<f64>, Option<f64>, f64)],
    rows: &[(f64, f64)],
    objective: Option<(usize, ObjectiveSense)>,
    order: DerivativeOrder,
) -> Case {
    case_with(registry, body, columns, rows, objective, order, vec![])
}
/// As [`case`], with constraint forms left to native handlers (ADR-0104).
fn case_with(
    registry: &QuantityRegistry,
    body: pse_math::guarded::PreparedBody,
    columns: &[(ModelingVariableDomain, Option<f64>, Option<f64>, f64)],
    rows: &[(f64, f64)],
    objective: Option<(usize, ObjectiveSense)>,
    order: DerivativeOrder,
    native: Vec<NativeConstraint>,
) -> Case {
    case_of(
        registry, body, columns, &[], rows, objective, order, native,
    )
}
/// As [`case_with`], with case parameters `id(80 + j)` bound after the columns: body input
/// `columns.len() + j` takes the value `parameters[j]`.
#[expect(
    clippy::too_many_arguments,
    reason = "a test case binds its body, columns, parameters, rows, objective, order and native forms"
)]
fn case_of(
    registry: &QuantityRegistry,
    body: pse_math::guarded::PreparedBody,
    columns: &[(ModelingVariableDomain, Option<f64>, Option<f64>, f64)],
    parameters: &[f64],
    rows: &[(f64, f64)],
    objective: Option<(usize, ObjectiveSense)>,
    order: DerivativeOrder,
    native: Vec<NativeConstraint>,
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
    let parameter = |j: usize| port(registry, 80 + u8::try_from(j).unwrap());
    let slots = (0..columns.len())
        .map(|i| {
            let p = port(registry, u8::try_from(i + 1).unwrap());
            SlotBinding::new(&p, &p, registry).unwrap()
        })
        .chain((0..parameters.len()).map(|j| {
            let p = parameter(j);
            SlotBinding::new(&p, &p, registry).unwrap()
        }))
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
            (0..parameters.len()).map(parameter).collect(),
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
        .unwrap()
        .with_native(native)
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
            .chain(
                parameters
                    .iter()
                    .enumerate()
                    .map(|(j, v)| (id(80 + u8::try_from(j).unwrap()), *v)),
            )
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
        assignment: &BTreeMap<usize, (f64, f64)>,
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
    run_with(
        case,
        program,
        intent,
        resolve,
        cancel,
        &ScipSettings::default(),
        &Controls::default(),
        &mut Retained::default(),
    )
}
/// [`run`] with typed settings, controls and a retained session.
#[expect(
    clippy::too_many_arguments,
    reason = "a test run binds its case, program, intent, re-solve, cancellation, settings, controls and session"
)]
fn run_with(
    case: &Case,
    program: &FactorableProgram,
    intent: SolveIntent,
    resolve: bool,
    cancel: bool,
    settings: &ScipSettings,
    controls: &Controls,
    retained: &mut Retained,
) -> Result<SolveReport, ProblemError> {
    let n = program.variables.len();
    let m = program.rows.len();
    let accuracy = ResolvedAccuracy::nominal();
    let tolerances = tolerances(n, m);
    let normalization = Normalization::identity(n, m);
    let mut original = Original(case);
    let mut fixed = |a: &BTreeMap<usize, (f64, f64)>| case.fixed_oracle(a);
    let presolve = crate::presolve::Policy::Auto;
    let initial = case.initial();
    let mut execution = execution(cancel);
    execution.time_limit = controls.time_limit;
    execution::factorable(
        Step {
            adapter: execution::adapter(Backend::Scip),
            settings: &BackendSettings::Scip(settings.clone()),
            controls,
            accuracy: &accuracy,
            execution,
            tolerances: &tolerances,
            normalization: &normalization,
            compatibility: stamp(Backend::Scip),
            warm: None,
        },
        retained,
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
            (ModelingVariableDomain::Continuous, x_box.0, x_box.1, 1.0),
            (
                ModelingVariableDomain::Continuous,
                Some(0.0),
                Some(4.0),
                1.0,
            ),
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
            (
                ModelingVariableDomain::Continuous,
                Some(0.0),
                Some(4.0),
                0.0,
            ),
            (
                ModelingVariableDomain::Continuous,
                Some(0.0),
                Some(3.0),
                3.0,
            ),
            (ModelingVariableDomain::Binary, None, None, 0.0),
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
    // Concurrent solving takes the admitted permits (Plan 22 G7).
    assert!(row.certifies && row.parallel);
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
        .map(|(l, u)| (ModelingVariableDomain::Continuous, Some(*l), Some(*u), *l))
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
fn certify_known_global_optimum() {
    let registry = standard_registry().unwrap();
    let case = quartic(&registry, (Some(-2.0), Some(2.0)));
    let program = case.program(&FactorableRequest::default());
    assert_eq!(program.fidelity(), Fidelity::Exact);
    let global = quartic_root(-1.3);
    let local = quartic_root(1.1);
    assert!(quartic_value(global) < quartic_value(local) - 2.0);
    // A local solve from the start point stays in the local basin.
    let controls = Controls::default();
    let accuracy = ResolvedAccuracy::nominal();
    let tolerances = tolerances(2, 1);
    let normalization = Normalization::identity(2, 1);
    let ipopt = execution::nlp(
        Step {
            adapter: execution::adapter(Backend::Ipopt),
            settings: &BackendSettings::Default,
            controls: &controls,
            accuracy: &accuracy,
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
            analysis: execution::Analysis::for_intent(SolveIntent::Optimize),
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
    // Tolerances, fidelity and the box are recorded with the claim.
    assert!(g.feasibility > 0.0 && g.gap_relative > 0.0 && g.gap_absolute > 0.0);
    assert!(g.nodes >= 1);
    assert_eq!(report.options["misc/catchctrlc"], OptionValue::Bool(false));
    let record = report.global.as_ref().unwrap();
    assert_eq!(record.boxes, vec![(-2.0, 2.0), (0.0, 4.0)]);
    assert!(record.pool.is_empty() && record.iis.is_none());
    // A nonconvex QP certifies too: min x·y over [−1, 2]² with x + y ≤ 1.5 has its
    // global minimum −2 at two corners, while (0.5, 0.5) is a local saddle start.
    let mut b = Body::new(&registry, 2);
    let (x, y) = (b.x[0].clone(), b.x[1].clone());
    let sum = b.op(Binary::Add, &x, &y);
    let product = b.op(Binary::Mul, &x, &y);
    let body = b.b.prepare(&[sum, product]).unwrap();
    let qp = super::scip_tests::case(
        &registry,
        body,
        &[
            (
                ModelingVariableDomain::Continuous,
                Some(-1.0),
                Some(2.0),
                0.5,
            ),
            (
                ModelingVariableDomain::Continuous,
                Some(-1.0),
                Some(2.0),
                0.5,
            ),
        ],
        &[(f64::NEG_INFINITY, 1.5)],
        Some((1, ObjectiveSense::Minimize)),
        DerivativeOrder::Second,
    );
    let program = qp.program(&FactorableRequest::default());
    let report = run(&qp, &program, SolveIntent::Certify, true, false).unwrap();
    let g = report.evidence.global.unwrap();
    assert!((g.dual_bound.unwrap() + 2.0).abs() < 1e-6, "{g:?}");
    let objective = report.observation.as_ref().unwrap().objective.unwrap();
    assert!((objective + 2.0).abs() < 1e-6, "{objective}");
    assert_eq!(report.qualification, Qualification::GapQualified);
    assert_eq!(report.termination.assurance, Assurance::GlobalBound);
}

#[test]
fn small_synthesis_minlp_optimal() {
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

#[test]
fn fixed_assignment_resolve_keeps_local_analysis() {
    use crate::kkt::{Activity, Curvature, Licq, Side};
    use pse_math::index::{OriginalCol, OriginalRow};
    let registry = standard_registry().unwrap();
    let case = synthesis(&registry);
    let program = case.program(&FactorableRequest::default());
    let report = run(&case, &program, SolveIntent::Optimize, true, false).unwrap();
    assert_eq!(
        report.evidence.global.unwrap().primal,
        PrimalSource::FixedAssignment
    );
    // The re-solve's KKT-point analysis is carried over, conditional on y = 1 like its
    // multipliers: the demand row and s = 0 are strongly active, and the fixed binary is a
    // pinned bound row, so the three active gradients span the space.
    let point = match &report.evidence.local {
        Some(Ok(point)) => point,
        other => panic!("{other:?}"),
    };
    assert_eq!(point.rows[OriginalRow::new(0)], Activity::Strong(Side::Lower));
    assert_eq!(point.bounds[OriginalCol::new(0)], Activity::Inactive);
    assert_eq!(point.bounds[OriginalCol::new(1)], Activity::Strong(Side::Lower));
    assert_eq!(point.bounds[OriginalCol::new(2)], Activity::Strong(Side::Equal));
    assert_eq!(point.licq, Licq::Independent);
    assert_eq!(point.curvature, Curvature::Sufficient);
    assert_eq!((point.inertia, point.reduced), ((3, 3, 0), (0, 0, 0)));
    // The incumbent of an exact export without a re-solve has no local analysis.
    let direct = run(&case, &program, SolveIntent::Optimize, false, false).unwrap();
    assert!(direct.evidence.local.is_none());
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
            (
                ModelingVariableDomain::Continuous,
                Some(0.5),
                Some(3.0),
                2.0,
            ),
            (
                ModelingVariableDomain::Continuous,
                Some(-1.0),
                Some(1.0),
                0.0,
            ),
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
            (
                ModelingVariableDomain::Continuous,
                Some(-1.0),
                Some(1.0),
                0.0,
            ),
            (ModelingVariableDomain::Continuous, None, None, 0.0),
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
    let accuracy = ResolvedAccuracy::nominal();
    // The nested Ipopt's linear solver comes from the typed setting, default MUMPS.
    let defaults =
        scip::testing::configured(&ScipSettings::default(), &controls, &accuracy).unwrap();
    assert_eq!(
        defaults["nlpi/ipopt/linear_solver"],
        OptionValue::Text("mumps".into())
    );
    let pardiso = ScipSettings {
        nlp_linear_solver: IpoptLinearSolver::Pardisomkl,
        seed: 3,
        nodes: Some(1000),
        ..ScipSettings::default()
    };
    let options = scip::testing::configured(&pardiso, &controls, &accuracy).unwrap();
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
        OptionValue::Real(accuracy.mip_relative_gap)
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
                scip::testing::configured(&ScipSettings::default(), &controls, &accuracy),
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
    assert_eq!(spral.admit(1).is_ok(), cancellation);
    // Admitted permits run SCIP concurrently; modes that exclude one another are refused.
    let adapter = execution::adapter(Backend::Scip);
    let threads = Controls {
        threads: 2,
        ..Controls::default()
    };
    assert!(
        adapter
            .admit_settings(&BackendSettings::Scip(pardiso), &Controls::default())
            .is_ok()
    );
    assert!(
        adapter
            .admit_settings(&BackendSettings::Default, &threads)
            .is_ok()
    );
    for (settings, controls) in [
        (
            ScipSettings {
                exact: true,
                reoptimize: true,
                ..ScipSettings::default()
            },
            Controls::default(),
        ),
        (
            ScipSettings {
                exact: true,
                iis: true,
                ..ScipSettings::default()
            },
            Controls::default(),
        ),
        (
            ScipSettings {
                exact: true,
                ..ScipSettings::default()
            },
            threads.clone(),
        ),
        (
            ScipSettings {
                reoptimize: true,
                ..ScipSettings::default()
            },
            threads.clone(),
        ),
    ] {
        assert!(
            matches!(
                adapter.admit_settings(&BackendSettings::Scip(settings.clone()), &controls),
                Err(ProblemError::Unsupported(_))
            ),
            "{settings:?}"
        );
    }
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

/// x·x ≥ 4, x·x ≤ 1 and the redundant x·x ≤ 100 over x ∈ [−3, 3]: infeasible, with
/// the first two rows the only infeasible subsystem.
fn obstruction(registry: &QuantityRegistry) -> Case {
    let mut b = Body::new(registry, 1);
    let x = b.x[0].clone();
    let xx = b.op(Binary::Mul, &x, &x);
    let body = b.b.prepare(&[xx.clone(), xx.clone(), xx]).unwrap();
    case(
        registry,
        body,
        &[(
            ModelingVariableDomain::Continuous,
            Some(-3.0),
            Some(3.0),
            1.5,
        )],
        &[
            (4.0, f64::INFINITY),
            (f64::NEG_INFINITY, 1.0),
            (f64::NEG_INFINITY, 100.0),
        ],
        None,
        DerivativeOrder::Second,
    )
}

#[test]
fn global_infeasibility_proof() {
    let registry = standard_registry().unwrap();
    let case = obstruction(&registry);
    let program = case.program(&FactorableRequest::default());
    assert_eq!(program.fidelity(), Fidelity::Exact);
    let report = run(&case, &program, SolveIntent::Certify, false, false).unwrap();
    assert_eq!(report.termination.category, Termination::Infeasible);
    assert_eq!(report.termination.name, "SCIP_STATUS_INFEASIBLE");
    // A global conclusion over the declared box, tolerance-qualified (T10), never a
    // solution: no candidate and no qualification.
    assert_eq!(report.termination.assurance, Assurance::ProvenInfeasible);
    assert!(report.candidate.is_none());
    assert_eq!(report.qualification, Qualification::Unqualified);
    let g = report.evidence.global.unwrap();
    assert!(g.infeasible && g.readback && !g.exact);
    assert_eq!(g.dual, BoundSource::ExactExport);
    // Without the obstruction the same box is feasible and no proof is claimed.
    let mut b = Body::new(&registry, 1);
    let x = b.x[0].clone();
    let xx = b.op(Binary::Mul, &x, &x);
    let body = b.b.prepare(&[xx]).unwrap();
    let feasible = super::scip_tests::case(
        &registry,
        body,
        &[(
            ModelingVariableDomain::Continuous,
            Some(-3.0),
            Some(3.0),
            1.5,
        )],
        &[(4.0, f64::INFINITY)],
        None,
        DerivativeOrder::Second,
    );
    let program = feasible.program(&FactorableRequest::default());
    let report = run(&feasible, &program, SolveIntent::Certify, false, false).unwrap();
    assert_eq!(report.termination.category, Termination::Success);
    assert_ne!(report.termination.assurance, Assurance::ProvenInfeasible);
    assert_eq!(report.qualification, Qualification::Feasible);
}

#[test]
fn nonlinear_iis_irreducible_flag() {
    let registry = standard_registry().unwrap();
    let case = obstruction(&registry);
    let program = case.program(&FactorableRequest::default());
    let settings = ScipSettings {
        iis: true,
        ..ScipSettings::default()
    };
    let report = run_with(
        &case,
        &program,
        SolveIntent::Certify,
        false,
        false,
        &settings,
        &Controls::default(),
        &mut Retained::default(),
    )
    .unwrap();
    assert_eq!(report.termination.assurance, Assurance::ProvenInfeasible);
    let iis = report.global.as_ref().unwrap().iis.clone().unwrap();
    assert!(iis.irreducible, "{iis:?}");
    // Both obstructing rows and the kept bounds of the variable they use; not the
    // redundant row.
    assert_eq!(
        iis.members,
        vec![
            IisMember::Row(id(101)),
            IisMember::Row(id(102)),
            IisMember::VariableLower(id(1)),
            IisMember::VariableUpper(id(1)),
        ]
    );
    // Without the request no subsystem is computed.
    let report = run(&case, &program, SolveIntent::Certify, false, false).unwrap();
    assert!(report.global.as_ref().unwrap().iis.is_none());
}

#[test]
fn mip_iis_on_true_mip() {
    // 2x + 2y = 3 over integers x, y ∈ [0, 5] with x − y ≤ 10: the continuous
    // relaxation is feasible (x = 1.5, y = 0), the mixed-integer program is not.
    let registry = standard_registry().unwrap();
    let mut b = Body::new(&registry, 2);
    let (x, y) = (b.x[0].clone(), b.x[1].clone());
    let two = b.c(2.0);
    let tx = b.op(Binary::Mul, &two, &x);
    let ty = b.op(Binary::Mul, &two, &y);
    let parity = b.op(Binary::Add, &tx, &ty);
    let spread = b.op(Binary::Sub, &x, &y);
    let body = b.b.prepare(&[parity, spread]).unwrap();
    let columns = [
        (ModelingVariableDomain::Integer, Some(0.0), Some(5.0), 0.0),
        (ModelingVariableDomain::Integer, Some(0.0), Some(5.0), 0.0),
    ];
    let rows = [(3.0, 3.0), (f64::NEG_INFINITY, 10.0)];
    let case = case(
        &registry,
        body,
        &columns,
        &rows,
        None,
        DerivativeOrder::Value,
    );
    let program = case.program(&FactorableRequest::default());
    // The relaxation's feasible point exists, so an LP IIS would find nothing.
    let values = program.evaluate(&[1.5, 0.0], &[]).unwrap();
    let row = |r: usize| values[program.rows[r].expression.unwrap()];
    assert!((row(0) - 3.0).abs() < 1e-12 && row(1) <= 10.0);
    let settings = ScipSettings {
        iis: true,
        ..ScipSettings::default()
    };
    let report = run_with(
        &case,
        &program,
        SolveIntent::Certify,
        false,
        false,
        &settings,
        &Controls::default(),
        &mut Retained::default(),
    )
    .unwrap();
    assert_eq!(report.termination.category, Termination::Infeasible);
    assert_eq!(report.termination.assurance, Assurance::ProvenInfeasible);
    let iis = report.global.as_ref().unwrap().iis.clone().unwrap();
    assert!(iis.irreducible, "{iis:?}");
    assert!(iis.members.contains(&IisMember::Row(id(101))), "{iis:?}");
    assert!(!iis.members.contains(&IisMember::Row(id(102))), "{iis:?}");
}

/// Native forms over one small MILP (ADR-0104):
/// max x₁ + 2x₂ + 3x₃ − 5b − 0.5c + w₁ + w₂ subject to
/// - x₁ + x₂ + x₃ ≤ 10, with {x₁, x₂, x₃} ⊂ [0, 4] an SOS1 set;
/// - x₃ ≤ 1 while b = 0 (indicator);
/// - z = b ∨ c (logic) and z = 1;
/// - at most one of w₁, w₂ ∈ [0, 1] nonzero (cardinality).
///
/// Enumeration: b = 1 gives 12 − 5 + 1 = 8; b = 0 forces c = 1 and gives
/// 2·4 − 0.5 + 1 = 8.5, the optimum.
fn native_forms(registry: &QuantityRegistry) -> Case {
    native_forms_only(registry, &[0, 1, 2, 3])
}
fn native_forms_only(registry: &QuantityRegistry, keep: &[usize]) -> Case {
    let mut b = Body::new(registry, 8);
    let x = b.x.clone();
    let s12 = b.op(Binary::Add, &x[0], &x[1]);
    let supply = b.op(Binary::Add, &s12, &x[2]);
    let two = b.c(2.0);
    let three = b.c(3.0);
    let five = b.c(5.0);
    let half = b.c(0.5);
    let t2 = b.op(Binary::Mul, &two, &x[1]);
    let t3 = b.op(Binary::Mul, &three, &x[2]);
    let tb = b.op(Binary::Mul, &five, &x[3]);
    let tc = b.op(Binary::Mul, &half, &x[4]);
    let o = b.op(Binary::Add, &x[0], &t2);
    let o = b.op(Binary::Add, &o, &t3);
    let o = b.op(Binary::Sub, &o, &tb);
    let o = b.op(Binary::Sub, &o, &tc);
    let o = b.op(Binary::Add, &o, &x[6]);
    let o = b.op(Binary::Add, &o, &x[7]);
    let body =
        b.b.prepare(&[supply, x[2].clone(), x[5].clone(), o])
            .unwrap();
    let continuous = |upper| {
        (
            ModelingVariableDomain::Continuous,
            Some(0.0),
            Some(upper),
            0.0,
        )
    };
    let binary = (ModelingVariableDomain::Binary, None, None, 0.0);
    case_with(
        registry,
        body,
        &[
            continuous(4.0),
            continuous(4.0),
            continuous(4.0),
            binary,
            binary,
            binary,
            continuous(1.0),
            continuous(1.0),
        ],
        &[
            (f64::NEG_INFINITY, 10.0),
            (f64::NEG_INFINITY, 1.0),
            (1.0, 1.0),
        ],
        Some((3, ObjectiveSense::Maximize)),
        DerivativeOrder::Value,
        vec![
            NativeConstraint::Sos {
                form: NativeConstraintForm::Sos1,
                members: vec![(id(1), 1.0), (id(2), 2.0), (id(3), 3.0)],
            },
            NativeConstraint::Indicator {
                row: id(102),
                variable: id(4),
                active: false,
            },
            NativeConstraint::Logic {
                form: NativeConstraintForm::Or,
                resultant: id(6),
                operands: vec![
                    LogicOperand {
                        variable: id(4),
                        negated: false,
                    },
                    LogicOperand {
                        variable: id(5),
                        negated: false,
                    },
                ],
            },
            NativeConstraint::Cardinality {
                members: vec![id(7), id(8)],
                bound: 1,
            },
        ]
        .into_iter()
        .enumerate()
        .filter(|(k, _)| keep.contains(k))
        .map(|(_, c)| c)
        .collect(),
    )
}

#[test]
fn native_forms_consumed_by_scip() {
    let registry = standard_registry().unwrap();
    let case = native_forms(&registry);
    let program = case.program(&FactorableRequest::default());
    assert_eq!(program.native.len(), 4);
    assert_eq!(program.fidelity(), Fidelity::Exact);
    // The record consumes every form the registry declares.
    let record = execution::adapter(Backend::Scip).capability();
    for form in NativeConstraintForm::ALL {
        assert!(record.native_forms.contains(&form), "{form:?}");
    }
    let report = run(&case, &program, SolveIntent::Optimize, true, false).unwrap();
    assert_eq!(report.termination.category, Termination::Success);
    let x = &report.candidate.as_ref().unwrap().primal;
    let objective = report.observation.as_ref().unwrap().objective.unwrap();
    assert!((objective - 8.5).abs() < 1e-6, "{objective} {x:?}");
    // b = 0 leaves x₃ ≤ 1 enforced, c = 1 satisfies the asserted disjunction, and the
    // SOS1 and cardinality sets each hold one nonzero.
    assert!(x[3].abs() < 1e-9 && (x[4] - 1.0).abs() < 1e-9 && (x[5] - 1.0).abs() < 1e-9);
    assert!((x[1] - 4.0).abs() < 1e-6 && x[0].abs() < 1e-9 && x[2].abs() < 1e-9);
    assert!((x[6] + x[7] - 1.0).abs() < 1e-6 && x[6].min(x[7]).abs() < 1e-9);
    assert!(matches!(
        report.metrics["export.constraints.native"],
        crate::solve::Metric::Integer(3)
    ));
    // Original qualification checks every form's structure beside rows and integrality.
    let quality = report.quality.as_ref().unwrap();
    assert!(quality.feasible());
    assert_eq!(quality.integrality.len(), 3 + 3);
    assert_eq!(report.qualification, Qualification::GapQualified);
    // The indicator row is enforced while its binary column holds the inactive value.
    let plan = execution::factorable::plan(&program, SolveIntent::Optimize).unwrap();
    let condition = execution::factorable::Enforcement::When(execution::factorable::Condition {
        column: 3,
        active: false,
    });
    assert_eq!(plan.rows[1], condition);
    assert!(condition.enforced(&[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0]));
    assert!(!condition.enforced(&[0.0, 0.0, 4.0, 1.0, 0.0, 1.0, 0.0, 0.0]));
}

#[test]
fn solution_pool_ranked() {
    let registry = standard_registry().unwrap();
    let case = native_forms(&registry);
    let program = case.program(&FactorableRequest::default());
    let settings = ScipSettings {
        pool: 4,
        ..ScipSettings::default()
    };
    let report = run_with(
        &case,
        &program,
        SolveIntent::Optimize,
        false,
        false,
        &settings,
        &Controls::default(),
        &mut Retained::default(),
    )
    .unwrap();
    let pool = &report.global.as_ref().unwrap().pool;
    assert!(!pool.is_empty() && pool.len() <= 4, "{pool:?}");
    // Ranked best first for the maximization; every member is re-qualified in original
    // coordinates, and the first is the candidate.
    for (rank, solution) in pool.iter().enumerate() {
        assert_eq!(solution.rank, rank);
        assert_eq!(solution.feasible, Some(true), "{solution:?}");
    }
    for pair in pool.windows(2) {
        assert!(pair[0].objective.unwrap() >= pair[1].objective.unwrap() - 1e-9);
    }
    assert_eq!(pool[0].primal, report.candidate.as_ref().unwrap().primal);
    assert!((pool[0].objective.unwrap() - 8.5).abs() < 1e-6);
}

/// max x + y over binaries with 10⁹x + 10⁹y ≤ 2·10⁹ − 1: the exact optimum is 1, while
/// x = y = 1 violates the row by one unit, a relative 5·10⁻¹⁰ below the float
/// feasibility tolerance.
fn delicate(registry: &QuantityRegistry) -> Case {
    let mut b = Body::new(registry, 2);
    let (x, y) = (b.x[0].clone(), b.x[1].clone());
    let scale = b.c(1e9);
    let sx = b.op(Binary::Mul, &scale, &x);
    let sy = b.op(Binary::Mul, &scale, &y);
    let row = b.op(Binary::Add, &sx, &sy);
    let objective = b.op(Binary::Add, &x, &y);
    let body = b.b.prepare(&[row, objective]).unwrap();
    let binary = (ModelingVariableDomain::Binary, None, None, 0.0);
    case(
        registry,
        body,
        &[binary, binary],
        &[(f64::NEG_INFINITY, 2e9 - 1.0)],
        Some((1, ObjectiveSense::Maximize)),
        DerivativeOrder::Value,
    )
}

#[test]
fn exact_mode_on_delicate_milp() {
    let registry = standard_registry().unwrap();
    let case = delicate(&registry);
    let program = case.program(&FactorableRequest::default());
    let exact = ScipSettings {
        exact: true,
        ..ScipSettings::default()
    };
    let report = run_with(
        &case,
        &program,
        SolveIntent::Certify,
        false,
        false,
        &exact,
        &Controls::default(),
        &mut Retained::default(),
    )
    .unwrap();
    assert_eq!(report.termination.category, Termination::Success);
    let g = report.evidence.global.unwrap();
    assert!(g.exact && g.readback, "{g:?}");
    assert_eq!(report.termination.assurance, Assurance::ExactCertificate);
    assert_eq!(report.qualification, Qualification::OptimalWithinTolerance);
    let objective = report.observation.as_ref().unwrap().objective.unwrap();
    assert!((objective - 1.0).abs() < 1e-12, "{objective}");
    assert_eq!(
        report.global.as_ref().unwrap().exact_objective.as_deref(),
        Some("1")
    );
    // Floating-point solving may accept the violated point within its relative
    // tolerance; original-coordinate qualification then refuses it, so no claim
    // transfers, and no float claim is ever an exact certificate.
    let float = run(&case, &program, SolveIntent::Certify, false, false).unwrap();
    let x = &float.candidate.as_ref().unwrap().primal;
    if x[0] + x[1] > 1.5 {
        assert!(!float.quality.as_ref().unwrap().feasible());
        assert_ne!(float.qualification, Qualification::GapQualified);
    }
    assert_ne!(float.termination.assurance, Assurance::ExactCertificate);
    // Exact solving refuses what it cannot represent: a nonlinear program.
    let quartic = quartic(&registry, (Some(-2.0), Some(2.0)));
    let program = quartic.program(&FactorableRequest::default());
    assert!(matches!(
        run_with(
            &quartic,
            &program,
            SolveIntent::Certify,
            false,
            false,
            &exact,
            &Controls::default(),
            &mut Retained::default(),
        ),
        Err(ProblemError::Unsupported(m)) if m.contains("exact")
    ));
}

/// A price sequence on one commitment MILP: max Σ pₜ·xₜ − 3·Σ uₜ with xₜ ≤ 5uₜ,
/// x₁ + x₂ ≤ 8, xₜ ∈ [0, 5], uₜ binary. Only the prices change between steps.
fn commitment(registry: &QuantityRegistry, prices: [f64; 2]) -> Case {
    let mut b = Body::new(registry, 4);
    let x = b.x.clone();
    let five = b.c(5.0);
    let three = b.c(3.0);
    let mut outputs = vec![];
    for t in 0..2 {
        let cap = b.op(Binary::Mul, &five, &x[2 + t]);
        outputs.push(b.op(Binary::Sub, &x[t], &cap));
    }
    outputs.push(b.op(Binary::Add, &x[0], &x[1]));
    let p0 = b.c(prices[0]);
    let p1 = b.c(prices[1]);
    let r0 = b.op(Binary::Mul, &p0, &x[0]);
    let r1 = b.op(Binary::Mul, &p1, &x[1]);
    let on = b.op(Binary::Add, &x[2], &x[3]);
    let cost = b.op(Binary::Mul, &three, &on);
    let revenue = b.op(Binary::Add, &r0, &r1);
    outputs.push(b.op(Binary::Sub, &revenue, &cost));
    let body = b.b.prepare(&outputs).unwrap();
    let flow = (
        ModelingVariableDomain::Continuous,
        Some(0.0),
        Some(5.0),
        0.0,
    );
    let binary = (ModelingVariableDomain::Binary, None, None, 0.0);
    case(
        registry,
        body,
        &[flow, flow, binary, binary],
        &[
            (f64::NEG_INFINITY, 0.0),
            (f64::NEG_INFINITY, 0.0),
            (f64::NEG_INFINITY, 8.0),
        ],
        Some((3, ObjectiveSense::Maximize)),
        DerivativeOrder::Value,
    )
}
/// Enumerated optimum of [`commitment`]: each commitment fills the better price first.
fn commitment_optimum(prices: [f64; 2]) -> f64 {
    let mut order = [0, 1];
    order.sort_by(|a, b| prices[*b].total_cmp(&prices[*a]));
    [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]]
        .into_iter()
        .map(|u: [f64; 2]| {
            let mut left = 8.0_f64;
            let mut value = -3.0 * (u[0] + u[1]);
            for t in order {
                let x = (5.0 * u[t]).min(left);
                if prices[t] > 0.0 {
                    value += prices[t] * x;
                    left -= x;
                }
            }
            value
        })
        .fold(f64::NEG_INFINITY, f64::max)
}

/// [`commitment`] with the prices as case parameter values instead of body constants: one
/// prepared structure whose objective coefficients follow the values.
fn priced_commitment(registry: &QuantityRegistry, prices: [f64; 2]) -> Case {
    let mut b = Body::new(registry, 6);
    let x = b.x.clone();
    let five = b.c(5.0);
    let three = b.c(3.0);
    let mut outputs = vec![];
    for t in 0..2 {
        let cap = b.op(Binary::Mul, &five, &x[2 + t]);
        outputs.push(b.op(Binary::Sub, &x[t], &cap));
    }
    outputs.push(b.op(Binary::Add, &x[0], &x[1]));
    let r0 = b.op(Binary::Mul, &x[4], &x[0]);
    let r1 = b.op(Binary::Mul, &x[5], &x[1]);
    let on = b.op(Binary::Add, &x[2], &x[3]);
    let cost = b.op(Binary::Mul, &three, &on);
    let revenue = b.op(Binary::Add, &r0, &r1);
    outputs.push(b.op(Binary::Sub, &revenue, &cost));
    let body = b.b.prepare(&outputs).unwrap();
    let flow = (
        ModelingVariableDomain::Continuous,
        Some(0.0),
        Some(5.0),
        0.0,
    );
    let binary = (ModelingVariableDomain::Binary, None, None, 0.0);
    case_of(
        registry,
        body,
        &[flow, flow, binary, binary],
        &prices,
        &[
            (f64::NEG_INFINITY, 0.0),
            (f64::NEG_INFINITY, 0.0),
            (f64::NEG_INFINITY, 8.0),
        ],
        Some((3, ObjectiveSense::Maximize)),
        DerivativeOrder::Value,
        vec![],
    )
}

/// A reoptimization session is identified by the exported constraint system, never by the
/// values the objective consumes: prices passed as case parameter values change the
/// program's value identity at every step, yet each later step reuses the retained search
/// tree. (A changed constraint system rebuilds: `reoptimized_sequence_matches_cold_solves`.)
#[test]
fn reoptimization_session_follows_the_constraint_system() {
    let registry = standard_registry().unwrap();
    let settings = ScipSettings {
        reoptimize: true,
        ..ScipSettings::default()
    };
    let controls = Controls {
        reuse: crate::solve::ReusePolicy::AllowRebuild,
        ..Controls::default()
    };
    let mut retained = Retained::default();
    let mut keys = Vec::new();
    for (step, prices) in [[4.0, 1.0], [1.0, 4.0], [2.0, 2.0]].into_iter().enumerate() {
        let case = priced_commitment(&registry, prices);
        let program = case.program(&FactorableRequest::default());
        keys.push(program.key);
        let report = run_with(
            &case,
            &program,
            SolveIntent::Optimize,
            false,
            false,
            &settings,
            &controls,
            &mut retained,
        )
        .unwrap();
        let objective = report.observation.as_ref().unwrap().objective.unwrap();
        let expected = commitment_optimum(prices);
        assert!(
            (objective - expected).abs() < 1e-6,
            "{step}: {objective} vs {expected}"
        );
        assert_eq!(report.evidence.reused_native_state, step > 0, "{step}");
    }
    // The consumed prices are part of each program's value identity.
    assert!(keys[0] != keys[1] && keys[1] != keys[2]);
}

#[test]
fn reoptimized_sequence_matches_cold_solves() {
    let registry = standard_registry().unwrap();
    let settings = ScipSettings {
        reoptimize: true,
        ..ScipSettings::default()
    };
    let controls = Controls {
        reuse: crate::solve::ReusePolicy::AllowRebuild,
        ..Controls::default()
    };
    let mut retained = Retained::default();
    for (step, prices) in [[4.0, 1.0], [1.0, 4.0], [0.5, 0.2], [2.0, 2.0]]
        .into_iter()
        .enumerate()
    {
        let case = commitment(&registry, prices);
        let program = case.program(&FactorableRequest::default());
        let report = run_with(
            &case,
            &program,
            SolveIntent::Optimize,
            false,
            false,
            &settings,
            &controls,
            &mut retained,
        )
        .unwrap();
        let objective = report.observation.as_ref().unwrap().objective.unwrap();
        let expected = commitment_optimum(prices);
        assert!(
            (objective - expected).abs() < 1e-6,
            "{step}: {objective} vs {expected}"
        );
        assert_eq!(report.qualification, Qualification::GapQualified, "{step}");
        // The first step builds the session; later steps reuse its search tree.
        assert_eq!(report.evidence.reused_native_state, step > 0, "{step}");
        assert_eq!(report.global.as_ref().unwrap().reoptimized, step > 0);
        assert_eq!(retained.backend(), Some(Backend::Scip));
        // The cold solve agrees.
        let cold = run(&case, &program, SolveIntent::Optimize, false, false).unwrap();
        let cold = cold.observation.as_ref().unwrap().objective.unwrap();
        assert!((cold - objective).abs() < 1e-6, "{step}");
    }
    // A changed constraint system rebuilds instead of reusing.
    let case = delicate(&registry);
    let program = case.program(&FactorableRequest::default());
    let report = run_with(
        &case,
        &program,
        SolveIntent::Optimize,
        false,
        false,
        &settings,
        &controls,
        &mut retained,
    )
    .unwrap();
    assert!(!report.evidence.reused_native_state);
}

#[test]
fn concurrent_mode_under_admitted_permits() {
    let registry = standard_registry().unwrap();
    let case = synthesis(&registry);
    let program = case.program(&FactorableRequest::default());
    let controls = Controls {
        threads: 2,
        ..Controls::default()
    };
    let report = run_with(
        &case,
        &program,
        SolveIntent::Optimize,
        true,
        false,
        &ScipSettings::default(),
        &controls,
        &mut Retained::default(),
    )
    .unwrap();
    assert!(matches!(
        report.metrics["scip.threads"],
        crate::solve::Metric::Integer(2)
    ));
    assert_eq!(
        report.options["parallel/maxnthreads"],
        OptionValue::Integer(2)
    );
    assert_eq!(report.options["parallel/mode"], OptionValue::Integer(1));
    let objective = report.observation.as_ref().unwrap().objective.unwrap();
    assert!((objective - 6.5).abs() < 1e-5, "{objective}");
    assert_eq!(report.qualification, Qualification::GapQualified);
}
/// SCIP 10.0.2's additive IIS phase can leave a redundant row while reporting the
/// subsystem irreducible when the redundant row comes first; the deletion filter alone
/// minimizes it. Rows: x·x ≤ 100 (redundant), x·x ≤ 1 and x·x ≥ 4.
#[test]
fn iis_irreducible_whatever_the_row_order() {
    let registry = standard_registry().unwrap();
    let mut b = Body::new(&registry, 1);
    let x = b.x[0].clone();
    let xx = b.op(Binary::Mul, &x, &x);
    let one = b.c(1.0);
    let hundred = b.c(100.0);
    let four = b.c(4.0);
    let hi = b.op(Binary::Sub, &xx, &one);
    let spare = b.op(Binary::Sub, &xx, &hundred);
    let lo = b.op(Binary::Sub, &xx, &four);
    let body = b.b.prepare(&[spare, hi, lo]).unwrap();
    let case = case(
        &registry,
        body,
        &[(
            ModelingVariableDomain::Continuous,
            Some(-3.0),
            Some(3.0),
            1.5,
        )],
        &[
            (f64::NEG_INFINITY, 0.0),
            (f64::NEG_INFINITY, 0.0),
            (0.0, f64::INFINITY),
        ],
        None,
        DerivativeOrder::Second,
    );
    let program = case.program(&FactorableRequest::default());
    let settings = ScipSettings {
        iis: true,
        ..ScipSettings::default()
    };
    let report = run_with(
        &case,
        &program,
        SolveIntent::Certify,
        false,
        false,
        &settings,
        &Controls::default(),
        &mut Retained::default(),
    )
    .unwrap();
    let iis = report.global.as_ref().unwrap().iis.clone().unwrap();
    assert!(iis.irreducible);
    assert_eq!(
        iis.members,
        vec![
            IisMember::Row(id(102)),
            IisMember::Row(id(103)),
            IisMember::VariableLower(id(1)),
            IisMember::VariableUpper(id(1)),
        ]
    );
}

/// min (x − 3)² + b/2 over x ∈ [0, 5] with x·x ≤ 4 enforced only while b = 0: running
/// unconstrained (b = 1) costs 0.5, the constrained alternative (x = 2) costs 1.
#[test]
fn indicator_on_nonlinear_row_unenforced_in_fixed_assignment_resolve() {
    let registry = standard_registry().unwrap();
    let mut b = Body::new(&registry, 2);
    let (x, on) = (b.x[0].clone(), b.x[1].clone());
    let xx = b.op(Binary::Mul, &x, &x);
    let three = b.c(3.0);
    let half = b.c(0.5);
    let gap = b.op(Binary::Sub, &x, &three);
    let square = b.op(Binary::Mul, &gap, &gap);
    let penalty = b.op(Binary::Mul, &half, &on);
    let objective = b.op(Binary::Add, &square, &penalty);
    let body = b.b.prepare(&[xx, objective]).unwrap();
    let case = case_with(
        &registry,
        body,
        &[
            (
                ModelingVariableDomain::Continuous,
                Some(0.0),
                Some(5.0),
                1.0,
            ),
            (ModelingVariableDomain::Binary, None, None, 0.0),
        ],
        &[(f64::NEG_INFINITY, 4.0)],
        Some((1, ObjectiveSense::Minimize)),
        DerivativeOrder::Second,
        vec![NativeConstraint::Indicator {
            row: id(101),
            variable: id(2),
            active: false,
        }],
    );
    let program = case.program(&FactorableRequest::default());
    let report = run(&case, &program, SolveIntent::Optimize, true, false).unwrap();
    // The nonlinear row is lifted through a slack the indicator zeroes; the candidate is the continuous
    // re-solve with b fixed at 1, where the row is not enforced.
    assert!(matches!(
        report.metrics["export.constraints.nonlinear"],
        crate::solve::Metric::Integer(n) if n >= 1
    ));
    let g = report.evidence.global.unwrap();
    assert_eq!(
        g.primal,
        PrimalSource::FixedAssignment,
        "{:?}",
        report.metrics
    );
    let x = &report.candidate.as_ref().unwrap().primal;
    assert!(
        (x[1] - 1.0).abs() < 1e-9 && (x[0] - 3.0).abs() < 1e-5,
        "{x:?}"
    );
    let objective = report.observation.as_ref().unwrap().objective.unwrap();
    assert!((objective - 0.5).abs() < 1e-6, "{objective}");
    // The unenforced row is unconstrained in original qualification.
    let observation = report.observation.as_ref().unwrap();
    assert_eq!(observation.bounds[0], (f64::NEG_INFINITY, f64::INFINITY));
    assert!(report.quality.as_ref().unwrap().feasible());
    assert_eq!(report.qualification, Qualification::GapQualified);
}

/// A strongly correlated 0-1 knapsack, `max Σ (wᵢ + 10)·xᵢ + 100` with `Σ wᵢ·xᵢ ≤ ½·Σ wᵢ`:
/// small, but it needs a branch-and-bound search, and its objective carries a constant.
fn knapsack_with_constant(registry: &QuantityRegistry) -> Case {
    knapsack_of(registry, 24)
}
/// [`knapsack_with_constant`] over `items` items.
fn knapsack_of(registry: &QuantityRegistry, items: usize) -> Case {
    let weights: Vec<f64> = (0..items).map(|i| 30.0 + ((i * 37) % 71) as f64).collect();
    let mut b = Body::new(registry, items);
    let x = b.x.clone();
    let mut load = None;
    let mut value = None;
    for (i, w) in weights.iter().enumerate() {
        let weight = b.c(*w);
        let term = b.op(Binary::Mul, &weight, &x[i]);
        load = Some(match load {
            Some(sum) => b.op(Binary::Add, &sum, &term),
            None => term,
        });
        let price = b.c(*w + 10.0);
        let term = b.op(Binary::Mul, &price, &x[i]);
        value = Some(match value {
            Some(sum) => b.op(Binary::Add, &sum, &term),
            None => term,
        });
    }
    let constant = b.c(100.0);
    let objective = b.op(Binary::Add, &value.unwrap(), &constant);
    let body = b.b.prepare(&[load.unwrap(), objective]).unwrap();
    let binary = (ModelingVariableDomain::Binary, None, None, 0.0);
    case(
        registry,
        body,
        &vec![binary; items],
        &[(f64::NEG_INFINITY, 0.5 * weights.iter().sum::<f64>())],
        Some((1, ObjectiveSense::Maximize)),
        DerivativeOrder::Value,
    )
}

/// Collects every event a solve reports, as a durable stream would.
#[derive(Debug, Default)]
struct Collected(std::sync::Mutex<Vec<crate::solve::Event>>);
impl crate::solve::ProgressTap for Collected {
    fn observe(&self, event: &crate::solve::Event) {
        self.0.lock().unwrap().push(event.clone());
    }
}

/// SCIP streams typed incumbents under the post-solve objective convention (Plan 22 G8):
/// the objective constant applied whether SCIP holds it as its objective offset or the
/// export holds it outside SCIP (reoptimization), solutions in program columns, and the
/// last incumbent equal to the result. Dual-bound events carry the same offset; a new best
/// solution is reported once, typed, not also as a bound event.
#[test]
fn scip_incumbent_events_apply_offset() {
    let registry = standard_registry().unwrap();
    let case = knapsack_with_constant(&registry);
    let program = case.program(&FactorableRequest::default());
    let n = program.variables.len();
    let m = program.rows.len();
    for reoptimize in [false, true] {
        let settings = ScipSettings {
            reoptimize,
            ..ScipSettings::default()
        };
        let controls = Controls::default();
        let accuracy = ResolvedAccuracy::nominal();
        let tolerances = tolerances(n, m);
        let normalization = Normalization::identity(n, m);
        let mut original = Original(&case);
        let initial = case.initial();
        let tap = Arc::new(Collected::default());
        let mut execution = execution(false);
        execution.progress = Arc::new(crate::solve::Progress::tapped(
            controls.history,
            tap.clone(),
        ));
        let report = execution::factorable(
            Step {
                adapter: execution::adapter(Backend::Scip),
                settings: &BackendSettings::Scip(settings),
                controls: &controls,
                accuracy: &accuracy,
                execution,
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
                resolve: None,
            },
        )
        .unwrap();
        assert_eq!(
            report.qualification,
            Qualification::GapQualified,
            "{reoptimize}"
        );
        let result = report.candidate.as_ref().unwrap().objective.unwrap();
        let events = tap.0.lock().unwrap().clone();
        let incumbents: Vec<&crate::solve::IncumbentEvent> =
            events.iter().filter_map(|e| e.incumbent.as_ref()).collect();
        assert!(!incumbents.is_empty(), "{reoptimize}: {events:?}");
        // The first solution is captured at once; each captured solution evaluates, in the
        // original model, to its reported objective, constant included.
        assert!(incumbents[0].primal.is_some());
        for incumbent in &incumbents {
            if let Some(primal) = &incumbent.primal {
                assert_eq!(primal.len(), n);
                let value = Original(&case).evaluate(primal).unwrap().objective.unwrap();
                assert!(
                    (value - incumbent.objective).abs() < 1e-6,
                    "{reoptimize}: {value} vs {}",
                    incumbent.objective
                );
            }
            // A maximization's dual bound is an upper bound on every incumbent.
            assert!(
                incumbent
                    .dual_bound
                    .is_none_or(|d| d >= incumbent.objective - 1e-6)
            );
            // The empty knapsack is worth the constant alone.
            assert!(incumbent.objective >= 100.0 - 1e-9);
        }
        for pair in incumbents.windows(2) {
            assert!(
                pair[1].objective >= pair[0].objective - 1e-9,
                "{incumbents:?}"
            );
        }
        let last = incumbents.last().unwrap();
        assert!(
            (last.objective - result).abs() < 1e-6,
            "{reoptimize}: {} vs {result}",
            last.objective
        );
        assert_eq!(
            report.evidence.global.as_ref().unwrap().primal_bound,
            Some(result)
        );
        // Bound events: offset applied (every dual bound bounds the optimum from above)
        // and never a new best solution restated.
        for event in events.iter().filter(|e| e.phase == "scip.bound") {
            assert!(event.incumbent.is_none());
            if let Some(crate::solve::Metric::Real(dual)) = event.values.get("dual_bound") {
                assert!(*dual >= result - 1e-6, "{reoptimize}: {dual} < {result}");
            }
        }
        assert!(
            events
                .iter()
                .all(|e| e.incumbent.is_none() || e.phase == "scip.incumbent")
        );
    }
}

/// Semicontinuous supply: min price·s + 3p  s.t.  s + p ≥ demand, s ∈ {0} ∪ [lower, 5]
/// (semi-integer when `domain` says so), p ∈ [0, 3].
fn semi_supply(
    registry: &QuantityRegistry,
    domain: ModelingVariableDomain,
    lower: f64,
    price: f64,
    demand: f64,
) -> Case {
    let mut b = Body::new(registry, 2);
    let (s, p) = (b.x[0].clone(), b.x[1].clone());
    let row = b.op(Binary::Add, &s, &p);
    let price = b.c(price);
    let three = b.c(3.0);
    let supply = b.op(Binary::Mul, &price, &s);
    let purchase = b.op(Binary::Mul, &three, &p);
    let objective = b.op(Binary::Add, &supply, &purchase);
    let body = b.b.prepare(&[row, objective]).unwrap();
    case(
        registry,
        body,
        &[
            (domain, Some(lower), Some(5.0), 0.0),
            (
                ModelingVariableDomain::Continuous,
                Some(0.0),
                Some(3.0),
                3.0,
            ),
        ],
        &[(demand, f64::INFINITY)],
        Some((1, ObjectiveSense::Minimize)),
        DerivativeOrder::Second,
    )
}
/// The same case on HiGHS, which consumes semi domains natively.
fn highs_native(case: &Case) -> SolveReport {
    let cancel = Arc::new(AtomicBool::new(false));
    let coefficients = case
        .assembly
        .coefficients(&case.values, 100, &cancel)
        .unwrap();
    let constants = coefficients.row_constants.clone();
    let problem = crate::CoefficientProblem::from_plan(&case.assembly, coefficients).unwrap();
    let n = problem.contract.variables.len();
    let m = problem.bounds.len();
    let controls = Controls::default();
    let accuracy = ResolvedAccuracy::nominal();
    let tolerances = tolerances(n, m);
    let normalization = Normalization::identity(n, m);
    let row_bounds = case
        .assembly
        .structure()
        .rows()
        .iter()
        .map(|r| (r.lower, r.upper))
        .collect();
    execution::coefficients(
        Step {
            adapter: execution::adapter(Backend::Highs),
            settings: &BackendSettings::Default,
            controls: &controls,
            accuracy: &accuracy,
            execution: execution(false),
            tolerances: &tolerances,
            normalization: &normalization,
            compatibility: stamp(Backend::Highs),
            warm: None,
        },
        &mut Retained::default(),
        execution::Coefficients {
            problem: &problem,
            certificate: None,
            row_constants: &constants,
            row_bounds,
            original: &mut Original(case),
        },
    )
    .unwrap()
}

#[test]
fn semi_indicator_lowering_matches_highs_native() {
    let registry = standard_registry().unwrap();
    // Demand 1.5 takes the zero branch (s = 0, p = 1.5: 4.5 against 5 at s = 2; the
    // continuous relaxation would reach 3.75), demand 3 the active branch (s = 3: 7.5).
    for (demand, expected, supply) in [(1.5, 4.5, 0.0), (3.0, 7.5, 3.0)] {
        let case = semi_supply(
            &registry,
            ModelingVariableDomain::Semicontinuous,
            2.0,
            2.5,
            demand,
        );
        let program = case.program(&FactorableRequest::default());
        assert!(execution::admit_program(&program, SolveIntent::Optimize).is_empty());
        let scip = run(&case, &program, SolveIntent::Optimize, true, false).unwrap();
        assert_eq!(scip.termination.category, Termination::Success);
        let x = &scip.candidate.as_ref().unwrap().primal;
        assert!((x[0] - supply).abs() < 1e-6, "{demand}: {x:?}");
        let objective = scip.observation.as_ref().unwrap().objective.unwrap();
        assert!((objective - expected).abs() < 1e-6, "{demand}: {objective}");
        assert_eq!(scip.qualification, Qualification::GapQualified, "{demand}");
        let quality = scip.quality.as_ref().unwrap();
        assert!(quality.feasible());
        // HiGHS solves the semi domain natively to the same optimum.
        let highs = highs_native(&case);
        assert_eq!(highs.backend, Backend::Highs);
        let y = &highs.candidate.as_ref().unwrap().primal;
        let native = highs.observation.as_ref().unwrap().objective.unwrap();
        assert!((native - objective).abs() < 1e-6, "{demand}: {native} {objective}");
        assert!((y[0] - x[0]).abs() < 1e-6, "{demand}: {y:?} {x:?}");
    }
}

#[test]
fn semiinteger_lowering_keeps_integrality() {
    // n ∈ {0} ∪ {2, …, 5} with demand 3.5 and p ≤ 3: n = 3, p = 0.5 costs 9, while a
    // semicontinuous n = 3.5 would cost 8.75 and n = 0 is infeasible.
    let registry = standard_registry().unwrap();
    let case = semi_supply(
        &registry,
        ModelingVariableDomain::Semiinteger,
        2.0,
        2.5,
        3.5,
    );
    let program = case.program(&FactorableRequest::default());
    let plan = execution::factorable::plan(&program, SolveIntent::Optimize).unwrap();
    assert_eq!(plan.semi.len(), 1);
    assert!(plan.discrete());
    let report = run(&case, &program, SolveIntent::Optimize, true, false).unwrap();
    let x = &report.candidate.as_ref().unwrap().primal;
    assert!((x[0] - 3.0).abs() < 1e-9, "{x:?}");
    assert!((x[1] - 0.5).abs() < 1e-6, "{x:?}");
    let objective = report.observation.as_ref().unwrap().objective.unwrap();
    assert!((objective - 9.0).abs() < 1e-6, "{objective}");
    assert_eq!(report.qualification, Qualification::GapQualified);
    // The semi-integer column keeps its integrality check beside its domain check.
    let quality = report.quality.as_ref().unwrap();
    assert_eq!(quality.integrality.len(), 1);
    assert_eq!(quality.integrality[0].id, id(1));
    let record = report.global.as_ref().unwrap();
    assert!(matches!(
        record.transformations[..],
        [crate::solve::ExportTransformation::SemiIndicator { integer: true, .. }]
    ));
}

/// min w·s² + 4p  s.t.  s + p ≥ 3, s ∈ {0} ∪ [2, 5], p ∈ [0, 4]. Over the relaxed box the
/// optimum s = 2/w falls in the gap (0, 2): w = 1.5 takes the active branch at its lower
/// end (s = 2, p = 1: 10 against 12), w = 3 the zero branch (s = 0, p = 3: 12 against 16).
fn semi_process(registry: &QuantityRegistry, weight: f64) -> Case {
    let mut b = Body::new(registry, 2);
    let (s, p) = (b.x[0].clone(), b.x[1].clone());
    let row = b.op(Binary::Add, &s, &p);
    let w = b.c(weight);
    let four = b.c(4.0);
    let ss = b.op(Binary::Mul, &s, &s);
    let heat = b.op(Binary::Mul, &w, &ss);
    let purchase = b.op(Binary::Mul, &four, &p);
    let objective = b.op(Binary::Add, &heat, &purchase);
    let body = b.b.prepare(&[row, objective]).unwrap();
    case(
        registry,
        body,
        &[
            (
                ModelingVariableDomain::Semicontinuous,
                Some(2.0),
                Some(5.0),
                0.0,
            ),
            (
                ModelingVariableDomain::Continuous,
                Some(0.0),
                Some(4.0),
                3.0,
            ),
        ],
        &[(3.0, f64::INFINITY)],
        Some((1, ObjectiveSense::Minimize)),
        DerivativeOrder::Second,
    )
}

#[test]
fn semi_minlp_fixed_assignment_resolve() {
    use crate::kkt::{Activity, Side};
    use pse_math::index::OriginalCol;
    let registry = standard_registry().unwrap();
    for (weight, supply, expected, activity) in [
        (1.5, 2.0, 10.0, Activity::Strong(Side::Lower)),
        (3.0, 0.0, 12.0, Activity::Strong(Side::Equal)),
    ] {
        let case = semi_process(&registry, weight);
        let program = case.program(&FactorableRequest::default());
        let report = run(&case, &program, SolveIntent::Optimize, true, false).unwrap();
        let g = report.evidence.global.unwrap();
        // A nonlinear program with a lowered semi column is mixed-integer: SCIP's incumbent
        // is an assignment proposal and the candidate the continuous re-solve.
        assert_eq!(g.primal, PrimalSource::FixedAssignment, "{weight}");
        let x = &report.candidate.as_ref().unwrap().primal;
        assert!((x[0] - supply).abs() < 1e-6, "{weight}: {x:?}");
        let objective = report.observation.as_ref().unwrap().objective.unwrap();
        assert!((objective - expected).abs() < 1e-5, "{weight}: {objective}");
        assert_eq!(report.qualification, Qualification::GapQualified, "{weight}");
        // The re-solve committed the branch: z = 1 keeps s in [2, 5], where the lower end
        // binds (a relaxed [0, 5] would reach s = 2/w); z = 0 pins s at zero.
        let point = match &report.evidence.local {
            Some(Ok(point)) => point,
            other => panic!("{weight}: {other:?}"),
        };
        assert_eq!(point.bounds[OriginalCol::new(0)], activity, "{weight}");
    }
    // Original qualification measures the distance to {0} ∪ [2, 5]: the relaxed optimum
    // s = 4/3 of w = 1.5 violates the semi domain by 2/3.
    let case = semi_process(&registry, 1.5);
    let program = case.program(&FactorableRequest::default());
    let plan = execution::factorable::plan(&program, SolveIntent::Optimize).unwrap();
    let gap = 4.0 / 3.0;
    let (quality, _) = execution::factorable::assess(
        &plan,
        &mut Original(&case),
        &tolerances(2, 1),
        &[gap, 3.0 - gap],
    )
    .unwrap();
    assert!(!quality.feasible());
    assert!((quality.bounds[0].physical - 2.0 / 3.0).abs() < 1e-12);
    for (x, violation) in [(0.0, 0.0), (0.5, 0.5), (1.5, 0.5), (3.0, 0.0), (6.0, 1.0)] {
        assert!((plan.semi[0].violation(x) - violation).abs() < 1e-12, "{x}");
    }
}

#[test]
fn semi_transformation_recorded() {
    let registry = standard_registry().unwrap();
    let case = semi_supply(
        &registry,
        ModelingVariableDomain::Semicontinuous,
        2.0,
        2.5,
        1.5,
    );
    let program = case.program(&FactorableRequest::default());
    // The projection keeps the zero branch in the box and the active interval beside it.
    assert_eq!(
        (program.variables[0].lower, program.variables[0].active_lower),
        (0.0, Some(2.0))
    );
    let plan = execution::factorable::plan(&program, SolveIntent::Optimize).unwrap();
    assert_eq!(plan.boxes, vec![(0.0, 5.0), (0.0, 3.0), (0.0, 1.0)]);
    assert_eq!(plan.links.len(), 2);
    assert!(plan.constraints[1..].iter().all(|c| matches!(
        c.origin,
        execution::factorable::Origin::SemiLink { semi: 0, .. }
    )));
    // The active interval is part of the branched domain's identity.
    let narrower = semi_supply(
        &registry,
        ModelingVariableDomain::Semicontinuous,
        2.5,
        2.5,
        1.5,
    )
    .program(&FactorableRequest::default());
    let other = execution::factorable::plan(&narrower, SolveIntent::Optimize).unwrap();
    assert_eq!(other.boxes, plan.boxes);
    assert_ne!(other.domain, plan.domain);
    let report = run(&case, &program, SolveIntent::Optimize, true, false).unwrap();
    let record = report.global.as_ref().unwrap();
    assert_eq!(
        record.transformations,
        vec![crate::solve::ExportTransformation::SemiIndicator {
            variable: id(1),
            lower: 2.0,
            upper: 5.0,
            integer: false,
        }]
    );
    // The recorded box is the declared one; the indicator belongs to the transformation.
    assert_eq!(record.boxes, vec![(0.0, 5.0), (0.0, 3.0)]);
    assert!(matches!(
        report.metrics["export.lowered.semi_indicator"],
        crate::solve::Metric::Integer(1)
    ));
    assert!(matches!(
        report.metrics["export.constraints.linear"],
        crate::solve::Metric::Integer(3)
    ));
    assert!(report.evidence.global.unwrap().readback);
    // An infeasible subsystem maps the links back to the semi column's declared bounds:
    // s ≥ 0.5 excludes the zero branch and s ≤ 1 the active one, while the continuous
    // relaxation of the indicator is feasible.
    let b = Body::new(&registry, 1);
    let s = b.x[0].clone();
    let body = b.b.prepare(&[s.clone(), s]).unwrap();
    let blocked = super::scip_tests::case(
        &registry,
        body,
        &[(
            ModelingVariableDomain::Semicontinuous,
            Some(2.0),
            Some(5.0),
            0.0,
        )],
        &[(0.5, f64::INFINITY), (f64::NEG_INFINITY, 1.0)],
        None,
        DerivativeOrder::Value,
    );
    let program = blocked.program(&FactorableRequest::default());
    let settings = ScipSettings {
        iis: true,
        ..ScipSettings::default()
    };
    let report = run_with(
        &blocked,
        &program,
        SolveIntent::Certify,
        false,
        false,
        &settings,
        &Controls::default(),
        &mut Retained::default(),
    )
    .unwrap();
    assert_eq!(report.termination.assurance, Assurance::ProvenInfeasible);
    let iis = report.global.as_ref().unwrap().iis.clone().unwrap();
    assert_eq!(
        iis.members,
        vec![
            IisMember::Row(id(101)),
            IisMember::Row(id(102)),
            IisMember::VariableLower(id(1)),
            IisMember::VariableUpper(id(1)),
        ],
        "{iis:?}"
    );
}

#[test]
fn exact_mode_accepts_semi_lowering() {
    // The lowering adds a binary and two linear rows, which exact solving represents.
    let registry = standard_registry().unwrap();
    let case = semi_supply(
        &registry,
        ModelingVariableDomain::Semicontinuous,
        2.0,
        2.5,
        1.5,
    );
    let program = case.program(&FactorableRequest::default());
    let exact = ScipSettings {
        exact: true,
        ..ScipSettings::default()
    };
    let report = run_with(
        &case,
        &program,
        SolveIntent::Certify,
        false,
        false,
        &exact,
        &Controls::default(),
        &mut Retained::default(),
    )
    .unwrap();
    assert_eq!(report.termination.category, Termination::Success);
    let g = report.evidence.global.unwrap();
    assert!(g.exact && g.readback, "{g:?}");
    assert_eq!(report.termination.assurance, Assurance::ExactCertificate);
    assert_eq!(report.qualification, Qualification::OptimalWithinTolerance);
    let x = &report.candidate.as_ref().unwrap().primal;
    assert!(x[0] == 0.0 && (x[1] - 1.5).abs() < 1e-12, "{x:?}");
    let record = report.global.as_ref().unwrap();
    assert_eq!(record.exact_objective.as_deref(), Some("9/2"));
    assert_eq!(record.transformations.len(), 1);
}

/// SCIP optimizes a nonlinear objective through its epigraph variable, whose value in a
/// stored solution only bounds the function: SCIP's trivial heuristic stores, for instance,
/// x = 2 with the epigraph at a large value. Every reported objective (the candidate's, each
/// streamed incumbent's and each pooled solution's) is the function at the solution.
#[test]
fn epigraph_incumbent_reports_function_value() {
    // min (x − 1)² − 3 over x ∈ [−2, 2]: optimum −3 at x = 1.
    let registry = standard_registry().unwrap();
    let mut b = Body::new(&registry, 1);
    let x = b.x[0].clone();
    let one = b.c(1.0);
    let three = b.c(3.0);
    let d = b.op(Binary::Sub, &x, &one);
    let dd = b.op(Binary::Mul, &d, &d);
    let objective = b.op(Binary::Sub, &dd, &three);
    let body = b.b.prepare(&[objective]).unwrap();
    let case = case(
        &registry,
        body,
        &[(
            ModelingVariableDomain::Continuous,
            Some(-2.0),
            Some(2.0),
            0.0,
        )],
        &[],
        Some((0, ObjectiveSense::Minimize)),
        DerivativeOrder::Second,
    );
    let program = case.program(&FactorableRequest::default());
    let plan = execution::factorable::plan(&program, SolveIntent::Optimize).unwrap();
    assert!(plan.nonlinear());
    let settings = ScipSettings {
        pool: 16,
        ..ScipSettings::default()
    };
    let controls = Controls::default();
    let accuracy = ResolvedAccuracy::nominal();
    let tolerances = tolerances(1, 0);
    let normalization = Normalization::identity(1, 0);
    let mut original = Original(&case);
    let initial = case.initial();
    let tap = Arc::new(Collected::default());
    let mut execution = execution(false);
    execution.progress = Arc::new(crate::solve::Progress::tapped(
        controls.history,
        tap.clone(),
    ));
    let report = execution::factorable(
        Step {
            adapter: execution::adapter(Backend::Scip),
            settings: &BackendSettings::Scip(settings),
            controls: &controls,
            accuracy: &accuracy,
            execution,
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
            resolve: None,
        },
    )
    .unwrap();
    let f = |primal: &[f64]| Original(&case).evaluate(primal).unwrap().objective.unwrap();
    let candidate = report.candidate.as_ref().unwrap();
    assert!((candidate.primal[0] - 1.0).abs() < 1e-5, "{candidate:?}");
    assert!((candidate.objective.unwrap() - f(&candidate.primal)).abs() < 1e-9);
    assert_eq!(report.qualification, Qualification::GapQualified);
    let pool = &report.global.as_ref().unwrap().pool;
    // The trivial upper-bound solution x = 2, stored with its epigraph at SCIP's large
    // value, is reported at f(2) = −2.
    assert!(
        pool.iter()
            .any(|s| s.primal == [2.0] && s.objective == Some(-2.0)),
        "{pool:?}"
    );
    for solution in pool {
        let value = f(&solution.primal);
        assert!(
            (solution.objective.unwrap() - value).abs() < 1e-9,
            "{solution:?}: {value}"
        );
    }
    let events = tap.0.lock().unwrap().clone();
    let incumbents: Vec<&crate::solve::IncumbentEvent> =
        events.iter().filter_map(|e| e.incumbent.as_ref()).collect();
    // The first incumbent is the trivial zero solution, whose epigraph variable SCIP holds
    // at 0: reported at f(0) = −2.
    assert_eq!(incumbents[0].primal.as_deref(), Some(&[0.0][..]));
    assert_eq!(incumbents[0].objective, -2.0);
    for incumbent in &incumbents {
        if let Some(primal) = &incumbent.primal {
            let value = f(primal);
            assert!(
                (incumbent.objective - value).abs() < 1e-9,
                "{incumbent:?}: {value}"
            );
        }
    }
}

/// A multidimensional 0-1 knapsack, `max Σ vᵢ·xᵢ + 7` with `dims` capacity rows
/// `Σ wₖᵢ·xᵢ ≤ ½·Σ wₖᵢ` over pseudo-random integer data: presolve does not solve it, so
/// the search branches and improves its incumbent several times.
fn multidimensional_knapsack(registry: &QuantityRegistry, items: usize, dims: usize) -> Case {
    let mut state = 12_345_u64;
    let mut next = |range: u64| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((state >> 33) % range) as f64
    };
    let weights: Vec<Vec<f64>> = (0..dims)
        .map(|_| (0..items).map(|_| 10.0 + next(90)).collect())
        .collect();
    let values: Vec<f64> = (0..items).map(|_| 20.0 + next(80)).collect();
    let mut b = Body::new(registry, items);
    let x = b.x.clone();
    let mut outputs = Vec::new();
    for row in &weights {
        let mut load = None;
        for (i, w) in row.iter().enumerate() {
            let weight = b.c(*w);
            let term = b.op(Binary::Mul, &weight, &x[i]);
            load = Some(match load {
                Some(sum) => b.op(Binary::Add, &sum, &term),
                None => term,
            });
        }
        outputs.push(load.unwrap());
    }
    let mut value = b.c(7.0);
    for (i, v) in values.iter().enumerate() {
        let price = b.c(*v);
        let term = b.op(Binary::Mul, &price, &x[i]);
        value = b.op(Binary::Add, &value, &term);
    }
    outputs.push(value);
    let body = b.b.prepare(&outputs).unwrap();
    let binary = (ModelingVariableDomain::Binary, None, None, 0.0);
    let rows: Vec<(f64, f64)> = weights
        .iter()
        .map(|row| (f64::NEG_INFINITY, 0.5 * row.iter().sum::<f64>()))
        .collect();
    case(
        registry,
        body,
        &vec![binary; items],
        &rows,
        Some((dims, ObjectiveSense::Maximize)),
        DerivativeOrder::Value,
    )
}

/// Concurrent solving under admitted permits streams incumbents while the search runs
/// (Plan 22 G7): the concurrent solvers' improving solutions map back to the program's
/// columns by name, the stream stays monotone, every captured solution evaluates in the
/// original model to its reported objective, and the last incumbent is the result.
#[test]
fn scip_concurrent_streams_incumbents() {
    let registry = standard_registry().unwrap();
    let case = multidimensional_knapsack(&registry, 40, 5);
    let program = case.program(&FactorableRequest::default());
    let n = program.variables.len();
    let m = program.rows.len();
    let accuracy = ResolvedAccuracy::nominal();
    let tolerances = tolerances(n, m);
    let normalization = Normalization::identity(n, m);
    let initial = case.initial();
    let solve = |threads: usize| {
        let controls = Controls {
            threads,
            ..Controls::default()
        };
        let mut original = Original(&case);
        let tap = Arc::new(Collected::default());
        let mut execution = execution(false);
        execution.progress = Arc::new(crate::solve::Progress::tapped(
            controls.history,
            tap.clone(),
        ));
        let report = execution::factorable(
            Step {
                adapter: execution::adapter(Backend::Scip),
                settings: &BackendSettings::Scip(ScipSettings::default()),
                controls: &controls,
                accuracy: &accuracy,
                execution,
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
                resolve: None,
            },
        )
        .unwrap();
        let events = tap.0.lock().unwrap().clone();
        (report, events)
    };
    let (serial, _) = solve(1);
    let optimum = serial.candidate.as_ref().unwrap().objective.unwrap();
    let (report, events) = solve(2);
    assert!(matches!(
        report.metrics["scip.threads"],
        crate::solve::Metric::Integer(2)
    ));
    assert_eq!(
        report.options["concurrent/presolvebefore"],
        OptionValue::Bool(false)
    );
    assert_eq!(report.qualification, Qualification::GapQualified);
    let result = report.candidate.as_ref().unwrap().objective.unwrap();
    assert!((result - optimum).abs() < 1e-6, "{result} vs {optimum}");
    let incumbents: Vec<&crate::solve::IncumbentEvent> =
        events.iter().filter_map(|e| e.incumbent.as_ref()).collect();
    // The search improved several times, and each improvement was streamed as it came.
    let mut distinct: Vec<f64> = incumbents.iter().map(|i| i.objective).collect();
    distinct.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    assert!(distinct.len() >= 3, "{incumbents:?}");
    for pair in incumbents.windows(2) {
        assert!(pair[1].objective >= pair[0].objective - 1e-6, "{incumbents:?}");
    }
    assert!(incumbents[0].primal.is_some());
    for incumbent in &incumbents {
        if let Some(primal) = &incumbent.primal {
            let value = Original(&case).evaluate(primal).unwrap().objective.unwrap();
            assert!(
                (value - incumbent.objective).abs() < 1e-6,
                "{value} vs {}",
                incumbent.objective
            );
        }
    }
    assert!((incumbents.last().unwrap().objective - result).abs() < 1e-6);
}
