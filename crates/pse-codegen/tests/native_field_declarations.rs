// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native field declarations retain domain meaning at every depth.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "independent contract assertions"
)]

use arrow_schema::{DataType, Field};
use pse_codegen::codegen::{Language, generate};
use pse_schema::{
    Registry, RegistryBuilder, SchemaError, arrow,
    model::{Authority, ColumnRole, FieldContract as F, Namespace, RelationDecl, SnapshotClass},
};
use std::{collections::HashMap, path::Path};

fn fixture(value: F) -> Result<Registry, SchemaError> {
    let mut builder = RegistryBuilder::new();
    builder.declare_enum(pse_schema::model::EnumDecl::platform(
        "BoundKind",
        vec![
            pse_schema::model::EnumMember::new("finite", "finite"),
            pse_schema::model::EnumMember::new("unbounded", "unbounded"),
        ],
    ));
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "native_fields",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "native fixture",
        )
        .pk(&["id"])
        .columns(vec![
            F::key("id", F::id(), "identity"),
            F::payload("value", value, "value"),
        ]),
    );
    builder.build()
}

#[test]
fn authored_foreign_keys_bind_generated_instances_and_domains() {
    let registry = pse_schema::registry().unwrap();
    for (relation, column, target, target_column) in [
        (
            "authored.instance_domain_bindings",
            "instance_id",
            "authored.instances",
            "instance_id",
        ),
        (
            "authored.domain_members",
            "domain_id",
            "authored.domains",
            "domain_id",
        ),
    ] {
        let field = registry.relation(relation).unwrap().column(column).unwrap();
        let reference = field.fk().unwrap();
        assert_eq!(
            (reference.relation, reference.column),
            (target, target_column)
        );
        let native = arrow::field_for(registry, field).unwrap();
        assert_eq!(
            native.metadata().get(arrow::KEY_FK),
            Some(&format!("{target}.{target_column}")),
        );
    }
}

#[test]
fn tagged_arms_project_to_native_fields_and_all_generated_contracts() {
    use pse_schema::model::TaggedAlternative;
    let declaration = TaggedAlternative::new(
        "kind",
        [
            ("number".into(), "number".into()),
            ("label".into(), "label".into()),
        ],
    );
    let value = F::structure(vec![
        F::native(DataType::Utf8).with_name("kind"),
        F::structure(vec![F::nonnegative(10).with_name("value")])
            .with_name("number")
            .optional(),
        F::structure(vec![F::native(DataType::Utf8).with_name("value")])
            .with_name("label")
            .optional(),
    ])
    .with_alternative(&declaration);
    let registry = fixture(value.clone()).unwrap();
    let relation = registry.relation("authored.native_fields").unwrap();
    let execution = arrow::relation_schema(&registry, relation).unwrap();
    let storage = pse_schema::delta::relation_schema(&registry, relation).unwrap();
    assert_eq!(
        pse_schema::delta::execution_schema(&storage).unwrap(),
        execution
    );
    assert_eq!(
        TaggedAlternative::from_field(execution.field(1)).unwrap(),
        Some(declaration)
    );
    let rust = generate(&registry, Language::Rust).unwrap();
    assert_shared_codec(&rust);
    let rust = std::str::from_utf8(
        &rust.files[Path::new("crates/pse-model/src/generated/authored/native_fields.rs")],
    )
    .unwrap();
    assert!(rust.contains("pub enum AuthoredNativeFieldsFieldValueSelected"));
    assert!(rust.contains("pub fn from_number"));
    let python = generate(&registry, Language::Python).unwrap();
    let python =
        std::str::from_utf8(&python.files[Path::new("python/pse/contracts/authored.py")]).unwrap();
    assert!(python.contains("def __attrs_post_init__"));
    let docs = generate(&registry, Language::Markdown).unwrap();
    let schema: serde_json::Value = serde_json::from_slice(
        &docs.files[Path::new("docs/generated/schema/authoring.schema.json")],
    )
    .unwrap();
    let arms =
        &schema["$defs"]["authored.native_fields"]["properties"]["value"]["allOf"][1]["oneOf"];
    assert_eq!(arms.as_array().unwrap().len(), 2);
    let mut malformed = value.field().clone();
    malformed.metadata_mut().insert(
        pse_schema::model::tagged_alternative::KEY_TAGGED_ALTERNATIVE.into(),
        "{}".into(),
    );
    assert!(fixture(F::from_field(malformed)).is_err());
    let duplicate = TaggedAlternative::new(
        "kind",
        [
            ("number".into(), "number".into()),
            ("label".into(), "number".into()),
        ],
    );
    assert!(fixture(value.with_alternative(&duplicate)).is_err());
}

