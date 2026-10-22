// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Protected-provider replacement controls through the actual immutable snapshot owner.
#![allow(
    unsafe_code,
    reason = "isolated canonical fixtures explicitly assert partial scientific availability"
)]

use super::*;
use datafusion::{
    arrow::array::{Array, FixedSizeBinaryArray, Float64Array},
    logical_expr::{col, lit},
    prelude::SessionContext,
};
use pse_model::generated::identities::RunId;
use pse_operations::canonical_execution::{
    AttemptFence, RunRequest, SeedEligibility, TerminalClass,
};
use pse_relations::generated::runtime::fit_parameters;

async fn pruning_fixture(
    name: &str,
) -> Result<(Runtime, AttemptFence), Box<dyn std::error::Error + Send + Sync>> {
    let runtime = super::super::durable_tests::durable_runtime();
    let store = runtime.canonical_store()?;
    let source = store.edit(name, None, "source", &[]).await?;
    store
        .begin_run(&RunRequest {
            key: name.into(),
            revision: source,
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        })
        .await?;
    let fence = store
        .claim_run(name, "claim", "worker", std::time::Duration::from_secs(60))
        .await?;
    let acknowledgments = super::super::durable::PublicationAcks::new(&runtime.shared.pool());
    let mut position = super::super::result_projection::ResultStreamPosition::default();
    let validation = runtime.validation_context();
    // The first batch is a min/max false positive for value=1. The second
    // contains the sole match. Their exact source keys and positions are known.
    for rows in [&[(1u8, 0.0), (2u8, 2.0)][..], &[(3u8, 1.0)][..]] {
        let mut builder = fit_parameters::Builder::new(&validation)?;
        for (owner, value) in rows {
            builder.push(fit_parameters::Row {
                run_id: RunId::from_bytes([42; 16]),
                parameter_id: pse_ids::SemanticId::from_bytes([*owner; 16]),
                fixed: false,
                value: Some(*value),
                unit_id: pse_ids::SemanticId::NIL,
                scale: 1.0,
                at_bound: None,
            })?;
        }
        super::super::result_projection::append_result_table_prefix(
            store,
            &fence,
            fit_parameters::RELATION_ID,
            super::super::ResultCursor::from_checked(builder.finish()?),
            &runtime.shared.pool(),
            &acknowledgments,
            &mut position,
            None,
        )
        .await?;
    }
    super::super::result_projection::finish_result_table_prefix(
        store,
        &fence,
        fit_parameters::RELATION_ID,
        &acknowledgments,
        position,
    )
    .await?;
    let closed = store.close_result_ingestion(&fence, "close").await?;
    let manifest = store.reconcile_closed_attempt(&closed).await?;
    // SAFETY: this protocol fixture publishes a partial class, never usable science.
    unsafe {
        store.seal_attempt(
            &manifest,
            "seal",
            TerminalClass::Partial,
            &[1],
            SeedEligibility::Unavailable,
        )
    }
    .await?;
    Ok((runtime, fence))
}

