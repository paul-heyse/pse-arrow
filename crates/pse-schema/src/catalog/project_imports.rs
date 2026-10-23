// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable logical-project import identity and monotonic completed membership.
use super::declarations::{column, relation};
use super::substrate::NativeIndex;
use crate::{RegistryBuilder, model::{FieldContract as T, Namespace as N, SnapshotClass as S}};
use arrow_schema::DataType as D;
pub const NATIVE_INDEXES:&[NativeIndex]=&[
    NativeIndex{name:"project_import_project",table:"canonical_project_imports",fields:&["project"],unique:true},
    NativeIndex{name:"project_import_problem",table:"canonical_project_imports",fields:&["problem"],unique:true},
];
pub(super) fn declare(b:&mut RegistryBuilder) {
    let text=||T::native(D::Utf8); let bytes=||T::native(D::Binary);
    relation(b,N::Runtime,"canonical_project_imports",S::Derived,&["key"],vec![
        column("key",text()),column("issuing_store",text()),column("bundle_digest",text()),
        column("project",text()),column("branch",text()),column("problem",text()),column("revision",text()),
        column("origin",bytes()),column("request",bytes()),column("artifact",bytes()).optional(),
        column("complete",T::native(D::Boolean)),
    ],"Original immutable fresh-authority import intent; source admission is pending until exact copied evidence artifact ownership finishes. Historical terminals remain origin evidence only.");
}
