// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Caller-owned inputs still require a fresh scratch reservation before semantic decoding.
use datafusion::arrow::array::RecordBatch;
use datafusion::datasource::source_as_provider;
use datafusion::execution::runtime_env::{RuntimeEnv, RuntimeEnvBuilder};
use datafusion::logical_expr::LogicalPlanBuilder;
use pse_columnar::PlanOrigin;
use pse_diagnostics::{DiagnosticCode, TypedDiagnostic};
use pse_engine::EngineError;
use pse_engine::session::{
    ExecutionSettings, ThreadBudget, admission::admit_plan, native_engine_profile,
};

use pse_columnar::{CancellationToken, MemoryPool};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[expect(
    clippy::expect_used,
    reason = "test fixture helper requires valid declared setup"
)]
fn input() -> (Arc<Registry>, RecordBatch) {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "nested",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "nested external input",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key(
                "id",
                FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64),
                "identity",
            ),
            FieldContract::payload(
                "members",
                FieldContract::list(FieldContract::native(
                    datafusion::arrow::datatypes::DataType::Utf8,
                )),
                "large nested strings",
            ),
        ]),
    );
    let registry = Arc::new(builder.build().expect("registry"));
    let spec = registry.relation("authored.nested").expect("relation");
    let batch = pse_relations::testing::batch_from_literals(
        &registry,
        spec,
        &[vec![
            serde_json::json!(["u64", 0]),
            serde_json::json!([
                "list",
                vec![serde_json::json!(["text", "x".repeat(64 << 10)]); 8]
            ]),
        ]],
    )
    .expect("caller-owned input");
    (registry, batch)
}

#[expect(
    clippy::expect_used,
    reason = "test fixture helper requires valid declared setup"
)]
fn thread_budget() -> ThreadBudget {
    let one = NonZeroUsize::new(1).expect("one");
    ThreadBudget {
        pool_threads: one,
        target_partitions: one,
    }
}

#[expect(
    clippy::expect_used,
    reason = "test fixture helper requires valid declared setup"
)]
fn build(
    registry: Arc<Registry>,
    batch: RecordBatch,
    pool: Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<pse_engine::session::EngineSession, EngineError> {
    let key = registry.relation("authored.nested").expect("relation").key;
    let runtime: Arc<RuntimeEnv> = Arc::new(RuntimeEnvBuilder::new().build().expect("runtime"));
    pse_engine::EngineFactory::new(
        runtime,
        pool,
        ExecutionSettings::default(),
        thread_budget(),
        native_engine_profile(),
    )
    .and_then(|factory| factory.candidate(BTreeMap::from([(key, batch)]), registry, cancel))
}

#[expect(
    clippy::expect_used,
    reason = "test fixture helper requires valid declared setup"
)]
fn code(error: &EngineError) -> DiagnosticCode {
    error.diagnostic_code().expect("typed diagnostic")
}

