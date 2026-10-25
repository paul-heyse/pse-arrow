// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "isolated bounded result fixture publishes partial availability, never admitted scientific usability"
)]
use super::*;
use pse_operations::canonical_execution::{
    AttemptFence, RunRequest, SeedEligibility, TerminalClass,
};
use pse_relations::generated::{enums::DualQualification, runtime::solve_variables};
use std::time::Duration;

#[allow(
    clippy::unwrap_used,
    reason = "isolated registered result setup fails the test on unexpected admission or publication errors"
)]
async fn wide_fixture(nested: bool) -> (Runtime, AttemptFence, RecordBatch) {
    wide_fixture_physical(nested, false).await
}
async fn wide_fixture_physical(nested: bool, retained_physical: bool) -> (Runtime, AttemptFence, RecordBatch) {
    wide_fixture_receipt(nested, retained_physical, None).await
}
async fn wide_fixture_receipt(nested: bool, retained_physical: bool, receipt_fault: Option<&str>) -> (Runtime, AttemptFence, RecordBatch) {
    // This fixture exercises canonical storage and protected reads, not a native
    // scientific execution receiver. Keep its isolated store and attestation real.
    let store = pse_operations::testing::canonical_fixture_store().unwrap();
    let runtime = super::super::tests::runtime_with_workspace(32 << 20)
        .with_canonical(super::super::CanonicalDeployment::new(
            store,
            super::super::OuterAttestation {
                source: Some(pse_ids::ContentHash::from_bytes([0; 32])),
                build: pse_ids::ContentHash::from_bytes([1; 32]),
            },
            None,
        ))
        .unwrap();
    let validation = runtime.validation_context();
    let checked = if nested {
        use pse_relations::generated::{
            enums::{StudyEffectState, StudyPointState, StudyStartKind},
            runtime::study_outcomes,
        };
        let mut builder =
            study_outcomes::Builder::with_registry(runtime.registry(), 3, &validation).unwrap();
        for point_index in 0..3 {
            builder
                .push(study_outcomes::Row {
                    study_id: pse_model::generated::identities::StudyId::from(
                        pse_ids::SemanticId::from_bytes([0x77; 16]),
                    ),
                    point_index,
                    case_id: None,
                    binding_hash: None,
                    state: StudyPointState::Cancelled,
                    attempt_id: None,
                    attempt_state: None,
                    result_id: None,
                    usable: false,
                    seed_permission: false,
                    candidate_use: None,
                    effect: StudyEffectState::Absent,
                    start: (point_index != 2).then_some(
                        study_outcomes::RuntimeStudyOutcomesFieldStart {
                            kind: StudyStartKind::Fresh,
                            predecessor: None,
                            role: None,
                            seed_id: None,
                            unavailable: None,
                        },
                    ),
                    diagnostic: None,
                    attempts: if point_index == 0 {
                        vec![study_outcomes::RuntimeStudyOutcomesFieldAttemptsItem {
                            attempt_id: None,
                            lifecycle: None,
                            usable: false,
                            seed_permission: false,
                            candidate_use: None,
                            effect: StudyEffectState::Absent,
                            start: None,
                            diagnostic: None,
                        }]
                    } else {
                        vec![]
                    },
                })
                .unwrap();
        }
        builder.finish().unwrap()
    } else {
        let mut builder =
            solve_variables::Builder::with_registry(runtime.registry(), 3, &validation).unwrap();
        let run_id = pse_operations::mint_id();
        // These are original opaque identities, not resolved unit or quantity definitions.
        let quantity_id = pse_ids::SemanticId::from_bytes([0x44; 16]);
        let unit_id = pse_ids::SemanticId::from_bytes([0x55; 16]);
        for (step, value) in [(0, Some(-0.0)), (1, None), (2, Some(273.15))] {
            builder
                .push(solve_variables::Row {
                    run_id,
                    step,
                    symbol_id: pse_ids::SemanticId::from_bytes([0x33; 16]),
                    quantity_id: Some(quantity_id),
                    unit_id: Some(unit_id),
                    fixed: step == 1,
                    parameter: false,
                    domain: None,
                    value,
                    lower: None,
                    upper: None,
                    lower_violation: None,
                    upper_violation: None,
                    tolerance: None,
                    lower_dual: None,
                    upper_dual: None,
                    reduced_cost: None,
                    stationarity: None,
                    dual_qualification: DualQualification::Unavailable,
                })
                .unwrap();
        }
        builder.finish().unwrap()
    };
    let batch = checked.batch().clone();
    let store = runtime.canonical_store().unwrap();
    let revision = store
        .edit("readable-fixture", None, "source", &[])
        .await
        .unwrap();
    let physical_rows = if retained_physical {
        let (physical, _) = super::super::tests::sources("package p { def Root { param x:Scalar=3; } }");
        let physical = runtime.physical_from_sources(&physical, &crate::CancelSource::new()).await.unwrap();
        let mut rows = runtime.operations().unwrap().put_physical_rows(&runtime, &physical).await.unwrap();
        if let Some(fault) = receipt_fault {
            let position = rows.revisions.iter().position(|revision| revision.problem.starts_with("physical:context:")).unwrap();
            let edits = if fault == "empty" { vec![] } else { vec![super::super::durable::source_edit("physical:context".into(), "physical".into(), "context".into(), "physical:rows:v1", vec![1])] };
            let bad = store.edit(&rows.revisions[position].problem, None, &format!("bad-context-receipt-{fault}"), &edits).await.unwrap();
            rows.revisions[position] = bad;
        }
        Some(rows)
    } else { None };
    store
        .begin_run(&RunRequest {
            key: "readable-fixture".into(),
            revision,
            sources: physical_rows.as_ref().map_or_else(Vec::new, |rows| rows.revisions.clone()),
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        })
        .await
        .unwrap();
    let fence = store
        .claim_run(
            "readable-fixture",
            "claim",
            "worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    super::super::result_projection::store_result_table(
        store,
        &fence,
        if nested {
            pse_relations::generated::runtime::study_outcomes::RELATION_ID
        } else {
            solve_variables::RELATION_ID
        },
        super::super::ResultCursor::from_checked(checked),
        &runtime.shared.pool(),
        &super::super::durable::PublicationAcks::new(&runtime.shared.pool()),
    )
    .await
    .unwrap();
    let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
    let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
    let completion = super::super::durable::inspection_fixture_completion(super::super::StoredCompletion {
        version: 2,
        attempt_id: pse_operations::mint_id(),
        completion: None,
        termination: super::super::TerminationDetail {
            version: pse_model::document::Version,
            cause: super::super::TerminationCause::Infrastructure { diagnostic: contract("fixture partial results have no scientific assessment").boundary_diagnostic() },
            point: None,
            retry_failure: None,
            effect: pse_model::study::EffectState::Absent,
        },
        state: pse_model::generated::enums::AttemptState::Failed,
        progress: super::super::ProgressCoverage::Unknown {},
    }).unwrap();
    // SAFETY: these generated observations assert only partial result availability.
    unsafe {
        store.seal_attempt(
            &manifest,
            "seal",
            TerminalClass::Partial,
            &completion,
            SeedEligibility::Unavailable,
        )
    }
    .await
    .unwrap();
    (runtime, fence, batch)
}

