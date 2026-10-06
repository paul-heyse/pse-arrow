// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Stable source/configuration framing shared by the build script and isolated tests.
use std::{fs, path::Path};

/// Complete outer product sources, including the executable worker composition root.
/// Relevant scientific producer inputs are selected separately by their actual unit DAG.
pub(crate) fn source_files(root: &Path) -> std::io::Result<Vec<(String, Vec<u8>)>> {
    let mut files = Vec::new();
    for name in ["crates", "vendor", "xtask"] {
        collect_sources(root, &root.join(name), &mut files)?;
    }
    Ok(files)
}

fn collect_sources(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> std::io::Result<()> {
    println!("cargo:rerun-if-changed={}", directory.display());
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if !matches!(
                entry.file_name().to_str(),
                Some("target" | "__pycache__" | ".git")
            ) {
                collect_sources(root, &path, files)?;
            }
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(std::io::Error::other)?
                .to_string_lossy()
                .replace('\\', "/");
            files.push((relative, fs::read(path)?));
        } else if kind.is_symlink() {
            return Err(std::io::Error::other(format!(
                "unqualified source symlink {}",
                path.display()
            )));
        }
    }
    Ok(())
}

pub(crate) fn digest(mut entries: Vec<(String, Vec<u8>)>) -> pse_ids::ContentHash {
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::BuildInputsV1);
    for (name, bytes) in entries {
        hash.str(&name).part(&bytes);
    }
    hash.finish_hash()
}
#[cfg(test)]
mod foundation_unit {
    #[test]
    fn actual_outer_inventory_rekeys_dirty_worker_sources_and_excludes_build_outputs() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        for name in ["crates/a/src", "vendor", "xtask/src", "xtask/target"] {
            std::fs::create_dir_all(root.join(name)).unwrap();
        }
        std::fs::write(root.join("crates/a/src/lib.rs"), b"library").unwrap();
        let worker = root.join("xtask/src/worker.rs");
        std::fs::write(&worker, b"worker one").unwrap();
        std::fs::write(root.join("xtask/target/generated.rs"), b"build output").unwrap();
        let entries = super::source_files(root).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(
            entries
                .iter()
                .any(|(name, bytes)| name == "xtask/src/worker.rs" && bytes == b"worker one")
        );
        let original = super::digest(entries);
        std::fs::write(&worker, b"worker two").unwrap();
        assert_ne!(original, super::digest(super::source_files(root).unwrap()));
        let changed = super::digest(super::source_files(root).unwrap());
        std::fs::write(
            root.join("xtask/target/generated.rs"),
            b"different build output",
        )
        .unwrap();
        assert_eq!(changed, super::digest(super::source_files(root).unwrap()));
    }

    #[test]
    fn dirty_bytes_names_and_configuration_affect_identity_but_enumeration_order_does_not() {
        let a = ("crates/a/src/lib.rs".into(), b"source".to_vec());
        let b = ("feature:ipopt".into(), b"false".to_vec());
        let original = super::digest(vec![a.clone(), b.clone()]);
        assert_eq!(original, super::digest(vec![b.clone(), a.clone()]));
        assert_ne!(
            original,
            super::digest(vec![(a.0.clone(), b"dirty source".to_vec()), b.clone()])
        );
        assert_ne!(
            original,
            super::digest(vec![("crates/other.rs".into(), a.1.clone()), b.clone()])
        );
        assert_ne!(original, super::digest(vec![a, (b.0, b"true".to_vec())]));
    }
}
