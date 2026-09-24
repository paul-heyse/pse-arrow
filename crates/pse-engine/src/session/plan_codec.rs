// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Noncanonical diagnostic plans resolving only actual selected native providers.

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

use pse_columnar::{AllocationLease, CancellationToken};
use serde::{Deserialize, Serialize};

use super::{EngineSession, admission, engine_session::engine};
use crate::EngineError;
use crate::provider::binding::TableBinding;

/// Platform diagnostic codec version, independent of semantic snapshot identity.
pub const PLAN_CODEC_VERSION: &str = "pse.plan.v3";
const MAX_PLAN_BYTES: usize = 16 << 20;
const MAX_PLAN_NODES: usize = 4096;

/// Decoded plan retaining its finite control allocation claim and actual providers.
/// It is diagnostic evidence, never a validity or memo-reuse certificate.
#[derive(Debug)]
pub struct DecodedPlan {
    plan: LogicalPlan,
    _lease: Arc<AllocationLease>,
}
impl DecodedPlan {
    /// Inspect the admitted plan while its reservation remains retained.
    pub const fn plan(&self) -> &LogicalPlan {
        &self.plan
    }
}

#[derive(Debug)]
struct SelectedCodec {
    tables: BTreeMap<TableReference, Arc<TableBinding>>,
    functions: Arc<super::functions::Functions>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    codec_version: String,
    engine_version: String,
    descriptor: Vec<u8>,
}

