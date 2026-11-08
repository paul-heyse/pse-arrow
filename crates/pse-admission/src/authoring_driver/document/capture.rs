// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact source capture through the ordinary package selection and header parser.

use super::{load, read};
use crate::authoring_driver::DriverError;
use pse_authoring::ParseBudget;
use pse_ids::ContentHash;
use pse_relations::generated::authored;
use pse_schema::{Registry, model::DocumentKind};
use std::{collections::BTreeMap, path::Path};

/// Candidate package identity, version, and dependencies from its exact header bytes.
#[derive(Debug)]
pub struct CapturedPackageHeader {
    /// Typed header. `content_hash` is unset until the declared package is captured.
    pub header: authored::packages::Row,
    /// Exact `package.toml` bytes used by the header parser.
    pub bytes: Vec<u8>,
}

/// Read and parse a candidate header without opening package bodies or data.
/// # Errors
/// The header is absent, invalid, or exceeds its parse budget.
pub fn capture_package_header(
    root: &Path,
    budget: ParseBudget,
) -> Result<CapturedPackageHeader, DriverError> {
    let bytes = read::package_header(root, &budget)?;
    let header = parse_header(&bytes, ContentHash::NIL, &budget)?;
    Ok(CapturedPackageHeader { header, bytes })
}

/// Immutable package bytes and their actual declared source membership.
#[derive(Debug)]
pub struct CapturedPackage {
    /// Typed header, with the ordinary complete-input package checksum.
    pub header: authored::packages::Row,
    /// Exact original bytes keyed by package-relative declared document path.
    pub files: BTreeMap<String, Vec<u8>>,
    /// Declared directories: absent is `None`, present includes sorted members.
    pub directories: BTreeMap<String, Option<Vec<String>>>,
    /// Modeling source paths selected by the registry, without a second glob list.
    pub modeling_documents: Vec<String>,
    registry: ContentHash,
    budget: ParseBudget,
}

impl CapturedPackage {
    /// Compare the original exact bytes and declared membership with the current source.
    /// This verification does not parse or replace the captured documents.
    /// # Errors
    /// The registry, selected membership or bytes changed, or the current source is unreadable.
    pub fn verify(&self, root: &Path, registry: &Registry) -> Result<(), DriverError> {
        if self.registry != registry.fingerprint() {
            return Err(load::contract(None, "captured package registry differs"));
        }
        let current = read::package_files_with_directories(root, registry, &self.budget)?;
        if current.files != self.files || current.directories != self.directories {
            return Err(load::contract(
                None,
                "package bytes or declared document membership changed during capture",
            ));
        }
        Ok(())
    }
}

/// Capture one package without an engine, store, or columnar admission attempt.
///
/// This uses the same bounded reads, registry selection, checksum and typed header
/// parser as ordinary loading. It establishes input identity, not scientific admission.
/// # Errors
/// Invalid declared paths, bytes exceeding the package budget, or an invalid header.
pub fn capture_package(
    root: &Path,
    registry: &Registry,
    budget: ParseBudget,
) -> Result<CapturedPackage, DriverError> {
    let read = read::package_files_with_directories(root, registry, &budget)?;
    let header_bytes = read
        .files
        .get("package.toml")
        .ok_or_else(|| load::contract(None, "captured package header absent"))?;
    let header = parse_header(header_bytes, load::package_checksum(&read.files), &budget)?;
    let modeling_documents = read
        .files
        .keys()
        .filter_map(|path| match load::select(registry, path) {
            Ok(document) if document.kind == DocumentKind::Modeling => Some(Ok(path.clone())),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CapturedPackage {
        header,
        files: read.files,
        directories: read.directories,
        modeling_documents,
        registry: registry.fingerprint(),
        budget,
    })
}

fn parse_header(
    bytes: &[u8],
    checksum: ContentHash,
    budget: &ParseBudget,
) -> Result<authored::packages::Row, DriverError> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| load::contract(None, &error.to_string()))?;
    let (header, _, _) = load::header(text, checksum, budget)?;
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_preserves_data_bytes_and_observes_membership_and_absence() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("package.toml"),
            r#"[package]
package_id = "01010101010101010101010101010101"
name = "capture"
version = "1.0.0"
kind = "model"
id_policy = "named"
dependencies = []
doc = "capture input"
"#,
        )
        .unwrap();
        std::fs::create_dir(root.path().join("data")).unwrap();
        let binary = [0, 255, 13, 10];
        std::fs::write(root.path().join("data/input.parquet"), binary).unwrap();
        std::fs::write(root.path().join("data/generator.json"), b"{}\n").unwrap();
        let registry = pse_schema::shared_registry().unwrap();
        let before = capture_package(root.path(), &registry, ParseBudget::default()).unwrap();
        assert_eq!(before.files["data/input.parquet"], binary);
        assert!(!before.files.contains_key("data/generator.json"));
        assert_eq!(
            before.directories["data"],
            Some(vec!["generator.json".into(), "input.parquet".into()])
        );
        assert_eq!(before.directories["models"], None);
        std::fs::create_dir(root.path().join("models")).unwrap();
        std::fs::write(root.path().join("models/added.pse"), b"package added {}\n").unwrap();
        let after = capture_package(root.path(), &registry, ParseBudget::default()).unwrap();
        assert_eq!(after.modeling_documents, ["models/added.pse"]);
        assert_ne!(before.header.content_hash, after.header.content_hash);
        assert_eq!(after.directories["models"], Some(vec!["added.pse".into()]));
        std::fs::remove_file(root.path().join("models/added.pse")).unwrap();
        let removed = capture_package(root.path(), &registry, ParseBudget::default()).unwrap();
        assert_eq!(removed.directories["models"], Some(vec![]));
        assert_eq!(removed.header.content_hash, before.header.content_hash);
        std::fs::write(root.path().join("data/input.parquet"), [0, 254, 13, 10]).unwrap();
        let changed = capture_package(root.path(), &registry, ParseBudget::default()).unwrap();
        assert_ne!(changed.header.content_hash, before.header.content_hash);
        let small = ParseBudget {
            max_bytes: 1,
            ..ParseBudget::default()
        };
        assert!(capture_package(root.path(), &registry, small).is_err());
    }
}
