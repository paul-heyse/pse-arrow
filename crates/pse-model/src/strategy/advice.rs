// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Frozen descriptive comparisons. They grant neither eligibility nor numerical tuning.
use super::WorkLimits;
use crate::{ModelError, malformed};
use pse_ids::ContentHash;
use std::collections::{BTreeMap, BTreeSet};

crate::projection_struct! {
/// Exact comparison premises chosen before observing outcomes.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AdviceCohort {
    projection: { name: "NumericalAdviceCohort", rust_path: crate::strategy::advice::AdviceCohort,
        version: 1, direction: crate::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Original problem and physical/provider interpretation.
    pub original: ContentHash => { tag: 1, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// Equivalent permitted profile family; explicit settings remain constraints.
    pub profiles: ContentHash => { tag: 2, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// Original branch permission and actual start policy.
    pub branch_start: ContentHash => { tag: 3, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// Original accuracy and assurance requirements.
    pub accuracy: ContentHash => { tag: 4, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// Declared exposure/allocation design, including placement and interval meaning.
    pub exposure: ContentHash => { tag: 5, direction: crate::boundary_projection::Direction::Both, rules: [] };
}
}
crate::projection_enum! {
/// Lower is better, within the exact cohort and one metric only.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AdviceMetric {
    projection: { name: "NumericalAdviceMetric", rust_path: crate::strategy::advice::AdviceMetric,
        version: 1, direction: crate::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Native execution only; excludes queue, preparation, assessment and publication.
    NativeExecutionNanoseconds => 1;
    /// Fresh original physical feasibility divided by declared tolerance.
    OriginalNormalizedResidual => 2;
}
}
crate::projection_enum! {
/// Distinct outcomes are retained, including competing interruptions.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AdviceOutcome {
    projection: { name: "NumericalAdviceOutcome", rust_path: crate::strategy::advice::AdviceOutcome,
        version: 1, direction: crate::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Completed original-qualified observation with a measured metric.
    Complete => 1;
    /// Finite enclosing allowance exhausted.
    BudgetExhausted => 2;
    /// Local governing policy ended the attempt.
    PolicyPreempted => 3;
    /// External cancellation.
    Cancelled => 4;
    /// Infrastructure interrupted observation.
    InfrastructureInterrupted => 5;
    /// Terminal numerical failure.
    NumericalFailure => 6;
    /// Observation or original assessment is incomplete.
    Partial => 7;
}
}
crate::projection_struct! {
/// One actual observation in a declared matched exposure.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdviceObservation {
    projection: { name: "NumericalAdviceObservation", rust_path: crate::strategy::advice::AdviceObservation,
        version: 1, direction: crate::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Exact admitted comparison premises.
    pub cohort: AdviceCohort => { tag: 1, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// Declared matched exposure identity.
    pub matched: ContentHash => { tag: 2, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// Exact immutable candidate identity.
    pub candidate: ContentHash => { tag: 3, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// Original execution identity.
    pub attempt: ContentHash => { tag: 4, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// Actual observation disposition.
    pub outcome: AdviceOutcome => { tag: 5, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// Missing is unknown, never zero. Censored values cannot rank candidates.
    pub value: Option<f64> => { tag: 6, direction: crate::boundary_projection::Direction::Both,
        rules: [crate::boundary_projection::LocalRule::Float64Domain(crate::boundary_projection::Float64Domain::NonnegativeFinite)] };
}
}
crate::projection_enum! {
/// Descriptive support, not a causal estimator or a speed claim.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AdviceSupport {
    projection: { name: "NumericalAdviceSupport", rust_path: crate::strategy::advice::AdviceSupport,
        version: 1, direction: crate::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Every declared candidate has a complete metric for every matched exposure.
    CompleteMatched {
        /// Number of complete matched exposures.
        pairs: usize => { tag: 1, direction: crate::boundary_projection::Direction::Both, rules: [] };
    } => 1;
    /// Incomplete support preserves the original order.
    BaselineTie => 2;
}
}
crate::projection_struct! {
/// Full immutable body retained with effective settings, including all excluded outcomes.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FrozenAdvice {
    projection: { name: "NumericalFrozenAdvice", rust_path: crate::strategy::advice::FrozenAdvice,
        version: 1, direction: crate::boundary_projection::Direction::Both,
        reserved_tags: [], reserved_names: [] };
    /// Exact admitted comparison premises.
    pub cohort: AdviceCohort => { tag: 1, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// One declared descriptive metric.
    pub metric: AdviceMetric => { tag: 2, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// Complete original candidate order.
    pub baseline: Vec<ContentHash> => { tag: 3, direction: crate::boundary_projection::Direction::Both,
        rules: [crate::boundary_projection::LocalRule::Length { min: 1, max: Self::MAX_CANDIDATES as u64 }] };
    /// Actual frozen candidate order.
    pub ordered: Vec<ContentHash> => { tag: 4, direction: crate::boundary_projection::Direction::Both,
        rules: [crate::boundary_projection::LocalRule::Length { min: 1, max: Self::MAX_CANDIDATES as u64 }] };
    /// Actual retained observations, including exclusions.
    pub observations: Vec<AdviceObservation> => { tag: 5, direction: crate::boundary_projection::Direction::Both,
        rules: [crate::boundary_projection::LocalRule::Length { min: 0, max: Self::MAX_OBSERVATIONS as u64 }] };
    /// Support established by the retained comparison.
    pub support: AdviceSupport => { tag: 6, direction: crate::boundary_projection::Direction::Both, rules: [] };
    /// Original allowances for applicability checking; advice never widens them.
    pub limits: WorkLimits => { tag: 7, direction: crate::boundary_projection::Direction::Both, rules: [] };
}
}
impl FrozenAdvice {
    /// Finite candidate inventory capacity.
    pub const MAX_CANDIDATES: usize = 64;
    /// Finite retained comparison observation capacity.
    pub const MAX_OBSERVATIONS: usize = 4096;

    /// Freeze a complete descriptive comparison or preserve the full baseline tie.
    pub fn freeze(
        cohort: AdviceCohort,
        metric: AdviceMetric,
        baseline: Vec<ContentHash>,
        observations: Vec<AdviceObservation>,
        limits: WorkLimits,
    ) -> Result<Self, ModelError> {
        let mut frozen = Self {
            cohort,
            metric,
            ordered: baseline.clone(),
            baseline,
            observations,
            support: AdviceSupport::BaselineTie,
            limits,
        };
        let (ordered, support) = frozen.comparison()?;
        frozen.ordered = ordered;
        frozen.support = support;
        Ok(frozen)
    }
    /// Re-admit supplied bodies rather than trusting a caller's asserted order/support.
    pub fn validate(&self) -> Result<(), ModelError> {
        let (ordered, support) = self.comparison()?;
        if ordered != self.ordered || support != self.support {
            return Err(malformed(
                "frozen advice differs from its retained matched observations",
            ));
        }
        Ok(())
    }
    /// Exact applicability. The owner separately checks equivalent permitted Auto choices.
    pub fn applies(
        &self,
        cohort: &AdviceCohort,
        baseline: &[ContentHash],
        limits: WorkLimits,
    ) -> bool {
        self.cohort == *cohort
            && self.baseline == baseline
            && self.limits == limits
            && self.validate().is_ok()
    }
    /// Rank is descriptive only; callers must first establish applicability/permissions.
    pub fn rank(&self, candidate: ContentHash) -> Option<usize> {
        self.ordered.iter().position(|id| *id == candidate)
    }
    fn comparison(&self) -> Result<(Vec<ContentHash>, AdviceSupport), ModelError> {
        crate::boundary_projection::ValidateProjection::validate_projection(self)
            .map_err(|error| malformed(&error.to_string()))?;
        let candidates: BTreeSet<_> = self.baseline.iter().copied().collect();
        if candidates.len() != self.baseline.len() {
            return Err(malformed(
                "advice requires bounded distinct candidates and finite nonnegative observed metrics",
            ));
        }
        let mut matches: BTreeMap<ContentHash, BTreeMap<ContentHash, f64>> = BTreeMap::new();
        let mut attempts = BTreeSet::new();
        for observation in &self.observations {
            // Any unmatched, partial or censored exposure retains the declared tie.
            if observation.cohort != self.cohort
                || !candidates.contains(&observation.candidate)
                || observation.outcome != AdviceOutcome::Complete
                || observation.value.is_none()
                || !attempts.insert(observation.attempt)
            {
                return Ok((self.baseline.clone(), AdviceSupport::BaselineTie));
            }
            if let Some(value) = observation.value
                && matches
                    .entry(observation.matched)
                    .or_default()
                    .insert(observation.candidate, value)
                    .is_some()
            {
                return Ok((self.baseline.clone(), AdviceSupport::BaselineTie));
            }
        }
        if matches.is_empty() || matches.values().any(|row| row.len() != candidates.len()) {
            return Ok((self.baseline.clone(), AdviceSupport::BaselineTie));
        }
        let count = matches.len() as f64;
        let mut scores = BTreeMap::new();
        for candidate in &self.baseline {
            let score: f64 = matches
                .values()
                .filter_map(|row| row.get(candidate))
                .map(|v| v / count)
                .sum();
            if !score.is_finite() {
                return Ok((self.baseline.clone(), AdviceSupport::BaselineTie));
            }
            scores.insert(*candidate, score);
        }
        let mut ordered = self.baseline.clone();
        ordered.sort_by(|a, b| scores[a].total_cmp(&scores[b]));
        Ok((
            ordered,
            AdviceSupport::CompleteMatched {
                pairs: matches.len(),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(value: u8) -> ContentHash {
        pse_ids::encoding_checksum(&[value]).content_hash()
    }
    fn cohort() -> AdviceCohort {
        AdviceCohort {
            original: id(0),
            profiles: id(1),
            branch_start: id(2),
            accuracy: id(3),
            exposure: id(4),
        }
    }
    fn frozen(outcome: AdviceOutcome, complete: bool) -> FrozenAdvice {
        let observations = (0..if complete { 2 } else { 1 })
            .map(|i| AdviceObservation {
                cohort: cohort(),
                matched: id(5),
                candidate: id(i + 10),
                attempt: id(i + 20),
                outcome,
                value: Some(if i == 0 { 10.0 } else { 1.0 }),
            })
            .collect();
        FrozenAdvice::freeze(
            cohort(),
            AdviceMetric::NativeExecutionNanoseconds,
            vec![id(10), id(11)],
            observations,
            WorkLimits {
                attempts: 2,
                evaluations: None,
                iterations: None,
                factorizations: None,
                proof_steps: None,
            },
        )
        .unwrap()
    }
    #[test]
    fn complete_matched_observations_rank_and_materialize_full_body() {
        let advice = frozen(AdviceOutcome::Complete, true);
        assert_eq!(advice.ordered, [id(11), id(10)]);
        let restored: FrozenAdvice =
            serde_json::from_slice(&serde_json::to_vec(&advice).unwrap()).unwrap();
        assert_eq!(advice, restored);
        assert!(restored.applies(&cohort(), &advice.baseline, advice.limits));
    }
    #[test]
    fn censored_unmatched_missing_and_foreign_observations_retain_baseline() {
        for outcome in [
            AdviceOutcome::BudgetExhausted,
            AdviceOutcome::PolicyPreempted,
            AdviceOutcome::Cancelled,
            AdviceOutcome::InfrastructureInterrupted,
            AdviceOutcome::NumericalFailure,
            AdviceOutcome::Partial,
        ] {
            let advice = frozen(outcome, true);
            assert_eq!(advice.ordered, advice.baseline);
        }
        assert_eq!(
            frozen(AdviceOutcome::Complete, false).support,
            AdviceSupport::BaselineTie
        );
        let mut advice = frozen(AdviceOutcome::Complete, true);
        advice.observations[0].value = None;
        assert_eq!(advice.comparison().unwrap().1, AdviceSupport::BaselineTie);
        advice.observations[0].cohort.original = id(99);
        assert_eq!(advice.comparison().unwrap().1, AdviceSupport::BaselineTie);
    }
    #[test]
    fn forged_orders_and_changed_allowances_cannot_apply() {
        let mut advice = frozen(AdviceOutcome::Complete, true);
        advice.ordered.reverse();
        assert!(advice.validate().is_err());
        assert!(!advice.applies(&cohort(), &advice.baseline, advice.limits));
    }

    #[test]
    fn frozen_advice_projection_retains_excluded_observations_and_rechecks_claims() {
        use crate::boundary_projection::{
            ProjectionGraph, ProjectionRecord, ValidateProjection as _,
        };
        let original = frozen(AdviceOutcome::InfrastructureInterrupted, true);
        let bytes = serde_json::to_vec(&original).unwrap();
        let decoded =
            FrozenAdvice::from_projection_fields(original.into_projection_fields()).unwrap();
        decoded.validate_projection().unwrap();
        decoded.validate().unwrap();
        assert_eq!(serde_json::to_vec(&decoded).unwrap(), bytes);
        assert!(
            decoded
                .observations
                .iter()
                .all(|observation| observation.outcome == AdviceOutcome::InfrastructureInterrupted)
        );
        assert_eq!(decoded.support, AdviceSupport::BaselineTie);
        let mut forged = decoded;
        forged.support = AdviceSupport::CompleteMatched { pairs: 1 };
        let forged = FrozenAdvice::from_projection_fields(forged.into_projection_fields()).unwrap();
        assert!(
            forged.validate().is_err(),
            "decoding asserted support grants no comparison evidence"
        );
        let mut graph = ProjectionGraph::default();
        graph
            .register::<super::super::CompositionRequest>()
            .unwrap();
        for name in [
            "NumericalConnectedPath",
            "NumericalWorkLimits",
            "NumericalFrozenAdvice",
            "NumericalAdviceCohort",
            "NumericalAdviceObservation",
            "NumericalAdviceSupport",
        ] {
            assert!(
                graph.definitions().contains_key(name),
                "missing actual owner {name}"
            );
        }
    }

    #[test]
    fn composition_projection_preserves_unknown_and_explicit_zero_work_counters() {
        use crate::boundary_projection::{ProjectionRecord, ValidateProjection as _};
        let original = super::super::CompositionRequest {
            limits: Some(WorkLimits {
                attempts: 1,
                evaluations: Some(0),
                iterations: None,
                factorizations: Some(0),
                proof_steps: None,
            }),
            ..Default::default()
        };
        let bytes = serde_json::to_vec(&original).unwrap();
        let mut decoded = super::super::CompositionRequest::from_projection_fields(
            original.into_projection_fields(),
        )
        .unwrap();
        decoded.validate_projection().unwrap();
        decoded.validate().unwrap();
        assert_eq!(serde_json::to_vec(&decoded).unwrap(), bytes);
        decoded.limits.as_mut().unwrap().attempts = 0;
        assert!(decoded.validate_projection().is_err());
        assert!(decoded.validate().is_err());
    }
}
