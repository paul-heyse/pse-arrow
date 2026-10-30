// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public workbench/observer controls over small original sealed storage fixtures.
#![allow(
    unsafe_code,
    clippy::unwrap_used,
    reason = "isolated typed result fixtures require exact setup and explicitly own their scientific observation assertions"
)]
use super::*;
use crate::workflow::{self as flow, StudyObservationContinuity, StudyObservationRequest};
use pse_model::{
    generated::enums::StudyPointState,
    study::{OccurrenceKey, PointOutcome, PointPolicy, ScientificFacts, SeedNeed, StartPolicy},
};
use pse_operations::{
    canonical::Revision,
    canonical_execution::{ResultPayload, RunRequest, SeedEligibility, TerminalClass},
    canonical_studies::{NewOccurrence, point_facts, point_key},
};
use pse_relations::generated::runtime::simulation_samples;
use std::time::Duration;
pub(in crate::workflow) async fn physical(runtime: &Runtime) -> flow::PhysicalSource {
    let (documents, _) = flow::tests::sources("package fixture { param x:Scalar=1; }");
    let cancel = crate::CancelSource::new();
    let admitted = runtime
        .physical_from_sources(&documents, &cancel)
        .await
        .unwrap();
    runtime
        .record_physical_source(&admitted, &cancel)
        .await
        .unwrap()
}
fn request(revision: &Revision, physical: &Revision, key: &str) -> RunRequest {
    RunRequest {
        purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
        admission_study: None,
        admission_point: None,
        key: key.into(),
        revision: revision.clone(),
        sources: vec![physical.clone()],
        request: vec![1],
        source_selection: vec![2],
        attestation: vec![3],
    }
}
pub(in crate::workflow) async fn create(
    runtime: &Runtime,
    study: &str,
    source: &Revision,
    physical: &flow::PhysicalSource,
    points: u32,
) {
    create_with_attempt_limit(runtime, study, source, physical, points, 1).await;
}
pub(in crate::workflow) async fn create_with_attempt_limit(
    runtime: &Runtime,
    study: &str,
    source: &Revision,
    physical: &flow::PhysicalSource,
    points: u32,
    attempt_limit: u32,
) {
    let store = runtime.canonical_store().unwrap();
    let physical_row = store.revision(&physical.revision).await.unwrap().unwrap();
    let points = (0..points)
        .map(|key| NewOccurrence {
            projection: None,
            policy: PointPolicy {
                key: OccurrenceKey(key),
                dependencies: vec![],
                start: StartPolicy::Fresh,
                seed_need: SeedNeed::NotNeeded,
                attempt_limit,
            },
            descriptor: vec![42],
            run: request(source, &physical_row, &format!("{study}-point-{key}")),
        })
        .collect::<Vec<_>>();
    let metadata=serde_json::to_vec(&serde_json::json!({"version":5,"physical":physical,"context":null,"variants":null,"diagnostics":null})).unwrap();
    store
        .create_study(
            study,
            &request(source, &physical_row, &format!("{study}-parent")),
            &metadata,
            &points,
            &|| false,
        )
        .await
        .unwrap();
}
async fn finish(
    runtime: &Runtime,
    study: &str,
    key: u32,
    quantity: pse_ids::SemanticId,
    unit: pse_ids::SemanticId,
    owner: pse_ids::SemanticId,
    value: Option<f64>,
) {
    finish_batched(runtime, study, key, (quantity, unit, owner), value, 1).await;
}
async fn finish_batched(
    runtime: &Runtime,
    study: &str,
    key: u32,
    coordinates: (
        pse_ids::SemanticId,
        pse_ids::SemanticId,
        pse_ids::SemanticId,
    ),
    value: Option<f64>,
    batch_count: usize,
) {
    let store = runtime.canonical_store().unwrap();
    let point = point_key(study, OccurrenceKey(key));
    let scope = store.study_scope(&point).await.unwrap();
    let claim = store
        .claim_study_point(
            &scope,
            None,
            &format!("claim-{point}"),
            "worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    finish_claimed(runtime, study, key, coordinates, value, batch_count, claim).await;
}
async fn finish_claimed(
    runtime: &Runtime,
    study: &str,
    key: u32,
    coordinates: (
        pse_ids::SemanticId,
        pse_ids::SemanticId,
        pse_ids::SemanticId,
    ),
    value: Option<f64>,
    batch_count: usize,
    claim: pse_operations::canonical_studies::StudyClaim,
) {
    let (quantity, unit, owner) = coordinates;
    let store = runtime.canonical_store().unwrap();
    let point = point_key(study, OccurrenceKey(key));
    if let Some(value) = value {
        let run_id = pse_operations::mint_id();
        let mut batches = Vec::with_capacity(batch_count);
        for block in 0..batch_count {
            let mut samples = simulation_samples::Builder::with_registry(
                runtime.registry(),
                3,
                &runtime.validation_context(),
            )
            .unwrap();
            for offset in 0..3 {
                let sample = i64::try_from(block * 3 + offset).unwrap();
                samples
                    .push(simulation_samples::Row {
                        run_id,
                        sample,
                        time: sample as f64,
                        symbol_id: owner,
                        quantity_id: quantity,
                        unit_id: unit,
                        value: value + sample as f64 * 2.0,
                    })
                    .unwrap();
            }
            batches.push(samples.finish().unwrap());
        }
        let cursor = flow::ResultCursor::from_retained(
            batches[0].batch().schema(),
            simulation_samples::RELATION_ID,
            &batches,
        )
        .unwrap();
        flow::result_projection::store_result_table(
            store,
            &claim.fence,
            simulation_samples::RELATION_ID,
            cursor,
            &runtime.shared.pool(),
            &flow::durable::PublicationAcks::new(&runtime.shared.pool()),
        )
        .await
        .unwrap();
        // An unrelated payload deliberately is not scientific IPC. Exact selected detail
        // must never decode it to open the requested original sample relation.
        store
            .append_result_batch(
                &claim.fence,
                &format!("opaque-{point}"),
                "unrelated-opaque-fixture",
                0,
                ResultPayload::new(vec![255, 254]).unwrap(),
                1,
            )
            .await
            .unwrap();
    }
    let closed = store
        .close_result_ingestion(&claim.fence, &format!("close-{point}"))
        .await
        .unwrap();
    let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
    // SAFETY: these small sealed typed samples and finite values are fixture-owned;
    // this control tests storage composition, not solver or scientific qualification.
    unsafe {
        store
            .seal_attempt(
                &manifest,
                &format!("seal-{point}"),
                if value.is_some() {
                    TerminalClass::Succeeded
                } else {
                    TerminalClass::Failed
                },
                &[],
                SeedEligibility::Unavailable,
            )
            .await
    }
    .unwrap();
    let mut facts = point_facts(&claim.point).unwrap();
    facts.native_started = true;
    facts.lifecycle = if value.is_some() {
        StudyPointState::Completed
    } else {
        StudyPointState::Failed
    };
    facts.scientific = ScientificFacts {
        usable: value.is_some(),
        candidate_use: None,
        seed_permission: false,
    };
    let outcome = PointOutcome {
        key: OccurrenceKey(key),
        lifecycle: facts.lifecycle,
        scientific: facts.scientific.clone(),
        diagnostic: None,
        start: None,
        effect: facts.effect,
        attempts: vec![],
    };
    // SAFETY: the immediately sealed fixture attempt backs these original observations.
    unsafe {
        store
            .observe_study_point(&claim.point, &facts, &outcome, true)
            .await
    }
    .unwrap();
}
fn cohort(physical: &flow::PhysicalSource) -> StudyCohortRequest {
    StudyCohortRequest {
        filter: StudyWorkbenchFilter {
            physical_revision: Some(physical.revision.clone()),
            ..Default::default()
        },
        maximum_studies: 4,
        maximum_members: 8,
    }
}
#[tokio::test]
#[allow(
    clippy::print_stderr,
    reason = "selected native population qualification records phase timing and accounted pool usage without performance thresholds"
)]
async fn workbench_actual_10000_mixed_members_compact_summary_and_shared_provider() {
    use datafusion::arrow::array::{Array, Float64Array, Int64Array};
    use futures_util::StreamExt;
    use pse_operations::canonical::{CanonicalError, RefusalCode};
    let started = std::time::Instant::now();
    let runtime = flow::durable_tests::durable_runtime();
    let physical = physical(&runtime).await;
    let store = runtime.canonical_store().unwrap();
    let source = store
        .edit(
            "cohort-10000-model",
            None,
            "selected population fixture",
            &[],
        )
        .await
        .unwrap();
    let study = pse_operations::mint_id::<pse_ids::SemanticId>().to_hex();
    create(&runtime, &study, &source, &physical, 10000).await;
    let created = started.elapsed();
    let quantity = pse_operations::mint_id();
    let unit = pse_operations::mint_id();
    let owner = pse_operations::mint_id();
    let usable = [0_u32, 1, 255, 256, 511, 512, 4095, 8191, 9998];
    for key in usable {
        finish_batched(
            &runtime,
            &study,
            key,
            (quantity, unit, owner),
            Some(f64::from(key)),
            if key == 0 { 65 } else { 1 },
        )
        .await;
    }
    finish(&runtime, &study, 127, quantity, unit, owner, None).await;
    finish(&runtime, &study, 8192, quantity, unit, owner, None).await;
    let scope = store
        .study_scope(&point_key(&study, OccurrenceKey(4096)))
        .await
        .unwrap();
    let active = store
        .claim_study_point(
            &scope,
            None,
            "scale-active-before-capture",
            "worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    let summary = store
        .canonical_study_summary(&study)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(summary.point_count, 10000);
    assert!(summary.active && !summary.terminal);
    let request = StudyCohortRequest {
        filter: StudyWorkbenchFilter {
            study: Some(study.clone()),
            ..Default::default()
        },
        maximum_studies: 1,
        maximum_members: 10000,
    };
    assert!(
        runtime
            .freeze_study_cohort(StudyCohortRequest {
                maximum_members: pse_schema::catalog::study_workbench::STUDY_COHORT_MAX_MEMBERS + 1,
                ..request.clone()
            })
            .await
            .is_err(),
        "population admission refuses beyond the single declared cap"
    );
    let pool_limit = runtime.shared.budget().memory_limit_bytes.get();
    let capture_start_live = runtime.shared.pool().reserved();
    runtime.shared.reset_observation_peak();
    let capture_started = std::time::Instant::now();
    let frozen = runtime.freeze_study_cohort(request).await.unwrap();
    let captured = capture_started.elapsed();
    let capture_peak = runtime.shared.observation_peak_bytes();
    let capture_end_live = runtime.shared.pool().reserved();
    assert_eq!(
        (frozen.receipt().header_count, frozen.receipt().member_count),
        (1, 10000)
    );
    let original = frozen
        .member_page(StudyCohortPageRequest {
            maximum: 1,
            cursor: Some(StudyCohortCursor {
                issuing_store: frozen.receipt().issuing_store.clone(),
                capture: frozen.receipt().capture.clone(),
                after: 4095,
            }),
        })
        .await
        .unwrap();
    assert!(original.members[0].value.attempt.is_some());
    assert!(original.members[0].value.manifest.is_none());
    let mut cursor = None;
    let mut count = 0;
    let mut pages = 0;
    let mut peak_retained = 0;
    let mut active = Some(active);
    let mut baseline = runtime.shared.pool().reserved();
    loop {
        let page = frozen
            .member_page(StudyCohortPageRequest {
                maximum: 256,
                cursor,
            })
            .await
            .unwrap();
        assert!(page.members.len() <= 256);
        for row in &page.members {
            assert_eq!(row.position, count);
            count += 1;
            if row.position == 4096 {
                assert_eq!(
                    row.value.metadata_revision,
                    original.members[0].value.metadata_revision
                );
                assert!(
                    row.value.manifest.is_none(),
                    "later completion cannot change the captured unavailable member"
                );
            }
        }
        pages += 1;
        peak_retained = peak_retained.max(page._owner.as_ref().unwrap().size());
        cursor = page.next.clone();
        drop(page);
        if pages == 1 {
            let writer = runtime.clone();
            let selected = study.clone();
            let claim = active.take().unwrap();
            tokio::spawn(async move {
                finish_claimed(
                    &writer,
                    &selected,
                    4096,
                    (quantity, unit, owner),
                    Some(100000.0),
                    1,
                    claim,
                )
                .await;
            })
            .await
            .unwrap();
            // Mutation still occurs between captured pages. Its independent
            // permission reservations cannot contaminate subsequent page grants.
            baseline = runtime.shared.pool().reserved();
        }
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!((count, pages), (10000, 40));
    assert!(
        peak_retained < 2 * 1024 * 1024,
        "this fixture retains only its actual bounded member page"
    );
    tokio::time::timeout(Duration::from_secs(10), async {
        while runtime.shared.pool().reserved().saturating_sub(baseline) >= 2 * 1024 * 1024 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("physical response drain leaves no accumulated cohort page grants");
    let kpi = StudyKpiRequest {
        relation: "runtime.simulation_samples".into(),
        owner,
        field: "value".into(),
        partition: "0".into(),
        start: 0,
        end: 195,
    };
    let aggregate_start_live = runtime.shared.pool().reserved();
    runtime.shared.reset_observation_peak();
    let aggregate_started = std::time::Instant::now();
    let analysis = frozen
        .aggregate(&kpi, CancellationToken::new())
        .await
        .unwrap();
    let aggregated = aggregate_started.elapsed();
    let aggregate_peak = runtime.shared.observation_peak_bytes();
    let aggregate_end_live = runtime.shared.pool().reserved();
    assert_eq!((analysis.included, analysis.unavailable), (9, 9991));
    assert_eq!(analysis.progress().state, StudyCohortScanState::Complete);
    let batch = analysis
        .batches
        .iter()
        .find(|batch| batch.num_rows() > 0)
        .unwrap();
    let rows = batch
        .column_by_name("count")
        .unwrap()
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .value(0);
    assert_eq!(
        rows, 219,
        "one65-block source and eight3-row sources were actually decoded"
    );
    let mean = batch
        .column_by_name("mean")
        .unwrap()
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap()
        .value(0);
    let expected = (37830.0
        + usable
            .into_iter()
            .filter(|key| *key != 0)
            .map(|key| 3.0 * f64::from(key) + 6.0)
            .sum::<f64>())
        / 219.0;
    assert!((mean - expected).abs() < 1e-9);
    let mut cursor = None;
    let mut classified = 0;
    let mut included = 0;
    loop {
        let page = analysis
            .classification_page(StudyCohortPageRequest {
                maximum: 256,
                cursor,
            })
            .await
            .unwrap();
        classified += page.members.len();
        included += page
            .members
            .iter()
            .filter(|row| row.value.availability == StudyCohortAvailability::Included)
            .count();
        if let Some(row) = page.members.iter().find(|row| row.position == 4096) {
            assert_eq!(
                row.value.availability,
                StudyCohortAvailability::CapturedUnavailable
            );
        }
        cursor = page.next.clone();
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!((classified, included), (10000, 9));
    assert!(
        frozen
            .result(
                4096,
                "runtime.simulation_samples",
                0,
                1,
                CancellationToken::new()
            )
            .await
            .is_err()
    );
    // Settle the remaining original unattempted occurrences through the existing
    // scoped cancellation producer, then execute the actual compact finalizer.
    store.cancel_study(&study).await.unwrap();
    let settlement_started = std::time::Instant::now();
    let mut after = None;
    loop {
        let page = store.study_point_page(&study, after).await.unwrap();
        if page.is_empty() {
            break;
        }
        after = page.last().map(|point| point.ordinal);
        // These independent fixture points have no dependency edges. Overlap
        // only their compact scoped I/O within this original bounded page;
        // production admission, request clocks and retries remain authoritative.
        let settled =
            futures_util::stream::iter(page.into_iter().filter(|point| !point.settled).map(
                |point| {
                    let store = &store;
                    async move {
                        let scope = store.study_scope(&point.key).await?;
                        store
                            .refuse_study_point(&scope, &scope.action(None)?, None)
                            .await
                    }
                },
            ))
            .buffer_unordered(16)
            .collect::<Vec<_>>()
            .await;
        // Drain every admitted sibling before asserting; retain each original
        // typed error rather than dropping in-flight work on the first refusal.
        let failures = settled
            .into_iter()
            .filter_map(Result::err)
            .collect::<Vec<_>>();
        assert!(
            failures.is_empty(),
            "fixture cancellation settlement failures: {failures:?}"
        );
    }
    let settled = settlement_started.elapsed();
    let summary_started = std::time::Instant::now();
    assert!(runtime.finalize_canonical_study(&study).await.unwrap());
    let parent = store
        .canonical_run(&format!("{study}-parent"))
        .await
        .unwrap()
        .unwrap();
    let read = store
        .read_results(&parent.key, parent.terminal_attempt.as_deref().unwrap())
        .await
        .unwrap();
    let selected = read
        .sets()
        .iter()
        .find(|set| {
            set.name == pse_relations::generated::runtime::study_outcomes::RELATION_ID.to_string()
        })
        .unwrap();
    assert_eq!(selected.row_count, 10000);
    assert!(
        selected.batch_count <= 16,
        "summary publication packs useful rows rather than page/four-row files"
    );
    let summary_batches = selected.batch_count;
    let summarized = summary_started.elapsed();
    drop(read);
    let mut summary_reader = runtime
        .results(
            &parent.key,
            parent.terminal_attempt.as_deref().unwrap(),
            "runtime.study_outcomes",
            0,
            10000,
            CancellationToken::new(),
        )
        .await
        .unwrap();
    let mut summary_rows = 0;
    let mut summary_usable = 0;
    while let Some(batch) = summary_reader.next_relation_batch().await.unwrap() {
        let view =
            pse_relations::generated::runtime::study_outcomes::View::from_checked(&batch).unwrap();
        for index in 0..view.len() {
            summary_rows += 1;
            summary_usable += usize::from(view.row(index).unwrap().usable);
        }
    }
    assert_eq!(
        (summary_rows, summary_usable),
        (10000, 10),
        "final summary reflects the later completed member while the captured cut stays at9"
    );
    summary_reader.close();
    drop(summary_reader);
    store.forget_study_results(&study).await.unwrap();
    assert!(matches!(
        store.forget_run_results(&format!("{study}-point-0")).await,
        Err(CanonicalError::Refused {
            code: RefusalCode::RunResultsHaveProtectedReaders,
            ..
        })
    ));
    let key = frozen.receipt().capture.clone();
    drop(analysis);
    drop(frozen);
    drop(original);
    store.recover_study_capture(&key).await.unwrap();
    store
        .forget_run_results(&format!("{study}-point-0"))
        .await
        .unwrap();
    store.remove_isolated_fixture().await.unwrap();
    eprintln!(
        "actual10000 mixed qualification: captured usable9 failed2 active1 unattempted9988; payload rows219 with65-block source; pages40 summary_batches={summary_batches}; accounted_pool_limit_bytes={pool_limit} capture_accounted_start_live_bytes={capture_start_live} capture_accounted_phase_peak_bytes={capture_peak} capture_accounted_end_live_bytes={capture_end_live} page_retained_peak_bytes={peak_retained} aggregate_accounted_start_live_bytes={aggregate_start_live} aggregate_accounted_phase_peak_bytes={aggregate_peak} aggregate_accounted_end_live_bytes={aggregate_end_live}; creation={created:?} capture={captured:?} aggregate={aggregated:?} cancellation_settlement={settled:?} compact_finalization={summarized:?} total={:?}",
        started.elapsed()
    );
}
#[tokio::test]
async fn workbench_public_cohort_retains_population_compares_value_variants_and_refuses_incompatible_quantity()
 {
    use datafusion::arrow::array::{Array, Float64Array, Int64Array};
    let runtime = flow::durable_tests::durable_runtime();
    let physical = physical(&runtime).await;
    let store = runtime.canonical_store().unwrap();
    let source = store
        .edit("workbench-model", None, "first", &[])
        .await
        .unwrap();
    create(&runtime, "study-a", &source, &physical, 3).await;
    let owner = pse_operations::mint_id();
    let quantity = pse_operations::mint_id();
    let unit = pse_operations::mint_id();
    finish(&runtime, "study-a", 0, quantity, unit, owner, Some(2.0)).await;
    finish(&runtime, "study-a", 1, quantity, unit, owner, None).await;
    let changed = store
        .edit("workbench-model", Some(&source.key), "value-only", &[])
        .await
        .unwrap();
    create(&runtime, "study-b", &changed, &physical, 1).await;
    finish(&runtime, "study-b", 0, quantity, unit, owner, Some(8.0)).await;
    let frozen = runtime
        .freeze_study_cohort(cohort(&physical))
        .await
        .unwrap();
    assert_eq!(frozen.receipt().member_count, 4);
    assert!(frozen.receipt().complete);
    let members = frozen
        .member_page(StudyCohortPageRequest {
            maximum: 4,
            cursor: None,
        })
        .await
        .unwrap();
    let headers = frozen
        .header_page(StudyCohortPageRequest {
            maximum: 4,
            cursor: None,
        })
        .await
        .unwrap();
    assert_eq!(
        members.members[1].value.facts.lifecycle,
        StudyPointState::Failed
    );
    assert!(members.members[2].value.attempt.is_none());
    assert_ne!(
        headers.headers[0].value.source,
        headers.headers[1].value.source
    );
    let kpi = StudyKpiRequest {
        relation: "runtime.simulation_samples".into(),
        owner,
        field: "value".into(),
        partition: "0".into(),
        start: 0,
        end: 3,
    };
    finish(&runtime, "study-a", 2, quantity, unit, owner, Some(50.0)).await;
    let later = frozen
        .member_page(StudyCohortPageRequest {
            maximum: 4,
            cursor: None,
        })
        .await
        .unwrap();
    assert!(
        later.members[2].value.manifest.is_none(),
        "late completion cannot reclassify captured absence"
    );
    // A limited scan establishes only the consumed member; its unread sibling
    // remains pending under the same exact predicate, never unavailable.
    let scan_state = frozen
        .relation(kpi.clone(), CancellationToken::new())
        .await
        .unwrap();
    let context = datafusion::prelude::SessionContext::new_with_state(scan_state.session_state());
    let frame = context.read_table(scan_state.table_provider()).unwrap();
    let _limited = frame
        .clone()
        .limit(0, Some(1))
        .unwrap()
        .collect()
        .await
        .unwrap();
    let partial = scan_state.progress();
    assert_eq!(partial.state, StudyCohortScanState::Partial);
    assert!(partial.pending > 0);
    assert!(
        partial.unavailable < 4,
        "unvisited members are never filled in as unavailable"
    );
    let (first, second) = tokio::join!(frame.clone().collect(), frame.collect());
    assert_eq!(
        first
            .unwrap()
            .iter()
            .map(|batch| batch.num_rows())
            .sum::<usize>(),
        6
    );
    assert_eq!(
        second
            .unwrap()
            .iter()
            .map(|batch| batch.num_rows())
            .sum::<usize>(),
        6
    );
    assert_eq!(scan_state.progress().state, StudyCohortScanState::Complete);
    assert_eq!(scan_state.progress().active_scans, 0);
    drop(scan_state);
    drop(context);
    let one = frozen
        .member_page(StudyCohortPageRequest {
            maximum: 1,
            cursor: None,
        })
        .await
        .unwrap();
    assert_eq!(one.members.len(), 1);
    assert_eq!(one.members[0].position, 0);
    let mut foreign = one.next.clone().unwrap();
    foreign.capture.push_str("-other");
    assert!(
        frozen
            .member_page(StudyCohortPageRequest {
                maximum: 1,
                cursor: Some(foreign)
            })
            .await
            .is_err()
    );
    let second = frozen
        .member_page(StudyCohortPageRequest {
            maximum: 1,
            cursor: one.next,
        })
        .await
        .unwrap();
    assert_eq!(second.members[0].position, 1);
    let exports = tempfile::tempdir().unwrap();
    let refused = exports.path().join("refused.ndjson");
    assert!(
        frozen
            .export_pages(&refused, 1, &CancellationToken::new())
            .await
            .is_err()
    );
    assert!(!refused.exists());
    assert!(exports.path().join("refused.ndjson.incomplete").is_file());
    let published = exports.path().join("cohort.ndjson");
    let bytes = frozen
        .export_pages(&published, 4 * 1024 * 1024, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(bytes, std::fs::metadata(&published).unwrap().len());
    let encoded = std::fs::read_to_string(published).unwrap();
    let lines = encoded
        .lines()
        .map(|line| serde_json::from_str::<StudyCohortExportRecord>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(
        matches!(lines.last(),Some(StudyCohortExportRecord::Receipt(receipt)) if receipt.member_count==4)
    );
    assert_eq!(
        lines
            .iter()
            .filter_map(
                |line| if let StudyCohortExportRecord::Members(page) = line {
                    Some(page.members.len())
                } else {
                    None
                }
            )
            .sum::<usize>(),
        4
    );
    let result = frozen
        .aggregate(&kpi, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(result.included, 2);
    assert_eq!(result.unavailable, 2);
    assert_eq!(result.progress().state, StudyCohortScanState::Complete);
    let classified = result
        .classification_page(StudyCohortPageRequest {
            maximum: 4,
            cursor: None,
        })
        .await
        .unwrap();
    assert_eq!(
        classified
            .members
            .iter()
            .filter(|row| row.value.availability == StudyCohortAvailability::Included)
            .map(|row| row.position)
            .collect::<Vec<_>>(),
        [0, 3]
    );
    assert_eq!(
        classified
            .members
            .iter()
            .filter(|row| row.value.availability == StudyCohortAvailability::CapturedUnavailable)
            .map(|row| row.position)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let batch = result.batches.iter().find(|b| b.num_rows() > 0).unwrap();
    assert_eq!(
        batch
            .column_by_name("count")
            .unwrap()
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0),
        6
    );
    assert_eq!(
        batch
            .column_by_name("mean")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap()
            .value(0),
        7.0
    );
    let mut selected = frozen
        .result(
            0,
            "runtime.simulation_samples",
            0,
            1,
            CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(selected.next_batch().await.unwrap().unwrap().num_rows(), 1);
    selected.close();
    assert!(
        frozen
            .result(
                2,
                "runtime.simulation_samples",
                0,
                1,
                CancellationToken::new()
            )
            .await
            .is_err()
    );
    create(&runtime, "study-c", &changed, &physical, 1).await;
    finish(
        &runtime,
        "study-c",
        0,
        pse_operations::mint_id(),
        unit,
        owner,
        Some(100.0),
    )
    .await;
    assert_eq!(
        frozen.receipt().member_count,
        4,
        "later live membership cannot alter frozen analysis"
    );
    let incompatible = runtime
        .freeze_study_cohort(cohort(&physical))
        .await
        .unwrap();
    assert!(
        incompatible
            .aggregate(&kpi, CancellationToken::new())
            .await
            .is_err()
    );
    assert!(
        runtime
            .freeze_study_cohort(StudyCohortRequest {
                maximum_members: 1,
                ..cohort(&physical)
            })
            .await
            .is_err()
    );
    drop(selected);
    drop(incompatible);
    drop(frozen);
    drop(result);
    store.remove_isolated_fixture().await.unwrap();
}
#[tokio::test]
async fn study_observer_public_reconnect_checks_authority_and_exit_preserves_science() {
    let runtime = flow::durable_tests::durable_runtime();
    let physical = physical(&runtime).await;
    let store = runtime.canonical_store().unwrap();
    let source = store
        .edit("observer-model", None, "source", &[])
        .await
        .unwrap();
    create(&runtime, "observed-study", &source, &physical, 1).await;
    let scope = store
        .study_scope(&point_key("observed-study", OccurrenceKey(0)))
        .await
        .unwrap();
    let claim = store
        .claim_study_point(
            &scope,
            None,
            "observer-claim",
            "worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    let request = StudyObservationRequest {
        study: "observed-study".into(),
        maximum_members: 2,
        maximum_events: 8,
    };
    let mut observer = runtime.observe_study(request.clone(), None).await.unwrap();
    let page = observer.poll().await.unwrap();
    assert_eq!(page.sources.len(), 1);
    assert_eq!(
        page.sources[0].continuity,
        StudyObservationContinuity::LiveNumericsUnavailable
    );
    let cursor = observer.cursor();
    observer.close();
    drop(observer);
    assert!(
        !store
            .canonical_study("observed-study")
            .await
            .unwrap()
            .unwrap()
            .cancelled
    );
    assert!(
        !store
            .canonical_run(&claim.point.run)
            .await
            .unwrap()
            .unwrap()
            .cancelled
    );
    let reopened = runtime
        .observe_study(request.clone(), Some(cursor.clone()))
        .await
        .unwrap();
    drop(reopened);
    let mut foreign = cursor.clone();
    foreign.issuing_store.push_str("-foreign");
    assert!(
        runtime
            .observe_study(request.clone(), Some(foreign))
            .await
            .is_err()
    );
    let mut wrong = cursor;
    wrong.context.source.push_str("-wrong");
    assert!(runtime.observe_study(request, Some(wrong)).await.is_err());
    // Explicit cancellation remains its own canonical authority after observer exit.
    let cancelled = store.cancel_study("observed-study").await.unwrap();
    assert!(cancelled.cancelled);
    let capture = page.status.receipt.capture.clone();
    drop(page);
    store.recover_study_capture(&capture).await.unwrap();
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn workbench_capture_continuation_survives_caller_close_and_terminal_drain_joins_release() {
    use pse_operations::canonical::CanonicalError;
    let runtime = flow::durable_tests::durable_runtime();
    let store = runtime.canonical_store().unwrap().clone();
    let capture = runtime
        .freeze_study_cohort(StudyCohortRequest {
            filter: StudyWorkbenchFilter::default(),
            maximum_studies: 1,
            maximum_members: 1,
        })
        .await
        .unwrap();
    runtime.close();
    assert!(matches!(
        store.disconnect().await,
        Err(CanonicalError::ConnectionBusy { .. })
    ));
    drop(capture);
    store
        .drain_owned_connection_until(tokio::time::Instant::now() + Duration::from_secs(30))
        .await
        .unwrap();
    // The original physical owner has completed every tracked release and SDK
    // callback; a second close is idempotent, never evidence of scientific work.
    store.disconnect().await.unwrap();
    store.remove_isolated_fixture().await.unwrap();
}
