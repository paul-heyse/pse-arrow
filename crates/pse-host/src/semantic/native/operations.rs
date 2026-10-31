// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native operations retain the request's local Salsa handle, including cancellation.
use super::{Error, Result};
use native_ra_syntax::{AstNode as _, SyntaxKind};
use ra_ap_hir::{AsAssocItem as _, Semantics};
use ra_ap_ide::{FilePosition, NavigationTarget, TryToNav as _, UpmappingResult};
use ra_ap_ide_db::{
    RootDatabase,
    defs::{Definition, IdentClass},
    search::ReferenceCategory,
};
use std::collections::HashSet;
use tokio_util::sync::CancellationToken;

fn selected<'db>(
    sema: &Semantics<'db, RootDatabase>,
    position: FilePosition,
    token: &CancellationToken,
) -> Result<Vec<Definition<'db>>> {
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    let source = sema.parse_guess_edition(position.file_id);
    let original = ra_ap_ide_db::helpers::pick_best_token(
        source.syntax().token_at_offset(position.offset),
        |kind| usize::from(kind == SyntaxKind::IDENT) + 1,
    );
    let Some(original) = original else {
        return Ok(Vec::new());
    };
    let mut seen = HashSet::new();
    let mut definitions = Vec::new();
    for token in sema.descend_into_macros_exact(original) {
        if let Some(class) = IdentClass::classify_token(sema, &token) {
            for (definition, _) in class.definitions() {
                if seen.insert(definition) {
                    definitions.push(definition);
                }
            }
        }
    }
    Ok(definitions)
}

pub(super) fn definitions(
    db: &RootDatabase,
    position: FilePosition,
    token: &CancellationToken,
) -> Result<Vec<NavigationTarget>> {
    let sema = Semantics::new(db);
    let mut seen = HashSet::new();
    let mut results = Vec::new();
    for definition in selected(&sema, position, token)? {
        if let Some(targets) = definition.try_to_nav(&sema) {
            for target in targets {
                if seen.insert(target.clone()) {
                    results.push(target);
                }
            }
        }
    }
    Ok(results)
}

#[derive(Debug)]
pub(super) struct Reference {
    pub entity: usize,
    pub file: ra_ap_vfs::FileId,
    pub range: native_ra_syntax::TextRange,
    pub category: ReferenceCategory,
}
pub(super) fn references(
    db: &RootDatabase,
    position: FilePosition,
    token: &CancellationToken,
    limit: usize,
) -> Result<(Vec<Reference>, usize)> {
    let sema = Semantics::new(db);
    let mut results = Vec::new();
    let mut total = 0usize;
    for (entity, definition) in selected(&sema, position, token)?.into_iter().enumerate() {
        let mut seen = HashSet::new();
        definition
            .usages(&sema)
            .include_self_refs()
            .search(&mut |file, reference| {
                if token.is_cancelled() {
                    return true;
                }
                let file = file.file_id(db);
                if seen.insert((file, reference.range, reference.category.bits())) {
                    total += 1;
                    if results.len() < limit {
                        results.push(Reference {
                            entity,
                            file,
                            range: reference.range,
                            category: reference.category,
                        });
                    }
                }
                false
            });
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
    }
    Ok((results, total))
}

pub(super) fn implementations(
    db: &RootDatabase,
    position: FilePosition,
    token: &CancellationToken,
) -> Result<Vec<NavigationTarget>> {
    let sema = Semantics::new(db);
    let mut seen = HashSet::new();
    let mut targets = Vec::new();
    for definition in selected(&sema, position, token)? {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let mut implementations = Vec::new();
        let associated = match definition {
            Definition::Trait(item) => {
                implementations = ra_ap_hir::Impl::all_for_trait(db, item);
                None
            }
            Definition::Adt(item) => {
                implementations = ra_ap_hir::Impl::all_for_type(db, item.ty(db));
                None
            }
            Definition::TypeAlias(item) => {
                implementations = ra_ap_hir::Impl::all_for_type(db, item.ty(db));
                None
            }
            Definition::BuiltinType(item) => {
                implementations = ra_ap_hir::Impl::all_for_type(db, item.ty(db));
                None
            }
            Definition::Function(item) => item.as_assoc_item(db),
            Definition::Const(item) => item.as_assoc_item(db),
            _ => None,
        };
        if let Some(associated) = associated {
            if let (Some(owner), Some(name)) = (
                associated.container_or_implemented_trait(db),
                associated.name(db),
            ) {
                for implementation in ra_ap_hir::Impl::all_for_trait(db, owner) {
                    if token.is_cancelled() {
                        return Err(Error::Cancelled);
                    }
                    for item in implementation.items(db) {
                        // Membership of the native resolved trait implementation
                        // precedes its native item-name lookup; this is no global
                        // same-spelling declaration identity join.
                        if item.name(db).as_ref() == Some(&name) {
                            if let Some(navigation) = item.try_to_nav(&sema) {
                                for target in navigation {
                                    if seen.insert(target.clone()) {
                                        targets.push(target);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        for implementation in implementations {
            if token.is_cancelled() {
                return Err(Error::Cancelled);
            }
            if let Some(navigation) = implementation.try_to_nav(&sema) {
                for target in navigation {
                    if seen.insert(target.clone()) {
                        targets.push(target);
                    }
                }
            }
        }
    }
    Ok(targets)
}

pub(super) fn symbols(
    db: &RootDatabase,
    query: ra_ap_ide::Query,
    limit: usize,
) -> Vec<NavigationTarget> {
    // Native indexing parallelizes using temporary DB handles; as in the
    // library's IDE facade, attach only after those scoped children return.
    let symbols = ra_ap_ide_db::symbol_index::world_symbols(db, query);
    ra_ap_hir::attach_db(db, || {
        symbols
            .into_iter()
            .filter_map(|symbol| symbol.try_to_nav(&Semantics::new(db)))
            .take(limit)
            .map(UpmappingResult::call_site)
            .collect()
    })
}
