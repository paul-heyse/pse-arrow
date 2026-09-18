// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A fixture points directly to its native root; no parent/object restoration index.
use pse_catalog::delta::publication::PublicationRoot;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Index {
    pub root: PublicationRoot,
    pub tables: Vec<(String, String, String)>,
}
