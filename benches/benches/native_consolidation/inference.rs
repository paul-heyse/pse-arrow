// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual inference key allocations and bounded-cache reset behavior.
use criterion::Criterion;
use pse_quantity::{
    IndexSet,
    infer::{InferenceCache, NoInvariantFacts, OpRequest, Operand},
};
use std::hint::black_box;

pub(super) fn measure(c: &mut Criterion) {
    let registry = pse_quantity::generated::standard_registry().unwrap();
    let quantity = registry.neutral_dimensionless().unwrap();
    let indices = IndexSet::new();
    let operands = [Operand {
        quantity_type: quantity,
        indices: &indices,
    }];
    let checker = NoInvariantFacts;
    let mut cache = InferenceCache::new(&registry, 1);
    let expected = cache
        .infer(&OpRequest::Neg, &operands, &checker)
        .unwrap()
        .result;
    let allocations = allocation_counter::measure(|| {
        for _ in 0..1000 {
            assert_eq!(
                cache
                    .infer(&OpRequest::Neg, &operands, &checker)
                    .unwrap()
                    .result,
                expected
            );
        }
    });
    let warm_counts = cache.counts();
    let mut group = c.benchmark_group("integrated/inference-cache");
    group.bench_function("warm-hit", |b| {
        b.iter(|| black_box(cache.infer(&OpRequest::Neg, &operands, &checker).unwrap()));
    });
    group.bench_function("capacity-one-reset", |b| {
        b.iter(|| {
            black_box(cache.infer(&OpRequest::Abs, &operands, &checker).unwrap());
            black_box(cache.infer(&OpRequest::Neg, &operands, &checker).unwrap());
        });
    });
    group.finish();
    if let Some(output) = std::env::var_os("PSE_ACCEPTANCE_OUTPUT") {
        let counts = serde_json::json!({"warm_requests":1000,"warm_key_allocations":allocations.count_total,"warm_allocated_bytes":allocations.bytes_total,"warm_hits_misses_resets":warm_counts,"final_hits_misses_resets":cache.counts()});
        std::fs::write(
            std::path::PathBuf::from(output).join("inference-counts.json"),
            serde_json::to_vec_pretty(&counts).unwrap(),
        )
        .unwrap();
    }
}
