// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Resolve authored Rustdoc symbol links only in their native declaration scope.
use super::*;
use crate::semantic::{DocumentationLinkObservation, NativeInvestigationFacts};
use ra_ap_hir::{DocLinkDef, HasAttrs as _, ModuleDef};

pub(super) fn inspect(
    db: &ra_ap_ide_db::RootDatabase,
    paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
    position: ra_ap_ide::FilePosition,
    limit: usize,
) -> Result<NativeInvestigationFacts> {
    let sema = Semantics::new(db);
    let parsed = sema.parse(ra_ap_hir::EditionedFileId::current_edition(
        db,
        position.file_id,
    ));
    let node = parsed
        .syntax()
        .token_at_offset(position.offset)
        .right_biased()
        .and_then(|token| token.parent());
    let mut definition = None;
    for node in node.into_iter().flat_map(|node| node.ancestors()) {
        definition = if let Some(item) = ast::Fn::cast(node.clone()) {
            sema.to_def(&item).map(ModuleDef::Function)
        } else if let Some(item) = ast::Adt::cast(node.clone()) {
            sema.to_adt_def(&item).map(ModuleDef::Adt)
        } else if let Some(item) = ast::Trait::cast(node.clone()) {
            sema.to_def(&item).map(ModuleDef::Trait)
        } else if let Some(item) = ast::TypeAlias::cast(node.clone()) {
            sema.to_def(&item).map(ModuleDef::TypeAlias)
        } else if let Some(item) = ast::Const::cast(node.clone()) {
            sema.to_def(&item).map(ModuleDef::Const)
        } else if let Some(item) = ast::Static::cast(node.clone()) {
            sema.to_def(&item).map(ModuleDef::Static)
        } else if let Some(item) = ast::Module::cast(node) {
            sema.to_def(&item).map(ModuleDef::Module)
        } else {
            None
        };
        if definition.is_some() {
            break;
        }
    }
    let Some(definition) = definition else {
        return Ok(NativeInvestigationFacts {
            limitations: vec![
                "selected declaration has no supported native documentation owner".into(),
            ],
            ..Default::default()
        });
    };
    let attributes = definition.attrs(db);
    let Some(docs) = attributes.hir_docs(db) else {
        return Ok(NativeInvestigationFacts::default());
    };
    let callback = |link: pulldown_cmark::BrokenLink<'_>| {
        Some((link.reference.into_string().into(), "".into()))
    };
    let parser = pulldown_cmark::Parser::new_with_broken_link_callback(
        docs.docs(),
        pulldown_cmark::Options::empty(),
        Some(callback),
    );
    let owner = investigation::definition_location(&sema, definition, paths);
    let mut result = NativeInvestigationFacts::default();
    let mut total = 0;
    for (event, range) in parser.into_offset_iter() {
        let pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link { dest_url, .. }) = event else {
            continue;
        };
        let written = dest_url.into_string();
        if written.contains("://") || written.contains('/') || written.starts_with('#') {
            continue;
        }
        total += 1;
        if result.documentation_links.len() >= limit {
            continue;
        }
        let mut link = written.trim_matches('`');
        let (namespace, namespace_name) = if let Some((prefix, path)) = link.split_once('@') {
            link = path;
            match prefix {
                "struct" | "enum" | "trait" | "type" | "mod" | "union" | "primitive" => {
                    (Some(ra_ap_hir::Namespace::Types), Some(prefix.to_owned()))
                }
                "fn" | "const" | "static" | "value" => {
                    (Some(ra_ap_hir::Namespace::Values), Some(prefix.to_owned()))
                }
                "macro" => (Some(ra_ap_hir::Namespace::Macros), Some(prefix.to_owned())),
                _ => (None, Some(prefix.to_owned())),
            }
        } else {
            (None, None)
        };
        let range = native_ra_syntax::TextRange::new(
            native_ra_syntax::TextSize::from(u32::try_from(range.start).map_err(|_| {
                Error::Unavailable("documentation offset exceeds native range".into())
            })?),
            native_ra_syntax::TextSize::from(u32::try_from(range.end).map_err(|_| {
                Error::Unavailable("documentation extent exceeds native range".into())
            })?),
        );
        let mapped = docs.find_ast_range(range);
        let mut limitations = Vec::new();
        let supported_namespace = namespace_name.as_deref().is_none_or(|prefix| {
            matches!(
                prefix,
                "struct"
                    | "enum"
                    | "trait"
                    | "type"
                    | "mod"
                    | "union"
                    | "primitive"
                    | "fn"
                    | "const"
                    | "static"
                    | "value"
                    | "macro"
            )
        });
        let target = if !supported_namespace {
            limitations.push(
                "unsupported explicit Rustdoc namespace disambiguator; no unqualified fallback"
                    .into(),
            );
            None
        } else if let Some((_, inner)) = mapped {
            ra_ap_hir::resolve_doc_path_on(db, definition, link, namespace, inner).and_then(
                |target| match target {
                    DocLinkDef::ModuleDef(target) => {
                        investigation::definition_location(&sema, target, paths)
                    }
                    DocLinkDef::SelfType(target) => {
                        investigation::definition_location(&sema, ModuleDef::Trait(target), paths)
                    }
                    DocLinkDef::Field(target) => {
                        let source = sema.source(target)?;
                        investigation::mapped(&sema, source.value.syntax(), paths)
                    }
                },
            )
        } else {
            limitations.push(
                "documentation range has no unique original mapping; inner/outer scope is unknown"
                    .into(),
            );
            None
        };
        if target.is_none() {
            limitations.push("native target unresolved or lacks original source mapping; written link is retained".into());
        }
        result
            .documentation_links
            .push(DocumentationLinkObservation {
                written,
                owner: owner.clone(),
                target,
                namespace: namespace_name,
                limitations,
            });
    }
    if total > result.documentation_links.len() {
        result.limitations.push(format!(
            "{} whole documentation link observations omitted",
            total - result.documentation_links.len()
        ));
    }
    Ok(result)
}
