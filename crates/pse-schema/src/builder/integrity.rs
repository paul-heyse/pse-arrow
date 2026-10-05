// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Invariant-query projection of the declaration-owned obligation product.
use super::RegistryBuilder;
use crate::{
    catalog::inv::{columns, identifier, literal, table},
    model::{IntegrityBinding, IntegrityDerivation, InvariantDecl, InvariantKind, InvariantOrigin},
};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    // Queries bind qualified table names, which resolve to the registry's current
    // (highest) version. Historical declarations remain migration authority, but
    // cannot supply a second executable projection against that current table.
    let mut current = std::collections::BTreeMap::new();
    for relation in &builder.relations {
        current
            .entry((relation.key.namespace.as_str(), relation.key.name))
            .and_modify(|selected: &mut &crate::model::RelationDecl| {
                if selected.key.version < relation.key.version {
                    *selected = relation;
                }
            })
            .or_insert(relation);
    }
    let relations = current.into_values().cloned().collect::<Vec<_>>();
    // Derivation can precede later declarations during catalog assembly. Rebuild
    // only native producer outputs, so a newly selected version replaces stale
    // projections without granting authored SQL a generated origin.
    builder
        .invariants
        .retain(|declaration| matches!(declaration.origin, InvariantOrigin::AuthoredQuery));
    for relation in relations {
        if relation.primary_key.is_none() {
            continue;
        }
        let Some(Ok(product)) = builder.obligations.get(&relation.key).cloned() else {
            continue;
        };
        let keys = &product.primary_key;
        let name = relation.key.qualified_name();
        let source = table(&name);
        let value = format!("s.{}", identifier(&product.value_column));
        let group = columns(keys, "s");
        let projection = if keys.is_empty() {
            "TRUE AS singleton".into()
        } else {
            group.clone()
        };
        let query = if keys.is_empty() {
            format!("SELECT COUNT(*) AS violations FROM {source} s HAVING COUNT(*) > 1")
        } else {
            format!("SELECT {group} FROM {source} s GROUP BY {group} HAVING COUNT(*) > 1")
        };
        invariant(
            builder,
            IntegrityBinding {
                relation: relation.key,
                derivation: IntegrityDerivation::PrimaryKey,
            },
            &name,
            "unique:pk",
            InvariantKind::Unique,
            keys,
            query,
            &[&name],
            "The declared primary key identifies exactly one row.",
        );
        for key in &relation.unique_keys {
            let group = columns(&key.columns, "u");
            let present = key
                .columns
                .iter()
                .map(|column| format!("u.{} IS NOT NULL", identifier(column)))
                .collect::<Vec<_>>()
                .join(" AND ");
            let joined = key
                .columns
                .iter()
                .map(|column| format!("s.{0} = d.{0}", identifier(column)))
                .collect::<Vec<_>>()
                .join(" AND ");
            invariant(
                builder,
                IntegrityBinding {
                    relation: relation.key,
                    derivation: IntegrityDerivation::UniqueKey(key.name.to_owned()),
                },
                &name,
                &format!("unique:{}", key.name),
                InvariantKind::Unique,
                keys,
                format!(
                    "SELECT DISTINCT {projection} FROM {source} s JOIN (SELECT {group} FROM {source} u WHERE {present} GROUP BY {group} HAVING COUNT(*) > 1) d ON {joined}"
                ),
                &[&name],
                "A declared unique key identifies at most one row among the rows whose key is present.",
            );
        }
        for reference in &relation.foreign_keys {
            let present = reference
                .columns
                .iter()
                .map(|column| format!("s.{} IS NOT NULL", identifier(column)))
                .collect::<Vec<_>>()
                .join(" AND ");
            let equal = reference
                .columns
                .iter()
                .zip(&reference.target_columns)
                .map(|(local, remote)| {
                    format!("s.{} = t.{}", identifier(local), identifier(remote))
                })
                .collect::<Vec<_>>()
                .join(" AND ");
            let target = table(reference.target);
            invariant(
                builder,
                IntegrityBinding {
                    relation: relation.key,
                    derivation: IntegrityDerivation::TableReference(reference.name.to_owned()),
                },
                &name,
                &format!("foreign_key:{}", reference.name),
                InvariantKind::ForeignKey,
                keys,
                format!(
                    "SELECT DISTINCT {projection} FROM {source} s WHERE {present} AND NOT EXISTS (SELECT 1 FROM {target} t WHERE {equal})"
                ),
                &[&name, reference.target],
                "Every reference whose columns are all present resolves to its declared target key.",
            );
        }
        for occurrence in &product.references {
            let reference = &occurrence.reference;
            let values = reference
                .columns
                .iter()
                .map(|mapping| {
                    mapping.source.iter().fold(value.clone(), |value, field| {
                        format!("get_field({value}, {})", literal(field))
                    })
                })
                .collect::<Vec<_>>();
            let present = values
                .iter()
                .map(|value| format!("{value} IS NOT NULL"))
                .collect::<Vec<_>>()
                .join(" AND ");
            let equal = values
                .iter()
                .zip(&reference.columns)
                .map(|(value, mapping)| format!("{value} = t.{}", identifier(&mapping.target)))
                .collect::<Vec<_>>()
                .join(" AND ");
            let target = table(&reference.relation);
            let path = display_path(&occurrence.path);
            invariant(
                builder,
                IntegrityBinding {
                    relation: relation.key,
                    derivation: IntegrityDerivation::ReferenceOccurrence(occurrence.path.clone()),
                },
                &name,
                &format!("foreign_key:{path}"),
                InvariantKind::ForeignKey,
                keys,
                format!(
                    "SELECT DISTINCT {projection} FROM ({}) s WHERE {present} AND NOT EXISTS (SELECT 1 FROM {target} t WHERE {equal})",
                    occurrence.input
                ),
                &[&name, &reference.relation],
                "Every present foreign-key value resolves to its declared relation and column.",
            );
        }
        for occurrence in &product.ordinals {
            invariant(
                builder,
                IntegrityBinding {
                    relation: relation.key,
                    derivation: IntegrityDerivation::OrdinalOccurrence(occurrence.path.clone()),
                },
                &name,
                &format!("ordinal_range:{}", display_path(&occurrence.path)),
                InvariantKind::Domain,
                keys,
                format!(
                    "SELECT DISTINCT {projection} FROM ({}) s WHERE {value} < 0 OR {value} >= (SELECT COUNT(*) FROM {})",
                    occurrence.input,
                    table(&occurrence.target)
                ),
                &[&name, &occurrence.target],
                "Every visible ordinal is within its explicitly declared target relation's row count.",
            );
        }
    }
}
fn display_path(path: &[String]) -> String {
    path.join(".").replace(".[]", "[]")
}

