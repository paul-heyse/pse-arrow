// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Frozen proposal forks, per-output statistics and sealed original boundaries.
use super::super::{PreparationServices, WorkflowError, contract};
use super::admission::LawCoordinate;
use super::*;
use pse_math::integration::{
    ContinuousGrid, DiscreteGrid, Grid, MonteCarloRng, StatisticsAccumulator,
};
use pse_model::{document::Version, generated::identities::StudyId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct OutputState {
    statistics: StatisticsAccumulator<f64>,
    available: usize,
    unavailable: Vec<u64>,
}

/// Private source-qualified estimator state at a completed original child barrier.
/// This is published only through protected original result custody, never arbitrary wire JSON.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BoundaryState {
    version: Version<1>,
    request: Vec<u8>,
    supplier: String,
    parent: StudyId,
    next_iteration: usize,
    issued: usize,
    children: Vec<StudyId>,
    rng: Vec<[u8; 32]>,
    grid: Option<Grid<f64>>,
    outputs: Vec<OutputState>,
    failure: StatisticsAccumulator<f64>,
    failure_available: usize,
    stopped: bool,
    #[serde(skip)]
    _owner: Option<std::sync::Arc<pse_columnar::AllocationLease>>,
}
/// Immutable original barrier state; clones retain the same charged mathematical backing.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectationCheckpoint {
    state: std::sync::Arc<BoundaryState>,
}
impl ExpectationCheckpoint {
    /// Decode only the retained original barrier after admitting its source-derived native backing.
    /// The caller obtains `children` from the protected original boundary ordinal.
    pub(super) fn decode_original(
        admitted: &AdmittedExpectation,
        children: usize,
        bytes: &[u8],
    ) -> Result<Self, WorkflowError> {
        let owner = admitted.services.math().reserve(
            "expectation:decoded-original-boundary",
            backing::state(admitted, children)?,
        )?;
        let mut checkpoint: Self =
            serde_json::from_slice(bytes).map_err(|error| contract(error.to_string()))?;
        let state = std::sync::Arc::get_mut(&mut checkpoint.state)
            .ok_or_else(|| contract("expectation decoded boundary unexpectedly shared"))?;
        state._owner = Some(owner);
        Ok(checkpoint)
    }
    /// Exact original parent whose protected result carries this state.
    pub fn parent(&self) -> StudyId {
        self.state.parent
    }
    /// Completed original children, in sealed iteration order.
    pub fn children(&self) -> &[StudyId] {
        &self.state.children
    }
    /// Next iteration admitted only after this boundary is retained.
    pub fn next_iteration(&self) -> usize {
        self.state.next_iteration
    }
}

/// One original estimator owner; native execution persists its completed boundary.
#[derive(Clone, Debug)]
pub struct ExpectationParent {
    admitted: AdmittedExpectation,
    state: std::sync::Arc<BoundaryState>,
    pending: Option<ExpectationIteration>,
    interrupted: bool,
    checkpoint_available: bool,
}

