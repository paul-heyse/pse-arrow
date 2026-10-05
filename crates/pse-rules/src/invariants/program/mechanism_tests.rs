// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent adversarial expectations for declaration-derived integrity SQL.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "bounded synthetic query assertions"
)]
use datafusion::{
    arrow::{
        array::RecordBatch,
        datatypes::{DataType, Field, UnionFields, UnionMode},
    },
    catalog::{CatalogProvider, MemoryCatalogProvider, MemorySchemaProvider, SchemaProvider},
    datasource::MemTable,
    prelude::SessionContext,
};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, ExtensionUse, FieldContract, IntegrityDerivation, InvariantDecl, InvariantKind,
        InvariantOrigin, Namespace, ReferenceColumn, ReferenceContract, ReferenceNullPolicy,
        RelationDecl, RelationKey, SnapshotClass, TaggedAlternative,
    },
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

fn integer(name: &'static str, nullable: bool) -> FieldContract {
    FieldContract::payload(
        name,
        FieldContract::native(DataType::Int64),
        "Synthetic integer",
    )
    .with_nullable(nullable)
}
fn relation(
    name: &'static str,
    keys: &[&'static str],
    columns: Vec<FieldContract>,
) -> RelationDecl {
    RelationDecl::new(
        Namespace::Authored,
        name,
        1,
        Authority::Authored,
        SnapshotClass::Model,
        "Independent integrity witness",
    )
    .pk(keys)
    .columns(columns)
}
fn assemble(relations: Vec<RelationDecl>) -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    for relation in relations {
        builder.declare_relation(relation);
    }
    Arc::new(builder.build().unwrap())
}
fn rows(registry: &Registry, name: &str, rows: Vec<Vec<Value>>) -> (RelationKey, RecordBatch) {
    let spec = registry.relation(name).unwrap();
    (
        spec.key,
        pse_relations::testing::untrusted_batch_from_literals(registry, spec, &rows).unwrap(),
    )
}
fn i(value: i64) -> Value {
    json!(["i64", value])
}
fn absent() -> Value {
    json!(["null", null])
}
fn structure(values: Vec<Value>) -> Value {
    json!(["struct", values])
}
fn list(values: Vec<Value>) -> Value {
    json!(["list", values])
}

/// The only unchecked seam is a test-local MemTable for hostile values. It grants no
/// production field certificate, and the expected keys never come from query evaluation.
async fn assert_keys(
    registry: &Arc<Registry>,
    source: &str,
    derivation: IntegrityDerivation,
    batches: &BTreeMap<RelationKey, RecordBatch>,
    expected: Vec<Vec<Value>>,
) {
    let invariant = registry.invariants().iter().find(|invariant| {
        invariant.relation == source && matches!(invariant.origin(), InvariantOrigin::GeneratedIntegrity(binding) if binding.derivation == derivation)
    }).unwrap();
    assert_query_keys(registry, invariant, batches, expected).await;
}
async fn assert_query_keys(
    registry: &Arc<Registry>,
    invariant: &pse_schema::model::InvariantSpec,
    batches: &BTreeMap<RelationKey, RecordBatch>,
    expected: Vec<Vec<Value>>,
) {
    let factory = pse_testkit::NativeFixture::new(NonZeroUsize::new(64 << 20).unwrap())
        .unwrap()
        .into_factory();
    let state = factory.native_state().clone();
    let catalog_name = state.config_options().catalog.default_catalog.clone();
    let context = SessionContext::new_with_state(state);
    let catalog = Arc::new(MemoryCatalogProvider::new());
    let mut schemas = BTreeMap::<String, Arc<MemorySchemaProvider>>::new();
    for name in &invariant.inputs {
        let spec = registry.relation(name).unwrap();
        let batch = batches.get(&spec.key).unwrap().clone();
        schemas
            .entry(spec.key.namespace.as_str().into())
            .or_default()
            .register_table(
                spec.key.name.into(),
                Arc::new(MemTable::try_new(batch.schema(), vec![vec![batch]]).unwrap()),
            )
            .unwrap();
    }
    for (name, schema) in schemas {
        catalog.register_schema(&name, schema).unwrap();
    }
    context.register_catalog(&catalog_name, catalog);
    let frame = context.sql(&invariant.query).await.unwrap_or_else(|error| {
        panic!(
            "{}: {error}\n{}",
            invariant.qualified_name(),
            invariant.query
        )
    });
    let mut actual = frame
        .collect()
        .await
        .unwrap()
        .iter()
        .flat_map(|batch| {
            let keys = invariant
                .key_columns
                .iter()
                .map(|name| batch.schema().index_of(name).unwrap())
                .collect::<Vec<_>>();
            pse_relations::testing::literal_rows(batch)
                .unwrap()
                .into_iter()
                .map(move |row| {
                    keys.iter()
                        .map(|index| row[*index].clone())
                        .collect::<Vec<_>>()
                })
        })
        .collect::<Vec<_>>();
    let mut expected = expected;
    actual.sort_by_key(|row| serde_json::to_string(row).unwrap());
    expected.sort_by_key(|row| serde_json::to_string(row).unwrap());
    assert_eq!(actual, expected, "{}", invariant.qualified_name());
}

