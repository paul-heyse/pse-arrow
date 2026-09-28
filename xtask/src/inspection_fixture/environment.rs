// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Fresh native plans and exported publications under one shared runtime.
//!
//! The fixture needs no operational store: it composes the candidate publication (member
//! tables and the admitted record) and writes the record as an export manifest whose
//! synthetic export lease outlives any test run. Readers open it with
//! `pse_runtime::workflow::open_export`, exactly as an offline reader of a real export.
#[path = "../../../tests/support/workflow_runtime.rs"]
mod workflow_runtime;
use anyhow::{Context, Result, ensure};
use datafusion::common::ResolvedTableReference;
use pse_catalog::{artifact::ArtifactPlan, delta::publication::Publication};
use pse_relations::generated::{enums::PublicationKind, runtime::publication_manifests};
use std::{collections::BTreeMap, path::Path, sync::Arc};

/// How long the fixture's synthetic export lease lasts: longer than any test run.
const FIXTURE_EXPORT_MICROS: i64 = 30 * 24 * 3600 * 1_000_000;
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
    /// Write the plan's members under `path` and export the admitted record; returns the
    /// export manifest's location.
    pub(crate) async fn publish(
        &self,
        path: &Path,
        plan: &ArtifactPlan,
        kind: PublicationKind,
    ) -> Result<url::Url> {
        let location = url::Url::from_directory_path(path.canonicalize()?)
            .map_err(|()| anyhow::anyhow!("invalid publication directory"))?;
        let mut destinations = BTreeMap::new();
        for (index, name) in plan.outputs().keys().enumerate() {
            destinations.insert(name.clone(), location.join(&format!("members/{index}/"))?);
        }
        let header = publication_manifests::Row {
            publication_id: pse_authoring::ids::uuid_v7(),
            workspace_id: pse_authoring::ids::uuid_v7(),
            parent_publication_id: None,
            attempt_id: pse_authoring::ids::uuid_v7(),
            kind,
            inputs: vec![],
            members: vec![],
            windows: vec![],
            exported_at: None,
            export_lease_id: None,
            export_expires_at: None,
            maintenance_epoch: None,
            store_fingerprint: None,
        };
        let (prepared, _ticket) = plan
            .prepare_publication(header, destinations, vec![], &self.cancel)
            .context("prepare complete source publication")?;
        let completed = prepared
            .execute(&self.cancel)
            .await
            .context("execute complete source publication")?;
        let [batch] = completed.batches() else {
            anyhow::bail!("publication did not return one record");
        };
        ensure!(
            batch.num_rows() == 1,
            "publication did not return one record"
        );
        let record = publication_manifests::View::try_from_batch_with_registry(&self.registry, batch)
            .and_then(|view| view.row(0))?;
        let now = i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_micros(),
        )?;
        // No store issued this export: its lease is synthetic and its store fingerprint
        // is the nil hash.
        let record = publication_manifests::Row {
            exported_at: Some(now),
            export_lease_id: Some(pse_authoring::ids::uuid_v7()),
            export_expires_at: Some(now + FIXTURE_EXPORT_MICROS),
            maintenance_epoch: Some(0),
            store_fingerprint: Some(pse_ids::ContentHash::NIL),
            ..record
        };
        let manifest = location.join("manifest/")?;
        let session = self
            .sessions
            .candidate(BTreeMap::new(), Arc::clone(&self.registry), &self.cancel)?;
        pse_catalog::delta::manifest::prepare_manifest(
            &session,
            manifest.clone(),
            record,
            &self.cancel,
        )
        .context("prepare the export manifest")?
        .execute(&self.cancel)
        .await
        .context("write the export manifest")?;
        Ok(manifest)
    }
    /// Open an exported publication offline.
    pub(crate) async fn open(&self, manifest: url::Url) -> Result<Publication> {
        Ok(pse_runtime::workflow::open_export(
            manifest,
            Arc::clone(&self.registry),
            &self.sessions,
            &self.cancel,
        )
        .await?)
    }
}
