// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Package parsing exercises generated contracts, actual identities and original spans.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixture assertions"
)]

use pse_authoring::{
    ParseBudget,
    document::{load_package, load_package_texts},
};
use pse_relations::generated::authored;
use std::collections::BTreeMap;
mod support;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/packages")
        .join(name)
}
fn texts(name: &str) -> BTreeMap<String, String> {
    let loaded = load_package(
        &fixture(name),
        pse_schema::registry().unwrap(),
        ParseBudget::default(),
    )
    .unwrap();
    loaded
        .documents
        .iter()
        .map(|document| (document.path.clone(), document.text.clone()))
        .collect()
}

#[test]
fn explicit_and_named_fixtures_decode_generated_rows_and_original_source_spans() {
    let registry_owner = support::registry();
    let registry = registry_owner.as_ref();
    let explicit = load_package(
        &fixture("minimal_explicit"),
        registry,
        ParseBudget::default(),
    )
    .unwrap();
    let species = authored::species::Row::from_cells(
        explicit.decode_rows(registry).unwrap()[&authored::species::RELATION_ID][0].clone(),
    )
    .unwrap();
    assert_eq!(species.package_id, explicit.package.package_id);
    assert_eq!(species.name, "water");
    let entity = explicit.decode_rows(registry).unwrap()[&authored::entities::RELATION_ID]
        .iter()
        .cloned()
        .map(authored::entities::Row::from_cells)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .into_iter()
        .find(|entity| entity.entity_id == species.species_id)
        .unwrap();
    assert_eq!(entity.entity_id, species.species_id);
    assert_eq!(entity.qualified_name, "minimal.water");
    let span = entity.source_span.unwrap();
    let document = explicit
        .documents
        .iter()
        .find(|document| document.id == span.document_id)
        .unwrap();
    let original =
        &document.text[usize::try_from(span.start).unwrap()..usize::try_from(span.end).unwrap()];
    assert!(
        original.contains("name: water"),
        "row span: {original:?}; {span:?}"
    );
    assert!(original.contains("component_type: Component"));
    let named = load_package(&fixture("minimal_named"), registry, ParseBudget::default()).unwrap();
    let entity = authored::entities::Row::from_cells(
        named.decode_rows(registry).unwrap()[&authored::entities::RELATION_ID][0].clone(),
    )
    .unwrap();
    assert_eq!(
        entity.entity_id,
        pse_ids::named_id(named.package.package_id, "minimal_named.H")
    );
    assert_eq!(entity.name, "H");
}

#[test]
fn missing_ids_unknown_fields_duplicate_keys_and_wrong_package_context_fail() {
    let registry_owner = support::registry();
    let registry = registry_owner.as_ref();
    let original = texts("minimal_explicit");
    let document = &original["materials/species.yaml"];
    let variants = [
        document.replace(
            "id: \"01991d6a-13a0-7000-8000-000000000002\"",
            "unused: true",
        ),
        document.replace("name: water", "name: water\n    typo: value"),
        document.replace("name: water", "name: water\n    name: duplicate"),
        document.replace(
            "name: water",
            "name: water\n    package_id: '00000000000000000000000000000000'",
        ),
        document.replace(
            "name: water",
            "name: water\n    species_id: '00000000000000000000000000000000'",
        ),
    ];
    for text in variants {
        let mut inputs = original.clone();
        inputs.insert("materials/species.yaml".to_owned(), text);
        assert!(load_package_texts(inputs, registry, ParseBudget::default()).is_err());
    }
}

