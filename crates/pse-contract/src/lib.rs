// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pure source-owned boundary contracts, without product or wire dependencies.
pub mod boundary_projection;
pub mod operation_effects;
/// Crossing direction of an owner-derived boundary declaration.
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash,serde::Serialize,serde::Deserialize,strum::EnumString,strum::Display)]
#[serde(rename_all="snake_case")]
#[strum(serialize_all="snake_case",const_into_str)]
pub enum BoundaryDirection {
    /// Accepted from clients.
    Input,
    /// Produced for clients.
    Output,
    /// Shared input and output meaning.
    Both,
}
impl BoundaryDirection {
 /// Every member in registry order.
 pub const ALL:[Self;3]=[Self::Input,Self::Output,Self::Both];
 /// Canonical registry spelling.
 pub const fn as_str(self)->&'static str {match self {Self::Input=>"input",Self::Output=>"output",Self::Both=>"both"}}
 /// Parse a canonical spelling without inventing an unknown member.
 pub fn parse(text:&str)->Option<Self>{text.parse().ok()}
}
impl BoundaryDirection {
    /// Whether this declaration participates in the selected projection.
    pub const fn includes(self, selected: Self) -> bool {
        matches!(self, Self::Both)
            || matches!(selected, Self::Both)
            || matches!(
                (self, selected),
                (Self::Input, Self::Input) | (Self::Output, Self::Output)
            )
    }
}