fn new_grid(admitted: &AdmittedExpectation) -> Result<Option<Grid<f64>>, WorkflowError> {
    let ExpectationEstimator::Adaptive(settings) = &admitted.request.estimator else {
        return Ok(None);
    };
    let continuous = admitted
        .coordinates
        .iter()
        .filter(|c| matches!(c, LawCoordinate::Uniform { .. }))
        .count();
    let category_counts = admitted
        .coordinates
        .iter()
        .filter_map(|coordinate| match coordinate {
            LawCoordinate::Categorical { categories, .. } => {
                Some(categories.iter().filter(|(_, _, p)| *p > 0.).count())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut nodes = 1_usize;
    let mut level = 1_usize;
    let mut weight_bound = 1. / settings.uniform_mixture;
    for count in &category_counts {
        level = level
            .checked_mul(*count)
            .ok_or_else(|| contract("expectation nested proposal extent"))?;
        nodes = nodes
            .checked_add(level)
            .ok_or_else(|| contract("expectation nested proposal extent"))?;
        weight_bound *= *count as f64 * settings.maximum_probability_ratio;
    }
    if nodes > super::super::MAXIMUM_STUDY_POINTS || !weight_bound.is_finite() {
        return Err(contract(
            "expectation finite nested proposal or weight extent",
        ));
    }
    fn construct(
        counts: &[usize],
        continuous: usize,
        settings: &ExpectationAdaptiveSettings,
    ) -> Result<Option<Grid<f64>>, WorkflowError> {
        if let Some((count, remaining)) = counts.split_first() {
            let mut bins = Vec::with_capacity(*count);
            for _ in 0..*count {
                bins.push(construct(remaining, continuous, settings)?);
            }
            return DiscreteGrid::new(bins, settings.maximum_probability_ratio, false)
                .map(Grid::Discrete)
                .map(Some)
                .map_err(contract);
        }
        if continuous == 0 {
            return Ok(None);
        }
        ContinuousGrid::new_with_min_probability_density(
            continuous,
            settings.bins,
            settings.minimum_training_samples,
            settings.bin_evolution.clone(),
            false,
            settings.uniform_mixture,
        )
        .map(Grid::Continuous)
        .map(Some)
        .map_err(contract)
    }
    construct(&category_counts, continuous, settings)
}

impl ExpectationParent {
    /// Start an original parent with explicit qualified supplier realization identity.
    pub fn new(
        admitted: AdmittedExpectation,
        parent: StudyId,
        supplier: String,
    ) -> Result<Self, WorkflowError> {
        if supplier.is_empty() {
            return Err(contract("expectation supplier realization is required"));
        }
        let owner = admitted
            .services
            .math()
            .reserve("expectation:estimator-state", backing::state(&admitted, 0)?)?;
        let grid = new_grid(&admitted)?;
        let rng = (0..admitted.request.controls.lanes)
            .map(|stream| MonteCarloRng::new(admitted.request.controls.seed, stream).export())
            .collect();
        let state = BoundaryState {
            version: Version,
            request: serde_json::to_vec(&admitted.request)
                .map_err(|error| contract(error.to_string()))?,
            supplier,
            parent,
            next_iteration: 0,
            issued: 0,
            children: vec![],
            rng,
            grid,
            outputs: admitted
                .outputs
                .iter()
                .map(|_| OutputState {
                    statistics: StatisticsAccumulator::new(),
                    available: 0,
                    unavailable: vec![],
                })
                .collect(),
            failure: StatisticsAccumulator::new(),
            failure_available: 0,
            stopped: false,
            _owner: Some(owner),
        };
        Ok(Self {
            admitted,
            state: std::sync::Arc::new(state),
            pending: None,
            interrupted: false,
            checkpoint_available: true,
        })
    }
    /// Restore only an original protected sealed boundary under exactly the same scientific source.
    pub fn restore(
        admitted: AdmittedExpectation,
        original_parent: StudyId,
        supplier: &str,
        checkpoint: ExpectationCheckpoint,
    ) -> Result<Self, WorkflowError> {
        let owner = admitted.services.math().reserve(
            "expectation:restored-estimator-state",
            backing::state(&admitted, checkpoint.state.children.len())?,
        )?;
        let mut checkpoint = checkpoint.state.as_ref().clone();
        checkpoint._owner = Some(owner);
        let request =
            serde_json::to_vec(&admitted.request).map_err(|error| contract(error.to_string()))?;
        let expected_issued = checkpoint
            .next_iteration
            .checked_mul(admitted.request.controls.samples_per_iteration)
            .map(|issued| issued.min(admitted.request.controls.maximum_samples));
        if checkpoint.request != request
            || checkpoint.parent != original_parent
            || checkpoint.supplier != supplier
            || checkpoint.rng.len() != admitted.request.controls.lanes
            || checkpoint
                .rng
                .iter()
                .any(|state| state.iter().all(|byte| *byte == 0))
            || checkpoint.outputs.len() != admitted.outputs.len()
            || checkpoint.children.len() != checkpoint.next_iteration
            || checkpoint.next_iteration > admitted.request.controls.maximum_iterations
            || expected_issued != Some(checkpoint.issued)
            || checkpoint.issued > admitted.request.controls.maximum_samples
            || checkpoint.failure_available > checkpoint.issued
            || checkpoint.failure.processed_samples > checkpoint.failure_available
            || checkpoint.children.iter().collect::<BTreeSet<_>>().len()
                != checkpoint.children.len()
            || checkpoint.outputs.iter().any(|output| {
                output.available.checked_add(output.unavailable.len()) != Some(checkpoint.issued)
                    || output.statistics.processed_samples > output.available
                    || output
                        .unavailable
                        .iter()
                        .any(|ordinal| *ordinal >= checkpoint.issued as u64)
                    || output.unavailable.iter().collect::<BTreeSet<_>>().len()
                        != output.unavailable.len()
            })
        {
            return Err(contract(
                "expectation original checkpoint source, supplier or population differs",
            ));
        }
        if checkpoint.grid.is_some()
            != matches!(
                admitted.request.estimator,
                ExpectationEstimator::Adaptive(_)
            )
        {
            return Err(contract(
                "expectation checkpoint estimator realization differs",
            ));
        }
        Ok(Self {
            admitted,
            state: std::sync::Arc::new(checkpoint),
            pending: None,
            interrupted: false,
            checkpoint_available: true,
        })
    }
    /// Original immutable admitted request and physical owners.
    pub fn admitted(&self) -> &AdmittedExpectation {
        &self.admitted
    }
    /// Original frozen population, including pre-issuance interruption attribution.
    pub fn pending_iteration(&self) -> Option<&ExpectationIteration> {
        self.pending.as_ref()
    }
    /// Stop parent issuance between barriers without rewriting the prior committed checkpoint.
    /// Native custody records this terminal parent report separately from adaptation boundaries.
    pub fn interrupt(&mut self) -> ExpectationReport {
        self.interrupted = true;
        self.report()
    }
    /// A new finite child is allowed only after the prior child's barrier commits.
    pub fn has_next(&self) -> bool {
        !self.state.stopped
            && !self.interrupted
            && self.pending.is_none()
            && self.state.next_iteration < self.admitted.request.controls.maximum_iterations
            && self
                .admitted
                .request
                .controls
                .maximum_samples
                .saturating_sub(self.state.issued)
                >= 2
            && !self.precision_met()
    }
    fn precision_met(&self) -> bool {
        self.state.issued >= 2
            && self
                .admitted
                .outputs
                .iter()
                .zip(&self.state.outputs)
                .all(|(output, state)| {
                    let selected =
                        output.absolute_error.is_some() || output.request.relative_error.is_some();
                    selected
                        && state.unavailable.is_empty()
                        && state.statistics.processed_samples == state.available
                        && output
                            .absolute_error
                            .is_none_or(|target| state.statistics.err <= target)
                        && output.request.relative_error.is_none_or(|target| {
                            state.statistics.err <= target * state.statistics.avg.abs()
                        })
                })
    }
    /// Freeze and retain the exact next population before the original child is issued.
    /// Repeating the call reconciles the same pending child; it never consumes another stream.
    pub fn prepare_iteration(
        &mut self,
        services: &PreparationServices,
        child: StudyId,
    ) -> Result<ExpectationIteration, WorkflowError> {
        if let Some(pending) = &self.pending {
            if pending.child != child {
                return Err(contract("expectation original pending child differs"));
            }
            return Ok(pending.clone());
        }
        if child == self.state.parent || self.state.children.contains(&child) || !self.has_next() {
            return Err(contract(
                "expectation next original finite child is unavailable",
            ));
        }
        let count = self
            .admitted
            .request
            .controls
            .samples_per_iteration
            .min(self.admitted.request.controls.maximum_samples - self.state.issued);
        let bytes = backing::population(&self.admitted, count)?;
        let grids = backing::grid(&self.admitted)?
            .checked_mul(self.state.rng.len())
            .ok_or_else(|| contract("expectation grid fork extent"))?;
        let owner = services
            .math()
            .reserve("expectation:frozen-population", bytes)?;
        let _sampling_owner = services
            .math()
            .reserve("expectation:sampling-grid-forks", grids)?;
        let mut rng = self
            .state
            .rng
            .iter()
            .copied()
            .map(MonteCarloRng::import)
            .collect::<Vec<_>>();
        let mut grids = (0..rng.len())
            .map(|_| self.state.grid.as_ref().map(Grid::clone_without_samples))
            .collect::<Vec<_>>();
        let mut samples = Vec::with_capacity(count);
        let lanes = rng.len();
        for index in 0..count {
            let lane = index % lanes;
            samples.push(population::draw(
                &self.admitted,
                &mut rng[lane],
                grids[lane].as_mut(),
                (self.state.issued + index) as u64,
                self.state.next_iteration,
                lane,
                index / lanes,
            )?);
        }
        let iteration = ExpectationIteration {
            parent: self.state.parent,
            child,
            iteration: self.state.next_iteration,
            samples: samples.into(),
            request: self.state.request.clone(),
            rng_after: rng.iter().map(MonteCarloRng::export).collect(),
            _owner: owner,
        };
        self.pending = Some(iteration.clone());
        self.checkpoint_available = false;
        Ok(iteration)
    }
    /// Replace only the pending population's physically admitted pushforward assignments.
    pub fn retain_transformed_iteration(
        &mut self,
        iteration: ExpectationIteration,
    ) -> Result<(), WorkflowError> {
        let pending = self
            .pending
            .as_ref()
            .ok_or_else(|| contract("expectation has no pending original child"))?;
        if pending.child != iteration.child
            || pending.request != iteration.request
            || pending.parent != iteration.parent
            || pending.iteration != iteration.iteration
            || pending.rng_after != iteration.rng_after
            || pending.samples.len() != iteration.samples.len()
            || pending
                .samples
                .iter()
                .zip(iteration.samples.iter())
                .any(|(a, b)| {
                    a.ordinal != b.ordinal
                        || a.iteration != b.iteration
                        || a.lane != b.lane
                        || a.lane_ordinal != b.lane_ordinal
                        || a.target_mass.to_bits() != b.target_mass.to_bits()
                        || a.proposal_weight.to_bits() != b.proposal_weight.to_bits()
                        || a.base != b.base
                        || a.categories != b.categories
                        || a.base_overlay != b.base_overlay
                })
        {
            return Err(contract(
                "expectation transformed population changed original sample identity",
            ));
        }
        for (original, retained) in pending.samples.iter().zip(iteration.samples.iter()) {
            let original =
                serde_json::to_vec(&(&original.base, &original.base_overlay, &original.proposal))
                    .map_err(|error| contract(error.to_string()))?;
            let retained =
                serde_json::to_vec(&(&retained.base, &retained.base_overlay, &retained.proposal))
                    .map_err(|error| contract(error.to_string()))?;
            if original != retained {
                return Err(contract(
                    "expectation retained sample differs from original frozen proposal",
                ));
            }
        }
        self.pending = Some(iteration);
        Ok(())
    }
    /// Prepare the exact next ordinary finite child, including its admitted joint transform.
    pub async fn prepare_child(
        &mut self,
        services: &super::super::ExecutionServices,
        child: StudyId,
        cancel: &crate::CancelSource,
    ) -> Result<(ExpectationIteration, super::super::LocalStudy), WorkflowError> {
        let mut iteration = self.prepare_iteration(services.preparation(), child)?;
        let local = iteration
            .admit_child(&self.admitted, services, cancel)
            .await?;
        self.retain_transformed_iteration(iteration.clone())?;
        Ok((iteration, local))
    }
    /// Merge one settled original child exactly once. The caller supplies its terminal inventory
    /// identity after joining every issued member and selecting actual physical outputs.
    pub fn commit_iteration(
        &mut self,
        original_child: StudyId,
        observations: &[MemberObservation],
    ) -> Result<ExpectationReport, WorkflowError> {
        let pending = self
            .pending
            .as_ref()
            .ok_or_else(|| contract("expectation child already merged or not prepared"))?;
        if pending.child != original_child
            || pending.iteration != self.state.next_iteration
            || observations.len() != pending.samples.len()
            || self.state.children.contains(&original_child)
        {
            return Err(contract(
                "expectation original child inventory or merge identity differs",
            ));
        }
        let mut ordinals = BTreeMap::new();
        for observation in observations {
            if observation.child != original_child
                || observation.values.len() != self.admitted.outputs.len()
                || ordinals.insert(observation.ordinal, observation).is_some()
                || observation
                    .values
                    .iter()
                    .flatten()
                    .any(|value| !value.is_finite())
            {
                return Err(contract(
                    "expectation physical observation identity, value or duplicate differs",
                ));
            }
        }
        if pending
            .samples
            .iter()
            .any(|sample| !ordinals.contains_key(&sample.ordinal))
        {
            return Err(contract("expectation original sample coverage differs"));
        }
        // Validate everything above before mutating the parent. Work on a clone so a
        // supplier refusal cannot leave a partially merged restart boundary.
        let owner = self.admitted.services.math().reserve(
            "expectation:barrier-state",
            backing::state(&self.admitted, self.state.children.len() + 1)?,
        )?;
        let _lane_owner = self.admitted.services.math().reserve(
            "expectation:lane-training-state",
            backing::lanes(&self.admitted)?,
        )?;
        let mut next = self.state.as_ref().clone();
        next._owner = Some(owner);
        let lanes = next.rng.len();
        let mut grids = (0..lanes)
            .map(|_| next.grid.as_ref().map(Grid::clone_without_samples))
            .collect::<Vec<_>>();
        let mut outputs = (0..lanes)
            .map(|_| {
                self.admitted
                    .outputs
                    .iter()
                    .map(|_| StatisticsAccumulator::new())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut failure = (0..lanes)
            .map(|_| StatisticsAccumulator::new())
            .collect::<Vec<_>>();
        let mut operational = false;
        let mut trainable = true;
        for sample in pending.samples.iter() {
            let observation = ordinals
                .get(&sample.ordinal)
                .ok_or_else(|| contract("expectation original observation absent"))?;
            operational |= observation.operational_incomplete;
            let correction = if matches!(
                self.admitted.request.estimator,
                ExpectationEstimator::Adaptive(_)
            ) {
                sample.target_mass * sample.proposal_weight
            } else {
                1.
            };
            if !correction.is_finite() || correction <= 0. {
                return Err(contract(
                    "expectation corrected target/proposal weight is nonfinite",
                ));
            }
            let mut training = 0_f64;
            let mut sample_trainable = !observation.operational_incomplete;
            for (index, output) in self.admitted.outputs.iter().enumerate() {
                if let Some(canonical) =
                    observation.values[index].filter(|_| !observation.operational_incomplete)
                {
                    let centered = output.represented(canonical)? - output.reference;
                    let contribution = centered * correction;
                    if !contribution.is_finite() {
                        return Err(contract("expectation physical contribution is nonfinite"));
                    }
                    outputs[sample.lane][index].add_sample(contribution, sample.proposal.as_ref());
                    next.outputs[index].available += 1;
                    if output.request.train {
                        training = training.max((centered / output.scale).abs());
                    }
                } else {
                    next.outputs[index].unavailable.push(sample.ordinal);
                    if output.request.train {
                        trainable = false;
                        sample_trainable = false;
                    }
                }
            }
            if let Some(indicator) = observation
                .scientific_failure
                .filter(|_| !observation.operational_incomplete)
            {
                failure[sample.lane].add_sample(
                    if indicator { correction } else { 0. },
                    sample.proposal.as_ref(),
                );
                next.failure_available += 1;
            }
            if let (Some(grid), Some(proposal), true) =
                (&mut grids[sample.lane], &sample.proposal, sample_trainable)
            {
                let target = training * sample.target_mass;
                if !target.is_finite() || !(target * sample.proposal_weight).is_finite() {
                    return Err(contract(
                        "expectation dimensionless training target or proposal-corrected value is nonfinite",
                    ));
                }
                grid.add_training_sample(proposal, target)
                    .map_err(contract)?;
            }
        }
        // Lane order is stable, while original occurrence order remains in attribution.
        for lane in 0..lanes {
            for (state, samples) in next.outputs.iter_mut().zip(&mut outputs[lane]) {
                state.statistics.merge_samples(samples);
            }
            next.failure.merge_samples(&mut failure[lane]);
        }
        for state in &mut next.outputs {
            if state.available - state.statistics.processed_samples >= 2 {
                state.statistics.update_iter(false);
            }
        }
        if next.failure_available - next.failure.processed_samples >= 2 {
            next.failure.update_iter(false);
        }
        if !operational && trainable {
            if let Some(grid) = &mut next.grid {
                for lane in grids.iter().flatten() {
                    grid.is_mergeable(lane).map_err(contract)?;
                    grid.merge(lane).map_err(contract)?;
                }
                if let ExpectationEstimator::Adaptive(settings) = &self.admitted.request.estimator {
                    grid.update(
                        settings.discrete_learning_rate,
                        settings.continuous_learning_rate,
                    );
                }
            }
        }
        next.issued += pending.samples.len();
        next.children.push(original_child);
        next.next_iteration += 1;
        next.rng.clone_from(&pending.rng_after);
        next.stopped = operational
            || next
                .outputs
                .iter()
                .any(|output| !output.unavailable.is_empty());
        self.checkpoint_available = !operational && (trainable || next.grid.is_none());
        self.interrupted = operational;
        self.state = std::sync::Arc::new(next);
        self.pending = None;
        Ok(self.report())
    }
    /// A completed original barrier, with no unconsumed lane deltas. Native custody seals it once.
    pub fn checkpoint(&self) -> Result<ExpectationCheckpoint, WorkflowError> {
        if self.pending.is_some() || !self.checkpoint_available {
            return Err(contract(
                "expectation has no completed merged checkpoint boundary",
            ));
        }
        Ok(ExpectationCheckpoint {
            state: self.state.clone(),
        })
    }
    /// Original scientific interpretation and actual precision/availability at this boundary.
    pub fn report(&self) -> ExpectationReport {
        let outputs = self
            .admitted
            .outputs
            .iter()
            .zip(&self.state.outputs)
            .map(|(output, state)| {
                // An unavailable required member does not define an ordinary
                // expectation over the successful subset. Original contributions
                // remain in the sample product and native statistics, with counts.
                let sufficient = state.available == self.state.issued
                    && state.unavailable.is_empty()
                    && state.statistics.processed_samples >= 2
                    && state.statistics.avg.is_finite()
                    && state.statistics.err.is_finite();
                let estimate = sufficient
                    .then_some(output.reference + state.statistics.avg)
                    .filter(|value| value.is_finite());
                let standard_error = sufficient
                    .then_some(state.statistics.err)
                    .filter(|value| value.is_finite());
                let complete = estimate.is_some()
                    && standard_error.is_some()
                    && state.unavailable.is_empty()
                    && state.available == self.state.issued
                    && !self.interrupted
                    && self.pending.is_none();
                let precision_met = complete
                    && (output.absolute_error.is_some() || output.request.relative_error.is_some())
                    && standard_error.is_some_and(|error| {
                        output.absolute_error.is_none_or(|target| error <= target)
                            && output
                                .request
                                .relative_error
                                .is_none_or(|target| error <= target * state.statistics.avg.abs())
                    });
                OutputEstimate {
                    output: output.request.output,
                    quantity: output.request.quantity,
                    unit: output.request.unit,
                    reference: output.reference,
                    issued: self.state.issued,
                    available: state.available,
                    unavailable: state.unavailable.clone(),
                    estimate,
                    standard_error,
                    complete,
                    precision_met,
                }
            })
            .collect();
        let failure_complete = self.admitted.request.failure_probability
            && self.state.failure_available == self.state.issued
            && self.state.failure.processed_samples >= 2
            && self.state.failure.avg.is_finite()
            && self.state.failure.err.is_finite()
            && !self.interrupted
            && self.pending.is_none();
        let limit_reached = self.state.next_iteration
            >= self.admitted.request.controls.maximum_iterations
            || self
                .admitted
                .request
                .controls
                .maximum_samples
                .saturating_sub(self.state.issued)
                < 2;
        let admitted_categories = self
            .admitted
            .category_masses()
            .into_iter()
            .flat_map(|(member, categories)| {
                categories
                    .into_iter()
                    .map(move |(category, probability)| ExpectationCategoryMass {
                        member,
                        category,
                        probability,
                    })
            })
            .collect();
        ExpectationReport {
            request: self.admitted.request.as_ref().clone(),
            completed_iterations: self.state.next_iteration,
            issued: self.state.issued,
            outputs,
            failure_probability: failure_complete.then_some(FailureProbability {
                estimate: self.state.failure.avg,
                standard_error: self.state.failure.err,
            }),
            failure_probability_complete: failure_complete,
            interrupted: self.interrupted || self.pending.is_some(),
            limit_reached: limit_reached && !self.precision_met(),
            admitted_categories,
        }
    }
}
