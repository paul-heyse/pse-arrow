// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded target syntax resolves through actual instance, declaration and domain rows.

mod native;
mod parse;
pub use native::resolve_native;

use crate::SourceSpan;
use pse_relations::generated::authored;

pub use parse::parse;
/// Exact generated target row contract; its source identity is supplied to resolution.
pub type TargetRow = authored::case_spec_targets::Row;

/// One concrete domain selector, before identity resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexSelector {
    /// The complete declared indexed shape.
    Wildcard,
    /// An exact semantic ID, member label or finite coordinate spelling.
    Value(String),
    /// A quoted exact member label, without numeric-coordinate interpretation.
    Label(String),
}
/// A retained target path with original source provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetPath {
    /// Instance/member names in path order.
    pub names: Vec<String>,
    /// Selectors on the final member, in declaration order.
    pub indices: Vec<IndexSelector>,
    /// Whether the final path segment selects every member of the instance.
    pub instance_wildcard: bool,
    /// Original source position.
    pub at: SourceSpan,
    /// Original text, used in diagnostic reports.
    pub text: String,
}
