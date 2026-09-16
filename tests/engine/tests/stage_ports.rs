// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual producer ports determine parent rows even when output contracts repeat.
#![allow(clippy::unwrap_used, reason = "bounded declared port fixtures")]
#[path = "../../support/physical_source.rs"]
mod physical_source;
#[path = "../../support/workflow_budget.rs"]
mod workflow_budget;
use datafusion::{
    execution::runtime_env::RuntimeEnv,
    logical_expr::{LogicalPlanBuilder, col, lit},
};
use pse_catalog::computation::ProducedStage;
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
};
use pse_catalog::{
    Catalog, EncodingPolicy, FixedClock, RelationContract, Snapshot, TrustLevel,
    store::{
        membership::AdmissionContext,
        publish::{BundleDraft, RelationDraft},
    },
};
use pse_compiler::{CompilerError, InputBundle, Pass, PassContext, PassStatus};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, SnapshotKind};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::PassSpec;
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, DerivationGranularity, Determinism, FieldContract, InputPort, Namespace,
        OutputPort, PassDecl, PortSource, RelationDecl, SnapshotClass,
    },
};
use std::{collections::BTreeMap, sync::Arc};

fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_foundations(&mut builder);
    for name in ["inputs", "outputs"] {
        let column = FieldContract::key(
            "id",
            FieldContract::native(arrow::datatypes::DataType::UInt64),
            "actual member",
        );
        let column = if name == "outputs" {
            column.with_fk("normalized.inputs", "id")
        } else {
            column
        };
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Normalized,
                name,
                1,
                Authority::Derived,
                SnapshotClass::Derived,
                "port fixture",
            )
            .granularity(DerivationGranularity::Rule)
            .pk(&["id"])
            .columns(vec![column]),
        );
    }
    for name in ["source", "extra_output", "missing_output"] {
        builder.declare_pass(
            PassDecl::new(name, "1", Determinism::Deterministic).outputs(
                ["left", "right"]
                    .into_iter()
                    .map(|port| OutputPort {
                        port,
                        relation: "normalized.inputs".to_owned(),
                    })
                    .collect(),
            ),
        );
    }
    builder.declare_pass(
        PassDecl::new("document_count", "1", Determinism::Deterministic).outputs(vec![
            OutputPort {
                port: "documents",
                relation: "normalized.inputs".into(),
            },
        ]),
    );
    for (name, source) in [
        (
            "selected",
            PortSource::Derived {
                pass: "source",
                port: "left",
            },
        ),
        ("ambiguous", PortSource::Pinned),
    ] {
        builder.declare_pass(
            PassDecl::new(name, "1", Determinism::Deterministic)
                .inputs(vec![InputPort {
                    port: "input",
                    relation: "normalized.inputs".to_owned(),
                    source,
                    required: true,
                }])
                .outputs(vec![OutputPort {
                    port: "result",
                    relation: "normalized.outputs".to_owned(),
                }]),
        );
    }
    builder.declare_pass(
        PassDecl::new("role_delta", "1", Determinism::Deterministic)
            .inputs(
                [("before", "left"), ("after", "right")]
                    .into_iter()
                    .map(|(port, source)| InputPort {
                        port,
                        relation: "normalized.inputs".into(),
                        source: PortSource::Derived {
                            pass: "source",
                            port: source,
                        },
                        required: true,
                    })
                    .collect(),
            )
            .outputs(vec![OutputPort {
                port: "result",
                relation: "normalized.inputs".into(),
            }]),
    );
    Arc::new(builder.build().unwrap())
}
fn draft(
    catalog: &Catalog,
    pass: &str,
    parent: Option<&Arc<Snapshot>>,
    values: &[(&str, u64)],
) -> Result<BundleDraft, pse_catalog::CatalogError> {
    let reg = catalog.registry();
    let pass = reg.pass(pass).unwrap();
    let context = AdmissionContext {
        traversal: Arc::default(),
        invocation: Some(pse_catalog::store::invocation::InvocationContext::capture(
            std::iter::empty(),
            None,
            None,
            catalog.reserver().as_ref(),
        )?),
        stage_pass: Some(pass.id),
        parents: parent.map_or_else(BTreeMap::new, |parent| {
            BTreeMap::from([("input".to_owned(), Arc::clone(parent))])
        }),
    };
    let context = catalog
        .prepare_production(context, None, &CancellationToken::new())?
        .inputs()
        .clone();
    let relations = values
        .iter()
        .map(|(port, value)| {
            let output = pass
                .outputs
                .iter()
                .find(|output| output.port == *port)
                .unwrap();
            let spec = reg.relation(&output.relation).unwrap();
            let relation = RelationDraft {
                contract: Arc::new(
                    RelationContract::from_spec(reg, spec, EncodingPolicy::IpcFile).unwrap(),
                ),
                batches: vec![
                    pse_relations::cells::batch_from_cells(reg, spec, &[vec![Cell::U64(*value)]])
                        .unwrap(),
                ],
            };
            ((*port).to_owned(), relation)
        })
        .collect();
    Ok(BundleDraft {
        manifest: catalog.manifest_template(SnapshotKind::Stage, &context)?,
        context,
        relations,
    })
}
struct PortPass(PassSpec);
impl Pass for PortPass {
    fn spec(&self) -> &PassSpec {
        &self.0
    }
    fn run<'a>(
        &'a self,
        ctx: &'a PassContext<'a>,
        inputs: &'a InputBundle,
    ) -> pse_catalog::BoxFut<'a, Result<ProducedStage, CompilerError>> {
        Box::pin(async move {
            if self.0.name == "role_delta" {
                return role_delta(ctx).await;
            }
            let mut ports = BTreeMap::new();
            for port in &self.0.outputs {
                let spec = ctx.registry.relation(&port.relation).unwrap();
                let values = if matches!(self.0.name, "source" | "extra_output" | "missing_output")
                {
                    vec![vec![Cell::U64(if port.port == "left" { 1 } else { 2 })]]
                } else if self.0.name == "document_count" {
                    let count = ctx
                        .documents
                        .bundles()
                        .iter()
                        .map(|bundle| bundle.documents.len())
                        .sum::<usize>();
                    vec![vec![Cell::U64(u64::try_from(count).unwrap())]]
                } else {
                    let input = inputs.port("input").unwrap().as_ref().unwrap();
                    let source = ctx.registry.relation("normalized.inputs").unwrap();
                    pse_relations::cells::cells_from_batch(
                        ctx.registry,
                        source,
                        input.relation().batch(),
                    )?
                };
                let batch = pse_relations::cells::batch_from_cells_owned(
                    ctx.registry,
                    spec,
                    &values,
                    ctx.reserver,
                    ctx.cancel,
                )?;
                ports.insert(
                    port.port.to_owned(),
                    FieldCheckedBatch::admit(ctx.registry, spec, batch)?,
                );
            }
            if self.0.name == "extra_output" {
                ports.insert("private_occurrences".into(), ports["left"].clone());
            } else if self.0.name == "missing_output" {
                ports.remove("left");
            }
            Ok(ProducedStage {
                outputs: ports,
                findings: vec![],
                derivations: vec![],
                plans: vec![],
            })
        })
    }
}
async fn role_delta(ctx: &PassContext<'_>) -> Result<ProducedStage, CompilerError> {
    let session = ctx.session;
    let spec = ctx.registry.relation("normalized.inputs").unwrap();
    assert_eq!(session.input_roles().count(), 2);
    assert!(
        session.table_source(&spec.key).is_err(),
        "repeated inputs have no implicit schema selection"
    );
    let before = LogicalPlanBuilder::from(session.scan_role("before")?)
        .alias("before")
        .unwrap()
        .build()
        .unwrap();
    let after = LogicalPlanBuilder::from(session.scan_role("after")?)
        .alias("after")
        .unwrap()
        .build()
        .unwrap();
    let plan = LogicalPlanBuilder::from(before)
        .cross_join(after)
        .unwrap()
        .project([(col("before.id") * lit(10_u64) + col("after.id")).alias("id")])
        .unwrap()
        .build()
        .unwrap();
    let plan =
        pse_catalog::session::output::declare_relation_output(plan, ctx.registry, spec).unwrap();
    let result = session
        .prepare_rule_plan(plan, ctx.cancel)?
        .execute(ctx.cancel)
        .await?
        .into_checked_relation(ctx.registry, spec, ctx.cancel)?;
    Ok(ProducedStage {
        outputs: BTreeMap::from([("result".into(), result)]),
        findings: vec![],
        derivations: vec![],
        plans: vec![],
    })
}
fn catalog(reg: &Arc<Registry>, reserver: Arc<dyn MemoryReserver>) -> Catalog {
    catalog_with_store(
        reg,
        reserver,
        Arc::new(object_store::memory::InMemory::new()),
    )
    .0
}
fn catalog_with_store(
    reg: &Arc<Registry>,
    reserver: Arc<dyn MemoryReserver>,
    store: Arc<object_store::memory::InMemory>,
) -> (Catalog, Arc<SessionFactory>) {
    let sessions = Arc::new(
        SessionFactory::new(
            Arc::new(RuntimeEnv::default()),
            Arc::clone(&reserver),
            ExecutionSettings::default(),
            ThreadBudget {
                pool_threads: 1.try_into().unwrap(),
                target_partitions: 2.try_into().unwrap(),
            },
            native_engine_profile(),
        )
        .unwrap(),
    );
    let invariants = pse_rules::validator::InvariantValidator::new(Arc::clone(reg));
    let mut validator =
        pse_compiler::validator::CompilerValidator::new(Arc::new(invariants), reg).unwrap();
    for name in [
        "source@1",
        "selected@1",
        "ambiguous@1",
        "document_count@1",
        "extra_output@1",
        "missing_output@1",
        "role_delta@1",
    ] {
        validator
            .register(Arc::new(PortPass(reg.pass(name).unwrap().clone())), reg)
            .unwrap();
    }
    let catalog = Catalog::open(
        store,
        Arc::clone(reg),
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-15T00:00:00Z".to_owned())),
        Arc::clone(&sessions),
    )
    .with_semantic_validator(Arc::new(validator.with_sessions(Arc::clone(&sessions))));
    (catalog, sessions)
}
async fn produce(catalog: &Catalog, pass: &str, parent: Option<&Arc<Snapshot>>) -> Arc<Snapshot> {
    let cancel = CancellationToken::new();
    let context = AdmissionContext {
        traversal: Arc::default(),
        invocation: Some(
            pse_catalog::store::invocation::InvocationContext::capture(
                std::iter::empty(),
                None,
                None,
                catalog.reserver().as_ref(),
            )
            .unwrap(),
        ),
        stage_pass: Some(catalog.registry().pass(pass).unwrap().id),
        parents: parent.map_or_else(BTreeMap::new, |parent| {
            BTreeMap::from([("input".to_owned(), Arc::clone(parent))])
        }),
    };
    let completed = catalog
        .prepare_production(context, None, &CancellationToken::new())
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    if pass == "source@1" {
        assert!(completed.computation().observation().explain_pgjson().len() < 32_768);
        assert!(
            completed
                .computation()
                .observation()
                .physical_plan()
                .unwrap()
                .contains("OperationExec")
        );
        let outputs = completed.output_session();
        for _ in 0..2 {
            let rows = outputs
                .sql("SELECT * FROM outputs.\"source@1\".left", &cancel)
                .await
                .unwrap();
            assert_eq!(rows.iter().map(|batch| batch.num_rows()).sum::<usize>(), 1);
            let rows = outputs
                .sql("SELECT * FROM outputs.\"source@1\".right", &cancel)
                .await
                .unwrap();
            assert_eq!(rows.iter().map(|batch| batch.num_rows()).sum::<usize>(), 1);
        }
        assert!(
            completed
                .computation()
                .prepared()
                .clone()
                .execute(&cancel)
                .await
                .is_err()
        );
    }
    catalog
        .publish_production(completed, &cancel)
        .await
        .unwrap()
        .snapshot
}
#[tokio::test]
async fn derived_parent_uses_only_the_named_output_and_pinned_ambiguity_is_refused() {
    let reg = registry();
    let reserver: Arc<dyn MemoryReserver> = FixedBudget::new(128 << 20);
    let catalog = catalog(&reg, Arc::clone(&reserver));
    let cancel = CancellationToken::new();
    let source = produce(&catalog, "source@1", None).await;
    assert_eq!(source.relations().len(), 2);
    assert!(source.relation("normalized", "inputs").is_none());
    let expected = draft(&catalog, "selected@1", Some(&source), &[("result", 1)]).unwrap();
    let context = expected.context.clone();
    let selected = produce(&catalog, "selected@1", Some(&source)).await;
    catalog
        .read_manifest(selected.manifest_ref(), &context, &cancel)
        .await
        .unwrap();
    assert!(
        catalog
            .publish_bundle(
                draft(&catalog, "selected@1", Some(&source), &[("result", 2)]).unwrap(),
                &cancel
            )
            .await
            .is_err(),
        "an unselected sibling output cannot satisfy the selected parent's foreign key"
    );
    assert!(matches!(
        draft(&catalog, "ambiguous@1", Some(&source), &[("result", 1)]),
        Err(pse_catalog::CatalogError::Membership { reason })
            if reason == "input role input lacks one unambiguous declared relation normalized.inputs"
    ));
}

