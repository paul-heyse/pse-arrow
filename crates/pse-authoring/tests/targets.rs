// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Target selection compares actual template/domain/member facts and bounds expansion.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixture assertions"
)]

use pse_authoring::{
    SourceSpan,
    targets::{TargetPath, TargetRow, parse},
};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
mod support;
use pse_relations::generated::{
    authored,
    enums::{DomainKind, EntityKind},
};
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn context() -> pse_authoring::document::Batches {
    let registry = support::registry();
    let budget = FixedBudget::new(512 << 20);
    let cancel = CancellationToken::new();
    let mut rows = pse_relations::columnar::Collection::new(&registry, budget.as_ref(), &cancel);
    rows.ensure::<authored::template_equations::Row>().unwrap();
    rows.ensure::<authored::template_ports::Row>().unwrap();
    rows.push(authored::instances::Row {
        instance_id: id(1),
        parent_instance_id: None,
        template_id: id(2),
        name: "H".to_owned(),
        param_values: vec![],
        feature_values: vec![],
        property_package_id: None,
        reaction_package_id: None,
        doc: String::new(),
    })
    .unwrap();
    rows.push(authored::entities::Row {
        entity_id: id(1),
        package_id: id(3),
        kind: EntityKind::Instance,
        name: "H".to_owned(),
        qualified_name: "example.H".to_owned(),
        parent_entity_id: None,
        source_span: None,
    })
    .unwrap();
    rows.push(serde_saphyr::from_str::<authored::template_symbols::Row>(&format!("template_id: '{}'\nsymbol_decl_id: '{}'\nname: flow\nrole: variable\nquantity_type_id: '{}'\nindexed_by: [p, j]\ndoc: ''\n", id(2), id(4), id(5))).unwrap()).unwrap();
    for (axis, name, kind) in [(6, "p", DomainKind::Phase), (7, "j", DomainKind::Species)] {
        rows.push(authored::template_domains::Row {
            template_id: id(2),
            name: name.to_owned(),
            kind,
            continuous: false,
            members_from: None,
            bounds: None,
            unit_id: None,
        })
        .unwrap();
        rows.push(authored::domains::Row {
            domain_id: id(axis),
            owner_entity_id: id(1),
            kind,
            continuous: false,
            unit_id: None,
            parent_domain_id: None,
            doc: String::new(),
        })
        .unwrap();
        rows.push(authored::instance_domain_bindings::Row {
            instance_id: id(1),
            domain_name: name.to_owned(),
            domain_id: id(axis),
        })
        .unwrap();
    }
    for (domain, member, label, ordinal) in [
        (6, 8, "L", 0),
        (6, 9, "V", 1),
        (7, 10, "water", 0),
        (7, 11, "salt", 1),
    ] {
        rows.push(authored::domain_members::Row {
            domain_id: id(domain),
            member_id: id(member),
            ordinal,
            label: label.to_owned(),
            coordinate: None,
            ref_entity_id: None,
        })
        .unwrap();
    }
    rows.finish()
        .unwrap()
        .into_values()
        .map(|batch| (batch.relation_id(), batch))
        .collect()
}
#[tokio::test]
async fn mixed_wildcards_expand_complete_actual_member_tuples() {
    let context = context();
    let path = parse("H.flow[*, 'water']", SourceSpan::head(id(12))).unwrap();
    let rows = resolve(&path, &context, id(13)).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0].member.symbol.as_ref().unwrap().index,
        Some(vec![id(8), id(10)])
    );
    assert_eq!(
        rows[1].member.symbol.as_ref().unwrap().index,
        Some(vec![id(9), id(10)])
    );
    assert!(rows.iter().all(|row| {
        row.member
            .symbol
            .as_ref()
            .is_some_and(|symbol| symbol.symbol_decl_id == id(4))
    }));
    let all = parse("H.flow", SourceSpan::head(id(12))).unwrap();
    assert_eq!(resolve(&all, &context, id(13)).await.unwrap().len(), 4);
}
#[tokio::test]
async fn missing_wrong_or_ambiguous_domain_facts_and_budgets_are_refused() {
    let context = context();
    let path = parse("H.flow[*, water]", SourceSpan::head(id(12))).unwrap();
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(
        resolve_cancel(&path, &context, id(13), &cancel)
            .await
            .is_err()
    );
    let mut missing = context.clone();
    change::<authored::instance_domain_bindings::Row>(&mut missing, |rows| {
        rows.pop();
    });
    assert!(resolve(&path, &missing, id(13)).await.is_err());
    let mut wrong = context.clone();
    change::<authored::domains::Row>(&mut wrong, |rows| rows[0].owner_entity_id = id(99));
    assert!(resolve(&path, &wrong, id(13)).await.is_err());
    let mut wrong_kind = context.clone();
    change::<authored::domains::Row>(&mut wrong_kind, |rows| rows[0].kind = DomainKind::Time);
    assert!(resolve(&path, &wrong_kind, id(13)).await.is_err());
    let mut ambiguous = context.clone();
    change::<authored::domain_members::Row>(&mut ambiguous, |rows| {
        rows[3].label = "water".to_owned();
    });
    assert!(resolve(&path, &ambiguous, id(13)).await.is_err());
    assert!(
        resolve(
            &parse("H.flow[L]", SourceSpan::head(id(12))).unwrap(),
            &context,
            id(13)
        )
        .await
        .is_err()
    );
}
#[test]
fn malformed_target_grammar_is_refused() {
    for text in [
        "",
        "H..flow",
        "H.flow[",
        "H.flow[]",
        "H.flow[water,]",
        "H.flow['water]",
        "H.flow[[water]]",
        "H.*[water]",
    ] {
        assert!(parse(text, SourceSpan::head(id(1))).is_err(), "{text}");
    }
}

async fn resolve(
    path: &TargetPath,
    context: &pse_authoring::document::Batches,
    source: SemanticId,
) -> Result<Vec<TargetRow>, pse_authoring::AuthoringError> {
    resolve_cancel(path, context, source, &CancellationToken::new()).await
}
async fn resolve_cancel(
    path: &TargetPath,
    context: &pse_authoring::document::Batches,
    source: SemanticId,
    cancel: &CancellationToken,
) -> Result<Vec<TargetRow>, pse_authoring::AuthoringError> {
    let registry_owner = support::registry();
    let budget = FixedBudget::new(512 << 20);
    let session = support::session(std::sync::Arc::clone(&registry_owner), budget);
    let mut work = session.reserver().open("target-fixture");
    pse_authoring::targets::resolve_native(
        path,
        context,
        &session,
        source,
        work.as_mut(),
        &mut Vec::new(),
        cancel,
    )
    .await
}

fn change<T: pse_relations::columnar::RelationRow>(
    batches: &mut pse_authoring::document::Batches,
    edit: impl FnOnce(&mut Vec<T>),
) {
    let registry = support::registry();
    let spec = T::relation(&registry).unwrap();
    let mut rows = T::rows(&batches[&spec.id]).unwrap();
    edit(&mut rows);
    let mut builder = T::builder(&registry, rows.len()).unwrap();
    for row in rows {
        T::push(&mut builder, row).unwrap();
    }
    batches.insert(spec.id, T::finish(builder).unwrap());
}
