// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Qualified source/document associations; written mention never becomes ownership.
use crate::{
    HostError,
    investigation::{self, Locator, ResultReference, SourceKind},
    ownership::document::Association,
};
use ra_ap_syntax::{AstNode as _, ast, ast::HasAttrs as _};
use schemars::JsonSchema;
use serde::Serialize;
use std::path::Path;
use tokio_util::sync::CancellationToken;

/// One source-bound association inquiry with independent authority and citation bases.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    /// Exact original source/native operand.
    pub reference: ResultReference,
    /// Written source evidence, independently useful without a document interpreter.
    pub source: investigation::SearchResult,
    /// Canonical source-path plan authority; package names do not supply this binding.
    pub owning_plan: crate::ownership::PlanOwnership,
    /// Current declared generated-output/contributor evidence, when applicable.
    pub generated: Option<crate::ownership::OwnerQuery>,
    /// Canonical original metadata/defaults/sections/native scope-table interpretation.
    pub document: Option<crate::ownership::document::AssociationExport>,
    /// Authored citations nominated from lossless Rust doc facts or Markdown syntax.
    pub citations: Vec<Association>,
    /// Original documentation/comment/literal facts, including decoded-to-source mappings.
    pub documentation: Vec<crate::source_read::SourceFact>,
    /// Optional explicitly selected native intra-document Rust symbol resolution.
    pub native: Option<crate::semantic::Answer>,
    /// Whole omitted citation records.
    pub omitted: usize,
    /// Selected evidence delivery, independent of implementation or qualification.
    pub complete: bool,
    /// Missing interpreter, historical inputs or unresolved references remain unknown.
    pub limitations: Vec<String>,
}