#[tokio::test]
async fn primary_key_singleton_scalar_and_composite_exact_offenders() {
    let registry = assemble(vec![
        relation("single", &[], vec![integer("value", false)]),
        relation("scalar", &["id"], vec![integer("id", false)]),
        relation(
            "pair",
            &["a", "b"],
            vec![integer("a", false), integer("b", false)],
        ),
    ]);
    for count in [0, 1, 2, 3] {
        let batches = BTreeMap::from([rows(&registry, "authored.single", vec![vec![i(7)]; count])]);
        assert_keys(
            &registry,
            "authored.single",
            IntegrityDerivation::PrimaryKey,
            &batches,
            if count > 1 { vec![vec![]] } else { vec![] },
        )
        .await;
    }
    let batches = BTreeMap::from([
        rows(
            &registry,
            "authored.scalar",
            vec![vec![i(3)], vec![i(1)], vec![i(3)], vec![i(2)]],
        ),
        rows(
            &registry,
            "authored.pair",
            vec![
                vec![i(1), i(2)],
                vec![i(2), i(1)],
                vec![i(1), i(3)],
                vec![i(1), i(2)],
            ],
        ),
    ]);
    assert_keys(
        &registry,
        "authored.scalar",
        IntegrityDerivation::PrimaryKey,
        &batches,
        vec![vec![i(3)]],
    )
    .await;
    assert_keys(
        &registry,
        "authored.pair",
        IntegrityDerivation::PrimaryKey,
        &batches,
        vec![vec![i(1), i(2)]],
    )
    .await;
}

#[tokio::test]
async fn unique_keys_preserve_composite_correlation_presence_and_row_deduplication() {
    let registry = assemble(vec![
        relation(
            "named",
            &["id"],
            vec![integer("id", false), integer("a", true), integer("b", true)],
        )
        .unique("pair", &["b", "a"]),
    ]);
    let batches = BTreeMap::from([rows(
        &registry,
        "authored.named",
        vec![
            vec![i(1), i(7), i(8)],
            vec![i(2), i(7), i(8)],
            vec![i(2), i(7), i(8)],
            vec![i(3), i(8), i(7)],
            vec![i(4), i(7), i(9)],
            vec![i(5), i(7), absent()],
            vec![i(6), i(7), absent()],
            vec![i(7), absent(), absent()],
            vec![i(8), absent(), absent()],
        ],
    )]);
    assert_keys(
        &registry,
        "authored.named",
        IntegrityDerivation::UniqueKey("pair".into()),
        &batches,
        vec![vec![i(1)], vec![i(2)]],
    )
    .await;
}

