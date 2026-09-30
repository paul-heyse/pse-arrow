// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded package documents retain parser-derived byte spans before typed DTO decoding.

mod allocation;
mod data;
mod decode;
mod edit;
mod hydrate;
mod load;
mod owned;
mod read;
mod spans;
mod value;

pub use spans::{DocPath, SpanIndex};

pub use data::UNIT_METADATA;
pub use load::{
    Batches, BundleData, Content, Document, DocumentBundle, load_package, load_package_documents,
    package_checksum,
};
pub use owned::{
    OwnedDocumentBundle, OwnedDocumentSet, load_package_documents_owned, load_package_sources_owned,
};

pub use edit::{DocumentEdit, apply_edits, assign_ids};

mod owned_reparse;
pub use owned_reparse::{load_bundles_owned, workspace_extent};
