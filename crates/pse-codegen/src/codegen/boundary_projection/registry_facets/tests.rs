// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use pse_contract::{boundary_projection::Direction, projection_struct};
use std::collections::BTreeMap;

fn encoder() -> OptionEncoder {
    OptionEncoder::new(&super::super::tests::imports()).unwrap()
}

fn field<T: pse_contract::boundary_projection::ProjectionType>(
    graph: &mut ProjectionGraph,
    facets: &[(&str, String)],
) -> Field {
    Field::of::<T>(graph, "native_field", 7, Direction::Both, vec![])
        .unwrap()
        .with_named_facets(
            facets
                .iter()
                .map(|(key, value)| ((*key).into(), value.clone()))
                .collect(),
        )
}

#[test]
fn registry_options_publish_typed_native_facets_and_unresolved_quantity_references() {
    let encoder = encoder();
    let mut graph = ProjectionGraph::default();
    let measure = field::<f64>(
        &mut graph,
        &[
            (keys::BOUNDARY_TAG, "7".into()),
            (keys::ROLE, "measure".into()),
            (keys::DOC, "native documentation".into()),
            (keys::QUANTITY, "quantity.temperature".into()),
            (keys::EXTENSION, "pse.expr_dsl".into()),
            (keys::PARAMETER, "predicate".into()),
            (keys::TRANSFER_CONTEXT, "1".into()),
            (keys::STRUCTURE, "native_name".into()),
        ],
    );
    let option = field_options(&measure, &graph, &encoder).unwrap().unwrap();
    assert_eq!(enumeration(&option, "role", 6).unwrap(), 3);
    assert_eq!(
        text(nested(&option, "quantity").unwrap(), "qualified_name").unwrap(),
        "quantity.temperature"
    );
    assert_eq!(
        enumeration(
            nested(&option, "extension").unwrap(),
            "expression_grammar",
            7
        )
        .unwrap(),
        3
    );
    assert_eq!(
        text(&option, "documentation").unwrap(),
        "native documentation"
    );
    check_option(&option).unwrap();

    let collection = CollectionContract {
        minimum: 2,
        maximum: Some(5),
        ..CollectionContract::SET
    };
    let items = field::<Vec<i64>>(
        &mut graph,
        &[(keys::KEY_COLLECTION, collection.canonical())],
    );
    let option = field_options(&items, &graph, &encoder).unwrap().unwrap();
    assert_eq!(
        integer(nested(&option, "collection").unwrap(), "maximum").unwrap(),
        5
    );
    assert_eq!(
        required(nested(&option, "collection").unwrap(), "unique").unwrap(),
        &Value::Bool(true)
    );
    let range = IntegerRange {
        minimum: -5,
        maximum: 23,
    };
    let count = field::<i64>(&mut graph, &[(keys::KEY_INTEGER_RANGE, range.canonical())]);
    let option = field_options(&count, &graph, &encoder).unwrap().unwrap();
    assert_eq!(
        integer(nested(&option, "integer_range").unwrap(), "minimum").unwrap(),
        -5
    );

    let reference = ReferenceContract {
        relation: "authored.target".into(),
        columns: vec![pse_schema::model::ReferenceColumn {
            source: vec![],
            target: "id".into(),
        }],
        null_policy: ReferenceNullPolicy::AllOrNone,
    };
    let id = field::<Option<u64>>(
        &mut graph,
        &[
            (keys::FK_RELATION, reference.relation.clone()),
            (keys::FK_COLUMN, "id".into()),
            (keys::KEY_REFERENCE, reference.canonical().unwrap()),
            (keys::GRAPH_ENDPOINT, "in".into()),
        ],
    );
    let option = field_options(&id, &graph, &encoder).unwrap().unwrap();
    assert_eq!(
        enumeration(nested(&option, "reference").unwrap(), "null_policy", 2).unwrap(),
        2
    );
    check_option(&option).unwrap();
    let alternative = TaggedAlternative {
        discriminator: "kind".into(),
        arms: BTreeMap::from([
            ("unit".into(), None),
            ("data".into(), Some("payload".into())),
        ]),
    };
    let selected = field::<String>(
        &mut graph,
        &[(keys::KEY_TAGGED_ALTERNATIVE, alternative.canonical())],
    );
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
    let shape = BoundaryShape {
        source: owner.source.into(),
        name: Some(owner.name.clone()),
        version: owner.version,
        direction: owner.direction,
        reserved_tags: owner.reserved_tags.clone(),
        reserved_names: owner.reserved_names.clone(),
        alternatives: vec![],
    };
    let selected =
        field::<Option<FacetChild>>(&mut graph, &[(keys::BOUNDARY_SHAPE, shape.canonical())]);
    field_options(&selected, &graph, &encoder).unwrap();
    let mut stale = shape.clone();
    stale.version += 1;
    let selected =
        field::<Option<FacetChild>>(&mut graph, &[(keys::BOUNDARY_SHAPE, stale.canonical())]);
    assert!(
        field_options(&selected, &graph, &encoder)
            .unwrap_err()
            .to_string()
            .contains(keys::BOUNDARY_SHAPE)
    );
    let mut stale = shape;
    stale.reserved_tags.clear();
    let selected =
        field::<Option<FacetChild>>(&mut graph, &[(keys::BOUNDARY_SHAPE, stale.canonical())]);
    assert!(field_options(&selected, &graph, &encoder).is_err());
}

