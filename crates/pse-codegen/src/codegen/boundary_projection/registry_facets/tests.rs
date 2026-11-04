// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use pse_contract::boundary_projection::{Direction, projection_struct};
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
    for (key, text) in [("pse.future.meaning", "x"), (keys::CANONICAL_KEY, "x"), (keys::ROLE, "future_role"), (keys::FK_RELATION, "target"), (keys::BOUNDARY_TAG, "9")] {
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
