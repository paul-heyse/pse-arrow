// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read and verify each selected authored scientific family once per test process.
#![allow(
    dead_code,
    reason = "each test target consumes its own scientific projection"
)]

use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};

#[path = "scientific-input-capture.rs"]
mod owner;
use owner::SelectedCapture;
#[allow(
    unused_imports,
    reason = "the PID exchange consumes package transport only"
)]
pub(crate) use owner::data::parse_sources;

pub(crate) fn family(name: &str) -> Arc<SelectedCapture> {
    type Slot = Arc<OnceLock<Arc<SelectedCapture>>>;
    static INPUTS: Mutex<BTreeMap<String, Slot>> = Mutex::new(BTreeMap::new());
    let slot = INPUTS
        .lock()
        .expect("scientific family selection cache")
        .entry(name.to_owned())
        .or_default()
        .clone();
    slot.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("resolve authored scientific checkout");
        Arc::new(
            owner::capture_family(
                &root,
                Path::new("tests/fixtures/scientific-inputs.toml"),
                name,
            )
            .expect("capture selected authored scientific inputs"),
        )
    })
    .clone()
}

pub(crate) fn family_documents(name: &str) -> Vec<(String, String)> {
    family(name).documents()
}

pub(crate) fn family_sources(name: &str) -> String {
    family(name).sources()
}
