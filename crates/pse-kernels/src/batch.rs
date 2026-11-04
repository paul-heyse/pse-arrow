// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Identified point groups, with frame integrity separate from scientific admission.
use super::*;

/// Exact invocation-local physical inputs and seeds. Raw nonfinite values remain representable.
#[derive(Clone, Debug)]
pub struct ProviderBatchPoint {
    /// Unique nonzero identity within the original group.
    pub id: SemanticId,
    /// Canonical ordered physical input values.
    pub inputs: Vec<f64>,
    /// Direction-major input seeds.
    pub directions: Vec<f64>,
    /// Cotangent-major selected-output weights.
    pub cotangents: Vec<f64>,
    /// Selected-output weights for a weighted second product.
    pub weights: Vec<f64>,
}
impl ProviderBatchPoint {
    /// A point without invocation-local derivative seeds.
    pub fn values(id: SemanticId, inputs: Vec<f64>) -> Self {
        Self {
            id,
            inputs,
            directions: vec![],
            cotangents: vec![],
            weights: vec![],
        }
    }
    /// Borrow invocation-local seeds; owner-sealed linearizations never cross groups.
    pub fn runtime(&self) -> ProviderRuntime<'_> {
        ProviderRuntime {
            directions: &self.directions,
            cotangents: &self.cotangents,
            weights: &self.weights,
            expected_linearization: None,
        }
    }
    pub(super) fn bytes(&self) -> Result<usize, ProviderError> {
        [
            self.inputs.len(),
            self.directions.len(),
            self.cotangents.len(),
            self.weights.len(),
        ]
        .into_iter()
        .try_fold(size_of::<Self>(), |n, cells| {
            cells
                .checked_mul(size_of::<f64>())
                .and_then(|bytes| n.checked_add(bytes))
                .ok_or(ProviderError::Limit("provider batch storage"))
        })
    }
}

/// Native group results in exactly the supplied point order. Uncertain work is never replayed.
pub type ProviderBatchResults = Vec<Result<ProviderValues, ProviderError>>;

/// Completion or refusal attributable to one original point; siblings remain independent.
#[derive(Debug)]
pub struct ProviderBatchOutcome {
    /// Original identity, preserved independently of native gathering order.
    pub id: SemanticId,
    /// Complete finite demanded product, or its original native refusal.
    pub result: Result<ProviderValues, ProviderError>,
}
