// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::routing::Ineligible;
use crate::{
    quality::{Quality, Violation},
    routing::{Eligibility, Route},
    solve::{
        Assurance, Candidate, CandidateKind, HessianMode, Metric, NativeTermination, Qualification,
        ReusePolicy, SolveIntent, SolverSelection, Termination,
    },
    solver_tests::{Polynomial, stamp},
};
use pse_kernels::DerivativeOrder;
use pse_math::facts::{BoundShape, ProblemFacts};
use pse_model::generated::enums::ModelingVariableDomain;

/// A test-only adapter. It is known to no routing, runner or workflow code: it reaches
/// execution only through its capability record and a table entry.
#[derive(Debug)]
struct Stub;
static STUB: Stub = Stub;
static STUB_CAPABILITY: Capability = Capability {
    structural: crate::structural::Policy::Roots,
    lexicographic_degradation: crate::routing::DegradationSupport::Max,
    classes: &[ProblemClass::SquareRoot],
    automatic_classes: &[ProblemClass::SquareRoot],
    derivatives: DerivativeCapability::JacobianOrProduct,
    warm: WarmCapability::Primal,
    general_bounds: false,
    sign_bounds: true,
    parallel: false,
    certifies: false,
    native_forms: &[],
    requirements: &[],
    lexicographic: &[],
    batch: false,
    sensitivities: false,
    reuse: "test session counter",
    cancellation: "none",
    diagnostics: "none",
};
static STUB_TABLE: Table = Table::new(&[&STUB]);
impl BackendExecution for Stub {
    fn backend(&self) -> Backend {
        // A registry value that has no algebraic adapter in the production table.
        Backend::Idas
    }
    fn capability(&self) -> &'static Capability {
        &STUB_CAPABILITY
    }
    fn representation(&self) -> Representation {
        Representation::Roots
    }
    fn linked(&self) -> bool {
        true
    }
    fn automatic(&self) -> Option<u8> {
        Some(0)
    }
    fn primal_start(&self, primal: Vec<f64>) -> Result<WarmPayload, ProblemError> {
        Ok(WarmPayload::Root(primal))
    }
    fn accepts(&self, payload: &WarmPayload) -> bool {
        matches!(payload, WarmPayload::Root(_))
    }
    fn execute(
        &self,
        retained: &mut Retained,
        input: Input<'_>,
    ) -> Result<SolveReport, ProblemError> {
        let Problem::Roots {
            mut oracle,
            initial,
            ..
        } = input.problem
        else {
            return Err(representation(self.backend()));
        };
        let (sessions, reused) = retained.session(
            self.backend(),
            input.controls.reuse,
            |n: &mut i64| {
                *n += 1;
                Ok(true)
            },
            || Ok(1_i64),
        )?;
        // Scalar Newton iteration on the normalized residual.
        let (mut x, mut r, mut j) = (initial.to_vec(), [0.0], [0.0]);
        for _ in 0..input.controls.iterations.min(64) {
            oracle.residual(&x, &mut r)?;
            if r[0].abs() <= input.tolerances.rows[0] {
                break;
            }
            oracle.jacobian(&x, &mut j)?;
            x[0] -= r[0] / j[0];
        }
        oracle.residual(&x, &mut r)?;
        let contract = oracle.contract().clone();
        let mut report = SolveReport::new(
            self.backend(),
            &contract,
            NativeTermination {
                code: 0,
                name: "stub".into(),
                message: None,
                category: Termination::Success,
                assurance: Assurance::None,
            },
            &input.execution,
        );
        report.quality = Some(Quality::new(
            vec![Violation {
                id: contract.rows[0],
                physical: r[0].abs(),
                tolerance: input.tolerances.rows[0],
            }],
            vec![Violation {
                id: contract.variables[0].id,
                physical: 0.0,
                tolerance: input.tolerances.variables[0],
            }],
            vec![],
        )?);
        report.observation = Some(oracle.observe(r.to_vec())?);
        report.candidate = Some(Candidate {
            kind: CandidateKind::FinalIterate,
            primal: x,
            objective: None,
            row_dual: None,
            bound_dual: None,
            reduced_costs: None,
            slacks: None,
            commitment: None,
        });
        report
            .metrics
            .insert("stub.sessions".into(), Metric::Integer(*sessions));
        report
            .metrics
            .insert("reuse.native_model".into(), Metric::Bool(reused));
        report.evidence.reused_native_state = reused;
        Ok(report)
    }
}

