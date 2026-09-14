// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P10 establishes physical contracts from complete actual predecessor relations.
pub mod fixture;

use crate::{
    CompilerError,
    mathir_relations::{Family, RelationSink, RelationSource},
    quantity_relations::{decode_quantity_registry, decode_symbol_source},
};
use pse_ids::{CancellationToken, MemoryReserver};
use pse_mathir::{
    NodeId,
    canonicalize::{CanonicalizeInput, Policy},
};
use pse_quantity::{QuantityKindId, QuantityTypeId};
use pse_relations::RecordBatch;
use pse_schema::{
    Registry,
    model::{Namespace, RelationKey},
};
use std::collections::BTreeMap;

/// Fixture-mode implementation with all semantic configuration read from declared rows.
#[derive(Debug)]
pub struct P10 {
    spec: pse_schema::model::PassSpec,
}
impl P10 {
    /// Bind only an explicitly declared P10 fixture pass.
    /// # Errors
    /// Production registries without P10 or its explicit context are refused.
    pub fn new(registry: &Registry) -> Result<Self, CompilerError> {
        let spec = registry
            .pass("P10@1")
            .ok_or_else(|| invalid("P10 fixture pass is not declared"))?
            .clone();
        if !spec
            .inputs
            .iter()
            .any(|port| port.relation == "reference.p10_fixture_context" && port.required)
        {
            return Err(invalid("P10 lacks its required explicit context port"));
        }
        Ok(Self { spec })
    }
}
impl crate::Pass for P10 {
    fn spec(&self) -> &pse_schema::model::PassSpec {
        &self.spec
    }
    fn run<'a>(
        &'a self,
        ctx: &'a crate::PassContext<'a>,
        inputs: &'a crate::InputBundle,
    ) -> pse_catalog::provider::BoxFut<'a, Result<crate::PassOutput, CompilerError>> {
        Box::pin(async move {
            let started = std::time::Instant::now();
            inputs.validate(&self.spec, ctx.registry)?;
            let rows = inputs.rows(ctx.registry)?;
            let context = fixture::context(&rows, ctx.registry)?;
            let output =
                canonicalize_rows(&rows, &context, ctx.registry, ctx.reserver, ctx.cancel)?;
            let ports = self
                .spec
                .outputs
                .iter()
                .map(|port| {
                    let spec = ctx
                        .registry
                        .relation(&port.relation)
                        .ok_or_else(|| invalid("P10 output contract absent"))?;
                    let batch = output
                        .rows
                        .get(&spec.key)
                        .ok_or_else(|| invalid("P10 output missing"))?;
                    Ok((port.port, vec![batch.clone()]))
                })
                .collect::<Result<_, CompilerError>>()?;
            Ok(crate::PassOutput {
                ports,
                findings: vec![],
                record: crate::PassRecordDraft {
                    pass_id: self.spec.id,
                    version: self.spec.version,
                    duration: started.elapsed(),
                    status: crate::PassStatus::Ok,
                },
            })
        })
    }
}

/// Explicit invocation context; a registered fixture pass binds this as actual input rows.
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// Package-designated neutral scalar, never inferred from a dimension.
    pub neutral: Option<QuantityTypeId>,
    /// Explicit Boolean kind used for actual guard contracts.
    pub boolean: Option<QuantityKindId>,
    /// Additional source storage roots, preserving order and multiplicity.
    pub roots: Vec<NodeId>,
}
/// Complete typed output rows and their explicit caller-root correspondence.
#[derive(Debug)]
pub struct TypedRows {
    /// Every compiled mathematical output binding, including explicit empties.
    pub rows: BTreeMap<RelationKey, RecordBatch>,
    /// Canonical roots corresponding to the explicit invocation context.
    pub roots: Vec<NodeId>,
}
/// Execute uncached P10 over exact admitted rows, without synthesizing predecessor passes.
/// Every graph selection/type/unit claim is re-established by actual physical inference.
///
/// # Errors
/// Invalid/missing actual dependencies, incomplete quantity context, malformed expressions,
/// mismatched conversions, static domain failures, resource exhaustion or cancellation.
pub fn canonicalize_rows(
    inputs: &BTreeMap<RelationKey, RecordBatch>,
    context: &Context,
    registry: &Registry,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<TypedRows, CompilerError> {
    cancel.checkpoint()?;
    let mut work = reserver.open("P10:physical-canonicalization");
    let bytes = inputs.values().try_fold(0_usize, |total, batch| {
        total
            .checked_add(pse_catalog::store::membership::validation_extent(batch)?)
            .ok_or_else(|| invalid("P10 workspace size overflow"))
    })?;
    work.try_grow(bytes)
        .map_err(|_| CompilerError::ResourceLimit {
            consumer: "P10 physical workspace".to_owned(),
            config_keys: vec!["memory_limit".to_owned()],
        })?;
    let quantities = decode_quantity_registry(inputs, registry, context.neutral)?;
    let symbols = decode_symbol_source(inputs, registry, &quantities, context.boolean)?;
    let source = RelationSource::from_batches(inputs, registry)?;
    let loaded = pse_mathir::relations::load_untyped(&source, &context.roots)?;
    cancel.checkpoint()?;
    let graph = pse_mathir::canonicalize::canonicalize(
        CanonicalizeInput {
            graph: &loaded.graph,
            equations: &loaded.equations,
            roots: &loaded.roots,
            symbols: &symbols,
            registry: &quantities,
            kernel_bindings: &loaded.kernel_bindings,
            selections: &loaded.selections,
        },
        Policy::Strict,
    )?;
    let mut sink = RelationSink::new(registry, Family::Compiled);
    pse_mathir::relations::emit(&graph, &mut sink)?;
    let mut emitted = sink.into_rows();
    let mut rows = BTreeMap::new();
    for spec in registry.relations().iter().filter(|spec| {
        spec.key.namespace == Namespace::Compiled
            && (spec.key.name.starts_with("math_") || spec.key.name == "kernel_bindings")
    }) {
        cancel.checkpoint()?;
        if spec.key.name == "math_implicit_systems" {
            let input = inputs
                .get(&spec.key)
                .ok_or_else(|| invalid("missing implicit system binding"))?;
            // Its references are symbol/equation identities, never expression ordinals.
            rows.insert(spec.key, input.clone());
            continue;
        }
        let values = emitted.remove(&spec.key).unwrap_or_default();
        rows.insert(
            spec.key,
            pse_relations::cells::batch_from_cells_owned(
                registry, spec, &values, reserver, cancel,
            )?,
        );
    }
    if !emitted.is_empty() {
        return Err(invalid("P10 emitted an undeclared output family"));
    }
    Ok(TypedRows {
        rows,
        roots: graph.roots()[..context.roots.len()].to_vec(),
    })
}
fn invalid(detail: impl Into<String>) -> CompilerError {
    pse_mathir::MathIrError::Malformed {
        node: None,
        detail: detail.into(),
    }
    .into()
}
