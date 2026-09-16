// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Independent native expansion cases. These exercise query results; they do not
//! mint producer receipts from the synthetic component-test inputs.
#![allow(clippy::unwrap_used, reason = "explicit finite native plan fixtures")]
use crate::passes::native_construction::Plans;
use crate::passes::p3::products::expansion;
use datafusion::{
    arrow::array::{Array, FixedSizeBinaryArray, Int64Array, ListArray},
    functions_nested::expr_fn::make_array,
    logical_expr::{LogicalPlan, LogicalPlanBuilder, lit},
};
use pse_catalog::session::scalar;
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{enums::DomainKind, normalized},
};
use pse_schema::model::{PassSpec, RelationKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn plans<'a>(
    registry: &Arc<pse_schema::Registry>,
    inputs: &BTreeMap<RelationKey, FieldCheckedBatch>,
    pass: &'a PassSpec,
    cancel: &'a CancellationToken,
) -> Plans<'a> {
    let (session, sources) = crate::passes::native_test::session(
        registry,
        inputs.clone(),
        &FixedBudget::new(2usize << 30),
        cancel,
    )
    .unwrap();
    Plans::new(inputs, &sources, pass, &session, cancel).unwrap()
}
fn seed(plans: &Plans<'_>, factors: &[SemanticId], present: bool) -> LogicalPlan {
    LogicalPlanBuilder::empty(present)
        .project([
            scalar::id_list(factors.iter().map(|id| plans.sid(*id).unwrap()).collect())
                .alias("factors"),
            make_array(vec![lit("component-test source occurrence")]).alias("supports"),
        ])
        .unwrap()
        .build()
        .unwrap()
}
fn ids(array: &ListArray, row: usize) -> Vec<SemanticId> {
    let value = array.value(row);
    let value = value
        .as_any()
        .downcast_ref::<FixedSizeBinaryArray>()
        .unwrap();
    (0..value.len())
        .map(|row| SemanticId::from_bytes(value.value(row).try_into().unwrap()))
        .collect()
}

#[tokio::test]
async fn ordered_subsets_keep_repeated_factor_positions_and_exact_identity_frame() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let pass = registry.pass("P3@1").unwrap();
    let mut plans = plans(&registry, &BTreeMap::new(), pass, &cancel);
    let parent = seed(&plans, &[id(1), id(2), id(1)], true);
    let subsets = expansion::subsets(&mut plans, parent).await.unwrap();
    let complete = plans
        .session
        .prepare_rule_plan(subsets.clone(), &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let mut actual = BTreeSet::new();
    for batch in complete.batches() {
        let factors = batch
            .column_by_name("factors")
            .unwrap()
            .as_any()
            .downcast_ref::<ListArray>()
            .unwrap();
        let positions = batch
            .column_by_name("positions")
            .unwrap()
            .as_any()
            .downcast_ref::<ListArray>()
            .unwrap();
        for row in 0..batch.num_rows() {
            let values = positions.value(row);
            let values = values.as_any().downcast_ref::<Int64Array>().unwrap();
            actual.insert((ids(factors, row), values.values().to_vec()));
        }
    }
    assert_eq!(
        actual,
        BTreeSet::from([
            (vec![], vec![]),
            (vec![id(1)], vec![0]),
            (vec![id(2)], vec![1]),
            (vec![id(1)], vec![2]),
            (vec![id(1), id(2)], vec![0, 1]),
            (vec![id(1), id(1)], vec![0, 2]),
            (vec![id(2), id(1)], vec![1, 2]),
            (vec![id(1), id(2), id(1)], vec![0, 1, 2]),
        ])
    );
    let labels = expansion::labels(&mut plans, subsets).await.unwrap();
    let complete = plans
        .session
        .prepare_rule_plan(labels, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let mut count = 0;
    for batch in complete.batches() {
        let factors = batch
            .column_by_name("factors")
            .unwrap()
            .as_any()
            .downcast_ref::<ListArray>()
            .unwrap();
        let labels = batch
            .column_by_name("product_id")
            .unwrap()
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .unwrap();
        for row in 0..batch.num_rows() {
            assert_eq!(
                labels.value(row),
                pse_templates::identity::domain_product_id(&ids(factors, row)).as_bytes()
            );
            count += 1;
        }
    }
    assert_eq!(count, 7);
}

#[tokio::test]
async fn scalar_product_requires_a_real_consumer() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let pass = registry.pass("P3@1").unwrap();
    for present in [false, true] {
        let mut plans = plans(&registry, &BTreeMap::new(), pass, &cancel);
        let parent = seed(&plans, &[], present);
        let subsets = expansion::subsets(&mut plans, parent).await.unwrap();
        let labels = expansion::labels(&mut plans, subsets).await.unwrap();
        let complete = plans
            .session
            .prepare_rule_plan(labels, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        assert_eq!(
            complete
                .batches()
                .iter()
                .map(|batch| batch.num_rows())
                .sum::<usize>(),
            usize::from(present)
        );
    }
}

#[tokio::test]
async fn cartesian_candidates_preserve_duplicate_axes_empty_and_continuous_domains() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let mut domains = normalized::domains::Builder::with_registry(&registry, 3).unwrap();
    for (domain, continuous) in [(id(1), false), (id(2), false), (id(3), true)] {
        domains
            .push(normalized::domains::Row {
                domain_id: domain,
                owner_entity_id: id(9),
                kind: if continuous {
                    DomainKind::Time
                } else {
                    DomainKind::Species
                },
                continuous,
                unit_id: None,
                parent_domain_id: None,
                doc: String::new(),
            })
            .unwrap();
    }
    let mut members = normalized::domain_members::Builder::with_registry(&registry, 2).unwrap();
    for (ordinal, member) in [id(4), id(5)].into_iter().enumerate() {
        members
            .push(normalized::domain_members::Row {
                domain_id: id(1),
                member_id: member,
                ordinal: u32::try_from(ordinal).unwrap(),
                label: ordinal.to_string(),
                coordinate: None,
                ref_entity_id: None,
            })
            .unwrap();
    }
    let mut products = normalized::domain_products::Builder::with_registry(&registry, 4).unwrap();
    for factors in [vec![], vec![id(1), id(1)], vec![id(2)], vec![id(3)]] {
        products
            .push(normalized::domain_products::Row {
                product_id: pse_templates::identity::domain_product_id(&factors),
                domain_ids: factors,
            })
            .unwrap();
    }
    let inputs = BTreeMap::from([
        (normalized::domains::RELATION_KEY, domains.finish().unwrap()),
        (
            normalized::domain_members::RELATION_KEY,
            members.finish().unwrap(),
        ),
        (
            normalized::domain_products::RELATION_KEY,
            products.finish().unwrap(),
        ),
    ]);
    let mut plans = plans(&registry, &inputs, registry.pass("P3@1").unwrap(), &cancel);
    let tuples = expansion::tuples(&mut plans).await.unwrap();
    let complete = plans
        .session
        .prepare_rule_plan(tuples, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let mut actual = BTreeSet::new();
    for batch in complete.batches() {
        let tuples = batch
            .column_by_name("tuple")
            .unwrap()
            .as_any()
            .downcast_ref::<ListArray>()
            .unwrap();
        for row in 0..batch.num_rows() {
            actual.insert(ids(tuples, row));
        }
    }
    assert_eq!(
        actual,
        BTreeSet::from([
            vec![],
            vec![id(4), id(4)],
            vec![id(4), id(5)],
            vec![id(5), id(4)],
            vec![id(5), id(5)]
        ])
    );
}
