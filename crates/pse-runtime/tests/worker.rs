// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Canonical worker child-process journeys: original authored science, retained seeds,
//! truthful killed-writer recovery, native cancellation and competing study claims.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr,
    reason = "integration assertions and child-process protocol"
)]
#[path = "../../../tests/fixtures/native-worker-environment.rs"]
mod native_worker_environment;

use futures_util::FutureExt;
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_backend_native::{
    presolve::PolicyKind,
    solve::{Backend, SolveIntent},
};
use pse_model::study::{OccurrenceKey, StartPolicy};
use pse_operations::native_service::{
    NativeCanonicalService, NativeModuleSource, NativeServiceSpec,
};
use pse_runtime::{
    CancelSource, SharedRuntime,
    math::{settings::SolveSettings, solves::Outcome},
    workflow::{
        CaseOperation, Durability, DurabilitySelection, LeasePolicy, ModelingPackage,
        NativeAllocation, NativeStudyWorker, NativeStudyWorkerSpec, OperationRequest, Operations,
        PackageSources, PhysicalContext, PointOverlay, PreparationSettings, RunReport, RunRequest,
        Runtime, StartSource, StoredStart, StudyHandle, StudyPlan, StudyPoint, StudyPointPolicy,
        StudyPointState, StudyState, WorkerSettings,
    },
};
use std::{
    collections::BTreeMap,
    num::NonZeroUsize,
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

/// Own only newly created private canonical state and the original scientific roots.
/// No configured database, legacy receiver metadata or unit-name adoption participates.
struct ManagedWorkerCase {
    service: NativeCanonicalService,
    allocation: NativeAllocation,
    workers: Vec<NativeStudyWorker>,
    runtimes: Vec<Runtime>,
    shared: Vec<Arc<SharedRuntime>>,
    children: Vec<tokio::process::Child>,
    checkpoints: Vec<PathBuf>,
}
fn allocation_for_workers(
    selected: NativeAllocation,
    workers: usize,
    observer: usize,
) -> NativeAllocation {
    let mut execution = selected.execution.unwrap();
    execution.observer_memory_bytes = execution.observer_memory_bytes.max(observer);
    let process = (selected.total_memory_bytes
        - selected.server_memory_bytes
        - execution.observer_memory_bytes)
        / workers;
    execution.pool_memory_bytes = process - execution.process_headroom_bytes;
    execution.worker_bytes = execution.worker_bytes.min(execution.pool_memory_bytes / 2);
    NativeAllocation::materialize(
        selected.total_memory_bytes,
        selected.server_memory_bytes,
        workers,
        process,
        Some(execution),
    )
    .unwrap()
}

impl ManagedWorkerCase {
    fn new(workers: usize) -> Self {
        let environment = std::env::vars_os().collect();
        let module_source =
            if let Some(receipt) = std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_RECEIPT") {
                NativeModuleSource::Prepared(
                    pse_host::study_module::select(
                        &PathBuf::from(receipt),
                        pse_host::study_module::Variant::Production,
                    )
                    .unwrap(),
                )
            } else {
                NativeModuleSource::RecordedState(PathBuf::from(
                    std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_STATE")
                        .expect("explicit prepared native module receipt selector"),
                ))
            };
        let helper = PathBuf::from(
            std::env::var_os("PSE_NATIVE_FIXTURE_SERVICE_HELPER")
                .expect("root supplies the already-built canonical-service helper"),
        );
        assert!(helper.is_absolute() && helper.is_file());
        // Share the declared workstation envelope across these original processes;
        // no independently invented small fixture pool can block default preparation.
        let profiles = pse_runtime::workflow::ProductionProfiles::read(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.config/agent-capacity.toml"),
        )
        .unwrap();
        let selected = profiles.select(profiles.default_profile).unwrap();
        let allocation = allocation_for_workers(
            selected,
            workers,
            selected.execution.unwrap().observer_memory_bytes,
        );
        let service = NativeCanonicalService::new(NativeServiceSpec {
            server: pse_host::server::select(&environment).unwrap(),
            helper,
            module_source,
            state_parent: std::env::temp_dir(),
        })
        .unwrap();
        eprintln!(
            "worker journey private state={} original_service={}",
            service.state().display(),
            service.intent().unit()
        );
        Self {
            service,
            allocation,
            workers: Vec::new(),
            runtimes: Vec::new(),
            shared: Vec::new(),
            children: Vec::new(),
            checkpoints: Vec::new(),
        }
    }
    async fn launch(&mut self) {
        self.service
            .launch(Instant::now() + Duration::from_secs(90))
            .await
            .unwrap();
    }
    fn runtime(&mut self, scip: bool) -> (Arc<SharedRuntime>, Runtime) {
        let store = self.service.ready().unwrap().store().clone();
        let (shared, local) = if scip {
            scip_runtime(store)
        } else {
            runtime_with_store(store)
        };
        // The actual observer's pool is part of this same envelope before any
        // scientific worker is constructed; SCIP and ordinary callers may differ.
        assert!(self.workers.is_empty());
        self.allocation = allocation_for_workers(
            self.allocation,
            self.allocation.native_workers,
            shared.budget().memory_limit_bytes.get(),
        );
        self.runtimes.push(local.clone());
        self.shared.push(shared.clone());
        (shared, local)
    }
    async fn spawn(&mut self) -> usize {
        let executable = PathBuf::from(
            std::env::var_os("CARGO_BIN_EXE_pse-worker")
                .expect("Cargo/nextest supplies the selected runtime worker executable"),
        );
        assert!(executable.is_absolute() && executable.is_file());
        let index = self.workers.len();
        self.workers.push(
            NativeStudyWorker::new(NativeStudyWorkerSpec {
                ready: self.service.ready().unwrap(),
                executable,
                environment: native_worker_environment::selected(),
                allocation: self.allocation,
                settings: WorkerSettings {
                    poll: Duration::from_millis(20),
                    recovery: Duration::from_millis(200),
                    // The competition fixture needs each process to claim an original
                    // point. This scheduling bound does not alter its 16/32 CPU/native profile.
                    maximum_in_flight: (self.allocation.native_workers == 2).then_some(1),
                    ..Default::default()
                },
            })
            .unwrap(),
        );
        // Retain the original owner before launch: uncertain startup still needs drain.
        self.workers[index]
            .launch(Instant::now() + Duration::from_secs(90))
            .await
            .unwrap();
        index
    }
    fn bind_worker(&self, local: Runtime, index: usize) -> Runtime {
        local
            .with_managed_worker(self.workers[index].observer().unwrap())
            .unwrap()
    }
    async fn stop_worker(&mut self, index: usize) {
        self.workers[index]
            .shutdown(Instant::now() + Duration::from_secs(45))
            .await
            .unwrap();
        let terminal = self.workers[index]
            .terminal()
            .unwrap()
            .expect("original scientific application join receipt, separate from host drain");
        assert!(
            terminal.native_joined,
            "original application must join its native lanes"
        );
        assert!(
            terminal.failure.is_none(),
            "original worker failure: {:?}",
            terminal.failure
        );
        assert!(
            terminal.processed.is_some(),
            "original scheduling count must be retained"
        );
    }
    fn worker_identity(&self, index: usize) -> String {
        // The real application chooses its process-scoped identity. Check the
        // canonical writer against the authenticated original terminal receipt.
        let terminal =
            serde_json::to_value(self.workers[index].terminal().unwrap().unwrap()).unwrap();
        let pid = terminal["pid"].as_u64().unwrap();
        let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".into());
        format!("pse-worker:{host}:{pid}")
    }
    fn child_logs(&self, name: &str) -> (std::fs::File, std::fs::File) {
        use std::os::unix::fs::OpenOptionsExt;
        let open = |suffix| {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(self.service.state().join(format!("{name}.{suffix}")))
                .unwrap()
        };
        (open("stdout"), open("stderr"))
    }
    async fn finish(&mut self, body: std::thread::Result<()>) {
        // Settle all independent scientific roots before closing their shared store.
        // Retain all failures; a panic never substitutes Drop for asynchronous cleanup.
        for local in &self.runtimes {
            local.close();
            local.clear_program_cache();
        }
        let (worker_drains, child_drains, math_drains) = tokio::join!(
            futures_util::future::join_all(
                self.workers
                    .iter_mut()
                    .map(|worker| worker.shutdown(Instant::now() + Duration::from_secs(45)))
            ),
            futures_util::future::join_all(self.children.iter_mut().map(|child| async {
                if child.try_wait()?.is_none() {
                    child.start_kill()?;
                }
                tokio::time::timeout(Duration::from_secs(45), child.wait())
                    .await
                    .map_err(std::io::Error::other)?
            })),
            futures_util::future::join_all(
                self.shared
                    .iter()
                    .map(|shared| shared.math().shutdown_native_lanes())
            ),
        );
        let service_drain = self
            .service
            .shutdown(Instant::now() + Duration::from_secs(45))
            .await;
        let settled = worker_drains.iter().all(Result::is_ok)
            && child_drains.iter().all(Result::is_ok)
            && math_drains.iter().all(Result::is_ok)
            && service_drain.is_ok();
        if body.is_err() || !settled {
            eprintln!(
                "worker journey retained state={}; worker drains={worker_drains:?}; child drains={child_drains:?}; math drains={math_drains:?}; canonical drain={service_drain:?}",
                self.service.state().display()
            );
            for worker in &self.workers {
                eprintln!(
                    "original worker diagnostics={} unit={}",
                    worker.directory().display(),
                    worker.intent().unit()
                );
            }
        }
        if let Err(panic) = body {
            std::panic::resume_unwind(panic);
        }
        assert!(
            settled,
            "all owned physical/connection/native drains must succeed"
        );
        // Attempt every independently owned disposal; preserve individual errors.
        let worker_disposals = self
            .workers
            .iter_mut()
            .map(NativeStudyWorker::dispose)
            .collect::<Vec<_>>();
        let checkpoint_disposals = self
            .checkpoints
            .iter()
            .map(std::fs::remove_file)
            .collect::<Vec<_>>();
        let disposed = worker_disposals.iter().all(Result::is_ok)
            && checkpoint_disposals.iter().all(Result::is_ok);
        // Retain canonical diagnosis if any independent diagnostic disposal failed.
        let service_disposal = if disposed {
            self.service.dispose()
        } else {
            Ok(())
        };
        if !disposed || service_disposal.is_err() {
            eprintln!(
                "worker journey disposal retained state={}; workers={worker_disposals:?}; checkpoints={checkpoint_disposals:?}; canonical={service_disposal:?}",
                self.service.state().display()
            );
        }
        assert!(disposed, "every original diagnostic disposal must succeed");
        service_disposal.unwrap();
    }
}

