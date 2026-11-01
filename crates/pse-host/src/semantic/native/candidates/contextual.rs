// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned projections of actual native call inference, never a hypothetical-call solver.
use super::*;
use crate::semantic::{CallableContextFact, CallableObligationFact, CallableSubstitutionFact};

fn function_target(
    sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
    function: ra_ap_hir::Function,
    paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
) -> TargetFact {
    let mapped = sema
        .source(function)
        .and_then(|source| sema.original_range_opt(source.value.syntax()));
    TargetFact {
        relationship: "contextual_callable".into(),
        name: function.name(sema.db).as_str().into(),
        location: mapped
            .and_then(|mapped| location(paths, mapped.file_id.file_id(sema.db), mapped.range)),
        precision: if mapped.is_some() {
            "native_original_range"
        } else {
            "unavailable_or_non_unique_macro_origin"
        }
        .into(),
    }
}

fn parameters(
    function: ra_ap_hir::Function,
    db: &ra_ap_ide_db::RootDatabase,
) -> Vec<ra_ap_hir::TypeParam> {
    let container = function
        .as_assoc_item(db)
        .map(|item| match item.container(db) {
            ra_ap_hir::AssocItemContainer::Impl(impl_) => ra_ap_hir::GenericDef::Impl(impl_),
            ra_ap_hir::AssocItemContainer::Trait(trait_) => ra_ap_hir::GenericDef::Trait(trait_),
        });
    container
        .into_iter()
        .chain([ra_ap_hir::GenericDef::Function(function)])
        .flat_map(|owner| owner.params(db))
        .filter_map(|parameter| match parameter {
            ra_ap_hir::GenericParam::TypeParam(parameter) => Some(parameter),
            _ => None,
        })
        .collect()
}

pub(super) fn call(
    sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
    expression: &ast::Expr,
    candidate: ra_ap_hir::Function,
    paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
    target: ra_ap_hir::DisplayTarget,
) -> (Option<CallableContextFact>, Vec<CallableObligationFact>) {
    let ast::Expr::MethodCallExpr(call) = expression else {
        return (None, Vec::new());
    };
    let Some(callable) = sema.resolve_method_call_as_callable(call) else {
        return (None, Vec::new());
    };
    let Some(callable_function) = callable.as_function() else {
        return (None, Vec::new());
    };
    let resolved = sema
        .resolve_method_call_fallback(call)
        .and_then(|(resolved, substitution)| {
            resolved.left().map(|function| (function, substitution))
        });
    let resolved_function = resolved
        .as_ref()
        .map(|(function, _)| *function)
        .unwrap_or(callable_function);
    if candidate != callable_function && candidate != resolved_function {
        return (None, Vec::new());
    }
    let db = sema.db;
    let params = parameters(resolved_function, db);
    let types = resolved
        .as_ref()
        .and_then(|(_, substitution)| substitution.as_ref())
        .map(|substitution| substitution.all_types(db));
    let substitutions = types.as_ref().map(|types| {
        types
            .iter()
            .enumerate()
            .map(|(index, ty)| CallableSubstitutionFact {
                parameter_index: index,
                parameter: params
                    .get(index)
                    .map(|parameter| parameter.name(db).as_str().into()),
                argument: type_fact(db, ty, target),
            })
            .collect()
    });
    let mut obligations = Vec::new();
    for (index, parameter) in params.iter().enumerate() {
        for trait_ in parameter.trait_bounds(db) {
            obligations.push(obligation(sema,trait_,parameter.name(db).as_str().into(),types.as_ref().and_then(|types|types.get(index)),paths,target,"Declared native type-parameter bound, observed on its actual inferred type argument. Additional method predicates/associated constraints may remain unavailable."));
        }
    }
    let result = callable.return_type();
    let adjusted = sema.type_of_expr(expression).and_then(|info| info.adjusted);
    let coercion = adjusted
        .as_ref()
        .filter(|adjusted| !adjusted.contains_unknown() && !result.contains_unknown())
        .map(|adjusted| result.could_coerce_to(db, adjusted));
    let mapped = sema.original_range_opt(call.syntax());
    (
        Some(CallableContextFact {
            callsite: mapped
                .and_then(|mapped| location(paths, mapped.file_id.file_id(db), mapped.range)),
            callable_target: function_target(sema, callable_function, paths),
            substitution_owner: function_target(sema, resolved_function, paths),
            receiver_parameter: callable
                .receiver_param(db)
                .map(|(_, ty)| type_fact(db, &ty, target)),
            parameters: callable
                .params()
                .iter()
                .map(|parameter| type_fact(db, parameter.ty(), target))
                .collect(),
            result: type_fact(db, &result, target),
            adjusted_result: adjusted.as_ref().map(|ty| type_fact(db, ty, target)),
            result_coerces_to_adjusted: coercion,
            substitutions,
            receiver_adjustments: call
                .receiver()
                .and_then(|receiver| sema.expr_adjustments(&receiver))
                .map(|adjustments| {
                    adjustments
                        .into_iter()
                        .map(|adjustment| crate::semantic::AdjustmentFact {
                            kind: format!("{:?}", adjustment.kind),
                            source: type_fact(db, &adjustment.source, target),
                            target: type_fact(db, &adjustment.target, target),
                        })
                        .collect()
                }),
        }),
        obligations,
    )
}

