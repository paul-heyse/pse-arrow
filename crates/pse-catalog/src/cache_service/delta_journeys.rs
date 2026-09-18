// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! C12 integration only, despite living under --lib: actual Delta commits and replay.
use super::{CacheBudget, NativeCacheService, snapshot::LoadRequirement};
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::{DataType, Field, Schema},
    },
    execution::{
        config::SessionConfig,
        memory_pool::{GreedyMemoryPool, MemoryPool},
        runtime_env::RuntimeEnvBuilder,
        session_state::SessionStateBuilder,
    },
};
use std::sync::Arc;

#[path = "../../tests/support/kernel_checksum.rs"]
mod kernel_checksum;

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "ordered integration fixture compares exact native states and file sets across updates"
)]
async fn exact_versions_load_classes_and_seeded_refresh_match_fresh_native_files() {
    let directory = tempfile::tempdir().unwrap();
    let root = url::Url::from_directory_path(directory.path()).unwrap();
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128 << 20));
    let runtime = RuntimeEnvBuilder::new()
        .with_memory_pool(pool.clone())
        .build_arc()
        .unwrap();
    let mut policy = CacheBudget::for_memory(128 << 20);
    policy.crc_replay_max_commits = 16;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let state = Arc::new(
        SessionStateBuilder::new()
            .with_default_features()
            .with_runtime_env(runtime)
            .with_config(SessionConfig::new().with_extension(service.clone()))
            .with_query_planner(Arc::new(crate::session::planner::UnifiedPlanner::default()))
            .build(),
    );
    let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));
    let batch =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![1, 2, 3]))]).unwrap();
    let table = crate::delta::provider::table_builder(root.clone(), &state)
        .unwrap()
        .build()
        .unwrap()
        .write([batch.clone()])
        .with_session_state(state.clone())
        .with_session_fallback_policy(
            deltalake::delta_datafusion::SessionFallbackPolicy::RequireSessionState,
        )
        .await
        .unwrap();
    let metadata = service
        .open_snapshot(
            root.clone(),
            Some(0),
            LoadRequirement::Metadata,
            state.clone(),
        )
        .await
        .unwrap();
    let query = service
        .open_snapshot(root.clone(), Some(0), LoadRequirement::Query, state.clone())
        .await
        .unwrap();
    assert!(metadata.table.get_file_uris().unwrap().next().is_none());
    let original_files = query.table.get_file_uris().unwrap().collect::<Vec<_>>();
    assert_eq!(
        original_files,
        table.get_file_uris().unwrap().collect::<Vec<_>>()
    );
    assert_eq!(original_files.len(), 1);
    let table = table
        .write([batch])
        .with_session_state(state.clone())
        .with_session_fallback_policy(
            deltalake::delta_datafusion::SessionFallbackPolicy::RequireSessionState,
        )
        .await
        .unwrap();
    let next = service
        .open_snapshot(root.clone(), Some(1), LoadRequirement::Query, state.clone())
        .await
        .unwrap();
    assert_eq!(
        next.table
            .get_file_uris()
            .unwrap()
            .collect::<std::collections::BTreeSet<_>>(),
        table.get_file_uris().unwrap().collect()
    );
    assert_eq!(
        query.table.get_file_uris().unwrap().collect::<Vec<_>>(),
        original_files
    );
    assert!(
        service
            .open_snapshot(root.clone(), Some(2), LoadRequirement::Query, state.clone())
            .await
            .is_err()
    );
    service.invalidate();
    assert!(
        service
            .report()
            .iter()
            .any(|row| row.pinned_bytes.is_some_and(|value| value > 0))
    );
    drop((query, metadata, next));
    assert_eq!(service.live_snapshot_bytes(), 0);
    // No CRC is required for a complete native metadata replay.
    let fresh = crate::delta::provider::table_builder(root.clone(), &state)
        .unwrap()
        .without_files()
        .load()
        .await
        .unwrap();
    assert_eq!(fresh.version(), Some(1));
    let unsupported = table
        .snapshot()
        .unwrap()
        .snapshot()
        .snapshot_ref()
        .clone()
        .write_checksum(table.log_store().engine(None))
        .await
        .unwrap_err();
    assert!(matches!(
        unsupported,
        deltalake::DeltaTableError::KernelError(
            deltalake::kernel::native::Error::ChecksumWriteUnsupported(_)
        )
    ));
}

#[tokio::test]
async fn native_kernel_crc_seed_advances_through_delta_writes_and_corruption_falls_back() {
    let directory = tempfile::tempdir().unwrap();
    let root = url::Url::from_directory_path(directory.path()).unwrap();
    let context = datafusion::prelude::SessionContext::new_with_state(
        SessionStateBuilder::new()
            .with_default_features()
            .with_query_planner(Arc::new(crate::session::planner::UnifiedPlanner::default()))
            .build(),
    );
    let state = Arc::new(context.state());
    let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));
    let table = crate::delta::provider::table_builder(root.clone(), &state)
        .unwrap()
        .with_crc_replay_max_commits(16)
        .build()
        .unwrap();
    kernel_checksum::create_with_checksum(root.clone(), &schema, table.log_store().engine(None))
        .await;
    let table = crate::delta::provider::table_builder(root.clone(), &state)
        .unwrap()
        .with_crc_replay_max_commits(16)
        .load()
        .await
        .unwrap();
    let batch = RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![5]))]).unwrap();
    let table = table
        .write([batch])
        .with_session_state(state.clone())
        .with_session_fallback_policy(
            deltalake::delta_datafusion::SessionFallbackPolicy::RequireSessionState,
        )
        .await
        .unwrap();
    let (_, written) = table
        .snapshot()
        .unwrap()
        .snapshot()
        .snapshot_ref()
        .clone()
        .write_checksum(table.log_store().engine(None))
        .await
        .unwrap();
    assert_eq!(written.version(), 1);
    let fresh = crate::delta::provider::table_builder(root.clone(), &state)
        .unwrap()
        .without_files()
        .load()
        .await
        .unwrap();
    assert_eq!(fresh.version(), Some(1));
    let advanced = table
        .set_tbl_properties()
        .with_properties(std::collections::HashMap::new())
        .await
        .unwrap();
    let stale_crc = crate::delta::provider::table_builder(root.clone(), &state)
        .unwrap()
        .without_files()
        .load()
        .await
        .unwrap();
    assert_eq!(stale_crc.version(), advanced.version());
    let crc = directory.path().join("_delta_log/00000000000000000001.crc");
    assert!(crc.exists());
    std::fs::write(crc, b"invalid checksum JSON").unwrap();
    // CRC is optional acceleration. Its absence/corruption must not certify stale state.
    let observed = crate::delta::provider::table_builder(root, &state)
        .unwrap()
        .without_files()
        .load()
        .await;
    match observed {
        Ok(observed) => assert_eq!(observed.version(), advanced.version()),
        Err(error) => {
            let diagnostic = error.to_string().to_lowercase();
            assert!(
                diagnostic.contains("checksum")
                    || diagnostic.contains("json")
                    || diagnostic.contains("crc"),
                "{diagnostic}"
            );
        }
    }
}
