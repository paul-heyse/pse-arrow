// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite owner-affine experiment lanes; the existing runtime owns active admission.
use super::oracle::{
    Demand, ExperimentProducts, ExperimentWorkers, UpgradeValidity, evaluate_experiment,
};
use super::*;
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
    fn drop(&mut self) {
        BORROWED_CAPACITY.with(|owner| owner.set(self.0));
    }
}
/// The profile owner calls this only inside its admitted quiescent team. Each
/// profile lane owns this many active cores, including its nested experiment work.
pub(super) fn borrowed_capacity<T>(cores: usize, work: impl FnOnce() -> T) -> T {
    let restore = RestoreCapacity(BORROWED_CAPACITY.with(|owner| owner.replace(Some(cores))));
    let result = work();
    drop(restore);
    result
}
/// Static ownership is fixed before constructing any experiment worker. Requests
/// carry only per-trial prior trajectories, in this lane's immutable inventory order.
fn stride_assignment(experiments: usize, lanes: usize) -> Vec<Vec<usize>> {
    (0..lanes).map(|lane| (lane..experiments).step_by(lanes).collect()).collect()
}
/// Compare the existing stride with stable largest-resident-first affinity. These
/// are source-issued worker bounds, not a prediction of native execution time.
/// Unknown extents and equal peaks retain the simpler authored stride.
pub(super) fn resident_assignment(weights: &[Option<usize>], lanes: usize) -> Vec<Vec<usize>> {
    let stride = stride_assignment(weights.len(), lanes);
    let Some(weights) = weights.iter().copied().collect::<Option<Vec<_>>>() else { return stride; };
    let peak = |assignment: &[Vec<usize>]| assignment.iter().map(|lane| lane.iter().map(|&i| weights[i] as u128).sum::<u128>()).max().unwrap_or(0);
    let mut order = (0..weights.len()).collect::<Vec<_>>();
    order.sort_by_key(|&i| (std::cmp::Reverse(weights[i]), i));
    let mut balanced = vec![Vec::new(); lanes];
    let mut totals = vec![0u128; lanes];
    for index in order {
        let lane = (0..lanes).min_by_key(|&lane| (totals[lane], balanced[lane].len(), lane)).unwrap_or(0);
        balanced[lane].push(index); totals[lane] += weights[index] as u128;
    }
    for lane in &mut balanced { lane.sort_unstable(); }
    if peak(&balanced) < peak(&stride) { balanced } else { stride }
}
fn resident_weights(problem: &FitProblem) -> Result<Vec<Option<usize>>, ProblemError> {
    problem.experiments.iter().map(|experiment| match experiment {
        Experiment::Steady(s) => Ok(s.providers.is_empty().then_some(s.case.assembly.numeric_worker_bytes())),
        Experiment::Transient(s) => {
            let allowances = crate::workflow::dynamics::allowance_bytes(&s.program.state_allowances, &s.program.output_allowances)
                .ok_or_else(|| ProblemError::memory("fit experiment allowance extent"))?;
            crate::workflow::dynamics::DynamicProgram::worker_allocation_bound(&s.program.contract,
                &s.program.programs, &s.program.coordinates, s.program.modes.iter(), &s.program.layout, allowances)
        }
    }).collect()
}
struct Request {
    observation: Option<pse_diagnostics::observation::ObservationContext>,
    x: Arc<[f64]>,
    demand: Demand,
    validity: UpgradeValidity,
    previous: Vec<Option<Arc<native::dynamics::Report>>>,
    reply: SyncSender<(usize, Result<ExperimentProducts, ProblemError>)>,
    #[cfg(all(test, feature = "solver-diffsol"))]
    gate: Option<Arc<std::sync::Barrier>>,
}
struct Lane {
    experiments: Vec<usize>,
    request: Option<SyncSender<Request>>,
    joined: Option<JoinHandle<()>>,
    #[cfg(all(test, feature = "solver-diffsol"))]
    constructions: Arc<std::sync::atomic::AtomicUsize>,
}
impl Drop for Lane {
    fn drop(&mut self) {
        self.request.take();
        if let Some(joined) = self.joined.take() {
            let _ = joined.join();
        }
    }
}
// Keep resident owners alive until every already-started lane has drained.
fn drain_failed_startup(
    peers: &mut Vec<Lane>,
    request: SyncSender<Request>,
    joined: JoinHandle<()>,
) {
    drop(request);
    let _ = joined.join();
    peers.clear();
}
/// Idle threads keep their original evaluator/provider owners and charged resident
/// memory. They receive at most one bounded wave while the parent lends active compute.
pub(super) struct ExperimentTeam {
    problem: Arc<FitProblem>,
    execution: Execution,
    lanes: Vec<Lane>,
    borrowed: Option<usize>,
    _owner: Arc<pse_columnar::AllocationLease>,
    #[cfg(all(test, feature = "solver-diffsol"))]
    wave_gate: Option<Arc<std::sync::Barrier>>,
}
impl std::fmt::Debug for ExperimentTeam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExperimentTeam")
            .field("lanes", &self.lanes.len())
            .finish_non_exhaustive()
    }
}
impl ExperimentTeam {
    pub(super) fn new(
        problem: Arc<FitProblem>,
        execution: Execution,
    ) -> Result<Option<Self>, ProblemError> {
        let requested = problem.profile.experiment_workers.into_inner();
        if requested <= 1 || problem.experiments.len() <= 1 {
            return Ok(None);
        }
        let service = problem.runtime.native().clone();
        let count = requested
            .min(problem.experiments.len())
            .min(service.cores());
        if count <= 1 {
            return Ok(None);
        }
        let assignments = resident_assignment(&resident_weights(&problem)?, count);
        Self::construct(problem, execution, assignments).map(Some)
    }
    #[cfg(all(test, feature = "solver-diffsol"))]
    pub(super) fn stride_for_test(problem: Arc<FitProblem>, execution: Execution) -> Result<Self, ProblemError> {
        let count = problem.profile.experiment_workers.into_inner().min(problem.experiments.len()).min(problem.runtime.native().cores());
        let assignments = stride_assignment(problem.experiments.len(), count);
        Self::construct(problem, execution, assignments)
    }
    fn construct(problem: Arc<FitProblem>, execution: Execution, assignments: Vec<Vec<usize>>) -> Result<Self, ProblemError> {
        let count = assignments.len();
        let service = problem.runtime.native().clone();
        let borrowed = BORROWED_CAPACITY.with(std::cell::Cell::get);
        if borrowed.is_some_and(|cores| count > cores) {
            return Err(ProblemError::memory(
                "fit experiment exceeds its admitted profile envelope",
            ));
        }
        let stack = service.stack_bytes();
        // Parent fit state remains charged. Additional lane scratch and stacks remain
        // charged even while active compute is lent and every lane is waiting.
        let bytes = stack
            .checked_mul(count)
            .and_then(|s| s.checked_add(problem.bytes))
            .ok_or_else(|| ProblemError::memory("fit experiment resident extent"))?;
        let owner = service
            .reserve("fit:experiment-lanes", bytes)
            .map_err(|e| e.into_problem())?;
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|e| ProblemError::internal(format!("fit experiment runtime: {e}")))?;
        let constructor = pse_math::construction::current();
        let launch = || {
            let mut lanes = Vec::with_capacity(count);
            for (lane, assigned) in assignments.iter().enumerate() {
                let experiments = assigned.clone();
                let assigned = assigned.clone();
                let (tx, rx) = mpsc::sync_channel::<Request>(1);
                let (ready, preparation) = mpsc::sync_channel::<Result<(), ProblemError>>(1);
                let problem = problem.clone();
                let execution = execution.clone();
                let runtime = runtime.clone();
                let constructor = constructor.clone();
                #[cfg(all(test, feature = "solver-diffsol"))]
                let constructions = Arc::new(std::sync::atomic::AtomicUsize::new(0));
                #[cfg(all(test, feature = "solver-diffsol"))]
                let observed_constructions = constructions.clone();
                let joined = std::thread::Builder::new()
                    .name(format!("pse-fit-{lane}"))
                    .stack_size(stack)
                    .spawn(move || {
                        let _runtime = runtime.enter();
                        let mut steady: Vec<Option<pse_math::assembly::CaseWorker>> =
                            (0..assigned.len()).map(|_| None).collect();
                        #[cfg(feature = "solver-diffsol")]
                        let mut transient: Vec<
                            Option<crate::workflow::dynamics::DynamicWorker>,
                        > = (0..assigned.len()).map(|_| None).collect();
                        // Prepare only this lane's finite assigned experiment set on its
                        // actual owner, before the enclosing native fit can attach callbacks.
                        let mut prepare = || -> Result<(), ProblemError> {
                            for (local, &ei) in assigned.iter().enumerate() {
                                match &problem.experiments[ei] {
                                    Experiment::Steady(s) => {
                                        steady[local] =
                                            Some(oracle::prepare_steady_worker(s, &execution)?)
                                    }
                                    #[cfg(feature = "solver-diffsol")]
                                    Experiment::Transient(s) => {
                                        transient[local] = Some(s.worker(&execution)?)
                                    }
                                    #[cfg(not(feature = "solver-diffsol"))]
                                    Experiment::Transient(_) => {}
                                }
                                #[cfg(all(test, feature = "solver-diffsol"))]
                                observed_constructions
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                            Ok(())
                        };
                        let prepared = catch_unwind(AssertUnwindSafe(|| {
                            native::execution::original_scoped(&execution, || match &constructor {
                                Some(owner) => {
                                    pse_math::construction::scoped(owner.clone(), prepare)
                                }
                                None => prepare(),
                            })
                        }))
                        .unwrap_or_else(|_| {
                            Err(ProblemError::internal(
                                "fit experiment lane preparation panicked",
                            ))
                        });
                        let succeeded = prepared.is_ok();
                        if ready.send(prepared).is_err() || !succeeded {
                            return;
                        }
                        while let Ok(request) = rx.recv() {
                            // A retained lane receives the context of each actual wave,
                            // rather than retaining the operation that created the team.
                            let operation = request.observation.as_ref().map(|parent| {
                                parent.count("fit.experiment_lane_waves", "count", 1);
                                parent.child(
                                    "fit.experiment_lane_wave",
                                    pse_diagnostics::observation::Relationship::Nested,
                                )
                            });
                            let context = operation.as_ref().map(|operation| operation.context());
                            let _entered = context.as_ref().map(|context| context.enter());
                            if let Some(context) = &context {
                                if let Ok(index) = i64::try_from(lane) {
                                    context.integer("fit.experiment_lane", "index", index);
                                } else {
                                    context.unavailable(
                                        "fit.experiment_lane",
                                        "lane exceeds signed diagnostic representation",
                                    );
                                }
                            }
                            if request.previous.len() != assigned.len() {
                                for &ei in &assigned {
                                    let _ = request.reply.send((ei, Err(ProblemError::internal("fit experiment request inventory"))));
                                }
                                continue;
                            }
                            let mut replies_open = true;
                            #[cfg(all(test, feature = "solver-diffsol"))]
                            if let Some(gate) = &request.gate {
                                let _ = gate.wait();
                            }
                            for (local, (&ei, previous)) in assigned.iter().zip(request.previous).enumerate() {
                                let mut work = || {
                                    evaluate_experiment(
                                        &problem,
                                        ei,
                                        &request.x,
                                        request.demand,
                                        &execution,
                                        ExperimentWorkers {
                                            steady: &mut steady[local],
                                            #[cfg(feature = "solver-diffsol")]
                                            transient: &mut transient[local],
                                        },
                                        previous.as_deref(),
                                        &request.validity,
                                    )
                                };
                                let result = catch_unwind(AssertUnwindSafe(|| {
                                    native::execution::original_scoped(&execution, || {
                                        match &constructor {
                                            Some(owner) => {
                                                pse_math::construction::scoped(owner.clone(), work)
                                            }
                                            None => work(),
                                        }
                                    })
                                }))
                                .unwrap_or_else(|_| {
                                    Err(ProblemError::internal("fit experiment lane panicked"))
                                });
                                if let Err(cause) = &result {
                                    if native::callback::classify(cause)
                                        == native::callback::Failure::Trial
                                    {
                                        if let Some(worker) = &mut steady[local] {
                                            worker.invalidate_trial();
                                        }
                                        #[cfg(feature = "solver-diffsol")]
                                        if let Some(worker) = &mut transient[local] {
                                            worker.invalidate_trial();
                                        }
                                    } else {
                                        // Unknown/terminal native validity is not reset by inference.
                                        // A later callback refuses this owner and requires a fresh fit attempt.
                                        steady[local] = None;
                                        #[cfg(feature = "solver-diffsol")]
                                        {
                                            transient[local] = None;
                                        }
                                    }
                                }
                                if request.reply.send((ei, result)).is_err() {
                                    replies_open = false;
                                    break;
                                }
                            }
                            if let Some(operation) = operation {
                                operation.finish(if replies_open {
                                    pse_diagnostics::observation::Outcome::Completed
                                } else {
                                    pse_diagnostics::observation::Outcome::Interrupted
                                });
                            }
                        }
                        // All native sessions have already returned; compiled/provider scratch
                        // is destroyed here on its actual owning lane before join completes.
                    })
                    .map_err(|e| {
                        ProblemError::internal(format!("fit experiment lane creation: {e}"))
                    })?;
                let prepared = preparation.recv().unwrap_or_else(|_| {
                    Err(ProblemError::internal(
                        "fit experiment lane stopped during preparation",
                    ))
                });
                if let Err(cause) = prepared {
                    drain_failed_startup(&mut lanes, tx, joined);
                    return Err(cause);
                }
                lanes.push(Lane {
                    experiments,
                    request: Some(tx),
                    joined: Some(joined),
                    #[cfg(all(test, feature = "solver-diffsol"))]
                    constructions,
                });
            }
            Ok::<_, ProblemError>(lanes)
        };
        let lanes = match borrowed {
            Some(_) => launch()?,
            None => service
                .quiescent_phase(count, 0, &execution, launch)
                .map_err(|e| e.into_problem())??,
        };
        Ok(Self {
            problem,
            execution,
            lanes,
            borrowed,
            _owner: owner,
            #[cfg(all(test, feature = "solver-diffsol"))]
            wave_gate: None,
        })
    }
    #[cfg(all(test, feature = "solver-diffsol"))]
    pub(super) fn require_concurrent_test_wave(&mut self) {
        self.wave_gate = Some(Arc::new(std::sync::Barrier::new(self.lanes.len())));
    }
    #[cfg(all(test, feature = "solver-diffsol"))]
    pub(super) fn test_constructions(&self) -> Vec<usize> {
        self.lanes
            .iter()
            .map(|lane| {
                lane.constructions
                    .load(std::sync::atomic::Ordering::Relaxed)
            })
            .collect()
    }
    pub(super) fn evaluate(
        &mut self,
        x: &[f64],
        demand: Demand,
        previous: Option<&BTreeMap<InstanceId, Arc<native::dynamics::Report>>>,
        validity: &UpgradeValidity,
    ) -> Result<Vec<ExperimentProducts>, ProblemError> {
        let _phase = pse_diagnostics::observation::phase("fit.experiment_dispatch");
        if let Some(context) = pse_diagnostics::observation::current() {
            context.gauge("fit.experiment_lanes", "count", self.lanes.len() as f64);
        }
        let service = self.problem.runtime.native().clone();
        let count = self.problem.experiments.len();
        let x: Arc<[f64]> = Arc::from(x);
        let work = || {
            self.execution.check()?;
            let (reply, replies) = mpsc::sync_channel(count);
            let mut results: Vec<Option<Result<ExperimentProducts, ProblemError>>> =
                (0..count).map(|_| None).collect();
            for owner in &self.lanes {
                let previous = owner.experiments.iter()
                    .map(|&ei| {
                        let prior = previous
                            .and_then(|p| {
                                p.get(&self.problem.declaration.experiments[ei].experiment_id)
                            })
                            .cloned();
                        prior
                    })
                    .collect::<Vec<_>>();
                let request = Request {
                    observation: pse_diagnostics::observation::current(),
                    x: x.clone(),
                    demand,
                    validity: validity.clone(),
                    previous,
                    reply: reply.clone(),
                    #[cfg(all(test, feature = "solver-diffsol"))]
                    gate: self.wave_gate.clone(),
                };
                if owner
                    .request
                    .as_ref()
                    .is_none_or(|sender| sender.send(request).is_err())
                {
                    for &ei in &owner.experiments {
                        results[ei] = Some(Err(ProblemError::internal(
                            "fit experiment lane stopped before dispatch",
                        )));
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
                        if stop.is_none() {
                            stop = self.execution.check().err();
                        }
                    }
                }
            }
            if let Some(error) = stop {
                return Err(error);
            }
            // Every dispatched request has returned or disconnected. Never replay a
            // started experiment following a panic, cancellation or lane failure.
            results
                .into_iter()
                .map(|value| {
                    value.unwrap_or_else(|| {
                        Err(ProblemError::internal(
                            "fit experiment lane returned no result",
                        ))
                    })
                })
                .collect()
        };
        match self.borrowed {
            Some(_) => work(),
            None => service
                .quiescent_phase(self.lanes.len(), 0, &self.execution, work)
                .map_err(|e| e.into_problem())?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn residency_affinity_compares_stride_before_private_worker_construction() {
        let weights = [Some(100), Some(1), Some(100), Some(1)];
        assert_eq!(resident_assignment(&weights, 2), vec![vec![0, 1], vec![2, 3]]);
        assert_eq!(resident_assignment(&[Some(1); 4], 2), vec![vec![0, 2], vec![1, 3]]);
        assert_eq!(resident_assignment(&[Some(100), None, Some(100), Some(1)], 2), vec![vec![0, 2], vec![1, 3]]);
        let mapped = resident_assignment(&weights, 2).into_iter().flatten().collect::<BTreeSet<_>>();
        assert_eq!(mapped, (0..4).collect());
    }
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

#[cfg(test)]
mod startup_tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    fn waiting_lane() -> (SyncSender<Request>, JoinHandle<()>, Arc<AtomicBool>) {
        let (request, incoming) = mpsc::sync_channel::<Request>(1);
        let drained = Arc::new(AtomicBool::new(false));
        let observed = drained.clone();
        let joined = std::thread::spawn(move || {
            assert!(incoming.recv().is_err());
            observed.store(true, Ordering::Release);
        });
        (request, joined, drained)
    }
    #[test]
    fn fitting_failed_lane_startup_drains_current_and_all_started_peers() {
        let mut peers = Vec::new();
        let mut completed = Vec::new();
        for _ in 0..3 {
            let (request, joined, drained) = waiting_lane();
            completed.push(drained);
            peers.push(Lane {
                experiments: Vec::new(),
                request: Some(request),
                joined: Some(joined),
                #[cfg(feature = "solver-diffsol")]
                constructions: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            });
        }
        let (request, joined, drained) = waiting_lane();
        completed.push(drained);
        drain_failed_startup(&mut peers, request, joined);
        assert!(peers.is_empty());
        assert!(
            completed.iter().all(|lane| lane.load(Ordering::Acquire)),
            "startup failure returns only after every actual thread drains"
        );
    }
}
