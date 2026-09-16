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
    Authority, Cell, CmpOp, ColumnRole, DependencyMode, DepthBound, Determinism, DocumentKind,
    DocumentSection, DocumentSpec, EnumDecl, EnumMember, ExtensionUse, FieldContract, InputPort,
    ManifestField, ManifestSpec, ManifestType, MigrationSpec, MigrationStep, Namespace,
    NullEquality, OutputPort, PassDecl, PortSource, RelationDecl, RuleDecl, RuleExpr, RuleHead,
    RulePlan, SnapshotClass,
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

fn scan(relation: &'static str, port: &'static str) -> RulePlan {
    RulePlan::Scan {
        relation: relation.to_owned(),
        port,
    }
}

fn rule(version: &'static str, plan: RulePlan) -> RuleDecl {
    RuleDecl::new(
        "fixture",
        version,
        0,
        RuleHead::Violations {
            of: "authored.base".to_owned(),
            key_columns: vec!["id"],
        },
        plan,
    )
}

fn filtered(predicate: RuleExpr) -> RulePlan {
    RulePlan::Filter {
        input: Box::new(scan("authored.base", "base")),
        predicate,
    }
}

fn rules(rules: Vec<RuleDecl>) -> Result<Registry, SchemaError> {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(relation("base", 1, vec![]));
    for rule in rules {
        builder.declare_rule(rule);
    }
    builder.build()
}

