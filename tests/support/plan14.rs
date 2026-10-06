// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-backed shared engineering fixtures. No expected value is obtained from the product.
#![allow(
    dead_code,
    reason = "shared acceptance and benchmark fixtures have distinct consumers"
)]
#[path = "workflow_runtime.rs"]
mod workflow_runtime;
use pse_backend_native::execution::BackendSettings;
use pse_backend_native::solve::{Backend, Controls, SolveIntent, SolverSelection};
use pse_ids::SemanticId;
use pse_model::generated::identities::{FitId, InstanceId};
use pse_runtime::math::solves::SolverProfile;
use pse_runtime::workflow::{RunReport, RunResult, Runtime};
use std::collections::BTreeMap;
pub(crate) use workflow_runtime::WorkflowRuntime;
pub(crate) fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("tests/fixtures/plan14").is_dir())
        .unwrap()
        .join("tests/fixtures/plan14")
        .join(name)
}
pub(crate) fn runtime(owner: &WorkflowRuntime) -> Runtime {
    let store=pse_operations::testing::canonical_fixture_store().unwrap();
    owner.register_fixture(store.clone()).unwrap();
    Runtime::from_shared(
        owner.runtime.clone(),
        owner.registry.clone(),
        owner.sessions.clone(),
        pse_runtime::workflow::CanonicalDeployment::new(
            store,
            pse_runtime::workflow::OuterAttestation {
                source: pse_ids::ContentHash::from_bytes([0; 32]),
                build: pse_ids::ContentHash::from_bytes([1; 32]),
            },
            None,
        ),
    )
}
fn package_documents(root: &std::path::Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(
        root: &std::path::Path,
        path: &std::path::Path,
        documents: &mut BTreeMap<String, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, documents);
            } else if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("toml" | "yaml" | "yml" | "pse" | "parquet")
            ) {
                documents.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/"),
                    std::fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut documents = BTreeMap::new();
    visit(root, root, &mut documents);
    documents
}
pub(crate) async fn physical(owner: &WorkflowRuntime) -> pse_runtime::workflow::PhysicalContext {
    let runtime = runtime(owner);
    let root = fixture("")
        .ancestors()
        .find(|p| p.join("packages/reference").is_dir())
        .unwrap()
        .join("packages/reference/physical");
    let texts = package_documents(&root);
    let pool = owner.runtime.pool();
    let bundle = pse_runtime::authoring_driver::document::load_package_documents_owned(
        &texts,
        &owner.registry,
        Default::default(),
        &pool,
        &owner.cancel,
        &owner.sessions.validation_context(&owner.registry).unwrap(),
    )
    .unwrap();
    let documents = pse_runtime::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
        vec![bundle],
        &pool,
        &owner.cancel,
    )
    .unwrap();
    runtime
        .physical_from_documents(&documents, &owner.cancel)
        .await
        .unwrap()
}
pub(crate) fn compiler() -> pse_compiler::workspace::Profile {
    let mut profile = pse_compiler::workspace::Profile {
        class_proof_work: 100_000_000,
        ..Default::default()
    };
    // The simultaneous scientific closure proves all original rows and levels.
    // Conservative symbolic work is independent of its byte reservation.
    profile.assembly.worker_bytes = 16 << 30;
    profile.evaluation.derivative_components = 1_000_000;
    profile.evaluation.operations = 100_000_000;
    profile.evaluation.scratch_bytes = 4 << 30;
    profile.evaluation.provider_calls = 1_000_000;
    profile
}
pub(crate) fn profile(backend: Backend, optimize: bool) -> SolverProfile {
    SolverProfile {
        presolve: Default::default(),
        composition: Default::default(),
        reconstruction: None,
        numerics: Default::default(),
        convexity: Default::default(),
        intent: if optimize {
            SolveIntent::Optimize
        } else if backend == Backend::Kinsol {
            SolveIntent::Root
        } else {
            SolveIntent::FeasiblePoint
        },
        selection: SolverSelection::Explicit(backend),
        controls: Controls::default(),
        backend: BackendSettings::Default,
        sensitivity: None,
    }
}
pub(crate) fn variable(result: &RunResult, id: SemanticId) -> f64 {
    use pse_relations::generated::runtime::solve_variables as wire;
    let batch = result.table("runtime.solve_variables").unwrap();
    let view = wire::View::from_checked(&batch).unwrap();
    (0..view.len())
        .find_map(|i| {
            let r = view.row(i).unwrap();
            (r.symbol_id == id).then_some(r.value).flatten()
        })
        .unwrap()
}
pub(crate) fn near(a: f64, b: f64, tol: f64) {
    assert!(
        a.is_finite() && (a - b).abs() <= tol,
        "{a} != {b}, tolerance {tol}"
    )
}
/// Load the explicit source package closure used by authored seed acceptance and timing.
pub(crate) async fn seed_package(
    owner: &WorkflowRuntime,
) -> pse_runtime::workflow::ModelingPackage {
    seed_package_on(owner, runtime(owner)).await
}
/// [`seed_package`] on an explicit runtime, such as a durable one.
pub(crate) async fn seed_package_on(
    owner: &WorkflowRuntime,
    runtime: Runtime,
) -> pse_runtime::workflow::ModelingPackage {
    use pse_runtime::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};

    let root = fixture("")
        .ancestors()
        .find(|p| p.join("packages/reference").is_dir())
        .unwrap()
        .join("packages/reference");
    let pool = owner.runtime.pool();
    let bundles = [
        "data/oracles/teqp-0.23.1",
        "data/oracles/feos-0.10.1",
        "data/gross-sadowski-2001",
        "data/references",
        "data/nist",
        "data/perry7",
        "data/poling2000",
        "data/oracles/idaes-2.13",
        "data/species",
        "data/ciaaw",
        "seed-data",
        "process",
        "thermodynamics",
        "methods",
        "domain",
        "physical",
    ]
    .into_iter()
    .map(|name| {
        load_package_documents_owned(
            &package_documents(&root.join(name)),
            &owner.registry,
            Default::default(),
            &pool,
            &owner.cancel,
            &owner.sessions.validation_context(&owner.registry).unwrap(),
        )
        .unwrap()
    })
    .collect();
    let documents = OwnedDocumentSet::try_from_bundles(bundles, &pool, &owner.cancel).unwrap();
    let physical = runtime
        .physical_from_documents(&documents, &owner.cancel)
        .await
        .unwrap();
    runtime
        .modeling_from_documents(&documents, physical)
        .await
        .unwrap()
}
pub(crate) fn seed_limits() -> pse_modeling::Limits {
    pse_modeling::Limits {
        items: 1_000_000,
        // Complete PC-SAFT Second support shares the body's finite construction ledger.
        body_occurrences: Some(16_777_216),
        body_slots: Some(65_536),
        ..Default::default()
    }
}
pub(crate) async fn heat_fit(
    package: &pse_runtime::workflow::ModelingPackage,
    kind: &str,
) -> (FitId, pse_runtime::workflow::FitProfile) {
    let id = match kind {
        "steady" => "d8e1e58d1db25d97b8c66a4503e12c8e",
        "transient" => "5f011b848cb65d76ad4f40a0feee0125",
        "mixed" => "21770d8f6a0c5d8ab9ce836154608fb6",
        "unidentifiable" => "a9fe50aaf6445a6b9a8af098f85d3c35",
        _ => panic!("unknown authored heat fit"),
    };
    let mut solver = profile(Backend::Ipopt, true);
    solver.presolve = pse_backend_native::presolve::Policy::Off;
    solver.controls.hessian = pse_backend_native::solve::HessianMode::LimitedMemory;
    solver.controls.time_limit = std::time::Duration::from_secs(600);
    let mut simulations = BTreeMap::new();
    if kind != "steady" {
        let simulation = package
            .declared_simulation(
                SemanticId::parse_hex("29dd6a1a3e444acfbf14992087f9d32c")
                    .unwrap()
                    .into(),
                compiler(),
                None,
                seed_limits(),
                &pse_runtime::CancelSource::new(),
            )
            .await
            .unwrap();
        let experiment = match kind {
            "transient" => "b39f24e05b7d5490904f6138b4d7e080",
            "mixed" => "1ea23c5bc42c536da3d7dfca12ad7bed",
            _ => "9d78ac0bbeb65c4ea555cbb40bf336dc",
        };
        let mut integration = simulation.profile().clone();
        integration.method = pse_backend_native::dynamics::Method::Idas;
        integration.rtol = 1e-8;
        integration.atol.fill(1e-10);
        simulations.insert(
            InstanceId::from(SemanticId::parse_hex(experiment).unwrap()),
            integration,
        );
    }
    (
        FitId::from(SemanticId::parse_hex(id).unwrap()),
        pse_runtime::workflow::FitProfile {
            solver,
            simulations,
            rank_tolerance: 1e-8,
            max_cells: 1 << 20,
            derivatives: pse_runtime::workflow::FitDerivatives::Responses,
            uncertainty: None,
        },
    )
}

/// Prepare a source-authored steady case with the requested native policy.
pub(crate) async fn seed_prepare(
    package: &pse_runtime::workflow::ModelingPackage,
    case: SemanticId,
    solver: SolverProfile,
    cancel: &pse_runtime::CancelSource,
) -> Result<pse_runtime::workflow::ModelingSolvePreparation, pse_runtime::workflow::WorkflowError> {
    let analysis = package
        .declared_execution(
            case.into(),
            compiler(),
            solver,
            Default::default(),
            seed_limits(),
            cancel,
        )
        .await?
        .analysis;
    package.prepare_analysis(&analysis, cancel).await
}
/// Both original physical checks and native feasibility are required.
pub(crate) fn authored_success(result: &RunResult) -> &pse_runtime::workflow::ModelingResult {
    let RunReport::Modeling(report) = result.report().unwrap() else {
        panic!("expected authored solve")
    };
    let report = &report[0];
    assert!(report.accepted, "{:?}", report.diagnostic());
    assert!(result.usable(), "{:?}", result.assessments());
    assert!(!report.checks.is_empty());
    assert!(report.checks.iter().all(|r| r.satisfied));
    report
}
