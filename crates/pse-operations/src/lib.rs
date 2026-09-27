// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The operational store and publication catalog on PostgreSQL 18 (ADR-0112).
//!
//! PostgreSQL owns what changes — attempts, jobs, leases, cancellation requests, live
//! progress, incumbents, reusable solutions, study status and the publication catalog —
//! and Delta owns what is published. This crate owns the store contract: the embedded
//! migrations ([`MIGRATOR`]), the one attempt transition table ([`lifecycle`]), typed
//! repositories with runtime-typed SQL, and the catalog with its reader leases.
//!
//! [`Store`] is the handle: a pool plus the migrator, with an explicit [`Store::connect`]
//! and [`Store::migrate`]. Durability classes (`Ephemeral`, `Durable`) are a runtime
//! policy and do not appear here.

pub mod attempts;
pub mod cancellation;
pub mod catalog;
mod codec;
mod error;
pub mod jobs;
pub mod lifecycle;
pub mod solutions;
mod store;
pub mod streams;

#[cfg(test)]
mod store_tests;

pub use codec::mint_id;
pub use error::{OperationsError, Target};
pub use store::{
    DATABASE_URL_ENV, DEFAULT_DATABASE_URL, MINIMUM_SERVER_VERSION, MigrationStatus, ServerInfo,
    Store, StoreOptions, database_url_from_env,
};

/// The embedded, versioned `pse_ops` migrations (LF line endings; their checksums are
/// part of the schema identity).
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();
