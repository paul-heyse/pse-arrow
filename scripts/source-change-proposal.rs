// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Digest-only handoff between the checked proposal producer and source-change owner.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Component, PathBuf};

pub(crate) const VERSION: u32 = 1;
pub(crate) const MAX_BYTES: usize = 128 * 1024;
pub(crate) const MAX_FILES: usize = 32;

/// Reviewed selection of regular-file replacements in one working checkout.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProposalHandoff {
    #[schemars(extend("const" = VERSION))]
    pub(crate) version: u32,
    pub(crate) checkout_id: String,
    pub(crate) checkout_revision: String,
    pub(crate) files: Vec<ProposalFile>,
}

/// Exact original and proposed whole-file identities, without retained source text.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProposalFile {
    pub(crate) path: PathBuf,
    pub(crate) before_sha256: String,
    pub(crate) after_sha256: String,
}

fn hex(value: &str, width: usize) -> bool {
    value.len() == width
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

impl ProposalHandoff {
    /// Enforce the shared interpretation before reading any named source.
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if self.version != VERSION
            || !hex(&self.checkout_id, 64)
            || !(hex(&self.checkout_revision, 40) || hex(&self.checkout_revision, 64))
        {
            return Err("proposal handoff has an invalid version or checkout identity");
        }
        if self.files.is_empty() || self.files.len() > MAX_FILES {
            return Err("proposal handoff requires 1–32 selected files");
        }
        let mut paths = std::collections::BTreeSet::new();
        for file in &self.files {
            let path = &file.path;
            if path.as_os_str().is_empty()
                || path.to_str().is_none()
                || !path
                    .components()
                    .all(|part| matches!(part, Component::Normal(_)))
                || path.as_os_str() != path.components().collect::<PathBuf>().as_os_str()
                || !paths.insert(path)
            {
                return Err("proposal handoff requires unique normal checkout-relative paths");
            }
            if !hex(&file.before_sha256, 64)
                || !hex(&file.after_sha256, 64)
                || file.before_sha256 == file.after_sha256
            {
                return Err(
                    "proposal handoff requires distinct original and proposed SHA-256 identities",
                );
            }
        }
        Ok(())
    }
}