#[test]
fn registry_options_refuse_unrepresented_source_facets_and_invalid_typed_payloads() {
    let encoder = encoder();
    let mut graph = ProjectionGraph::default();
    for (key, text) in [
        ("pse.future.meaning", "x"),
        (keys::ROLE, "future_role"),
        (keys::FK_RELATION, "target"),
        (keys::BOUNDARY_TAG, "9"),
    ] {
        let selected = field::<u64>(&mut graph, &[(key, text.into())]);
        let failure = field_options(&selected, &graph, &encoder)
            .unwrap_err()
            .to_string();
        assert!(failure.contains("native_field"), "{failure}");
        assert!(failure.contains(key) || key == keys::ROLE, "{failure}");
    }
    let unknown_version = encoder
        .message(
            "pse.schema.v1.RegistryFieldFacets",
            &[("metadata_version", Value::U32(99))],
        )
        .unwrap();
    assert!(check_option(&unknown_version).is_err());
    let reversed = message(
        &encoder,
        "CollectionContract",
        &[
            ("order", Value::EnumNumber(1)),
            ("minimum", Value::I64(10)),
            ("maximum", Value::I64(3)),
            ("unique", Value::Bool(false)),
        ],
    )
    .unwrap();
    let option = encoder
        .message(
            "pse.schema.v1.RegistryFieldFacets",
            &[
                ("metadata_version", Value::U32(2)),
                ("collection", reversed),
            ],
        )
        .unwrap();
    assert!(check_option(&option).is_err());
    let empty_extension = message(&encoder, "ExtensionUse", &[]).unwrap();
    let option = encoder
        .message(
            "pse.schema.v1.RegistryFieldFacets",
            &[
                ("metadata_version", Value::U32(2)),
                ("extension", empty_extension),
            ],
        )
        .unwrap();
    assert!(check_option(&option).is_err());
}

