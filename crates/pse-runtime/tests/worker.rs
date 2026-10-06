// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Canonical worker child-process journeys: original authored science, retained seeds,
//! truthful killed-writer recovery, native cancellation and competing study claims.
#![recursion_limit = "256"]
#![allow(clippy::unwrap_used,clippy::expect_used,clippy::panic,reason="integration assertions and child-process protocol")]
use pse_backend_native::{presolve::PolicyKind,solve::{Backend,SolveIntent}};
use pse_model::study::{OccurrenceKey,StartPolicy};
use pse_runtime::{CancelSource,SharedRuntime,authoring_driver::document::{OwnedDocumentSet,load_package_documents_owned},math::{settings::SolveSettings,solves::Outcome},workflow::{Durability,LeasePolicy,ModelingPackage,Operations,PhysicalContext,RunReport,RunRequest,Runtime,StartSource,StoredStart,StudyHandle,StudyPlan,StudyPoint,StudyPointPolicy,CaseOperation,OperationRequest,PackageSources,PointOverlay,PreparationSettings,StudyState,StudyPointState}};
use std::{collections::BTreeMap,num::NonZeroUsize,path::Path,sync::Arc,time::Duration};
/// One journey owns this profile's finite worker slots at a time, including across
/// Nextest test processes. The two-worker study consumes both configured slots.
struct ManagedWorkerCase {
    _lock: std::fs::File,
    units: Vec<String>,
}
impl ManagedWorkerCase {
    fn acquire() -> Self {
        use std::os::unix::fs::OpenOptionsExt;
        let state = std::env::var_os("PSE_SURREAL_STATE")
            .expect("worker-test selects owned canonical state");
        pse_operations::canonical::CanonicalOptions::from_state(Path::new(&state)).unwrap();
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(Path::new(&state).join(".worker-integration.lock"))
            .unwrap();
        lock.lock().unwrap();
        let output = supervisor().arg("status").output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let status: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let units = status["worker_units"]
            .as_array()
            .unwrap()
            .iter()
            .map(|unit| unit.as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        let checked = supervisor_python("from scripts import surreal_server as s; import os,pathlib; p=pathlib.Path(os.environ['PSE_SURREAL_STATE']); s.workers_drained(p,s.config_for(p))").status().unwrap();
        assert!(
            checked.success(),
            "another process already owns this fixture profile's worker slots"
        );
        Self { _lock: lock, units }
    }
    fn kill_solver(&self) {
        // The scientific process is in slot zero in this serial single-worker case.
        // Killing only its launcher would deliberately leave the scope alive.
        let code = "from scripts import surreal_server as s; import sys; s.systemctl('kill','--kill-whom=all','--signal=SIGKILL',sys.argv[1])";
        assert!(
            supervisor_python(code)
                .arg(&self.units[0])
                .status()
                .unwrap()
                .success()
        );
    }
}
impl Drop for ManagedWorkerCase {
    fn drop(&mut self) {
        for unit in &self.units {
            let _ = supervisor_python("from scripts import surreal_server as s; import sys; s.systemctl('stop',sys.argv[1],check=False)").arg(unit).status();
        }
    }
}
fn python() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.venv/bin/python")
}
fn supervisor() -> std::process::Command {
    let mut command = std::process::Command::new(python());
    command.arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/surreal_server.py"));
    command
}
fn supervisor_python(code: &str) -> std::process::Command {
    let mut command = std::process::Command::new(python());
    command
        .args(["-c", code])
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    command
}
fn managed_worker(database: &str) -> std::process::Command {
    let mut command = supervisor();
    command
        .args(["worker", "--worker-command"])
        .arg(std::env::var_os("PSE_WORKER_BINARY").expect("worker-test supplies its built worker"))
        .args(["--canonical-database", database]);
    command
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

fn runtime() -> (Arc<SharedRuntime>, Runtime) {
    let n = |v| NonZeroUsize::new(v).unwrap();
    let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
        memory_limit_bytes: n(1 << 30),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: n(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(2),
            target_partitions: n(1),
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
            ..Default::default()
        },
        hashing_may_use_pool: false,
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
            registry,
            sessions,
            pse_runtime::workflow::CanonicalDeployment::new(
                pse_operations::testing::canonical_fixture_store().unwrap(),
                pse_runtime::workflow::OuterAttestation {
                    source: pse_ids::ContentHash::from_bytes([0; 32]),
                    build: pse_ids::ContentHash::from_bytes([1; 32]),
                },
                None,
            ),
        ),
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
    let validation = runtime
        .sessions()
        .validation_context(runtime.registry())
        .unwrap();
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
            .modeling_from_documents(&modeling, context.clone())
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

/// A runtime budgeted as the worker binary is with `--memory-mib 8192`, whose native jobs
/// admit SCIP's memory limit (the default foreign allowance).
fn scip_runtime() -> (Arc<SharedRuntime>, Runtime) {
    let n = |v| NonZeroUsize::new(v).unwrap();
    let memory: usize = 8 << 30;
    let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
        memory_limit_bytes: n(memory),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: n(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(2),
            target_partitions: n(1),
        },
        execution: Default::default(),
        cache: pse_runtime::CacheBudget::disabled(1024),
        math: pse_runtime::math::MathPolicy {
            workspace_bytes: memory / 8,
            worker_bytes: memory / 16,
            artifact_bytes: memory / 8,
            ..Default::default()
        },
        hashing_may_use_pool: false,
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
            registry,
            sessions,
            pse_runtime::workflow::CanonicalDeployment::new(
                pse_operations::testing::canonical_fixture_store().unwrap(),
                pse_runtime::workflow::OuterAttestation {
                    source: pse_ids::ContentHash::from_bytes([0; 32]),
                    build: pse_ids::ContentHash::from_bytes([1; 32]),
                },
                None,
            ),
        ),
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


fn bind_operations(shared:&SharedRuntime,local:Runtime,worker:&str,policy:LeasePolicy)->(Runtime,Operations){
    let operations=Operations::from_store(local.canonical_store().clone(),worker,policy,shared.pool());
    (local.with_durability(Durability::Durable(operations.clone())),operations)
}
fn occurrence(case:pse_model::generated::identities::DeclarationId,key:u32,settings:SolveSettings,attempt_limit:u32)->StudyPoint{
    StudyPoint{operation:OperationRequest::DeclaredCase(CaseOperation{case,route:pse_model::generated::enums::ModelingAnalysisRoute::Steady,settings}),preparation:PreparationSettings::default(),overlay:PointOverlay::default(),policy:StudyPointPolicy{key:OccurrenceKey(key),dependencies:vec![],start:StartPolicy::Fresh,attempt_limit}}
}
async fn submit_single(shared:&SharedRuntime,local:&Runtime,physical:BTreeMap<String,Vec<u8>>,modeling:BTreeMap<String,Vec<u8>>,settings:SolveSettings,attempt_limit:u32)->(ModelingPackage,pse_model::generated::identities::DeclarationId,StudyHandle){
    let(package,_)=package(shared,local,&physical,&modeling).await;
    let case=package.declarations().await.unwrap().iter().find(|d|d.name=="Root").unwrap().declaration_id;
    let handle=local.start_study(StudyPlan{sources:PackageSources{physical,modeling:vec![modeling]},points:vec![occurrence(case,0,settings,attempt_limit)]}).await.unwrap();
    (package,case,handle)
}
fn spawn_worker(name:&str,lease_seconds:u64,canonical_database:&str)->std::process::Child{
    managed_worker(canonical_database).args(["--name",name,"--until-idle","--lease-seconds",&lease_seconds.to_string(),"--heartbeat-ms","200","--threads","2"]).spawn().unwrap()
}
async fn drain_worker(name:&str,local:&Runtime){
    let database=local.canonical_store().database().to_owned();let name=name.to_owned();
    let status=tokio::task::spawn_blocking(move||spawn_worker(&name,30,&database).wait().unwrap()).await.unwrap();
    assert!(status.success(),"{status}");
}
async fn until<F:Future<Output=bool>>(limit:Duration,what:&str,mut probe:impl FnMut()->F){
    let deadline=tokio::time::Instant::now()+limit;
    while !probe().await{assert!(tokio::time::Instant::now()<deadline,"timed out: {what}");tokio::time::sleep(Duration::from_millis(100)).await;}
}
async fn assert_value(local:&Runtime,run:&str,attempt:&str,wanted:f64){
    let mut stream=local.results(run,attempt,"runtime.solve_variables",0,u64::MAX,pse_columnar::CancellationToken::new()).await.unwrap();
    let validation=local.sessions().validation_context(local.registry()).unwrap();let mut found=false;
    while let Some(batch)=stream.next_batch().await.unwrap(){let view=pse_relations::generated::runtime::solve_variables::View::try_from_batch_with_registry(local.registry(),&batch,&validation).unwrap();for row in view.rows().unwrap(){found|=row.value.is_some_and(|value|(value-wanted).abs()<1e-8);}}
    assert!(found,"original variable result {wanted} was absent");
}

#[tokio::test(flavor="multi_thread",worker_threads=2)]
async fn worker_runs_authored_case_end_to_end(){
    let _managed=ManagedWorkerCase::acquire();let(shared,local)=runtime();let(local,operations)=bind_operations(&shared,local,"enqueuer",LeasePolicy::default());
    let(physical,modeling)=sources();let(package,case,handle)=submit_single(&shared,&local,physical,modeling,settings(),1).await;
    drain_worker("worker-child",&local).await;
    let status=handle.status().await.unwrap();assert_eq!(status.state,StudyState::Concluded);let point=&status.points[0];assert!(point.outcome.as_ref().unwrap().scientific.usable);assert_eq!(point.outcome.as_ref().unwrap().attempts.len(),1);
    let attempt=point.attempt.as_deref().unwrap();let stored=operations.store().canonical_attempt(attempt).await.unwrap().unwrap();assert!(stored.terminal);assert_eq!(stored.outcome.as_deref(),Some("succeeded"));assert_eq!(stored.worker,"worker-child");
    assert_value(&local,&point.run,attempt,2.).await;
    let mut progress=local.progress(&point.run,attempt,pse_columnar::CancellationToken::new()).await.unwrap();let mut count=0;while let Some(page)=progress.next_page().await.unwrap(){count+=page.len();}assert!(count>0,"worker native progress must be retained");
    stored_seed_reused_across_processes(&operations,&package,case,&point.run,attempt).await;
}

#[tokio::test(flavor="multi_thread",worker_threads=2)]
async fn durable_worker_round_trips_a_package_with_a_data_document(){
    let _managed=ManagedWorkerCase::acquire();let(shared,local)=runtime();let(local,operations)=bind_operations(&shared,local,"data-enqueuer",LeasePolicy::default());
    let(physical,mut modeling)=sources_of(DATA_SQUARE);modeling.insert("data/target.parquet".into(),target_document());
    let(package,_,handle)=submit_single(&shared,&local,physical,modeling,settings(),1).await;
    assert!(operations.store().revision(&package.canonical_revision().key).await.unwrap().is_some());
    drain_worker("worker-data",&local).await;let status=handle.status().await.unwrap();assert_eq!(status.state,StudyState::Concluded);let point=&status.points[0];assert!(point.outcome.as_ref().unwrap().scientific.usable);
    assert_value(&local,&point.run,point.attempt.as_deref().unwrap(),3.).await;
}

async fn stored_seed_reused_across_processes(operations:&Operations,package:&ModelingPackage,case:pse_model::generated::identities::DeclarationId,run:&str,attempt:&str){
    let cancel=CancelSource::new();let record=operations.record(run,attempt).await.unwrap();let(step,solution)=record.solutions[0];assert_eq!(step,0);
    let stored=operations.store().result_seed(&solution.to_string()).await.unwrap().unwrap();assert_eq!(stored.attempt,attempt);assert_eq!(stored.run,run);
    let analysis=package.declared_execution(case,Default::default(),settings().profile().unwrap(),Default::default(),pse_modeling::Limits::default(),&cancel).await.unwrap();
    let prepared=package.prepare_declared(&analysis,&cancel).await.unwrap();let seeded=prepared.with_stored_start(operations,StoredStart::Solution(solution)).await.unwrap();
    assert!(seeded.starts.values().any(|source|matches!(source,StartSource::Stored{solution:actual} if *actual==solution.as_id())));
    let result=seeded.start().unwrap().wait().await.unwrap();assert!(result.usable());
    let RunReport::Modeling(steps)=result.report().unwrap() else{panic!("original model expected")};let Outcome::Native(native)=&steps[0].outcome else{panic!("native correction expected")};
    let receipt=native.start_receipt.as_ref().unwrap();assert!(receipt.submitted);assert_eq!(receipt.seed.as_ref().unwrap().compatibility.layout.to_string(),stored.layout);
    let plain=package.prepare_declared(&analysis,&cancel).await.unwrap().start().unwrap().wait().await.unwrap();
    assert_ne!(result.completion().unwrap().lineage[0].request_identity,plain.completion().unwrap().lineage[0].request_identity);
    let RunRequest::Modeling(requests)=result.request() else{panic!("modeling request expected")};assert!(requests[0].starts.values().any(|source|matches!(source,StartSource::Stored{solution:actual} if *actual==solution.as_id())));
}

async fn long_scip_study(shared:&SharedRuntime,local:&Runtime,time_limit:Duration,attempt_limit:u32)->StudyHandle{
    let mut profile=SolveSettings{intent:SolveIntent::Optimize,backend:Some(Backend::Scip),..Default::default()};profile.controls.time_limit=time_limit;
    submit_single(shared,local,physical_with_indicator(),discrete_sources(&market_split()),profile,attempt_limit).await.2
}
async fn wait_native_start(local:&Runtime,handle:&StudyHandle)->(String,String){
    let key=pse_operations::canonical_studies::point_key(&handle.study_id().to_string(),OccurrenceKey(0));
    until(Duration::from_secs(40),"claimed native SCIP dispatch",||async{let point=local.canonical_store().canonical_study_point(&key).await.unwrap().unwrap();pse_operations::canonical_studies::point_facts(&point).unwrap().native_started}).await;
    let point=local.canonical_store().canonical_study_point(&key).await.unwrap().unwrap();let attempt=point.attempt.unwrap();
    assert!(!local.canonical_store().canonical_attempt(&attempt).await.unwrap().unwrap().terminal);
    // Dispatch follows the persisted start fence. Give the authored long native search
    // time to report scientific observations before testing process death/cancellation.
    tokio::time::sleep(Duration::from_secs(5)).await;(point.run,attempt)
}

#[tokio::test(flavor="multi_thread",worker_threads=4)]
async fn killed_worker_freezes_truthful_observations_and_retries_only_by_authored_policy(){
    let managed=ManagedWorkerCase::acquire();let(shared,local)=scip_runtime();let policy=LeasePolicy{lease:Duration::from_secs(2),heartbeat:Duration::from_millis(200),..Default::default()};let(local,operations)=bind_operations(&shared,local,"killed-enqueuer",policy);
    let handle=long_scip_study(&shared,&local,Duration::from_secs(30),2).await;
    let mut worker=spawn_worker("worker-killed",2,local.canonical_store().database());let(run,first)=wait_native_start(&local,&handle).await;
    managed.kill_solver();assert!(!worker.wait().unwrap().success());assert!(!operations.store().canonical_attempt(&first).await.unwrap().unwrap().terminal);
    tokio::time::sleep(Duration::from_secs(3)).await;drain_worker("worker-replacement",&local).await;
    let lost=operations.store().canonical_attempt(&first).await.unwrap().unwrap();assert!(lost.terminal);assert_eq!(lost.outcome.as_deref(),Some("failed"));
    let record=operations.record(&run,&first).await.unwrap();assert!(record.completion.as_ref().unwrap().completion.is_none());assert!(record.solutions.is_empty(),"unqualified staged incumbent cannot become a reusable scientific seed");
    let mut events=local.progress(&run,&first,pse_columnar::CancellationToken::new()).await.unwrap();let mut saw_native=false;while let Some(page)=events.next_page().await.unwrap(){saw_native|=page.iter().any(|event|event.phase=="scip.incumbent"&&event.incumbent.is_some());}assert!(saw_native,"frozen failed attempt preserves actual native observations");
    let status=handle.status().await.unwrap();assert_eq!(status.state,StudyState::Concluded);let point=&status.points[0];assert_ne!(point.attempt.as_deref(),Some(first.as_str()));assert_eq!(point.outcome.as_ref().unwrap().attempts.len(),2);let replacement=operations.store().canonical_attempt(point.attempt.as_deref().unwrap()).await.unwrap().unwrap();assert!(replacement.terminal);assert!(replacement.generation>lost.generation);
}

#[tokio::test(flavor="multi_thread",worker_threads=4)]
async fn cross_process_cancel_stops_scip(){
    let _managed=ManagedWorkerCase::acquire();let(shared,local)=scip_runtime();let(local,operations)=bind_operations(&shared,local,"cancel-enqueuer",LeasePolicy::default());
    let handle=long_scip_study(&shared,&local,Duration::from_secs(120),1).await;let mut worker=spawn_worker("worker-cancelled",30,local.canonical_store().database());let(_,attempt)=wait_native_start(&local,&handle).await;
    let requested=tokio::time::Instant::now();handle.cancel().await.unwrap();let status=tokio::task::spawn_blocking(move||worker.wait().unwrap()).await.unwrap();assert!(status.success(),"{status}");assert!(requested.elapsed()<Duration::from_secs(30));
    let status=handle.status().await.unwrap();assert_eq!(status.state,StudyState::Concluded);assert!(status.cancelled);assert_eq!(status.points[0].state,StudyPointState::Cancelled);assert_eq!(operations.store().canonical_attempt(&attempt).await.unwrap().unwrap().outcome.as_deref(),Some("cancelled"));
}

#[tokio::test(flavor="multi_thread",worker_threads=4)]
async fn study_parallel_workers_admit_one_summary_and_exact_point_attempts(){
    use pse_runtime::workflow::{BindingAssignment,BindingQuantity,BindingTarget};
    let _managed=ManagedWorkerCase::acquire();const POINTS:usize=8;let(shared,local)=runtime();let(local,operations)=bind_operations(&shared,local,"parallel-enqueuer",LeasePolicy::default());let(physical,modeling)=sources_of(PARAMETRIC);let(package,context)=package(&shared,&local,&physical,&modeling).await;
    let case=package.declarations().await.unwrap().iter().find(|d|d.name=="Root").unwrap().declaration_id;let scalar=context.quantities().quantity_types().find(|quantity|quantity.name.as_deref()==Some("Scalar")).unwrap();
    let points=(0..POINTS).map(|index|{let mut point=occurrence(case,index as u32,settings(),1);point.overlay.assignments.push(BindingAssignment{target:BindingTarget::Path("a".into()),value:BindingQuantity{magnitude:pse_model::scalars::FiniteBound::try_new(((index+2)*(index+2))as f64).unwrap(),quantity:scalar.id.as_id(),unit:scalar.canonical_unit.as_id()}});point}).collect();
    let handle=local.start_study(StudyPlan{sources:PackageSources{physical,modeling:vec![modeling]},points}).await.unwrap();let mut children=["worker-left","worker-right"].map(|name|spawn_worker(name,30,local.canonical_store().database()));
    let statuses=tokio::task::spawn_blocking(move||children.each_mut().map(|worker|worker.wait().unwrap())).await.unwrap();for status in statuses{assert!(status.success(),"{status}");}
    let status=handle.status().await.unwrap();assert_eq!(status.state,StudyState::Concluded);assert_eq!(status.points.len(),POINTS);let mut workers=std::collections::BTreeSet::new();
    for(index,point)in status.points.iter().enumerate(){assert_eq!(point.outcome.as_ref().unwrap().attempts.len(),1);assert!(point.outcome.as_ref().unwrap().scientific.usable);let attempt=point.attempt.as_deref().unwrap();let header=operations.store().canonical_attempt(attempt).await.unwrap().unwrap();assert!(header.terminal);workers.insert(header.worker);assert_value(&local,&point.run,attempt,(index+2)as f64).await;}
    assert_eq!(workers.into_iter().collect::<Vec<_>>(),["worker-left","worker-right"]);
    let summary=handle.result().await.unwrap().unwrap();let parent=operations.store().canonical_attempt(&summary.attempt).await.unwrap().unwrap();assert_eq!(parent.outcome.as_deref(),Some("succeeded"));assert_eq!(handle.result().await.unwrap().unwrap().attempt,summary.attempt);assert_eq!(summary.points.len(),POINTS);
}