fn request(fence: &AttemptFence, view: ResultInspectionView) -> ResultInspectionRequest {
    ResultInspectionRequest {
        run: "readable-fixture".into(),
        attempt: fence.attempt().into(),
        relation: "runtime.solve_variables".into(),
        start: 0,
        end: 3,
        expected_manifest: None,
        cursor: None,
        view,
    }
}

#[tokio::test]
async fn canonical_result_physical_pages_reopen_original_context_bind_cursor_and_hold_protections() {
    let (runtime, fence, _) = wide_fixture_physical(false, true).await;
    let store = runtime.canonical_store().unwrap().clone();
    let mut query = request(&fence, ResultInspectionView::Physical { request: super::super::PhysicalInspectionRequest {
        query: super::super::PhysicalInspectionQuery::Quantities, offset: 0, limit: 1,
    }});
    let first = runtime.inspect_result(query.clone(), CancellationToken::new()).await.unwrap();
    let ResultInspectionDocument::Physical { original, page: Some(super::super::PhysicalInspectionDocument::Page { quantities, next, .. }), next_cursor: Some(cursor), selection, .. } = &first else { panic!("original physical owner must reopen: {first:?}") };
    assert_eq!(selection.usability, PackScientificState::Unknown);
    assert_eq!(original.governing_permission, None);
    assert!(matches!(original.physical, ResultPhysicalContext::Available { .. }));
    assert_eq!(quantities.len(), 1);
    assert_eq!(*next, Some(1));
    let original_id = quantities[0].quantity.id;
    let first_pins = original._custody.as_ref().unwrap()._physical.as_ref().unwrap()._source_owner.as_ref().unwrap().pins();
    assert!(!first_pins.is_empty());
    for pin in first_pins { pin.renew(Duration::from_secs(60)).await.unwrap(); }
    let first_pin_keys = first_pins.iter().map(|pin| pin.key().to_owned()).collect::<BTreeSet<_>>();
    let sources = store.analysis_run_sources("readable-fixture").await.unwrap();
    let physical_revision = sources.iter().find(|source| source.problem.starts_with("physical:context:")).unwrap();
    assert!(store.protect(physical_revision.clone(), Duration::from_secs(60)).await.is_ok());
    query.cursor = Some(cursor.clone());
    let second = runtime.inspect_result(query.clone(), CancellationToken::new()).await.unwrap();
    let ResultInspectionDocument::Physical { page: Some(super::super::PhysicalInspectionDocument::Page { quantities, .. }), original: second_original, .. } = &second else { panic!("second original physical page") };
    assert_ne!(quantities[0].quantity.id, original_id);
    assert_eq!(original.physical, second_original.physical);
    let second_pins = second_original._custody.as_ref().unwrap()._physical.as_ref().unwrap()._source_owner.as_ref().unwrap().pins();
    assert!(second_pins.iter().all(|pin| !first_pin_keys.contains(pin.key())), "each bounded page reacquires fresh exact source protection");
    for pin in second_pins { pin.renew(Duration::from_secs(60)).await.unwrap(); }
    let mut changed = query.clone();
    changed.cursor.as_mut().unwrap().physical = ResultPhysicalContext::Unavailable { reason: "different original context".into() };
    assert!(matches!(runtime.inspect_result(changed, CancellationToken::new()).await.unwrap(), ResultInspectionDocument::Refused { reason: ResultInspectionRefusal::CursorChanged, .. }));
    drop((first, second));
    pse_operations::testing::drain_result_releases(&store).await.unwrap();
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn canonical_result_missing_physical_closure_stays_unavailable_while_raw_values_remain_readable() {
    let (runtime, fence, _) = wide_fixture(false).await;
    let store = runtime.canonical_store().unwrap().clone();
    let result = runtime.inspect_result(request(&fence, ResultInspectionView::Values { fields: vec!["value".into()], maximum_rows: 1 }), CancellationToken::new()).await.unwrap();
    let ResultInspectionDocument::Values { original, rows, selection, .. } = &result else { panic!("original raw values") };
    assert_eq!(rows.len(), 1);
    assert_eq!(original.governing_permission, None);
    assert_eq!(selection.usability, PackScientificState::Unknown);
    assert!(matches!(original.physical, ResultPhysicalContext::Unavailable { .. }));
    drop(result);
    pse_operations::testing::drain_result_releases(&store).await.unwrap();
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn canonical_result_physical_context_requires_the_original_receipt_payload_not_just_its_hash() {
    for fault in ["empty", "wrong_kind"] {
        let (runtime, fence, _) = wide_fixture_receipt(false, true, Some(fault)).await;
        let store = runtime.canonical_store().unwrap().clone();
        let page = runtime.inspect_result(request(&fence, ResultInspectionView::Physical { request: super::super::PhysicalInspectionRequest { query: super::super::PhysicalInspectionQuery::Quantities, offset: 0, limit: 1 } }), CancellationToken::new()).await.unwrap();
        let ResultInspectionDocument::Physical { original, page: None, .. } = &page else { panic!("bad original receipt must be unavailable despite intact physical rows: {page:?}") };
        let ResultPhysicalContext::Unavailable { reason } = &original.physical else { panic!("unavailable original context") };
        assert!(reason.contains("receipt"), "preserve the explicit original receipt failure: {reason}");
        let values = runtime.inspect_result(request(&fence, ResultInspectionView::Values { fields: vec!["value".into()], maximum_rows: 1 }), CancellationToken::new()).await.unwrap();
        assert!(matches!(&values, ResultInspectionDocument::Values { rows, .. } if rows.len() == 1));
        drop((page, values));
        pse_operations::testing::drain_result_releases(&store).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
}

#[tokio::test]
#[allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "registered bounded protected-result fixture assertions"
)]
async fn canonical_nested_result_paths_retain_context_and_continuation() {
    let (runtime, fence, original) = wide_fixture(true).await;
    let mut selected = request(
        &fence,
        ResultInspectionView::Values {
            fields: vec![
                ArrowFieldPath {
                    field: "start".into(),
                    steps: vec![ArrowFieldStep::StructField {
                        name: "kind".into(),
                    }],
                },
                ArrowFieldPath {
                    field: "attempts".into(),
                    steps: vec![
                        ArrowFieldStep::ListElement { index: 0 },
                        ArrowFieldStep::StructField {
                            name: "usable".into(),
                        },
                    ],
                },
            ],
            maximum_rows: 1,
        },
    );
    selected.relation = "runtime.study_outcomes".into();
    let first = runtime
        .inspect_result(selected.clone(), CancellationToken::new())
        .await
        .unwrap();
    let ResultInspectionDocument::Values {
        rows,
        selected_fields,
        next_cursor,
        readable_complete,
        ..
    } = first
    else {
        panic!("expected readable nested values: {first:?}")
    };
    assert_eq!(
        rows[0].values[0],
        ResultInspectionCell::Text {
            value: "fresh".into()
        }
    );
    assert_eq!(
        rows[0].values[1],
        ResultInspectionCell::Boolean { value: false }
    );
    assert_eq!(selected_fields[0].path.field, "start");
    assert_eq!(selected_fields[0].ancestor_metadata.len(), 1);
    assert_eq!(
        selected_fields[0].ancestor_metadata_omitted[0][0].key,
        pse_schema::arrow::KEY_LOGICAL_TYPE
    );
    assert!(!rows[0].context_complete);
    assert!(
        !readable_complete,
        "readable selected cells do not assert complete context or scientific usability"
    );
    selected.cursor = next_cursor;
    let mut wrong = selected.clone();
    wrong.end = 2;
    assert!(matches!(
        runtime
            .inspect_result(wrong, CancellationToken::new())
            .await
            .unwrap(),
        ResultInspectionDocument::Refused {
            reason: ResultInspectionRefusal::CursorChanged,
            ..
        }
    ));
    let mut wrong = selected.clone();
    wrong.view = ResultInspectionView::Values {
        fields: vec!["usable".into()],
        maximum_rows: 1,
    };
    assert!(matches!(
        runtime
            .inspect_result(wrong, CancellationToken::new())
            .await
            .unwrap(),
        ResultInspectionDocument::Refused {
            reason: ResultInspectionRefusal::CursorChanged,
            ..
        }
    ));
    let second = runtime
        .inspect_result(selected.clone(), CancellationToken::new())
        .await
        .unwrap();
    let ResultInspectionDocument::Values {
        rows,
        next_cursor,
        selection,
        ..
    } = second
    else {
        panic!("expected second nested page")
    };
    assert_eq!(selection.row_start, 0);
    assert_eq!(selection.row_end, 3);
    assert_eq!(rows[0].source_row, 1);
    assert_eq!(
        rows[0].values[1],
        ResultInspectionCell::MissingElement {
            depth: 0,
            length: 0
        }
    );
    selected.cursor = next_cursor;
    let third = runtime
        .inspect_result(selected.clone(), CancellationToken::new())
        .await
        .unwrap();
    let ResultInspectionDocument::Values {
        rows, next_cursor, ..
    } = third
    else {
        panic!("expected null-parent page")
    };
    assert_eq!(
        rows[0].values[0],
        ResultInspectionCell::ParentNull { depth: 0 }
    );
    assert!(next_cursor.is_none());
    selected.cursor = None;
    selected.view = ResultInspectionView::Children {
        path: "start".into(),
        offset: 0,
        limit: 2,
    };
    let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
    let observed = hub.start("nested child metadata pages");
    let children = pse_diagnostics::observation::scope(
        Some(observed.context()),
        runtime.inspect_result(selected.clone(), CancellationToken::new()),
    )
    .await
    .unwrap();
    let ResultInspectionDocument::Schema {
        fields,
        field_count,
        next_cursor,
        ..
    } = children
    else {
        panic!("expected shallow child page")
    };
    assert_eq!(field_count, 5);
    assert_eq!(fields.len(), 2);
    assert!(next_cursor.is_some());
    assert_eq!(fields[0].name, "kind");
    selected.cursor = next_cursor;
    let children = pse_diagnostics::observation::scope(
        Some(observed.context()),
        runtime.inspect_result(selected, CancellationToken::new()),
    )
    .await
    .unwrap();
    let ResultInspectionDocument::Schema { fields, offset, .. } = children else {
        panic!("continued child page")
    };
    assert_eq!(offset, 2);
    assert_eq!(fields[0].name, "role");
    assert_eq!(
        decode_count(&hub),
        0,
        "child descriptor continuation must not read Arrow payloads"
    );
    let original_schema = original.schema();
    let start = original_schema.field_with_name("start").unwrap();
    let mut keys = start.metadata().keys().collect::<Vec<_>>();
    keys.sort();
    let offset = keys
        .iter()
        .position(|key| key.as_str() == pse_schema::arrow::KEY_LOGICAL_TYPE)
        .unwrap();
    let mut metadata_request = request(
        &fence,
        ResultInspectionView::Metadata {
            path: "start".into(),
            offset,
            limit: 1,
        },
    );
    metadata_request.relation = "runtime.study_outcomes".into();
    let detail = pse_diagnostics::observation::scope(
        Some(observed.context()),
        runtime.inspect_result(metadata_request, CancellationToken::new()),
    )
    .await
    .unwrap();
    let ResultInspectionDocument::Metadata { entries, .. } = detail else {
        panic!("exact deferred composite metadata")
    };
    assert_eq!(
        entries[pse_schema::arrow::KEY_LOGICAL_TYPE],
        start.metadata()[pse_schema::arrow::KEY_LOGICAL_TYPE]
    );
    assert_eq!(
        decode_count(&hub),
        0,
        "exact metadata detail must not decode Arrow payloads"
    );
    let store = runtime.canonical_store().unwrap();
    pse_operations::testing::drain_result_releases(store)
        .await
        .unwrap();
    store.forget_run_results("readable-fixture").await.unwrap();
    store.remove_isolated_fixture().await.unwrap();
}

fn decode_count(hub: &pse_diagnostics::observation::ObservationHub) -> u64 {
    hub.list(0, 128)
        .operations
        .iter()
        .filter_map(|operation| hub.reader(operation.id))
        .flat_map(|reader| reader.snapshot(Duration::from_secs(60)).metrics)
        .filter(|metric| metric.name == "canonical.result.native_decodes")
        .map(|metric| match metric.cumulative {
            pse_diagnostics::observation::Value::Count(value) => value,
            _ => 0,
        })
        .sum()
}

#[tokio::test]
async fn canonical_readable_wide_result_retains_physical_context_protection_and_exact_ipc() {
    let (runtime, fence, original) = wide_fixture(false).await;
    let store = runtime.canonical_store().unwrap().clone();
    let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
    let observed = hub.start("readable result acceptance");
    let discovered = pse_diagnostics::observation::scope(
        Some(observed.context()),
        runtime.inspect_result(
            request(
                &fence,
                ResultInspectionView::Schema {
                    offset: 0,
                    limit: 2,
                },
            ),
            CancellationToken::new(),
        ),
    )
    .await
    .unwrap();
    assert_eq!(
        decode_count(&hub),
        0,
        "schema inquiry must not decode original Arrow data"
    );
    let ResultInspectionDocument::Schema {
        manifest,
        field_count,
        fields,
        next,
        recorded_rows,
        selection,
        ..
    } = &discovered
    else {
        panic!("schema page: {discovered:?}")
    };
    assert_eq!(*field_count, 19);
    assert!(
        *field_count > 16,
        "registered relation exceeds the schema page cap"
    );
    assert_eq!(fields.len(), 2);
    assert_eq!(*next, Some(2));
    assert_eq!(*recorded_rows, 3);
    assert_eq!(selection.usability, PackScientificState::Unknown);
    let mut schema_request = request(
        &fence,
        ResultInspectionView::Schema {
            offset: 0,
            limit: 2,
        },
    );
    schema_request.cursor = match &discovered {
        ResultInspectionDocument::Schema { next_cursor, .. } => next_cursor.clone(),
        _ => unreachable!(),
    };
    let continued = pse_diagnostics::observation::scope(
        Some(observed.context()),
        runtime.inspect_result(schema_request.clone(), CancellationToken::new()),
    )
    .await
    .unwrap();
    let ResultInspectionDocument::Schema { offset, fields, .. } = &continued else {
        panic!("schema continuation")
    };
    assert_eq!(*offset, 2);
    assert_eq!(fields[0].name, *original.schema().field(2).name());
    for changed_schema in [false, true] {
        let mut wrong = schema_request.clone();
        let cursor = wrong.cursor.as_mut().unwrap();
        if changed_schema {
            cursor.schema_fingerprint.push_str("-different");
        } else {
            cursor.issuing_store.push_str("-different");
        }
        assert!(matches!(
            runtime
                .inspect_result(wrong, CancellationToken::new())
                .await
                .unwrap(),
            ResultInspectionDocument::Refused {
                reason: ResultInspectionRefusal::CursorChanged,
                ..
            }
        ));
    }
    let original_schema = original.schema();
    let metadata_field = original_schema
        .fields()
        .iter()
        .max_by_key(|field| field.metadata().len())
        .unwrap();
    let expected_metadata = metadata_field
        .metadata()
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    assert!(
        expected_metadata.len() > 1,
        "fixture must exercise original metadata continuation"
    );
    let mut metadata_request = request(
        &fence,
        ResultInspectionView::Metadata {
            path: metadata_field.name().as_str().into(),
            offset: 0,
            limit: 1,
        },
    );
    let mut collected = BTreeMap::new();
    for position in 0..expected_metadata.len() {
        let page = pse_diagnostics::observation::scope(
            Some(observed.context()),
            runtime.inspect_result(metadata_request.clone(), CancellationToken::new()),
        )
        .await
        .unwrap();
        let ResultInspectionDocument::Metadata {
            offset,
            entries,
            next_cursor,
            entry_count,
            ..
        } = page
        else {
            panic!("exact metadata page")
        };
        assert_eq!(offset, position);
        assert_eq!(entry_count, expected_metadata.len());
        assert_eq!(entries.len(), 1);
        collected.extend(entries);
        metadata_request.cursor = next_cursor;
    }
    assert!(metadata_request.cursor.is_none());
    assert_eq!(collected, expected_metadata);
    assert_eq!(
        decode_count(&hub),
        0,
        "schema and context-detail continuation must not decode Arrow payloads"
    );
    let cancelled = CancellationToken::new();
    cancelled.cancel();
    assert!(
        runtime
            .inspect_result(schema_request, cancelled)
            .await
            .is_err()
    );
    let mut selected = request(
        &fence,
        ResultInspectionView::Values {
            fields: vec!["value".into()],
            maximum_rows: 2,
        },
    );
    selected.expected_manifest = Some(manifest.clone());
    let values = pse_diagnostics::observation::scope(
        Some(observed.context()),
        runtime.inspect_result(selected.clone(), CancellationToken::new()),
    )
    .await
    .unwrap();
    assert!(decode_count(&hub) > 0);
    let ResultInspectionDocument::Values {
        selected_fields,
        context_fields,
        rows,
        recorded_rows,
        next_row,
        readable_complete,
        schema_metadata,
        ..
    } = &values
    else {
        panic!("readable values: {values:?}")
    };
    assert_eq!(selected_fields[0].name, "value");
    assert_eq!(
        context_fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        [
            "run_id",
            "step",
            "symbol_id",
            "quantity_id",
            "unit_id",
            "fixed",
            "parameter",
            "domain",
            "dual_qualification",
        ]
    );
    assert_eq!(*recorded_rows, 3);
    assert_eq!(*next_row, Some(2));
    assert!(*readable_complete);
    assert_eq!(rows[0].source_row, 0);
    assert_eq!(
        rows[0].values,
        [ResultInspectionCell::Float64 {
            bits: (-0.0f64).to_bits(),
            finite_value: Some(-0.0),
            class: ResultFloatClass::Finite
        }]
    );
    assert_eq!(rows[1].values, [ResultInspectionCell::Null]);
    assert_eq!(
        rows[0].context[3],
        ResultInspectionCell::Binary {
            hex: "44".repeat(16)
        }
    );
    assert_eq!(
        rows[0].context[4],
        ResultInspectionCell::Binary {
            hex: "55".repeat(16)
        }
    );
    assert_eq!(
        rows[1].context[5],
        ResultInspectionCell::Boolean { value: true }
    );
    assert_eq!(rows[0].context[7], ResultInspectionCell::Null);
    assert_eq!(
        rows[0].context[8],
        ResultInspectionCell::Text {
            value: "unavailable".into()
        },
        "original declared unavailable dual qualification, not scientific usability"
    );
    // Every descriptor retains the registered original interpretation. Opaque unit
    // identities do not establish resolved definitions or a physical basis.
    for field in selected_fields.iter().chain(context_fields) {
        let original_schema = original.schema();
        let source = original_schema.field(field.index);
        assert_eq!(field.name, *source.name());
        assert_eq!(field.nullable, source.is_nullable());
        assert_eq!(
            match &field.storage {
                ResultInspectionStorage::Scalar { datatype_json } =>
                    serde_json::from_str::<DataType>(datatype_json).unwrap(),
                _ => panic!("wide fixture is scalar"),
            },
            *source.data_type()
        );
        assert_eq!(
            field.metadata,
            source
                .metadata()
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<BTreeMap<_, _>>()
        );
    }
    assert_eq!(
        schema_metadata.get(pse_schema::arrow::KEY_CONTRACT_FINGERPRINT),
        original
            .schema()
            .metadata()
            .get(pse_schema::arrow::KEY_CONTRACT_FINGERPRINT)
    );
    assert!(serde_json::to_vec(&values).unwrap().len() < DOCUMENT_BYTES);
    selected.cursor = match &values {
        ResultInspectionDocument::Values { next_cursor, .. } => next_cursor.clone(),
        _ => panic!("values cursor"),
    };
    let last = runtime
        .inspect_result(selected, CancellationToken::new())
        .await
        .unwrap();
    let ResultInspectionDocument::Values { rows, next_row, .. } = &last else {
        panic!("continuation")
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source_row, 2);
    assert_eq!(*next_row, None);
    assert_eq!(
        rows[0].values[0],
        ResultInspectionCell::Float64 {
            bits: 273.15f64.to_bits(),
            finite_value: Some(273.15),
            class: ResultFloatClass::Finite
        }
    );
    let ipc = pse_columnar::ipc_file::encode(
        std::slice::from_ref(&original),
        super::super::result_files::MAX_FILE_BYTES,
    )
    .unwrap();
    let independent =
        datafusion::arrow::ipc::reader::FileReader::try_new(std::io::Cursor::new(ipc), None)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
    let mut reader = runtime
        .results(
            "readable-fixture",
            fence.attempt(),
            "runtime.solve_variables",
            0,
            3,
            CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(reader.next_batch().await.unwrap().unwrap(), independent);
    assert!(reader.next_batch().await.unwrap().is_none());
    reader.close();
    drop(reader);
    for (view, reason) in [
        (
            ResultInspectionView::Values {
                fields: vec!["absent".into()],
                maximum_rows: 1,
            },
            ResultInspectionRefusal::UnknownField,
        ),
        (
            ResultInspectionView::Values {
                fields: vec!["value".into()],
                maximum_rows: 17,
            },
            ResultInspectionRefusal::Selection,
        ),
        (
            ResultInspectionView::Schema {
                offset: 0,
                limit: 17,
            },
            ResultInspectionRefusal::Selection,
        ),
    ] {
        assert!(
            matches!(runtime.inspect_result(request(&fence, view), CancellationToken::new()).await.unwrap(), ResultInspectionDocument::Refused { reason: actual, .. } if actual == reason)
        );
    }
    let mut stale = request(
        &fence,
        ResultInspectionView::Schema {
            offset: 0,
            limit: 1,
        },
    );
    stale.expected_manifest = Some("different-original-manifest".into());
    assert!(matches!(
        runtime
            .inspect_result(stale, CancellationToken::new())
            .await
            .unwrap(),
        ResultInspectionDocument::Refused {
            reason: ResultInspectionRefusal::ManifestChanged,
            ..
        }
    ));
    drop((discovered, continued, values, last));
    pse_operations::testing::drain_result_releases(&store)
        .await
        .unwrap();
    store.forget_run_results("readable-fixture").await.unwrap();
    assert!(
        runtime
            .inspect_result(
                request(
                    &fence,
                    ResultInspectionView::Schema {
                        offset: 0,
                        limit: 1
                    }
                ),
                CancellationToken::new()
            )
            .await
            .is_err(),
        "released results must not be silently reopened or replaced"
    );
    store.remove_isolated_fixture().await.unwrap();
}
