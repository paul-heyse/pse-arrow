// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry-admitted math relation adapters shared by normalization and physical typing.

mod sink;
mod source;
pub use sink::{Family, RelationSink};
pub use source::RelationSource;

use pse_mathir::MathIrError;
use pse_schema::model::{Cell, RelationKey};
use std::collections::BTreeMap;

/// Actual ordered values under registered mathematical relation identities.
pub type MathRows = BTreeMap<RelationKey, Vec<Vec<Cell>>>;

pub(crate) fn malformed(detail: impl Into<String>) -> MathIrError {
    MathIrError::Malformed {
        node: None,
        detail: detail.into(),
    }
}
