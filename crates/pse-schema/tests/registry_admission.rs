// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "assertions report fixture failures directly"
)]

//! Semantic admission and dependency checks precede every registry fingerprint.

use pse_schema::builder::RegistryBuilder;
use pse_schema::model::{
    AlgorithmDecl, ArgumentSpec, Authority, Cell, ColumnRole, DependencyMode, Determinism,
    DocumentKind, DocumentSection, DocumentSpec, EnumDecl, EnumMember, ExtensionUse, FieldContract,
    InvariantDecl, InvariantKind, MigrationSpec, MigrationStep, Namespace, RelationDecl,
    ResultSpec, RuleDecl, RuleInput, RuleQuery, SnapshotClass,
};
use pse_schema::{Registry, SchemaError};

fn relation(name: &'static str, version: u32, extra: Vec<FieldContract>) -> RelationDecl {
    let mut columns = vec![FieldContract::key("id", FieldContract::id(), "identity")];
    columns.extend(extra);
    RelationDecl::new(
        Namespace::Authored,
        name,
        version,
        Authority::Authored,
        SnapshotClass::Model,
        "fixture",
    )
    .pk(&["id"])
    .columns(columns)
}

fn registry_with(decl: RelationDecl) -> Result<Registry, SchemaError> {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(decl);
    builder.build()
}

#[test]
fn singleton_key_is_explicit_and_distinct_from_an_omitted_key() {
    let declaration = relation("singleton", 1, vec![]).pk(&[]);
    let registry = registry_with(declaration.clone()).unwrap();
    assert!(
        registry
            .relation("authored.singleton")
            .unwrap()
            .primary_key
            .is_empty()
    );
    let mut omitted = declaration;
    omitted.primary_key = None;
    assert!(
        registry_with(omitted)
            .unwrap_err()
            .to_string()
            .contains("explicit primary key")
    );
}

#[test]
fn native_sql_checks_are_part_of_exact_contract_identity() {
    let declaration = relation("checked", 1, vec![]);
    let old = registry_with(declaration.clone()).unwrap();
    let new = registry_with(declaration.checks(std::collections::BTreeMap::from([(
        "value".into(),
        "id IS NOT NULL".into(),
    )])))
    .unwrap();
    let left = old.relation("authored.checked").unwrap();
    let right = new.relation("authored.checked").unwrap();
    assert_eq!(left.id, right.id);
    assert_ne!(left.fingerprint, right.fingerprint);
    assert_ne!(old.fingerprint(), new.fingerprint());
    let schema = pse_schema::arrow::relation_schema(&new, right).unwrap();
    assert_eq!(
        pse_schema::arrow::native_checks(&schema).unwrap(),
        right.checks
    );
    assert!(
        registry_with(
            relation("checked", 1, vec![]).checks(std::collections::BTreeMap::from([(
                "empty".into(),
                " ".into()
            ),]))
        )
        .is_err()
    );
}

fn rule(version: &'static str, sql: &str) -> RuleDecl {
    RuleDecl::new(
        "fixture",
        version,
        0,
        "inferred.output",
        sql,
        vec![RuleInput {
            relation: "authored.base".into(),
            port: "base",
            mode: DependencyMode::Read,
        }],
    )
}

fn rules(rules: Vec<RuleDecl>) -> Result<Registry, SchemaError> {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(relation("base", 1, vec![]));
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Inferred,
            "output",
            1,
            Authority::Derived,
            SnapshotClass::Model,
            "Native query output",
        )
        .granularity(pse_schema::model::DerivationGranularity::Row)
        .pk(&["id"])
        .columns(vec![FieldContract::key(
            "id",
            FieldContract::id(),
            "identity",
        )]),
    );
    for rule in rules {
        builder.declare_rule(rule);
    }
    builder.build()
}

