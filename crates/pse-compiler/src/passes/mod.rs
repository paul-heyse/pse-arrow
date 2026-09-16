// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The pass contract: what every pass is, and what it is handed (blueprint §14.1, §14.3).
//!
//! A pass is a `reference.pass_specs` row first and a Rust type second. [`Pass::spec`]
//! returns the declaration, so the implementation cannot quietly read or write a port the
//! registry does not know about. Ports bind relational dependencies; [`PassContext`]
//! carries explicitly selected document, policy and engine inputs.
//!
//! # Why absence is a value
//!
//! [`InputBundle::ports`] maps every declared port to `Option<BoundInput>`, and the `None`
//! enters the stage key. "This optional port was absent" and "this port was never asked
//! for" produce different keys, because a memo hit that conflated them would reuse a result
//! computed from different inputs (§14.3, ADR-0041).

pub mod bundle;
pub mod dag;
pub mod key;
pub(crate) mod native_construction;
mod native_graph;
pub(crate) mod native_outputs;
pub(crate) mod native_rows;
pub(crate) mod native_sources;
#[cfg(test)]
mod native_test;
pub mod p0;
pub mod p1;
pub mod p10;
pub mod p2;
pub mod p3;
pub mod p4;
pub mod p5;
pub mod p6;
pub mod p7;
pub mod p8;
pub mod p9;
pub(crate) mod parameter_indices;
pub mod registry;

use std::collections::BTreeMap;

use pse_catalog::{LoadedRelation, Snapshot};
use pse_ids::{
    CancellationToken, ContentHash, LogicalHash, MemoryReserver, SchemaVersion, SemanticId,
};
use pse_schema::Registry;
use pse_schema::model::PassSpec;
use std::sync::Arc;

use crate::error::CompilerError;

/// One compiler stage (blueprint §14.1).
pub trait Pass: Send + Sync {
    /// The declaration this pass implements.
    fn spec(&self) -> &PassSpec;

    /// Run over actual bound inputs and return the catalog's producer result.
    /// The catalog executor checks complete output obligations before creating a
    /// publication completion. Attempt identity and status belong to the Driver.
    ///
    /// # Errors
    ///
    /// [`CompilerError`] — including the authoring, rule and registry errors it forwards
    /// transparently.
    fn run<'a>(
        &'a self,
        ctx: &'a PassContext<'a>,
        inputs: &'a InputBundle,
    ) -> pse_catalog::provider::BoxFut<
        'a,
        Result<pse_catalog::computation::ProducedStage, CompilerError>,
    >;
}

/// Everything a pass may read that is not one of its input ports (blueprint §14.3).
///
/// Deliberately small. A pass that reached for ambient state — the clock, an environment
/// variable, a global session — would not be a function of its declared inputs, and its
/// stage key would be a lie.
#[derive(Debug)]
pub struct PassContext<'a> {
    /// Shared physical algorithm inventory for this exact bound input set.
    /// Absent only in control-only contexts that do not run a physical pass.
    pub physical: Option<&'a Arc<crate::quantity_relations::PhysicalInventory>>,
    /// The registry every contract is read from.
    pub registry: &'a Arc<Registry>,
    /// Original documents reopened from the exact bound snapshot source artifacts.
    pub documents: &'a pse_authoring::document::OwnedDocumentSet,
    /// Policies resolved to actual admitted rows.
    pub policies: &'a PolicySet,
    /// Cooperative cancellation at every bounded work boundary.
    pub cancel: &'a CancellationToken,
    /// The reservation provider used by store and session.
    pub reserver: &'a dyn MemoryReserver,
    /// Bound native environment required by every operation.
    pub session: &'a pse_catalog::session::SnapshotSession,
}
impl PassContext<'_> {
    /// Actual shared physical inventory established from this invocation's Arrow inputs.
    /// # Errors
    /// A physical pass was invoked without preparing its actual input inventory.
    pub fn physical(&self) -> Result<&crate::quantity_relations::PhysicalInventory, CompilerError> {
        self.physical
            .map(AsRef::as_ref)
            .ok_or_else(|| dag::invalid("physical inventory absent from prepared pass context"))
    }
}

/// Policies selected from actual admitted rows.
#[derive(Clone, Debug, Default)]
pub struct PolicySet(pub BTreeMap<String, PolicyBinding>);

/// A selected identity within a declared policy relation.
#[derive(Clone, Debug)]
pub struct PolicyBinding {
    /// Selected semantic identity; admission checks actual membership.
    pub policy_id: SemanticId,
    /// Complete relation containing the selected policy.
    pub input: BoundInput,
}
impl PolicySet {
    /// No selected policies.
    pub fn new() -> Self {
        Self::default()
    }
    /// Actual selected policy binding.
    pub fn get(&self, name: &str) -> Option<&PolicyBinding> {
        self.0.get(name)
    }
}

/// One exact immutable relation handle, minted only from an admitted snapshot.
#[derive(Clone, Debug)]
pub struct BoundInput {
    pub(crate) snapshot: Arc<Snapshot>,
    pub(crate) relation: Arc<LoadedRelation>,
}
impl BoundInput {
    /// The exact admitted snapshot, including its manifest checksum and parents.
    pub fn snapshot(&self) -> &Arc<Snapshot> {
        &self.snapshot
    }
    /// Actual schema and rows retained for semantic dependency comparison.
    pub fn relation(&self) -> &Arc<LoadedRelation> {
        &self.relation
    }
    /// Declared relation identity.
    pub fn relation_id(&self) -> SemanticId {
        self.relation.contract().canonical.relation_id
    }
    /// Declared relation version.
    pub fn schema_version(&self) -> SchemaVersion {
        self.relation.contract().canonical.schema_version
    }
    /// Independently admitted logical content identity, used only as a lookup key.
    pub fn logical_hash(&self) -> LogicalHash {
        self.relation.member().logical_hash
    }
}

/// Every declared input port of a pass, bound or explicitly absent.
#[derive(Clone, Debug, Default)]
pub struct InputBundle {
    /// Port name to its binding. `None` is an explicit absence and enters the stage key.
    pub ports: BTreeMap<&'static str, Option<BoundInput>>,
}

impl InputBundle {
    /// An empty bundle.
    pub fn new() -> Self {
        Self::default()
    }

    /// The binding of `port`, distinguishing "absent" from "not declared".
    ///
    /// `Some(None)` is a declared port with no rows; `None` is a port this pass does not
    /// declare, which is a caller error rather than an empty input.
    pub fn port(&self, port: &str) -> Option<&Option<BoundInput>> {
        self.ports.get(port)
    }
}

pub use pse_schema::model::PassStatus;

/// A stage memo key (blueprint §14.3, ADR-0041).
///
/// A newtype over the digest for the same reason `LogicalHash` and `EncodingChecksum` are
/// newtypes over theirs: the compiler refuses the substitution the design refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StageKey(pub ContentHash);

impl StageKey {
    /// The digest underneath.
    pub const fn content_hash(&self) -> ContentHash {
        self.0
    }
}

impl core::fmt::Display for StageKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
