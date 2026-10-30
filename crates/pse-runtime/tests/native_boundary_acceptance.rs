// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Retained scientific boundary assertions formerly exercised through the Python adapter.
//! Both callers own fresh canonical services; managed detail comes from a real external worker.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr,
    reason = "independent scientific assertions and explicit original-fixture settlement"
)]

#[path = "../../../tests/fixtures/native-worker-environment.rs"]
mod native_worker_environment;

use futures_util::{FutureExt, future::BoxFuture};
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_backend_native::{
    presolve::PolicyKind,
    solve::{Backend, Controls, SolveIntent},
};
use pse_columnar::CancellationToken;
use pse_ids::{ContentHash, Frame, FramedHasher};
use pse_model::{
    generated::{
        enums::{
            CandidateRefusal, CanonicalAttemptPhase, CanonicalTerminalOutcome, NativeAssurance,
            NativeBackend, NativeBoundaryClass, NativeQualification, NativeTermination,
        },
        identities::DeclarationId,
    },
    study::{OccurrenceKey, StartPolicy},
};
use pse_operations::{
    canonical::CanonicalStore,
    native_service::{NativeCanonicalService, NativeModuleSource, NativeServiceSpec},
};
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{candidate_assessments, modeling_findings, solve_runs, solve_variables},
};
use pse_runtime::{
    CancelSource, SharedRuntime,
    math::settings::SolveSettings,
    observation_detail::{
        CaptureBudget, DetailOutcome, DetailRequest, DetailRequirement, start_detailed_collection,
    },
    workflow::{
        CanonicalDeployment, CaseOperation, DurabilitySelection, LeasePolicy,
        ManagedDetailReferenceState, ModelingPackage, NativeAllocation, NativeExecutionAllocation,
        NativeStudyWorker, NativeStudyWorkerSpec, OperationRequest, OperationSource,
        OuterAttestation, PointOverlay, PreparationSettings, Runtime, RuntimeOperationRequest,
        StudyInputs, StudyOutputDemand, StudyPoint, StudyPointPolicy, WorkerSettings,
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

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn service() -> NativeCanonicalService {
    let environment = std::env::vars_os().collect();
    let module_source = if let Some(receipt) = std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_RECEIPT")
    {
        NativeModuleSource::Prepared(
            pse_host::study_module::select(
                &PathBuf::from(receipt),
                pse_host::study_module::Variant::Production,
            )
            .unwrap(),
        )
    } else {
        NativeModuleSource::RecordedState(PathBuf::from(std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_STATE")
            .expect("explicit prepared module selector; the fixture copies its immutable module into fresh state")))
    };
    NativeCanonicalService::new(NativeServiceSpec {
        server: pse_host::server::select(&environment).unwrap(),
        helper: PathBuf::from(
            std::env::var_os("PSE_NATIVE_FIXTURE_SERVICE_HELPER")
                .expect("explicit already-built native canonical helper"),
        ),
        module_source,
        state_parent: std::env::temp_dir(),
    })
    .unwrap()
}

struct Science {
    shared: Arc<SharedRuntime>,
    runtime: Runtime,
    _spill: tempfile::TempDir,
}
impl Science {
    fn new(store: CanonicalStore, observer: bool) -> Self {
        let spill = tempfile::tempdir().unwrap();
        let n = |value| NonZeroUsize::new(value).unwrap();
        let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
            memory_limit_bytes: n(if observer { 512 << 20 } else { 4 << 30 }),
            spill_dir: spill.path().into(),
            max_temp_dir_bytes: 1 << 30,
            top_consumers: n(16),
            threads: pse_engine::ThreadBudget {
                pool_threads: n(16),
                target_partitions: n(16),
            },
            execution: Default::default(),
            cache: pse_runtime::CacheBudget::disabled(1024),
            math: pse_runtime::math::MathPolicy {
                worker_bytes: if observer { 128 << 20 } else { 1 << 30 },
                workspace_bytes: if observer { 128 << 20 } else { 1 << 30 },
                foreign_bytes: 16 << 20,
                jobs: std::thread::available_parallelism().unwrap().get(),
                ..Default::default()
            },
        })
        .unwrap();
        let mut executable = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
        let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
        hash.part_reader(
            executable.metadata().unwrap().len(),
            &mut executable,
            &|| Ok(()),
        )
        .unwrap();
        let build: ContentHash = hash.finish_hash();
        let factory = Arc::new(
            shared
                .session_factory(pse_engine::session::native_engine_profile())
                .unwrap(),
        );
        let runtime = Runtime::from_shared(
            shared.clone(),
            factory
                .bind_model(pse_schema::shared_registry().unwrap())
                .unwrap(),
            CanonicalDeployment::new(
                store,
                OuterAttestation {
                    source: None,
                    build,
                },
                None,
            ),
        )
        .unwrap()
        .with_durability(DurabilitySelection::Durable {
            worker: "native-boundary-acceptance".into(),
            policy: LeasePolicy::new(
                Duration::from_secs(30),
                Duration::from_millis(100),
                64,
                Duration::from_millis(20),
            )
            .unwrap(),
        })
        .unwrap();
        Self {
            shared,
            runtime,
            _spill: spill,
        }
    }
    async fn close(&self) -> Result<(), pse_runtime::math::MathRuntimeError> {
        self.runtime.close();
        self.runtime.clear_program_cache();
        self.shared.math().shutdown_native_lanes().await
    }
}
fn texts(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, at: &Path, output: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, output);
            } else if path.extension().is_some_and(|suffix| {
                ["toml", "yaml", "yml", "pse"]
                    .iter()
                    .any(|allowed| suffix == *allowed)
            }) {
                output.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/"),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
}
fn package<'a>(
    science: &'a Science,
    physical_root: PathBuf,
    source: String,
) -> BoxFuture<'a, (ModelingPackage, DeclarationId)> {
    Box::pin(async move {
        let physical_texts = texts(&physical_root);
        let declaration: toml::Value =
            toml::from_str(std::str::from_utf8(&physical_texts["package.toml"]).unwrap()).unwrap();
        let physical = &declaration["package"];
        let version: Vec<_> = physical["version"]
            .as_str()
            .unwrap()
            .split('.')
            .map(|value| value.parse::<u32>().unwrap())
            .collect();
        assert_eq!(version.len(), 3);
        let dependency = format!(
            "dependencies = [{{ package_id = \"{}\", version_req = {{ operator = \"exact\", major = {}, minor = {}, patch = {} }} }}]",
            physical["package_id"].as_str().unwrap(),
            version[0],
            version[1],
            version[2]
        );
        let manifest = std::fs::read_to_string(
            root().join("tests/fixtures/packages/minimal_explicit/package.toml"),
        )
        .unwrap()
        .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
        .replace("dependencies = []", &dependency);
        let modeling = BTreeMap::from([
            ("package.toml".into(), manifest.into_bytes()),
            ("models/root.pse".into(), source.into_bytes()),
        ]);
        let runtime = &science.runtime;
        let pool = science.shared.pool();
        let token = CancellationToken::new();
        let validation = runtime.validation_context();
        let load = |texts: &BTreeMap<String, Vec<u8>>| {
            OwnedDocumentSet::try_from_bundles(
                vec![
                    load_package_documents_owned(
                        texts,
                        runtime.registry(),
                        Default::default(),
                        &pool,
                        &token,
                        &validation,
                    )
                    .unwrap(),
                ],
                &pool,
                &token,
            )
            .unwrap()
        };
        let physical = Box::pin(runtime.physical_from_documents(&load(&physical_texts), &token))
            .await
            .unwrap();
        let package = Box::pin(runtime.modeling_from_documents(
            &load(&modeling),
            physical,
            &CancelSource::new(),
        ))
        .await
        .unwrap();
        let case = package
            .declarations(&runtime.preparation_services())
            .await
            .unwrap()
            .iter()
            .find(|declaration| declaration.name == "Root")
            .unwrap()
            .declaration_id;
        (package, case)
    })
}