#[test]
fn registry_options_refuse_schema_allocation_and_native_vocabulary_drift() {
    let mut graph = ProjectionGraph::default();
    let selected = field::<u64>(&mut graph, &[(keys::ROLE, "key".into())]);
    let mut imports = super::super::tests::imports();
    let domain = imports
        .iter_mut()
        .find(|file| file.name.as_deref() == Some(IMPORT))
        .unwrap();
    domain.extension[0].number = Some(51067);
    let changed = OptionEncoder::new(&imports).unwrap();
    assert!(
        field_options(&selected, &graph, &changed)
            .unwrap_err()
            .to_string()
            .contains("allocation")
    );
    let mut imports = super::super::tests::imports();
    let domain = imports
        .iter_mut()
        .find(|file| file.name.as_deref() == Some(IMPORT))
        .unwrap();
    let role = domain
        .enum_type
        .iter_mut()
        .find(|enumeration| enumeration.name.as_deref() == Some("ColumnRole"))
        .unwrap();
    role.value[1].name = Some("COLUMN_ROLE_FUTURE".into());
    let changed = OptionEncoder::new(&imports).unwrap();
    assert!(
        field_options(&selected, &graph, &changed)
            .unwrap_err()
            .to_string()
            .contains("vocabulary")
    );
    for alteration in ["tag", "type", "field", "default"] {
        let mut imports = super::super::tests::imports();
        let domain = imports
            .iter_mut()
            .find(|file| file.name.as_deref() == Some(IMPORT))
            .unwrap();
        let identity = domain
            .message_type
            .iter_mut()
            .find(|message| message.name.as_deref() == Some("IdentityContract"))
            .unwrap();
        match alteration {
            "tag" => identity.field[0].number = Some(23),
            "type" => identity.field[0].r#type = Some(
                buffa_descriptor::generated::descriptor::field_descriptor_proto::Type::TYPE_BYTES,
            ),
            "field" => {
                let mut hidden = identity.field[0].clone();
                hidden.name = Some("hidden".into());
                hidden.json_name = None;
                hidden.number = Some(23);
                identity.field.push(hidden);
            }
            "default" => identity.field[0].default_value = Some("future".into()),
            _ => panic!("unknown registry facet alteration fixture: {alteration}"),
        }
        let changed = OptionEncoder::new(&imports).unwrap();
        assert!(
            field_options(&selected, &graph, &changed).is_err(),
            "unknown {alteration} schema semantics must refuse even when that facet is unset"
        );
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
    assert_eq!(
        required(identity, "owns_identity").unwrap(),
        &Value::Bool(false)
    );
    let owner = field::<u64>(
        &mut graph,
        &[
            (keys::IDENTITY, "accuracy_goal".into()),
            (keys::IDENTITY_OWNER, "true".into()),
        ],
    );
    let option = field_options(&owner, &graph, &encoder).unwrap().unwrap();
    assert_eq!(
        required(nested(&option, "identity").unwrap(), "owns_identity").unwrap(),
        &Value::Bool(true)
    );
    let orphan = field::<u64>(&mut graph, &[(keys::IDENTITY_OWNER, "true".into())]);
    assert!(field_options(&orphan, &graph, &encoder).is_err());
    let empty = message(
        &encoder,
        "IdentityContract",
        &[
            ("entity_name", Value::String(String::new())),
            ("owns_identity", Value::Bool(true)),
        ],
    )
    .unwrap();
    let option = encoder
        .message(
            "pse.schema.v1.RegistryFieldFacets",
            &[("metadata_version", Value::U32(2)), ("identity", empty)],
        )
        .unwrap();
    assert!(check_option(&option).is_err());
}

