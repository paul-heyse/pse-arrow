// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared test harness for actual source commits and target pass execution.
use pse_authoring::document::DocumentBundle;
use pse_catalog::{
    Catalog, ExecutionSettings, FixedClock, RefName, Snapshot, ThreadBudget, TrustLevel,
    session::{SessionFactory, native_engine_profile},
};
use pse_compiler::{
    PolicySet,
    driver::{CommitRequest, Driver, PipelineReport, PipelineRequest},
};
use pse_ids::{CancellationToken, MemoryReserver, SemanticId};
use pse_relations::generated::authored;
use pse_runtime::{ResourceBudget, SharedRuntime};
use pse_schema::Registry;
use std::{num::NonZeroUsize, sync::Arc};

#[path = "workflow_budget.rs"]
mod workflow_budget;

pub(crate) struct Fixture {
    pub(crate) registry: Arc<Registry>,
    pub(crate) catalog: Arc<Catalog>,
    pub(crate) driver: Driver,
    #[allow(
        dead_code,
        reason = "only cold-reopening tests construct another catalog"
    )]
    store: Arc<object_store::memory::InMemory>,
    #[allow(
        dead_code,
        reason = "only cold-reopening tests construct another catalog"
    )]
    sessions: Arc<SessionFactory>,
    _spill: tempfile::TempDir,
}
impl Fixture {
    pub(crate) fn new() -> Self {
        let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
        let spill = tempfile::tempdir().unwrap();
        let one = NonZeroUsize::new(1).unwrap();
        let threads = ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        };
        let runtime = SharedRuntime::build(ResourceBudget {
            memory_limit_bytes: NonZeroUsize::new(workflow_budget::MEMORY_LIMIT_BYTES).unwrap(),
            spill_dir: spill.path().to_path_buf(),
            max_temp_dir_bytes: 1 << 30,
            top_consumers: NonZeroUsize::new(16).unwrap(),
            threads,
            execution: ExecutionSettings::default(),
            hashing_may_use_pool: false,
        })
        .unwrap();
        let reserver: Arc<dyn MemoryReserver> = runtime.reserver();
        let sessions = Arc::new(
            SessionFactory::new(
                runtime.runtime_env(),
                Arc::clone(&reserver),
                ExecutionSettings::default(),
                threads,
                native_engine_profile(),
            )
            .unwrap()
            .with_query_planner(pse_compiler::query_planner()),
        );
        let invariants = pse_rules::validator::InvariantValidator::new(Arc::clone(&registry));
        let validator =
            pse_compiler::validator::CompilerValidator::new(Arc::new(invariants), &registry)
                .unwrap()
                .with_sessions(Arc::clone(&sessions));
        let store = Arc::new(object_store::memory::InMemory::new());
        let catalog = Arc::new(
            Catalog::open(
                store.clone(),
                Arc::clone(&registry),
                TrustLevel::Untrusted,
                Arc::new(FixedClock("2026-09-14T00:00:00Z".into())),
                Arc::clone(&sessions),
            )
            .with_semantic_validator(Arc::new(validator)),
        );
        let driver = Driver::new(Arc::clone(&catalog)).unwrap();
        Self {
            registry,
            catalog,
            driver,
            store,
            sessions,
            _spill: spill,
        }
    }
    #[allow(dead_code, reason = "shared source fixtures do not all reopen outputs")]
    pub(crate) fn reader(&self) -> Catalog {
        let invariants = pse_rules::validator::InvariantValidator::new(self.registry.clone());
        let validator =
            pse_compiler::validator::CompilerValidator::new(Arc::new(invariants), &self.registry)
                .unwrap()
                .with_sessions(self.sessions.clone());
        Catalog::open(
            self.store.clone(),
            self.registry.clone(),
            TrustLevel::Untrusted,
            Arc::new(FixedClock("2026-09-14T00:00:00Z".into())),
            Arc::clone(&self.sessions),
        )
        .with_semantic_validator(Arc::new(validator))
    }
    pub(crate) async fn commit(&mut self, documents: Vec<DocumentBundle>) -> Arc<Snapshot> {
        assert!(Arc::ptr_eq(&self.registry, self.catalog.registry()));
        let report = self
            .driver
            .commit(
                CommitRequest {
                    revision_ids: None,
                    reference: RefName::parse("semantic_fixture").unwrap(),
                    base: None,
                    documents,
                    changes: None,
                    header: authored::change_sets::Row {
                        change_set_id: pse_authoring::ids::uuid_v7(),
                        base_revision_id: SemanticId::NIL,
                        author: "semantic fixture".into(),
                        message: "actual source input".into(),
                        created_at: 1_000_000_000,
                    },
                },
                &CancellationToken::new(),
            )
            .await
            .unwrap_or_else(|error| panic!("source commit failed: {error}"));
        assert_eq!(
            report.validation.error_count(),
            0,
            "{:?}",
            report
                .validation
                .findings()
                .iter()
                .flat_map(|batch| {
                    pse_relations::generated::runtime::diagnostics_findings::View::from_checked(
                        batch,
                    )
                    .unwrap()
                    .rows()
                    .unwrap()
                })
                .map(|row| (row.evidence, row.message))
                .collect::<Vec<_>>()
        );
        report.tip.unwrap()
    }
    #[allow(
        dead_code,
        reason = "source-only integration binaries share this harness without invoking a semantic stage"
    )]
    pub(crate) async fn run(
        &mut self,
        snapshot: Arc<Snapshot>,
        through: &str,
    ) -> Result<PipelineReport, pse_compiler::CompilerError> {
        self.driver
            .run(
                PipelineRequest {
                    through: through.to_owned(),
                    snapshot,
                    policies: PolicySet::default(),
                    reuse: false,
                },
                &CancellationToken::new(),
            )
            .await
    }
}
