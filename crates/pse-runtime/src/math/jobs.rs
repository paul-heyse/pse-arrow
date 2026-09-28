// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native ownership ends after join, including thread-local foreign destructors.
use super::{MathRuntimeError, MathService};
use pse_columnar::flight::FlightCancellation;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
/// The worker capacity of one admitted job, carved from the job's reservation. Every
/// evaluator the job builds charges its numeric storage here and releases the charge when
/// it is dropped, so the reservation covers all of the job's live workers together, never
/// only the first; a worker that does not fit is refused (F31).
#[derive(Debug)]
pub struct WorkerBudget {
    capacity: usize,
    used: AtomicUsize,
}
impl WorkerBudget {
    /// A budget of `capacity` bytes, the worker share of the job's reservation.
    pub(crate) fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            capacity,
            used: AtomicUsize::new(0),
        })
    }
    /// Reserve `bytes` for one worker until the returned charge is dropped.
    ///
    /// # Errors
    /// The job's live workers and this one exceed its worker capacity.
    pub fn charge(self: &Arc<Self>, bytes: usize) -> Result<WorkerCharge, MathRuntimeError> {
        self.used
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes).filter(|n| *n <= self.capacity)
            })
            .map_err(|_| MathRuntimeError::Limit("worker storage"))?;
        Ok(WorkerCharge {
            budget: self.clone(),
            bytes,
        })
    }
    /// The worker share of the job's reservation.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    /// Bytes charged by the job's live workers.
    pub fn used(&self) -> usize {
        self.used.load(Ordering::Acquire)
    }
}
/// One live worker's share of its job's reservation.
#[derive(Debug)]
pub struct WorkerCharge {
    budget: Arc<WorkerBudget>,
    bytes: usize,
}
impl Drop for WorkerCharge {
    fn drop(&mut self) {
        self.budget.used.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
/// Cancels on future abandonment while the detached supervisor retains all resource owners.
struct Caller(Option<FlightCancellation>);
impl Drop for Caller {
    fn drop(&mut self) {
        if let Some(cancel) = &self.0 {
            cancel.cancel();
        }
    }
}
/// How a submitted operation is admitted, cancelled and observed.
#[derive(Debug)]
pub(crate) struct Submission {
    /// The operation's cancellation, shared with its public handle.
    pub(crate) cancel: FlightCancellation,
    /// Its event stream: bounded in memory, and tapped by a durable stream when durable.
    pub(crate) progress: Arc<pse_backend_native::solve::Progress>,
    /// Durable work waits for a job slot; other work is refused when none is free.
    pub(crate) queue: bool,
    /// Signalled once the operation holds its job slot, CPU permits and reservation.
    pub(crate) admitted: Option<tokio::sync::oneshot::Sender<()>>,
}
impl Submission {
    /// In-memory work: a fresh cancellation, 256 retained events, refused when full.
    pub(crate) fn ephemeral() -> Self {
        Self {
            cancel: FlightCancellation::default(),
            progress: Arc::new(pse_backend_native::solve::Progress::new(256)),
            queue: false,
            admitted: None,
        }
    }
}
impl MathService {
    /// Submit one finite operation under the existing admission, teardown and join owner.
    pub(crate) fn submit<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        work: impl FnOnce(
            Arc<std::sync::atomic::AtomicBool>,
            Arc<pse_backend_native::solve::Progress>,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<super::solves::SolveHandle<(T, Arc<pse_columnar::AllocationLease>)>, MathRuntimeError>
    {
        self.submit_with(cores, bytes, Submission::ephemeral(), work)
    }
    /// Submit one finite operation with an explicit admission policy, cancellation and
    /// event stream.
    pub(crate) fn submit_with<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        submission: Submission,
        work: impl FnOnce(
            Arc<std::sync::atomic::AtomicBool>,
            Arc<pse_backend_native::solve::Progress>,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<super::solves::SolveHandle<(T, Arc<pse_columnar::AllocationLease>)>, MathRuntimeError>
    {
        let service = self.clone();
        let Submission {
            cancel,
            progress,
            queue,
            admitted,
        } = submission;
        let flag = cancel.clone();
        let events = progress.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result = service
                .admitted(cores, bytes, flag, (queue, admitted), move |flag| {
                    work(flag, events)
                })
                .await;
            let _ = sender.send(result);
        });
        Ok(super::solves::SolveHandle {
            cancel,
            receiver: Some(receiver),
            progress,
        })
    }
    pub(super) async fn job<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: FlightCancellation,
        work: impl FnOnce(Arc<std::sync::atomic::AtomicBool>) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<T, MathRuntimeError> {
        self.job_retained(cores, bytes, cancel, move |flag| work(flag).map(|r| (r, 0)))
            .await
            .map(|(r, _)| r)
    }
    /// Transfer retained capacity from the active job reservation after native join.
    /// The returned owner is acquired without releasing/reacquiring pool capacity.
    pub(super) async fn job_retained<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: FlightCancellation,
        work: impl FnOnce(Arc<std::sync::atomic::AtomicBool>) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        self.admitted(cores, bytes, cancel, (false, None), work)
            .await
    }
    /// [`MathService::job_retained`] under an explicit admission: a queued job waits for a
    /// job slot (durable admission, ADR-0112 Outcome 14); otherwise a full service refuses
    /// it.
    async fn admitted<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: FlightCancellation,
        (queue, admitted): (bool, Option<tokio::sync::oneshot::Sender<()>>),
        work: impl FnOnce(Arc<std::sync::atomic::AtomicBool>) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        if cores == 0 || cores > self.cores || cores > u32::MAX as usize {
            return Err(MathRuntimeError::Limit("optimizer cores"));
        }
        let slot = if queue {
            tokio::select! {
                slot = self.jobs.clone().acquire_owned() => {
                    slot.map_err(|_| MathRuntimeError::Limit("native job admission closed"))?
                }
                () = cancel.cancelled() => return Err(MathRuntimeError::Cancelled),
            }
        } else {
            self.jobs
                .clone()
                .try_acquire_owned()
                .map_err(|_| MathRuntimeError::Limit("native jobs"))?
        };
        let mut caller = Caller(Some(cancel.clone()));
        let service = self.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result=async {
                let cpu=tokio::select!{p=service.cpu.clone().acquire_many_owned(cores as u32)=>p.map_err(|_|MathRuntimeError::Limit("CPU admission closed"))?,()=cancel.cancelled()=>return Err(MathRuntimeError::Cancelled)};
                // Parallel libraries may keep a coordinator in addition to their
                // admitted worker team. Charge its stack plus the whole team; CPU
                // permits bound the active workers. Serial jobs have one stack.
                let stacks = if cores > 1 { cores.checked_add(1) } else { Some(1) };
                let bytes=stacks.and_then(|n|n.checked_mul(service.policy.stack_bytes))
                    .and_then(|n|n.checked_add(bytes)).and_then(|n|n.checked_add(service.policy.foreign_bytes)).ok_or(MathRuntimeError::Limit("native allowance overflow"))?;
                let lease=datafusion::execution::memory_pool::MemoryConsumer::new("math:native-job").register(&service.pool);
                lease.try_grow(bytes)?;
                if cancel.flag().load(Ordering::Acquire){return Err(MathRuntimeError::Cancelled);}
                if let Some(admitted)=admitted {let _=admitted.send(());}
                let flag=cancel.flag();
                let handle=std::thread::Builder::new().name("pse-math".into()).stack_size(service.policy.stack_bytes).spawn(move||work(flag)).map_err(|e|MathRuntimeError::Infrastructure(e.to_string()))?;
                // Joining, not receipt of an early result, witnesses TLS destruction.
                let result=tokio::task::spawn_blocking(move||handle.join()).await.map_err(|e|MathRuntimeError::Infrastructure(e.to_string()))?.map_err(|_|MathRuntimeError::Infrastructure("native worker panic".into()))?;
                let result = result.and_then(|(value, retained)| {
                    if retained > lease.size() { return Err(MathRuntimeError::Limit("retained result exceeds admitted capacity")); }
                    Ok((value, pse_columnar::AllocationLease::new(lease.split(retained))))
                });
                drop(lease);drop(cpu);result
            }.await;
            drop(slot);
            let _ = tx.send(result);
        });
        let result = rx
            .await
            .map_err(|_| MathRuntimeError::Infrastructure("lost native supervisor".into()))?;
        caller.0.take();
        drop(caller);
        result
    }
}
