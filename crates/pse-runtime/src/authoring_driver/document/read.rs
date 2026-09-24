// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded reads and declared package-relative document selection.

use crate::authoring_driver::{DriverError, ParseBudget};
use pse_schema::Registry;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

pub(super) fn package_files(
    root: &Path,
    registry: &Registry,
    budget: &ParseBudget,
) -> Result<BTreeMap<String, String>, DriverError> {
    let mut files = BTreeMap::new();
    let mut remaining = budget.max_bytes;
    read_one(root, "package.toml", &mut remaining, &mut files)?;
    let mut directories = std::collections::BTreeSet::new();
    for document in registry.documents() {
        let Some((directory, pattern)) = document.path_glob.split_once('/') else {
            continue;
        };
        if pattern != "*.yaml" || directory.contains(['.', '\\']) {
            return Err(failure(
                document.path_glob,
                "unsupported declared document glob",
            ));
        }
        if !directories.insert(directory) {
            continue;
        }
        let path = root.join(directory);
        if !path.exists() {
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
        for entry in entries {
            let entry = entry.map_err(|error| failure(directory, &error.to_string()))?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| failure(directory, "non-UTF8 document path"))?;
            if Path::new(&name).extension() != Some(std::ffi::OsStr::new("yaml")) {
                return Err(failure(
                    &name,
                    "only declared *.yaml package documents are admitted",
                ));
            }
            read_one(
                root,
                &format!("{directory}/{name}"),
                &mut remaining,
                &mut files,
            )?;
        }
    }
    Ok(files)
}

fn read_one(
    root: &Path,
    relative: &str,
    remaining: &mut u64,
    files: &mut BTreeMap<String, String>,
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
    let mut text = String::new();
    std::fs::File::open(path)
        .map_err(|error| failure(relative, &error.to_string()))?
        .take(remaining.saturating_add(1))
        .read_to_string(&mut text)
        .map_err(|error| failure(relative, &error.to_string()))?;
    let needed = u64::try_from(text.len()).unwrap_or(u64::MAX);
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
    files.insert(relative.to_owned(), text);
    Ok(())
}

fn failure(path: &str, reason: &str) -> DriverError {
    DriverError::Authoring(pse_authoring::AuthoringError::DocumentIo {
        path: path.to_owned(),
        reason: reason.to_owned(),
    })
}