#[test]
fn unit_and_shared_payload_tags_have_one_declared_shape() {
    use pse_schema::model::TaggedAlternative;
    let declaration = TaggedAlternative::new(
        "kind",
        [
            ("first".into(), "payload".into()),
            ("second".into(), "payload".into()),
        ],
    )
    .with_unit("absent");
    // Fingerprints must not depend on serde_json's feature-unified map representation.
    assert_eq!(
        declaration.canonical(),
        r#"{"arms":{"absent":null,"first":"payload","second":"payload"},"discriminator":"kind"}"#
    );
    let value = F::structure(vec![
        F::native(DataType::Utf8).with_name("kind"),
        F::structure(vec![F::nonnegative(10).with_name("value")])
            .with_name("payload")
            .optional(),
    ])
    .with_alternative(&declaration);
    let registry = fixture(value.clone()).unwrap();
    assert_eq!(
        TaggedAlternative::from_field(value.field()).unwrap(),
        Some(declaration)
    );
    // Value selection is checked by the shared prepared-validator units; schema
    // admission owns the exact native tagged declaration, not a second scalar checker.
    let rust = generate(&registry, Language::Rust).unwrap();
    assert_shared_codec(&rust);
    let rust = std::str::from_utf8(
        &rust.files[Path::new("crates/pse-model/src/generated/authored/native_fields.rs")],
    )
    .unwrap();
    for constructor in ["from_first", "from_second", "from_absent"] {
        assert!(rust.contains(constructor));
    }
    syn::parse_file(rust).unwrap();
    let docs = generate(&registry, Language::Markdown).unwrap();
    let json: serde_json::Value = serde_json::from_slice(
        &docs.files[Path::new("docs/generated/schema/authoring.schema.json")],
    )
    .unwrap();
    let variants =
        json["$defs"]["authored.native_fields"]["properties"]["value"]["allOf"][1]["oneOf"]
            .as_array()
            .unwrap();
    assert_eq!(variants.len(), 3);
    assert_eq!(variants[0]["required"], serde_json::json!(["kind"]));
}

#[test]
fn entirely_payload_free_alternatives_do_not_generate_a_borrow_lifetime() {
    use pse_schema::model::TaggedAlternative;
    let declaration = TaggedAlternative::new("kind", [])
        .with_unit("enabled")
        .with_unit("disabled");
    let value = F::structure(vec![F::native(DataType::Utf8).with_name("kind")])
        .with_alternative(&declaration);
    let registry = fixture(value).unwrap();
    let rust = generate(&registry, Language::Rust).unwrap();
    assert_shared_codec(&rust);
    let rust = std::str::from_utf8(
        &rust.files[Path::new("crates/pse-model/src/generated/authored/native_fields.rs")],
    )
    .unwrap();
    assert!(rust.contains("pub enum AuthoredNativeFieldsFieldValueSelected {"));
    assert!(!rust.contains("AuthoredNativeFieldsFieldValueSelected<'a>"));
    syn::parse_file(rust).unwrap();
    let python = generate(&registry, Language::Python).unwrap();
    let python =
        std::str::from_utf8(&python.files[Path::new("python/pse/contracts/authored.py")]).unwrap();
    assert!(!python.contains(" and )"));
}

