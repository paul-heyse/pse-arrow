// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generic tagged payloads retain their selected arm across typed and raw boundaries.
#![allow(clippy::unwrap_used, reason = "explicit boundary regression fixtures")]

use arrow::buffer::NullBuffer;
use arrow_array::{Array, StringArray, StructArray};
use pse_ids::SemanticId;
use pse_relations::generated::enums::InputConsumptionKind;
use pse_relations::generated::reference::algorithm_arguments::{
    self as arguments, ReferenceAlgorithmArgumentsFieldConsumption as Consumption,
    ReferenceAlgorithmArgumentsFieldConsumptionColumns as Columns,
    ReferenceAlgorithmArgumentsFieldConsumptionSelected as Selected,
};
use std::sync::Arc;

fn row(consumption: Consumption) -> arguments::Row {
    arguments::Row {
        algorithm_id: SemanticId::from_bytes([1; 16]),
        port: "source".into(),
        relation_id: SemanticId::from_bytes([2; 16]),
        required: true,
        consumption,
    }
}

#[test]
fn tagged_consumption_round_trips_and_rejects_missing_or_overlapping_arms() {
    let columns = Columns {
        names: vec!["value".into()],
    };
    let values = [
        Consumption::from_whole(),
        Consumption::from_columns(columns.clone()),
    ];
    let mut builder = arguments::Builder::new().unwrap();
    for value in &values {
        builder.push(row(value.clone())).unwrap();
    }
    let batch = builder.finish().unwrap();
    assert_eq!(
        arguments::View::from_checked(&batch)
            .unwrap()
            .rows()
            .unwrap(),
        values.into_iter().map(row).collect::<Vec<_>>()
    );
    arguments::validate(batch.batch()).unwrap();
    for value in [
        Consumption {
            kind: InputConsumptionKind::Columns,
            columns: None,
        },
        Consumption {
            kind: InputConsumptionKind::Whole,
            columns: Some(columns),
        },
    ] {
        assert!(value.selected().is_err());
        let mut invalid = arguments::Builder::new().unwrap();
        invalid.push(row(value)).unwrap();
        assert!(invalid.finish().is_err());
    }
}

#[test]
fn tagged_consumption_ignores_masked_payload_and_rejects_visible_overlap() {
    let mut builder = arguments::Builder::new().unwrap();
    builder
        .push(row(Consumption::from_columns(Columns {
            names: vec!["hidden".into()],
        })))
        .unwrap();
    let batch = builder.finish().unwrap().into_batch();
    let position = batch.schema().index_of("consumption").unwrap();
    let consumption = batch
        .column(position)
        .as_any()
        .downcast_ref::<StructArray>()
        .unwrap();
    let payload = consumption
        .column(1)
        .as_any()
        .downcast_ref::<StructArray>()
        .unwrap();
    for masked in [false, true] {
        let payload = StructArray::new(
            payload.fields().clone(),
            payload.columns().to_vec(),
            masked.then(|| NullBuffer::new_null(1)),
        );
        let candidate = StructArray::new(
            consumption.fields().clone(),
            vec![
                Arc::new(StringArray::from(vec!["whole"])),
                Arc::new(payload),
            ],
            None,
        );
        let mut columns = batch.columns().to_vec();
        columns[position] = Arc::new(candidate);
        let candidate = pse_relations::RecordBatch::try_new(batch.schema(), columns).unwrap();
        assert_eq!(arguments::validate(&candidate).is_ok(), masked);
        if masked {
            let row = arguments::View::try_from_batch(&candidate)
                .unwrap()
                .row(0)
                .unwrap();
            assert_eq!(row.consumption.selected().unwrap(), Selected::Whole);
        }
    }
}
