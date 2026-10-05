// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Common semantic prediction and start screening. Predictions carry start-only meaning;
//! native payload transfer and scientific/control application remain separate decisions.
use pse_backend_native::{
    OracleContract, ProblemError, kkt, solve::Execution, square_response::SparseFactor,
};
use pse_ids::{ContentHash, SemanticId};
use pse_model::strategy::{BranchPolicy, SemanticProductKey};

/// Original-coordinate numerical proposal. Its source point/parameters are distinct from
/// the bound target; neither endpoint feasibility nor an origin label proves sheet transport.
#[derive(Clone, Debug)]
pub struct Proposal {
    coordinates: std::sync::Arc<Vec<SemanticId>>,
    values: std::sync::Arc<Vec<f64>>,
    source: SemanticProductKey,
    target: ContentHash,
    branch: BranchPolicy,
    owner: Option<std::sync::Arc<dyn pse_math::AllocationOwner>>,
}
impl Proposal {
    pub(crate) fn modified_specification(
        coordinates: Vec<SemanticId>,
        values: Vec<f64>,
        mut source: SemanticProductKey,
        target: ContentHash,
        branch: BranchPolicy,
    ) -> Result<Self, ProblemError> {
        branch
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        if branch != BranchPolicy::any_qualified() {
            return Err(ProblemError::Unsupported(
                "multistart cannot replace a connected original path".into(),
            ));
        }
        if coordinates.is_empty()
            || coordinates.len() != values.len()
            || values.iter().any(|v| !v.is_finite())
            || coordinates
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != coordinates.len()
            || source.binding != target
            || source.point != Some(pse_backend_native::square_response::point_key(&values))
        {
            return Err(ProblemError::Contract(
                "modified specification original source, physical inventory or finite point".into(),
            ));
        }
        // A fresh specification has neither an accepted-origin sheet nor consumed
        // numerical accuracy evidence. Its point is only a start proposal.
        source.branch = None;
        source.accuracy = None;
        Ok(Self {
            coordinates: std::sync::Arc::new(coordinates),
            values: std::sync::Arc::new(values),
            source,
            target,
            branch,
            owner: None,
        })
    }
    pub(crate) fn path(
        coordinates: Vec<SemanticId>,
        values: Vec<f64>,
        source: SemanticProductKey,
        target: ContentHash,
        branch: BranchPolicy,
    ) -> Result<Self, ProblemError> {
        branch
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        if coordinates.is_empty()
            || coordinates.len() != values.len()
            || values.iter().any(|v| !v.is_finite())
            || coordinates
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != coordinates.len()
        {
            return Err(ProblemError::Contract(
                "path proposal original physical inventory or finite values".into(),
            ));
        }
        Ok(Self {
            coordinates: std::sync::Arc::new(coordinates),
            values: std::sync::Arc::new(values),
            source,
            target,
            branch,
            owner: None,
        })
    }
    pub(crate) fn surrogate(
        point: &pse_math::surrogate::SurrogateProposal,
        correspondence: &pse_math::surrogate::FidelityCorrespondence,
        task: ContentHash,
        mut source: SemanticProductKey,
        target: ContentHash,
        branch: BranchPolicy,
    ) -> Result<Self, ProblemError> {
        correspondence.validate()?;
        branch
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        if branch.connected.is_some()
            || correspondence.original_target != target
            || point.original_target != target
            || point.fidelity != correspondence.fidelity
            || point.task != task
            || point.coordinates.len() != correspondence.coordinates.len()
            || point.coordinates.iter().any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "surrogate proposal target, fidelity, task, branch or coordinate mismatch".into(),
            ));
        }
        source.derivation = Some(task);
        source.accuracy = None;
        Ok(Self {
            coordinates: std::sync::Arc::new(correspondence.coordinates.clone()),
            values: std::sync::Arc::new(point.coordinates.as_ref().clone()),
            source,
            target,
            branch,
            owner: None,
        })
    }
    pub(crate) fn with_owner(
        mut self,
        owner: std::sync::Arc<dyn pse_math::AllocationOwner>,
    ) -> Self {
        self.owner = Some(pse_math::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    pub(crate) fn retained_bytes(&self) -> Result<usize, ProblemError> {
        size_of::<Self>()
            .checked_add(
                self.coordinates
                    .capacity()
                    .checked_mul(size_of::<SemanticId>())
                    .ok_or_else(|| ProblemError::memory("proposal coordinate extent"))?,
            )
            .and_then(|v| v.checked_add(self.values.capacity().checked_mul(size_of::<f64>())?))
            .ok_or_else(|| ProblemError::memory("proposal value extent"))
    }
    /// Proposal coordinates and values in physical original units.
    pub fn values(&self) -> impl Iterator<Item = (SemanticId, f64)> + '_ {
        self.coordinates
            .iter()
            .copied()
            .zip(self.values.iter().copied())
    }
    /// Exact fresh source dependencies; transported target values need not equal base values.
    pub fn source(&self) -> SemanticProductKey {
        self.source
    }
    /// Intended bound target identity, consumed by screening.
    pub fn target(&self) -> ContentHash {
        self.target
    }
    /// Transport/orientation requirements retained by this proposal.
    pub fn branch(&self) -> BranchPolicy {
        self.branch
    }
    /// Screen the complete original coordinate inventory, finite values, authored bounds
    /// and actual evaluability. Equation infeasibility is lawful for a start. No clamping.
    /// # Errors
    /// Wrong target/branch/coordinates, bounds, scope or original evaluation failure.
    pub fn screen(
        &self,
        contract: &OracleContract,
        target: ContentHash,
        branch: BranchPolicy,
        execution: &Execution,
        mut evaluate: impl FnMut(&[f64]) -> Result<(), ProblemError>,
    ) -> Result<Screened, ProblemError> {
        execution.check()?;
        if self.target != target
            || self.branch != branch
            || self.coordinates.len() != contract.variables.len()
            || self.values.len() != self.coordinates.len()
        {
            return Err(ProblemError::Contract(
                "proposal target, branch or coordinate extent mismatch".into(),
            ));
        }
        branch
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        for ((id, value), variable) in self
            .coordinates
            .iter()
            .zip(self.values.iter())
            .zip(&contract.variables)
        {
            if *id != variable.id
                || !value.is_finite()
                || *value < variable.lower
                || *value > variable.upper
            {
                return Err(ProblemError::Contract(
                    "semantic proposal original coordinate/bound screening".into(),
                ));
            }
        }
        evaluate(&self.values)?;
        execution.check()?;
        Ok(Screened {
            proposal: self.clone(),
            owner: None,
        })
    }
}
/// A screened start, not an original result permission or a transferable native working set.
#[derive(Clone, Debug)]
pub struct Screened {
    proposal: Proposal,
    owner: Option<std::sync::Arc<dyn pse_math::AllocationOwner>>,
}
impl Screened {
    /// Only physical values transfer between native adapters.
    pub fn point(&self) -> &[f64] {
        &self.proposal.values
    }
    /// Provenance for the consumed start operation.
    pub fn proposal(&self) -> &Proposal {
        &self.proposal
    }
    pub(crate) fn retained_bytes(&self) -> Result<usize, ProblemError> {
        size_of::<Self>()
            .checked_add(
                self.proposal
                    .coordinates
                    .capacity()
                    .checked_mul(size_of::<SemanticId>())
                    .ok_or_else(|| {
                        ProblemError::Contract("screened coordinate extent overflow".into())
                    })?,
            )
            .and_then(|n| {
                n.checked_add(
                    self.proposal
                        .values
                        .capacity()
                        .checked_mul(size_of::<f64>())?,
                )
            })
            .ok_or_else(|| ProblemError::Contract("screened start extent overflow".into()))
    }
    pub(crate) fn with_owner(
        mut self,
        owner: std::sync::Arc<dyn pse_math::AllocationOwner>,
    ) -> Self {
        self.owner = Some(owner);
        self
    }
}
/// Fresh-root tangent action. `rhs` is the physically normalized `-F_p Δp` supplied by
/// the prepared parametric evaluator. No dense `X_p` response is materialized here.
/// # Errors
/// Invalid branch, shape, scope, sparse backsolve or nonfinite proposal.
pub fn root(
    factor: &SparseFactor,
    rhs: &[f64],
    target: ContentHash,
    branch: BranchPolicy,
    execution: &Execution,
) -> Result<
    (
        Proposal,
        pse_backend_native::square_response::ActionEvidence,
    ),
    ProblemError,
