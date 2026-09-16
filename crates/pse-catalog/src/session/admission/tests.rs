// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{admit_plan, intermediate_schema, restore_semantic_fields};
use crate::session::candidate::CandidateTable;
use datafusion::arrow::array::RecordBatch;
use datafusion::catalog::TableProvider;
use datafusion::common::Column;
use datafusion::datasource::provider_as_source;
use datafusion::logical_expr::{LogicalPlan, LogicalPlanBuilder, col};
use pse_ids::{CancellationToken, FixedBudget};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, EnumDecl, EnumMember, FieldContract, Namespace, RelationDecl,
        SnapshotClass,
    },
};
use std::sync::Arc;

#[test]
fn delta_view_adaptation_requires_its_exact_durable_descriptor() {
    use datafusion::arrow::datatypes::{DataType, Schema};
    let registry = pse_schema::registry().unwrap();
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
    let batch = pse_relations::cells::batch_from_cells(
        &registry,
        spec,
        &[vec![Cell::U64(0), Cell::List(vec![Cell::Enum("liquid")])]],
    )
    .unwrap();
    let table: Arc<dyn TableProvider> = Arc::new(CandidateTable {
        input: pse_relations::columnar::FieldCheckedBatch::admit(&registry, spec, batch).unwrap(),
    });
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
        FixedBudget::new(16 << 20).as_ref(),
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
        FixedBudget::new(16 << 20).as_ref(),
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
            FixedBudget::new(16 << 20).as_ref(),
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
            FixedBudget::new(16 << 20).as_ref(),
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
    let budget = FixedBudget::new(16 << 20);
    let cancel = CancellationToken::default();
    admit_plan(
        &sum,
        &registry,
        &[Arc::clone(&table)],
        budget.as_ref(),
        &cancel,
    )
    .expect("native decimal plan without a relation claim");
    let integer = col("sum").cast_to(&DataType::UInt64, sum.schema()).unwrap();
    let checked = LogicalPlanBuilder::from(sum)
        .project(vec![integer.alias("total")])
        .unwrap()
        .build()
        .unwrap();
    admit_plan(
        &checked,
        &registry,
        &[Arc::clone(&table)],
        budget.as_ref(),
        &cancel,
    )
    .unwrap();
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
        admit_plan(&invalid, &registry, &[table], budget.as_ref(), &cancel).is_ok(),
        "an explicit native cast is eligible; exact domain conversion is a separate contract"
    );
}

#[tokio::test]
async fn executed_unnest_exports_the_exact_source_enum_contract() {
    use crate::session::{
        ExecutionSettings, ThreadBudget, build_candidate_session, native_engine_profile,
    };
    use datafusion::execution::{memory_pool::GreedyMemoryPool, runtime_env::RuntimeEnvBuilder};
    use pse_ids::{CancellationToken, FixedBudget};
    use std::{collections::BTreeMap, num::NonZeroUsize};
    let (registry, table, _) = fixture();
    let candidate = table.as_ref().downcast_ref::<CandidateTable>().unwrap();
    let key = registry
        .relation_by_id(candidate.input.relation_id())
        .unwrap()
        .key;
    let inputs = BTreeMap::from([(key, candidate.input.batch().clone())]);
    let registry = Arc::new(registry);
    let runtime = Arc::new(
        RuntimeEnvBuilder::new()
            .with_memory_pool(Arc::new(GreedyMemoryPool::new(8 << 20)))
            .build()
            .unwrap(),
    );
    let thread = NonZeroUsize::new(1).unwrap();
    let session = build_candidate_session(
        inputs,
        registry,
        runtime,
        FixedBudget::new(8 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: thread,
            target_partitions: thread,
        },
        native_engine_profile(),
    )
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
