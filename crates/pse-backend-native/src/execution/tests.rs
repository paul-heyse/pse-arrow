// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
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
    classes: &[ProblemClass::SquareRoot],
    automatic_classes: &[ProblemClass::SquareRoot],
    derivatives: DerivativeCapability::JacobianOrProduct,
    warm: WarmCapability::Primal,
    general_bounds: false,
    sign_bounds: true,
    parallel: false,
    certifies: false,
    native_forms: &[],
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
        table: &STUB_TABLE,
        facts: &facts,
        intent: SolveIntent::Root,
        convex: false,
        controls: &controls,
    };
    // Routing reads the stub's capability record through the table.
    let backend = Backend::Idas;
    assert_eq!(
        requirements.select(SolverSelection::Auto).unwrap(),
        Route::Native(backend)
    );
    let adapter = STUB_TABLE.get(backend).unwrap();
    assert!(
        adapter
            .admit_settings(&BackendSettings::Default, &controls)
            .is_ok()
    );
    assert!(
        adapter
            .admit_settings(
                &BackendSettings::Default,
                &Controls {
                    threads: 2,
                    ..controls.clone()
                }
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
                adapter,
                settings: &BackendSettings::Default,
                controls: &controls,
                accuracy: &ResolvedAccuracy::nominal(),
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
        classes: Box::leak(row.classes.clone().into_boxed_slice()),
        automatic_classes: Box::leak(row.automatic_classes.clone().into_boxed_slice()),
        derivatives: row.derivatives,
        warm: row.warm,
        general_bounds: row.general_bounds,
        sign_bounds: row.sign_bounds,
        parallel: row.parallel,
        certifies: row.certifies,
        native_forms: Box::leak(row.native_forms.clone().into_boxed_slice()),
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
                        out.push(ProblemFacts {
                            variables: 1,
                            rows,
                            objective,
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
                            native,
                        });
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
                for convex in [false, true] {
                    for c in &controls {
                        let r = Requirements {
                            table: &LINKED,
                            facts: f,
                            intent,
                            convex,
                            controls: c,
                        };
                        assert_eq!(
                            adapter.admit(&r),
                            probe.admit(&r),
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
            table: &LINKED,
            facts: f,
            intent: SolveIntent::Root,
            convex: false,
            controls: &parallel,
        };
        assert_eq!(
            adapter.admit(&r).contains(&Ineligible::Serial),
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
/// registry and serde spellings, never Rust type names (ADR-0116 Outcome 9). These values
/// were captured when the identity serializer stopped framing type names (Plan 22 B5); they
/// change only when a document's encoding changes.
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
    let mut observed = documents
        .into_iter()
        .map(|(label, document)| {
            let settings = serde_json::from_value::<BackendSettings>(document).unwrap();
            (label, settings.identity().unwrap().to_prefixed())
        })
        .collect::<Vec<_>>();
    observed.push((
        "default",
        BackendSettings::Default.identity().unwrap().to_prefixed(),
    ));
    let controls: Controls = serde_json::from_value(json!({
        "hessian": "limited_memory",
        "reuse": "allow_rebuild",
    }))
    .unwrap();
    observed.push(("controls", controls.identity().unwrap().to_prefixed()));
    let expected: Vec<(&str, String)> = [
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
            "blake3:34a548735d2bb3735d9ba941b0e5bcea59456bac0efa542d5339d3c9aa4ed8c1",
        ),
    ]
    .into_iter()
    .map(|(label, identity)| (label, identity.to_owned()))
    .collect();
    assert_eq!(observed, expected);
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
        "version": 1,
        "linear": {"kind": "spgmr", "dimension": 5, "preconditioner": "jacobi"},
        "sensitivity": "staggered",
        "initialization": "steady_states",
        "constraints": ["non_negative", "free", "negative"],
    });
    assert_eq!(
        serde_json::to_value(
            serde_json::from_value::<crate::dynamics::IdasSettings>(idas.clone()).unwrap()
        )
        .unwrap(),
        idas
    );
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
        ["ipopt", "pounce", "kinsol", "highs", "clarabel", "scip"]
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
            "ScipSettings"
        ]
    );
    let definitions = schema["$defs"].as_object().unwrap();
    assert_eq!(
        definitions["MuStrategy"],
        serde_json::json!({"type": "string", "enum": ["monotone", "adaptive"]})
    );
    assert_eq!(definitions["Tolerance"]["exclusiveMinimum"], 0.0);
}
