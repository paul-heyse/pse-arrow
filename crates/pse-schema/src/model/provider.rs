// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Canonical operation effects and scoped provider policy declarations (§5.4).
//! Native providers implement support; these declarations never manufacture it.

use pse_ids::SemanticId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Effects of an invocation, independent of the native operator/function family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OperationEffect {
    /// Read captured relation inputs.
    Read,
    /// Read external state whose capture belongs to the invocation.
    Observe,
    /// Use time, randomness or another explicitly varying input.
    Nondeterministic,
    /// Construct or change private candidate data.
    Write,
    /// Change an attempt's namespace/configuration generation.
    Namespace,
    /// Write external artifacts or conditionally publish authoritative visibility.
    Publish,
}
impl OperationEffect {
    /// Canonical effect vocabulary; it does not enumerate eligible engine features.
    pub const ALL: [Self; 6] = [
        Self::Read,
        Self::Observe,
        Self::Nondeterministic,
        Self::Write,
        Self::Namespace,
        Self::Publish,
    ];
    /// Stable metadata spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Observe => "observe",
            Self::Nondeterministic => "nondeterministic",
            Self::Write => "write",
            Self::Namespace => "namespace",
            Self::Publish => "publish",
        }
    }
}

/// Why a product invocation runs. The selected scope may restrict these effects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationPurpose {
    /// Inspect an admitted generation without external observation or mutation.
    Inspect,
    /// Execute an analytical observation over explicitly bound sources.
    #[default]
    Query,
    /// Resolve/admit external inputs before exposing handles.
    Resolve,
    /// Deterministic construction into a private attempt.
    Construct,
    /// Change private candidate data or namespace state.
    Mutate,
    /// Explicit external output or conditional authoritative publication.
    Publish,
}
impl OperationPurpose {
    /// The effects inherent in the declared purpose, before scoped restrictions.
    pub fn effects(self) -> BTreeSet<OperationEffect> {
        use OperationEffect as E;
        match self {
            Self::Inspect => [E::Read].into_iter().collect(),
            Self::Query => [E::Read, E::Observe, E::Nondeterministic]
                .into_iter()
                .collect(),
            Self::Resolve => [E::Read, E::Observe].into_iter().collect(),
            Self::Construct => [E::Read, E::Write].into_iter().collect(),
            Self::Mutate => [
                E::Read,
                E::Observe,
                E::Nondeterministic,
                E::Write,
                E::Namespace,
            ]
            .into_iter()
            .collect(),
            Self::Publish => E::ALL.into_iter().collect(),
        }
    }
}

/// Exact canonical name scope. SQL normalization occurs before these values bind.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderScope {
    /// Shared root contract.
    Root,
    /// One catalog name.
    Catalog(String),
    /// Catalog and schema names.
    Schema(String, String),
    /// Catalog, schema and table names.
    Table(String, String, String),
    /// Explicit operation selection, strongest default precedence.
    Invocation,
}
impl ProviderScope {
    /// Precedence applies to defaults only; facts/requirements/effects compose separately.
    pub const fn precedence(&self) -> u8 {
        match self {
            Self::Root => 0,
            Self::Catalog(_) => 1,
            Self::Schema(..) => 2,
            Self::Table(..) => 3,
            Self::Invocation => 4,
        }
    }
    /// Whether this declaration participates in a resolved table binding.
    pub fn covers(&self, catalog: &str, schema: &str, table: &str) -> bool {
        match self {
            Self::Root | Self::Invocation => true,
            Self::Catalog(c) => c == catalog,
            Self::Schema(c, s) => c == catalog && s == schema,
            Self::Table(c, s, t) => c == catalog && s == schema && t == table,
        }
    }
}

/// One authoritative scoped policy. Effective policy and introspection are derived.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderPolicy {
    /// Canonical declaration identity; replacing it requires a new bound generation.
    pub id: SemanticId,
    /// Where the policy applies.
    pub scope: ProviderScope,
    /// Invariant IDs whose obligations conjoin with inherited requirements.
    pub requirements: BTreeSet<SemanticId>,
    /// Effect ceiling. This cannot create provider implementation support.
    pub effects: BTreeSet<OperationEffect>,
    /// Overridable native configuration defaults.
    pub defaults: BTreeMap<String, String>,
    /// Required settings, which defaults cannot override.
    pub required_settings: BTreeMap<String, String>,
    /// Accounted child allocation ceiling under the shared parent runtime.
    pub max_bytes: Option<usize>,
}
impl ProviderPolicy {
    /// An unrestricted declaration, ready to state its actual requirements.
    pub fn new(id: SemanticId, scope: ProviderScope) -> Self {
        Self {
            id,
            scope,
            requirements: BTreeSet::new(),
            effects: OperationEffect::ALL.into_iter().collect(),
            defaults: BTreeMap::new(),
            required_settings: BTreeMap::new(),
            max_bytes: None,
        }
    }
}
