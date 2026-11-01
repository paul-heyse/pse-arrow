// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive grammar information from the exact linked grammar packages.
use super::QueryError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A linked grammar; this selection never acquires a source or analyzer.
#[derive(Clone, Copy, Debug, clap::ValueEnum, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GrammarLanguage {
    /// The grammar used by Rust structural search.
    Rust,
    /// The grammar used by Bash source investigation.
    Bash,
}

/// Native `NODE_TYPES` data, qualified by its linked producer rather than a local taxonomy.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GrammarReport {
    /// Explicit linked language.
    pub language: GrammarLanguage,
    /// Producer package and exact version.
    pub grammar: String,
    /// Linked parser runtime and exact version.
    pub runtime: String,
    /// Exact requested type name, if filtered.
    pub node_kind: Option<String>,
    /// Total entries in the grammar before exact filtering.
    pub total_node_types: usize,
    /// Unmodified native entries, including fields, children and subtypes when present.
    pub node_types: Vec<serde_json::Value>,
}

/// Read the compiled-in native grammar table; no filesystem or provider preparation occurs.
pub fn grammar(
    language: GrammarLanguage,
    node_kind: Option<&str>,
) -> Result<GrammarReport, QueryError> {
    let (package, data) = match language {
        GrammarLanguage::Rust => ("tree-sitter-rust/0.24.2", tree_sitter_rust::NODE_TYPES),
        GrammarLanguage::Bash => ("tree-sitter-bash/0.25.1", tree_sitter_bash::NODE_TYPES),
    };
    let mut node_types: Vec<serde_json::Value> = serde_json::from_str(data)?;
    let total_node_types = node_types.len();
    if let Some(selected) = node_kind {
        node_types
            .retain(|node| node.get("type").and_then(serde_json::Value::as_str) == Some(selected));
        if node_types.is_empty() {
            return Err(QueryError::Unavailable(format!(
                "node kind {selected:?} is absent from {package}; omit --node-kind to inspect its native NODE_TYPES"
            )));
        }
    }
    Ok(GrammarReport {
        language,
        grammar: package.into(),
        runtime: "tree-sitter/0.27.0".into(),
        node_kind: node_kind.map(str::to_owned),
        total_node_types,
        node_types,
    })
}
