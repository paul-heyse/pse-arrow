// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admitted continuation and independent studies, with explicit adapter and clock scopes.
use super::k4_support as support;
use super::*;
use pse_model::{
    generated::enums::{ModelingAnalysisRoute, StudyPointState, StudyState},
    scalars::FiniteBound,
    study::*,
};
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{solve_metrics, solve_strategy_events, solve_variables},
};
use pse_runtime::workflow::{self, Runtime};
use serde_json::{Value, json};
const VALUES: [f64; 8] = [2., 2., 3., 2., 4., 3., 2., 2.];

/// Separate recipe-selected profile; the existing eight-point fixtures retain
/// their one-thread/64 GiB budget and native policy.
fn study_owner(independent: bool, threads: usize, managed: bool) -> WorkflowRuntime {
    if managed {
        WorkflowRuntime::managed_observer().unwrap()
    } else if independent {
        WorkflowRuntime::independent_reference(NonZeroUsize::new(threads).unwrap()).unwrap()
    } else {
        WorkflowRuntime::with_threads(NonZeroUsize::new(1).unwrap()).unwrap()
    }
}

/// Retain the isolated database until its exact external receiver has drained.
/// Public package.study owns primary activation; this guard adds fixture teardown.
struct ManagedReceiver {
    store: Option<pse_operations::canonical::CanonicalStore>,
    finish_result: Option<Result<(), String>>,
    drain_recorded: bool,
}
// Keep every failed receiver owner rather than dropping/replacing an earlier one.
// One undrained receiver prevents later managed benchmark launches in this process.
static UNDRAINED_RECEIVER: std::sync::Mutex<Vec<pse_operations::canonical::CanonicalStore>> =
    std::sync::Mutex::new(Vec::new());
impl ManagedReceiver {
    fn new(runtime: &Runtime) -> Self {
        audit::managed_target(runtime);
        assert!(
            UNDRAINED_RECEIVER
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty(),
            "previous managed receiver did not drain; isolated fixture preserved"
        );
        Self {
            store: Some(runtime.canonical().unwrap().store().clone()),
            finish_result: None,
            drain_recorded: false,
        }
    }
    /// Attempt this exact registered drain once; failure preserves the fixture owner.
    fn finish(&mut self) -> Result<(), String> {
        if let Some(result) = &self.finish_result {
            return result.clone();
        }
        // Mark the attempt before fallible selection/launch. Drop must never
        // retry it while unwinding or turn an original failure into a double panic.
        self.finish_result = Some(Err("managed drain attempt did not return".into()));
        let result = self.try_finish();
        if result.is_err() {
            self.preserve_store();
        }
        audit::receiver_drain_result(result.is_ok());
        self.drain_recorded = true;
        self.finish_result = Some(result.clone());
        result
    }
    fn try_finish(&self) -> Result<(), String> {
        let store = self
            .store
            .as_ref()
            .ok_or("managed fixture owner is absent")?;
        let selected = workflow::NativeExecutionSelection::from_store(store)
            .map_err(|error| format!("managed drain selection: {error}"))?;
        let receiver = selected
            .receiver()
            .ok_or("selected managed drain receiver is absent")?;
        let status = std::process::Command::new(&receiver.supervisor_executable)
            .current_dir(support::repository())
            .arg("-m")
            .arg("scripts.case_measure")
            .arg("--stop-managed-primary")
            .arg(store.database())
            .arg("--state")
            .arg(selected.receiver_state())
            .status()
            .map_err(|error| format!("managed drain launch: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "exact managed receiver did not drain ({status}); isolated fixture preserved"
            ))
        }
    }
    fn preserve_store(&mut self) {
        if let Some(store) = self.store.take() {
            UNDRAINED_RECEIVER
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(store);
        }
    }
}
impl Drop for ManagedReceiver {
    fn drop(&mut self) {
        if self.finish_result.is_none() {
            // A destructor is a best-effort safety net, never a second source
            // of panics. Explicit finish supplies the checked successful route.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.finish()));
        }
        if self.finish_result.as_ref().is_some_and(Result::is_err) {
            if !self.drain_recorded {
                audit::receiver_drain_result(false);
                self.drain_recorded = true;
            }
            self.preserve_store();
        }
    }
}

