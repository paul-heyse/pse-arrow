// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned facts crossing the bounded relationship operation's snapshot boundary.
use super::{AnalysisContext, NativeLocation, Query};
use crate::investigation::{Direction, Relationship, graph};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Normalized relationship intent, not a second query language.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GraphQuery {
    /// Guarded source positions, resolved inside this operation's snapshot.
    pub seeds: Vec<Query>,
    /// Selected canonical relation meanings.
    pub relations: Vec<Relationship>,
    /// Walk direction; exported edge direction remains canonical.
    pub direction: Direction,
    /// Explicit working sources admitted as seeds or intermediates.
    pub traversal_paths: Vec<PathBuf>,
    /// Ordinary result selection, separate from supporting evidence.
    pub output_paths: Vec<PathBuf>,
    /// Intentional maximum edge distance.
    pub depth: usize,
    /// Distinct node allowance, including seeds.
    pub max_nodes: usize,
    /// Examined edge-observation allowance.
    pub max_edges: usize,
    /// Any seed or all seeds must reach an ordinary answer.
    pub seed_match: graph::SeedMatch,
    /// Stop after the first established selected positive when only existence is demanded.
    pub exists: bool,
}
/// Original declaration or occurrence address qualified by the enclosing answer world.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GraphNode {
    /// True only for a native declaration; a use-site address does not establish one.
    pub declaration: bool,
    /// Exact original native bytes; no HIR identity crosses this boundary.
    pub location: NativeLocation,
    /// Presentation only, never an identity join.
    pub name: Option<String>,
    /// Original snapshot source guard.
    pub source_sha256: String,
}
/// Structural role; separate positions preserve parallel witnessed edges.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GraphRole {
    /// Parent declaration contains the destination declaration.
    Containment,
    /// Native parameter index includes a receiver when present.
    Parameter {
        /// Native parameter position, including a receiver if present.
        index: usize,
    },
    /// Async output differs from the function's declared future return.
    Return {
        /// True for the independently observed output of an async future.
        async_output: bool,
    },
    /// Field position, including enum variant fields.
    Field {
        /// Field position in its native struct, union or enum variant.
        index: usize,
        /// Original native field name when supplied.
        name: Option<String>,
    },
    /// Explicit resolved use role; implicit language operations are a separate meaning.
    Reference {
        /// Native read role when the provider supplies it.
        read: Option<bool>,
        /// Native write role when the provider supplies it.
        write: Option<bool>,
        /// Native import role, separate from runtime execution.
        import: Option<bool>,
        /// Native test context, separate from test execution.
        test: Option<bool>,
    },
    /// Existing native navigation meaning with no additional structural role.
    Navigation,
}
/// Original witness and type context, separate from the declaration endpoints.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GraphWitness {
    /// Original occurrence when the provider maps one precisely.
    pub location: Option<NativeLocation>,
    /// Exact original snapshot digest for a mapped occurrence.
    pub source_sha256: Option<String>,
    /// Structural/native role.
    pub role: GraphRole,
    /// Declaration-owned type rendering, never parsed for relationships.
    pub declared_type: Option<String>,
    /// Contextually instantiated rendering only when independently supplied.
    pub instantiated_type: Option<String>,
    /// Mapping/type limits; missing evidence is not a negative fact.
    pub limitations: Vec<String>,
}
/// Final bounded graph; integer addresses are local to this one response.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GraphAnswer {
    /// Original prepared provider/configuration owner.
    pub session: AnalysisContext,
    /// Original immutable snapshot generation.
    pub generation: u64,
    /// Shared supporting and selected native nodes.
    pub nodes: Vec<GraphNode>,
    /// Shared original witnesses; equal endpoints do not collapse these.
    pub witnesses: Vec<GraphWitness>,
    /// Pure kernel attribution, coverage and frontier.
    pub traversal: graph::TraversalResult,
    /// Provider/extraction limits affecting this answer.
    pub limitations: Vec<String>,
}
