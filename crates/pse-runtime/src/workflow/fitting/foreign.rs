// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual disjoint foreign envelopes for this oracle's fixed transient owners.
use super::*;
use native::{
    ProblemError,
    solve::{Execution, NativeStorageScope, WorkAdmission, WorkEvidence},
};
use std::sync::{Mutex, Weak};

#[derive(Clone)]
struct Envelope {
    extent: usize,
    _token: Arc<dyn std::any::Any + Send + Sync>,
}
impl std::fmt::Debug for Envelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Envelope")
            .field("extent", &self.extent)
            .finish_non_exhaustive()
    }
}
#[derive(Debug)]
struct Coverage {
    parent: Option<Arc<dyn WorkAdmission>>,
    lease: Option<Envelope>,
    retain: bool,
    claimant: Mutex<Weak<()>>,
    // Keeps the original task's exclusive claim while its detached lease is in use.
    _original: Arc<()>,
}
impl WorkAdmission for Coverage {
    fn prepaid_storage(
        &self,
        identity: &Arc<()>,
        bytes: usize,
    ) -> Result<Option<Arc<dyn std::any::Any + Send + Sync>>, ProblemError> {
        let mut claimant = self
            .claimant
            .lock()
            .map_err(|_| ProblemError::internal("fit native coverage poisoned"))?;
        if let Some(current) = claimant.upgrade()
            && !Arc::ptr_eq(&current, identity)
        {
            return Err(ProblemError::memory(
                "fit foreign envelope already has a resident native owner",
            ));
        }
        if !self.retain {
            return Ok(None);
        }
        let Some(lease) = &self.lease else {
            return Ok(None);
        };
        if bytes == 0 || bytes > lease.extent {
            return Ok(None);
        }
        *claimant = Arc::downgrade(identity);
        // Passive coverage follows the native allocation independently of the
        // current delegating Execution, without retaining its parent work ledger.
        Ok(Some(Arc::new((lease.clone(), self._original.clone()))))
    }
    fn admit(&self, work: WorkEvidence) -> Result<(), ProblemError> {
        self.parent
            .as_ref()
            .map_or(Ok(()), |parent| parent.admit(work))
    }
    fn observe(&self, work: WorkEvidence) -> Result<(), ProblemError> {
        self.parent
            .as_ref()
            .map_or(Ok(()), |parent| parent.observe(work))
    }
    fn admit_storage(
        &self,
        scope: NativeStorageScope,
        owner: &str,
        bytes: usize,
        opaque: bool,
    ) -> Result<(), ProblemError> {
        if scope == NativeStorageScope::Linear
            && let Some(lease) = &self.lease
        {
            // Preserve strict unknown-bound refusal without charging this separate
            // envelope a second time to the original task's cumulative storage.
            self.parent.as_ref().map_or(Ok(()), |parent| {
                parent.admit_storage(scope, owner, 0, opaque)
            })?;
            if bytes > lease.extent {
                return Err(ProblemError::memory(
                    "fit native extent exceeds its actual foreign envelope",
                ));
            }
            return Ok(());
        }
        self.parent.as_ref().map_or(Ok(()), |parent| {
            parent.admit_storage(scope, owner, bytes, opaque)
        })
    }
}
fn execution_for(
    execution: &Execution,
    lease: Option<Envelope>,
    retain: bool,
    original: &Arc<()>,
) -> Execution {
    let mut current = execution.clone();
    current.work_admission = Some(Arc::new(Coverage {
        parent: execution.work_admission.clone(),
        lease,
        retain,
        claimant: Mutex::new(Weak::new()),
        _original: original.clone(),
    }));
    current
}
/// One root for each actual simultaneous profile owner. Each root may issue its
/// opaque passive token to a nested experiment population without type guessing.
pub(super) fn chain_metadata_bytes() -> Option<usize> {
    size_of::<Coverage>()
        .checked_add(size_of::<Envelope>())?
        .checked_add(size_of::<Execution>())?
        .checked_add(128)
}
pub(super) fn chains(
    problem: &FitProblem,
    execution: &Execution,
    workers: usize,
) -> Result<Option<Vec<Execution>>, ProblemError> {
    execution.check()?;
    if !problem.experiments.iter().any(native_experiment) {
        return Ok(Some(vec![execution.clone(); workers]));
    }
    let foreign = execution.memory.unwrap_or_else(|| {
        problem
            .runtime
            .shared
            .budget()
            .math
            .foreign_allowance(&problem.profile.solver.controls)
    });
    let original = Arc::new(());
    let first = execution
        .work_admission
        .as_ref()
        .map_or(Ok(None), |owner| owner.prepaid_storage(&original, foreign))?
        .map(|token| Envelope {
            extent: foreign,
            _token: token,
        });
    let mut leases = vec![first];
    for _ in 1..workers {
        match problem
            .runtime
            .native()
            .reserve("fit:profile-chain-foreign", foreign)
        {
            Ok(lease) => leases.push(Some(Envelope {
                extent: foreign,
                _token: lease,
            })),
            Err(error) if capacity(&error) => return Ok(None),
            Err(error) => return Err(error.into_problem()),
        }
    }
    execution.check()?;
    Ok(Some(
        leases
            .into_iter()
            .map(|lease| {
                let retain = lease.is_some();
                execution_for(execution, lease, retain, &original)
            })
            .collect(),
    ))
}
#[cfg(all(test, feature = "solver-idas"))]
pub(super) fn actual_lease(
    token: &Arc<dyn std::any::Any + Send + Sync>,
) -> Option<Arc<pse_columnar::AllocationLease>> {
    if let Ok(lease) = Arc::downcast::<pse_columnar::AllocationLease>(token.clone()) {
        return Some(lease);
    }
    let held = token.downcast_ref::<(Envelope, Arc<()>)>()?;
    actual_lease(&held.0._token)
}
fn native_experiment(experiment: &Experiment) -> bool {
    matches!(experiment, Experiment::Transient(source) if source.profile.method == native::dynamics::Method::Idas)
}
pub(super) fn metadata_bytes(experiments: usize) -> Option<usize> {
    let executions = size_of::<Execution>().checked_mul(3)?;
    let owners = size_of::<Coverage>().checked_add(128)?.checked_mul(2)?;
    experiments.checked_mul(executions.checked_add(owners)?)
}
pub(super) fn capacity(error: &crate::math::MathRuntimeError) -> bool {
    matches!(
        error,
        crate::math::MathRuntimeError::Pool(
            datafusion::common::DataFusionError::ResourcesExhausted(_)
        ) | crate::math::MathRuntimeError::Limit(_)
    )
}
#[derive(Debug)]
pub(super) struct Population {
    pub(super) parent: Vec<Execution>,
    pub(super) child: Vec<Execution>,
}
impl Population {
    /// None means the wider active footprint did not fit before any source effect;
    /// the caller retries a narrower assignment. Optional retention never refuses.
    pub(super) fn prepare(
        problem: &FitProblem,
        execution: &Execution,
        assignments: Option<&[Vec<usize>]>,
    ) -> Result<Option<Self>, ProblemError> {
        execution.check()?;
        let service = problem.runtime.native();
        let foreign = execution.memory.unwrap_or_else(|| {
            problem
                .runtime
                .shared
                .budget()
                .math
                .foreign_allowance(&problem.profile.solver.controls)
        });
        let mut parent = vec![execution.clone(); problem.experiments.len()];
        let mut child = parent.clone();
        let parent_owned =
            assignments.is_none() || problem.profile.solver.controls.hessian == HessianMode::Exact;
        let parents: Vec<_> = problem
            .experiments
            .iter()
            .enumerate()
            .filter_map(|(i, source)| (parent_owned && native_experiment(source)).then_some(i))
            .collect();
        let children: Vec<_> = assignments
            .into_iter()
            .flatten()
            .flat_map(|lane| lane.iter().copied())
            .filter(|&i| native_experiment(&problem.experiments[i]))
            .collect();
        let active: Vec<_> = assignments
            .into_iter()
            .flatten()
            .enumerate()
            .filter_map(|(lane, assigned)| {
                assigned
                    .iter()
                    .any(|&i| native_experiment(&problem.experiments[i]))
                    .then_some(lane)
            })
            .collect();
        let slots = parents.len() + children.len();
        if slots == 0 {
            return Ok(Some(Self { parent, child }));
        }
        let active_count = active.len().max(1);
        let original = Arc::new(());
        let first = execution
            .work_admission
            .as_ref()
            .map_or(Ok(None), |owner| owner.prepaid_storage(&original, foreign))?
            .map(|token| Envelope {
                extent: foreign,
                _token: token,
            });
        let mut leases = vec![first];
        // Extra simultaneously active lanes are required even when every session
        // is fresh. One job allowance cannot cover concurrent native populations.
        for _ in 1..active_count {
            match service.reserve("fit:active-transient-foreign", foreign) {
                Ok(lease) => leases.push(Some(Envelope {
                    extent: foreign,
                    _token: lease,
                })),
                Err(error) if capacity(&error) => return Ok(None),
                Err(error) => return Err(error.into_problem()),
            }
        }
        let mut retained = execution.memory.is_some() && leases[0].is_some();
        let mut optional = Vec::new();
        if retained {
            for _ in active_count..slots {
                match service.reserve("fit:resident-transient-foreign", foreign) {
                    Ok(lease) => optional.push(Some(Envelope {
                        extent: foreign,
                        _token: lease,
                    })),
                    Err(error) if capacity(&error) => {
                        retained = false;
                        break;
                    }
                    Err(error) => return Err(error.into_problem()),
                }
            }
        }
        if retained {
            leases.extend(optional);
        } else {
            drop(optional);
        }
        execution.check()?;
        let bind = |lease: Option<Envelope>| execution_for(execution, lease, retained, &original);
        if retained {
            let mut leases = leases.into_iter();
            for index in parents {
                parent[index] = bind(leases.next().flatten());
            }
            for index in children {
                child[index] = bind(leases.next().flatten());
            }
        } else {
            // No native allocation survives a return in this branch. Parent exact
            // work runs after the child wave, and each lane's assigned work is serial.
            let first = bind(leases[0].clone());
            for index in parents {
                parent[index] = first.clone();
            }
            if let Some(assignments) = assignments {
                for (ordinal, lane) in active.into_iter().enumerate() {
                    let current = bind(leases[ordinal].clone());
                    for &index in &assignments[lane] {
                        if native_experiment(&problem.experiments[index]) {
                            child[index] = current.clone();
                        }
                    }
                }
            }
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count(
                if retained {
                    "fit.native_resident_population"
                } else {
                    "fit.native_fresh_population"
                },
                "count",
                slots as u64,
            );
        }
        Ok(Some(Self { parent, child }))
    }
}
