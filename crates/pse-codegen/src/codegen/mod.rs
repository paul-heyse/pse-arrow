// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Code generation: the registry rendered into the committed generated trees
//! (blueprint §4.2, ADR-0031, ADR-0051).
//!
//! [`generate`] returns a [`GeneratedTree`] — a path-to-bytes map and the roots it may
//! write under — and never touches the filesystem. `cargo xtask codegen` owns the writing,
//! the stale-file removal and the two-way `--check` diff; a generator that wrote files
//! could not be asked "what would you write?" without side effects, and `--check` is
//! exactly that question.
//!
//! Every generator must be deterministic. `files` is a `BTreeMap` for that reason: a
//! `HashMap` reaching output would make `codegen --check` fail intermittently, which reads
//! as a flaky test rather than as the nondeterminism it is (ADR-0051).
//!
//! Rust rows, manifest envelopes and authoring decoders, Python contracts, Markdown reference pages and
//! authoring JSON Schema are generated from the same admitted registry.

#[cfg(test)]
mod consolidation_unit;
pub mod documents;
pub mod jsonschema;
pub mod markdown;
mod native;

pub mod python;
pub mod rust;
mod surreal;
mod surreal_analyses;
mod surreal_artifacts;
mod surreal_evidence;
mod surreal_execution;
mod surreal_files;
mod surreal_history;
mod surreal_maintenance;
mod surreal_projects;
mod surreal_retention;
mod surreal_studies;

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::builder::Registry;
use crate::error::SchemaError;

/// A generation target (ADR-0051).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Language {
    /// Generated contracts and explicitly selected physical package fixtures.
    Rust,
    /// `python/pse/contracts/`, including its `GENERATED.sha256`, except `documents/`,
    /// which the document types own ([`documents`]).
    Python,
    /// `docs/generated/`, except `schema/`, which the document schemas own
    /// ([`documents`]).
    Markdown,
    /// Registry-generated canonical native schema and codecs.
    Surreal,
}

impl Language {
    /// Every target.
    pub const ALL: [Self; 4] = [Self::Rust, Self::Python, Self::Markdown, Self::Surreal];

    /// The target's name, as `cargo xtask codegen --only <name>` spells it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Markdown => "markdown",
            Self::Surreal => "surreal",
        }
    }

    /// The committed trees this target owns, relative to the workspace root.
    ///
    /// A path outside these roots is not a generated path, and `codegen --check` will
    /// not look for it.
    pub fn roots(self) -> Vec<PathBuf> {
        match self {
            Self::Rust => vec![
                PathBuf::from("crates/pse-schema/src/generated"),
                PathBuf::from("crates/pse-relations/src/generated"),
                PathBuf::from("crates/pse-model/src/generated"),
                PathBuf::from("crates/pse-authoring/src/generated"),
                PathBuf::from("crates/pse-catalog/src/generated"),
                PathBuf::from("crates/pse-admission/src/generated"),
                // The Rust owner prunes the replaced adapter output during publication.
                PathBuf::from("crates/pse-runtime/src/generated"),
                PathBuf::from("crates/pse-quantity/src/generated"),
                PathBuf::from("crates/pse-columnar/src/generated"),
            ],
            Self::Python => vec![PathBuf::from("python/pse/contracts")],
            Self::Markdown => vec![PathBuf::from("docs/generated")],
            Self::Surreal => vec![
                PathBuf::from("crates/pse-operations/src/generated"),
                PathBuf::from("crates/pse-operations-queries"),
            ],
        }
    }
}

/// What a generator would write.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GeneratedTree {
    /// Path relative to the workspace root, to the exact bytes of the file.
    pub files: BTreeMap<PathBuf, Vec<u8>>,
    /// The roots the writer may create, replace within and prune.
    pub roots: Vec<PathBuf>,
}

impl GeneratedTree {
    /// An empty tree over `roots`.
    pub fn empty(roots: Vec<PathBuf>) -> Self {
        Self {
            files: BTreeMap::new(),
            roots,
        }
    }
}

/// Renders `reg` for `language`.
///
/// # Errors
///
/// [`SchemaError::Codegen`] once the generators can fail to render a declaration.
pub fn generate(reg: &Registry, language: Language) -> Result<GeneratedTree, SchemaError> {
    match language {
        Language::Rust => rust::generate(reg),
        Language::Python => python::generate(reg),
        Language::Markdown => markdown::generate(reg),
        Language::Surreal => {
            let mut tree = GeneratedTree::empty(Language::Surreal.roots());
            tree.files.insert(PathBuf::from("crates/pse-operations/src/generated/mod.rs"), b"// @generated by pse-schema; do not edit\n//! Canonical native schema and exact codecs.\n/// Generated native schema and typed wire lowering.\npub mod surreal;\n/// Independently installed derived evidence schema and codecs.\npub mod evidence;\n".to_vec());
            surreal::append(reg, &mut tree)?;
            surreal_evidence::append(reg, &mut tree)?;
            Ok(tree)
        }
    }
}

