// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Validated selection evidence, separate from a numerical inner solve (ADR-0153).

use crate::{MathError, factorable::RootIsolationProgram};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::DerivativeOrder;
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

/// A finite closed interval in original coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProofInterval {
    /// Lower endpoint, rounded outward by the validating library.
    pub lower: f64,
    /// Upper endpoint, rounded outward by the validating library.
    pub upper: f64,
}
impl ProofInterval {
    /// Whether the interval is finite and nonempty.
    pub fn valid(self) -> bool {
        self.lower.is_finite() && self.upper.is_finite() && self.lower <= self.upper
    }
    /// Membership, including both endpoints.
    pub fn contains(self, value: f64) -> bool {
        self.valid() && value.is_finite() && self.lower <= value && value <= self.upper
    }
    /// Strict membership needed for an input neighborhood.
    pub fn interior_contains(self, value: f64) -> bool {
        self.contains(value) && self.lower < value && value < self.upper
    }
}

/// One original alternative in a minimum-score selection problem.
#[derive(Debug)]
pub struct SelectionAlternative<'a> {
    /// Authored regime identity, in the selector's original alternative order.
    pub id: SemanticId,
    /// Exact residual, eligibility, criterion and guard projection.
    pub program: &'a Arc<RootIsolationProgram>,
    /// Compiled original residual identity.
    pub residual_identity: ContentHash,
    /// Physical bounds resolved for this trial, distinct from eligibility.
    pub unknowns: &'a [super::Unknown],
}

/// A numerical winning proposal lent to validated global minimum selection.
#[derive(Debug)]
pub struct SelectionProofRequest<'a> {
    /// Authored selector identity; evidence never crosses workers or source bindings.
    pub selection: SemanticId,
    /// Complete original alternative union, including failed numerical rivals.
    pub alternatives: &'a [SelectionAlternative<'a>],
    /// Alternative containing the verified, eligible numerical proposal.
    pub winner: usize,
    /// Actual independent inputs in original order.
    pub parameters: &'a [f64],
    /// Original-space verified winning root proposal.
    pub candidate: &'a [f64],
    /// Required original residual and selector derivative neighborhood.
    pub order: DerivativeOrder,
    /// Remaining allowance of the same outer attempt.
    pub time_limit: Duration,
    /// Same outer cancellation owner.
    pub cancel: &'a Arc<AtomicBool>,
}

/// Exact source and physical domain covered by a selection certificate.
#[derive(Clone, Debug)]
pub struct SelectionScope {
    /// Original alternative identity.
    pub id: SemanticId,
    /// Exact immutable source, retained without a second hash declaration.
    pub program: Arc<RootIsolationProgram>,
    /// Original compiled residual identity.
    pub residual_identity: ContentHash,
    /// Original physical coordinate identities and bounds.
    pub unknowns: Vec<super::Unknown>,
}

/// A regular winning chart with complete exclusion of all competitive roots.
#[derive(Clone, Debug)]
pub struct SelectionChart {
    /// Authored selector whose entire alternative union was covered.
    pub selection: SemanticId,
    /// Exact alternative source and physical domains covered, in request order.
    pub alternatives: Vec<SelectionScope>,
    /// Winning alternative index.
    pub winner: usize,
    /// Exact interval-library implementation and rounding provenance.
    pub verifier_identity: ContentHash,
    /// Independent-input box, in request order.
    pub parameters: Vec<ProofInterval>,
    /// Winning root existence enclosure, in unknown order.
    pub existence: Vec<ProofInterval>,
    /// Winning root's local uniqueness box for every parameter in the parameter box.
    pub uniqueness: Vec<ProofInterval>,
    /// Highest original residual/guard order established on the winning chart.
    pub order: DerivativeOrder,
}
impl SelectionChart {
    /// Check transport and scope; the validating library supplies the mathematics.
    /// A worker may reuse this certificate only when this exact check still succeeds.
    pub fn validate(
        &self,
        request: &SelectionProofRequest<'_>,
        verifier: ContentHash,
    ) -> Result<(), MathError> {
        let Some(winner) = request.alternatives.get(request.winner) else {
            return Err(MathError::Contract(
                "nonlinear selection winning alternative".into(),
            ));
        };
        if self.selection != request.selection
            || self.winner != request.winner
            || self.verifier_identity != verifier
            || self.order < request.order
            || self.alternatives.len() != request.alternatives.len()
            || self
                .alternatives
                .iter()
                .zip(request.alternatives)
                .any(|(scope, alternative)| {
                    scope.id != alternative.id
                        || scope.residual_identity != alternative.residual_identity
                        || scope.program.as_ref() != alternative.program.as_ref()
                        || scope.unknowns.len() != alternative.unknowns.len()
                        || scope
                            .unknowns
                            .iter()
                            .zip(alternative.unknowns)
                            .any(|(a, b)| {
                                a.id != b.id
                                    || a.lower.to_bits() != b.lower.to_bits()
                                    || a.upper.to_bits() != b.upper.to_bits()
                            })
                })
            || self.parameters.len() != request.parameters.len()
            || self.existence.len() != winner.unknowns.len()
            || self.uniqueness.len() != winner.unknowns.len()
            || request.candidate.len() != winner.unknowns.len()
            || self
                .parameters
                .iter()
                .zip(request.parameters)
                .any(|(box_, value)| !box_.interior_contains(*value))
            || self
                .existence
                .iter()
                .zip(&self.uniqueness)
                .zip(winner.unknowns)
                .zip(request.candidate)
                .any(|(((existence, uniqueness), unknown), candidate)| {
                    !existence.valid()
                        || !uniqueness.valid()
                        || existence.lower <= uniqueness.lower
                        || existence.upper >= uniqueness.upper
                        || existence.lower <= unknown.lower
                        || existence.upper >= unknown.upper
                        || uniqueness.lower < unknown.lower
                        || uniqueness.upper > unknown.upper
                        || !uniqueness.contains(*candidate)
                })
        {
            return Err(MathError::Contract(
                "nonlinear selection chart transport".into(),
            ));
        }
        Ok(())
    }
}

/// Global minimum evidence, distinct from a numerical candidate or local root chart.
#[derive(Clone, Debug)]
pub enum SelectionEvidence {
    /// One regular eligible winning chart; every other competitive root is excluded.
    Unique(SelectionChart),
    /// Validated evidence of tied or better competitive roots.
    Multiple,
    /// Coverage or chart obligations remain unresolved; no derivative permission.
    Incomplete(SelectionProofRefusal),
}

/// Bounded, explicit reasons why evidence could not be supplied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionProofRefusal {
    /// The original operation, guard or finite domain cannot be represented.
    Unsupported,
    /// The local chart could not be validated.
    Chart,
    /// Complete eligible-domain coverage was not established.
    Coverage,
    /// A finite proof-resource allowance was exhausted.
    Resource,
    /// A strict guard, eligibility or original physical bound has no proved margin.
    Boundary,
}

/// Injected validated mathematics; a root solver continues to own numerical iteration.
pub trait SelectionVerifier: std::fmt::Debug + Send + Sync {
    /// Versioned library/adapter/rounding identity.
    fn identity(&self) -> ContentHash;
    /// Complete transient extent: all input transports and maximum sequential native work.
    fn workspace_bytes(&self, programs: &[Arc<RootIsolationProgram>]) -> Result<usize, MathError>;
    /// Establish the unique minimum over the original eligible-root union or refuse.
    fn certify(&self, request: &SelectionProofRequest<'_>) -> Result<SelectionEvidence, MathError>;
}
