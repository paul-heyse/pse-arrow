// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit probability laws and estimators over original finite study populations.
mod admission;
mod backing;
mod declarations;
mod estimator;
mod execution;
mod population;
mod operation;
pub use operation::{PreparedExpectationOperation, ExpectationProduct};
mod products;
mod reading;
mod retained;
pub use retained::RetainedExpectationReport;

pub use admission::{AdmittedExpectation, AdmittedOutput};
pub use declarations::*;
pub use estimator::{ExpectationCheckpoint, ExpectationParent};
pub use population::{ExpectationIteration, ExpectationSample, MemberObservation};

#[cfg(test)]
mod tests;
