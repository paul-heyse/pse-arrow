// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The declaration types: what a catalog module is allowed to say.
//!
//! Every type here is a *declaration*, never a runtime value. `RelationSpec` describes a
//! relation; `Cell` is a registry row, which is the one place the registry is data about
//! itself (blueprint §4.1). Nothing in this module reads an artifact or touches a session.
//!
//! - [`relation`] — relations, columns, keys, quantity contracts.
//! - [`logical_type`] — the §4.5 scalars, the composites, and the §4.4 extension uses.
//! - [`extension`] — the eleven `pse.*` extension types and their metadata shapes.
//! - [`enums`] — the model's vocabularies and the declaration form of a `pse.enum`.
//! - [`invariant`] — the commit contract, declared.
//! - [`migration`] — schema migrations.
//! - [`pass`] — pass contracts and their ports.
//! - [`rule`], [`rule_expr`] — inference as a typed relational plan.
//! - [`cell`] — the registry's own row model.
//! - [`manifest`] — the `pse.manifest.v2` envelope.
//! - [`document`] — authoring document shapes.

pub mod cell;
pub mod document;
pub mod enums;
pub mod extension;
pub mod invariant;
pub mod logical_type;
pub mod manifest;
pub mod migration;
pub mod pass;
pub mod relation;
pub mod rule;
pub mod rule_expr;

pub use crate::model::cell::Cell;
pub use crate::model::document::{DocumentKind, DocumentSection, DocumentSpec};
pub use crate::model::enums::{
    Authority, ColumnRole, DerivationGranularity, EnumDecl, EnumMember, EnumSpec, InvariantKind,
    Namespace, Severity, SnapshotClass, Stability,
};
pub use crate::model::extension::{EXTENSION_TYPES, ExtensionMetadataShape, ExtensionTypeSpec};
pub use crate::model::invariant::{InvariantDecl, InvariantSpec};
pub use crate::model::logical_type::{ExtensionUse, LogicalType, LogicalTypeRow, render_data_type};
pub use crate::model::manifest::{ManifestField, ManifestSpec, ManifestType};
pub use crate::model::migration::{MigrationSpec, MigrationStep};
pub use crate::model::pass::{Determinism, InputPort, OutputPort, PassDecl, PassSpec, PortSource};
pub use crate::model::relation::{
    ColumnSpec, ForeignKey, QuantityContract, RelationDecl, RelationKey, RelationSpec,
};
pub use crate::model::rule::{
    AggregateEmptyPolicy, AggregateNullPolicy, ConflictPolicy, DependencyMode, DepthBound,
    EmptyListPolicy, NON_DERIVABLE_NAMESPACES, NegationPolicy, NullEquality, NullListPolicy,
    RuleAggregate, RuleAggregateFn, RuleDecl, RuleDependency, RuleHead, RulePlan, RuleSpec,
};
pub use crate::model::rule_expr::{CmpOp, RuleExpr};
