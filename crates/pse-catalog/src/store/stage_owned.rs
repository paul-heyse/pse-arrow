// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned stage controls: wire DTOs remain caller-owned; catalog results retain leases.

use super::stage::StageHint;
use crate::{CatalogError, RelationMember};
use std::collections::BTreeMap;

/// Immutable catalog-owned hint; clones share its metadata and reservation.
pub type OwnedStageHint = super::control::OwnedControl<StageHint>;

pub(super) fn hint_extent(hint: &StageHint) -> Result<usize, CatalogError> {
    let mut bytes = add(size_of::<StageHint>(), 64)?;
    bytes = add(bytes, map_slots(&hint.inputs)?)?;
    for name in hint.inputs.keys() {
        bytes = add(bytes, name.capacity())?;
    }
    bytes = add(bytes, member_extent(hint.pass_record.member())?)?;
    Ok(bytes)
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