#[test]
fn stub_backend_routes_through_adapter_table() {
    let facts = crate::routing::oracle_facts(&Polynomial::new().c, false, true);
    let controls = Controls {
        reuse: ReusePolicy::AllowRebuild,
        ..Controls::default()
    };
    let requirements = Requirements {
        context: crate::routing::test_context(&STUB_TABLE),
        table: &STUB_TABLE,
        facts: &facts,
        intent: SolveIntent::Root,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &BackendSettings::Default,
        sensitivity: false,
    };
    // Routing reads the stub's capability record through the table.
    let backend = Backend::Idas;
    assert_eq!(
        requirements
            .policy_select_for_test(SolverSelection::Auto)
            .unwrap(),
        Route::Native(backend)
    );
    let adapter = STUB_TABLE.get(backend).unwrap();
    assert!(
        adapter
            .admit_settings(
                &BackendSettings::Default,
                &controls,
                &Snapshot::observe(&STUB_TABLE)
            )
            .is_ok()
    );
    assert!(
        adapter
            .admit_settings(
                &BackendSettings::Default,
                &Controls {
                    threads: 2,
                    ..controls.clone()
                },
                &Snapshot::observe(&STUB_TABLE)
            )
            .is_err()
    );
    // The adapter owns its seed payload.
    let seed = WarmStart {
        origin: None,
        compatibility: stamp(backend),
        payload: adapter.primal_start(vec![3.0]).unwrap(),
    };
    assert!(adapter.accepts(&seed.payload));
    // The shared root runner solves a trivial problem and the retained session is reused.
    let tolerances = Tolerances {
        variables: vec![1e-9],
        rows: vec![1e-10],
        integrality: 1e-9,
    };
    let normalization = Normalization::identity(1, 1);
    let mut retained = Retained::default();
    for (attempt, warm) in [(1, None), (2, Some(&seed))] {
        let report = roots(
            Step {
                snapshot: &Snapshot::observe(&STUB_TABLE),
                structure: None,
                adapter,
                settings: &BackendSettings::Default,
                controls: &controls,
                accuracy: &ResolvedAccuracy::verification(),
                execution: Execution::new(
                    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                    &controls,
                ),
                tolerances: &tolerances,
                normalization: &normalization,
                compatibility: stamp(backend),
                warm,
            },
            &mut retained,
            Roots {
                oracle: Box::new(Polynomial::new()),
                initial: &[2.0],
                owner: None,
            },
        )
        .unwrap();
        assert_eq!(report.backend, Backend::Idas);
        let candidate = report.candidate.as_ref().unwrap();
        assert!((candidate.primal[0] - 1.0).abs() < 1e-9, "{candidate:?}");
        assert_eq!(report.qualification, Qualification::Feasible);
        assert_eq!(report.metrics["stub.sessions"], Metric::Integer(attempt));
        assert_eq!(
            report.metrics["reuse.native_model"],
            Metric::Bool(attempt > 1)
        );
    }
    assert_eq!(retained.backend(), Some(Backend::Idas));
    // A reuse-only policy with foreign retained state refuses instead of rebuilding, and
    // names the backend that holds it.
    let mut foreign_state = Retained::default();
    foreign_state
        .session(
            Backend::Kinsol,
            ReusePolicy::AllowRebuild,
            |_: &mut u8| Ok(true),
            || Ok(0_u8),
        )
        .unwrap();
    let refused = foreign_state.session(
        Backend::Idas,
        ReusePolicy::RequireReuse,
        |_: &mut i64| Ok(true),
        || Ok(0_i64),
    );
    assert!(matches!(
        refused,
        Err(ProblemError::Reuse {
            backend: Backend::Idas,
            refusal: crate::ReuseRefusal::Foreign(Backend::Kinsol),
        })
    ));
    // The same backend with state its step cannot refresh is a structural refusal.
    let refused = foreign_state.session(
        Backend::Kinsol,
        ReusePolicy::RequireReuse,
        |_: &mut u8| Ok(false),
        || Ok(0_u8),
    );
    assert!(matches!(
        refused,
        Err(ProblemError::Reuse {
            backend: Backend::Kinsol,
            refusal: crate::ReuseRefusal::Structure,
        })
    ));
    // The production table maps that registry value to its own, trajectory-only adapter.
    assert_eq!(
        LINKED.get(Backend::Idas).unwrap().representation(),
        Representation::Trajectory
    );
    assert!(
        Requirements {
            table: &LINKED,
            ..requirements
        }
        .eligibility()
        .iter()
        .all(|e: &Eligibility| e.backend != Backend::Idas)
    );
}

