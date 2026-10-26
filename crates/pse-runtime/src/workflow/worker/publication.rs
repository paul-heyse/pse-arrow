// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private two-way handoff; public completion still waits for export and ACK.
use super::*;

#[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
type ScientificReadyObserver = Arc<dyn Fn() + Send + Sync>;
#[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
tokio::task_local! { static SCIENTIFIC_READY: ScientificReadyObserver; }
#[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
tokio::task_local! { static IDLE: ScientificReadyObserver; }
#[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
pub(in crate::workflow) async fn observed<T>(
    observer: ScientificReadyObserver,
    work: impl Future<Output = T>,
) -> T {
    SCIENTIFIC_READY.scope(observer, work).await
}
#[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
pub(in crate::workflow) async fn observed_idle<T>(
    observer: ScientificReadyObserver,
    work: impl Future<Output = T>,
) -> T {
    IDLE.scope(observer, work).await
}
#[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
pub(super) fn idle() {
    let _ = IDLE.try_with(|observer| observer());
}

pub(super) struct Ready {
    pub(super) lane: usize,
    pub(super) acknowledged: tokio::sync::oneshot::Sender<()>,
}
/// Captured by exactly one claimed attempt before its run supervisor starts.
pub(in crate::workflow) struct Handoff {
    lane: usize,
    cancel: crate::CancelSource,
    #[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
    observer: Option<ScientificReadyObserver>,
    ready: tokio::sync::mpsc::Sender<Ready>,
    acknowledged: tokio::sync::oneshot::Receiver<()>,
    acknowledge: tokio::sync::oneshot::Sender<()>,
}
impl Handoff {
    pub(super) fn new(
        lane: usize,
        ready: tokio::sync::mpsc::Sender<Ready>,
        cancel: crate::CancelSource,
    ) -> Self {
        let (acknowledge, acknowledged) = tokio::sync::oneshot::channel();
        Self {
            lane,
            cancel,
            ready,
            acknowledged,
            acknowledge,
            #[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
            observer: SCIENTIFIC_READY.try_with(Arc::clone).ok(),
        }
    }
    /// The scheduler retains the source/lane while acknowledging publication capacity.
    /// A lost scheduler cannot strand the attempt: its existing supervisor still
    /// settles publication/heartbeat and typed ACK uncertainty before completing.
    /// False requests the existing cancellation settlement route, never an
    /// extra payload publisher outside the scheduler's finite capacity.
    #[tracing::instrument(name = "pse.publication.capacity.wait", skip_all, fields(pse.owner.stage = true))]
    pub(in crate::workflow) async fn ready(self) -> Result<bool, WorkflowError> {
        let Self {
            lane,
            cancel,
            ready,
            acknowledged,
            acknowledge,
            #[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
            observer,
        } = self;
        #[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
        if let Some(observer) = observer {
            observer();
        }
        let sent = tokio::select! {
            biased;
            ()=cancel.cancelled()=>return Ok(false),
            sent=ready.send(Ready {lane,acknowledged:acknowledge})=>sent,
        };
        if sent.is_err() {
            return if cancel.token().is_cancelled() {
                Ok(false)
            } else {
                Err(contract(
                    "worker publication scheduler unavailable before handoff",
                ))
            };
        }
        tokio::select! {
            biased;
            ()=cancel.cancelled()=>Ok(false),
            acknowledged=acknowledged=>match acknowledged {
                Ok(())=>Ok(true),
                Err(_) if cancel.token().is_cancelled()=>Ok(false),
                Err(_)=>Err(contract("worker publication capacity acknowledgement unavailable")),
            },
        }
    }
}
impl std::fmt::Debug for Handoff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PublicationHandoff")
            .field("lane", &self.lane)
            .finish_non_exhaustive()
    }
}

#[derive(Default)]
pub(super) enum State {
    #[default]
    Idle,
    Running,
    Waiting(tokio::sync::oneshot::Sender<()>),
    Publishing,
}
/// Finite coordination metadata, prepaid before any canonical claim.
pub(super) fn coordination_bytes(lanes: usize) -> Result<usize, WorkflowError> {
    // Tokio's bounded channel allocates blocks of 32 messages plus fixed list
    // metadata. Each live handoff also has one fixed-size oneshot allocation.
    // The retained futures already charge their own complete concrete frames.
    lanes
        .max(32)
        .checked_mul(size_of::<Ready>() + 128)
        .and_then(|bytes| {
            lanes
                .checked_mul(size_of::<State>() + size_of::<Handoff>() + size_of::<usize>() + 128)
                .and_then(|slots| bytes.checked_add(slots))
        })
        .and_then(|bytes| bytes.checked_add(512))
        .ok_or_else(|| contract("worker publication coordination extent"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lost_scheduler_and_capacity_ack_are_not_cancelled_or_granted() {
        for lose_ack in [false, true] {
            let cancel = crate::CancelSource::new();
            let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
            let handoff = Handoff::new(0, sender, cancel.clone());
            let result = if lose_ack {
                let (result, ()) = tokio::join!(handoff.ready(), async {
                    let ready = receiver.recv().await.unwrap();
                    drop(ready.acknowledged);
                });
                result
            } else {
                drop(receiver);
                handoff.ready().await
            };
            assert!(result.is_err());
            assert!(!cancel.token().is_cancelled());
        }
        let cancel = crate::CancelSource::new();
        cancel.cancel();
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        assert!(!Handoff::new(0, sender, cancel).ready().await.unwrap());
        assert!(
            receiver.try_recv().is_err(),
            "pre-cancelled owner never requests a publisher slot"
        );
    }
}
