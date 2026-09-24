// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{admit_plan, intermediate_schema, restore_semantic_fields};
use crate::session::materialized::ImmutableTable;
use datafusion::arrow::array::RecordBatch;
use datafusion::catalog::TableProvider;
use datafusion::common::Column;
use datafusion::datasource::provider_as_source;
use datafusion::logical_expr::{LogicalPlan, LogicalPlanBuilder, col};
use pse_columnar::CancellationToken;
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, EnumDecl, EnumMember, FieldContract, Namespace, RelationDecl, SnapshotClass,
    },
};
use std::sync::Arc;

#[test]
fn expression_fields_reuse_only_exact_schema_and_function_owners() {
    use datafusion::arrow::datatypes::{DataType, Field, Schema};
    use datafusion::common::{DFSchema, ScalarValue};
    use datafusion::logical_expr::{ColumnarValue, Volatility, create_udf};
    let registry = RegistryBuilder::new().build().unwrap();
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
    {
        let mut fields = super::FieldAdmissions::new(&registry, &pool);
        let schema =
            DFSchema::try_from(Schema::new(vec![Field::new("n", DataType::Int64, false)])).unwrap();
        let expression = col("n").alias("value");
        let first = fields.expression(&expression, &schema).unwrap();
        let reserved = pool.reserved();
        assert!(Arc::ptr_eq(
            &first,
            &fields.expression(&expression, &schema).unwrap()
        ));
        assert_eq!(pool.reserved(), reserved);
        let other =
            DFSchema::try_from(Schema::new(vec![Field::new("n", DataType::Utf8, true)])).unwrap();
        assert_eq!(
            fields.expression(&expression, &other).unwrap().data_type(),
            &DataType::Utf8
        );
        let function = || {
            create_udf(
                "same_name",
                vec![],
                DataType::Int64,
                Volatility::Immutable,
                Arc::new(|_| Ok(ColumnarValue::Scalar(ScalarValue::Int64(Some(1))))),
            )
        };
        let a = function().call(vec![]);
        let b = function().call(vec![]);
        let first = fields.expression(&a, &schema).unwrap();
        let next = fields.expression(&b, &schema).unwrap();
        assert!(
            !Arc::ptr_eq(&first, &next),
            "SQL names do not identify implementation owners"
        );
    }
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn field_admission_reuse_checks_full_fields_and_releases_or_bypasses_budget() {
    let (registry, provider, _) = fixture();
    let field = Arc::clone(&provider.schema().fields()[0]);
    let mut forged = field.as_ref().clone();
    forged.metadata_mut().insert(
        pse_schema::arrow::KEY_EXTENSION_NAME.into(),
        "pse.unregistered".into(),
    );
    for limit in [0, 1 << 20] {
        let budget: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(limit));
        {
            let mut checks = super::FieldAdmissions::new(&registry, &budget);
            checks.intermediate(&field).unwrap();
            let reserved = budget.reserved();
            checks.intermediate(&field).unwrap();
            checks
                .intermediate(&Arc::new(field.as_ref().clone()))
                .unwrap();
            assert_eq!(budget.reserved(), reserved);
            assert!(checks.intermediate(&Arc::new(forged.clone())).is_err());
            assert_eq!(checks.accepted.len(), usize::from(limit > 0));
        }
        assert_eq!(budget.reserved(), 0);
    }
}

