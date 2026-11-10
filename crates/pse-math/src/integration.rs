// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native integration sampling/statistics contracts consumed by scientific workflow policy.
//! The maintained supplier owns grids, sampling, random streams and statistical reduction;
//! runtime admission owns target measures, physical coordinates and original populations.
pub use symbolica::domains::float::Constructible;
pub use symbolica::numerical_integration::{
    Bin, ContinuousDimension, ContinuousGrid, DiscreteGrid, Grid, MonteCarloRng, Sample,
    StatisticsAccumulator,
};
