// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original implicit residual definitions for the library-neutral factorable projection.
use super::{AdmittedImplicit, ModelingHint};
use pse_ids::SemanticId;
use pse_kernels::ProviderKey;
use pse_math::factorable::{ImplicitBounds, ImplicitDefinition};

impl AdmittedImplicit {
    /// The original residual definition a factorable export uses in place of this block's
    /// realization (nested, accelerated or affine), keyed by the provider its callers
    /// invoke. Declared bounds are the block's bound hints, which the evaluator enforces;
    /// the case owner may intersect `unknowns` with declared case bounds of the unknowns.
    /// A regime selection has no single residual and stays a provider output.
    pub fn factorable_definition(&self) -> Option<(ProviderKey, ImplicitDefinition)> {
        let [residual] = self.residuals.as_slice() else {
            return None;
        };
        if residual.assessment.is_some() {
            return None;
        }
        let bounds = residual.hints.as_ref().map(|hints| {
            let ordinal = |unknown: &SemanticId, kind: ModelingHint| {
                residual
                    .hint_targets
                    .iter()
                    .position(|(target, _, k)| target == unknown && *k == kind)
            };
            ImplicitBounds {
                body: hints.math.clone(),
                lower: self
                    .unknowns
                    .iter()
                    .map(|u| ordinal(u, ModelingHint::Lower))
                    .collect(),
                upper: self
                    .unknowns
                    .iter()
                    .map(|u| ordinal(u, ModelingHint::Upper))
                    .collect(),
            }
        });
        Some((
            self.descriptor.spec().key(),
            ImplicitDefinition {
                residual: residual.body.math.clone(),
                unknowns: vec![(f64::NEG_INFINITY, f64::INFINITY); self.unknowns.len()],
                bounds,
            },
        ))
    }
}
