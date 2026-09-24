// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked allocation bounds for actual native configuration observations.
use crate::EngineError;
use std::collections::BTreeMap;

pub(super) fn engine_inputs_extent(
    engine_version_bytes: usize,
    arrow_version_bytes: usize,
    profile: &crate::session::EngineProfile,
    settings: &BTreeMap<String, Option<String>>,
    functions: &BTreeMap<String, Vec<String>>,
) -> Result<usize, EngineError> {
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
fn strings_extent(values: &Vec<String>) -> Result<usize, EngineError> {
    values
        .iter()
        .try_fold(vector_slots(values)?, |bytes, value| {
            add(bytes, value.capacity())
        })
}
fn vector_slots<T>(values: &Vec<T>) -> Result<usize, EngineError> {
    mul(values.capacity(), size_of::<T>())
}
pub(super) fn map_slots<K, V>(values: &BTreeMap<K, V>) -> Result<usize, EngineError> {
    // BTreeMap does not expose capacity. Conservatively retain a complete node per
    // occupied entry (key/value slots and links), including the one-entry case.
    map_extent::<K, V>(values.len())
}
pub(super) fn map_extent<K, V>(len: usize) -> Result<usize, EngineError> {
    mul(len, add(mul(size_of::<(K, V)>(), 12)?, 128)?)
}
pub(super) fn add(left: usize, right: usize) -> Result<usize, EngineError> {
    left.checked_add(right).ok_or_else(overflow)
}
fn mul(left: usize, right: usize) -> Result<usize, EngineError> {
    left.checked_mul(right).ok_or_else(overflow)
}

fn overflow() -> EngineError {
    EngineError::Admission {
        path: "native.configuration".into(),
        reason: "configuration allocation extent overflow".into(),
    }
}
