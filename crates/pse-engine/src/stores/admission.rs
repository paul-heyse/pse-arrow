// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One I/O gate across every store. Native `LimitStore` owns a separate private gate.
use futures_util::{
    StreamExt,
    stream::{self, BoxStream},
};
use object_store::{
    CopyOptions, Error, GetOptions, GetResult, GetResultPayload, ListResult, MultipartUpload,
    ObjectMeta, ObjectStore, PutMultipartOptions, PutOptions, PutPayload, PutResult, RenameOptions,
    Result, UploadPart, path::Path,
};
use std::{ops::Range, sync::Arc};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Debug)]
pub(super) struct SharedIo {
    pub inner: Arc<dyn ObjectStore>,
    pub gate: Arc<Semaphore>,
}
impl std::fmt::Display for SharedIo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SharedIo({})", self.inner)
    }
}
// Scope is installed only while polling delegated work. Nested wrappers borrow
// the same owner; unrelated tasks and idle listing streams hold no borrowed scope.
tokio::task_local! { static ACTIVE_IO: Arc<IoLease>; }
#[derive(Debug)]
struct IoLease {
    gate: Arc<Semaphore>,
    _permit: OwnedSemaphorePermit,
    parent: Option<Arc<Self>>,
}
async fn acquire(gate: Arc<Semaphore>) -> Result<Arc<IoLease>> {
    let parent = ACTIVE_IO.try_with(Arc::clone).ok();
    let mut current = parent.as_ref();
    while let Some(owner) = current {
        if Arc::ptr_eq(&owner.gate, &gate) {
            return Ok(owner.clone());
        }
        current = owner.parent.as_ref();
    }
    let permit = gate
        .clone()
        .acquire_owned()
        .await
        .map_err(|source| Error::Generic {
            store: "pse shared I/O",
            source: Box::new(source),
        })?;
    Ok(Arc::new(IoLease {
        gate,
        _permit: permit,
        parent,
    }))
}
fn retain<T: Send + 'static>(
    stream: BoxStream<'static, T>,
    permit: Arc<IoLease>,
) -> BoxStream<'static, T> {
    stream::unfold((stream, permit), |(mut stream, permit)| async move {
        ACTIVE_IO
            .scope(permit.clone(), stream.next())
            .await
            .map(|item| (item, (stream, permit)))
    })
    .fuse()
    .boxed()
}
fn gated<T: Send + 'static>(
    stream: BoxStream<'static, Result<T>>,
    gate: Arc<Semaphore>,
) -> BoxStream<'static, Result<T>> {
    stream::unfold(Some((stream, gate)), |state| async move {
        let (mut stream, gate) = state?;
        let permit = match acquire(gate.clone()).await {
            Ok(permit) => permit,
            Err(error) => return Some((Err(error), None)),
        };
        // A consumer may read an object between list items. Release at every
        // yield; retain through actual body reads separately in `get_opts`.
        ACTIVE_IO
            .scope(permit, stream.next())
            .await
            .map(|item| (item, Some((stream, gate))))
    })
    .fuse()
    .boxed()
}
#[async_trait::async_trait]
impl ObjectStore for SharedIo {
    async fn put_opts(
        &self,
        path: &Path,
        payload: PutPayload,
        opts: PutOptions,
    ) -> Result<PutResult> {
        let permit = acquire(self.gate.clone()).await?;
        ACTIVE_IO
            .scope(permit, self.inner.put_opts(path, payload, opts))
            .await
    }
    async fn put_multipart_opts(
        &self,
        path: &Path,
        opts: PutMultipartOptions,
    ) -> Result<Box<dyn MultipartUpload>> {
        let permit = acquire(self.gate.clone()).await?;
        Ok(Box::new(SharedUpload {
            inner: ACTIVE_IO
                .scope(permit, self.inner.put_multipart_opts(path, opts))
                .await?,
            gate: self.gate.clone(),
        }))
    }
    async fn get_opts(&self, path: &Path, opts: GetOptions) -> Result<GetResult> {
        let permit = acquire(self.gate.clone()).await?;
        let result = ACTIVE_IO
            .scope(permit.clone(), self.inner.get_opts(path, opts))
            .await?;
        // Convert even a local file payload through the native stream adapter, so
        // the gate covers actual file reads rather than only opening the handle.
        let (meta, range, attributes) = (
            result.meta.clone(),
            result.range.clone(),
            result.attributes.clone(),
        );
        Ok(GetResult {
            meta,
            range,
            attributes,
            payload: GetResultPayload::Stream(retain(result.into_stream(), permit)),
        })
    }
    async fn get_ranges(&self, path: &Path, ranges: &[Range<u64>]) -> Result<Vec<bytes::Bytes>> {
        let permit = acquire(self.gate.clone()).await?;
        ACTIVE_IO
            .scope(permit, self.inner.get_ranges(path, ranges))
            .await
    }
    fn delete_stream(
        &self,
        paths: BoxStream<'static, Result<Path>>,
    ) -> BoxStream<'static, Result<Path>> {
        gated(self.inner.delete_stream(paths), self.gate.clone())
    }
    fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, Result<ObjectMeta>> {
        gated(self.inner.list(prefix), self.gate.clone())
    }
    async fn list_with_delimiter(&self, prefix: Option<&Path>) -> Result<ListResult> {
        let permit = acquire(self.gate.clone()).await?;
        ACTIVE_IO
            .scope(permit, self.inner.list_with_delimiter(prefix))
            .await
    }
    async fn copy_opts(&self, from: &Path, to: &Path, opts: CopyOptions) -> Result<()> {
        let permit = acquire(self.gate.clone()).await?;
        ACTIVE_IO
            .scope(permit, self.inner.copy_opts(from, to, opts))
            .await
    }
    async fn rename_opts(&self, from: &Path, to: &Path, opts: RenameOptions) -> Result<()> {
        let permit = acquire(self.gate.clone()).await?;
        ACTIVE_IO
            .scope(permit, self.inner.rename_opts(from, to, opts))
            .await
    }
}
#[derive(Debug)]
struct SharedUpload {
    inner: Box<dyn MultipartUpload>,
    gate: Arc<Semaphore>,
}
#[async_trait::async_trait]
impl MultipartUpload for SharedUpload {
    fn put_part(&mut self, payload: PutPayload) -> UploadPart {
        let (future, gate) = (self.inner.put_part(payload), self.gate.clone());
        Box::pin(async move {
            let permit = acquire(gate).await?;
            ACTIVE_IO.scope(permit, future).await
        })
    }
    async fn complete(&mut self) -> Result<PutResult> {
        let permit = acquire(self.gate.clone()).await?;
        ACTIVE_IO.scope(permit, self.inner.complete()).await
    }
    async fn abort(&mut self) -> Result<()> {
        let permit = acquire(self.gate.clone()).await?;
        ACTIVE_IO.scope(permit, self.inner.abort()).await
    }
}