#[test]
fn nested_named_identities_use_the_actual_declared_parent_independent_of_file_order() {
    let registry_owner = support::registry();
    let registry = registry_owner.as_ref();
    let mut inputs = texts("minimal_named");
    let parent = pse_ids::named_id(
        pse_authoring::ids::parse_id(
            "01991d6a-13a0-7000-8000-000000000003",
            pse_authoring::SourceSpan::head(pse_ids::SemanticId::NIL),
        )
        .unwrap(),
        "minimal_named.heater",
    );
    inputs.insert(
        "templates/z_parent.yaml".to_owned(),
        "templates:\n  - name: heater\n    version: '1.0.0'\n    kind: unit\n    doc: Heater.\n"
            .to_owned(),
    );
    inputs.insert("templates/a_child.yaml".to_owned(), format!("template_symbols:\n  - template_id: '{parent}'\n    name: temperature\n    role: variable\n    quantity_type_id: '00000000000000000000000000000001'\n    indexed_by: []\n    doc: Temperature.\n"));
    let bundle = load_package_texts(inputs.clone(), registry, ParseBudget::default()).unwrap();
    let entities = bundle.decode_rows(registry).unwrap()[&authored::entities::RELATION_ID]
        .iter()
        .cloned()
        .map(authored::entities::Row::from_cells)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let child = entities
        .iter()
        .find(|entity| entity.name == "temperature")
        .unwrap();
    assert_eq!(child.parent_entity_id, Some(parent));
    assert_eq!(child.qualified_name, "minimal_named.heater.temperature");
    inputs.remove("templates/z_parent.yaml");
    assert!(load_package_texts(inputs, registry, ParseBudget::default()).is_err());
}

#[test]
fn composite_relationships_never_accept_an_entity_id_alias() {
    let registry_owner = support::registry();
    let registry = registry_owner.as_ref();
    let mut inputs = texts("minimal_explicit");
    inputs.insert("materials/bad.yaml".to_owned(), "phase_species:\n  - id: '00000000000000000000000000000001'\n    phase_id: '00000000000000000000000000000002'\n    species_id: '00000000000000000000000000000003'\n".to_owned());
    assert!(load_package_texts(inputs, registry, ParseBudget::default()).is_err());
}

