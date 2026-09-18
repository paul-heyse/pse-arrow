// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The same native predicate drives typed findings and no-scan provider policies.
#![allow(
    clippy::unwrap_used,
    reason = "independent native-check fixtures and expected results"
)]

use super::{budget, session};
use datafusion::arrow::{
    array::{BooleanArray, Int64Array, RecordBatch},
    datatypes::DataType,
};
use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{ExecutionSettings, SessionFactory, native_engine_profile};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
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
    Arc::new(builder.build().unwrap())
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
    let bound = session(&registry, &input);
    let selected = BTreeSet::from([check]);
    let report = run_invariants(
        &input,
        &bound,
        &registry,
        InvariantScope::Required(&selected),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(report.check_count(), 1);
    assert_eq!(report.error_count(), 2);
    let mut keys = BTreeSet::new();
    for batch in report.findings() {
        let view =
            pse_relations::generated::runtime::diagnostics_findings::View::from_checked(batch)
                .unwrap();
        for ordinal in 0..batch.batch().num_rows() {
            let finding = view.row(ordinal).unwrap();
            assert_eq!(finding.check_id, check);
            assert_eq!(finding.evidence.row.as_ref().unwrap().relation_id, spec.id);
            keys.insert(finding.evidence.row.unwrap().row_key);
        }
    }
    let field = spec.column("id").unwrap();
    for invalid in [1, 2] {
        assert!(
            keys.contains(
                &super::row_key::values(
                    &registry,
                    spec.id,
                    &[field],
                    &[pse_schema::model::Cell::I64(invalid)]
                )
                .await
            )
        );
    }
    let plan = report
        .completion()
        .unwrap()
        .prepared()
        .optimized_plan()
        .display_indent()
        .to_string();
    assert!(plan.contains("IS NOT TRUE"));
    assert!(!plan.contains("RulePlan"));
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
    let factory = SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        budget(),
        native_engine_profile(),
    )
    .unwrap()
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
        let bound = factory
            .candidate(rows(&registry, values), Arc::clone(&registry), &cancel)
            .unwrap()
            .with_policy(policy)
            .unwrap();
        let prepared = bound.prepare_sql("SELECT 42", &cancel).await.unwrap();
        assert_eq!(prepared.execute(&cancel).await.is_ok(), valid);
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

pub(super) fn violations(
    session: &pse_catalog::session::SnapshotSession,
    spec: &pse_schema::model::RelationSpec,
    name: &str,
) -> datafusion::logical_expr::LogicalPlan {
    use datafusion::logical_expr::{LogicalPlanBuilder, col};
    let predicate = session
        .row_check_expressions(spec.key)
        .unwrap()
        .remove(name)
        .unwrap();
    LogicalPlanBuilder::scan(
        session.table_reference(&spec.key).unwrap(),
        session.table_source(&spec.key).unwrap(),
        None,
    )
    .unwrap()
    .filter(predicate.is_not_true())
    .unwrap()
    .project(spec.primary_key.iter().map(|name| col(*name).alias(*name)))
    .unwrap()
    .build()
    .unwrap()
}

