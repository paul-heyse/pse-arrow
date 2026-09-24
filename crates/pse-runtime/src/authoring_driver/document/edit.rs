// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reviewable original-byte edits, applied only against their exact preimages.

use super::{load, value};
use crate::authoring_driver::{DriverError, ParseBudget, SourceSpan, ids::IdPolicy};
use pse_ids::SemanticId;
use pse_schema::{Registry, model::DocumentKind};
use std::collections::{BTreeMap, BTreeSet};

/// One complete document replacement with an exact byte precondition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentEdit {
    /// Stable document identity.
    pub document_id: SemanticId,
    /// Package-relative source path.
    pub path: String,
    /// Exact expected original text.
    pub before: String,
    /// New text, ready to reparse for hashes and spans.
    pub after: String,
}

/// Insert missing required UUID identities using parser row ranges; no files are written.
///
/// # Errors
/// Invalid headers, unknown document shapes, malformed rows, missing parser locations,
/// duplicate generated identities and parse budgets are refused.
pub fn assign_ids(
    texts: &BTreeMap<String, String>,
    registry: &Registry,
    budget: ParseBudget,
    next: &mut dyn FnMut() -> SemanticId,
) -> Result<Vec<DocumentEdit>, DriverError> {
    let text = texts
        .get("package.toml")
        .ok_or_else(|| load::contract(None, "missing package.toml"))?;
    let checksum = pse_ids::encoding_checksum(text.as_bytes()).content_hash();
    let (package, _, _) = load::header(text, checksum, &budget)?;
    let mut edits = Vec::new();
    let mut generated = BTreeSet::new();
    for (path, text) in texts {
        let declaration = load::select(registry, path)?;
        if declaration.kind == DocumentKind::PackageHeader {
            continue;
        }
        let document = pse_ids::named_id(package.package_id, path);
        let (parsed, spans) = value::parse_yaml(text, document, &budget)?;
        let mut insertions = Vec::new();
        for section in &declaration.sections {
            let Some(identity) = section.identity_column else {
                continue;
            };
            if package.id_policy == IdPolicy::Named && section.entity_kind.is_some() {
                continue;
            }
            let Some(value::Value::List(rows)) = parsed.value.get(section.key) else {
                continue;
            };
            for (ordinal, row) in rows.iter().enumerate() {
                if row.value.get("id").is_some() || row.value.get(identity).is_some() {
                    continue;
                }
                let at = spans
                    .span(&format!("/{}/{ordinal}", section.key))
                    .ok_or_else(|| load::contract(None, "missing original row range"))?;
                let id = next();
                if id == SemanticId::NIL || !generated.insert(id) {
                    return Err(load::contract(
                        Some(at),
                        "ID supplier returned nil or a duplicate identity",
                    ));
                }
                insertions.push(insertion(text, at, id)?);
            }
        }
        if !insertions.is_empty() {
            insertions.sort_by_key(|(offset, _)| std::cmp::Reverse(*offset));
            let mut after = text.clone();
            for (offset, insertion) in insertions {
                after.insert_str(offset, &insertion);
            }
            edits.push(DocumentEdit {
                document_id: document,
                path: path.clone(),
                before: text.clone(),
                after,
            });
        }
    }
    let mut candidate = texts.clone();
    apply_edits(&mut candidate, &edits)?;
    // Exact typed decode validates both the inserted IDs and all retained data.
    load::load_package_texts(candidate, registry, budget)?;
    Ok(edits)
}

fn insertion(text: &str, at: SourceSpan, id: SemanticId) -> Result<(usize, String), DriverError> {
    let offset = usize::try_from(at.start)
        .map_err(|_| load::contract(Some(at), "source offset overflow"))?;
    let rest = text
        .get(offset..)
        .ok_or_else(|| load::contract(Some(at), "source offset is not a character boundary"))?;
    if rest.starts_with('{') {
        return Ok((offset + 1, format!("id: '{id}', ")));
    }
    let line = text[..offset].rfind('\n').map_or(0, |offset| offset + 1);
    let indent = offset - line;
    Ok((offset, format!("id: '{id}'\n{}", " ".repeat(indent))))
}

/// Apply a complete set of replacements only when all exact original texts agree.
///
/// # Errors
/// Missing, duplicated or stale document preimages leave the input map unchanged.
pub fn apply_edits(
    texts: &mut BTreeMap<String, String>,
    edits: &[DocumentEdit],
) -> Result<(), DriverError> {
    let mut paths = BTreeSet::new();
    for edit in edits {
        if !paths.insert(&edit.path) || texts.get(&edit.path) != Some(&edit.before) {
            return Err(load::contract(
                None,
                "document edits have a missing, duplicate or changed exact preimage",
            ));
        }
    }
    for edit in edits {
        texts.insert(edit.path.clone(), edit.after.clone());
    }
    Ok(())
}
