// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{CandidateTable, admit_plan, intermediate_schema, restore_semantic_fields};
use datafusion::arrow::array::RecordBatch;
use datafusion::catalog::TableProvider;
use datafusion::common::Column;
use datafusion::datasource::provider_as_source;
use datafusion::logical_expr::{LogicalPlan, LogicalPlanBuilder, col};
use pse_ids::{CancellationToken, FixedBudget};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, ColumnSpec, EnumDecl, EnumMember, LogicalType, Namespace, RelationDecl,
        SnapshotClass,
    },
};
use std::sync::Arc;

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
            ColumnSpec::key("id", LogicalType::U64, "identity"),
            ColumnSpec::payload(
                "members",
                LogicalType::list(LogicalType::enumeration("Phase")),
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
        key: spec.key,
        batch,
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
    let mut expected = pse_schema::arrow::field_for(
        &registry,
        &ColumnSpec::payload("phase", LogicalType::enumeration("Phase"), "expected"),
    )
    .unwrap();
    expected.metadata_mut().remove(pse_schema::arrow::KEY_ROLE);
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
        &ColumnSpec::payload("phase", LogicalType::enumeration("OtherPhase"), "forged"),
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

#[tokio::test]
async fn executed_unnest_exports_the_exact_source_enum_contract() {
    use crate::session::{
        ExecutionSettings, ThreadBudget, build_candidate_session, phase0_reference_profile,
    };
    use datafusion::execution::{memory_pool::GreedyMemoryPool, runtime_env::RuntimeEnvBuilder};
    use pse_ids::{CancellationToken, FixedBudget};
    use std::{collections::BTreeMap, num::NonZeroUsize};
    let (registry, table, _) = fixture();
    let candidate = table.as_ref().downcast_ref::<CandidateTable>().unwrap();
    let inputs = BTreeMap::from([(candidate.key, candidate.batch.clone())]);
    let key = candidate.key;
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
        phase0_reference_profile(),
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
    let mut expected = pse_schema::arrow::field_for(
        session.registry(),
        &ColumnSpec::payload("phase", LogicalType::enumeration("Phase"), "expected"),
    )
    .unwrap();
    expected.metadata_mut().remove(pse_schema::arrow::KEY_ROLE);
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
