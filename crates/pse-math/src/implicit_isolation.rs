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
        self.validate_scope(request, verifier)?;
        if self.order < request.order {
            return Err(MathError::Contract(
                "nonlinear selection chart guard order".into(),
            ));
        }
        Ok(())
    }
    /// Check unchanged complete exclusion/source scope independently of newly demanded
    /// winning-chart guard order. Numerical root accuracy remains a separate obligation.
    pub fn validate_scope(
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

/// Which continuity obligation the caller actually consumes. Root-sheet transport
/// alone never establishes that this root wins the selector throughout the segment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartChainCoverage {
    /// One regular eligible root sheet joins independently certified selected endpoints.
    RootSheet,
    /// The same selected function wins throughout the entire covered parameter segment.
    SelectedFunction,
}
/// A finite segment transport request, independent of numerical accuracy and endpoint
/// competitive-exclusion evidence. The same outer time/cancellation allowance applies.
#[derive(Debug)]
pub struct ChartChainRequest<'a> {
    /// Current endpoint's complete original source, domain, order and remaining controls.
    pub endpoint: &'a SelectionProofRequest<'a>,
    /// Accepted originating chart, whose selected-root existence is already proved.
    pub previous: &'a SelectionChart,
    /// Independently certified selected endpoint chart.
    pub next: &'a SelectionChart,
    /// Accepted originating parameter point; never inferred from a regime label.
    pub origin: &'a [f64],
    /// Required mathematical coverage, explicitly separate from sheet identity.
    pub coverage: ChartChainCoverage,
}
impl ChartChainRequest<'_> {
    /// Validate exact endpoint scope before invoking the validating mathematics.
    pub fn validate(&self, verifier: ContentHash) -> Result<(), MathError> {
        self.next.validate(self.endpoint, verifier)?;
        let candidate = self
            .previous
            .existence
            .iter()
            .map(|i| i.lower + (i.upper - i.lower) * 0.5)
            .collect::<Vec<_>>();
        let origin = SelectionProofRequest {
            selection: self.endpoint.selection,
            alternatives: self.endpoint.alternatives,
            winner: self.endpoint.winner,
            parameters: self.origin,
            candidate: &candidate,
            order: DerivativeOrder::First,
            time_limit: self.endpoint.time_limit,
            cancel: self.endpoint.cancel,
        };
        self.previous.validate(&origin, verifier)
    }
}
/// Actual completed proof observations. Counts describe library calls/certified charts;
/// they are neither a numerical accuracy certificate nor intermediate selector evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChartChainProof {
    /// Coverage actually established by the producer.
    pub coverage: ChartChainCoverage,
    /// Newly constructed regular uniform charts, excluding the two selected endpoints.
    pub charts: u64,
    /// Certified common-root connections between adjacent charts.
    pub connections: u64,
    /// Proof cells attempted under the producer's one finite shared account.
    pub proof_cells: u64,
}
/// Actual attempt observations retained even when transport refuses. Partial chart
/// counts are work observations and establish no covered segment or selected lineage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChartChainWork {
    /// Uniform charts successfully proved before the attempt ended.
    pub charts: u64,
    /// Common-root anchors proved before the attempt ended.
    pub connections: u64,
    /// All attempted chart/anchor cells, including failed subdivision work.
    pub proof_cells: u64,
}
/// Complete transport or an explicit bounded refusal. No certificate is synthesized
/// from overlapping boxes, residual-small proposals or a matching alternative id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartChainEvidence {
    /// The validating producer covered the segment and connected every adjacent chart.
    Connected(ChartChainProof),
    /// Transport or the specifically demanded selection coverage remains unestablished.
    Incomplete(SelectionProofRefusal),
    /// Finite transport attempt refused after actual proof work.
    Refused {
        /// Actual bounded refusal; partial observations establish no transport.
        reason: SelectionProofRefusal,
        /// Attempted/proved work observed before refusal.
        work: ChartChainWork,
    },
    /// Original cancellation interrupted a native attempt after observed work.
    Interrupted(ChartChainWork),
}
impl ChartChainEvidence {
    /// Work observations are available independently of mathematical acceptance.
    pub fn work(self) -> ChartChainWork {
        match self {
            Self::Connected(proof) => ChartChainWork {
                charts: proof.charts,
                connections: proof.connections,
                proof_cells: proof.proof_cells,
            },
            Self::Refused { work, .. } | Self::Interrupted(work) => work,
            Self::Incomplete(_) => ChartChainWork::default(),
        }
    }
}
/// Verified IFT action at this actual parameter point. This interval product is
/// separate from root accuracy and from the caller's approximate numerical tangent.
#[derive(Clone, Debug, PartialEq)]
pub enum RootActionEvidence {
    /// Original residual interval derivatives and a library interval inverse enclose
    /// the true action on the certified winning root. No estimates are relabeled.
    Enclosed {
        /// Original physical action intervals in unknown coordinate order.
        intervals: Vec<ProofInterval>,
        /// Actual attempted interval derivative/inverse proof cells.
        proof_cells: u64,
    },
    /// Interval derivative/inverse work did not establish the demanded product.
    Incomplete {
        /// Why the requested product remains unestablished.
        reason: SelectionProofRefusal,
        /// Actual attempted cells before refusal, including failed inverse work.
        proof_cells: u64,
    },
    /// Original cancellation interrupted actual interval proof work.
    Interrupted {
        /// Actual attempted cells before original cancellation.
        proof_cells: u64,
    },
}