#[test]
fn candidate_nested_decode_is_refused_before_allocation_with_a_tiny_budget() {
    let (registry, batch) = input();
    let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1024));
    let pool: Arc<dyn MemoryPool> = budget.clone();
    let Err(error) = build(registry, batch, pool, &CancellationToken::default()) else {
        panic!("large candidate cannot fit its validation scratch");
    };
    assert_eq!(code(&error), DiagnosticCode::RuntimeResourceLimit);
    assert!(error.to_string().contains("relations:raw-admission"));
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn repeated_plan_admission_reuses_immutable_input_without_value_scratch() {
    let (registry, batch) = input();
    let extent = pse_columnar::algorithm_decode_extent(&batch).expect("extent");
    let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(extent * 2));
    let pool: Arc<dyn MemoryPool> = budget.clone();
    let session = build(
        Arc::clone(&registry),
        batch,
        pool,
        &CancellationToken::default(),
    )
    .expect("candidate");
    let key = registry.relation("authored.nested").expect("relation").key;
    let source = session.table_source(&key).expect("actual source");
    let table = source_as_provider(&source).expect("actual provider");
    let plan = LogicalPlanBuilder::scan("nested", source, None)
        .expect("scan")
        .build()
        .expect("plan");
    let retained = budget.reserved();
    assert!(retained > 0);
    let cancel = CancellationToken::default();
    for _ in 0..16 {
        admit_plan(&plan, &registry, &[Arc::clone(&table)], &budget, &cancel).expect("admission");
        assert_eq!(budget.reserved(), retained);
    }
    let pressure = pse_columnar::MemoryConsumer::new("test:other-live-consumer").register(&budget);
    // Admission charges its native plan/control memo, but must not decode the
    // half-megabyte immutable value again. Zero headroom must refuse honestly.
    pressure
        .try_grow(extent * 2 - retained)
        .expect("other consumer");
    let errors = pse_columnar::classify(
        admit_plan(&plan, &registry, &[Arc::clone(&table)], &budget, &cancel)
            .expect_err("zero plan-control headroom"),
        PlanOrigin::RuleCompiler,
    );
    assert_eq!(
        code(&EngineError::from(errors)),
        DiagnosticCode::RuntimeResourceLimit
    );
    pressure.shrink(8 << 10);
    let baseline = budget.reserved();
    admit_plan(&plan, &registry, &[Arc::clone(&table)], &budget, &cancel)
        .expect("bound immutable providers need no repeated value scratch");
    assert_eq!(budget.reserved(), baseline);
    pressure.free();
    cancel.cancel();
    let errors = pse_columnar::classify(
        admit_plan(&plan, &registry, &[Arc::clone(&table)], &budget, &cancel)
            .expect_err("cancelled"),
        PlanOrigin::RuleCompiler,
    );
    assert_eq!(
        code(&EngineError::from(errors)),
        DiagnosticCode::RuntimeCancelled
    );
    assert_eq!(budget.reserved(), retained);
    drop(plan);
    drop(table);
    drop(session);
    assert_eq!(budget.reserved(), 0);
}

#[derive(Debug)]
struct CancelOnReserve {
    budget: Arc<dyn MemoryPool>,
    cancel: CancellationToken,
}
impl std::fmt::Display for CancelOnReserve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CancelOnReserve")
    }
}
impl MemoryPool for CancelOnReserve {
    fn name(&self) -> &'static str {
        "CancelOnReserve"
    }
    fn register(&self, c: &pse_columnar::MemoryConsumer) {
        self.budget.register(c);
    }
    fn unregister(&self, c: &pse_columnar::MemoryConsumer) {
        self.budget.unregister(c);
    }
    fn grow(&self, r: &pse_columnar::MemoryReservation, n: usize) {
        self.budget.grow(r, n);
    }
    fn shrink(&self, r: &pse_columnar::MemoryReservation, n: usize) {
        self.budget.shrink(r, n);
    }
    fn try_grow(
        &self,
        r: &pse_columnar::MemoryReservation,
        n: usize,
    ) -> datafusion::common::Result<()> {
        self.budget.try_grow(r, n)?;
        self.cancel.cancel();
        Ok(())
    }
    fn reserved(&self) -> usize {
        self.budget.reserved()
    }
    fn memory_limit(&self) -> datafusion::execution::memory_pool::MemoryLimit {
        self.budget.memory_limit()
    }
}

