// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Public terminal-attempt protocol under actual catalog admission. The fixture keeps
//! production relation/enum/document contracts and all seven terminal invariants;
//! unrelated model invariants are deliberately outside this bounded protocol test.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "typed integration fixture assertions"
)]

#[path = "../../lifecycle/src/fault_store.rs"]
mod fault_store;

use datafusion::arrow::array::RecordBatch;
use fault_store::{Fault, FaultPlan, FaultStore};
use pse_authoring::{
    ParseBudget,
    document::{DocumentBundle, load_package_texts},
};
use pse_catalog::computation::ProducedStage;
use pse_catalog::{
    Catalog, EncodingPolicy, ExecutionSettings, FixedClock, RefName, RelationContract, Snapshot,
    ThreadBudget, TrustLevel,
    session::{SessionFactory, native_engine_profile},
    store::{
        membership::AdmissionContext,
        publish::{BundleDraft, RelationDraft},
        sidecar::SidecarArtifact,
    },
};
use pse_compiler::{
    CompilerError, InputBundle, Pass, PassContext, PolicySet,
    driver::{CommitBase, CommitRequest, Driver, PipelineRequest},
};
use pse_ids::{
    CancellationToken, CanonError, MemoryReserver, Reservation, ReserveError, SemanticId,
    SnapshotId, SnapshotKind,
};
use pse_relations::generated::authored;
use pse_runtime::{ResourceBudget, SharedRuntime};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, CmpOp, Determinism, EnumDecl, FieldContract, InputPort, InvariantDecl,
        InvariantKind, Namespace, NullEquality, PassDecl, PassSpec, PortSource, RelationDecl,
        RuleDecl, RuleExpr, RuleHead, RulePlan, SnapshotClass,
    },
};
use std::{
    collections::BTreeMap,
    num::NonZeroUsize,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

fn registry() -> Arc<Registry> {
    static REGISTRY: OnceLock<Arc<Registry>> = OnceLock::new();
    Arc::clone(REGISTRY.get_or_init(|| {
        let original = pse_schema::registry().unwrap();
        let mut builder = RegistryBuilder::new();
        for relation in original.relations() {
            builder.declare_relation(RelationDecl {
                key: relation.key,
                authority: relation.authority,
                snapshot_class: relation.snapshot_class,
                derivation_granularity: relation.derivation_granularity,
                stability: relation.stability,
                primary_key: relation.primary_key.clone(),
                columns: relation.columns.clone(),
                doc: relation.doc,
            });
        }
        for enumeration in original.enums() {
            builder.declare_enum(EnumDecl {
                name: enumeration.name,
                idaes_source: enumeration.idaes_source,
                members: enumeration.members.clone(),
            });
        }
        for document in original.documents() {
            builder.declare_document(document.clone());
        }
        builder.declare_manifest(original.manifest().unwrap().clone());
        for pass in original.passes() {
            builder.declare_pass(PassDecl {
                name: pass.name,
                version: pass.version,
                inputs: pass.inputs.clone(),
                outputs: pass.outputs.clone(),
                preconditions: vec![],
                postconditions: vec![],
                determinism: pass.determinism,
                diagnostics: pass.diagnostics.clone(),
                effects: pass.effects.clone(),
            });
        }
        let mut terminal_count = 0;
        for invariant in original.invariants().iter().filter(|invariant| {
            invariant.relation == "provenance.pass_records"
                && invariant.name.starts_with("check:terminal_")
        }) {
            terminal_count += 1;
            let rule = original.rule(&invariant.rule).unwrap();
            builder.declare_rule(RuleDecl {
                name: rule.name.clone(),
                version: rule.version,
                stratum: rule.stratum,
                head: rule.head.clone(),
                assertion_relation: rule.assertion_relation.clone(),
                plan: rule.plan.clone(),
                negation: rule.negation,
                monotonic: rule.monotonic,
                conflict_policy: rule.conflict_policy,
            });
            builder.declare_invariant(InvariantDecl {
                name: invariant.name.clone(),
                relation: invariant.relation.clone(),
                kind: invariant.kind,
                rule: invariant.rule.clone(),
                severity: invariant.severity,
                doc: invariant.doc,
            });
        }
        assert_eq!(
            terminal_count, 7,
            "the complete production terminal invariant set is retained"
        );
        positive_mw(&mut builder);
        joined_precondition(&mut builder);
        for name in ["FixtureFailure", "FixtureCancel", "FixtureMissing"] {
            let mut declaration = PassDecl::new(name, "1", Determinism::Deterministic)
                .diagnostics(vec!["internal.invariant", "runtime.cancelled"]);
            if name == "FixtureMissing" {
                declaration.inputs.push(InputPort {
                    port: "required",
                    relation: "compiled.math_expr_nodes".to_owned(),
                    source: PortSource::Pinned,
                    required: true,
                });
            }
            builder.declare_pass(declaration);
        }
        Arc::new(builder.build().unwrap())
    }))
}
fn positive_mw(builder: &mut RegistryBuilder) {
    let name = "fixture_positive_mw";
    builder.declare_rule(RuleDecl::new(
        name,
        "1",
        1,
        RuleHead::Violations {
            of: "authored.species".to_owned(),
            key_columns: vec!["species_id"],
        },
        RulePlan::Project {
            input: Box::new(RulePlan::Filter {
                input: Box::new(RulePlan::Scan {
                    relation: "authored.species".to_owned(),
                    port: "subject",
                }),
                predicate: RuleExpr::cmp(
                    CmpOp::LtEq,
                    RuleExpr::col("mw"),
                    RuleExpr::Lit(Cell::F64(0.0)),
                ),
            }),
            columns: (vec![("species_id", RuleExpr::col("species_id"))])
                .into_iter()
                .map(|(name, expression)| (name.into(), expression))
                .collect(),
        },
    ));
    builder.declare_invariant(InvariantDecl::error(
        "authored.species",
        name,
        InvariantKind::Check,
        format!("{name}@1"),
        "Bounded test precondition: supplied molecular weight is positive.",
    ));
}

#[derive(Debug)]
struct RecordBudget {
    inner: Arc<dyn MemoryReserver>,
    deny: Arc<AtomicBool>,
}
impl MemoryReserver for RecordBudget {
    fn open(&self, owner: &str) -> Box<dyn Reservation> {
        Box::new(RecordReservation {
            inner: self.inner.open(owner),
            terminal: owner == "compiler:terminal-record",
            deny: Arc::clone(&self.deny),
        })
    }
}
#[derive(Debug)]
struct RecordReservation {
    inner: Box<dyn Reservation>,
    terminal: bool,
    deny: Arc<AtomicBool>,
}
impl Reservation for RecordReservation {
    fn try_grow(&mut self, bytes: usize) -> Result<(), ReserveError> {
        if self.terminal && self.deny.load(Ordering::SeqCst) {
            return Err(ReserveError::Exhausted {
                owner: "compiler:terminal-record".to_owned(),
                requested: bytes,
                reserved: self.inner.size(),
                limit_hint: "injected terminal reservation limit".to_owned(),
            });
        }
        self.inner.try_grow(bytes)
    }
    fn shrink(&mut self, bytes: usize) {
        self.inner.shrink(bytes);
    }
    fn size(&self) -> usize {
        self.inner.size()
    }
    fn release(&mut self) {
        self.inner.release();
    }
}
struct Fixture {
    registry: Arc<Registry>,
    catalog: Arc<Catalog>,
    store: Arc<FaultStore>,
    deny_record: Arc<AtomicBool>,
    calls: BTreeMap<String, Arc<AtomicUsize>>,
    _spill: tempfile::TempDir,
}
impl Fixture {
    fn new() -> Self {
        let registry = registry();
        let spill = tempfile::tempdir().unwrap();
        let one = NonZeroUsize::new(1).unwrap();
        let threads = ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        };
        let runtime = SharedRuntime::build(ResourceBudget {
            memory_limit_bytes: NonZeroUsize::new(512 << 20).unwrap(),
            spill_dir: spill.path().to_path_buf(),
            max_temp_dir_bytes: 1 << 30,
            top_consumers: NonZeroUsize::new(16).unwrap(),
            threads,
            execution: ExecutionSettings::default(),
            hashing_may_use_pool: false,
        })
        .unwrap();
        let deny_record = Arc::new(AtomicBool::new(false));
        let reserver: Arc<dyn MemoryReserver> = Arc::new(RecordBudget {
            inner: runtime.reserver(),
            deny: Arc::clone(&deny_record),
        });
        let sessions = Arc::new(
            SessionFactory::new(
                runtime.runtime_env(),
                Arc::clone(&reserver),
                ExecutionSettings::default(),
                threads,
                native_engine_profile(),
            )
            .unwrap(),
        );
        let invariants = pse_rules::validator::InvariantValidator::new(Arc::clone(&registry));
        let mut validator =
            pse_compiler::validator::CompilerValidator::new(Arc::new(invariants), &registry)
                .unwrap();
        let mut calls = BTreeMap::new();
        for name in ["FixtureFailure", "FixtureCancel", "FixtureMissing"] {
            let counter = Arc::new(AtomicUsize::new(0));
            validator
                .register(
                    Arc::new(FailingPass {
                        spec: registry.pass(name).unwrap().clone(),
                        calls: Arc::clone(&counter),
                        findings: findings(&registry).0,
                        cancel_during: name == "FixtureCancel",
                    }),
                    &registry,
                )
                .unwrap();
            calls.insert(name.to_owned(), counter);
        }
        let store = FaultStore::new(Arc::new(object_store::memory::InMemory::new()));
        let catalog = Arc::new(
            Catalog::open(
                store.clone(),
                Arc::clone(&registry),
                TrustLevel::Untrusted,
                Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
                Arc::clone(&sessions),
            )
            .with_semantic_validator(Arc::new(validator.with_sessions(Arc::clone(&sessions)))),
        );
        Self {
            registry,
            catalog,
            store,
            deny_record,
            calls,
            _spill: spill,
        }
    }
    fn driver(&self) -> Driver {
        Driver::new(Arc::clone(&self.catalog)).unwrap()
    }
    async fn source_free_model(&self) -> Arc<Snapshot> {
        self.source_free_value_model(1).await
    }
    async fn source_free_value_model(&self, value: u64) -> Arc<Snapshot> {
        // Only the declared synthetic left/right fixture rows are present. No
        // production entities/documents are claimed; schema rows come from this
        // exact registry. Complete actual semantic admission stays enabled.
        let mut relations = BTreeMap::new();
        for spec in self
            .registry
            .relations()
            .iter()
            .filter(|spec| spec.snapshot_class == SnapshotClass::Model)
        {
            let fixture_rows = vec![vec![Cell::U64(1), Cell::U64(value)]];
            let rows = if matches!(spec.key.name, "terminal_left" | "terminal_right") {
                fixture_rows.as_slice()
            } else {
                self.registry
                    .schema_rows_ref()
                    .iter()
                    .find(|(key, _)| key == &spec.key)
                    .map_or(&[][..], |(_, rows)| rows.as_slice())
            };
            let batch = pse_relations::cells::batch_from_cells_owned(
                &self.registry,
                spec,
                rows,
                self.catalog.reserver().as_ref(),
                &CancellationToken::new(),
            )
            .unwrap();
            relations.insert(
                pse_ids::model_port_name(spec.key.namespace.as_str(), spec.id),
                RelationDraft {
                    contract: Arc::new(
                        RelationContract::from_spec(&self.registry, spec, EncodingPolicy::IpcFile)
                            .unwrap(),
                    ),
                    batches: vec![batch],
                },
            );
        }
        let context = AdmissionContext::default();
        let manifest = self
            .catalog
            .manifest_template(SnapshotKind::Model, &context)
            .unwrap();
        self.catalog
            .publish_bundle(
                BundleDraft {
                    manifest,
                    context,
                    relations,
                },
                &CancellationToken::new(),
            )
            .await
            .unwrap()
    }
    fn fault(&self, prefix: String, call: usize, fault: Fault) {
        self.store.arm(FaultPlan {
            operation: "put",
            prefix,
            call,
            fault,
        });
    }
}

