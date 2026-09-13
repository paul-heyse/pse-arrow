// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Schema, layout, pin and supply-chain governance tests (blueprint §24.1).
//!
//! Everything lives in `tests/`: these are checks about the repository, not a library.
//! They run against the working tree (never a snapshot of it) so a governance failure
//! names the file the author just wrote. `cargo xtask governance` runs this package, then
//! `codegen --check` and `family-check`, which need git and a resolved graph.
