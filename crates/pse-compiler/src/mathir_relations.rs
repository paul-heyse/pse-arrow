// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry-admitted math relation adapters shared by normalization and physical typing.

pub(crate) mod sink;
mod source;
pub use sink::{Family, RelationSink};
pub use source::{RelationSource, SourceFamily};

use pse_mathir::MathIrError;
pub(crate) fn malformed(detail: impl Into<String>) -> MathIrError {
    MathIrError::Malformed {
        node: None,
        detail: detail.into(),
    }
}
