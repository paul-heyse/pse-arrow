// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Protected source ordering and actual exclusive-writer controls.
use super::*;
use std::sync::atomic::AtomicBool;

fn context() -> pse_compiler::workspace::CompilerContext {
    pse_compiler::workspace::CompilerContext {
        quantities: Arc::new(pse_quantity::standard::standard_registry().unwrap()),
        preconditions: Arc::new(
            pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
        ),
        providers: BTreeMap::new(),
    }
}
fn source(value: u64) -> Vec<Declaration> {
    pse_authoring::language::parse(
        &format!("package p {{def Root {{var x:Scalar;eq e:x=={value};}}}}"),
        SemanticId::from_bytes([98; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap()
}
fn admit(
    service: &Arc<MathService>,
    workspace: &Workspace,
    sequence: i64,
    key: &str,
    value: u64,
) -> ModelingRevision {
    let inputs = context();
    let physical =
        pse_compiler::workspace::physical_identity(&inputs.quantities, &inputs.preconditions);
    let service = service.clone();
    let workspace = workspace.clone();
    let key = key.to_owned();
    // Exercise the admitted asynchronous production path even from synchronous
    // source-order controls; the dedicated runtime is joined before returning.
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let worker_service = service.clone();
        runtime
            .block_on(service.compiler_job(
                workspace,
                super::super::ConstructionEstimate::Known(0),
                Cancellation::default(),
                None,
                move |authority, flag| {
                    worker_service.modeling_revision_at(
                        authority,
                        source(value),
                        pse_modeling::PhysicalScope::default(),
                        Arc::default(),
                        &physical,
                        LaneStamp {
                            sequence,
                            revision: key,
                        },
                        flag,
                        &BTreeMap::new(),
                    )
                },
            ))
            .unwrap()
    })
    .join()
    .unwrap()
}

#[tokio::test]
async fn native_admission_evicts_idle_compiler_workspace_under_pool_pressure() {
    use crate::workflow::tests as fixture;
    let memory = 512 << 20;
    let policy = super::super::MathPolicy {
        worker_bytes: 1 << 30,
        workspace_bytes: 128 << 20,
        foreign_bytes: 1 << 20,
        inner_session_bytes: 0,
        ..Default::default()
    };
    let overhead = policy.stack_bytes + policy.foreign_bytes + policy.inner_session_bytes;
    let runtime = fixture::runtime_on(memory, policy);
    let package = runtime
        .modeling_package(source(1), fixture::physical())
        .await
        .unwrap();
    let service = runtime.shared.math();
    let baseline = service.pool.reserved();
    let workspace = package.pure_workspace().unwrap();
    let retained = service.pool.reserved() - baseline;
    assert!(retained >= service.policy.workspace_bytes);
    service.trim_compiler_lanes();
    assert_eq!(service.compiler_lanes.lock().unwrap().len(), 1);
    assert_eq!(service.pool.reserved(), baseline + retained);
    drop(workspace);
    let demand = memory - baseline - overhead - (1 << 20);
    assert!(baseline + retained + demand + overhead > memory);
    service
        .job_scoped(
            1,
            demand,
            Cancellation::default(),
            Some(std::time::Instant::now() + std::time::Duration::from_secs(5)),
            |_| Ok(()),
        )
        .await
        .unwrap();
    assert_eq!(service.pool.reserved(), baseline);
    assert!(service.compiler_lanes.lock().unwrap().is_empty());
}

#[test]
fn historical_and_branch_admission_and_preparation_never_rewind_live_compiler() {
    let service = super::super::tests::service();
    let workspace = service
        .workspace(
            context(),
            pse_compiler::workspace::WorkspaceLimits::default(),
        )
        .unwrap();
    let head = admit(&service, &workspace, 2, "main-2", 2);
    let historical = admit(&service, &workspace, 1, "main-1", 1);
    let branch = admit(&service, &workspace, 2, "branch-2", 7);
    assert_ne!(head.admitted.checked(), historical.admitted.checked());
    assert_ne!(head.admitted.checked(), branch.admitted.checked());
    assert!(
        workspace
            .compiler
            .lock()
            .unwrap()
            .selects_modeling_revision(&head.admitted)
    );
    service
        .with_modeling_compiler(
            super::super::CompilerAuthority::private(workspace.clone()).unwrap(),
            &historical,
            |compiler| {
                assert!(compiler.selects_modeling_revision(&historical.admitted));
                Ok(())
            },
        )
        .unwrap();
    assert!(
        workspace
            .compiler
            .lock()
            .unwrap()
            .selects_modeling_revision(&head.admitted)
    );
    assert_eq!(
        workspace
            .lane_head
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .revision,
        "main-2"
    );
    let latest = admit(&service, &workspace, 3, "main-3", 3);
    assert!(
        workspace
            .compiler
            .lock()
            .unwrap()
            .selects_modeling_revision(&latest.admitted)
    );
    drop(workspace);
    // Detached immutable products retain their actual source owners, not Salsa storage.
    assert!(service.pool.reserved() > 0);
    drop((head, historical, branch, latest));
    assert_eq!(service.pool.reserved(), 0);
}

#[test]
fn concurrent_protected_writers_publish_monotonic_complete_head() {
    let service = super::super::tests::service();
    let workspace = service
        .workspace(
            context(),
            pse_compiler::workspace::WorkspaceLimits::default(),
        )
        .unwrap();
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let writers = (1..=2)
        .map(|sequence| {
            let service = service.clone();
            let workspace = workspace.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                admit(
                    &service,
                    &workspace,
                    sequence.try_into().unwrap(),
                    &format!("main-{sequence}"),
                    sequence,
                )
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let products = writers
        .into_iter()
        .map(|writer| writer.join().unwrap())
        .collect::<Vec<_>>();
    let latest = products
        .iter()
        .find(|product| product.admitted.declarations() == source(2))
        .unwrap();
    assert!(
        workspace
            .compiler
            .lock()
            .unwrap()
            .selects_modeling_revision(&latest.admitted)
    );
    assert_eq!(
        workspace
            .lane_head
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .sequence,
        2
    );
    drop(workspace);
    drop(products);
    assert_eq!(service.pool.reserved(), 0);
}

#[test]
fn rejected_canonical_supplier_keeps_exact_live_head_and_original_failure_kind() {
    let service = super::super::tests::service();
    let workspace = service
        .workspace(
            context(),
            pse_compiler::workspace::WorkspaceLimits::default(),
        )
        .unwrap();
    let head = admit(&service, &workspace, 1, "main-1", 1);
    let inputs = context();
    let physical =
        pse_compiler::workspace::physical_identity(&inputs.quantities, &inputs.preconditions);
    let record = DeclarationId::from_id(SemanticId::from_bytes([99; 16]));
    let result = service.modeling_revision_at(
        super::super::CompilerAuthority::private(workspace.clone()).unwrap(),
        source(2),
        pse_modeling::PhysicalScope::default(),
        Arc::default(),
        &physical,
        LaneStamp {
            sequence: 2,
            revision: "main-2".into(),
        },
        Arc::new(AtomicBool::new(false)),
        &[(record, record)].into(),
    );
    assert!(matches!(result, Err(MathRuntimeError::Infrastructure(_))));
    assert!(
        workspace
            .compiler
            .lock()
            .unwrap()
            .selects_modeling_revision(&head.admitted)
    );
    assert_eq!(
        workspace
            .lane_head
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .sequence,
        1
    );
    let repaired = admit(&service, &workspace, 2, "main-2", 2);
    assert!(
        workspace
            .compiler
            .lock()
            .unwrap()
            .selects_modeling_revision(&repaired.admitted)
    );
}

#[tokio::test]
async fn dynamic_snapshot_pressure_evicts_idle_lane_but_retains_historical_source_owners() {
    use pse_engine::session::{EngineFactory, ExecutionSettings, ThreadBudget};
    use std::num::NonZeroUsize;
    // Actual source admission now retains the charged worker stack/session envelope
    // alongside the live workspace, historical workspace and source candidate.
    // The later borrower still fills this pool to establish real snapshot pressure.
    let capacity = 128 << 20;
    let registry = pse_schema::shared_registry().unwrap();
    let resources = pse_engine::resources::EngineResources::build(
        NonZeroUsize::new(capacity).unwrap(),
        NonZeroUsize::new(8).unwrap(),
        std::env::temp_dir(),
        1 << 20,
        pse_engine::cache_service::CacheBudget::disabled(1024),
    )
    .unwrap();
    let factory = Arc::new(
        EngineFactory::new(
            resources.runtime.clone(),
            resources.pool.clone(),
            ExecutionSettings::default(),
            ThreadBudget {
                pool_threads: NonZeroUsize::new(1).unwrap(),
                target_partitions: NonZeroUsize::new(1).unwrap(),
            },
            pse_engine::session::native_engine_profile(),
        )
        .unwrap(),
    );
    let owner = factory.bind_model(registry.clone()).unwrap();
    let lineage = CompilerLineage {
        lineage: "local-pressure-control".into(),
        interpretation: "local".into(),
        validation: Arc::as_ptr(&factory) as usize,
        registry,
        _validation_owner: owner,
    };
    let (service, _cache) = super::super::tests::service_with_policy(
        capacity,
        super::super::MathPolicy {
            foreign_bytes: 1 << 20,
            worker_bytes: 8 << 20,
            workspace_bytes: 16 << 20,
            ..super::super::MathPolicy::default()
        },
    );
    let workspace = service.compiler_lane(lineage.clone(), context()).unwrap();
    let head = admit(&service, &workspace, 2, "main-2", 2);
    let historical = admit(&service, &workspace, 1, "main-1", 1);
    let retained = head._lease.size() + historical._lease.size();
    assert!(retained > 0);
    drop(workspace);
    assert_eq!(service.compiler_lanes.lock().unwrap().len(), 1);
    let before = service.pool.reserved();
    let live = service
        .reserve("test:live-pressure-borrower", capacity - before - 1024)
        .unwrap();
    let snapshots = service.dynamic_snapshots();
    let snapshot = snapshots.reserve(2 << 20).unwrap();
    assert!(service.compiler_lanes.lock().unwrap().is_empty());
    assert_eq!(service.pool.reserved(), live.size() + (2 << 20) + retained);
    assert_eq!(historical.admitted.declarations(), source(1));
    // The next request constructs a new charged generation, rather than borrowing
    // a database kept alive by the detached historical product.
    drop((snapshot, live));
    assert_eq!(service.pool.reserved(), retained);
    let rebuilt = service.compiler_lane(lineage, context()).unwrap();
    assert!(!rebuilt.compiler.lock().unwrap().has_modeling_revision());
    drop(rebuilt);
    service.trim_compiler_lanes();
    assert_eq!(service.pool.reserved(), retained);
    drop((head, historical));
    assert_eq!(service.pool.reserved(), 0);
}

#[tokio::test]
async fn local_package_reload_reuses_live_lane_and_keeps_historical_source_maps() {
    let runtime = crate::workflow::tests::runtime();
    let service = runtime.shared.math();
    let pool = runtime.shared.pool();
    let baseline = pool.reserved();
    let physical = crate::workflow::tests::physical();
    let cancel = crate::CancelSource::new();
    let text = "package minimal_named {def Root {var x:Scalar;eq e:x==1;}}";
    let sources = |text: &str| {
        BTreeMap::from([
            (
                "package.toml".into(),
                include_bytes!("../../../../../tests/fixtures/packages/minimal_named/package.toml")
                    .to_vec(),
            ),
            ("models/bank.pse".into(), text.as_bytes().to_vec()),
        ])
    };
    let token = cancel.token();
    let bundle = pse_admission::authoring_driver::document::load_package_documents_owned(
        &sources(text),
        runtime.registry(),
        pse_authoring::ParseBudget::default(),
        &pool,
        &token,
        &runtime.validation_context(),
    )
    .unwrap();
    let documents = pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
        vec![bundle],
        &pool,
        &token,
    )
    .unwrap();
    let package = runtime
        .modeling_from_documents(&documents, physical.clone(), &cancel)
        .await
        .unwrap();
    drop(documents);
    let rows = package.declarations().await.unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let equation = rows
        .iter()
        .find(|row| row.value.equation.is_some())
        .unwrap()
        .declaration_id;
    drop(rows);
    let instance = pse_modeling::specialize::root_instance(root);
    let cold = package
        .prepare(
            root,
            instance,
            Bindings::default(),
            Limits::default(),
            &cancel,
        )
        .await
        .unwrap();
    let lane = |package: &crate::workflow::ModelingPackage| package.pure_workspace().unwrap();
    let workspace = lane(&package);
    let workspaces = service.preparation_effects().workspaces;
    let warm = package
        .prepare(
            root,
            instance,
            Bindings::default(),
            Limits::default(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(
        service.preparation_effects().workspaces,
        workspaces,
        "ordinary warm package preparation reuses its retained compiler"
    );
    let revised_text = format!("\n\n{}", text.replace("x==1", "x==2"));
    let changed = package
        .reload_from_parent(&[sources(&revised_text)], &cancel)
        .await
        .unwrap();
    let before_prepare = service.preparation_effects().workspaces;
    let current = changed
        .prepare(
            root,
            instance,
            Bindings::default(),
            Limits::default(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(
        service.preparation_effects().workspaces,
        before_prepare,
        "changed selected admission and preparation share the live lane"
    );
    let shared = lane(&changed);
    assert!(Arc::ptr_eq(&workspace.compiler, &shared.compiler));
    drop(shared);
    let key = pse_modeling::expression::occurrences::OccurrenceKey {
        declaration: equation,
        role: "equation.expression".into(),
        position: 0,
    };
    let old = &cold.compiled().model.source_occurrences()[&key];
    let new = &current.compiled().model.source_occurrences()[&key];
    assert_eq!(old.text, "x==1");
    assert_eq!(new.text, "x==2");
    assert_ne!(
        old.source, new.source,
        "parsed reload installs its exact current field attribution"
    );
    let historical = package
        .prepare(
            root,
            instance,
            Bindings::default(),
            Limits::default(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(
        historical.compiled().model.source_occurrences()[&key].text,
        "x==1"
    );
    let live = workspace
        .compiler
        .lock()
        .unwrap()
        .specialize_modeling(root, instance, Bindings::default(), Limits::default())
        .unwrap();
    assert_eq!(
        live.source_occurrences()[&key].text,
        "x==2",
        "historical preparation cannot rewind the admitted live head"
    );
    drop((
        live, historical, current, warm, cold, workspace, changed, package,
    ));
    service.clear_program_cache();
    assert_eq!(
        pool.reserved(),
        baseline,
        "completed requests and idle compiler generations drain their actual source owners"
    );
}