> {
    branch
        .validate()
        .map_err(|e| ProblemError::Contract(e.to_string()))?;
    if branch.connected.is_some() {
        return Err(ProblemError::Unsupported("connected root prediction requires consumed verifier transport, beyond a matching sheet identity".into()));
    }
    let (delta, evidence) = factor.action(rhs, execution)?;
    let values = factor
        .point()
        .iter()
        .zip(delta)
        .map(|(x, d)| x + d)
        .collect::<Vec<_>>();
    if values.iter().any(|x| !x.is_finite()) {
        return Err(ProblemError::numerical("nonfinite root tangent proposal"));
    }
    Ok((
        Proposal {
            coordinates: std::sync::Arc::new(factor.states().to_vec()),
            values: std::sync::Arc::new(values),
            source: factor.key(),
            target,
            branch,
            owner: None,
        },
        evidence,
    ))
}
/// Apply the parameter partials and fresh factor retained by the shared postsolve owner.
/// The target producer checks unchanged authored meaning; this operation produces only
/// a physical start proposal and preserves the requested branch policy.
/// # Errors
/// Missing sheet transport, parameter mismatch or bounded sparse action failure.
pub fn parameter_root(
    predictor: &pse_backend_native::square_response::SparsePredictor,
    parameters: &[(SemanticId, f64)],
    target: ContentHash,
    branch: BranchPolicy,
    execution: &Execution,
) -> Result<
    (
        Proposal,
        pse_backend_native::square_response::ActionEvidence,
    ),
    ProblemError,
