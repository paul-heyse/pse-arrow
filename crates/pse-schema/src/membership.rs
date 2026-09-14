// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Snapshot membership, generated from the declaration (blueprint §5.3 step 7).
//!
//! Membership is not a runtime choice. `schema_relations.snapshot_class` decides it, and
//! registry admission requires a class for every relation, so there is no namespace
//! wildcard and no implicit omission: a model snapshot contains *every* relation of class
//! [`SnapshotClass::Model`] in the admitted scope, including the empty ones. An omitted
//! empty relation and a forgotten relation are the same bytes, and a snapshot ID has to
//! tell them apart.

use pse_ids::model_port_name;

use crate::builder::Registry;
use crate::model::{RelationSpec, SnapshotClass};

/// Every relation of `class`, in [`Registry::relations`] order.
///
/// ```
/// use pse_schema::{membership, model::SnapshotClass, registry};
///
/// let reg = registry()?;
/// // The registry describes itself, so its own contracts are structural model members.
/// assert!(!membership::members(reg, SnapshotClass::Model).is_empty());
/// # Ok::<(), pse_schema::SchemaError>(())
/// ```
pub fn members(reg: &Registry, class: SnapshotClass) -> Vec<&RelationSpec> {
    reg.relations()
        .iter()
        .filter(|spec| spec.snapshot_class == class)
        .collect()
}

/// A model or case member's port name: `<namespace>/<32 lowercase hex digits>`
/// (blueprint §5.3 step 7).
///
/// The relation *identity*, not its name, so that renaming a relation cannot change a port
/// and therefore cannot change a snapshot ID.
///
/// ```
/// use pse_schema::{membership, registry};
///
/// let reg = registry()?;
/// let spec = reg.relation("reference.schema_relations").expect("declared in §4.1");
/// assert_eq!(
///     membership::port_name(spec),
///     format!("reference/{}", spec.id.to_hex()),
/// );
/// # Ok::<(), pse_schema::SchemaError>(())
/// ```
pub fn port_name(spec: &RelationSpec) -> String {
    model_port_name(spec.key.namespace.as_str(), spec.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;

    #[test]
    fn every_relation_has_exactly_one_class() {
        let reg = registry().expect("the registry assembles");
        let total: usize = SnapshotClass::ALL
            .iter()
            .map(|class| members(reg, *class).len())
            .sum();
        assert_eq!(
            total,
            reg.relations().len(),
            "membership is generated from the class and admits no wildcard"
        );
    }

    #[test]
    fn port_names_are_unique() {
        let reg = registry().expect("the registry assembles");
        let mut ports: Vec<String> = reg.relations().iter().map(port_name).collect();
        let before = ports.len();
        ports.sort_unstable();
        ports.dedup();
        assert_eq!(before, ports.len(), "two relations share a member port");
    }
}
