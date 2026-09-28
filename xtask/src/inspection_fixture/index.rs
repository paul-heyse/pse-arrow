// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A fixture points directly to its export manifest; no parent/object restoration index.
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Index {
    /// The export manifest's location, opened offline.
    pub manifest: url::Url,
    pub tables: Vec<(String, String, String)>,
}
