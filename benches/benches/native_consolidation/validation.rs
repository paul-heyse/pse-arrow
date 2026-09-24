// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native validation shape and null-density matrix; preparation is outside evaluation.
use arrow::{
    array::{Array, ArrayRef, Int64Array, ListArray, RecordBatch},
    datatypes::{DataType, Field, Int64Type, Schema},
};
use criterion::{BenchmarkId, Criterion, Throughput};
use pse_columnar::CancellationToken;
use pse_relations::validate::ValidationContext;
use std::{hint::black_box, sync::Arc};

pub(super) fn measure(c: &mut Criterion) {
    let fixture = pse_testkit::NativeFixture::new((128 << 20).try_into().unwrap()).unwrap();
    let registry = pse_schema::RegistryBuilder::new().build().unwrap();
    let context = ValidationContext::new(
        &registry,
        pse_engine::validation::NativeValidation(fixture.factory.native_state().clone()),
    );
    let cancel = CancellationToken::new();
    let mut group = c.benchmark_group("consolidation/validation-shape");
    for rows in [1, 1024, 65_536] {
        for null_percent in [0, 25, 100] {
            let values = (0..rows).map(|n| (n % 100 >= null_percent).then_some(i64::from(n)));
            let scalar: ArrayRef = Arc::new(Int64Array::from_iter(values.clone()));
            let nested: ArrayRef = Arc::new(ListArray::from_iter_primitive::<Int64Type, _, _>(
                values.map(|v| v.map(|n| vec![Some(n), Some(n + 1)])),
            ));
            for (kind, array) in [("scalar", scalar), ("list", nested)] {
                let field = Field::new("value", array.data_type().clone(), true);
                let prepared = context.column(&registry, &field).unwrap();
                let batch =
                    RecordBatch::try_new(Arc::new(Schema::new(vec![field])), vec![array]).unwrap();
                group.throughput(Throughput::Elements(u64::try_from(rows).unwrap()));
                group.bench_with_input(
                    BenchmarkId::new(format!("{kind}-null{null_percent}"), rows),
                    &rows,
                    |b, _| b.iter(|| black_box(prepared.evaluate(&batch, 16, &cancel).unwrap())),
                );
            }
        }
    }
    group.finish();
    let field = Field::new("value", DataType::Int64, false);
    c.bench_function("consolidation/registry-preparation", |b| {
        b.iter(|| {
            let registry = pse_schema::catalog::assemble().unwrap();
            let relation = registry.relations().first().unwrap();
            black_box(registry.contract(relation).unwrap());
        });
    });
    c.bench_function("consolidation/field-preparation", |b| {
        b.iter(|| {
            let context = ValidationContext::new(
                &registry,
                pse_engine::validation::NativeValidation(fixture.factory.native_state().clone()),
            );
            black_box(context.column(&registry, &field).unwrap());
        });
    });
}