/// Render only the registry-owned canonical native schema, including declared indexes.
/// Explicit schema fixtures use the same lowering as production installation.
/// # Errors
/// An unsupported native field or invalid declared physical access path.
pub fn canonical_schema(registry: &Registry) -> Result<String, SchemaError> {
    let mut tree = GeneratedTree::empty(Vec::new());
    surreal::append(registry, &mut tree)?;
    let bytes = tree
        .files
        .remove(&PathBuf::from(
            "crates/pse-operations/src/generated/surreal.surql",
        ))
        .ok_or_else(|| SchemaError::Codegen {
            language: "surreal",
            reason: "native schema output absent".into(),
        })?;
    String::from_utf8(bytes).map_err(|error| SchemaError::Codegen {
        language: "surreal",
        reason: error.to_string(),
    })
}

/// Resolve a declared enum child in either an unbound field or composite extension
/// storage. Physical string/dictionary shapes alone do not imply enum meaning.
fn enum_name(field: &arrow_schema::Field) -> Option<&str> {
    crate::model::FieldContract::enum_name_of(field)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;

    #[test]
    fn every_language_declares_its_roots() {
        let reg = registry().expect("the registry assembles");
        for language in Language::ALL {
            let tree = generate(reg, language).expect("generator renders");
            assert_eq!(tree.roots, language.roots());
        }
    }

    #[test]
    fn evidence_family_does_not_change_canonical_installation() {
        use crate::model::{
            Authority, DerivationGranularity, FieldContract, Namespace, RelationDecl, SnapshotClass,
        };
        let original = crate::catalog::assemble().expect("platform declarations");
        let mut builder = crate::RegistryBuilder::new();
        crate::catalog::declare(&mut builder);
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Runtime,
                "evidence_installation_fixture",
                1,
                Authority::Derived,
                SnapshotClass::Derived,
                "Independent tooling-only extension.",
            )
            .pk(&["key"])
            .columns(vec![FieldContract::payload(
                "key",
                FieldContract::native(arrow_schema::DataType::Utf8),
                "Fixture identity.",
            )])
            .granularity(DerivationGranularity::Row),
        );
        let extended = builder.build().expect("extended declarations");
        assert_eq!(
            canonical_schema(&original).unwrap(),
            canonical_schema(&extended).unwrap()
        );
        let before = generate(&original, Language::Surreal).unwrap();
        let after = generate(&extended, Language::Surreal).unwrap();
        let canonical = PathBuf::from("crates/pse-operations/src/generated/surreal.rs");
        assert_eq!(before.files[&canonical], after.files[&canonical]);
        let evidence = PathBuf::from("crates/pse-operations/src/generated/evidence.surql");
        assert_ne!(before.files[&evidence], after.files[&evidence]);
    }

    #[test]
    fn the_roots_are_the_paths_prime_directive_two_protects() {
        assert_eq!(
            Language::Rust.roots(),
            vec![
                PathBuf::from("crates/pse-schema/src/generated"),
                PathBuf::from("crates/pse-relations/src/generated"),
                PathBuf::from("crates/pse-model/src/generated"),
                PathBuf::from("crates/pse-authoring/src/generated"),
                PathBuf::from("crates/pse-catalog/src/generated"),
                PathBuf::from("crates/pse-admission/src/generated"),
                // The Rust owner prunes the replaced adapter output during publication.
                PathBuf::from("crates/pse-runtime/src/generated"),
                PathBuf::from("crates/pse-quantity/src/generated"),
                PathBuf::from("crates/pse-columnar/src/generated"),
            ]
        );
        assert_eq!(
            Language::Python.roots(),
            vec![PathBuf::from("python/pse/contracts")]
        );
        assert_eq!(
            Language::Markdown.roots(),
            vec![PathBuf::from("docs/generated")]
        );
        assert_eq!(
            Language::Surreal.roots(),
            vec![
                PathBuf::from("crates/pse-operations/src/generated"),
                PathBuf::from("crates/pse-operations-queries")
            ]
        );
    }
}
