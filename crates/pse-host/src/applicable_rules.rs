// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Advisory navigation from authored shared-rule scopes, never a second edit guard.
use crate::HostError;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Exact current rule source and only the authored patterns matching the selected path.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Match {
    /// Checkout-relative rule file to read before changing this scope.
    pub source: PathBuf,
    /// Every original rule byte, including its prose, participates in this identity.
    pub source_sha256: String,
    /// Original matching globs, rather than a second hard-coded scope table.
    pub matching_patterns: Vec<String>,
}
#[derive(Debug)]
struct Rule {
    source: PathBuf,
    digest: String,
    patterns: Vec<String>,
    matcher: GlobSet,
}
/// One current read, reused across all paths in a changed-source answer.
#[derive(Debug)]
pub struct Catalog {
    rules: Vec<Rule>,
}
#[derive(Deserialize)]
struct Declaration {
    paths: Vec<String>,
}
fn invalid(path: &Path, reason: impl std::fmt::Display) -> HostError {
    HostError::Ownership(format!("shared rule {}: {reason}", path.display()))
}
fn declaration(path: &Path, text: &str) -> Result<Declaration, HostError> {
    let mut lines = text.split_inclusive('\n');
    if lines.next().map(|line| line.trim_end_matches(['\r', '\n'])) != Some("---") {
        return Err(invalid(path, "authored paths frontmatter is missing"));
    }
    let mut header = String::new();
    for line in lines {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return serde_saphyr::from_str(&header).map_err(|error| invalid(path, error));
        }
        header.push_str(line);
    }
    Err(invalid(path, "unterminated paths frontmatter"))
}
impl Catalog {
    /// Read every current shared Markdown rule once. Malformed or unavailable scope
    /// is a failed source observation, never an empty successful list of applicable rules.
    pub fn load(root: &Path) -> Result<Self, HostError> {
        let root = root.canonicalize()?;
        let directory = root.join(".claude/rules");
        crate::diagnostic_tools::no_symlinks(&directory)?;
        let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(fs::DirEntry::file_name);
        let mut rules = Vec::new();
        for entry in entries {
            let path = entry.path();
            if path.extension().is_none_or(|extension| extension != "md") {
                continue;
            }
            crate::diagnostic_tools::no_symlinks(&path)?;
            let bytes = fs::read(&path)?;
            let text = std::str::from_utf8(&bytes).map_err(|error| invalid(&path, error))?;
            let declared = declaration(&path, text)?;
            let mut matcher = GlobSetBuilder::new();
            for pattern in &declared.paths {
                matcher.add(
                    GlobBuilder::new(pattern)
                        .literal_separator(true)
                        .build()
                        .map_err(|error| invalid(&path, error))?,
                );
            }
            rules.push(Rule {
                source: path
                    .strip_prefix(&root)
                    .map_err(|error| invalid(&path, error))?
                    .to_owned(),
                digest: crate::probe::digest(&bytes),
                patterns: declared.paths,
                matcher: matcher.build().map_err(|error| invalid(&path, error))?,
            });
        }
        Ok(Self { rules })
    }
    /// Match literal native path bytes against authored globs. This advisory answer
    /// grants no edit permission or exception to AGENTS, native hooks or scope owners.
    pub fn matching(&self, relative: &Path) -> Vec<Match> {
        self.rules
            .iter()
            .filter_map(|rule| {
                let matches = rule.matcher.matches(relative);
                (!matches.is_empty()).then(|| Match {
                    source: rule.source.clone(),
                    source_sha256: rule.digest.clone(),
                    matching_patterns: matches
                        .into_iter()
                        .map(|index| rule.patterns[index].clone())
                        .collect(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_yaml_scopes_drive_native_paths_and_duplicate_or_missing_sources_refuse() {
        use std::os::unix::ffi::OsStringExt as _;
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".claude/rules")).unwrap();
        let source = directory.path().join(".claude/rules/source.md");
        fs::write(&source,"---\ndescription: authored\npaths: ['crates/**', '*.toml']\n---\nRead these instructions.\n").unwrap();
        let catalog = Catalog::load(directory.path()).unwrap();
        let relative =
            pathdiff::diff_paths(directory.path(), std::env::current_dir().unwrap()).unwrap();
        assert!(!relative.is_absolute());
        assert_eq!(
            Catalog::load(&relative)
                .unwrap()
                .matching(Path::new("Cargo.toml"))[0]
                .source,
            catalog.matching(Path::new("Cargo.toml"))[0].source
        );
        let path = PathBuf::from(std::ffi::OsString::from_vec(
            b"crates/opaque-\xff.rs".to_vec(),
        ));
        assert_eq!(catalog.matching(&path)[0].matching_patterns, ["crates/**"]);
        assert!(
            catalog
                .matching(Path::new("nested/package.toml"))
                .is_empty()
        );
        let first = catalog.matching(Path::new("Cargo.toml"));
        assert_eq!(first[0].source, Path::new(".claude/rules/source.md"));
        fs::write(&source, "---\npaths: [docs/**]\n---\nNew instructions.\n").unwrap();
        let changed = Catalog::load(directory.path()).unwrap();
        assert!(changed.matching(&path).is_empty());
        assert_ne!(
            changed.matching(Path::new("docs/a.md"))[0].source_sha256,
            first[0].source_sha256
        );
        fs::write(&source, "---\npaths: [docs/**]\npaths: [crates/**]\n---\n").unwrap();
        assert!(Catalog::load(directory.path()).is_err());
        assert!(Catalog::load(&directory.path().join("unavailable")).is_err());
    }
}