const SQUARE: &str = r#"package algebraic { def Root {
    var x:Scalar;
    eq square:x*x==4;
    annotation start x(1);
    annotation bounds x(0,10);
    annotation report x("root");
    annotation check x(x>1);
} }"#;

fn texts(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut texts = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let key = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/");
                texts.insert(key, std::fs::read(&path).unwrap());
            }
        }
    }
    texts
}

fn sources() -> (BTreeMap<String, Vec<u8>>, BTreeMap<String, Vec<u8>>) {
    sources_of(SQUARE)
}

/// The sources of a one-document package over the physical primitives fixture.
fn sources_of(source: &str) -> (BTreeMap<String, Vec<u8>>, BTreeMap<String, Vec<u8>>) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
    let manifest = std::fs::read_to_string(fixtures.join("minimal_explicit/package.toml"))
        .unwrap()
        .replace(r#"id_policy = "explicit""#, r#"id_policy = "named""#)
        // The primitives declare `Scalar`; depending on them makes it visible (ADR-0123
        // Outcome 6).
        .replace(
            "dependencies = []",
            r#"dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]"#,
        );
    (
        texts(&fixtures.join("physical-primitives")),
        BTreeMap::from([
            ("package.toml".to_owned(), manifest.into_bytes()),
            ("models/root.pse".to_owned(), source.as_bytes().to_vec()),
        ]),
    )
}

fn build_identity() -> pse_ids::ContentHash {
    let mut executable = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
    hash.part_reader(
        executable.metadata().unwrap().len(),
        &mut executable,
        &|| Ok(()),
    )
    .unwrap();
    hash.finish_hash()
}

fn runtime_with_store(
    store: pse_operations::canonical::CanonicalStore,
) -> (Arc<SharedRuntime>, Runtime) {
    let n = |v| NonZeroUsize::new(v).unwrap();
    let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
        memory_limit_bytes: n(1 << 30),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: n(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(32),
            target_partitions: n(16),
        },
        execution: Default::default(),
        cache: pse_runtime::CacheBudget::disabled(1024),
        math: pse_runtime::math::MathPolicy {
            // Default compilation reserves its declared scratch allowance; the positive
            // deployment also admits the original evaluator and result buffers.
            worker_bytes: pse_compiler::workspace::Profile::default()
                .evaluation
                .scratch_bytes
                + (64 << 20),
            workspace_bytes: 128 << 20,
            foreign_bytes: 1 << 20,
            jobs: 32,
            ..Default::default()
        },
    })
    .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let sessions = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .unwrap(),
    );
    (
        shared.clone(),
        Runtime::from_shared(
            shared,
            sessions.bind_model(registry.clone()).unwrap(),
            pse_runtime::workflow::CanonicalDeployment::new(
                store,
                pse_runtime::workflow::OuterAttestation {
                    source: None,
                    build: build_identity(),
                },
                None,
            ),
        )
        .unwrap(),
    )
}