#[test]
fn native_rule_declarations_require_outcomes_and_resolved_scopes() {
    let base = rule("1", "SELECT id FROM authored.base");
    assert!(rules(vec![base.clone()]).is_ok());
    let mut invalid = Vec::new();
    let mut empty = base.clone();
    empty.queries.clear();
    invalid.push(empty);
    for (truth, sql) in [("true", " "), ("conflict", "SELECT id FROM authored.base")] {
        let mut query = base.clone();
        query.queries = vec![RuleQuery {
            truth,
            sql: sql.into(),
        }];
        invalid.push(query);
    }
    let mut missing = base.clone();
    missing.inputs[0].relation = "authored.absent".into();
    invalid.push(missing);
    let mut write_input = base.clone();
    write_input.inputs[0].mode = DependencyMode::Write;
    invalid.push(write_input);
    for head in ["authored.base", "inferred.absent"] {
        let mut wrong_head = base.clone();
        wrong_head.head = head.into();
        invalid.push(wrong_head);
    }
    let mut ambiguous = base;
    ambiguous.inputs.push(RuleInput {
        relation: "inferred.output".into(),
        port: "base",
        mode: DependencyMode::Read,
    });
    invalid.push(ambiguous);
    for declaration in invalid {
        assert!(rules(vec![declaration]).is_err());
    }
}

fn rows(registry: &Registry, name: &str) -> Vec<Vec<Cell>> {
    registry
        .schema_rows()
        .into_iter()
        .find(|(key, _)| key.name == name)
        .unwrap()
        .1
}

#[test]
fn malformed_keys_and_duplicate_columns_are_rejected_before_identity() {
    for decl in [
        relation("a", 1, vec![]).pk(&["id", "id"]),
        relation("a", 1, vec![]).columns(vec![
            FieldContract::key("id", FieldContract::id(), "id").optional(),
        ]),
        relation("a", 1, vec![]).columns(vec![FieldContract::key(
            "id",
            FieldContract::native(arrow_schema::DataType::Float64),
            "float",
        )]),
        relation(
            "a",
            1,
            vec![FieldContract::key("id", FieldContract::id(), "duplicate")],
        ),
        relation(
            "a",
            1,
            vec![FieldContract::label(
                "payload",
                FieldContract::fixed_list(
                    FieldContract::native(arrow_schema::DataType::UInt64),
                    -1,
                ),
                "bad width",
            )],
        ),
        relation(
            "a",
            1,
            vec![FieldContract::label(
                "payload",
                FieldContract::structure(vec![
                    FieldContract::native(arrow_schema::DataType::UInt64)
                        .with_name("x")
                        .with_nullable(false),
                    FieldContract::native(arrow_schema::DataType::UInt64)
                        .with_name("x")
                        .with_nullable(false),
                ]),
                "duplicate fields",
            )],
        ),
    ] {
        assert!(registry_with(decl).is_err());
    }
}

#[test]
fn duplicate_enum_members_and_document_sections_are_rejected() {
    let mut builder = RegistryBuilder::new();
    builder.declare_enum(EnumDecl::platform(
        "Choice",
        vec![
            EnumMember::new("one", "one"),
            EnumMember::new("one", "duplicate"),
        ],
    ));
    assert!(matches!(
        builder.build(),
        Err(SchemaError::DuplicateDeclaration {
            kind: "enum member",
            ..
        })
    ));
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(relation("base", 1, vec![]));
    let section = DocumentSection {
        key: "items",
        relation: "authored.base",
        repeated: true,
        identity_column: None,
        entity_kind: None,
        name_column: None,
        naming_scope_column: None,
        expression_owner_column: None,
        expression_owner_kind: None,
        expression_fields: &[],
        doc: "items",
    };
    builder.declare_document(DocumentSpec {
        name: "fixture",
        kind: DocumentKind::Entities,
        path_glob: "*.yaml",
        sections: vec![section, section],
        doc: "fixture",
    });
    assert!(matches!(
        builder.build(),
        Err(SchemaError::DuplicateDeclaration {
            kind: "document section",
            ..
        })
    ));
}

#[test]
fn foreign_key_types_must_match() {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(relation("target", 1, vec![]));
    builder.declare_relation(relation(
        "source",
        1,
        vec![
            FieldContract::reference(
                "target_id",
                FieldContract::native(arrow_schema::DataType::UInt64),
                "wrong identity type",
            )
            .with_fk("authored.target", "id"),
        ],
    ));
    assert!(matches!(
        builder.build(),
        Err(SchemaError::InvalidDeclaration { .. })
    ));
}

#[test]
fn resolved_ordinal_target_is_present_in_the_consuming_contract() {
    let build = |target_version| {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(relation("target", target_version, vec![]));
        builder.declare_relation(relation(
            "source",
            1,
            vec![FieldContract::reference(
                "target",
                FieldContract::extended(ExtensionUse::OrdinalRef {
                    target: "authored.target",
                }),
                "ordinal",
            )],
        ));
        builder.build().unwrap()
    };
    let before = build(1);
    let after = build(2);
    let target = after.relation("authored.target").unwrap();
    let metadata = after
        .logical_type("ordinal_ref:authored.target")
        .unwrap()
        .metadata_schema
        .as_ref()
        .unwrap();
    assert!(metadata.contains(&format!("\"const\":\"{}\"", target.id.to_hex())));
    assert_ne!(
        before.relation("authored.source").unwrap().fingerprint,
        after.relation("authored.source").unwrap().fingerprint
    );
}