/// An adapter rebuilt from nothing but a published row and linkage.
#[derive(Debug)]
struct Probe {
    backend: Backend,
    representation: Representation,
    capability: &'static Capability,
}
impl BackendExecution for Probe {
    fn backend(&self) -> Backend {
        self.backend
    }
    fn capability(&self) -> &'static Capability {
        self.capability
    }
    fn representation(&self) -> Representation {
        self.representation
    }
    fn linked(&self) -> bool {
        true
    }
    fn automatic(&self) -> Option<u8> {
        None
    }
    fn execute(&self, _: &mut Retained, _: Input<'_>) -> Result<SolveReport, ProblemError> {
        Err(ProblemError::Internal("probe".into()))
    }
}
fn from_row(row: &pse_model::generated::runtime::solver_capabilities::Row) -> &'static Capability {
    Box::leak(Box::new(Capability {
        structural: row.structural_policy,
        lexicographic_degradation: row.lexicographic_degradation,
        classes: Box::leak(row.classes.clone().into_boxed_slice()),
        automatic_classes: Box::leak(row.automatic_classes.clone().into_boxed_slice()),
        derivatives: row.derivatives,
        warm: row.warm,
        general_bounds: row.general_bounds,
        sign_bounds: row.sign_bounds,
        parallel: row.parallel,
        certifies: row.certifies,
        native_forms: Box::leak(row.native_forms.clone().into_boxed_slice()),
        requirements: Box::leak(row.requirements.clone().into_boxed_slice()),
        lexicographic: Box::leak(row.lexicographic_classes.clone().into_boxed_slice()),
        batch: row.batch,
        sensitivities: row.sensitivities,
        reuse: "",
        cancellation: "",
        diagnostics: "",
    }))
}
fn grid() -> Vec<ProblemFacts> {
    let mut out = Vec::new();
    for bounds in [
        BoundShape::Free,
        BoundShape::Nonnegative,
        BoundShape::Nonpositive,
        BoundShape::Lower,
        BoundShape::Boxed,
    ] {
        for derivatives in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ] {
            for domain in [
                ModelingVariableDomain::Continuous,
                ModelingVariableDomain::Integer,
            ] {
                for (objective, equalities, rows) in
                    [(false, true, 1), (true, false, 1), (true, true, 2)]
                {
                    for ((coefficients, quadratic), native) in
                        [(false, false), (true, false), (true, true)]
                            .into_iter()
                            .flat_map(|c| [(c, vec![]), (c, vec![NativeConstraintForm::Indicator])])
                    {
                        for convexity in [
                            pse_math::convexity::Convexity::not_assessed(ContentHash::from_bytes(
                                [0; 32],
                            )),
                            pse_math::convexity::Convexity {
                                key: ContentHash::from_bytes([1; 32]),
                                class: pse_math::convexity::ConvexityClass::Cone(
                                    pse_math::convexity::ConeSummary::default(),
                                ),
                            },
                        ] {
                            out.push(ProblemFacts {
                                class_status: pse_math::presolve::ClassStatus::Established,
                                variables: 1,
                                rows,
                                objective,
                                objectives: usize::from(objective),
                                equalities,
                                domains: vec![domain],
                                derivatives,
                                prepared_derivatives: derivatives,
                                bounds: vec![bounds],
                                guarded: false,
                                coefficients,
                                affine_rows: vec![coefficients; rows],
                                objective_degree: Some(if quadratic { 2 } else { 1 }),
                                bound_assumptions: ContentHash::from_bytes([0; 32]),
                                quadratic,
                                native: native.clone(),
                                requirements: vec![],
                                convexity,
                            });
                        }
                    }
                }
            }
        }
    }
    out
}