#[tokio::test]
async fn table_references_keep_pair_mapping_presence_empty_targets_and_self_resolution() {
    let registry = assemble(vec![
        relation(
            "target",
            &["a", "b"],
            vec![integer("a", false), integer("b", false)],
        ),
        relation(
            "source",
            &["id"],
            vec![integer("id", false), integer("x", true), integer("y", true)],
        )
        .foreign_key("pair", &["x", "y"], "authored.target", &["b", "a"]),
        relation(
            "self_ref",
            &["id"],
            vec![integer("id", false), integer("parent", true)],
        )
        .foreign_key("parent", &["parent"], "authored.self_ref", &["id"]),
    ]);
    let mut batches = BTreeMap::from([
        rows(
            &registry,
            "authored.target",
            vec![vec![i(1), i(2)], vec![i(3), i(4)]],
        ),
        rows(
            &registry,
            "authored.source",
            vec![
                vec![i(10), i(2), i(1)],
                vec![i(11), i(2), i(3)],
                vec![i(12), i(1), i(2)],
                vec![i(13), i(9), absent()],
                vec![i(14), absent(), absent()],
            ],
        ),
        rows(
            &registry,
            "authored.self_ref",
            vec![
                vec![i(20), i(21)],
                vec![i(21), i(21)],
                vec![i(22), i(99)],
                vec![i(23), absent()],
            ],
        ),
    ]);
    assert_keys(
        &registry,
        "authored.source",
        IntegrityDerivation::TableReference("pair".into()),
        &batches,
        vec![vec![i(11)], vec![i(12)]],
    )
    .await;
    assert_keys(
        &registry,
        "authored.self_ref",
        IntegrityDerivation::TableReference("parent".into()),
        &batches,
        vec![vec![i(22)]],
    )
    .await;
    // A revealing local negative control swaps the declared pair mapping. The
    // independently chosen tuples distinguish that defect from the real predicate.
    let mut wrong = registry.invariants().iter().find(|rule| rule.relation == "authored.source" && matches!(rule.origin(), InvariantOrigin::GeneratedIntegrity(binding) if binding.derivation == IntegrityDerivation::TableReference("pair".into()))).unwrap().clone();
    wrong.query = wrong
        .query
        .replace("s.\"x\" = t.\"b\"", "s.\"x\" = t.\"a\"")
        .replace("s.\"y\" = t.\"a\"", "s.\"y\" = t.\"b\"");
    assert_query_keys(&registry, &wrong, &batches, vec![vec![i(10)], vec![i(11)]]).await;

    batches.insert(
        rows(&registry, "authored.target", vec![]).0,
        rows(&registry, "authored.target", vec![]).1,
    );
    assert_keys(
        &registry,
        "authored.source",
        IntegrityDerivation::TableReference("pair".into()),
        &batches,
        vec![vec![i(10)], vec![i(11)], vec![i(12)]],
    )
    .await;
}

