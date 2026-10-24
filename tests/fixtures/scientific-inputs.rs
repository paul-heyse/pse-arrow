// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read the invocation's immutable scientific capsule, never the mutable checkout.
#![allow(
    dead_code,
    reason = "different test binaries consume different capsule projections"
)]
use pse_authoring::language::{Declaration, IdentityPolicy, parse};
use pse_ids::SemanticId;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

const MARKER: &str = "// pse-test-source:";
#[derive(Deserialize)]
struct Family {
    packages: Vec<String>,
    documents: Vec<String>,
    #[serde(default)]
    substitutions: Vec<String>,
}
#[derive(Deserialize)]
struct Capsule {
    schema: String,
    files: BTreeMap<String, Vec<u8>>,
    families: BTreeMap<String, Family>,
}
fn capsule() -> &'static Capsule {
    static CAPTURE: OnceLock<Capsule> = OnceLock::new();
    CAPTURE.get_or_init(|| {
        let path = std::env::var_os("PSE_SCIENTIFIC_INPUTS").expect(
            "scientific inputs require explicit preparation: scripts/pse-env -- python scripts/scientific_inputs.py capture <output>; bind PSE_SCIENTIFIC_INPUTS to that capsule",
        );
        let bytes = std::fs::read(path).expect("read captured scientific inputs");
        let input: Capsule = serde_json::from_slice(&bytes).expect("decode scientific capsule");
        assert_eq!(input.schema, "scientific-inputs-v1", "unknown scientific input contract");
        input
    })
}

/// Exact captured bytes for an independently consumed source or frozen data bank.
pub(crate) fn file_bytes(logical_path: &str) -> Vec<u8> {
    capsule()
        .files
        .get(logical_path)
        .expect("captured scientific file")
        .clone()
}

/// Complete authenticated package transport with paths relative to the package root.
/// Header-only inventory entries are not a captured package closure.
pub(crate) fn package_documents(logical_root: &str) -> BTreeMap<String, Vec<u8>> {
    let input = capsule();
    assert!(
        input
            .families
            .values()
            .any(|family| family.packages.iter().any(|root| root == logical_root)),
        "package is outside every captured family closure: {logical_root}"
    );
    let prefix = format!("{logical_root}/");
    let documents = input
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
pub(crate) fn family_packages(name: &str) -> Vec<(String, BTreeMap<String, Vec<u8>>)> {
    capsule()
        .families
        .get(name)
        .expect("declared scientific family")
        .packages
        .iter()
        .map(|path| (path.clone(), package_documents(path)))
        .collect()
}

pub(crate) fn family_documents(name: &str) -> Vec<(String, String)> {
    let input = capsule();
    let family = input
        .families
        .get(name)
        .expect("declared scientific family");
    family
        .documents
        .iter()
        .chain(&family.substitutions)
        .map(|path| {
            let bytes = input.files.get(path).expect("captured source path");
            (
                path.clone(),
                String::from_utf8(bytes.clone()).expect("modeling source is UTF-8"),
            )
        })
        .collect()
}

/// Mark exact captured sources so mutation tests retain separate document attribution.
pub(crate) fn family_sources(name: &str) -> String {
    let mut text = String::new();
    for (path, source) in family_documents(name) {
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

/// Synthetic declaration controls retain path-derived source identity. Authentic transport
/// uses the production document loader and its package-derived document identities.
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
