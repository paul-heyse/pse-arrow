// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated E4 controls: only visible values, not null payloads or batching, are identity input.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "bounded test fixture construction reports direct assertion failures"
)]

use arrow_array::{Float64Array, RecordBatch, UInt32Array};
use arrow_buffer::{NullBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field, Schema};
use proptest::prelude::*;
use pse_ids::{
    CanonicalContract, CanonicalizeOptions, ContentHash, FixedBudget, SchemaVersion, SemanticId,
};
use std::collections::BTreeMap;
use std::sync::Arc;

fn contract() -> CanonicalContract {
    CanonicalContract::try_new(
        SemanticId::from_bytes([1; 16]),
        SchemaVersion(1),
        ContentHash::from_bytes([2; 32]),
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::UInt32, false),
            Field::new("value", DataType::Float64, true),
        ])),
        &["id"],
        &BTreeMap::new(),
    )
    .expect("declared property relation")
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, max_shrink_iters: 1024, ..ProptestConfig::default() })]
    #[test]
    fn arbitrary_null_payloads_and_batch_boundaries_do_not_enter_the_preimage(
        values in prop::collection::vec((any::<u64>(), any::<bool>()), 1..65),
        garbage in any::<u64>(),
        split in 0usize..65,
    ) {
        let contract = contract();
        let make = |hidden| RecordBatch::try_new(Arc::clone(&contract.schema), vec![
            Arc::new(UInt32Array::from_iter_values((0..u32::try_from(values.len()).expect("bounded")).rev())),
            Arc::new(Float64Array::new(ScalarBuffer::from(values.iter().map(|(bits, valid)|
                f64::from_bits(if *valid { *bits } else { hidden })).collect::<Vec<_>>()),
                Some(NullBuffer::from(values.iter().map(|value| value.1).collect::<Vec<_>>())))),
        ]).expect("bounded actual Arrow rows");
        let budget = FixedBudget::new(16 << 20);
        let options = CanonicalizeOptions { keep_preimage: true, ..Default::default() };
        let one = make(0);
        let other = make(garbage);
        let boundary = split.min(values.len());
        let expected = pse_ids::canonicalize(&contract, &[one], budget.as_ref(), options.clone()).expect("admitted");
        let actual = pse_ids::canonicalize(&contract,
            &[other.slice(boundary, values.len() - boundary), other.slice(0, boundary)],
            budget.as_ref(), options).expect("admitted alternate");
        prop_assert_eq!(&actual.preimage, &expected.preimage);
        drop((actual, expected));
        prop_assert_eq!(budget.reserved(), 0);
    }
}

#[cfg(feature = "bench-instrumentation")]
#[test]
fn instrumentation_keeps_the_production_admission_and_reservation_boundary() {
    let contract = contract();
    let source = RecordBatch::try_new(
        Arc::clone(&contract.schema),
        vec![
            Arc::new(UInt32Array::from(vec![1, 0])),
            Arc::new(Float64Array::from(vec![Some(-0.0), None])),
        ],
    )
    .expect("source");
    let budget = FixedBudget::new(16 << 20);
    let options = CanonicalizeOptions {
        keep_preimage: true,
        keep_sorted: true,
        ..Default::default()
    };
    let (observed, timings) = pse_ids::canon::benchmark::canonicalize_stages(
        &contract,
        std::slice::from_ref(&source),
        budget.as_ref(),
        options.clone(),
    )
    .expect("actual stages");
    let ordinary = pse_ids::canonicalize(
        &contract,
        std::slice::from_ref(&source),
        budget.as_ref(),
        options.clone(),
    )
    .expect("ordinary");
    assert_eq!(observed.preimage, ordinary.preimage);
    assert_eq!(observed.sorted, ordinary.sorted);
    assert_eq!(timings.stages().len(), 6);
    assert!(budget.reserved() > 0);
    drop((ordinary, observed));
    assert_eq!(budget.reserved(), 0);
    assert!(
        pse_ids::canon::benchmark::canonicalize_stages(
            &contract,
            &[source],
            FixedBudget::new(1).as_ref(),
            options
        )
        .is_err(),
        "instrumentation cannot bypass reservation"
    );
}
