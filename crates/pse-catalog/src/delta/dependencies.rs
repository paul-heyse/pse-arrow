// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact commit dependencies exposed by the native streaming-table provider.
use crate::{
    CatalogError,
    provider::binding::{BindingKey, TableBinding},
    session::{RelationPlan, SnapshotSession},
};
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::streaming::StreamingTable,
    common::{DataFusionError, ResolvedTableReference, TableReference},
    execution::{TaskContext, session_state::SessionState},
    physical_plan::{
        SendableRecordBatchStream, stream::RecordBatchStreamAdapter, streaming::PartitionStream,
    },
};
use pse_ids::CancellationToken;
use pse_relations::generated::runtime::{
    native_dependencies, publications::RuntimePublicationsFieldMembersItem as Member,
};
use std::sync::Arc;

impl SnapshotSession {
    /// Bind a selected member's exact native commit dependencies without IO. The
    /// native scan reads only when polled; missing/expired evidence refuses reuse.
    /// # Errors
    /// Unselected member, repeated role, invalid output schema or cancellation.
    pub fn with_member_dependencies(
        &self,
        reference: &ResolvedTableReference,
        role: &str,
        cancel: &CancellationToken,
    ) -> Result<(Self, RelationPlan), CatalogError> {
        cancel.checkpoint()?;
        if role.is_empty() || self.bindings.input(role).is_some() {
            return Err(crate::session::engine(invalid(
                "dependency role is empty or already bound",
            )));
        }
        let member = self.selected_member(reference)?;
        let schema = native_dependencies::schema()?;
        let partition = Dependencies {
            schema: Arc::clone(&schema),
            member,
            state: Arc::new(self.bound_state()?),
            leases: self.leases.clone(),
        };
        let provider = Arc::new(
            StreamingTable::try_new(schema, vec![Arc::new(partition)])
                .map_err(crate::session::engine)?,
        );
        let reference = TableReference::full("roles", "inputs", role.to_owned());
        let mut session = self.clone();
        session
            .bindings
            .insert(
                BindingKey::Input(role.into()),
                TableBinding::new(
                    reference,
                    provider,
                    Some(native_dependencies::spec(self.registry())?.key),
                    None,
                ),
            )
            .map_err(crate::session::engine)?;
        let plan = session.relation_plan(&ResolvedTableReference {
            catalog: "roles".into(),
            schema: "inputs".into(),
            table: role.to_owned().into(),
        })?;
        Ok((session, plan))
    }
}

#[derive(Clone, Debug)]
struct Dependencies {
    schema: SchemaRef,
    member: Member,
    state: Arc<SessionState>,
    leases: Vec<Arc<super::lease::ReadLease>>,
}
impl PartitionStream for Dependencies {
    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
    fn execute(&self, context: Arc<TaskContext>) -> SendableRecordBatchStream {
        let selected = self.clone();
        let stream = futures_util::stream::once(async move {
            let services = context
                .session_config()
                .get_extension::<crate::session::execution::NativeExecutionContext>()
                .ok_or_else(|| invalid("native dependencies require common execution ownership"))?;
            let cancel = services.cancellation();
            cancel.checkpoint().map_err(external)?;
            let location = url::Url::parse(&selected.member.table_uri).map_err(external)?;
            let _lease = super::lease::read(&location, cancel).await?;
            let _owners = selected.leases;
            let opened = cancel
                .until_cancelled(super::provider::open_native(
                    location,
                    Some(super::provider::delta_version(
                        selected.member.delta_version,
                    )?),
                    crate::cache_service::snapshot::LoadRequirement::Metadata,
                    &selected.state,
                ))
                .await
                .map_err(external)??;
            let table = &opened.table;
            super::contract::DeclaredCheck::new(services.registry(), selected.member.relation_id)?
                .verify(table)?;
            let (rows, _decoded_owner) = cancel
                .until_cancelled(super::attempt::read_dependencies(
                    table,
                    &selected.member,
                    &selected.state,
                ))
                .await
                .map_err(external)??;
            let mut builder =
                native_dependencies::Builder::with_registry(services.registry(), rows.len())
                    .map_err(external)?;
            for row in rows {
                cancel.checkpoint().map_err(external)?;
                builder.push(row).map_err(external)?;
            }
            let batch = builder.finish().map_err(external)?.into_batch();
            Ok(pse_ids::owned_buffer::OwnedRecordBatch::export(
                batch,
                services.reserver().as_ref(),
                cancel,
            )
            .map_err(external)?
            .into_batch())
        });
        Box::pin(RecordBatchStreamAdapter::new(
            Arc::clone(&self.schema),
            stream,
        ))
    }
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