#[test]
fn registry_options_preserve_canonical_mutability_and_expression_occurrence_policies() {
    let encoder = encoder();
    let mut graph = ProjectionGraph::default();
    for kind in CanonicalKeyKind::ALL {
        let source = field::<String>(
            &mut graph,
            &[
                (keys::CANONICAL_KEY, kind.as_str().into()),
                (keys::CANONICAL_READONLY, "false".into()),
            ],
        );
        let option = field_options(&source, &graph, &encoder).unwrap().unwrap();
        let policy = nested(&option, "canonical").unwrap();
        assert_eq!(required(policy, "readonly").unwrap(), &Value::Bool(false));
        assert_eq!(required(policy, "record_key").unwrap(), &Value::Bool(false));
        assert_eq!(
            required(policy, "key_kind").unwrap(),
            &Value::EnumNumber(canonical_number(option.pool(), *kind).unwrap())
        );
        check_option(&option).unwrap();
    }
    let key = field::<String>(
        &mut graph,
        &[
            (keys::CANONICAL_KEY, "project_context".into()),
            (keys::CANONICAL_READONLY, "true".into()),
            (keys::CANONICAL_RECORD_KEY, "true".into()),
        ],
    );
    let option = field_options(&key, &graph, &encoder).unwrap().unwrap();
    assert_eq!(
        required(nested(&option, "canonical").unwrap(), "record_key").unwrap(),
        &Value::Bool(true)
    );
    for (ordinal, number) in [("list", 1), ("zero", 2), ("flatten", 3)] {
        let source = field::<String>(
            &mut graph,
            &[
                (keys::EXPRESSION_ROLE, "scope.fixture.{0}.expression".into()),
                (keys::EXPRESSION_ORDINAL, ordinal.into()),
            ],
        );
        let option = field_options(&source, &graph, &encoder).unwrap().unwrap();
        let policy = nested(&option, "expression_source").unwrap();
        assert_eq!(
            text(policy, "role_path_template").unwrap(),
            "scope.fixture.{0}.expression"
        );
        assert_eq!(
            required(policy, "ordinal").unwrap(),
            &Value::EnumNumber(number)
        );
    }
    let source = field::<Vec<String>>(
        &mut graph,
        &[(keys::EXPRESSION_BINDER, "domain-before-name".into())],
    );
    check_option(&field_options(&source, &graph, &encoder).unwrap().unwrap()).unwrap();
    for facets in [
        vec![(keys::EXPRESSION_ROLE, "missing.ordinal".into())],
        vec![(keys::EXPRESSION_ORDINAL, "list".into())],
        vec![(keys::EXPRESSION_BINDER, "future".into())],
        vec![(keys::CANONICAL_KEY, "future".into())],
        vec![(keys::CANONICAL_READONLY, "TRUE".into())],
    ] {
        let source = field::<String>(&mut graph, &facets);
        assert!(field_options(&source, &graph, &encoder).is_err());
    }
    let incomplete = message(
        &encoder,
        "ExpressionSourceContract",
        &[("role_path_template", Value::String("owner.field".into()))],
    )
    .unwrap();
    let option = encoder
        .message(
            "pse.schema.v1.RegistryFieldFacets",
            &[
                ("metadata_version", Value::U32(2)),
                ("expression_source", incomplete),
            ],
        )
        .unwrap();
    assert!(check_option(&option).is_err());
}

