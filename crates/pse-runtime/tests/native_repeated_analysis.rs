// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public repeated analysis of original protected scientific results and recorded points.
//! Requires the production native-service helper and an explicitly prepared module receipt.
#![cfg(all(feature = "native-study-tests", feature = "solver-kinsol"))]
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr,
    reason = "isolated integration fixture and independent scientific assertions"
)]

use datafusion::{arrow::array::{Array, FixedSizeBinaryArray, Float64Array, Int64Array}, prelude::{SessionContext, col}, logical_expr::JoinType};
use futures_util::{FutureExt, StreamExt, future::BoxFuture};
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_backend_native::{presolve::PolicyKind, solve::{Backend, SolveIntent}};
use pse_columnar::CancellationToken;
use pse_ids::{ContentHash, Frame, FramedHasher, SemanticId};
use pse_model::{scalars::FiniteBound, study::{OccurrenceKey, StartPolicy}};
use pse_operations::{
    canonical::CanonicalStore,
    native_service::{NativeCanonicalService, NativeServiceSpec},
};
use pse_relations::generated::runtime::{solve_runs, solve_variables};
use pse_runtime::{
    CancelSource, SharedRuntime,
    math::settings::SolveSettings,
    workflow::{
        BindingTarget, CanonicalDeployment, CaseOperation,
        DurabilitySelection, FrozenStudyCohort, LeasePolicy, ModelingPackage,
        OperationRequest, OperationSource, OuterAttestation, PointOverlay, PreparationSettings,
        PreparedOperation, PreparedStudyOperation, RetainedDataServices, Runtime,
        RuntimeOperationRequest, SourceDelta, StudyCohortAvailability, StudyCohortPageRequest,
        StudyCohortRelation, StudyCohortRequest, StudyCohortScanState, StudyDefinition,
        StudyHandle, StudyInputs, StudyKpiRequest, StudyOutputDemand, StudyPoint,
        StudyPointPolicy, StudyState, StudyStatus, StudyWorkbenchFilter, WorkerSettings,
        StudyObservationRequest, StudyObservationContinuity,
        WorkflowError,
        sampling::{DesignCoordinate, DesignKind, DesignRequest},
    },
};
use std::{
    collections::BTreeMap, num::NonZeroUsize, panic::AssertUnwindSafe,
    path::{Path, PathBuf}, sync::Arc, time::{Duration, Instant},
};

// Independent oracle: for the realized offsets 0 and 5, the positive roots are
// (2, 3) at p=4 and (3, sqrt(14)) after the authored calibration changes to p=9.
const SQUARE: &str = "package local { test square fixture { dof 0; } { param p:Scalar=4; param offset:Scalar=0; var x:Scalar; eq root:x*x==p+offset; annotation start x(1); } }";
// Authored identities deliberately differ from vector and captured-page positions.
const POINT_KEYS: [u32; 2] = [13, 29];

fn service() -> NativeCanonicalService {
    let environment = std::env::vars_os().collect();
    let helper = PathBuf::from(
        std::env::var_os("PSE_NATIVE_FIXTURE_SERVICE_HELPER")
            .expect("explicit already-built native-service helper"),
    );
    assert!(
        helper.is_absolute() && helper.is_file(),
        "build -p pse-operations --features native-service --bin pse-canonical-service first"
    );
    let module_source = if let Some(record) = std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_RECEIPT")
    {
        pse_operations::native_service::NativeModuleSource::Prepared(
            pse_host::study_module::select(
                &PathBuf::from(record),
                pse_host::study_module::Variant::Production,
            )
            .unwrap(),
        )
    } else {
        pse_operations::native_service::NativeModuleSource::RecordedState(PathBuf::from(
            std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_STATE")
                .expect("explicit private module receipt selector"),
        ))
    };
    let service = NativeCanonicalService::new(NativeServiceSpec {
        server: pse_host::server::select(&environment).unwrap(),
        helper,
        module_source,
        state_parent: std::env::temp_dir(),
    })
    .unwrap();
    eprintln!(
        "repeated-analysis private state={} original_service={}",
        service.state().display(),
        service.intent().unit()
    );
    service
}
fn texts(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, at: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, out);
            } else {
                out.insert(
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
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}
struct Science {
    shared: Arc<SharedRuntime>,
    runtimes: Vec<Runtime>,
    build: ContentHash,
    _spill: tempfile::TempDir,
}
impl Science {
    fn new(stores: &[CanonicalStore; 3]) -> Self {
        let spill = tempfile::tempdir().unwrap();
        let n = |v| NonZeroUsize::new(v).unwrap();
        let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
            memory_limit_bytes: n(4 << 30),
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
                worker_bytes: 1 << 30,
                workspace_bytes: 1 << 30,
                foreign_bytes: 16 << 20,
                jobs: std::thread::available_parallelism().unwrap().get(),
                ..Default::default()
            },
        })
        .unwrap();
        let mut hasher = FramedHasher::new(Frame::CanonicalPayloadV1);
        let mut executable = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
        hasher
            .part_reader(
                executable.metadata().unwrap().len(),
                &mut executable,
                &|| Ok(()),
            )
            .unwrap();
        let mut science = Self {
            shared,
            runtimes: vec![],
            build: hasher.finish_hash(),
            _spill: spill,
        };
        for (store, name) in stores
            .iter()
            .zip(["producer", "worker-left", "worker-right"])
        {
            science.attach(store, name);
        }
        science
    }
    fn attach(&mut self, store: &CanonicalStore, worker: &str) -> usize {
        let factory = Arc::new(
            self.shared
                .session_factory(pse_engine::session::native_engine_profile())
                .unwrap(),
        );
        let runtime = Runtime::from_shared(
            self.shared.clone(),
            factory
                .bind_model(pse_schema::shared_registry().unwrap())
                .unwrap(),
            CanonicalDeployment::new(
                store.clone(),
                OuterAttestation {
                    source: None,
                    build: self.build,
                },
                None,
            ),
        )
        .unwrap()
        .with_durability(DurabilitySelection::Durable {
            worker: worker.into(),
            policy: LeasePolicy::new(
                Duration::from_secs(30),
                Duration::from_millis(100),
                64,
                Duration::from_millis(20),
            )
            .unwrap(),
        })
        .unwrap();
        let index = self.runtimes.len();
        self.runtimes.push(runtime);
        index
    }
    fn close(&self) -> BoxFuture<'_, Result<(), pse_runtime::math::MathRuntimeError>> {
        Box::pin(async move {
            for runtime in &self.runtimes {
                runtime.close();
                runtime.clear_program_cache();
            }
            self.shared.math().shutdown_native_lanes().await
        })
    }
    fn package(&self) -> BoxFuture<'_, ModelingPackage> {
        Box::pin(async move {
            let runtime = &self.runtimes[0];
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
            let physical = texts(&root.join("physical-primitives"));
            let manifest=std::fs::read_to_string(root.join("minimal_explicit/package.toml")).unwrap()
            .replace("id_policy = \"explicit\"","id_policy = \"named\"")
            .replace("dependencies = []","dependencies = [{ package_id = \"5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", version_req = { operator = \"exact\", major = 1, minor = 0, patch = 0 } }]");
            let modeling = BTreeMap::from([
                ("package.toml".into(), manifest.into_bytes()),
                ("models/root.pse".into(), SQUARE.as_bytes().to_vec()),
            ]);
            let pool = self.shared.pool();
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
            let physical = runtime
                .physical_from_documents(&load(&physical), &token)
                .await
                .unwrap();
            runtime
                .modeling_from_documents(&load(&modeling), physical, &CancelSource::new())
                .await
                .unwrap()
        })
    }
}

