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
                    for (coefficients, quadratic) in [(false, false), (true, false), (true, true)] {
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