#[tokio::test]
async fn recursive_work_tables_use_nullable_anchor_fields_and_remain_scoped() {
    let context = datafusion::prelude::SessionContext::new();
    let plan = context.sql("WITH RECURSIVE walk(n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM walk WHERE n < 3) SELECT n FROM walk")
        .await.unwrap().into_unoptimized_plan();
    let registry = RegistryBuilder::new().build().unwrap();
    let cancel = CancellationToken::new();
    let budget: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
    let mut admitted = restore_semantic_fields(plan, &registry, &[], &budget, &cancel).unwrap();
    let description = admitted.display_indent().to_string();
    for _ in 0..4 {
        admitted = restore_semantic_fields(admitted, &registry, &[], &budget, &cancel).unwrap();
        assert_eq!(
            admitted.display_indent().to_string(),
            description,
            "recursive field transport grew during repeated admission"
        );
    }
    let factory = crate::session::EngineFactory::from_builder(
        context.runtime_env(),
        budget.clone(),
        "recursive-dependency-unit",
        datafusion::execution::session_state::SessionStateBuilder::from(context.state()),
    );
    let session = factory
        .candidate_checked(
            std::collections::BTreeMap::new(),
            Arc::new(registry),
            &cancel,
        )
        .unwrap();
    assert!(
        session
            .source_dependencies(&admitted, &cancel)
            .unwrap()
            .is_empty()
    );
    admitted
        .apply_with_subqueries(|node| {
            if matches!(node, LogicalPlan::TableScan(_)) {
                assert!(
                    session.source_dependencies(node, &cancel).is_err(),
                    "an extracted work table has no owning query scope"
                );
            }
            Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
        })
        .unwrap();
    assert_eq!(
        context
            .execute_logical_plan(admitted)
            .await
            .unwrap()
            .collect()
            .await
            .unwrap()
            .iter()
            .map(RecordBatch::num_rows)
            .sum::<usize>(),
        3
    );
}

#[test]
fn native_worktable_admits_anchor_or_wider_nullability_but_refuses_changed_meaning() {
    use datafusion::arrow::datatypes::{DataType, Field, Schema};
    let registry = RegistryBuilder::new().build().unwrap();
    let make_scan = |field| {
        let provider = Arc::new(datafusion_catalog::cte_worktable::CteWorkTable::new(
            "walk",
            Arc::new(Schema::new(vec![field])),
        ));
        let LogicalPlan::TableScan(scan) =
            LogicalPlanBuilder::scan("walk", provider_as_source(provider), None)
                .unwrap()
                .build()
                .unwrap()
        else {
            panic!("scan expected")
        };
        scan
    };
    let field = Field::new("n", DataType::Int64, false);
    let scope = vec![("walk".into(), Arc::new(Schema::new(vec![field.clone()])))];
    assert!(super::admit_scan(&make_scan(field.clone()), &registry, &[], &scope).is_ok());
    assert!(
        super::admit_scan(
            &make_scan(field.clone().with_nullable(true)),
            &registry,
            &[],
            &scope
        )
        .is_ok()
    );
    assert!(
        super::admit_scan(
            &make_scan(field.clone().with_name("forged")),
            &registry,
            &[],
            &scope
        )
        .is_err()
    );
    assert!(super::admit_scan(&make_scan(field.clone()), &registry, &[], &[]).is_err());
    let nullable_scope = vec![(
        "walk".into(),
        Arc::new(Schema::new(vec![field.clone().with_nullable(true)])),
    )];
    assert!(super::admit_scan(&make_scan(field), &registry, &[], &nullable_scope).is_err());
}

#[tokio::test]
async fn recursive_join_terms_transport_their_declared_schema_metadata() {
    use datafusion::arrow::{
        array::Int64Array,
        datatypes::{DataType, Field, Schema},
    };
    use datafusion::physical_planner::PhysicalPlanner;
    let schema = Arc::new(Schema::new_with_metadata(
        vec![
            Field::new("n", DataType::Int64, false),
            Field::new("parent", DataType::Int64, true),
        ],
        [("query.scope".into(), "edges".into())]
            .into_iter()
            .collect(),
    ));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![1, 2, 3])),
            Arc::new(Int64Array::from(vec![None, Some(1), Some(2)])),
        ],
    )
    .unwrap();
    let provider: Arc<dyn TableProvider> =
        Arc::new(datafusion::datasource::MemTable::try_new(schema, vec![vec![batch]]).unwrap());
    let context = datafusion::prelude::SessionContext::new();
    context.register_table("edges", provider.clone()).unwrap();
    let seed_schema = Arc::new(Schema::new_with_metadata(
        vec![Field::new("n", DataType::Int64, false)],
        [("query.scope".into(), "seeds".into())]
            .into_iter()
            .collect(),
    ));
    let seeds: Arc<dyn TableProvider> = Arc::new(
        datafusion::datasource::MemTable::try_new(
            seed_schema.clone(),
            vec![vec![
                RecordBatch::try_new(seed_schema, vec![Arc::new(Int64Array::from(vec![1]))])
                    .unwrap(),
            ]],
        )
        .unwrap(),
    );
    context.register_table("seeds", seeds.clone()).unwrap();
    let plan = context.sql("WITH RECURSIVE walk(n) AS (SELECT n FROM seeds UNION ALL SELECT e.n FROM walk w JOIN edges e ON e.parent = w.n) SELECT n FROM walk")
        .await.unwrap().into_optimized_plan().unwrap();
    let registry = RegistryBuilder::new().build().unwrap();
    let cancel = CancellationToken::new();
    let budget: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
    let admitted =
        restore_semantic_fields(plan, &registry, &[provider, seeds], &budget, &cancel).unwrap();
    // Planning directly keeps the final admission boundary after optimization.
    let state = context.state();
    let physical = datafusion::physical_planner::DefaultPhysicalPlanner::default()
        .create_physical_plan(&admitted, &state)
        .await
        .unwrap();
    let batches = datafusion::physical_plan::collect(physical, state.task_ctx())
        .await
        .unwrap();
    assert_eq!(batches.iter().map(RecordBatch::num_rows).sum::<usize>(), 3);
    assert_eq!(
        batches[0]
            .schema()
            .metadata()
            .get("query.scope")
            .map(String::as_str),
        Some("seeds")
    );
}

