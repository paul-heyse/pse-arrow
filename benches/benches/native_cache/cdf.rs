// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native four-image CDF comparison, using the deployment's namespaced cache.
use super::delta::{Environment, create};
use datafusion::{
    arrow::array::Array,
    common::ScalarValue,
    logical_expr::{col, lit},
    prelude::SessionContext,
};
use deltalake::delta_datafusion::{DeltaCdfTableProvider, SessionFallbackPolicy};
use std::{collections::BTreeMap, sync::Arc, time::Instant};

pub(super) async fn measure(root: &url::Url) {
    let env = Environment::new(true, 0, 1, 16);
    let table = create(root, 16, &env.state).await;
    let table = table
        .update()
        .with_update("v1", col("v0") + lit(100_i64))
        .with_predicate(col("v0").eq(lit(0_i64)))
        .with_session_state(env.state.clone())
        .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
        .await
        .unwrap()
        .0;
    let table = table
        .delete()
        .with_predicate(col("v0").eq(lit(1_i64)))
        .with_session_state(env.state.clone())
        .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
        .await
        .unwrap()
        .0;
    for enabled in [false, true] {
        let env = Environment::new(enabled, 0, 1, 16);
        let state = env.caches.native().bind_state(root, &env.state).unwrap();
        let selected = pse_catalog::delta::provider::table_builder(root.clone(), &state)
            .unwrap()
            .with_version(table.version().unwrap())
            .load()
            .await
            .unwrap();
        for phase in ["fresh_cache", "warm_cache"] {
            env.store.reset();
            let started = Instant::now();
            let provider = DeltaCdfTableProvider::try_new(
                selected
                    .clone()
                    .scan_cdf()
                    .with_starting_version(0)
                    .with_ending_version(table.version().unwrap())
                    .with_file_metadata_cache(
                        state.runtime_env().cache_manager.get_file_metadata_cache(),
                    ),
            )
            .unwrap();
            let context = SessionContext::new_with_state(state.as_ref().clone());
            let batches = context
                .read_table(Arc::new(provider))
                .unwrap()
                .collect()
                .await
                .unwrap();
            let mut counts = BTreeMap::<String, usize>::new();
            for batch in batches {
                let kinds = batch.column_by_name("_change_type").unwrap();
                for row in 0..kinds.len() {
                    *counts
                        .entry(ScalarValue::try_from_array(kinds, row).unwrap().to_string())
                        .or_default() += 1;
                }
            }
            assert_eq!(
                counts,
                BTreeMap::from([
                    ("insert".into(), 16),
                    ("delete".into(), 1),
                    ("update_preimage".into(), 1),
                    ("update_postimage".into(), 1)
                ])
            );
            println!(
                "{}",
                serde_json::json!({"experiment":"cdf_four_images","cache_enabled":enabled,"phase":phase,"seconds":started.elapsed().as_secs_f64(),"counts":counts,"resources":env.report()})
            );
        }
    }
}
