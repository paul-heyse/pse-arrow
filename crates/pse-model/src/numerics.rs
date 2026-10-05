// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-neutral numerical requests and their single resolved interpretation.
use crate::{
    SemanticFrame,
    generated::{
        authored::{
            accuracy_goals, engineering_default_rules, engineering_scales, numerical_requirements,
        },
        enums::{
            ClosurePolicy, IncumbentPolicy, NumericalProvenanceField, NumericalSource,
            NumericalTarget,
        },
    },
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};

/// One registry declaration, also used for explicit analysis overrides.
pub type NumericalRequirement = numerical_requirements::Row;
/// Registry-owned engineering goal declaration, shared by authored and analysis sources.
pub type AccuracyGoal = accuracy_goals::Row;
/// Explicitly tagged engineering characteristic magnitude, separate from conditioning.
pub type EngineeringScale = engineering_scales::Row;
/// Typed inherited engineering allowance rule.
pub type EngineeringRule = engineering_default_rules::Row;
/// Frozen interpretation of a contextual engineering default in target-unit magnitudes.
pub type EngineeringContext =
    crate::generated::runtime::resolved_numerics::RuntimeResolvedNumericsFieldEngineering;

/// Starting accuracy for ordinary engineering design: 0.1% of a frozen meaningful
/// characteristic scale. It is not an output-error guarantee. Decision-sensitive
/// analyses and verification supply explicit tighter requirements and KKT budgets.
pub const DEFAULT_ENGINEERING_ACCURACY: f64 = 1e-3;

