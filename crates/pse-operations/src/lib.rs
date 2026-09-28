// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The operational store and publication catalog on PostgreSQL 18 (ADR-0114).
//!
//! PostgreSQL owns what changes — attempts, jobs, leases, cancellation requests, live
//! progress, incumbents, reusable solutions, study status and the publication catalog —
//! and Delta owns what is published. The registry owns the meaning and the shape of every
//! operational relation: this crate embeds the schema generated from it ([`generated`]),
//! creates it or refuses a store whose schema differs ([`Store::open`]), and owns the one
//! attempt transition table ([`lifecycle`]), typed repositories and the catalog with its
//! reader leases.
//!
//! [`Store`] is the handle: a deadpool pool of tokio-postgres connections with an
//! explicit [`Store::connect`], which does no schema work, and [`Store::open`]. Statements
//! are SQL files compiled by Cornucopia into `pse-operations-queries`; the repositories
//! call them and return the generated registry rows. Durability classes (`Ephemeral`,
//! `Durable`) are a runtime policy and do not appear here.

pub mod attempts;
mod bulk;
pub mod cancellation;
pub mod catalog;
mod codec;
mod error;
mod ids;
/// The store's schema generated from the registry: DDL, Cornucopia mapping, fingerprint.
#[rustfmt::skip]
pub mod generated;
pub mod jobs;
pub mod lifecycle;
mod listener;
mod schema;
pub mod solutions;
pub mod sources;
mod store;
pub mod streams;
#[cfg(any(test, feature = "test-support"))]
pub mod testing;

#[cfg(test)]
mod store_tests;

pub use ids::mint_id;
pub use error::{DriverError, InvariantKind, OperationsError, Target};
pub use listener::LISTENER_APPLICATION;
pub use schema::{Opened, PHYSICAL_SQL, SchemaStatus};
pub use store::{
    DATABASE_URL_ENV, DEFAULT_DATABASE_URL, MINIMUM_SERVER_VERSION, ServerInfo, Store,
    StoreOptions, database_url_from_env,
};