#[tokio::test]
async fn inspection_requires_exact_repeated_output_port() {
    let reg = registry();
    let reserver: Arc<dyn MemoryReserver> = FixedBudget::new(128 << 20);
    let (catalog, sessions) = catalog_with_store(
        &reg,
        reserver,
        Arc::new(object_store::memory::InMemory::new()),
    );
    let cancel = CancellationToken::new();
    let source = produce(&catalog, "source@1", None).await;
    let session = sessions.inspect_snapshot(&source, &cancel).unwrap();
    assert!(
        pse_catalog::inspection::TableReader::new(
            &session,
            "normalized.inputs",
            None,
            std::num::NonZeroUsize::MIN,
            cancel.clone(),
        )
        .await
        .is_err()
    );
    for (port, expected) in [("left", 1), ("right", 2)] {
        let mut reader = pse_catalog::inspection::TableReader::new(
            &session,
            "normalized.inputs",
            Some(port),
            std::num::NonZeroUsize::MIN,
            cancel.clone(),
        )
        .await
        .unwrap();
        let batch = reader.next_batch().await.unwrap().unwrap();
        let spec = reg.relation("normalized.inputs").unwrap();
        let cells = pse_relations::cells::cells_from_batch(&reg, spec, &batch).unwrap();
        assert_eq!(cells, vec![vec![Cell::U64(expected)]]);
        assert!(reader.next_batch().await.unwrap().is_none());
    }
}

