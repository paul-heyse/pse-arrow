// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Entity identity assignment (blueprint §5.1).
//!
//! `explicit` policy: the authoring tool assigns a `UUIDv7` when an entity is first written,
//! and P1 rejects a document that still lacks one. `named` policy:
//! `named_id(package_id, qualified_name)`, under which a rename is by definition a new
//! entity. The policy is recorded on the package, never in the ID.
//!
//! Packet C-1 fills this module.
