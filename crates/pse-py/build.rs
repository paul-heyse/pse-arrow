// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Expose the selected interpreter configuration to the Python boundary.

fn main() {
    // Publish the actual interpreter/ABI configuration, including free-threading.
    // The local-replay environment owner must never infer a GIL from abi3 features.
    pyo3_build_config::use_pyo3_cfgs();
}
