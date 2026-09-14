// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Caller-owned inputs still require a fresh scratch reservation before semantic decoding.
use datafusion::arrow::array::RecordBatch;
use datafusion::datasource::source_as_provider;
use datafusion::execution::runtime_env::{RuntimeEnv, RuntimeEnvBuilder};
use datafusion::logical_expr::LogicalPlanBuilder;
use miette::Diagnostic;
use pse_catalog::session::{
    ExecutionSettings, ThreadBudget, admission::admit_plan, build_candidate_session_with_cancel,
    phase0_reference_profile,
};
use pse_catalog::{CatalogError, PlanOrigin};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, Reservation, ReserveError};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{Authority, Cell, ColumnSpec, LogicalType, Namespace, RelationDecl, SnapshotClass},
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
            ColumnSpec::key("id", LogicalType::U64, "identity"),
            ColumnSpec::payload(
                "members",
                LogicalType::list(LogicalType::Text),
                "large nested strings",
            ),
        ]),
    );
    let registry = Arc::new(builder.build().expect("registry"));
    let spec = registry.relation("authored.nested").expect("relation");
    let batch = pse_relations::cells::batch_from_cells(
        &registry,
        spec,
        &[vec![
            Cell::U64(0),
            Cell::List(vec![Cell::text("x".repeat(64 << 10)); 8]),
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
    reserver: Arc<dyn MemoryReserver>,
    cancel: &CancellationToken,
) -> Result<pse_catalog::session::SnapshotSession, CatalogError> {
    let key = registry.relation("authored.nested").expect("relation").key;
    let runtime: Arc<RuntimeEnv> = Arc::new(RuntimeEnvBuilder::new().build().expect("runtime"));
    build_candidate_session_with_cancel(
        BTreeMap::from([(key, batch)]),
        registry,
        runtime,
        reserver,
        ExecutionSettings::default(),
        thread_budget(),
        phase0_reference_profile(),
        cancel,
    )
}

#[expect(
    clippy::expect_used,
    reason = "test fixture helper requires valid declared setup"
)]
fn code(error: &CatalogError) -> String {
    error.code().expect("typed diagnostic").to_string()
}