// Each relation's generated view retains registry validation at this stored-result boundary.
macro_rules! stored_rows {
    ($name:ident, $relation:ident) => {
        async fn $name(runtime: &Runtime, run: &str, attempt: &str) -> Vec<$relation::Row> {
            let mut reader = runtime
                .results(
                    run,
                    attempt,
                    $relation::QUALIFIED_NAME,
                    0,
                    u64::MAX,
                    CancellationToken::new(),
                )
                .await
                .unwrap();
            let mut rows = Vec::new();
            while let Some(batch) = reader.next_batch().await.unwrap() {
                let view = $relation::View::try_from_batch_with_registry(
                    runtime.registry(),
                    &batch,
                    &runtime.validation_context(),
                )
                .unwrap();
                rows.extend(view.rows().unwrap());
            }
            reader.close();
            rows
        }
    };
}
stored_rows!(stored_solves, solve_runs);
stored_rows!(stored_assessments, candidate_assessments);
stored_rows!(stored_findings, modeling_findings);
stored_rows!(stored_variables, solve_variables);

async fn memory_stop(science: &Science) {
    let runtime = &science.runtime;
    let cancel = CancelSource::new();
    let (package, case) = package(
        science,
        root().join("packages/reference/physical"),
        std::fs::read_to_string(root().join("tests/fixtures/models/native-price-taker.pse"))
            .unwrap(),
    )
    .await;
    let prepared = Box::pin(package.prepare_operation(
        &runtime.preparation_services(),
        &OperationSource::of(&package),
        &PreparationSettings::default(),
        RuntimeOperationRequest::DeclaredCase {
            case,
            settings: Box::new(SolveSettings {
                intent: SolveIntent::Optimize,
                backend: Some(Backend::Scip),
                // The original scientific heap allowance, not a host memory cap.
                controls: Controls {
                    foreign_bytes: Some(1 << 20),
                    ..Default::default()
                },
                ..Default::default()
            }),
        },
        None,
        &cancel,
    ))
    .await
    .unwrap();
    let handle = prepared
        .start(&runtime.execution_services(), &cancel)
        .await
        .unwrap();
    let run = handle
        .canonical_run_key()
        .expect("original durable run")
        .to_owned();
    let attempt = handle
        .canonical_attempt_key()
        .expect("original durable attempt")
        .to_owned();
    let result = Box::pin(handle.wait()).await.unwrap();
    assert_eq!(result.canonical_run_key(), Some(run.as_str()));
    assert_eq!(result.canonical_attempt_key(), Some(attempt.as_str()));
    let stored = runtime
        .canonical_store()
        .unwrap()
        .canonical_attempt(&attempt)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.run, run);
    assert_eq!(stored.phase, CanonicalAttemptPhase::Terminal);
    assert_eq!(stored.outcome, Some(CanonicalTerminalOutcome::Failed));
    let completion = result
        .completion()
        .expect("actual resource-stopped completion");
    assert_eq!(completion.solves.len(), 1);
    let solve = &completion.solves[0];
    assert_eq!(solve.backend, Some(NativeBackend::Scip));
    assert_eq!(solve.native_status.as_deref(), Some("SCIP_STATUS_MEMLIMIT"));
    assert_eq!(
        solve.termination,
        Some(NativeTermination::ResourceExhausted)
    );
    assert!(
        solve.candidate_kind.is_none() && solve.feasible.is_none() && solve.objective.is_none()
    );
    assert_eq!(solve.qualification, NativeQualification::Unqualified);
    assert_eq!(solve.assurance, NativeAssurance::None);
    assert!(!result.usable());
    assert_eq!(completion.assessments.len(), 1);
    let assessment = &completion.assessments[0];
    assert!(assessment.numerically_feasible.is_none());
    assert_eq!(assessment.validated, Some(false));
    assert_eq!(
        assessment.qualification,
        Some(NativeQualification::Unqualified)
    );
    assert!(assessment.refusals.contains(&CandidateRefusal::NoCandidate));
    assert!(
        assessment
            .refusals
            .contains(&CandidateRefusal::FeasibilityUnavailable)
    );
    assert!(!assessment.refusals.contains(&CandidateRefusal::Infeasible));
    assert!(!assessment.permits_result && !assessment.permits_seed);
    assert!(
        completion
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.class
                == pse_model::diagnostic::BoundaryClass::ResourceLimit)
    );
    assert_eq!(
        stored_solves(runtime, &run, &attempt).await,
        completion.solves
    );
    assert_eq!(
        stored_assessments(runtime, &run, &attempt).await,
        completion.assessments
    );
    let local_findings =
        modeling_findings::Row::rows(&result.table("runtime.modeling_findings").unwrap()).unwrap();
    let retained_findings = stored_findings(runtime, &run, &attempt).await;
    assert_eq!(retained_findings, local_findings);
    assert!(
        retained_findings
            .iter()
            .any(|finding| finding.class == NativeBoundaryClass::ResourceLimit)
    );
    let variables =
        solve_variables::Row::rows(&result.table("runtime.solve_variables").unwrap()).unwrap();
    let candidates: Vec<_> = variables
        .iter()
        .filter(|variable| !variable.parameter)
        .collect();
    assert_eq!(candidates.len(), 9);
    assert!(candidates.iter().all(|variable| variable.value.is_none()));
    let mut retained_variables = stored_variables(runtime, &run, &attempt).await;
    let mut local_variables = variables;
    // The relational reader orders by native keys, while the local table keeps
    // its construction order. Compare the complete keyed population, including
    // duplicates, rather than impose one incidental ordering across boundaries.
    retained_variables.sort_by_key(|row| (row.step, row.symbol_id));
    local_variables.sort_by_key(|row| (row.step, row.symbol_id));
    assert_eq!(retained_variables, local_variables);
}

