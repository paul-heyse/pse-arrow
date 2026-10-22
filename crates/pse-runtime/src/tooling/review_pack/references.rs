// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One supplied-member resolver for both publication and offline readback.
use super::*;
use std::collections::BTreeMap;

struct Resolver<'a> {
    directory: &'a File,
    index: &'a PackIndex,
    json: BTreeMap<String, serde_json::Value>,
}
impl Resolver<'_> {
    fn resolve(&mut self, member: &str, pointer: &str) -> Result<Option<String>, PackError> {
        let supplied = self
            .index
            .members
            .iter()
            .find(|m| m.selection.id == member)
            .ok_or_else(|| invalid("source reference names an unselected member"))?;
        if !pointer.is_empty() && !pointer.starts_with('/') {
            return Err(invalid("invalid supplied-member JSON pointer"));
        }
        let mut bytes = pointer.bytes();
        while let Some(byte) = bytes.next() {
            if byte == b'~' && !matches!(bytes.next(), Some(b'0' | b'1')) {
                return Err(invalid("invalid JSON pointer escape"));
            }
        }
        if supplied.availability != PackAvailability::Available {
            return Ok(Some(format!(
                "required source {member}#{pointer} is unavailable"
            )));
        }
        if pointer.is_empty() {
            return Ok(None);
        }
        if !matches!(
            supplied.selection.format,
            PackFormat::Json | PackFormat::FrozenEvidenceV3 | PackFormat::AnalysisV2
        ) {
            return Ok(Some(format!(
                "required source {member}#{pointer} has no supported typed coordinate interpretation; only the whole member is addressable"
            )));
        }
        if !self.json.contains_key(member) {
            let value = decode(
                open_contained(self.directory, &supplied.relative_path)?
                    .take(supplied.readable_bytes),
                self.index.selection.maximum_document_bytes,
            )?;
            self.json.insert(member.to_owned(), value);
        }
        if self
            .json
            .get(member)
            .and_then(|value| value.pointer(pointer))
            .is_none()
        {
            return Err(invalid(format!(
                "source pointer {member}#{pointer} is absent from actual supplied JSON"
            )));
        }
        Ok(None)
    }
}
/// A dangling supported pointer is a contract error. Unsupported required coordinates
/// and unavailable required members are explicit missing closure, never fabricated facts.
pub(super) fn validate(directory: &File, index: &PackIndex) -> Result<Vec<String>, PackError> {
    let mut resolver = Resolver {
        directory,
        index,
        json: BTreeMap::new(),
    };
    let mut missing = Vec::new();
    for member in &index.members {
        for reference in &member.selection.references {
            if let Some(reason) = resolver.resolve(&reference.member, &reference.pointer)?
                && member.selection.required
            {
                missing.push(reason);
            }
        }
        if let Some(table) = &member.selection.diagnostic_table
            && let Some(reason) = resolver.resolve(&table.source.member, &table.source.pointer)?
            && member.selection.required
        {
            missing.push(reason);
        }
    }
    let available = |id: &str| {
        index
            .members
            .iter()
            .find(|m| m.selection.id == id && m.availability == PackAvailability::Available)
    };
    if let (Some(frozen), Some(analysis)) = (available("frozen"), available("analysis")) {
        let frozen: FrozenEvidenceSet = decode(
            open_contained(directory, &frozen.relative_path)?.take(frozen.readable_bytes),
            index.selection.maximum_document_bytes,
        )?;
        let analysis: AnalysisDocument = decode(
            open_contained(directory, &analysis.relative_path)?.take(analysis.readable_bytes),
            index.selection.maximum_document_bytes,
        )?;
        frozen.validate().map_err(|e| invalid(e.to_string()))?;
        analysis.validate().map_err(|e| invalid(e.to_string()))?;
        let requirements = crate::tooling::review_analysis::requirements::assess(
            &frozen,
            &analysis.request,
            &analysis
                .selections
                .iter()
                .map(|selection| selection.operation)
                .collect::<Vec<_>>(),
        )
        .map_err(|e| invalid(e.to_string()))?;
        for requirement in &requirements {
            if let Some(reason) = &requirement.missing_reason {
                missing.push(reason.clone());
            }
            for reference in &requirement.sources {
                if let Some(reason) = resolver.resolve(&reference.member, &reference.pointer)? {
                    missing.push(reason);
                }
            }
        }
        for requirement in &analysis.requirements {
            for reference in &requirement.sources {
                if let Some(reason) = resolver.resolve(&reference.member, &reference.pointer)? {
                    missing.push(reason);
                }
            }
        }
        for question in &analysis.questions {
            for statement in &question.statements {
                for reference in &statement.evidence {
                    if let Some(reason) = resolver.resolve(&reference.member, &reference.pointer)? {
                        missing.push(reason);
                    }
                }
            }
        }
        // Existing frozen event identity validation is shared with publication.
        let frozen_json = serde_json::to_value(&frozen).map_err(|e| invalid(e.to_string()))?;
        validate_analysis_sources(&analysis, index, Some(&frozen_json))?;
        missing.extend(missing_result_context(&frozen, index));
    }
    Ok(missing)
}