> {
    branch
        .validate()
        .map_err(|e| ProblemError::Contract(e.to_string()))?;
    if branch.connected.is_some() {
        return Err(ProblemError::Unsupported(
            "connected root prediction requires consumed sheet transport".into(),
        ));
    }
    let (delta, evidence) = predictor.action(parameters, execution)?;
    let factor = predictor.factor();
    let values = factor
        .point()
        .iter()
        .zip(delta)
        .map(|(x, d)| x + d)
        .collect::<Vec<_>>();
    if values.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::numerical("nonfinite parameter root proposal"));
    }
    Ok((
        Proposal {
            coordinates: std::sync::Arc::new(factor.states().to_vec()),
            values: std::sync::Arc::new(values),
            source: factor.key(),
            target,
            branch,
            owner: None,
        },
        evidence,
    ))
}
/// The shared KKT predictor operation used by related-target consumers. Activity changes
/// retain the library's typed refusal; an admitted prediction still has only start meaning.
/// Horizon application is a separate control decision made by its existing owner.
/// # Errors
/// The actual pinned factor's parameter, backsolve or activity refusal.
pub fn kkt(
    advance: &kkt::Advance,
    parameters: &[(SemanticId, f64)],
) -> Result<kkt::Prediction, kkt::Fallback> {
    kkt::predict(advance, parameters)
}