#[test]
fn delta_view_adaptation_requires_its_exact_durable_descriptor() {
    use datafusion::arrow::datatypes::{DataType, Schema};
    let registry = crate::validation::registry().unwrap();
    let execution = pse_schema::arrow::field_for(
        registry,
        &FieldContract::payload("path", FieldContract::native(DataType::Utf8), "source path"),
    )
    .unwrap();
    let storage = pse_schema::delta::storage_schema(&Schema::new(vec![execution.clone()])).unwrap();
    let view = storage.field(0).clone().with_data_type(DataType::Utf8View);
    super::admit_field(registry, &view).unwrap();
    // Ordinary execution admission remains exact; only the checked durable
    // descriptor authorizes the view representation at the native scan leaf.
    assert!(super::admit_field(registry, &execution.with_data_type(DataType::Utf8View)).is_err());
    assert!(super::admit_field(registry, &view.clone().with_name("renamed")).is_err());
    super::admit_intermediate_field(
        registry,
        &view.clone().with_name("renamed").with_nullable(true),
    )
    .unwrap();
    let mut forged = view;
    forged
        .metadata_mut()
        .insert(pse_schema::delta::KEY_LAYOUT_VERSION.into(), "99".into());
    assert!(super::admit_field(registry, &forged).is_err());
}

