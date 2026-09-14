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
    targets::{TargetBudget, TargetContext, parse, resolve, resolve_with_budget},
};
use pse_ids::SemanticId;
use pse_relations::generated::{
    authored,
    enums::{DomainKind, EntityKind},
};
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn context() -> TargetContext {
    let mut context = TargetContext::default();
    context.instances.push(authored::instances::Row {
        instance_id: id(1),
        parent_instance_id: None,
        template_id: id(2),
        name: "H".to_owned(),
        param_values: vec![],
        feature_values: vec![],
        property_package_id: None,
        reaction_package_id: None,
        doc: String::new(),
    });
    context.entities.push(authored::entities::Row {
        entity_id: id(1),
        package_id: id(3),
        kind: EntityKind::Instance,
        name: "H".to_owned(),
        qualified_name: "example.H".to_owned(),
        parent_entity_id: None,
        source_span: None,
    });
    context.symbols.push(serde_saphyr::from_str(&format!("template_id: '{}'\nsymbol_decl_id: '{}'\nname: flow\nrole: variable\nquantity_type_id: '{}'\nindexed_by: [p, j]\ndoc: ''\n", id(2), id(4), id(5))).unwrap());
    for (axis, name, kind) in [(6, "p", DomainKind::Phase), (7, "j", DomainKind::Species)] {
        context
            .template_domains
            .push(authored::template_domains::Row {
                template_id: id(2),
                name: name.to_owned(),
                kind,
                continuous: false,
                members_from: None,
                bounds: None,
                unit_id: None,
            });
        context.domains.push(authored::domains::Row {
            domain_id: id(axis),
            owner_entity_id: id(1),
            kind,
            continuous: false,
            unit_id: None,
            parent_domain_id: None,
            doc: String::new(),
        });
        context
            .domain_bindings
            .push(authored::instance_domain_bindings::Row {
                instance_id: id(1),
                domain_name: name.to_owned(),
                domain_id: id(axis),
            });
    }
    for (domain, member, label, ordinal) in [
        (6, 8, "L", 0),
        (6, 9, "V", 1),
        (7, 10, "water", 0),
        (7, 11, "salt", 1),
    ] {
        context.members.push(authored::domain_members::Row {
            domain_id: id(domain),
            member_id: id(member),
            ordinal,
            label: label.to_owned(),
            coordinate: None,
            ref_entity_id: None,
        });
    }
    context
}
#[test]
fn mixed_wildcards_expand_complete_actual_member_tuples() {
    let context = context();
    let path = parse("H.flow[*, 'water']", SourceSpan::head(id(12))).unwrap();
    let rows = resolve(&path, &context, id(13)).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].index, Some(vec![id(8), id(10)]));
    assert_eq!(rows[1].index, Some(vec![id(9), id(10)]));
    assert!(
        rows.iter()
            .all(|row| !row.wildcard && row.symbol_decl_id == Some(id(4)))
    );
    let all = parse("H.flow", SourceSpan::head(id(12))).unwrap();
    assert_eq!(resolve(&all, &context, id(13)).unwrap().len(), 4);
}
#[test]
fn missing_wrong_or_ambiguous_domain_facts_and_budgets_are_refused() {
    let context = context();
    let path = parse("H.flow[*, water]", SourceSpan::head(id(12))).unwrap();
    assert!(
        resolve_with_budget(
            &path,
            &context,
            id(13),
            TargetBudget { max_targets: 1 },
            &|| false
        )
        .is_err()
    );
    assert!(
        resolve_with_budget(&path, &context, id(13), TargetBudget::default(), &|| true).is_err()
    );
    let mut missing = context.clone();
    missing.domain_bindings.pop();
    assert!(resolve(&path, &missing, id(13)).is_err());
    let mut wrong = context.clone();
    wrong.domains[0].owner_entity_id = id(99);
    assert!(resolve(&path, &wrong, id(13)).is_err());
    let mut wrong_kind = context.clone();
    wrong_kind.domains[0].kind = DomainKind::Time;
    assert!(resolve(&path, &wrong_kind, id(13)).is_err());
    let mut ambiguous = context.clone();
    ambiguous.members[3].label = "water".to_owned();
    assert!(resolve(&path, &ambiguous, id(13)).is_err());
    assert!(
        resolve(
            &parse("H.flow[L]", SourceSpan::head(id(12))).unwrap(),
            &context,
            id(13)
        )
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
