// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded package documents retain parser-derived byte spans before typed DTO decoding.

mod allocation;
pub mod binding;
mod decode;
mod edit;
mod hydrate;
mod load;
mod owned;
mod read;
mod rename;
mod spans;
mod value;

pub use spans::{DocPath, SpanIndex};

pub use load::{Document, DocumentBundle, Rows, load_package, load_package_texts};
pub use owned::{
    OwnedDocumentBundle, OwnedDocumentSet, load_package_sources_owned, load_package_texts_owned,
};

pub use edit::{DocumentEdit, apply_edits, assign_ids};

pub use rename::{amend_rename_sources, amend_rename_sources_owned, rename, rename_owned};

mod owned_reparse;
pub use owned_reparse::{load_bundles_owned, workspace_extent};