impl EngineSession {
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
    ) -> std::result::Result<Bytes, EngineError> {
        cancel.checkpoint()?;
        let extent = plan_extent(plan).map_err(engine)?;
        let reservation =
            pse_columnar::MemoryConsumer::new("session:encode-plan").register(&self.pool);
        reservation.try_grow(control_extent(extent).map_err(engine)?)?;
        admission::admit_plan(
            plan,
            &self.registry,
            &self.bindings.providers(),
            &self.pool,
            cancel,
        )
        .map_err(engine)?;
        let codec = self.codec();
        let encoded = plan
            .clone()
            .transform_down_with_subqueries(|mut plan| {
                if let LogicalPlan::TableScan(scan) = &mut plan {
                    let provider = datafusion::datasource::source_as_provider(&scan.source)?;
                    let binding = codec
                        .tables
                        .get(&scan.table_name)
                        .ok_or_else(|| invalid("source has no selected native descriptor"))?;
                    if binding.witness.is_none() || !Arc::ptr_eq(&provider, &binding.provider) {
                        return Err(invalid(
                            "source differs from its exact selected native binding",
                        ));
                    }
                    scan.source =
                        datafusion::datasource::provider_as_source(Arc::new(DescriptorSource {
                            binding: Arc::clone(binding),
                        }));
                    return Ok(datafusion::common::tree_node::Transformed::new(
                        plan,
                        true,
                        TreeNodeRecursion::Jump,
                    ));
                }
                Ok(datafusion::common::tree_node::Transformed::no(plan))
            })
            .map_err(engine)?
            .data;
        // Exact selected providers are encoded as descriptors. Their private
        // decode expressions remain owned by that admitted provider, rather than
        // becoming independently serialized name-based function references.
        self.native
            .function_bindings
            .admit_plan(&encoded, &self.pool, cancel)
            .map_err(engine)?;
        let bytes = logical_plan_to_bytes_with_extension_codec(&encoded, &codec).map_err(engine)?;
        check_extent(bytes.len()).map_err(engine)?;
        let mut output = bytes.to_vec();
        drop(bytes);
        output.shrink_to_fit();
        reservation.shrink(reservation.size().saturating_sub(output.capacity()));
        cancel.checkpoint()?;
        Ok(pse_columnar::owned_buffer::attach_bytes(
            output,
            AllocationLease::new(reservation),
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
    ) -> std::result::Result<DecodedPlan, EngineError> {
        cancel.checkpoint()?;
        check_extent(bytes.len()).map_err(engine)?;
        let reservation =
            pse_columnar::MemoryConsumer::new("session:decode-plan").register(&self.pool);
        reservation.try_grow(control_extent(bytes.len()).map_err(engine)?)?;
        let plan = logical_plan_from_bytes_with_extension_codec(
            bytes,
            &self.native.context.task_ctx(),
            &self.codec(),
        )
        .map_err(engine)?;
        let plan = plan
            .transform_down_with_subqueries(|mut node| {
                if let LogicalPlan::TableScan(scan) = &mut node {
                    let provider = datafusion::datasource::source_as_provider(&scan.source)?;
                    if let Some(descriptor) = provider.downcast_ref::<DescriptorSource>() {
                        scan.source = datafusion::datasource::provider_as_source(Arc::clone(
                            &descriptor.binding.provider,
                        ));
                        return Ok(datafusion::common::tree_node::Transformed::new(
                            node,
                            true,
                            TreeNodeRecursion::Jump,
                        ));
                    }
                }
                Ok(datafusion::common::tree_node::Transformed::no(node))
            })
            .map_err(engine)?
            .data;
        plan_extent(&plan).map_err(engine)?;
        self.native
            .function_bindings
            .admit_plan(&plan, &self.pool, cancel)
            .map_err(engine)?;
        admission::admit_plan(
            &plan,
            &self.registry,
            &self.bindings.providers(),
            &self.pool,
            cancel,
        )
        .map_err(engine)?;
        cancel.checkpoint()?;
        Ok(DecodedPlan {
            plan,
            _lease: AllocationLease::new(reservation),
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
    ) -> std::result::Result<Vec<RecordBatch>, EngineError> {
        self.execute_plan(plan.plan.clone(), cancel).await
    }

    fn codec(&self) -> SelectedCodec {
        SelectedCodec {
            functions: Arc::clone(&self.native.function_bindings),
            tables: self
                .bindings
                .iter()
                .map(|(_, binding)| (binding.reference.clone(), binding))
                .collect(),
        }
    }
}

impl LogicalExtensionCodec for SelectedCodec {
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
        let table = self
            .tables
            .get(table_ref)
            .ok_or_else(|| invalid("table reference is outside the selected native inventory"))?;
        let actual = binding_for(table)?;
        if binding.codec_version != actual.codec_version
            || binding.engine_version != actual.engine_version
            || binding.descriptor != actual.descriptor
        {
            return Err(invalid(
                "serialized descriptor differs from the actual selected native member",
            ));
        }
        if schema.as_ref() != table.provider.schema().as_ref() {
            return Err(invalid("serialized fields differ from selected member"));
        }
        // Keep the selected source opaque while DataFusion builds the decoded
        // scan; its generic scan builder otherwise expands a native ViewTable.
        Ok(Arc::new(DescriptorSource {
            binding: Arc::clone(table),
        }))
    }
    fn try_encode_table_provider(
        &self,
        table_ref: &TableReference,
        node: Arc<dyn TableProvider>,
        buf: &mut Vec<u8>,
    ) -> Result<()> {
        let source = node
            .downcast_ref::<DescriptorSource>()
            .ok_or_else(|| invalid("source has no selected descriptor"))?;
        let actual = self
            .tables
            .get(table_ref)
            .ok_or_else(|| invalid("source name is outside selected inventory"))?;
        if !Arc::ptr_eq(actual, &source.binding) {
            return Err(invalid("descriptor source owner differs"));
        }
        serde_json::to_writer(buf, &binding_for(actual)?)
            .map_err(|error| invalid(&error.to_string()))
    }
}
fn binding_for(table: &TableBinding) -> Result<Binding> {
    Ok(Binding {
        codec_version: PLAN_CODEC_VERSION.to_owned(),
        engine_version: datafusion::DATAFUSION_VERSION.to_owned(),
        descriptor: table
            .witness
            .as_ref()
            .ok_or_else(|| {
                invalid("unpublished computations cannot claim durable provider descriptors")
            })?
            .descriptor()?,
    })
}
/// Codec-only view of an actual selected source. The descriptor carries no row
/// payload, native Delta implementation detail, or producing compiler graph.
#[derive(Debug)]
struct DescriptorSource {
    binding: Arc<TableBinding>,
}
#[async_trait::async_trait]
impl TableProvider for DescriptorSource {
    fn schema(&self) -> SchemaRef {
        self.binding.provider.schema()
    }
    fn table_type(&self) -> datafusion::logical_expr::TableType {
        self.binding.provider.table_type()
    }
    async fn scan(
        &self,
        _: &dyn datafusion::catalog::Session,
        _: Option<&Vec<usize>>,
        _: &[datafusion::logical_expr::Expr],
        _: Option<usize>,
    ) -> Result<Arc<dyn datafusion::physical_plan::ExecutionPlan>> {
        Err(invalid(
            "encoding-only descriptor source must be rebound before execution",
        ))
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
        node.apply_expressions(|expression| {
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
            Ok(TreeNodeRecursion::Continue)
        })?;
        Ok(TreeNodeRecursion::Continue)
    })?;
    Ok(size.0)
}