async fn assert_predicate(
    relation: &str,
    name: &str,
    columns: Vec<(&str, Arc<dyn datafusion::arrow::array::Array>)>,
    expected: &[bool],
) {
    use datafusion::{
        arrow::datatypes::{Field, Schema},
        common::DFSchema,
        execution::context::SessionContext,
    };
    let registry = pse_schema::registry().unwrap();
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
async fn normalized_domain_checks_require_nonempty_template_names() {
    use pse_relations::{
        generated::normalized::{expression_index_bindings as i, predicate_nodes as p},
        typed::CellCodec,
    };
    let registry = pse_schema::registry().unwrap();
    let actual = p::NormalizedPredicateNodesFieldValueInDomain::from_actual(
        p::NormalizedPredicateNodesFieldValueInDomainActual {
            domain_id: SemanticId::NIL,
        },
    );
    let template = |name: &str| {
        p::NormalizedPredicateNodesFieldValueInDomain::from_template(
            p::NormalizedPredicateNodesFieldValueInDomainTemplate {
                template_id: SemanticId::NIL,
                name: name.into(),
            },
        )
    };
    let domains = [actual, template("species"), template("")];
    let schema = i::schema().unwrap();
    let domain_array = pse_relations::cells::array_from_cells(
        registry,
        schema.field_with_name("domain").unwrap(),
        &domains
            .iter()
            .cloned()
            .map(CellCodec::into_cell)
            .collect::<Vec<_>>(),
    )
    .unwrap();
    assert_predicate(
        "normalized.expression_index_bindings",
        "domain_name",
        vec![("domain", domain_array)],
        &[true, true, false],
    )
    .await;
    let mut values = domains
        .into_iter()
        .map(|domain| {
            p::NormalizedPredicateNodesFieldValue::from_in(
                p::NormalizedPredicateNodesFieldValueIn {
                    expression: 0,
                    domain,
                },
            )
            .into_cell()
        })
        .collect::<Vec<_>>();
    values.push(p::NormalizedPredicateNodesFieldValue::from_null().into_cell());
    let schema = p::schema().unwrap();
    let array = pse_relations::cells::array_from_cells(
        registry,
        schema.field_with_name("value").unwrap(),
        &values,
    )
    .unwrap();
    assert_predicate(
        "normalized.predicate_nodes",
        "domain_name",
        vec![("value", array)],
        &[true, true, false, true],
    )
    .await;
}

#[tokio::test]
async fn native_participation_predicates_refuse_incomplete_alternatives() {
    use datafusion::arrow::{
        array::{BinaryArray, StringArray, StructArray, new_null_array},
        buffer::NullBuffer,
    };
    use datafusion::common::DFSchema;
    use datafusion::prelude::SessionContext;
    let registry = pse_schema::registry().unwrap();
    let spec = registry.relation("compiled.law_participation").unwrap();
    let contract = pse_catalog::delta::contract::DeclaredCheck::new(registry, spec.id).unwrap();
    let schema = contract.layout().storage_schema().clone();
    let DataType::Struct(decision_fields) = schema.field_with_name("decision").unwrap().data_type()
    else {
        panic!("typed decision")
    };
    let DataType::Struct(included_fields) = decision_fields[1].data_type() else {
        panic!("included arm")
    };
    let DataType::Struct(excluded_fields) = decision_fields[2].data_type() else {
        panic!("excluded arm")
    };
    let included = Arc::new(StructArray::new(
        included_fields.clone(),
        vec![
            Arc::new(StringArray::from(vec![
                "negative", "positive", "invalid", "positive", "positive", "positive",
            ])),
            new_null_array(included_fields[1].data_type(), 6),
        ],
        Some(NullBuffer::from(vec![true, true, true, false, false, true])),
    ));
    let excluded = Arc::new(StructArray::new(
        excluded_fields.clone(),
        vec![Arc::new(StringArray::from(vec![
            "family_mismatch",
            "family_mismatch",
            "family_mismatch",
            "family_mismatch",
            "invalid",
            "family_mismatch",
        ]))],
        Some(NullBuffer::from(vec![
            false, false, false, true, true, true,
        ])),
    ));
    let decision = Arc::new(StructArray::new(
        decision_fields.clone(),
        vec![
            Arc::new(StringArray::from(vec![
                "included", "included", "included", "excluded", "excluded", "excluded",
            ])),
            included,
            excluded,
        ],
        None,
    ));
    let id: Arc<dyn datafusion::arrow::array::Array> =
        Arc::new(BinaryArray::from_vec(vec![&[0_u8; 16]; 6]));
    let batch =
        RecordBatch::try_new(schema.clone(), vec![id.clone(), id.clone(), decision, id]).unwrap();
    let state = contract.bind(&SessionContext::new().state()).unwrap();
    let context = SessionContext::new_with_state(state);
    let predicate = context
        .state()
        .create_logical_expr(
            &contract.properties()["delta.constraints.pse_contract"],
            &DFSchema::try_from(schema.as_ref().clone()).unwrap(),
        )
        .unwrap();
    let result = context
        .read_batch(batch)
        .unwrap()
        .select(vec![predicate.is_true()])
        .unwrap()
        .collect()
        .await
        .unwrap();
    let actual = result
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
    assert_eq!(actual, [true, true, false, true, false, false]);
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
        (
            "inferred.tear_candidates",
            "nonnegative_cost",
            "cost",
            [true, true, false, false],
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
