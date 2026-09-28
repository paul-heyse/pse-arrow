// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The operational store's relation set (ADR-0114 Outcome 22).
//!
//! PostgreSQL owns what changes; the registry owns the meaning and the shape of every
//! operational relation. Relation `runtime.operational_<t>` is table `pse_ops.<t>`: this
//! module is the one statement of that correspondence, which the PostgreSQL generator,
//! the operational store and its tests share.

use crate::Registry;
use crate::model::{Namespace, RelationSpec};

/// The PostgreSQL schema holding the operational store.
pub const SCHEMA: &str = "pse_ops";

/// Runtime relations named `operational_<t>` are the store's tables `<t>`.
pub const RELATION_PREFIX: &str = "operational_";

/// The store table of a registry relation, when the relation belongs to the store.
pub fn table(spec: &RelationSpec) -> Option<&'static str> {
    if spec.key.namespace != Namespace::Runtime {
        return None;
    }
    spec.key.name.strip_prefix(RELATION_PREFIX)
}

/// Every store relation with its table, sorted by table name.
pub fn relations(registry: &Registry) -> Vec<(&'static str, &RelationSpec)> {
    let mut out = registry
        .relations()
        .iter()
        .filter_map(|spec| table(spec).map(|table| (table, spec)))
        .collect::<Vec<_>>();
    out.sort_by_key(|(table, _)| *table);
    out
}
