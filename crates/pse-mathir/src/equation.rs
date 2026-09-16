// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Indexed equation records and residual contracts (blueprint §6.9, §7.5).
use crate::{MathIrError, NodeId};
use pse_ids::SemanticId;
use pse_quantity::{BoundIndexId, DomainId, QuantityTypeId};

use pse_schema::math::Sense;

/// One declared free-index axis, retaining its semantic position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FreeIndex {
    /// Binder identity.
    pub bound_index: BoundIndexId,
    /// Domain containing its members.
    pub domain: DomainId,
    /// Position in the equation's declared index product.
    pub position: u16,
}
/// Exactly the indexed equation facts stored before and after P10.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EquationRecord {
    /// Stable indexed equation identity.
    pub indexed_equation_id: SemanticId,
    /// Owning instance.
    pub owner_instance: SemanticId,
    /// Optional declaration reference.
    pub equation_decl: Option<SemanticId>,
    /// Diagnostic qualified name; source spans remain in the referenced derivation.
    pub qualified_name: String,
    /// Optional declared index product.
    pub product: Option<SemanticId>,
    /// Optional Boolean filter expression.
    pub filter: Option<NodeId>,
    /// Mathematical body.
    pub body: NodeId,
    /// Bound semantics.
    pub sense: Sense,
    /// Optional lower bound expression.
    pub lower: Option<NodeId>,
    /// Optional upper bound expression.
    pub upper: Option<NodeId>,
    /// Ordered free-index declarations (separate relation in storage).
    pub free_indices: Vec<FreeIndex>,
    /// Absent before inference; complete subtraction result after P10.
    pub residual_quantity_type: Option<QuantityTypeId>,
    /// Optional law provenance.
    pub law_instance: Option<SemanticId>,
    /// Source derivation, including source-span provenance.
    pub derivation: SemanticId,
}
impl EquationRecord {
    /// All expression references, in body/lower/upper/filter order.
    pub fn referenced_nodes(&self) -> Vec<NodeId> {
        std::iter::once(self.body)
            .chain(self.lower)
            .chain(self.upper)
            .chain(self.filter)
            .collect()
    }
    /// Remap all expression references without changing equation identities or provenance.
    ///
    /// # Errors
    /// Propagates any unresolved node reference.
    pub fn map_node_references(
        &mut self,
        mut map: impl FnMut(NodeId) -> Result<NodeId, MathIrError>,
    ) -> Result<(), MathIrError> {
        self.body = map(self.body)?;
        self.lower = self.lower.map(&mut map).transpose()?;
        self.upper = self.upper.map(&mut map).transpose()?;
        self.filter = self.filter.map(&mut map).transpose()?;
        Ok(())
    }
}
