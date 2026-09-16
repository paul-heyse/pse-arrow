// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Local sidecar checks never claim cross-artifact receipt or snapshot admission.
#![allow(
    clippy::expect_used,
    reason = "explicit full-registry sidecar fixtures"
)]
#[path = "../../support/workflow_budget.rs"]
mod workflow_budget;

use object_store::memory::InMemory;
use pse_catalog::{
    Catalog, CatalogError, EncodingPolicy, ExecutionSettings, FixedClock, RelationContract,
    ThreadBudget, TrustLevel,
    session::{SessionFactory, native_engine_profile},
    store::{membership::SemanticValidator, publish::RelationDraft},
};
use pse_ids::{CancellationToken, ContentHash, MemoryReserver, SemanticId};
use pse_relations::RecordBatch;
use pse_runtime::{ResourceBudget, SharedRuntime};
use pse_schema::{
    Registry,
    model::{Cell, RelationKey},
};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

struct Fixture {
    registry: Arc<Registry>,
    checked: Catalog,
    forged: Catalog,
    validator: Arc<dyn SemanticValidator>,
    _spill: tempfile::TempDir,
}
fn fixture() -> Fixture {
    let registry = Arc::new(pse_schema::catalog::assemble().expect("full registry"));
    let store = Arc::new(InMemory::new());
    let spill = tempfile::tempdir().expect("spill fixture");
    let one = NonZeroUsize::new(1).expect("one thread");
    let threads = ThreadBudget {
        pool_threads: one,
        target_partitions: one,
    };
    let runtime = SharedRuntime::build(ResourceBudget {
        memory_limit_bytes: NonZeroUsize::new(workflow_budget::MEMORY_LIMIT_BYTES)
            .expect("bounded runtime"),
        spill_dir: spill.path().to_path_buf(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: NonZeroUsize::new(16).expect("consumer reporting"),
        threads,
        execution: ExecutionSettings::default(),
        hashing_may_use_pool: false,
    })
    .expect("actual runtime");
    let reserver: Arc<dyn MemoryReserver> = runtime.reserver();
    let sessions = Arc::new(
        SessionFactory::new(
            runtime.runtime_env(),
            Arc::clone(&reserver),
            ExecutionSettings::default(),
            threads,
            native_engine_profile(),
        )
        .expect("sealed engine"),
    );
    let invariant = pse_rules::validator::InvariantValidator::new(Arc::clone(&registry));
    let validator: Arc<dyn SemanticValidator> = Arc::new(
        pse_compiler::validator::CompilerValidator::new(Arc::new(invariant), &registry)
            .expect("registered producers")
            .with_sessions(Arc::clone(&sessions)),
    );
    let checked_store: Arc<dyn object_store::ObjectStore> = Arc::<InMemory>::clone(&store);
    let checked = Catalog::open(
        checked_store,
        Arc::clone(&registry),
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
        Arc::clone(&sessions),
    )
    .with_semantic_validator(Arc::clone(&validator));
    let forged = Catalog::open(
        store,
        Arc::clone(&registry),
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
        Arc::clone(&sessions),
    )
    .with_semantic_validator(Arc::new(ForgedLocalArtifact));
    Fixture {
        registry,
        checked,
        forged,
        validator,
        _spill: spill,
    }
}

/// Deliberately creates a representation-valid artifact with invalid local semantics.
/// Only reopening through the production validator may establish admission.
#[derive(Debug)]
struct ForgedLocalArtifact;
impl SemanticValidator for ForgedLocalArtifact {
    fn validate<'a>(
        &'a self,
        _: &'a Registry,
        _: &'a BTreeMap<RelationKey, RecordBatch>,
        _session: &'a pse_catalog::session::SnapshotSession,
        _: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async {
            Err(CatalogError::Membership {
                reason: "fixture producer refuses candidate admission".to_owned(),
            })
        })
    }
    fn validate_sidecar<'a>(
        &'a self,
        _: &'a Registry,
        _: &'a BTreeMap<RelationKey, RecordBatch>,
        _session: &'a pse_catalog::session::SnapshotSession,
        _: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async { Ok(()) })
    }
}
fn id(value: u8) -> Cell {
    Cell::Id(SemanticId::from_bytes([value; 16]))
}
fn draft(registry: &Registry, relation: &str, row: &[Cell]) -> RelationDraft {
    let spec = registry.relation(relation).expect("declared sidecar");
    RelationDraft {
        contract: Arc::new(
            RelationContract::from_spec(registry, spec, EncodingPolicy::IpcFile)
                .expect("actual contract"),
        ),
        batches: vec![
            pse_relations::cells::batch_from_cells(registry, spec, &[row.to_vec()])
                .expect("actual typed fields"),
        ],
    }
}
#[tokio::test]
async fn standalone_change_operation_admits_locally_but_not_as_complete_candidate() {
    let fixture = fixture();
    let cancel = CancellationToken::default();
    let row = vec![
        id(20),
        Cell::U64(0),
        Cell::Enum("insert"),
        Cell::Id(
            fixture
                .registry
                .relation("authored.species")
                .expect("target relation")
                .id,
        ),
        Cell::Struct(vec![Cell::text("operation/0/key"), Cell::U64(0)]),
        Cell::Struct(vec![Cell::text("operation/0/key"), Cell::U64(0)]),
        Cell::Null,
    ];
    let artifact = fixture
        .checked
        .publish_sidecar(
            draft(&fixture.registry, "authored.change_ops", &row),
            &cancel,
        )
        .await
        .expect("standalone operation, header is a separate artifact");
    let reopened = fixture
        .checked
        .read_sidecar(artifact.reference(), &cancel)
        .await
        .expect("reopen actual local operation");
    let spec = fixture
        .registry
        .relation("authored.change_ops")
        .expect("operation declaration");
    assert_eq!(
        pse_relations::cells::cells_from_batch(
            &fixture.registry,
            spec,
            reopened.relation().batch()
        )
        .expect("decoded actual operation"),
        vec![row]
    );
    let incomplete = BTreeMap::from([(spec.key, reopened.relation().batch().clone())]);
    let failure = fixture
        .validator
        .validate(
            &fixture.registry,
            &incomplete,
            &fixture
                .checked
                .session_factory()
                .candidate(BTreeMap::new(), Arc::clone(&fixture.registry), &cancel)
                .expect("actual native scope"),
            &cancel,
        )
        .await
        .expect_err("full candidate still requires its actual header relation");
    assert!(
        format!("{failure:?}").contains("authored.change_sets"),
        "{failure:?}"
    );
}
fn success(registry: &Registry) -> Vec<Cell> {
    vec![
        id(1),
        Cell::Id(registry.pass("P0").expect("actual pass").id),
        Cell::text("1"),
        Cell::Null,
        Cell::Null,
        Cell::Null,
        Cell::List(vec![]),
        Cell::List(vec![]),
        Cell::List(vec![]),
        Cell::F64(1.0),
        Cell::U64(0),
        Cell::Enum("ok"),
        Cell::List(vec![]),
        Cell::Null,
        Cell::List(vec![]),
    ]
}
fn failed(registry: &Registry) -> Vec<Cell> {
    let mut row = success(registry);
    row[10] = Cell::U64(1);
    row[11] = Cell::Enum("failed");
    row[13] = Cell::Enum("runtime.infrastructure");
    row[12] = Cell::List(vec![Cell::Struct(vec![
        id(3),
        Cell::Null,
        Cell::Null,
        Cell::Null,
        Cell::Enum("error"),
        Cell::List(vec![]),
        Cell::text("{}"),
        Cell::text("actual failed execution"),
        Cell::List(vec![]),
    ])]);
    row
}
fn invalid_attempts(registry: &Registry) -> Vec<(&'static str, Vec<Cell>)> {
    let mut cases = Vec::new();
    let mut count = success(registry);
    count[10] = Cell::U64(1);
    cases.push(("terminal_count", count));
    let mut class = success(registry);
    class[13] = Cell::Enum("runtime.infrastructure");
    cases.push(("terminal_failure_class", class));
    let mut cancellation = failed(registry);
    cancellation[11] = Cell::Enum("cancelled");
    cases.push(("terminal_cancel_class", cancellation));
    let mut output = failed(registry);
    output[4] = Cell::Hash(ContentHash::from_bytes([9; 32]));
    cases.push(("terminal_failed_output", output));
    let mut absent = failed(registry);
    absent[10] = Cell::U64(0);
    absent[12] = Cell::List(vec![]);
    cases.push(("terminal_failure_finding", absent));
    let mut duration = success(registry);
    duration[9] = Cell::F64(-1.0);
    cases.push(("terminal_duration", duration));
    let mut origin = failed(registry);
    if let Cell::List(findings) = &mut origin[12]
        && let Cell::Struct(fields) = &mut findings[0]
    {
        fields[4] = Cell::Enum("warning");
    }
    cases.push(("terminal_finding_origin", origin));
    cases
}
#[tokio::test]
async fn every_terminal_local_predicate_rechecks_published_and_reopened_values() {
    let fixture = fixture();
    let cancel = CancellationToken::default();
    let name = "provenance.pass_records";
    for row in [success(&fixture.registry), failed(&fixture.registry)] {
        let artifact = fixture
            .checked
            .publish_sidecar(draft(&fixture.registry, name, &row), &cancel)
            .await
            .expect("valid complete typed terminal attempt");
        let reopened = fixture
            .checked
            .read_sidecar(artifact.reference(), &cancel)
            .await
            .expect("valid terminal reopen");
        assert_eq!(
            pse_relations::cells::cells_from_batch(
                &fixture.registry,
                fixture.registry.relation(name).expect("record declaration"),
                reopened.relation().batch()
            )
            .expect("complete actual finding values"),
            vec![row]
        );
    }
    for (rule, row) in invalid_attempts(&fixture.registry) {
        let failure = fixture
            .checked
            .publish_sidecar(draft(&fixture.registry, name, &row), &cancel)
            .await
            .expect_err("invalid local predicate refuses publication");
        assert_check(&fixture.registry, &failure, rule).expect("exact declared rule finding");
        let forged = fixture
            .forged
            .publish_sidecar(draft(&fixture.registry, name, &row), &cancel)
            .await
            .expect("fixture writes actual encoded values with valid identity");
        let failure = fixture
            .checked
            .read_sidecar(forged.reference(), &cancel)
            .await
            .expect_err("matching encoding identity cannot certify local semantics");
        assert_check(&fixture.registry, &failure, rule).expect("exact declared rule finding");
    }
}

