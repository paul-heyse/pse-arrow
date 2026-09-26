// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native ownership ends after join, including thread-local foreign destructors.
use super::{MathRuntimeError, MathService};
use pse_columnar::flight::FlightCancellation;
use std::sync::Arc;
/// Cancels on future abandonment while the detached supervisor retains all resource owners.
struct Caller(Option<FlightCancellation>);
impl Drop for Caller {
    fn drop(&mut self) {
        if let Some(cancel) = &self.0 {
            cancel.cancel();
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
        let service = self.clone();
        let cancel = FlightCancellation::default();
        let progress = Arc::new(pse_backend_native::solve::Progress::new(256));
        let flag = cancel.clone();
        let events = progress.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result = service
                .job_retained(cores, bytes, flag, move |flag| work(flag, events))
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
        if cores == 0 || cores > self.cores || cores > u32::MAX as usize {
            return Err(MathRuntimeError::Limit("optimizer cores"));
        }
        let slot = self
            .jobs
            .clone()
            .try_acquire_owned()
            .map_err(|_| MathRuntimeError::Limit("native jobs"))?;
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
                if cancel.flag().load(std::sync::atomic::Ordering::Acquire){return Err(MathRuntimeError::Cancelled);}
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
