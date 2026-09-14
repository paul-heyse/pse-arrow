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
//! Phase 0 emits nothing: packet A-6 fills [`rust`], packet A-6's Python sibling fills
//! [`python`], and [`markdown`] and [`jsonschema`] follow. The roots are already correct,
//! so the xtask arm that removes stale files is exercised before there is anything to
//! remove.

pub mod jsonschema;
pub mod markdown;
pub mod python;
pub mod rust;

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::builder::Registry;
use crate::error::SchemaError;

/// A generation target (ADR-0051).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Language {
    /// `crates/pse-relations/src/generated/` and `crates/pse-authoring/src/generated/`.
    Rust,
    /// `python/pse/contracts/`, including its `GENERATED.sha256`.
    Python,
    /// `docs/generated/`, including `schema/authoring.schema.json`.
    Markdown,
}

impl Language {
    /// Every target.
    pub const ALL: [Self; 3] = [Self::Rust, Self::Python, Self::Markdown];

    /// The target's name, as `cargo xtask codegen --only <name>` spells it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Markdown => "markdown",
        }
    }

    /// The committed trees this target owns, relative to the workspace root.
    ///
    /// A path outside these roots is not a generated path, and `codegen --check` will
    /// not look for it.
    pub fn roots(self) -> Vec<PathBuf> {
        match self {
            Self::Rust => vec![
                PathBuf::from("crates/pse-relations/src/generated"),
                PathBuf::from("crates/pse-authoring/src/generated"),
            ],
            Self::Python => vec![PathBuf::from("python/pse/contracts")],
            Self::Markdown => vec![PathBuf::from("docs/generated")],
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
#[expect(
    clippy::unnecessary_wraps,
    reason = "the fallible signature is the contract packets A-6 and R-1 code against; making it infallible now would break every caller on the day the first generator rejects a declaration"
)]
pub fn generate(reg: &Registry, language: Language) -> Result<GeneratedTree, SchemaError> {
    let _ = reg;
    Ok(GeneratedTree::empty(language.roots()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;

    #[test]
    fn every_language_declares_its_roots_and_writes_nothing_yet() {
        let reg = registry().expect("the registry assembles");
        for language in Language::ALL {
            let tree = generate(reg, language).expect("phase 0 emits nothing");
            assert_eq!(tree.roots, language.roots());
            assert!(
                tree.files.is_empty(),
                "{} emitted files before its generator exists",
                language.as_str()
            );
        }
    }

    #[test]
    fn the_roots_are_the_paths_prime_directive_two_protects() {
        assert_eq!(
            Language::Rust.roots(),
            vec![
                PathBuf::from("crates/pse-relations/src/generated"),
                PathBuf::from("crates/pse-authoring/src/generated"),
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
    }
}