fn record_prefix() -> String {
    pse_catalog::relation_path(
        "provenance",
        "pass_records",
        pse_ids::SchemaVersion(pse_relations::generated::provenance::pass_records::VERSION),
        &pse_ids::encoding_checksum(b"path fixture"),
        pse_catalog::EncodingFormat::ArrowIpcFile,
    )
    .parent()
    .unwrap()
    .to_string()
}
fn decode_record(fixture: &Fixture, artifact: &SidecarArtifact) -> Vec<Cell> {
    let spec = fixture
        .registry
        .relation("provenance.pass_records")
        .unwrap();
    let rows = pse_relations::cells::cells_from_batch(
        &fixture.registry,
        spec,
        artifact.relation().batch(),
    )
    .unwrap();
    assert_eq!(rows.len(), 1);
    rows.into_iter().next().unwrap()
}
async fn checked_record(
    fixture: &Fixture,
    artifact: &SidecarArtifact,
    pass: &str,
    status: &'static str,
    class: Option<&'static str>,
    output: Option<SnapshotId>,
) -> Vec<Cell> {
    let row = decode_record(fixture, artifact);
    assert_eq!(row[1], Cell::Id(fixture.registry.pass(pass).unwrap().id));
    assert_eq!(row[2], Cell::text("1"));
    assert_eq!(
        row[4],
        output.map_or(Cell::Null, |id| Cell::Hash(id.content_hash()))
    );
    assert_eq!(row[11], Cell::Enum(status));
    assert_eq!(row[13], class.map_or(Cell::Null, Cell::Enum));
    assert!(matches!(
        &row[12],
        Cell::List(findings) if row[10] == Cell::U64(findings.len() as u64)
    ));
    assert!(matches!(
        row[9],
        Cell::F64(duration) if duration.is_finite() && duration >= 0.0
    ));
    let reopened = fixture
        .catalog
        .read_sidecar(artifact.reference(), &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        decode_record(fixture, &reopened),
        row,
        "stored bytes re-admit to exactly the same typed observation"
    );
    row
}
fn findings(registry: &Registry) -> (RecordBatch, Vec<Cell>) {
    let check = registry
        .invariants()
        .iter()
        .find(|check| check.name == "fixture_positive_mw")
        .unwrap()
        .id;
    let rows = [(1_u8, "error"), (2, "warning")].map(|(id, severity)| {
        vec![
            Cell::Id(SemanticId::from_bytes([id; 16])),
            Cell::Null,
            Cell::Id(SemanticId::from_bytes([3; 16])),
            Cell::Id(check),
            Cell::Enum(severity),
            Cell::List(vec![Cell::Id(SemanticId::from_bytes([4; 16]))]),
            Cell::text("{\"actual\":7}"),
            Cell::text(format!("retained finding {id}")),
            Cell::List(vec![Cell::text("retain this suggestion")]),
        ]
    });
    let spec = registry.relation("runtime.diagnostics_findings").unwrap();
    (
        pse_relations::cells::batch_from_cells(registry, spec, &rows).unwrap(),
        rows.into_iter().map(Cell::Struct).collect(),
    )
}
struct FailingPass {
    spec: PassSpec,
    calls: Arc<AtomicUsize>,
    findings: RecordBatch,
    cancel_during: bool,
}
impl Pass for FailingPass {
    fn spec(&self) -> &PassSpec {
        &self.spec
    }
    fn run<'a>(
        &'a self,
        ctx: &'a PassContext<'a>,
        _: &'a InputBundle,
    ) -> pse_catalog::BoxFut<'a, Result<ProducedStage, CompilerError>> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.cancel_during {
                ctx.cancel.cancel();
                return Err(CompilerError::Cancelled {
                    findings: vec![self.findings.clone()],
                });
            }
            Err(CompilerError::Postcondition {
                pass: self.spec.name.to_owned(),
                findings: vec![self.findings.clone()],
            })
        })
    }
}
fn request(snapshot: Arc<Snapshot>, pass: &str) -> PipelineRequest {
    PipelineRequest {
        through: pass.to_owned(),
        snapshot,
        policies: PolicySet::default(),
        reuse: false,
    }
}
fn failure(fixture: &Fixture, name: &str) -> (Arc<AtomicUsize>, Vec<Cell>) {
    let calls = Arc::clone(&fixture.calls[name]);
    calls.store(0, Ordering::SeqCst);
    (calls, findings(&fixture.registry).1)
}

