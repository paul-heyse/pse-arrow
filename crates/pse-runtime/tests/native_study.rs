// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Real Rust worker science over an explicitly owned private native service.
// Match the existing runtime and local-operation downstream target: the public
// async ownership graph needs this finite compiler proof depth.
#![recursion_limit = "512"]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stderr,
    reason = "private service and independent scientific assertions fail directly")]
use futures_util::{FutureExt, future::BoxFuture};
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_backend_native::{presolve::PolicyKind, solve::{Backend, SolveIntent}};
use pse_ids::{ContentHash, Frame, FramedHasher};
use pse_model::study::{Dependency, OccurrenceKey, SeedEdge, SeedRole, StartPolicy, StartProvenance};
use pse_operations::{canonical::CanonicalStore, native_service::{NativeCanonicalService, NativeServiceSpec}};
use pse_relations::generated::runtime::{solve_runs, solve_variables};
use pse_runtime::{CancelSource, SharedRuntime, math::settings::SolveSettings,
    workflow::{CanonicalDeployment, CaseOperation, DurabilitySelection, LeasePolicy, ModelingPackage,
        OperationRequest, OuterAttestation, PointOverlay, PreparationSettings, Runtime, StudyHandle,
        OperationSource, RuntimeOperationRequest, PreparedOperation, PreparedStudyOperation,
        StudyInputs, StudyOutputDemand, StudyPoint, StudyPointPolicy, WorkerSettings, LocalStudy,
        StudyDefinition, ExecutionServices, WorkflowError}};
use std::{collections::BTreeMap, num::NonZeroUsize, path::{Path, PathBuf}, panic::AssertUnwindSafe,
    sync::{Arc, atomic::{AtomicUsize, Ordering}}, time::{Duration, Instant}};

// Same independent nonlinear equation as the public local-operation fixture.
// Its known positive root is 2, regardless of the value retained by the runtime.
const SQUARE: &str = "package local { test square fixture { dof 0; } { param p:Scalar=4; var x:Scalar; eq root:x*x==p; annotation start x(1); } }";