async fn definition(
    package: &ModelingPackage,
    physical: &workflow::PhysicalContext,
    sources: &workflow::PackageSources,
    values: &[f64],
    parallel: bool,
) -> workflow::LocalStudy {
    let declarations = package.declarations().await.unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    // Assignments use the admitted registry and its declared canonical unit.
    let context = workflow::OperationSource::of(package);
    let scalar = physical.quantities().neutral_dimensionless().unwrap();
    let quantity = scalar.as_id();
    let unit = physical
        .quantities()
        .quantity_type(scalar)
        .unwrap()
        .canonical_unit
        .as_id();
    let points: Vec<_> = values
        .iter()
        .copied()
        .enumerate()
        .map(|(index, value)| workflow::StudyPoint {
            operation: workflow::OperationRequest::DeclaredCase(workflow::CaseOperation {
                case: root,
                route: ModelingAnalysisRoute::Steady,
                settings: pse_runtime::math::settings::SolveSettings {
                    backend: Some(if parallel {
                        Backend::Ipopt
                    } else {
                        Backend::Kinsol
                    }),
                    intent: pse_backend_native::solve::SolveIntent::Root,
                    presolve: pse_backend_native::presolve::PolicyKind::Off,
                    controls: pse_backend_native::solve::Controls {
                        threads: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
            preparation: workflow::PreparationSettings::default(),
            overlay: workflow::PointOverlay {
                assignments: vec![workflow::BindingAssignment {
                    target: workflow::BindingTarget::Path("a".into()),
                    value: workflow::BindingQuantity {
                        magnitude: FiniteBound::try_new(value).unwrap(),
                        quantity,
                        unit,
                    },
                }],
            },
            policy: workflow::StudyPointPolicy {
                key: OccurrenceKey(index as u32),
                dependencies: if parallel || index == 0 {
                    vec![]
                } else {
                    vec![Dependency::Ordering(OccurrenceKey(index as u32 - 1))]
                },
                start: if parallel || index == 0 {
                    StartPolicy::Fresh
                } else {
                    StartPolicy::Continuation(SeedEdge {
                        predecessor: OccurrenceKey(index as u32 - 1),
                        role: SeedRole::PrimalSolution,
                        permission: ContinuationPermission::RequireUsable,
                        unavailable: UnavailableSeedPolicy::Refuse,
                    })
                },
                attempt_limit: 1,
            },
        })
        .collect();
    let admitted = package
        .admit_study(
            &workflow::StudyInputs::Explicit(points.to_vec()),
            &workflow::StudyOutputDemand::Full {},
            &audit::cancel_source(),
        )
        .await
        .unwrap();
    if audit::review_context_request().is_some() {
        audit::retain_math_output(
            "study-original-inputs",
            &json!({
                "source_documents":{"physical":sources.physical,"modeling":sources.modeling},
                "declarations":&*declarations,
                "requested_points":points,
                "admitted_definition":{"modeling_revision":admitted.modeling_revision,
                    "variants":admitted.variants,"points":admitted.points},
                "workflow_context":{"status":"unsupported","reason":"stored study occurrences expose canonical keys, not original joined RunResult owners; managed workflow review-context bridge unavailable"}
            }),
        );
    }
    assert_eq!(
        admitted.points[0].binding_hash,
        admitted.points[1].binding_hash
    );
    assert_eq!(
        admitted.points[0].binding_hash,
        admitted.points[3].binding_hash
    );
    assert!(
        admitted
            .points
            .iter()
            .all(|p| p.operation.source.physical_context == context.physical_context)
    );
    admitted
}
fn check(outcomes: &[PointOutcome], occurrences: usize, parallel: bool) {
    assert_eq!(outcomes.len(), occurrences);
    for (index, outcome) in outcomes.iter().enumerate() {
        assert_eq!(outcome.key, OccurrenceKey(index as u32));
        assert_eq!(outcome.lifecycle, StudyPointState::Completed, "{outcome:?}");
        assert!(outcome.scientific.usable, "{outcome:?}");
        assert!(outcome.diagnostic.is_none(), "{outcome:?}");
        assert_eq!(outcome.attempts.len(), 1);
        if parallel {
            assert!(
                matches!(outcome.start, Some(StartProvenance::Fresh)),
                "{outcome:?}"
            );
        } else if index > 0 {
            assert!(
                matches!(outcome.start,Some(StartProvenance::Continuation { predecessor, .. }) if predecessor == OccurrenceKey(index as u32-1)),
                "{outcome:?}"
            );
        }
    }
}
fn metrics(rows: Vec<solve_metrics::Row>) -> Value {
    Value::Array(rows.into_iter().map(|r| json!({"namespace":r.namespace,"name":r.name,"kind":r.kind,"real":r.real,"integer":r.integer,"boolean":r.boolean,"text":r.text,"unavailable":r.unavailable})).collect())
}
async fn stored_rows<R: RelationRow>(
    runtime: &Runtime,
    owner: &WorkflowRuntime,
    run: &str,
    attempt: &str,
    name: &str,
) -> Vec<R> {
    let mut reader = runtime
        .results(run, attempt, name, 0, u64::MAX, owner.cancel.clone())
        .await
        .unwrap();
    let mut rows = Vec::new();
    while let Some(batch) = reader.next_relation_batch().await.unwrap() {
        rows.extend(R::rows(&batch).unwrap());
    }
    rows
}
fn checked_values(variables: &[solve_variables::Row], expected: f64) -> Vec<f64> {
    let values: Vec<_> = variables
        .iter()
        .filter(|row| !row.parameter && !row.fixed)
        .map(|row| row.value.unwrap())
        .collect();
    assert_eq!(values.len(), 2);
    assert!(values.iter().any(|value| (*value - expected).abs() < 1e-7));
    assert!(values.iter().any(|value| (*value - 3.).abs() < 1e-7));
    values
}

fn ephemeral(
    report: &workflow::StudyReport,
    values: &[f64],
    numerical: &mut observations::Observations,
    parallel: bool,
    service: &pse_runtime::math::MathService,
) -> Vec<Value> {
    // Retain every available original before acceptance assertions can unwind.
    for result in report.results.iter().flatten() {
        if let workflow::StudyOccurrenceResult::Ephemeral(original) = result {
            audit::retain_result(original);
        }
    }
    assert_eq!(report.results.len(), values.len());
    let mut identities = BTreeSet::new();
    report.results.iter().zip(values).enumerate().map(|(index, (result, expected))| {
        let result = result.as_ref().expect("every usable occurrence retains its original result");
        assert!(identities.insert(result.run_id()), "occurrences must retain distinct scientific identities");
        let workflow::StudyOccurrenceResult::Ephemeral(result) = result else {
            panic!("explicit ephemeral adapter returned a retained canonical handle");
        };
        // Count original joined native reports, not reopened metric projections.
        numerical.run(result, service);
        let RunReport::Modeling(reports) = result.report().unwrap() else { panic!("root study report") };
        assert_eq!(reports.len(), 1);
        if let pse_runtime::math::solves::Outcome::Constant(completed) = &reports[0].outcome {
            for native in completed.component_reports() { numerical.native(native); }
        }
        if parallel {
            match &reports[0].outcome {
                pse_runtime::math::solves::Outcome::Native(native) => assert_eq!(native.backend, Backend::Ipopt, "independent workload uses the nonbatch Ipopt adapter"),
                pse_runtime::math::solves::Outcome::Constant(completed) => {
                    assert!(completed.component_reports().len() > 0, "complete original evaluation must retain actual native component evidence");
                    for native in completed.component_reports() {
                        assert_eq!(native.backend, Backend::Ipopt, "automatic blocks retain actual nonbatch Ipopt component reports");
                    }
                }
                pse_runtime::math::solves::Outcome::Rejected(_) => panic!("usable independent root rejected native execution"),
            }
        }
        let native = metrics(solve_metrics::Row::rows(&result.table("runtime.solve_metrics").unwrap()).unwrap());
        let variables = solve_variables::Row::rows(&result.table("runtime.solve_variables").unwrap()).unwrap();
        json!({"key":index,"run_id":result.run_id,"execution":match result.report().unwrap() { RunReport::Modeling(reports) => match &reports[0].outcome { pse_runtime::math::solves::Outcome::Native(_) => "direct native", pse_runtime::math::solves::Outcome::Constant(_) => "complete original evaluation with automatic native components", pse_runtime::math::solves::Outcome::Rejected(_) => "rejected" }, _ => "other" },"native_metrics":native,"variables":checked_values(&variables, *expected)})
    }).collect()
}

/// Copy only exact original stored selections while the fixture remains live.
/// Descriptive/export failures are independent of the original study outcome.
async fn export_study_results(runtime: &Runtime, points: &[(u32, String, Option<String>)]) {
    if std::env::var("PSE_CASE_REVIEW_RESULTS").as_deref() != Ok("1") {
        return;
    }
    let Some(output) = std::env::var_os("PSE_PROCESS_COST_OUTPUT").map(PathBuf::from) else {
        return;
    };
    let parent = output.join("review-results");
    let destination = parent.join("study-original-results");
    let outcome: Result<Value, String> = async {
        let operation = pse_diagnostics::observation::current().ok_or("original study observation context unavailable")?.id();
        let cancel = pse_columnar::CancellationToken::new();
        let mut selections = Vec::new();
        let mut unavailable = Vec::new();
        for (occurrence, run, attempt) in points {
            let Some(attempt) = attempt else { continue; };
            // Discover actual exact-attempt membership through an existing protected reader.
            let reader = runtime.results(run, attempt, "runtime.solve_variables", 0, u64::MAX, cancel.clone())
                .await.map_err(|error| error.to_string())?;
            let available = reader.selection().sets().iter().map(|set| set.name.clone()).collect::<BTreeSet<_>>();
            drop(reader);
            for (index, relation) in ["runtime.solve_variables", "runtime.solve_metrics", "runtime.solve_runs",
                "runtime.solve_strategy_events", "runtime.modeling_checks"].into_iter().enumerate() {
                let Some(spec) = runtime.registry().relation(relation) else {
                    unavailable.push(format!("occurrence {occurrence}: {relation} absent from registry"));
                    continue;
                };
                if !available.contains(&spec.id.to_string()) {
                    unavailable.push(format!("occurrence {occurrence}: {relation} absent from original attempt"));
                    continue;
                }
                selections.push(workflow::ReviewScientificSelection {
                    member:format!("point-{occurrence:03}-{index}"),run:run.clone(),attempt:attempt.clone(),
                    relation:relation.into(),row_start:0,row_end:u64::MAX,output:None,
                });
            }
        }
        if selections.is_empty() { return Err("no original terminal stored point selections available".into()); }
        let evidence = pse_runtime::frozen_inspection::FrozenEvidenceSet::default();
        let pool: std::sync::Arc<dyn pse_columnar::MemoryPool> = std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
        let analysis = pse_runtime::review_analysis::analyze(&evidence,
            &pse_runtime::review_analysis::AnalysisRequest::new(operation),&pool,&cancel)
            .map_err(|error| error.to_string())?;
        let mut request = pse_runtime::review_pack::PackExportRequest::default();
        request.selection.maximum_output_bytes = 256 << 20;
        request.selection.missing_context = unavailable;
        request.selection.missing_context.push("Original study source/definition bytes retained separately; original managed RunResult review-context bridge unavailable".into());
        request.scientific_results = selections;
        std::fs::create_dir_all(&parent).map_err(|error| error.to_string())?;
        let index = runtime.export_review_pack(&destination,&evidence,&analysis,request,cancel)
            .await.map_err(|error| error.to_string())?;
        Ok(json!(index))
    }.await;
    let record = match outcome {
        Ok(index) => {
            json!({"version":1,"status":"published","pack":destination,"points":points,"index":index})
        }
        Err(error) => {
            audit::review_context_error(format!("study protected results: {error}"));
            json!({"version":1,"status":"unavailable","pack":destination,"points":points,"error":error})
        }
    };
    if let Err(error) = std::fs::create_dir_all(&parent)
        .map_err(|error| error.to_string())
        .and_then(|()| {
            let bytes = serde_json::to_vec_pretty(&record).map_err(|error| error.to_string())?;
            std::fs::write(parent.join("study-original-results-export.json"), bytes)
                .map_err(|error| error.to_string())
        })
    {
        audit::review_context_error(error);
    }
}

async fn persisted(
    runtime: &Runtime,
    owner: &WorkflowRuntime,
    points: &[(u32, String, Option<String>)],
    numerical: &mut observations::Observations,
    values: &[f64],
    independent: bool,
) -> Vec<Value> {
    let mut observations = Vec::new();
    assert_eq!(points.len(), values.len());
    assert_eq!(
        points
            .iter()
            .map(|(_, run, _)| run)
            .collect::<BTreeSet<_>>()
            .len(),
        points.len(),
        "durable occurrences must retain distinct run identities"
    );
    for (index, expected) in values.iter().copied().enumerate() {
        let (occurrence, run, attempt) = &points[index];
        assert_eq!(*occurrence, index as u32);
        let attempt = attempt
            .as_ref()
            .expect("usable occurrence has exact producing attempt");
        let rows = stored_rows::<solve_metrics::Row>(
            runtime,
            owner,
            run,
            attempt,
            "runtime.solve_metrics",
        )
        .await;
        numerical.persisted_metrics(&rows);
        let native = metrics(rows);
        let strategy = stored_rows::<solve_strategy_events::Row>(
            runtime,
            owner,
            run,
            attempt,
            "runtime.solve_strategy_events",
        )
        .await;
        if independent {
            assert!(
                strategy
                    .iter()
                    .any(|row| row.backend == Some(Backend::Ipopt)),
                "original retained independent occurrence must include actual Ipopt execution"
            );
            assert!(
                strategy
                    .iter()
                    .all(|row| row.backend.is_none() || row.backend == Some(Backend::Ipopt)),
                "retained original components must use the independent Ipopt adapter"
            );
        }
        numerical.rows(&strategy);
        let variables = stored_rows::<solve_variables::Row>(
            runtime,
            owner,
            run,
            attempt,
            "runtime.solve_variables",
        )
        .await;
        let values = checked_values(&variables, expected);
        observations.push(json!({"key":index,"run":run,"attempt":attempt,"native_metrics":native,"variables":values}));
    }
    observations
}

pub(super) fn measure(
    c: &mut Criterion,
    spec: &Value,
    output: &std::path::Path,
    phases: &phases::Phases,
) {
    let durable_mode = spec["adapter"] == "durable";
    let managed = spec["adapter"] == "managed-durable";
    let independent = managed
        || spec["adapter"] == "in-process-parallel"
        || spec["adapter"] == "in-process-serial";
    assert!(durable_mode || independent || spec["adapter"] == "in-process");
    let occurrences = if independent { 16 } else { VALUES.len() };
    let threads = if managed || spec["adapter"] == "in-process-parallel" {
        16
    } else {
        1
    };
    let observer_threads = if managed { 1 } else { threads };
    assert_eq!(spec["blocks"].as_u64(), Some(occurrences as u64));
    assert_eq!(spec["threads"].as_u64(), Some(threads));
    let values: Vec<_> = (0..occurrences)
        .map(|index| VALUES[index % VALUES.len()])
        .collect();
    let name = spec["id"].as_str().unwrap();
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(observer_threads as usize)
        .enable_all()
        .build()
        .unwrap();
    let mut records = Vec::new();
    let mut invocation_records = observations::InvocationRecords::new(output, name);
    let mut group = c.benchmark_group("process");
    group
        .sample_size(10)
        .sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(1))
        .measurement_time(Duration::from_secs(1));
    group.bench_function(name, |b| b.iter_custom(|iterations| {
        let mut timed = Duration::ZERO;
        for _ in 0..iterations {
            let (elapsed, record) = invocation_records.catch(|| executor.block_on(Box::pin(async {
                let mut numerical=observations::Observations::default();
                let started=Instant::now();
                let owner=study_owner(independent, threads as usize, managed);
                let _binding=audit::bind_owner(&owner);
                let pool=owner.runtime.pool();
                let runtime=if durable_mode || managed { runtime(&owner) } else {
                    runtime(&owner).with_durability(workflow::DurabilitySelection::Ephemeral).unwrap()
                };
                let mut receiver=managed.then(|| ManagedReceiver::new(&runtime));
                let result=audit::caught(Box::pin(async {
                let setup_stage=audit::stage("study_source_and_definition_admission");
                let budget=owner.runtime.budget();
                assert_eq!(budget.threads.pool_threads.get(), observer_threads as usize);
                if independent && !managed {
                    assert_eq!(budget.memory_limit_bytes.get(), 128_usize << 30);
                    assert_eq!(budget.math.jobs, 32);
                    assert_eq!(budget.math.worker_bytes, 16_usize << 30);
                    assert_eq!(budget.threads.target_partitions.get(), 16);
                    assert!(std::thread::available_parallelism().unwrap().get() >= threads as usize,
                        "reference workload requires its declared effective CPU permits; no reduced profile fallback");
                }
                let runtime_profile=json!({"threads":budget.threads.pool_threads.get(),"target_partitions":budget.threads.target_partitions.get(),"jobs":budget.math.jobs,"pool_limit_bytes":budget.memory_limit_bytes.get(),"worker_bytes":budget.math.worker_bytes,"cache":format!("{:?}",budget.cache),"math_policy":format!("{:?}",budget.math)});

                let receiving_profile=if managed {
                    let selected=workflow::NativeExecutionSelection::from_store(runtime.canonical().unwrap().store()).unwrap();
                    let allocation=selected.allocation();
                    allocation.validate().unwrap();
                    assert_eq!(allocation.native_workers, 1);
                    assert_eq!(allocation.native_worker_memory_bytes, 140_usize << 30);
                    let profile=allocation.execution.unwrap();
                    assert_eq!((profile.pool_memory_bytes,profile.worker_bytes,profile.cpu_threads,profile.case_lanes,profile.math_jobs,profile.compiler_cores,profile.observer_memory_bytes),
                        (128_usize << 30,16_usize << 30,16,16,32,1,4_usize << 30));
                    assert!(budget.memory_limit_bytes.get() <= profile.observer_memory_bytes);
                    json!({"pool_limit_bytes":profile.pool_memory_bytes,"worker_bytes":profile.worker_bytes,"cpu_threads":profile.cpu_threads,"case_lanes":profile.case_lanes,"jobs":profile.math_jobs,"compiler_cores":profile.compiler_cores,"process_cap_bytes":allocation.native_worker_memory_bytes,"observer_cap_bytes":profile.observer_memory_bytes})
                } else { Value::Null };
                let cancel=audit::cancel_source();
                let database_setup=started.elapsed();
                let sources=support::sources(support::SOURCE);
                if audit::review_context_request().is_some() {
                    audit::retain_math_output("study-original-inputs",&json!({
                        "source_documents":{"physical":sources.physical,"modeling":sources.modeling},
                        "definition_status":"not yet admitted",
                        "workflow_context":{"status":"unsupported","reason":"original managed RunResult context bridge unavailable; actual source input bytes only"}
                    }));
                }
                phases.reset();
                let started=Instant::now();
                let ((physical,package,definition,recorded_definition),admission_counts)=pse_runtime::invocation::counted(async {
                let physical=support::physical(&runtime,&owner,&sources).await;
                let package=runtime.modeling_from_documents(&support::admitted_documents(&owner,&sources.modeling),physical.clone(),&cancel).await.unwrap();
                let definition=definition(&package,&physical,&sources,&values,independent).await;
                assert!(definition.points.iter().all(|point| point.operation.preparation.compiler.optimization.cores == 1),
                    "reference comparison retains one construction core per occurrence");
                let recorded_definition = if durable_mode || managed { Some(runtime.record_study(&definition, &cancel).await.unwrap()) } else { None };
                if audit::review_context_request().is_some()
                    && let Some(recorded) = &recorded_definition {
                    audit::retain_math_output("study-recorded-definition",&json!({"definition":recorded,
                        "workflow_context":{"status":"unsupported","reason":"recorded study definition is original input, not an original joined managed RunResult"}}));
                }
                (physical,package,definition,recorded_definition)
                }).await;
                let admission=started.elapsed();
                drop(setup_stage);
                let _execution_stage=audit::stage("study_dispatch_execution_and_results");
                let admission_phases=phases.report(occurrences as u64);
                let admission_constructions=phases.constructions();
                // Each adapter begins with source/definition admission complete;
                // the reference parallel selector has its own declared shared profile.
                phases.reset();
                let started=Instant::now();
                let mut worker_passes=Vec::new();
                let ((elapsed,submission,submission_counts,worker_counts,result_read_seconds,outcomes,metrics),execution_counts) = pse_runtime::invocation::counted(async { if durable_mode {
                    let start=Instant::now();
                    let (handle,submission_counts)=pse_runtime::invocation::counted(runtime.start_recorded_study(recorded_definition.clone().unwrap(),&cancel)).await;
                    let handle=handle.unwrap();
                    let submission=start.elapsed();
                    let submission_counts=serde_json::to_value(submission_counts).unwrap();
                    let mut worker_counts=pse_runtime::math::PreparationCounts::default();
                    let mut terminal=false;
                    for _ in 0..256 {
                        let start=Instant::now();
                        let (processed,pass_counts)=pse_runtime::invocation::counted(runtime.work_once()).await;
                        let processed=processed.unwrap();
                        worker_counts.views+=pass_counts.views; worker_counts.observations+=pass_counts.observations;
                        worker_counts.rebuilt+=pass_counts.rebuilt; worker_counts.shared+=pass_counts.shared;
                        let seconds=start.elapsed().as_secs_f64();
                        match processed {
                            workflow::Processed::Idle=> {
                                if handle.status().await.unwrap().state==StudyState::Concluded { terminal=true; break; }
                                tokio::time::sleep(Duration::from_millis(10)).await;
                            },
                            workflow::Processed::Ran {record,..}=> {
                                audit::direct_joined();
                                worker_passes.push(json!({"seconds":seconds,"outcome":record.attempt.as_ref().unwrap().outcome,"solutions":record.solutions.len(),"preparations":pass_counts}));
                            },
                            workflow::Processed::Maintenance{..}|workflow::Processed::Settled{..}|workflow::Processed::Finalized{..}=>worker_passes.push(json!({"seconds":seconds,"preparations":pass_counts})),
                        }
                        assert!(worker_passes.len()<=32,"unexpected study retry");
                        if handle.status().await.unwrap().state==StudyState::Concluded { terminal=true; break; }
                    }
                    let elapsed=started.elapsed();
                    let status=handle.status().await.unwrap();
                    if audit::review_context_request().is_some() {
                        match handle.diagnostic_selection().await {
                            Ok(document) => audit::retain_math_output("managed-study-diagnostics",&json!(document)),
                            Err(error) => audit::review_context_error(format!("stored study diagnostic selection: {error}")),
                        }
                    }
                    let selected_points=status.points.iter().filter(|point| point.settled && point.outcome.is_some())
                        .map(|point| (point.point_index,point.run.clone(),point.attempt.clone())).collect::<Vec<_>>();
                    export_study_results(&runtime,&selected_points).await;
                    audit::checkpoint();
                    assert!(terminal,"study did not conclude during the bounded serial drain");
                    let worker_counts=serde_json::to_value(worker_counts).unwrap();
                    let read=Instant::now();
                    assert_eq!(status.state,StudyState::Concluded,"{status:?}");
                    let outcomes:Vec<_>=status.points.into_iter().map(|p|p.outcome.unwrap()).collect();
                    check(&outcomes, occurrences, independent);
                    let retained=handle.result().await.unwrap().unwrap();
                    let metrics=persisted(&runtime,&owner,&retained.points,&mut numerical,&values,independent).await;
                    (elapsed,submission,submission_counts,worker_counts,read.elapsed().as_secs_f64(),outcomes,metrics)
                } else if managed {
                    // This is the ordinary public adapter: its production ensure-primary
                    // admission and complete retained study are both inside this clock.
                    // Keep one large public study future on the heap. Separate
                    // audit/ordinary awaits must not duplicate it in this harness frame.
                    let study=Box::pin(runtime.execute_study(recorded_definition.as_ref().unwrap(),occurrences,&cancel));
                    let report=if audit::enabled() {
                        study.await.unwrap()
                    } else {
                        tokio::time::timeout(Duration::from_secs(120),study).await.unwrap().unwrap()
                    };
                    audit::direct_joined();
                    let elapsed=started.elapsed();
                    if audit::review_context_request().is_some() {
                        match report.diagnostic_selection().await {
                            Ok(Some(document)) => audit::retain_math_output("managed-study-diagnostics",&json!(document)),
                            Ok(None) => audit::review_context_error("managed study returned no retained diagnostic selection"),
                            Err(error) => audit::review_context_error(format!("managed study diagnostic selection: {error}")),
                        }
                    }
                    let selected_points=report.results.iter().enumerate().filter_map(|(index,result)| {
                        let (run,attempt)=result.as_ref()?.stored_keys()?;
                        Some((index as u32,run.to_owned(),Some(attempt.to_owned())))
                    }).collect::<Vec<_>>();
                    export_study_results(&runtime,&selected_points).await;
                    audit::checkpoint();
                    check(&report.outcomes,occurrences,true);
                    assert_eq!(report.results.len(),occurrences);
                    assert_eq!(report.results.iter().map(|result| result.as_ref().unwrap().run_id()).collect::<BTreeSet<_>>().len(),occurrences);
                    let points=report.results.iter().enumerate().map(|(index,result)| {
                        let result=result.as_ref().unwrap();
                        let (run,attempt)=result.stored_keys().expect("public managed study retains original canonical result");
                        (index as u32,run.to_owned(),Some(attempt.to_owned()))
                    }).collect::<Vec<_>>();
                    let read=Instant::now();
                    let metrics=persisted(&runtime,&owner,&points,&mut numerical,&values,true).await;
                    (elapsed,Duration::ZERO,Value::Null,Value::Null,read.elapsed().as_secs_f64(),report.outcomes,metrics)
                } else {
                    let report=Box::pin(package.study(&definition,occurrences,&cancel)).await.unwrap();
                    audit::direct_joined();
                    let elapsed=started.elapsed();
                    let read=Instant::now();
                    let metrics=ephemeral(&report,&values,&mut numerical,independent,owner.runtime.math());
                    audit::checkpoint();
                    check(&report.outcomes, occurrences, independent);
                    (elapsed,Duration::ZERO,json!({"views":0,"observations":0,"rebuilt":0,"shared":0}),serde_json::to_value(report.preparations).unwrap(),read.elapsed().as_secs_f64(),report.outcomes,metrics)
                }}).await;
                numerical.counted_preparations(execution_counts);
                let execution_phases=phases.report(occurrences as u64);
                let peak=owner.runtime.observation_peak_bytes();
                let rss=owner.runtime.report().unwrap().process_peak_rss_bytes;
                let retained=pool.reserved();
                let record=json!({"runtime_profile":runtime_profile,"receiving_profile":receiving_profile,"seconds":{"database_setup":database_setup.as_secs_f64(),"source_and_definition_admission":admission.as_secs_f64(),"execution_total":elapsed.as_secs_f64(),"submission":submission.as_secs_f64(),"result_read":result_read_seconds},"admission_preparations":admission_counts,"submission_preparations":submission_counts,"worker_preparations":worker_counts,"execution_preparations":execution_counts,"admission_phases":admission_phases,"execution_phases":execution_phases,"worker_passes":worker_passes,"outcomes":outcomes,"point_metrics":metrics,"numerical_observations":numerical.json(),"definition":recorded_definition.map_or_else(|| json!({"modeling_revision":definition.modeling_revision,"points":definition.points,"variants":definition.variants}), |recorded| json!(recorded)),"pool_peak_bytes":peak,"process_peak_rss_bytes":rss,"retained_runtime_bytes":retained});
                drop(package); drop(physical);
                let mut record=record;
                record["admission_constructions"]=admission_constructions;
                record["execution_constructions"]=phases.constructions();
                (elapsed,record)
                })).await;
                let _drain_stage=audit::stage("managed_receiver_drain_and_fixture_teardown");
                let drain=Instant::now();
                if let Some(receiver)=&mut receiver
                    && let Err(error)=receiver.finish() {
                        // Preserve the original scientific failure when one already
                        // exists; the audit separately records failed receiver drain.
                        let _=std::io::Write::write_all(&mut std::io::stderr(),format!("{error}\n").as_bytes());
                        if let Err(panic)=result { std::panic::resume_unwind(panic); }
                        panic!("{error}");
                }
                let receiver_drain_seconds=drain.elapsed().as_secs_f64();
                drop(receiver);
                drop(runtime);
                let cleanup=owner.cleanup_fixtures().await;
                audit::cleanup_result(cleanup.is_ok());
                cleanup.unwrap();
                drop(owner);
                tokio::task::yield_now().await;
                let (elapsed,mut record)=match result { Ok(result)=>result, Err(panic)=>std::panic::resume_unwind(panic) };
                record["seconds"]["receiver_drain"]=receiver_drain_seconds.into();
                record["after_runtime_teardown_bytes"]=pool.reserved().into();
                assert_eq!(pool.reserved(), 0, "study allocations survive all runtime owners");
                (elapsed,record)
            })));
            let mut record=record;
            record["invocation_sequence"] = (records.len() as u64).into();
            record["criterion_membership"] = json!({"status":"unavailable","reason":"Criterion callback does not expose measured-sample membership"});
            invocation_records.push(record.clone());
            timed+=elapsed; records.push(record);
        }
        timed
    }));
    group.finish();
    invocation_records.flush();
    let maximum = |field: &str| records.iter().filter_map(|r| r[field].as_u64()).max();
    let record = json!({"id":name,"workload":spec,"iterations":records.len(),"occurrences":occurrences,"threads":threads,"native_threads":1,"runtime_profile":records.first().map(|r|&r["runtime_profile"]),"math_policy":records.first().map(|r|&r["runtime_profile"]["math_policy"]),"pool_limit_bytes":records.first().map(|r|&r["runtime_profile"]["pool_limit_bytes"]),"pool_peak_bytes":maximum("pool_peak_bytes"),"process_peak_rss_bytes":maximum("process_peak_rss_bytes"),"retained_runtime_bytes":maximum("retained_runtime_bytes"),"pool_metric_scope":if managed {"observer pool only; primary pool unobserved"} else {"shared calling-process pool"},"retained_runtime_scope":if managed {"observer pool after connected readers release; external primary and canonical server are unobserved"} else {"shared pool after occurrence reports or connected readers release; excludes external canonical server storage"},"after_case_teardown_bytes":maximum("after_runtime_teardown_bytes"),"after_retained_runtime_teardown_bytes":maximum("after_runtime_teardown_bytes"),"observer_threads":observer_threads,"receiving_profile":records.first().map(|r|&r["receiving_profile"]),"primary_process_peak_rss_bytes":null,"primary_pool_peak_bytes":null,"adapter":if managed {"public managed durable sixteen independent Ipopt occurrences"} else if durable_mode {"standalone durable serial work_once"} else if independent {"explicit ephemeral sixteen independent Ipopt occurrences; dispatch width is the declared threads; admitted automatic blocks retain their component-native reports"} else {"explicit ephemeral eight-point KINSOL continuation"},"cache_state":if managed {"fresh observer runtime and isolated canonical database per iteration; production primary launched and drained per iteration; source/definition admission precedes execution"} else {"fresh shared runtime per iteration; source/definition admission precedes execution"},"timed_scope":if managed {"public package.study production primary admission, canonical submission, external native execution/persistence and complete retained occurrence report; connected result reopening outside timer"} else if durable_mode {"canonical submission, serial work_once passes including per-point persistence and finalization"} else {"local admitted ephemeral study dispatch including preparation, native attempts and original joined reports; Arrow result projection outside timer"},"persistence_scope":if managed {"external primary canonical scientific retention; exact producing run/attempt reopened metrics and variables outside timer"} else if durable_mode {"canonical scientific retention in standalone worker clocks; exact reopened per-point metrics outside timer"} else {"ephemeral original reports and checked owned result tables; no canonical result retention"},"preparation_counter_scope":if managed {"observer process only; external primary preparation counters and native work are unobserved"} else {"actual task-local calling-operation scope; durable work_once runs in that same process; external worker scopes unobserved"}, "phase_scope":if managed {"observer admission, public dispatch and canonical reopening only; external primary phases unobserved"} else {"execution dispatch and post-dispatch table projection or canonical reopening; result reads are separately clocked"}, "rss_scope":if managed {"whole observer process lifetime high-water mark; includes admission and connected reads; excludes primary native work and canonical server"} else {"whole benchmark process lifetime high-water mark; includes setup, tables and native work, excludes canonical server"}, "acceptance":{"distinct_occurrence_results":occurrences,"original_root_checks":"x equals assignment and y equals3, absolute tolerance1e-7", "ordering":if independent {"independent fresh points, exact definition order in outcomes"} else {"original eight-point continuation chain"}},"sampling":"10 flat Criterion samples; smoke runs the same acceptance once without measurement; setup/admission and result projection or reopening outside timer","observations_file":format!("{name}-observations.jsonl")});
    std::fs::write(
        output.join(format!("{name}-memory.json")),
        serde_json::to_vec_pretty(&record).unwrap(),
    )
    .unwrap();
}
