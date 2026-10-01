// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original implicit residual definitions for the library-neutral factorable projection.
use super::implicit::{ImplicitMeaning, SelectionEquivalence};
use super::{AdmittedImplicit, ModelingHint};
use pse_ids::SemanticId;
use pse_kernels::ProviderKey;
use pse_math::factorable::{ImplicitBounds, ImplicitDefinition, SelectedGraph};

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
        let represented_domain =
            self.selection.restriction.is_none() || self.selection.sign.is_some();
        let selection = match (
            &self.selection.meaning,
            self.selection.equivalence,
            represented_domain,
        ) {
            (ImplicitMeaning::Relation, _, _) => SelectedGraph::Relation,
            (_, SelectionEquivalence::NondegenerateAffine, true) => {
                SelectedGraph::NondegenerateAffine {
                    sign: self.selection.sign,
                    strict: self.selection.strict,
                }
            }
            (_, SelectionEquivalence::RestrictedSquareRoot, true) => match self.selection.sign {
                Some(positive) => SelectedGraph::RestrictedSquareRoot {
                    positive,
                    strict: self.selection.strict,
                },
                None => SelectedGraph::Unestablished {
                    meaning: "restricted-square-root:missing-sign".into(),
                },
            },
            _ => SelectedGraph::Unestablished {
                meaning: match &self.selection.meaning {
                    ImplicitMeaning::Relation => "relation".into(),
                    ImplicitMeaning::Unique => "unique".into(),
                    ImplicitMeaning::Branch => "branch".into(),
                    ImplicitMeaning::Operational(settings) => format!("operational:{settings}"),
                    ImplicitMeaning::MinimumScore => "minimum-score".into(),
                },
            },
        };
        Some((
            self.descriptor.spec().key(),
            ImplicitDefinition {
                residual: residual.body.math.clone(),
                unknowns: vec![(f64::NEG_INFINITY, f64::INFINITY); self.unknowns.len()],
                bounds,
                selection,
            },
        ))
    }
}
