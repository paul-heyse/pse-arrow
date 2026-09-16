// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One-shot faults around actual immutable object operations and conditional ref writes.
#![allow(
    dead_code,
    clippy::expect_used,
    reason = "shared test factories and controlled poisoned-lock failures"
)]

use futures_core::stream::BoxStream;
use object_store::{
    CopyOptions, GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta, ObjectStore,
    ObjectStoreExt, PutMultipartOptions, PutOptions, PutPayload, PutResult, Result, path::Path,
};
use pse_ids::CancellationToken;
use std::sync::{Arc, Mutex};

/// A specific one-shot failure, corruption or cooperative interruption.
#[derive(Clone, Debug)]
pub(crate) enum Fault {
    /// Return before the backend is called.
    FailBefore,
    /// Refuse exactly one conditional write.
    Precondition,
    /// Truncate an existing object's actual bytes just before a read.
    Truncate,
    /// Flip one stored byte just before a read, preserving its length.
    FlipByte,
    /// Cancel after a successful immutable object write.
    Cancel(CancellationToken),
    /// Commit the conditional write, then lose its response.
    LostResponse,
}
/// Explicit operation/path/call selection independent of hashing or object identity.
#[derive(Clone, Debug)]
pub(crate) struct FaultPlan {
    /// `put` or `get`.
    pub operation: &'static str,
    /// Literal path prefix.
    pub prefix: String,
    /// One-based matching call to inject.
    pub call: usize,
    /// The selected action.
    pub fault: Fault,
}
#[derive(Debug)]
struct State {
    plan: Option<FaultPlan>,
    matches: usize,
    fired: usize,
    puts: Vec<String>,
}
/// Delegates every normal operation to an actual backend and injects one chosen fault.
#[derive(Debug)]
pub(crate) struct FaultStore {
    inner: Arc<dyn ObjectStore>,
    state: Mutex<State>,
}
impl FaultStore {
    /// A wrapper initially behaving exactly like the supplied backend.
    pub(crate) fn new(inner: Arc<dyn ObjectStore>) -> Arc<Self> {
        Arc::new(Self {
            inner,
            state: Mutex::new(State {
                plan: None,
                matches: 0,
                fired: 0,
                puts: Vec::new(),
            }),
        })
    }
    /// Replace the next one-shot selection, resetting observation counters.
    pub(crate) fn arm(&self, plan: FaultPlan) {
        *self.state.lock().expect("fault lock") = State {
            plan: Some(plan),
            matches: 0,
            fired: 0,
            puts: Vec::new(),
        };
    }
    /// How often the selected action actually fired.
    pub(crate) fn fired(&self) -> usize {
        self.state.lock().expect("fault lock").fired
    }
    /// Discard preceding writes before observing one actual publication attempt.
    pub(crate) fn clear_trace(&self) {
        self.state.lock().expect("fault lock").puts.clear();
    }
    /// Exact ordered write destinations, including the operation interrupted before I/O.
    pub(crate) fn put_trace(&self) -> Vec<String> {
        self.state.lock().expect("fault lock").puts.clone()
    }
    fn action(&self, operation: &str, path: &Path) -> Option<Fault> {
        let mut state = self.state.lock().expect("fault lock");
        if operation == "put" {
            state.puts.push(path.to_string());
        }
        let plan = state.plan.as_ref()?;
        if operation != plan.operation || !path.as_ref().starts_with(&plan.prefix) {
            return None;
        }
        let call = plan.call;
        state.matches += 1;
        if state.matches != call {
            return None;
        }
        state.fired += 1;
        state.plan.take().map(|plan| plan.fault)
    }
}
impl std::fmt::Display for FaultStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FaultStore({})", self.inner)
    }
}
fn failure(path: &Path, precondition: bool) -> object_store::Error {
    let source = Box::new(std::io::Error::other(
        "injected one-shot object-store fault",
    ));
    if precondition {
        object_store::Error::Precondition {
            path: path.to_string(),
            source,
        }
    } else {
        object_store::Error::Generic {
            store: "fault fixture",
            source,
        }
    }
}
#[async_trait::async_trait]
impl ObjectStore for FaultStore {
    async fn put_opts(
        &self,
        location: &Path,
        payload: PutPayload,
        opts: PutOptions,
    ) -> Result<PutResult> {
        let action = self.action("put", location);
        match action {
            Some(Fault::FailBefore) => return Err(failure(location, false)),
            Some(Fault::Precondition) => return Err(failure(location, true)),
            _ => {}
        }
        let result = self.inner.put_opts(location, payload, opts).await?;
        if matches!(action, Some(Fault::LostResponse)) {
            return Err(failure(location, false));
        }
        if let Some(Fault::Cancel(cancel)) = action {
            cancel.cancel();
        }
        Ok(result)
    }
    async fn put_multipart_opts(
        &self,
        location: &Path,
        opts: PutMultipartOptions,
    ) -> Result<Box<dyn MultipartUpload>> {
        self.inner.put_multipart_opts(location, opts).await
    }
    async fn get_opts(&self, location: &Path, opts: GetOptions) -> Result<GetResult> {
        match self.action("get", location) {
            Some(Fault::FailBefore) => return Err(failure(location, false)),
            Some(fault @ (Fault::Truncate | Fault::FlipByte)) => {
                let mut bytes = self.inner.get(location).await?.bytes().await?.to_vec();
                match fault {
                    Fault::Truncate => {
                        bytes.truncate(bytes.len() / 2);
                    }
                    Fault::FlipByte => {
                        if let Some(first) = bytes.first_mut() {
                            *first ^= 0xff;
                        }
                    }
                    _ => {}
                }
                self.inner.put(location, bytes.into()).await?;
            }
            _ => {}
        }
        self.inner.get_opts(location, opts).await
    }
    fn delete_stream(
        &self,
        locations: BoxStream<'static, Result<Path>>,
    ) -> BoxStream<'static, Result<Path>> {
        self.inner.delete_stream(locations)
    }
    fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, Result<ObjectMeta>> {
        self.inner.list(prefix)
    }
    async fn list_with_delimiter(&self, prefix: Option<&Path>) -> Result<ListResult> {
        self.inner.list_with_delimiter(prefix).await
    }
    async fn copy_opts(&self, from: &Path, to: &Path, opts: CopyOptions) -> Result<()> {
        self.inner.copy_opts(from, to, opts).await
    }
}
