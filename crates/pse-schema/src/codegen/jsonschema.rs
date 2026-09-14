// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The authoring JSON Schema: `docs/generated/schema/authoring.schema.json`
//! (blueprint §4.2, ADR-0051).
//!
//! Generated from the registry's [`crate::model::DocumentSpec`]s so that an editor
//! validating a package document and the loader parsing it agree by construction.
//!
//! The extension-metadata schemas are not here: they are
//! [`crate::ext_metadata::json_schema`], next to the canonical strings they describe, so
//! the shape and its schema cannot drift apart.