fn definition<'a>(
    runtime: &'a Runtime,
    package: &'a ModelingPackage,
    overlays: &'a [PointOverlay],
) -> BoxFuture<'a, StudyDefinition> {
    Box::pin(async move {
        let cancel = CancelSource::new();
        let case = package.declarations(&runtime.preparation_services()).await.unwrap()
            .iter().find(|row| row.name == "square").unwrap().declaration_id;
    let inspect = package
        .prepare_operation(
            &runtime.preparation_services(),
            &OperationSource::of(&package),
            &PreparationSettings::default(),
            RuntimeOperationRequest::DeclaredCase {
                case,
                settings: Box::new(SolveSettings {
                    intent: SolveIntent::Root,
                    backend: Some(Backend::Kinsol),
                    presolve: PolicyKind::Off,
                    ..Default::default()
                }),
            },
            None,
            &cancel,
        )
        .await
        .unwrap();
    let PreparedOperation::Study(PreparedStudyOperation::DeclaredCase(inspect)) = inspect
    else {
        panic!("original declared square case");
    };
    let solved = inspect.model.model.solved();
    // This primitive fixture has no characteristic engineering scale.
    // Request tight physical budgets on its actual admitted coordinates.
    let requirements = inspect
        .solve
        .numerics()
        .targets
        .iter()
        .filter(|target| {
            matches!(
                target.kind,
                pse_model::generated::enums::NumericalTarget::Variable
                    | pse_model::generated::enums::NumericalTarget::Row
            )
        })
        .map(|target| pse_model::numerics::NumericalRequirement {
            requirement_id: pse_ids::named_id(
                case.as_id(),
                &format!("native-study-accuracy:{:?}:{}", target.kind, target.id),
            ),
            model_id: Some(solved.model()),
            case_id: solved.case(),
            instance_id: Some(solved.instance()),
            fit_id: None,
            target_id: target.id,
            target_kind: target.kind,
            nominal: None,
            scaling_factor: None,
            absolute_tolerance: Some(1e-10),
            relative_tolerance: Some(0.),
            shared_engineering_allowance: None,
            engineering_rule_id: None,
            unit_id: Some(target.unit),
            coordinates: pse_model::generated::enums::NumericalCoordinates::Physical,
            priority: 0,
            required: true,
            provenance: "repeated-analysis physical variable/equation verification".into(),
        })
        .collect::<Vec<_>>();
    assert!(requirements.iter().any(
        |row| row.target_kind == pse_model::generated::enums::NumericalTarget::Variable
    ));
    assert!(requirements.iter().any(|row|row.target_kind==pse_model::generated::enums::NumericalTarget::Row));
    drop(inspect);
        let points = overlays.iter().enumerate().map(|(key, overlay)| StudyPoint {
            operation: OperationRequest::DeclaredCase(CaseOperation {
                case,
                route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                settings: SolveSettings {
                    intent: SolveIntent::Root,
                    backend: Some(Backend::Kinsol),
                    presolve: PolicyKind::Off,
                    numerics: pse_model::numerics::NumericalPolicy {
                        requirements: requirements.clone(),
                        engineering_relative_fraction: 1e-10,
                        supplier_action_accuracy: 1e-10,
                        linear_backward_error: 1e-10,
                        kkt: pse_model::numerics::KktTolerances {
                            stationarity: 1e-10, complementarity: 1e-10,
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
            preparation: PreparationSettings::default(),
            overlay: overlay.clone(),
            policy: StudyPointPolicy {
                key: OccurrenceKey(POINT_KEYS[key]),
                dependencies: vec![],
                start: StartPolicy::Fresh,
                attempt_limit: 1,
            },
        }).collect();
        let services = runtime.execution_services();
        let inputs = StudyInputs::Explicit(points);
        let local = Box::pin(package.admit_study(
            &services, &inputs, &StudyOutputDemand::Full {}, &cancel,
        )).await.unwrap();
        let recorded = Box::pin(runtime.record_study(&local, &cancel)).await.unwrap();
        assert_eq!(recorded.context.as_ref(), package.project_context());
        assert_eq!(recorded.points.len(), 2);
        for (key, point) in recorded.points.iter().enumerate() {
            assert_eq!(point.policy.key, OccurrenceKey(POINT_KEYS[key]));
            assert_eq!(point.operation.source, OperationSource::of(package));
            assert_eq!(point.binding_hash, point.binding.identity());
            assert_eq!(point.binding.entries.len(), 1);
            let entry = point.binding.entries.values().next().unwrap();
            assert!(entry.parameter);
            assert_eq!(entry.supplied, BindingTarget::Path("offset".into()));
            assert_eq!(entry.canonical.into_inner(), [0., 5.][key]);
        }
        recorded
    })
}

// This reader is constructed from an actual bound engine model and its own
// SchemaReady connection. It never constructs preparation or math services.
struct Data(RetainedDataServices);
impl Data {
    fn new(store: &CanonicalStore) -> Self {
        let pool: Arc<dyn datafusion::execution::memory_pool::MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(8 << 30),
        );
        let environment = Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default());
        let factory = Arc::new(pse_engine::session::EngineFactory::new(
            environment,
            pool,
            Default::default(),
            pse_engine::ThreadBudget {
                pool_threads: NonZeroUsize::new(16).unwrap(),
                target_partitions: NonZeroUsize::new(16).unwrap(),
            },
            pse_engine::session::native_engine_profile(),
        ).unwrap());
        Self(RetainedDataServices::new(
            store.clone(), factory.bind_model(pse_schema::shared_registry().unwrap()).unwrap(),
        ).unwrap())
    }
}
impl Drop for Data {
    fn drop(&mut self) {
        self.0.close();
    }
}

struct StopWorkers([CancelSource; 2]);
impl Drop for StopWorkers {
    fn drop(&mut self) {
        for stop in &self.0 {
            stop.cancel();
        }
    }
}
fn serve<'a>(runtime: &'a Runtime, stop: &'a CancelSource)
    -> BoxFuture<'a, Result<usize, WorkflowError>>
{
    Box::pin(runtime.serve(WorkerSettings {
        poll: Duration::from_millis(20),
        recovery: Duration::from_secs(1),
        maximum_actions: None,
        maximum_in_flight: Some(1),
        until_idle: true,
    }, stop))
}
async fn execute(science: &Science, workers: [usize; 2], handle: &StudyHandle) -> StudyStatus {
    let stops = StopWorkers([CancelSource::new(), CancelSource::new()]);
    let work = async {
        tokio::join!(
            serve(&science.runtimes[workers[0]], &stops.0[0]),
            serve(&science.runtimes[workers[1]], &stops.0[1]),
        )
    };
    tokio::pin!(work);
    let (left, right) = tokio::select! {
        result = &mut work => result,
        () = tokio::time::sleep(Duration::from_secs(120)) => {
            for stop in &stops.0 { stop.cancel(); }
            let drained = work.await;
            panic!("repeated-analysis workers exceeded allowance; drain={drained:?}");
        }
    };
    assert!(left.is_ok() && right.is_ok(), "left={left:?}; right={right:?}");
    assert!(left.unwrap() + right.unwrap() > 0);
    let status = handle.status().await.unwrap();
    assert_eq!(status.state, StudyState::Concluded);
    assert_eq!(status.points.len(), 2);
    for point in &status.points {
        assert!(point.settled);
        let outcome = point.outcome.as_ref().unwrap();
        assert!(outcome.scientific.usable, "{outcome:?}");
        assert_eq!(outcome.attempts.len(), 1);
        let store = science.runtimes[workers[0]].canonical_store().unwrap();
        let run = store.canonical_run(&point.run).await.unwrap().unwrap();
        let attempt = store.canonical_attempt(point.attempt.as_ref().unwrap()).await.unwrap().unwrap();
        assert_eq!(attempt.run, run.key);
        assert_eq!(attempt.phase, pse_model::generated::enums::CanonicalAttemptPhase::Terminal);
    }
    status
}

fn request(definition: &StudyDefinition, study: &str, qualified: bool) -> StudyCohortRequest {
    StudyCohortRequest {
        filter: StudyWorkbenchFilter {
            study: Some(study.into()),
            source: Some(definition.modeling_revision.clone()),
            physical_revision: Some(definition.physical.revision.clone()),
            context: Some(definition.context.as_ref().unwrap().context.key.clone()),
            outcome: qualified.then_some(
                pse_model::generated::enums::CanonicalTerminalOutcome::Succeeded,
            ),
            ..Default::default()
        },
        maximum_studies: 1,
        maximum_members: 2,
    }
}

// Preserve full original compact facts, including scientific qualification and
// exact attempt/manifest identities; a floating live workbench is never the cut.
async fn captured(cohort: &FrozenStudyCohort, qualified: bool) -> serde_json::Value {
    assert!(cohort.receipt().complete);
    assert_eq!(cohort.receipt().header_count, 1);
    assert_eq!(cohort.receipt().member_count, 2);
    let headers = cohort.header_page(StudyCohortPageRequest { maximum: 1, cursor: None })
        .await.unwrap();
    assert!(headers.next.is_none());
    assert_eq!(headers.headers[0].value.terminal, qualified);
    let first = cohort.member_page(StudyCohortPageRequest { maximum: 1, cursor: None })
        .await.unwrap();
    let second = cohort.member_page(StudyCohortPageRequest {
        maximum: 1, cursor: Some(first.next.clone().unwrap()),
    }).await.unwrap();
    assert!(second.next.is_none());
    let members = [&first.members[0], &second.members[0]];
    for (position, row) in members.iter().enumerate() {
        assert_eq!(row.position, position as u64);
        assert_eq!(row.value.ordinal, position as u64);
        assert_eq!(row.value.key, pse_operations::canonical_studies::point_key(
            &row.value.study, OccurrenceKey(POINT_KEYS[position]),
        ));
        assert_eq!(row.value.settled, qualified);
        assert_eq!(row.value.facts.scientific.usable, qualified);
        assert_eq!(row.value.facts.attempt_count, u32::from(qualified));
        assert_eq!(row.value.facts.native_started, qualified);
        assert_eq!(row.value.manifest.is_some(), qualified);
        assert_eq!(row.value.attempt.is_some(), qualified);
    }
    serde_json::json!({ "receipt": cohort.receipt(), "headers": headers.headers, "members": members })
}

async fn assert_captured_attempts(cohort: &FrozenStudyCohort, status: &StudyStatus) {
    let page = cohort.member_page(StudyCohortPageRequest { maximum: 2, cursor: None })
        .await.unwrap();
    assert!(page.next.is_none());
    assert_eq!(page.members.len(), status.points.len());
    for captured in &page.members {
        let point = status.points.iter()
            .find(|point| pse_operations::canonical_studies::point_key(
                &captured.value.study, OccurrenceKey(point.point_index),
            ) == captured.value.key)
            .expect("captured original occurrence");
        assert_eq!(captured.value.run, point.run);
        assert_eq!(captured.value.attempt, point.attempt);
        assert!(captured.value.manifest.is_some());
        assert_eq!(captured.value.facts.scientific, point.outcome.as_ref().unwrap().scientific);
    }
}

#[derive(Debug, PartialEq)]
struct Coordinates {
    variable: SemanticId,
    offset: SemanticId,
    partition: String,
    quantity: SemanticId,
    unit: SemanticId,
}
async fn science_rows(
    data: &RetainedDataServices,
    status: &StudyStatus,
    definition: &StudyDefinition,
    p: f64,
) -> (Coordinates, BTreeMap<u32, Vec<solve_variables::Row>>) {
    let mut coordinates = None;
    let mut retained = BTreeMap::new();
    for point in &status.points {
        let position = POINT_KEYS.iter().position(|key| *key == point.point_index)
            .expect("original authored occurrence identity");
        let offset = [0., 5.][position];
        let root = if p == 4. { [2., 3.][position] }
            else { [3., 14_f64.sqrt()][position] };
        let attempt = point.attempt.as_ref().unwrap();
        let run = data.canonical_store().unwrap().canonical_run(&point.run).await.unwrap().unwrap();
        assert_eq!(run.revision, definition.modeling_revision);
        let mut reader = data.results(
            &point.run, attempt, "runtime.solve_variables", 0, u64::MAX, CancellationToken::new(),
        ).await.unwrap();
        let mut rows = vec![];
        while let Some(batch) = reader.next_batch().await.unwrap() {
            let view = solve_variables::View::try_from_batch_with_registry(
                data.registry(), &batch, &data.validation_context(),
            ).unwrap();
            rows.extend(view.rows().unwrap());
        }
        reader.close();
        let variables = rows.iter().filter(|row| !row.parameter && !row.fixed).collect::<Vec<_>>();
        assert_eq!(variables.len(), 1);
        let variable = variables[0];
        let x = variable.value.unwrap();
        assert!((x - root).abs() <= 1e-7, "independent root {root}, actual {x}");
        assert!((x * x - (p + offset)).abs() <= 1e-6, "original residual at realized offset {offset}");
        let mut parameters = rows.iter().filter(|row| row.parameter).map(|row| row.value.unwrap())
            .collect::<Vec<_>>();
        parameters.sort_by(f64::total_cmp);
        let mut expected = vec![p, offset];
        expected.sort_by(f64::total_cmp);
        assert_eq!(parameters, expected, "original authored parameter and realized offset");
        let offset_row = rows.iter().find(|row| row.parameter && row.value == Some(offset)).unwrap();
        let actual = Coordinates {
            variable: variable.symbol_id,
            offset: offset_row.symbol_id,
            partition: format!("step:{}", variable.step),
            quantity: variable.quantity_id.expect("physically typed solution"),
            unit: variable.unit_id.expect("physical representation"),
        };
        if let Some(previous) = &coordinates { assert_eq!(&actual, previous); }
        else { coordinates = Some(actual); }
        let mut runs = data.results(
            &point.run, attempt, "runtime.solve_runs", 0, u64::MAX, CancellationToken::new(),
        ).await.unwrap();
        let mut count = 0;
        while let Some(batch) = runs.next_batch().await.unwrap() {
            let view = solve_runs::View::try_from_batch_with_registry(
                data.registry(), &batch, &data.validation_context(),
            ).unwrap();
            for row in view.rows().unwrap() {
                assert_eq!(row.backend, Some(pse_model::generated::enums::NativeBackend::Kinsol));
                assert_eq!(row.revision, Some(definition.points[position].operation.source.revision.as_id()));
                count += 1;
            }
        }
        assert_eq!(count, 1);
        runs.close();
        retained.insert(point.point_index, rows);
    }
    (coordinates.unwrap(), retained)
}

fn kpi(coordinates: &Coordinates, owner: SemanticId) -> StudyKpiRequest {
    StudyKpiRequest {
        relation: "runtime.solve_variables".into(), owner, field: "value".into(),
        partition: coordinates.partition.clone(), start: 0, end: u64::MAX,
    }
}
async fn relation(cohort: &FrozenStudyCohort, coordinates: &Coordinates, owner: SemanticId)
    -> StudyCohortRelation
{
    cohort.relation(kpi(coordinates, owner), CancellationToken::new()).await.unwrap()
}

async fn assert_aggregate(
    cohort: &FrozenStudyCohort,
    coordinates: &Coordinates,
    expected_roots: Option<[f64; 2]>,
) {
    let analysis = cohort.aggregate(
        &kpi(coordinates, coordinates.variable), CancellationToken::new(),
    ).await.unwrap();
    let available = expected_roots.is_some();
    assert_eq!((analysis.included, analysis.unavailable), if available { (2, 0) } else { (0, 2) });
    let progress = analysis.progress();
    assert_eq!(progress.state, StudyCohortScanState::Complete);
    assert_eq!((progress.active_scans, progress.pending), (0, 0));
    assert_eq!(analysis.cohort.capture, cohort.receipt().capture);
    let mut rows = 0;
    for batch in &analysis.batches {
        for row in 0..batch.num_rows() {
            let roots = expected_roots.expect("unavailable cohort has no numerical aggregate row");
            for (field, expected) in [
                ("quantity_id", coordinates.quantity), ("unit_id", coordinates.unit),
            ] {
                let values = batch.column_by_name(field).unwrap().as_any()
                    .downcast_ref::<FixedSizeBinaryArray>().unwrap();
                assert!(!values.is_null(row));
                assert_eq!(values.value(row), expected.as_bytes());
            }
            let count = batch.column_by_name("count").unwrap().as_any()
                .downcast_ref::<Int64Array>().unwrap();
            assert!(!count.is_null(row));
            assert_eq!(count.value(row), 2);
            for (field, expected) in [
                ("minimum", roots[0]), ("maximum", roots[1]),
                ("mean", (roots[0] + roots[1]) / 2.),
            ] {
                let values = batch.column_by_name(field).unwrap().as_any()
                    .downcast_ref::<Float64Array>().unwrap();
                assert!(!values.is_null(row));
                assert!((values.value(row) - expected).abs() <= 1e-7);
            }
            rows += 1;
        }
    }
    assert_eq!(rows, usize::from(available));
    let classified = analysis.classification_page(StudyCohortPageRequest { maximum: 2, cursor: None })
        .await.unwrap();
    assert!(classified.next.is_none());
    assert_eq!(classified.members.len(), 2);
    assert!(classified.members.iter().all(|member|
        member.value.availability == if available {
            StudyCohortAvailability::Included
        } else {
            StudyCohortAvailability::CapturedUnavailable
        }
    ));
}

async fn assert_wrong_pool_refused(cohort: &FrozenStudyCohort, coordinates: &Coordinates) {
    let relation = relation(cohort, coordinates, coordinates.variable).await;
    let unrelated = SessionContext::new();
    assert!(!Arc::ptr_eq(
        &unrelated.runtime_env().memory_pool,
        &relation.session_state().runtime_env().memory_pool,
    ));
    let frame = unrelated.read_table(relation.table_provider()).unwrap();
    let error = match frame.execute_stream().await {
        Err(error) => error,
        Ok(_) => panic!("an unrelated deployment pool acquired a protected cohort scan"),
    };
    assert!(error.to_string().contains("cohort query uses another deployment pool"), "{error}");
    let progress = relation.progress();
    assert_eq!(progress.state, StudyCohortScanState::Pending);
    assert_eq!((progress.active_scans, progress.included, progress.pending), (0, 0, 2));
}

async fn assert_passive_observation(data: &RetainedDataServices, study: &str) {
    let request = StudyObservationRequest {
        study: study.into(), maximum_members: 2, maximum_events: 512,
    };
    let mut observer = data.observe_study(request.clone(), None).await.unwrap();
    let page = observer.poll().await.unwrap();
    assert_eq!(page.status.header.study, study);
    assert!(page.status.header.terminal && !page.status.header.cancelled);
    assert_eq!(page.status.members.members.len(), 2);
    let parent = page.sources.iter().find(|source|
        source.position.run == page.cursor.context.run
    ).expect("original parent source");
    assert_eq!(parent.continuity, StudyObservationContinuity::HistoryUnavailable);
    assert!(parent.position.history_complete && parent.events.is_empty());
    assert!(matches!(parent.coverage, Some(pse_runtime::workflow::ProgressCoverage::Unknown {})));
    let cursor = observer.cursor();
    observer.close();
    assert!(observer.poll().await.is_err());

    let mut foreign = cursor.clone();
    foreign.issuing_store.push_str("-foreign");
    assert!(data.observe_study(request.clone(), Some(foreign)).await.is_err());
    let mut wrong = cursor.clone();
    wrong.context.source.push_str("-other");
    assert!(data.observe_study(request.clone(), Some(wrong)).await.is_err());
    let mut reconnected = data.observe_study(request, Some(cursor.clone())).await.unwrap();
    let continued = reconnected.poll().await.unwrap();
    assert_eq!(continued.cursor.context, cursor.context);
    assert!(!continued.status.header.cancelled);
    reconnected.observation_token().cancel();
    assert!(reconnected.follow(Duration::ZERO).await.is_err());
    reconnected.close();
    // Observation close/cancellation never changes the governing scientific state.
    let mut current = data.observe_study(cursor.request, None).await.unwrap();
    assert!(!current.poll().await.unwrap().status.header.cancelled);
    current.close();
}

async fn assert_captured_exports(
    data: &RetainedDataServices,
    cohort: &FrozenStudyCohort,
    definition: &StudyDefinition,
    study: &str,
    coordinates: &Coordinates,
    expected: &[solve_variables::Row],
) {
    let output = tempfile::tempdir().unwrap();
    let refused = output.path().join("refused.ndjson");
    assert!(cohort.export_pages(&refused, 1, &CancellationToken::new()).await.is_err());
    assert!(!refused.exists());
    assert!(output.path().join("refused.ndjson.incomplete").is_file());
    let pages = output.path().join("cohort.ndjson");
    let bytes = cohort.export_pages(&pages, 1 << 20, &CancellationToken::new()).await.unwrap();
    assert_eq!(bytes, std::fs::metadata(&pages).unwrap().len());
    let records = std::fs::read_to_string(pages).unwrap().lines()
        .map(|line| serde_json::from_str::<pse_runtime::workflow::StudyCohortExportRecord>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(matches!(records.last(), Some(pse_runtime::workflow::StudyCohortExportRecord::Receipt(receipt))
        if receipt.capture == cohort.receipt().capture && receipt.member_count == 2));
    assert_eq!(records.iter().map(|record| match record {
        pse_runtime::workflow::StudyCohortExportRecord::Members(page) => page.members.len(),
        _ => 0,
    }).sum::<usize>(), 2);

    let detail = output.path().join("detail.arrow");
    let mut reader = cohort.result(0, "runtime.solve_variables", 0, u64::MAX, CancellationToken::new())
        .await.unwrap();
    let original_run = reader.selection().run().key.clone();
    let original_attempt = reader.selection().attempt().key.clone();
    let original_manifest = reader.selection().manifest().digest.clone();
    reader.export_ipc_bounded(&detail, 1 << 20, &CancellationToken::new()).await.unwrap();
    reader.close();
    let stream = datafusion::arrow::ipc::reader::StreamReader::try_new(std::fs::File::open(detail).unwrap(), None).unwrap();
    let mut position = 0;
    for batch in stream {
        let batch = batch.unwrap();
        // Portable export adds its own canonical attribution metadata. It is
        // not a raw registry relation and must not be admitted as one.
        let schema = batch.schema();
        for (key, expected) in [
            ("pse.canonical.run", original_run.as_str()),
            ("pse.canonical.attempt", original_attempt.as_str()),
            ("pse.canonical.manifest", original_manifest.as_str()),
            ("pse.canonical.relation", "runtime.solve_variables"),
        ] {
            assert_eq!(schema.metadata().get(key).map(String::as_str), Some(expected));
        }
        let end = position + batch.num_rows();
        let mut builder = solve_variables::Builder::new(&data.validation_context()).unwrap();
        for row in expected.get(position..end).expect("export cannot add original rows") {
            builder.push(row.clone()).unwrap();
        }
        let original = builder.finish().unwrap();
        assert_eq!(schema.fields(), original.batch().schema().fields());
        assert_eq!(batch.columns(), original.batch().columns(),
            "export retains exact original variables, tolerances and provenance");
        position = end;
    }
    assert_eq!(position, expected.len());

    let analysis = output.path().join("aggregate.arrow");
    let exported = data.export_study_analysis(request(definition, study, true),
        &kpi(coordinates, coordinates.variable), &analysis, &CancellationToken::new()).await.unwrap();
    assert_eq!((exported.included, exported.unavailable), (2, 0));
    assert_eq!(exported.output.as_ref(), Some(&analysis));
    let file = datafusion::arrow::ipc::reader::FileReader::try_new(std::fs::File::open(analysis).unwrap(), None).unwrap();
    let mut rows = 0;
    for batch in file {
        let batch = batch.unwrap();
        for row in 0..batch.num_rows() {
            let mean = batch.column_by_name("mean").unwrap().as_any().downcast_ref::<Float64Array>().unwrap();
            let count = batch.column_by_name("count").unwrap().as_any().downcast_ref::<Int64Array>().unwrap();
            assert!(!mean.is_null(row) && !count.is_null(row));
            assert_eq!(count.value(row), 2);
            assert!((mean.value(row) - 2.5).abs() <= 1e-7);
            for (field, expected) in [("quantity_id", coordinates.quantity), ("unit_id", coordinates.unit)] {
                let ids = batch.column_by_name(field).unwrap().as_any().downcast_ref::<FixedSizeBinaryArray>().unwrap();
                assert!(!ids.is_null(row));
                assert_eq!(ids.value(row), expected.as_bytes());
            }
            rows += 1;
        }
    }
    assert_eq!(rows, 1);
}

async fn compare(
    original: &FrozenStudyCohort,
    corrected: &FrozenStudyCohort,
    coordinates: &Coordinates,
) {
    let original_x = relation(original, coordinates, coordinates.variable).await;
    let original_offset = relation(original, coordinates, coordinates.offset).await;
    let corrected_x = relation(corrected, coordinates, coordinates.variable).await;
    let corrected_offset = relation(corrected, coordinates, coordinates.offset).await;
    // Preserve each provider's configured engine state, functions and deployment pool.
    let context = SessionContext::new_with_state(original_x.session_state());
    for relation in [&original_x, &original_offset, &corrected_x, &corrected_offset] {
        assert!(Arc::ptr_eq(
            &context.runtime_env().memory_pool,
            &relation.session_state().runtime_env().memory_pool,
        ));
    }
    // Match realized inputs inside the native query. Neither completion order nor
    // a client-side numeric hydration loop determines which observations compare.
    // Native states intentionally have no mutable default catalog. Direct
    // DataFrame composition needs neither catalog registration nor a fresh state.
    let ox = context.read_table(original_x.table_provider()).unwrap().alias("ox").unwrap();
    let om = context.read_table(original_offset.table_provider()).unwrap().alias("om").unwrap();
    let cm = context.read_table(corrected_offset.table_provider()).unwrap().alias("cm").unwrap();
    let cx = context.read_table(corrected_x.table_provider()).unwrap().alias("cx").unwrap();
    let query = ox.join_on(om, JoinType::Inner, [
        col("ox.run_id").eq(col("om.run_id")), col("ox.step").eq(col("om.step")),
    ]).unwrap().join_on(cm, JoinType::Inner, [
        col("om.value").eq(col("cm.value")),
        col("om.quantity_id").eq(col("cm.quantity_id")),
        col("om.unit_id").eq(col("cm.unit_id")),
    ]).unwrap().join_on(cx, JoinType::Inner, [
        col("cx.run_id").eq(col("cm.run_id")), col("cx.step").eq(col("cm.step")),
        col("cx.symbol_id").eq(col("ox.symbol_id")),
        col("cx.quantity_id").eq(col("ox.quantity_id")),
        col("cx.unit_id").eq(col("ox.unit_id")),
    ]).unwrap().select(vec![
        col("om.value").alias("offset_value"), col("ox.value").alias("original_root"),
        col("cx.value").alias("corrected_root"),
        (col("cx.value") - col("ox.value")).alias("change"),
    ]).unwrap().sort(vec![col("offset_value").sort(true, true)]).unwrap();
    let mut stream = query.execute_stream().await.unwrap();
    let expected = [[0., 2., 3., 1.], [5., 3., 14_f64.sqrt(), 14_f64.sqrt()-3.]];
    let mut count = 0;
    while let Some(batch) = stream.next().await {
        let batch = batch.unwrap();
        for row in 0..batch.num_rows() {
            assert!(count < expected.len());
            for (column, expected) in expected[count].iter().enumerate() {
                let values = batch.column(column).as_any().downcast_ref::<Float64Array>().unwrap();
                assert!(!values.is_null(row));
                assert!((values.value(row)-expected).abs() <= 1e-7);
            }
            count += 1;
        }
    }
    assert_eq!(count, 2, "two independently specified comparison rows");
    for selected in [&original_x, &original_offset, &corrected_x, &corrected_offset] {
        let progress = selected.progress();
        assert_eq!(progress.state, StudyCohortScanState::Complete);
        assert_eq!((progress.included, progress.unavailable, progress.pending), (2, 0, 0));
        assert_eq!(progress.active_scans, 0);
        let classified = selected.classification_page(StudyCohortPageRequest { maximum: 2, cursor: None })
            .await.unwrap();
        assert!(classified.next.is_none());
        assert_eq!(classified.members.len(), 2);
        assert!(classified.members.iter().all(|row|
            row.value.availability == StudyCohortAvailability::Included
            && row.value.member.facts.scientific.usable
        ));
    }
}

async fn assert_no_observations(
    cohort: &FrozenStudyCohort,
    coordinates: &Coordinates,
    empty: bool,
) {
    let mut demand = kpi(coordinates, coordinates.variable);
    if empty { demand.end = 0; }
    let relation = cohort.relation(demand, CancellationToken::new()).await.unwrap();
    let context = SessionContext::new_with_state(relation.session_state());
    let mut stream = context.read_table(relation.table_provider()).unwrap()
        .execute_stream().await.unwrap();
    let mut rows = 0;
    while let Some(batch) = stream.next().await { rows += batch.unwrap().num_rows(); }
    assert_eq!(rows, 0, "unavailable/empty is never a zero-valued scientific observation");
    let classified = relation.classification_page(StudyCohortPageRequest { maximum: 2, cursor: None })
        .await.unwrap();
    assert_eq!(classified.progress.state, StudyCohortScanState::Complete);
    assert_eq!((classified.progress.included, classified.progress.unavailable, classified.progress.pending), (0, 2, 0));
    assert_eq!(classified.members.len(), 2);
    for member in &classified.members {
        assert_eq!(member.value.availability, if empty {
            StudyCohortAvailability::EmptySelection
        } else {
            StudyCohortAvailability::CapturedUnavailable
        });
        assert_eq!(member.value.member.facts.scientific.usable, empty);
    }
}

fn journey<'a>(science: &'a mut Science, service: &'a mut NativeCanonicalService)
    -> BoxFuture<'a, ()>
{
    Box::pin(async move {
        let cancel = CancelSource::new();
        let store = service.ready().unwrap().store().clone();
        let read_ready = service.connect().await.unwrap();
        let data = Data::new(read_ready.store());
        let package = science.package().await;
        let runtime = &science.runtimes[0];
        store.create_project("create-analysis-project", "analysis-project", "Repeated native analysis")
            .await.unwrap();
        let project = store.project("analysis-project").await.unwrap().unwrap();

        // These are domain effective instants and explicit journal cuts. No
        // storage timestamp or MVCC snapshot participates in the selection.
        Box::pin(runtime.project_record_context(
            "original-calibration", &project, "calibration", &package, 0, None, None, false,
        )).await.unwrap();
        let original_cut = store.project_context_cut(&project.key).await.unwrap();
        let original_context = store.select_project_context(&original_cut, "calibration", 20)
            .await.unwrap().unwrap();
        let selected = Box::pin(runtime.project_context_reopen(&original_context, &cancel))
            .await.unwrap();
        let declarations = selected.declarations(&runtime.preparation_services()).await.unwrap();
        let case = declarations.iter().find(|row| row.name == "square").unwrap().declaration_id;
        let quantity = selected.physical_context().resolve_quantity("Scalar").unwrap();
        let unit = selected.physical_context().quantities().quantity_type(quantity).unwrap().canonical_unit;
        let preparation = runtime.preparation_services();
        let generated = Box::pin(selected.generate_design(
            &preparation,
            &DesignRequest {
                root: case, bindings: Default::default(),
                coordinates: vec![DesignCoordinate {
                    target: BindingTarget::Path("offset".into()), quantity, unit,
                    range: Some([FiniteBound::try_new(0.).unwrap(), FiniteBound::try_new(5.).unwrap()]),
                    levels: vec![],
                }],
                kind: DesignKind::Grid, count: 2, seed: 91, grid_levels: vec![2], maximum_work: 1000,
            },
            Default::default(), &cancel,
        )).await.unwrap();
        assert_eq!(generated.attribution().kind, DesignKind::Grid);
        assert_eq!(generated.attribution().count, 2);
        let overlays = generated.points().to_vec();
        assert_eq!(overlays.iter().map(|overlay|
            overlay.assignments[0].value.magnitude.into_inner()
        ).collect::<Vec<_>>(), vec![0., 5.]);
        let original_definition = definition(runtime, &selected, &overlays).await;
        let encoded = serde_json::to_string(&original_definition).unwrap();
        let realized = serde_json::to_value(&original_definition.points).unwrap();
        let original_handle = Box::pin(runtime.start_recorded_study(original_definition.clone(), &cancel))
            .await.unwrap();
        let original_study = original_handle.study_id().to_string();
        // Generation is over. Execution and replay receive only admitted recorded
        // coordinates, never a seed or access to the original design producer.
        drop(generated);
        drop(preparation);
        drop(declarations);
        drop(selected);
        drop(package);

        let pending = data.0.freeze_study_cohort(request(&original_definition, &original_study, false))
            .await.unwrap();
        let pending_before = captured(&pending, false).await;
        let mut passive = data.0.observe_study(StudyObservationRequest {
            study: original_study.clone(), maximum_members: 2, maximum_events: 8,
        }, None).await.unwrap();
        assert!(!passive.poll().await.unwrap().status.header.cancelled);
        passive.close();
        assert!(!original_handle.status().await.unwrap().cancelled);
        let original_status = execute(science, [1, 2], &original_handle).await;
        let original = data.0.freeze_study_cohort(request(&original_definition, &original_study, true))
            .await.unwrap();
        let frozen_before = captured(&original, true).await;
        assert_captured_attempts(&original, &original_status).await;
        let (coordinates, original_rows) = science_rows(&data.0, &original_status, &original_definition, 4.).await;
        assert_eq!(captured(&pending, false).await, pending_before);
        assert_no_observations(&pending, &coordinates, false).await;
        assert_no_observations(&original, &coordinates, true).await;

        // All three original logical callers close before the replacement
        // producer and competing workers attach through independent SchemaReady
        // connections. Data/capture owners do not inherit their admission.
        for runtime in &science.runtimes {
            runtime.close();
            runtime.clear_program_cache();
        }
        assert!(original_handle.status().await.is_err());
        let producer_ready = service.connect().await.unwrap();
        let left_ready = service.connect().await.unwrap();
        let right_ready = service.connect().await.unwrap();
        let producer = science.attach(producer_ready.store(), "later-producer");
        let left = science.attach(left_ready.store(), "later-worker-left");
        let right = science.attach(right_ready.store(), "later-worker-right");
        let runtime = &science.runtimes[producer];
        let reopened = runtime.study(original_handle.study_id());
        let original_reopened = reopened.status().await.unwrap();
        assert_eq!(serde_json::to_value(&original_reopened).unwrap(), serde_json::to_value(&original_status).unwrap());
        let historical = Box::pin(runtime.project_context_reopen(&original_context, &cancel))
            .await.unwrap();
        let declarations = historical.declarations(&runtime.preparation_services()).await.unwrap();
        let mut replacement = declarations.iter().find(|row| row.name == "p").unwrap().clone();
        replacement.value.binding.as_mut().unwrap().expression = Some("9".into());
        let edited = Box::pin(historical.with_delta(
            &runtime.source_services(),
            SourceDelta {
                parent: historical.revision_id().into(), replacements: vec![replacement],
                ..Default::default()
            }, &cancel,
        )).await.unwrap();
        assert_ne!(OperationSource::of(&historical).revision, OperationSource::of(&edited).revision);
        let edited_revision = edited.canonical_revision().expect("durably published source correction");
        assert_ne!(edited_revision.key, original_context.context.revision);
        assert_eq!(store.revision(&edited_revision.key).await.unwrap().as_ref(), Some(edited_revision));
        assert_eq!(historical.physical_context().identity(), edited.physical_context().identity());
        assert!(edited.project_context().is_none(), "a source edit cannot inherit historical context");
        // A real immutable engineering-context correction points at the edited
        // calibration source. Physical definitions and their interpretation stay
        // identical; this is not a correction to the physical type system.
        Box::pin(runtime.project_record_context(
            "corrected-calibration", &project, "calibration", &edited, 10, None,
            Some(&original_context.context.key), false,
        )).await.unwrap();
        let corrected_cut = store.project_context_cut(&project.key).await.unwrap();
        assert!(corrected_cut.sequence > original_cut.sequence);
        let corrected_context = store.select_project_context(&corrected_cut, "calibration", 20)
            .await.unwrap().unwrap();
        assert_eq!(corrected_context.effective_at_us, original_context.effective_at_us);
        assert_ne!(corrected_context.context.key, original_context.context.key);
        assert_ne!(corrected_context.context.revision, original_context.context.revision);
        assert_eq!(corrected_context.context.revision, edited_revision.key);
        assert_eq!(corrected_context.context.corrects.as_deref(), Some(original_context.context.key.as_str()));
        assert_eq!(corrected_context.context.physical_context, original_context.context.physical_context);
        assert_eq!(store.select_project_context(&original_cut, "calibration", 20).await.unwrap().as_ref(), Some(&original_context));
        assert_eq!(store.select_project_context(&corrected_cut, "calibration", 5).await.unwrap().unwrap().context.key, original_context.context.key);
        let corrected_package = Box::pin(runtime.project_context_reopen(&corrected_context, &cancel))
            .await.unwrap();
        let corrected_definition = definition(runtime, &corrected_package, &overlays).await;
        for (original, corrected) in original_definition.points.iter().zip(&corrected_definition.points) {
            assert_eq!(original.policy.key, corrected.policy.key);
            assert_eq!(original.binding.entries, corrected.binding.entries);
            assert_eq!(original.binding_hash, corrected.binding_hash);
            assert_ne!(original.binding.revision, corrected.binding.revision);
        }
        let corrected_handle = Box::pin(runtime.start_recorded_study(corrected_definition.clone(), &cancel))
            .await.unwrap();
        assert_ne!(corrected_handle.study_id(), original_handle.study_id());
        let corrected_status = execute(science, [left, right], &corrected_handle).await;
        let corrected = data.0.freeze_study_cohort(request(
            &corrected_definition, &corrected_handle.study_id().to_string(), true,
        )).await.unwrap();
        captured(&corrected, true).await;
        assert_captured_attempts(&corrected, &corrected_status).await;
        let (corrected_coordinates, _) = science_rows(&data.0, &corrected_status, &corrected_definition, 9.).await;
        assert_eq!(coordinates, corrected_coordinates);

        // Replay the recorded original after the correction, without generating
        // any points or asking the current context to reinterpret old inputs.
        drop(corrected_package);
        drop(edited);
        drop(historical);
        drop(declarations);
        drop(overlays);
        let replay = StudyDefinition::readmission(&encoded).unwrap();
        assert_eq!(serde_json::to_value(&replay.points).unwrap(), realized);
        assert_eq!(replay.context.as_ref(), Some(&original_context));
        let replay_handle = Box::pin(runtime.start_recorded_study(replay.clone(), &cancel)).await.unwrap();
        assert_ne!(replay_handle.study_id(), original_handle.study_id());
        assert_ne!(replay_handle.study_id(), corrected_handle.study_id());
        let replay_status = execute(science, [left, right], &replay_handle).await;
        let (replay_coordinates, _) = science_rows(&data.0, &replay_status, &replay, 4.).await;
        assert_eq!(coordinates, replay_coordinates);
        assert!(replay_status.points.iter().all(|point|
            original_status.points.iter().all(|old| old.run != point.run && old.attempt != point.attempt)
        ));

        // Settle every scientific producer and native lane before relational
        // comparison. Frozen cohorts retain their original protected payloads.
        science.close().await.unwrap();
        assert_passive_observation(&data.0, &original_study).await;
        assert_eq!(captured(&original, true).await, frozen_before);
        assert_eq!(captured(&pending, false).await, pending_before);
        let (_, after) = science_rows(&data.0, &original_status, &original_definition, 4.).await;
        assert_eq!(after, original_rows, "original values, tolerances and provenance are unmodified");
        assert_aggregate(&pending, &coordinates, None).await;
        assert_aggregate(&original, &coordinates, Some([2., 3.])).await;
        assert_aggregate(&corrected, &coordinates, Some([3., 14_f64.sqrt()])).await;
        assert_captured_exports(&data.0, &original, &original_definition, &original_study,
            &coordinates, &original_rows[&POINT_KEYS[0]]).await;
        assert_wrong_pool_refused(&original, &coordinates).await;
        compare(&original, &corrected, &coordinates).await;
        // A second fresh native query must select the same protected original
        // attempts; no producer, generated design or result hydration is needed.
        compare(&original, &corrected, &coordinates).await;
        assert_captured_attempts(&original, &original_status).await;
        assert_eq!(captured(&original, true).await, frozen_before);
        assert_eq!(serde_json::to_value(&original_definition.points).unwrap(), realized);
    })
}

#[tokio::test]
async fn public_native_repeated_analysis_preserves_frozen_cut_and_replays_realized_points() {
    let mut service = service();
    let outcome = AssertUnwindSafe(service.run(
        Instant::now() + Duration::from_secs(600),
        |service| Box::pin(async move {
            service.launch(Instant::now() + Duration::from_secs(90)).await?;
            let store = service.ready()?.store().clone();
            let left = service.connect().await?;
            let right = service.connect().await?;
            let mut science = Science::new(&[store, left.store().clone(), right.store().clone()]);
            let outcome = AssertUnwindSafe(journey(&mut science, service)).catch_unwind().await;
            // This drain runs even when any consumer assertion panics. The
            // service owner independently settles every retained connection and
            // original process after this scope, preserving teardown failures.
            let cleanup = science.close().await;
            if let Err(error) = &cleanup { eprintln!("native-lane cleanup failed: {error}"); }
            if let Err(panic) = outcome { std::panic::resume_unwind(panic); }
            cleanup.unwrap();
            Ok(())
        }),
    )).catch_unwind().await;
    // run() settles service/connection owners before propagating an assertion.
    // Attempt private-state disposal on both success and panic, retaining its
    // authenticated path if physical settlement made disposal unavailable.
    let disposal = service.dispose();
    if let Err(error) = &disposal {
        eprintln!("private repeated-analysis cleanup failed: {error}; state={}", service.state().display());
    }
    match outcome {
        Err(panic) => std::panic::resume_unwind(panic),
        Ok(result) => { result.unwrap(); disposal.unwrap(); }
    }
}