#[test]
fn explicit_assertion_is_allowed_only_beneath_output_projections() {
    let assertion = RulePlan::Assert {
        input: Box::new(scan("authored.base", "base")),
        predicate: RuleExpr::Lit(Cell::Bool(true)),
    };
    assert!(
        rules(vec![rule(
            "1",
            RulePlan::Project {
                input: Box::new(assertion.clone()),
                columns: vec![("id".into(), RuleExpr::col("id"))],
            }
        )])
        .is_ok()
    );
    for invalid in [
        RulePlan::Filter {
            input: Box::new(assertion.clone()),
            predicate: RuleExpr::Lit(Cell::Bool(true)),
        },
        RulePlan::Union(vec![assertion.clone(), assertion.clone()]),
        RulePlan::Assert {
            input: Box::new(assertion),
            predicate: RuleExpr::Lit(Cell::Bool(true)),
        },
    ] {
        let error = rules(vec![rule("1", invalid)]).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("assertion predicate is allowed only"),
            "{error}"
        );
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
        relation("a", 1, vec![]).pk(&[]),
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
fn duplicate_enum_members_document_sections_and_manifest_fields_are_rejected() {
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
    let manifest = || ManifestSpec::new("test", "test", vec![]);
    let mut builder = RegistryBuilder::new();
    builder
        .declare_manifest(manifest())
        .declare_manifest(manifest());
    assert!(matches!(
        builder.build(),
        Err(SchemaError::DuplicateDeclaration {
            kind: "manifest",
            ..
        })
    ));
    let mut builder = RegistryBuilder::new();
    builder.declare_manifest(ManifestSpec::new(
        "test",
        "test",
        vec![ManifestField::new(
            "payload",
            ManifestType::Struct(vec![
                ManifestField::new("x", ManifestType::U64, "x"),
                ManifestField::new("x", ManifestType::U64, "duplicate"),
            ]),
            "payload",
        )],
    ));
    assert!(matches!(
        builder.build(),
        Err(SchemaError::DuplicateDeclaration {
            kind: "manifest field",
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
        .rule("ordinal_range:nested.items[]:authored.source@1")
        .unwrap();
    let dependencies = ordinal.plan.dependencies();
    assert!(
        dependencies
            .iter()
            .any(|(name, _, _)| *name == "authored.source")
    );
    assert!(
        dependencies
            .iter()
            .any(|(name, _, _)| *name == "authored.target")
    );
}

#[test]
fn a_conflicting_manual_integrity_projection_cannot_replace_the_declaration() {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(relation("base", 1, vec![]));
    builder.declare_rule(
        RuleDecl::new(
            "unique:pk:authored.base",
            "1",
            1,
            RuleHead::Violations {
                of: "authored.base".into(),
                key_columns: vec!["id"],
            },
            RulePlan::Project {
                input: Box::new(filtered(RuleExpr::Lit(Cell::Bool(false)))),
                columns: (vec![("id", RuleExpr::col("id"))])
                    .into_iter()
                    .map(|(name, expression)| (name.into(), expression))
                    .collect(),
            },
        )
        .stratified_negation(),
    );
    assert!(matches!(
        builder.build(),
        Err(SchemaError::DuplicateDeclaration { .. })
    ));
}

#[test]
fn rule_predicate_payloads_are_rows_and_change_the_registry_fingerprint() {
    let before = rules(vec![rule("1", filtered(RuleExpr::Lit(Cell::Bool(true))))]).unwrap();
    let after = rules(vec![rule("1", filtered(RuleExpr::Lit(Cell::Bool(false))))]).unwrap();
    assert_eq!(rows(&before, "rule_expr_nodes")[0][7], Cell::Bool(true));
    assert_eq!(rows(&after, "rule_expr_nodes")[0][7], Cell::Bool(false));
    assert_ne!(before.fingerprint(), after.fingerprint());
    assert!(
        before
            .rule_dependencies()
            .iter()
            .all(|dependency| dependency.mode != DependencyMode::Write)
    );
}

#[test]
fn nested_literals_and_expression_edges_preserve_every_scalar_payload() {
    let literal = Cell::Struct(vec![
        Cell::List(vec![Cell::U64(u64::MAX), Cell::I64(i64::MIN)]),
        Cell::F64(f64::from_bits(0x7ff8_0000_0000_0001)),
        Cell::Enum("enum literal"),
    ]);
    let registry = rules(vec![rule(
        "1",
        filtered(RuleExpr::IsNull(Box::new(RuleExpr::Lit(literal)))),
    )])
    .unwrap();
    let rule_id = registry.rule("fixture@1").unwrap().id;
    let nodes = rows(&registry, "rule_expr_nodes")
        .into_iter()
        .filter(|row| row[1] == Cell::Id(rule_id))
        .collect::<Vec<_>>();
    let edges = rows(&registry, "rule_expr_edges")
        .into_iter()
        .filter(|row| nodes.iter().any(|node| node[0] == row[0]))
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), 7);
    assert_eq!(edges.len(), 6);
    assert!(nodes.iter().any(|row| row[6] == Cell::Enum("struct")));
    assert!(nodes.iter().any(|row| row[6] == Cell::Enum("list")));
    assert!(nodes.iter().any(|row| row[9] == Cell::U64(u64::MAX)));
    assert!(nodes.iter().any(|row| row[8] == Cell::I64(i64::MIN)));
    assert!(
        nodes
            .iter()
            .any(|row| row[10] == Cell::U64(0x7ff8_0000_0000_0001))
    );
    assert!(
        nodes
            .iter()
            .any(|row| row[6] == Cell::Enum("enum") && row[11] == Cell::text("enum literal"))
    );
    for edge in edges {
        assert!(nodes.iter().any(|row| row[0] == edge[0]));
        assert!(nodes.iter().any(|row| row[0] == edge[2]));
    }
}

#[test]
fn comparison_changes_are_persisted_and_invalid_operand_types_are_rejected() {
    let build = |op| {
        rules(vec![rule(
            "1",
            filtered(RuleExpr::cmp(op, RuleExpr::col("id"), RuleExpr::col("id"))),
        )])
        .unwrap()
    };
    let equal = build(CmpOp::Eq);
    let unequal = build(CmpOp::NotEq);
    assert_ne!(equal.fingerprint(), unequal.fingerprint());
    assert!(
        rows(&unequal, "rule_expr_nodes")
            .iter()
            .any(|row| row[4] == Cell::Enum("not_eq"))
    );
    assert!(
        rules(vec![rule(
            "1",
            filtered(RuleExpr::cmp(
                CmpOp::Eq,
                RuleExpr::col("id"),
                RuleExpr::Lit(Cell::Enum("not an identity"))
            ))
        )])
        .is_err()
    );
}

#[test]
fn rule_versions_have_exact_dependencies_and_ambiguous_names_do_not_resolve() {
    let registry = rules(vec![
        rule("1", filtered(RuleExpr::Lit(Cell::Bool(true)))),
        rule("2", filtered(RuleExpr::Lit(Cell::Bool(false)))),
    ])
    .unwrap();
    assert!(registry.rule("fixture").is_none());
    for version in ["fixture@1", "fixture@2"] {
        let id = registry.rule(version).unwrap().id;
        assert!(
            registry
                .rule_dependencies()
                .iter()
                .any(|dependency| dependency.rule_id == id)
        );
    }
    assert_ne!(
        registry.rule_dependencies()[0].rule_id,
        registry.rule_dependencies()[1].rule_id
    );
}

#[test]
fn recursive_references_bind_lexically_and_do_not_become_external_dependencies() {
    let recursive = |seed, step| RulePlan::Recursive {
        name: "closure",
        seed: Box::new(seed),
        step: Box::new(step),
        is_distinct: false,
        depth_bound: DepthBound::Bounded(4),
    };
    let reference = || RulePlan::RecursiveRef { name: "closure" };
    assert!(rules(vec![rule("1", reference())]).is_err());
    assert!(
        rules(vec![rule(
            "1",
            recursive(reference(), scan("authored.base", "base"))
        )])
        .is_err()
    );
    let registry = rules(vec![rule(
        "1",
        recursive(scan("authored.base", "base"), reference()),
    )])
    .unwrap();
    assert_eq!(
        registry
            .rule_dependencies()
            .iter()
            .filter(|dependency| dependency.rule_id == registry.rule("fixture@1").unwrap().id)
            .count(),
        1
    );
    let nodes = rows(&registry, "rule_plan_nodes");
    let binder = nodes
        .iter()
        .find(|row| row[2] == Cell::Enum("recursive"))
        .unwrap();
    let reference = nodes
        .iter()
        .find(|row| row[2] == Cell::Enum("recursive_ref"))
        .unwrap();
    assert_eq!(reference[12], binder[0]);
    assert_eq!(binder[15], Cell::U64(4));
}

#[test]
fn recursive_policy_retains_native_closure_and_resolves_nearest_binders() {
    let recursive = |seed, step, is_distinct, depth_bound| RulePlan::Recursive {
        name: "closure",
        seed: Box::new(seed),
        step: Box::new(step),
        is_distinct,
        depth_bound,
    };
    for (is_distinct, depth_bound, accepted) in [
        (false, DepthBound::FixedPoint, true),
        (true, DepthBound::FixedPoint, true),
        (true, DepthBound::Bounded(4), true),
        (false, DepthBound::Bounded(0), false),
    ] {
        assert_eq!(
            rules(vec![rule(
                "1",
                recursive(
                    scan("authored.base", "base"),
                    RulePlan::RecursiveRef { name: "closure" },
                    is_distinct,
                    depth_bound
                ),
            )])
            .is_ok(),
            accepted
        );
    }
    let registry = rules(vec![rule(
        "1",
        recursive(
            scan("authored.base", "base"),
            recursive(
                scan("authored.base", "base"),
                RulePlan::RecursiveRef { name: "closure" },
                false,
                DepthBound::Bounded(2),
            ),
            false,
            DepthBound::SeedRows,
        ),
    )])
    .unwrap();
    let nodes = rows(&registry, "rule_plan_nodes");
    let inner = nodes.iter().find(|row| row[15] == Cell::U64(2)).unwrap();
    let reference = nodes
        .iter()
        .find(|row| row[2] == Cell::Enum("recursive_ref"))
        .unwrap();
    assert_eq!(reference[12], inner[0]);
    assert!(nodes.iter().any(|row| row[14] == Cell::Enum("seed_rows")));
    assert_eq!(
        registry
            .rule_dependencies()
            .iter()
            .filter(|dependency| dependency.rule_id == registry.rule("fixture@1").unwrap().id)
            .count(),
        1
    );
}

#[test]
fn projected_float_keys_are_rejected_in_the_operator_that_uses_them() {
    let plan = RulePlan::Distinct(Box::new(RulePlan::Project {
        input: Box::new(scan("authored.base", "base")),
        columns: (vec![("id", RuleExpr::Lit(Cell::F64(1.0)))])
            .into_iter()
            .map(|(name, expression)| (name.into(), expression))
            .collect(),
    }));
    assert!(matches!(
        rules(vec![rule("1", plan)]),
        Err(SchemaError::RuleFloatKey { .. })
    ));
}

#[test]
fn join_keys_resolve_against_their_own_sides_and_preserve_null_semantics() {
    let build = |null_equality| {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(relation("base", 1, vec![]));
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "right",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "right",
            )
            .pk(&["key"])
            .columns(vec![
                FieldContract::key("key", FieldContract::id(), "key"),
                FieldContract::new(
                    "id",
                    FieldContract::native(arrow_schema::DataType::Float64),
                    false,
                    ColumnRole::Measure,
                    "unrelated same-name float",
                ),
            ]),
        );
        builder.declare_rule(rule(
            "1",
            RulePlan::Project {
                input: Box::new(RulePlan::EquiJoin {
                    left: Box::new(scan("authored.base", "left")),
                    right: Box::new(scan("authored.right", "right")),
                    keys: (vec![("id", "key")])
                        .into_iter()
                        .map(|(left, right)| (left.into(), right.into()))
                        .collect(),
                    null_equality,
                }),
                columns: (vec![("id", RuleExpr::col("left.id"))])
                    .into_iter()
                    .map(|(name, expression)| (name.into(), expression))
                    .collect(),
            },
        ));
        builder.build().unwrap()
    };
    let sql = build(NullEquality::NullEqualsNothing);
    let distinct = build(NullEquality::NullEqualsNull);
    let nodes = rows(&sql, "rule_plan_nodes");
    assert!(
        nodes
            .iter()
            .any(|row| row[10] == Cell::Enum("null_equals_nothing"))
    );
    assert_ne!(sql.fingerprint(), distinct.fingerprint());
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
fn stage_ports_must_match_relations_and_have_an_acyclic_producer_graph() {
    let build = |mismatch, cycle| {
        let mut builder = RegistryBuilder::new();
        builder
            .declare_relation(relation("base", 1, vec![]))
            .declare_relation(relation("other", 1, vec![]));
        builder.declare_pass(
            PassDecl::new("P0", "1", Determinism::Deterministic)
                .inputs(if cycle {
                    vec![InputPort {
                        port: "back",
                        relation: "authored.base".to_owned(),
                        source: PortSource::Derived {
                            pass: "P1",
                            port: "out",
                        },
                        required: true,
                    }]
                } else {
                    vec![]
                })
                .outputs(vec![OutputPort {
                    port: "out",
                    relation: "authored.base".to_owned(),
                }]),
        );
        builder.declare_pass(
            PassDecl::new("P1", "1", Determinism::Deterministic)
                .inputs(vec![InputPort {
                    port: "in",
                    relation: if mismatch {
                        "authored.other"
                    } else {
                        "authored.base"
                    }
                    .to_owned(),
                    source: PortSource::Derived {
                        pass: "P0",
                        port: "out",
                    },
                    required: true,
                }])
                .outputs(vec![OutputPort {
                    port: "out",
                    relation: "authored.base".to_owned(),
                }]),
        );
        builder.build()
    };
    assert!(build(false, false).is_ok());
    assert!(matches!(
        build(true, false),
        Err(SchemaError::StageGraph { .. })
    ));
    assert!(matches!(
        build(false, true),
        Err(SchemaError::StageGraph { .. })
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
        builder.declare_pass(
            PassDecl::new("FixtureBadDiagnostic", "1", Determinism::Deterministic)
                .diagnostics(diagnostics),
        );
        assert!(builder.build().is_err());
    }
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare(&mut builder);
    builder.declare_pass(
        PassDecl::new("FixtureDiagnostic", "1", Determinism::Deterministic)
            .diagnostics(vec!["compile.math", "runtime.resource_limit"]),
    );
    assert!(builder.build().is_ok());
    let mut builder = RegistryBuilder::new();
    builder.declare_pass(
        PassDecl::new("FixtureMissingVocabulary", "1", Determinism::Deterministic)
            .diagnostics(vec!["compile.math"]),
    );
    assert!(builder.build().is_err());
}