fn service() -> NativeCanonicalService {
    let environment=std::env::vars_os().collect();
    let helper=PathBuf::from(std::env::var_os("PSE_NATIVE_FIXTURE_SERVICE_HELPER").expect("explicit already-built native-service helper"));
    assert!(helper.is_absolute() && helper.is_file(), "build -p pse-operations --features native-service --bin pse-canonical-service first");
    let module_source = if let Some(record) = std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_RECEIPT") {
        pse_operations::native_service::NativeModuleSource::Prepared(pse_host::study_module::select(&PathBuf::from(record),pse_host::study_module::Variant::Production).unwrap())
    } else {
        pse_operations::native_service::NativeModuleSource::RecordedState(PathBuf::from(std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_STATE").expect("explicit private module receipt selector")))
    };
    let service=NativeCanonicalService::new(NativeServiceSpec {
        server:pse_host::server::select(&environment).unwrap(), helper, module_source,
        state_parent:std::env::temp_dir(),
    }).unwrap();
    eprintln!("scientific native fixture state={} original_service={}",service.state().display(),service.intent().unit());
    service
}
fn texts(root:&Path) -> BTreeMap<String,Vec<u8>> {
    fn visit(root:&Path,at:&Path,out:&mut BTreeMap<String,Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path=entry.unwrap().path();
            if path.is_dir() {visit(root,&path,out);} else {
                out.insert(path.strip_prefix(root).unwrap().to_str().unwrap().replace('\\',"/"),std::fs::read(path).unwrap());
            }
        }
    }
    let mut out=BTreeMap::new(); visit(root,root,&mut out); out
}
struct Science {
    shared:Arc<SharedRuntime>,
    runtimes:Vec<Runtime>,
    build:ContentHash,
    _spill:tempfile::TempDir,
}
impl Science {
    fn new(stores:&[CanonicalStore;3]) -> Self {
        let spill=tempfile::tempdir().unwrap();
        let n=|v|NonZeroUsize::new(v).unwrap();
        let shared=SharedRuntime::build(pse_runtime::ResourceBudget {
            memory_limit_bytes:n(4<<30),spill_dir:spill.path().into(),max_temp_dir_bytes:1<<30,top_consumers:n(16),
            threads:pse_engine::ThreadBudget {pool_threads:n(16),target_partitions:n(16)},
            execution:Default::default(), cache:pse_runtime::CacheBudget::disabled(1024),
            math:pse_runtime::math::MathPolicy {worker_bytes:1<<30,workspace_bytes:1<<30,foreign_bytes:16<<20,
                jobs:std::thread::available_parallelism().unwrap().get(),..Default::default()},
        }).unwrap();
        let mut hasher=FramedHasher::new(Frame::CanonicalPayloadV1);
        let mut executable=std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
        hasher.part_reader(executable.metadata().unwrap().len(),&mut executable,&||Ok(())).unwrap();
        let mut science=Self {shared,runtimes:vec![],build:hasher.finish_hash(),_spill:spill};
        for (store,name) in stores.iter().zip(["producer","worker-left","worker-right"]) {science.attach(store,name);}
        science
    }
    fn attach(&mut self,store:&CanonicalStore,worker:&str) -> usize {
        let factory=Arc::new(self.shared.session_factory(pse_engine::session::native_engine_profile()).unwrap());
        let runtime=Runtime::from_shared(self.shared.clone(),factory.bind_model(pse_schema::shared_registry().unwrap()).unwrap(),
            CanonicalDeployment::new(store.clone(),OuterAttestation {source:None,build:self.build},None)).unwrap()
            .with_durability(DurabilitySelection::Durable {worker:worker.into(),policy:LeasePolicy::new(Duration::from_secs(30),Duration::from_millis(100),64,Duration::from_millis(20)).unwrap()}).unwrap();
        let index=self.runtimes.len(); self.runtimes.push(runtime); index
    }
    fn close(&self) -> BoxFuture<'_,Result<(),pse_runtime::math::MathRuntimeError>> {
        Box::pin(async move {
        for runtime in &self.runtimes {runtime.close();runtime.clear_program_cache();}
        self.shared.math().shutdown_native_lanes().await
        })
    }
    fn package(&self) -> BoxFuture<'_,ModelingPackage> {
        Box::pin(async move {
        let runtime=&self.runtimes[0];
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
        let physical=texts(&root.join("physical-primitives"));
        let manifest=std::fs::read_to_string(root.join("minimal_explicit/package.toml")).unwrap()
            .replace("id_policy = \"explicit\"","id_policy = \"named\"")
            .replace("dependencies = []","dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
        let modeling=BTreeMap::from([("package.toml".into(),manifest.into_bytes()),("models/root.pse".into(),SQUARE.as_bytes().to_vec())]);
        let pool=self.shared.pool(); let token=pse_columnar::CancellationToken::new();
        let validation=runtime.validation_context();
        let load=|texts:&BTreeMap<String,Vec<u8>>| OwnedDocumentSet::try_from_bundles(vec![load_package_documents_owned(
            texts,runtime.registry(),Default::default(),&pool,&token,&validation).unwrap()],&pool,&token).unwrap();
        let physical=runtime.physical_from_documents(&load(&physical),&token).await.unwrap();
        runtime.modeling_from_documents(&load(&modeling),physical,&CancelSource::new()).await.unwrap()
        })
    }
    fn submit(&self,count:u32) -> BoxFuture<'_,(StudyHandle,String)> {
        self.submit_with_seed_policy(count,Default::default())
    }
    fn submit_with_seed_policy(&self,count:u32,unavailable:pse_model::study::UnavailableSeedPolicy) -> BoxFuture<'_,(StudyHandle,String)> {
        Box::pin(async move {
        let runtime=&self.runtimes[0]; let cancel=CancelSource::new();
        let package=self.package().await;
        let case=package.declarations(&runtime.preparation_services()).await.unwrap().iter().find(|row|row.name=="square").unwrap().declaration_id;
        let inspect=package.prepare_operation(&runtime.preparation_services(),&OperationSource::of(&package),&PreparationSettings::default(),
            RuntimeOperationRequest::DeclaredCase {case,settings:Box::new(SolveSettings {intent:SolveIntent::Root,backend:Some(Backend::Kinsol),presolve:PolicyKind::Off,..Default::default()})},None,&cancel).await.unwrap();
        let PreparedOperation::Study(PreparedStudyOperation::DeclaredCase(inspect))=inspect else {panic!("original declared square case");};
        let solved=inspect.model.model.solved();
        // This primitive fixture has no characteristic engineering scale.
        // Request tight physical budgets on its actual admitted coordinates.
        let requirements=inspect.solve.numerics().targets.iter().filter(|target|matches!(target.kind,
            pse_model::generated::enums::NumericalTarget::Variable|pse_model::generated::enums::NumericalTarget::Row))
            .map(|target|pse_model::numerics::NumericalRequirement {
                requirement_id:pse_ids::named_id(case.as_id(),&format!("native-study-accuracy:{:?}:{}",target.kind,target.id)),
                model_id:Some(solved.model()),case_id:solved.case(),instance_id:Some(solved.instance()),fit_id:None,
                target_id:target.id,target_kind:target.kind,nominal:None,scaling_factor:None,
                absolute_tolerance:Some(1e-10),relative_tolerance:Some(0.),shared_engineering_allowance:None,
                engineering_rule_id:None,unit_id:Some(target.unit),coordinates:pse_model::generated::enums::NumericalCoordinates::Physical,
                priority:0,required:true,provenance:"private native square variable/equation verification".into(),
            }).collect::<Vec<_>>();
        assert!(requirements.iter().any(|row|row.target_kind==pse_model::generated::enums::NumericalTarget::Variable));
        assert!(requirements.iter().any(|row|row.target_kind==pse_model::generated::enums::NumericalTarget::Row));
        drop(inspect);
        let points=(0..count).map(|key|StudyPoint {
            operation:OperationRequest::DeclaredCase(CaseOperation {case,route:pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                settings:SolveSettings {intent:SolveIntent::Root,backend:Some(Backend::Kinsol),presolve:PolicyKind::Off,
                    numerics:pse_model::numerics::NumericalPolicy {
                        requirements:requirements.clone(),
                        engineering_relative_fraction:1e-10,supplier_action_accuracy:1e-10,linear_backward_error:1e-10,
                        kkt:pse_model::numerics::KktTolerances {stationarity:1e-10,complementarity:1e-10},
                        ..Default::default()
                    },..Default::default()}}),
            preparation:PreparationSettings::default(),overlay:PointOverlay::default(),
            policy:StudyPointPolicy {key:OccurrenceKey(key),dependencies:if key==0 {vec![]} else {vec![Dependency::Ordering(OccurrenceKey(0))]},
                start:if key==0 {StartPolicy::Fresh} else {StartPolicy::Continuation(SeedEdge {predecessor:OccurrenceKey(0),role:SeedRole::PrimalSolution,permission:Default::default(),unavailable})},attempt_limit:1},
        }).collect();
        let local=admit(&package,&runtime.execution_services(),&StudyInputs::Explicit(points),&cancel).await.unwrap();
        let definition=record(runtime,&local,&cancel).await.unwrap();
        let revision=definition.modeling_revision.clone();
        let handle=start(runtime,definition,&cancel).await.unwrap();
        (handle,revision)
        })
    }
}
// Each original ownership stage is awaited behind one explicit Send future;
// the private host callback does not inherit every nested implementation type.
fn admit<'a>(package:&'a ModelingPackage,services:&'a ExecutionServices,inputs:&'a StudyInputs,cancel:&'a CancelSource) -> BoxFuture<'a,Result<LocalStudy,WorkflowError>> {
    Box::pin(package.admit_study(services,inputs,&StudyOutputDemand::Full {},cancel))
}
fn record<'a>(runtime:&'a Runtime,local:&'a LocalStudy,cancel:&'a CancelSource) -> BoxFuture<'a,Result<StudyDefinition,WorkflowError>> {
    Box::pin(runtime.record_study(local,cancel))
}
fn start<'a>(runtime:&'a Runtime,definition:StudyDefinition,cancel:&'a CancelSource) -> BoxFuture<'a,Result<StudyHandle,WorkflowError>> {
    Box::pin(runtime.start_recorded_study(definition,cancel))
}
fn settings() -> WorkerSettings {WorkerSettings {poll:Duration::from_millis(20),recovery:Duration::from_secs(1),maximum_actions:None,maximum_in_flight:Some(1),until_idle:true}}
fn serve<'a>(runtime:&'a Runtime,stop:&'a CancelSource) -> BoxFuture<'a,Result<usize,WorkflowError>> {
    Box::pin(runtime.serve(settings(),stop))
}
fn value<'a>(runtime:&'a Runtime,run:&'a str,attempt:&'a str) -> BoxFuture<'a,()> {
    Box::pin(async move {
    let validation=runtime.validation_context();
    let mut reader=runtime.results(run,attempt,"runtime.solve_variables",0,u64::MAX,pse_columnar::CancellationToken::new()).await.unwrap();
    assert!(runtime.canonical_store().unwrap().disconnect().await.is_err(),"live protected scientific reader must refuse physical disconnect");
    let mut variables=0;
    while let Some(batch)=reader.next_batch().await.unwrap() {
        let view=solve_variables::View::try_from_batch_with_registry(runtime.registry(),&batch,&validation).unwrap();
        for row in view.rows().unwrap() {
            if !row.parameter && !row.fixed {
                let x=row.value.unwrap();
                assert!((x-2.).abs()<=1e-7,"independent positive root: {x}");
                assert!((x*x-4.).abs()<=1e-6,"original equation residual: {x}"); variables+=1;
            }
        }
    }
    assert_eq!(variables,1); reader.close(); drop(reader);
    let mut reader=runtime.results(run,attempt,"runtime.solve_runs",0,u64::MAX,pse_columnar::CancellationToken::new()).await.unwrap();
    let mut runs=0;
    while let Some(batch)=reader.next_batch().await.unwrap() {
        let view=solve_runs::View::try_from_batch_with_registry(runtime.registry(),&batch,&validation).unwrap();
        for row in view.rows().unwrap() {assert_eq!(row.backend,Some(pse_model::generated::enums::NativeBackend::Kinsol));runs+=1;}
    }
    assert_eq!(runs,1);reader.close();
    })
}
fn journey<'a>(science:&'a mut Science,store:&'a CanonicalStore,service:&'a mut NativeCanonicalService,cancelled:bool) -> BoxFuture<'a,()> {
    Box::pin(async move {
    let (handle,revision)=science.submit(if cancelled {1}else{2}).await;
    let study=handle.study_id();
    if !cancelled {
        let before=science.shared.math().preparations().views;
        let selected=store.select_study_workbench(&pse_operations::canonical_study_workbench::StudyWorkbenchFilter {
            study:Some(study.to_string()),..Default::default()
        },None,1).await.unwrap();
        assert_eq!(selected.headers.len(),1);
        assert_eq!(selected.headers[0].study,study.to_string());
        assert_eq!(selected.headers[0].point_count,2);
        assert!(!selected.has_more,"bounded exact selection has no continuation");
        assert_eq!(science.shared.math().preparations().views,before,"connection-only workbench selection constructs no native mathematics");
    }
    if cancelled {handle.cancel().await.unwrap();}
    let preparation_views=science.shared.math().preparations().views;
    let entered=Arc::new(AtomicUsize::new(0));
    let observer= {let entered=entered.clone(); Arc::new(move |_:&std::sync::atomic::AtomicBool| {entered.fetch_add(1,Ordering::SeqCst);})};
    // Spawned run supervisors inherit the qualified process observer; local
    // scopes additionally cover preparation in this submitting worker task.
    if !cancelled {pse_runtime::math::install_qualified_native_entry(observer.clone()).unwrap();}
    let left_stop=CancelSource::new();let right_stop=CancelSource::new();
    let (left,right)={let work=async {tokio::join!(
        pse_runtime::math::observed_native_entry(observer.clone(),serve(&science.runtimes[1],&left_stop)),
        pse_runtime::math::observed_native_entry(observer,serve(&science.runtimes[2],&right_stop)))};
    tokio::pin!(work);
    tokio::select! {
        result=&mut work => result,
        ()=tokio::time::sleep(Duration::from_secs(120)) => {
            left_stop.cancel();right_stop.cancel();
            let joined=work.await;
            panic!("private scientific workers exceeded their original finite journey allowance: {joined:?}");
        }
    }};
    assert!(left.is_ok() && right.is_ok(),"independent worker outcomes: left={left:?}, right={right:?}");
    let left=left.unwrap();let right=right.unwrap();
    assert!(left+right>0);
    let status=handle.status().await.unwrap();
    assert!(matches!(status.state,pse_runtime::workflow::StudyState::Concluded));
    if cancelled {
        assert_eq!(entered.load(Ordering::SeqCst),0);
        assert_eq!(science.shared.math().preparations().views,preparation_views,"canonical cancellation needs no new native preparation");
    } else {
        assert!(entered.load(Ordering::SeqCst)>=2,"both scientific occurrences must enter actual native sessions");
    }
    let summary=store.canonical_study_summary(&study.to_string()).await.unwrap().unwrap();
    assert_eq!(summary.revision,revision);
    let definitions=store.study_definition_page(&study.to_string(),None).await.unwrap();
    assert_eq!(definitions.len(),status.points.len());
    for point in &status.points {
        assert!(point.settled);
        let key=pse_operations::canonical_studies::point_key(&study.to_string(),OccurrenceKey(point.point_index));
        let canonical=store.canonical_study_point(&key).await.unwrap().unwrap();
        let facts=pse_operations::canonical_studies::point_facts(&canonical).unwrap();
        assert_eq!(facts.attempt_count,if cancelled {0}else{1});
        assert_eq!(facts.native_started,!cancelled);
        if cancelled {assert!(point.attempt.is_none());} else {
            assert!(point.outcome.as_ref().unwrap().scientific.usable);
            assert_eq!(point.outcome.as_ref().unwrap().attempts.len(),1);
            let run=store.canonical_run(&point.run).await.unwrap().unwrap();
            assert_eq!(run.revision,revision);
            let attempt=store.canonical_attempt(point.attempt.as_ref().unwrap()).await.unwrap().unwrap();
            assert_eq!(attempt.run,point.run);
            assert_eq!(attempt.phase,pse_model::generated::enums::CanonicalAttemptPhase::Terminal);
            if point.point_index==1 {assert!(matches!(point.outcome.as_ref().unwrap().start,Some(StartProvenance::Continuation {predecessor:OccurrenceKey(0),..})));}
        }
    }
    science.runtimes[0].close();
    assert!(handle.status().await.is_err(),"closed producer grants no new request");
    let replacement_ready=service.connect().await.unwrap();
    let replacement=science.attach(replacement_ready.store(),"replacement-reader");
    let reopened=science.runtimes[replacement].study(study);
    let result=reopened.result().await.unwrap().expect("protected terminal parent result after producer close");
    assert_eq!(result.points.len(),status.points.len());
    if !cancelled {
        for (_,run,attempt) in &result.points {value(&science.runtimes[replacement],run,attempt.as_ref().unwrap()).await;}
    }
    drop(result); drop(reopened);
    })
}
#[derive(Clone,Copy)]
enum NativeStudyCase { Completed, BeforeClaim, ActiveCancellation, CrashReplacement }

