// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-derived admission for maintained Numerica's declared finite grid shape.
//! Snapshot copies retain their own owners; this does not constrain host execution placement.
use super::super::{WorkflowError, contract};
use super::admission::LawCoordinate;
use super::*;
use pse_math::integration::{
    Bin, ContinuousDimension, ContinuousGrid, DiscreteGrid, Grid, Sample, StatisticsAccumulator,
};

fn add(a: usize, b: usize) -> Result<usize, WorkflowError> {
    a.checked_add(b)
        .ok_or_else(|| contract("expectation backing extent overflow"))
}
fn mul(a: usize, b: usize) -> Result<usize, WorkflowError> {
    a.checked_mul(b)
        .ok_or_else(|| contract("expectation backing extent overflow"))
}

pub(super) fn sample(admitted: &AdmittedExpectation) -> Result<usize, WorkflowError> {
    // One nested box per category and a coordinate vector at the continuous leaf.
    // Two sides of a temporary/copy overlap and allocator headers are covered separately.
    add(
        mul(
            admitted.coordinates.len() + 1,
            size_of::<Sample<f64>>() + 32,
        )?,
        mul(admitted.coordinates.len(), size_of::<f64>())?,
    )
}

pub(super) fn population(
    admitted: &AdmittedExpectation,
    count: usize,
) -> Result<usize, WorkflowError> {
    let coordinates = admitted.coordinates.len();
    let realized = admitted
        .request
        .transform
        .as_ref()
        .map_or(coordinates, |transform| transform.outputs.len());
    let assignments = add(coordinates, realized)?;
    let assignment = size_of::<super::super::BindingAssignment>() + 64;
    let descriptor = super::super::study_variants::document_extent(&admitted.request, 16 << 20)?;
    // The source descriptor bounds all repeated target path strings. Native
    // proposal boxes and both base/realized assignment vectors are additional.
    let point = add(
        size_of::<ExpectationSample>() + 256,
        add(
            sample(admitted)?,
            add(
                mul(
                    coordinates,
                    size_of::<f64>() + size_of::<pse_ids::SemanticId>(),
                )?,
                add(mul(assignments, assignment)?, descriptor)?,
            )?,
        )?,
    )?;
    // Pushforward admission may detach one shared frozen population before replacing overlays.
    add(
        mul(mul(count, point)?, 2)?,
        add(descriptor, mul(admitted.request.controls.lanes, 32)?)?,
    )
}

pub(super) fn grid(admitted: &AdmittedExpectation) -> Result<usize, WorkflowError> {
    let ExpectationEstimator::Adaptive(settings) = &admitted.request.estimator else {
        return Ok(0);
    };
    let statistics = add(
        size_of::<StatisticsAccumulator<f64>>(),
        mul(2, sample(admitted)?)?,
    )?;
    let mut total = add(size_of::<Grid<f64>>(), statistics)?;
    let mut level = 1_usize;
    let mut nodes = 1_usize;
    for coordinate in admitted.coordinates.iter() {
        if let LawCoordinate::Categorical { categories, .. } = coordinate {
            let count = categories.iter().filter(|(_, _, p)| *p > 0.).count();
            let bins = mul(
                count,
                add(size_of::<Bin<f64>>() + 32, mul(2, sample(admitted)?)?)?,
            )?;
            total = add(
                total,
                mul(
                    level,
                    add(size_of::<DiscreteGrid<f64>>() + 32, add(bins, statistics)?)?,
                )?,
            )?;
            level = mul(level, count)?;
            nodes = add(nodes, level)?;
            if nodes > super::super::MAXIMUM_STUDY_POINTS {
                return Err(contract("expectation nested grid population extent"));
            }
        }
    }
    let dimensions = admitted
        .coordinates
        .iter()
        .filter(|coordinate| matches!(coordinate, LawCoordinate::Uniform { .. }))
        .count();
    if dimensions > 0 {
        let bins = settings
            .bin_evolution
            .as_ref()
            .and_then(|bins| bins.iter().copied().max())
            .unwrap_or(settings.bins)
            .max(settings.bins);
        // Inspected ContinuousDimension owns partitions, per-bin accumulators,
        // importances, counters and the declared bin evolution. Its per-bin
        // add_training_sample stores no Sample payload. The grid accumulator does.
        let per_bin =
            size_of::<StatisticsAccumulator<f64>>() + size_of::<f64>() + size_of::<usize>();
        let evolution = mul(
            settings.bin_evolution.as_ref().map_or(1, Vec::len),
            size_of::<usize>(),
        )?;
        let dimension = add(
            size_of::<ContinuousDimension<f64>>() + 160,
            add(
                mul(bins + 1, size_of::<f64>())?,
                add(mul(bins, per_bin)?, evolution)?,
            )?,
        )?;
        let continuous = add(
            size_of::<ContinuousGrid<f64>>() + 32,
            add(statistics, mul(dimensions, dimension)?)?,
        )?;
        total = add(total, mul(level, continuous)?)?;
    }
    // Construction/update may overlap old and new vectors, including bin evolution.
    mul(total, 2)
}

pub(super) fn state(
    admitted: &AdmittedExpectation,
    children: usize,
) -> Result<usize, WorkflowError> {
    // The parent stops after its first unavailable required output, so missing
    // ordinal storage is bounded by one finite child, not the whole sample limit.
    let output = add(
        size_of::<StatisticsAccumulator<f64>>() + 64,
        add(
            mul(2, sample(admitted)?)?,
            mul(
                admitted.request.controls.samples_per_iteration,
                size_of::<u64>(),
            )?,
        )?,
    )?;
    let outputs = mul(admitted.outputs.len() + 1, output)?;
    let children = mul(
        children,
        size_of::<pse_model::generated::identities::StudyId>(),
    )?;
    let request = super::super::study_variants::document_extent(&admitted.request, 16 << 20)?;
    add(
        grid(admitted)?,
        add(
            outputs,
            add(
                children,
                add(request, mul(admitted.request.controls.lanes, 64)?)?,
            )?,
        )?,
    )
}

pub(super) fn lanes(admitted: &AdmittedExpectation) -> Result<usize, WorkflowError> {
    let lanes = admitted.request.controls.lanes;
    let output = add(
        size_of::<StatisticsAccumulator<f64>>(),
        mul(2, sample(admitted)?)?,
    )?;
    let lane = add(grid(admitted)?, mul(admitted.outputs.len() + 1, output)?)?;
    mul(lanes, lane)
}
