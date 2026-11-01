// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Demand-driven native callable candidates and same-crate extension-import obligations.
use super::*;
use ra_ap_hir::{AsAssocItem as _, HasVisibility as _};
mod contextual;

pub(super) fn inspect(
    sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
    expression: &ast::Expr,
    paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
    limit: usize,
) -> Vec<super::super::CallableCandidateFact> {
    let db = sema.db;
    let Some(scope) = sema.scope(expression.syntax()) else {
        return Vec::new();
    };
    let receiver = match expression {
        ast::Expr::MethodCallExpr(call) => call.receiver(),
        _ => Some(expression.clone()),
    };
    let Some(receiver) = receiver else {
        return Vec::new();
    };
    let Some(ty) = sema.type_of_expr(&receiver).map(|ty| ty.original) else {
        return Vec::new();
    };
    let selected_name = match expression {
        ast::Expr::MethodCallExpr(call) => call.name_ref().map(|name| name.text().to_string()),
        _ => None,
    };
    let target = scope.krate().to_display_target(db);
    let visible = scope.visible_traits().0;
    let mut candidates = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut add = |function: ra_ap_hir::Function, applicability, origin: &str| {
        if !seen.insert(function) {
            return;
        }
        if selected_name
            .as_ref()
            .is_some_and(|name| function.name(db).as_str() != name)
        {
            return;
        }
        if candidates.len() >= limit {
            return;
        }
        let trait_ = function
            .as_assoc_item(db)
            .and_then(|item| item.container_or_implemented_trait(db));
        let missing_import = trait_.is_some_and(|trait_| !visible.contains(&trait_.into()));
        let required_import = trait_
            .filter(|_| missing_import)
            .and_then(|trait_| {
                scope.module().find_use_path(
                    db,
                    ra_ap_hir::ModuleDef::Trait(trait_),
                    ra_ap_hir::PrefixKind::ByCrate,
                    ra_ap_hir::FindPathConfig {
                        prefer_no_std: false,
                        prefer_prelude: false,
                        prefer_absolute: false,
                        allow_unstable: false,
                    },
                )
            })
            .map(|path| path.display(db, scope.krate().edition(db)).to_string());
        let mapped = sema
            .source(function)
            .and_then(|source| sema.original_range_opt(source.value.syntax()));
        let (context, mut obligations) =
            contextual::call(sema, expression, function, paths, target);
        if context.is_none() {
            obligations.extend(contextual::declaration_bounds(
                sema, function, paths, target,
            ));
        }
        if let Some(trait_) = trait_ {
            obligations.push(contextual::obligation(sema, trait_, "Self".into(), Some(&ty), paths, target, "Native receiver trait implementation observation; imports remain a separate obligation."));
            if context.is_none() {
                obligations.extend(contextual::extension_bounds(
                    sema, trait_, &ty, paths, target,
                ));
            }
        }
        let mut limitations = vec!["Native type-argument projection does not expose lifetime/const substitutions or every associated-type predicate.".into()];
        if context.is_none() {
            limitations.push("No matching resolved authored call supplies instantiated parameters/result or receiver adjustments for this alternative; hypothetical-call substitution remains unavailable.".into());
        }
        candidates.push(super::super::CallableCandidateFact {
            target: TargetFact { relationship: "callable_candidate".into(), name: function.name(db).as_str().into(), location: mapped.and_then(|mapped|location(paths,mapped.file_id.file_id(db),mapped.range)), precision: if mapped.is_some(){"native_original_range"}else{"unavailable_or_non_unique_macro_origin"}.into() },
            signature: function.ty(db).display(db,target).to_string(),
            receiver: type_fact(db,&ty,target),
            applicability,
            required_import,
            context,
            obligations,
            limitations,
            reasons: vec![origin.into(), if missing_import { "Extension trait is absent from native visible-trait scope; importing it is a separate obligation." } else { "Candidate is in the selected native method scope." }.into(), "Declaration signature and actual contextual-call inference are separate observations; neither establishes rustc acceptance.".into()],
        });
    };
    ty.iterate_method_candidates(db, &scope, None, |function| {
        add(
            function,
            super::super::PredicateOutcome::Holds,
            "HIR method candidate enumeration for the actual receiver",
        );
        None::<()>
    });
    // Deliberately bounded to same-crate declarations. No global name cache or
    // spelling-based identity is introduced to discover a missing extension import.
    if selected_name.is_some() {
        for module in scope.krate().modules(db) {
            for declaration in module.declarations(db) {
                let ra_ap_hir::ModuleDef::Trait(trait_) = declaration else {
                    continue;
                };
                if !trait_.is_visible_from(db, scope.module()) {
                    continue;
                }
                let outcome = if ra_ap_hir::GenericDef::Trait(trait_)
                    .params(db)
                    .into_iter()
                    .any(|parameter| match parameter {
                        ra_ap_hir::GenericParam::TypeParam(parameter) => !parameter.is_implicit(db),
                        _ => true,
                    }) {
                    super::super::PredicateOutcome::Unsupported
                } else if ty.autoderef(db).any(|ty| ty.impls_trait(db, trait_, &[])) {
                    super::super::PredicateOutcome::Holds
                } else {
                    super::super::PredicateOutcome::NotProven
                };
                for item in trait_.items_with_supertraits(db) {
                    if let ra_ap_hir::AssocItem::Function(function) = item {
                        add(
                            function,
                            outcome,
                            "Same-crate extension trait declaration plus native receiver trait observation; failed proof remains NotProven",
                        );
                    }
                }
            }
        }
    }
    candidates
}