fn correlated(name: &'static str) -> FieldContract {
    FieldContract::structure(vec![integer("x", true), integer("y", true)])
        .with_name(name)
        .with_nullable(true)
        .with_reference(&ReferenceContract {
            relation: "authored.target".into(),
            columns: vec![
                ReferenceColumn {
                    source: vec!["x".into()],
                    target: "b".into(),
                },
                ReferenceColumn {
                    source: vec!["y".into()],
                    target: "a".into(),
                },
            ],
            null_policy: ReferenceNullPolicy::AllOrNone,
        })
        .unwrap()
}
#[tokio::test]
async fn nested_references_preserve_occurrences_in_lists_nested_collections_and_maps() {
    let map_entry = FieldContract::structure(vec![integer("key", false), correlated("value")])
        .with_name("entries");
    let registry = assemble(vec![
        relation(
            "target",
            &["a", "b"],
            vec![integer("a", false), integer("b", false)],
        ),
        relation(
            "nested",
            &["id"],
            vec![
                integer("id", false),
                correlated("direct"),
                FieldContract::list(correlated("item"))
                    .with_name("pairs")
                    .with_nullable(true),
                FieldContract::list(
                    FieldContract::list(correlated("item"))
                        .with_name("inner")
                        .with_nullable(true),
                )
                .with_name("groups")
                .with_nullable(true),
                FieldContract::native(DataType::Map(Arc::new(map_entry.field().clone()), false))
                    .with_name("mapping")
                    .with_nullable(true),
            ],
        ),
    ]);
    let valid = structure(vec![i(2), i(1)]);
    let crossed = structure(vec![i(2), i(3)]);
    let batches = BTreeMap::from([
        rows(
            &registry,
            "authored.target",
            vec![vec![i(1), i(2)], vec![i(3), i(4)]],
        ),
        rows(
            &registry,
            "authored.nested",
            vec![
                vec![
                    i(10),
                    valid.clone(),
                    list(vec![valid.clone(), crossed.clone(), crossed.clone()]),
                    list(vec![list(vec![valid.clone()]), list(vec![crossed.clone()])]),
                    json!([
                        "map",
                        [
                            structure(vec![i(1), valid.clone()]),
                            structure(vec![i(2), crossed.clone()])
                        ]
                    ]),
                ],
                vec![
                    i(11),
                    crossed.clone(),
                    list(vec![]),
                    list(vec![list(vec![]), absent()]),
                    json!(["map", []]),
                ],
                vec![
                    i(12),
                    structure(vec![i(99), absent()]),
                    absent(),
                    absent(),
                    absent(),
                ],
                vec![
                    i(13),
                    absent(),
                    list(vec![valid.clone()]),
                    list(vec![list(vec![valid])]),
                    json!(["map", [structure(vec![i(99), absent()])]]),
                ],
            ],
        ),
    ]);
    for (path, expected) in [
        (vec!["direct"], vec![vec![i(11)]]),
        (vec!["pairs", "[]"], vec![vec![i(10)]]),
        (vec!["groups", "[]", "[]"], vec![vec![i(10)]]),
        (vec!["mapping", "[]", "value"], vec![vec![i(10)]]),
    ] {
        assert_keys(
            &registry,
            "authored.nested",
            IntegrityDerivation::ReferenceOccurrence(path.into_iter().map(str::to_owned).collect()),
            &batches,
            expected,
        )
        .await;
    }
}

#[tokio::test]
async fn ordinal_queries_use_count_boundaries_and_visible_scalar_or_nested_occurrences() {
    let ordinal = || {
        FieldContract::extended(ExtensionUse::OrdinalRef {
            target: "authored.target",
        })
        .with_nullable(true)
    };
    let registry = assemble(vec![
        relation("target", &["id"], vec![integer("id", false)]),
        relation(
            "ordinals",
            &["id"],
            vec![
                integer("id", false),
                ordinal().with_name("position"),
                FieldContract::list(ordinal())
                    .with_name("positions")
                    .with_nullable(true),
            ],
        ),
    ]);
    let mut batches = BTreeMap::from([
        rows(
            &registry,
            "authored.target",
            vec![vec![i(100)], vec![i(200)]],
        ),
        rows(
            &registry,
            "authored.ordinals",
            vec![
                vec![i(10), i(-1), list(vec![i(-1), i(-1)])],
                vec![i(11), i(0), list(vec![i(0), i(1)])],
                vec![i(12), i(1), list(vec![])],
                vec![i(13), i(2), list(vec![i(2)])],
                vec![i(14), absent(), absent()],
            ],
        ),
    ]);
    for path in [vec!["position"], vec!["positions", "[]"]] {
        assert_keys(
            &registry,
            "authored.ordinals",
            IntegrityDerivation::OrdinalOccurrence(path.into_iter().map(str::to_owned).collect()),
            &batches,
            vec![vec![i(10)], vec![i(13)]],
        )
        .await;
    }
    // Relaxing the exclusive upper bound loses the equal-to-count witness.
    let mut wrong = registry.invariants().iter().find(|rule| rule.relation == "authored.ordinals" && matches!(rule.origin(), InvariantOrigin::GeneratedIntegrity(binding) if binding.derivation == IntegrityDerivation::OrdinalOccurrence(vec!["position".into()]))).unwrap().clone();
    wrong.query = wrong.query.replace(" >= ", " > ");
    assert_query_keys(&registry, &wrong, &batches, vec![vec![i(10)]]).await;
    let (key, empty) = rows(&registry, "authored.target", vec![]);
    batches.insert(key, empty);
    assert_keys(
        &registry,
        "authored.ordinals",
        IntegrityDerivation::OrdinalOccurrence(vec!["position".into()]),
        &batches,
        vec![vec![i(10)], vec![i(11)], vec![i(12)], vec![i(13)]],
    )
    .await;
    assert_keys(
        &registry,
        "authored.ordinals",
        IntegrityDerivation::OrdinalOccurrence(vec!["positions".into(), "[]".into()]),
        &batches,
        vec![vec![i(10)], vec![i(11)], vec![i(13)]],
    )
    .await;
}

