// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The publication protocol: bundle validation, encoding, `PutMode::Create` of every
//! object, then the manifest, then the ref (blueprint §20.1).
//!
//! Packet B-store fills this.
