// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The exact declaration-key steps of a normalized instance-relative path.

/// A path step, resolved under the actual selected owning instance and template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpressionPathSegmentKind {
    /// Declared submodel composite key under the current template.
    Child,
    /// A declared symbol or another explicitly bound member of the selected instance.
    Member,
    /// Root instance selected by an actual semantic-ID template parameter.
    InstanceParameter,
}
impl ExpressionPathSegmentKind {
    /// Complete admitted step vocabulary.
    pub const ALL: [Self; 3] = [Self::Child, Self::Member, Self::InstanceParameter];
    /// Exact wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Child => "child",
            Self::Member => "member",
            Self::InstanceParameter => "instance_parameter",
        }
    }
}