/// One shared proposal producer selection. The caller supplies genuinely composed original
/// permission and source dependencies; every produced endpoint still requires target screening.
pub(crate) enum ProposalMechanism<'a> {
    Root {
        predictor: &'a pse_backend_native::square_response::SparsePredictor,
        parameters: &'a [(SemanticId, f64)],
    },
    #[cfg(feature = "solver-diffsol")]
    Kkt {
        advance: &'a kkt::Advance,
        parameters: &'a [(SemanticId, f64)],
        activity: Option<(usize, kkt::activity::Limits)>,
    },
    Activity {
        advance: &'a kkt::Advance,
        parameters: &'a [(SemanticId, f64)],
        segments: usize,
        limits: kkt::activity::Limits,
    },
    Qp {
        request: kkt::path::qp::Request<'a>,
        coordinates: &'a [SemanticId],
    },
    Secant {
        history: &'a SecantHistory,
        parameter: f64,
        scaled_step_limit: f64,
    },
}
pub(crate) struct SelectionRequest<'a> {
    pub(crate) mechanism: ProposalMechanism<'a>,
    pub(crate) permission: &'a crate::workflow::numerics::CandidateDecision,
    pub(crate) source: SemanticProductKey,
    pub(crate) target: ContentHash,
    pub(crate) branch: BranchPolicy,
}
/// Actual source-owned proposal and operation evidence. Partial activity coverage remains
/// partial; every non-control endpoint has only start meaning.
pub(crate) enum SelectedProposal {
    Root {
        proposal: Box<Proposal>,
        work: pse_backend_native::square_response::ActionEvidence,
    },
    #[cfg(feature = "solver-diffsol")]
    Kkt {
        prediction: kkt::Prediction,
    },
    Activity {
        proposal: Box<Proposal>,
        path: kkt::path::PathPrediction,
        fallback: Option<kkt::Fallback>,
    },
    Qp {
        proposal: Box<Proposal>,
        outcome: Box<kkt::path::qp::Outcome>,
    },
    Secant {
        proposal: Box<Proposal>,
    },
}
pub(crate) enum SelectionFailure {
    #[cfg(feature = "solver-diffsol")]
    Kkt(kkt::Fallback),
    Cause(std::sync::Arc<ProblemError>),
}
impl From<ProblemError> for SelectionFailure {
    fn from(cause: ProblemError) -> Self {
        Self::Cause(std::sync::Arc::new(cause))
    }
}
impl From<std::sync::Arc<ProblemError>> for SelectionFailure {
    fn from(cause: std::sync::Arc<ProblemError>) -> Self {
        Self::Cause(cause)
    }
}
impl SelectionFailure {
    pub(crate) fn into_problem(self) -> ProblemError {
        match self {
            #[cfg(feature = "solver-diffsol")]
            Self::Kkt(fallback) => ProblemError::numerical(fallback.to_string()),
            Self::Cause(cause) => ProblemError::Math(pse_math::MathError::Typed {
                retained: cause.retained_bytes(),
                cause: pse_model::diagnostic::DiagnosticCause::from_shared(cause),
            }),
        }
    }
}
pub(crate) fn select(
    request: SelectionRequest<'_>,
    execution: &Execution,
) -> Result<SelectedProposal, SelectionFailure> {
    execution.check()?;
    request
        .branch
        .validate()
        .map_err(|error| ProblemError::Contract(error.to_string()))?;
    if !request.permission.permits_use() || request.source.point.is_none() {
        return Err(ProblemError::Contract(
            "proposal selection requires composed original permission and exact source point"
                .into(),
        )
        .into());
    }
    // These existing producers possess no connected transport/orientation verifier.
    if request.branch.connected.is_some() {
        return Err(ProblemError::Unsupported(
            "selected proposal producer has no connected-sheet transport witness".into(),
        )
        .into());
    }
    let verify_point = |point: &[f64]| -> Result<(), ProblemError> {
        if request.source.point != Some(pse_backend_native::square_response::point_key(point)) {
            Err(ProblemError::Contract(
                "proposal producer point differs from the permitted source".into(),
            ))
        } else {
            Ok(())
        }
    };
    let selected = match request.mechanism {
        ProposalMechanism::Root {
            predictor,
            parameters,
        } => {
            if predictor.factor().key() != request.source {
                return Err(ProblemError::Contract(
                    "root predictor source dependencies differ from original permission".into(),
                )
                .into());
            }
            let (proposal, work) = parameter_root(
                predictor,
                parameters,
                request.target,
                request.branch,
                execution,
            )?;
            SelectedProposal::Root {
                proposal: Box::new(proposal),
                work,
            }
        }
        #[cfg(feature = "solver-diffsol")]
        ProposalMechanism::Kkt {
            advance,
            parameters,
            activity,
        } => {
            verify_point(advance.point())?;
            match kkt(advance, parameters) {
                Ok(prediction) => SelectedProposal::Kkt { prediction },
                Err(fallback @ kkt::Fallback::ActiveSet { .. }) => {
                    let Some((segments, limits)) = activity else {
                        return Err(SelectionFailure::Kkt(fallback));
                    };
                    let selected = match select(
                        SelectionRequest {
                            mechanism: ProposalMechanism::Activity {
                                advance,
                                parameters,
                                segments,
                                limits,
                            },
                            permission: request.permission,
                            source: request.source,
                            target: request.target,
                            branch: request.branch,
                        },
                        execution,
                    ) {
                        Ok(selected) => selected,
                        Err(error) => {
                            let cause = error.into_problem();
                            let class =
                                pse_model::diagnostic::DiagnosticProjection::boundary_diagnostic(
                                    &cause,
                                    pse_diagnostics::DiagnosticStage::Native,
                                )
                                .class;
                            if matches!(
                                class,
                                pse_model::diagnostic::BoundaryClass::Unsupported
                                    | pse_model::diagnostic::BoundaryClass::Numerical
                            ) {
                                return Err(SelectionFailure::Kkt(fallback));
                            }
                            return Err(cause.into());
                        }
                    };
                    let SelectedProposal::Activity { proposal, path, .. } = selected else {
                        return Err(ProblemError::internal(
                            "activity fallback selected another producer",
                        )
                        .into());
                    };
                    return Ok(SelectedProposal::Activity {
                        proposal,
                        path,
                        fallback: Some(fallback),
                    });
                }
                Err(fallback) => return Err(SelectionFailure::Kkt(fallback)),
            }
        }
        ProposalMechanism::Activity {
            advance,
            parameters,
            segments,
            limits,
        } => {
            verify_point(advance.point())?;
            let path =
                kkt::path::predict(advance, parameters, segments, limits, execution.clone())?;
            let proposal = Proposal::path(
                path.prediction.variables.clone(),
                path.prediction.primal.clone(),
                request.source,
                request.target,
                request.branch,
            )?;
            SelectedProposal::Activity {
                proposal: Box::new(proposal),
                path,
                fallback: None,
            }
        }
        ProposalMechanism::Qp {
            request: qp,
            coordinates,
        } => {
            verify_point(&qp.source.x)?;
            if coordinates.len() != qp.previous.n
                || coordinates
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != coordinates.len()
            {
                return Err(ProblemError::Contract(
                    "QP proposal requires distinct original named coordinates".into(),
                )
                .into());
            }
            let outcome = kkt::path::qp::predict(qp, execution.clone())?;
            let proposal = Proposal::path(
                coordinates.to_vec(),
                outcome.solution.x.clone(),
                request.source,
                request.target,
                request.branch,
            )?;
            SelectedProposal::Qp {
                proposal: Box::new(proposal),
                outcome: Box::new(outcome),
            }
        }
        ProposalMechanism::Secant {
            history,
            parameter,
            scaled_step_limit,
        } => {
            if history.newer.key != request.source {
                return Err(ProblemError::Contract(
                    "secant history differs from the permitted source dependencies".into(),
                )
                .into());
            }
            let proposal =
                history.predict(parameter, scaled_step_limit, request.target, request.branch)?;
            SelectedProposal::Secant {
                proposal: Box::new(proposal),
            }
        }
    };
    execution.check()?;
    Ok(selected)
}

