// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned stage controls: wire DTOs remain caller-owned; catalog results retain leases.

use super::stage::StageHint;
use super::stage_context::StageContext;
use crate::{CatalogError, RelationMember};
use std::collections::BTreeMap;

/// Immutable catalog-owned stage context; clones share data and its reservation.
/// Explicitly cloning the underlying [`StageHint`] creates a caller-owned wire DTO.
pub type OwnedStageHint = super::control::OwnedControl<StageHint>;

pub(super) fn hint_extent(hint: &StageHint) -> Result<usize, CatalogError> {
    let mut bytes = add(size_of::<StageHint>(), 64)?;
    bytes = add(bytes, map_slots(&hint.inputs)?)?;
    for name in hint.inputs.keys() {
        bytes = add(bytes, name.capacity())?;
    }
    bytes = add(bytes, member_extent(hint.pass_record.member())?)?;
    if let Some(context) = &hint.context {
        bytes = add(bytes, context_extent(context)?)?;
    }
    Ok(bytes)
}

pub(super) fn context_extent(context: &StageContext) -> Result<usize, CatalogError> {
    let mut bytes = vector_slots(&context.declarations)?;
    for (name, rows) in &context.declarations {
        bytes = add(bytes, add(name.capacity(), rows_extent(rows)?)?)?;
    }
    bytes = add(bytes, vector_slots(&context.sources)?)?;
    for source in &context.sources {
        bytes = add(bytes, add(source.path.capacity(), source.text.capacity())?)?;
    }
    bytes = add(bytes, map_slots(&context.policies)?)?;
    for (name, policy) in &context.policies {
        bytes = add(bytes, add(name.capacity(), policy.relation.capacity())?)?;
        bytes = add(bytes, rows_extent(&policy.rows)?)?;
    }
    if let Some(engine) = &context.engine {
        bytes = add(bytes, engine_extent(engine)?)?;
    }
    Ok(bytes)
}

fn engine_extent(engine: &crate::session::SessionSemantics) -> Result<usize, CatalogError> {
    engine_parts_extent(
        engine.engine_version.capacity(),
        engine.arrow_version.capacity(),
        &engine.profile,
        &engine.settings,
        &engine.functions,
    )
}
pub(super) fn engine_parts_extent(
    engine_version_bytes: usize,
    arrow_version_bytes: usize,
    profile: &crate::session::EngineProfile,
    settings: &BTreeMap<String, Option<String>>,
    functions: &BTreeMap<String, Vec<String>>,
) -> Result<usize, CatalogError> {
    let mut bytes = add(engine_version_bytes, arrow_version_bytes)?;
    bytes = add(bytes, profile.version.capacity())?;
    for names in [
        &profile.analyzer_rules,
        &profile.optimizer_rules,
        &profile.physical_optimizer_rules,
    ] {
        bytes = add(bytes, strings_extent(names)?)?;
    }
    bytes = add(bytes, map_slots(settings)?)?;
    for (name, value) in settings {
        bytes = add(bytes, name.capacity())?;
        if let Some(value) = value {
            bytes = add(bytes, value.capacity())?;
        }
    }
    bytes = add(bytes, map_slots(functions)?)?;
    for (family, names) in functions {
        bytes = add(bytes, add(family.capacity(), strings_extent(names)?)?)?;
    }
    Ok(bytes)
}
pub(super) fn member_extent(member: &RelationMember) -> Result<usize, CatalogError> {
    let mut bytes = vector_slots(&member.encodings)?;
    for value in [&member.port, &member.namespace, &member.name] {
        bytes = add(bytes, value.capacity())?;
    }
    for encoding in &member.encodings {
        bytes = add(
            bytes,
            add(encoding.writer_version.capacity(), encoding.path.capacity())?,
        )?;
    }
    Ok(bytes)
}
fn rows_extent(rows: &Vec<Vec<String>>) -> Result<usize, CatalogError> {
    rows.iter().try_fold(vector_slots(rows)?, |bytes, row| {
        add(bytes, strings_extent(row)?)
    })
}
fn strings_extent(values: &Vec<String>) -> Result<usize, CatalogError> {
    values
        .iter()
        .try_fold(vector_slots(values)?, |bytes, value| {
            add(bytes, value.capacity())
        })
}
fn vector_slots<T>(values: &Vec<T>) -> Result<usize, CatalogError> {
    mul(values.capacity(), size_of::<T>())
}
pub(super) fn map_slots<K, V>(values: &BTreeMap<K, V>) -> Result<usize, CatalogError> {
    // BTreeMap does not expose capacity. Conservatively retain a complete node per
    // occupied entry (key/value slots and links), including the one-entry case.
    map_extent::<K, V>(values.len())
}
pub(super) fn map_extent<K, V>(len: usize) -> Result<usize, CatalogError> {
    mul(len, add(mul(size_of::<(K, V)>(), 12)?, 128)?)
}
pub(super) fn add(left: usize, right: usize) -> Result<usize, CatalogError> {
    left.checked_add(right).ok_or_else(super::encode::overflow)
}
fn mul(left: usize, right: usize) -> Result<usize, CatalogError> {
    left.checked_mul(right).ok_or_else(super::encode::overflow)
}
