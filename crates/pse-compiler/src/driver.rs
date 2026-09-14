// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The pipeline driver (blueprint §14.3).
//!
//! Owns the catalog and the shared runtime, commits document bundles and runs the declared
//! pass DAG through a requested stage. `miette::Result` and the graphical reporter live
//! here and in the CLI, never in a library crate (blueprint §23.2).
//!
//! Packet C-4 fills this module.
