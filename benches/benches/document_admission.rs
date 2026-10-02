// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Package data document admission (ADR-0125, Plan 23 KR9): a 10⁵-row Parquet document is
//! loaded with its package and decoded by Arrow type, then admitted through its declared
//! relation. Each admission runs in a fresh compiler workspace, so every measured admission
//! is cold; no tracked result is reused. `just bench-smoke` runs each once.
#![allow(
    clippy::unwrap_used,
    reason = "a standalone measurement harness fails on invalid setup"
)]

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use datafusion::arrow::array::{ArrayRef, Float64Array, Int64Array, RecordBatch};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use pse_compiler::workspace::{CompilerContext, CompilerWorkspace, WorkspaceLimits};
use pse_modeling::PhysicalScope;
use pse_modeling::document::DocumentInventory;
use pse_relations::columnar::RelationRow;
use pse_runtime::authoring_driver::document::load_package_documents;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Rows of the admitted document.
const ROWS: i64 = 100_000;
const MANIFEST: &[u8] = include_bytes!("../../tests/fixtures/packages/minimal_named/package.toml");
const TEXT: &str = r#"package minimal_named {
 entity kind source provenance { attribute title: Text; }
 enum role { given }
 entity source s { title = "document admission bench" }
 table level[n: Integer]: Energy storage {kJ} missing optional;
 dataset levels: level provenance(s, role.given) from "data/levels.parquet";
}"#;

/// The package: its manifest, the declarations and a `ROWS`-row Parquet document.
fn sources() -> BTreeMap<String, Vec<u8>> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("n", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]));
    let columns: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from_iter_values(0..ROWS)),
        Arc::new(Float64Array::from_iter_values(
            (0..ROWS).map(|n| n as f64 * 0.25),
        )),
    ];
    let batch = RecordBatch::try_new(Arc::clone(&schema), columns).unwrap();
    let mut document = Vec::new();
    let mut writer = parquet::arrow::ArrowWriter::try_new(&mut document, schema, None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
    BTreeMap::from([
        ("package.toml".to_owned(), MANIFEST.to_vec()),
        ("models/levels.pse".to_owned(), TEXT.as_bytes().to_vec()),
        ("data/levels.parquet".to_owned(), document),
    ])
}

fn context() -> CompilerContext {
    CompilerContext {
        quantities: Arc::new(pse_quantity::standard::standard_registry().unwrap()),
        preconditions: Arc::new(pse_quantity::PhysicalPreconditions::new(vec![]).unwrap()),
        providers: BTreeMap::new(),
    }
}

fn admission(c: &mut Criterion) {
    let registry = pse_schema::shared_registry().unwrap();
    let native = pse_testkit::NativeFixture::new((256 << 20).try_into().unwrap()).unwrap();
    let validation = native.factory.validation_context(&registry).unwrap();
    let sources = sources();
    let load = || {
        load_package_documents(
            sources.clone(),
            &registry,
            pse_authoring::ParseBudget::default(),
            &validation,
        )
        .unwrap()
    };
    let bundle = load();
    let rows = pse_relations::generated::authored::modeling_declarations::Row::rows(
        &bundle.batches[&pse_relations::generated::authored::modeling_declarations::RELATION_ID],
    )
    .unwrap();
    let mut inventory = DocumentInventory::default();
    for document in &bundle.documents {
        for (path, span) in document.spans.iter() {
            if let Some(field) = path.strip_prefix("/modeling-fields/")
                && let Some((declaration, role)) = field.split_once('/')
            {
                inventory
                    .field_spans
                    .entry(pse_ids::SemanticId::parse_hex(declaration).unwrap().into())
                    .or_default()
                    .insert(role.into(), span);
            }
        }
        match document.data() {
            Some(data) => {
                inventory.documents.insert(document.id, Arc::clone(data));
            }
            None => {
                inventory
                    .packages
                    .insert(document.id, bundle.package.package_id.as_id());
            }
        }
    }
    let inventory = Arc::new(inventory);
    let mut peak_retained_bytes = 0;
    let mut group = c.benchmark_group("document_admission");
    group.sample_size(10);
    group.bench_function("load_and_decode/100000", |b| b.iter(load));
    group.bench_function("admit/100000", |b| {
        b.iter_batched(
            || CompilerWorkspace::new(context(), WorkspaceLimits::default()).unwrap(),
            |mut workspace| {
                let revision = workspace
                    .publish_modeling_with(
                        rows.clone(),
                        PhysicalScope::default(),
                        inventory.clone(),
                    )
                    .unwrap();
                let checked = revision.checked();
                let level = checked.table(checked.entry("minimal_named.level").unwrap());
                assert_eq!(level.unwrap().rows.len(), ROWS as usize);
                peak_retained_bytes = peak_retained_bytes.max(workspace.retention_usage().1);
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
    if let Some(output) = std::env::var_os("PSE_ADMISSION_OUTPUT") {
        let output = std::path::PathBuf::from(output);
        std::fs::create_dir_all(&output).unwrap();
        let rss = std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|text| {
                text.lines().find_map(|line| {
                    line.strip_prefix("VmHWM:").and_then(|value| {
                        value
                            .split_whitespace()
                            .next()?
                            .parse::<u64>()
                            .ok()?
                            .checked_mul(1024)
                    })
                })
            });
        std::fs::write(output.join("admission.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "rows":ROWS,"compiler_peak_retained_bytes":peak_retained_bytes,"process_peak_rss_bytes":rss,
            "scope":"cold package decode and declared table admission; input generation and workspace construction untimed",
            "memory_scope":"known compiler-owned Salsa retention and process-lifetime RSS; compiler retention is not an engine-pool observation",
        })).unwrap()).unwrap();
    }
}

fn configuration() -> Criterion {
    match std::env::var_os("PSE_ADMISSION_OUTPUT") {
        Some(output) => Criterion::default()
            .output_directory(&std::path::PathBuf::from(output).join("criterion")),
        None => Criterion::default(),
    }
}
criterion_group! {name=benches;config=configuration();targets=admission}
criterion_main!(benches);
