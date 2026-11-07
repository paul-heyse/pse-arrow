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
#[allow(unused_imports, reason = "transport-only consumers do not parse declarations")]
pub(crate) use data::parse_sources;

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

pub(crate) fn family_documents(name: &str) -> Vec<(String, String)> {
    capsule().family_documents(name)
}

pub(crate) fn family_sources(name: &str) -> String {
    capsule().family_sources(name)
}