#[test]
fn published_capabilities_equal_routing_rules() {
    let rows = LINKED.published();
    let linked: Vec<_> = LINKED.adapters().filter(|a| a.linked()).collect();
    assert_eq!(rows.len(), linked.len());
    let intents = SolveIntent::ALL;
    let controls: Vec<Controls> = [1, 2]
        .into_iter()
        .flat_map(|threads| {
            [HessianMode::Exact, HessianMode::LimitedMemory].map(|hessian| Controls {
                threads,
                hessian,
                ..Controls::default()
            })
        })
        .collect();
    let facts = grid();
    for adapter in linked {
        let row = rows
            .iter()
            .find(|r| r.backend == adapter.backend())
            .unwrap();
        assert_eq!(*row, adapter.capability().row(adapter.backend()));
        // Routing reads nothing beyond the published row and linkage: an adapter rebuilt
        // from the row is assessed identically on every fact, intent and control.
        let probe = Probe {
            backend: adapter.backend(),
            representation: adapter.representation(),
            capability: from_row(row),
        };
        for f in &facts {
            for intent in intents {
                for numerical_psd in [false, true] {
                    for c in &controls {
                        let r = Requirements {
                            context: crate::routing::test_context(&LINKED),
                            table: &LINKED,
                            facts: f,
                            intent,
                            numerical_psd,
                            least_squares: false,
                            controls: c,
                            settings: &BackendSettings::Default,
                            sensitivity: false,
                        };
                        assert_eq!(
                            crate::routing::admit(
                                adapter.backend(),
                                adapter.capability(),
                                r.context.snapshot.linked(adapter.backend()),
                                &r
                            ),
                            crate::routing::admit(
                                probe.backend(),
                                probe.capability(),
                                r.context.snapshot.linked(probe.backend()),
                                &r
                            ),
                            "{:?} {f:?} {intent:?} {c:?}",
                            adapter.backend()
                        );
                    }
                }
            }
        }
        // The published serial flag is the thread rule.
        let f = &facts[0];
        let parallel = Controls {
            threads: 2,
            ..Controls::default()
        };
        let r = Requirements {
            context: crate::routing::test_context(&LINKED),
            table: &LINKED,
            facts: f,
            intent: SolveIntent::Root,
            numerical_psd: false,
            least_squares: false,
            controls: &parallel,
            settings: &BackendSettings::Default,
            sensitivity: false,
        };
        assert_eq!(
            crate::routing::admit(
                adapter.backend(),
                adapter.capability(),
                r.context.snapshot.linked(adapter.backend()),
                &r
            )
            .contains(&Ineligible::Serial),
            !row.parallel
        );
    }
    // An unlinked adapter is never published and routing reports it unlinked.
    for adapter in LINKED.adapters().filter(|a| !a.linked()) {
        assert!(rows.iter().all(|r| r.backend != adapter.backend()));
    }
}
/// ADR-0113, ADR-0116: each adapter's settings type is its boundary form, tagged by the
/// registry backend spelling and present in every build. Omitted fields take the Rust
/// defaults, unknown fields and backends are refused, and documents round-trip with the
/// same identity.
#[test]
fn settings_documents_round_trip_every_variant() {
    use serde_json::json;
    let variants = [
        (
            Backend::Clarabel,
            json!({"mode": "reusable_data", "max_step_fraction": 0.9}),
        ),
        (Backend::Scip, json!({"seed": 7, "nodes": 100})),
        (Backend::Highs, json!({"method": "simplex"})),
        (Backend::Pounce, json!({"method": "active_set_sqp"})),
        (
            Backend::Kinsol,
            json!({"strategy": "newton", "linear": {"kind": "spgmr", "dimension": 8}}),
        ),
        // The linear solver keeps Ipopt's native `linear_solver` spelling (ADR-0108).
        (
            Backend::Ipopt,
            json!({
                "linear": {"kind": "pardisomkl", "ordering": "parallel_metis", "matching": "complete_plus2x2"},
                "mu_strategy": "adaptive",
                "bound_push": 0.001,
            }),
        ),
    ];
    let decode = |document: serde_json::Value| serde_json::from_value::<BackendSettings>(document);
    let tagged = |backend: Backend, fields: &serde_json::Value| {
        let mut document = fields.clone();
        document["backend"] = json!(backend.as_str());
        document
    };
    for (backend, fields) in variants {
        let settings = decode(tagged(backend, &fields)).unwrap();
        assert_eq!(settings.backend(), Some(backend));
        // Every supplied field is kept; every other field is present with its default.
        let effective = serde_json::to_value(&settings).unwrap();
        let defaults = serde_json::to_value(decode(tagged(backend, &json!({}))).unwrap()).unwrap();
        assert_eq!(
            effective.as_object().unwrap().keys().collect::<Vec<_>>(),
            defaults.as_object().unwrap().keys().collect::<Vec<_>>()
        );
        for (key, value) in fields.as_object().unwrap() {
            assert_eq!(&effective[key], value, "{backend:?} {key}");
        }
        let back = decode(effective).unwrap();
        assert_eq!(back.identity().unwrap(), settings.identity().unwrap());
        // Unknown fields are refused.
        let mut unknown = tagged(backend, &fields);
        unknown["not_a_field"] = json!(1);
        assert!(decode(unknown).is_err(), "{backend:?}");
    }
    // Backends without algebraic settings, and names that are no backend, are refused.
    for backend in ["diffsol", "idas", "default", "gurobi"] {
        assert!(decode(json!({"backend": backend})).is_err(), "{backend}");
    }
    // The routed backend's native defaults have no document form.
    assert!(BackendSettings::Default.document().is_none());
    assert!(serde_json::to_value(&BackendSettings::Default).is_err());
    // A nested restart takes its own defaults for the fields it omits.
    let partial = serde_json::to_value(
        decode(
            json!({"backend": "ipopt", "restart": {"barrier": {"kind": "value", "value": 0.01}}}),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        partial["restart"]["barrier"],
        json!({"kind": "value", "value": 0.01})
    );
    assert_eq!(partial["restart"]["bound_push"], json!(1e-9));
    assert_eq!(
        partial["linear"],
        json!({"kind": "mumps", "ordering": "metis"})
    );
    // Solver parameters are stated, never inherited, and HSL or the runtime-loaded Pardiso
    // are not representable (ADR-0108 items 9 and 10).
    for linear in [
        json!({"kind": "spral", "ordering": "metis"}),
        json!({"kind": "mumps", "ordering": "metis", "pivot": "block"}),
        json!({"kind": "ma57"}),
        json!("pardiso"),
    ] {
        assert!(decode(json!({"backend": "ipopt", "linear": linear})).is_err());
    }
}

/// Settings identity is the serde data model of the typed document: field names and the
/// registry and serde spellings, never Rust type names (ADR-0116 Outcome 9). ADR-0150
/// versions canonical float identity; historical vectors retain their original document
/// shape and values, while the current vectors include current typed defaults.
#[test]
fn settings_identity_is_type_name_independent() {
    use serde_json::json;
    let documents = [
        (
            "scip",
            json!({"backend": "scip", "nlp_linear_solver": "pardisomkl", "seed": 3}),
        ),
        (
            "ipopt-spral",
            json!({
                "backend": "ipopt",
                "linear": {"kind": "spral", "ordering": "matching", "scaling": "mc64", "pivot": "threshold"},
                "mu_strategy": "adaptive",
            }),
        ),
        (
            "ipopt-mumps",
            json!({"backend": "ipopt", "linear": {"kind": "mumps", "ordering": "qamd"}}),
        ),
        ("highs", json!({"backend": "highs", "method": "ipm"})),
        (
            "pounce",
            json!({"backend": "pounce", "method": "l1_exact_penalty"}),
        ),
        (
            "kinsol",
            json!({
                "backend": "kinsol",
                "strategy": "fixed_point",
                "orthogonalization": "inverse_compact_wy",
                "preconditioner": "jacobi",
            }),
        ),
        (
            "clarabel",
            json!({"backend": "clarabel", "mode": "reusable_data"}),
        ),
    ];
    #[derive(serde::Serialize)]
    #[serde(transparent)]
    struct RenamedSettings<'a>(Option<&'a BackendSettings>);
    // Frozen pre-Plan-25n serde data model. JSON would erase option, enum and
    // numeric-type distinctions, so serialize this test-only document directly.
    // These literals reproduce the prior Settings/FeralIdentity declarations and
    // their defaults; no current decoding, readmission or native defaults are used.
    #[derive(serde::Serialize)]
    #[serde(tag = "backend", rename_all = "snake_case")]
    enum HistoricalBackend {
        Pounce(HistoricalPounce),
    }
    #[derive(serde::Serialize)]
    struct HistoricalPounce {
        method: HistoricalMethod,
        linear: HistoricalLinear,
        restart: HistoricalRestart,
    }
    #[derive(serde::Serialize)]
    #[serde(rename_all = "snake_case")]
    enum HistoricalMethod {
        L1ExactPenalty,
    }
    #[derive(serde::Serialize)]
    #[serde(rename_all = "snake_case")]
    enum HistoricalAuto {
        Auto,
    }
    #[derive(serde::Serialize)]
    struct HistoricalLinear {
        cascade_break: Option<bool>,
        fma: bool,
        refine: bool,
        increase_quality: bool,
        refine_max_steps: usize,
        refine_target: f64,
        singular_pivot_floor: f64,
        inertia_pivot_floor: Option<f64>,
        pivtol: f64,
        ordering: HistoricalAuto,
        scaling: HistoricalAuto,
        parallel: Option<bool>,
        min_par_flops: Option<u64>,
        static_pivoting: Option<bool>,
    }
    #[derive(serde::Serialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum HistoricalBarrier {
        Seed,
    }
    #[derive(serde::Serialize)]
    struct HistoricalRestart {
        barrier: HistoricalBarrier,
        bound_push: f64,
        bound_frac: f64,
        slack_bound_push: f64,
        slack_bound_frac: f64,
        mult_bound_push: f64,
    }
    let historical_pounce = HistoricalBackend::Pounce(HistoricalPounce {
        method: HistoricalMethod::L1ExactPenalty,
        linear: HistoricalLinear {
            cascade_break: None,
            fma: false,
            refine: true,
            increase_quality: true,
            refine_max_steps: 10,
            refine_target: 0.0,
            singular_pivot_floor: 1e-20,
            inertia_pivot_floor: None,
            pivtol: 1e-8,
            ordering: HistoricalAuto::Auto,
            scaling: HistoricalAuto::Auto,
            parallel: None,
            min_par_flops: None,
            static_pivoting: None,
        },
        restart: HistoricalRestart {
            barrier: HistoricalBarrier::Seed,
            bound_push: 1e-9,
            bound_frac: 1e-9,
            slack_bound_push: 1e-9,
            slack_bound_frac: 1e-9,
            mult_bound_push: 1e-9,
        },
    });
    let mut historical = Vec::new();
    let mut observed = documents
        .into_iter()
        .map(|(label, document)| {
            let settings = serde_json::from_value::<BackendSettings>(document).unwrap();
            let current = settings.identity().unwrap();
            assert_eq!(
                current,
                pse_ids::document::of(
                    pse_ids::Frame::BackendSettingsV5,
                    &RenamedSettings(settings.document())
                )
                .unwrap(),
                "Rust type names and transparent wrappers cannot change identity"
            );
            let historical_identity = if label == "pounce" {
                pse_ids::document::of(pse_ids::Frame::BackendSettingsV4, &Some(&historical_pounce))
            } else {
                pse_ids::document::of(pse_ids::Frame::BackendSettingsV4, &settings.document())
            };
            historical.push((label, historical_identity.unwrap().to_prefixed()));
            (label, current.to_prefixed())
        })
        .collect::<Vec<_>>();
    observed.push((
        "default",
        BackendSettings::Default.identity().unwrap().to_prefixed(),
    ));
    historical.push((
        "default",
        pse_ids::document::of(
            pse_ids::Frame::BackendSettingsV4,
            &BackendSettings::Default.document(),
        )
        .unwrap()
        .to_prefixed(),
    ));
    let controls: Controls = serde_json::from_value(json!({
        "hessian": "limited_memory",
        "reuse": "allow_rebuild",
    }))
    .unwrap();
    observed.push(("controls", controls.identity().unwrap().to_prefixed()));
    historical.push((
        "controls",
        pse_ids::document::of(pse_ids::Frame::NativeControlsV2, &controls)
            .unwrap()
            .to_prefixed(),
    ));
    let historical_expected: Vec<(&str, String)> = [
        (
            "scip",
            "blake3:611480d096c386b314b60cb20ace70af661323f94b337d07c00578ea10295eff",
        ),
        (
            "ipopt-spral",
            "blake3:054c03e7a702a4aa183dd1213606feea4af76b656259cf0ef15906fdaaf082a6",
        ),
        (
            "ipopt-mumps",
            "blake3:26b9ab676250861a2cbaa02288925d6d3ec2d8c2bfb43b5b0d4e473d9ea83437",
        ),
        (
            "highs",
            "blake3:f65e308742439f30b066a0158bc34583d959c567364c5fee73b967a8c6861e00",
        ),
        (
            "pounce",
            "blake3:968e74f3f12dbdac45bbc4c586d821ce6c94f48aecb67bcdf9a113bf8e60e043",
        ),
        (
            "kinsol",
            "blake3:c1b7a9c8cd89a06292dd796b079990c612607bf9916aa2b66090d450bc258f20",
        ),
        (
            "clarabel",
            "blake3:77a643954ab01bb9183d439bd225b7283b80b9ee41ce1b14e57e151fca44ffd2",
        ),
        (
            "default",
            "blake3:143ab881422592a7baf5075867178c53eacb8b8d9f3fe02d1dcdd01cc6d897e0",
        ),
        (
            "controls",
            "blake3:392947a1762675c7145fc532eceb57c1a52a352640c5a2d50a5b85384de8abc2",
        ),
    ]
    .into_iter()
    .map(|(label, identity)| (label, identity.to_owned()))
    .collect();
    let expected = [
        (
            "scip",
            "blake3:9112367f376ff2354172d9fd75a15b1681babf8a5ce150884b12e03c52d86c72",
        ),
        (
            "ipopt-spral",
            "blake3:c8d2badf3dc6062d6fc81518b118fc7d303e3d23d9bf3d49c68d04b3018df50a",
        ),
        (
            "ipopt-mumps",
            "blake3:5a11a523d8f5ccc6ecc47f0d074dde4aaf8d8b0ee9a4c3acc70737efa4bb9f52",
        ),
        (
            "highs",
            "blake3:3893536a85a8ddc73942b4b0ee9cc886643c5c1dd06e3fa50bf505a9d341b493",
        ),
        (
            "pounce",
            // Current FeralIdentity includes the optional storage dimension bound,
            // including its unset state. The source revision belongs to the native
            // build/request identity, not this typed settings document.
            "blake3:dbbc460a77cea086b9d2e787b4f5fd8191dda0f9721aa5dd65c1ebc318b737e7",
        ),
        (
            "kinsol",
            "blake3:d2d46285fea4049ac20f5938aa5c60f06608481017e73255aa4e9e0cb33d4301",
        ),
        (
            "clarabel",
            "blake3:f3fb09bae3d811d458f129ed696951e36ee40b7eca2096dfdc8d68b7346a4c59",
        ),
        (
            "default",
            "blake3:6e4c1011c85f48b669a14f7d509c2bff7df3bdef0301f3bc741cd1bbd6fbd6be",
        ),
        (
            "controls",
            "blake3:864a5fbb98459a261b2db003e491ef6ab04a97b94519719515b82039cb2fe28d",
        ),
    ]
    .into_iter()
    .map(|(label, identity)| (label, identity.to_owned()))
    .collect::<Vec<_>>();
    assert_eq!(
        historical, historical_expected,
        "historical document bytes remain frozen"
    );
    assert_eq!(
        observed, expected,
        "current typed defaults use canonical V5 settings identity"
    );
    // Plan 25n's native controls are identity-bearing even when callers submit
    // partial documents and the remaining settings take their typed defaults.
    let pounce = serde_json::from_value::<BackendSettings>(json!({"backend": "pounce"}))
        .unwrap()
        .identity()
        .unwrap();
    for fields in [
        json!({"partitioned": {"update_type": "bfgs"}}),
        json!({"partitioned": {"max_element": 32}}),
        json!({"partitioned": {"elements": "primal_block"}}),
        json!({"partitioned": {"block_size": 32}}),
        json!({"partitioned": {"curvature_cap": 2.0}}),
        json!({"finite_difference": {"pattern": "jacobian"}}),
        json!({"finite_difference": {"coloring": "star"}}),
        json!({"finite_difference": {"reuse_tolerance": 0.001}}),
        json!({"linear": {"bounded_storage_max_dimension": 7}}),
    ] {
        let mut document = fields;
        document["backend"] = json!("pounce");
        let changed = serde_json::from_value::<BackendSettings>(document.clone()).unwrap();
        assert_ne!(pounce, changed.identity().unwrap(), "{document}");
    }
    // The time limit of the controls document is its seconds.
    assert_eq!(
        serde_json::to_value(&controls).unwrap()["time_limit"],
        json!(300.0)
    );
    // The dynamics settings documents carry their version.
    assert_eq!(
        serde_json::to_string(&crate::dynamics::DiffsolSettings {
            method: crate::dynamics::DiffsolMethod::TrBdf2,
            linear: crate::dynamics::DiffsolLinear::Klu,
            ..Default::default()
        })
        .unwrap(),
        r#"{"version":1,"method":"tr_bdf2","linear":"klu"}"#
    );
    let idas = json!({
        "version": 2,
        "linear": {"kind": "spgmr", "dimension": 5, "preconditioner": "jacobi"},
        "sensitivity": "staggered",
        "initialization": "steady_states",
        "initial_conditions": {
            "step_trials": null,
            "jacobian_attempts": 4,
            "newton_iterations": 10,
            "convergence_coefficient": 0.0033,
            "line_search": true,
            "backtracks": null,
            "step_tolerance": null,
        },
    });
    assert_eq!(
        serde_json::to_value(
            serde_json::from_value::<crate::dynamics::IdasSettings>(idas.clone()).unwrap()
        )
        .unwrap(),
        idas
    );
    let mut omitted = idas.clone();
    omitted
        .as_object_mut()
        .unwrap()
        .remove("initial_conditions");
    let effective = serde_json::from_value::<crate::dynamics::IdasSettings>(omitted).unwrap();
    assert_eq!(serde_json::to_value(&effective).unwrap(), idas);
    let mut profile = crate::dynamics::Profile {
        method: crate::dynamics::Method::Idas,
        idas: effective,
        ..Default::default()
    };
    let identity = pse_ids::document::of(pse_ids::Frame::DynamicProfileV9, &profile).unwrap();
    profile.idas.initial_conditions.newton_iterations += 1;
    assert_ne!(
        identity,
        pse_ids::document::of(pse_ids::Frame::DynamicProfileV9, &profile).unwrap()
    );
    // Version 2 has no sign constraints: they derive from the authored bounds
    // (ADR-0119 Outcome 4), so a version 1 document or a constraint field is refused.
    let mut constrained = idas.clone();
    constrained["constraints"] = json!(["non_negative"]);
    assert!(serde_json::from_value::<crate::dynamics::IdasSettings>(constrained).is_err());
    let mut previous = idas.clone();
    previous["version"] = json!(1);
    assert!(serde_json::from_value::<crate::dynamics::IdasSettings>(previous).is_err());
    let mut unversioned = idas;
    unversioned.as_object_mut().unwrap().remove("version");
    assert!(serde_json::from_value::<crate::dynamics::IdasSettings>(unversioned).is_err());
}

