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
//! 5. move the ref against its observed predicate: native conditional update for remote
//!    stores, or exact control bytes under the cooperative local lock and atomic rename
//!    for [`open::Catalog::open_local`] ([`refs`]).
//!
//! An interruption anywhere in 1–4 leaves the old ref valid and at most some unreachable
//! immutable objects. That property is why the order is a contract and not an
//! implementation detail: reversed, step 5 would publish a ref to a manifest whose members
//! may not exist.
//!
//! Snapshot admission requires explicit parent handles and the registered semantic rule
//! validator. Sidecars have separate typed receipts and never enter membership. The
//! stage validator also establishes the actual registered producer's semantics. Stage
//! contexts read from lookup hints retain accounted immutable ownership through clones.
//! The current Parquet import baseline is uncompressed PLAIN with an exact declared Arrow
//! schema envelope; unsupported encodings and undeclared run producers are refused.

pub mod changes;
pub mod clock;
pub(crate) mod control;
pub mod documents;
pub mod encode;
pub mod layout;
mod local;
pub mod manifest;
pub mod membership;
pub mod open;
pub mod publish;
pub mod refs;
pub mod sidecar;
pub mod stage;
pub mod stage_context;
mod stage_owned;
pub mod verify;