fn producer_error(error: &CompilerError) -> &CompilerError {
    match error {
        CompilerError::Catalog(pse_catalog::CatalogError::Semantic(source)) => {
            let source: &(dyn std::error::Error + 'static) = source.as_ref();
            source
                .downcast_ref::<CompilerError>()
                .map_or(error, producer_error)
        }
        _ => error,
    }
}

#[tokio::test]
async fn driver_failed_and_mid_cancelled_passes_keep_every_typed_finding() {
    let fixture = Fixture::new();
    let model = fixture.source_free_model().await;
    for (name, cancelled, status, class) in [
        ("FixtureFailure", false, "failed", "internal.invariant"),
        ("FixtureCancel", true, "cancelled", "runtime.cancelled"),
    ] {
        let mut driver = fixture.driver();
        let (calls, expected) = failure(&fixture, name);
        let cancel = CancellationToken::new();
        let error = driver
            .run(request(Arc::clone(&model), name), &cancel)
            .await
            .err()
            .expect("the operation must fail");
        let CompilerError::AttemptFailed {
            pass_run_id,
            record,
            source,
            ..
        } = error
        else {
            panic!("expected durable terminal failure, got {error}")
        };
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(cancel.is_cancelled(), cancelled);
        let row = checked_record(&fixture, &record, name, status, Some(class), None).await;
        assert_eq!(row[0], Cell::Id(pass_run_id));
        assert_eq!(row[3], Cell::Hash(model.snapshot_id().content_hash()));
        assert_eq!(row[12], Cell::List(expected));
        assert!(matches!(
            producer_error(&source),
            CompilerError::Cancelled { .. } | CompilerError::Postcondition { .. }
        ));
    }
}

#[tokio::test]
async fn driver_pre_cancel_and_missing_required_input_are_recorded_before_body_runs() {
    let fixture = Fixture::new();
    let model = fixture.source_free_model().await;
    for (name, pre_cancel, status, class) in [
        ("FixtureFailure", true, "cancelled", "runtime.cancelled"),
        ("FixtureMissing", false, "failed", "internal.invariant"),
    ] {
        let mut driver = fixture.driver();
        let (calls, _) = failure(&fixture, name);
        let cancel = CancellationToken::new();
        if pre_cancel {
            cancel.cancel();
        }
        let error = driver
            .run(request(Arc::clone(&model), name), &cancel)
            .await
            .err()
            .expect("the operation must fail");
        let CompilerError::AttemptFailed { record, source, .. } = error else {
            panic!("expected terminal failure, got {error}")
        };
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let row = checked_record(&fixture, &record, name, status, Some(class), None).await;
        let Cell::List(findings) = &row[12] else {
            panic!("findings")
        };
        assert_eq!(findings.len(), 1);
        let Cell::Struct(finding) = &findings[0] else {
            panic!("typed finding")
        };
        assert_eq!(
            finding[3],
            Cell::Null,
            "an execution-origin diagnostic is never assigned a fake check ID"
        );
        assert_eq!(finding[4], Cell::Enum("error"));
        if pre_cancel {
            assert!(matches!(
                *source,
                CompilerError::Canon(CanonError::Cancelled)
            ));
        }
    }
}

#[tokio::test]
async fn failed_terminal_storage_and_reservation_keep_original_execution_error() {
    let fixture = Fixture::new();
    let model = fixture.source_free_model().await;
    for budget_failure in [false, true] {
        let mut driver = fixture.driver();
        let (_, expected) = failure(&fixture, "FixtureFailure");
        if budget_failure {
            fixture.deny_record.store(true, Ordering::SeqCst);
        } else {
            fixture.fault(record_prefix(), 1, Fault::FailBefore);
        }
        let error = driver
            .run(
                request(Arc::clone(&model), "FixtureFailure"),
                &CancellationToken::new(),
            )
            .await
            .err()
            .expect("the operation must fail");
        fixture.deny_record.store(false, Ordering::SeqCst);
        let CompilerError::TerminalRecording {
            pass_run_id,
            errors,
        } = error
        else {
            panic!("expected both original and recording error, got {error}")
        };
        assert_ne!(pass_run_id, SemanticId::NIL);
        assert_eq!(errors.len(), 2);
        let CompilerError::Postcondition { findings, .. } = producer_error(&errors[0]) else {
            panic!("original typed execution error lost")
        };
        let spec = fixture
            .registry
            .relation("runtime.diagnostics_findings")
            .unwrap();
        let rows =
            pse_relations::cells::cells_from_batch(&fixture.registry, spec, &findings[0]).unwrap();
        assert_eq!(
            rows.into_iter().map(Cell::Struct).collect::<Vec<_>>(),
            expected
        );
        if budget_failure {
            assert!(matches!(
                errors[1],
                CompilerError::Canon(CanonError::Reservation(_))
            ));
        } else {
            assert_eq!(fixture.store.fired(), 1);
            assert!(matches!(errors[1], CompilerError::Catalog(_)));
        }
    }
}

fn source(registry: &Registry, bad_mw: bool) -> DocumentBundle {
    let mut documents = BTreeMap::from([(
        "package.toml".to_owned(),
        include_str!("../../fixtures/packages/minimal_explicit/package.toml").to_owned(),
    )]);
    if bad_mw {
        documents.insert(
            "materials/species.yaml".to_owned(),
            include_str!("../../fixtures/packages/minimal_explicit/materials/species.yaml")
                .replace("mw: 0.01801528", "mw: -1.0"),
        );
    }
    load_package_texts(documents, registry, ParseBudget::default()).unwrap()
}
fn commit_request(
    fixture: &Fixture,
    reference: &RefName,
    base: Option<CommitBase>,
    bad_mw: bool,
) -> CommitRequest {
    let revision = base.as_ref().map_or(SemanticId::NIL, |base| {
        base.observed.revision_ref().unwrap().revision_id
    });
    CommitRequest {
        reference: reference.clone(),
        revision_ids: None,
        base,
        documents: vec![source(&fixture.registry, bad_mw)],
        header: authored::change_sets::Row {
            change_set_id: pse_authoring::ids::uuid_v7(),
            base_revision_id: revision,
            author: "terminal fixture".to_owned(),
            message: "observe final publication boundary".to_owned(),
            created_at: 1_000_000_000,
        },
        changes: None,
    }
}
async fn base_commit(fixture: &Fixture, driver: &mut Driver, reference: &RefName) -> CommitBase {
    let report = driver
        .commit(
            commit_request(fixture, reference, None, false),
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(report.validation.error_count(), 0);
    assert_eq!(report.attempts.len(), 3);
    let tip = report.tip.unwrap();
    for (index, name) in ["P0", "P1", "P2"].into_iter().enumerate() {
        checked_record(
            fixture,
            &report.attempts[index],
            name,
            "ok",
            None,
            (name == "P2").then(|| tip.snapshot_id()),
        )
        .await;
    }
    let observed = fixture
        .catalog
        .read_ref(reference, &CancellationToken::new())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(observed.manifest_ref(), tip.manifest_ref());
    CommitBase {
        observed,
        snapshot: tip,
    }
}
async fn unchanged_ref(fixture: &Fixture, reference: &RefName, base: &CommitBase) {
    let observed = fixture
        .catalog
        .read_ref(reference, &CancellationToken::new())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(observed.manifest_ref(), base.observed.manifest_ref());
    assert_eq!(observed.revision_ref(), base.observed.revision_ref());
}

#[tokio::test]
async fn commit_pre_cancel_and_unresolved_dependency_publish_truthful_p0_terminal_records() {
    let fixture = Fixture::new();
    let reference = RefName::parse("terminal-p0").unwrap();
    for pre_cancel in [true, false] {
        let mut driver = fixture.driver();
        let mut request = commit_request(&fixture, &reference, None, false);
        let cancel = CancellationToken::new();
        if pre_cancel {
            cancel.cancel();
        } else {
            let mut texts = request.documents[0]
                .documents
                .iter()
                .map(|document| (document.path.clone(), document.text.clone()))
                .collect::<BTreeMap<_, _>>();
            let package = texts.get_mut("package.toml").unwrap();
            *package = package.replace("dependencies = []", "dependencies = [{ package_id = 'ffffffffffffffffffffffffffffffff', version_req = '=1.0.0' }]");
            request.documents[0] =
                load_package_texts(texts, &fixture.registry, ParseBudget::default()).unwrap();
        }
        let error = driver
            .commit(request, &cancel)
            .await
            .err()
            .expect("the operation must fail");
        let CompilerError::AttemptFailed { record, source, .. } = error else {
            panic!("expected P0 terminal diagnostic, got {error}")
        };
        let (status, class) = if pre_cancel {
            ("cancelled", "runtime.cancelled")
        } else {
            ("failed", "authoring.reference")
        };
        checked_record(&fixture, &record, "P0", status, Some(class), None).await;
        assert!(matches!(
            *source,
            CompilerError::Canon(CanonError::Cancelled) | CompilerError::Authoring(_)
        ));
        assert!(
            fixture
                .catalog
                .read_ref(&reference, &CancellationToken::new())
                .await
                .unwrap()
                .is_none()
        );
    }
}

#[tokio::test]
async fn commit_p2_violation_retains_actual_rule_finding_and_leaves_old_ref() {
    let fixture = Fixture::new();
    let mut driver = fixture.driver();
    let reference = RefName::parse("terminal-p2").unwrap();
    let base = base_commit(&fixture, &mut driver, &reference).await;
    fixture.store.clear_trace();
    let report = driver
        .commit(
            commit_request(&fixture, &reference, Some(base.clone()), true),
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(report.validation.error_count(), 1);
    assert!(report.model.is_none() && report.tip.is_none() && report.revision_id.is_none());
    assert_eq!(report.attempts.len(), 3);
    let row = checked_record(
        &fixture,
        &report.attempts[2],
        "P2",
        "failed",
        Some("validation.invariant"),
        None,
    )
    .await;
    let spec = fixture
        .registry
        .relation("runtime.diagnostics_findings")
        .unwrap();
    let actual = report
        .validation
        .findings()
        .iter()
        .flat_map(|batch| {
            pse_relations::cells::cells_from_batch(&fixture.registry, spec, batch.batch()).unwrap()
        })
        .map(Cell::Struct)
        .collect::<Vec<_>>();
    assert_eq!(row[12], Cell::List(actual.clone()));
    let invariant = fixture
        .registry
        .invariants()
        .iter()
        .find(|invariant| invariant.name == "fixture_positive_mw")
        .unwrap();
    let Cell::Struct(finding) = &actual[0] else {
        panic!("typed rule finding")
    };
    assert_eq!(finding[3], Cell::Id(invariant.id));
    assert_eq!(finding[4], Cell::Enum("error"));
    unchanged_ref(&fixture, &reference, &base).await;
    assert!(
        !fixture
            .store
            .put_trace()
            .iter()
            .any(|path| path == pse_catalog::ref_path(&reference).as_ref())
    );
}

#[tokio::test]
async fn successful_p2_cannot_move_ref_when_its_terminal_record_storage_fails() {
    let fixture = Fixture::new();
    let mut driver = fixture.driver();
    let reference = RefName::parse("terminal-record-failure").unwrap();
    let base = base_commit(&fixture, &mut driver, &reference).await;
    fixture.fault(record_prefix(), 3, Fault::FailBefore);
    let error = driver
        .commit(
            commit_request(&fixture, &reference, Some(base.clone()), false),
            &CancellationToken::new(),
        )
        .await
        .err()
        .expect("the operation must fail");
    let CompilerError::SuccessRecording {
        pass_run_id,
        output,
        source,
    } = error
    else {
        panic!("expected P2 terminal recording failure, got {error}")
    };
    assert_ne!(pass_run_id, SemanticId::NIL);
    assert!(
        output.is_some(),
        "complete immutable P2 result preceded its terminal write"
    );
    assert!(matches!(*source, CompilerError::Catalog(_)));
    assert_eq!(fixture.store.fired(), 1);
    let writes = fixture.store.put_trace();
    assert_eq!(
        writes
            .iter()
            .filter(|path| path.starts_with(&record_prefix()))
            .count(),
        3
    );
    assert!(
        !writes
            .iter()
            .any(|path| path == pse_catalog::ref_path(&reference).as_ref()),
        "no final CAS may precede a successful P2 terminal record"
    );
    unchanged_ref(&fixture, &reference, &base).await;
}

#[tokio::test]
async fn final_cas_failure_retains_the_already_successful_p2_record() {
    let fixture = Fixture::new();
    let mut driver = fixture.driver();
    let reference = RefName::parse("terminal-cas-failure").unwrap();
    let base = base_commit(&fixture, &mut driver, &reference).await;
    let ref_path = pse_catalog::ref_path(&reference).to_string();
    fixture.fault(ref_path.clone(), 1, Fault::Precondition);
    let error = driver
        .commit(
            commit_request(&fixture, &reference, Some(base.clone()), false),
            &CancellationToken::new(),
        )
        .await
        .err()
        .expect("the operation must fail");
    let CompilerError::CommitPublication {
        pass_run_id,
        output,
        record,
        source,
    } = error
    else {
        panic!("expected recorded success followed by CAS failure, got {error}")
    };
    let row = checked_record(&fixture, &record, "P2", "ok", None, Some(output)).await;
    assert_eq!(row[0], Cell::Id(pass_run_id));
    assert!(matches!(*source, CompilerError::Catalog(_)));
    assert_eq!(fixture.store.fired(), 1);
    let writes = fixture.store.put_trace();
    let record_position = writes
        .iter()
        .rposition(|path| path.starts_with(&record_prefix()))
        .unwrap();
    let cas_position = writes.iter().position(|path| path == &ref_path).unwrap();
    assert!(
        record_position < cas_position,
        "the exact terminal artifact write precedes the sole ref mutation"
    );
    assert_eq!(
        writes
            .iter()
            .filter(|path| path.starts_with(&record_prefix()))
            .count(),
        3
    );
    unchanged_ref(&fixture, &reference, &base).await;
}

fn joined_precondition(builder: &mut RegistryBuilder) {
    for name in ["terminal_left", "terminal_right"] {
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                name,
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "Explicit source-free terminal protocol fixture values.",
            )
            .pk(&["record_id"])
            .columns(vec![
                FieldContract::key(
                    "record_id",
                    FieldContract::native(arrow::datatypes::DataType::UInt64),
                    "actual key",
                ),
                FieldContract::payload(
                    "value",
                    FieldContract::native(arrow::datatypes::DataType::UInt64),
                    "actual value",
                ),
            ]),
        );
    }
    let left = RulePlan::Scan {
        relation: "authored.terminal_left".to_owned(),
        port: "left",
    };
    let right = RulePlan::Project {
        input: Box::new(RulePlan::Scan {
            relation: "authored.terminal_right".to_owned(),
            port: "right",
        }),
        columns: (vec![
            ("other_id", RuleExpr::col("record_id")),
            ("other_value", RuleExpr::col("value")),
        ])
        .into_iter()
        .map(|(name, expression)| (name.into(), expression))
        .collect(),
    };
    let joined = RulePlan::EquiJoin {
        left: Box::new(left),
        right: Box::new(right),
        keys: (vec![("record_id", "other_id")])
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
        null_equality: NullEquality::NullEqualsNothing,
    };
    let violations = RulePlan::Project {
        input: Box::new(RulePlan::Filter {
            input: Box::new(joined),
            predicate: RuleExpr::cmp(
                CmpOp::NotEq,
                RuleExpr::col("value"),
                RuleExpr::col("other_value"),
            ),
        }),
        columns: (vec![("record_id", RuleExpr::col("record_id"))])
            .into_iter()
            .map(|(name, expression)| (name.into(), expression))
            .collect(),
    };
    builder.declare_rule(RuleDecl::new(
        "fixture_join_matches",
        "1",
        1,
        RuleHead::Violations {
            of: "authored.terminal_left".to_owned(),
            key_columns: vec!["record_id"],
        },
        violations,
    ));
    builder.declare_invariant(InvariantDecl::error(
        "authored.terminal_left",
        "fixture_join_matches",
        InvariantKind::Check,
        "fixture_join_matches@1",
        "A bounded fixture precondition checks actual paired values.",
    ));
}
