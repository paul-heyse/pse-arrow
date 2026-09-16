// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed syntax discovers reads; native plans bind properties and finite domains.
mod native;
pub(super) use native::emit;

use super::{demand_paths, lower};
use crate::CompilerError;
use pse_ids::{CancellationToken, Reservation, SemanticId};
use pse_relations::{
    columnar::{Collection, FieldCheckedBatch},
    generated::{normalized, provenance::property_read_occurrences},
};
use pse_schema::model::RelationKey;
use std::collections::BTreeMap;

/// Preserve syntax occurrence identity before any relational property lookup.
pub(super) fn syntax(
    lowered: &mut lower::Lowered,
    source_id: SemanticId,
    offset: u64,
    session: &pse_catalog::session::SnapshotSession,
    cancel: &CancellationToken,
    work: &mut dyn Reservation,
) -> Result<BTreeMap<RelationKey, FieldCheckedBatch>, CompilerError> {
    let mut columns = Collection::new(session.registry(), session.reserver(), cancel);
    columns.ensure::<property_read_occurrences::Row>()?;
    columns.ensure::<normalized::property_path_demands::Row>()?;
    for (read, guard) in demand_paths::collect(lowered, source_id, offset, cancel, work)? {
        match read {
            demand_paths::Read::Symbol { symbol, node } => {
                let id = pse_ids::named_id(
                    source_id,
                    &format!(
                        "property-symbol:{}:read:{node}:guard:{guard:?}",
                        symbol.to_hex()
                    ),
                );
                columns.push(property_read_occurrences::Row {
                    read_id: id,
                    source_id,
                    symbol_decl_id: symbol,
                    read_node_id: node,
                    guard_predicate_id: guard,
                })?;
            }
            demand_paths::Read::Path { path, node } => {
                let id = pse_ids::named_id(
                    source_id,
                    &format!("property-path:{path}:read:{node}:guard:{guard:?}"),
                );
                columns.push(normalized::property_path_demands::Row {
                    demand_id: id,
                    source_id,
                    path_id: path,
                    guard_predicate_id: guard,
                    read_node_id: node,
                    derivation_id: SemanticId::NIL,
                })?;
            }
        }
    }
    Ok(columns.finish()?)
}