#[test]
fn cancellation_after_validation_reservation_releases_the_entire_scratch_claim() {
    let (registry, batch) = input();
    let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
    let cancel = CancellationToken::default();
    let pool: Arc<dyn MemoryPool> = Arc::new(CancelOnReserve {
        budget: Arc::clone(&budget),
        cancel: cancel.clone(),
    });
    let Err(error) = build(registry, batch, pool, &cancel) else {
        panic!("cancellation must stop candidate validation");
    };
    assert_eq!(code(&error), DiagnosticCode::RuntimeCancelled);
    assert_eq!(budget.reserved(), 0);
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "keep the complete independent nested fixture and its assertions together"
)]
fn many_invalid_nested_values_are_budgeted_before_diagnostics_and_release_scratch() {
    use datafusion::arrow::array::{Array, ArrayRef, Int64Array, ListArray, StructArray};
    use pse_schema::model::ExtensionUse;
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare(&mut builder);
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "nested",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "invalid source span fixture",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key(
                "id",
                FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64),
                "identity",
            ),
            FieldContract::payload(
                "members",
                FieldContract::list(FieldContract::extended(ExtensionUse::SourceSpan)),
                "nested source spans",
            ),
        ]),
    );
    let registry = Arc::new(builder.build().expect("registry"));
    let spec = registry.relation("authored.nested").expect("relation");
    let row = serde_json::json!([
        "struct",
        vec![
            serde_json::json!(["id", (pse_ids::SemanticId::NIL).to_hex()]),
            serde_json::json!(["i64", 0]),
            serde_json::json!(["i64", 1]),
        ]
    ]);
    let valid = pse_relations::testing::batch_from_literals(
        &registry,
        spec,
        &[vec![
            serde_json::json!(["u64", 0]),
            serde_json::json!(["list", vec![row; 1024]]),
        ]],
    )
    .expect("valid spans");
    let list = valid
        .column(1)
        .as_any()
        .downcast_ref::<ListArray>()
        .expect("list");
    let spans = list
        .values()
        .as_any()
        .downcast_ref::<StructArray>()
        .expect("spans");
    let invalid_spans: ArrayRef = Arc::new(StructArray::new(
        spans.fields().clone(),
        vec![
            Arc::clone(spans.column(0)),
            Arc::new(Int64Array::from(vec![2; 1024])),
            Arc::clone(spans.column(2)),
        ],
        spans.nulls().cloned(),
    ));
    let datafusion::arrow::datatypes::DataType::List(field) = list.data_type() else {
        panic!("list field");
    };
    let invalid: ArrayRef = Arc::new(ListArray::new(
        Arc::clone(field),
        list.offsets().clone(),
        invalid_spans,
        list.nulls().cloned(),
    ));
    let invalid = RecordBatch::try_new(valid.schema(), vec![Arc::clone(valid.column(0)), invalid])
        .expect("valid physical storage with invalid semantic values");
    let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
    let pool: Arc<dyn MemoryPool> = budget.clone();
    let Err(error) = build(
        Arc::clone(&registry),
        invalid.clone(),
        pool,
        &CancellationToken::default(),
    ) else {
        panic!("invalid spans must refuse");
    };
    assert_eq!(code(&error), DiagnosticCode::ValidationInvariant);
    let EngineError::Relation(relation) = &error else {
        panic!("expected structured relation diagnostics: {error:?}");
    };
    let pse_relations::RelationError::LocalFindings { report } = relation.as_ref() else {
        panic!("expected one native validation report: {relation:?}");
    };
    assert_eq!(report.violations, 1024);
    assert!(report.truncated);
    assert_eq!(report.findings.num_rows(), 256);
    let paths = report
        .findings
        .column_by_name("path")
        .unwrap()
        .as_any()
        .downcast_ref::<datafusion::arrow::array::StringArray>()
        .unwrap();
    let rows = report
        .findings
        .column_by_name("row")
        .unwrap()
        .as_any()
        .downcast_ref::<datafusion::arrow::array::UInt64Array>()
        .unwrap();
    for index in 0..256 {
        assert_eq!(paths.value(index), format!("/members/{index}"));
        assert_eq!(rows.value(index), 0);
    }
    assert_eq!(budget.reserved(), 0);
    let tiny: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1024));
    let pool: Arc<dyn MemoryPool> = tiny.clone();
    let Err(error) = build(registry, invalid, pool, &CancellationToken::default()) else {
        panic!("scratch must be reserved before semantic diagnostics");
    };
    assert_eq!(code(&error), DiagnosticCode::RuntimeResourceLimit);
    assert_eq!(tiny.reserved(), 0);
}