#[test]
fn arbitrary_registries_project_complete_declared_integrity_programs() {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(relation("target", 1, vec![]));
    builder.declare_relation(relation(
        "source",
        1,
        vec![
            FieldContract::reference("target_id", FieldContract::id(), "Foreign key")
                .with_fk("authored.target", "id")
                .optional(),
            FieldContract::payload(
                "nested",
                FieldContract::structure(vec![
                    FieldContract::list(FieldContract::extended(ExtensionUse::OrdinalRef {
                        target: "authored.target",
                    }))
                    .with_name("items")
                    .with_nullable(true),
                ]),
                "Nested ordinal values",
            )
            .optional(),
        ],
    ));
    let registry = builder.build().unwrap();
    let names = registry
        .invariants()
        .iter()
        .map(pse_schema::model::InvariantSpec::qualified_name)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        names,
        std::collections::BTreeSet::from([
            "authored.target:unique:pk".to_owned(),
            "authored.source:unique:pk".to_owned(),
            "authored.source:foreign_key:target_id".to_owned(),
            "authored.source:ordinal_range:nested.items[]".to_owned(),
        ])
    );
    let ordinal = registry
        .invariants()
        .iter()
        .find(|invariant| {
            invariant.qualified_name() == "authored.source:ordinal_range:nested.items[]"
        })
        .unwrap();
    assert!(ordinal.inputs.contains(&"authored.source".to_owned()));
    assert!(ordinal.inputs.contains(&"authored.target".to_owned()));
}

#[test]
fn a_conflicting_manual_integrity_projection_cannot_replace_the_declaration() {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(relation("base", 1, vec![]));
    builder.declare_invariant(InvariantDecl::error(
        "authored.base",
        "unique:pk",
        InvariantKind::Unique,
        "SELECT id FROM authored.base WHERE false",
        vec!["authored.base".into()],
        vec!["id"],
        "Conflicting manual uniqueness declaration",
    ));
    assert!(matches!(
        builder.build(),
        Err(SchemaError::DuplicateDeclaration { .. })
    ));
}

#[test]
fn native_query_text_and_outcomes_are_reflected_and_change_identity() {
    let before = rules(vec![rule("1", "SELECT id FROM authored.base WHERE true")]).unwrap();
    let after = rules(vec![rule("1", "SELECT id FROM authored.base WHERE false")]).unwrap();
    assert_eq!(
        rows(&before, "rule_specs")[0][5],
        Cell::List(vec![Cell::Struct(vec![
            Cell::text("true"),
            Cell::text("SELECT id FROM authored.base WHERE true"),
        ])])
    );
    assert_ne!(
        rows(&before, "rule_specs")[0][5],
        rows(&after, "rule_specs")[0][5]
    );
    assert_ne!(before.fingerprint(), after.fingerprint());
    let mut unknown = rule("1", "SELECT id FROM authored.base WHERE true");
    unknown.queries[0].truth = "unknown";
    let unknown = rules(vec![unknown]).unwrap();
    assert_ne!(before.fingerprint(), unknown.fingerprint());
    assert_eq!(
        unknown.rule("fixture@1").unwrap().queries[0].truth,
        "unknown"
    );
}

#[test]
fn rule_versions_have_exact_read_and_write_dependencies_and_unambiguous_names() {
    let registry = rules(vec![
        rule("1", "SELECT id FROM authored.base WHERE true"),
        rule("2", "SELECT id FROM authored.base WHERE false"),
    ])
    .unwrap();
    assert!(registry.rule("fixture").is_none());
    let source = registry.relation("authored.base").unwrap().id;
    let output = registry.relation("inferred.output").unwrap().id;
    for version in ["fixture@1", "fixture@2"] {
        let id = registry.rule(version).unwrap().id;
        let dependencies = registry
            .rule_dependencies()
            .iter()
            .filter(|dependency| dependency.rule_id == id)
            .collect::<Vec<_>>();
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.iter().any(|dep| dep.relation_id == source
            && dep.mode == DependencyMode::Read
            && dep.input_port == Some("base")));
        assert!(dependencies.iter().any(|dep| dep.relation_id == output
            && dep.mode == DependencyMode::Write
            && dep.input_port.is_none()));
    }
    assert_ne!(
        registry.rule("fixture@1").unwrap().id,
        registry.rule("fixture@2").unwrap().id
    );
}

