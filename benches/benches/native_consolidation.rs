// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Current native/computation measurements. Compile before W18; execute only through the guarded recipe.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benchmark fixtures fail on invalid construction"
)]
use arrow::{
    array::{Int64Array, RecordBatch},
    datatypes::{DataType, Field, Schema},
};
use criterion::{Criterion, criterion_group, criterion_main};
#[path = "native_consolidation/inference.rs"]
mod inference;
#[path = "native_consolidation/observation.rs"]
mod observation;
#[path = "native_consolidation/ownership.rs"]
mod ownership;
#[path = "native_consolidation/validation.rs"]
mod validation;
use pse_relations::validate::ValidationContext;
use std::{hint::black_box, sync::Arc};

fn local_validation(c: &mut Criterion) {
    let registry = pse_schema::RegistryBuilder::new().build().unwrap();
    let field = Field::new("value", DataType::Int64, false);
    let fixture = pse_testkit::NativeFixture::new((128 << 20).try_into().unwrap()).unwrap();
    let state = fixture.factory.native_state().clone();
    let context = ValidationContext::new(
        &registry,
        pse_engine::validation::NativeValidation(state.clone()),
    );
    let prepared = context.column(&registry, &field).unwrap();
    let batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![field.clone()])),
        vec![Arc::new(Int64Array::from_iter_values(0..65_536))],
    )
    .unwrap();
    let cancel = pse_columnar::CancellationToken::new();
    let mut group = c.benchmark_group("consolidation/native-local-validation");
    group.bench_function("prepare-cold", |b| {
        b.iter(|| {
            let owner = ValidationContext::new(
                &registry,
                pse_engine::validation::NativeValidation(state.clone()),
            );
            black_box(owner.column(&registry, &field).unwrap());
        });
    });
    group.bench_function("prepare-reused", |b| {
        b.iter(|| black_box(context.column(&registry, &field).unwrap()));
    });
    group.bench_function("evaluate-65536", |b| {
        b.iter(|| black_box(prepared.evaluate(&batch, 16, &cancel).unwrap()));
    });
    group.finish();
}

fn consolidation(c: &mut Criterion) {
    local_validation(c);
    let mut group = c.benchmark_group("consolidation/native-value-framing");
    for rows in [1, 1024, 65_536] {
        let array: arrow::array::ArrayRef = Arc::new(Int64Array::from_iter_values(0..rows));
        let field = Arc::new(Field::new("key", DataType::Int64, false));
        let columns = [("key", Arc::clone(&field), Arc::clone(&array))];
        group.bench_with_input(
            criterion::BenchmarkId::new("row-tokens", rows),
            &rows,
            |b, _| {
                b.iter(|| {
                    black_box(
                        pse_columnar::row_token::tokens(
                            pse_ids::SemanticId::NIL,
                            &columns,
                            array.len(),
                        )
                        .unwrap(),
                    )
                });
            },
        );
        group.bench_with_input(
            criterion::BenchmarkId::new("literal-encoding", rows),
            &rows,
            |b, _| {
                b.iter(|| {
                    for row in 0..array.len() {
                        black_box(
                            pse_columnar::native_value::to_json(array.as_ref(), &field, row)
                                .unwrap(),
                        );
                    }
                });
            },
        );
    }
    group.finish();
    let registry = pse_schema::registry().unwrap();
    c.bench_function("consolidation/self-description-shared-buffers", |b| {
        b.iter(|| black_box(registry.schema_batches().to_vec()));
    });
    let declared = registry.relations().first().unwrap();
    let handle = registry.contract(declared).unwrap();
    let independent = pse_schema::catalog::assemble().unwrap();
    let foreign = independent
        .contract(independent.relation_by_id(declared.id).unwrap())
        .unwrap();
    c.bench_function("consolidation/equivalent-foreign-contract", |b| {
        b.iter(|| handle.require_equivalent(black_box(&foreign)).unwrap());
    });
    c.bench_function("consolidation/admitted-contract-borrow", |b| {
        b.iter(|| black_box(handle.clone()));
    });
    c.bench_function("consolidation/native-error-tree", |b| {
        b.iter(|| {
            black_box(pse_columnar::classify(
                datafusion::common::DataFusionError::Collection(vec![
                    datafusion::common::DataFusionError::Plan("fixture".into()),
                    datafusion::common::DataFusionError::Execution("fixture".into()),
                ]),
                pse_columnar::PlanOrigin::RuleCompiler,
            ))
        });
    });
    c.bench_function("consolidation/rust-contract-generation", |b| {
        b.iter(|| {
            black_box(
                pse_codegen::codegen::generate(registry, pse_codegen::codegen::Language::Rust)
                    .unwrap(),
            );
        });
    });
}
fn configuration() -> Criterion {
    let base = Criterion::default();
    match std::env::var_os("PSE_ACCEPTANCE_OUTPUT") {
        Some(path) => {
            base.output_directory(&std::path::PathBuf::from(path).join("measurements/criterion"))
        }
        None => base,
    }
}
criterion_group! { name = benches; config = configuration(); targets = consolidation, validation::measure, observation::measure, ownership::measure, inference::measure }

criterion_main!(benches);
