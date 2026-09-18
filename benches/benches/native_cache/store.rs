// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native IO instrumentation; preserves validators, ranges and conditional writes.
use futures_util::{TryStreamExt, stream::BoxStream};
use object_store::{ObjectStore, path::Path};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
#[derive(Debug)]
pub(super) struct CountingStore {
    inner: Arc<dyn ObjectStore>,
    gets: AtomicUsize,
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
        self.inner.put_multipart_opts(path, options).await
    }
    async fn get_opts(
        &self,
        path: &Path,
        options: object_store::GetOptions,
    ) -> object_store::Result<object_store::GetResult> {
        self.gets.fetch_add(1, Ordering::Relaxed);
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
        kind.fetch_add(1, Ordering::Relaxed);
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
    pub(super) fn new(inner: Arc<dyn ObjectStore>) -> Arc<Self> {
        Arc::new(Self {
            inner,
            gets: AtomicUsize::new(0),
            puts: AtomicUsize::new(0),
            bytes: Arc::default(),
            lists: AtomicUsize::new(0),
            log_gets: AtomicUsize::new(0),
            checkpoint_gets: AtomicUsize::new(0),
            parquet_gets: AtomicUsize::new(0),
        })
    }
    pub(super) fn reset(&self) {
        for value in [
            &self.gets,
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
    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"gets":self.gets.load(Ordering::Relaxed),"bytes_consumed":self.bytes.load(Ordering::Relaxed),"puts":self.puts.load(Ordering::Relaxed),"lists":self.lists.load(Ordering::Relaxed),"log_gets":self.log_gets.load(Ordering::Relaxed),"checkpoint_gets":self.checkpoint_gets.load(Ordering::Relaxed),"parquet_gets":self.parquet_gets.load(Ordering::Relaxed)})
    }
}
