// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use pse_contract::{projection_struct, boundary_projection::Direction};
use std::collections::BTreeMap;

fn encoder() -> OptionEncoder { OptionEncoder::new(&super::super::tests::imports()).unwrap() }

fn field<T: pse_contract::boundary_projection::ProjectionType>(graph: &mut ProjectionGraph, facets: &[(&str, String)]) -> Field {
    Field::of::<T>(graph, "native_field", 7, Direction::Both, vec![]).unwrap()
        .with_named_facets(facets.iter().map(|(key, value)| ((*key).into(), value.clone())).collect())
}

#[test]
fn registry_options_publish_typed_native_facets_and_unresolved_quantity_references() {
    let encoder = encoder();
    let mut graph = ProjectionGraph::default();
    let measure = field::<f64>(&mut graph, &[
        (keys::BOUNDARY_TAG, "7".into()), (keys::ROLE, "measure".into()),
        (keys::DOC, "native documentation".into()), (keys::QUANTITY, "quantity.temperature".into()),
        (keys::EXTENSION, "pse.expr_dsl".into()), (keys::PARAMETER, "predicate".into()),
        (keys::TRANSFER_CONTEXT, "1".into()), (keys::STRUCTURE, "native_name".into()),
    ]);
    let option = field_options(&measure, &graph, &encoder).unwrap().unwrap();
    assert_eq!(enumeration(&option, "role", 6).unwrap(), 3);
    assert_eq!(text(nested(&option, "quantity").unwrap(), "qualified_name").unwrap(), "quantity.temperature");
    assert_eq!(enumeration(nested(&option, "extension").unwrap(), "expression_grammar", 7).unwrap(), 3);
    assert_eq!(text(&option, "documentation").unwrap(), "native documentation");
    check_option(&option).unwrap();

    let collection = CollectionContract { minimum: 2, maximum: Some(5), ..CollectionContract::SET };
    let items = field::<Vec<i64>>(&mut graph, &[(keys::KEY_COLLECTION, collection.canonical())]);
    let option = field_options(&items, &graph, &encoder).unwrap().unwrap();
    assert_eq!(integer(nested(&option, "collection").unwrap(), "maximum").unwrap(), 5);
    assert_eq!(required(nested(&option, "collection").unwrap(), "unique").unwrap(), &Value::Bool(true));
    let range = IntegerRange { minimum: -5, maximum: 23 };
    let count = field::<i64>(&mut graph, &[(keys::KEY_INTEGER_RANGE, range.canonical())]);
    let option = field_options(&count, &graph, &encoder).unwrap().unwrap();
    assert_eq!(integer(nested(&option, "integer_range").unwrap(), "minimum").unwrap(), -5);

    let reference = ReferenceContract { relation: "authored.target".into(), columns: vec![pse_schema::model::ReferenceColumn { source: vec![], target: "id".into() }], null_policy: ReferenceNullPolicy::AllOrNone };
    let id = field::<Option<u64>>(&mut graph, &[
        (keys::FK_RELATION, reference.relation.clone()), (keys::FK_COLUMN, "id".into()),
        (keys::KEY_REFERENCE, reference.canonical().unwrap()), (keys::GRAPH_ENDPOINT, "in".into()),
    ]);
    let option = field_options(&id, &graph, &encoder).unwrap().unwrap();
    assert_eq!(enumeration(nested(&option, "reference").unwrap(), "null_policy", 2).unwrap(), 2);
    check_option(&option).unwrap();
    let alternative = TaggedAlternative { discriminator: "kind".into(), arms: BTreeMap::from([("unit".into(), None), ("data".into(), Some("payload".into()))]) };
    let selected = field::<String>(&mut graph, &[(keys::KEY_TAGGED_ALTERNATIVE, alternative.canonical())]);
    check_option(&field_options(&selected, &graph, &encoder).unwrap().unwrap()).unwrap();
}

projection_struct! {
    struct FacetChild {
        projection: { name: "FacetChild", version: 2, direction: Direction::Both, reserved_tags: [4], reserved_names: ["retired"] };
        count: u64 => { tag: 1, direction: Direction::Both, rules: [] };
    }
}