fn fixture() -> (Registry, Arc<dyn TableProvider>, LogicalPlan) {
    let mut builder = RegistryBuilder::new();
    for name in ["Phase", "OtherPhase"] {
        builder.declare_enum(EnumDecl::platform(
            name,
            vec![EnumMember::new("liquid", "member")],
        ));
    }
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "labels",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "fixture",
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
                FieldContract::list(FieldContract::enumeration("Phase")),
                "declared list",
            ),
        ]),
    );
    let registry = builder.build().unwrap();
    let spec = registry.relation("authored.labels").unwrap();
    let batch = pse_relations::testing::batch_from_literals(
        &registry,
        spec,
        &[vec![
            serde_json::json!(["u64", 0]),
            serde_json::json!(["list", vec![serde_json::json!(["enum", "liquid"])]]),
        ]],
    )
    .unwrap();
    let table: Arc<dyn TableProvider> = Arc::new(ImmutableTable::candidate(
        pse_relations::columnar::FieldCheckedBatch::admit(&registry, spec, batch).unwrap(),
    ));
    let plan = LogicalPlanBuilder::scan("labels", provider_as_source(Arc::clone(&table)), None)
        .unwrap()
        .unnest_column(Column::from_name("members"))
        .unwrap()
        .project(vec![col("members").alias("phase")])
        .unwrap()
        .build()
        .unwrap();
    (registry, table, plan)
}
#[test]
fn enum_projection_recovers_only_the_exact_actual_list_child_contract() {
    let (registry, table, plan) = fixture();
    assert!(plan.schema().field(0).metadata().is_empty());
    admit_plan(
        &plan,
        &registry,
        &[Arc::clone(&table)],
        &{
            let pool: Arc<dyn pse_columnar::MemoryPool> =
                Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
            pool
        },
        &CancellationToken::default(),
    )
    .unwrap();
    let recovered = intermediate_schema(&plan, &registry).unwrap();
    let expected = pse_schema::arrow::field_for(
        &registry,
        &FieldContract::payload("phase", FieldContract::enumeration("Phase"), "expected"),
    )
    .unwrap();
    assert_eq!(recovered.field(0).metadata(), expected.metadata());
    let restored = restore_semantic_fields(
        plan.clone(),
        &registry,
        &[Arc::clone(&table)],
        &{
            let pool: Arc<dyn pse_columnar::MemoryPool> =
                Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
            pool
        },
        &CancellationToken::default(),
    )
    .unwrap();
    assert_eq!(restored.schema().field(0).metadata(), expected.metadata());
    assert_eq!(
        restored.schema(),
        restore_semantic_fields(
            restored.clone(),
            &registry,
            &[Arc::clone(&table)],
            &{
                let pool: Arc<dyn pse_columnar::MemoryPool> =
                    Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
                pool
            },
            &CancellationToken::default()
        )
        .unwrap()
        .schema(),
        "restoration is idempotent"
    );
    let mut forged = plan;
    let LogicalPlan::Projection(project) = &mut forged else {
        return;
    };
    let different = pse_schema::arrow::field_for(
        &registry,
        &FieldContract::payload("phase", FieldContract::enumeration("OtherPhase"), "forged"),
    )
    .unwrap();
    project.schema = Arc::new(
        datafusion::common::DFSchema::new_with_metadata(
            vec![(None, Arc::new(different))],
            std::collections::HashMap::new(),
        )
        .unwrap(),
    );
    assert!(
        intermediate_schema(&forged, &registry).is_err(),
        "matching storage cannot replace the actual enum declaration"
    );
    assert!(
        restore_semantic_fields(
            forged,
            &registry,
            &[table],
            &{
                let pool: Arc<dyn pse_columnar::MemoryPool> =
                    Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
                pool
            },
            &CancellationToken::default()
        )
        .is_err()
    );
}
#[test]
fn unnest_refuses_forged_depth_and_dependency_mapping() {
    let (registry, _, plan) = fixture();
    let LogicalPlan::Projection(project) = plan else {
        return;
    };
    let mut unnest = project.input.as_ref().clone();
    let LogicalPlan::Unnest(value) = &mut unnest else {
        return;
    };
    value.list_type_columns[0].1.depth = 2;
    assert!(intermediate_schema(&unnest, &registry).is_err());
    let mut unnest = project.input.as_ref().clone();
    let LogicalPlan::Unnest(value) = &mut unnest else {
        return;
    };
    value.dependency_indices[1] = usize::MAX;
    assert!(intermediate_schema(&unnest, &registry).is_err());
}

#[test]
fn native_decimal_and_cast_plans_remain_eligible() {
    use datafusion::arrow::datatypes::DataType;
    use datafusion::logical_expr::ExprSchemable;
    let (registry, table, _) = fixture();
    let input = LogicalPlanBuilder::scan("labels", provider_as_source(Arc::clone(&table)), None)
        .unwrap()
        .build()
        .unwrap();
    let wide = col("id")
        .cast_to(&DataType::Decimal256(76, 0), input.schema())
        .unwrap();
    let sum = LogicalPlanBuilder::from(input)
        .aggregate(
            Vec::<datafusion::logical_expr::Expr>::new(),
            vec![datafusion::functions_aggregate::expr_fn::sum(wide).alias("sum")],
        )
        .unwrap()
        .build()
        .unwrap();
    let budget: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
    let cancel = CancellationToken::default();
    admit_plan(&sum, &registry, &[Arc::clone(&table)], &budget, &cancel)
        .expect("native decimal plan without a relation claim");
    let integer = col("sum").cast_to(&DataType::UInt64, sum.schema()).unwrap();
    let checked = LogicalPlanBuilder::from(sum)
        .project(vec![integer.alias("total")])
        .unwrap()
        .build()
        .unwrap();
    admit_plan(&checked, &registry, &[Arc::clone(&table)], &budget, &cancel).unwrap();
    let floating = datafusion::logical_expr::lit(1.5f64)
        .cast_to(&DataType::Decimal256(76, 0), checked.schema())
        .unwrap();
    let narrowed = floating
        .cast_to(&DataType::UInt64, checked.schema())
        .unwrap();
    let invalid = LogicalPlanBuilder::from(checked)
        .project(vec![narrowed.alias("invalid")])
        .unwrap()
        .build()
        .unwrap();
    assert!(
        admit_plan(&invalid, &registry, &[table], &budget, &cancel).is_ok(),
        "an explicit native cast is eligible; exact domain conversion is a separate contract"
    );
}

