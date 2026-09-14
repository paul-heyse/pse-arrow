// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The Python generator: contract classes, the `pyarrow` extension types and the manifest
//! struct (blueprint §4.2, §21, ADR-0051).
//!
//! Writes `python/pse/contracts/` and its `GENERATED.sha256`. The manifest is generated
//! from [`crate::model::ManifestSpec`] rather than hand-written, so the envelope the Python
//! side parses and the envelope the Rust side writes are one declaration.
//!
//! Generated Python must pass the `typing.Any` lint and must not use
//! `from __future__ import annotations`: PEP 563 turns annotations into strings and breaks
//! the import-time check in `pse.governance`.
//!
//! Packet A-6's Python sibling fills this module.
