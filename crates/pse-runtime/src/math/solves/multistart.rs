// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fresh declared specifications for the common original numerical strategy.
//! Each product is one seed, not an independent worker or a private retry loop.
use super::*;
use crate::math::prediction::Proposal;
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::ExecutionScope;
use pse_model::strategy::{BranchPolicy, ProfileRef, StartOrigin};
use std::collections::BTreeSet;
#[cfg(test)]
#[cfg(any(feature = "solver-kinsol", feature = "solver-ipopt"))]
#[path = "multistart_tests.rs"]
mod tests;

#[derive(Debug)]
struct Data {
    original: Arc<PreparedSolve>,
    profile: SolverProfile,
    backend: Backend,
    proposal: Proposal,
    scope: ExecutionScope,
    key: ContentHash,
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// One immutable fresh seed over the complete original source. It never carries
/// original result permission, a sheet witness or a native working set.
#[derive(Clone, Debug)]
pub struct PreparedMultistart {
    data: Arc<Data>,
}
impl PreparedMultistart {
    /// Complete original target whose usual assessment alone can grant permission.
    pub fn original_target(&self) -> &PreparedSolve {
        &self.data.original
    }
    /// Exact native profile after binding a primal-only explicit seed. Its method and
    /// remaining controls are those of the original target.
    pub fn profile(&self) -> &SolverProfile {
        &self.data.profile
    }
    /// Actual original source and canonical physical seed identity.
    pub fn key(&self) -> ContentHash {
        self.data.key
    }
    /// The enclosing task's unrenewed finite deadline and cancellation owner.
    pub fn scope(&self) -> &ExecutionScope {
        &self.data.scope
    }
    /// Any-qualified root permission required by this start mechanism.
    pub fn branch(&self) -> BranchPolicy {
        self.data.proposal.branch()
    }
    /// Actual full physical point, in original native coordinate order.
    pub fn point(&self) -> Vec<f64> {
        self.data.proposal.values().map(|(_, v)| v).collect()
    }
    /// Actual original identity; changing a start never changes the scientific target.
    pub fn original_identity(&self) -> Result<ContentHash, ProblemError> {
        self.original_target().original_identity()
    }
    /// Exact inner native profile, including explicit primal submission.
    pub fn strategy_profile(&self) -> Result<ContentHash, ProblemError> {
        Ok(profile_key(self.profile())?.as_id())
    }
    /// Frozen native profile declaration consumed by the common strategy.
    pub fn profile_ref(&self) -> Result<ProfileRef, ProblemError> {
        Ok(ProfileRef {
            backend: self.backend(),
            key: self.strategy_profile()?,
        })
    }
    /// Selected original backend, checked at preparation.
    pub fn backend(&self) -> Backend {
        self.data.backend
    }
    /// Actual source products and fresh physical specification consumed by this rung.
    pub fn support(&self) -> Result<BTreeSet<ContentHash>, ProblemError> {
        let mut keys = BTreeSet::from([
            self.key(),
            self.original_identity()?,
            self.original_target().preparation_identity()?,
        ]);
        if let Representation::Algebraic(source) = &self.original_target().representation {
            keys.extend(source.prepared.compiled().artifacts.iter().map(|a| a.key()));
            keys.extend(source.providers.values().map(|p| p.configuration_key()));
        }
        Ok(keys)
    }
    /// A current-task specification never claims accepted or connected provenance.
    pub fn origin(&self) -> StartOrigin {
        StartOrigin::ModifiedSpecification
    }
    pub(crate) fn result_bytes(&self) -> Result<usize, MathRuntimeError> {
        self.original_target().result_bytes()
    }
}
/// The common driver keeps the reservation alive through original native teardown.
/// No storage is retained by cloning an unreserved source binding.
pub(crate) struct BoundMultistart {
    pub(crate) step: PreparedSolve,
    pub(crate) _bindings: super::super::WorkerCharge,
    pub(crate) screening_evaluations: u64,
}
impl MathService {
    /// Freeze one declared seed over the actual compiled original inventory and bounds.
    /// Original guards are evaluated immediately before native dispatch on the common
    /// admitted worker. Equation infeasibility is lawful for a start.
    ///
    /// # Errors
    /// Connected policy, incomplete/nonfinite/out-of-bounds coordinates, changed original
    /// task, non-root/non-explicit native source or finite storage admission fails.
    pub fn prepare_multistart(
        &self,
        mut original: PreparedSolve,
        seed: BTreeMap<SemanticId, f64>,
        branch: BranchPolicy,
        scope: ExecutionScope,
    ) -> Result<PreparedMultistart, MathRuntimeError> {
        scope.check().map_err(ProblemError::from)?;
        branch
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        if branch != BranchPolicy::any_qualified() {
            return Err(ProblemError::Unsupported(
                "multistart cannot replace a connected original path".into(),
            )
            .into());
        }
        if scope.deadline().is_none()
            || original.task_scope().is_some_and(|prior| {
                !Arc::ptr_eq(prior.cancellation(), scope.cancellation())
                    || prior.deadline() != scope.deadline()
            })
        {
            return Err(ProblemError::Contract(
                "multistart must preserve one original finite task scope".into(),
            )
            .into());
        }
        if original.composition_is_declared()
            || original.profile.intent != SolveIntent::Root
            || !matches!(original.route, Route::Native(_))
            || !matches!(original.profile.selection, SolverSelection::Explicit(_))
        {
            return Err(ProblemError::Unsupported(
                "multistart needs an uncomposed actual explicit original root profile".into(),
            )
            .into());
        }
        let backend = match original.route {
            Route::Native(backend)
                if original.profile.selection == SolverSelection::Explicit(backend) =>
            {
                backend
            }
            _ => {
                return Err(ProblemError::Contract(
                    "multistart explicit backend differs from its actual original route".into(),
                )
                .into());
            }
        };
        let Representation::Algebraic(source) = &original.representation else {
            return Err(ProblemError::Unsupported(
                "multistart requires complete compiled original source callbacks".into(),
            )
            .into());
        };
        if source.case.is_none() {
            return Err(ProblemError::Unsupported(
                "multistart original compiled evaluator is not admitted".into(),
            )
            .into());
        }
        let contract = native::assembled::contract(&source.prepared.compiled().plan);
        let coordinates = original.original_coordinates()?;
        if seed.len() != coordinates.len()
            || coordinates.is_empty()
            || contract.variables.iter().any(|v| {
                seed.get(&v.id)
                    .is_none_or(|value| !value.is_finite() || *value < v.lower || *value > v.upper)
            })
        {
            return Err(ProblemError::Contract("multistart specification must cover all original coordinates with finite authored-bound values".into()).into());
        }
        let values = coordinates.iter().map(|id| seed[id]).collect::<Vec<_>>();
        let point_key = original.semantic_point_key(&values)?;
        let proposal = Proposal::modified_specification(
            coordinates,
            values,
            point_key,
            original.original_identity()?,
            branch,
        )?;
        let mut profile = original.profile.clone();
        profile.controls.start = StartPolicy::Explicit;
        let mut key = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        key.str("fresh-original-modified-specification")
            .hash(&original.original_identity()?)
            .hash(&original.preparation_identity()?)
            .hash(&profile_key(&profile)?.as_id());
        for (id, value) in proposal.values() {
            key.id(&id).f64(value);
        }
        let key = key.finish_hash();
        let bytes = derived::binding_metadata_bytes(source, &profile)?
            .checked_add(proposal.retained_bytes()?)
            .and_then(|n| {
                n.checked_add(
                    size_of::<Data>() + size_of::<PreparedMultistart>() + 6 * size_of::<usize>(),
                )
            })
            .ok_or(MathRuntimeError::Limit("multistart prepared metadata"))?;
        let owner = self.reserve("math:multistart-specification", bytes)?;
        original = original.within_task(scope.clone())?;
        scope.check().map_err(ProblemError::from)?;
        Ok(PreparedMultistart {
            data: Arc::new(Data {
                original: Arc::new(original),
                profile,
                backend,
                proposal,
                scope,
                key,
                _owner: owner,
            }),
        })
    }
    /// Screen the same full original supplier on an already admitted worker, then bind
    /// only the physical primal seed. No retry, native call or scientific assessment is
    /// performed here; the shared driver owns those operations.
    #[cfg(all(test, feature = "solver-kinsol"))]
    pub(crate) fn screen_multistart_worker(
        &self,
        prepared: &PreparedMultistart,
        scope: ExecutionScope,
        budget: &Arc<WorkerBudget>,
    ) -> Result<BoundMultistart, MathRuntimeError> {
        self.screen_multistart_worker_observed(prepared, scope, budget, &mut 0)
    }
    /// Count actual original objective/constraint invocations, including those that
    /// return a cause before a native step exists. The caller retains this observation
    /// on failure; successful bindings record only the work of this screening call.
    pub(crate) fn screen_multistart_worker_observed(
        &self,
        prepared: &PreparedMultistart,
        scope: ExecutionScope,
        budget: &Arc<WorkerBudget>,
        evaluations: &mut u64,
    ) -> Result<BoundMultistart, MathRuntimeError> {
        let before = *evaluations;
        scope.check().map_err(ProblemError::from)?;
        if !Arc::ptr_eq(scope.cancellation(), prepared.scope().cancellation())
            || scope.deadline().is_none()
            || scope.deadline() > prepared.scope().deadline()
        {
            return Err(ProblemError::Contract(
                "multistart screening changed the original task owner or renewed its deadline"
                    .into(),
            )
            .into());
        }
        let original = prepared.original_target();
        let Representation::Algebraic(source) = &original.representation else {
            return Err(ProblemError::Internal(
                "admitted multistart lost its original compiled source".into(),
            )
            .into());
        };
        // One map belongs to the original screening oracle; another belongs to the
        // cloned native step. Its providers/profile/normalization remain charged until
        // the common driver's native handles and callbacks have ended.
        let bytes = derived::binding_metadata_bytes(source, prepared.profile())?
            .checked_add(derived::case_values_bytes(source.values.scalars.len())?)
            .and_then(|n| n.checked_add(size_of::<PreparedSolve>()))
            .and_then(|n| {
                n.checked_add(
                    (original.normalization.variables.capacity()
                        + original.normalization.rows.capacity())
                    .checked_mul(size_of::<f64>())?,
                )
            })
            .ok_or(MathRuntimeError::Limit("multistart source binding extent"))?;
        let bindings = budget.charge(bytes)?;
        let screened = self.screen_start_worker_observed(
            original,
            prepared.data.proposal.clone(),
            prepared.branch(),
            scope.clone(),
            budget,
            evaluations,
        )?;
        let step = original.clone().with_screened_start(&screened)?;
        if step.strategy_profile()? != prepared.strategy_profile()?
            || step.original_identity()? != prepared.original_identity()?
        {
            return Err(ProblemError::Contract(
                "multistart native binding changed its admitted profile or original target".into(),
            )
            .into());
        }
        scope.check().map_err(ProblemError::from)?;
        Ok(BoundMultistart {
            step,
            _bindings: bindings,
            screening_evaluations: *evaluations - before,
        })
    }
}
