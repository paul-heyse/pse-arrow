// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The artifact store: layout, manifest, encoding, verification, refs and publication
//! (blueprint §20.1, §20.2).
//!
//! Publication is a local single-writer protocol over atomic object creation and
//! conditional ref updates, in this order and no other (§20.1):
//!
//! 1. validate the complete output bundle and compute logical hashes ([`crate::contract`],
//!    `pse-ids`);
//! 2. serialize each selected encoding with an explicit successful finish and verify its
//!    size, format and checksum ([`encode`], [`verify`]);
//! 3. store each object with `PutMode::Create`, verifying any object that already exists
//!    rather than accepting `AlreadyExists` on the strength of its name ([`publish`]);
//! 4. write the manifest under its own checksum ([`manifest`]);
//! 5. move the ref with `PutMode::Update` against its prior version ([`refs`]).
//!
//! An interruption anywhere in 1–4 leaves the old ref valid and at most some unreachable
//! immutable objects. That property is why the order is a contract and not an
//! implementation detail: reversed, step 5 would publish a ref to a manifest whose members
//! may not exist.
//!
//! Phase 0 lands the layout, the manifest codec and the clock. The remaining modules are
//! packet B-store.

pub mod clock;
pub mod encode;
pub mod layout;
pub mod manifest;
pub mod membership;
pub mod open;
pub mod publish;
pub mod refs;
pub mod verify;