/// Two original-permitted points in one scaled path coordinate. Source dependencies remain
/// fixed while binding/point/parameter identities are allowed to change along that path.
#[derive(Clone, Debug)]
pub struct SecantHistory {
    older: Anchor,
    newer: Anchor,
    parameter_scale: f64,
}
/// An original-permitted path sample, constructed by the shared completion consumer.
#[derive(Clone, Debug)]
pub struct Anchor {
    key: SemanticProductKey,
    coordinates: std::sync::Arc<Vec<SemanticId>>,
    values: std::sync::Arc<Vec<f64>>,
    parameter: f64,
    owner: Option<std::sync::Arc<dyn pse_math::AllocationOwner>>,
}
impl Anchor {
    pub(crate) fn with_owner(
        mut self,
        owner: std::sync::Arc<dyn pse_math::AllocationOwner>,
    ) -> Self {
        self.owner = Some(pse_math::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    pub(crate) fn retained_bytes(&self) -> Result<usize, ProblemError> {
        size_of::<Self>()
            .checked_add(
                self.coordinates
                    .capacity()
                    .checked_mul(size_of::<SemanticId>())
                    .ok_or_else(|| ProblemError::memory("anchor coordinate extent"))?,
            )
            .and_then(|v| v.checked_add(self.values.capacity().checked_mul(size_of::<f64>())?))
            .ok_or_else(|| ProblemError::memory("anchor value extent"))
    }
    pub(crate) fn admitted(
        key: SemanticProductKey,
        coordinates: Vec<SemanticId>,
        values: Vec<f64>,
        parameter: f64,
        permission: crate::workflow::numerics::CandidateDecision,
    ) -> Result<Self, ProblemError> {
        if !permission.permits_use()
            || coordinates.is_empty()
            || coordinates.len() != values.len()
            || coordinates
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != coordinates.len()
            || values.iter().any(|x| !x.is_finite())
            || !parameter.is_finite()
            || key.point != Some(pse_backend_native::square_response::point_key(&values))
        {
            return Err(ProblemError::Contract(
                "secant anchor requires original completion permission and exact finite point"
                    .into(),
            ));
        }
        Ok(Self {
            key,
            coordinates: std::sync::Arc::new(coordinates),
            values: std::sync::Arc::new(values),
            parameter,
            owner: None,
        })
    }
}
impl SecantHistory {
    /// Bind two independently permitted original points. The completion owner supplies
    /// permission; a native success/feasibility flag cannot construct this history.
    /// # Errors
    /// No original permission, changed fixed dependencies, nonfinite or coincident samples.
    pub fn new(older: Anchor, newer: Anchor, parameter_scale: f64) -> Result<Self, ProblemError> {
        let fixed = |key: SemanticProductKey| {
            (
                key.structure,
                key.numerical_policy,
                key.normalization,
                key.derivation,
                key.branch,
                key.accuracy,
            )
        };
        if !parameter_scale.is_finite()
            || parameter_scale <= 0.
            || older.coordinates != newer.coordinates
            || fixed(older.key) != fixed(newer.key)
            || older.parameter == newer.parameter
        {
            return Err(ProblemError::Contract(
                "secant fixed dependencies or scaled parameter mismatch".into(),
            ));
        }
        Ok(Self {
            older,
            newer,
            parameter_scale,
        })
    }
    /// Exact dependencies of the newer independently accepted source sample.
    pub fn source(&self) -> SemanticProductKey {
        self.newer.key
    }
    /// Predict in the declared physical parameter, with a finite scaled extrapolation cap.
    /// # Errors
    /// Invalid parameter/cap/branch or a step outside the admitted numerical history slice.
    pub fn predict(
        &self,
        parameter: f64,
        scaled_step_limit: f64,
        target: ContentHash,
        branch: BranchPolicy,
    ) -> Result<Proposal, ProblemError> {
        branch
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        let step = (parameter - self.newer.parameter) / self.parameter_scale;
        if !step.is_finite()
            || !scaled_step_limit.is_finite()
            || scaled_step_limit <= 0.
            || step.abs() > scaled_step_limit
            || branch.connected.is_some()
        {
            return Err(ProblemError::Contract(
                "secant extrapolation allowance or sheet mismatch".into(),
            ));
        }
        let fraction =
            (parameter - self.newer.parameter) / (self.newer.parameter - self.older.parameter);
        let values = self
            .newer
            .values
            .iter()
            .zip(self.older.values.iter())
            .map(|(new, old)| new + fraction * (new - old))
            .collect::<Vec<_>>();
        if values.iter().any(|x| !x.is_finite()) {
            return Err(ProblemError::numerical("nonfinite scaled secant proposal"));
        }
        Ok(Proposal {
            coordinates: self.newer.coordinates.clone(),
            values: std::sync::Arc::new(values),
            source: self.newer.key,
            target,
            branch,
            owner: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::numerics::CandidateDecision;
    use pse_model::generated::enums::CandidateUse;
    fn sample(
        value: f64,
        parameter: f64,
        permission: CandidateUse,
    ) -> Result<Anchor, ProblemError> {
        let hash = ContentHash::from_bytes([1; 32]);
        let values = vec![value];
        let key = SemanticProductKey {
            structure: hash,
            binding: ContentHash::from_bytes([parameter as u8; 32]),
            numerical_policy: Some(hash),
            normalization: Some(hash),
            point: Some(pse_backend_native::square_response::point_key(&values)),
            parameters: Some(ContentHash::from_bytes([parameter as u8; 32])),
            derivation: None,
            branch: None,
            accuracy: Some(hash),
        };
        Anchor::admitted(
            key,
            vec![SemanticId::NIL],
            values,
            parameter,
            CandidateDecision {
                usability: permission,
                qualifiers: Vec::new(),
                refusals: Vec::new(),
                bound: None,
            },
        )
    }
    #[test]
    fn secant_transports_bound_values_with_fixed_dependencies_and_start_only_screening() {
        let history = SecantHistory::new(
            sample(2., 1., CandidateUse::Usable).unwrap(),
            sample(4., 2., CandidateUse::Usable).unwrap(),
            10.,
        )
        .unwrap();
        let target = ContentHash::from_bytes([9; 32]);
        let branch = BranchPolicy::any_qualified();
        let execution = Execution::new(
            std::sync::Arc::default(),
            &pse_backend_native::solve::Controls::default(),
        );
        let permission = CandidateDecision {
            usability: CandidateUse::Usable,
            qualifiers: Vec::new(),
            refusals: Vec::new(),
            bound: None,
        };
        let selected = select(
            SelectionRequest {
                mechanism: ProposalMechanism::Secant {
                    history: &history,
                    parameter: 3.,
                    scaled_step_limit: 0.2,
                },
                permission: &permission,
                source: history.source(),
                target,
                branch,
            },
            &execution,
        )
        .unwrap_or_else(|failure| panic!("{}", failure.into_problem()));
        let SelectedProposal::Secant { proposal } = selected else {
            panic!("secant expected");
        };
        assert_eq!(proposal.values.as_slice(), [6.]);
        let contract = OracleContract {
            identity: target,
            variables: vec![pse_backend_native::Variable {
                id: SemanticId::NIL,
                lower: 0.,
                upper: 10.,
            }],
            rows: vec![SemanticId::from_bytes([2; 16])],
            derivatives: pse_kernels::DerivativeOrder::Value,
            smoothness: pse_kernels::DerivativeOrder::Value,
        };
        let execution = Execution::new(
            std::sync::Arc::default(),
            &pse_backend_native::solve::Controls::default(),
        );
        let mut calls = 0;
        let screened = proposal
            .screen(&contract, target, branch, &execution, |x| {
                calls += 1;
                assert_eq!(x, [6.]);
                Ok(())
            })
            .unwrap();
        assert_eq!(calls, 1);
        assert_eq!(screened.point(), [6.]);
        let outside = history.predict(6., 1., target, branch).unwrap();
        assert!(matches!(
            outside.screen(&contract, target, branch, &execution, |_| panic!(
                "bound refusal precedes evaluator"
            )),
            Err(ProblemError::Contract(_))
        ));
        assert!(
            proposal
                .screen(
                    &contract,
                    ContentHash::from_bytes([8; 32]),
                    branch,
                    &execution,
                    |_| panic!("target refusal precedes evaluator")
                )
                .is_err()
        );
        assert!(
            proposal
                .screen(&contract, target, branch, &execution, |_| Err(
                    ProblemError::Contract("authored guard".into())
                ))
                .is_err()
        );
    }
    #[test]
    fn common_secant_selection_refuses_wrong_source_and_connected_transport() {
        let history = SecantHistory::new(
            sample(2., 1., CandidateUse::Usable).unwrap(),
            sample(4., 2., CandidateUse::Usable).unwrap(),
            1.,
        )
        .unwrap();
        let execution = Execution::new(
            std::sync::Arc::default(),
            &pse_backend_native::solve::Controls::default(),
        );
        let permission = CandidateDecision {
            usability: CandidateUse::Usable,
            qualifiers: Vec::new(),
            refusals: Vec::new(),
            bound: None,
        };
        let hash = ContentHash::from_bytes([9; 32]);
        let mut wrong = history.source();
        wrong.normalization = None;
        assert!(
            select(
                SelectionRequest {
                    mechanism: ProposalMechanism::Secant {
                        history: &history,
                        parameter: 3.,
                        scaled_step_limit: 1.
                    },
                    permission: &permission,
                    source: wrong,
                    target: hash,
                    branch: BranchPolicy::any_qualified()
                },
                &execution
            )
            .is_err()
        );
        let connected = BranchPolicy {
            kind: pse_model::strategy::BranchKind::Connected,
            connected: Some(pse_model::strategy::ConnectedPath {
                path: hash,
                sheet: hash,
                transport: hash,
                orientation: hash,
            }),
        };
        assert!(
            select(
                SelectionRequest {
                    mechanism: ProposalMechanism::Secant {
                        history: &history,
                        parameter: 3.,
                        scaled_step_limit: 1.
                    },
                    permission: &permission,
                    source: history.source(),
                    target: hash,
                    branch: connected
                },
                &execution
            )
            .is_err()
        );
    }
    #[test]
    fn secant_refuses_seed_only_changed_fixed_dependencies_and_extrapolation_slice() {
        assert!(sample(1., 1., CandidateUse::SeedOnly).is_err());
        let mut newer = sample(4., 2., CandidateUse::Usable).unwrap();
        newer.key.accuracy = None;
        assert!(
            SecantHistory::new(sample(2., 1., CandidateUse::Usable).unwrap(), newer, 1.).is_err()
        );
        let mut older = sample(2., 1., CandidateUse::Usable).unwrap();
        let mut newer = sample(4., 2., CandidateUse::Usable).unwrap();
        older.key.branch = Some(ContentHash::from_bytes([3; 32]));
        newer.key.branch = Some(ContentHash::from_bytes([4; 32]));
        assert!(
            SecantHistory::new(older, newer, 1.).is_err(),
            "unrelated sheets cannot supply one history"
        );
        let history = SecantHistory::new(
            sample(2., 1., CandidateUse::Usable).unwrap(),
            sample(4., 2., CandidateUse::Usable).unwrap(),
            1.,
        )
        .unwrap();
        assert!(
            history
                .predict(
                    3.,
                    0.5,
                    ContentHash::from_bytes([9; 32]),
                    BranchPolicy::any_qualified()
                )
                .is_err()
        );
    }
}