/// Read original source, canonical documentation and optionally an already prepared native world.
pub async fn inspect(
    root: &Path,
    locator: &Locator,
    semantic: bool,
    budget: usize,
    token: &CancellationToken,
) -> Result<Report, HostError> {
    if token.is_cancelled() {
        return Err(investigation::Error::Cancelled.into());
    }
    let (source_root, selected, text) =
        crate::relevance::source_witness(root, locator, token).await?;
    let reference = ResultReference::new(if matches!(locator, Locator::Source { .. }) {
        selected.clone()
    } else {
        locator.clone()
    });
    let mut source = investigation::providers::inspect(&source_root, &selected)?;
    let Locator::Source {
        path,
        source: source_kind,
        range,
        ..
    } = &selected
    else {
        return Err(investigation::Error::Invalid(
            "associations require source/native evidence".into(),
        )
        .into());
    };
    let markdown = path
        .to_path_buf()
        .extension()
        .is_some_and(|extension| extension == "md");
    let (documentation, documentation_limits) = if markdown {
        (Vec::new(), Vec::new())
    } else {
        rust_documentation(text.text(), range.as_ref())
    };
    let docs = if markdown {
        source.context.clone()
    } else {
        documentation
            .iter()
            .filter_map(|fact| match &fact.value {
                Some(crate::source_read::InterpretedValue::Text(text)) => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut citations = Vec::new();
    let mut requested = Vec::new();
    for event in pulldown_cmark::Parser::new(&docs) {
        if let pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link { dest_url, .. }) = event {
            let written = dest_url.into_string();
            requested.push(written.clone());
            citations.push(Association { basis: if markdown { "explicit_markdown_link" } else { "explicit_source_documentation_link" }.into(), written, resolved: None, limitation: Some("written citation only; canonical resolution is separate and does not establish ownership or implementation".into()) });
        }
    }
    // Source doc prose nominates stable architecture citations; only the native section
    // interpreter can resolve a moved section to its selected original owner.
    for part in docs.split("blueprint §").skip(1) {
        let section: String = part
            .chars()
            .take_while(|character| character.is_ascii_digit() || *character == '.')
            .collect();
        if section.is_empty() {
            continue;
        }
        let written = format!("blueprint §{}", section.trim_end_matches('.'));
        requested.push(written.clone());
        citations.push(Association {
            basis: "explicit_architecture_section_citation".into(),
            written,
            resolved: None,
            limitation: Some(
                "native section owner not yet resolved; citation is not ownership".into(),
            ),
        });
    }
    let mut report = Report { reference, source: source.clone(), owning_plan: crate::ownership::PlanOwnership::NotDeclared { reason: "source/package membership and prose mentions do not declare a source-path-to-active-plan binding".into() }, generated: None, document: None, citations, documentation, native: None, omitted: 0, complete: documentation_limits.is_empty(), limitations: vec!["Declared ownership, generated-source origin, explicit citation, native Rust relation and heuristic nomination have distinct bases. A rejected-alternative mention remains a mention.".into(), "Document status is an original native observation, not implementation acceptance, qualification or permission to edit. Library documentation is reference evidence with its own version/conditions.".into()] };
    report.limitations.extend(documentation_limits);
    if matches!(source_kind, SourceKind::Working) {
        match crate::ownership::query(root, &path.to_path_buf()) {
            Ok(value) => report.generated = Some(value),
            Err(error) => {
                report.complete = false;
                report.limitations.push(format!(
                    "declared generated-source routing unavailable: {error}"
                ));
            }
        }
    } else {
        report.complete = false;
        report.limitations.push("Current generated-output declarations are not substituted for historical/index origin declarations; use original origin evidence where available.".into());
    }
    let inputs = std::env::vars_os().collect();
    match crate::prepared_xtask::PreparedNativeTool::current(root, &inputs).and_then(|tool| {
        tool.document_associations(
            &path.to_path_buf(),
            source_kind,
            text.identity().source_sha256(),
            &requested,
            &inputs,
        )
    }) {
        Ok(document) => {
            report.complete &= document.limitations.is_empty();
            report.document = Some(document);
        }
        Err(error) => {
            report.complete = false;
            report.limitations.push(format!("canonical document interpreter unavailable: {error}; source citations remain independently inspectable; no helper is prepared implicitly"));
        }
    }
    if semantic {
        match investigation::rust::source_inquiry(root, locator, token).await {
            Ok(native) => match investigation::rust::semantic_operation_cancellable(
                root,
                &native,
                crate::semantic::Kind::DocumentationLinks,
                token,
            )
            .await
            {
                Ok(answer) => {
                    let incomplete = crate::relevance::native_incompleteness(&answer);
                    report.complete &= incomplete.is_empty();
                    report.limitations.extend(incomplete);
                    report.native = Some(answer);
                }
                Err(error) => {
                    if token.is_cancelled() {
                        return Err(error.into());
                    }
                    report.complete = false;
                    report.limitations.push(format!(
                        "requested native documentation resolution unavailable: {error}"
                    ));
                }
            },
            Err(error) => {
                if token.is_cancelled() {
                    return Err(error.into());
                }
                report.complete = false;
                report.limitations.push(format!(
                    "requested native source correspondence unavailable: {error}"
                ));
            }
        }
    }
    report.omitted = report.citations.len().saturating_sub(budget);
    report.citations.truncate(budget);
    report.complete &= report.omitted == 0;
    source.context.clear();
    report.source = source;
    Ok(report)
}

fn rust_documentation(
    source: &str,
    selected: Option<&investigation::MatchRange>,
) -> (Vec<crate::source_read::SourceFact>, Vec<String>) {
    let (outline, syntax) =
        crate::source_read::rust_with_syntax(source, crate::source_read::RustEdition::E2024);
    let enclosing = selected.and_then(|selected| {
        outline
            .entries
            .iter()
            .filter(|entry| {
                entry.range.byte_start <= selected.byte_start
                    && entry.range.byte_end >= selected.byte_end
            })
            .min_by_key(|entry| entry.range.byte_end - entry.range.byte_start)
    });
    let mut attributes = Vec::new();
    if selected.is_none()
        && let Some(file) = ast::SourceFile::cast(syntax.clone())
    {
        attributes.extend(file.attrs_with_doc());
    }
    for item in syntax.descendants().filter_map(ast::Item::cast) {
        let range = item.syntax().text_range();
        if selected.is_none()
            || enclosing.is_some_and(|owner| {
                owner.range.byte_start == usize::from(range.start())
                    && owner.range.byte_end == usize::from(range.end())
            })
        {
            attributes.extend(item.attrs_with_doc());
        }
    }
    let mut output = Vec::new();
    let mut limitations = Vec::new();
    for attribute in attributes {
        let node = attribute.syntax();
        let target = if ast::DocComment::cast(node.clone()).is_some() {
            Some(node.text_range())
        } else if let Some(attribute) = ast::Attr::cast(node.clone()) {
            match attribute.meta() {
                Some(ast::Meta::KeyValueMeta(meta))
                    if meta
                        .path()
                        .is_some_and(|path| path.syntax().text() == "doc") =>
                {
                    if let Some(ast::Expr::Literal(literal)) = meta.expr() {
                        Some(literal.syntax().text_range())
                    } else {
                        limitations.push("authored doc attribute has a nonliteral expression; source inspection does not expand or evaluate it".into());
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        };
        if let Some(target) = target {
            if let Some(fact) = outline.facts.iter().find(|fact| {
                fact.range.byte_start == usize::from(target.start())
                    && fact.range.byte_end == usize::from(target.end())
            }) {
                if matches!(
                    fact.value,
                    Some(crate::source_read::InterpretedValue::Text(_))
                ) {
                    output.push(fact.clone());
                } else {
                    limitations.push("authored documentation literal is invalid or nontextual; original source remains inspectable".into());
                }
            } else {
                limitations.push("documentation has no exact interpreted source fact; original source remains inspectable".into());
            }
        }
    }
    if !outline.parse_issues.is_empty() {
        limitations.push(format!(
            "source documentation parse has {} recovery issues",
            outline.parse_issues.len()
        ));
    }
    (output, limitations)
}