/// The backend settings document publishes a JSON Schema: one tagged alternative per
/// backend in every build, registry vocabularies by name, unknown fields refused
/// (ADR-0116 Outcome 7).
#[test]
fn backend_settings_schema_generated() {
    let schema = serde_json::to_value(schemars::schema_for!(BackendSettings)).unwrap();
    let alternatives = schema["oneOf"].as_array().unwrap();
    let tags = alternatives
        .iter()
        .map(|alternative| {
            assert_eq!(alternative["additionalProperties"], false);
            alternative["properties"]["backend"]["const"]
                .as_str()
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        tags,
        [
            "ipopt",
            "pounce",
            "kinsol",
            "highs",
            "clarabel",
            "scip",
            "pounce_convex",
            "uno",
            "petsc"
        ]
    );
    let titles = alternatives
        .iter()
        .map(|alternative| alternative["title"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        titles,
        [
            "IpoptSettings",
            "PounceSettings",
            "KinsolSettings",
            "HighsSettings",
            "ClarabelSettings",
            "ScipSettings",
            "PounceConvexSettings",
            "UnoSettings",
            "PetscSettings"
        ]
    );
    let definitions = schema["$defs"].as_object().unwrap();
    assert_eq!(
        definitions["MuStrategy"],
        serde_json::json!({"type": "string", "enum": ["monotone", "adaptive"]})
    );
    assert_eq!(definitions["Tolerance"]["exclusiveMinimum"], 0.0);
}

/// Routing treats an unknown backend only through its declared capability, including QP priorities.
#[test]
fn declared_lexicographic_capability_native_staged_and_no_fallback() {
    #[derive(Debug)]
    struct LexProbe {
        capability: &'static Capability,
    }
    impl BackendExecution for LexProbe {
        fn backend(&self) -> Backend {
            Backend::Idas
        }
        fn capability(&self) -> &'static Capability {
            self.capability
        }
        fn representation(&self) -> Representation {
            Representation::Factorable
        }
        fn linked(&self) -> bool {
            true
        }
        fn automatic(&self) -> Option<u8> {
            Some(0)
        }
        fn assess(&self, r: &Requirements<'_>) -> Eligibility {
            let reasons =
                crate::routing::admit(self.backend(), self.capability(), self.linked(), r);
            let state = if reasons.is_empty() {
                crate::routing::AssessmentState::Ready
            } else {
                crate::routing::AssessmentState::Refused
            };
            Eligibility {
                backend: self.backend(),
                reasons,
                causes: vec![],
                class_dependencies: vec![],
                evidence: vec![],
                artifacts: vec![],
                factorable_refusals: vec![],
                structure: None,
                state,
            }
        }
        fn execute(&self, _: &mut Retained, _: Input<'_>) -> Result<SolveReport, ProblemError> {
            Err(ProblemError::Internal("routing-only probe".into()))
        }
    }
    let capability = Box::leak(Box::new(Capability {
        classes: &[ProblemClass::NonconvexQuadratic],
        automatic_classes: &[ProblemClass::NonconvexQuadratic],
        derivatives: DerivativeCapability::Factorable,
        general_bounds: true,
        lexicographic: &[ProblemClass::NonconvexQuadratic],
        structural: crate::structural::Policy::Factorable,
        ..STUB_CAPABILITY
    }));
    let probe: &'static dyn BackendExecution = Box::leak(Box::new(LexProbe { capability }));
    let entries: &'static [&'static dyn BackendExecution] =
        Box::leak(vec![probe].into_boxed_slice());
    let table = Table::new(entries);
    let mut facts = grid()
        .into_iter()
        .find(|f| {
            f.coefficients
                && f.quadratic
                && f.objective
                && f.domains == [ModelingVariableDomain::Continuous]
        })
        .unwrap();
    facts.objectives = 2;
    let controls = Controls::default();
    let requirements = Requirements {
        context: crate::routing::test_context(&table),
        table: &table,
        facts: &facts,
        intent: SolveIntent::Optimize,
        numerical_psd: false,
        least_squares: false,
        controls: &controls,
        settings: &BackendSettings::Default,
        sensitivity: false,
    };
    let native = requirements.lexicographic(SolverSelection::Auto, false);
    assert_eq!(native.route().unwrap(), Route::Native(Backend::Idas));
    assert_eq!(
        native.lexicographic,
        Some(crate::routing::Lexicographic::Native)
    );
    let staged_capability = Box::leak(Box::new(Capability {
        lexicographic: &[],
        ..*capability
    }));
    let staged_probe: &'static dyn BackendExecution = Box::leak(Box::new(LexProbe {
        capability: staged_capability,
    }));
    let entries: &'static [&'static dyn BackendExecution] =
        Box::leak(vec![staged_probe].into_boxed_slice());
    let table = Table::new(entries);
    let requirements = Requirements {
        table: &table,
        ..requirements
    };
    let staged = requirements.lexicographic(SolverSelection::Explicit(Backend::Idas), false);
    assert_eq!(staged.route().unwrap(), Route::Native(Backend::Idas));
    assert_eq!(
        staged.lexicographic,
        Some(crate::routing::Lexicographic::Staged)
    );
    let refused = requirements.lexicographic(SolverSelection::Explicit(Backend::Highs), true);
    assert!(refused.selected.is_none());
    assert!(matches!(
        refused.refusal,
        Some(crate::routing::Refusal::Unavailable(Backend::Highs))
    ));
    let Err(ProblemError::RouteRefused(retained)) = refused.route() else {
        panic!("missing retained route refusal");
    };
    assert_eq!(
        retained.selection,
        SolverSelection::Explicit(Backend::Highs)
    );
    assert!(!retained.classes.is_empty());
    assert_eq!(retained.eligibility.len(), 1);
}