/// Original residual interval proof at one actual parameter point, independent of
/// the uniform chart and its competitive selection coverage.
#[derive(Clone, Debug, PartialEq)]
pub enum RootPointEvidence {
    /// Library-contracted selected root and uniform inverse over the hull of that
    /// enclosure and the approximate numerical candidate.
    Enclosed {
        /// Physical unknown intervals, in the winning alternative's unknown order.
        intervals: Vec<ProofInterval>,
        /// Outward upper bound of `||Sz^-1 Fz^-1 Sr||_infinity` over the hull.
        /// Positive physical `Sz` and `Sr` follow unknown and residual row order.
        inverse_norm_upper: f64,
        /// Actual attempted library proof cells, including inverse validation.
        proof_cells: u64,
    },
    /// The requested fixed-point/inverse proof remains unestablished.
    Incomplete {
        /// Why the fixed-point/inverse product remains unestablished.
        reason: SelectionProofRefusal,
        /// Actual attempted cells before refusal, including failed inverse work.
        proof_cells: u64,
    },
    /// The original caller interrupted actual library proof work.
    Interrupted {
        /// Actual attempted cells before original cancellation.
        proof_cells: u64,
    },
}
/// Uniform point and optional action enclosure over an incoming parameter box
/// inside an unchanged competitively certified selected chart.
#[derive(Clone, Debug, PartialEq)]
pub enum RootNeighborhoodEvidence {
    /// Original interval residual mathematics established both products.
    Enclosed {
        /// Original physical root intervals in unknown order for every incoming parameter.
        points: Vec<ProofInterval>,
        /// Original physical action intervals for every incoming point/direction pair.
        actions: Option<Vec<ProofInterval>>,
        /// Actual attempted interval root and inverse cells, including failed work.
        proof_cells: u64,
    },
    /// No uniform product was established within the actual allowance.
    Incomplete {
        /// Why uniform incoming point/action accuracy remains unestablished.
        reason: SelectionProofRefusal,
        /// Actual attempted cells before this refusal.
        proof_cells: u64,
    },
    /// Original caller interrupted actual proof work.
    Interrupted {
        /// Actual attempted cells before original cancellation.
        proof_cells: u64,
    },
}
/// Injected validated mathematics; a root solver continues to own numerical iteration.
pub trait SelectionVerifier: std::fmt::Debug + Send + Sync {
    /// Enclose the original selected solution and derivative for uncertain incoming
    /// point/direction coordinates. Every parameter interval must remain inside the
    /// unchanged chart; exact-point proofs cannot implement this by inference.
    fn enclose_neighborhood(
        &self,
        _request: &SelectionProofRequest<'_>,
        _chart: &SelectionChart,
        _parameters: &[ProofInterval],
        _directions: Option<&[ProofInterval]>,
        _max_cells: u64,
    ) -> Result<RootNeighborhoodEvidence, MathError> {
        Ok(RootNeighborhoodEvidence::Incomplete {
            reason: SelectionProofRefusal::Unsupported,
            proof_cells: 0,
        })
    }
    /// Versioned library/adapter/rounding identity.
    fn identity(&self) -> ContentHash;
    /// Complete transient extent: all input transports and maximum sequential native work.
    fn workspace_bytes(&self, programs: &[Arc<RootIsolationProgram>]) -> Result<usize, MathError>;
    /// Establish the unique minimum over the original eligible-root union or refuse.
    fn certify(&self, request: &SelectionProofRequest<'_>) -> Result<SelectionEvidence, MathError>;
    /// Establish newly demanded winning-chart guards on the unchanged uniform chart.
    /// Competitive exclusion is inherited only after exact scope validation; a default
    /// supplier cannot invent higher-order certification from an existing chart.
    fn promote(
        &self,
        _request: &SelectionProofRequest<'_>,
        _chart: &SelectionChart,
    ) -> Result<SelectionEvidence, MathError> {
        Ok(SelectionEvidence::Incomplete(
            SelectionProofRefusal::Unsupported,
        ))
    }
    /// Enclose the actual implicit derivative action at the verified selected root.
    /// The request carries original complete scope; physical direction coordinates
    /// follow its parameter order. Unsupported suppliers establish no action accuracy.
    fn enclose_action(
        &self,
        _request: &SelectionProofRequest<'_>,
        _chart: &SelectionChart,
        _direction: &[f64],
    ) -> Result<RootActionEvidence, MathError> {
        Ok(RootActionEvidence::Incomplete {
            reason: SelectionProofRefusal::Unsupported,
            proof_cells: 0,
        })
    }
    /// Contract at singleton parameters inside the unchanged original chart and
    /// bound the interval inverse over the resulting enclosure/candidate hull.
    /// Positive physical scales follow original unknown/residual row order. The
    /// remaining cell allowance is shared by the consumer's refinement rounds.
    fn refine_point(
        &self,
        _request: &SelectionProofRequest<'_>,
        _chart: &SelectionChart,
        _unknown_scales: &[f64],
        _row_scales: &[f64],
        _max_cells: u64,
    ) -> Result<RootPointEvidence, MathError> {
        Ok(RootPointEvidence::Incomplete {
            reason: SelectionProofRefusal::Unsupported,
            proof_cells: 0,
        })
    }
    /// Enclose the action under the consumer's remaining shared proof allowance.
    /// A supplier exposing only the old full-cap action cannot invent this bound.
    fn enclose_action_bounded(
        &self,
        _request: &SelectionProofRequest<'_>,
        _chart: &SelectionChart,
        _direction: &[f64],
        _max_cells: u64,
    ) -> Result<RootActionEvidence, MathError> {
        Ok(RootActionEvidence::Incomplete {
            reason: SelectionProofRefusal::Unsupported,
            proof_cells: 0,
        })
    }
    /// Build a bounded segment-covering chain, consuming the same outer controls.
    /// A supplier without this capability refuses; endpoints alone imply no transport.
    fn connect_chain(
        &self,
        _request: &ChartChainRequest<'_>,
    ) -> Result<ChartChainEvidence, MathError> {
        Ok(ChartChainEvidence::Incomplete(
            SelectionProofRefusal::Unsupported,
        ))
    }
    /// Establish a certified common root at a shared parameter anchor, inside both
    /// charts' uniform uniqueness regions. Endpoint box overlap supplies no proof.
    fn connect(
        &self,
        _request: &SelectionProofRequest<'_>,
        _previous: &SelectionChart,
        _next: &SelectionChart,
    ) -> Result<bool, MathError> {
        Ok(false)
    }
}
