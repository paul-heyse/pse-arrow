// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed model hints are collected for their declared downstream consumer.
use crate::specialize::{Engine, Environment, Lineage};
use crate::{DeclarationId, InstanceId, Result, Type, invalid};
use pse_authoring::{
    dsl::{self, Expr, ExprKind, Predicate},
    language::{StaticValue, parse_static},
};
use pse_ids::SemanticId;
/// Orientation of the single selected analysis objective.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectiveSense {
    /// Minimize the original physical value.
    Minimize,
    /// Maximize the original physical value.
    Maximize,
}
pub(crate) fn objective_sense(source: &str, at: DeclarationId) -> Result<ObjectiveSense> {
    match label(source, at)?.as_str() {
        "minimize" => Ok(ObjectiveSense::Minimize),
        "maximize" => Ok(ObjectiveSense::Maximize),
        _ => Err(invalid(at, "objective sense must be minimize or maximize")),
    }
}
/// The declared extrapolation policy of an `annotation valid` range (ADR-0115 Outcome 3).
pub(crate) fn extrapolation_policy(
    source: &str,
    at: DeclarationId,
) -> Result<pse_model::generated::enums::ExtrapolationPolicy> {
    label(source, at)?.parse().map_err(|e| {
        invalid(
            at,
            format!("validity policy must be reject or explicitly selected extrapolate: {e}"),
        )
    })
}
/// Checked annotation meaning. A start never fixes a variable and a bound is not a start.
#[derive(Clone, Debug, PartialEq)]
pub enum AnnotationValue {
    /// Physical start expression for initialization.
    Start(Expr),
    /// Physical characteristic magnitude for numerical policy.
    Nominal(Expr),
    /// Physical lower and upper bound expressions.
    Bounds(Expr, Expr),
    /// Declared generic scaling scheme name.
    Scale(pse_model::generated::enums::ConstraintScalingScheme),
    /// Select this existing scalar member as the analysis objective.
    Objective(ObjectiveSense),
    /// Output label.
    Report(String),
    /// Physical validity range and explicit extrapolation policy.
    Valid {
        lower: Expr,
        upper: Expr,
        policy: pse_model::generated::enums::ExtrapolationPolicy,
    },
    /// Knowledge-specific post-solve check, not a new equation.
    Check(Predicate),
}
/// Explicit maximum incidences of a port; `None` means authored `many`.
#[derive(Clone, Debug, PartialEq)]
pub struct Connectivity {
    /// Maximum original connections targeting the port.
    pub incoming: Option<usize>,
    /// Maximum original connections originating at the port.
    pub outgoing: Option<usize>,
    /// Original annotation and owning instance.
    pub lineage: Lineage,
}
pub(crate) fn connectivity_limits(
    arguments: &[String],
    at: DeclarationId,
) -> Result<(Option<usize>, Option<usize>)> {
    let [incoming, outgoing] = arguments else {
        return Err(invalid(
            at,
            "connectivity requires incoming and outgoing maxima",
        ));
    };
    let limit = |text: &str| {
        if text == "many" {
            Ok(None)
        } else {
            text.parse::<usize>().map(Some).map_err(|_| {
                invalid(
                    at,
                    "connection maximum must be a nonnegative integer or many",
                )
            })
        }
    };
    Ok((limit(incoming)?, limit(outgoing)?))
}
/// Instantiated target and original source for a typed downstream hint.
#[derive(Clone, Debug, PartialEq)]
pub struct Annotation {
    /// Existing scalar member identity, never a copied variable.
    pub target: SemanticId,
    /// Typed payload.
    pub value: AnnotationValue,
    /// Demand and declaration provenance.
    pub lineage: Lineage,
}
pub(crate) fn label(source: &str, at: DeclarationId) -> Result<String> {
    match parse_static(source).map_err(|e| invalid(at, e.to_string()))? {
        StaticValue::Text(value) => Ok(value),
        StaticValue::Expression(Expr {
            kind: ExprKind::Path(path),
            ..
        }) if path.segments.iter().all(|s| s.indices.is_empty()) => Ok(dsl::render_path(&path)),
        _ => Err(invalid(at, "annotation requires a label")),
    }
}
impl Engine<'_, '_> {
    pub(super) fn check_connectivity(&self) -> Result<()> {
        let mut counts = std::collections::BTreeMap::<SemanticId, (usize, usize)>::new();
        for connection in self.model.connections.values() {
            for port in [connection.from, connection.to] {
                if !self.model.ports.contains_key(&port)
                    || !self.model.connectivity.contains_key(&port)
                {
                    return Err(invalid(
                        connection.lineage.declaration,
                        "connected port requires an explicit connectivity policy",
                    ));
                }
            }
            counts.entry(connection.from).or_default().1 += 1;
            counts.entry(connection.to).or_default().0 += 1;
        }
        for (port, policy) in &self.model.connectivity {
            if !self.model.ports.contains_key(port) {
                return Err(invalid(
                    policy.lineage.declaration,
                    "connectivity target is not an active port",
                ));
            }
            let (incoming, outgoing) = counts.get(port).copied().unwrap_or_default();
            if policy.incoming.is_some_and(|max| incoming > max)
                || policy.outgoing.is_some_and(|max| outgoing > max)
            {
                return Err(invalid(
                    policy.lineage.declaration,
                    format!(
                        "port {port} exceeds declared connectivity: {incoming} incoming, {outgoing} outgoing"
                    ),
                ));
            }
        }
        Ok(())
    }
    pub(super) fn annotation(
        &mut self,
        instance: InstanceId,
        row: &crate::Declaration,
        env: &Environment,
    ) -> Result<()> {
        let at = row.declaration_id;
        let a = row
            .value
            .annotation
            .as_ref()
            .ok_or_else(|| invalid(at, "annotation payload"))?;
        let ports = a.annotation_type == "connectivity";
        let targets = self.annotation_targets(instance, &a.target, env, at, ports)?;
        if ports {
            let (incoming, outgoing) = connectivity_limits(&a.arguments, at)?;
            for (target, _, _) in targets {
                self.reserve(1)?;
                let policy = Connectivity {
                    incoming,
                    outgoing,
                    lineage: self.lineage(instance, row, &[at]),
                };
                if self.model.connectivity.insert(target, policy).is_some() {
                    return Err(invalid(at, "multiple connectivity policies for one port"));
                }
            }
            return Ok(());
        }
        for (target, ty, env) in targets {
            let env = &env;
            let expression = |engine: &mut Self, index: usize| -> Result<Expr> {
                let source = a
                    .arguments
                    .get(index)
                    .ok_or_else(|| invalid(at, "annotation argument missing"))?;
                let parsed = dsl::parse_expr(source).map_err(|e| invalid(at, e.to_string()))?;
                let value = engine.rewrite(instance, &parsed, env, &[at])?;
                let types = engine
                    .model
                    .symbols
                    .iter()
                    .map(|(id, s)| (crate::specialize::symbol_name(*id), s.ty.clone()))
                    .collect();
                if crate::expression::infer(
                    &value,
                    &types,
                    &engine.model.function_contracts(engine.p),
                    engine.c,
                    at,
                    Some(&ty),
                )? != ty
                {
                    return Err(invalid(at, "annotation physical type differs from target"));
                }
                Ok(value)
            };
            let value = match (a.annotation_type.as_str(), a.arguments.len()) {
                ("start", 1) => AnnotationValue::Start(expression(self, 0)?),
                ("nominal", 1) => AnnotationValue::Nominal(expression(self, 0)?),
                ("bounds", 2) => {
                    AnnotationValue::Bounds(expression(self, 0)?, expression(self, 1)?)
                }
                ("scale", 1) => AnnotationValue::Scale(
                    label(&a.arguments[0], at)?
                        .parse()
                        .map_err(|e| invalid(at, format!("scaling scheme: {e}")))?,
                ),
                ("objective", 1) => {
                    AnnotationValue::Objective(objective_sense(&a.arguments[0], at)?)
                }
                ("report", 1) => AnnotationValue::Report(label(&a.arguments[0], at)?),
                ("valid", 3) => {
                    let policy = extrapolation_policy(&a.arguments[2], at)?;
                    AnnotationValue::Valid {
                        lower: expression(self, 0)?,
                        upper: expression(self, 1)?,
                        policy,
                    }
                }
                ("check", 1) => {
                    let predicate = dsl::parse_predicate(&a.arguments[0])
                        .map_err(|e| invalid(at, e.to_string()))?;
                    let predicate = self.rewrite_predicate(instance, &predicate, env, &[at])?;
                    let types = self
                        .model
                        .symbols
                        .iter()
                        .map(|(id, s)| (crate::specialize::symbol_name(*id), s.ty.clone()))
                        .collect();
                    crate::expression::predicate(
                        &predicate,
                        &types,
                        &self.model.function_contracts(self.p),
                        self.c,
                        at,
                    )?;
                    AnnotationValue::Check(predicate)
                }
                _ => return Err(invalid(at, "unknown annotation or invalid argument count")),
            };
            if !matches!(ty, Type::Quantity(_)) {
                return Err(invalid(
                    at,
                    "annotation target must have a complete physical type",
                ));
            }
            if matches!(value, AnnotationValue::Objective(_)) {
                if !self.model.symbols.contains_key(&target) {
                    return Err(invalid(at, "objective requires a scalar value member"));
                }
                if self
                    .model
                    .annotations
                    .iter()
                    .any(|a| matches!(a.value, AnnotationValue::Objective(_)))
                {
                    return Err(invalid(at, "a selected analysis has exactly one objective"));
                }
            }
            self.reserve(1)?;
            self.model.annotations.push(Annotation {
                target,
                value,
                lineage: self.lineage(instance, row, &[at]),
            });
        }
        Ok(())
    }
}