async fn empty_model(catalog: &Catalog, cancel: &CancellationToken) -> Arc<Snapshot> {
    let reg = catalog.registry();
    let context = AdmissionContext::default();
    catalog
        .publish_bundle(
            BundleDraft {
                manifest: catalog
                    .manifest_template(SnapshotKind::Model, &context)
                    .unwrap(),
                context,
                relations: reg
                    .relations()
                    .iter()
                    .filter(|spec| spec.snapshot_class == SnapshotClass::Model)
                    .map(|spec| {
                        (
                            pse_ids::model_port_name(spec.key.namespace.as_str(), spec.id),
                            RelationDraft {
                                contract: Arc::new(
                                    RelationContract::from_spec(
                                        &reg,
                                        spec,
                                        EncodingPolicy::IpcFile,
                                    )
                                    .unwrap(),
                                ),
                                batches: vec![],
                            },
                        )
                    })
                    .collect(),
            },
            cancel,
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn durable_context_shares_ownership_and_reopens_actual_producer_records() {
    use pse_catalog::store::stage::StageInputs;
    use pse_compiler::{
        PolicySet,
        driver::{Driver, PipelineRequest},
    };

    let reg = registry();
    let store = Arc::new(object_store::memory::InMemory::new());
    let budget = FixedBudget::new(workflow_budget::MEMORY_LIMIT_BYTES);
    let (catalog, _) = catalog_with_store(&reg, budget, Arc::clone(&store));
    let catalog = Arc::new(catalog);
    let cancel = CancellationToken::new();
    let model = empty_model(&catalog, &cancel).await;
    let model_ref = model.manifest_ref();
    let mut driver = Driver::new(Arc::clone(&catalog)).unwrap();
    let report = driver
        .run(
            PipelineRequest {
                through: "selected".to_owned(),
                snapshot: model,
                policies: PolicySet::default(),
                reuse: false,
            },
            &cancel,
        )
        .await
        .unwrap_or_else(|error| panic!("registered stage execution failed: {error}"));
    assert_eq!(report.stages.len(), 2);
    let source = &report.stages[0];
    let selected = &report.stages[1];
    let key = selected.key.content_hash();
    let hints = catalog.stage_hints(key, &cancel).await.unwrap();
    assert_eq!(hints.len(), 1);
    let inputs = StageInputs::from([("input".to_owned(), Some(Arc::clone(&source.snapshot)))]);
    let hint = catalog
        .write_stage_hint(key, &inputs, &selected.snapshot, &selected.record, &cancel)
        .await
        .unwrap();
    assert!(
        std::ptr::eq(
            selected.snapshot.invocation().unwrap().as_ref(),
            source.snapshot.invocation().unwrap().as_ref()
        ),
        "one immutable request shares its actual invocation owner"
    );
    assert!(
        catalog
            .open_stage_hint(&hint, &StageInputs::new(), &cancel)
            .await
            .is_err()
    );

    let reader_budget = FixedBudget::new(workflow_budget::MEMORY_LIMIT_BYTES);
    let (reader, _) = catalog_with_store(&reg, reader_budget.clone(), Arc::clone(&store));
    let hints = reader.stage_hints(key, &cancel).await.unwrap();
    assert!(hints[0] == hint, "reopened complete hint values agree");
    let restored_source = reader
        .read_pinned_manifest(source.snapshot.manifest_ref(), &cancel)
        .await
        .unwrap();
    let restored_inputs = StageInputs::from([("input".to_owned(), Some(restored_source))]);
    let (restored, record) = reader
        .open_stage_hint(&hints[0], &restored_inputs, &cancel)
        .await
        .unwrap();
    assert_eq!(restored.snapshot_id(), selected.snapshot.snapshot_id());
    assert_eq!(record.reference(), selected.record.reference());
    let retained = restored.invocation().unwrap().clone();
    drop((restored, record, restored_inputs));
    let reopened_model = reader
        .read_pinned_manifest(model_ref, &cancel)
        .await
        .unwrap();
    let mut restarted = Driver::new(Arc::new(reader.clone())).unwrap();
    let reused = restarted
        .run(
            PipelineRequest {
                through: "selected".into(),
                snapshot: reopened_model,
                policies: PolicySet::default(),
                reuse: true,
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(reused.stages.len(), 2);
    for (actual, original) in reused.stages.iter().zip(&report.stages) {
        assert_eq!(actual.status, PassStatus::Reused);
        assert_eq!(
            actual.snapshot.manifest_ref(),
            original.snapshot.manifest_ref()
        );
        assert_ne!(
            actual.record.reference(),
            original.record.reference(),
            "a reused result still receives a new attempt record"
        );
    }
    drop((restarted, reused));
    let used = reader_budget.reserved();
    let retained_hint = hints[0].clone();
    assert_eq!(reader_budget.reserved(), used);
    drop((hints, reader));
    assert!(
        retained_hint == hint,
        "detached immutable hint retains its exact references"
    );
    assert!(
        reader_budget.reserved() > 0,
        "the admitted invocation survives its snapshot, reader and hint"
    );
    assert_eq!(
        retained.document_source.as_ref().unwrap().manifest_ref(),
        model_ref
    );
    drop((retained, retained_hint));
    assert!(
        reader_budget.reserved() > 0,
        "the memory store still owns the new durable attempt record bytes"
    );
    drop((driver, catalog, store, report, hint, inputs));
    assert_eq!(reader_budget.reserved(), 0);
}

#[tokio::test]
async fn cold_reopen_restores_original_documents_without_relational_input_ports() {
    use pse_compiler::{
        PolicySet,
        driver::{CommitRequest, Driver, PipelineRequest},
    };
    use pse_ids::SemanticId;
    let reg = registry();
    let store = Arc::new(object_store::memory::InMemory::new());
    let budget = FixedBudget::new(workflow_budget::MEMORY_LIMIT_BYTES);
    let (catalog, _) = catalog_with_store(&reg, budget, store.clone());
    let catalog = Arc::new(catalog);
    let mut driver = Driver::new(catalog.clone()).unwrap();
    let document = pse_authoring::document::load_package_texts(
        physical_source::physical_texts(&reg),
        &reg,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let expected = u64::try_from(document.documents.len()).unwrap();
    assert!(expected > 0);
    let cancel = CancellationToken::new();
    let committed = driver
        .commit(
            CommitRequest {
                revision_ids: None,
                reference: pse_catalog::RefName::parse("document_source").unwrap(),
                base: None,
                documents: vec![document],
                changes: None,
                header: pse_relations::generated::authored::change_sets::Row {
                    change_set_id: pse_authoring::ids::uuid_v7(),
                    base_revision_id: SemanticId::NIL,
                    author: "fixture".into(),
                    message: "actual source selection".into(),
                    created_at: 1,
                },
            },
            &cancel,
        )
        .await
        .unwrap()
        .tip
        .unwrap();
    let source = committed.manifest_ref();
    let report = driver
        .run(
            PipelineRequest {
                through: "document_count".into(),
                snapshot: committed,
                policies: PolicySet::default(),
                reuse: false,
            },
            &cancel,
        )
        .await
        .unwrap();
    let output = &report.stages.last().unwrap().snapshot;
    assert!(
        output.parents().is_empty(),
        "this producer has no relational input port"
    );
    let (reader, _) = catalog_with_store(
        &reg,
        FixedBudget::new(workflow_budget::MEMORY_LIMIT_BYTES),
        store,
    );
    let reopened = reader
        .read_pinned_manifest(output.manifest_ref(), &cancel)
        .await
        .unwrap();
    assert_eq!(
        reopened
            .invocation()
            .unwrap()
            .document_source
            .as_ref()
            .unwrap()
            .manifest_ref(),
        source
    );
    let relation = reopened.relation_port("documents").unwrap();
    let rows = pse_relations::cells::cells_from_batch(
        &reg,
        reg.relation("normalized.inputs").unwrap(),
        relation.batch(),
    )
    .unwrap();
    assert_eq!(rows, vec![vec![Cell::U64(expected)]]);
}

#[tokio::test]
async fn binding_requires_actual_named_output_port_when_schema_is_repeated() {
    use pse_compiler::{BoundInput, PolicySet, memo::Dependencies};
    let registry = registry();
    let spec = registry.relation("normalized.inputs").unwrap();
    let reserver: Arc<dyn MemoryReserver> = FixedBudget::new(128 << 20);
    let (catalog, sessions) = catalog_with_store(
        &registry,
        Arc::clone(&reserver),
        Arc::new(object_store::memory::InMemory::new()),
    );
    let native = sessions
        .candidate(
            BTreeMap::new(),
            Arc::clone(&registry),
            &CancellationToken::new(),
        )
        .unwrap();
    let snapshot = produce(&catalog, "source@1", None).await;
    assert!(BoundInput::bind(Arc::clone(&snapshot), spec.key, &registry).is_err());
    let mut dependencies = Vec::new();
    for (port, value) in [("left", 1_u64), ("right", 2)] {
        let input =
            BoundInput::bind_port(Arc::clone(&snapshot), spec.key, port, &registry).unwrap();
        assert_eq!(
            pse_relations::cells::cells_from_batch(&registry, spec, input.relation().batch())
                .unwrap(),
            vec![vec![Cell::U64(value)]]
        );
        dependencies.push(
            Dependencies::capture(
                &InputBundle {
                    ports: BTreeMap::from([("input", Some(input))]),
                },
                &PassContext {
                    physical: None,
                    registry: &registry,
                    documents: &pse_authoring::document::OwnedDocumentSet::default(),
                    policies: &PolicySet::default(),
                    cancel: &CancellationToken::default(),
                    reserver: reserver.as_ref(),
                    session: &native,
                },
            )
            .unwrap(),
        );
    }
    assert!(
        !dependencies[0]
            .equivalent(
                &dependencies[1],
                reserver.as_ref(),
                &CancellationToken::new()
            )
            .unwrap()
    );
    assert!(BoundInput::bind_port(snapshot, spec.key, "missing", &registry).is_err());
}

#[tokio::test]
async fn private_or_missing_outputs_are_refused_by_catalog_completion() {
    let registry = registry();
    let budget = FixedBudget::new(128 << 20);
    let catalog = catalog(&registry, budget);
    let cancel = CancellationToken::new();
    for name in ["extra_output@1", "missing_output@1"] {
        let context = draft(&catalog, name, None, &[]).unwrap().context;
        let error = catalog
            .prepare_production(context, None, &CancellationToken::new())
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap_err();
        assert!(
            matches!(error, pse_catalog::CatalogError::Membership { reason }
            if reason == "producer did not complete every declared output port")
        );
    }
}

#[tokio::test]
async fn same_schema_input_roles_execute_and_reopen_through_the_registered_driver() {
    use pse_compiler::{
        PolicySet,
        driver::{Driver, PipelineRequest},
    };
    let reg = registry();
    let store = Arc::new(object_store::memory::InMemory::new());
    let (catalog, _) = catalog_with_store(
        &reg,
        FixedBudget::new(workflow_budget::MEMORY_LIMIT_BYTES),
        Arc::clone(&store),
    );
    let catalog = Arc::new(catalog);
    let cancel = CancellationToken::new();
    let model = empty_model(&catalog, &cancel).await;
    let mut driver = Driver::new(Arc::clone(&catalog)).unwrap();
    let report = driver
        .run(
            PipelineRequest {
                through: "role_delta".into(),
                snapshot: model,
                policies: PolicySet::default(),
                reuse: false,
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(report.stages.len(), 2);
    let result = &report.stages[1].snapshot;
    assert_eq!(result.parents().len(), 2);
    let spec = reg.relation("normalized.inputs").unwrap();
    let values = |snapshot: &Snapshot| {
        pse_relations::cells::cells_from_batch(
            &reg,
            spec,
            snapshot.relation_port("result").unwrap().batch(),
        )
        .unwrap()
    };
    assert_eq!(values(result), vec![vec![Cell::U64(12)]]);
    let reference = result.manifest_ref();
    drop((report, driver, catalog));
    let (reader, _) = catalog_with_store(
        &reg,
        FixedBudget::new(workflow_budget::MEMORY_LIMIT_BYTES),
        store,
    );
    let reopened = reader
        .read_pinned_manifest(reference, &cancel)
        .await
        .unwrap();
    assert_eq!(values(&reopened), vec![vec![Cell::U64(12)]]);
    assert_eq!(reopened.parents().len(), 2);
}
