// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native Delta preserves complete syntax, operand order and payload-free alternatives.
use super::*;
use pse_relations::generated::{enums::PredicateComparison, normalized::predicate_nodes as p};

#[tokio::test]
async fn cold_declared_syntax_keeps_ordered_operand_pairs_and_rejects_invalid_domains() {
    let registry = pse_schema::registry().unwrap();
    let contract = DeclaredCheck::new(registry, p::RELATION_ID).unwrap();
    let root = tempfile::tempdir().unwrap();
    let uri = url::Url::from_directory_path(root.path()).unwrap();
    let context = context();
    let expected = expected();
    let mut rows = p::Builder::new().unwrap();
    for row in &expected {
        rows.push(row.clone()).unwrap();
    }
    let input = context
        .read_batch(rows.finish().unwrap().into_batch())
        .unwrap()
        .into_unoptimized_plan();
    let write = DeltaWrite::declared(
        DeltaTableBuilder::from_url(uri.clone())
            .unwrap()
            .build()
            .unwrap(),
        input,
        SaveMode::Append,
        CommitProperties::default(),
        contract,
    )
    .unwrap();
    let state = context.state();
    native_execution::run(&state, pse_schema::shared_registry().unwrap(), &write)
        .await
        .unwrap();
    drop(context);

    let cold = super::context();
    let table = DeltaTableBuilder::from_url(uri.clone())
        .unwrap()
        .load()
        .await
        .unwrap();
    let version = table.version().unwrap();
    let contract = DeclaredCheck::open(&table, &cold.state()).unwrap();
    let view = pse_catalog::delta::provider::open_declared_view(
        uri.clone(),
        i64::try_from(version).unwrap(),
        &contract,
        Arc::new(cold.state()),
    )
    .await
    .unwrap();
    cold.register_table("syntax", Arc::new(view)).unwrap();
    let batches = cold
        .sql("SELECT * FROM syntax ORDER BY predicate_id")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let restored = batches
        .iter()
        .flat_map(|batch| restored_rows(batch, contract.layout().execution_schema()))
        .collect::<Vec<_>>();
    assert_eq!(restored, expected);

    let mut rows = p::Builder::new().unwrap();
    rows.push(invalid_domain()).unwrap();
    let input = cold
        .read_batch(rows.finish().unwrap().into_batch())
        .unwrap()
        .into_unoptimized_plan();
    let write = DeltaWrite::declared(
        table,
        input,
        SaveMode::Append,
        CommitProperties::default(),
        contract,
    )
    .unwrap();
    let state = cold.state();
    let failure = native_execution::run(&state, pse_schema::shared_registry().unwrap(), &write)
        .await
        .unwrap_err();
    assert!(
        failure.to_string().contains("failed validation check"),
        "{failure}"
    );
    assert_eq!(
        DeltaTableBuilder::from_url(uri)
            .unwrap()
            .load()
            .await
            .unwrap()
            .version(),
        Some(version)
    );
}

fn expected() -> Vec<p::Row> {
    let values = [
        p::NormalizedPredicateNodesFieldValue::from_compare(
            p::NormalizedPredicateNodesFieldValueCompare {
                comparison: PredicateComparison::Eq,
                operands: vec![
                    p::NormalizedPredicateNodesFieldValueCompareOperandsItem::from_expression(
                        p::NormalizedPredicateNodesFieldValueCompareOperandsItemExpression {
                            node_id: 17,
                        },
                    ),
                    p::NormalizedPredicateNodesFieldValueCompareOperandsItem::from_enum_literal(
                        p::NormalizedPredicateNodesFieldValueCompareOperandsItemEnumLiteral {
                            enum_id: SemanticId::from_bytes([7; 16]),
                            member: "vapour".into(),
                        },
                    ),
                ],
            },
        ),
        p::NormalizedPredicateNodesFieldValue::from_null(),
    ];
    values
        .into_iter()
        .enumerate()
        .map(|(id, value)| p::Row {
            source_id: SemanticId::NIL,
            predicate_id: i64::try_from(id).unwrap(),
            value,
        })
        .collect()
}

fn invalid_domain() -> p::Row {
    p::Row {
        source_id: SemanticId::NIL,
        predicate_id: 2,
        value: p::NormalizedPredicateNodesFieldValue::from_in(
            p::NormalizedPredicateNodesFieldValueIn {
                expression: 0,
                domain: p::NormalizedPredicateNodesFieldValueInDomain::from_template(
                    p::NormalizedPredicateNodesFieldValueInDomainTemplate {
                        template_id: SemanticId::NIL,
                        name: String::new(),
                    },
                ),
            },
        ),
    }
}

fn restored_rows(batch: &RecordBatch, schema: &SchemaRef) -> Vec<p::Row> {
    // SQL owns its output schema annotations. The cold declaration can restore
    // relation annotations only after every field agrees exactly.
    assert_eq!(batch.schema().fields(), schema.fields());
    let declared = RecordBatch::try_new(Arc::clone(schema), batch.columns().to_vec()).unwrap();
    p::View::try_from_batch(&declared).unwrap().rows().unwrap()
}
