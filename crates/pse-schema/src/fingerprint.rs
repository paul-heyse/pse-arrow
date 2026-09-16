// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The registry fingerprint and the per-relation fingerprint (blueprint §4.3, §5.3).
//!
//! Two digests with two jobs. The *registry* fingerprint enters every snapshot frame, every
//! stage key and every canonical preimage: it identifies the declared schema content. The
//! *relation* fingerprint is the `pse.contract.fingerprint` metadata value that identifies
//! one declaration. Neither digest admits a batch or proves a cached artifact is valid:
//! readers still validate its exact schema, metadata, arrays and semantic constraints.
//!
//! Both are keyed with [`pse_ids::derive::context::REGISTRY`] and framed with
//! [`pse_ids::FramedHasher`], so the framing rules of ADR-0050 hold here with no second
//! implementation. [`FRAME_VERSION`] versions that framing, independently of changes to
//! the declared rows it contains. Every stored `logical_hash` and `snapshot_id` is a
//! function of the complete registry content.

use pse_ids::{ContentHash, FramedHasher, derive::context};

use crate::builder::Registry;
use crate::model::{Cell, RelationKey, RelationSpec};

/// The version string of this framing (ADR-0050).
pub const FRAME_VERSION: &str = "pse.schema.fingerprint.v1";

/// The digest of the registry's own rows (blueprint §4.3 `pse.contract.fingerprint`).
///
/// `rows` is [`Registry::schema_rows`]: the relations in their fixed order, each
/// primary-key sorted with its cells in column order.
pub fn registry(rows: &[(RelationKey, Vec<Vec<Cell>>)]) -> ContentHash {
    let mut hasher = FramedHasher::new(context::REGISTRY);
    hasher.part(FRAME_VERSION.as_bytes());
    for (key, table) in rows {
        frame_table(&mut hasher, key, table);
    }
    hasher.finish_hash()
}

/// The digest of one relation's declaration (blueprint §4.3).
///
/// Frames the relation's `reference.schema_relations` row, its `reference.schema_columns`
/// rows, and the `reference.schema_logical_types` and `reference.schema_enums` rows those
/// columns reference. The referenced rows are included because a column's meaning is not
/// in its own row: a `pse.enum` column that kept its name while its enumeration lost a
/// member would otherwise keep its fingerprint. Exact schema and enum-domain validation
/// remains required even when the fingerprint matches.
///
/// # Errors
/// Native field serialization fails. No incomplete fingerprint is emitted.
pub fn relation(reg: &Registry, spec: &RelationSpec) -> Result<ContentHash, crate::SchemaError> {
    let mut hasher = FramedHasher::new(context::REGISTRY);
    hasher.part(FRAME_VERSION.as_bytes());
    hasher.str("relation");
    hasher.str(&spec.key.to_string());

    frame_row(&mut hasher, &Registry::schema_relations_row(spec));

    let column_rows = reg.schema_columns_rows_of(spec)?;
    hasher.u64(len(column_rows.len()));
    for row in &column_rows {
        frame_row(&mut hasher, row);
    }

    let mut type_names: Vec<String> = Vec::new();
    let mut enum_names: Vec<String> = Vec::new();
    for column in &spec.columns {
        enum_names.extend(column.enum_domains());
        let mut reachable = Vec::new();
        column.value_type().walk(&mut reachable);
        for ty in reachable {
            let name = ty.type_name()?;
            if !type_names.contains(&name) {
                type_names.push(name);
            }
        }
    }
    type_names.sort_unstable();
    enum_names.sort_unstable();
    enum_names.dedup();

    hasher.u64(len(type_names.len()));
    for name in &type_names {
        match reg.logical_type(name) {
            Some(row) => frame_row(&mut hasher, &Registry::schema_logical_types_row(row)),
            None => frame_row(&mut hasher, &[Cell::Null]),
        }
    }

    hasher.u64(len(enum_names.len()));
    for name in &enum_names {
        let rows = reg
            .enum_spec(name)
            .map(Registry::schema_enums_rows_of)
            .unwrap_or_default();
        hasher.u64(len(rows.len()));
        for row in &rows {
            frame_row(&mut hasher, row);
        }
    }

    Ok(hasher.finish_hash())
}

/// Frames one relation's table: its key, its row count and its rows.
fn frame_table(hasher: &mut FramedHasher, key: &RelationKey, table: &[Vec<Cell>]) {
    hasher.part(key.to_string().as_bytes());
    hasher.u64(len(table.len()));
    for row in table {
        frame_row(hasher, row);
    }
}

/// Frames one row: its cell count and its cells, in column order.
fn frame_row(hasher: &mut FramedHasher, row: &[Cell]) {
    hasher.u64(len(row.len()));
    for cell in row {
        cell.frame(hasher);
    }
}

/// A length as the `u64` the framing takes.
///
/// The saturating conversion is unreachable on every supported target; it exists because
/// the crate's panic policy has no room for an `expect` only a 128-bit address space could
/// reach.
fn len(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
