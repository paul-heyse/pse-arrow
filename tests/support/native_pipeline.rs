// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native logical compositions and owned Arrow results for functional assertions.
#![allow(
    dead_code,
    reason = "shared fixture capabilities vary by integration binary"
)]
#![allow(
    clippy::unwrap_used,
    clippy::print_stderr,
    reason = "test fixtures, independent assertions and opt-in phase measurements"
)]

use datafusion::{arrow::array::Int64Array, common::ResolvedTableReference};
use pse_authoring::document::DocumentBundle;
use pse_catalog::{
    ExecutionSettings, ThreadBudget,
    artifact::ArtifactPlan,
    session::{RelationPlan, SessionFactory, SnapshotSession, native_engine_profile},
};
use pse_compiler::{Algorithm, CompilerError, passes};
use pse_ids::CancellationToken;
use pse_relations::{columnar::FieldCheckedBatch, generated::authored};
use pse_runtime::{ResourceBudget, SharedRuntime};
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};
#[path = "workflow_budget.rs"]
mod workflow_budget;
pub(crate) type Values = BTreeMap<RelationKey, FieldCheckedBatch>;
pub(crate) type Source = (SnapshotSession, RelationPlan);
pub(crate) struct Fixture {
    pub(crate) registry: Arc<Registry>,
    pub(crate) sessions: Arc<SessionFactory>,
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
            cache: pse_runtime::CacheBudget::for_memory(workflow_budget::MEMORY_LIMIT_BYTES),
            hashing_may_use_pool: false,
        })
        .unwrap();
        let sessions = Arc::new(
            runtime
                .session_factory(native_engine_profile())
                .unwrap()
                .with_query_planner(pse_compiler::query_planner())
                .with_requirement_planner(Arc::new(
                    pse_rules::invariants::RegistryRequirementPlanner,
                )),
        );
        Self {
            registry,
            sessions,
            _spill: spill,
        }
    }
    pub(crate) fn source(&self, documents: Vec<DocumentBundle>) -> Source {
        let cancel = CancellationToken::new();
        let rows = pse_authoring::p1::source_batches(&documents, &self.registry).unwrap();
        drop(documents);
        let session = self
            .sessions
            .candidate_checked(
                BTreeMap::from([(
                    authored::documents::RELATION_KEY,
                    rows[&authored::documents::RELATION_ID].clone(),
                )]),
                Arc::clone(&self.registry),
                &cancel,
            )
            .unwrap();
        let name = session
            .table_reference(&authored::documents::RELATION_KEY)
            .unwrap();
        let documents = session
            .relation_plan(&ResolvedTableReference {
                catalog: name.catalog().unwrap().into(),
                schema: name.schema().unwrap().into(),
                table: name.table().into(),
            })
            .unwrap();
        (session, documents)
    }
    pub(crate) async fn plan(
        &self,
        source: Source,
        through: &str,
    ) -> Result<ArtifactPlan, CompilerError> {
        if std::env::var_os("PSE_TEST_PHASE_TIMINGS").is_some() {
            eprintln!("native_pipeline phase=compose_start through={through}");
        }
        let registry = &self.registry;
        // Test cases select the domain algorithms whose outcomes they assert.
        // No stage records, snapshots, parent traversal or durable intermediate state.
        let all: Vec<Arc<dyn Algorithm>> = vec![
            Arc::new(passes::p3::P3::new(registry)?),
            Arc::new(passes::p4::P4::new(registry)?),
            Arc::new(passes::p5::P5::new(registry)?),
            Arc::new(passes::p6::P6::new(registry)?),
            Arc::new(passes::p7::P7::new(registry)?),
            Arc::new(passes::p8::P8::new(registry)?),
            Arc::new(passes::p9::P9::new(registry)?),
            Arc::new(passes::p10::P10::new(registry)?),
        ];
        let count = all.iter().position(|a| a.spec().name == through).unwrap() + 1;
        pse_compiler::native::model::from_documents_using(
            &source.0,
            source.1,
            &CancellationToken::new(),
            all.into_iter().take(count).collect(),
        )
        .await
    }
    pub(crate) async fn evaluate(
        &self,
        source: Source,
        through: &str,
    ) -> Result<Values, CompilerError> {
        let started = std::time::Instant::now();
        let plan = self.plan(source, through).await?;
        if std::env::var_os("PSE_TEST_PHASE_TIMINGS").is_some() {
            eprintln!(
                "native_pipeline phase=compose through={through} seconds={:.6}",
                started.elapsed().as_secs_f64()
            );
        }
        self.capture(&plan).await
    }
    #[expect(
        clippy::too_many_lines,
        reason = "complete target fixture construction and its independent assertions are kept in execution order"
    )]
    pub(crate) async fn roundtrip(&self, values: &Values) -> Values {
        use pse_catalog::{
            artifact::{PublicationTarget, RelationOutput},
            delta::publication::{Publication, PublicationRoot},
        };
        use pse_relations::generated::{enums::PublicationKind, runtime::publications};
        let cancel = CancellationToken::new();
        let session = self
            .sessions
            .candidate_checked(values.clone(), Arc::clone(&self.registry), &cancel)
            .unwrap();
        let mut outputs = BTreeMap::new();
        for (key, batch) in values {
            let name = session.table_reference(key).unwrap();
            let selected = ResolvedTableReference {
                catalog: name.catalog().unwrap().into(),
                schema: name.schema().unwrap().into(),
                table: name.table().into(),
            };
            let plan = session.relation_plan(&selected).unwrap();
            outputs.insert(
                ResolvedTableReference {
                    catalog: "artifact".into(),
                    schema: key.namespace.as_str().into(),
                    table: key.name.into(),
                },
                RelationOutput {
                    relation_id: batch.relation_id(),
                    plan: plan.plan().clone(),
                },
            );
        }
        let plan = ArtifactPlan::new(session, outputs, &cancel).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let base = url::Url::from_directory_path(directory.path()).unwrap();
        let control = base.join("control/").unwrap();
        let destinations = plan
            .outputs()
            .keys()
            .enumerate()
            .map(|(index, name)| {
                (
                    name.clone(),
                    base.join(&format!("members/{index}/")).unwrap(),
                )
            })
            .collect();
        let header = publications::Row {
            workspace_id: pse_authoring::ids::uuid_v7(),
            publication_id: pse_authoring::ids::uuid_v7(),
            parent_publication_id: None,
            attempt_id: pse_authoring::ids::uuid_v7(),
            kind: PublicationKind::Relations,
            inputs: vec![],
            members: vec![],
        };
        let command = plan
            .prepare_publication(
                PublicationTarget {
                    reference: ResolvedTableReference {
                        catalog: "artifact".into(),
                        schema: "runtime".into(),
                        table: "publications".into(),
                    },
                    location: control.clone(),
                },
                header,
                destinations,
                vec![],
                &cancel,
            )
            .unwrap();
        let completed = command.execute(&cancel).await.unwrap();
        let version = completed.batches()[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0);
        drop(completed);
        drop(plan);
        let publication = Publication::open(
            PublicationRoot {
                location: control,
                version,
            },
            Arc::clone(&self.registry),
            &self.sessions,
            &cancel,
        )
        .await
        .unwrap();
        let mut rows = BTreeMap::new();
        for member in &publication.record().members {
            let reference = ResolvedTableReference {
                catalog: member.catalog_name.clone().into(),
                schema: member.schema_name.clone().into(),
                table: member.table_name.clone().into(),
            };
            let facts = publication
                .session()
                .capture_relation(&reference, &cancel)
                .await
                .unwrap();
            rows.insert(
                self.registry
                    .relation_by_id(member.relation_id)
                    .unwrap()
                    .key,
                facts.checked().clone(),
            );
        }
        rows
    }
    pub(crate) async fn capture(&self, plan: &ArtifactPlan) -> Result<Values, CompilerError> {
        let cancel = CancellationToken::new();
        let mut values = BTreeMap::new();
        let started = std::time::Instant::now();
        let completed = plan.execute_outputs(&cancel).await?;
        if std::env::var_os("PSE_TEST_PHASE_TIMINGS").is_some() {
            eprintln!(
                "native_pipeline outputs={} execute_seconds={:.6}",
                completed.len(),
                started.elapsed().as_secs_f64()
            );
        }
        for ((_, output), (_, completed)) in plan.outputs().iter().zip(completed) {
            let spec = self.registry.relation_by_id(output.relation_id).unwrap();
            values.insert(
                spec.key,
                completed.into_checked_relation(&self.registry, spec, &cancel)?,
            );
        }
        Ok(values)
    }
}
pub(crate) fn relation<'a>(
    values: &'a Values,
    namespace: &str,
    name: &str,
) -> Option<&'a FieldCheckedBatch> {
    values
        .iter()
        .find(|(key, _)| key.namespace.as_str() == namespace && key.name == name)
        .map(|(_, value)| value)
}