#[test]
fn collection_facets_preserve_empty_order_and_unique_meanings_across_projections() {
    use pse_schema::model::{CollectionContract, CollectionOrder};
    let contract = CollectionContract {
        minimum: 1,
        maximum: Some(3),
        ..CollectionContract::SET
    };
    let field = F::list(F::nonnegative(10).optional()).with_collection(contract);
    let registry = fixture(field.clone()).unwrap();
    assert_eq!(
        CollectionContract::from_field(field.field()).unwrap(),
        Some(contract)
    );
    assert!(contract.accepts(&[Some(2), None, Some(1)]));
    assert!(!contract.accepts::<i64>(&[]));
    assert!(!contract.accepts(&[None::<i64>, None]));
    assert!(!contract.accepts(&[1, 2, 3, 4]));
    assert!(CollectionContract::SEQUENCE.accepts(&[1, 1]));
    let list = F::list(F::nonnegative(10));
    assert_eq!(
        CollectionContract::from_field(list.field())
            .unwrap()
            .unwrap()
            .order,
        CollectionOrder::Sequence
    );
    assert!(fixture(F::native(DataType::Int64).with_collection(contract)).is_err());
    assert!(fixture(F::fixed_list(F::id(), 4).with_collection(contract)).is_err());
    assert!(
        fixture(field.with_collection(CollectionContract {
            minimum: -1,
            ..contract
        }))
        .is_err()
    );
    let rust = generate(&registry, Language::Rust).unwrap();
    assert_shared_codec(&rust);
    let source = std::str::from_utf8(
        &rust.files[Path::new("crates/pse-model/src/generated/authored/native_fields.rs")],
    )
    .unwrap();
    syn::parse_file(source).unwrap();
    let docs = generate(&registry, Language::Markdown).unwrap();
    let json: serde_json::Value = serde_json::from_slice(
        &docs.files[Path::new("docs/generated/schema/authoring.schema.json")],
    )
    .unwrap();
    let schema = &json["$defs"]["authored.native_fields"]["properties"]["value"];
    assert_eq!(schema["uniqueItems"], true);
    assert_eq!(schema["minItems"], 1);
    assert_eq!(schema["maxItems"], 3);
    assert_eq!(schema["x-pse-order"], "unordered");
}

#[test]
fn string_enums_keep_their_domain_in_execution_and_durable_fields() {
    let registry = fixture(F::extended(pse_schema::model::ExtensionUse::Bound)).unwrap();
    let relation = registry.relation("authored.native_fields").unwrap();
    let execution = arrow::relation_schema(&registry, relation).unwrap();
    let storage = pse_schema::delta::relation_schema(&registry, relation).unwrap();
    for schema in [&execution, &storage] {
        let DataType::Struct(fields) = schema.field(1).data_type() else {
            panic!("bound structure");
        };
        let kind = &fields[0];
        assert_eq!(kind.data_type(), &DataType::Utf8);
        assert_eq!(kind.metadata()[arrow::KEY_EXTENSION_NAME], "pse.enum");
        assert_eq!(kind.metadata()[arrow::KEY_LOGICAL_TYPE], "enum:BoundKind");
        assert_eq!(
            kind.metadata()[arrow::KEY_ENUM],
            registry.enum_spec("BoundKind").unwrap().id.to_hex()
        );
    }
    let field = arrow::field_for(&registry, &F::enumeration("BoundKind")).unwrap();
    let old_storage = field.clone().with_data_type(DataType::Dictionary(
        Box::new(DataType::Int32),
        Box::new(DataType::Utf8),
    ));
    assert!(
        pse_schema::field_contract::declaration(&arrow_schema::Schema::new(vec![old_storage]))
            .is_err()
    );
    let contract = registry.contract(relation).unwrap();
    assert!(!contract.resolved().enums.is_empty());
    assert_eq!(
        contract.resolved().fields[0].field(),
        &arrow::field_for(&registry, &relation.columns[0]).unwrap()
    );
}

