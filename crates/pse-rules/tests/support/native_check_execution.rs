// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The same native predicate drives typed findings and no-scan provider policies.
#![allow(
    clippy::unwrap_used,
    reason = "independent native-check fixtures and expected results"
)]

use super::session;
use datafusion::arrow::{
    array::{Array, BooleanArray, Int64Array, RecordBatch},
    datatypes::DataType,
};
use pse_columnar::CancellationToken;
use pse_ids::SemanticId;
use pse_rules::invariants::{InvariantScope, RegistryRequirementPlanner, run_invariants};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, FieldContract as T, Namespace, RelationDecl, RelationKey, SnapshotClass,
        provider::{ProviderPolicy, ProviderScope},
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "native_checked",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Native check fixture",
        )
        .pk(&["id"])
        .columns(vec![
            T::key("id", T::native(DataType::Int64), "Actual identity"),
            T::native(DataType::Boolean)
                .with_name("approved")
                .optional(),
        ])
        .checks(BTreeMap::from([("approved".into(), "approved".into())])),
    );
    let registry = Arc::new(builder.build().unwrap());
    pse_engine::validation::bind_defaults(&registry).unwrap();
    registry
}
fn rows(registry: &Registry, values: Vec<Option<bool>>) -> BTreeMap<RelationKey, RecordBatch> {
    let spec = registry.relation("authored.native_checked").unwrap();
    let schema = Arc::new(pse_schema::arrow::relation_schema(registry, spec).unwrap());
    BTreeMap::from([(
        spec.key,
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from_iter_values(
                    (0..values.len()).map(|n| i64::try_from(n).unwrap()),
                )),
                Arc::new(BooleanArray::from(values)),
            ],
        )
        .unwrap(),
    )])
}

