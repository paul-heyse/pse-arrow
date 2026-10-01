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
/// Orientation of an authored objective member: the registry enumeration.
pub use pse_model::generated::enums::NativeObjectiveSense as ObjectiveSense;
/// An `annotation objective` with its members evaluated to canonical values (ADR-0111).
/// Grouping into levels, and every rule across members, belongs to the objective
/// admission after specialization ([`crate::specialize::Objectives`]).
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectiveDeclaration {
    /// Authored orientation of the member.
    pub sense: ObjectiveSense,
    /// Lexicographic priority; a lower value is optimized first.
    pub priority: Option<i64>,
    /// Positive, finite, dimensionless weight within the member's level.
    pub weight: Option<f64>,
    /// Positive canonical value of the member's type dividing it into a dimensionless term.
    pub normalization: Option<crate::specialize::Value>,
    /// Nonnegative level degradation tolerance, a difference of the member's term type.
    pub absolute_tolerance: Option<crate::specialize::Value>,
    /// Nonnegative dimensionless level degradation tolerance relative to the level optimum.
    pub relative_tolerance: Option<f64>,
}
/// The registry annotation kinds (ADR-0123 Outcome 1).
pub use pse_model::generated::enums::ModelingAnnotationKind as AnnotationKind;
type Declared = pse_authoring::language::AuthoredModelingDeclarationsFieldValueAnnotation;
/// What an annotation's arguments are, by kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shape {
    /// This many expressions of the target's physical type.
    Expressions(usize),
    /// One presentation label.
    Label,
    /// One predicate over the model.
    Predicate,
    /// The typed members of an objective (ADR-0111).
    Objective,
    /// The typed scaling scheme.
    Scheme,
    /// The typed connection maxima of a port.
    Connectivity,
}
/// The shape of a declared annotation. The match over the registry kinds is exhaustive:
/// every kind states its argument count and exactly which typed members it carries.
/// # Errors
/// Arguments or typed members that disagree with the kind.
pub(crate) fn shape(a: &Declared, at: DeclarationId) -> Result<Shape> {
    // (arguments, objective, scheme, connectivity)
    let (shape, expected) = match a.kind {
        AnnotationKind::Start | AnnotationKind::Nominal => {
            (Shape::Expressions(1), (1, false, false, false))
        }
        AnnotationKind::Bounds => (Shape::Expressions(2), (2, false, false, false)),
        // Hard closure ranges have exactly two physical endpoints.
        AnnotationKind::Valid => (Shape::Expressions(2), (2, false, false, false)),
        AnnotationKind::Report => (Shape::Label, (1, false, false, false)),
        AnnotationKind::Check => (Shape::Predicate, (1, false, false, false)),
        AnnotationKind::Objective => (Shape::Objective, (0, true, false, false)),
        AnnotationKind::Scale => (Shape::Scheme, (0, false, true, false)),
        AnnotationKind::Connectivity => (Shape::Connectivity, (0, false, false, true)),
    };
    let actual = (
        a.arguments.len(),
        a.objective.is_some(),
        a.scheme.is_some(),
        a.connectivity.is_some(),
    );
    if actual != expected {
        return Err(invalid(
            at,
            format!(
                "annotation {} takes {} argument(s) and exactly its typed members",
                a.kind.as_str(),
                expected.0
            ),
        ));
    }
    Ok(shape)
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
    /// Select this existing scalar member as an objective member (ADR-0111).
    Objective(ObjectiveDeclaration),
    /// Output label.
    Report(String),
    /// An unconditional physical validity range and its owning layer.
    Valid {
        /// Lower endpoint of the validity range.
        lower: Expr,
        /// Upper endpoint of the validity range.
        upper: Expr,
        /// The validity layer the range belongs to.
        layer: pse_model::generated::enums::ModelingValidityLayer,
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
/// The typed connection maxima of a connectivity annotation; `None` admits any number.
pub(crate) fn connectivity_limits(
    a: &Declared,
    at: DeclarationId,
) -> Result<(Option<usize>, Option<usize>)> {
    let limits = a
        .connectivity
        .as_ref()
        .ok_or_else(|| invalid(at, "connectivity requires incoming and outgoing maxima"))?;
    let limit = |value: Option<i64>| {
        value
            .map(|v| {
                usize::try_from(v)
                    .map_err(|_| invalid(at, "connection maximum must be nonnegative"))
            })
            .transpose()
    };
    Ok((limit(limits.incoming)?, limit(limits.outgoing)?))
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
                if !(self.model.ports.contains_key(&port)
                    || self.model.material_ports.contains_key(&port))
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
            if !(self.model.ports.contains_key(port)
                || self.model.material_ports.contains_key(port))
            {
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
        let kind = shape(a, at)?;
        let ports = kind == Shape::Connectivity;
        let targets = self.annotation_targets(instance, &a.target, env, at, ports)?;
        if ports {
            let (incoming, outgoing) = connectivity_limits(a, at)?;
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
            if let Some(members) = a.objective.as_ref().filter(|_| kind == Shape::Objective) {
                if !self.model.symbols.contains_key(&target) {
                    return Err(invalid(at, "objective requires a scalar value member"));
                }
                if self
                    .model
                    .annotations
                    .iter()
                    .any(|a| a.target == target && matches!(a.value, AnnotationValue::Objective(_)))
                {
                    return Err(invalid(at, "a member is one objective at most"));
                }
                let value =
                    AnnotationValue::Objective(self.objective_declaration(members, &ty, env, at)?);
                self.reserve(1)?;
                self.model.annotations.push(Annotation {
                    target,
                    value,
                    lineage: self.lineage(instance, row, &[at]),
                });
                continue;
            }
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
            let value = match a.kind {
                AnnotationKind::Start => AnnotationValue::Start(expression(self, 0)?),
                AnnotationKind::Nominal => AnnotationValue::Nominal(expression(self, 0)?),
                AnnotationKind::Bounds => {
                    AnnotationValue::Bounds(expression(self, 0)?, expression(self, 1)?)
                }
                AnnotationKind::Scale => {
                    AnnotationValue::Scale(a.scheme.ok_or_else(|| invalid(at, "scaling scheme"))?)
                }
                AnnotationKind::Report => AnnotationValue::Report(label(&a.arguments[0], at)?),
                AnnotationKind::Valid => AnnotationValue::Valid {
                    lower: expression(self, 0)?,
                    upper: expression(self, 1)?,
                    layer: pse_model::generated::enums::ModelingValidityLayer::Closure,
                },
                AnnotationKind::Check => {
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
                AnnotationKind::Objective | AnnotationKind::Connectivity => {
                    return Err(invalid(at, "annotation kind handled above"));
                }
            };
            if ty.quantity_scheme().is_none() {
                return Err(invalid(
                    at,
                    "annotation target must have a complete physical type",
                ));
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

impl Engine<'_, '_> {
    /// Evaluate an objective's members in its instance (ADR-0111): the weight is a positive
    /// dimensionless number, the normalization a positive value of the member's type, and
    /// the tolerances nonnegative, the absolute one a difference of the member's term: the
    /// member itself, or the dimensionless quotient by its normalization.
    fn objective_declaration(
        &self,
        members: &pse_authoring::language::AuthoredModelingDeclarationsFieldValueAnnotationObjective,
        ty: &Type,
        env: &Environment,
        at: DeclarationId,
    ) -> Result<ObjectiveDeclaration> {
        use crate::ObjectiveRefusal as Refusal;
        let refuse = |reason| crate::ModelingError::Objective {
            declaration: at.as_id(),
            reason,
        };
        let scalar = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
            self.c
                .quantities
                .neutral_dimensionless()
                .ok_or_else(|| invalid(at, "objective members need a neutral scalar type"))?,
        ));
        let number =
            |text: &Option<String>, ty: &Type| -> Result<Option<(crate::specialize::Value, f64)>> {
                text.as_deref()
                    .map(|text| {
                        let value = self.eval(at, env, text, Some(ty))?;
                        let scalar = value.scalar(at)?;
                        Ok((value, scalar))
                    })
                    .transpose()
            };
        let weight = number(&members.weight, &scalar)?;
        if weight
            .as_ref()
            .is_some_and(|(_, w)| !(w.is_finite() && *w > 0.0))
        {
            return Err(refuse(Refusal::InvalidWeight));
        }
        let normalization = number(&members.normalization, ty)?;
        if normalization
            .as_ref()
            .is_some_and(|(_, n)| !(n.is_finite() && *n > 0.0))
        {
            return Err(refuse(Refusal::InvalidNormalization));
        }
        let term = if normalization.is_some() { &scalar } else { ty };
        let Type::Quantity(scheme) = term else {
            return Err(invalid(at, "objective requires a physical member"));
        };
        let difference = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
            pse_quantity::scheme::Scheme::Delta(Box::new(scheme.clone()))
                .resolve_with_evidence(
                    self.c.quantities,
                    &std::collections::BTreeMap::new(),
                    self.c.preconditions,
                )
                .map_err(|e| invalid(at, e.to_string()))?,
        ));
        let absolute = number(&members.absolute_tolerance, &difference)?;
        let relative = number(&members.relative_tolerance, &scalar)?;
        if [&absolute, &relative]
            .into_iter()
            .flatten()
            .any(|(_, t)| !(t.is_finite() && *t >= 0.0))
        {
            return Err(refuse(Refusal::InvalidTolerance));
        }
        Ok(ObjectiveDeclaration {
            sense: members.sense,
            priority: members.priority,
            weight: weight.map(|(_, w)| w),
            normalization: normalization.map(|(v, _)| v),
            absolute_tolerance: absolute.map(|(v, _)| v),
            relative_tolerance: relative.map(|(_, r)| r),
        })
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
        if (row
            .value
            .binding
            .as_ref()
            .is_some_and(|b| !b.indices.is_empty())
            || row
                .value
                .accumulator
                .as_ref()
                .is_some_and(|a| !a.indices.is_empty()))
            && let Some(ty) = p.types.get(&id)
        {
            return Ok(ty.clone());
        }
    }
    let expression = dsl::parse_expr(source).map_err(|e| invalid(at, e.to_string()))?;
    crate::expression::infer(&expression, env, p, c, at, None)
}