#[tokio::test]
async fn authored_generic_spelling_and_kind_keep_independent_sign_semantics() {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(relation(
        "custom",
        &["id"],
        vec![integer("id", false), integer("value", false)],
    ));
    builder.declare_invariant(InvariantDecl::error(
        "authored.custom",
        "unique:value",
        InvariantKind::Unique,
        "SELECT id FROM authored.custom WHERE value < 0",
        vec!["authored.custom".into()],
        vec!["id"],
        "Independent negative value predicate",
    ));
    let registry = Arc::new(builder.build().unwrap());
    let invariant = registry
        .invariants()
        .iter()
        .find(|rule| rule.name == "unique:value")
        .unwrap();
    assert_eq!(invariant.origin(), &InvariantOrigin::AuthoredQuery);
    let batches = BTreeMap::from([rows(
        &registry,
        "authored.custom",
        vec![
            vec![i(1), i(7)],
            vec![i(2), i(7)],
            vec![i(3), i(-2)],
            vec![i(4), i(0)],
        ],
    )]);
    assert_query_keys(&registry, invariant, &batches, vec![vec![i(3)]]).await;
}

#[tokio::test]
async fn reference_sql_lowers_list_variants_dictionaries_runs_unions_and_tagged_arms() {
    let reference = || integer("ref", true).with_fk("authored.target", "id");
    let child = Arc::new(reference().field().clone());
    let variants = [
        DataType::List(Arc::clone(&child)),
        DataType::LargeList(Arc::clone(&child)),
        DataType::FixedSizeList(Arc::clone(&child), 2),
        DataType::ListView(Arc::clone(&child)),
        DataType::LargeListView(child),
    ];
    for kind in variants {
        let registry = assemble(vec![
            relation("target", &["id"], vec![integer("id", false)]),
            relation(
                "encoded",
                &["id"],
                vec![
                    integer("id", false),
                    FieldContract::native(kind.clone())
                        .with_name("wrapped")
                        .with_nullable(true),
                ],
            ),
        ]);
        let batches = BTreeMap::from([
            rows(&registry, "authored.target", vec![vec![i(7)]]),
            rows(
                &registry,
                "authored.encoded",
                vec![
                    vec![i(1), list(vec![i(7), i(7)])],
                    vec![i(2), list(vec![i(7), i(99)])],
                    vec![i(3), absent()],
                ],
            ),
        ]);
        assert_keys(
            &registry,
            "authored.encoded",
            IntegrityDerivation::ReferenceOccurrence(vec!["wrapped".into(), "[]".into()]),
            &batches,
            vec![vec![i(2)]],
        )
        .await;
    }
    let dictionary = FieldContract::native(DataType::Dictionary(
        Box::new(DataType::Int32),
        Box::new(DataType::Struct(
            vec![Arc::new(reference().field().clone())].into(),
        )),
    ))
    .with_name("wrapped")
    .with_nullable(true);
    let run = FieldContract::native(DataType::RunEndEncoded(
        Arc::new(Field::new("run_ends", DataType::Int32, false)),
        Arc::new(reference().field().clone().with_name("values")),
    ))
    .with_name("wrapped")
    .with_nullable(true);
    for (field, encoded) in [(dictionary, true), (run, false)] {
        let registry = assemble(vec![
            relation("target", &["id"], vec![integer("id", false)]),
            relation("encoded", &["id"], vec![integer("id", false), field]),
        ]);
        let value = |number| {
            if encoded {
                structure(vec![i(number)])
            } else {
                i(number)
            }
        };
        let values = vec![
            vec![i(1), value(7)],
            vec![i(2), value(99)],
            vec![i(3), absent()],
        ];
        let batches = BTreeMap::from([
            rows(&registry, "authored.target", vec![vec![i(7)]]),
            rows(&registry, "authored.encoded", values),
        ]);
        let path = if encoded {
            vec!["wrapped".into(), "ref".into()]
        } else {
            vec!["wrapped".into()]
        };
        assert_keys(
            &registry,
            "authored.encoded",
            IntegrityDerivation::ReferenceOccurrence(path.clone()),
            &batches,
            vec![vec![i(2)]],
        )
        .await;
        if !encoded {
            // The repaired run-end lowering also executes through typed production
            // field admission, exact required selection and diagnostic projection.
            let invariant = registry.invariants().iter().find(|rule| rule.relation == "authored.encoded" && matches!(rule.origin(), InvariantOrigin::GeneratedIntegrity(binding) if binding.derivation == IntegrityDerivation::ReferenceOccurrence(path.clone()))).unwrap();
            let session = super::tests::session(&registry, batches.clone());
            let selected = std::collections::BTreeSet::from([invariant.id]);
            let report = crate::invariants::run_invariants(
                &batches,
                &session,
                &registry,
                crate::invariants::InvariantScope::Required(&selected),
                &pse_columnar::CancellationToken::new(),
            )
            .await
            .unwrap();
            assert_eq!(report.check_count(), 1);
            assert_eq!(report.error_count(), 1);
            let view = pse_relations::generated::runtime::diagnostics_findings::View::from_checked(
                &report.findings()[0],
            )
            .unwrap();
            assert_eq!(view.check_id_column().value(0), invariant.id.as_bytes());
        }
    }
    for mode in [UnionMode::Dense, UnionMode::Sparse] {
        let union = FieldContract::native(DataType::Union(
            UnionFields::try_new(
                [1, 3],
                vec![
                    reference().field().clone(),
                    integer("other", true).field().clone(),
                ],
            )
            .unwrap(),
            mode,
        ))
        .with_name("wrapped")
        .with_nullable(true);
        let registry = assemble(vec![
            relation("target", &["id"], vec![integer("id", false)]),
            relation("encoded", &["id"], vec![integer("id", false), union]),
        ]);
        let batches = BTreeMap::from([
            rows(&registry, "authored.target", vec![vec![i(7)]]),
            rows(
                &registry,
                "authored.encoded",
                vec![
                    vec![i(1), json!(["union", [1, i(7)]])],
                    vec![i(2), json!(["union", [1, i(99)]])],
                    vec![i(3), json!(["union", [3, i(99)]])],
                ],
            ),
        ]);
        assert_keys(
            &registry,
            "authored.encoded",
            IntegrityDerivation::ReferenceOccurrence(vec!["wrapped".into(), "ref".into()]),
            &batches,
            vec![vec![i(2)]],
        )
        .await;
    }
    let tagged = FieldContract::structure(vec![
        FieldContract::native(DataType::Utf8).with_name("kind"),
        FieldContract::structure(vec![reference()])
            .with_name("reference")
            .with_nullable(true),
        FieldContract::structure(vec![integer("value", true)])
            .with_name("other")
            .with_nullable(true),
    ])
    .with_name("wrapped")
    .with_nullable(true)
    .with_alternative(&TaggedAlternative {
        discriminator: "kind".into(),
        arms: BTreeMap::from([
            ("reference".into(), Some("reference".into())),
            ("other".into(), Some("other".into())),
        ]),
    });
    let registry = assemble(vec![
        relation("target", &["id"], vec![integer("id", false)]),
        relation("encoded", &["id"], vec![integer("id", false), tagged]),
    ]);
    let batches = BTreeMap::from([
        rows(&registry, "authored.target", vec![vec![i(7)]]),
        rows(
            &registry,
            "authored.encoded",
            vec![
                vec![
                    i(1),
                    structure(vec![
                        json!(["text", "reference"]),
                        structure(vec![i(7)]),
                        absent(),
                    ]),
                ],
                vec![
                    i(2),
                    structure(vec![
                        json!(["text", "reference"]),
                        structure(vec![i(99)]),
                        absent(),
                    ]),
                ],
                vec![
                    i(3),
                    structure(vec![
                        json!(["text", "other"]),
                        structure(vec![i(99)]),
                        structure(vec![i(99)]),
                    ]),
                ],
                vec![i(4), absent()],
            ],
        ),
    ]);
    assert_keys(
        &registry,
        "authored.encoded",
        IntegrityDerivation::ReferenceOccurrence(vec![
            "wrapped".into(),
            "reference".into(),
            "ref".into(),
        ]),
        &batches,
        vec![vec![i(2)]],
    )
    .await;
}

