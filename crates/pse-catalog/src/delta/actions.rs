// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact native commit observations, admitted before retained bytes/JSON allocation.
use datafusion::{
    common::{DataFusionError, Result},
    execution::{
        memory_pool::{MemoryConsumer, MemoryReservation},
        session_state::SessionState,
    },
};
use deltalake::{DeltaTable, kernel::Action};
use futures_util::TryStreamExt;
use object_store::{GetOptions, ObjectStore, ObjectStoreExt};
use std::sync::Arc;

pub(super) struct CommitActions {
    bytes: Vec<u8>,
    pub owner: MemoryReservation,
}
impl CommitActions {
    pub(super) fn iter(&self) -> impl Iterator<Item = Result<Action>> + '_ {
        let mut actions = serde_json::Deserializer::from_slice(&self.bytes).into_iter::<Action>();
        let mut seen = false;
        std::iter::from_fn(move || match actions.next() {
            Some(action) => {
                seen = true;
                Some(action.map_err(|error| DataFusionError::External(Box::new(error))))
            }
            None if !seen => {
                seen = true;
                Some(Err(invalid("empty Delta commit")))
            }
            None => None,
        })
    }
    /// Consume every action, including malformed tails; consumers retain only their summary.
    pub(super) fn visit(&self, mut visitor: impl FnMut(Action) -> Result<()>) -> Result<()> {
        for action in self.iter() {
            visitor(action?)?;
        }
        Ok(())
    }
}

pub(super) async fn read(
    table: &DeltaTable,
    version: u64,
    state: &SessionState,
    cancel: &pse_columnar::CancellationToken,
) -> Result<CommitActions> {
    let path = deltalake::logstore::commit_uri_from_version(Some(version));
    read_store(
        table.log_store().object_store(None),
        &path,
        &state.runtime_env().memory_pool,
        cancel,
    )
    .await
}

async fn read_store(
    store: Arc<dyn ObjectStore>,
    path: &object_store::path::Path,
    pool: &Arc<dyn datafusion::execution::memory_pool::MemoryPool>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<CommitActions> {
    cancel.checkpoint().map_err(pse_columnar::external)?;
    let metadata = cancel
        .until_cancelled(store.head(path))
        .await
        .map_err(pse_columnar::external)?
        .map_err(external)?;
    let size = usize::try_from(metadata.size)
        .map_err(|error| DataFusionError::External(Box::new(error)))?;
    let extent = size
        .checked_mul(96)
        .and_then(|n| n.checked_add(4096))
        .ok_or_else(|| invalid("commit decoding extent overflows"))?;
    let owner = MemoryConsumer::new("pse.delta.commit_actions").register(pool);
    owner.try_grow(extent)?;
    if size == 0 {
        return Err(invalid("empty Delta commit"));
    }
    let options = GetOptions::default()
        .with_if_match(metadata.e_tag.clone())
        .with_version(metadata.version.clone())
        .with_range(Some(0..metadata.size));
    let result = cancel
        .until_cancelled(store.get_opts(path, options))
        .await
        .map_err(pse_columnar::external)?
        .map_err(external)?;
    finish_read(result, &metadata, owner, cancel).await
}

async fn finish_read(
    result: object_store::GetResult,
    metadata: &object_store::ObjectMeta,
    owner: MemoryReservation,
    cancel: &pse_columnar::CancellationToken,
) -> Result<CommitActions> {
    let size = usize::try_from(metadata.size)
        .map_err(|error| DataFusionError::External(Box::new(error)))?;
    if result.range != (0..metadata.size)
        || result.meta.size != metadata.size
        || result.meta.e_tag != metadata.e_tag
        || result.meta.version != metadata.version
    {
        return Err(invalid("commit changed during observation"));
    }
    let mut stream = result.into_stream();
    let mut bytes = Vec::with_capacity(size);
    while let Some(chunk) = cancel
        .until_cancelled(stream.try_next())
        .await
        .map_err(pse_columnar::external)?
        .map_err(external)?
    {
        if chunk.len() > size - bytes.len() {
            return Err(invalid("commit exceeds admitted extent"));
        }
        bytes.extend_from_slice(&chunk);
    }
    if bytes.len() != size {
        return Err(invalid("truncated Delta commit"));
    }
    Ok(CommitActions { bytes, owner })
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Execution(reason.into())
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}

#[cfg(test)]
mod delta_boundary_unit {
    use super::*;
    use datafusion::execution::memory_pool::{GreedyMemoryPool, MemoryPool};
    use pse_columnar::CancellationToken;

    #[tokio::test]
    async fn exact_log_read_admits_memory_and_validates_the_entire_json_tail() {
        let store = Arc::new(object_store::memory::InMemory::new());
        let path = object_store::path::Path::from("_delta_log/00000000000000000007.json");
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128 * 1024));
        let good = br#"{"txn":{"appId":"request","version":1}}"#;
        for (contents, valid) in [
            (good.to_vec(), true),
            ([good.as_slice(), b"\n{"].concat(), false),
            (b"   ".to_vec(), false),
        ] {
            store.put(&path, contents.into()).await.unwrap();
            let actions = read_store(store.clone(), &path, &pool, &CancellationToken::new())
                .await
                .unwrap();
            assert!(pool.reserved() > 0);
            assert_eq!(actions.visit(|_| Ok(())).is_ok(), valid);
            drop(actions);
            assert_eq!(pool.reserved(), 0);
        }
        let tiny: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(1));
        assert!(
            read_store(store.clone(), &path, &tiny, &CancellationToken::new())
                .await
                .is_err()
        );
        assert_eq!(tiny.reserved(), 0);
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(
            read_store(store.clone(), &path, &pool, &cancel)
                .await
                .is_err()
        );
        assert!(
            read_store(store, &"missing".into(), &pool, &CancellationToken::new())
                .await
                .is_err()
        );
        assert_eq!(pool.reserved(), 0);
    }
    #[tokio::test]
    async fn changed_truncated_and_oversized_payloads_refuse_without_leaking_reservations() {
        use futures_util::StreamExt;
        let store = object_store::memory::InMemory::new();
        let path = object_store::path::Path::from("commit");
        store.put(&path, b"abc".as_slice().into()).await.unwrap();
        let metadata = store.head(&path).await.unwrap();
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(16_384));
        for payload in [vec![0; 2], vec![0; 4]] {
            let mut result = store.get(&path).await.unwrap();
            result.payload = object_store::GetResultPayload::Stream(
                futures_util::stream::iter(vec![Ok(payload.into())]).boxed(),
            );
            let owner = MemoryConsumer::new("fake payload").register(&pool);
            owner.try_grow(8192).unwrap();
            assert!(
                finish_read(result, &metadata, owner, &CancellationToken::new())
                    .await
                    .is_err()
            );
            assert_eq!(pool.reserved(), 0);
        }
        let mut result = store.get(&path).await.unwrap();
        result.meta.e_tag = Some("replacement".into());
        let owner = MemoryConsumer::new("changed payload").register(&pool);
        owner.try_grow(8192).unwrap();
        assert!(
            finish_read(result, &metadata, owner, &CancellationToken::new())
                .await
                .is_err()
        );
        assert_eq!(pool.reserved(), 0);
    }
}
