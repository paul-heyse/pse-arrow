// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite owner-affine experiment lanes; the existing runtime owns active admission.
use super::*;
use super::oracle::{Demand, ExperimentProducts, evaluate_experiment};
use native::{ProblemError, solve::Execution};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::mpsc::{self, SyncSender},
    thread::JoinHandle,
    time::Duration,
};

thread_local! {
    static BORROWED_CAPACITY: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}
struct RestoreCapacity(Option<usize>);
impl Drop for RestoreCapacity {
    fn drop(&mut self) { BORROWED_CAPACITY.with(|owner| owner.set(self.0)); }
}
/// The profile owner calls this only inside its admitted quiescent team. Each
/// profile lane owns this many active cores, including its nested experiment work.
pub(super) fn borrowed_capacity<T>(cores: usize, work: impl FnOnce() -> T) -> T {
    let restore = RestoreCapacity(BORROWED_CAPACITY.with(|owner| owner.replace(Some(cores))));
    let result = work();
    drop(restore);
    result
}
struct Request {
    x: Arc<[f64]>,
    demand: Demand,
    experiments: Vec<(usize, Option<Arc<native::dynamics::Report>>)>,
    reply: SyncSender<(usize, Result<ExperimentProducts, ProblemError>)>,
    #[cfg(test)]
    gate: Option<Arc<std::sync::Barrier>>,
}
struct Lane {
    request: Option<SyncSender<Request>>,
    joined: Option<JoinHandle<()>>,
    #[cfg(test)]
    constructions: Arc<std::sync::atomic::AtomicUsize>,
}
impl Drop for Lane {
    fn drop(&mut self) {
        self.request.take();
        if let Some(joined) = self.joined.take() { let _ = joined.join(); }
    }
}
/// Idle threads keep their original evaluator/provider owners and charged resident
/// memory. They receive at most one bounded wave while the parent lends active compute.
pub(super) struct ExperimentTeam {
    problem: Arc<FitProblem>,
    execution: Execution,
    lanes: Vec<Lane>,
    borrowed: Option<usize>,
    _owner: Arc<pse_columnar::AllocationLease>,
    #[cfg(test)]
    wave_gate: Option<Arc<std::sync::Barrier>>,
}
impl std::fmt::Debug for ExperimentTeam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExperimentTeam").field("lanes", &self.lanes.len()).finish_non_exhaustive()
    }
}
impl ExperimentTeam {
    pub(super) fn new(problem: Arc<FitProblem>, execution: Execution) -> Result<Option<Self>, ProblemError> {
        let requested = problem.profile.experiment_workers.into_inner();
        if requested <= 1 || problem.experiments.len() <= 1 { return Ok(None); }
        let service = problem.runtime.native().clone();
        let count = requested.min(problem.experiments.len()).min(service.cores());
        if count <= 1 { return Ok(None); }
        let borrowed = BORROWED_CAPACITY.with(std::cell::Cell::get);
        if borrowed.is_some_and(|cores| count > cores) {
            return Err(ProblemError::memory("fit experiment exceeds its admitted profile envelope"));
        }
        let stack = service.stack_bytes();
        // Parent fit state remains charged. Additional lane scratch and stacks remain
        // charged even while active compute is lent and every lane is waiting.
        let bytes = stack.checked_mul(count).and_then(|s| s.checked_add(problem.bytes))
            .ok_or_else(|| ProblemError::memory("fit experiment resident extent"))?;
        let owner = service.reserve("fit:experiment-lanes", bytes).map_err(|e| e.into_problem())?;
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|e| ProblemError::internal(format!("fit experiment runtime: {e}")))?;
        let constructor = pse_math::construction::current();
        let launch = || {
            let mut lanes = Vec::with_capacity(count);
            for lane in 0..count {
                let (tx, rx) = mpsc::sync_channel::<Request>(1);
                let problem = problem.clone();
                let execution = execution.clone();
                let runtime = runtime.clone();
                let constructor = constructor.clone();
                #[cfg(test)]
                let constructions = Arc::new(std::sync::atomic::AtomicUsize::new(0));
                #[cfg(test)]
                let observed_constructions = constructions.clone();
                let joined = std::thread::Builder::new().name(format!("pse-fit-{lane}"))
                    .stack_size(stack).spawn(move || {
                        let _runtime = runtime.enter();
                        let mut steady: Vec<Option<pse_math::assembly::CaseWorker>> =
                            (0..problem.experiments.len()).map(|_| None).collect();
                        #[cfg(feature = "solver-diffsol")]
                        let mut transient: Vec<Option<crate::workflow::dynamics::DynamicWorker>> =
                            (0..problem.experiments.len()).map(|_| None).collect();
                        while let Ok(request) = rx.recv() {
                            #[cfg(test)]
                            if let Some(gate) = &request.gate { let _ = gate.wait(); }
                            for (ei, previous) in request.experiments {
                                #[cfg(test)]
                                let constructing = match &problem.experiments[ei] {
                                    Experiment::Steady(_) => steady[ei].is_none(),
                                    #[cfg(feature = "solver-diffsol")]
                                    Experiment::Transient(_) => transient[ei].is_none(),
                                    #[cfg(not(feature = "solver-diffsol"))]
                                    Experiment::Transient(_) => false,
                                };
                                let mut work = || evaluate_experiment(&problem, ei, &request.x,
                                    request.demand, &execution, &mut steady[ei],
                                    #[cfg(feature = "solver-diffsol")]
                                    &mut transient[ei], previous.as_deref());
                                let result = catch_unwind(AssertUnwindSafe(||
                                    native::execution::original_scoped(&execution, || {
                                        match &constructor {
                                            Some(owner) => pse_math::construction::scoped(owner.clone(), work),
                                            None => work(),
                                        }
                                    })))
                                    .unwrap_or_else(|_| Err(ProblemError::internal("fit experiment lane panicked")));
                                #[cfg(test)]
                                if constructing && result.is_ok() {
                                    observed_constructions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                }
                                if result.is_err() {
                                    steady[ei] = None;
                                    #[cfg(feature = "solver-diffsol")]
                                    { transient[ei] = None; }
                                }
                                if request.reply.send((ei, result)).is_err() { break; }
                            }
                        }
                        // All native sessions have already returned; compiled/provider scratch
                        // is destroyed here on its actual owning lane before join completes.
                    }).map_err(|e| ProblemError::internal(format!("fit experiment lane creation: {e}")))?;
                lanes.push(Lane { request: Some(tx), joined: Some(joined),
                    #[cfg(test)] constructions });
            }
            Ok::<_, ProblemError>(lanes)
        };
        let lanes = match borrowed {
            Some(_) => launch()?,
            None => service.quiescent_phase(count, 0, &execution, launch).map_err(|e| e.into_problem())??,
        };
        Ok(Some(Self { problem, execution, lanes, borrowed, _owner: owner,
            #[cfg(test)] wave_gate: None }))
    }
    #[cfg(test)]
    pub(super) fn require_concurrent_test_wave(&mut self) {
        self.wave_gate = Some(Arc::new(std::sync::Barrier::new(self.lanes.len())));
    }
    #[cfg(test)]
    pub(super) fn test_constructions(&self) -> Vec<usize> {
        self.lanes.iter().map(|lane| lane.constructions.load(std::sync::atomic::Ordering::Relaxed)).collect()
    }
    pub(super) fn evaluate(&mut self, x: &[f64], demand: Demand,
        previous: Option<&BTreeMap<InstanceId, Arc<native::dynamics::Report>>>)
        -> Result<Vec<ExperimentProducts>, ProblemError> {
        let service = self.problem.runtime.native().clone();
        let count = self.problem.experiments.len();
        let x: Arc<[f64]> = Arc::from(x);
        let work = || {
            self.execution.check()?;
            let (reply, replies) = mpsc::sync_channel(count);
            let mut results: Vec<Option<Result<ExperimentProducts, ProblemError>>> =
                (0..count).map(|_| None).collect();
            for (lane, owner) in self.lanes.iter().enumerate() {
                let experiments = (lane..count).step_by(self.lanes.len()).map(|ei| {
                    let prior = previous.and_then(|p| p.get(&self.problem.declaration.experiments[ei].experiment_id)).cloned();
                    (ei, prior)
                }).collect::<Vec<_>>();
                let request = Request { x: x.clone(), demand, experiments, reply: reply.clone(),
                    #[cfg(test)] gate: self.wave_gate.clone() };
                if owner.request.as_ref().is_none_or(|sender| sender.send(request).is_err()) {
                    for ei in (lane..count).step_by(self.lanes.len()) {
                        results[ei] = Some(Err(ProblemError::internal("fit experiment lane stopped before dispatch")));
                    }
                }
            }
            drop(reply);
            let mut stop = None;
            loop {
                match replies.recv_timeout(Duration::from_millis(10)) {
                    Ok((ei, value)) => results[ei] = Some(value),
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if stop.is_none() { stop = self.execution.check().err(); }
                    }
                }
            }
            if let Some(error) = stop { return Err(error); }
            // Every dispatched request has returned or disconnected. Never replay a
            // started experiment following a panic, cancellation or lane failure.
            results.into_iter().map(|value| value.unwrap_or_else(||
                Err(ProblemError::internal("fit experiment lane returned no result")))).collect()
        };
        match self.borrowed {
            Some(_) => work(),
            None => service.quiescent_phase(self.lanes.len(), 0, &self.execution, work).map_err(|e| e.into_problem())?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn borrowed_profile_envelope_restores_after_nested_scope_and_panic() {
        assert_eq!(BORROWED_CAPACITY.with(std::cell::Cell::get), None);
        borrowed_capacity(4, || {
            assert_eq!(BORROWED_CAPACITY.with(std::cell::Cell::get), Some(4));
            let failed = catch_unwind(|| borrowed_capacity(2, || panic!("injected phase panic")));
            assert!(failed.is_err());
            assert_eq!(BORROWED_CAPACITY.with(std::cell::Cell::get), Some(4));
        });
        assert_eq!(BORROWED_CAPACITY.with(std::cell::Cell::get), None);
    }
}