#[test]
fn candidate_nested_decode_is_refused_before_allocation_with_a_tiny_budget() {
    let (registry, batch) = input();
    let budget = FixedBudget::new(1024);
    let reserver: Arc<dyn MemoryReserver> = budget.clone();
    let Err(error) = build(registry, batch, reserver, &CancellationToken::default()) else {
        panic!("large candidate cannot fit its validation scratch");
    };
    assert_eq!(code(&error), "runtime::resource_limit");
    assert!(error.to_string().contains("session:validate-batch"));
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn repeated_plan_admission_releases_scratch_and_cannot_borrow_retained_input_budget() {
    let (registry, batch) = input();
    let extent = pse_catalog::store::membership::validation_extent(&batch).expect("extent");
    let budget = FixedBudget::new(extent * 2);
    let reserver: Arc<dyn MemoryReserver> = budget.clone();
    let session = build(
        Arc::clone(&registry),
        batch,
        reserver,
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
        admit_plan(
            &plan,
            &registry,
            &[Arc::clone(&table)],
            budget.as_ref(),
            &cancel,
        )
        .expect("admission");
        assert_eq!(budget.reserved(), retained);
    }
    let mut pressure = budget.open("test:other-live-consumer");
    pressure
        .try_grow(budget.limit_bytes() - retained - extent + 1)
        .expect("other consumer");
    let baseline = budget.reserved();
    let error = admit_plan(
        &plan,
        &registry,
        &[Arc::clone(&table)],
        budget.as_ref(),
        &cancel,
    )
    .expect_err("no scratch headroom");
    let errors = pse_catalog::failure::classify(error, PlanOrigin::RuleCompiler);
    assert_eq!(errors.len(), 1);
    assert_eq!(code(&errors[0]), "runtime::resource_limit");
    assert_eq!(budget.reserved(), baseline);
    pressure.release();
    cancel.cancel();
    let errors = pse_catalog::failure::classify(
        admit_plan(
            &plan,
            &registry,
            &[Arc::clone(&table)],
            budget.as_ref(),
            &cancel,
        )
        .expect_err("cancelled"),
        PlanOrigin::RuleCompiler,
    );
    assert_eq!(code(&errors[0]), "runtime::cancelled");
    assert_eq!(budget.reserved(), retained);
    drop(plan);
    drop(table);
    drop(session);
    assert_eq!(budget.reserved(), 0);
}

#[derive(Debug)]
struct CancelOnReserve {
    budget: Arc<FixedBudget>,
    cancel: CancellationToken,
}
#[derive(Debug)]
struct CancellingReservation {
    inner: Box<dyn Reservation>,
    cancel: CancellationToken,
}
impl MemoryReserver for CancelOnReserve {
    fn open(&self, owner: &str) -> Box<dyn Reservation> {
        Box::new(CancellingReservation {
            inner: self.budget.open(owner),
            cancel: self.cancel.clone(),
        })
    }
}
impl Reservation for CancellingReservation {
    fn try_grow(&mut self, bytes: usize) -> Result<(), ReserveError> {
        self.inner.try_grow(bytes)?;
        self.cancel.cancel();
        Ok(())
    }
    fn shrink(&mut self, bytes: usize) {
        self.inner.shrink(bytes);
    }
    fn size(&self) -> usize {
        self.inner.size()
    }
    fn release(&mut self) {
        self.inner.release();
    }
}

#[test]
fn cancellation_after_validation_reservation_releases_the_entire_scratch_claim() {
    let (registry, batch) = input();
    let budget = FixedBudget::new(64 << 20);
    let cancel = CancellationToken::default();
    let reserver: Arc<dyn MemoryReserver> = Arc::new(CancelOnReserve {
        budget: Arc::clone(&budget),
        cancel: cancel.clone(),
    });
    let Err(error) = build(registry, batch, reserver, &cancel) else {
        panic!("cancellation must stop candidate validation");
    };
    assert_eq!(code(&error), "runtime::cancelled");
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn many_invalid_nested_values_are_budgeted_before_diagnostics_and_release_scratch() {
    use datafusion::arrow::array::{Array, ArrayRef, ListArray, StructArray, UInt32Array};
    use pse_schema::model::ExtensionUse;
    let mut builder = RegistryBuilder::new();
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
            ColumnSpec::key("id", LogicalType::U64, "identity"),
            ColumnSpec::payload(
                "members",
                LogicalType::list(LogicalType::Ext(ExtensionUse::SourceSpan)),
                "nested source spans",
            ),
        ]),
    );
    let registry = Arc::new(builder.build().expect("registry"));
    let spec = registry.relation("authored.nested").expect("relation");
    let row = Cell::Struct(vec![
        Cell::Id(pse_ids::SemanticId::NIL),
        Cell::U64(0),
        Cell::U64(1),
    ]);
    let valid = pse_relations::cells::batch_from_cells(
        &registry,
        spec,
        &[vec![Cell::U64(0), Cell::List(vec![row; 1024])]],
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
            Arc::new(UInt32Array::from(vec![2; 1024])),
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
    let budget = FixedBudget::new(64 << 20);
    let reserver: Arc<dyn MemoryReserver> = budget.clone();
    let Err(error) = build(
        Arc::clone(&registry),
        invalid.clone(),
        reserver,
        &CancellationToken::default(),
    ) else {
        panic!("invalid spans must refuse");
    };
    assert_eq!(code(&error), "schema::admission");
    assert_eq!(
        error
            .to_string()
            .matches("source span start exceeds end")
            .count(),
        1024
    );
    assert_eq!(budget.reserved(), 0);
    let tiny = FixedBudget::new(1024);
    let reserver: Arc<dyn MemoryReserver> = tiny.clone();
    let Err(error) = build(registry, invalid, reserver, &CancellationToken::default()) else {
        panic!("scratch must be reserved before semantic diagnostics");
    };
    assert_eq!(code(&error), "runtime::resource_limit");
    assert_eq!(tiny.reserved(), 0);
}