/// Distinct normalized optimality requirements; no scalar means every stopping test.
#[derive(
    Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
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
            stationarity: DEFAULT_ENGINEERING_ACCURACY,
            complementarity: DEFAULT_ENGINEERING_ACCURACY,
        }
    }
}
/// Shared semantic controls resolved before invoking any native adapter.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NumericalPolicy {
    /// Required standalone and nested numerical-policy interpretation version.
    pub version: crate::document::Version<1>,
    /// ID-keyed analysis overrides, using the same declaration as model/case sources.
    #[serde(default)]
    pub requirements: Vec<NumericalRequirement>,
    /// Selected output goals; empty does not assert decision invariance.
    #[serde(default)]
    pub goals: Vec<AccuracyGoal>,
    /// Admitted explicitly tagged scales, with source and scope retained.
    #[serde(default)]
    pub engineering_scales: Vec<EngineeringScale>,
    /// Shared rules matched by full quantity meaning, not dimensions alone.
    #[serde(default)]
    pub engineering_rules: Vec<EngineeringRule>,
    /// Reject an engineering allowance relying on the canonical fallback.
    #[serde(default)]
    pub strict_engineering_context: bool,
    /// Shared relative engineering fraction; an explicit selected rule may override it.
    #[serde(default = "policy_defaults::engineering_relative_fraction")]
    pub engineering_relative_fraction: f64,
    /// Refuse canonical-unit fallback where no authored or quantity nominal is known.
    #[serde(default)]
    pub strict_nominals: bool,
    /// Permit native algorithmic scaling after model normalization.
    #[serde(default = "policy_defaults::native_scaling")]
    pub native_scaling: bool,
    /// Operational normalized supplier action error, independent of KKT acceptance.
    #[serde(default = "policy_defaults::supplier_action_accuracy")]
    pub supplier_action_accuracy: f64,
    /// Implicit derivative linear-system backward error, independent of stationarity.
    #[serde(default = "policy_defaults::linear_backward_error")]
    pub linear_backward_error: f64,
    /// Independent KKT budgets.
    #[serde(default)]
    pub kkt: KktTolerances,
    /// Optional separately qualified acceptable-termination budgets.
    #[serde(default)]
    pub acceptable: Option<KktTolerances>,
    /// Integer-lattice violation budget in original variable coordinates.
    #[serde(default = "policy_defaults::integrality")]
    pub integrality: f64,
    /// Absolute continuous optimality gap in normalized objective coordinates.
    #[serde(default = "policy_defaults::gap_absolute")]
    pub gap_absolute: f64,
    /// Dimensionless relative continuous optimality gap.
    #[serde(default = "policy_defaults::gap_relative")]
    pub gap_relative: f64,
    /// Absolute gap in normalized objective coordinates.
    #[serde(default = "policy_defaults::mip_absolute_gap")]
    pub mip_absolute_gap: f64,
    /// Dimensionless relative MIP gap.
    #[serde(default = "policy_defaults::mip_relative_gap")]
    pub mip_relative_gap: f64,
    /// Physical acceptance, separate from native termination.
    #[serde(default = "policy_defaults::closure")]
    pub closure: ClosurePolicy,
    /// Whether a validated original-feasible limit incumbent may be used as a result.
    #[serde(default = "policy_defaults::incumbent")]
    pub incumbent: IncumbentPolicy,
}
impl Default for NumericalPolicy {
    fn default() -> Self {
        Self {
            version: crate::document::Version,
            requirements: vec![],
            goals: vec![],
            engineering_scales: vec![],
            engineering_rules: vec![],
            strict_engineering_context: false,
            engineering_relative_fraction: DEFAULT_ENGINEERING_ACCURACY,
            strict_nominals: false,
            native_scaling: true,
            supplier_action_accuracy: DEFAULT_ENGINEERING_ACCURACY,
            linear_backward_error: DEFAULT_ENGINEERING_ACCURACY,
            kkt: KktTolerances::default(),
            acceptable: None,
            // Integer feasibility protects the discrete domain, not engineering
            // output resolution; relaxing it can change the selected decision.
            integrality: 1e-8,
            gap_absolute: DEFAULT_ENGINEERING_ACCURACY,
            gap_relative: DEFAULT_ENGINEERING_ACCURACY,
            mip_absolute_gap: DEFAULT_ENGINEERING_ACCURACY,
            mip_relative_gap: DEFAULT_ENGINEERING_ACCURACY,
            closure: ClosurePolicy::RequireClosed,
            incumbent: IncumbentPolicy::Refuse,
        }
    }
}
mod policy_defaults {
    use super::*;
    macro_rules! field {
        ($name:ident, $ty:ty) => {
            pub(super) fn $name() -> $ty {
                NumericalPolicy::default().$name
            }
        };
    }
    field!(engineering_relative_fraction, f64);
    field!(native_scaling, bool);
    field!(supplier_action_accuracy, f64);
    field!(linear_backward_error, f64);
    field!(integrality, f64);
    field!(gap_absolute, f64);
    field!(gap_relative, f64);
    field!(mip_absolute_gap, f64);
    field!(mip_relative_gap, f64);
    field!(closure, ClosurePolicy);
    field!(incumbent, IncumbentPolicy);
}
impl NumericalPolicy {
    /// Check only library-neutral policy meaning; adapters validate their native ranges.
    pub fn validate(&self) -> Result<(), crate::ModelError> {
        if [
            self.kkt.stationarity,
            self.kkt.complementarity,
            self.supplier_action_accuracy,
            self.linear_backward_error,
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
        if !self.engineering_relative_fraction.is_finite()
            || self.engineering_relative_fraction < 0.0
        {
            return Err(crate::malformed("invalid shared engineering fraction"));
        }
        let mut goals = std::collections::BTreeSet::new();
        for goal in &self.goals {
            if !goals.insert(goal.goal_id) {
                return Err(crate::malformed(
                    "duplicate engineering accuracy goal identity",
                ));
            }
            crate::engineering_accuracy::validate_goal(goal)?;
        }
        let mut scales = std::collections::BTreeSet::new();
        for scale in &self.engineering_scales {
            if !scales.insert(scale.scale_id)
                || !scale.value.is_finite()
                || scale.value < 0.0
                || scale.provenance.trim().is_empty()
                || matches!(
                    scale.source,
                    NumericalSource::DerivedNominal | NumericalSource::CanonicalFallback
                )
            {
                return Err(crate::malformed(
                    "invalid or duplicate engineering scale declaration",
                ));
            }
        }
        let mut rules = std::collections::BTreeSet::new();
        for rule in &self.engineering_rules {
            if !rules.insert(rule.rule_id)
                || rule
                    .physical_allowance
                    .is_some_and(|v| !v.is_finite() || v <= 0.0)
                || rule
                    .relative_fraction
                    .is_some_and(|v| !v.is_finite() || v < 0.0)
                || rule.provenance.trim().is_empty()
            {
                return Err(crate::malformed(
                    "invalid or duplicate engineering default rule",
                ));
            }
        }
        Ok(())
    }
    /// Effective request identity with canonical registry framing and source ordering.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::NumericalPolicyV3);
        let mut rows = self.requirements.iter().collect::<Vec<_>>();
        rows.sort_by_key(|r| r.requirement_id);
        h.u64(rows.len() as u64);
        for row in rows {
            row.frame(&mut h);
        }
        let mut goals = self.goals.iter().collect::<Vec<_>>();
        goals.sort_by_key(|r| r.goal_id);
        h.u64(goals.len() as u64);
        for goal in goals {
            goal.frame(&mut h);
        }
        let mut scales = self.engineering_scales.iter().collect::<Vec<_>>();
        scales.sort_by_key(|r| r.scale_id);
        h.u64(scales.len() as u64);
        for scale in scales {
            scale.frame(&mut h);
        }
        let mut rules = self.engineering_rules.iter().collect::<Vec<_>>();
        rules.sort_by_key(|r| r.rule_id);
        h.u64(rules.len() as u64);
        for rule in rules {
            rule.frame(&mut h);
        }
        h.bool(self.strict_engineering_context)
            .u64(pse_ids::canonical_f64_bits(
                self.engineering_relative_fraction,
            ));
        h.bool(self.strict_nominals)
            .bool(self.native_scaling)
            .bool(self.acceptable.is_some());
        for value in [
            self.kkt.stationarity,
            self.kkt.complementarity,
            self.supplier_action_accuracy,
            self.linear_backward_error,
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
        self.incumbent.frame(&mut h);
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
    /// The numerical field resolved by this entry.
    pub field: NumericalProvenanceField,
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
    /// Selected engineering default interpretation; absent for explicit numeric budgets.
    pub engineering: Option<EngineeringContext>,
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

#[cfg(test)]
mod policy_identity_tests {
    use super::*;
    #[test]
    fn engineering_defaults_preserve_discrete_and_physical_acceptance() {
        let policy = NumericalPolicy::default();
        policy.validate().unwrap();
        for budget in [
            policy.kkt.stationarity,
            policy.kkt.complementarity,
            policy.gap_absolute,
            policy.gap_relative,
            policy.mip_absolute_gap,
            policy.mip_relative_gap,
        ] {
            assert_eq!(budget, DEFAULT_ENGINEERING_ACCURACY);
        }
        assert_eq!(policy.integrality, 1e-8);
        assert_eq!(policy.acceptable, None);
        assert_eq!(policy.incumbent, IncumbentPolicy::Refuse);
        assert_eq!(policy.closure, ClosurePolicy::RequireClosed);
        let mut verification = policy.clone();
        verification.kkt.stationarity = 1e-8;
        verification.kkt.complementarity = 1e-8;
        verification.validate().unwrap();
        assert_ne!(verification.key(), policy.key());
    }
    #[test]
    fn incumbent_permission_has_an_independent_request_identity() {
        let conservative = NumericalPolicy::default();
        assert_eq!(conservative.incumbent, IncumbentPolicy::Refuse);
        assert_eq!(conservative.closure, ClosurePolicy::RequireClosed);
        let mut feasible = conservative.clone();
        feasible.incumbent = IncumbentPolicy::AcceptFeasible;
        let mut within_gap = conservative.clone();
        within_gap.incumbent = IncumbentPolicy::AcceptWithinGap;
        assert_ne!(conservative.key(), feasible.key());
        assert_ne!(conservative.key(), within_gap.key());
        assert_ne!(feasible.key(), within_gap.key());
    }
}