pub(super) fn obligation(
    sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
    trait_: ra_ap_hir::Trait,
    parameter: String,
    subject: Option<&ra_ap_hir::Type<'_>>,
    paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
    target: ra_ap_hir::DisplayTarget,
    reason: &str,
) -> CallableObligationFact {
    let db = sema.db;
    let supported = !ra_ap_hir::GenericDef::Trait(trait_)
        .params(db)
        .into_iter()
        .any(|parameter| match parameter {
            ra_ap_hir::GenericParam::TypeParam(parameter) => !parameter.is_implicit(db),
            _ => true,
        });
    let outcome = match subject {
        Some(subject) if supported && !subject.contains_unknown() => {
            if subject.impls_trait(db, trait_, &[]) {
                crate::semantic::PredicateOutcome::Holds
            } else {
                crate::semantic::PredicateOutcome::NotProven
            }
        }
        _ => crate::semantic::PredicateOutcome::Unsupported,
    };
    let mapped = sema
        .source(trait_)
        .and_then(|source| sema.original_range_opt(source.value.syntax()));
    CallableObligationFact {
        declared_trait: TargetFact {
            relationship: "declared_trait_obligation".into(),
            name: trait_.name(db).as_str().into(),
            location: mapped
                .and_then(|mapped| location(paths, mapped.file_id.file_id(db), mapped.range)),
            precision: if mapped.is_some() {
                "native_original_range"
            } else {
                "unavailable_or_non_unique_macro_origin"
            }
            .into(),
        },
        parameter,
        subject: subject.map(|subject| type_fact(db, subject, target)),
        outcome,
        reason: format!(
            "{reason} Generic trait arguments, lifetimes and missing subjects are Unsupported; a failed supported proof is NotProven."
        ),
    }
}

pub(super) fn extension_bounds(
    sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
    trait_: ra_ap_hir::Trait,
    receiver: &ra_ap_hir::Type<'_>,
    paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
    target: ra_ap_hir::DisplayTarget,
) -> Vec<CallableObligationFact> {
    let db = sema.db;
    let mut facts = Vec::new();
    for impl_ in ra_ap_hir::Impl::all_for_trait(db, trait_) {
        if impl_.module(db).krate(db) != trait_.module(db).krate(db) {
            continue;
        }
        let self_ty = impl_.self_ty(db);
        // This public native binding is exact for a blanket impl whose whole Self is T.
        // Structured generic matching is left to inference, never reconstructed from names.
        let Some(self_parameter) = self_ty.as_type_param(db) else {
            continue;
        };
        for parameter in ra_ap_hir::GenericDef::Impl(impl_).params(db) {
            let ra_ap_hir::GenericParam::TypeParam(parameter) = parameter else {
                continue;
            };
            for bound in parameter.trait_bounds(db) {
                facts.push(obligation(sema,bound,parameter.name(db).as_str().into(),(parameter==self_parameter).then_some(receiver),paths,target,"Native whole-Self blanket implementation parameter bound. The actual receiver supplies this native parameter identity; no structural generic matching is inferred."));
            }
        }
    }
    facts
}

pub(super) fn declaration_bounds(
    sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
    function: ra_ap_hir::Function,
    paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
    target: ra_ap_hir::DisplayTarget,
) -> Vec<CallableObligationFact> {
    parameters(function,sema.db).into_iter().flat_map(|parameter|parameter.trait_bounds(sema.db).into_iter().map(move |trait_|obligation(sema,trait_,parameter.name(sema.db).as_str().into(),None,paths,target,"Declared native parameter bound only; no matching authored call supplies its contextual type argument."))).collect()
}
