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

/// Attribution of operational declarations to independently versioned histories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryOwner {
    /// Publication inventory, catalog control and reader protection.
    Catalog,
    /// Attempts, workers, queues, native streams and reusable solutions.
    Operations,
}
/// One owning history per physical table; references consume keys, not target vocabulary.
pub fn history_owner(table: &str) -> HistoryOwner {
    match table {
        "workspaces"
        | "publication_intents"
        | "publications"
        | "publication_heads"
        | "publication_members"
        | "publication_windows"
        | "reader_leases"
        | "retention_marks"
        | "settlements"
        | "schema_support_state" => HistoryOwner::Catalog,
        _ => HistoryOwner::Operations,
    }
}
