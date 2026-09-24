// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native document ingestion, checked Arrow construction and source editing.
pub mod document;
mod error;
pub mod native;
mod native_relations;
pub mod p0;
pub mod p1;
pub mod targets;
mod work;
pub use error::AuthoringDriverError as DriverError;
use pse_authoring::{ParseBudget, SourceSpan, dsl, ids};

pub(crate) fn contract(reason: impl Into<String>) -> DriverError {
    pse_authoring::AuthoringError::Contract {
        at: None,
        reason: reason.into(),
    }
    .into()
}