#[derive(Default)]
struct HeldNativeEntry {
    entered: AtomicUsize,
    stop_observed: std::sync::atomic::AtomicBool,
    exited: std::sync::atomic::AtomicBool,
    released: std::sync::Mutex<bool>,
    changed: std::sync::Condvar,
}
impl HeldNativeEntry {
    fn release(&self) {
        *self.released.lock().unwrap_or_else(std::sync::PoisonError::into_inner)=true;
        self.changed.notify_all();
    }
}
struct ReleaseNativeEntry {entry:Arc<HeldNativeEntry>,left:CancelSource,right:CancelSource}
impl Drop for ReleaseNativeEntry {
    fn drop(&mut self) {self.left.cancel();self.right.cancel();self.entry.release();}
}
fn active_cancellation<'a>(science:&'a mut Science,store:&'a CanonicalStore,service:&'a mut NativeCanonicalService)->BoxFuture<'a,()> {
    Box::pin(async move {
        let (handle,_)=science.submit(1).await;
        let study=handle.study_id();
        let point_key=pse_operations::canonical_studies::point_key(&study.to_string(),OccurrenceKey(0));
        let entry=Arc::new(HeldNativeEntry::default());
        let observer={let entry=entry.clone();Arc::new(move |stop:&std::sync::atomic::AtomicBool| {
            entry.entered.fetch_add(1,Ordering::SeqCst);
            let mut released=entry.released.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            while !*released {
                if stop.load(Ordering::SeqCst) {entry.stop_observed.store(true,Ordering::SeqCst);}
                released=entry.changed.wait_timeout(released,Duration::from_millis(10))
                    .unwrap_or_else(std::sync::PoisonError::into_inner).0;
            }
            entry.exited.store(true,Ordering::SeqCst);
        })};
        pse_runtime::math::install_qualified_native_entry(observer.clone()).unwrap();
        let original={
        let left_stop=CancelSource::new();let right_stop=CancelSource::new();
        let work=async {tokio::join!(
            pse_runtime::math::observed_native_entry(observer.clone(),serve(&science.runtimes[1],&left_stop)),
            pse_runtime::math::observed_native_entry(observer,serve(&science.runtimes[2],&right_stop)))};
        tokio::pin!(work);
        // Declared after the original work: unwind releases the native barrier
        // before dropping a future whose synchronous join may depend on it.
        let _release=ReleaseNativeEntry {entry:entry.clone(),left:left_stop.clone(),right:right_stop.clone()};
        tokio::select! {
            ()=async {while entry.entered.load(Ordering::SeqCst)==0 {tokio::time::sleep(Duration::from_millis(5)).await;}}=>{},
            result=&mut work=>panic!("workers finished before actual native admission: {result:?}"),
            ()=tokio::time::sleep(Duration::from_secs(120))=>{
                left_stop.cancel();right_stop.cancel();entry.release();let result=work.await;
                panic!("native entry never arrived; original workers drained: {result:?}");
            }
        }
        let original=store.canonical_study_point(&point_key).await.unwrap().unwrap();
        assert!(original.assigned && original.attempt.is_some());
        assert!(pse_operations::canonical_studies::point_facts(&original).unwrap().native_started);
        handle.cancel().await.unwrap();
        left_stop.cancel();right_stop.cancel();
        tokio::select! {
            ()=async {while !entry.stop_observed.load(Ordering::SeqCst) {tokio::time::sleep(Duration::from_millis(5)).await;}}=>{},
            result=&mut work=>panic!("stop intent released held native ownership early: {result:?}"),
            ()=tokio::time::sleep(Duration::from_secs(30))=>{
                entry.release();let result=work.await;
                panic!("native owner did not observe stop; original workers drained: {result:?}");
            }
        }
        assert!(!entry.exited.load(Ordering::SeqCst));
        tokio::select! {
            result=&mut work=>panic!("logical stop manufactured physical completion: {result:?}"),
            ()=tokio::time::sleep(Duration::from_millis(50))=>{}
        }
        entry.release();
        let (left,right)=work.await;
        let settled = |result: &Result<usize, WorkflowError>| matches!(result,
            Ok(_) | Err(WorkflowError::Math(pse_runtime::math::MathRuntimeError::Cancelled)));
        assert!(settled(&left) && settled(&right),"original worker settlement: {left:?}, {right:?}");
        assert!(entry.exited.load(Ordering::SeqCst));
        original
        };
        let fresh=service.connect().await.unwrap();
        let replacement=science.attach(fresh.store(),"after-native-stop");
        serve(&science.runtimes[replacement],&CancelSource::new()).await.unwrap();
        let status=handle.status().await.unwrap();
        assert!(status.cancelled && matches!(status.state,pse_runtime::workflow::StudyState::Concluded));
        let final_point=store.canonical_study_point(&point_key).await.unwrap().unwrap();
        assert_eq!(final_point.attempt,original.attempt);
        assert_eq!(pse_operations::canonical_studies::point_facts(&final_point).unwrap().attempt_count,1);
        assert_eq!(entry.entered.load(Ordering::SeqCst),1,"replacement must not rerun cancelled science");
        assert!(!status.points[0].outcome.as_ref().unwrap().scientific.usable);
    })
}

