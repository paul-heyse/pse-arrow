// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native package admission around the pure package graph resolver.
use crate::authoring_driver::{DriverError, contract};
use pse_authoring::p0::resolve_rows;
use pse_structural::projection::GraphLimits;
/// Native boundary: decode once, resolve typed values, and encode once.
/// # Errors
/// Generated field validation, package resolution or resource admission fails.
pub fn resolve(
    packages: &pse_relations::columnar::FieldCheckedBatch,
    session: &pse_engine::session::EngineSession,
    cancel: &pse_columnar::CancellationToken,
) -> Result<pse_relations::columnar::FieldCheckedBatch, DriverError> {
    cancel.checkpoint()?;
    let work =
        pse_columnar::MemoryConsumer::new("authoring:package-graph").register(session.pool());
    let extent = pse_columnar::algorithm_decode_extent(packages.batch())?
        .checked_mul(8)
        .ok_or_else(|| contract("package graph extent overflow"))?;
    work.try_grow(extent)?;
    let headers =
        pse_relations::generated::authored::packages::View::from_checked(packages)?.rows()?;
    let rows = resolve_rows(
        &headers,
        GraphLimits {
            nodes: headers.len(),
            edges: extent,
        },
    )?;
    let mut output = pse_relations::generated::normalized::package_graph::Builder::with_registry(
        session.registry(),
        rows.len(),
    )?;
    for row in rows {
        cancel.checkpoint()?;
        output.push(row)?;
    }
    Ok(output.finish()?)
}
