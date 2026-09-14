// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P0: package and schema resolution (blueprint §14.1).
//!
//! The pass wrapper around `pse_authoring::p0`.
//!
use crate::{CompilerError, passes::dag::invalid};
use pse_ids::{CancellationToken, MemoryReserver};
use pse_relations::{RecordBatch, generated::authored};
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;

/// Resolve exact package headers from actual candidate rows, before snapshot identity exists.
/// # Errors
/// Missing inventory, malformed headers, dependency cycles or incompatible exact versions.
pub fn resolve(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<RecordBatch, CompilerError> {
    cancel.checkpoint()?;
    let spec = registry
        .relation("authored.packages")
        .ok_or_else(|| invalid("P0 packages undeclared"))?;
    let batch = rows
        .get(&spec.key)
        .ok_or_else(|| invalid("P0 packages input absent"))?;
    let headers = pse_relations::cells::cells_from_batch(registry, spec, batch)?
        .into_iter()
        .map(authored::packages::Row::from_cells)
        .collect::<Result<Vec<_>, _>>()?;
    let graph = pse_authoring::p0::resolve_headers(&headers, registry)?;
    let output = registry
        .relation("normalized.package_graph")
        .ok_or_else(|| invalid("P0 output undeclared"))?;
    Ok(pse_relations::cells::batch_from_cells_owned(
        registry,
        output,
        &graph
            .rows
            .into_iter()
            .map(pse_relations::generated::normalized::package_graph::Row::into_cells)
            .collect::<Vec<_>>(),
        reserver,
        cancel,
    )?)
}
