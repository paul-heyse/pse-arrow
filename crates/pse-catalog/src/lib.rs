// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Snapshot catalog (`CatalogProviderList`, `SchemaProvider`, TableProvider) and the
//! artifact store (blueprint §3.2, §20).
//!
//! Providers expose exactly the read-path method set and every `Exact` filter they
//! claim must actually be applied (blueprint §20, §24.1 "Pushdown truthfulness").
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