const CRASH_TEST: &str = "native_recorded_study_killed_process_is_drained_before_replacement_and_preserves_unknown_effect";
const CRASH_STATE: &str = "PSE_NATIVE_STUDY_CRASH_STATE";

// The same already-built test executable supplies an actual scientific worker.
// No CLI deployment profile, configured service or production worker is selected.
async fn crash_child(state:&Path) {
    let ready=CanonicalStore::connect_ready(&pse_operations::canonical::CanonicalOptions::from_state(state).unwrap()).await.unwrap();
    let store=ready.store();
    let science=Science::new(&[store.clone(),store.clone(),store.clone()]);
    let entry_path=state.join("crash-native-entry");
    let observer=Arc::new(move |stop:&std::sync::atomic::AtomicBool| {
        use std::{io::Write,os::unix::fs::OpenOptionsExt};
        let temporary=entry_path.with_extension("pending");
        let mut file=std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary).unwrap();
        write!(file,"{}",std::process::id()).unwrap();file.sync_all().unwrap();
        std::fs::rename(temporary,&entry_path).unwrap();
        // Hold a real admitted native session, with a finite orphan fallback.
        // SIGKILL must destroy this process before this callback can return.
        let entry=HeldNativeEntry::default();
        let deadline=Instant::now()+Duration::from_secs(90);
        let mut released=entry.released.lock().unwrap();
        while !stop.load(Ordering::SeqCst) && Instant::now()<deadline {
            released=entry.changed.wait_timeout(released,Duration::from_millis(10)).unwrap().0;
        }
        stop.store(true,Ordering::SeqCst);
    });
    pse_runtime::math::install_qualified_native_entry(observer.clone()).unwrap();
    let stop=CancelSource::new();
    let result=AssertUnwindSafe(pse_runtime::math::observed_native_entry(observer,serve(&science.runtimes[1],&stop))).catch_unwind().await;
    let closed=science.close().await;
    let disconnected=store.disconnect().await;
    if let Err(error)=&closed {eprintln!("crash child native drain failed: {error}");}
    if let Err(error)=&disconnected {eprintln!("crash child connection drain failed: {error}");}
    if let Err(panic)=result {std::panic::resume_unwind(panic);}
    closed.unwrap();
    disconnected.unwrap();
    panic!("crash worker returned instead of being killed: {:?}",result.unwrap());
}

