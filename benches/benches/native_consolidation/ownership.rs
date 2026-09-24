// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Copy, shared ports, tiny retained views and the final foreign Arrow reader.
use arrow::{
    array::{Int64Array, RecordBatch},
    datatypes::{DataType, Field, Schema},
};
use criterion::{BenchmarkId, Criterion};
use datafusion::execution::memory_pool::PeakRecordingPool;

use pse_columnar::{
    CancellationToken, GreedyMemoryPool, MemoryPool, owned_buffer::OwnedRecordBatch,
};
use std::{hint::black_box, sync::Arc, time::Instant};

pub(super) fn measure(c: &mut Criterion) {
    let mut receipts = Vec::new();
    let mut group = c.benchmark_group("integrated/ownership");
    for rows in [1, 1024, 65_536] {
        let peak = Arc::new(PeakRecordingPool::new(Arc::new(GreedyMemoryPool::new(
            64 << 20,
        ))));
        let pool: Arc<dyn MemoryPool> = peak.clone();
        let source = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "value",
                DataType::Int64,
                false,
            )])),
            vec![Arc::new(Int64Array::from_iter_values(0..rows))],
        )
        .unwrap();
        let cancel = CancellationToken::new();
        group.bench_with_input(BenchmarkId::new("foreign-copy", rows), &rows, |b, _| {
            b.iter(|| black_box(OwnedRecordBatch::copy(&source, &pool, &cancel).unwrap()));
        });
        let started = Instant::now();
        let owned = OwnedRecordBatch::copy(&source, &pool, &cancel).unwrap();
        let copy_seconds = started.elapsed().as_secs_f64();
        assert_eq!(owned.batch(), &source);
        assert_ne!(
            source.column(0).to_data().buffers()[0].as_ptr(),
            owned.column(0).to_data().buffers()[0].as_ptr()
        );
        let extent = owned.retained_bytes().unwrap();
        group.bench_with_input(
            BenchmarkId::new("known-owner-export", rows),
            &rows,
            |b, _| {
                b.iter(|| {
                    black_box(
                        owned
                            .ownership()
                            .export(owned.batch().clone(), &pool, &cancel)
                            .unwrap(),
                    )
                });
            },
        );
        let left = owned.project(&[0]).unwrap();
        let right = owned.project(&[0]).unwrap();
        let slice = right.slice(0, 1).unwrap();
        assert_eq!(slice.retained_bytes().unwrap(), extent);
        assert_eq!(pool.reserved(), extent);
        group.bench_with_input(
            BenchmarkId::new("compact-one-row-view", rows),
            &rows,
            |b, _| {
                b.iter(|| black_box(slice.compact(&pool, &cancel).unwrap()));
            },
        );
        let compact = slice.compact(&pool, &cancel).unwrap();
        let compact_extent = compact.retained_bytes().unwrap();
        drop(compact);
        let started = Instant::now();
        let foreign = arrow::ffi::FFI_ArrowArray::new(&slice.column(0).to_data());
        let export_seconds = started.elapsed().as_secs_f64();
        drop(owned);
        drop(left);
        drop(right);
        drop(slice);
        let foreign_retained = pool.reserved();
        assert_eq!(foreign_retained, extent);
        drop(foreign);
        assert_eq!(
            pool.reserved(),
            0,
            "the last foreign reader releases the source pool"
        );
        receipts.push(serde_json::json!({
            "rows": rows, "columns": 1, "ports": 2, "slice_rows": 1,
            "copy_seconds": copy_seconds, "ffi_export_seconds": export_seconds,
            "source_visible_bytes": source.get_array_memory_size(),
            "completed_owner_bytes": extent, "compact_owner_bytes": compact_extent,
            "foreign_retained_after_parent_drop_bytes": foreign_retained,
            "reserved_after_last_reader_bytes": pool.reserved(),
            "pool_peak_bytes": peak.max_reserved(),
            "copy_boundary": "foreign ingress; known-owner exports retain the same buffers",
            "pool_budget_bytes": 64 << 20
        }));
    }
    group.finish();
    if let Some(output) = std::env::var_os("PSE_ACCEPTANCE_OUTPUT") {
        std::fs::write(
            std::path::PathBuf::from(output).join("ownership-counts.json"),
            serde_json::to_vec_pretty(&receipts).unwrap(),
        )
        .unwrap();
    }
}
