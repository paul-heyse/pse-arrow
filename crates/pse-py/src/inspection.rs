// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable admitted store and table inspection (ADR-0067).
mod errors;
mod handles;
mod runtime;
mod settings;
mod stream;

pub(crate) use errors::InspectionError;
pub(crate) use handles::{Snapshot, Store, open_store};
pub(crate) use settings::EngineSettings;
pub(crate) use stream::TableStream;