#[expect(
    clippy::too_many_arguments,
    reason = "independent invariant contract dimensions"
)]
fn invariant(
    builder: &mut RegistryBuilder,
    binding: IntegrityBinding,
    relation: &str,
    name: &str,
    kind: InvariantKind,
    keys: &[&'static str],
    query: impl Into<String>,
    inputs: &[&str],
    doc: &'static str,
) {
    let mut inputs = inputs
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    inputs.sort();
    inputs.dedup();
    let mut declaration =
        InvariantDecl::error(relation, name, kind, query, inputs, keys.to_vec(), doc);
    declaration.origin = InvariantOrigin::GeneratedIntegrity(binding);
    if !builder.declared_invariants().contains(&declaration) {
        builder.invariants.push(declaration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};

    fn builder() -> RegistryBuilder {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "example",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "Synthetic relation",
            )
            .pk(&["id"])
            .columns(vec![FieldContract::key(
                "id",
                FieldContract::native(arrow_schema::DataType::Int64),
                "Key",
            )]),
        );
        builder
    }

    #[test]
    fn repeated_integrity_derivation_is_idempotent_and_public_redeclaration_is_authored() {
        let mut builder = builder();
        builder.derive_integrity();
        let generated = builder.declared_invariants().to_vec();
        builder.derive_integrity();
        assert_eq!(builder.declared_invariants(), generated);
        let mut copied = generated[0].clone();
        copied.name = "unique:custom".into();
        copied.query = "SELECT id FROM authored.example WHERE id < 0".into();
        builder.declare_invariant(copied);
        assert_eq!(
            builder.declared_invariants().last().unwrap().origin(),
            &InvariantOrigin::AuthoredQuery
        );
        assert!(builder.build().is_ok());
    }

    #[test]
    fn an_identical_manual_copy_cannot_claim_generated_provenance() {
        let mut builder = builder();
        builder.derive_integrity();
        builder.declare_invariant(builder.declared_invariants()[0].clone());
        assert!(matches!(
            builder.build(),
            Err(crate::SchemaError::DuplicateDeclaration {
                kind: "invariant",
                ..
            })
        ));
    }

    #[test]
    fn current_version_replaces_generated_bindings_and_preserves_authored_queries() {
        for derive_before_upgrade in [false, true] {
            for latest_declared_first in [false, true] {
                let mut builder = builder();
                builder.relations[0] = builder.relations[0].clone().unique("obsolete", &["id"]);
                if derive_before_upgrade {
                    builder.derive_integrity();
                }
                let latest = RelationDecl::new(
                    Namespace::Authored,
                    "example",
                    2,
                    Authority::Authored,
                    SnapshotClass::Model,
                    "Current relation",
                )
                .pk(&["new_id"])
                .columns(vec![
                    FieldContract::key(
                        "new_id",
                        FieldContract::native(arrow_schema::DataType::Int64),
                        "New key",
                    ),
                    FieldContract::payload(
                        "slot",
                        FieldContract::native(arrow_schema::DataType::Int64),
                        "Current value",
                    ),
                ])
                .unique("current", &["slot"]);
                builder.declare_relation(latest);
                if latest_declared_first {
                    builder.relations.reverse();
                }
                builder.declare_invariant(InvariantDecl::error(
                    "authored.example",
                    "custom",
                    InvariantKind::Domain,
                    "SELECT new_id FROM authored.example WHERE new_id < 0",
                    vec!["authored.example".into()],
                    vec!["new_id"],
                    "Independent authored query",
                ));
                let registry = builder.build().unwrap();
                assert_eq!(
                    registry.relations().len(),
                    2,
                    "migration declarations remain intact"
                );
                assert_eq!(
                    registry.relation("authored.example").unwrap().key.version,
                    2
                );
                assert_eq!(registry.invariants().len(), 3);
                for invariant in registry.invariants() {
                    match invariant.origin() {
                        InvariantOrigin::AuthoredQuery => assert_eq!(invariant.name, "custom"),
                        InvariantOrigin::GeneratedIntegrity(binding) => {
                            assert_eq!(binding.relation.version, 2);
                            assert_eq!(invariant.key_columns, ["new_id"]);
                            assert!(!invariant.query.contains("\"id\""));
                            match &binding.derivation {
                                IntegrityDerivation::PrimaryKey => {
                                    assert_eq!(invariant.name, "unique:pk")
                                }
                                IntegrityDerivation::UniqueKey(name) => assert_eq!(name, "current"),
                                other => panic!("unexpected current-version binding: {other:?}"),
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_origin_does_not_enter_durable_metadata_or_fingerprint() {
        let mut registry = builder().build().unwrap();
        let original = registry.fingerprint();
        let before = super::super::native::materialize(&registry).unwrap();
        for invariant in &mut registry.invariants {
            invariant.origin = InvariantOrigin::AuthoredQuery;
        }
        let after = super::super::native::materialize(&registry).unwrap();
        assert_eq!(before, after);
        assert_eq!(crate::fingerprint::registry(&after).unwrap(), original);
    }
}