async fn package(
    shared: &SharedRuntime,
    runtime: &Runtime,
    physical: &BTreeMap<String, Vec<u8>>,
    modeling: &BTreeMap<String, Vec<u8>>,
) -> (ModelingPackage, PhysicalContext) {
    let pool = shared.pool();
    let cancel = pse_columnar::CancellationToken::new();
    let validation = runtime.validation_context();
    let load = |texts| {
        load_package_documents_owned(
            texts,
            runtime.registry(),
            pse_authoring::ParseBudget::default(),
            &pool,
            &cancel,
            &validation,
        )
        .unwrap()
    };
    let physical =
        OwnedDocumentSet::try_from_bundles(vec![load(physical)], &pool, &cancel).unwrap();
    let context = runtime
        .physical_from_documents(&physical, &cancel)
        .await
        .unwrap();
    let modeling =
        OwnedDocumentSet::try_from_bundles(vec![load(modeling)], &pool, &cancel).unwrap();
    (
        runtime
            .modeling_from_documents(&modeling, context.clone(), &CancelSource::new())
            .await
            .unwrap(),
        context,
    )
}

fn settings() -> SolveSettings {
    SolveSettings {
        intent: SolveIntent::FeasiblePoint,
        backend: Some(Backend::Ipopt),
        presolve: PolicyKind::Off,
        ..SolveSettings::default()
    }
}

/// A package whose `Root` squares to a value its data document supplies (ADR-0125).
const DATA_SQUARE: &str = r#"package algebraic {
    entity kind source provenance { attribute title: Text; }
    enum role { given }
    entity source s { title = "KR9 durable job" }
    table target[n: 1..1]: Scalar storage {dimensionless} complete_over(n in 1..1) missing required;
    dataset targets: target provenance(s, role.given) from "data/target.parquet";
    def Root {
        var x:Scalar;
        eq square:x*x==target[1];
        annotation start x(1);
        annotation bounds x(0,10);
        annotation report x("root");
    }
}"#;

/// The Parquet bytes of `target`: one row, `n = 1`, `value = 9`.
fn target_document() -> Vec<u8> {
    use datafusion::arrow::{
        array::{ArrayRef, Float64Array, Int64Array, RecordBatch},
        datatypes::{DataType, Field, Schema},
    };
    let schema = Arc::new(Schema::new(vec![
        Field::new("n", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]));
    let columns: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from(vec![1])),
        Arc::new(Float64Array::from(vec![9.0])),
    ];
    let batch = RecordBatch::try_new(Arc::clone(&schema), columns).unwrap();
    let mut bytes = Vec::new();
    let mut writer = parquet::arrow::ArrowWriter::try_new(&mut bytes, schema, None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
    bytes
}

/// The physical primitives with a dimensionless indicator kind and type, so an authored
/// model can declare binary decisions (ADR-0103: a discrete domain needs a count or
/// indicator quantity).
fn physical_with_indicator() -> BTreeMap<String, Vec<u8>> {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
    let mut physical = texts(&fixtures.join("physical-primitives"));
    let bytes = physical.get_mut("materials/physical.yaml").unwrap();
    let document = String::from_utf8(std::mem::take(bytes)).unwrap();
    let zero = r#"{"num": 0, "den": 1}"#;
    let dimension = [zero; 8].join(", ");
    let kind = format!(
        r#""quantity_kinds": [
    {{"quantity_kind_id": "18181818181818181818181818181818", "name": "indicator",
      "dimension": [{dimension}], "extensive": false, "addition_kind": "additive",
      "category": "indicator", "doc": "Zero-or-one decisions."}},"#
    );
    let ty = r#""quantity_types": [
    {"quantity_type_id": "1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c", "name": "Indicator",
      "quantity_kind_id": "18181818181818181818181818181818", "basis_id": null,
      "reference_state_id": null, "scale_kind": "point", "shape": [], "subject_kind": null,
      "canonical_unit_id": "0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a", "nominal_magnitude": null,
      "doc": "Zero-or-one decision."},"#;
    *bytes = document
        .replacen(r#""quantity_kinds": ["#, &kind, 1)
        .replacen(r#""quantity_types": ["#, ty, 1)
        .into_bytes();
    physical
}

/// A market-split problem with a squared deviation: `min Σᵢ sᵢ²` where
/// `sᵢ = Σⱼ aᵢⱼ·xⱼ − bᵢ` over 30 binary decisions and four rows, `bᵢ = ⌊Σⱼ aᵢⱼ / 2⌋`.
/// SCIP finds incumbents at once (the empty split is one) but proving optimality takes far
/// longer than any test allows, so a time limit ends every try. Deterministic coefficients.
fn market_split() -> String {
    const ROWS: usize = 4;
    const COLUMNS: usize = 30;
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % 100
    };
    let mut body = String::new();
    for j in 0..COLUMNS {
        body.push_str(&format!(
            "var x{j}: Indicator in binary; annotation start x{j}(0{{dimensionless}}); "
        ));
    }
    let mut deviation = Vec::new();
    for i in 0..ROWS {
        let a: Vec<u64> = (0..COLUMNS).map(|_| next()).collect();
        let b = a.iter().sum::<u64>() / 2;
        let terms: Vec<String> = a
            .iter()
            .enumerate()
            .map(|(j, a)| format!("{a}*x{j}"))
            .collect();
        body.push_str(&format!(
            "var s{i}: Scalar; annotation bounds s{i}(-5000, 5000); annotation start s{i}(0); \
             eq split{i}: {} - s{i} == {b}; ",
            terms.join(" + ")
        ));
        deviation.push(format!("s{i}*s{i}"));
    }
    format!(
        "package algebraic {{ def Root {{ {body}\
         let deviation: Scalar = {}; \
         annotation objective deviation(minimize); annotation report deviation(\"deviation\"); }} }}",
        deviation.join(" + ")
    )
}

/// The modeling sources of `source`, which sees the primitives' `Scalar` and `Indicator`.
fn discrete_sources(source: &str) -> BTreeMap<String, Vec<u8>> {
    let (_, mut modeling) = sources();
    modeling.insert("models/root.pse".to_owned(), source.as_bytes().to_vec());
    modeling
}

/// A scientific observer runtime with the original eight-GiB pool, admitting the
/// actual SCIP foreign-memory policy while retaining the host's 16/32 widths.
fn scip_runtime(store: pse_operations::canonical::CanonicalStore) -> (Arc<SharedRuntime>, Runtime) {
    let n = |v| NonZeroUsize::new(v).unwrap();
    let memory: usize = 8 << 30;
    let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
        memory_limit_bytes: n(memory),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: n(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(32),
            target_partitions: n(16),
        },
        execution: Default::default(),
        cache: pse_runtime::CacheBudget::disabled(1024),
        math: pse_runtime::math::MathPolicy {
            workspace_bytes: memory / 8,
            worker_bytes: memory / 16,
            artifact_bytes: memory / 8,
            jobs: 32,
            ..Default::default()
        },
    })
    .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let sessions = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .unwrap(),
    );
    (
        shared.clone(),
        Runtime::from_shared(
            shared,
            sessions.bind_model(registry.clone()).unwrap(),
            pse_runtime::workflow::CanonicalDeployment::new(
                store,
                pse_runtime::workflow::OuterAttestation {
                    source: None,
                    build: build_identity(),
                },
                None,
            ),
        )
        .unwrap(),
    )
}

/// `Root` with the right-hand side of its equation as parameter `a`: each study point sets
/// its own value.
const PARAMETRIC: &str = r#"package algebraic { def Root {
    param a:Scalar = 4;
    var x:Scalar;
    eq square:x*x==a;
    annotation start x(1);
    annotation bounds x(0,10);
    annotation report x("root");
    annotation check x(x>1);
} }"#;

