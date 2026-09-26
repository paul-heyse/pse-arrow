// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fresh native plans and Delta publications under one shared runtime.
#[path = "../../../tests/support/workflow_runtime.rs"]
mod workflow_runtime;
use anyhow::{Context, Result, ensure};
use datafusion::{arrow::array::Int64Array, common::ResolvedTableReference};
use pse_catalog::{
    artifact::{ArtifactPlan, PublicationTarget},
    delta::publication::{Publication, PublicationRoot},
};
use pse_relations::generated::{enums::PublicationKind, runtime::publications};
use std::{collections::BTreeMap, path::Path, sync::Arc};
use workflow_runtime::WorkflowRuntime;

pub(crate) struct Environment(WorkflowRuntime);
impl std::ops::Deref for Environment {
    type Target = WorkflowRuntime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Environment {
    pub(crate) fn new(_: &Path) -> Result<Self> {
        Ok(Self(
            WorkflowRuntime::new().map_err(anyhow::Error::from_boxed)?,
        ))
    }
    pub(crate) async fn source_plan(
        &self,
        bundles: &[pse_runtime::authoring_driver::document::DocumentBundle],
    ) -> Result<ArtifactPlan> {
        let rows = pse_runtime::authoring_driver::p1::source_batches(bundles, &self.registry)?;
        let mut rows = rows
            .into_iter()
            .map(|(id, batch)| {
                Ok((
                    self.registry
                        .relation_by_id(id)
                        .context("source relation absent")?
                        .key,
                    batch,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        for (key, batch) in self.registry.schema_batches() {
            let spec = self
                .registry
                .relation_by_key(*key)
                .context("registry reflection declaration absent")?;
            rows.insert(
                *key,
                pse_relations::columnar::FieldCheckedBatch::admit(
                    &self.registry,
                    spec,
                    batch.clone(),
                )?,
            );
        }
        let keys = rows.keys().copied().collect::<Vec<_>>();
        let session =
            self.sessions
                .candidate_checked(rows, Arc::clone(&self.registry), &self.cancel)?;
        let mut outputs = BTreeMap::new();
        for key in keys {
            let reference = session.table_reference(&key)?;
            let reference = ResolvedTableReference {
                catalog: reference.catalog().context("source catalog")?.into(),
                schema: reference.schema().context("source schema")?.into(),
                table: reference.table().into(),
            };
            let plan = session.relation_plan(&reference)?.plan().clone();
            let relation_id = self
                .registry
                .relation(&key.qualified_name())
                .context("source declaration")?
                .id;
            outputs.insert(
                reference,
                pse_catalog::artifact::RelationOutput { relation_id, plan },
            );
        }
        Ok(ArtifactPlan::new(session, outputs, &self.cancel)?)
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
        let (prepared, _ticket) = plan
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
}