fn assert_check(
    registry: &Registry,
    failure: &CatalogError,
    name: &str,
) -> Result<(), CatalogError> {
    let CatalogError::Semantic(diagnostic) = failure else {
        return Err(CatalogError::Internal {
            message: format!("expected actual semantic finding: {failure:?}"),
        });
    };
    let diagnostic: &dyn std::error::Error = diagnostic.as_ref();
    let error = diagnostic
        .downcast_ref::<pse_rules::RuleError>()
        .expect("actual invariant failure type");
    let pse_rules::RuleError::InvariantViolations { findings, .. } = error else {
        return Err(CatalogError::Internal {
            message: format!("expected actual invariant findings: {error:?}"),
        });
    };
    let check = registry
        .invariants()
        .iter()
        .find(|invariant| {
            invariant.relation == "provenance.pass_records"
                && invariant.name == format!("check:{name}")
        })
        .expect("exact terminal check declaration");
    let spec = registry
        .relation("runtime.diagnostics_findings")
        .expect("finding declaration");
    let ids = findings
        .iter()
        .flat_map(|batch| {
            pse_relations::cells::cells_from_batch(registry, spec, batch)
                .expect("actual complete rule findings")
        })
        .map(|row| row[3].clone())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![Cell::Id(check.id)],
        "{name}: exact violated rule identity"
    );
    Ok(())
}
