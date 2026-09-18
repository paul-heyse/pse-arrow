// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fresh native plans and Delta publications under one shared runtime.
#[path = "../../../tests/support/workflow_budget.rs"]
mod workflow_budget;
use anyhow::{Context, Result, ensure};
use datafusion::{arrow::array::Int64Array, common::ResolvedTableReference};
use pse_catalog::{
    ExecutionSettings, ThreadBudget,
    artifact::{ArtifactPlan, PublicationTarget},
    delta::publication::{Publication, PublicationRoot},
    session::{SessionFactory, SnapshotSession, native_engine_profile},
};
use pse_ids::CancellationToken;
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{authored, enums::PublicationKind, runtime::publications},
};
use pse_runtime::{ResourceBudget, SharedRuntime};
use std::{collections::BTreeMap, num::NonZeroUsize, path::Path, sync::Arc};

pub(crate) struct Environment {
    pub registry: Arc<pse_schema::Registry>,
    pub sessions: Arc<SessionFactory>,
    pub cancel: CancellationToken,
    pub runtime: Arc<SharedRuntime>,
    _spill: tempfile::TempDir,
}
impl Environment {
    pub(crate) fn new(_: &Path) -> Result<Self> {
        let registry = Arc::new(pse_schema::catalog::assemble()?);
        let spill = tempfile::tempdir()?;
        let one = NonZeroUsize::new(1).context("one is positive")?;
        let threads = ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        };
        let runtime = SharedRuntime::build(ResourceBudget {
            memory_limit_bytes: NonZeroUsize::new(workflow_budget::MEMORY_LIMIT_BYTES)
                .context("positive memory limit")?,
            spill_dir: spill.path().to_path_buf(),
            max_temp_dir_bytes: 1 << 30,
            top_consumers: NonZeroUsize::new(16).context("positive consumers")?,
            threads,
            execution: ExecutionSettings::default(),
            cache: pse_runtime::CacheBudget::for_memory(workflow_budget::MEMORY_LIMIT_BYTES),
            hashing_may_use_pool: false,
        })?;
        let sessions = Arc::new(
            runtime
                .session_factory(native_engine_profile())?
                .with_requirement_planner(Arc::new(
                    pse_rules::invariants::RegistryRequirementPlanner,
                )),
        );
        Ok(Self {
            registry,
            sessions,
            runtime,
            cancel: CancellationToken::new(),
            _spill: spill,
        })
    }
    pub(crate) fn documents(
        &self,
        bundles: &[pse_authoring::document::DocumentBundle],
    ) -> Result<SnapshotSession> {
        let rows = pse_authoring::p1::source_batches(bundles, &self.registry)?;
        let documents = rows
            .get(&authored::documents::RELATION_ID)
            .context("source documents absent")?
            .clone();
        Ok(self
            .sessions
            .candidate_checked(
                BTreeMap::from([(authored::documents::RELATION_KEY, documents)]),
                Arc::clone(&self.registry),
                &self.cancel,
            )?
            .with_purpose(pse_schema::model::provider::OperationPurpose::Construct))
    }
    pub(crate) async fn source_plan(
        &self,
        bundles: &[pse_authoring::document::DocumentBundle],
        compile: bool,
    ) -> Result<ArtifactPlan> {
        let session = self.documents(bundles)?;
        let reference = session.table_reference(&authored::documents::RELATION_KEY)?;
        let documents = session.relation_plan(&ResolvedTableReference {
            catalog: reference
                .catalog()
                .context("document catalog absent")?
                .into(),
            schema: reference.schema().context("document schema absent")?.into(),
            table: reference.table().into(),
        })?;
        Ok(if compile {
            pse_compiler::native::model::from_documents(&session, documents, &self.cancel).await?
        } else {
            pse_compiler::native::model::source(&session, documents, &self.cancel).await?
        })
    }
    pub(crate) async fn publish(
        &self,
        path: &Path,
        plan: &ArtifactPlan,
        kind: PublicationKind,
    ) -> Result<PublicationRoot> {
        let location = url::Url::from_directory_path(path.canonicalize()?)
            .map_err(|()| anyhow::anyhow!("invalid publication directory"))?;
        let control = location.join("control/")?;
        let mut destinations = BTreeMap::new();
        for (index, name) in plan.outputs().keys().enumerate() {
            destinations.insert(name.clone(), location.join(&format!("members/{index}/"))?);
        }
        let header = publications::Row {
            workspace_id: pse_authoring::ids::uuid_v7(),
            publication_id: pse_authoring::ids::uuid_v7(),
            parent_publication_id: None,
            attempt_id: pse_authoring::ids::uuid_v7(),
            kind,
            inputs: vec![],
            members: vec![],
        };
        let prepared = plan
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
                &self.cancel,
            )
            .context("prepare complete source publication")?;
        let completed = prepared
            .execute(&self.cancel)
            .await
            .context("execute complete source publication")?;
        let [batch] = completed.batches() else {
            anyhow::bail!("publication did not return one outcome");
        };
        ensure!(
            batch.num_rows() == 1,
            "publication did not return one version"
        );
        let version = batch
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .context("publication version type")?
            .value(0);
        Ok(PublicationRoot {
            location: control,
            version,
        })
    }
    pub(crate) async fn open(&self, root: PublicationRoot) -> Result<Publication> {
        Ok(Publication::open(
            root,
            Arc::clone(&self.registry),
            &self.sessions,
            &self.cancel,
        )
        .await?)
    }
    pub(crate) async fn capture(
        &self,
        publication: &Publication,
    ) -> Result<BTreeMap<pse_schema::model::RelationKey, FieldCheckedBatch>> {
        let mut rows = BTreeMap::new();
        for member in &publication.record().members {
            let key = self
                .registry
                .relation_by_id(member.relation_id)
                .context("member declaration absent")?
                .key;
            let reference = ResolvedTableReference {
                catalog: member.catalog_name.clone().into(),
                schema: member.schema_name.clone().into(),
                table: member.table_name.clone().into(),
            };
            let facts = publication
                .session()
                .capture_relation(&reference, &self.cancel)
                .await?;
            ensure!(
                rows.insert(key, facts.checked().clone()).is_none(),
                "fixture has ambiguous relation roles"
            );
        }
        Ok(rows)
    }
}