#[tokio::test]
async fn canonical_pruning_inexact_residual_precedes_limit_and_predicate_projection() {
    let (runtime, fence) = pruning_fixture("res-inexact-limit").await.unwrap();
    let reader = runtime
        .results(
            "res-inexact-limit",
            fence.attempt(),
            "runtime.fit_parameters",
            0,
            u64::MAX,
            CancellationToken::new(),
        )
        .await
        .unwrap();
    let snapshot = reader.snapshot();
    assert_eq!(snapshot.blocks().len(), 2);
    assert!(
        snapshot
            .blocks()
            .iter()
            .all(|block| block.statistics.is_some())
    );
    let predicate = col("value").eq(lit(1.0));
    let provider = snapshot.table_provider();
    assert_eq!(
        provider.supports_filters_pushdown(&[&predicate]).unwrap(),
        vec![TableProviderFilterPushDown::Inexact]
    );
    let context = SessionContext::new_with_state(snapshot.session_state());
    let batches = context
        .read_table(provider)
        .unwrap()
        .filter(predicate)
        .unwrap()
        .select_columns(&["parameter_id"])
        .unwrap()
        .limit(0, Some(1))
        .unwrap()
        .collect()
        .await
        .unwrap();
    assert_eq!(
        batches.iter().map(|batch| batch.num_rows()).sum::<usize>(),
        1
    );
    let selected = batches.iter().find(|batch| batch.num_rows() != 0).unwrap();
    assert_eq!(
        selected.num_columns(),
        1,
        "predicate columns survive until residual evaluation, then project away"
    );
    assert_eq!(
        selected
            .column(0)
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .unwrap()
            .value(0),
        &[3u8; 16]
    );

    let mut typed = runtime
        .output_results(
            "res-inexact-limit",
            fence.attempt(),
            "runtime.fit_parameters",
            pse_ids::SemanticId::from_bytes([3; 16]),
            "value",
            "0",
            0,
            u64::MAX,
            Some(1.0),
            Some(1.0),
            None,
            CancellationToken::new(),
        )
        .await
        .unwrap();
    let original = typed
        .selected_snapshot(CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(original.row_count(), 1);
    assert_eq!(original.0.entries[0].block.start, 2);
    assert!(matches!(
        original.0.entries[0].rows,
        SelectedRows::Range {
            offset: 0,
            length: 1
        }
    ));
    let selected = typed.next_batch().await.unwrap().unwrap();
    assert_eq!(
        selected
            .column_by_name("value")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap()
            .value(0)
            .to_bits(),
        1.0f64.to_bits()
    );
    assert!(typed.next_batch().await.unwrap().is_none());
    drop((
        selected, original, typed, batches, context, snapshot, reader,
    ));
    pse_operations::testing::drain_result_releases(runtime.canonical_store().unwrap())
        .await
        .unwrap();
}

#[tokio::test]
async fn canonical_selection_reuses_one_physical_plan_for_pages_unknown_facts_and_residual() {
    let (runtime, fence) = pruning_fixture("res-plan-reuse").await.unwrap();
    let reader = runtime
        .results(
            "res-plan-reuse",
            fence.attempt(),
            "runtime.fit_parameters",
            0,
            u64::MAX,
            CancellationToken::new(),
        )
        .await
        .unwrap();
    let snapshot = reader.snapshot();
    let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
    let observed = hub.start("selection native preparation");
    pse_diagnostics::observation::scope(Some(observed.context()), async {
        let selection = NativeResultSelection::new(
            &runtime,
            fit_parameters::RELATION_ID,
            snapshot.schema(),
            &snapshot.session_state(),
            col("value").eq(lit(1.0)),
        )
        .unwrap();
        // Two successive pages use the retained physical plan. The first is a
        // genuine min/max false positive, so pruning cannot replace residuals.
        for block in snapshot.blocks() {
            let descriptor = snapshot.0.batches.get(&block.batch).unwrap();
            assert!(
                selection
                    .block_may_match(&runtime, block, descriptor)
                    .unwrap()
            );
        }
        let block = &snapshot.blocks()[0];
        let descriptor = snapshot.0.batches.get(&block.batch).unwrap();
        let mut unknown = block.clone();
        unknown.statistics = None;
        assert!(
            selection
                .block_may_match(&runtime, &unknown, descriptor)
                .unwrap()
        );
        unknown.statistics = Some(vec![0xff].into());
        assert!(
            selection
                .block_may_match(&runtime, &unknown, descriptor)
                .unwrap()
        );
        let checked = snapshot
            .read_block(block, &CancellationToken::new())
            .await
            .unwrap();
        let mask = selection.mask(checked.batch()).unwrap();
        assert_eq!(mask.true_count(), 0);
        let last = snapshot.blocks().last().unwrap();
        let checked = snapshot
            .read_block(last, &CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(selection.mask(checked.batch()).unwrap().true_count(), 1);
        let projected = checked.batch().project(&[0]).unwrap();
        assert!(
            selection.mask(&projected).is_err(),
            "a different schema cannot reuse this plan"
        );
    })
    .await;
    let metrics = observed
        .reader()
        .snapshot(std::time::Duration::from_secs(60))
        .metrics;
    assert_eq!(
        metrics
            .iter()
            .find(|metric| metric.name == "canonical.result.physical_predicate_plans")
            .unwrap()
            .cumulative,
        pse_diagnostics::observation::Value::Count(1)
    );
    drop((snapshot, reader));
    pse_operations::testing::drain_result_releases(runtime.canonical_store().unwrap())
        .await
        .unwrap();
}

#[tokio::test]
async fn canonical_pruned_provider_validates_protected_schema_and_skips_native_batch_decode() {
    let (runtime, fence) = pruning_fixture("res-protected-prune").await.unwrap();
    let reader = runtime
        .results(
            "res-protected-prune",
            fence.attempt(),
            "runtime.fit_parameters",
            0,
            u64::MAX,
            CancellationToken::new(),
        )
        .await
        .unwrap();
    let snapshot = reader.snapshot();
    let context = SessionContext::new_with_state(snapshot.session_state());
    let batches = context
        .read_table(snapshot.table_provider())
        .unwrap()
        .filter(col("value").gt(lit(1000.0)))
        .unwrap()
        .select_columns(&["parameter_id"])
        .unwrap()
        .collect()
        .await
        .unwrap();
    assert_eq!(
        batches.iter().map(|batch| batch.num_rows()).sum::<usize>(),
        0
    );
    // Even an empty pruned answer opened the protected file and verified its
    // complete integrity/footer/schema. The final batch iterator is unconsumed.
    let decode = snapshot.decoder();
    let inspection = MemoryConsumer::new("test:protected-pruned-iterator-inspection")
        .register(&runtime.shared.pool());
    inspection.try_grow(1024 * 1024).unwrap();
    {
        let mut cache = decode.cache.lock().await;
        let cached = cache
            .as_mut()
            .expect("pruning cannot bypass protected file validation");
        let next = cached
            .reader
            .next()
            .expect("the pruned native batch was not decoded")
            .unwrap();
        assert_eq!(next.num_rows(), 1);
        assert_eq!(
            next.column_by_name("value")
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap()
                .value(0),
            1.0
        );
    }
    drop(inspection);
    let block = snapshot.blocks().last().unwrap();
    let descriptor = snapshot.0.batches.get(&block.batch).unwrap();
    let mut unknown = block.clone();
    unknown.statistics = None;
    assert!(
        NativeResultSelection::new(
            &runtime,
            fit_parameters::RELATION_ID,
            snapshot.schema(),
            &snapshot.session_state(),
            col("value").gt(lit(1000.0)),
        )
        .unwrap()
        .block_may_match(&runtime, &unknown, descriptor)
        .unwrap()
    );
    unknown.statistics = Some(vec![0xff].into());
    assert!(
        NativeResultSelection::new(
            &runtime,
            fit_parameters::RELATION_ID,
            snapshot.schema(),
            &snapshot.session_state(),
            col("value").gt(lit(1000.0)),
        )
        .unwrap()
        .block_may_match(&runtime, &unknown, descriptor)
        .unwrap()
    );

    let mut fields = snapshot
        .schema()
        .fields()
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let index = snapshot.schema().index_of("value").unwrap();
    fields[index] = Arc::new(
        fields[index]
            .as_ref()
            .clone()
            .with_name("wrong_protected_value"),
    );
    let wrong_schema = Arc::new(datafusion::arrow::datatypes::Schema::new_with_metadata(
        fields,
        snapshot.schema().metadata().clone(),
    ));
    let wrong = ProtectedResultDecode::new(
        &runtime,
        decode.read.clone(),
        decode.read_owner.clone(),
        fit_parameters::RELATION_ID,
        wrong_schema,
    )
    .unwrap();
    assert!(
        wrong
            .validate_pruned_block(block, descriptor, &CancellationToken::new())
            .await
            .is_err(),
        "pruning cannot waive complete protected schema admission"
    );
    let cancel = CancellationToken::new();
    cancel.cancel();
    let before = runtime.shared.pool().reserved();
    assert!(
        decode
            .validate_pruned_block(block, descriptor, &cancel)
            .await
            .is_err()
    );
    assert_eq!(
        runtime.shared.pool().reserved(),
        before,
        "cancelled decoder turns create no new working grant"
    );
    let mut foreign = descriptor.clone();
    foreign.file_instance = "00000000-0000-0000-0000-000000000099".into();
    assert!(
        wrong
            .validate_pruned_block(block, &foreign, &CancellationToken::new())
            .await
            .is_err(),
        "the statistics premise cannot expand exact protected file membership"
    );
    drop((wrong, decode, batches, context, snapshot, reader));
    pse_operations::testing::drain_result_releases(runtime.canonical_store().unwrap())
        .await
        .unwrap();
}
