// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Named structures (Plan 22 X11).
//!
//! A struct value may carry a presentation name ([`FieldContract::named`]). Every
//! occurrence of one name, in any relation and at any depth, must declare the same
//! contract; the generators then emit the named structure once and every relation
//! references it. The name is presentation only: two occurrences are compared under
//! the execution-identity projection, which excludes prose and the name itself.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use pse_columnar::native_field::{MetadataPurpose, project};

use crate::SchemaError;
use crate::model::{FieldContract, RelationSpec};

/// Collect every named structure, refusing two different contracts under one name.
///
/// # Errors
/// Two occurrences of one name whose contracts differ, or an invalid native field.
pub(super) fn resolve(
    relations: &[RelationSpec],
) -> Result<BTreeMap<String, FieldContract>, SchemaError> {
    let mut structures: BTreeMap<String, (FieldContract, String, String)> = BTreeMap::new();
    for spec in relations {
        for column in &spec.columns {
            let mut fields = Vec::new();
            column.walk(&mut fields);
            for field in fields {
                let Some(name) = field.structure_name() else {
                    continue;
                };
                let contract = field.value_type();
                let identity = identity(&contract)?;
                let site = format!("{}.{}", spec.qualified_name(), column.name());
                match structures.entry(name.to_owned()) {
                    Entry::Vacant(entry) => {
                        entry.insert((contract, identity, site));
                    }
                    Entry::Occupied(entry) => {
                        let (_, existing, first) = entry.get();
                        if *existing != identity {
                            return Err(crate::checks::invalid(
                                format!("named structure {name}"),
                                format!(
                                    "{site} declares a contract different from {first}; one name has one contract"
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(structures
        .into_iter()
        .map(|(name, (contract, _, _))| (name, contract))
        .collect())
}

/// The canonical contract of a structure: its value type without prose or name.
fn identity(contract: &FieldContract) -> Result<String, SchemaError> {
    let projected = project(contract.field(), MetadataPurpose::ExecutionIdentity)
        .map_err(|error| crate::checks::invalid("named structure", error.to_string()))?;
    crate::model::field::render_field(&projected)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "assertions over small test registries")]

    use crate::RegistryBuilder;
    use crate::model::{Authority, FieldContract as F, Namespace, RelationDecl, SnapshotClass};
    use arrow_schema::DataType;

    fn pair(second: DataType) -> F {
        F::structure(vec![
            F::native(DataType::Utf8).with_name("name"),
            F::native(second).with_name("value"),
        ])
    }

    fn relation(name: &'static str, value: F) -> RelationDecl {
        RelationDecl::new(
            Namespace::Authored,
            name,
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Named structure test relation",
        )
        .pk(&["id"])
        .columns(vec![
            F::key("id", F::native(DataType::Int64), "Key"),
            F::payload("values", F::list(value), "Values"),
        ])
    }

    fn registry(values: F) -> crate::Registry {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(relation("pairs", values));
        builder.build().unwrap()
    }

    #[test]
    fn structure_name_is_presentation_only() {
        let named = registry(pair(DataType::Int64).named("Pair"));
        let plain = registry(pair(DataType::Int64));
        let spec =
            |registry: &crate::Registry| registry.relation("authored.pairs").unwrap().clone();
        let (left, right) = (spec(&named), spec(&plain));
        // The relation contract fingerprint, the semantic description and the physical
        // encoding are all unchanged by naming the structure.
        assert_eq!(left.fingerprint, right.fingerprint);
        assert_eq!(
            crate::fingerprint::semantic_description(&named, &left).unwrap(),
            crate::fingerprint::semantic_description(&plain, &right).unwrap()
        );
        assert_eq!(
            crate::fingerprint::encoding_relation(&named, &left).unwrap(),
            crate::fingerprint::encoding_relation(&plain, &right).unwrap()
        );
        // The name is recorded once, with its one contract.
        assert_eq!(named.structures().keys().collect::<Vec<_>>(), ["Pair"]);
        assert!(plain.structures().is_empty());
        assert_eq!(named.structures()["Pair"].children().len(), 2);
        // Only a PascalCase name on a native struct is admitted.
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(relation("bad", pair(DataType::Int64).named("pair")));
        assert!(builder.build().is_err());
    }

    #[test]
    fn named_structure_conflict_rejected() {
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(relation("first", pair(DataType::Int64).named("Pair")));
        builder.declare_relation(relation("second", pair(DataType::Utf8).named("Pair")));
        let error = builder.build().unwrap_err().to_string();
        assert!(error.contains("named structure Pair"), "{error}");
        // The same contract under one name in two relations is one structure.
        let mut builder = RegistryBuilder::new();
        builder.declare_relation(relation("first", pair(DataType::Int64).named("Pair")));
        builder.declare_relation(relation("second", pair(DataType::Int64).named("Pair")));
        assert_eq!(builder.build().unwrap().structures().len(), 1);
    }
}