const STUDY_ROOT: &str = "package algebraic { def Root { param a:Scalar = 4; var x:Scalar; eq square:x*x==a; annotation start x(1); annotation bounds x(0,10); annotation report x(\"root\"); annotation check x(x>1); } }";

async fn selected_managed_detail(science: &Science, output: &Path) {
    let runtime = &science.runtime;
    let (package, case) = package(
        science,
        root().join("tests/fixtures/packages/physical-primitives"),
        STUDY_ROOT.into(),
    )
    .await;
    let points = [7, 11]
        .into_iter()
        .map(|key| StudyPoint {
            operation: OperationRequest::DeclaredCase(CaseOperation {
                case,
                route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                settings: SolveSettings {
                    intent: SolveIntent::FeasiblePoint,
                    backend: Some(Backend::Ipopt),
                    presolve: PolicyKind::Off,
                    ..Default::default()
                },
            }),
            preparation: PreparationSettings::default(),
            overlay: PointOverlay::default(),
            policy: StudyPointPolicy {
                key: OccurrenceKey(key),
                dependencies: vec![],
                start: StartPolicy::Fresh,
                attempt_limit: 1,
            },
        })
        .collect();
    let services = runtime.execution_services();
    let cancel = CancelSource::new();
    let local = Box::pin(package.admit_study(
        &services,
        &StudyInputs::Explicit(points),
        &StudyOutputDemand::Full {},
        &cancel,
    ))
    .await
    .unwrap();
    let operation = runtime
        .observations()
        .hub()
        .start("native-boundary.acceptance.managed-detail");
    let capture = start_detailed_collection(
        operation.context(),
        DetailRequest {
            version: Default::default(),
            families: vec!["native_owner_return.v1".into()],
            requirement: DetailRequirement::Complete,
            sample_every: 1,
            boundary_thread_cpu: false,
            expiry_monotonic_ns: None,
            duration: Duration::from_secs(120),
            output: output.to_owned(),
            budget: CaptureBudget {
                memory_bytes: 16 << 20,
                output_bytes: 16 << 20,
            },
            managed_budget: Some(CaptureBudget {
                memory_bytes: 2 * (4 << 20),
                output_bytes: 2 * (8 << 20),
            }),
        },
    )
    .unwrap();
    pse_diagnostics::observation::scope(Some(capture.context()), async {
        let recorded = Box::pin(runtime.record_study(&local, &cancel))
            .await
            .unwrap();
        let report = Box::pin(runtime.execute_study(&recorded, 2, &cancel))
            .await
            .unwrap();
        assert_eq!(report.outcomes.len(), 2);
        assert_eq!(report.results.len(), 2);
        assert_eq!(report.decision.conclusion.availability.as_str(), "complete");
        assert!(
            report
                .outcomes
                .iter()
                .all(|outcome| outcome.scientific.usable)
        );
        let selected = report
            .diagnostic_selection()
            .await
            .unwrap()
            .expect("recorded native selection");
        assert!(selected.selection.as_ref().unwrap().refusal.is_none());
        assert_eq!(selected.omitted_points, 0);
        assert_eq!(selected.references.len(), 2);
        for reference in selected.references {
            assert_eq!(reference.state, ManagedDetailReferenceState::Captured);
            let owner = reference.owner.expect("actual external scientific owner");
            assert_ne!(owner.pid, std::process::id());
            assert_eq!(owner.run, reference.run);
            assert_eq!(owner.attempt, reference.attempt);
            assert!(
                report
                    .results
                    .iter()
                    .flatten()
                    .any(|result| result.stored_keys()
                        == Some((reference.run.as_str(), reference.attempt.as_str())))
            );
            let detail = reference.detail.expect("readable original detail prefix");
            assert_eq!(detail.outcome, DetailOutcome::Complete);
            assert!(!detail.source_partial);
            assert_eq!(detail.families.len(), 1);
            assert_eq!(detail.families[0].observed, detail.families[0].acknowledged);
            assert!(detail.families[0].observed > 0);
            let values = stored_variables(runtime, &reference.run, &reference.attempt).await;
            let candidates: Vec<_> = values
                .iter()
                .filter(|variable| !variable.parameter)
                .collect();
            assert_eq!(candidates.len(), 1);
            let value = candidates[0].value.expect("original feasible root");
            let tolerance = candidates[0]
                .tolerance
                .expect("original physical allowance");
            assert!(
                value.is_finite()
                    && tolerance.is_finite()
                    && tolerance > 0.
                    && (value - 2.).abs() <= tolerance
            );
        }
    })
    .await;
    let local = capture.finish().unwrap();
    assert_eq!(
        local.outcome,
        DetailOutcome::Incomplete,
        "an absent local family cannot acquire completeness from external child coverage"
    );
    operation.finish(pse_diagnostics::observation::Outcome::Completed);
}