#[test]
fn selected_purpose_preserves_native_storage_and_filters_only_declared_provenance() {
    use arrow_schema::{DataType, TimeUnit};
    let encoder = encoder();
    let mut graph = ProjectionGraph::default();
    let native = |kind, documentation: &str| {
        arrow_schema::Field::new("native_field", kind, true).with_metadata(
            std::collections::HashMap::from([
                (keys::ROLE.into(), "measure".into()),
                (keys::DOC.into(), documentation.into()),
                (keys::BOUNDARY_TAG.into(), "7".into()),
            ]),
        )
    };
    let source = |graph: &mut ProjectionGraph, native: &arrow_schema::Field| {
        let mut metadata = native
            .metadata()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<BTreeMap<_, _>>();
        metadata.insert(
            super::super::REGISTRY_FIELD_SOURCE.into(),
            pse_schema::model::field::render_field(native).unwrap(),
        );
        Field::of::<Option<i64>>(graph, "native_field", 7, Direction::Both, vec![])
            .unwrap()
            .with_named_facets(metadata)
    };
    let field_a = native(
        DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
        "before",
    );
    let a = field_options(&source(&mut graph, &field_a), &graph, &encoder)
        .unwrap()
        .unwrap();
    let field_b = native(
        DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
        "after",
    );
    let b = field_options(&source(&mut graph, &field_b), &graph, &encoder)
        .unwrap()
        .unwrap();
    assert_ne!(
        a.try_encode_to_vec().unwrap(),
        b.try_encode_to_vec().unwrap(),
        "full original carrier retains prose"
    );
    assert_eq!(
        normalize_option(&a).unwrap(),
        normalize_option(&b).unwrap(),
        "named execution/publication purpose excludes source prose only"
    );
    let plain = native(DataType::Int64, "before");
    let plain = field_options(&source(&mut graph, &plain), &graph, &encoder)
        .unwrap()
        .unwrap();
    assert_ne!(
        normalize_option(&a).unwrap(),
        normalize_option(&plain).unwrap(),
        "timestamp micros+UTC must not collapse to native Rust i64"
    );
    let nanos = native(
        DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())),
        "before",
    );
    let nanos = field_options(&source(&mut graph, &nanos), &graph, &encoder)
        .unwrap()
        .unwrap();
    assert_ne!(
        normalize_option(&a).unwrap(),
        normalize_option(&nanos).unwrap()
    );
    let mut mismatched = source(&mut graph, &field_a);
    mismatched
        .named_facets
        .insert(keys::ROLE.into(), "label".into());
    assert!(
        field_options(&mismatched, &graph, &encoder).is_err(),
        "typed facets cannot disagree with the native IPC field metadata"
    );

    let doc_only = field::<String>(&mut graph, &[(keys::DOC, "retained prose".into())]);
    let option = field_options(&doc_only, &graph, &encoder).unwrap().unwrap();
    assert_eq!(
        normalize_option(&option).unwrap(),
        None,
        "prose-only introduction has no semantic facts under this named purpose"
    );
}

#[test]
fn canonical_publication_policy_uses_native_parser_and_inert_ast_normalization() {
    let encoder = encoder();
    let mut graph = ProjectionGraph::default();
    let option = |graph: &mut ProjectionGraph, expression: &str| {
        let selected = field::<u64>(
            graph,
            &[
                (keys::CANONICAL_ASSERT, expression.into()),
                (keys::CANONICAL_DEFAULT, "1+2".into()),
            ],
        );
        field_options(&selected, graph, &encoder).unwrap().unwrap()
    };
    let a = option(&mut graph, "$value>0");
    let b = option(&mut graph, " $value > 0 ");
    let c = option(&mut graph, "$value>1");
    assert_eq!(normalize_option(&a).unwrap(), normalize_option(&b).unwrap());
    assert_ne!(
        normalize_option(&a).unwrap(),
        normalize_option(&c).unwrap(),
        "canonical policy remains meaningful although Arrow execution projection excludes it"
    );
    for text in ["$value > 0 trailing", ""] {
        let selected = field::<u64>(&mut graph, &[(keys::CANONICAL_ASSERT, text.into())]);
        let failure = field_options(&selected, &graph, &encoder).unwrap_err();
        let mut cause: Option<&dyn std::error::Error> = Some(&failure);
        let mut original = false;
        while let Some(value) = cause {
            original |= value
                .downcast_ref::<pse_schema::model::CanonicalExpressionParseError>()
                .is_some();
            cause = value.source();
        }
        assert!(
            original,
            "native source parser cause must survive option publication"
        );
    }
}

