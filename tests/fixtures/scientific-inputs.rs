// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read the invocation's immutable scientific capsule, never the mutable checkout.
#![allow(
    dead_code,
    reason = "different test binaries consume different capsule projections"
)]
use std::{collections::BTreeMap, sync::OnceLock};

#[path = "scientific-input-data.rs"]
mod data;
use data::Capsule;

fn capsule() -> &'static Capsule {
    static CAPTURE: OnceLock<Capsule> = OnceLock::new();
    CAPTURE.get_or_init(|| {
        let path = std::env::var_os("PSE_SCIENTIFIC_INPUTS").expect(
            "scientific inputs require explicit preparation: scripts/pse-env -- scripts/cargo-bin/cargo-pse-correctness run -p xtask --bin xtask --features physical-admission --locked -- scientific-inputs tests/fixtures/scientific-inputs.toml <output>; bind PSE_SCIENTIFIC_INPUTS to that capsule",
        );
        let bytes = std::fs::read(path).expect("read captured scientific inputs");
        let input: Capsule = serde_json::from_slice(&bytes).expect("decode scientific capsule");
        assert_eq!(input.schema, data::SCHEMA, "unknown scientific input contract");
        input
    })
}

pub(crate) fn file_bytes(logical_path: &str) -> Vec<u8> {
    capsule().file_bytes(logical_path)
}

pub(crate) fn package_documents(logical_root: &str) -> BTreeMap<String, Vec<u8>> {
    capsule().package_documents(logical_root)
}

pub(crate) fn family_packages(name: &str) -> Vec<(String, BTreeMap<String, Vec<u8>>)> {
    capsule().family_packages(name)
}

/// Validate the retained transport consumed by the benchmark without cloning its bytes.
pub(crate) fn require_package(logical_root: &str) {
    let capture = capsule();
    assert!(
        capture
            .families
            .values()
            .any(|family| family.packages.iter().any(|root| root == logical_root)),
        "package is outside the captured scientific transport: {logical_root}"
    );
    assert!(
        capture
            .files
            .contains_key(&format!("{logical_root}/package.toml")),
        "captured package manifest: {logical_root}"
    );
    let registry = pse_schema::shared_registry().expect("captured document registry");
    let mut documents = 0;
    for document in registry.documents() {
        let Some((directory, suffix)) = document.path_glob.split_once("/*") else {
            continue;
        };
        let directory = format!("{logical_root}/{directory}");
        let members = capture
            .directories
            .get(&directory)
            .expect("captured package directory inventory");
        for member in members
            .iter()
            .flatten()
            .filter(|name| name.ends_with(suffix))
        {
            assert!(
                capture.files.contains_key(&format!("{directory}/{member}")),
                "captured package document: {directory}/{member}"
            );
            documents += 1;
        }
    }
    assert!(
        documents != 0,
        "captured package has no transport inventory: {logical_root}"
    );
}

/// Validate the actual ordered transport family consumed by a selected seed benchmark.
pub(crate) fn require_transport_family(name: &str, expected_root: &str) {
    let capture = capsule();
    let family = capture
        .families
        .get(name)
        .expect("captured scientific transport family");
    assert_eq!(family.projection.as_deref(), Some("authentic-transport"));
    assert!(
        family.packages.iter().any(|root| root == expected_root),
        "selected transport lacks its seed package: {expected_root}"
    );
    assert!(
        !family.documents.is_empty(),
        "selected transport lacks authored documents: {name}"
    );
    for root in &family.packages {
        require_package(root);
    }
    for document in &family.documents {
        assert!(
            capture.files.contains_key(document),
            "missing selected transport document: {document}"
        );
    }
}