fn bind_operations(
    _shared: &SharedRuntime,
    local: Runtime,
    worker: &str,
    policy: LeasePolicy,
) -> (Runtime, Operations) {
    let local = local
        .with_durability(DurabilitySelection::Durable {
            worker: worker.into(),
            policy,
        })
        .unwrap();
    let Durability::Durable(operations) = local.durability() else {
        panic!("accepted durable selection must install its operations owner")
    };
    let operations = operations.as_ref().clone();
    (local, operations)
}
fn occurrence(
    case: pse_model::generated::identities::DeclarationId,
    key: u32,
    settings: SolveSettings,
    attempt_limit: u32,
) -> StudyPoint {
    StudyPoint {
        operation: OperationRequest::DeclaredCase(CaseOperation {
            case,
            route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
            settings,
        }),
        preparation: PreparationSettings::default(),
        overlay: PointOverlay::default(),
        policy: StudyPointPolicy {
            key: OccurrenceKey(key),
            dependencies: vec![],
            start: StartPolicy::Fresh,
            attempt_limit,
        },
    }
}
async fn submit_single(
    shared: &SharedRuntime,
    local: &Runtime,
    physical: BTreeMap<String, Vec<u8>>,
    modeling: BTreeMap<String, Vec<u8>>,
    settings: SolveSettings,
    attempt_limit: u32,
) -> (
    ModelingPackage,
    pse_model::generated::identities::DeclarationId,
    StudyHandle,
) {
    let (package, _) = package(shared, local, &physical, &modeling).await;
    let case = package
        .declarations(&local.preparation_services())
        .await
        .unwrap()
        .iter()
        .find(|d| d.name == "Root")
        .unwrap()
        .declaration_id;
    let handle = local
        .start_study(
            StudyPlan {
                output: pse_runtime::workflow::StudyOutputDemand::Full {},
                sources: PackageSources {
                    physical,
                    modeling: vec![modeling],
                },
                inputs: pse_runtime::workflow::StudyInputs::Explicit(vec![occurrence(
                    case,
                    0,
                    settings,
                    attempt_limit,
                )]),
            },
            &CancelSource::new(),
        )
        .await
        .unwrap();
    (package, case, handle)
}
async fn drain_worker(handle: &StudyHandle, worker: usize, managed: &mut ManagedWorkerCase) {
    until(Duration::from_secs(180), "canonical study conclusion", || async {
        if handle.status().await.unwrap().state == StudyState::Concluded { return true; }
        let terminal=managed.workers[worker].terminal().unwrap();
        assert!(terminal.is_none(), "original scientific worker ended before canonical conclusion: {terminal:?}; diagnostics={}", managed.workers[worker].directory().display());
        false
    }).await;
    // Sustained applications stop only after canonical scientific facts are visible.
    managed.stop_worker(worker).await;
}
async fn until<F: Future<Output = bool>>(
    limit: Duration,
    what: &str,
    mut probe: impl FnMut() -> F,
) {
    let deadline = tokio::time::Instant::now() + limit;
    while !probe().await {
        assert!(tokio::time::Instant::now() < deadline, "timed out: {what}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
async fn assert_value(local: &Runtime, run: &str, attempt: &str, wanted: f64) {
    let mut stream = local
        .results(
            run,
            attempt,
            "runtime.solve_variables",
            0,
            u64::MAX,
            pse_columnar::CancellationToken::new(),
        )
        .await
        .unwrap();
    let validation = local.validation_context();
    let mut variables = 0;
    while let Some(batch) = stream.next_batch().await.unwrap() {
        let view =
            pse_relations::generated::runtime::solve_variables::View::try_from_batch_with_registry(
                local.registry(),
                &batch,
                &validation,
            )
            .unwrap();
        for row in view.rows().unwrap() {
            if !row.parameter && !row.fixed {
                assert_eq!(row.step, 0);
                let value = row.value.expect("retained original variable value");
                let allowance = row
                    .tolerance
                    .expect("published production variable allowance");
                assert!(allowance.is_finite() && allowance > 0.);
                assert!(
                    (value - wanted).abs() <= allowance,
                    "original variable {}: {value}, expected {wanted}, allowance {allowance}",
                    row.symbol_id
                );
                variables += 1;
            }
        }
    }
    assert_eq!(
        variables, 1,
        "one original free scalar variable, excluding fixed parameters"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[allow(
    clippy::too_many_lines,
    reason = "two original native process owners retain the same authored scientific case"
)]
async fn worker_replays_same_authored_case_in_a_fresh_default_process() {
    let mut managed = ManagedWorkerCase::new(1);
    let body =
        AssertUnwindSafe(async {
            managed.launch().await;
            let (shared, local) = managed.runtime(false);
            let (local, operations) =
                bind_operations(&shared, local, "replay-enqueuer", LeasePolicy::default());
            let (physical, modeling) = sources();
            let (package, _) = package(&shared, &local, &physical, &modeling).await;
            let case = package
                .declarations(&local.preparation_services())
                .await
                .unwrap()
                .iter()
                .find(|d| d.name == "Root")
                .unwrap()
                .declaration_id;
            let mut previous = None;
            let mut previous_worker = None;
            for _ in 0..2 {
                let worker = managed.spawn().await;
                let local = managed.bind_worker(local.clone(), worker);
                let handle =
                    local
                        .start_study(
                            StudyPlan {
                                output: pse_runtime::workflow::StudyOutputDemand::Full {},
                                sources: PackageSources {
                                    physical: physical.clone(),
                                    modeling: vec![modeling.clone()],
                                },
                                inputs: pse_runtime::workflow::StudyInputs::Explicit(vec![
                                    occurrence(case, 0, settings(), 1),
                                ]),
                            },
                            &CancelSource::new(),
                        )
                        .await
                        .unwrap();
                drain_worker(&handle, worker, &mut managed).await;
                let writer = managed.worker_identity(worker);
                if let Some(previous) = previous_worker {
                    assert_ne!(writer, previous);
                }
                previous_worker = Some(writer.clone());
                let status = handle.status().await.unwrap();
                assert_eq!(status.state, StudyState::Concluded);
                assert_eq!(status.points.len(), 1);
                let point = &status.points[0];
                assert_eq!(point.state, StudyPointState::Completed);
                let outcome = point.outcome.as_ref().unwrap();
                assert!(outcome.scientific.usable);
                assert_eq!(outcome.attempts.len(), 1);
                let attempt = point.attempt.as_deref().unwrap();
                let stored = operations
                    .store()
                    .canonical_attempt(attempt)
                    .await
                    .unwrap()
                    .unwrap();
                assert!(stored.terminal());
                assert_eq!(
                    stored.outcome.as_ref().map(|value| value.as_str()),
                    Some("succeeded")
                );
                assert_eq!(stored.worker, writer);
                assert_value(&local, &point.run, attempt, 2.).await;
                let identity = (
                    handle.study_id().to_string(),
                    point.run.clone(),
                    attempt.to_owned(),
                );
                if let Some((study, run, attempt)) = previous {
                    assert_ne!(identity.0, study);
                    assert_ne!(identity.1, run);
                    assert_ne!(identity.2, attempt);
                }
                previous = Some(identity);
            }
            drop(package);
            local.clear_program_cache();
        })
        .catch_unwind()
        .await;
    managed.finish(body).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn worker_runs_authored_case_end_to_end() {
    let mut managed = ManagedWorkerCase::new(1);
    let body = AssertUnwindSafe(async {
        managed.launch().await;
        let (shared, local) = managed.runtime(false);
        let (local, operations) =
            bind_operations(&shared, local, "enqueuer", LeasePolicy::default());
        let worker = managed.spawn().await;
        let local = managed.bind_worker(local, worker);
        let (physical, modeling) = sources();
        let (package, case, handle) =
            submit_single(&shared, &local, physical, modeling, settings(), 1).await;
        drain_worker(&handle, worker, &mut managed).await;
        let status = handle.status().await.unwrap();
        assert_eq!(status.state, StudyState::Concluded);
        let point = &status.points[0];
        assert!(point.outcome.as_ref().unwrap().scientific.usable);
        assert_eq!(point.outcome.as_ref().unwrap().attempts.len(), 1);
        let attempt = point.attempt.as_deref().unwrap();
        let stored = operations
            .store()
            .canonical_attempt(attempt)
            .await
            .unwrap()
            .unwrap();
        assert!(stored.terminal());
        assert_eq!(
            stored.outcome.as_ref().map(|value| value.as_str()),
            Some("succeeded")
        );
        assert_eq!(stored.worker, managed.worker_identity(worker));
        assert_value(&local, &point.run, attempt, 2.).await;
        let mut progress = local
            .progress(&point.run, attempt, pse_columnar::CancellationToken::new())
            .await
            .unwrap();
        let mut count = 0;
        while let Some(page) = progress.next_page().await.unwrap() {
            count += page.len();
        }
        assert!(count > 0, "worker native progress must be retained");
        let diagnostics = local.observations();
        let root = diagnostics.checkpoint_path().parent().unwrap().to_owned();
        let owned = std::fs::read_dir(root)
            .unwrap()
            .filter_map(Result::ok)
            .find_map(|entry| {
                if entry
                    .path()
                    .extension()
                    .is_none_or(|extension| extension != "json")
                {
                    return None;
                }
                let image = pse_runtime::observation::read_checkpoint(&entry.path()).ok()?;
                let lane = image.operations.iter().find(|operation| {
                    operation.sources.iter().any(|source| {
                        source.name == "canonical.attempt" && source.bytes == attempt.as_bytes()
                    })
                })?;
                assert_ne!(
                    lane.id.process_instance,
                    diagnostics.hub().process_instance()
                );
                assert!(
                    lane.sources
                        .iter()
                        .any(|source| source.name == "canonical.run"
                            && source.bytes == point.run.as_bytes())
                );
                assert!(image.operations.iter().any(|operation| {
                    operation.metrics.iter().any(|metric| {
                        metric.name == "native.joined"
                            && metric.latest == pse_diagnostics::observation::Value::Boolean(true)
                    })
                }));
                Some(entry.path())
            })
            .expect("actual managed worker publishes its own attempt-linked checkpoint");
        // The original native owner has drained this process; this is its diagnostic file only.
        managed.checkpoints.push(owned);
        stored_seed_reused_across_processes(
            &local,
            &operations,
            &package,
            case,
            &point.run,
            attempt,
        )
        .await;
        drop(progress);
        drop(handle);
        drop(package);
        local.clear_program_cache();
    })
    .catch_unwind()
    .await;
    managed.finish(body).await;
}

/// The child executes this same selected test only through the explicit branch.
/// Its acknowledged submission survives process::exit, without runtime destructors
/// or a worker already running in the submitting process.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[allow(
    clippy::exit,
    reason = "intentional submitting-process exit is the tested fault model"
)]
async fn durable_submission_survives_submitting_process_exit() {
    const RECEIPT: &str = "PSE_STO_SUBMITTER_RECEIPT";
    const DATABASE: &str = "PSE_STO_SUBMITTER_DATABASE";
    const STATE: &str = "PSE_STO_SUBMITTER_STATE";
    if let Some(receipt) = std::env::var_os(RECEIPT) {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let state = std::env::var_os(STATE).unwrap();
        let mut options =
            pse_operations::canonical::CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = std::env::var(DATABASE).unwrap();
        let store = pse_operations::canonical::CanonicalStore::connect_ready(&options)
            .await
            .unwrap()
            .into_store();
        let (shared, local) = runtime_with_store(store);
        let (local, _) = bind_operations(
            &shared,
            local,
            "departing-submitter",
            LeasePolicy::default(),
        );
        let (physical, modeling) = sources();
        let (_, _, handle) =
            submit_single(&shared, &local, physical, modeling, settings(), 1).await;
        let status = handle.status().await.unwrap();
        assert_ne!(status.state, StudyState::Concluded);
        let payload = serde_json::to_vec(&serde_json::json!({
            "database": options.database, "study": handle.study_id().to_string(),
            "run": status.points[0].run, "pid": std::process::id(),
        }))
        .unwrap();
        assert!(payload.len() < 4096);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(receipt)
            .unwrap();
        file.write_all(&payload).unwrap();
        file.sync_all().unwrap();
        std::process::exit(0);
    }

    let mut managed = ManagedWorkerCase::new(1);
    let body = AssertUnwindSafe(async {
        managed.launch().await;
        let (shared, local) = managed.runtime(false);
        let store = local.canonical_store().unwrap();
        let database = store.database().to_owned();
        let state = store.deployment_state().to_path_buf();
        let directory = state.join(format!("submitter-exit-{}", uuid::Uuid::new_v4().simple()));
        {
            use std::os::unix::fs::DirBuilderExt;
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&directory)
                .unwrap();
        }
        let receipt = directory.join("submitted.json");
        let (stdout, stderr) = managed.child_logs("submitter");
        managed.children.push(
            tokio::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "durable_submission_survives_submitting_process_exit",
                    "--nocapture",
                ])
                .env(RECEIPT, &receipt)
                .env(STATE, &state)
                .env(DATABASE, &database)
                .stdout(stdout)
                .stderr(stderr)
                .spawn()
                .unwrap(),
        );
        let output = tokio::time::timeout(
            Duration::from_secs(120),
            managed.children.last_mut().unwrap().wait(),
        )
        .await
        .expect("bounded submitting process; original child retained for explicit kill/reap")
        .unwrap();
        assert!(
            output.success(),
            "{}",
            String::from_utf8_lossy(&std::fs::read(state.join("submitter.stderr")).unwrap())
        );
        let bytes = std::fs::read(&receipt).unwrap();
        assert!(bytes.len() < 4096);
        let submitted: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_ne!(
            submitted["pid"].as_u64().unwrap(),
            u64::from(std::process::id())
        );
        assert_eq!(submitted["database"], database);
        let (local, operations) =
            bind_operations(&shared, local, "surviving-observer", LeasePolicy::default());
        let study = pse_model::generated::identities::StudyId::from_id(
            pse_ids::SemanticId::parse_hex(submitted["study"].as_str().unwrap()).unwrap(),
        );
        let handle = local.study(study);
        assert_ne!(handle.status().await.unwrap().state, StudyState::Concluded);
        // The submitting process is reaped before this independent worker starts.
        let worker = managed.spawn().await;
        let local = managed.bind_worker(local, worker);
        let handle = local.study(study);
        drain_worker(&handle, worker, &mut managed).await;
        let status = handle.status().await.unwrap();
        assert_eq!(status.state, StudyState::Concluded);
        let point = &status.points[0];
        assert_eq!(point.run, submitted["run"].as_str().unwrap());
        assert!(point.outcome.as_ref().unwrap().scientific.usable);
        assert_eq!(point.outcome.as_ref().unwrap().attempts.len(), 1);
        let attempt = point.attempt.as_deref().unwrap();
        let stored = operations
            .store()
            .canonical_attempt(attempt)
            .await
            .unwrap()
            .unwrap();
        assert!(stored.terminal());
        assert_eq!(stored.worker, managed.worker_identity(worker));
        assert_eq!(
            stored.outcome.as_ref().map(|value| value.as_str()),
            Some("succeeded")
        );
        assert_value(&local, &point.run, attempt, 2.).await;
        drop(handle);
    })
    .catch_unwind()
    .await;
    managed.finish(body).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn durable_worker_round_trips_a_package_with_a_data_document() {
    let mut managed = ManagedWorkerCase::new(1);
    let body = AssertUnwindSafe(async {
        managed.launch().await;
        let (shared, local) = managed.runtime(false);
        let (local, operations) =
            bind_operations(&shared, local, "data-enqueuer", LeasePolicy::default());
        let worker = managed.spawn().await;
        let local = managed.bind_worker(local, worker);
        let (physical, mut modeling) = sources_of(DATA_SQUARE);
        modeling.insert("data/target.parquet".into(), target_document());
        let (package, _, handle) =
            submit_single(&shared, &local, physical, modeling, settings(), 1).await;
        assert!(
            operations
                .store()
                .revision(&package.canonical_revision().unwrap().key)
                .await
                .unwrap()
                .is_some()
        );
        drain_worker(&handle, worker, &mut managed).await;
        let status = handle.status().await.unwrap();
        assert_eq!(status.state, StudyState::Concluded);
        let point = &status.points[0];
        assert!(point.outcome.as_ref().unwrap().scientific.usable);
        assert_value(&local, &point.run, point.attempt.as_deref().unwrap(), 3.).await;
        drop(handle);
        drop(package);
        local.clear_program_cache();
    })
    .catch_unwind()
    .await;
    managed.finish(body).await;
}

async fn stored_seed_reused_across_processes(
    local: &Runtime,
    operations: &Operations,
    package: &ModelingPackage,
    case: pse_model::generated::identities::DeclarationId,
    run: &str,
    attempt: &str,
) {
    let cancel = CancelSource::new();
    let record = operations.record(run, attempt).await.unwrap();
    let (step, solution) = record.solutions[0];
    assert_eq!(step, 0);
    let stored = operations
        .store()
        .result_seed(&solution.to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.attempt, attempt);
    assert_eq!(stored.run, run);
    let analysis = package
        .declared_execution(
            &local.preparation_services(),
            case,
            Default::default(),
            settings().profile().unwrap(),
            Default::default(),
            pse_modeling::Limits::default(),
            &cancel,
        )
        .await
        .unwrap();
    let prepared = package
        .prepare_declared(&local.preparation_services(), &analysis, &cancel)
        .await
        .unwrap();
    let seeded = prepared
        .with_stored_start(operations, StoredStart::Solution(solution))
        .await
        .unwrap();
    assert!(seeded.starts.values().any(
        |source| matches!(source,StartSource::Stored{solution:actual} if *actual==solution.as_id())
    ));
    let result = seeded
        .start(&local.execution_services())
        .unwrap()
        .wait()
        .await
        .unwrap();
    assert!(result.usable());
    let RunReport::Modeling(steps) = result.report().unwrap() else {
        panic!("original model expected")
    };
    let Outcome::Native(native) = &steps[0].outcome else {
        panic!("native correction expected")
    };
    let receipt = native.start_receipt.as_ref().unwrap();
    assert!(receipt.submitted);
    assert_eq!(
        receipt
            .seed
            .as_ref()
            .unwrap()
            .compatibility
            .layout
            .to_string(),
        stored.layout
    );
    let plain = package
        .prepare_declared(&local.preparation_services(), &analysis, &cancel)
        .await
        .unwrap()
        .start(&local.execution_services())
        .unwrap()
        .wait()
        .await
        .unwrap();
    assert_ne!(
        result.completion().unwrap().lineage[0].request_identity,
        plain.completion().unwrap().lineage[0].request_identity
    );
    let RunRequest::Modeling(requests) = result.request() else {
        panic!("modeling request expected")
    };
    assert!(requests[0].starts.values().any(
        |source| matches!(source,StartSource::Stored{solution:actual} if *actual==solution.as_id())
    ));
}

async fn long_scip_study(
    shared: &SharedRuntime,
    local: &Runtime,
    time_limit: Duration,
    attempt_limit: u32,
) -> StudyHandle {
    let mut profile = SolveSettings {
        intent: SolveIntent::Optimize,
        backend: Some(Backend::Scip),
        ..Default::default()
    };
    profile.controls.time_limit = time_limit;
    submit_single(
        shared,
        local,
        physical_with_indicator(),
        discrete_sources(&market_split()),
        profile,
        attempt_limit,
    )
    .await
    .2
}
async fn wait_study_assignment(
    local: &Runtime,
    handle: &StudyHandle,
    worker: &NativeStudyWorker,
) -> (String, String) {
    let key = pse_operations::canonical_studies::point_key(
        &handle.study_id().to_string(),
        OccurrenceKey(0),
    );
    until(
        // Finite candidate-preparation watchdog. Original application readiness
        // is already established; assignment is not proof of native solver entry.
        Duration::from_secs(600),
        "study assignment",
        || {
            let key = &key;
            async move {
                let point = local
                    .canonical_store()
                    .unwrap()
                    .canonical_study_point(key)
                    .await
                    .unwrap()
                    .unwrap();
                let started = pse_operations::canonical_studies::point_facts(&point)
                    .unwrap()
                    .native_started;
                if !started {
                    assert!(
                        worker.terminal().unwrap().is_none(),
                        "original native application ended before assignment; diagnostics={}",
                        worker.directory().display()
                    );
                }
                started
            }
        },
    )
    .await;
    let point = local
        .canonical_store()
        .unwrap()
        .canonical_study_point(&key)
        .await
        .unwrap()
        .unwrap();
    let attempt = point.attempt.unwrap();
    assert!(
        !local
            .canonical_store()
            .unwrap()
            .canonical_attempt(&attempt)
            .await
            .unwrap()
            .unwrap()
            .terminal()
    );
    // Preserve the fixed post-fence observation window before testing process
    // death/cancellation; the fence records assignment, not native solver entry.
    tokio::time::sleep(Duration::from_secs(5)).await;
    (point.run, attempt)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn killed_worker_preserves_native_observations_and_refuses_unknown_effect_retry() {
    let mut managed = ManagedWorkerCase::new(1);
    let body = AssertUnwindSafe(async {
        managed.launch().await;
        let (shared, local) = managed.runtime(true);
        let policy = LeasePolicy::new(
            Duration::from_secs(2),
            Duration::from_millis(200),
            64,
            Duration::from_millis(200),
        )
        .unwrap();
        let (local, operations) = bind_operations(&shared, local, "killed-enqueuer", policy);
        let worker = managed.spawn().await;
        let local = managed.bind_worker(local, worker);
        let handle = long_scip_study(&shared, &local, Duration::from_secs(30), 2).await;
        let (run, first) = wait_study_assignment(&local, &handle, &managed.workers[worker]).await;
        // Abruptly kill through the original retained owner. A manager signal reply
        // is insufficient: this capability also proves the population physically drained.
        let killed = managed.workers[worker]
            .kill_and_drain_for_qualification(Instant::now() + Duration::from_secs(45))
            .await
            .unwrap();
        assert_eq!(
            killed.completed_status(),
            Some(128 + 9),
            "original SIGKILL exit, separate from proof of an empty population"
        );
        assert!(
            managed.workers[worker].terminal().unwrap().is_none(),
            "SIGKILL cannot fabricate an application native-join receipt"
        );
        assert!(
            !operations
                .store()
                .canonical_attempt(&first)
                .await
                .unwrap()
                .unwrap()
                .terminal()
        );
        // NativeStudyWorker currently uses the production thirty-second lease.
        // Expiry is authority recovery, independent of the already-proven physical drain.
        tokio::time::sleep(Duration::from_secs(31)).await;
        let replacement_worker = managed.spawn().await;
        let local = managed.bind_worker(local, replacement_worker);
        let handle = local.study(handle.study_id());
        drain_worker(&handle, replacement_worker, &mut managed).await;
        let lost = operations
            .store()
            .canonical_attempt(&first)
            .await
            .unwrap()
            .unwrap();
        assert!(lost.terminal());
        assert_eq!(
            lost.outcome.as_ref().map(|value| value.as_str()),
            Some("failed")
        );
        let record = operations.record(&run, &first).await.unwrap();
        assert!(record.completion.as_ref().unwrap().completion.is_none());
        assert!(
            record.solutions.is_empty(),
            "unqualified staged incumbent cannot become a reusable scientific seed"
        );
        let mut events = local
            .progress(&run, &first, pse_columnar::CancellationToken::new())
            .await
            .unwrap();
        let mut saw_native = false;
        while let Some(page) = events.next_page().await.unwrap() {
            saw_native |= page
                .iter()
                .any(|event| event.phase == "scip.incumbent" && event.incumbent.is_some());
        }
        assert!(
            saw_native,
            "frozen failed attempt preserves actual native observations"
        );
        let status = handle.status().await.unwrap();
        assert_eq!(status.state, StudyState::Concluded);
        let point = &status.points[0];
        // The authored attempt allowance is necessary but insufficient for retry:
        // shared may_retry also requires a transient failure and absent/idempotent
        // effects. A physically killed foreign solver leaves effects unknown.
        assert_eq!(point.attempt.as_deref(), Some(first.as_str()));
        let outcome = point.outcome.as_ref().unwrap();
        assert_eq!(outcome.attempts.len(), 1);
        assert_eq!(outcome.effect, pse_model::study::EffectState::Unknown);
        assert!(!outcome.scientific.usable && !outcome.scientific.seed_permission);
        assert_eq!(
            operations
                .store()
                .canonical_attempt_page(&run, None, lost.generation)
                .await
                .unwrap()
                .len(),
            1
        );
        let recovered_run = operations
            .store()
            .canonical_run(&run)
            .await
            .unwrap()
            .unwrap();
        assert!(recovered_run.current_generation > lost.generation);
        assert!(
            operations
                .store()
                .canonical_attempt_page(
                    &run,
                    Some(lost.generation),
                    recovered_run.current_generation
                )
                .await
                .unwrap()
                .is_empty(),
            "new authority grants no automatic reexecution"
        );
        drop(events);
        drop(record);
        drop(handle);
        // The runtime may retain physical admissions after the caller has departed.
        // Retire that optional ownership before the store's authentic protection drain.
        local.clear_program_cache();
    })
    .catch_unwind()
    .await;
    managed.finish(body).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cross_process_cancel_stops_scip() {
    const REQUEST: &str = "PSE_STO_CANCEL_REQUEST";
    const RESPONSE: &str = "PSE_STO_CANCEL_RESPONSE";
    const STATE: &str = "PSE_STO_CANCEL_STATE";
    if let Some(request) = std::env::var_os(REQUEST) {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let request: pse_operations::canonical_control::ControlCancelRequest =
            serde_json::from_slice(&std::fs::read(request).unwrap()).unwrap();
        let state = PathBuf::from(std::env::var_os(STATE).unwrap());
        let options = pse_operations::canonical::CanonicalOptions::from_state(&state).unwrap();
        let ready = pse_operations::canonical::CanonicalStore::connect_ready(&options)
            .await
            .unwrap();
        let store = ready.into_store();
        let receipt = store.cancel_study_command(&request).await;
        // Attempt connection drain even if the actual native command fails.
        let disconnected = store.disconnect().await;
        let receipt = receipt.unwrap();
        disconnected.unwrap();
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(std::env::var_os(RESPONSE).unwrap())
            .unwrap();
        file.write_all(&serde_json::to_vec(&receipt).unwrap())
            .unwrap();
        file.sync_all().unwrap();
        return;
    }

    let mut managed = ManagedWorkerCase::new(1);
    let body = AssertUnwindSafe(async {
        managed.launch().await;
        let (shared, local) = managed.runtime(true);
        let (local, operations) =
            bind_operations(&shared, local, "cancel-enqueuer", LeasePolicy::default());
        let worker = managed.spawn().await;
        let local = managed.bind_worker(local, worker);
        let handle = long_scip_study(&shared, &local, Duration::from_secs(120), 1).await;
        let store = local.canonical_store().unwrap();
        let study = handle.study_id().to_string();
        let generation = store
            .canonical_study(&study)
            .await
            .unwrap()
            .unwrap()
            .generation;
        let request = pse_operations::canonical_control::ControlCancelRequest::issue(
            store.issuing_store().unwrap().to_owned(),
            study.clone(),
        );
        let state = managed.service.state().to_path_buf();
        let request_path = state.join("cancel-request.json");
        let response_path = state.join("cancel-response.json");
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&request_path)
                .unwrap();
            file.write_all(&serde_json::to_vec(&request).unwrap())
                .unwrap();
            file.sync_all().unwrap();
        }
        let (run, attempt) = wait_study_assignment(&local, &handle, &managed.workers[worker]).await;
        let requested = tokio::time::Instant::now();
        let (stdout, stderr) = managed.child_logs("cancel-control");
        managed.children.push(
            tokio::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "cross_process_cancel_stops_scip", "--nocapture"])
                .env(REQUEST, &request_path)
                .env(RESPONSE, &response_path)
                .env(STATE, &state)
                .stdout(stdout)
                .stderr(stderr)
                .spawn()
                .unwrap(),
        );
        let reply = tokio::time::timeout(
            Duration::from_secs(30),
            managed.children.last_mut().unwrap().wait(),
        )
        .await
        .expect("native control process bounded; original child retained for explicit reap")
        .unwrap();
        assert!(
            reply.success(),
            "{}",
            String::from_utf8_lossy(&std::fs::read(state.join("cancel-control.stderr")).unwrap())
        );
        let receipt: pse_operations::canonical_control::ControlCommandReceipt =
            serde_json::from_slice(&std::fs::read(&response_path).unwrap()).unwrap();
        assert_eq!(receipt.issuing_store, request.issuing_store);
        assert_eq!(receipt.operation_id, request.operation_id);
        assert_eq!(receipt.study, study);
        assert_eq!(receipt.generation, Some((generation + 1).to_string()));
        assert_eq!(
            receipt.status,
            pse_operations::canonical_control::ControlCommandStatus::Accepted
        );
        drain_worker(&handle, worker, &mut managed).await;
        assert!(requested.elapsed() < Duration::from_secs(30));
        let status = handle.status().await.unwrap();
        assert_eq!(status.state, StudyState::Concluded);
        assert!(status.cancelled);
        assert_eq!(status.points[0].state, StudyPointState::Cancelled);
        assert_eq!(
            store
                .canonical_study(&study)
                .await
                .unwrap()
                .unwrap()
                .generation,
            generation + 1
        );
        assert_eq!(
            operations
                .store()
                .canonical_attempt(&attempt)
                .await
                .unwrap()
                .unwrap()
                .outcome
                .as_ref()
                .map(|value| value.as_str()),
            Some("cancelled")
        );
        // Assignment alone is not native entry. The frozen original attempt must
        // contain an actual SCIP callback. This does not establish its ordering
        // relative to the native control cancellation request.
        let mut events = local
            .progress(&run, &attempt, pse_columnar::CancellationToken::new())
            .await
            .unwrap();
        let mut saw_native = false;
        while let Some(page) = events.next_page().await.unwrap() {
            saw_native |= page
                .iter()
                .any(|event| event.phase == "scip.incumbent" && event.incumbent.is_some());
        }
        assert!(
            saw_native,
            "cancelled attempt must retain an actual SCIP callback"
        );
        drop(events);
        drop(handle);
        local.clear_program_cache();
    })
    .catch_unwind()
    .await;
    managed.finish(body).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn study_parallel_workers_admit_one_summary_and_exact_point_attempts() {
    use pse_runtime::workflow::{BindingAssignment, BindingQuantity, BindingTarget};
    let mut managed = ManagedWorkerCase::new(2);
    let body = AssertUnwindSafe(async {
        managed.launch().await;
        const POINTS: usize = 8;
        assert_eq!(managed.allocation.native_workers, 2);
        assert!(managed.allocation.native_worker_memory_bytes > 64_usize << 30);
        assert_eq!(managed.allocation.execution.unwrap().case_lanes, 16);
        let (shared, local) = managed.runtime(false);
        let (local, operations) =
            bind_operations(&shared, local, "parallel-enqueuer", LeasePolicy::default());
        let left = managed.spawn().await;
        let right = managed.spawn().await;
        let local = managed.bind_worker(local, left);
        let (physical, modeling) = sources_of(PARAMETRIC);
        let (package, context) = package(&shared, &local, &physical, &modeling).await;
        let case = package
            .declarations(&local.preparation_services())
            .await
            .unwrap()
            .iter()
            .find(|d| d.name == "Root")
            .unwrap()
            .declaration_id;
        let scalar = context
            .quantities()
            .quantity_types()
            .find(|quantity| quantity.name.as_deref() == Some("Scalar"))
            .unwrap();
        let points = (0..POINTS)
            .map(|index| {
                let mut point = occurrence(case, index as u32, settings(), 1);
                point.overlay.assignments.push(BindingAssignment {
                    target: BindingTarget::Path("a".into()),
                    value: BindingQuantity {
                        magnitude: pse_model::scalars::FiniteBound::try_new(
                            ((index + 2) * (index + 2)) as f64,
                        )
                        .unwrap(),
                        quantity: scalar.id,
                        unit: scalar.canonical_unit,
                    },
                });
                point
            })
            .collect();
        let handle = local
            .start_study(
                StudyPlan {
                    output: pse_runtime::workflow::StudyOutputDemand::Full {},
                    sources: PackageSources {
                        physical,
                        modeling: vec![modeling],
                    },
                    inputs: pse_runtime::workflow::StudyInputs::Explicit(points),
                },
                &CancelSource::new(),
            )
            .await
            .unwrap();
        until(
            Duration::from_secs(600),
            "competing workers conclude one summary",
            || async { handle.status().await.unwrap().state == StudyState::Concluded },
        )
        .await;
        // Both original owners are drained independently, even if the first fails.
        let (left_workers, right_workers) = managed.workers.split_at_mut(right);
        let (left_drain, right_drain) = tokio::join!(
            left_workers[left].shutdown(Instant::now() + Duration::from_secs(45)),
            right_workers[0].shutdown(Instant::now() + Duration::from_secs(45)),
        );
        left_drain.unwrap();
        right_drain.unwrap();
        managed.stop_worker(left).await;
        managed.stop_worker(right).await;
        let status = handle.status().await.unwrap();
        assert_eq!(status.state, StudyState::Concluded);
        assert_eq!(status.points.len(), POINTS);
        let mut workers = std::collections::BTreeSet::new();
        for (index, point) in status.points.iter().enumerate() {
            assert_eq!(point.outcome.as_ref().unwrap().attempts.len(), 1);
            assert!(point.outcome.as_ref().unwrap().scientific.usable);
            let attempt = point.attempt.as_deref().unwrap();
            let header = operations
                .store()
                .canonical_attempt(attempt)
                .await
                .unwrap()
                .unwrap();
            assert!(header.terminal());
            workers.insert(header.worker);
            assert_value(&local, &point.run, attempt, (index + 2) as f64).await;
        }
        assert_eq!(workers.into_iter().collect::<Vec<_>>(), {
            let mut identities = vec![
                managed.worker_identity(left),
                managed.worker_identity(right),
            ];
            identities.sort();
            identities
        });
        let summary = handle.result().await.unwrap().unwrap();
        let parent = operations
            .store()
            .canonical_attempt(&summary.attempt)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            parent.outcome.as_ref().map(|value| value.as_str()),
            Some("succeeded")
        );
        assert_eq!(
            handle.result().await.unwrap().unwrap().attempt,
            summary.attempt
        );
        assert_eq!(summary.points.len(), POINTS);
        drop(handle);
        drop(package);
        local.clear_program_cache();
    })
    .catch_unwind()
    .await;
    managed.finish(body).await;
}
