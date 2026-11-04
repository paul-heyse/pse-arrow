// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_contract_proto::{OptionEncoder, check_descriptors, compare_descriptors, DescriptorInput};
use pse_contract::metadata::{CorePolicy, Disposition};
use buffa_descriptor::Value;
use buffa::ExtensionSet;
struct QuantityAnnotatedOwner;
impl pse_contract::boundary_projection::ProjectionType for QuantityAnnotatedOwner {
    fn projection(graph: &mut pse_contract::boundary_projection::ProjectionGraph) -> Result<pse_contract::boundary_projection::TypeRef, pse_contract::boundary_projection::ProjectionError> {
        use pse_contract::boundary_projection::{Direction, Field, Shape};
        graph.declare::<Self>("QuantityAnnotatedOwner", file!(), 1, Direction::Both, &[], &[], |graph| Ok(Shape::Record(vec![
            Field::of::<u64>(graph, "temperature", 1, Direction::Both, vec![])?.with_named_facets(std::collections::BTreeMap::from([
                (pse_schema::model::field_facets::ROLE.into(), "measure".into()),
                (pse_schema::model::field_facets::QUANTITY.into(), "quantity.temperature".into()),
            ])),
        ])))
    }
}

#[test]
fn source_publication_refuses_native_ipc_that_contradicts_the_selected_core_type() {
    use pse_contract::boundary_projection::{ProjectionGraph, Direction, Field, Shape};
    struct DeclaredOwner;
    let graph = |data_type| {
        let column = pse_schema::model::FieldContract::payload("count", pse_schema::model::FieldContract::native(data_type), "counter");
        let mut facets = column.field().metadata().iter().map(|(key, value)| (key.clone(), value.clone())).collect::<std::collections::BTreeMap<_, _>>();
        facets.insert(super::super::REGISTRY_FIELD_SOURCE.into(), column.canonical_json().unwrap());
        let mut graph = ProjectionGraph::default();
        graph.declare::<DeclaredOwner>("DeclaredOwner", file!(), 1, Direction::Both, &[], &[], |graph| Ok(Shape::Record(vec![
            Field::of::<u64>(graph, "count", 1, Direction::Both, vec![])?.with_named_facets(facets),
        ]))).unwrap();
        graph
    };
    let imports = super::super::tests::imports();
    super::super::descriptors_with_imports(&graph(arrow_schema::DataType::UInt64), "pse.correspondence.v1", "correspondence.proto", &imports).unwrap();
    let failure = super::super::descriptors_with_imports(&graph(arrow_schema::DataType::Int64), "pse.correspondence.v1", "correspondence.proto", &imports).unwrap_err();
    assert!(failure.to_string().contains("correspondence.proto:pse.correspondence.v1.DeclaredOwner.count"));
    assert!(failure.to_string().contains("contradicts its core"));
}

#[test]
fn physical_facet_reaches_inspection_and_requires_its_selected_owner_policy() {
    use super::super::registry_facets;
    let graph = pse_contract::boundary_projection::ProjectionGraph::of::<QuantityAnnotatedOwner>().unwrap();
    let set = super::super::descriptors_with_imports(&graph, "pse.physical.v1", "physical.proto", &super::super::tests::imports()).unwrap();
    let extensions = [registry_facets::EXTENSION];
    assert_eq!(check_descriptors(&set, "physical.proto", &CorePolicy { interpretations: &[("pse.physical.v1.QuantityAnnotatedOwner", 1)] }, &extensions).unwrap().disposition, Disposition::Unknown);
    let encoder = OptionEncoder::new(&set.file).unwrap();
    let validators: [(&str, DomainCheck); 1] = [(registry_facets::EXTENSION, registry_facets::normalize_field_option)];
    let selection = NativeSelection { owners: &["pse.physical.v1.QuantityAnnotatedOwner"], fields: &["pse.physical.v1.QuantityAnnotatedOwner.temperature"], methods: &[], facets: &[], facet_fields: &[] };
    let policy = TypedPolicy::new(&validators, &[("pse.physical.v1.QuantityAnnotatedOwner", 1)], selection);
    assert_eq!(check_descriptors(&set, "physical.proto", &policy, &extensions).unwrap().disposition, Disposition::Compatible);
    let file = set.file.iter().find(|file| file.name.as_deref() == Some("physical.proto")).unwrap();
    let option = encoder.decode_extension(file.message_type[0].field[0].options.as_option().unwrap(), registry_facets::EXTENSION).unwrap().unwrap();
    let Value::Message(quantity) = option.field_by_number(6).unwrap() else { panic!("quantity reference is typed"); };
    assert_eq!(quantity.field_by_number(1), Some(&Value::String("quantity.temperature".into())));
    let mut before = set.clone();
    let file = before.file.iter_mut().find(|file| file.name.as_deref() == Some("physical.proto")).unwrap();
    file.message_type[0].field[0].options.as_option_mut().unwrap().unknown_fields_mut().retain(|field| field.number != 51066);
    let compared = compare_descriptors(DescriptorInput { descriptors: &before, selected_file: "physical.proto", policy: &policy }, DescriptorInput { descriptors: &set, selected_file: "physical.proto", policy: &policy }, &policy, &extensions).unwrap();
    assert_eq!(compared.disposition, Disposition::Incompatible, "a valid newly introduced physical facet changes existing field meaning");

    // Literal expected source obligations do not come from acquired options.
    let strict = TypedPolicy::new(&validators, &[("pse.physical.v1.QuantityAnnotatedOwner", 1)], NativeSelection {
        facets: &[("pse.physical.v1.QuantityAnnotatedOwner.temperature", registry_facets::EXTENSION)],
        facet_fields: &[("pse.physical.v1.QuantityAnnotatedOwner.temperature", registry_facets::EXTENSION, "quantity")],
        ..selection
    });
    let input = || DescriptorInput { descriptors: &before, selected_file: "physical.proto", policy: &strict };
    assert!(compare_descriptors(input(), input(), &strict, &extensions).is_err(), "stripping the selected extension from both worlds must not vacuously compare");
    let mut missing_fact = set.clone();
    let options = missing_fact.file.iter_mut().find(|file| file.name.as_deref() == Some("physical.proto")).unwrap().message_type[0].field[0].options.as_option_mut().unwrap();
    options.unknown_fields_mut().retain(|field| field.number != 51066);
    let role_only = encoder.message("pse.schema.v1.RegistryFieldFacets", &[("metadata_version", Value::U32(2)), ("role", Value::EnumNumber(3))]).unwrap();
    encoder.attach(options, registry_facets::EXTENSION, role_only).unwrap();
    let input = || DescriptorInput { descriptors: &missing_fact, selected_file: "physical.proto", policy: &strict };
    let refusal = compare_descriptors(input(), input(), &strict, &extensions).unwrap_err();
    assert!(refusal.to_string().contains("physical.proto:pse.physical.v1.QuantityAnnotatedOwner.temperature"));
    assert!(refusal.to_string().contains("required selected domain fact"));
}
