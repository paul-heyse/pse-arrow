// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native body, comparison and origin observations in their original snapshot.
use super::*;
use crate::semantic::{
    CompatibilityObservation, CompatibilityOperation, ImportTarget, NativeBodyObservation,
    NativeInvestigationFacts, OriginKind, OriginObservation, OriginTargetKind,
};
use ra_ap_hir::HasVisibility;
pub(super) fn binary_context<'a>(
    targets: &'a [crate::semantic::RunnableTargetFact],
    root: Option<&PathBuf>,
    features: &BTreeSet<String>,
) -> Option<&'a crate::semantic::RunnableTargetFact> {
    let mut matching = targets.iter().filter(|target| {
        target.kind == "binary"
            && Some(&target.root) == root
            && &target.active_features == features
            && target.required_features_satisfied
            && target
                .required_features
                .iter()
                .all(|feature| features.contains(feature))
    });
    let selected = matching.next()?;
    matching.next().is_none().then_some(selected)
}
fn definition_source<D: ra_ap_hir::HasSource>(
    sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
    definition: D,
    paths: &BTreeMap<ra_ap_ide::FileId, PathBuf>,
) -> Option<NativeLocation>
where
    D::Ast: native_ra_syntax::AstNode,
{
    let source = sema.source(definition)?;
    mapped(sema, source.value.syntax(), paths)
}
fn definition_location(
    sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
    definition: ra_ap_hir::ModuleDef,
    paths: &BTreeMap<ra_ap_ide::FileId, PathBuf>,
) -> Option<NativeLocation> {
    use ra_ap_hir::ModuleDef;
    match definition {
        ModuleDef::Module(m) => m
            .declaration_source(sema.db)
            .and_then(|src| mapped(sema, src.value.syntax(), paths)),
        ModuleDef::Function(v) => definition_source(sema, v, paths),
        ModuleDef::Adt(v) => definition_source(sema, v, paths),
        ModuleDef::EnumVariant(v) => definition_source(sema, v, paths),
        ModuleDef::Const(v) => definition_source(sema, v, paths),
        ModuleDef::Static(v) => definition_source(sema, v, paths),
        ModuleDef::Trait(v) => definition_source(sema, v, paths),
        ModuleDef::TypeAlias(v) => definition_source(sema, v, paths),
        ModuleDef::Macro(v) => definition_source(sema, v, paths),
        ModuleDef::BuiltinType(_) => None,
    }
}
fn mapped(
    sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
    node: &native_ra_syntax::SyntaxNode,
    paths: &BTreeMap<ra_ap_ide::FileId, PathBuf>,
) -> Option<NativeLocation> {
    let r = sema.original_range_opt(node)?;
    location(paths, r.file_id.file_id(sema.db), r.range)
}
fn node_at<'db>(
    snapshot: &QuerySnapshot<'db>,
    sema: &Semantics<'db, ra_ap_ide_db::RootDatabase>,
    position: ra_ap_ide::FilePosition,
) -> Result<native_ra_syntax::SyntaxNode> {
    let parsed = sema.parse(ra_ap_hir::EditionedFileId::current_edition(
        snapshot.db,
        position.file_id,
    ));
    parsed
        .syntax()
        .token_at_offset(position.offset)
        .right_biased()
        .and_then(|t| t.parent())
        .ok_or_else(|| Error::Unavailable("no native source at selected operand".into()))
}
fn operand<'db>(
    s: &QuerySnapshot<'db>,
    sema: &Semantics<'db, ra_ap_ide_db::RootDatabase>,
    p: ra_ap_ide::FilePosition,
) -> Result<(ra_ap_hir::Type<'db>, NativeLocation)> {
    let node = node_at(s, sema, p)?;
    for n in node.ancestors() {
        if let Some(expr) = ast::Expr::cast(n.clone()) {
            if let Some(t) = sema.type_of_expr(&expr) {
                return Ok((
                    t.original,
                    mapped(sema, expr.syntax(), s.paths).ok_or_else(|| {
                        Error::Unavailable("type operand has no unique original mapping".into())
                    })?,
                ));
            }
        }
        if let Some(ty) = ast::Type::cast(n.clone()) {
            if let Some(t) = sema.resolve_type(&ty) {
                return Ok((
                    t,
                    mapped(sema, ty.syntax(), s.paths).ok_or_else(|| {
                        Error::Unavailable("type operand has no unique original mapping".into())
                    })?,
                ));
            }
        }
        if let Some(adt) = ast::Adt::cast(n.clone()) {
            if let Some(a) = sema.to_adt_def(&adt) {
                return Ok((
                    a.ty(s.db),
                    mapped(sema, adt.syntax(), s.paths).ok_or_else(|| {
                        Error::Unavailable("type declaration has no original mapping".into())
                    })?,
                ));
            }
        }
    }
    Err(Error::Unavailable(
        "selected operand has no established native type".into(),
    ))
}
fn against(
    s: &QuerySnapshot<'_>,
    target: &ImportTarget,
    generation: Option<u64>,
) -> Result<ra_ap_ide::FilePosition> {
    if generation.is_some_and(|g| g != SNAPSHOT_GENERATION) {
        return Err(Error::Stale);
    }
    let path = target.file.canonicalize()?;
    let id = *s.files.get(&path).ok_or_else(|| {
        Error::Unavailable("comparison destination is outside the prepared world".into())
    })?;
    if s.digests.get(&path) != Some(&target.source_sha256) {
        return Err(Error::Stale);
    }
    let text = s.db.file_text(id);
    if target.byte_start > target.byte_end
        || text
            .text(s.db)
            .get(target.byte_start..target.byte_end)
            .is_none()
    {
        return Err(Error::Stale);
    }
    let offset=crate::source_read::inquiry_anchor(text.text(s.db),target.byte_start..target.byte_end).ok_or_else(||Error::InvalidRequest("native operand requires one exact expression, type, declaration, string token, or explicit byte position".into()))?;
    Ok(ra_ap_ide::FilePosition {
        file_id: id,
        offset: native_ra_syntax::TextSize::from(u32::try_from(offset).map_err(|_| Error::Stale)?),
    })
}
pub(super) fn compatibility(
    s: &QuerySnapshot<'_>,
    left: ra_ap_ide::FilePosition,
    target: &ImportTarget,
    generation: Option<u64>,
    operation: CompatibilityOperation,
) -> Result<NativeInvestigationFacts> {
    let sema = Semantics::new(s.db);
    let right = against(s, target, generation)?;
    let (lhs, left) = operand(s, &sema, left)?;
    let (rhs, right) = operand(s, &sema, right)?;
    let module = sema
        .file_to_module_defs(s.files[&left.path])
        .next()
        .ok_or_else(|| Error::Unavailable("comparison operand is inactive".into()))?;
    let display = module.krate(s.db).to_display_target(s.db);
    let mut limitations = Vec::new();
    let unknown = lhs.contains_unknown() || rhs.contains_unknown();
    let rebased = lhs
        .try_rebase_into(s.db, &rhs)
        .map(|l| (l, rhs.clone()))
        .or_else(|| rhs.try_rebase_into(s.db, &lhs).map(|r| (lhs.clone(), r)));
    let compatible_owner = rebased.is_some();
    let result = if unknown {
        limitations.push("unresolved components cannot establish comparison truth; native could APIs may replace them with inference variables".into());
        None
    } else if let Some((l, r)) = rebased {
        Some(match operation {
            CompatibilityOperation::Equal => l == r,
            CompatibilityOperation::CouldUnify => l.could_unify_with_deeply(s.db, &r),
            CompatibilityOperation::CouldCoerce => l.could_coerce_to(s.db, &r),
        })
    } else {
        limitations
            .push("native generic owners cannot be fallibly rebased into a common context".into());
        None
    };
    Ok(NativeInvestigationFacts {
        compatibility: Some(CompatibilityObservation {
            operation,
            left,
            right,
            left_type: lhs.display(s.db, display).to_string(),
            right_type: rhs.display(s.db, display).to_string(),
            result,
            compatible_owner,
            limitations,
        }),
        ..Default::default()
    })
}
pub(super) fn body(
    sema: &Semantics<'_, ra_ap_ide_db::RootDatabase>,
    node: &native_ra_syntax::SyntaxNode,
    paths: &BTreeMap<ra_ap_ide::FileId, PathBuf>,
) -> NativeInvestigationFacts {
    let selected = node.ancestors().find(|n| {
        ast::ClosureExpr::can_cast(n.kind())
            || ast::Fn::can_cast(n.kind())
            || ast::BlockExpr::cast(n.clone()).is_some_and(|b| b.async_token().is_some())
    });
    let missing = selected.is_none();
    let body = selected.map(|n| {
        let location = mapped(sema, &n, paths);
        let owner = n
            .ancestors()
            .skip(1)
            .find_map(ast::Fn::cast)
            .and_then(|f| mapped(sema, f.syntax(), paths));
        NativeBodyObservation {
            package_id: None,
            role: None,
            kind: if ast::ClosureExpr::can_cast(n.kind()) {
                "closure"
            } else if ast::BlockExpr::can_cast(n.kind()) {
                "async_block"
            } else {
                "function"
            }
            .into(),
            limitations: if location.is_none() {
                vec!["selected body has no unique authored source mapping".into()]
            } else {
                Vec::new()
            },
            location,
            owner,
        }
    });
    NativeInvestigationFacts {
        body,
        limitations: if missing {
            vec!["selection has no enclosing supported body".into()]
        } else {
            Vec::new()
        },
        ..Default::default()
    }
}
pub(super) fn visibility(
    s: &QuerySnapshot<'_>,
    p: ra_ap_ide::FilePosition,
    destination: &ImportTarget,
    generation: Option<u64>,
) -> Result<NativeInvestigationFacts> {
    let sema = Semantics::new(s.db);
    let n = node_at(s, &sema, p)?;
    let dest = node_at(s, &sema, against(s, destination, generation)?)?;
    let module = sema
        .scope(&dest)
        .ok_or_else(|| Error::Unavailable("destination has no native module".into()))?
        .module();
    let mut definition = None;
    for n in n.ancestors() {
        let field = if let Some(v) = ast::FieldExpr::cast(n.clone()) {
            sema.resolve_field(&v).and_then(|f| f.left())
        } else if let Some(v) = ast::RecordField::cast(n.clone()) {
            sema.to_def(&v)
        } else if let Some(v) = ast::TupleField::cast(n.clone()) {
            sema.to_def(&v)
        } else {
            None
        };
        if let Some(field) = field {
            return Ok(NativeInvestigationFacts {
                visible: Some(field.is_visible_from(s.db, module)),
                ..Default::default()
            });
        }
        if let Some(path) = ast::Path::cast(n.clone()) {
            match sema.resolve_path(&path) {
                Some(ra_ap_hir::PathResolution::Def(d)) => {
                    definition = Some(d);
                    break;
                }
                Some(_) => break,
                None => {}
            }
        }
        if let Some(f) = ast::Fn::cast(n.clone()) {
            definition = sema.to_def(&f).map(ra_ap_hir::ModuleDef::Function);
            break;
        }
        if let Some(a) = ast::Adt::cast(n.clone()) {
            definition = sema.to_adt_def(&a).map(ra_ap_hir::ModuleDef::Adt);
            break;
        }
        if let Some(a) = ast::Trait::cast(n.clone()) {
            definition = sema.to_def(&a).map(ra_ap_hir::ModuleDef::Trait);
            break;
        }
        if let Some(a) = ast::TypeAlias::cast(n.clone()) {
            definition = sema.to_def(&a).map(ra_ap_hir::ModuleDef::TypeAlias);
            break;
        }
        if let Some(a) = ast::Const::cast(n.clone()) {
            definition = sema.to_def(&a).map(ra_ap_hir::ModuleDef::Const);
            break;
        }
        if let Some(a) = ast::Static::cast(n.clone()) {
            definition = sema.to_def(&a).map(ra_ap_hir::ModuleDef::Static);
            break;
        }
        if let Some(a) = ast::Module::cast(n) {
            definition = sema.to_def(&a).map(ra_ap_hir::ModuleDef::Module);
            break;
        }
    }
    Ok(NativeInvestigationFacts {
        visible: definition.map(|d| d.is_visible_from(s.db, module)),
        limitations: if definition.is_none() {
            vec!["selected declaration visibility is not established".into()]
        } else {
            Vec::new()
        },
        ..Default::default()
    })
}
pub(super) fn origins(
    s: &QuerySnapshot<'_>,
    p: ra_ap_ide::FilePosition,
    kind: OriginKind,
    limit: usize,
    token: &CancellationToken,
) -> Result<NativeInvestigationFacts> {
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    if limit == 0 {
        return Err(Error::InvalidRequest(
            "origin observation limit must be positive".into(),
        ));
    }
    let sema = Semantics::new(s.db);
    let node = node_at(s, &sema, p)?;
    let mut facts = NativeInvestigationFacts::default();
    match kind {
        OriginKind::Format => {
            use native_ra_syntax::AstToken as _;
            let parsed = sema.parse(ra_ap_hir::EditionedFileId::current_edition(s.db, p.file_id));
            let string = parsed
                .syntax()
                .token_at_offset(p.offset)
                .right_biased()
                .and_then(ast::String::cast)
                .ok_or_else(|| {
                    Error::Unavailable("select an original format string token".into())
                })?;
            let parts=sema.as_format_args_parts(&string).ok_or_else(||Error::Unavailable("native owner cannot resolve this format or assembly template; required expansion may be unprepared".into()))?;
            if parts.len() > limit {
                facts
                    .limitations
                    .push("format origin observations truncated by requested limit".into());
            }
            for (range, target) in parts
                .into_iter()
                .take(limit)
                .enumerate()
                .map(|(i, (r, t))| ((i, r), t))
            {
                let (ordinal, range) = range;
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                let occurrence = location(s.paths, p.file_id, range);
                let mut o = OriginObservation {
                    ordinal,
                    target_kind: OriginTargetKind::Unresolved,
                    occurrence,
                    target: None,
                    name: None,
                    originating_adt: None,
                    limitations: Vec::new(),
                };
                if let Some(target) = target {
                    if let Some(resolution) = target.as_ref().left() {
                        match resolution {
                            ra_ap_hir::PathResolution::Local(local) => {
                                o.target_kind = OriginTargetKind::Local;
                                o.name = Some(
                                    local
                                        .name(s.db)
                                        .display(s.db, native_ra_syntax::Edition::CURRENT)
                                        .to_string(),
                                );
                                let locations = local
                                    .sources(s.db)
                                    .into_iter()
                                    .filter_map(|src| mapped(&sema, src.syntax(), s.paths))
                                    .collect::<Vec<_>>();
                                if locations.len() == 1 {
                                    o.target = locations.into_iter().next();
                                } else {
                                    o.limitations.push("local has plural or unavailable original declaration mapping".into());
                                }
                            }
                            ra_ap_hir::PathResolution::Def(d) => {
                                o.target_kind = OriginTargetKind::Declaration;
                                o.target = definition_location(&sema, *d, s.paths);
                                if o.target.is_none() {
                                    o.limitations.push(
                                        "resolved declaration has no unique original mapping"
                                            .into(),
                                    );
                                }
                                o.name = d.name(s.db).map(|n| {
                                    n.display(s.db, native_ra_syntax::Edition::CURRENT)
                                        .to_string()
                                });
                            }
                            _ => o
                                .limitations
                                .push("format target is not a unique declaration/local".into()),
                        }
                    } else if let Some(asm) = target.right() {
                        o.target_kind = OriginTargetKind::AssemblyOperand;
                        o.name = asm.name(s.db).map(|n| {
                            n.display(s.db, native_ra_syntax::Edition::CURRENT)
                                .to_string()
                        });
                        o.limitations.push("assembly operand belongs to this invocation; pinned native API does not export its ordinal".into());
                    }
                } else {
                    o.limitations
                        .push("format part has no resolved target".into());
                }
                facts.origins.push(o);
            }
        }
        OriginKind::Derive => {
            let adt = node.ancestors().find_map(ast::Adt::cast).ok_or_else(|| {
                Error::Unavailable("select an originating ADT or its derive attribute".into())
            })?;
            let native = sema
                .to_adt_def(&adt)
                .ok_or_else(|| Error::Unavailable("derive ADT is unresolved".into()))?;
            let origin = mapped(&sema, adt.syntax(), s.paths);
            facts.origins.push(OriginObservation {
                ordinal: 0,
                target_kind: OriginTargetKind::OriginatingAdt,
                occurrence: origin.clone(),
                target: origin.clone(),
                name: None,
                originating_adt: origin.clone(),
                limitations: Vec::new(),
            });
            if facts.origins.len() >= limit {
                facts.limitations.push("derive origin observation limit reached; remaining impl/helper populations were not examined".into());
                return Ok(facts);
            }
            for impl_ in ra_ap_hir::Impl::all_for_type(s.db, native.ty(s.db)) {
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                if facts.origins.len() >= limit {
                    facts.limitations.push("derive origin observation limit reached; remaining impl/helper populations were not examined".into());
                    return Ok(facts);
                }
                if sema.impl_generated_from_derive(impl_) != Some(native) {
                    continue;
                }
                let name = impl_.trait_(s.db).map(|t| {
                    t.name(s.db)
                        .display(s.db, native_ra_syntax::Edition::CURRENT)
                        .to_string()
                });
                let ordinal = facts.origins.len();
                facts.origins.push(OriginObservation{ordinal,target_kind:OriginTargetKind::GeneratedImpl,occurrence:None,target:None,name,originating_adt:origin.clone(),limitations:vec!["originating ADT is not exact inverse macro history or generated implementation identity".into()]});
            }
            for attr in adt.syntax().children().filter_map(ast::Attr::cast) {
                if token.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                if facts.origins.len() >= limit {
                    facts.limitations.push("derive origin observation limit reached; remaining helper populations were not examined".into());
                    return Ok(facts);
                }
                if let Some(helpers) = sema.derive_helper(&attr) {
                    for (macro_, _) in helpers {
                        if token.is_cancelled() {
                            return Err(Error::Cancelled);
                        }
                        if facts.origins.len() >= limit {
                            facts.limitations.push("derive origin observation limit reached; remaining helper nominations were not examined".into());
                            return Ok(facts);
                        }
                        facts.origins.push(OriginObservation {
                            ordinal: facts.origins.len(),
                            target_kind: OriginTargetKind::DeriveHelper,
                            occurrence: mapped(&sema, attr.syntax(), s.paths),
                            target: None,
                            name: Some(
                                macro_
                                    .name(s.db)
                                    .display(s.db, native_ra_syntax::Edition::CURRENT)
                                    .to_string(),
                            ),
                            originating_adt: origin.clone(),
                            limitations: vec![
                                "helper nomination is not exact macro expansion history".into(),
                            ],
                        });
                    }
                }
            }
            facts.limitations.push("generated impl enumeration is native selected-type evidence; missing mappings and blanket applicability remain unknown".into());
        }
    }
    Ok(facts)
}