#[test]
fn migration_defaults_are_lossless_and_type_checked() {
    let build = |default| {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(relation("base", 1, vec![]));
        builder.declare_relation(relation(
            "base",
            2,
            vec![FieldContract::new(
                "value",
                FieldContract::native(arrow_schema::DataType::Float64),
                false,
                ColumnRole::Measure,
                "value",
            )],
        ));
        builder.declare_migration(MigrationSpec {
            relation: "authored.base",
            from_version: 1,
            to_version: 2,
            steps: vec![MigrationStep::AddColumn {
                name: "value",
                default,
            }],
            doc: "add value",
        });
        builder.build()
    };
    let positive = build(Cell::F64(0.0)).unwrap();
    let negative = build(Cell::F64(-0.0)).unwrap();
    assert!(
        negative.migrations()[0]
            .plan_spec()
            .contains("8000000000000000")
    );
    assert_ne!(positive.fingerprint(), negative.fingerprint());
    assert!(build(Cell::text("not a float")).is_err());
    let nested = Cell::List(vec![Cell::Struct(vec![
        Cell::text("quote\"\n"),
        Cell::Enum("a"),
        Cell::F64(f64::from_bits(0x7ff8_0000_0000_0001)),
    ])]);
    assert!(nested.literal_spec().contains("7ff8000000000001"));
    assert!(nested.literal_spec().contains("quote\\\"\\n"));
}

#[test]
fn algorithm_signatures_check_local_arguments_without_a_stored_producer_graph() {
    let build = |input: &str, duplicate: bool| {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(relation("base", 1, vec![]));
        let argument = ArgumentSpec {
            port: "input".into(),
            relation: input.into(),
            required: true,
            consumption: pse_schema::model::algorithm::InputConsumption::Whole,
        };
        builder.declare_algorithm(
            AlgorithmDecl::new("domain", "1", Determinism::Deterministic)
                .inputs(if duplicate {
                    vec![argument.clone(), argument]
                } else {
                    vec![argument]
                })
                .outputs(vec![ResultSpec {
                    port: "result".into(),
                    relation: "authored.base".into(),
                }]),
        );
        builder.build()
    };
    let valid = build("authored.base", false).unwrap();
    assert_eq!(valid.algorithms().len(), 1);
    assert_eq!(
        valid.algorithms()[0].effects,
        [pse_schema::model::provider::OperationEffect::Read]
            .into_iter()
            .collect()
    );
    assert!(matches!(
        build("authored.missing", false),
        Err(SchemaError::UnknownReference { .. })
    ));
    assert!(matches!(
        build("authored.base", true),
        Err(SchemaError::DuplicateDeclaration { .. })
    ));
}

#[test]
fn document_identity_projection_is_admitted_and_fingerprinted() {
    fn builder() -> RegistryBuilder {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "things",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "Things.",
            )
            .pk(&["thing_id"])
            .columns(vec![
                FieldContract::key("thing_id", FieldContract::id(), "Identity."),
                FieldContract::label(
                    "name",
                    FieldContract::native(arrow_schema::DataType::Utf8),
                    "Name.",
                ),
                FieldContract::label(
                    "label",
                    FieldContract::native(arrow_schema::DataType::Utf8),
                    "Alternate name.",
                ),
            ]),
        );
        builder.declare_enum(EnumDecl::platform(
            "EntityKind",
            vec![EnumMember::new("thing", "A thing.")],
        ));
        builder
    }
    fn document() -> DocumentSpec {
        DocumentSpec {
            name: "things",
            kind: DocumentKind::Entities,
            path_glob: "things/*.yaml",
            doc: "Things.",
            sections: vec![DocumentSection {
                key: "things",
                relation: "authored.things",
                repeated: true,
                identity_column: Some("thing_id"),
                entity_kind: Some("thing"),
                name_column: Some("name"),
                naming_scope_column: None,
                expression_owner_column: None,
                expression_owner_kind: None,
                expression_fields: &[],
                doc: "Rows.",
            }],
        }
    }
    let mut first = builder();
    first.declare_document(document());
    let first = first.build().expect("valid projection");
    let mut renamed = document();
    renamed.sections[0].name_column = Some("label");
    let mut second = builder();
    second.declare_document(renamed);
    assert_ne!(
        first.fingerprint(),
        second.build().expect("alternate projection").fingerprint()
    );
    for mutate in [0, 1, 2, 3] {
        let mut document = document();
        match mutate {
            0 => document.sections[0].identity_column = Some("name"),
            1 => document.sections[0].entity_kind = Some("unknown"),
            2 => document.sections[0].name_column = Some("thing_id"),
            _ => document.sections[0].naming_scope_column = Some("thing_id"),
        }
        let mut invalid = builder();
        invalid.declare_document(document);
        assert!(invalid.build().is_err(), "invalid projection {mutate}");
    }
    let mut conflicting = builder();
    conflicting.declare_document(document());
    let mut duplicate = document();
    duplicate.name = "elsewhere";
    duplicate.sections[0].name_column = Some("label");
    conflicting.declare_document(duplicate);
    assert!(conflicting.build().is_err());
}

