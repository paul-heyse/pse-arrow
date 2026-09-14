// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Noncanonical diagnostic plans resolving only actual retained snapshot providers.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::sync::Arc;

use bytes::Bytes;
use datafusion::arrow::{array::RecordBatch, datatypes::SchemaRef};
use datafusion::catalog::TableProvider;
use datafusion::common::{
    DataFusionError, Result, TableReference,
    tree_node::{TreeNode, TreeNodeRecursion},
};
use datafusion::execution::TaskContext;
use datafusion::logical_expr::{Extension, LogicalPlan};
use datafusion_proto::bytes::{
    logical_plan_from_bytes_with_extension_codec, logical_plan_to_bytes_with_extension_codec,
};
use datafusion_proto::logical_plan::LogicalExtensionCodec;
use pse_ids::{CancellationToken, ContentHash, ReservationLease, SemanticId};
use serde::{Deserialize, Serialize};

use super::{SnapshotSession, admission, snapshot_session::engine};
use crate::{CatalogError, provider::table::RelationTable, snapshot::ManifestRef};

/// Platform diagnostic codec version, independent of semantic snapshot identity.
pub const PLAN_CODEC_VERSION: &str = "pse.plan.v1";
const MAX_PLAN_BYTES: usize = 16 << 20;
const MAX_PLAN_NODES: usize = 4096;

/// Decoded plan retaining its finite control allocation claim and actual providers.
/// It is diagnostic evidence, never a validity or memo-reuse certificate.
#[derive(Debug)]
pub struct DecodedPlan {
    plan: LogicalPlan,
    _lease: Arc<ReservationLease>,
}
impl DecodedPlan {
    /// Inspect the admitted plan while its reservation remains retained.
    pub const fn plan(&self) -> &LogicalPlan {
        &self.plan
    }
}

#[derive(Debug)]
struct SnapshotCodec {
    tables: BTreeMap<TableReference, Arc<dyn TableProvider>>,
    functions: Arc<super::functions::Functions>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    codec_version: String,
    engine_version: String,
    manifest: ManifestRef,
    relation_id: SemanticId,
    schema_version: u32,
    logical_hash: ContentHash,
}

impl SnapshotSession {
    /// Encode a diagnostic protobuf plan after admitting its actual source objects.
    /// Bytes are noncanonical and must never enter semantic identities or memo keys.
    /// Unpublished candidates have no durable provider binding and are refused.
    ///
    /// # Errors
    /// Invalid/foreign plans, unsupported extension nodes, finite control limits or cancellation.
    pub fn encode_plan(
        &self,
        plan: &LogicalPlan,
        cancel: &CancellationToken,
    ) -> std::result::Result<Bytes, CatalogError> {
        cancel.checkpoint()?;
        let extent = plan_extent(plan).map_err(engine)?;
        let mut reservation = self.reserver.open("session:encode-plan");
        reservation.try_grow(control_extent(extent).map_err(engine)?)?;
        self.function_bindings.admit_plan(plan).map_err(engine)?;
        admission::admit_plan(
            plan,
            &self.registry,
            &self.tables,
            self.reserver.as_ref(),
            cancel,
        )
        .map_err(engine)?;
        let bytes =
            logical_plan_to_bytes_with_extension_codec(plan, &self.codec()).map_err(engine)?;
        check_extent(bytes.len()).map_err(engine)?;
        let mut output = bytes.to_vec();
        drop(bytes);
        output.shrink_to_fit();
        reservation.shrink(reservation.size().saturating_sub(output.capacity()));
        cancel.checkpoint()?;
        Ok(pse_ids::owned_buffer::attach_bytes(
            output,
            ReservationLease::new(reservation),
        )?)
    }