#[test]
fn bounded_signed_domains_project_to_languages_and_durable_storage() {
    use pse_schema::model::IntegerRange;
    let registry = fixture(F::list(F::nonnegative(255).optional())).unwrap();
    let relation = registry.relation("authored.native_fields").unwrap();
    let execution = arrow::relation_schema(&registry, relation).unwrap();
    let storage = pse_schema::delta::relation_schema(&registry, relation).unwrap();
    let reconstructed = pse_schema::delta::execution_schema(&storage).unwrap();
    let DataType::List(stored_element) = storage.field(1).data_type() else {
        panic!("list")
    };
    assert_eq!(stored_element.data_type(), &DataType::Int64);
    for schema in [&execution, &reconstructed] {
        let DataType::List(child) = schema.field(1).data_type() else {
            panic!("list")
        };
        assert_eq!(child.data_type(), &DataType::Int64);
        assert_eq!(
            IntegerRange::from_field(child).unwrap(),
            Some(IntegerRange::nonnegative(255))
        );
    }
    let rust = generate(&registry, Language::Rust).unwrap();
    assert_shared_codec(&rust);
    let rust = std::str::from_utf8(
        &rust.files[Path::new("crates/pse-model/src/generated/authored/native_fields.rs")],
    )
    .unwrap();
    assert!(rust.contains("Vec<Option<i64>>"));
    let python = generate(&registry, Language::Python).unwrap();
    assert!(python.files.values().any(|value| {
        std::str::from_utf8(value)
            .unwrap()
            .contains("v.integer_range(0, 255)")
    }));
    let markdown = generate(&registry, Language::Markdown).unwrap();
    let json: serde_json::Value = serde_json::from_slice(
        &markdown.files[Path::new("docs/generated/schema/authoring.schema.json")],
    )
    .unwrap();
    let item = &json["$defs"]["authored.native_fields"]["properties"]["value"]["items"]["anyOf"][0];
    assert_eq!(item["minimum"], 0);
    assert_eq!(item["maximum"], 255);
    assert_ne!(
        registry.fingerprint(),
        fixture(F::list(F::nonnegative(256).optional()))
            .unwrap()
            .fingerprint()
    );
}

#[test]
fn malformed_integer_domains_and_unsigned_predecessor_extensions_refuse() {
    use pse_schema::model::{IntegerRange, integer_range::KEY_INTEGER_RANGE};
    for text in ["[1,0]", "[0, 255]", "[0]", "[0,9223372036854775808]"] {
        let mut field = IntegerRange::nonnegative(255).field("value");
        field
            .metadata_mut()
            .insert(KEY_INTEGER_RANGE.into(), text.into());
        assert!(fixture(F::from_field(field)).is_err(), "{text}");
    }
    let field = IntegerRange::nonnegative(255)
        .field("value")
        .with_data_type(DataType::UInt64);
    assert!(fixture(F::from_field(field)).is_err());
    let registry = fixture(F::id()).unwrap();
    let declaration = F::extended(pse_schema::model::ExtensionUse::OrdinalRef {
        target: "authored.native_fields",
    });
    let field = arrow::field_for(&registry, &declaration).unwrap();
    assert_eq!(field.data_type(), &DataType::Int64);
    let mut missing = field.clone();
    missing.metadata_mut().remove(KEY_INTEGER_RANGE);
    for field in [missing, field.with_data_type(DataType::UInt64)] {
        assert!(
            pse_schema::field_contract::declaration(&arrow_schema::Schema::new(vec![field]))
                .is_err()
        );
    }
}

#[test]
fn nested_reference_quantity_role_and_absence_project_to_native_fields() {
    let value = F::structure(vec![
        F::list(
            F::reference("member", F::id(), "referenced identity")
                .with_fk("authored.native_fields", "id")
                .optional(),
        )
        .with_name("members"),
        F::new(
            "temperature",
            F::native(DataType::Float64),
            true,
            ColumnRole::Measure,
            "physical measurement",
        )
        .with_quantity("temperature"),
    ]);
    let reg = fixture(value.clone()).unwrap();
    let schema =
        arrow::relation_schema(&reg, reg.relation("authored.native_fields").unwrap()).unwrap();
    let DataType::Struct(fields) = schema.field_with_name("value").unwrap().data_type() else {
        panic!("struct");
    };
    let DataType::List(member) = fields[0].data_type() else {
        panic!("list");
    };
    assert_eq!(member.name(), "member");
    assert!(member.is_nullable());
    assert_eq!(member.metadata()[arrow::KEY_ROLE], "reference");
    assert_eq!(
        member.metadata()[arrow::KEY_FK],
        "authored.native_fields.id"
    );
    assert_eq!(
        member.metadata()[arrow::KEY_EXTENSION_NAME],
        "pse.semantic_id"
    );
    assert_eq!(
        fields[1].metadata()[arrow::KEY_QUANTITY_TYPE],
        pse_schema::builder::quantity_type_id("temperature").to_hex()
    );
    assert_eq!(fields[1].metadata()[arrow::KEY_ROLE], "measure");
    let saved: Field = serde_json::from_str(&value.canonical_json().unwrap()).unwrap();
    assert_eq!(&saved, value.field());
    assert_eq!(
        F::from_field(saved).children()[0].children()[0]
            .fk()
            .unwrap()
            .relation,
        "authored.native_fields"
    );
}

