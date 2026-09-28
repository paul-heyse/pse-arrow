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
use pse_model::generated::enums::ModelingVariableDomain;
use pse_math::{
    facts::{BoundShape, ProblemFacts},
};

/// A test-only adapter. It is known to no routing, runner or workflow code: it reaches
/// execution only through its capability record and a table entry.
#[derive(Debug)]
struct Stub;
static STUB: Stub = Stub;
static STUB_CAPABILITY: Capability = Capability {
    classes: &[ProblemClass::SquareRoot],
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
    // A fresh-only policy with foreign retained state refuses instead of rebuilding.
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
    assert!(matches!(refused, Err(ProblemError::Unsupported(_))));
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
            for domain in [ModelingVariableDomain::Continuous, ModelingVariableDomain::Integer] {
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

/// ADR-0113: each adapter's settings type is its boundary form. Omitted fields take the
/// Rust defaults, unknown fields and versions are refused, and documents round-trip with
/// the same identity.
#[test]
fn settings_documents_round_trip_every_variant() {
    use serde_json::json;
    let mut variants = vec![
        (Backend::Clarabel, json!({"mode": "reusable_data", "max_step_fraction": 0.9})),
        (Backend::Scip, json!({"seed": 7, "nodes": 100})),
    ];
    if cfg!(feature = "highs") {
        variants.push((Backend::Highs, json!({"method": "simplex"})));
    }
    if cfg!(feature = "pounce") {
        variants.push((Backend::Pounce, json!({"method": "active_set_sqp"})));
    }
    if cfg!(feature = "kinsol") {
        variants.push((
            Backend::Kinsol,
            json!({"strategy": "newton", "linear": {"spgmr": {"dimension": 8}}}),
        ));
    }
    if cfg!(feature = "ipopt") {
        // The linear solver keeps Ipopt's native `linear_solver` spelling (ADR-0108).
        variants.push((
            Backend::Ipopt,
            json!({
                "linear": {"pardisomkl": {"ordering": "parallel_metis", "matching": "complete_plus2x2"}},
                "mu_strategy": "adaptive",
                "bound_push": 0.001,
            }),
        ));
    }
    for (backend, fields) in variants {
        let settings = BackendSettings::from_fields(backend, fields.clone()).unwrap();
        assert_eq!(settings.backend(), Some(backend));
        let document = settings.document().unwrap();
        assert_eq!(document.version, SETTINGS_VERSION);
        // Every supplied field is kept; every other field is present with its default.
        let defaults = BackendSettings::from_fields(backend, json!({}))
            .unwrap()
            .fields()
            .unwrap();
        let effective = document.settings.as_object().unwrap();
        assert_eq!(
            effective.keys().collect::<Vec<_>>(),
            defaults.as_object().unwrap().keys().collect::<Vec<_>>()
        );
        for (key, value) in fields.as_object().unwrap() {
            assert_eq!(&effective[key], value, "{backend:?} {key}");
        }
        let text = serde_json::to_string(&document).unwrap();
        let back = BackendSettings::from_document(serde_json::from_str(&text).unwrap()).unwrap();
        assert_eq!(back.identity().unwrap(), settings.identity().unwrap());
        // Unknown fields and document versions are refused.
        let mut unknown = fields.clone();
        unknown["not_a_field"] = json!(1);
        assert!(BackendSettings::from_fields(backend, unknown).is_err());
        let mut future = document.clone();
        future.version += 1;
        assert!(BackendSettings::from_document(future).is_err());
    }
    // Backends without algebraic settings are refused, never defaulted.
    let mut refused = vec![Backend::Diffsol, Backend::Idas];
    if !cfg!(feature = "ipopt") {
        refused.push(Backend::Ipopt);
    }
    for backend in refused {
        assert!(BackendSettings::from_fields(backend, json!({})).is_err());
    }
    assert!(BackendSettings::Default.document().is_err());
    if cfg!(feature = "ipopt") {
        // A nested restart takes its own defaults for the fields it omits.
        let partial = BackendSettings::from_fields(
            Backend::Ipopt,
            json!({"restart": {"barrier": {"value": 0.01}}}),
        )
        .unwrap()
        .fields()
        .unwrap();
        assert_eq!(partial["restart"]["barrier"], json!({"value": 0.01}));
        assert_eq!(partial["restart"]["bound_push"], json!(1e-9));
        assert_eq!(partial["linear"], json!({"mumps": {"ordering": "metis"}}));
        // Solver parameters are stated, never inherited, and HSL or the runtime-loaded
        // Pardiso are not representable (ADR-0108 items 9 and 10).
        for linear in [
            json!({"spral": {"ordering": "metis"}}),
            json!({"mumps": {"ordering": "metis", "pivot": "block"}}),
            json!({"ma57": {}}),
            json!("pardiso"),
        ] {
            let fields = json!({"linear": linear});
            assert!(BackendSettings::from_fields(Backend::Ipopt, fields).is_err());
        }
    }
}

/// The registry-owned settings vocabularies keep the serde spellings and the type names of
/// the enums they replaced, so a settings document and its serde-derived identity
/// (`pse.backend.settings.v3`, `pse.native.controls.v1`) keep their bytes (Plan 22 B4).
/// The values were captured from the hand-written enums before they were deleted.
#[test]
fn settings_identity_bytes_unchanged() {
    use serde_json::json;
    let mut observed = Vec::new();
    let mut documents = vec![(
        "scip",
        Backend::Scip,
        json!({"nlp_linear_solver": "pardisomkl", "seed": 3}),
    )];
    if cfg!(feature = "ipopt") {
        documents.push((
            "ipopt-spral",
            Backend::Ipopt,
            json!({
                "linear": {"spral": {"ordering": "matching", "scaling": "mc64", "pivot": "threshold"}},
                "mu_strategy": "adaptive",
            }),
        ));
        documents.push((
            "ipopt-pardiso",
            Backend::Ipopt,
            json!({
                "linear": {"pardisomkl": {"ordering": "parallel_metis", "matching": "complete_plus2x2"}},
            }),
        ));
        documents.push((
            "ipopt-mumps",
            Backend::Ipopt,
            json!({"linear": {"mumps": {"ordering": "qamd"}}}),
        ));
    }
    for (label, backend, fields) in documents {
        let settings = BackendSettings::from_fields(backend, fields).unwrap();
        observed.push((
            label,
            serde_json::to_string(&settings.document().unwrap()).unwrap(),
            settings.identity().unwrap().to_prefixed(),
        ));
    }
    let controls = crate::solve::Controls {
        hessian: HessianMode::LimitedMemory,
        reuse: ReusePolicy::AllowRebuild,
        ..Default::default()
    };
    observed.push((
        "controls",
        serde_json::to_string(&controls).unwrap(),
        controls.identity().unwrap().to_prefixed(),
    ));
    let restart = r#""restart":{"barrier":"seed","bound_push":1e-9,"bound_frac":1e-9,"slack_bound_push":1e-9,"slack_bound_frac":1e-9,"mult_bound_push":1e-9}"#;
    let mut expected = vec![(
        "scip",
        r#"{"version":1,"backend":"scip","settings":{"nlp_linear_solver":"pardisomkl","seed":3,"nodes":null,"pool":0,"iis":false,"exact":false,"reoptimize":false}}"#.to_owned(),
        "blake3:72f817e397d3fbe5bf3d6907af1451462387361c4551eb226055dd1e5961550c",
    )];
    if cfg!(feature = "ipopt") {
        expected.extend([
            (
                "ipopt-spral",
                format!(
                    r#"{{"version":1,"backend":"ipopt","settings":{{"linear":{{"spral":{{"ordering":"matching","scaling":"mc64","pivot":"threshold"}}}},"mu_strategy":"adaptive","bound_push":0.01,"bound_frac":0.01,{restart}}}}}"#
                ),
                "blake3:1695712d4f089905da60c7251a38f086710eae08953d58fda1283bfd15efb2c4",
            ),
            (
                "ipopt-pardiso",
                format!(
                    r#"{{"version":1,"backend":"ipopt","settings":{{"linear":{{"pardisomkl":{{"ordering":"parallel_metis","matching":"complete_plus2x2"}}}},"mu_strategy":"monotone","bound_push":0.01,"bound_frac":0.01,{restart}}}}}"#
                ),
                "blake3:b2c4165793bb5a6b9b5f5c8c963bcf85006e1a5312f532dc32afe9b16272f3de",
            ),
            (
                "ipopt-mumps",
                format!(
                    r#"{{"version":1,"backend":"ipopt","settings":{{"linear":{{"mumps":{{"ordering":"qamd"}}}},"mu_strategy":"monotone","bound_push":0.01,"bound_frac":0.01,{restart}}}}}"#
                ),
                "blake3:d49e9d0faaa65fcdfd1d4f4a6901b0322f255a338d97104028485db4674ea2dc",
            ),
        ]);
    }
    expected.push((
        "controls",
        r#"{"time_limit":{"secs":300,"nanos":0},"iterations":3000,"threads":1,"history":256,"hessian":"limited_memory","reuse":"allow_rebuild","start":"no_prior_start","options":{}}"#.to_owned(),
        "blake3:9247c3908515c1cf548cfe3c33772ba236c5d105f031d3cd5351c7cc9ccc387d",
    ));
    let expected = expected
        .into_iter()
        .map(|(label, document, identity)| (label, document, identity.to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(observed, expected);
    // The documents of the other settings types keep their bytes too. (Their identities
    // frame the renamed registry type names: `HighsMethod`, `PounceMethod`,
    // `KinsolStrategy`, `KinsolOrthogonalization`, `ClarabelMode`, `ClarabelMergeMethod`.)
    let document = |backend, fields| {
        let settings = BackendSettings::from_fields(backend, fields).unwrap();
        serde_json::to_string(&settings.document().unwrap()).unwrap()
    };
    if cfg!(feature = "highs") {
        assert_eq!(
            document(Backend::Highs, json!({"method": "ipm"})),
            r#"{"version":1,"backend":"highs","settings":{"method":"ipm","nodes":null,"diagnostics":{"rays":false,"iis":false,"ranging":false,"relaxation":null,"fixed_lp":false,"basis_inverse":null,"presolve":false,"cut_pool":false},"sparse_start":null}}"#
        );
    }
    if cfg!(feature = "kinsol") {
        let fields = json!({
            "strategy": "fixed_point",
            "orthogonalization": "inverse_compact_wy",
            "preconditioner": "jacobi",
        });
        assert_eq!(
            document(Backend::Kinsol, fields),
            r#"{"version":1,"backend":"kinsol","settings":{"strategy":"fixed_point","linear":"klu","anderson":0,"damping":1.0,"setup_interval":10,"max_newton_step":null,"eta":"choice1","preconditioner":"jacobi","orthogonalization":"inverse_compact_wy","anderson_delay":0}}"#
        );
    }
    if cfg!(feature = "pounce") {
        let fields = BackendSettings::from_fields(Backend::Pounce, json!({"method": "l1_exact_penalty"}))
            .unwrap()
            .fields()
            .unwrap();
        assert_eq!(fields["method"], "l1_exact_penalty");
    }
    let fields = BackendSettings::from_fields(
        Backend::Clarabel,
        json!({"mode": "reusable_data", "chordal_decomposition_merge_method": "parent_child"}),
    )
    .unwrap()
    .fields()
    .unwrap();
    assert_eq!(fields["mode"], "reusable_data");
    assert_eq!(fields["chordal_decomposition_merge_method"], "parent_child");
    assert_eq!(
        serde_json::to_string(&crate::dynamics::DiffsolSettings {
            method: crate::dynamics::DiffsolMethod::TrBdf2,
            linear: crate::dynamics::DiffsolLinear::Klu,
        })
        .unwrap(),
        r#"{"method":"tr_bdf2","linear":"klu"}"#
    );
    let idas = json!({
        "linear": {"spgmr": {"dimension": 5, "preconditioner": "jacobi"}},
        "sensitivity": "staggered",
        "initialization": "steady_states",
        "constraints": ["non_negative", "free", "negative"],
    });
    assert_eq!(
        serde_json::to_string(
            &serde_json::from_value::<crate::dynamics::IdasSettings>(idas).unwrap()
        )
        .unwrap(),
        r#"{"linear":{"spgmr":{"dimension":5,"preconditioner":"jacobi"}},"sensitivity":"staggered","initialization":"steady_states","constraints":["non_negative","free","negative"]}"#
    );
}