async fn acceptance(managed: bool) {
    let mut service = service();
    let outcome = AssertUnwindSafe(service.run(Instant::now() + Duration::from_secs(600), |service| {
        Box::pin(async move {
            service.launch(Instant::now() + Duration::from_secs(90)).await?;
            let mut science = Science::new(service.ready()?.store().clone(), managed);
            let mut worker: Option<NativeStudyWorker> = None;
            let body = AssertUnwindSafe(async {
                if managed {
                    let allocation = NativeAllocation::materialize(8 << 30, 512 << 20, 1, 6 << 30,
                        Some(NativeExecutionAllocation { pool_memory_bytes: 4 << 30, worker_bytes: 1 << 30,
                            cpu_threads: 16, case_lanes: 16, math_jobs: 32, compiler_cores: 1,
                            admission_wait_ms: 30_000, process_headroom_bytes: 2 << 30, observer_memory_bytes: 512 << 20 })).unwrap();
                    worker = Some(NativeStudyWorker::new(NativeStudyWorkerSpec { ready: service.ready()?,
                        executable: PathBuf::from(env!("CARGO_BIN_EXE_pse-worker")),
                        environment: native_worker_environment::selected(), allocation,
                        settings: WorkerSettings { maximum_in_flight: Some(2), ..Default::default() },
                    }).unwrap());
                    let worker = worker.as_mut().unwrap();
                    worker.launch(Instant::now() + Duration::from_secs(90)).await.unwrap();
                    science.runtime = science.runtime.clone().with_managed_worker(worker.observer().unwrap()).unwrap();
                    selected_managed_detail(&science, &service.state().join("selected-native-detail")).await;
                } else { memory_stop(&science).await; }
                Ok::<(), pse_operations::native_service::NativeServiceError>(())
            }).catch_unwind().await;
            // Independent owners settle even when science or a preceding teardown fails.
            let worker_drain = match worker.as_mut() {
                Some(worker) => worker.shutdown(Instant::now() + Duration::from_secs(45)).await.map(|_| ()) ,
                None => Ok(()),
            };
            let native_drain = science.close().await;
            if body.is_ok() && worker_drain.is_ok()
                && let Some(worker) = worker.as_mut() { worker.dispose().unwrap(); }
            if let Err(panic) = body {
                eprintln!("native boundary assertion failed; worker={worker_drain:?}; lanes={native_drain:?}; state={}", service.state().display());
                if let Some(worker) = &worker { eprintln!("external native diagnostics={}", worker.directory().display()); }
                std::panic::resume_unwind(panic);
            }
            worker_drain.unwrap(); native_drain.unwrap();
            body.unwrap()?;
            Ok(())
        })
    })).catch_unwind().await;
    match outcome {
        Err(panic) => {
            eprintln!(
                "native boundary private state retained={}",
                service.state().display()
            );
            std::panic::resume_unwind(panic);
        }
        Ok(Err(error)) => panic!(
            "native boundary failure={error}; private state={}",
            service.state().display()
        ),
        Ok(Ok(())) => service.dispose().unwrap(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn public_durable_scip_memory_stop_retains_unavailable_feasibility_and_exact_rows() {
    acceptance(false).await;
}
#[tokio::test(flavor = "multi_thread")]
async fn public_managed_detail_keeps_external_owners_exact_attempts_and_complete_prefixes() {
    acceptance(true).await;
}
