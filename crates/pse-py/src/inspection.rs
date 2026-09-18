// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact Delta publication and owned native table inspection.
mod cache_report;
mod cache_settings;
pub(crate) use cache_report::CacheReport;
mod errors;
mod handles;
mod runtime;
mod settings;
mod stream;

pub(crate) use cache_settings::CacheSettings;
pub(crate) use errors::InspectionError;
pub(crate) use handles::{Publication, open_publication};
pub(crate) use settings::EngineSettings;
pub(crate) use stream::TableStream;
