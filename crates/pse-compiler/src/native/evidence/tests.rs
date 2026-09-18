// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use pse_ids::{ContentHash, FixedBudget, SemanticId};
use pse_relations::columnar::RelationRow;
use std::collections::BTreeMap;

#[tokio::test]
async fn repeated_derivation_heads_union_support_without_hiding_conflicts_or_empty_heads() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let (session, _) = crate::passes::native_test::session(
        &registry,
        BTreeMap::new(),
        &FixedBudget::new(128 << 20),
        &cancel,
    )
    .unwrap();
    let support = |value| derivations::ProvenanceDerivationsFieldSupportingItem {
        relation_id: SemanticId::from_bytes([9; 16]),
        row_key: ContentHash::from_bytes([value; 32]),
    };
    let head = |value, supporting| derivations::Row {
        derivation_id: SemanticId::from_bytes([value; 16]),
        relation_id: SemanticId::from_bytes([8; 16]),
        row_key: ContentHash::from_bytes([value; 32]),
        rule_id: Some(SemanticId::from_bytes([7; 16])),
        algorithm_id: None,
        supporting,
    };
    let mut conflict = head(3, vec![support(4)]);
    conflict.relation_id = SemanticId::from_bytes([6; 16]);
    let fragments = [
        vec![
            head(1, vec![support(2), support(1)]),
            head(2, vec![]),
            head(3, vec![]),
        ],
        vec![
            head(1, vec![support(3), support(2)]),
            head(2, vec![]),
            conflict.clone(),
        ],
    ];
    let mut inputs = BTreeMap::new();
    for (index, rows) in fragments.into_iter().enumerate() {
        let mut builder = derivations::Row::builder(&registry, rows.len()).unwrap();
        for row in rows {
            derivations::Row::push(&mut builder, row).unwrap();
        }
        inputs.insert(
            format!("fragment{index}"),
            derivations::Row::finish(builder).unwrap(),
        );
    }
    let session = session.with_checked_role_inputs(inputs, &cancel).unwrap();
    let plan = combine(
        derivations::RELATION_KEY,
        vec![
            session.scan_role("fragment0").unwrap(),
            session.scan_role("fragment1").unwrap(),
        ],
        &session,
        &cancel,
    )
    .unwrap();
    let rows = session
        .prepare_rule_plan(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap()
        .into_checked_relation(&registry, derivations::spec(&registry).unwrap(), &cancel)
        .unwrap();
    let actual = derivations::Row::rows(&rows).unwrap();
    assert_eq!(actual.len(), 4);
    for expected in [
        head(1, vec![support(1), support(2), support(3)]),
        head(2, vec![]),
        head(3, vec![]),
        conflict,
    ] {
        assert!(
            actual.contains(&expected),
            "missing exact derivation: {expected:?}"
        );
    }
}
