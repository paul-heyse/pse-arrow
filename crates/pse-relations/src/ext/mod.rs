// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The eleven `pse.*` `ExtensionType` implementations and the canonical metadata codec
//! (blueprint §4.4).
//!
//! Each type implements `arrow_schema::extension::ExtensionType` with a typed `Metadata`
//! whose `deserialize_metadata` **errors** on an unexpected shape: accepting an unknown
//! generation is how an extension silently forks. `Field::extension_type()` is banned
//! because it panics; the read path is `try_extension_type`.
//!
//! Packet A-7 fills this module.