/// Determine the physical target before instantiation, including whole equation families.
pub(crate) fn target_declaration(
    p: &crate::CheckedPackage,
    c: &crate::TypeContext<'_>,
    at: DeclarationId,
    source: &str,
    env: &std::collections::BTreeMap<String, Type>,
) -> Result<Option<DeclarationId>> {
    let ExprKind::Path(mut path) = dsl::parse_expr(source)
        .map_err(|e| invalid(at, e.to_string()))?
        .kind
    else {
        return Ok(None);
    };
    let Some(last) = path.segments.pop() else {
        return Ok(None);
    };
    if !last.indices.is_empty() {
        return Ok(None);
    }
    if let Some(id) = p.resolve(at, source) {
        return Ok(Some(id));
    }
    if path.segments.is_empty() {
        return Ok(None);
    }
    let parent = Expr {
        kind: ExprKind::Path(path),
        span: Default::default(),
    };
    let owner = crate::expression::infer(&parent, env, p, c, at, None)?;
    match owner {
        Type::Definition(id) | Type::Interface(id) => {
            Ok(p.members.get(&id).and_then(|m| m.get(&last.name)).copied())
        }
        _ => Err(invalid(
            at,
            "whole-member annotation requires a scalar child path",
        )),
    }
}

pub(crate) fn target_type(
    p: &crate::CheckedPackage,
    c: &crate::TypeContext<'_>,
    at: DeclarationId,
    source: &str,
    env: &std::collections::BTreeMap<String, Type>,
) -> Result<Type> {
    if let Some(id) = target_declaration(p, c, at, source, env)? {
        let row = &p.declarations[&id];
        if let Some(e) = &row.value.equation {
            let mut local = env.clone();
            for i in &e.indices {
                let expression =
                    dsl::parse_expr(&i.domain).map_err(|e| invalid(at, e.to_string()))?;
                let (Type::Set(element) | Type::Continuous(_, element)) =
                    crate::expression::infer(&expression, &local, p, c, id, None)?
                else {
                    return Err(invalid(at, "annotation coordinate domain"));
                };
                local.insert(i.name.clone(), *element);
            }
            let equation =
                dsl::parse_equation(&e.expression).map_err(|e| invalid(at, e.to_string()))?;
            let dsl::EquationKind::Relation { lhs, .. } = equation.kind else {
                return Err(invalid(
                    at,
                    "annotation target must have a fixed physical row contract",
                ));
            };
            let Type::Quantity(s) = crate::expression::infer(&lhs, &local, p, c, id, None)? else {
                return Err(invalid(at, "physical row annotation required"));
            };
            let quantity = pse_quantity::scheme::Scheme::Delta(Box::new(s))
                .resolve_with_evidence(
                    c.quantities,
                    &std::collections::BTreeMap::new(),
                    c.preconditions,
                )
                .map_err(|e| invalid(at, e.to_string()))?;
            return Ok(Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                quantity,
            )));
        }
        if row
            .value
            .binding
            .as_ref()
            .is_some_and(|b| !b.indices.is_empty())
            || row
                .value
                .accumulator
                .as_ref()
                .is_some_and(|a| !a.indices.is_empty())
        {
            if let Some(ty) = p.types.get(&id) {
                return Ok(ty.clone());
            }
        }
    }
    let expression = dsl::parse_expr(source).map_err(|e| invalid(at, e.to_string()))?;
    crate::expression::infer(&expression, env, p, c, at, None)
}

#[cfg(test)]
mod tests {
    use pse_model::generated::enums::ExtrapolationPolicy;

    /// The policy of `annotation valid` is the registry enumeration; any other label is
    /// refused at parse, so no consumer compares its spelling (ADR-0115 Outcome 3).
    #[test]
    fn extrapolation_policy_typed() {
        let at = crate::DeclarationId::from(pse_ids::SemanticId::NIL);
        assert_eq!(
            super::extrapolation_policy("reject", at).ok(),
            Some(ExtrapolationPolicy::Reject)
        );
        assert_eq!(
            super::extrapolation_policy("extrapolate", at).ok(),
            Some(ExtrapolationPolicy::Extrapolate)
        );
        for refused in ["clamp", "Reject", "\"extrapolate \""] {
            assert!(
                super::extrapolation_policy(refused, at).is_err(),
                "{refused}"
            );
        }
        assert_eq!(
            ExtrapolationPolicy::ALL.map(ExtrapolationPolicy::as_str),
            ["reject", "extrapolate"]
        );
    }
}
