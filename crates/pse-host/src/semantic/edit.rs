// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Translate native workspace effects into the one immutable proposal contract.
use super::{Configuration, Error, Result};
use crate::{
    source_proposal::{
        ChangeProposal, EditCandidate, FileOperation, ProducerContext, UnresolvedPlaceholder,
    },
    source_read::SourceText,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

enum Change {
    Edit {
        uri: String,
        version: Option<i64>,
        edits: Value,
    },
    Create(String),
    Rename(String, String),
    Delete(String),
}
/// Adapt the exact native SourceChange into the existing immutable proposal owner.
/// No raw analyzer handles cross the helper boundary and no source is mutated.
#[cfg(feature = "native-hir")]
pub(super) fn native_proposal(
    configuration: &Configuration,
    backend: &str,
    change: &ra_ap_ide_db::source_change::SourceChange,
    paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
    digests: &BTreeMap<PathBuf, String>,
) -> Result<ChangeProposal> {
    if !change.file_system_edits.is_empty() {
        return Err(Error::unsupported(
            "native filesystem SourceChange needs explicit anchored-path mapping; no file operation was discarded",
        ));
    }
    let mut sources = BTreeMap::new();
    let mut candidates = Vec::new();
    let mut placeholders = Vec::new();
    for (id, (edits, snippet)) in &change.source_file_edits {
        let path = paths.get(&id).ok_or_else(|| {
            Error::Unavailable("native edit has no unique original VFS path".into())
        })?;
        if !path.starts_with(&configuration.project) {
            return Err(Error::unsupported(
                "native edit lies outside the selected project",
            ));
        }
        let source = crate::source_read::SourceRequest::Working { path: path.clone() }.acquire()?;
        if digests
            .get(path)
            .is_none_or(|digest| digest != source.identity().source_sha256())
        {
            return Err(Error::Stale);
        }
        for edit in edits.iter() {
            if change.is_snippet
                || snippet.is_some()
                || edit.insert.contains("todo!")
                || edit.insert.contains("unimplemented!")
            {
                placeholders.push(UnresolvedPlaceholder { candidate: candidates.len(), description: "Native snippet or nested synthesis hole remains unresolved; import obligations require review.".into() });
            }
            candidates.push(EditCandidate::new(
                source.identity().clone(),
                usize::from(edit.delete.start())..usize::from(edit.delete.end()),
                edit.insert.clone(),
            ));
        }
        sources.insert(path.clone(), source);
    }
    let selected = (0..candidates.len()).collect();
    Ok(ChangeProposal::with_file_operations(
        ProducerContext::Semantic { backend: backend.into(), configuration: configuration.identity()?, synchronized_generation: "1".into() },
        sources.into_values().collect(), candidates, selected, Vec::new(),
        vec!["Preview only; native source changes do not establish compiler acceptance or equivalence.".into(), "Native synthesis has bounded fuel and unknown completeness. Required imports and nested alternatives remain review obligations.".into(), format!("Native change annotations: {:?}", change.annotations)], placeholders,
    )?)
}
fn invalid(message: impl Into<String>) -> Error {
    Error::Protocol(message.into())
}
fn changes(edit: &Value) -> Result<Vec<Change>> {
    let mut output = Vec::new();
    if let Some(documents) = edit["documentChanges"].as_array() {
        for document in documents {
            let string = |field: &str| {
                document[field]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| invalid(format!("workspace operation lacks {field}")))
            };
            if document["options"]["overwrite"] == true
                || document["options"]["ignoreIfExists"] == true
                || document["options"]["ignoreIfNotExists"] == true
            {
                return Err(Error::Unavailable("overwrite/ignore filesystem operation semantics cannot be silently discarded from this preview".into()));
            }
            output.push(match document["kind"].as_str() {
                Some("create") => Change::Create(string("uri")?),
                Some("rename") => Change::Rename(string("oldUri")?, string("newUri")?),
                Some("delete") => Change::Delete(string("uri")?),
                Some(other) => {
                    return Err(Error::Unavailable(format!(
                        "unsupported native workspace operation {other}"
                    )));
                }
                None => Change::Edit {
                    uri: document["textDocument"]["uri"]
                        .as_str()
                        .ok_or_else(|| invalid("edit lacks source URI"))?
                        .into(),
                    version: document["textDocument"]["version"].as_i64(),
                    edits: document["edits"].clone(),
                },
            });
        }
    } else if let Some(files) = edit["changes"].as_object() {
        for (uri, edits) in files {
            output.push(Change::Edit {
                uri: uri.clone(),
                version: None,
                edits: edits.clone(),
            });
        }
    } else if !edit.is_null() && edit.as_object().is_none() {
        return Err(invalid("native workspace edit is not an object"));
    }
    Ok(output)
}
fn path(uri: &str, project: &Path, destination: bool) -> Result<PathBuf> {
    let original = url::Url::parse(uri)
        .map_err(|error| invalid(error.to_string()))?
        .to_file_path()
        .map_err(|()| invalid("workspace URI is not a local file"))?;
    let path = if destination {
        if original.exists() {
            return Err(Error::Unavailable(
                "filesystem preview destination already exists".into(),
            ));
        }
        original
            .parent()
            .ok_or_else(|| invalid("destination has no parent"))?
            .canonicalize()?
            .join(
                original
                    .file_name()
                    .ok_or_else(|| invalid("destination has no filename"))?,
            )
    } else {
        original.canonicalize()?
    };
    if !path.starts_with(project) {
        return Err(Error::Unavailable(
            "workspace edit lies outside selected project".into(),
        ));
    }
    Ok(path)
}
/// Original existing files that the native edit's next computation must synchronize.
pub(super) fn input_paths(edit: &Value, project: &Path) -> Result<Vec<PathBuf>> {
    let mut originals = BTreeSet::new();
    let mut created = BTreeSet::new();
    let mut aliases = BTreeMap::new();
    for change in changes(edit)? {
        match change {
            Change::Create(uri) => {
                created.insert(uri);
            }
            Change::Rename(old, new) => {
                let original = aliases.get(&old).cloned().unwrap_or(old);
                originals.insert(path(&original, project, false)?);
                aliases.insert(new, original);
            }
            Change::Delete(uri) => {
                originals.insert(path(aliases.get(&uri).unwrap_or(&uri), project, false)?);
            }
            Change::Edit { uri, .. } => {
                if !created.contains(&uri) {
                    originals.insert(path(aliases.get(&uri).unwrap_or(&uri), project, false)?);
                }
            }
        }
    }
    Ok(originals.into_iter().collect())
}
/// Acquire only sources synchronized before the final native computation. Versions stay checked.
pub(super) fn proposal(
    configuration: &Configuration,
    backend: &str,
    generation: u64,
    encoding: &str,
    edit: &Value,
    acquire: impl Fn(&Path, Option<i64>) -> Result<SourceText>,
) -> Result<ChangeProposal> {
    let mut sources = BTreeMap::new();
    let mut candidates = Vec::new();
    let mut operations = Vec::new();
    let mut placeholders = Vec::new();
    let mut created = BTreeMap::<String, usize>::new();
    let mut aliases = BTreeMap::new();
    let mut edited = BTreeSet::new();
    let mut limitations=vec!["Preview only. Preflight cannot fence concurrent noncooperating editors; no apply action is available.".into(),"Backend applicability/evaluation-order obligations require review; no equivalence or compiler acceptance follows.".into()];
    for change in changes(edit)? {
        match change {
            Change::Create(uri) => {
                let destination = path(&uri, &configuration.project, true)?;
                created.insert(uri, operations.len());
                operations.push(FileOperation::Create {
                    path: destination.into_os_string(),
                    text: String::new(),
                });
            }
            Change::Rename(old, new) => {
                let original = aliases.get(&old).cloned().unwrap_or(old);
                let source = acquire(&path(&original, &configuration.project, false)?, None)?;
                let destination = path(&new, &configuration.project, true)?;
                operations.push(FileOperation::Rename {
                    source: source.identity().clone(),
                    destination: destination.into_os_string(),
                });
                sources.insert(source.identity().path().to_owned(), source);
                aliases.insert(new, original);
            }
            Change::Delete(uri) => {
                let source = acquire(
                    &path(
                        aliases.get(&uri).unwrap_or(&uri),
                        &configuration.project,
                        false,
                    )?,
                    None,
                )?;
                operations.push(FileOperation::Delete {
                    source: source.identity().clone(),
                });
                sources.insert(source.identity().path().to_owned(), source);
            }
            Change::Edit {
                uri,
                version,
                edits,
            } => {
                if !edited.insert(uri.clone()) {
                    return Err(Error::Unavailable("multiple sequential native edit batches for one file need an explicit ordered proposal representation".into()));
                }
                let edits = edits
                    .as_array()
                    .ok_or_else(|| invalid("workspace replacements are not a list"))?;
                if edits.is_empty() {
                    continue;
                }
                if let Some(index) = created.get(&uri) {
                    let FileOperation::Create { text, .. } = &mut operations[*index] else {
                        return Err(invalid("created-file state differs"));
                    };
                    let mut ranges = Vec::new();
                    for edit in edits {
                        let start =
                            super::engine::byte_position(text, &edit["range"]["start"], encoding)?;
                        let end =
                            super::engine::byte_position(text, &edit["range"]["end"], encoding)?;
                        let replacement = edit["newText"]
                            .as_str()
                            .ok_or_else(|| invalid("new file edit lacks text"))?;
                        if edit["insertTextFormat"] == 2 {
                            limitations.push(
                                "Created-file text retains an unresolved native snippet verbatim."
                                    .into(),
                            );
                        }
                        ranges.push((start, end, replacement));
                    }
                    ranges.sort_by_key(|(start, end, _)| (*start, *end));
                    if ranges
                        .windows(2)
                        .any(|pair| pair[0].1 > pair[1].0 || pair[0].0 == pair[1].0)
                    {
                        return Err(Error::Unavailable(
                            "overlapping native new-file edits have no selected order".into(),
                        ));
                    }
                    for (start, end, replacement) in ranges.into_iter().rev() {
                        if text.get(start..end).is_none() {
                            return Err(invalid("invalid created-file range"));
                        }
                        text.replace_range(start..end, replacement);
                    }
                } else {
                    let source = acquire(
                        &path(
                            aliases.get(&uri).unwrap_or(&uri),
                            &configuration.project,
                            false,
                        )?,
                        version,
                    )?;
                    for edit in edits {
                        let start = super::engine::byte_position(
                            source.text(),
                            &edit["range"]["start"],
                            encoding,
                        )?;
                        let end = super::engine::byte_position(
                            source.text(),
                            &edit["range"]["end"],
                            encoding,
                        )?;
                        let replacement = edit["newText"]
                            .as_str()
                            .ok_or_else(|| invalid("workspace replacement has no text"))?
                            .to_string();
                        if edit["insertTextFormat"] == 2
                            || replacement.contains("$0")
                            || replacement.contains("${")
                        {
                            placeholders.push(UnresolvedPlaceholder{candidate:candidates.len(),description:"Native snippet retained verbatim; placeholders remain unresolved.".into()});
                        }
                        candidates.push(EditCandidate::new(
                            source.identity().clone(),
                            start..end,
                            replacement,
                        ));
                    }
                    sources.insert(source.identity().path().to_owned(), source);
                }
            }
        }
    }
    let selected = (0..candidates.len()).collect();
    Ok(ChangeProposal::with_file_operations(
        ProducerContext::Semantic {
            backend: backend.into(),
            configuration: configuration.identity()?,
            synchronized_generation: generation.to_string(),
        },
        sources.into_values().collect(),
        candidates,
        selected,
        operations,
        limitations,
        placeholders,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn workspace_effects_preserve_originals_snippets_and_all_files_without_writes() {
        let scratch = tempfile::tempdir().unwrap();
        let project = scratch.path().to_owned();
        let old = project.join("old.rs");
        let gone = project.join("gone.rs");
        let new = project.join("new.rs");
        let moved = project.join("moved.rs");
        std::fs::write(&old, "pub fn old() {}\n").unwrap();
        std::fs::write(&gone, "obsolete\n").unwrap();
        let uri = |path: &Path| super::super::engine::uri(path).unwrap();
        let point = json!({"line":0,"character":0});
        let native = json!({"documentChanges":[
            {"kind":"create","uri":uri(&new)},
            {"textDocument":{"uri":uri(&new),"version":null},"edits":[{"range":{"start":point,"end":point},"newText":"pub fn ${1:new}() {}\n","insertTextFormat":2}]},
            {"textDocument":{"uri":uri(&old),"version":7},"edits":[{"range":{"start":{"line":0,"character":7},"end":{"line":0,"character":10}},"newText":"renamed"}]},
            {"kind":"rename","oldUri":uri(&old),"newUri":uri(&moved)},
            {"kind":"delete","uri":uri(&gone)}
        ]});
        let inputs = input_paths(&native, &project).unwrap();
        assert!(inputs.contains(&old) && inputs.contains(&gone));
        assert_eq!(inputs.len(), 2);
        let configuration = Configuration {
            backend: super::super::Backend::Lsp,
            project,
            features: vec![],
            no_default_features: false,
            target: None,
            build_macros: false,
            test_context: false,
        };
        let proposal = proposal(
            &configuration,
            "test-native-provider",
            3,
            "utf-8",
            &native,
            |path, version| {
                if let Some(version) = version {
                    assert_eq!(version, 7);
                }
                Ok(crate::source_read::SourceRequest::Working {
                    path: path.to_owned(),
                }
                .acquire()?)
            },
        )
        .unwrap();
        assert_eq!(proposal.file_operations().len(), 3);
        assert_eq!(proposal.selected().len(), 1);
        assert_eq!(proposal.inspect(0).unwrap().original, "old");
        assert!(
            matches!(&proposal.file_operations()[0],FileOperation::Create{text,..}if text.contains("${1:new}"))
        );
        assert!(
            proposal
                .limitations()
                .iter()
                .any(|limit| limit.contains("snippet"))
        );
        assert_eq!(std::fs::read_to_string(&old).unwrap(), "pub fn old() {}\n");
        assert!(gone.exists());
        assert!(!new.exists() && !moved.exists());
        let overwrite = json!({"documentChanges":[{"kind":"create","uri":uri(&new),"options":{"overwrite":true}}]});
        assert!(input_paths(&overwrite, &configuration.project).is_err());
    }
}
