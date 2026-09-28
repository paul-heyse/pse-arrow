// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The Rust-owned boundary documents whose JSON Schemas and Python types are published
//! (ADR-0116 Outcomes 6 and 7). Each schema is derived by schemars from the document's
//! serde type in its owning crate; the pure generator renders them.

use pse_codegen::codegen::documents::Document;

fn document<T: schemars::JsonSchema>(name: &'static str) -> Document {
    Document {
        name,
        schema: schemars::schema_for!(T).to_value(),
    }
}

/// Every published document, in publication order.
pub(super) fn documents() -> Vec<Document> {
    vec![
        document::<pse_runtime::math::settings::SolveSettings>("solve-settings"),
        document::<pse_backend_native::execution::BackendSettings>("backend-settings"),
        document::<pse_backend_native::dynamics::DiffsolSettings>("diffsol-settings"),
        document::<pse_backend_native::dynamics::IdasSettings>("idas-settings"),
        document::<pse_runtime::workflow::ModelingJob>("modeling-job"),
        document::<pse_runtime::workflow::TerminationDetail>("termination-detail"),
        document::<pse_runtime::workflow::SourceManifest>("source-manifest"),
    ]
}
