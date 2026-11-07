// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable scientific fixture projections shared by the authored capture and capsule reader.
#![allow(
    dead_code,
    reason = "each test target consumes its own scientific projection"
)]

use pse_authoring::language::{Declaration, IdentityPolicy, parse};
use pse_ids::SemanticId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) const SCHEMA: &str = "scientific-inputs-v1";
const MARKER: &str = "// pse-test-source:";

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Capsule {
    pub(crate) schema: String,
    pub(crate) files: BTreeMap<String, Vec<u8>>,
    pub(crate) families: BTreeMap<String, CapturedFamily>,
    #[serde(default)]
    pub(crate) directories: BTreeMap<String, Option<Vec<String>>>,
    #[serde(default)]
    pub(crate) package_headers: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CapturedFamily {
    pub(crate) packages: Vec<String>,
    pub(crate) documents: Vec<String>,
    #[serde(default)]
    pub(crate) substitutions: Vec<String>,
    pub(crate) projection: Option<String>,
}

impl Capsule {
    /// Exact captured bytes for an independently consumed source or frozen data bank.
    pub(crate) fn file_bytes(&self, logical_path: &str) -> Vec<u8> {
        self.files
            .get(logical_path)
            .expect("captured scientific file")
            .clone()
    }

    /// Complete authenticated package transport with paths relative to the package root.
    /// Header-only inventory entries are not a captured package closure.
    pub(crate) fn package_documents(&self, logical_root: &str) -> BTreeMap<String, Vec<u8>> {
        assert!(
            self.families
                .values()
                .any(|family| family.packages.iter().any(|root| root == logical_root)),
            "package is outside every captured family closure: {logical_root}"
        );
        let prefix = format!("{logical_root}/");
        let documents = self
            .files
            .iter()
            .filter_map(|(path, bytes)| {
                path.strip_prefix(&prefix)
                    .map(|relative| (relative.to_owned(), bytes.clone()))
            })
            .collect::<BTreeMap<_, _>>();
        assert!(
            documents.contains_key("package.toml"),
            "captured package has no manifest: {logical_root}"
        );
        documents
    }

    /// The loader-derived package closure for this family; each consumer still admits it fresh.
    pub(crate) fn family_packages(&self, name: &str) -> Vec<(String, BTreeMap<String, Vec<u8>>)> {
        self.families
            .get(name)
            .expect("declared scientific family")
            .packages
            .iter()
            .map(|path| (path.clone(), self.package_documents(path)))
            .collect()
    }

    pub(crate) fn family_documents(&self, name: &str) -> Vec<(String, String)> {
        let family = self.families.get(name).expect("declared scientific family");
        family
            .documents
            .iter()
            .chain(&family.substitutions)
            .map(|path| {
                let bytes = self.files.get(path).expect("captured source path");
                (
                    path.clone(),
                    String::from_utf8(bytes.clone()).expect("modeling source is UTF-8"),
                )
            })
            .collect()
    }

    /// Mark exact captured sources so mutation tests retain separate document attribution.
    pub(crate) fn family_sources(&self, name: &str) -> String {
        let mut text = String::new();
        for (path, source) in self.family_documents(name) {
            text.push_str(MARKER);
            text.push_str(&path);
            text.push('\n');
            text.push_str(&source);
            text.push('\n');
        }
        text.push_str(MARKER);
        text.push_str("local-test-source\n");
        text
    }
}

/// Synthetic declaration controls retain path-derived source identity. Authentic transport
/// uses the production document loader and its package-derived document identities.
#[allow(
    clippy::panic,
    reason = "this fixture assertion identifies the exact authored test document that failed to parse"
)]
pub(crate) fn parse_sources(text: &str, policy: IdentityPolicy) -> Vec<Declaration> {
    let mut rows = Vec::new();
    let mut path = "local-test-source";
    let mut source = String::new();
    let append = |rows: &mut Vec<Declaration>, path: &str, source: &str| {
        if !source.trim().is_empty() {
            let document = pse_ids::named_id(SemanticId::from_bytes([194; 16]), path);
            rows.extend(
                parse(
                    source,
                    document,
                    policy,
                    pse_authoring::ParseBudget::default(),
                )
                .unwrap_or_else(|error| panic!("{path}: {error}")),
            );
        }
    };
    for line in text.split_inclusive('\n') {
        if let Some(next) = line.strip_prefix(MARKER) {
            append(&mut rows, path, &source);
            source.clear();
            path = next.trim_end();
        } else {
            source.push_str(line);
        }
    }
    append(&mut rows, path, &source);
    rows
}
