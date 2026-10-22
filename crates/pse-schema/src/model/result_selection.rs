// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Access roles carried by the scientific relation's own field declarations.
/// A scientific field's role in protected result selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultSelectionRole {
    /// Original execution identity.
    Run,
    /// Original scientific output identity.
    Owner,
    /// Original solve step partition.
    Step,
    /// Original fit experiment partition.
    Experiment,
    /// Scientific scalar on which finite/missing predicates are supported.
    Value,
}
impl ResultSelectionRole {
    /// Stable field facet spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Owner => "owner",
            Self::Step => "step",
            Self::Experiment => "experiment",
            Self::Value => "value",
        }
    }
    /// Closed lookup; an unknown role is never treated as a value.
    pub fn from_name(name: &str) -> Option<Self> {
        [
            Self::Run,
            Self::Owner,
            Self::Step,
            Self::Experiment,
            Self::Value,
        ]
        .into_iter()
        .find(|role| role.as_str() == name)
    }
}
