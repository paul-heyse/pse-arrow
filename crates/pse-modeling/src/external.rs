// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checked foreign-function meaning, independent of concrete runtime factories.
use crate::{DeclarationId, Result, invalid};
use pse_authoring::dsl::Expr;
use pse_ids::{ContentHash, SemanticId};
pub use pse_model::generated::enums::ExternalDerivativeSource as DerivativeSource;
/// Immutable selected external implementation and declared derivative contract.
#[derive(Clone, Debug, PartialEq)]
pub struct External {
    /// Registered implementation name.
    pub implementation: String,
    /// Content hash of the implementation revision the model was authored against.
    pub revision: ContentHash,
    /// Content hash of the implementation's bound data.
    pub data: ContentHash,
    /// Output ordinal; an exact integer literal after specialization.
    pub output: Expr,
    /// Where the implementation's derivatives come from.
    pub derivative_source: DerivativeSource,
    /// Highest derivative order the implementation supplies, at most two.
    pub derivatives: u8,
    /// Declared continuity order of the implementation, at most two.
    pub smoothness: u8,
    /// Instantiated input axes after finite specialization.
    pub shapes: Vec<ArgumentShape>,
}
/// One finite indexed argument's ordered coordinates and scalar gather range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentShape {
    /// Identity of the indexed formal argument.
    pub argument: SemanticId,
    /// Ordered domain identities of the argument.
    pub axes: Vec<SemanticId>,
    /// Ordered coordinate tuples of the admitted group.
    pub coordinates: Vec<Vec<SemanticId>>,
    /// Position of the argument's first scalar in the flattened call inputs.
    pub start: usize,
}
impl External {
    pub(crate) fn check(
        v:&pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueFunctionExternal,
        at: DeclarationId,
        output: &Expr,
    ) -> Result<Self> {
        if v.implementation.is_empty()
            || !(0..=2).contains(&v.derivatives)
            || !(0..=2).contains(&v.smoothness)
        {
            return Err(invalid(
                at,
                "external implementation and derivative profile required",
            ));
        }
        Ok(Self {
            implementation: v.implementation.clone(),
            revision: ContentHash::parse_hex(&v.revision)
                .map_err(|e| invalid(at, e.to_string()))?,
            data: ContentHash::parse_hex(&v.data).map_err(|e| invalid(at, e.to_string()))?,
            output: output.clone(),
            derivative_source: v.derivative_source,
            derivatives: v.derivatives as u8,
            smoothness: v.smoothness as u8,
            shapes: vec![],
        })
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self.implementation.capacity()
            + crate::expression::retained_bytes(&self.output)
            + self
                .shapes
                .iter()
                .map(|s| {
                    size_of::<ArgumentShape>()
                        + s.axes.capacity() * size_of::<SemanticId>()
                        + s.coordinates.capacity() * size_of::<Vec<SemanticId>>()
                        + s.coordinates
                            .iter()
                            .map(|c| c.capacity() * size_of::<SemanticId>())
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}
