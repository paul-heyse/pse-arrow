// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical outbound custody. h2 may copy a small DATA suffix into its codec
//! buffer before dropping the source Buf. Source drop alone is not an I/O fence.
use super::request::{PayloadRetention, RequestContext, RequestLifetimeRetention};
use bytes::{Buf, Bytes};
use http_body::{Body, Frame};
use std::{
    io::{self, IoSlice},
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};

#[derive(Debug, Default)]
pub(super) struct Progress {
    live: AtomicUsize,
    changed: tokio::sync::Notify,
}
impl Progress {
    fn retain(self: &Arc<Self>) -> Hold {
        self.live.fetch_add(1, Ordering::AcqRel);
        Hold(self.clone())
    }
    pub(super) fn is_drained(&self) -> bool {
        self.live.load(Ordering::Acquire) == 0
    }
    pub(super) async fn wait(&self) {
        loop {
            let wake = self.changed.notified();
            tokio::pin!(wake);
            wake.as_mut().enable();
            if self.live.load(Ordering::Acquire) == 0 {
                return;
            }
            wake.await;
        }
    }
}
#[derive(Debug)]
struct Hold(Arc<Progress>);
impl Clone for Hold {
    fn clone(&self) -> Self {
        self.0.retain()
    }
}
impl Drop for Hold {
    fn drop(&mut self) {
        self.0.live.fetch_sub(1, Ordering::AcqRel);
        self.0.changed.notify_waiters();
    }
}

struct Custody {
    _payload: Option<PayloadRetention>,
    _lifetime: Option<RequestLifetimeRetention>,
    _progress: Option<Hold>,
}
// A sole private implementation erases the recursive native owner graph at
// physical storage, after the exact allocation/read custody has been built.
trait FlushCustody: Send {
    fn fork(&self) -> Box<dyn FlushCustody>;
}
impl FlushCustody for Custody {
    fn fork(&self) -> Box<dyn FlushCustody> {
        Box::new(Self {
            _payload: self._payload.as_ref().map(PayloadRetention::fork_driver),
            _lifetime: self._lifetime.clone(),
            _progress: self._progress.clone(),
        })
    }
}
struct Pending {
    epoch: u64,
    _custody: Box<dyn FlushCustody>,
}
#[derive(Default)]
struct FlushState {
    epoch: u64,
    joined: bool,
    pending: Vec<Pending>,
}
/// Shared by the actual I/O and typed outgoing Bufs; it never parses HTTP/2.
#[derive(Default)]
pub(super) struct FlushFence {
    state: Mutex<FlushState>,
    #[cfg(test)]
    consume_hook: Mutex<Option<Box<dyn FnOnce() + Send>>>,
}
impl std::fmt::Debug for FlushFence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NativeOutboundFlushFence")
    }
}
impl FlushFence {
    fn after_consumption(&self) -> u64 {
        let epoch = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .epoch
            .saturating_add(1);
        #[cfg(test)]
        {
            let hook = self
                .consume_hook
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            if let Some(hook) = hook {
                hook();
            }
        }
        epoch
    }
    #[cfg(test)]
    pub(super) fn on_consumption(&self, hook: impl FnOnce() + Send + 'static) {
        *self
            .consume_hook
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Box::new(hook));
    }
    #[cfg(test)]
    pub(super) fn pending_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pending
            .len()
    }
    fn retire(&self, epoch: Option<u64>, custody: Box<dyn FlushCustody>) {
        let retired = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if epoch.is_some_and(|epoch| !state.joined && epoch > state.epoch) {
                state.pending.push(Pending {
                    epoch: epoch.unwrap_or(0),
                    _custody: custody,
                });
                None
            } else {
                Some(custody)
            }
        };
        drop(retired);
    }
    fn flushed(&self) {
        let retired = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.epoch = state.epoch.saturating_add(1);
            let epoch = state.epoch;
            let mut retired = Vec::new();
            let mut index = 0;
            while index < state.pending.len() {
                if state.pending[index].epoch <= epoch {
                    retired.push(state.pending.swap_remove(index));
                } else {
                    index += 1;
                }
            }
            retired
        };
        drop(retired);
    }
    /// Only the whole native task closure may discharge an unsuccessful flush.
    pub(super) fn joined(&self) {
        let retired = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.joined = true;
            std::mem::take(&mut state.pending)
        };
        drop(retired);
    }
}

pub(super) struct OwnedIo<I> {
    inner: I,
    fence: Arc<FlushFence>,
}
impl<I> OwnedIo<I> {
    pub(super) fn new(inner: I, fence: Arc<FlushFence>) -> Self {
        Self { inner, fence }
    }
}
impl<I: hyper::rt::Read + Unpin> hyper::rt::Read for OwnedIo<I> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}
impl<I: hyper::rt::Write + Unpin> hyper::rt::Write for OwnedIo<I> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write_vectored(cx, buf)
    }
    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let result = Pin::new(&mut self.inner).poll_flush(cx);
        if matches!(result, Poll::Ready(Ok(()))) {
            self.fence.flushed();
        }
        result
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// Hyper/h2 consume this actual Buf. `advance` occurs while the codec copies or
/// writes its bytes, before the associated underlying successful poll_flush.
/// The final source destructor can occur on either side of that flush.
pub(super) struct Data {
    bytes: Bytes,
    fence: Arc<FlushFence>,
    required: Option<u64>,
    custody: Option<Box<dyn FlushCustody>>,
}
impl Buf for Data {
    fn remaining(&self) -> usize {
        self.bytes.remaining()
    }
    fn chunk(&self) -> &[u8] {
        self.bytes.chunk()
    }
    fn chunks_vectored<'a>(&'a self, dst: &mut [IoSlice<'a>]) -> usize {
        self.bytes.chunks_vectored(dst)
    }
    fn advance(&mut self, count: usize) {
        if count > 0 {
            self.required = Some(self.fence.after_consumption());
        }
        self.bytes.advance(count);
    }
}
impl Drop for Data {
    fn drop(&mut self) {
        drop(std::mem::take(&mut self.bytes));
        if let Some(custody) = self.custody.take() {
            self.fence.retire(self.required, custody);
        }
    }
}

pub(super) struct OutboundBody {
    inner: Pin<Box<tonic::body::Body>>,
    fence: Arc<FlushFence>,
    custody: Box<dyn FlushCustody>,
}
impl OutboundBody {
    pub(super) fn new(
        body: tonic::body::Body,
        context: Option<&RequestContext>,
        fence: Arc<FlushFence>,
    ) -> Self {
        Self {
            inner: Box::pin(body),
            fence,
            custody: Box::new(Custody {
                _payload: context
                    .and_then(RequestContext::payload_owner)
                    .map(|owner| owner.retain_driver()),
                _lifetime: context
                    .and_then(RequestContext::lifetime_owner)
                    .map(|owner| owner.retain()),
                _progress: context.map(|context| context.outbound().retain()),
            }),
        }
    }
}
impl Body for OutboundBody {
    type Data = Data;
    type Error = tonic::Status;
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Data>, tonic::Status>>> {
        match self.inner.as_mut().poll_frame(cx) {
            Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(Ok(frame.map_data(|bytes| Data {
                bytes,
                fence: self.fence.clone(),
                required: None,
                custody: Some(self.custody.fork()),
            })))),
            Poll::Ready(Some(Err(error))) => Poll::Ready(Some(Err(error))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }
    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}
