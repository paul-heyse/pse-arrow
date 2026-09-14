// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reading an artifact written under an earlier version of a relation
//! (blueprint §4.1, §20.5).
//!
//! Migrations are generated from `reference.schema_migrations`, so the steps a reader
//! applies and the steps the registry declares are one statement.
//!
//! Packet A-7 fills this module.
