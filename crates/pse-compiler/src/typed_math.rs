// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Direct producer-issued typed graphs to immutable numerical bodies.
#[path = "typed_math/direct.rs"]
mod direct;
#[path = "typed_math_portable.rs"]
mod portable;
#[path = "typed_math/providers.rs"]
mod providers;
pub use direct::Request;
pub use portable::{portable_payload_hash, reconstruct_portable};
pub use providers::{SelectedProviderCall, SelectedProviders, prepare_selected_providers};
use pse_ids::{ContentHash, SemanticId};
use pse_math::{MathError, binding::BodySpec, typed::BodyLimits};
use pse_quantity::{QuantityRegistry, QuantityTypeId};
use std::sync::{Arc, atomic::AtomicBool};
/// Current receiving field occurrence, exterior to every reusable numerical body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Occurrence {
    /// Actual original occurrence identity.
    pub id: SemanticId,
    /// Actual original declaring source owner.
    pub definition: SemanticId,
    /// Original authored field location.
    pub span: pse_authoring::dsl::Span,
}
/// Stable local numerical occurrence role; actual source locations are instance bindings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LocalOccurrence {
    pub id: SemanticId,
}
/// Typed physical input metadata for projection inspection; it does not lower source names.
#[derive(Clone, Debug, PartialEq)]
pub struct Formal {
    /// Actual source member path for inspection.
    pub path: String,
    /// Admitted nominal scalar quantity.
    pub quantity: QuantityTypeId,
}
/// One actual registered provider selection resolved by the inventory owner.
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderCall {
    /// Actual immutable admitted registration.
    pub descriptor: pse_kernels::AdmittedProvider,
    /// Selected scalar output in registration order.
    pub output: usize,
}
/// Immutable physically admitted definition, independent of compilation options.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedBody {
    semantic_identity: Option<pse_ids::roles::SemanticBodyHash>,
    portable: Option<Arc<[u8]>>,
    /// Complete admission and dependency identity descriptor.
    pub(crate) spec: BodySpec,
    /// Result's complete physical contract.
    pub(crate) quantities: Vec<QuantityTypeId>,
    /// Authored occurrence provenance.
    pub(crate) occurrences: Vec<LocalOccurrence>,
    /// Symbolica regions and complete structural support, without mutable workspaces.
    pub(crate) math: Arc<pse_math::guarded::PreparedBody>,
}
impl AdmittedBody {
    /// Compiler-issued complete dependency identity, sealed against external mutation.
    pub fn semantic_identity(&self) -> Option<pse_ids::roles::SemanticBodyHash> {
        self.semantic_identity
    }
    pub(crate) fn for_semantic_identity(
        mut self,
        identity: pse_ids::roles::SemanticBodyHash,
    ) -> Result<Self, MathError> {
        if let Some(payload) = &self.portable {
            self.portable = match portable::attach_identity(payload, identity) {
                Ok(payload) => Some(payload),
                Err(MathError::ByteLimit {
                    resource: "portable body payload",
                    ..
                }) => None,
                Err(error) => return Err(error),
            };
        }
        self.semantic_identity = Some(identity);
        Ok(self)
    }
    /// Whether this admitted workflow retained complete portable construction receipts.
    pub fn has_portable_payload(&self) -> bool {
        self.portable.is_some()
    }
    /// Encode the selected admitted semantic recipe and complete concrete physical receipts.
    /// # Errors
    /// A workflow lacking complete receipt coverage or an oversized payload.
    pub fn portable_payload(&self) -> Result<Vec<u8>, MathError> {
        Ok(self
            .portable
            .as_ref()
            .ok_or_else(|| {
                MathError::Contract("body lacks complete portable construction records".into())
            })?
            .to_vec())
    }
    /// Exact encoded extent, before a caller reserves encoding scratch.
    pub fn portable_payload_bytes(&self) -> Result<usize, MathError> {
        Ok(self
            .portable
            .as_ref()
            .ok_or_else(|| {
                MathError::Contract("body lacks complete portable construction records".into())
            })?
            .len())
    }
    /// Borrow its admitted specification without exposing mutation of its witness.
    pub fn spec(&self) -> &BodySpec {
        &self.spec
    }
    /// Ordered admitted output quantities.
    pub fn quantities(&self) -> &[QuantityTypeId] {
        &self.quantities
    }
    /// Immutable admitted mathematics; body clones retain original accounting.
    pub fn math(&self) -> &Arc<pse_math::guarded::PreparedBody> {
        &self.math
    }
    /// Known descriptor/vector allocation extent, excluding shared mathematics.
    pub fn descriptor_bytes(&self) -> usize {
        size_of::<Self>()
            + self.spec.providers.capacity() * size_of::<ContentHash>()
            + self.quantities.capacity() * size_of::<QuantityTypeId>()
            + self.occurrences.capacity() * size_of::<LocalOccurrence>()
            + self.portable.as_ref().map_or(0, |v| v.len())
    }
    /// Additional owned descriptor and math-handle wrappers when attaching an owner;
    /// immutable library mathematics stays shared.
    pub(crate) fn owner_attachment_bytes(&self) -> usize {
        size_of::<Self>()
            + size_of::<pse_math::guarded::PreparedBody>()
            + 256
            + self.spec.providers.len() * size_of::<ContentHash>()
            + self.quantities.len() * size_of::<QuantityTypeId>()
            + self.occurrences.len() * size_of::<LocalOccurrence>()
    }
    /// Attach accounting without changing sealed scientific meaning.
    pub fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.math = Arc::new(self.math.as_ref().clone().with_owner(owner));
        self
    }
}

#[cfg(test)]
#[path = "typed_math_direct_tests.rs"]
pub(crate) mod direct_tests;