fn crash_replacement<'a>(science:&'a mut Science,store:&'a CanonicalStore,service:&'a mut NativeCanonicalService)->BoxFuture<'a,()> {
    Box::pin(async move {
        use std::{os::unix::{fs::OpenOptionsExt,process::ExitStatusExt},process::{Command,Stdio}};
        let (handle,revision)=science.submit_with_seed_policy(2,pse_model::study::UnavailableSeedPolicy::FreshOnUnavailable).await;
        let study=handle.study_id();
        let point_key=pse_operations::canonical_studies::point_key(&study.to_string(),OccurrenceKey(0));
        let log_path=service.state().join("crash-worker.log");
        let log=std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&log_path).unwrap();
        let mut command=Command::new(std::env::current_exe().unwrap());
        command.args(["--exact",CRASH_TEST,"--nocapture"])
            .env(CRASH_STATE,service.state()).stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone().unwrap())).stderr(Stdio::from(log));
        let mut child=pse_diagnostics::SupervisedChild::spawn(&mut command,
            pse_diagnostics::DrainFailurePolicy::Refuse(Duration::from_secs(30)),None).unwrap();
        let result=AssertUnwindSafe(async {
            let original_identity=child.identity().unwrap().clone();
            let entered=service.state().join("crash-native-entry");
            let deadline=Instant::now()+Duration::from_secs(120);
            while !entered.exists() {
                assert!(child.try_wait().unwrap().is_none(),"scientific child exited before entry; log={}",log_path.display());
                assert!(Instant::now()<deadline,"scientific child never entered; log={}",log_path.display());
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            assert_eq!(std::fs::read_to_string(&entered).unwrap(),original_identity.pid.to_string());
            let original=store.canonical_study_point(&point_key).await.unwrap().unwrap();
            assert!(original.assigned && !original.settled);
            assert!(pse_operations::canonical_studies::point_facts(&original).unwrap().native_started);
            let original_attempt=store.canonical_attempt(original.attempt.as_ref().unwrap()).await.unwrap().unwrap();
            assert!(!original_attempt.terminal());
            // The existing owner verifies pidfd identity, kills only its fresh
            // process group, drains every member and reaps the original leader.
            let status=child.cancel().unwrap();
            assert_eq!(status.signal(),Some(9));
            assert_eq!(child.retained_status(),Some(status),"retention follows whole-group physical drain");
            assert!(chrono::Utc::now().timestamp_micros()<original_attempt.expires_at,
                "original physical death must precede even its previously observed lease expiry");
            let frozen=store.canonical_attempt(&original_attempt.key).await.unwrap().unwrap();
            assert!(!frozen.terminal(),"process death alone does not manufacture canonical completion");
            assert_eq!(frozen.generation,original_attempt.generation);
            // Wait on the original canonical lease fact, never use a sleep as
            // evidence of death or manufacture expiry through database writes.
            let deadline=Instant::now()+Duration::from_secs(45);
            while chrono::Utc::now().timestamp_micros()<=frozen.expires_at {
                assert!(Instant::now()<deadline,"original lease did not become recoverable");
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            let fresh=service.connect().await.unwrap();
            let replacement=science.attach(fresh.store(),"after-process-death");
            let entered=Arc::new(AtomicUsize::new(0));
            let observer={let entered=entered.clone();Arc::new(move |_:&std::sync::atomic::AtomicBool| {entered.fetch_add(1,Ordering::SeqCst);})};
            pse_runtime::math::install_qualified_native_entry(observer.clone()).unwrap();
            let stop=CancelSource::new();
            // A transient idle pass is not a terminal-study witness: the last
            // issued heartbeat may still settle after physical process death.
            // Keep one replacement service until the canonical conclusion, then
            // cancel and join that same owner under the existing finite watchdog.
            let mut replacement_settings=settings(); replacement_settings.until_idle=false;
            let work=pse_runtime::math::observed_native_entry(observer,
                Box::pin(science.runtimes[replacement].serve(replacement_settings,&stop)));
            tokio::pin!(work);
            let recovered=async {
                loop {
                    let status=handle.status().await?;
                    if status.points[0].settled { return Ok::<_,WorkflowError>(status); }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            };
            let recovered=tokio::select! {
                result=&mut work=>panic!("replacement service ended before original recovery: {result:?}"),
                observed=tokio::time::timeout(Duration::from_secs(120),recovered)=>observed,
            };
            let recovered=match recovered {
                Ok(Ok(status))=>status,
                other=>{ stop.cancel();let drained=work.await;panic!("original recovery unavailable: {other:?}; drain={drained:?}"); }
            };
            assert_eq!(recovered.state,pse_runtime::workflow::StudyState::Open);
            assert_eq!(recovered.points[0].attempt.as_ref(),Some(&original_attempt.key));
            assert!(!recovered.points[0].outcome.as_ref().unwrap().scientific.usable);
            assert!(!recovered.points[0].outcome.as_ref().unwrap().scientific.seed_permission);
            assert_eq!(recovered.points[0].outcome.as_ref().unwrap().effect,pse_model::study::EffectState::Unknown);
            assert_eq!(recovered.points[1].state,pse_model::generated::enums::StudyPointState::Pending);
            assert!(recovered.points[1].attempt.is_none());
            assert_eq!(entered.load(Ordering::SeqCst),0,"physical death grants no resolution of unknown effects");

            // Explicit independent authored work can use the replacement. It is
            // neither a retry nor a continuation of the unresolved original.
            let (independent,independent_revision)=science.submit(1).await;
            assert_ne!(independent.study_id(),study);
            let observe=async {
                loop {
                    let status=independent.status().await?;
                    if status.state==pse_runtime::workflow::StudyState::Concluded { return Ok::<_,WorkflowError>(status); }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            };
            let observed=tokio::select! {
                result=&mut work=>panic!("replacement service ended before independent conclusion: {result:?}"),
                observed=tokio::time::timeout(Duration::from_secs(120),observe)=>observed,
            };
            stop.cancel();
            let worked=work.await;
            let status=match observed {
                Ok(Ok(status))=>status,
                other=> {
                    let original=handle.status().await;
                    let attempt=store.canonical_attempt(&original_attempt.key).await;
                    panic!("independent conclusion unavailable: {other:?}; original={original:?}; attempt={attempt:?}; replacement drain={worked:?}");
                }
            };
            assert!(worked.unwrap()>0);
            // One occurrence can admit preparation and solve sessions separately.
            // The canonical attempt assertions below establish occurrence identity.
            assert!(entered.load(Ordering::SeqCst)>0,"independent authored work must actually enter native science");
            let services=science.runtimes[replacement].execution_services();
            let pse_runtime::workflow::Durability::Durable(operations)=services.durability() else {panic!("durable replacement");};
            let lost=operations.record(&original.run,&original_attempt.key).await.unwrap();
            assert_eq!(lost.attempt.as_ref().unwrap().outcome.as_ref().map(|value|value.as_str()),Some("failed"));
            assert!(lost.completion.as_ref().unwrap().completion.is_none());
            assert!(lost.solutions.is_empty(),"killed native entry supplies no reusable seed");
            assert_eq!(lost.attempt.as_ref().unwrap().generation,original_attempt.generation);
            let lost_id=lost.attempt_id;drop(lost);
            let original_status=handle.status().await.unwrap();
            assert_eq!(original_status.state,pse_runtime::workflow::StudyState::Open);
            assert_eq!(original_status.points[0].outcome.as_ref().unwrap().attempts.len(),1);
            assert_eq!(original_status.points[0].outcome.as_ref().unwrap().attempts[0].attempt_id,Some(lost_id));
            assert!(original_status.points[1].attempt.is_none());
            let original_run=store.canonical_run(&original.run).await.unwrap().unwrap();
            assert_eq!(original_run.revision,revision);
            assert!(original_run.current_generation>original_attempt.generation);
            let original_summary=store.canonical_study_summary(&study.to_string()).await.unwrap().unwrap();
            let parent=store.canonical_run(&original_summary.run).await.unwrap().unwrap();
            assert!(store.canonical_attempt_page(&original_summary.run,None,parent.current_generation).await.unwrap().is_empty(),
                "unknown original effects cannot produce a completed parent inventory");
            let [point]=status.points.as_slice() else {panic!("one independent occurrence");};
            assert!(point.settled && point.outcome.as_ref().unwrap().scientific.usable);
            assert_eq!(point.outcome.as_ref().unwrap().attempts.len(),1);
            let run=store.canonical_run(&point.run).await.unwrap().unwrap();
            assert_eq!(run.revision,independent_revision);
            value(&science.runtimes[replacement],&point.run,point.attempt.as_ref().unwrap()).await;
            let summary=store.canonical_study_summary(&independent.study_id().to_string()).await.unwrap().unwrap();
            let parent=store.canonical_run(&summary.run).await.unwrap().unwrap();
            let attempts=store.canonical_attempt_page(&summary.run,None,parent.current_generation).await.unwrap();
            assert_eq!(attempts.len(),1,"one independent parent finalization");assert!(attempts[0].terminal);
        }).catch_unwind().await;
        // Even failed assertions attempt child drain before the enclosing owner
        // independently closes native lanes, connections and the private service.
        let drained=child.cancel();
        if let Err(error)=&drained {eprintln!("crash worker physical drain failed: {error}; log={}",log_path.display());}
        if let Err(panic)=result {std::panic::resume_unwind(panic);}
        drained.unwrap();
    })
}
async fn run_private(case:NativeStudyCase) {
    let mut service=service();
    service.run(Instant::now()+Duration::from_secs(240),|service|Box::pin(async move {
        service.launch(Instant::now()+Duration::from_secs(90)).await?;
        let store=service.ready()?.store().clone();
        let left=service.connect().await?;
        let right=service.connect().await?;
        let mut science=Science::new(&[store.clone(),left.store().clone(),right.store().clone()]);
        let body=match case {
            NativeStudyCase::Completed=>journey(&mut science,&store,service,false),
            NativeStudyCase::BeforeClaim=>journey(&mut science,&store,service,true),
            NativeStudyCase::ActiveCancellation=>active_cancellation(&mut science,&store,service),
            NativeStudyCase::CrashReplacement=>crash_replacement(&mut science,&store,service),
        };
        let result=AssertUnwindSafe(body).catch_unwind().await;
        if let Err(error)=science.close().await {
            eprintln!("independent native-lane cleanup failed: {error}");
            if result.is_ok() {panic!("native-lane cleanup failed after successful science: {error}");}
        }
        if let Err(panic)=result {std::panic::resume_unwind(panic);}
        Ok(())
    })).await.unwrap();
    service.dispose().unwrap();
}
#[tokio::test]
async fn native_recorded_study_two_workers_preserve_science_protection_and_lineage_after_producer_close() {run_private(NativeStudyCase::Completed).await;}
#[tokio::test]
async fn native_recorded_study_cancelled_before_claim_never_enters_native_and_reopens_its_summary() {run_private(NativeStudyCase::BeforeClaim).await;}
#[tokio::test]
async fn native_recorded_study_active_cancellation_joins_original_owner_before_replacement() {run_private(NativeStudyCase::ActiveCancellation).await;}
#[tokio::test]
async fn native_recorded_study_killed_process_is_drained_before_replacement_and_preserves_unknown_effect() {
    if let Some(state)=std::env::var_os(CRASH_STATE) {crash_child(Path::new(&state)).await;return;}
    run_private(NativeStudyCase::CrashReplacement).await;
}
