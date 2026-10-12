// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Parser-derived document paths and byte spans.

use std::collections::BTreeMap;

use crate::authoring_driver::SourceSpan;

/// A JSON-pointer-shaped path within an authoring document.
pub type DocPath = str;

/// Exact source ranges attached by the input parser, before contextual values are injected.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpanIndex {
    spans: BTreeMap<String, SourceSpan>,
}

impl SpanIndex {
    /// The original byte range of a section, row, field or key.
    pub fn span(&self, path: &DocPath) -> Option<SourceSpan> {
        self.spans.get(path).copied()
    }
    /// All original paths in deterministic lexical order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, SourceSpan)> {
        self.spans.iter().map(|(path, span)| (path.as_str(), *span))
    }
    pub(super) fn retained_extent(&self) -> Result<usize, crate::authoring_driver::DriverError> {
        self.spans.keys().try_fold(0, |sum, path| {
            super::allocation::add(
                sum,
                super::allocation::add(
                    super::allocation::map_entry::<String, SourceSpan>(),
                    path.capacity(),
                )?,
            )
        })
    }
    pub(super) fn insert(&mut self, path: String, span: SourceSpan) {
        self.spans.insert(path, span);
    }
}

pub(super) fn child_path(parent: &str, key: &str) -> String {
    format!("{parent}/{}", key.replace('~', "~0").replace('/', "~1"))
}
