// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Materializes the registry's own declared rows without introducing a second row model.

use crate::RelationError;
use arrow_array::RecordBatch;
use pse_schema::Registry;
use pse_schema::model::RelationKey;
use std::collections::BTreeMap;

/// Builds and admits every self-description relation supplied by this registry.
///
/// # Errors
/// An undeclared row relation or any schema, cell or visible-value admission failure.
pub fn materialize(reg: &Registry) -> Result<BTreeMap<RelationKey, RecordBatch>, RelationError> {
    let mut batches = BTreeMap::new();
    for (key, rows) in reg.schema_rows() {
        let spec = reg
            .relations()
            .iter()
            .find(|spec| spec.key == key)
            .ok_or_else(|| RelationError::UnknownRegistry {
                relation: key.to_string(),
            })?;
        batches.insert(key, crate::cells::batch_from_cells(reg, spec, &rows)?);
    }
    Ok(batches)
}
