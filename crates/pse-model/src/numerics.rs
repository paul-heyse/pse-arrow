// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-neutral numerical requests and their single resolved interpretation.
use crate::{
    SemanticFrame,
    generated::{
        authored::numerical_requirements,
        enums::{ClosurePolicy, NumericalSource, NumericalTarget},
    },
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};

/// One registry declaration, also used for explicit analysis overrides.
pub type NumericalRequirement = numerical_requirements::Row;

/// Distinct normalized optimality requirements; no scalar means every stopping test.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KktTolerances {
    /// Normalized stationarity budget.
    pub stationarity: f64,
    /// Normalized complementarity budget.
    pub complementarity: f64,
}
impl Default for KktTolerances {
    fn default() -> Self {
        Self {
            stationarity: 1e-8,
            complementarity: 1e-8,
        }
    }
}
/// Shared semantic controls resolved before invoking any native adapter.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NumericalPolicy {
    /// ID-keyed analysis overrides, using the same declaration as model/case sources.
    pub requirements: Vec<NumericalRequirement>,
    /// Refuse canonical-unit fallback where no authored or quantity nominal is known.
    pub strict_nominals: bool,
    /// Permit native algorithmic scaling after model normalization.
    pub native_scaling: bool,
    /// Independent KKT budgets.
    pub kkt: KktTolerances,
    /// Optional separately qualified acceptable-termination budgets.
    pub acceptable: Option<KktTolerances>,
    /// Integer-lattice violation budget in original variable coordinates.
    pub integrality: f64,
    /// Absolute continuous optimality gap in normalized objective coordinates.
    pub gap_absolute: f64,
    /// Dimensionless relative continuous optimality gap.
    pub gap_relative: f64,
    /// Absolute gap in normalized objective coordinates.
    pub mip_absolute_gap: f64,
    /// Dimensionless relative MIP gap.
    pub mip_relative_gap: f64,
    /// Physical acceptance, separate from native termination.
    pub closure: ClosurePolicy,
}
impl Default for NumericalPolicy {
    fn default() -> Self {
        Self {
            requirements: vec![],
            strict_nominals: false,
            native_scaling: true,
            kkt: KktTolerances::default(),
            acceptable: None,
            integrality: 1e-8,
            gap_absolute: 1e-8,
            gap_relative: 1e-8,
            mip_absolute_gap: 1e-6,
            mip_relative_gap: 1e-4,
            closure: ClosurePolicy::RequireClosed,
        }
    }
}
impl NumericalPolicy {
    /// Check only library-neutral policy meaning; adapters validate their native ranges.
    pub fn validate(&self) -> Result<(), crate::ModelError> {
        if [
            self.kkt.stationarity,
            self.kkt.complementarity,
            self.integrality,
            self.gap_absolute,
            self.gap_relative,
        ]
        .into_iter()
        .any(|v| !v.is_finite() || v <= 0.0)
            || [self.mip_absolute_gap, self.mip_relative_gap]
                .into_iter()
                .any(|v| !v.is_finite() || v < 0.0)
            || self.acceptable.is_some_and(|k| {
                !k.stationarity.is_finite()
                    || !k.complementarity.is_finite()
                    || k.stationarity < self.kkt.stationarity
                    || k.complementarity < self.kkt.complementarity
            })
        {
            return Err(crate::malformed(
                "invalid independent numerical acceptance budgets",
            ));
        }
        Ok(())
    }
    /// Effective request identity with canonical registry framing and source ordering.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new("pse.numerical.policy.v1");
        let mut rows = self.requirements.iter().collect::<Vec<_>>();
        rows.sort_by_key(|r| r.requirement_id);
        h.u64(rows.len() as u64);
        for row in rows {
            row.frame(&mut h);
        }
        h.bool(self.strict_nominals)
            .bool(self.native_scaling)
            .bool(self.acceptable.is_some());
        for value in [
            self.kkt.stationarity,
            self.kkt.complementarity,
            self.integrality,
            self.gap_absolute,
            self.gap_relative,
            self.mip_absolute_gap,
            self.mip_relative_gap,
        ] {
            h.u64(pse_ids::canonical_f64_bits(value));
        }
        if let Some(k) = self.acceptable {
            h.u64(k.stationarity.to_bits())
                .u64(k.complementarity.to_bits());
        }
        self.closure.frame(&mut h);
        h.finish_hash()
    }
}
/// Provenance for a selected or overridden declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericalProvenance {
    /// Authored source identity; defaults have no invented authored row.
    pub declaration: Option<SemanticId>,
    /// Semantic precedence category.
    pub source: NumericalSource,
    /// Name of the numerical field resolved by this entry.
    pub field: &'static str,
    /// Whether this candidate established the effective field.
    pub selected: bool,
    /// Candidate magnitude after conversion to this target representation.
    pub value: f64,
    /// Human-readable retained source description.
    pub description: String,
}
/// A requirement resolved into magnitudes in each target's declared unit coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedTarget {
    /// Original source target; the objective uses NIL because it has no row identity.
    pub id: SemanticId,
    /// Meaning of this coordinate.
    pub kind: NumericalTarget,
    /// Physical quantity contract of this coordinate.
    pub quantity: SemanticId,
    /// Unit representing original oracle values and these magnitude budgets.
    pub unit: SemanticId,
    /// Frozen target-unit nominal used for relative acceptance.
    pub nominal: f64,
    /// Positive model coordinate transformation; integers retain scale one.
    pub coordinate_scale: f64,
    /// A selected required declaration fixes the coordinate substitution.
    pub required_scale: bool,
    /// Physical absolute tolerance magnitude.
    pub absolute: f64,
    /// Dimensionless relative tolerance.
    pub relative: f64,
    /// Frozen physical absolute plus relative budget.
    pub budget: f64,
    /// Selected and overridden interpretation sources.
    pub provenance: Vec<NumericalProvenance>,
}
/// Immutable result of resolving one admitted analysis.
#[derive(Clone, Debug)]
pub struct ResolvedNumericalPolicy {
    /// Independent semantic controls and original analysis requirements.
    pub policy: NumericalPolicy,
    /// Canonical selected coordinates and their budgets.
    pub targets: Vec<ResolvedTarget>,
    /// Complete interpretation identity, including provenance.
    pub key: ContentHash,
}
