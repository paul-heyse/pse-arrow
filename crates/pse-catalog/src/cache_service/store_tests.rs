// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! No Delta replay: verify the registered IO decorator seam for all object kinds.
use datafusion::execution::{runtime_env::RuntimeEnv, session_state::SessionStateBuilder};
use futures_util::stream::BoxStream;
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt};
use pse_engine::cache_service::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Default)]
pub(super) struct CountingStore {
    inner: object_store::memory::InMemory,
    pub(super) gets: AtomicUsize,
    pub(super) lists: AtomicUsize,
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
        self.inner.get_opts(path, options).await
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

#[tokio::test]
async fn log_checkpoint_and_data_share_the_actual_registered_store_and_conditions() {
    let root = url::Url::parse("memory://cache-unit/table/").unwrap();
    let store = Arc::new(CountingStore::default());
    let runtime = RuntimeEnv::default();
    runtime.register_object_store(&root, store.clone());
    let state = SessionStateBuilder::new()
        .with_runtime_env(Arc::new(runtime))
        .with_default_features()
        .build();
    let native = crate::delta::provider::table_builder(root.clone(), &state)
        .unwrap()
        .build()
        .unwrap();
    let log = native.log_store();
    let delta_store = log.object_store(None);
    for path in [
        "_delta_log/00000000000000000000.json",
        "_delta_log/00000000000000000010.checkpoint.parquet",
        "data/part.parquet",
    ] {
        let path = Path::from(path);
        delta_store
            .put_opts(
                &path,
                bytes::Bytes::from_static(b"first").into(),
                object_store::PutMode::Create.into(),
            )
            .await
            .unwrap();
        assert!(
            delta_store
                .put_opts(
                    &path,
                    bytes::Bytes::from_static(b"wrong").into(),
                    object_store::PutMode::Create.into()
                )
                .await
                .is_err()
        );
        assert_eq!(
            delta_store.get(&path).await.unwrap().bytes().await.unwrap(),
            bytes::Bytes::from_static(b"first")
        );
    }
    assert_eq!(store.gets.load(Ordering::Relaxed), 3);
    assert_eq!(store.puts.load(Ordering::Relaxed), 6);
    let pool = state.runtime_env().memory_pool.clone();
    let mut policy = CacheBudget::disabled(1);
    policy.inspection_bytes = 4096;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let before = service.generation(&root, store.clone()).unwrap();
    assert_eq!(service.generation(&root, store).unwrap(), before);
    let replacement = Arc::new(CountingStore::default());
    assert_ne!(service.generation(&root, replacement).unwrap(), before);
}
