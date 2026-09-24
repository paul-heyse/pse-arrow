// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The declaration types: what a catalog module is allowed to say.
//!
//! Every type here is a *declaration*, never a runtime value. `RelationSpec` describes a
//! relation; `serde_json::Value` is a registry row, which is the one place the registry is data about
//! itself (blueprint §4.1). Nothing in this module reads an artifact or touches a session.
//!
//! - [`relation`] — relations, columns, keys, quantity contracts.
//! - [`field`] — native Arrow fields and recursive domain facets.
//! - [`extension`] — the eleven `pse.*` extension types and their metadata shapes.
//! - [`enums`] — the model's vocabularies and the declaration form of a `pse.enum`.
//! - [`invariant`] — the commit contract, declared.
//! - [`migration`] — schema migrations.
//! - [`algorithm`] — finite algorithm contracts, typed arguments and results.
//! - [`rule`] — native inference queries and domain policies.
//! - [`document`] — authoring document shapes.

pub mod algorithm;
pub mod collection;
pub mod document;
pub mod enums;
pub mod extension;
pub mod field;
pub mod integer_range;
pub mod invariant;
pub mod migration;
pub mod provider;
pub mod reference;
pub mod relation;
pub mod tagged_alternative;

pub use field::ROW_KEY_ENCODING;

pub use crate::model::algorithm::{
    AlgorithmDecl, AlgorithmSpec, ArgumentSpec, Determinism, ResultSpec,
};
pub use crate::model::document::{
    DocumentKind, DocumentSection, DocumentSpec, DslSyntax, ExpressionOwnerKind, SourceColumn,
};
pub use crate::model::enums::{
    Authority, ColumnRole, DerivationGranularity, EnumDecl, EnumMember, EnumSpec, InvariantKind,
    Namespace, Severity, SnapshotClass, Stability,
};
pub use crate::model::extension::ExtensionUse;
pub use crate::model::extension::{EXTENSION_TYPES, ExtensionMetadataShape, ExtensionTypeSpec};
pub use crate::model::field::{FieldTypeRow, render_data_type};
pub use crate::model::invariant::{InvariantDecl, InvariantSpec};
pub use crate::model::migration::{MigrationSpec, MigrationStep};
pub use crate::model::relation::{
    ForeignKey, QuantityContract, RelationDecl, RelationKey, RelationSpec,
};
pub use pse_ids::source_path::ExpressionPathSegmentKind;

pub use collection::{CollectionContract, CollectionOrder};
pub use field::FieldContract;
pub use integer_range::IntegerRange;
pub use reference::{ReferenceColumn, ReferenceContract, ReferenceNullPolicy};
pub use tagged_alternative::TaggedAlternative;
