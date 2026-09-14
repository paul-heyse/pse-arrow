// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual schema-admitted canonicalization of unsorted, nullable engineering values.
#![allow(
    clippy::expect_used,
    reason = "benchmark fixture assertions have fixed inputs"
)]

use arrow::{
    array::{Float64Array, RecordBatch, UInt32Array},
    datatypes::{DataType, Field, Schema},
};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use pse_ids::canon::benchmark::{StageTimings, canonicalize_stages};
use pse_ids::{
    CancellationToken, CanonicalContract, CanonicalizeOptions, ContentHash, FixedBudget,
    SchemaVersion, SemanticId, canonicalize,
};
use std::{collections::BTreeMap, hint::black_box, sync::Arc};

fn canonicalization(c: &mut Criterion) {
    let schema = Arc::new(Schema::new(vec![
        Field::new("key", DataType::UInt32, false),
        Field::new("value", DataType::Float64, true),
    ]));
    let contract = CanonicalContract::try_new(
        SemanticId::from_bytes([1; 16]),
        SchemaVersion(1),
        ContentHash::from_bytes([2; 32]),
        Arc::clone(&schema),
        &["key"],
        &BTreeMap::new(),
    )
    .expect("declared benchmark relation");
    let budget = FixedBudget::new(128 << 20);
    let cancel = CancellationToken::default();
    for (stage, label) in std::iter::once((None, "whole")).chain(
        StageTimings::default()
            .stages()
            .into_iter()
            .enumerate()
            .map(|(i, (name, _))| (Some(i), name)),
    ) {
        let mut group = c.benchmark_group(format!("canonicalization/{label}"));
        for count in [1_000_u32, 10_000] {
            let batch = RecordBatch::try_new(
                Arc::clone(&schema),
                vec![
                    Arc::new(UInt32Array::from_iter_values((0..count).rev())),
                    Arc::new(Float64Array::from_iter(
                        (0..count).map(|row| (row % 7 != 0).then_some(f64::from(row) / 3.)),
                    )),
                ],
            )
            .expect("actual Arrow input");
            group.throughput(Throughput::Elements(u64::from(count)));
            group.bench_with_input(
                BenchmarkId::new("unsorted_nullable", count),
                &batch,
                |b, batch| {
                    if let Some(stage) = stage {
                        // Only this actual stage contributes elapsed time. Every iteration
                        // still executes the complete reserved pipeline to produce its input.
                        b.iter_custom(|iterations| {
                            (0..iterations)
                                .map(|_| {
                                    let (output, times) = canonicalize_stages(
                                        &contract,
                                        std::slice::from_ref(black_box(batch)),
                                        budget.as_ref(),
                                        CanonicalizeOptions {
                                            cancel: Some(cancel.clone()),
                                            ..Default::default()
                                        },
                                    )
                                    .expect("reserved production stages");
                                    black_box(output);
                                    times.stages()[stage].1
                                })
                                .sum()
                        });
                    } else {
                        b.iter(|| {
                            canonicalize(
                                &contract,
                                std::slice::from_ref(black_box(batch)),
                                budget.as_ref(),
                                CanonicalizeOptions {
                                    cancel: Some(cancel.clone()),
                                    ..Default::default()
                                },
                            )
                            .expect("complete canonicalization")
                        });
                    }
                },
            );
        }
        group.finish();
    }
}

criterion_group!(benches, canonicalization);
criterion_main!(benches);
