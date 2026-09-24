// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One-row native provider hook units; no compiler, Delta or publication journey.
use super::*;
use crate::session::{EngineFactory, execution::NativeExecutionContext};
use datafusion::{
    arrow::{
        array::Int64Array,
        datatypes::{DataType, Field, Schema},
    },
    common::TableReference,
    datasource::MemTable,
    execution::{runtime_env::RuntimeEnv, session_state::SessionStateBuilder},
    logical_expr::{LogicalPlan, col, lit},
};
use pse_columnar::CancellationToken;

#[test]
fn shared_effect_preparation_preserves_owners_and_defers_nested_commands() {
    use crate::session::contract::ExecutionContract;
    use datafusion::logical_expr::{DdlStatement, DropTable, Union};
    use pse_schema::model::provider::OperationEffect;

    let cancel = CancellationToken::new();
    let budget: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
    let factory = EngineFactory::from_builder(
        Arc::new(RuntimeEnv::default()),
        budget.clone(),
        "shared-effects",
        SessionStateBuilder::new_with_default_features(),
    );
    let session = factory
        .candidate_checked(
            std::collections::BTreeMap::new(),
            Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
            &cancel,
        )
        .unwrap();
    let empty = datafusion::logical_expr::LogicalPlanBuilder::empty(false)
        .build()
        .unwrap();
    for command in [false, true] {
        let mut plan = if command {
            LogicalPlan::Ddl(DdlStatement::DropTable(DropTable {
                name: TableReference::full("native", "private", "missing"),
                if_exists: false,
                schema: Arc::clone(empty.schema()),
            }))
        } else {
            empty.clone()
        };
        for _ in 0..40 {
            let input = Arc::new(ExecutionContract::plan(
                plan,
                None,
                [OperationEffect::Read].into_iter().collect(),
            ));
            plan = LogicalPlan::Union(Union {
                schema: Arc::clone(input.schema()),
                inputs: vec![Arc::clone(&input), input],
            });
        }
        let before = budget.reserved();
        let rewritten = isolate(&plan, &session, &cancel).unwrap();
        assert_eq!(budget.reserved(), before);
        if !command {
            let (LogicalPlan::Union(original), LogicalPlan::Union(result)) = (&plan, &rewritten)
            else {
                panic!("expected union roots")
            };
            assert!(Arc::ptr_eq(&original.inputs[0], &result.inputs[0]));
        }
        let mut node = &rewritten;
        while let Some(child) = node.inputs().first().copied() {
            node = child;
        }
        if command {
            let LogicalPlan::Extension(extension) = node else {
                panic!("namespace operation was not deferred")
            };
            assert_eq!(extension.node.name(), "NativeCommand");
        } else {
            assert!(matches!(node, LogicalPlan::EmptyRelation(_)));
        }
    }
    cancel.cancel();
    let before = budget.reserved();
    assert!(isolate(&empty, &session, &cancel).is_err());
    assert_eq!(budget.reserved(), before);
}

async fn values(table: Arc<dyn TableProvider>, state: &SessionState) -> Vec<i64> {
    let physical = table.scan(state, None, &[], None).await.unwrap();
    datafusion::physical_plan::execute_stream(physical, state.task_ctx())
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .unwrap()
        .iter()
        .flat_map(|batch| {
            batch
                .column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .values()
                .to_vec()
        })
        .collect()
}

#[expect(
    clippy::too_many_lines,
    reason = "ordered one-row mutation and ownership assertions"
)]
#[tokio::test]
async fn native_write_hook_consumes_real_children_and_replaces_only_private_generation() {
    let cancel = CancellationToken::new();
    let factory = EngineFactory::from_builder(
        Arc::new(RuntimeEnv::default()),
        Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20)),
        "dml-unit",
        SessionStateBuilder::new_with_default_features(),
    );
    let mut session = factory
        .candidate_checked(
            std::collections::BTreeMap::new(),
            Arc::new(pse_schema::builder::RegistryBuilder::new().build().unwrap()),
            &cancel,
        )
        .unwrap()
        .with_purpose(pse_schema::model::provider::OperationPurpose::Mutate);
    let schema = Arc::new(Schema::new(vec![Field::new(
        "value",
        DataType::Int64,
        false,
    )]));
    let source: Arc<dyn TableProvider> = Arc::new(
        MemTable::try_new(
            Arc::clone(&schema),
            vec![vec![
                RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![7]))]).unwrap(),
            ]],
        )
        .unwrap(),
    );
    let reference = TableReference::full("native", "private", "sample");
    let mut binding = TableBinding::new(reference.clone(), Arc::clone(&source), None, None);
    binding.mutation = Some(Arc::new(super::super::MemoryTableFactory));
    session
        .bindings
        .insert(
            crate::provider::binding::BindingKey::Native(reference.clone()),
            binding.clone(),
        )
        .unwrap();
    let state =
        NativeExecutionContext::bind(&session, session.bound_state().unwrap(), &cancel).unwrap();
    let target = IsolatedTarget {
        binding: Arc::new(binding),
        session: session.clone(),
    };
    let physical = target
        .update(
            &state,
            vec![("value".into(), col("value") + lit(2_i64))],
            vec![],
        )
        .await
        .unwrap();
    assert_eq!(physical.children().len(), 1);
    assert_eq!(values(Arc::clone(&source), &state).await, vec![7]);
    let result = datafusion::physical_plan::execute_stream(physical, state.task_ctx())
        .unwrap()
        .try_collect::<Vec<_>>()
        .await
        .unwrap();
    assert_eq!(
        result[0]
            .column(0)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap()
            .value(0),
        1
    );
    let changed = state
        .catalog_list()
        .catalog("native")
        .unwrap()
        .schema("private")
        .unwrap()
        .table("sample")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(values(changed, &state).await, vec![9]);
    assert_eq!(values(source, &state).await, vec![7]);
    assert_eq!(
        values(
            session
                .bindings
                .get(&crate::provider::binding::BindingKey::Native(reference))
                .unwrap()
                .provider
                .clone(),
            &state
        )
        .await,
        vec![7]
    );
    let resulting = session
        .capture_namespace(&state, None, &cancel)
        .await
        .unwrap();
    let current = resulting
        .bindings
        .get(&crate::provider::binding::BindingKey::Native(
            TableReference::full("native", "private", "sample"),
        ))
        .unwrap();
    assert!(current.mutation.is_some());
    assert_eq!(values(Arc::clone(&current.provider), &state).await, vec![9]);
}