#[test]
fn registry_options_check_nested_presentation_against_actual_owner_graph() {
    let encoder = encoder();
    let mut graph = ProjectionGraph::of::<FacetChild>().unwrap();
    let owner = &graph.definitions()["FacetChild"];
    let shape = BoundaryShape { source: owner.source.into(), name: Some(owner.name.clone()), version: owner.version,
        direction: owner.direction, reserved_tags: owner.reserved_tags.clone(), reserved_names: owner.reserved_names.clone(), alternatives: vec![] };
    let selected = field::<Option<FacetChild>>(&mut graph, &[(keys::BOUNDARY_SHAPE, shape.canonical())]);
    field_options(&selected, &graph, &encoder).unwrap();
    let mut stale = shape.clone(); stale.version += 1;
    let selected = field::<Option<FacetChild>>(&mut graph, &[(keys::BOUNDARY_SHAPE, stale.canonical())]);
    assert!(field_options(&selected, &graph, &encoder).unwrap_err().to_string().contains(keys::BOUNDARY_SHAPE));
    let mut stale = shape; stale.reserved_tags.clear();
    let selected = field::<Option<FacetChild>>(&mut graph, &[(keys::BOUNDARY_SHAPE, stale.canonical())]);
    assert!(field_options(&selected, &graph, &encoder).is_err());
}

#[test]
fn registry_options_refuse_unrepresented_source_facets_and_invalid_typed_payloads() {
    let encoder = encoder();
    let mut graph = ProjectionGraph::default();
    for (key, text) in [("pse.future.meaning", "x"), (keys::CANONICAL_ASSERT, "true"), (keys::ROLE, "future_role"), (keys::FK_RELATION, "target"), (keys::BOUNDARY_TAG, "9")] {
        let selected = field::<u64>(&mut graph, &[(key, text.into())]);
        let failure = field_options(&selected, &graph, &encoder).unwrap_err().to_string();
        assert!(failure.contains("native_field"), "{failure}");
        assert!(failure.contains(key) || key == keys::ROLE, "{failure}");
    }
    let unknown_version = encoder.message("pse.schema.v1.RegistryFieldFacets", &[("metadata_version", Value::U32(2))]).unwrap();
    assert!(check_option(&unknown_version).is_err());
    let reversed = message(&encoder, "CollectionContract", &[("order", Value::EnumNumber(1)), ("minimum", Value::I64(10)), ("maximum", Value::I64(3)), ("unique", Value::Bool(false))]).unwrap();
    let option = encoder.message("pse.schema.v1.RegistryFieldFacets", &[("metadata_version", Value::U32(1)), ("collection", reversed)]).unwrap();
    assert!(check_option(&option).is_err());
    let empty_extension = message(&encoder, "ExtensionUse", &[]).unwrap();
    let option = encoder.message("pse.schema.v1.RegistryFieldFacets", &[("metadata_version", Value::U32(1)), ("extension", empty_extension)]).unwrap();
    assert!(check_option(&option).is_err());
}

#[test]
fn registry_options_refuse_schema_allocation_and_native_vocabulary_drift() {
    let mut graph = ProjectionGraph::default();
    let selected = field::<u64>(&mut graph, &[(keys::ROLE, "key".into())]);
    let mut imports = super::super::tests::imports();
    let domain = imports.iter_mut().find(|file| file.name.as_deref() == Some(IMPORT)).unwrap();
    domain.extension[0].number = Some(51067);
    let changed = OptionEncoder::new(&imports).unwrap();
    assert!(field_options(&selected, &graph, &changed).unwrap_err().to_string().contains("allocation"));
    let mut imports = super::super::tests::imports();
    let domain = imports.iter_mut().find(|file| file.name.as_deref() == Some(IMPORT)).unwrap();
    let role = domain.enum_type.iter_mut().find(|enumeration| enumeration.name.as_deref() == Some("ColumnRole")).unwrap();
    role.value[1].name = Some("COLUMN_ROLE_FUTURE".into());
    let changed = OptionEncoder::new(&imports).unwrap();
    assert!(field_options(&selected, &graph, &changed).unwrap_err().to_string().contains("vocabulary"));
    for alteration in ["tag", "type", "field", "default"] {
        let mut imports = super::super::tests::imports();
        let domain = imports.iter_mut().find(|file| file.name.as_deref() == Some(IMPORT)).unwrap();
        let identity = domain.message_type.iter_mut().find(|message| message.name.as_deref() == Some("IdentityContract")).unwrap();
        match alteration {
            "tag" => identity.field[0].number = Some(23),
            "type" => identity.field[0].r#type = Some(buffa_descriptor::generated::descriptor::field_descriptor_proto::Type::TYPE_BYTES),
            "field" => { let mut hidden = identity.field[0].clone(); hidden.name = Some("hidden".into()); hidden.json_name = None; hidden.number = Some(23); identity.field.push(hidden); }
            "default" => identity.field[0].default_value = Some("future".into()),
            _ => unreachable!(),
        }
        let changed = OptionEncoder::new(&imports).unwrap();
        assert!(field_options(&selected, &graph, &changed).is_err(), "unknown {alteration} schema semantics must refuse even when that facet is unset");
    }
}

