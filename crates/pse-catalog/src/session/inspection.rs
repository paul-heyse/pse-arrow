// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Inspection reads the same fully qualified native hierarchy as every operation.
use super::SnapshotSession;
use std::collections::BTreeSet;

impl SnapshotSession {
    /// Exact catalog/schema/table names for declared relations. Semantic aliases
    /// do not duplicate one actual native name; distinct table names remain distinct.
    pub fn inspection_tables(&self) -> Vec<(String, String, String)> {
        self.bindings
            .iter()
            .filter_map(|(_, binding)| {
                binding.relation?;
                Some((
                    binding.reference.catalog()?.to_owned(),
                    binding.reference.schema()?.to_owned(),
                    binding.reference.table().to_owned(),
                ))
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
