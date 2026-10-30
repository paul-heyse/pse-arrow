// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read-only document ownership through existing native metadata and section readers.
#[path = "../../../scripts/document-ownership.rs"]
pub mod protocol;
use super::{adr, lifecycle, metadata, site};
use anyhow::{Context, Result, ensure};
use clap::Args;
use protocol::{DocumentOwnership, Section, Workflow};
use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
};

/// Native owner selection.
#[derive(Debug, Args)]
#[command(group(clap::ArgGroup::new("document-owner-selection").required(true).multiple(false).args(["path", "schema"])))]
pub struct Query {
    /// Exact document path, absolute or checkout relative.
    pub path: Option<PathBuf>,
    /// Emit the response schema without accessing a checkout.
    #[arg(long)]
    pub schema: bool,
}

/// Interpret exactly one existing document through its native owners.
/// # Errors
/// Outside/non-Markdown paths, unavailable sources or native interpretation failures.
pub fn read(root: &Path, path: &Path) -> Result<DocumentOwnership> {
    let root = site::resolved(root)?;
    let path = site::resolved(&root.join(path))?;
    let relative = path
        .strip_prefix(&root)
        .context("document is outside checkout")?;
    ensure!(
        path.extension().is_some_and(|extension| extension == "md"),
        "document owner requires a Markdown source"
    );
    let identity = relative
        .to_str()
        .context("native document identity requires UTF-8")?
        .replace('\\', "/");
    let raw = fs::read(&path)?;
    let document = metadata::read(&raw, &identity, false, Some(&identity), &root)?;
    let policy = lifecycle::load_policy(&root, None)?;
    let config = site::configuration(&root)?;
    let values = lifecycle::interpreted(&root, &path, &document, &policy, &config)?;
    let field = |name: &str| {
        values
            .get(name)
            .and_then(metadata::Value::as_str)
            .map(str::to_owned)
    };
    let status = document
        .metadata
        .get("status")
        .and_then(metadata::Value::as_str)
        .map(str::to_owned);
    let architecture = relative.starts_with("docs/authoritative_design");
    let decision = if relative.starts_with("docs/adr") {
        adr::adr_paths(&root)?
            .into_iter()
            .map(|path| site::resolved(&path))
            .collect::<Result<Vec<_>>>()?
            .contains(&path)
    } else {
        false
    };
    let workflow = if architecture {
        Workflow::ArchitectureDecision
    } else if decision && status.as_deref() == Some("proposed") {
        Workflow::ProposedDecision
    } else if decision {
        Workflow::SupersedeDecision
    } else {
        Workflow::DocumentOwner
    };
    let sections = if architecture {
        adr::section_owners(&root.join("docs/authoritative_design/blueprint.md"))?
            .into_iter()
            .filter(|(_, (owner, _))| owner == &path)
            .map(|(id, (_, heading))| Section { id, heading })
            .collect()
    } else {
        Vec::new()
    };
    Ok(DocumentOwnership {
        path: path.into_os_string(), doc_owner: field("doc_owner"),
        historical_owner: field("historical_owner"), doc_role: field("doc_role"),
        doc_retention: field("doc_retention"), status, sections, workflow,
        coverage_limits: vec![
            "Current native document metadata and exact sections only; no code-path-to-plan binding is inferred.".into(),
            "Read availability does not establish implementation, qualification or permission to mutate an immutable record.".into(),
        ],
    })
}

/// Emit only the response schema, without workspace lookup.
/// # Errors
/// Serialization or terminal I/O failure.
pub fn schema() -> Result<()> {
    let mut output = std::io::stdout().lock();
    serde_json::to_writer_pretty(&mut output, &schemars::schema_for!(DocumentOwnership))?;
    writeln!(output)?;
    Ok(())
}
/// Execute a selected native query.
/// # Errors
/// Contradictory selection, native source failure or terminal I/O failure.
pub fn execute(root: &Path, query: Query) -> Result<()> {
    ensure!(
        query.path.is_some() != query.schema,
        "select exactly PATH or --schema"
    );
    if query.schema {
        return schema();
    }
    let value = read(root, query.path.as_deref().context("document path absent")?)?;
    let mut output = std::io::stdout().lock();
    serde_json::to_writer_pretty(&mut output, &value)?;
    writeln!(output)?;
    Ok(())
}

#[cfg(test)]
mod tests;
