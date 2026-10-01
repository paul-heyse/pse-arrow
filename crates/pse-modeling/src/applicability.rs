// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed declaration admission for scientific evidence, separate from hard domains.
use crate::{
    CheckedPackage, Declaration, DeclarationId, Function, Result, Type, TypeContext, invalid,
};
use pse_authoring::dsl::{self, Expr, Predicate};
use pse_model::applicability::Node;
use pse_model::generated::enums::ModelingApplicabilityKind as Kind;
use std::collections::{BTreeMap, BTreeSet};

/// Actual bound applicability application retained in a finite function.
#[derive(Clone, Debug, PartialEq)]
pub struct Use {
    /// Immutable identity, declared unions and required dependencies.
    pub node: Node,
    /// Predicates evaluated by the mathematical library at actual arguments.
    pub predicates: Vec<Predicate>,
    /// Canonical physical values captured on the same active execution path.
    pub inputs: Vec<Expr>,
}
impl Use {
    /// Frame the complete evidence product in function and dispatch identities.
    pub fn frame(&self, h: &mut pse_ids::FramedHasher) {
        self.node.frame(h);
        h.u64(self.predicates.len() as u64);
        for p in &self.predicates {
            h.str(&dsl::render_predicate(p));
        }
        h.u64(self.inputs.len() as u64);
        for e in &self.inputs {
            h.str(&dsl::render_expr(e));
        }
    }
}
/// Claim signatures are ordinary typed callable references with a nominal evidence output.
pub(crate) fn signature(
    p: &mut CheckedPackage,
    c: &TypeContext<'_>,
    row: &Declaration,
    names: &BTreeMap<String, Type>,
) -> Result<()> {
    let v = row
        .value
        .applicability
        .as_ref()
        .ok_or_else(|| invalid(row.declaration_id, "claim payload absent"))?;
    let id = row.declaration_id;
    let mut seen = BTreeSet::new();
    let arguments = v
        .arguments
        .iter()
        .map(|a| {
            if !seen.insert(a.name.clone()) || a.default_value.is_some() {
                return Err(invalid(
                    id,
                    "claim arguments are distinct, explicit and default-free",
                ));
            }
            Ok((
                a.name.clone(),
                c.resolve(&a.r#type, &BTreeSet::new(), names, id)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    p.types.insert(
        id,
        Type::Function {
            arguments: arguments.clone(),
            result: Box::new(Type::Applicability),
        },
    );
    p.functions.insert(
        id,
        Function {
            applicability: Vec::new(),
            applicability_uses: Vec::new(),
            physical_admissions: BTreeMap::new(),
            physical_operation: None,
            reduction: None,
            validity: None,
            envelopes: Vec::new(),
            validity_reads: Default::default(),
            external: None,
            continuity: None,
            id,
            variables: BTreeSet::new(),
            arguments,
            result: Type::Applicability,
            body: None,
        },
    );
    Ok(())
}
/// Admission never turns a missing claim or an arbitrary predicate into unrestricted evidence.
pub(crate) fn admit(p: &CheckedPackage, c: &TypeContext<'_>) -> Result<()> {
    for row in p.declarations.values() {
        let id = row.declaration_id;
        if let Some(v) = &row.value.applicability {
            let function = &p.functions[&id];
            let env = function.arguments.iter().cloned().collect();
            let owner = p
                .resolve(id, &v.owner)
                .ok_or_else(|| invalid(id, "applicability owner is absent"))?;
            if !matches!(
                p.types.get(&owner),
                Some(
                    Type::Entity(_)
                        | Type::Definition(_)
                        | Type::Interface(_)
                        | Type::Row(_)
                        | Type::Function { .. }
                )
            ) && !p.tables.contains_key(&owner)
            {
                return Err(invalid(
                    id,
                    "applicability owner is a declared scientific family or model",
                ));
            }
            let evidence = dsl::parse_expr(&v.evidence).map_err(|e| invalid(id, e.to_string()))?;
            let evidence_type = crate::expression::infer(&evidence, &env, p, c, id, None)?;
            if !matches!(evidence_type,Type::Entity(kind) if p.kinds.get(&kind).is_some_and(|k|k.provenance))
            {
                return Err(invalid(
                    id,
                    "applicability evidence names a typed provenance source",
                ));
            }
            match v.claim_kind {
                Kind::Region
                    if v.basis.is_some()
                        && v.predicate.is_some()
                        && v.axis.is_none()
                        && v.reason.is_none()
                        && v.alternatives.is_empty() =>
                {
                    let predicate =
                        dsl::parse_predicate(v.predicate.as_deref().unwrap_or_default())
                            .map_err(|e| invalid(id, e.to_string()))?;
                    crate::expression::predicate(&predicate, &env, p, c, id)?;
                }
                Kind::Interval
                    if v.basis.is_some()
                        && v.axis.is_some()
                        && v.lower.is_some()
                        && v.upper.is_some()
                        && v.predicate.is_none()
                        && v.reason.is_none()
                        && v.alternatives.is_empty() =>
                {
                    let predicate = interval_predicate(
                        v.axis.as_deref().unwrap_or_default(),
                        v.lower.as_deref().unwrap_or_default(),
                        v.upper.as_deref().unwrap_or_default(),
                        id,
                    )?;
                    crate::expression::predicate(&predicate, &env, p, c, id)?;
                }
                Kind::Unknown
                    if v.reason.as_ref().is_some_and(|s| !s.trim().is_empty())
                        && v.basis.is_none()
                        && v.predicate.is_none()
                        && v.axis.is_none()
                        && v.alternatives.is_empty() => {}
                Kind::Unrestricted
                    if v.basis.is_none()
                        && v.predicate.is_none()
                        && v.reason.is_none()
                        && v.axis.is_none()
                        && v.alternatives.is_empty() => {}
                Kind::Union
                    if !v.alternatives.is_empty()
                        && v.basis.is_none()
                        && v.predicate.is_none()
                        && v.reason.is_none()
                        && v.axis.is_none() => {}
                _ => {
                    return Err(invalid(
                        id,
                        "claim payload does not match its declared evidence kind",
                    ));
                }
            }
            for application in v.alternatives.iter().chain(&v.dependencies) {
                let e = dsl::parse_expr(application).map_err(|e| invalid(id, e.to_string()))?;
                if crate::expression::infer(&e, &env, p, c, id, Some(&Type::Applicability))?
                    != Type::Applicability
                {
                    return Err(invalid(
                        id,
                        "claim unions and dependencies require typed applicability applications",
                    ));
                }
            }
        }
        if let Some(v) = &row.value.permission {
            if v.targets.is_empty() {
                return Err(invalid(
                    id,
                    "permission targets at least one named record or declared family",
                ));
            }
            let mut parent = row.parent_id;
            while parent.is_some_and(|at| {
                matches!(
                    p.declarations[&at].value.kind,
                    pse_model::generated::enums::ModelingDeclarationKind::When
                        | pse_model::generated::enums::ModelingDeclarationKind::Stage
                )
            }) {
                parent = parent.and_then(|at| p.declarations[&at].parent_id);
            }
            let owner = parent
                .and_then(|id| p.declarations.get(&id))
                .ok_or_else(|| invalid(id, "permission belongs to a consuming scope"))?;
            if !matches!(
                owner.value.kind,
                pse_model::generated::enums::ModelingDeclarationKind::Definition
                    | pse_model::generated::enums::ModelingDeclarationKind::Test
                    | pse_model::generated::enums::ModelingDeclarationKind::Case
                    | pse_model::generated::enums::ModelingDeclarationKind::Preset
                    | pse_model::generated::enums::ModelingDeclarationKind::Interface
            ) {
                return Err(invalid(
                    id,
                    "permission belongs to a consuming definition, interface, preset, test or case",
                ));
            }
        }
    }
    for f in p.functions.values() {
        let env = f.arguments.iter().cloned().collect();
        for application in &f.applicability {
            if crate::expression::infer(application, &env, p, c, f.id, Some(&Type::Applicability))?
                != Type::Applicability
            {
                return Err(invalid(
                    f.id,
                    "form applicability binds a typed evidence claim",
                ));
            }
        }
    }
    Ok(())
}
pub(crate) fn interval_predicate(
    axis: &str,
    lower: &str,
    upper: &str,
    at: DeclarationId,
) -> Result<Predicate> {
    dsl::parse_predicate(&format!("({axis}) >= ({lower}) and ({axis}) <= ({upper})"))
        .map_err(|e| invalid(at, e.to_string()))
}
