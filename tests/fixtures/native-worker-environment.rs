// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A production scientific child does not consume the test runner's environment protocol.
pub(crate) fn selected() -> std::collections::BTreeMap<String, String> {
    std::env::vars()
        .filter(|(key, _)| !key.starts_with("NEXTEST_") && !key.starts_with("CARGO_BIN_EXE_"))
        .collect()
}
