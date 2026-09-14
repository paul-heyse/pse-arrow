// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Cached, independently admitted row counts; unrecorded statistics remain absent.

use crate::LoadedRelation;
use datafusion::common::{Statistics, stats::Precision};

/// No I/O or row scan occurs when the optimizer requests these statistics.
pub fn cached(relation: &LoadedRelation) -> Statistics {
    Statistics::new_unknown(relation.batch().schema().as_ref())
        .with_num_rows(Precision::Exact(relation.rows()))
}