    /// Decode diagnostic bytes against exact retained provider bindings and re-admit the
    /// complete resulting plan. No registry or provider is reconstructed from a digest.
    ///
    /// # Errors
    /// Unknown codec/engine, stale bindings, altered schemas, foreign plans, or control limits.
    pub fn decode_plan(
        &self,
        bytes: &[u8],
        cancel: &CancellationToken,
    ) -> std::result::Result<DecodedPlan, CatalogError> {
        cancel.checkpoint()?;
        check_extent(bytes.len()).map_err(engine)?;
        let mut reservation = self.reserver.open("session:decode-plan");
        reservation.try_grow(control_extent(bytes.len()).map_err(engine)?)?;
        let plan = logical_plan_from_bytes_with_extension_codec(
            bytes,
            &self.context.task_ctx(),
            &self.codec(),
        )
        .map_err(engine)?;
        plan_extent(&plan).map_err(engine)?;
        self.function_bindings.admit_plan(&plan).map_err(engine)?;
        admission::admit_plan(
            &plan,
            &self.registry,
            &self.tables,
            self.reserver.as_ref(),
            cancel,
        )
        .map_err(engine)?;
        cancel.checkpoint()?;
        Ok(DecodedPlan {
            plan,
            _lease: ReservationLease::new(reservation),
        })
    }

    /// Execute decoded evidence while retaining its allocation claim and rechecking the
    /// receiving session's actual inventory. A different session cannot substitute rows.
    ///
    /// # Errors
    /// Plan admission, engine execution, cancellation or shared resource exhaustion.
    pub async fn execute_decoded(
        &self,
        plan: &DecodedPlan,
        cancel: &CancellationToken,
    ) -> std::result::Result<Vec<RecordBatch>, CatalogError> {
        self.execute_plan(plan.plan.clone(), cancel).await
    }

    fn codec(&self) -> SnapshotCodec {
        SnapshotCodec {
            functions: Arc::clone(&self.function_bindings),
            tables: self
                .sources
                .values()
                .map(|(name, table)| (name.clone(), Arc::clone(table)))
                .collect(),
        }
    }
}

impl LogicalExtensionCodec for SnapshotCodec {
    fn try_encode_udf(
        &self,
        node: &datafusion::logical_expr::ScalarUDF,
        _buf: &mut Vec<u8>,
    ) -> Result<()> {
        self.functions.scalar(node)
    }
    fn try_encode_udaf(
        &self,
        node: &datafusion::logical_expr::AggregateUDF,
        _buf: &mut Vec<u8>,
    ) -> Result<()> {
        self.functions.aggregate(node)
    }
    fn try_encode_udwf(
        &self,
        node: &datafusion::logical_expr::WindowUDF,
        _buf: &mut Vec<u8>,
    ) -> Result<()> {
        self.functions.window(node)
    }
    fn try_encode_higher_order_function(
        &self,
        node: &datafusion::logical_expr::HigherOrderUDF,
        _buf: &mut Vec<u8>,
    ) -> Result<()> {
        self.functions.higher_order(node)
    }
    fn try_decode(
        &self,
        _buf: &[u8],
        _inputs: &[LogicalPlan],
        _ctx: &TaskContext,
    ) -> Result<Extension> {
        Err(invalid(
            "custom logical extensions are not registered in this codec version",
        ))
    }
    fn try_encode(&self, _node: &Extension, _buf: &mut Vec<u8>) -> Result<()> {
        Err(invalid(
            "custom logical extensions are not registered in this codec version",
        ))
    }
    fn try_decode_table_provider(
        &self,
        buf: &[u8],
        table_ref: &TableReference,
        schema: SchemaRef,
        _ctx: &TaskContext,
    ) -> Result<Arc<dyn TableProvider>> {
        let binding: Binding =
            serde_json::from_slice(buf).map_err(|error| invalid(&error.to_string()))?;
        let provider = self
            .tables
            .get(table_ref)
            .ok_or_else(|| invalid("table reference is outside the pinned inventory"))?;
        let table = provider
            .as_ref()
            .downcast_ref::<RelationTable>()
            .ok_or_else(|| invalid("unpublished candidates cannot supply durable plan bindings"))?;
        let actual = binding_for(table);
        if binding.codec_version != actual.codec_version
            || binding.engine_version != actual.engine_version
            || binding.manifest != actual.manifest
            || binding.relation_id != actual.relation_id
            || binding.schema_version != actual.schema_version
            || binding.logical_hash != actual.logical_hash
        {
            return Err(invalid(
                "serialized binding differs from the actual admitted provider",
            ));
        }
        if schema.as_ref() != provider.schema().as_ref() {
            return Err(invalid(
                "serialized fields or metadata differ from the actual provider",
            ));
        }
        Ok(Arc::clone(provider))
    }
    fn try_encode_table_provider(
        &self,
        table_ref: &TableReference,
        node: Arc<dyn TableProvider>,
        buf: &mut Vec<u8>,
    ) -> Result<()> {
        if self
            .tables
            .get(table_ref)
            .is_none_or(|actual| !Arc::ptr_eq(actual, &node))
        {
            return Err(invalid(
                "plan source is not the exact retained named provider",
            ));
        }
        let table = node
            .as_ref()
            .downcast_ref::<RelationTable>()
            .ok_or_else(|| invalid("unpublished candidates cannot supply durable plan bindings"))?;
        serde_json::to_writer(buf, &binding_for(table)).map_err(|error| invalid(&error.to_string()))
    }
}

