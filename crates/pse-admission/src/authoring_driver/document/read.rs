// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded reads and declared package-relative document selection.

use crate::authoring_driver::{DriverError, ParseBudget};
use pse_schema::Registry;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

/// Every declared document of the package at `root` as its exact bytes, within the
/// package byte budget.
pub(super) fn package_files(
    root: &Path,
    registry: &Registry,
    budget: &ParseBudget,
) -> Result<BTreeMap<String, Vec<u8>>, DriverError> {
    Ok(package_files_with_directories(root, registry, budget)?.files)
}

pub(super) struct PackageFiles {
    pub files: BTreeMap<String, Vec<u8>>,
    pub directories: BTreeMap<String, Option<Vec<String>>>,
}

pub(super) fn package_files_with_directories(
    root: &Path,
    registry: &Registry,
    budget: &ParseBudget,
) -> Result<PackageFiles, DriverError> {
    let mut files = BTreeMap::new();
    let mut membership = BTreeMap::new();
    let mut remaining = budget.max_bytes;
    read_one(root, "package.toml", &mut remaining, &mut files)?;
    let mut directories = BTreeMap::<&str, Vec<&str>>::new();
    for document in registry.documents() {
        let Some((directory, pattern)) = document.path_glob.split_once('/') else {
            continue;
        };
        if !matches!(pattern, "*.yaml" | "*.pse" | "*.parquet") || directory.contains(['.', '\\']) {
            return Err(failure(
                document.path_glob,
                "unsupported declared document glob",
            ));
        }
        directories.entry(directory).or_default().push(pattern);
    }
    for (directory, patterns) in directories {
        let path = root.join(directory);
        if !path.exists() {
            membership.insert(directory.to_owned(), None);
            continue;
        }
        if std::fs::symlink_metadata(&path)
            .map_err(|error| failure(directory, &error.to_string()))?
            .file_type()
            .is_symlink()
        {
            return Err(failure(directory, "symlink document directory"));
        }
        let entries =
            std::fs::read_dir(path).map_err(|error| failure(directory, &error.to_string()))?;
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| failure(directory, &error.to_string()))?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| failure(directory, "non-UTF8 document path"))?;
            names.push(name.clone());
            if entry
                .file_type()
                .map_err(|error| failure(directory, &error.to_string()))?
                .is_symlink()
            {
                return Err(failure(
                    &format!("{directory}/{name}"),
                    "symlink document member",
                ));
            }
            if !patterns.iter().any(|pattern| name.ends_with(&pattern[1..])) {
                // Generator inputs and other non-documents remain membership facts;
                // only the registry's declared globs own document selection.
                continue;
            }
            super::load::select(registry, &format!("{directory}/{name}"))?;
            read_one(
                root,
                &format!("{directory}/{name}"),
                &mut remaining,
                &mut files,
            )?;
        }
        names.sort();
        membership.insert(directory.to_owned(), Some(names));
    }
    Ok(PackageFiles {
        files,
        directories: membership,
    })
}

fn read_one(
    root: &Path,
    relative: &str,
    remaining: &mut u64,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), DriverError> {
    let path = root.join(relative);
    let metadata =
        std::fs::symlink_metadata(&path).map_err(|error| failure(relative, &error.to_string()))?;
    if !metadata.file_type().is_file() {
        return Err(failure(
            relative,
            "document must be a regular file, not a symlink",
        ));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|error| failure(relative, &error.to_string()))?
        .take(remaining.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| failure(relative, &error.to_string()))?;
    let needed = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if needed > *remaining {
        return Err(DriverError::Authoring(
            pse_authoring::AuthoringError::Budget {
                limit: "package bytes",
                allowed: *remaining,
                needed,
            },
        ));
    }
    *remaining -= needed;
    files.insert(relative.to_owned(), bytes);
    Ok(())
}

fn failure(path: &str, reason: &str) -> DriverError {
    DriverError::Authoring(pse_authoring::AuthoringError::DocumentIo {
        path: path.to_owned(),
        reason: reason.to_owned(),
    })
}
