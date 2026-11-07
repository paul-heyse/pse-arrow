// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read and verify authored scientific inputs once per test process.
#![allow(dead_code, reason = "each test target consumes its own scientific projection")]

use std::{collections::BTreeMap, path::Path, sync::OnceLock};

#[path = "scientific-input-capture.rs"]
mod owner;
use owner::data::Capsule;
#[allow(unused_imports, reason = "the PID exchange consumes package transport only")]
pub(crate) use owner::data::parse_sources;

fn inputs() -> &'static Capsule {
    static INPUTS: OnceLock<Capsule> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("resolve authored scientific checkout");
        owner::capture(&root, Path::new("tests/fixtures/scientific-inputs.toml"))
            .expect("capture current authored scientific inputs")
    })
}

pub(crate) fn package_documents(logical_root: &str) -> BTreeMap<String, Vec<u8>> {
    inputs().package_documents(logical_root)
}

pub(crate) fn family_packages(name: &str) -> Vec<(String, BTreeMap<String, Vec<u8>>)> {
    inputs().family_packages(name)
}

pub(crate) fn family_sources(name: &str) -> String {
    inputs().family_sources(name)
}
