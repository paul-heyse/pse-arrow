// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native IO instrumentation; preserves validators, ranges and conditional writes.
use futures_util::{TryStreamExt, stream::BoxStream};
use object_store::{ObjectStore, path::Path};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
/// Native object-store counters for consumed payloads and requested operations.
#[derive(Debug)]
pub struct CountingStore {
    inner: Arc<dyn ObjectStore>,
    gets: AtomicUsize,
    heads: AtomicUsize,
    ranges: AtomicUsize,
    multipart: AtomicUsize,
    bytes: Arc<AtomicUsize>,
    lists: AtomicUsize,
    log_gets: AtomicUsize,
    checkpoint_gets: AtomicUsize,
    parquet_gets: AtomicUsize,
    puts: AtomicUsize,
}
impl std::fmt::Display for CountingStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CountingStore")
    }
}
#[async_trait::async_trait]
impl ObjectStore for CountingStore {
    async fn put_opts(
        &self,
        path: &Path,
        payload: object_store::PutPayload,
        options: object_store::PutOptions,
    ) -> object_store::Result<object_store::PutResult> {
        self.puts.fetch_add(1, Ordering::Relaxed);
        self.inner.put_opts(path, payload, options).await
    }
    async fn put_multipart_opts(
        &self,
        path: &Path,
        options: object_store::PutMultipartOptions,
    ) -> object_store::Result<Box<dyn object_store::MultipartUpload>> {
        self.multipart.fetch_add(1, Ordering::Relaxed);
        self.inner.put_multipart_opts(path, options).await
    }
    async fn get_opts(
        &self,
        path: &Path,
        options: object_store::GetOptions,
    ) -> object_store::Result<object_store::GetResult> {
        if options.head {
            self.heads.fetch_add(1, Ordering::Relaxed);
        } else {
            self.gets.fetch_add(1, Ordering::Relaxed);
        }
        if options.range.is_some() {
            self.ranges.fetch_add(1, Ordering::Relaxed);
        }
        let kind = if path
            .filename()
            .is_some_and(|name| name == "_last_checkpoint" || name.contains(".checkpoint."))
        {
            &self.checkpoint_gets
        } else if path.parts().any(|part| part.as_ref() == "_delta_log") {
            &self.log_gets
        } else {
            &self.parquet_gets
        };
        if !options.head {
            kind.fetch_add(1, Ordering::Relaxed);
        }
        let result = self.inner.get_opts(path, options).await?;
        let bytes = self.bytes.clone();
        let meta = result.meta.clone();
        let range = result.range.clone();
        let attributes = result.attributes.clone();
        let payload = object_store::GetResultPayload::Stream(Box::pin(
            result.into_stream().inspect_ok(move |chunk| {
                bytes.fetch_add(chunk.len(), Ordering::Relaxed);
            }),
        ));
        Ok(object_store::GetResult {
            payload,
            meta,
            range,
            attributes,
        })
    }
    fn delete_stream(
        &self,
        paths: BoxStream<'static, object_store::Result<Path>>,
    ) -> BoxStream<'static, object_store::Result<Path>> {
        self.inner.delete_stream(paths)
    }
    fn list(
        &self,
        prefix: Option<&Path>,
    ) -> BoxStream<'static, object_store::Result<object_store::ObjectMeta>> {
        self.lists.fetch_add(1, Ordering::Relaxed);
        self.inner.list(prefix)
    }
    async fn list_with_delimiter(
        &self,
        prefix: Option<&Path>,
    ) -> object_store::Result<object_store::ListResult> {
        self.lists.fetch_add(1, Ordering::Relaxed);
        self.inner.list_with_delimiter(prefix).await
    }
    async fn copy_opts(
        &self,
        from: &Path,
        to: &Path,
        options: object_store::CopyOptions,
    ) -> object_store::Result<()> {
        self.inner.copy_opts(from, to, options).await
    }
}

impl CountingStore {
    /// Wrap the supplied backend without changing its request options.
    pub fn new(inner: Arc<dyn ObjectStore>) -> Arc<Self> {
        Arc::new(Self {
            inner,
            gets: AtomicUsize::new(0),
            heads: AtomicUsize::new(0),
            ranges: AtomicUsize::new(0),
            multipart: AtomicUsize::new(0),
            puts: AtomicUsize::new(0),
            bytes: Arc::default(),
            lists: AtomicUsize::new(0),
            log_gets: AtomicUsize::new(0),
            checkpoint_gets: AtomicUsize::new(0),
            parquet_gets: AtomicUsize::new(0),
        })
    }
    /// Reset counters between explicitly separated observations.
    pub fn reset(&self) {
        for value in [
            &self.gets,
            &self.heads,
            &self.ranges,
            &self.multipart,
            &self.puts,
            &self.bytes,
            &self.lists,
            &self.log_gets,
            &self.checkpoint_gets,
            &self.parquet_gets,
        ] {
            value.store(0, Ordering::Relaxed);
        }
    }
    /// Snapshot operation counts and actually consumed payload bytes.
    pub fn report(&self) -> serde_json::Value {
        serde_json::json!({"gets":self.gets.load(Ordering::Relaxed),"heads":self.heads.load(Ordering::Relaxed),"ranges":self.ranges.load(Ordering::Relaxed),"multipart":self.multipart.load(Ordering::Relaxed),"bytes_consumed":self.bytes.load(Ordering::Relaxed),"puts":self.puts.load(Ordering::Relaxed),"lists":self.lists.load(Ordering::Relaxed),"log_gets":self.log_gets.load(Ordering::Relaxed),"checkpoint_gets":self.checkpoint_gets.load(Ordering::Relaxed),"parquet_gets":self.parquet_gets.load(Ordering::Relaxed)})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use object_store::ObjectStoreExt;
    #[tokio::test]
    async fn native_unit_requests_are_distinct_from_consumed_payload() {
        let store = CountingStore::new(Arc::new(object_store::memory::InMemory::new()));
        let path = Path::from("sample.parquet");
        store.put(&path, "abcdef".into()).await.unwrap();
        store.reset();
        store.head(&path).await.unwrap();
        assert_eq!(store.report()["heads"], 1);
        assert_eq!(store.report()["gets"], 0);
        let pending = store.get(&path).await.unwrap();
        assert_eq!(store.report()["bytes_consumed"], 0);
        drop(pending);
        assert_eq!(store.report()["bytes_consumed"], 0);
        let bytes = store.get_range(&path, 1..3).await.unwrap();
        assert_eq!(bytes.as_ref(), b"bc");
        assert_eq!(store.report()["ranges"], 1);
        assert_eq!(store.report()["bytes_consumed"], 2);
        assert_eq!(store.report()["parquet_gets"], 2);
    }
}