#[tokio::test]
async fn native_predicates_report_exact_false_and_unknown_keys_without_a_rule_declaration() {
    let registry = registry();
    let spec = registry.relation("authored.native_checked").unwrap();
    let check = spec.row_check_id("approved").unwrap();
    assert!(spec.row_check_id("missing").is_none());
    assert!(
        !registry
            .invariants()
            .iter()
            .any(|declaration| declaration.id == check)
    );
    let input = rows(&registry, vec![Some(true), Some(false), None]);
    let report = pse_relations::validate::ValidationContext::for_registry(&registry)
        .unwrap()
        .relation(&registry, spec)
        .unwrap()
        .evaluate(&input[&spec.key], 10, &CancellationToken::new())
        .unwrap();
    assert_eq!(report.violations, 2);
    assert_eq!(
        report.valid_rows.values().iter().collect::<Vec<_>>(),
        [true, false, false]
    );
    let findings = &report.findings;
    let keys = findings
        .column_by_name("key_literals")
        .unwrap()
        .as_any()
        .downcast_ref::<datafusion::arrow::array::StringArray>()
        .unwrap();
    let rules = findings
        .column_by_name("rule")
        .unwrap()
        .as_any()
        .downcast_ref::<datafusion::arrow::array::StringArray>()
        .unwrap();
    for (row, invalid) in [1, 2].into_iter().enumerate() {
        assert_eq!(rules.value(row), "check:approved");
        let observed: BTreeMap<String, String> = serde_json::from_str(keys.value(row)).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&observed["id"]).unwrap(),
            serde_json::json!(["i64", invalid])
        );
    }
    // Provider execution sees admitted inputs; the same declared check still runs
    // when explicitly required, even for a query with no scan.
    let input = rows(&registry, vec![Some(true)]);
    let bound = session(&registry, &input);
    let selected = BTreeSet::from([check]);
    let checked = run_invariants(
        &input,
        &bound,
        &registry,
        InvariantScope::Required(&selected),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(checked.check_count(), 1);
    assert_eq!(checked.error_count(), 0);
    let affected = run_invariants(
        &input,
        &bound,
        &registry,
        InvariantScope::Affected(&BTreeSet::new()),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(affected.check_count(), 0);
}

#[tokio::test]
async fn native_check_provider_policy_applies_to_no_scan_execution_and_refuses_absent_sources() {
    let registry = registry();
    let required = registry
        .relation("authored.native_checked")
        .unwrap()
        .row_check_id("approved")
        .unwrap();
    let factory = pse_testkit::NativeFixture::new(std::num::NonZeroUsize::new(64 << 20).unwrap())
        .unwrap()
        .into_factory()
        .with_requirement_planner(Arc::new(RegistryRequirementPlanner));
    for (values, valid) in [
        (vec![], true),
        (vec![Some(true)], true),
        (vec![Some(false)], false),
        (vec![None], false),
    ] {
        let cancel = CancellationToken::new();
        let mut policy = ProviderPolicy::new(SemanticId::from_bytes([8; 16]), ProviderScope::Root);
        policy.requirements.insert(required);
        let candidate = factory.candidate(rows(&registry, values), Arc::clone(&registry), &cancel);
        if !valid {
            assert!(
                candidate.is_err(),
                "invalid rows must refuse before provider binding"
            );
            continue;
        }
        let bound = candidate.unwrap().with_policy(policy).unwrap();
        let prepared = bound.prepare_sql("SELECT 42", &cancel).await.unwrap();
        assert!(prepared.execute(&cancel).await.is_ok());
    }
    let mut policy = ProviderPolicy::new(SemanticId::from_bytes([8; 16]), ProviderScope::Root);
    policy.requirements.insert(required);
    let cancel = CancellationToken::new();
    let missing = factory
        .candidate(BTreeMap::new(), registry, &cancel)
        .unwrap()
        .with_policy(policy)
        .unwrap();
    assert!(
        missing
            .prepare_sql("SELECT 42", &cancel)
            .await
            .unwrap()
            .execute(&cancel)
            .await
            .is_err()
    );
}

pub(super) fn invalid_keys(
    registry: &Registry,
    spec: &pse_schema::model::RelationSpec,
    rows: &[Vec<serde_json::Value>],
    check: &str,
    key: &str,
) -> Vec<Vec<u8>> {
    let input =
        pse_relations::testing::untrusted_batch_from_literals(registry, spec, rows).unwrap();
    pse_engine::validation::bind_defaults(registry).unwrap();
    let report = pse_relations::validate::ValidationContext::for_registry(registry)
        .unwrap()
        .relation(registry, spec)
        .unwrap()
        .evaluate(&input, rows.len(), &CancellationToken::new())
        .unwrap();
    assert!(!report.truncated);
    let rules = report
        .findings
        .column_by_name("rule")
        .unwrap()
        .as_any()
        .downcast_ref::<datafusion::arrow::array::StringArray>()
        .unwrap();
    assert!((0..rules.len()).all(|row| rules.value(row) == format!("check:{check}")));
    let keys = input
        .column_by_name(key)
        .unwrap()
        .as_any()
        .downcast_ref::<datafusion::arrow::array::FixedSizeBinaryArray>()
        .unwrap();
    report
        .valid_rows
        .values()
        .iter()
        .enumerate()
        .filter(|(_, valid)| !valid)
        .map(|(row, _)| keys.value(row).to_vec())
        .collect()
}

async fn assert_predicate(
    relation: &str,
    name: &str,
    columns: Vec<(&str, Arc<dyn Array>)>,
    expected: &[bool],
) {
    use datafusion::{
        arrow::datatypes::{Field, Schema},
        common::DFSchema,
        execution::context::SessionContext,
    };
    let registry = pse_engine::validation::registry().unwrap();
    let sql = &registry.relation(relation).unwrap().checks[name];
    let schema = Arc::new(Schema::new(
        columns
            .iter()
            .map(|(name, array)| Field::new(*name, array.data_type().clone(), true))
            .collect::<Vec<_>>(),
    ));
    let input = RecordBatch::try_new(
        Arc::clone(&schema),
        columns.into_iter().map(|(_, array)| array).collect(),
    )
    .unwrap();
    let context = SessionContext::new();
    let predicate = context
        .state()
        .create_logical_expr(sql, &DFSchema::try_from(schema.as_ref().clone()).unwrap())
        .unwrap();
    let output = context
        .read_batch(input)
        .unwrap()
        .select(vec![predicate.is_true()])
        .unwrap()
        .collect()
        .await
        .unwrap();
    let actual = output
        .iter()
        .flat_map(|batch| {
            batch
                .column(0)
                .as_any()
                .downcast_ref::<BooleanArray>()
                .unwrap()
                .values()
                .iter()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "{relation}:{name}");
}

#[tokio::test]
async fn native_physical_predicates_keep_positive_optional_and_ordered_bound_rules() {
    use datafusion::arrow::array::Float64Array;
    for (relation, name, column, expected) in [
        (
            "reference.units",
            "positive_scale",
            "scale_to_canonical",
            [true, false, false, false],
        ),
        (
            "reference.quantity_types",
            "positive_nominal",
            "nominal_magnitude",
            [true, false, false, true],
        ),
    ] {
        assert_predicate(
            relation,
            name,
            vec![(
                column,
                Arc::new(Float64Array::from(vec![
                    Some(1.0),
                    Some(0.0),
                    Some(-1.0),
                    None,
                ])),
            )],
            &expected,
        )
        .await;
    }
    assert_predicate(
        "authored.continuous_domains",
        "ordered_bounds",
        vec![
            ("lower", Arc::new(Float64Array::from(vec![1.0, 1.0, 2.0]))),
            ("upper", Arc::new(Float64Array::from(vec![2.0, 1.0, 1.0]))),
        ],
        &[true, false, false],
    )
    .await;
}
