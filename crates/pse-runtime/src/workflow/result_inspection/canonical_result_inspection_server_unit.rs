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
async fn wide_fixture() -> (Runtime, AttemptFence, RecordBatch) {
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
    let checked = builder.finish().unwrap();
    let batch = checked.batch().clone();
    let store = runtime.canonical_store().unwrap();
    let revision = store
        .edit("readable-fixture", None, "source", &[])
        .await
        .unwrap();
    store
        .begin_run(&RunRequest {
            key: "readable-fixture".into(),
            revision,
            sources: vec![],
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
        solve_variables::RELATION_ID,
        super::super::ResultCursor::from_checked(checked),
        &runtime.shared.pool(),
        &super::super::durable::PublicationAcks::new(&runtime.shared.pool()),
    )
    .await
    .unwrap();
    let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
    let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
    // SAFETY: these generated observations assert only partial result availability.
    unsafe {
        store.seal_attempt(
            &manifest,
            "seal",
            TerminalClass::Partial,
            &[1],
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
        view,
    }
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
    let (runtime, fence, original) = wide_fixture().await;
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
            serde_json::from_str::<DataType>(&field.datatype_json).unwrap(),
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
    selected.start = 2;
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
    drop((discovered, values, last));
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