#[test]
fn document_dsl_grammar_requires_complete_nested_leaf_coverage_and_affects_projection() {
    use pse_schema::model::DslSyntax;
    fn build(fields: &'static [(&'static str, DslSyntax)]) -> Result<Registry, SchemaError> {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "templates",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "owner",
            )
            .pk(&["template_id"])
            .columns(vec![FieldContract::key(
                "template_id",
                FieldContract::id(),
                "id",
            )]),
        );
        builder.declare_relation(relation(
            "expressions",
            1,
            vec![
                FieldContract::reference("template_id", FieldContract::id(), "owner")
                    .with_fk("authored.templates", "template_id"),
                FieldContract::payload(
                    "bindings",
                    FieldContract::list(FieldContract::structure(vec![
                        FieldContract::extended(ExtensionUse::ExprDsl)
                            .with_name("value")
                            .with_nullable(false),
                    ])),
                    "nested source",
                ),
            ],
        ));
        builder.declare_document(DocumentSpec {
            name: "expressions",
            kind: DocumentKind::Entities,
            path_glob: "expressions/*.yaml",
            doc: "fixture",
            sections: vec![DocumentSection {
                key: "expressions",
                relation: "authored.expressions",
                repeated: true,
                identity_column: Some("id"),
                entity_kind: None,
                name_column: None,
                naming_scope_column: None,
                expression_owner_column: Some("template_id"),
                expression_owner_kind: Some(pse_schema::model::ExpressionOwnerKind::Template),
                expression_fields: fields,
                doc: "grammar",
            }],
        });
        builder.build()
    }
    let expression = build(&[("bindings[].value", DslSyntax::Expression)]).unwrap();
    let predicate = build(&[("bindings[].value", DslSyntax::Predicate)]).unwrap();
    assert_ne!(expression.schema_rows(), predicate.schema_rows());
    assert_ne!(expression.fingerprint(), predicate.fingerprint());
    for fields in [
        &[][..],
        &[("bindings.value", DslSyntax::Expression)][..],
        &[
            ("bindings[].value", DslSyntax::Expression),
            ("bindings[].value", DslSyntax::Predicate),
        ][..],
    ] {
        assert!(build(fields).is_err());
    }
}

#[test]
fn pass_diagnostics_resolve_actual_declared_failure_members_before_materialization() {
    for diagnostics in [
        vec!["resource_limit"],
        vec!["compile.quantity"],
        vec!["runtime.resource_limit", "runtime.resource_limit"],
    ] {
        let mut builder = RegistryBuilder::new();
        pse_schema::catalog::declare(&mut builder);
        builder.declare_algorithm(
            AlgorithmDecl::new("FixtureBadDiagnostic", "1", Determinism::Deterministic)
                .diagnostics(diagnostics),
        );
        assert!(builder.build().is_err());
    }
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare(&mut builder);
    builder.declare_algorithm(
        AlgorithmDecl::new("FixtureDiagnostic", "1", Determinism::Deterministic)
            .diagnostics(vec!["compile.math", "runtime.resource_limit"]),
    );
    assert!(builder.build().is_ok());
    let mut builder = RegistryBuilder::new();
    builder.declare_algorithm(
        AlgorithmDecl::new("FixtureMissingVocabulary", "1", Determinism::Deterministic)
            .diagnostics(vec!["compile.math"]),
    );
    assert!(builder.build().is_err());
}
