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
    Runtime::from_shared(
        owner.runtime.clone(),
        owner.registry.clone(),
        owner.sessions.clone(),
    )
}
pub(crate) async fn physical(owner: &WorkflowRuntime) -> pse_runtime::workflow::PhysicalContext {
    let runtime = runtime(owner);
    let texts = BTreeMap::from([
        (
            "package.toml".into(),
            std::fs::read_to_string(fixture("../packages/physical-primitives/package.toml"))
                .unwrap(),
        ),
        (
            "materials/physical.yaml".into(),
            std::fs::read_to_string(fixture(
                "../packages/physical-primitives/materials/physical.yaml",
            ))
            .unwrap(),
        ),
    ]);
    let pool = owner.runtime.pool();
    let bundle = pse_runtime::authoring_driver::document::load_package_texts_owned(
        &texts,
        &owner.registry,
        Default::default(),
        &pool,
        &owner.cancel,
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
    Default::default()
}
pub(crate) fn profile(backend: Backend, optimize: bool) -> SolverProfile {
    SolverProfile {
        presolve: Default::default(),
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
/// A durable runtime over `owner` whose runs are attempts in the operational store at
/// `url`; only durable runs publish (ADR-0112 Outcome 16).
pub(crate) async fn durable(owner: &WorkflowRuntime, url: &str) -> Runtime {
    use pse_runtime::workflow::{Durability, LeasePolicy, Operations};
    let operations = Operations::connect(url, "plan14", LeasePolicy::default())
        .await
        .unwrap();
    runtime(owner).with_durability(Durability::Durable(operations))
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
    use pse_runtime::authoring_driver::document::{OwnedDocumentSet, load_package_texts_owned};
    fn documents(root: &std::path::Path) -> BTreeMap<String, String> {
        fn visit(
            root: &std::path::Path,
            path: &std::path::Path,
            texts: &mut BTreeMap<String, String>,
        ) {
            for entry in std::fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    visit(root, &path, texts);
                } else if matches!(
                    path.extension().and_then(|s| s.to_str()),
                    Some("toml" | "yaml" | "yml" | "pse")
                ) {
                    let mut text = std::fs::read_to_string(&path).unwrap();
                    if path.extension().and_then(|s| s.to_str()) == Some("pse") {
                        // Select workload fixtures through the language IR. Production
                        // definitions, functions, tables and measurements remain intact.
                        let mut rows = pse_authoring::language::parse(
                            &text,
                            SemanticId::NIL,
                            pse_authoring::language::IdentityPolicy::Explicit,
                            Default::default(),
                        )
                        .unwrap();
                        let selected = [
                            "29dd6a1a3e444acfbf14992087f9d32c",
                            "f1b94720fef75b3bab2e29090ef0c315",
                            "68ba8dc2d6b05d9a9fe1b1a3625d8015",
                            "040af20814bc57abb565c3c7f680be05",
                            "079a378ba3ce46728b32c87f4fe6a3df",
                            "fc52409793e44e61adb3eff88946fdb6",
                            "efcd1d0ad288438daf6764b4ab25a2a6",
                            "8c22c4a4f87141b083bfc0d9442d382c",
                            "d84e844726e64a2c9b23d96a4039b4f9",
                        ]
                        .map(|id| SemanticId::parse_hex(id).unwrap());
                        let mut removed = rows
                            .iter()
                            .filter(|r| {
                                r.value.kind.as_str() == "test"
                                    && !selected.contains(&r.declaration_id.as_id())
                            })
                            .map(|r| r.declaration_id)
                            .collect::<std::collections::BTreeSet<_>>();
                        loop {
                            let descendants = rows
                                .iter()
                                .filter(|r| r.parent_id.is_some_and(|p| removed.contains(&p)))
                                .map(|r| r.declaration_id)
                                .collect::<Vec<_>>();
                            let old = removed.len();
                            removed.extend(descendants);
                            if old == removed.len() {
                                break;
                            }
                        }
                        rows.retain(|r| !removed.contains(&r.declaration_id));
                        text = pse_authoring::language::render(&rows).unwrap();
                    }
                    texts.insert(
                        path.strip_prefix(root)
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .replace('\\', "/"),
                        text,
                    );
                }
            }
        }
        let mut texts = BTreeMap::new();
        visit(root, root, &mut texts);
        texts
    }
    let root = fixture("")
        .ancestors()
        .find(|p| p.join("packages/reference").is_dir())
        .unwrap()
        .join("packages/reference");
    let pool = owner.runtime.pool();
    let physical_bundle = load_package_texts_owned(
        &documents(&root.join("physical")),
        &owner.registry,
        Default::default(),
        &pool,
        &owner.cancel,
    )
    .unwrap();
    let physical_documents =
        OwnedDocumentSet::try_from_bundles(vec![physical_bundle], &pool, &owner.cancel).unwrap();
    let physical = runtime
        .physical_from_documents(&physical_documents, &owner.cancel)
        .await
        .unwrap();
    let bundles = [
        "seed-data",
        "process",
        "thermodynamics",
        "methods",
        "physical",
    ]
    .into_iter()
    .map(|name| {
        load_package_texts_owned(
            &documents(&root.join(name)),
            &owner.registry,
            Default::default(),
            &pool,
            &owner.cancel,
        )
        .unwrap()
    })
    .collect();
    let documents = OwnedDocumentSet::try_from_bundles(bundles, &pool, &owner.cancel).unwrap();
    runtime
        .modeling_from_documents(&documents, physical)
        .unwrap()
}
pub(crate) fn seed_limits() -> pse_modeling::Limits {
    pse_modeling::Limits {
        items: 1_000_000,
        body_occurrences: Some(65_536),
        ..Default::default()
    }
}
pub(crate) async fn heat_fit(
    package: &pse_runtime::workflow::ModelingPackage,
    kind: &str,
) -> (SemanticId, pse_runtime::workflow::FitProfile) {
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
        simulations.insert(SemanticId::parse_hex(experiment).unwrap(), integration);
    }
    (
        SemanticId::parse_hex(id).unwrap(),
        pse_runtime::workflow::FitProfile {
            solver,
            simulations,
            modes: BTreeMap::new(),
            rank_tolerance: 1e-8,
            max_cells: 1 << 20,
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
        .declared_analysis(
            case.into(),
            pse_relations::generated::enums::ModelingAnalysisRoute::Steady,
            compiler(),
            solver,
            Default::default(),
            seed_limits(),
            cancel,
        )
        .await?;
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