#[test]
fn nested_references_and_invalid_domain_facets_fail_registry_admission() {
    let bad = F::list(F::structure(vec![
        F::reference("missing", F::id(), "reference").with_fk("authored.absent", "id"),
    ]));
    assert!(
        fixture(bad)
            .unwrap_err()
            .to_string()
            .contains("authored.absent")
    );
    let bad = F::structure(vec![
        F::reference("wrong_type", F::native(DataType::Int64), "reference")
            .with_fk("authored.native_fields", "id"),
    ]);
    assert!(fixture(bad).is_err());
    for metadata in [
        HashMap::from([("pse.domain.role".into(), "unknown".into())]),
        HashMap::from([(
            "pse.domain.fk.relation".into(),
            "authored.native_fields".into(),
        )]),
        HashMap::from([("pse.domain.extension".into(), "pse.misspelled".into())]),
    ] {
        assert!(
            fixture(F::list(F::from_field(
                Field::new("item", DataType::Int64, false).with_metadata(metadata)
            )))
            .is_err()
        );
    }
}

#[test]
fn engine_eligibility_uses_native_arrow_types_and_keeps_external_annotations() {
    for data_type in [
        DataType::Float16,
        DataType::Decimal256(50, 4),
        DataType::Date32,
        DataType::LargeUtf8,
        DataType::FixedSizeList(Field::new("item", DataType::Boolean, false).into(), 0),
    ] {
        let native =
            Field::new("value", data_type.clone(), false).with_metadata(HashMap::from([(
                "external.meaning".into(),
                "measure".into(),
            )]));
        let reg = fixture(F::from_field(native)).unwrap();
        let schema =
            arrow::relation_schema(&reg, reg.relation("authored.native_fields").unwrap()).unwrap();
        let field = schema.field_with_name("value").unwrap();
        assert_eq!(
            field.data_type(),
            &arrow::bind_type(&reg, &data_type, "value").unwrap()
        );
        assert_eq!(field.metadata()["external.meaning"], "measure");
        pse_schema::field_contract::declaration(&schema).unwrap();
    }
}

#[test]
fn metadata_participates_in_both_registry_and_relation_fingerprints() {
    let build = |meaning: &str| {
        fixture(F::from_field(
            Field::new("value", DataType::Int64, false)
                .with_metadata(HashMap::from([("external.meaning".into(), meaning.into())])),
        ))
        .unwrap()
    };
    let a = build("count");
    let same = build("count");
    let b = build("duration");
    assert_eq!(a.fingerprint(), same.fingerprint());
    assert_ne!(a.fingerprint(), b.fingerprint());
    assert_ne!(
        a.relation("authored.native_fields").unwrap().fingerprint,
        b.relation("authored.native_fields").unwrap().fingerprint
    );
}

#[test]
fn native_dictionary_ordering_participates_in_declaration_equality_and_collections() {
    let plain = F::from_field(Field::new_dictionary(
        "value",
        DataType::Int32,
        DataType::Utf8,
        false,
    ));
    let ordered = F::from_field(plain.field().clone().with_dict_is_ordered(true));
    for (plain, ordered) in [
        (plain.clone(), ordered.clone()),
        (F::list(plain.clone()), F::list(ordered.clone())),
        (F::structure(vec![plain]), F::structure(vec![ordered])),
    ] {
        assert_ne!(plain, ordered);
        assert_eq!(
            std::collections::BTreeSet::from([plain.clone(), ordered.clone()]).len(),
            2
        );
        assert_eq!(std::collections::HashSet::from([plain, ordered]).len(), 2);
    }
}