fn binding_for(table: &RelationTable) -> Binding {
    Binding {
        codec_version: PLAN_CODEC_VERSION.to_owned(),
        engine_version: datafusion::DATAFUSION_VERSION.to_owned(),
        manifest: table.snapshot().manifest_ref(),
        relation_id: table.relation().contract().canonical.relation_id,
        schema_version: table.relation().contract().canonical.schema_version.0,
        logical_hash: table.relation().member().logical_hash.content_hash(),
    }
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("plan codec: {reason}"))
}
fn check_extent(bytes: usize) -> Result<()> {
    if bytes == 0 || bytes > MAX_PLAN_BYTES {
        return Err(invalid("plan exceeds the finite diagnostic byte envelope"));
    }
    Ok(())
}
fn control_extent(bytes: usize) -> Result<usize> {
    bytes
        .checked_mul(64)
        .and_then(|bytes| bytes.checked_add(65536))
        .ok_or_else(|| invalid("control allocation extent overflow"))
}

struct Extent(usize);
impl Write for Extent {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        self.0 = self
            .0
            .checked_add(value.len())
            .filter(|n| *n <= MAX_PLAN_BYTES)
            .ok_or(std::fmt::Error)?;
        Ok(())
    }
}
fn plan_extent(plan: &LogicalPlan) -> Result<usize> {
    let mut size = Extent(1);
    let mut nodes = 0usize;
    plan.apply_with_subqueries(|node| {
        nodes += 1;
        if nodes > MAX_PLAN_NODES {
            return Err(invalid("too many diagnostic plan nodes"));
        }
        write!(size, "{}", node.display())
            .map_err(|_| invalid("diagnostic plan text exceeds control envelope"))?;
        for field in node.schema().fields() {
            size.0 = size
                .0
                .checked_add(field.size())
                .filter(|n| *n <= MAX_PLAN_BYTES)
                .ok_or_else(|| invalid("diagnostic field extent exceeds control envelope"))?;
        }
        for expression in node.expressions() {
            expression.apply(|expression| {
                if let datafusion::logical_expr::Expr::Literal(value, _) = expression {
                    size.0 = size
                        .0
                        .checked_add(value.size())
                        .filter(|n| *n <= MAX_PLAN_BYTES)
                        .ok_or_else(|| {
                            invalid("diagnostic literal extent exceeds control envelope")
                        })?;
                }
                Ok(TreeNodeRecursion::Continue)
            })?;
        }
        Ok(TreeNodeRecursion::Continue)
    })?;
    Ok(size.0)
}
