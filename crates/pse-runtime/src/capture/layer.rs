// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Caller-composed lifecycle collection. Callbacks only read a clock and try a bounded enqueue.

use super::CaptureRequest;
use pse_diagnostics::observation::OperationId;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{Receiver, SyncSender, sync_channel},
};
use tracing::{
    Subscriber,
    span::{Attributes, Id},
};
use tracing_subscriber::{Layer, layer::Context};

pub(super) enum LifecycleKind {
    Begin,
    End,
    Instant,
}
pub(super) struct Lifecycle {
    pub kind: LifecycleKind,
    pub span: u64,
    pub at: u64,
    pub name: String,
    pub operation: Option<OperationId>,
}
pub(super) struct ReceiverState {
    pub receiver: Receiver<Lifecycle>,
    pub dropped: Arc<AtomicU64>,
}
/// Optional caller-composed layer. It never installs a subscriber or writes from a callback.
/// Span creation/close describe span lifetime, not active CPU time or thread residency.
#[derive(Clone, Debug)]
pub struct CaptureLayer {
    sender: SyncSender<Lifecycle>,
    stop: Arc<AtomicBool>,
    dropped: Arc<AtomicU64>,
}
pub(super) fn channel(
    request: &CaptureRequest,
    stop: &Arc<AtomicBool>,
) -> (CaptureLayer, ReceiverState) {
    let capacity = (request.budget.memory_bytes / 64 / 512).clamp(1, 1024);
    let (sender, receiver) = sync_channel(capacity);
    let dropped = Arc::new(AtomicU64::new(0));
    (
        CaptureLayer {
            sender,
            stop: stop.clone(),
            dropped: dropped.clone(),
        },
        ReceiverState { receiver, dropped },
    )
}
impl CaptureLayer {
    fn enqueue(&self, kind: LifecycleKind, span: u64, name: &str) {
        if self.stop.load(Ordering::Relaxed) {
            return;
        }
        let Some(at) = monotonic_now() else {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        };
        let packet = Lifecycle {
            kind,
            span,
            at,
            name: name.chars().take(192).collect(),
            operation: pse_diagnostics::observation::current().map(|context| context.id()),
        };
        if self.sender.try_send(packet).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}
impl<S: Subscriber> Layer<S> for CaptureLayer {
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, _ctx: Context<'_, S>) {
        self.enqueue(LifecycleKind::Begin, id.into_u64(), attrs.metadata().name());
    }
    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        self.enqueue(LifecycleKind::End, id.into_u64(), "");
    }
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        self.enqueue(LifecycleKind::Instant, 0, event.metadata().name());
    }
}
pub(super) fn monotonic_now() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let now = rustix::time::clock_gettime(rustix::time::ClockId::Monotonic);
        u64::try_from(now.tv_sec)
            .ok()?
            .checked_mul(1_000_000_000)?
            .checked_add(u64::try_from(now.tv_nsec).ok()?)
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}
