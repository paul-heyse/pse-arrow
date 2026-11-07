// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read one checkout-authored package through the ordinary bounded package owner.

use pse_admission::authoring_driver::document::capture_package;
use std::{collections::BTreeMap, error::Error, io, path::Path};

pub(crate) fn package_documents(
    checkout: &Path,
    logical_root: &str,
    registry: &pse_schema::Registry,
) -> Result<BTreeMap<String, Vec<u8>>, Box<dyn Error + Send + Sync>> {
    let package = checkout.canonicalize()?.join(logical_root);
    if package.canonicalize()? != package {
        return Err(io::Error::other("authored fixture path traverses a symlink").into());
    }
    let original = capture_package(&package, registry, Default::default())?;
    let observed = capture_package(&package, registry, Default::default())?;
    if original.files != observed.files || original.directories != observed.directories {
        return Err(io::Error::other(
            "authored fixture bytes or document membership changed during capture",
        )
        .into());
    }
    Ok(original.files)
}
