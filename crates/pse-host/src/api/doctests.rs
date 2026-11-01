// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pinned native doctest preprocessing; extraction never executes an example binary.
#![allow(
    missing_docs,
    reason = "fields retain the pinned rustdoc format2 protocol names"
)]
use super::{Acquisition, ApiError, ArtifactRecord, Context, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const FORMAT_VERSION: u32 = 2;
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Document {
    pub format_version: u32,
    pub doctests: Vec<Example>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Example {
    pub file: String,
    pub line: usize,
    pub name: String,
    pub doctest_attributes: Attributes,
    /// Native collector input; authored fence display remains a separate observation.
    pub original_code: String,
    /// None means native syntax processing could not produce transformed source.
    pub doctest_code: Option<Transformed>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Transformed {
    pub crate_level: String,
    pub code: String,
    pub wrapper: Option<Wrapper>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Wrapper {
    pub before: String,
    pub after: String,
    pub returns_result: bool,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum Ignore {
    All,
    None,
    Some(Vec<String>),
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Attributes {
    pub original: String,
    pub should_panic: bool,
    pub no_run: bool,
    pub ignore: Ignore,
    pub rust: bool,
    pub test_harness: bool,
    pub compile_fail: bool,
    pub standalone_crate: bool,
    pub error_codes: Vec<String>,
    pub edition: Option<String>,
    pub added_css_classes: Vec<String>,
    /// Native unrecognized attribute tokens, never interpreted by this consumer.
    pub unknown: Vec<String>,
}
#[derive(Debug, Serialize, JsonSchema)]
pub struct Page {
    pub artifact: PathBuf,
    pub artifact_sha256: String,
    pub package_id: String,
    pub context_key: String,
    pub target: String,
    pub format_version: u32,
    pub examples: Vec<Example>,
    pub total_examples: usize,
    pub omitted_examples: usize,
    pub compilation: &'static str,
    pub execution: &'static str,
}
pub fn decode(bytes: &[u8]) -> Result<Document, ApiError> {
    #[derive(Deserialize)]
    struct Header {
        format_version: u32,
    }
    let header: Header = serde_json::from_slice(bytes)?;
    if header.format_version != FORMAT_VERSION {
        return Err(invalid(format!(
            "native doctest format {} is unsupported; expected {FORMAT_VERSION} (independent of API JSON61)",
            header.format_version
        )));
    }
    Ok(serde_json::from_slice(bytes)?)
}
/// Publish exact native stdout only after successful managed extraction and input revalidation.
pub fn finish(
    acquisition: &Acquisition,
    snapshot: &crate::catalog::MetadataSnapshot,
    bytes: &[u8],
) -> Result<PathBuf, ApiError> {
    super::validate_acquisition(acquisition, snapshot)?;
    decode(bytes)?;
    let path = acquisition.directory.join("doctests.json");
    super::write(&path, bytes)?;
    super::write(
        &acquisition.directory.join("artifact.json"),
        &serde_json::to_vec(&ArtifactRecord {
            context_key: acquisition.context.key()?,
            sha256: crate::probe::digest(bytes),
        })?,
    )?;
    Ok(path)
}
/// Inspect explicitly addressed retained evidence; historical source grants no current selection.
pub fn inspect(path: &Path, limit: usize) -> Result<Page, ApiError> {
    if path.file_name().is_none_or(|name| name != "doctests.json") {
        return Err(invalid(
            "select an acquired doctests.json with original sidecars",
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| invalid("native doctest parent missing"))?;
    let read = |path: &Path| {
        crate::test_inventory::read_private(path).map_err(|error| invalid(error.to_string()))
    };
    let context: Context = serde_json::from_slice(&read(&parent.join("context.json"))?)?;
    let record: ArtifactRecord = serde_json::from_slice(&read(&parent.join("artifact.json"))?)?;
    let bytes = read(path)?;
    if record.context_key != context.key()?
        || record.sha256 != crate::probe::digest(&bytes)
        || context.rustdoc_format != FORMAT_VERSION
        || context
            .extraction_flags
            .last()
            .is_none_or(|flag| flag != "doctest")
    {
        return Err(invalid(
            "native doctest original bytes or acquisition context changed",
        ));
    }
    let mut document = decode(&bytes)?;
    let total = document.doctests.len();
    document.doctests.truncate(limit);
    Ok(Page {
        artifact: path.to_owned(),
        artifact_sha256: record.sha256,
        package_id: context.package_id.clone(),
        context_key: context.key()?,
        target: context.target,
        format_version: document.format_version,
        omitted_examples: total - document.doctests.len(),
        total_examples: total,
        examples: document.doctests,
        compilation: "not_run",
        execution: "not_run",
    })
}