#[test]
fn derived_metadata_cannot_replace_the_domain_declaration() {
    for (key, value) in [
        (arrow::KEY_FK, "authored.native_fields.id"),
        (arrow::KEY_QUANTITY_TYPE, "00000000000000000000000000000000"),
        (arrow::KEY_ENUM, "00000000000000000000000000000000"),
        (arrow::KEY_ROLE, "reference"),
        (arrow::KEY_EXTENSION_NAME, "pse.semantic_id"),
    ] {
        let value = F::from_field(
            Field::new("value", DataType::Int64, false)
                .with_metadata(HashMap::from([(key.into(), value.into())])),
        );
        assert!(fixture(F::list(value)).is_err(), "{key}");
    }
    let external = F::from_field(Field::new("value", DataType::Int64, false).with_metadata(
        HashMap::from([
            (arrow::KEY_EXTENSION_NAME.into(), "example.counter".into()),
            (arrow::KEY_EXTENSION_METADATA.into(), "{\"v\":1}".into()),
        ]),
    ));
    fixture(F::list(external)).unwrap();
}

#[test]
fn generated_language_views_preserve_nullable_list_elements() {
    let reg = fixture(F::list(F::id().optional())).unwrap();
    let rust = generate(&reg, Language::Rust).unwrap();
    assert_shared_codec(&rust);
    let rust = std::str::from_utf8(
        &rust.files[Path::new("crates/pse-model/src/generated/authored/native_fields.rs")],
    )
    .unwrap();
    assert!(rust.contains("Vec<Option<pse_ids::SemanticId>>"), "{rust}");
    let python = generate(&reg, Language::Python).unwrap();
    let contents = python
        .files
        .values()
        .map(|bytes| std::str::from_utf8(bytes).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(contents.contains("b.tuple[v.SemanticId | None, ...]"));
    let markdown = generate(&reg, Language::Markdown).unwrap();
    let json: serde_json::Value = serde_json::from_slice(
        &markdown.files[Path::new("docs/generated/schema/authoring.schema.json")],
    )
    .unwrap();
    assert_eq!(
        json["$defs"]["authored.native_fields"]["properties"]["value"]["items"]["anyOf"][1]["type"],
        "null"
    );
}

#[test]
fn native_binary_and_dictionary_do_not_acquire_undeclared_domain_meaning() {
    for data_type in [
        DataType::FixedSizeBinary(16),
        DataType::FixedSizeBinary(32),
        DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8)),
    ] {
        let reg = fixture(F::native(data_type.clone())).unwrap();
        let schema =
            arrow::relation_schema(&reg, reg.relation("authored.native_fields").unwrap()).unwrap();
        let field = schema.field_with_name("value").unwrap();
        assert_eq!(
            field.data_type(),
            &arrow::bind_type(&reg, &data_type, "value").unwrap()
        );
        assert!(!field.metadata().contains_key(arrow::KEY_EXTENSION_NAME));
        for language in [Language::Rust, Language::Python, Language::Markdown] {
            assert!(
                generate(&reg, language)
                    .unwrap_err()
                    .to_string()
                    .contains("explicit declaration")
            );
        }
    }
}

fn assert_shared_codec(tree: &pse_codegen::codegen::GeneratedTree) {
    let codec = std::str::from_utf8(
        &tree.files[Path::new("crates/pse-relations/src/generated/authored/native_fields.rs")],
    )
    .unwrap();
    let syntax = syn::parse_file(codec).unwrap();
    let alias = syntax
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Type(alias) if alias.ident == "AuthoredNativeFieldsBuilder" => Some(alias),
            _ => None,
        })
        .unwrap();
    let syn::Type::Path(kind) = alias.ty.as_ref() else {
        panic!("shared builder type")
    };
    assert_eq!(
        kind.path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>(),
        ["crate", "columnar", "RowBuilder"]
    );
    let syn::PathArguments::AngleBracketed(arguments) =
        &kind.path.segments.last().unwrap().arguments
    else {
        panic!("row argument")
    };
    assert_eq!(arguments.args.len(), 1);
    let syn::GenericArgument::Type(syn::Type::Path(row)) = arguments.args.first().unwrap() else {
        panic!("semantic row type")
    };
    assert_eq!(row.path.get_ident().unwrap(), "AuthoredNativeFieldsRow");
    assert!(codec.contains("pub use pse_model::generated::"));
    assert!(!codec.contains("pub struct AuthoredNativeFieldsRow"));
}