#[tokio::test]
async fn declared_field_grammar_preserves_atomic_guards_and_refuses_equations_in_values() {
    let registry_owner = support::registry();
    let registry = registry_owner.as_ref();
    let mut inputs = texts("minimal_explicit");
    let template = "00000000000000000000000000000031";
    let source = format!(
        "templates:\n  - id: '{template}'\n    name: model\n    version: '1.0.0'\n    kind: unit\n    doc: ''\ntemplate_guards:\n  - id: '00000000000000000000000000000032'\n    template_id: '{template}'\n    predicate: 'true'\n    doc: ''\ntemplate_display:\n  - template_id: '{template}'\n    kind: expression\n    label: value\n    expression: '2 + 3'\n"
    );
    inputs.insert("templates/grammar.yaml".to_owned(), source.clone());
    let bundle = load_package_texts(inputs.clone(), registry, ParseBudget::default()).unwrap();
    let budget = pse_ids::FixedBudget::new(512 << 20);
    let session = support::session(std::sync::Arc::clone(&registry_owner), budget.clone());
    let cancel = pse_ids::CancellationToken::new();
    let owned =
        pse_authoring::document::load_bundles_owned(&[bundle], registry, budget.as_ref(), &cancel)
            .unwrap();
    let batches = pse_authoring::p1::source_batches(owned.bundles(), registry).unwrap();
    let bound =
        pse_authoring::document::binding::bind_sources_owned(&owned, &batches, &session, &cancel)
            .await
            .unwrap();
    assert!(bound.expressions().iter().any(|source| matches!(
        source.parsed,
        pse_authoring::document::binding::ParsedExpression::Predicate(_)
    )));
    inputs.insert(
        "templates/grammar.yaml".to_owned(),
        source.replace("2 + 3", "2 == 3"),
    );
    assert!(load_package_texts(inputs, registry, ParseBudget::default()).is_err());
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "one source fixture covers known, unknown and deferred enum predicates"
)]
async fn enum_predicates_bind_exact_known_members_and_explicit_deferred_types() {
    use pse_authoring::document::binding::{PathMeaning, bind_sources_owned};
    let registry = support::registry();
    let material = registry
        .enums()
        .iter()
        .find(|value| value.name == "MaterialBalanceType")
        .unwrap()
        .id;
    let id_type = registry.logical_type("semantic_id").unwrap().id;
    let owner = "00000000000000000000000000000031";
    let child = "00000000000000000000000000000032";
    let declaration = format!(
        r"
templates:
  - id: '{owner}'
    name: parent
    version: '1.0.0'
    kind: unit
    doc: ''
  - id: '{child}'
    name: child
    version: '1.0.0'
    kind: state_block
    doc: ''
template_params:
  - template_id: '{owner}'
    name: selected_template
    logical_type_id: '{id_type}'
    enum_id: null
    default: null
    required: true
    domain_spec: null
    doc: ''
template_features:
  - template_id: '{child}'
    name: enabled
    kind: bool
    enum_id: null
    default: 'false'
    inherit_from: null
    doc: ''
  - template_id: '{child}'
    name: policy
    kind: enum
    enum_id: '{material}'
    default: componentTotal
    inherit_from: null
    doc: ''
  - template_id: '{owner}'
    name: enabled
    kind: bool
    enum_id: null
    default: 'true'
    inherit_from: null
    doc: ''
template_submodels:
  - template_id: '{owner}'
    name: known
    child_template_id: '{child}'
    child_from_param: null
    multiplicity_domain: null
    bindings:
      - child_param: enabled
        value: parent.enabled
    guard_id: null
  - template_id: '{owner}'
    name: selected
    child_template_id: null
    child_from_param: selected_template
    multiplicity_domain: null
    bindings:
      - child_param: enabled
        value: 'true'
    guard_id: null
template_guards:
  - id: '00000000000000000000000000000033'
    template_id: '{owner}'
    predicate: 'PREDICATE'
    doc: ''
"
    );
    for (predicate, admitted) in [
        ("known.policy == componentTotal", true),
        (
            "selected.policy == MaterialBalanceType.componentTotal",
            true,
        ),
        ("selected.policy == componentTotal", false),
        ("known.policy == EnergyBalanceType.enthalpyTotal", false),
        ("selected.policy == MaterialBalanceType.missing", false),
        ("enabled == MaterialBalanceType.componentTotal", false),
        (
            "selected.policy < MaterialBalanceType.componentTotal",
            false,
        ),
    ] {
        let mut inputs = texts("minimal_explicit");
        inputs.insert(
            "templates/enums.yaml".to_owned(),
            declaration.replace("PREDICATE", predicate),
        );
        let bundle = load_package_texts(inputs, &registry, ParseBudget::default()).unwrap();
        let budget = pse_ids::FixedBudget::new(128 << 20);
        let session = support::session(registry.clone(), budget.clone());
        let cancel = pse_ids::CancellationToken::new();
        let documents = pse_authoring::document::load_bundles_owned(
            &[bundle],
            &registry,
            budget.as_ref(),
            &cancel,
        )
        .unwrap();
        let batches = pse_authoring::p1::source_batches(documents.bundles(), &registry).unwrap();
        let result = bind_sources_owned(&documents, &batches, &session, &cancel).await;
        assert_eq!(
            result.is_ok(),
            admitted,
            "{predicate}: {}",
            result
                .as_ref()
                .err()
                .map(ToString::to_string)
                .unwrap_or_default()
        );
        if let Ok(bound) = result {
            assert!(
                bound
                    .expressions()
                    .iter()
                    .flat_map(|expression| &expression.paths)
                    .any(|path| matches!(path.meaning, PathMeaning::BooleanLiteral(true)))
            );
            assert!(bound.expressions().iter().flat_map(|expression| &expression.paths).any(|path| {
                matches!(&path.meaning, PathMeaning::EnumLiteral { enum_id, member } if *enum_id == material && member == "componentTotal")
            }));
        }
    }
}
