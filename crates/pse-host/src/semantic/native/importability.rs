// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native visibility and import paths from one immutable HIR world.
use crate::semantic::{Error, Result};
use native_ra_syntax::{
    AstNode as _, SyntaxNode, TextSize,
    ast::{self, HasName as _},
};
use ra_ap_hir::{AsAssocItem as _, HasVisibility as _, ModuleDef, Semantics};
use serde::Serialize;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Serialize)]
pub(super) struct Fact {
    pub visibility: Option<bool>,
    pub importable: Option<bool>,
    pub path: Option<String>,
    pub limitations: Vec<String>,
}
impl Fact {
    fn unknown(reason: &str) -> Self {
        Self {
            visibility: None,
            importable: None,
            path: None,
            limitations: vec![reason.into()],
        }
    }
}

fn definition(
    sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
    node: &SyntaxNode,
    offset: TextSize,
) -> Option<ModuleDef> {
    for ancestor in node.ancestors() {
        if let Some(variant) = ast::Variant::cast(ancestor.clone()) {
            if variant
                .name()
                .is_some_and(|name| name.syntax().text_range().contains(offset))
            {
                return sema.to_def(&variant).map(ModuleDef::from);
            }
        }
        let Some(item) = ast::Item::cast(ancestor) else {
            continue;
        };
        macro_rules! named {
            ($item:expr) => {
                if $item
                    .name()
                    .is_some_and(|name| name.syntax().text_range().contains(offset))
                {
                    return sema.to_def(&$item).map(ModuleDef::from);
                }
            };
        }
        match item {
            ast::Item::Fn(item) => named!(item),
            ast::Item::Struct(item) => named!(item),
            ast::Item::Enum(item) => named!(item),
            ast::Item::Union(item) => named!(item),
            ast::Item::Trait(item) => named!(item),
            ast::Item::Module(item) => named!(item),
            ast::Item::Const(item) => named!(item),
            ast::Item::Static(item) => named!(item),
            ast::Item::TypeAlias(item) => named!(item),
            _ => {}
        }
    }
    None
}

pub(super) fn inspect(
    db: &ra_ap_ide_db::RootDatabase,
    target: ra_ap_ide::FilePosition,
    destination: ra_ap_ide::FilePosition,
    token: &CancellationToken,
) -> Result<Fact> {
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    let sema = Semantics::new(db);
    if sema.file_to_module_defs(target.file_id).take(2).count() != 1
        || sema
            .file_to_module_defs(destination.file_id)
            .take(2)
            .count()
            != 1
    {
        return Ok(Fact::unknown(
            "Target or destination is inactive or has multiple native crate/module contexts; importability is not established.",
        ));
    }
    let target_source = sema.parse(ra_ap_hir::EditionedFileId::current_edition(
        db,
        target.file_id,
    ));
    let Some(node) = target_source
        .syntax()
        .token_at_offset(target.offset)
        .right_biased()
        .and_then(|token| token.parent())
    else {
        return Ok(Fact::unknown(
            "The guarded target has no native declaration token.",
        ));
    };
    let Some(item) = definition(&sema, &node, target.offset) else {
        return Ok(Fact::unknown(
            "The guarded token is not a uniquely mapped native importable item declaration.",
        ));
    };
    if match item {
        ModuleDef::Function(item) => item.as_assoc_item(db).is_some(),
        ModuleDef::Const(item) => item.as_assoc_item(db).is_some(),
        ModuleDef::TypeAlias(item) => item.as_assoc_item(db).is_some(),
        _ => false,
    } {
        return Ok(Fact::unknown(
            "Associated-item use requires its receiver/trait context; a module import path alone does not establish applicability.",
        ));
    }
    let destination_source = sema.parse(ra_ap_hir::EditionedFileId::current_edition(
        db,
        destination.file_id,
    ));
    let Some(node) = destination_source
        .syntax()
        .token_at_offset(destination.offset)
        .right_biased()
        .and_then(|token| token.parent())
    else {
        return Ok(Fact::unknown("The destination has no native scope token."));
    };
    let Some(scope) = sema.scope_at_offset(&node, destination.offset) else {
        return Ok(Fact::unknown(
            "The destination native scope is unavailable.",
        ));
    };
    let module = scope.module();
    let visible = item.is_visible_from(db, module);
    if !visible {
        return Ok(Fact {
            visibility: Some(false),
            importable: Some(false),
            path: None,
            limitations: vec![
                "Native HIR visibility excludes this item from the selected destination module."
                    .into(),
            ],
        });
    }
    let config = ra_ap_hir::FindPathConfig {
        prefer_no_std: false,
        prefer_prelude: false,
        prefer_absolute: false,
        allow_unstable: false,
    };
    let path = module
        .find_use_path(db, item, ra_ap_hir::PrefixKind::ByCrate, config)
        .map(|path| path.display(db, scope.krate().edition(db)).to_string());
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    Ok(Fact {visibility:Some(true),importable:path.as_ref().map(|_|true),path,limitations:vec![
        "This proves native item visibility and a native use-path from this module in this selected world; it does not execute or apply an import.".into(),
        "Compiler acceptance, item stability requirements, alias/name collisions and receiver/generic/trait applicability remain separate questions. Missing paths are unknown, not a proof of absence.".into(),
    ]})
}