#[tokio::test]
async fn reference_sql_ignores_hidden_struct_storage_and_preserves_collision_safe_root_keys() {
    use datafusion::arrow::{
        array::{ArrayRef, Int64Array, StructArray},
        buffer::NullBuffer,
    };
    let child = integer("ref", true).with_fk("authored.target", "id");
    let parent = FieldContract::structure(vec![child])
        .with_name("wrapped")
        .with_nullable(true);
    let entry = FieldContract::structure(vec![
        integer("key", false),
        integer("value", true).with_fk("authored.target", "id"),
    ])
    .with_name("entries");
    let mapping = FieldContract::native(DataType::Map(Arc::new(entry.field().clone()), false))
        .with_name("mapping")
        .with_nullable(true);
    let registry = assemble(vec![
        relation("target", &["id"], vec![integer("id", false)]),
        relation(
            "hidden",
            &["__pse_value", "__pse_value__key", "__pse_value__item"],
            vec![
                integer("__pse_value", false),
                integer("__pse_value__key", false),
                integer("__pse_value__item", false),
                parent,
                mapping,
            ],
        ),
    ]);
    let spec = registry.relation("authored.hidden").unwrap();
    let schema = Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap());
    let DataType::Struct(fields) = schema.field(3).data_type() else {
        panic!("struct declaration")
    };
    let storage = StructArray::new(
        fields.clone(),
        vec![Arc::new(Int64Array::from(vec![7, 99, 99]))],
        Some(NullBuffer::from(vec![true, false, true])),
    );
    let (_, map_rows) = rows(
        &registry,
        "authored.hidden",
        vec![
            vec![
                i(1),
                i(10),
                i(100),
                absent(),
                json!(["map", [structure(vec![i(1), i(7)])]]),
            ],
            vec![
                i(2),
                i(20),
                i(200),
                absent(),
                json!(["map", [structure(vec![i(1), i(99)])]]),
            ],
            vec![i(3), i(30), i(300), absent(), json!(["map", []])],
        ],
    );
    let columns: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from(vec![1, 2, 3])),
        Arc::new(Int64Array::from(vec![10, 20, 30])),
        Arc::new(Int64Array::from(vec![100, 200, 300])),
        Arc::new(storage),
        Arc::clone(map_rows.column(4)),
    ];
    let batches = BTreeMap::from([
        rows(&registry, "authored.target", vec![vec![i(7)]]),
        (spec.key, RecordBatch::try_new(schema, columns).unwrap()),
    ]);
    assert_keys(
        &registry,
        "authored.hidden",
        IntegrityDerivation::ReferenceOccurrence(vec!["wrapped".into(), "ref".into()]),
        &batches,
        vec![vec![i(3), i(30), i(300)]],
    )
    .await;
    assert_keys(
        &registry,
        "authored.hidden",
        IntegrityDerivation::ReferenceOccurrence(vec![
            "mapping".into(),
            "[]".into(),
            "value".into(),
        ]),
        &batches,
        vec![vec![i(2), i(20), i(200)]],
    )
    .await;
}
