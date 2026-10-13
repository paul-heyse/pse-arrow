// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Frozen descriptive comparisons. They grant neither eligibility nor numerical tuning.
use super::WorkLimits;
use crate::{ModelError, malformed};
use pse_ids::ContentHash;
use std::collections::{BTreeMap, BTreeSet};

/// Exact comparison premises chosen before observing outcomes.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdviceCohort {
    /// Original problem and physical/provider interpretation.
    pub original: ContentHash,
    /// Equivalent permitted profile family; explicit settings remain constraints.
    pub profiles: ContentHash,
    /// Original branch permission and actual start policy.
    pub branch_start: ContentHash,
    /// Original accuracy and assurance requirements.
    pub accuracy: ContentHash,
    /// Declared exposure/allocation design, including placement and interval meaning.
    pub exposure: ContentHash,
}
/// Lower is better, within the exact cohort and one metric only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdviceMetric {
    /// Native execution only; excludes queue, preparation, assessment and publication.
    NativeExecutionNanoseconds,
    /// Fresh original physical feasibility divided by declared tolerance.
    OriginalNormalizedResidual,
}
/// Distinct outcomes are retained, including competing interruptions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdviceOutcome {
    /// Completed original-qualified observation with a measured metric.
    Complete,
    /// Finite enclosing allowance exhausted.
    BudgetExhausted,
    /// Local governing policy ended the attempt.
    PolicyPreempted,
    /// External cancellation.
    Cancelled,
    /// Infrastructure interrupted observation.
    InfrastructureInterrupted,
    /// Terminal numerical failure.
    NumericalFailure,
    /// Observation or original assessment is incomplete.
    Partial,
}
/// One actual observation in a declared matched exposure.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdviceObservation {
    pub cohort: AdviceCohort,
    pub matched: ContentHash,
    pub candidate: ContentHash,
    pub attempt: ContentHash,
    pub outcome: AdviceOutcome,
    /// Missing is unknown, never zero. Censored values cannot rank candidates.
    pub value: Option<f64>,
}
/// Descriptive support, not a causal estimator or a speed claim.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdviceSupport {
    CompleteMatched { pairs: usize },
    BaselineTie,
}
/// Full immutable body retained with effective settings, including all excluded outcomes.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FrozenAdvice {
    pub cohort: AdviceCohort,
    pub metric: AdviceMetric,
    pub baseline: Vec<ContentHash>,
    pub ordered: Vec<ContentHash>,
    pub observations: Vec<AdviceObservation>,
    pub support: AdviceSupport,
    /// Original allowances for applicability checking; advice never widens them.
    pub limits: WorkLimits,
}
impl FrozenAdvice {
    pub const MAX_CANDIDATES: usize = 64;
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
            cohort, metric, ordered: baseline.clone(), baseline, observations,
            support: AdviceSupport::BaselineTie, limits,
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
            return Err(malformed("frozen advice differs from its retained matched observations"));
        }
        Ok(())
    }
    /// Exact applicability. The owner separately checks equivalent permitted Auto choices.
    pub fn applies(&self, cohort: &AdviceCohort, baseline: &[ContentHash], limits: WorkLimits) -> bool {
        self.cohort == *cohort && self.baseline == baseline && self.limits == limits
            && self.validate().is_ok()
    }
    /// Rank is descriptive only; callers must first establish applicability/permissions.
    pub fn rank(&self, candidate: ContentHash) -> Option<usize> {
        self.ordered.iter().position(|id| *id == candidate)
    }
    fn comparison(&self) -> Result<(Vec<ContentHash>, AdviceSupport), ModelError> {
        self.limits.validate()?;
        let candidates: BTreeSet<_> = self.baseline.iter().copied().collect();
        if self.baseline.is_empty() || self.baseline.len() > Self::MAX_CANDIDATES
            || candidates.len() != self.baseline.len()
            || self.observations.len() > Self::MAX_OBSERVATIONS
            || self.observations.iter().any(|o| o.value.is_some_and(|v| !v.is_finite() || v < 0.0)) {
            return Err(malformed("advice requires bounded distinct candidates and finite nonnegative observed metrics"));
        }
        let mut matches: BTreeMap<ContentHash, BTreeMap<ContentHash, f64>> = BTreeMap::new();
        let mut attempts = BTreeSet::new();
        for observation in &self.observations {
            // Any unmatched, partial or censored exposure retains the declared tie.
            if observation.cohort != self.cohort || !candidates.contains(&observation.candidate)
                || observation.outcome != AdviceOutcome::Complete || observation.value.is_none()
                || !attempts.insert(observation.attempt) {
                return Ok((self.baseline.clone(), AdviceSupport::BaselineTie));
            }
            if let Some(value) = observation.value
                && matches.entry(observation.matched).or_default()
                    .insert(observation.candidate, value).is_some() {
                return Ok((self.baseline.clone(), AdviceSupport::BaselineTie));
            }
        }
        if matches.is_empty() || matches.values().any(|row| row.len() != candidates.len()) {
            return Ok((self.baseline.clone(), AdviceSupport::BaselineTie));
        }
        let count = matches.len() as f64;
        let mut scores = BTreeMap::new();
        for candidate in &self.baseline {
            let score: f64 = matches.values().filter_map(|row| row.get(candidate)).map(|v| v / count).sum();
            if !score.is_finite() {
                return Ok((self.baseline.clone(), AdviceSupport::BaselineTie));
            }
            scores.insert(*candidate, score);
        }
        let mut ordered = self.baseline.clone();
        ordered.sort_by(|a, b| scores[a].total_cmp(&scores[b]));
        Ok((ordered, AdviceSupport::CompleteMatched { pairs: matches.len() }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(value: u8) -> ContentHash { pse_ids::encoding_checksum(&[value]).content_hash() }
    fn cohort() -> AdviceCohort {
        AdviceCohort { original: id(0), profiles: id(1), branch_start: id(2), accuracy: id(3), exposure: id(4) }
    }
    fn frozen(outcome: AdviceOutcome, complete: bool) -> FrozenAdvice {
        let observations = (0..if complete { 2 } else { 1 }).map(|i| AdviceObservation {
            cohort: cohort(), matched: id(5), candidate: id(i + 10), attempt: id(i + 20),
            outcome, value: Some(if i == 0 { 10.0 } else { 1.0 }),
        }).collect();
        FrozenAdvice::freeze(cohort(), AdviceMetric::NativeExecutionNanoseconds,
            vec![id(10), id(11)], observations,
            WorkLimits { attempts: 2, evaluations: None, iterations: None, factorizations: None, proof_steps: None }).unwrap()
    }
    #[test]
    fn complete_matched_observations_rank_and_materialize_full_body() {
        let advice = frozen(AdviceOutcome::Complete, true);
        assert_eq!(advice.ordered, [id(11), id(10)]);
        let restored: FrozenAdvice = serde_json::from_slice(&serde_json::to_vec(&advice).unwrap()).unwrap();
        assert_eq!(advice, restored);
        assert!(restored.applies(&cohort(), &advice.baseline, advice.limits));
    }
    #[test]
    fn censored_unmatched_missing_and_foreign_observations_retain_baseline() {
        for outcome in [AdviceOutcome::BudgetExhausted, AdviceOutcome::PolicyPreempted,
            AdviceOutcome::Cancelled, AdviceOutcome::InfrastructureInterrupted,
            AdviceOutcome::NumericalFailure, AdviceOutcome::Partial] {
            let advice = frozen(outcome, true);
            assert_eq!(advice.ordered, advice.baseline);
        }
        assert_eq!(frozen(AdviceOutcome::Complete, false).support, AdviceSupport::BaselineTie);
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
}