#[test]
fn acquired_native_references_preserve_nullability_and_empty_component_paths() {
    let encoder = encoder();
    let mut graph = ProjectionGraph::default();
    for nullable in [false, true] {
        let native = FieldContract::id()
            .with_name("native_field")
            .with_fk("authored.packages", "package_id");
        let native = if nullable { native.optional() } else { native };
        let reference = ReferenceContract::for_contract(&native).unwrap().unwrap();
        let mut facets = native
            .field()
            .metadata()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<BTreeMap<_, _>>();
        facets.insert(keys::KEY_REFERENCE.into(), reference.canonical().unwrap());
        // Raw original IPC deliberately has the scalar convenience form, while
        // the typed option has the native owner's normalized null policy.
        facets.insert(
            super::super::REGISTRY_FIELD_SOURCE.into(),
            native.canonical_json().unwrap(),
        );
        let selected = if nullable {
            field::<Option<pse_ids::SemanticId>>(&mut graph, &[])
        } else {
            field::<pse_ids::SemanticId>(&mut graph, &[])
        }
        .with_named_facets(facets);
        let option = field_options(&selected, &graph, &encoder).unwrap().unwrap();
        let mut wire = buffa_descriptor::generated::descriptor::FieldOptions::default();
        encoder
            .attach(&mut wire, EXTENSION, option.clone())
            .unwrap();
        let acquired = encoder.decode_extension(&wire, EXTENSION).unwrap().unwrap();
        check_option(&acquired).unwrap();
        assert_eq!(
            normalize_option(&option).unwrap(),
            normalize_option(&acquired).unwrap()
        );
        assert_eq!(
            enumeration(nested(&acquired, "reference").unwrap(), "null_policy", 2).unwrap(),
            if nullable { 2 } else { 1 }
        );
        let mut explicit = native.field().clone();
        explicit.metadata_mut().remove(keys::FK_RELATION);
        explicit.metadata_mut().remove(keys::FK_COLUMN);
        let explicit = reference.annotate(explicit).unwrap();
        assert_eq!(
            semantic_native_field(native.field()).unwrap(),
            semantic_native_field(&explicit).unwrap(),
            "convenience and correlated spelling denote one source-owned reference meaning"
        );
        let mut corrupted = selected.clone();
        let mut wrong = reference;
        wrong.null_policy = if nullable {
            ReferenceNullPolicy::Required
        } else {
            ReferenceNullPolicy::AllOrNone
        };
        corrupted
            .named_facets
            .insert(keys::KEY_REFERENCE.into(), wrong.canonical().unwrap());
        assert!(
            field_options(&corrupted, &graph, &encoder).is_err(),
            "a contradictory null policy is not equivalent spelling"
        );
    }
}

#[test]
fn acquired_native_collection_references_keep_correlated_components() {
    let reference = ReferenceContract {
        relation: "authored.components".into(),
        columns: vec![
            pse_schema::model::ReferenceColumn {
                source: vec!["first".into()],
                target: "left".into(),
            },
            pse_schema::model::ReferenceColumn {
                source: vec!["second".into()],
                target: "right".into(),
            },
        ],
        null_policy: ReferenceNullPolicy::AllOrNone,
    };
    let child = FieldContract::structure(vec![
        FieldContract::id().with_name("first").optional(),
        FieldContract::id().with_name("second").optional(),
    ])
    .with_name("item")
    .with_reference(&reference)
    .unwrap();
    let native = FieldContract::list(child).with_name("native_field");
    let bytes = native_ipc::encode(native.field()).unwrap();
    let acquired = native_ipc::decode(&bytes).unwrap();
    assert_eq!(
        semantic_native_field(native.field()).unwrap(),
        semantic_native_field(&acquired).unwrap()
    );
    let arrow_schema::DataType::List(child) = acquired.data_type() else {
        panic!("list");
    };
    assert_eq!(native_reference(child).unwrap(), Some(reference.clone()));
    let mut wrong = reference;
    wrong.columns[1].target = "other".into();
    let changed = wrong.annotate(child.as_ref().clone()).unwrap();
    let changed = acquired
        .clone()
        .with_data_type(arrow_schema::DataType::List(std::sync::Arc::new(changed)));
    assert_ne!(
        semantic_native_field(&acquired).unwrap(),
        semantic_native_field(&changed).unwrap(),
        "changing either correlated target remains semantic"
    );
}