#[cfg(test)]
mod durability_unit {
    use super::*;
    use object_store::memory;
    #[tokio::test]
    async fn exhausted_nested_streams_release_admission_and_remain_exhausted() {
        let gate = Arc::new(Semaphore::new(1));
        let store = SharedIo {
            inner: Arc::new(SharedIo {
                inner: Arc::new(memory::InMemory::new()),
                gate: gate.clone(),
            }),
            gate: gate.clone(),
        };
        let work = async {
            let path = Path::from("exhausted");
            store
                .put_opts(&path, "body".into(), PutOptions::default())
                .await
                .unwrap();
            let mut body = store
                .get_opts(&path, GetOptions::default())
                .await
                .unwrap()
                .into_stream();
            assert_eq!(gate.available_permits(), 0);
            assert_eq!(body.next().await.unwrap().unwrap(), "body");
            for _ in 0..3 {
                assert!(body.next().await.is_none());
                assert_eq!(gate.available_permits(), 1);
            }
            let mut listing = store.list(None);
            assert_eq!(listing.next().await.unwrap().unwrap().location, path);
            let mut deletion = store.delete_stream(stream::iter([Ok(path)]).boxed());
            assert!(deletion.next().await.unwrap().is_ok());
            for _ in 0..3 {
                assert!(listing.next().await.is_none());
                assert!(deletion.next().await.is_none());
                assert_eq!(gate.available_permits(), 1);
            }
            gate.close();
            let mut closed = store.list(None);
            assert!(closed.next().await.unwrap().is_err());
            for _ in 0..3 {
                assert!(closed.next().await.is_none());
            }
        };
        tokio::time::timeout(std::time::Duration::from_secs(5), work)
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn nested_wrappers_borrow_the_gate_and_idle_lists_release_it() {
        let gate = Arc::new(Semaphore::new(1));
        let inner = SharedIo {
            inner: Arc::new(memory::InMemory::new()),
            gate: gate.clone(),
        };
        let outer = SharedIo {
            inner: Arc::new(inner),
            gate: gate.clone(),
        };
        let work = async {
            let path = Path::from("nested");
            outer
                .put_opts(&path, "value".into(), PutOptions::default())
                .await
                .unwrap();
            let mut listed = outer.list(None);
            assert_eq!(listed.next().await.unwrap().unwrap().location, path);
            assert_eq!(gate.available_permits(), 1);
            let body = outer.get_opts(&path, GetOptions::default()).await.unwrap();
            assert_eq!(gate.available_permits(), 0);
            assert_eq!(body.bytes().await.unwrap(), "value");
            assert_eq!(gate.available_permits(), 1);
            assert!(listed.next().await.is_none());
            let mut upload = outer
                .put_multipart_opts(&path, PutMultipartOptions::default())
                .await
                .unwrap();
            upload.put_part("part".into()).await.unwrap();
            upload.complete().await.unwrap();
            assert_eq!(gate.available_permits(), 1);
        };
        tokio::time::timeout(std::time::Duration::from_secs(5), work)
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn stores_share_one_gate_through_body_consumption_and_drop() {
        let gate = Arc::new(Semaphore::new(1));
        let first = SharedIo {
            inner: Arc::new(memory::InMemory::new()),
            gate: gate.clone(),
        };
        let second = SharedIo {
            inner: Arc::new(memory::InMemory::new()),
            gate: gate.clone(),
        };
        let path = Path::from("value");
        first
            .put_opts(&path, "a".into(), PutOptions::default())
            .await
            .unwrap();
        let body = first.get_opts(&path, GetOptions::default()).await.unwrap();
        assert_eq!(gate.available_permits(), 0);
        let next = second.put_opts(&path, "b".into(), PutOptions::default());
        tokio::pin!(next);
        assert!(futures_util::poll!(&mut next).is_pending());
        drop(body);
        next.await.unwrap();
        assert_eq!(gate.available_permits(), 1);
        let body = first.get_opts(&path, GetOptions::default()).await.unwrap();
        assert_eq!(body.bytes().await.unwrap(), "a");
        assert_eq!(gate.available_permits(), 1);
    }
}
