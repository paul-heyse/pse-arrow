// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Local source construction crosses publication through its registered typed consumer.

use crate::{
    Catalog, CatalogError, Snapshot,
    store::{
        membership::{self, AdmissionContext},
        publish::{BundleDraft, RelationDraft},
    },
};
use pse_ids::{CancellationToken, SnapshotKind};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::{RelationKey, SnapshotClass};
use std::{any::Any, collections::BTreeMap, sync::Arc};

/// The source model and its exact case, produced by one publication operation.
pub type SourceSnapshots = (Arc<Snapshot>, Arc<Snapshot>);

/// The registered compiler consumer of its private source/P2 completion owner.
/// An arbitrary caller cannot substitute an implementation at publication time.
pub trait SourceProducer: Send + Sync + std::fmt::Debug {
    /// Consume the producer's immutable typed arguments without replaying completed work.
    /// # Errors
    /// Different argument type or incomplete source/obligation construction.
    fn complete(
        &self,
        catalog: &Catalog,
        arguments: &(dyn Any + Send + Sync),
    ) -> Result<BTreeMap<RelationKey, FieldCheckedBatch>, CatalogError>;
}

/// A complete locally constructed source universe. Its fields are private and it is consumed once.
pub struct CompletedSources {
    catalog: Catalog,
    _producer: Arc<dyn SourceProducer>,
    _arguments: Arc<dyn Any + Send + Sync>,
    rows: BTreeMap<RelationKey, FieldCheckedBatch>,
}
impl std::fmt::Debug for CompletedSources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompletedSources")
            .field("relations", &self.rows.keys())
            .finish_non_exhaustive()
    }
}

impl Catalog {
    /// Retain an actual registered source construction and its completed residual program.
    /// # Errors
    /// Absent producer, foreign arguments, or incomplete declared output inventory.
    pub fn complete_sources(
        &self,
        arguments: Arc<dyn Any + Send + Sync>,
    ) -> Result<CompletedSources, CatalogError> {
        let producer = self
            .validator
            .as_ref()
            .and_then(|validator| validator.source_producer())
            .ok_or_else(|| membership::refused("source producer is not registered"))?;
        let rows = producer.complete(self, arguments.as_ref())?;
        for spec in self.registry.relations().iter().filter(|spec| {
            matches!(
                spec.snapshot_class,
                SnapshotClass::Model | SnapshotClass::Case
            )
        }) {
            rows.get(&spec.key)
                .ok_or_else(|| {
                    membership::refused("source construction omits a declared relation")
                })?
                .check_declaration(&self.registry, spec)?;
        }
        Ok(CompletedSources {
            catalog: self.clone(),
            _producer: producer,
            _arguments: arguments,
            rows,
        })
    }

    /// Publish the complete source model and its case without repeating P1 or P2.
    /// Both immutable snapshots finish before a caller may publish their revision ref.
    /// # Errors
    /// Foreign completion, encoding/storage failure, cancellation or resource exhaustion.
    pub async fn publish_sources(
        &self,
        completed: CompletedSources,
        cancel: &CancellationToken,
    ) -> Result<(Arc<Snapshot>, Arc<Snapshot>), CatalogError> {
        Ok(self
            .prepare_source_publication(completed, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Prepare model/case publication over the completed source providers.
    /// Both complete immutable snapshots are outputs of this one native operation.
    /// # Errors
    /// Foreign completion, incompatible fields/policies or native preparation failure.
    pub fn prepare_source_publication(
        &self,
        completed: CompletedSources,
        cancel: &CancellationToken,
    ) -> Result<crate::store::operation::PreparedStoreOperation<SourceSnapshots>, CatalogError>
    {
        use pse_schema::model::provider::{OperationPurpose, ProviderScope};
        let session = self
            .validation_session(cancel)?
            .select_inputs(&std::collections::BTreeSet::new(), cancel)?
            .with_checked_workspace(completed.rows.clone(), cancel)?;
        self.prepare_store_operation_in(
            &session,
            crate::store::operation::StoreCommand {
                name: "store.publish_sources",
                scope: ProviderScope::Schema("store".into(), "manifests".into()),
                purpose: OperationPurpose::Publish,
                arguments: vec![],
            },
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let snapshots = catalog.publish_sources_inner(completed, &cancel).await?;
                    Ok((snapshots, 2))
                })
            }),
            cancel,
        )
    }

    async fn publish_sources_inner(
        &self,
        completed: CompletedSources,
        cancel: &CancellationToken,
    ) -> Result<(Arc<Snapshot>, Arc<Snapshot>), CatalogError> {
        if !Arc::ptr_eq(&self.admission, &completed.catalog.admission) {
            return Err(membership::refused(
                "source completion belongs to another catalog context",
            ));
        }
        let model = self
            .publish_source_class(
                &completed.rows,
                SnapshotKind::Model,
                SnapshotClass::Model,
                AdmissionContext::default(),
                cancel,
            )
            .await?;
        let context = AdmissionContext {
            traversal: Arc::default(),
            invocation: None,
            parents: BTreeMap::from([("model".to_owned(), Arc::clone(&model))]),
            stage_pass: None,
        };
        let case = self
            .publish_source_class(
                &completed.rows,
                SnapshotKind::Case,
                SnapshotClass::Case,
                context,
                cancel,
            )
            .await?;
        Ok((model, case))
    }

    async fn publish_source_class(
        &self,
        rows: &BTreeMap<RelationKey, FieldCheckedBatch>,
        kind: SnapshotKind,
        class: SnapshotClass,
        context: AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<Arc<Snapshot>, CatalogError> {
        let manifest = self.manifest_template(kind, &context)?;
        let mut relations = BTreeMap::new();
        let mut candidates = BTreeMap::new();
        for spec in self
            .registry
            .relations()
            .iter()
            .filter(|spec| spec.snapshot_class == class)
        {
            let batch = rows
                .get(&spec.key)
                .ok_or_else(|| membership::refused("source output disappeared"))?;
            let port = pse_schema::membership::port_name(spec);
            candidates.insert(port.clone(), batch.clone());
            relations.insert(
                port,
                RelationDraft {
                    contract: Arc::new(crate::RelationContract::from_spec(
                        &self.registry,
                        spec,
                        crate::EncodingPolicy::IpcFile,
                    )?),
                    batches: vec![batch.batch().clone()],
                },
            );
        }
        self.publish_admitted_bundle(
            BundleDraft {
                manifest,
                relations,
                context,
            },
            candidates,
            cancel,
        )
        .await
    }
}
