// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One original source cut for canonical lifecycle, section and citation interpretation.
use super::*;
use protocol::{Association, AssociationExport, InputEvidence};
use pulldown_cmark::{Event, Parser, Tag};
use sha2::{Digest as _, Sha256};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    path::Component,
    process::Command,
};

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn git(root: &Path, arguments: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()?;
    ensure!(
        output.status.success(),
        "selected Git source unavailable: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}
fn selected(root: &Path, context: &str, path: &str) -> Result<Vec<u8>> {
    if context == "working" {
        Ok(fs::read(root.join(path))?)
    } else {
        git(
            root,
            &[
                "show",
                &format!("{}:{path}", if context == "index" { "" } else { context }),
            ],
        )
    }
}
fn membership(root: &Path, context: &str, prefix: &str) -> Result<Vec<PathBuf>> {
    if context == "working" {
        let mut paths = Vec::new();
        fn walk(path: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
            for entry in fs::read_dir(path)? {
                let path = entry?.path();
                if path.is_dir() {
                    walk(&path, paths)?;
                } else if path.extension().is_some_and(|extension| extension == "md") {
                    paths.push(path);
                }
            }
            Ok(())
        }
        walk(&root.join(prefix), &mut paths)?;
        Ok(paths)
    } else {
        let bytes = if context == "index" {
            git(root, &["ls-files", "-z", "--", prefix])?
        } else {
            git(
                root,
                &["ls-tree", "-r", "--name-only", "-z", context, "--", prefix],
            )?
        };
        bytes
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .map(|path| Ok(root.join(std::str::from_utf8(path)?)))
            .filter(|path: &Result<PathBuf>| {
                path.as_ref()
                    .is_ok_and(|path| path.extension().is_some_and(|extension| extension == "md"))
            })
            .collect()
    }
}

/// Read native document associations exclusively through the selected original source universe.
/// # Errors
/// Invalid source paths, unavailable selected document, changed witness or changed input cut.
pub fn read_associations(
    root: &Path,
    path: &Path,
    revision: Option<&str>,
    index: bool,
    if_source: Option<&str>,
    citations: &[String],
) -> Result<AssociationExport> {
    let root = site::resolved(root)?;
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    ensure!(
        !path
            .components()
            .any(|part| matches!(part, Component::ParentDir)),
        "document association path contains parent traversal"
    );
    let relative = path
        .strip_prefix(&root)?
        .to_str()
        .context("document identity requires UTF-8")?
        .replace('\\', "/");
    let markdown = path.extension().is_some_and(|extension| extension == "md");
    ensure!(
        markdown || path.extension().is_some_and(|extension| extension == "rs"),
        "document associations require Markdown or authored Rust documentation citations"
    );
    let context = if let Some(revision) = revision {
        String::from_utf8(git(
            &root,
            &["rev-parse", "--verify", &format!("{revision}^{{commit}}")],
        )?)?
        .trim()
        .to_owned()
    } else if index {
        "index".into()
    } else {
        "working".into()
    };
    let cache = RefCell::new(BTreeMap::<String, Vec<u8>>::new());
    let reader = |relative: &str| -> Result<Vec<u8>> {
        if let Some(bytes) = cache.borrow().get(relative) {
            return Ok(bytes.clone());
        }
        let bytes = selected(&root, &context, relative)?;
        cache.borrow_mut().insert(relative.into(), bytes.clone());
        Ok(bytes)
    };
    let raw = reader(&relative)?;
    let source_sha256 = digest(&raw);
    ensure!(
        if_source.is_none_or(|expected| expected == source_sha256),
        "selected document source changed"
    );
    let mut result = AssociationExport {
        path: path.clone().into_os_string(),
        context: context.clone(),
        source_sha256,
        document: None,
        metadata_origins: serde_json::Value::Null,
        scope_items: Vec::new(),
        associations: Vec::new(),
        inputs: Vec::new(),
        limitations: Vec::new(),
    };
    let policy_raw = reader("docs/lifecycle.toml");
    let policy = lifecycle::load_policy(&root, Some(&reader));
    let config = reader("docs/site.toml").and_then(|raw| {
        Ok(toml::from_str::<site::Configuration>(std::str::from_utf8(
            &raw,
        )?)?)
    });
    let document = policy_raw
        .as_ref()
        .map_err(|error| anyhow::anyhow!(error.to_string()))
        .and_then(|policy_raw| {
            Ok(metadata::read_with_policy(
                &raw,
                &relative,
                false,
                Some(&relative),
                &root,
                Some(std::str::from_utf8(policy_raw)?),
            )?)
        });
    let mut sections = BTreeMap::new();
    match membership(&root, &context, "docs/authoritative_design").and_then(|paths| {
        adr::section_owners_from_sources(&paths, &|path| {
            Ok(String::from_utf8(reader(
                path.strip_prefix(&root)?
                    .to_str()
                    .context("section path is not UTF-8")?,
            )?)?)
        })
    }) {
        Ok(value) => sections = value,
        Err(error) => result.limitations.push(format!(
            "selected native section interpretation unavailable: {error}"
        )),
    }
    if markdown {
        match (document, policy, config) {
            (Ok(document), Ok(policy), Ok(config)) => {
                match lifecycle::interpreted_with_reader(
                    &root,
                    &path,
                    &document,
                    &policy,
                    &config,
                    Some(&reader),
                ) {
                    Ok(values) => {
                        let field = |name: &str| {
                            values
                                .get(name)
                                .and_then(metadata::Value::as_str)
                                .map(str::to_owned)
                        };
                        result.metadata_origins = values
                            .get("metadata_origins")
                            .map(serde_json::to_value)
                            .transpose()?
                            .unwrap_or_default();
                        let status = document
                            .metadata
                            .get("status")
                            .and_then(metadata::Value::as_str)
                            .map(str::to_owned);
                        let architecture = relative.starts_with("docs/authoritative_design/");
                        let decision = relative.starts_with("docs/adr/")
                            && adr::is_adr_path(Path::new(&relative));
                        let workflow = if architecture {
                            Workflow::ArchitectureDecision
                        } else if decision && status.as_deref() == Some("proposed") {
                            Workflow::ProposedDecision
                        } else if decision {
                            Workflow::SupersedeDecision
                        } else {
                            Workflow::DocumentOwner
                        };
                        result.document = Some(DocumentOwnership { path: path.clone().into_os_string(), doc_owner: field("doc_owner"), historical_owner: field("historical_owner"), doc_role: field("doc_role"), doc_retention: field("doc_retention"), status, sections: sections.iter().filter(|(_, (owner, _))| owner == &path).map(|(id, (_, heading))| Section { id: id.clone(), heading: heading.clone() }).collect(), workflow, coverage_limits: vec!["Metadata, status and workflow are interpreted from the selected original inputs; they establish neither implementation nor current authorization.".into()] });
                    }
                    Err(error) => result.limitations.push(format!(
                        "selected native metadata interpretation unavailable: {error}"
                    )),
                }
                let documents = BTreeSet::from([relative.clone()]);
                match lifecycle::aggregate(&root, &policy, Some(&documents), Some(&reader)) {
                    Ok(items) => {
                        result.scope_items = items
                            .iter()
                            .filter(|item| item.document == relative)
                            .map(serde_json::to_value)
                            .collect::<Result<Vec<_>, _>>()?
                    }
                    Err(error) => result.limitations.push(format!(
                        "selected native scope bindings unavailable: {error}"
                    )),
                }
            }
            (document, policy, config) => {
                for (name, error) in [
                    ("metadata", document.err().map(|error| error.to_string())),
                    ("policy", policy.err().map(|error| error.to_string())),
                    (
                        "site configuration",
                        config.err().map(|error| error.to_string()),
                    ),
                ] {
                    if let Some(error) = error {
                        result.limitations.push(format!("selected {name} unavailable: {error}; current defaults are not substituted"));
                    }
                }
            }
        }
    }
    let text = std::str::from_utf8(&raw)?;
    for event in Parser::new(if markdown { text } else { "" }) {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            let written = dest_url.into_string();
            let resolved =
                lifecycle::reference_with_reader(&root, &written, Some(&path), Some(&reader));
            result.associations.push(Association {
                basis: "explicit_markdown_link; not_ownership".into(),
                written,
                resolved: resolved.as_ref().ok().cloned(),
                limitation: resolved.err().map(|error| error.to_string()),
            });
        }
    }
    let section_pattern = regex::Regex::new(r"blueprint\s+§([0-9]+(?:\.[0-9]+)*)")?;
    let citations_text = if markdown {
        text.to_owned()
    } else {
        citations.join("\n")
    };
    for matched in section_pattern.captures_iter(&citations_text) {
        let resolved = sections.get(&matched[1]).map(|(owner, _)| {
            format!(
                "{}#section-{}",
                owner.strip_prefix(&root).unwrap_or(owner).to_string_lossy(),
                matched[1].replace('.', "-")
            )
        });
        result.associations.push(Association {
            basis: "explicit_architecture_section_citation; not_ownership".into(),
            written: matched[0].into(),
            limitation: resolved
                .is_none()
                .then(|| "native section owner unavailable in selected source universe".into()),
            resolved,
        });
    }
    for citation in citations
        .iter()
        .filter(|citation| !citation.starts_with("blueprint §"))
    {
        let resolved =
            lifecycle::reference_with_reader(&root, citation, Some(&path), Some(&reader));
        result.associations.push(Association {
            basis: "explicit_source_documentation_citation; not_ownership".into(),
            written: citation.clone(),
            resolved: resolved.as_ref().ok().cloned(),
            limitation: resolved.err().map(|error| error.to_string()),
        });
    }
    for (relative, bytes) in cache.into_inner() {
        ensure!(
            selected(&root, &context, &relative)? == bytes,
            "selected document/policy input changed during interpretation: {relative}"
        );
        result.inputs.push(InputEvidence {
            path: relative,
            sha256: digest(&bytes),
        });
    }
    Ok(result)
}
