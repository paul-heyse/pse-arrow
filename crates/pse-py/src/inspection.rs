// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact Delta publication and owned native table inspection.
mod cache_report;
mod cache_settings;
pub(crate) use cache_report::CacheReport;
pub(crate) mod errors;
mod handles;
pub(crate) mod inputs;
mod reports;
pub(crate) mod runtime;
pub(crate) use reports::{ResourceConsumer, ResourceReport, TableName};
mod settings;
mod stream;
mod tuple;

pub(crate) use cache_settings::CacheSettings;
pub(crate) use errors::{
    DiagnosticAnnotation, DiagnosticCause, DiagnosticContext, DiagnosticNote,
    DiagnosticObservation, DiagnosticReport, DiagnosticSpan, InspectionError,
};
pub(crate) use handles::{Publication, open_publication};
pub(crate) use settings::EngineSettings;
pub(crate) use stream::TableStream;
