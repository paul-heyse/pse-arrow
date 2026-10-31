// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact retained documentation spans can nominate current source; native analysis resolves it.
use super::{ApiError, Artifact, invalid};
use crate::{environment::Environment, semantic::Configuration, source_read::SourceText};
use std::{collections::BTreeSet, ops::Range, path::Path};

/// Bridge an original local API item into its byte-identical selected working project.
/// This proves an original source nomination, never equality of rustdoc and semantic IDs.
pub fn source(
    root: &Path,
    artifact: &Artifact,
    item: &str,
    configuration: &Configuration,
    environment: &Environment,
) -> Result<(SourceText, Range<usize>), ApiError> {
    let original = super::retained(&artifact.path)?;
    if original.sha256 != artifact.sha256 || original.context != artifact.context {
        return Err(invalid("API artifact or acquisition context changed"));
    }
    let root = root.canonicalize()?;
    let project = configuration.project.canonicalize()?;
    let manifest = artifact.context.manifest.canonicalize()?;
    if !manifest.starts_with(&project) || !manifest.starts_with(&root) {
        return Err(invalid(
            "API package is outside the selected semantic project/source universe",
        ));
    }
    if configuration.target.as_deref() != Some(artifact.context.target.as_str()) {
        return Err(invalid(
            "API/source bridge requires the same explicitly selected target; an implicit target is not a proved match",
        ));
    }
    let manifest_text = std::fs::read_to_string(&manifest)?;
    let manifest_value: toml::Value =
        toml::from_str(&manifest_text).map_err(|error| invalid(error.to_string()))?;
    let features = effective_features(&manifest_value, configuration)?;
    if features != artifact.context.features {
        return Err(invalid(
            "requested semantic feature closure differs from the acquired documentation unit",
        ));
    }
    let environment = super::extraction_environment(environment)?;
    if !artifact
        .context
        .source
        .recorded_environment_matches(&environment)
    {
        return Err(invalid(
            "API build-input environment differs from the selected working environment",
        ));
    }
    if !artifact
        .context
        .source
        .recorded_sources_match(&root)
        .map_err(|error| invalid(error.to_string()))?
    {
        return Err(invalid(
            "API acquisition source/configuration inputs changed",
        ));
    }
    let package_root = manifest
        .parent()
        .ok_or_else(|| invalid("API package manifest has no parent"))?;
    let mut files = Vec::new();
    super::source_files(package_root, Path::new(""), &mut files)?;
    if super::source_files_digest(&files) != artifact.context.package_source_sha256 {
        return Err(invalid("API package source bytes or membership changed"));
    }
    let id = rustdoc_types::Id(
        item.parse()
            .map_err(|_| invalid("API item ID is not an artifact-local integer"))?,
    );
    let declaration = artifact
        .document
        .index
        .get(&id)
        .ok_or_else(|| invalid("API item is absent from its retained artifact"))?;
    let span = declaration
        .span
        .as_ref()
        .ok_or_else(|| invalid("API declaration has no original source span"))?;
    let path = root.join(&span.filename).canonicalize()?;
    if !path.starts_with(package_root) || !path.starts_with(&project) {
        return Err(invalid(
            "API span is not an original file in the selected package/project",
        ));
    }
    let source = SourceText::new(path.clone(), std::fs::read_to_string(&path)?)
        .map_err(|error| invalid(error.to_string()))?;
    let relative = path
        .strip_prefix(&root)
        .map_err(|_| invalid("API span is outside the recorded source root"))?;
    if artifact
        .context
        .source
        .inputs()
        .find(|(path, _)| *path == relative)
        .is_none_or(|(_, hash)| hash != source.identity().source_sha256())
    {
        return Err(invalid(
            "API span source is not byte-identical to its retained acquisition input",
        ));
    }
    let start = offset(source.text(), span.begin)?;
    let end = offset(source.text(), span.end)?;
    if end < start {
        return Err(invalid("API source span is reversed"));
    }
    Ok((source, start..end))
}

fn effective_features(
    manifest: &toml::Value,
    configuration: &Configuration,
) -> Result<BTreeSet<String>, ApiError> {
    let declared = manifest.get("features").and_then(toml::Value::as_table);
    let mut pending = configuration.features.clone();
    if !configuration.no_default_features
        && declared.is_some_and(|features| features.contains_key("default"))
    {
        pending.push("default".into());
    }
    let mut result = BTreeSet::new();
    while let Some(feature) = pending.pop() {
        if feature.contains('/') || feature.starts_with("dep:") {
            return Err(invalid(
                "dependency-qualified feature requests need native effective-feature evidence before API/source bridging",
            ));
        }
        if !result.insert(feature.clone()) {
            continue;
        }
        if let Some(values) = declared
            .and_then(|features| features.get(&feature))
            .and_then(toml::Value::as_array)
        {
            for value in values {
                let value = value
                    .as_str()
                    .ok_or_else(|| invalid("feature activation is not a string"))?;
                if value.contains('/') || value.starts_with("dep:") {
                    return Err(invalid(
                        "dependency feature activation needs native effective-feature evidence before API/source bridging",
                    ));
                }
                pending.push(value.into());
            }
        }
    }
    Ok(result)
}

fn offset(text: &str, (line, column): (usize, usize)) -> Result<usize, ApiError> {
    if line == 0 || column == 0 {
        return Err(invalid("rustdoc source coordinates must be one-based"));
    }
    let mut start = 0;
    for (index, part) in text.split_inclusive('\n').enumerate() {
        if index + 1 == line {
            let offset = part
                .char_indices()
                .map(|(offset, _)| offset)
                .chain(std::iter::once(part.len()))
                .nth(column - 1)
                .ok_or_else(|| invalid("API source column is outside its original line"))?;
            return Ok(start + offset);
        }
        start += part.len();
    }
    if line == text.bytes().filter(|byte| *byte == b'\n').count() + 1 && column == 1 {
        return Ok(text.len());
    }
    Err(invalid("API source line is outside its original input"))
}