#[tokio::test]
async fn executed_unnest_exports_the_exact_source_enum_contract() {
    use crate::session::{ExecutionSettings, ThreadBudget, native_engine_profile};
    use datafusion::execution::{memory_pool::GreedyMemoryPool, runtime_env::RuntimeEnvBuilder};
    use pse_columnar::CancellationToken;
    use std::{collections::BTreeMap, num::NonZeroUsize};
    let (registry, table, _) = fixture();
    let candidate = table
        .as_ref()
        .downcast_ref::<ImmutableTable<pse_relations::columnar::FieldCheckedBatch>>()
        .unwrap();
    let key = registry
        .relation_by_id(candidate.data.relation_id())
        .unwrap()
        .key;
    let inputs = BTreeMap::from([(key, candidate.data.batch().clone())]);
    let registry = Arc::new(registry);
    let runtime = Arc::new(
        RuntimeEnvBuilder::new()
            .with_memory_pool(Arc::new(GreedyMemoryPool::new(8 << 20)))
            .build()
            .unwrap(),
    );
    let thread = NonZeroUsize::new(1).unwrap();
    let session = crate::EngineFactory::new(
        runtime,
        Arc::new(GreedyMemoryPool::new(8 << 20)),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: thread,
            target_partitions: thread,
        },
        native_engine_profile(),
    )
    .and_then(|factory| factory.candidate(inputs, registry, &CancellationToken::new()))
    .unwrap();
    let plan = LogicalPlanBuilder::scan("labels", session.table_source(&key).unwrap(), None)
        .unwrap()
        .unnest_column("members")
        .unwrap()
        .project(vec![col("members").alias("phase")])
        .unwrap()
        .build()
        .unwrap();
    let result = session
        .execute_plan(plan, &CancellationToken::default())
        .await
        .unwrap();
    assert_eq!(result.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
    let expected = pse_schema::arrow::field_for(
        session.registry(),
        &FieldContract::payload("phase", FieldContract::enumeration("Phase"), "expected"),
    )
    .unwrap();
    for batch in result {
        assert_eq!(batch.schema().field(0).metadata(), expected.metadata());
        pse_relations::validate::validate_column(
            session.registry(),
            &expected,
            batch.column(0).as_ref(),
        )
        .unwrap();
    }
}

#[test]
fn unresolved_higher_order_variables_are_bound_before_field_admission() {
    use datafusion::arrow::datatypes::{DataType, Field, Schema};
    use datafusion::logical_expr::expr_fn::{lambda, lambda_var};
    let registry = RegistryBuilder::new().build().unwrap();
    let schema = datafusion::common::DFSchema::try_from(Schema::new(vec![Field::new(
        "values",
        DataType::List(Arc::new(Field::new("item", DataType::Int64, false))),
        false,
    )]))
    .unwrap();
    let expression = datafusion::functions_nested::expr_fn::array_any_match(
        col("values"),
        lambda(
            ["item"],
            lambda_var("item").gt(datafusion::logical_expr::lit(0_i64)),
        ),
    );
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
    let mut fields = super::FieldAdmissions::new(&registry, &pool);
    super::admit_expression(&expression, &schema, &registry, &mut fields).unwrap();
    assert!(
        super::admit_expression(&lambda_var("unbound"), &schema, &registry, &mut fields).is_err()
    );
}
