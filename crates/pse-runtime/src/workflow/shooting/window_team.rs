// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed-trial windows with private owner-affine workers and ordered publication.
use super::*;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::mpsc::{self, SyncSender},
    thread::JoinHandle,
    time::Duration,
};

type Product = (native::dynamics::Report, Option<Vec<f64>>);
#[cfg(test)]
type BufferIdentity = (usize, usize, usize, usize);
#[cfg(test)]
type BufferIdentities = BTreeMap<usize, BufferIdentity>;
#[derive(Debug)]
struct WindowCoverage {
    parent: Option<Arc<dyn native::solve::WorkAdmission>>,
    lease: Arc<pse_columnar::AllocationLease>,
    claimant: std::sync::Mutex<std::sync::Weak<()>>,
    _original_claim: Arc<()>,
}
impl native::solve::WorkAdmission for WindowCoverage {
    fn prepaid_storage(
        &self,
        identity: &Arc<()>,
        required: usize,
    ) -> Result<Option<Arc<dyn std::any::Any + Send + Sync>>, ProblemError> {
        let mut claimant = self
            .claimant
            .lock()
            .map_err(|_| ProblemError::internal("window foreign coverage poisoned"))?;
        if let Some(current) = claimant.upgrade()
            && !Arc::ptr_eq(&current, identity)
        {
            return Err(ProblemError::memory(
                "window foreign allowance already has a resident owner",
            ));
        }
        if required == 0 || required > self.lease.size() {
            return Ok(None);
        }
        *claimant = Arc::downgrade(identity);
        Ok(Some(Arc::new((
            self.lease.clone(),
            self._original_claim.clone(),
        ))))
    }
    fn admit(&self, work: native::solve::WorkEvidence) -> Result<(), ProblemError> {
        self.parent
            .as_ref()
            .map_or(Ok(()), |parent| parent.admit(work))
    }
    fn observe(&self, work: native::solve::WorkEvidence) -> Result<(), ProblemError> {
        self.parent
            .as_ref()
            .map_or(Ok(()), |parent| parent.observe(work))
    }
    fn admit_storage(
        &self,
        scope: native::solve::NativeStorageScope,
        owner: &str,
        bytes: usize,
        opaque: bool,
    ) -> Result<(), ProblemError> {
        if scope == native::solve::NativeStorageScope::Linear {
            if bytes > self.lease.size() {
                return Err(ProblemError::memory(
                    "window native storage exceeds prepaid foreign allowance",
                ));
            }
            return Ok(());
        }
        self.parent.as_ref().map_or(Ok(()), |parent| {
            parent.admit_storage(scope, owner, bytes, opaque)
        })
    }
}
#[derive(Clone, Copy)]
pub(super) enum Demand {
    Forward {
        sensitivity: DynamicSensitivity,
        quadratures: bool,
    },
    Integral,
}
/// One current trial's borrowed scientific inputs and integration attribution.
struct WindowEvaluation<'a> {
    x: &'a [f64],
    demand: Demand,
    execution: &'a Execution,
    producer: Option<pse_diagnostics::observation::Producer>,
    purpose: crate::invocation::IntegrationPurpose,
}
struct Request {
    context: crate::ExecutionContext,
    producer: Option<pse_diagnostics::observation::Producer>,
    purpose: crate::invocation::IntegrationPurpose,
    execution: Execution,
    x: Arc<[f64]>,
    demand: Demand,
    reply: SyncSender<(usize, Result<Product, ProblemError>)>,
    #[cfg(test)]
    gate: Option<Arc<std::sync::Barrier>>,
    #[cfg(test)]
    failure: Option<usize>,
}
struct Lane {
    windows: Vec<usize>,
    send: Option<SyncSender<Request>>,
    joined: Option<JoinHandle<()>>,
}
impl Drop for Lane {
    fn drop(&mut self) {
        self.send.take();
        if let Some(joined) = self.joined.take() {
            let _ = joined.join();
        }
    }
}
#[allow(
    clippy::large_enum_variant,
    reason = "width one keeps its admitted current worker inline without a new heap or thread grant, preserving finite-memory serial fallback"
)]
enum Owners {
    // Width one needs no extra thread stack or worker population. Both physical
    // arrangements consume the same window operation and ordered reduction.
    Inline(Option<super::super::dynamics::DynamicWorker>),
    Parallel {
        lanes: Vec<Lane>,
        _owner: Arc<pse_columnar::AllocationLease>,
    },
}
pub(super) struct WindowTeam {
    math: Arc<crate::math::MathService>,
    pub(super) source: Arc<ShootingProblem>,
    pub(super) execution: Execution,
    owners: Owners,
    #[cfg(test)]
    identities: Arc<std::sync::Mutex<BufferIdentities>>,
    #[cfg(test)]
    gate: Option<Arc<std::sync::Barrier>>,
    #[cfg(test)]
    failure: Option<usize>,
}
impl std::fmt::Debug for WindowTeam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowTeam")
            .field("width", &self.width())
            .finish_non_exhaustive()
    }
}
impl WindowTeam {
    pub(super) fn width(&self) -> usize {
        match &self.owners {
            Owners::Inline(_) => 1,
            Owners::Parallel { lanes, .. } => lanes.len(),
        }
    }
    pub(super) fn new(
        problem: Arc<ShootingProblem>,
        math: Arc<crate::math::MathService>,
        execution: Execution,
        initial: Option<super::super::dynamics::DynamicWorker>,
    ) -> Result<Self, ProblemError> {
        Self::prepare(problem, math, execution, initial, None)
    }
    pub(super) fn inline(
        problem: Arc<ShootingProblem>,
        math: Arc<crate::math::MathService>,
        execution: Execution,
    ) -> Result<Self, ProblemError> {
        Self::prepare(problem, math, execution, None, Some(1))
    }
    #[cfg(all(test, feature = "solver-ipopt"))]
    pub(super) fn with_width(
        problem: Arc<ShootingProblem>,
        math: Arc<crate::math::MathService>,
        execution: Execution,
        width: usize,
    ) -> Result<Self, ProblemError> {
        Self::prepare(problem, math, execution, None, Some(width))
    }
    fn prepare(
        problem: Arc<ShootingProblem>,
        math: Arc<crate::math::MathService>,
        execution: Execution,
        initial: Option<super::super::dynamics::DynamicWorker>,
        forced_width: Option<usize>,
    ) -> Result<Self, ProblemError> {
        #[cfg(feature = "solver-idas")]
        let mut initial = initial;
        execution.check()?;
        let service = math;
        let program = &problem.experiment.program;
        let allowances = super::super::dynamics::allowance_bytes(
            &program.state_allowances,
            &program.output_allowances,
        )
        .ok_or_else(|| ProblemError::memory("shooting lane allowance extent"))?;
        let worker = super::super::dynamics::DynamicProgram::worker_allocation_bound(
            &program.contract,
            &program.programs,
            &program.coordinates,
            program.modes.iter(),
            &program.layout,
            allowances,
        )?;
        // Unknown provider constructors consume an explicit prepaid opaque envelope.
        // Service cores bound a candidate; only quiescent_phase grants active compute.
        let mut width = forced_width
            .unwrap_or(problem.windows.len())
            .min(problem.windows.len())
            .min(service.cores())
            .max(1);
        #[cfg(feature = "solver-idas")]
        if width > 1
            && let Some(initial) = &mut initial
        {
            // Initial anchors have completed. Retire their optional native owner
            // before claiming the original envelope for independent child windows;
            // keep the mathematical worker available if wider admission falls back.
            initial.discard_idas_workspace();
        }
        let estimate =
            crate::math::ConstructionEstimate::from_bound(worker, service.policy().worker_bytes);
        let mut grant = None;
        while width > 1 {
            let bytes = service
                .stack_bytes()
                .checked_mul(width)
                .and_then(|bytes| bytes.checked_add(problem.windows.len().checked_mul(512)?))
                .and_then(|bytes| bytes.checked_add(size_of::<Self>()));
            let Some(bytes) = bytes else {
                width -= 1;
                continue;
            };
            let candidate = (|| {
                let owner = service.reserve("shooting:window-lanes", bytes)?;
                let constructors = (0..problem.windows.len())
                    .map(|_| {
                        service.reserve_constructor("shooting:window-worker", estimate, &execution)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let foreign = execution
                    .memory
                    .ok_or(crate::math::MathRuntimeError::Limit(
                        "window foreign allowance absent",
                    ))?;
                let identity = Arc::new(());
                let original = execution
                    .work_admission
                    .as_ref()
                    .map_or(Ok(None), |parent| {
                        parent.prepaid_storage(&identity, foreign)
                    })?
                    .and_then(|owner| Arc::downcast::<pse_columnar::AllocationLease>(owner).ok());
                let mut coverages = Vec::with_capacity(problem.windows.len());
                for window in 0..problem.windows.len() {
                    let lease = if window == 0 { original.clone() } else { None }
                        .map_or_else(|| service.reserve("shooting:window-foreign", foreign), Ok)?;
                    coverages.push(Arc::new(WindowCoverage {
                        parent: execution.work_admission.clone(),
                        lease,
                        claimant: std::sync::Mutex::new(std::sync::Weak::new()),
                        _original_claim: identity.clone(),
                    }));
                }
                Ok::<_, crate::math::MathRuntimeError>((owner, constructors, coverages))
            })();
            match candidate {
                Ok(owner) => {
                    grant = Some(owner);
                    break;
                }
                Err(crate::math::MathRuntimeError::Pool(
                    datafusion::common::DataFusionError::ResourcesExhausted(_),
                ))
                | Err(crate::math::MathRuntimeError::Limit(_)) => {
                    width -= 1;
                }
                Err(error) => return Err(error.into_problem()),
            }
        }
        #[cfg(test)]
        let identities = Arc::new(std::sync::Mutex::new(BTreeMap::new()));
        let owners = if let Some((owner, constructors, coverages)) = grant {
            // A parent-created !Send worker is destroyed on its actual owner. Child
            // workers are constructed only after their complete resident admission.
            drop(initial);
            let runtime = tokio::runtime::Handle::try_current().map_err(|error| {
                ProblemError::internal(format!("shooting lane runtime: {error}"))
            })?;
            let launch = || {
                let mut lanes = Vec::with_capacity(width);
                for lane in 0..width {
                    let assigned: Vec<_> = (lane..problem.windows.len()).step_by(width).collect();
                    let windows = assigned.clone();
                    let (send, requests) = mpsc::sync_channel::<Request>(1);
                    let (ready, prepared) = mpsc::sync_channel(1);
                    let source = problem.clone();
                    let execution = execution.clone();
                    let runtime = runtime.clone();
                    let preparation_context = crate::ExecutionContext::capture();
                    let constructors: Vec<_> =
                        assigned.iter().map(|&k| constructors[k].clone()).collect();
                    let coverages: Vec<_> =
                        assigned.iter().map(|&k| coverages[k].clone()).collect();
                    #[cfg(test)]
                    let observed = identities.clone();
                    let joined = std::thread::Builder::new()
                        .name(format!("pse-shooting-{lane}"))
                        .stack_size(service.stack_bytes())
                        .spawn(move || {
                            let _runtime = runtime.enter();
                            let scope = match execution.scope() {
                                Ok(scope) => {
                                    scope.with_admitted_threads(std::num::NonZeroUsize::MIN)
                                }
                                Err(error) => {
                                    let _ = ready.send(Err(error));
                                    return;
                                }
                            };
                            // Each finite worker has its own prepaid constructor owner.
                            // Keep these locals until every worker has been destroyed.
                            let worker_owners = constructors;
                            let prepare = || {
                                assigned
                                    .iter()
                                    .enumerate()
                                    .map(|(local, _)| {
                                        pse_math::construction::scoped(
                                            worker_owners[local].clone(),
                                            || {
                                                source
                                                    .experiment
                                                    .worker_scoped(scope.clone())
                                                    .map(Some)
                                            },
                                        )
                                    })
                                    .collect::<Result<Vec<_>, _>>()
                            };
                            let prepared = {
                                let _context = preparation_context.enter_native();
                                catch_unwind(AssertUnwindSafe(|| {
                                    native::execution::original_scoped(&execution, prepare)
                                }))
                            }
                            .unwrap_or_else(|_| {
                                Err(ProblemError::internal("shooting lane preparation panic"))
                            });
                            drop(preparation_context);
                            let mut workers = match prepared {
                                Ok(workers) => {
                                    if ready.send(Ok(())).is_err() {
                                        return;
                                    }
                                    workers
                                }
                                Err(error) => {
                                    let _ = ready.send(Err(error));
                                    return;
                                }
                            };
                            while let Ok(mut request) = requests.recv() {
                                #[cfg(test)]
                                if let Some(gate) = &request.gate {
                                    gate.wait();
                                }
                                match request.execution.scope() {
                                    Ok(scope) => {
                                        request.execution.enclosing_scope = Some(
                                            scope
                                                .with_admitted_threads(std::num::NonZeroUsize::MIN),
                                        )
                                    }
                                    Err(error) => {
                                        for &k in &assigned {
                                            let _ = request.reply.send((
                                                k,
                                                Err(ProblemError::internal(format!(
                                                    "shooting lane scope: {error}"
                                                ))),
                                            ));
                                        }
                                        continue;
                                    }
                                }
                                let _context = request.context.enter_native();
                                for (local, &k) in assigned.iter().enumerate() {
                                    let work = || {
                                        let worker = workers[local].as_mut().ok_or_else(|| {
                                            ProblemError::internal(
                                                "shooting lane requires a fresh attempt",
                                            )
                                        })?;
                                        #[cfg(test)]
                                        if let Ok(mut identities) = observed.lock() {
                                            identities.insert(
                                                k,
                                                worker.buffer_identity(
                                                    0,
                                                    native::dynamics::Function::Rhs,
                                                ),
                                            );
                                        }
                                        #[cfg(test)]
                                        if request.failure == Some(k) {
                                            return Err(ProblemError::numerical(
                                                "forced middle window refusal",
                                            ));
                                        }
                                        let mut execution = request.execution.clone();
                                        execution.work_admission = Some(coverages[local].clone());
                                        evaluate_window(
                                            &source,
                                            worker,
                                            k,
                                            WindowEvaluation {
                                                x: &request.x,
                                                demand: request.demand,
                                                execution: &execution,
                                                producer: request.producer.clone(),
                                                purpose: request.purpose,
                                            },
                                        )
                                    };
                                    let result = catch_unwind(AssertUnwindSafe(|| {
                                        native::execution::original_scoped(
                                            &request.execution,
                                            || {
                                                pse_math::construction::scoped(
                                                    worker_owners[local].clone(),
                                                    work,
                                                )
                                            },
                                        )
                                    }))
                                    .unwrap_or_else(|_| {
                                        Err(ProblemError::internal("shooting lane panic"))
                                    });
                                    if let Err(error) = &result {
                                        if native::callback::classify(error)
                                            == native::callback::Failure::Trial
                                        {
                                            if let Some(worker) = &mut workers[local] {
                                                worker.invalidate_trial();
                                            }
                                        } else {
                                            workers[local] = None;
                                        }
                                    }
                                    if request.reply.send((k, result)).is_err() {
                                        break;
                                    }
                                }
                            }
                            // Every retained worker/provider is destroyed here before join.
                        })
                        .map_err(|error| {
                            ProblemError::internal(format!("shooting lane creation: {error}"))
                        })?;
                    let lane_owner = Lane {
                        windows,
                        send: Some(send),
                        joined: Some(joined),
                    };
                    match prepared.recv() {
                        Ok(Ok(())) => lanes.push(lane_owner),
                        Ok(Err(error)) => {
                            drop(lane_owner);
                            drop(lanes);
                            return Err(error);
                        }
                        Err(_) => {
                            drop(lane_owner);
                            drop(lanes);
                            return Err(ProblemError::internal(
                                "shooting lane stopped during preparation",
                            ));
                        }
                    }
                }
                Ok::<_, ProblemError>(lanes)
            };
            let lanes = service
                .quiescent_phase(width, 0, &execution, launch)
                .map_err(|error| error.into_problem())??;
            Owners::Parallel {
                lanes,
                _owner: owner,
            }
        } else {
            Owners::Inline(Some(match initial {
                Some(worker) => worker,
                None => problem.experiment.worker(&execution)?,
            }))
        };
        Ok(Self {
            math: service,
            source: problem,
            execution,
            owners,
            #[cfg(test)]
            identities,
            #[cfg(test)]
            gate: None,
            #[cfg(test)]
            failure: None,
        })
    }
    pub(super) fn evaluate(
        &mut self,
        x: &[f64],
        demand: Demand,
        execution: &Execution,
        producer: Option<pse_diagnostics::observation::Producer>,
        purpose: crate::invocation::IntegrationPurpose,
    ) -> Result<Vec<Product>, ProblemError> {
        execution.check()?;
        // This selected product requires an actual current foreign allowance.
        // Refuse before touching any retained evaluator or dispatching a wave.
        if matches!(
            demand,
            Demand::Forward {
                quadratures: true,
                ..
            }
        ) && execution.memory.is_none()
        {
            return Err(ProblemError::memory(
                "forward quadratures require admitted foreign allowance",
            ));
        }
        let source = &self.source;
        match &mut self.owners {
            Owners::Inline(worker) => {
                let mut products = Vec::with_capacity(source.windows.len());
                for k in 0..source.windows.len() {
                    let current = worker.as_mut().ok_or_else(|| {
                        ProblemError::internal("shooting lane requires a fresh attempt")
                    })?;
                    #[cfg(test)]
                    if let Ok(mut identities) = self.identities.lock() {
                        identities.insert(
                            k,
                            current.buffer_identity(0, native::dynamics::Function::Rhs),
                        );
                    }
                    match evaluate_window(
                        source,
                        current,
                        k,
                        WindowEvaluation {
                            x,
                            demand,
                            execution,
                            producer: producer.clone(),
                            purpose,
                        },
                    ) {
                        Ok(product) => products.push(product),
                        Err(error) => {
                            if native::callback::classify(&error)
                                == native::callback::Failure::Trial
                            {
                                current.invalidate_trial();
                            } else {
                                *worker = None;
                            }
                            return Err(error);
                        }
                    }
                }
                Ok(products)
            }
            Owners::Parallel { lanes, .. } => {
                let x: Arc<[f64]> = Arc::from(x);
                let work = || {
                    let (reply, replies) = mpsc::sync_channel(source.windows.len());
                    let mut results: Vec<_> = (0..source.windows.len()).map(|_| None).collect();
                    for lane in lanes.iter() {
                        let request = Request {
                            context: crate::ExecutionContext::capture(),
                            producer: producer.clone(),
                            purpose,
                            execution: execution.clone(),
                            x: x.clone(),
                            demand,
                            reply: reply.clone(),
                            #[cfg(test)]
                            gate: self.gate.clone(),
                            #[cfg(test)]
                            failure: self.failure,
                        };
                        if lane
                            .send
                            .as_ref()
                            .is_none_or(|send| send.send(request).is_err())
                        {
                            for &k in &lane.windows {
                                results[k] = Some(Err(ProblemError::internal(
                                    "shooting lane stopped before dispatch",
                                )));
                            }
                        }
                    }
                    drop(reply);
                    let mut stop = None;
                    loop {
                        match replies.recv_timeout(Duration::from_millis(10)) {
                            Ok((k, result)) => results[k] = Some(result),
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                            Err(mpsc::RecvTimeoutError::Timeout) => {
                                if stop.is_none() {
                                    stop = execution.check().err();
                                }
                            }
                        }
                    }
                    // Drain every started window before returning a failure or stronger point.
                    if let Some(error) = stop {
                        return Err(error);
                    }
                    execution.check()?;
                    results
                        .into_iter()
                        .map(|result| {
                            result.unwrap_or_else(|| {
                                Err(ProblemError::internal("shooting lane returned no result"))
                            })
                        })
                        .collect()
                };
                self.math
                    .quiescent_phase(lanes.len(), 0, execution, work)
                    .map_err(|error| error.into_problem())?
            }
        }
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-idas"))]
    pub(super) fn require_concurrent_wave(&mut self) {
        assert!(self.width() > 1);
        self.gate = Some(Arc::new(std::sync::Barrier::new(self.width())));
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-idas"))]
    pub(super) fn fail_window(&mut self, window: Option<usize>) {
        self.failure = window;
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-idas"))]
    pub(super) fn buffer_identities(&self) -> Result<BufferIdentities, ProblemError> {
        self.identities
            .lock()
            .map(|identities| identities.clone())
            .map_err(|_| ProblemError::internal("shooting fixture buffer identities poisoned"))
    }
    #[cfg(all(test, feature = "solver-ipopt"))]
    pub(super) fn buffer_identity(&self) -> Result<BufferIdentity, ProblemError> {
        self.identities
            .lock()
            .map_err(|_| ProblemError::internal("shooting fixture buffer identities poisoned"))?
            .values()
            .next()
            .copied()
            .ok_or_else(|| {
                ProblemError::internal("shooting fixture has no completed window buffer")
            })
    }
}
fn evaluate_window(
    problem: &ShootingProblem,
    worker: &mut super::super::dynamics::DynamicWorker,
    k: usize,
    request: WindowEvaluation<'_>,
) -> Result<Product, ProblemError> {
    let WindowEvaluation {
        x,
        demand,
        execution,
        producer,
        purpose,
    } = request;
    execution.check()?;
    let window = &problem.windows[k];
    let integration = problem.integration(x);
    let (forward, adjoint) = match demand {
        Demand::Forward { sensitivity, .. } => (sensitivity == DynamicSensitivity::Forward, false),
        Demand::Integral => (false, true),
    };
    let _phase = pse_diagnostics::observation::current().map(|context| {
        if !adjoint {
            context.count("shooting.window_integrations", "count", 1);
        }
        context.integer(
            "shooting.window_index",
            "index",
            i64::try_from(k).unwrap_or(i64::MAX),
        );
        context.integer(
            "shooting.window_columns",
            "count",
            i64::try_from(window.columns.len()).unwrap_or(i64::MAX),
        );
        context.gauge(
            "shooting.window_start",
            "physical seconds",
            window.profile.start,
        );
        context.gauge(
            "shooting.window_end",
            "physical seconds",
            window.profile.end,
        );
        context.boolean("shooting.forward_sensitivities", forward);
        context.phase("shooting.window")
    });
    let observation = crate::invocation::IntegrationObservation::begin(
        producer,
        purpose,
        forward,
        adjoint,
        false,
        k as u64,
        forward.then(|| {
            window
                .profile
                .forward_axes
                .as_ref()
                .map_or(window.columns.len(), Vec::len)
        }),
    );
    let result = match demand {
        Demand::Forward {
            sensitivity: _,
            quadratures: true,
        } => {
            #[cfg(feature = "solver-idas")]
            {
                problem
                    .experiment
                    .window_forward_quadratures(
                        worker,
                        window,
                        &integration,
                        problem.anchors(k, x),
                        execution,
                    )
                    .map(|product| (product.report, product.derivatives))
            }
            #[cfg(not(feature = "solver-idas"))]
            {
                Err(ProblemError::unsupported(
                    "IDAS forward quadratures are not linked",
                ))
            }
        }
        Demand::Forward {
            sensitivity,
            quadratures: false,
        } => problem
            .experiment
            .integrate_window(
                worker,
                window,
                &integration,
                problem.anchors(k, x),
                execution,
                sensitivity,
            )
            .map(|report| (report, None)),
        Demand::Integral => {
            let m = problem.experiment.program.contract.outputs.len();
            let nq = problem.experiment.program.contract.quadratures.len();
            let mut cotangent = |report: &native::dynamics::Report| {
                let mut weights = vec![0.; report.samples.len() * (m + nq)];
                let last = (report.samples.len() - 1) * (m + nq);
                for (q, weight) in &problem.integral {
                    weights[last + m + q] += weight;
                }
                Ok(weights)
            };
            problem
                .experiment
                .window_gradient(
                    worker,
                    window,
                    &integration,
                    problem.anchors(k, x),
                    execution,
                    &problem.adjoint_axes(k),
                    &mut cotangent,
                )
                .and_then(|(report, axes, gradient)| {
                    let mut product = vec![0.; x.len()];
                    for (axis, derivative) in axes.iter().zip(gradient) {
                        let column = window
                            .columns
                            .get(*axis)
                            .ok_or_else(|| ProblemError::internal("shooting adjoint coordinate"))?;
                        if let Some(variable) = problem.variable(k, column) {
                            product[variable] += derivative;
                        }
                    }
                    Ok((report, Some(product)))
                })
        }
    };
    observation.finish(result.is_ok());
    result
}