#[test]
fn registry_options_preserve_unresolved_identity_carriers_and_explicit_ownership() {
    let encoder = encoder();
    let mut graph = ProjectionGraph::default();
    let carrier = field::<u64>(&mut graph, &[(keys::IDENTITY, "accuracy_goal".into())]);
    let option = field_options(&carrier, &graph, &encoder).unwrap().unwrap();
    let identity = nested(&option, "identity").unwrap();
    assert_eq!(text(identity, "entity_name").unwrap(), "accuracy_goal");
    assert_eq!(required(identity, "owns_identity").unwrap(), &Value::Bool(false));
    let owner = field::<u64>(&mut graph, &[(keys::IDENTITY, "accuracy_goal".into()), (keys::IDENTITY_OWNER, "true".into())]);
    let option = field_options(&owner, &graph, &encoder).unwrap().unwrap();
    assert_eq!(required(nested(&option, "identity").unwrap(), "owns_identity").unwrap(), &Value::Bool(true));
    let orphan = field::<u64>(&mut graph, &[(keys::IDENTITY_OWNER, "true".into())]);
    assert!(field_options(&orphan, &graph, &encoder).is_err());
    let empty = message(&encoder, "IdentityContract", &[("entity_name", Value::String(String::new())), ("owns_identity", Value::Bool(true))]).unwrap();
    let option = encoder.message("pse.schema.v1.RegistryFieldFacets", &[("metadata_version", Value::U32(1)), ("identity", empty)]).unwrap();
    assert!(check_option(&option).is_err());
}

#[test]
fn registry_options_preserve_canonical_mutability_and_expression_occurrence_policies() {
    let encoder = encoder();
    let mut graph = ProjectionGraph::default();
    for kind in CanonicalKeyKind::ALL {
        let source = field::<String>(&mut graph, &[(keys::CANONICAL_KEY, kind.as_str().into()), (keys::CANONICAL_READONLY, "false".into())]);
        let option = field_options(&source, &graph, &encoder).unwrap().unwrap();
        let policy = nested(&option, "canonical").unwrap();
        assert_eq!(required(policy, "readonly").unwrap(), &Value::Bool(false));
        assert_eq!(required(policy, "record_key").unwrap(), &Value::Bool(false));
        assert_eq!(required(policy, "key_kind").unwrap(), &Value::EnumNumber(canonical_number(option.pool(), *kind).unwrap()));
        check_option(&option).unwrap();
    }
    let key = field::<String>(&mut graph, &[(keys::CANONICAL_KEY, "project_context".into()), (keys::CANONICAL_READONLY, "true".into()), (keys::CANONICAL_RECORD_KEY, "true".into())]);
    let option = field_options(&key, &graph, &encoder).unwrap().unwrap();
    assert_eq!(required(nested(&option, "canonical").unwrap(), "record_key").unwrap(), &Value::Bool(true));
    for (ordinal, number) in [("list", 1), ("zero", 2), ("flatten", 3)] {
        let source = field::<String>(&mut graph, &[(keys::EXPRESSION_ROLE, "scope.fixture.{0}.expression".into()), (keys::EXPRESSION_ORDINAL, ordinal.into())]);
        let option = field_options(&source, &graph, &encoder).unwrap().unwrap();
        let policy = nested(&option, "expression_source").unwrap();
        assert_eq!(text(policy, "role_path_template").unwrap(), "scope.fixture.{0}.expression");
        assert_eq!(required(policy, "ordinal").unwrap(), &Value::EnumNumber(number));
    }
    let source = field::<Vec<String>>(&mut graph, &[(keys::EXPRESSION_BINDER, "domain-before-name".into())]);
    check_option(&field_options(&source, &graph, &encoder).unwrap().unwrap()).unwrap();
    for facets in [vec![(keys::EXPRESSION_ROLE, "missing.ordinal".into())], vec![(keys::EXPRESSION_ORDINAL, "list".into())],
        vec![(keys::EXPRESSION_BINDER, "future".into())], vec![(keys::CANONICAL_KEY, "future".into())],
        vec![(keys::CANONICAL_READONLY, "TRUE".into())]] {
        let source = field::<String>(&mut graph, &facets);
        assert!(field_options(&source, &graph, &encoder).is_err());
    }
    let incomplete = message(&encoder, "ExpressionSourceContract", &[("role_path_template", Value::String("owner.field".into()))]).unwrap();
    let option = encoder.message("pse.schema.v1.RegistryFieldFacets", &[("metadata_version", Value::U32(1)), ("expression_source", incomplete)]).unwrap();
    assert!(check_option(&option).is_err());
}
