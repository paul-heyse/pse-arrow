// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned canonical Arrow table inspection.
mod cache_settings;
pub(crate) mod errors;
pub(crate) mod inputs;
mod registry;
pub(crate) mod runtime;
mod settings;
mod stream;
mod tuple;

pub(crate) use cache_settings::CacheSettings;
pub(crate) use errors::{DiagnosticReport, InspectionError};
pub(crate) use registry::registry_table;
pub(crate) use settings::EngineSettings;
pub(crate) use stream::TableStream;
