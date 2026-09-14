// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The pass contract: what every pass is, and what it is handed (blueprint §14.1, §14.3).
//!
//! A pass is a `reference.pass_specs` row first and a Rust type second. [`Pass::spec`]
//! returns the declaration, so the implementation cannot quietly read or write a port the
//! registry does not know about — the ports are the compiler's only dependency edges, and
//! a read without a declared input binding is invalid.
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
pub mod p0;
pub mod p1;
pub mod p10;
pub mod p2;
pub mod p3;
pub mod registry;

use std::collections::BTreeMap;
use std::time::Duration;

use pse_ids::{ContentHash, LogicalHash, SchemaVersion, SemanticId, SnapshotId};
use pse_relations::RecordBatch;
use pse_schema::Registry;
use pse_schema::model::PassSpec;

use crate::error::CompilerError;

/// One compiler stage (blueprint §14.1).
pub trait Pass: Send + Sync {
    /// The declaration this pass implements.
    fn spec(&self) -> &'static PassSpec;

    /// Runs the pass over its bound inputs.
    ///
    /// # Errors
    ///
    /// [`CompilerError`] — including the authoring, rule and registry errors it forwards
    /// transparently.
    fn run(&self, ctx: &PassContext<'_>, inputs: &InputBundle)
    -> Result<PassOutput, CompilerError>;
}

/// Everything a pass may read that is not one of its input ports (blueprint §14.3).
///
/// Deliberately small. A pass that reached for ambient state — the clock, an environment
/// variable, a global session — would not be a function of its declared inputs, and its
/// stage key would be a lie.
#[derive(Debug)]
pub struct PassContext<'a> {
    /// The registry every contract is read from.
    pub registry: &'a Registry,
    /// The policies in force, by name.
    pub policies: &'a PolicySet,
    /// The shared runtime. A handle, not a runtime: passes borrow the driver's, they do
    /// not build their own (blueprint §14.3).
    pub runtime: &'a tokio::runtime::Handle,
    /// Inputs supplied from outside the snapshot.
    pub external: &'a ExternalInputs,
}

/// The policies in force, by name, each identified by its content hash.
///
/// A `BTreeMap` because the set is a stage-key input and has to frame in a fixed order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PolicySet(pub BTreeMap<String, ContentHash>);

impl PolicySet {
    /// An empty policy set.
    pub fn new() -> Self {
        Self::default()
    }

    /// The hash of the named policy, if it is in force.
    pub fn get(&self, name: &str) -> Option<ContentHash> {
        self.0.get(name).copied()
    }
}

/// Inputs supplied from outside the snapshot.
///
/// Empty in phase 0. It exists as a named place so that a test fixture binding an input
/// directly is visibly *outside* the snapshot rather than indistinguishable from one that
/// came from it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExternalInputs {
    /// Bindings by port name, for test fixtures only.
    pub bindings: BTreeMap<String, SnapshotId>,
}

/// What one input port resolved to (blueprint §14.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundInput {
    /// The snapshot the relation was read from.
    pub snapshot: SnapshotId,
    /// The relation.
    pub relation_id: SemanticId,
    /// The schema version it was written under.
    pub schema_version: SchemaVersion,
    /// Its logical digest: what it means, independent of how it is stored.
    pub logical_hash: LogicalHash,
}

/// Every declared input port of a pass, bound or explicitly absent.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

/// What a pass produced (blueprint §14.1).
///
/// Every declared output port appears, empties explicit: an output bundle that omitted an
/// empty relation would be indistinguishable from one that forgot it.
#[derive(Clone, Debug)]
pub struct PassOutput {
    /// Port name to the batches written to it.
    pub ports: BTreeMap<&'static str, Vec<RecordBatch>>,
    /// Findings, as relation rows. A rendered diagnostic is a projection of a finding,
    /// never its storage (blueprint §23.2).
    pub findings: Vec<RecordBatch>,
    /// The sidecar record of this attempt.
    pub record: PassRecordDraft,
}

/// The sidecar record of one pass attempt (blueprint §6.13).
///
/// A draft: the driver fills the identity, the snapshot and the attempt number, because a
/// pass does not know how many times it has been retried.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PassRecordDraft {
    /// The pass.
    pub pass_id: SemanticId,
    /// Its version.
    pub version: &'static str,
    /// How long the attempt took.
    pub duration: Duration,
    /// How it ended.
    pub status: PassStatus,
}

/// How a pass attempt ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PassStatus {
    /// Every postcondition holds and every output port was written.
    Ok,
    /// The pass ran and reported findings that stop the pipeline.
    Failed,
    /// The stage key hit the memo; the outputs were reused, not recomputed.
    Reused,
}

impl PassStatus {
    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::Reused => "reused",
        }
    }
}

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
