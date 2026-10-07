// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed model hints are collected for their declared downstream consumer.
use crate::specialize::{Engine, Environment, Lineage};
use crate::{DeclarationId, InstanceId, Result, Type, invalid};
use pse_authoring::{
    dsl::{self, Expr, ExprKind, Predicate},
    language::StaticValue,
};
use pse_ids::SemanticId;
/// Orientation of an authored objective member: the registry enumeration.
pub use pse_model::generated::enums::NativeObjectiveSense as ObjectiveSense;
use pse_quantity::scheme::Scheme;
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
    /// Authored physical accuracy goal attached to one scalar target.
    AccuracyGoal,
    /// Tagged engineering characteristic magnitude attached to one target.
    EngineeringScale,
    /// Inherited shared engineering rule selected by one target.
    EngineeringDefault,
    /// Package-level marker that makes an existing constant a shared engineering rule.
    EngineeringRule,
}
/// The shape of a declared annotation. The match over the registry kinds is exhaustive:
/// every kind states its argument count and exactly which typed members it carries.
/// # Errors
/// Arguments or typed members that disagree with the kind.
pub(crate) fn shape(a: &Declared, at: DeclarationId) -> Result<Shape> {
    // (arguments, objective, scheme, connectivity)
    let (shape, expected) = match a.kind {
        AnnotationKind::Start | AnnotationKind::Nominal => (
            Shape::Expressions(1),
            (1, false, false, false, false, false, false),
        ),
        AnnotationKind::Bounds => (
            Shape::Expressions(2),
            (2, false, false, false, false, false, false),
        ),
        // Hard closure ranges have exactly two physical endpoints.
        AnnotationKind::Valid => (
            Shape::Expressions(2),
            (2, false, false, false, false, false, false),
        ),
        AnnotationKind::Report => (Shape::Label, (1, false, false, false, false, false, false)),
        AnnotationKind::Check => (
            Shape::Predicate,
            (1, false, false, false, false, false, false),
        ),
        AnnotationKind::Objective => (
            Shape::Objective,
            (0, true, false, false, false, false, false),
        ),
        AnnotationKind::Scale => (Shape::Scheme, (0, false, true, false, false, false, false)),
        AnnotationKind::Connectivity => (
            Shape::Connectivity,
            (0, false, false, true, false, false, false),
        ),
        AnnotationKind::AccuracyGoal => (
            Shape::AccuracyGoal,
            (0, false, false, false, true, false, false),
        ),
        AnnotationKind::EngineeringScale => (
            Shape::EngineeringScale,
            (0, false, false, false, false, true, false),
        ),
        AnnotationKind::EngineeringDefault => (
            Shape::EngineeringDefault,
            (0, false, false, false, false, false, true),
        ),
        AnnotationKind::EngineeringRule => (
            Shape::EngineeringRule,
            (0, false, false, false, false, false, false),
        ),
    };
    let actual = (
        a.arguments.len(),
        a.objective.is_some(),
        a.scheme.is_some(),
        a.connectivity.is_some(),
        a.accuracy_goal.is_some(),
        a.engineering_scale.is_some(),
        a.engineering_default.is_some(),
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
    /// Scoped scalar output or original-objective goal. Physical expressions retain their
    /// parsed AST and are interpreted by the existing typed preparation path.
    AccuracyGoal(Box<AccuracyGoal>),
    /// Tagged physical characteristic magnitude, independent of conditioning scales.
    EngineeringScale(EngineeringScale),
    /// A shared default selected by the identity of its source constant.
    EngineeringDefault {
        /// The marked constant whose allowance is selected for this target.
        rule_id: SemanticId,
        /// Authored precedence from the actual declaring scope.
        source: pse_model::generated::enums::NumericalSource,
    },
}
/// One admitted accuracy-goal declaration after target and lexical specialization.
#[derive(Clone, Debug, PartialEq)]
pub struct AccuracyGoal {
    /// Stable identity derived from declaration identity and instantiated target identity.
    pub id: SemanticId,
    /// Target's physical quantity scheme.
    pub quantity: Type,
    /// Whether the goal concerns a selected output or the original optimum value.
    pub subject: pse_model::generated::enums::AccuracyGoalSubject,
    /// The supported result observation.
    pub observation: pse_model::generated::enums::AccuracyObservation,
    /// Optional admitted sample time.
    pub time: Option<Expr>,
    /// Optional physical error resolution; its expected type is the target difference.
    pub resolution: Option<Expr>,
    /// Inclusive physical lower criterion endpoint.
    pub criterion_lower: Option<Expr>,
    /// Inclusive physical upper criterion endpoint.
    pub criterion_upper: Option<Expr>,
    /// Evidence class the assessment must meet.
    pub required_class: pse_model::generated::enums::NumericalAccuracyClass,
    /// Whether a resolved violation may be returned or must refuse result use.
    pub use_policy: pse_model::generated::enums::AccuracyGoalUse,
    /// Whether the enclosing strategy may refine this goal under its existing grant.
    pub refine: bool,
    /// Authored precedence is derived from the declaration's actual owner chain.
    pub source: pse_model::generated::enums::NumericalSource,
}
/// One explicitly declared engineering scale bound to an existing target.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineeringScale {
    /// Stable identity derived from declaration identity and instantiated target identity.
    pub id: SemanticId,
    /// Physical type of the target's difference magnitude.
    pub quantity: Type,
    /// Whether value is a characteristic magnitude, range width, or reference difference.
    pub kind: pse_model::generated::enums::EngineeringScaleKind,
    /// Authored scale expression, specialized in its declaring context.
    pub value: Expr,
    /// Authored precedence is derived from the declaration's actual owner chain.
    pub source: pse_model::generated::enums::NumericalSource,
}
/// A shared rule declared by marking a typed package constant.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineeringRule {
    /// The marked constant identity is the rule identity.
    pub id: SemanticId,
    /// Full declared physical type of the constant, including point/difference meaning.
    pub quantity: Type,
    /// Admitted exact value of the constant.
    pub value: crate::specialize::Value,
    /// Marker declaration retained as provenance separately from rule identity.
    pub marker: DeclarationId,
}
/// Materialize each package marker against its already-admitted typed constant.
///
/// The marker never owns a second numeric value: the rule ID is the constant ID and its
/// type/value are read from the checked package's normal constant inventory.
pub(crate) fn engineering_rules(package: &crate::CheckedPackage) -> Result<Vec<EngineeringRule>> {
    use pse_authoring::language::Selected;
    use pse_model::generated::enums::ModelingAnnotationKind as Kind;
    let mut rules = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (marker, declaration) in &package.declarations {
        let Selected::Annotation(annotation) = declaration
            .value
            .selected()
            .map_err(|error| invalid(*marker, error.to_string()))?
        else {
            continue;
        };
        if annotation.kind != Kind::EngineeringRule {
            continue;
        }
        shape(annotation, *marker)?;
        let constant = package
            .names
            .get(annotation.target.trim())
            .copied()
            .ok_or_else(|| invalid(*marker, "engineering rule constant path is unresolved"))?;
        if !seen.insert(constant) {
            return Err(invalid(
                *marker,
                "a shared engineering rule has one source marker",
            ));
        }
        if package.is_test_only(constant) {
            return Err(invalid(
                *marker,
                "a shared engineering rule is not test-only data",
            ));
        }
        let ty = package
            .types
            .get(&constant)
            .cloned()
            .ok_or_else(|| invalid(*marker, "engineering rule constant has no checked type"))?;
        let scheme = ty.quantity_scheme().ok_or_else(|| {
            invalid(
                *marker,
                "engineering rule constant has a physical quantity type",
            )
        })?;
        let resolved = scheme
            .resolve(&package.quantities, &Default::default())
            .map_err(|error| invalid(*marker, error.to_string()))?;
        // Delta is the admitted full-contract subtraction Q - Q. Additive
        // quantities (such as component flow) retain their ordinary scale kind;
        // origin-sensitive points instead resolve to a distinct difference type.
        let difference = Scheme::Delta(Box::new(Scheme::Concrete(resolved)))
            .resolve(&package.quantities, &Default::default())
            .map_err(|error| invalid(*marker, error.to_string()))?;
        if difference != resolved {
            return Err(invalid(
                *marker,
                "engineering rule constant declares its legal physical self-difference type",
            ));
        }
        let typed = package
            .constants
            .get(&constant)
            .ok_or_else(|| invalid(*marker, "engineering rule constant is not admitted"))?;
        if typed.uncertainty.is_some() {
            return Err(invalid(
                *marker,
                "engineering rule constant has no uncertainty",
            ));
        }
        let crate::specialize::Value::Number { bits, quantity } = &typed.value else {
            return Err(invalid(
                *marker,
                "engineering rule constant is a scalar quantity",
            ));
        };
        let magnitude = f64::from_bits(*bits);
        if !magnitude.is_finite() || magnitude <= 0.0 {
            return Err(invalid(
                *marker,
                "engineering rule allowance is finite and positive",
            ));
        }
        if resolved != *quantity {
            return Err(invalid(
                *marker,
                "engineering rule value matches its declared type",
            ));
        }
        rules.push(EngineeringRule {
            id: constant.as_id(),
            quantity: ty,
            value: typed.value.clone(),
            marker: *marker,
        });
    }
    Ok(rules)
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
pub(crate) fn label(source: &StaticValue, at: DeclarationId) -> Result<String> {
    match source {
        StaticValue::Text(value) => Ok(value.clone()),
        StaticValue::Expression(Expr {
            kind: ExprKind::Path(path),
            ..
        }) if path.segments.iter().all(|s| s.indices.is_empty()) => Ok(dsl::render_path(path)),
        _ => Err(invalid(at, "annotation requires a label")),
    }
}
fn difference_type(
    target: &Type,
    quantities: &pse_quantity::QuantityRegistry,
    at: DeclarationId,
) -> Result<Type> {
    let Some(scheme) = target.quantity_scheme() else {
        return Err(invalid(
            at,
            "engineering magnitude target has a complete physical type",
        ));
    };
    Ok(Type::Quantity(Scheme::Concrete(
        Scheme::Delta(Box::new(scheme.clone()))
            .resolve(quantities, &Default::default())
            .map_err(|error| invalid(at, error.to_string()))?,
    )))
}
fn compatible_difference(
    target: &Type,
    rule: &Type,
    quantities: &pse_quantity::QuantityRegistry,
    at: DeclarationId,
) -> Result<bool> {
    let expected = difference_type(target, quantities, at)?;
    let Some(expected) = expected.quantity_scheme() else {
        return Err(invalid(at, "target difference type"));
    };
    let Some(rule) = rule.quantity_scheme() else {
        return Ok(false);
    };
    let expected = expected
        .resolve(quantities, &Default::default())
        .map_err(|error| invalid(at, error.to_string()))?;
    let rule = rule
        .resolve(quantities, &Default::default())
        .map_err(|error| invalid(at, error.to_string()))?;
    Ok(expected == rule)
}
fn annotation_source(
    package: &crate::CheckedPackage,
    declaration: DeclarationId,
) -> pse_model::generated::enums::NumericalSource {
    use pse_model::generated::enums::{ModelingDeclarationKind as Kind, NumericalSource};
    let mut cursor = package
        .declarations
        .get(&declaration)
        .and_then(|row| row.parent_id);
    while let Some(owner) = cursor {
        let Some(row) = package.declarations.get(&owner) else {
            break;
        };
        if row.value.kind == Kind::Case {
            return NumericalSource::Case;
        }
        cursor = row.parent_id;
    }
    NumericalSource::Model
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
        // Shared rules are package constants and were admitted into the rule inventory
        // before instance traversal; this marker does not resolve a coordinate target.
        if a.kind == AnnotationKind::EngineeringRule {
            return Ok(());
        }
        let ports = kind == Shape::Connectivity;
        let targets = self.annotation_targets(
            instance,
            &self.p.expression_at(at, "annotation.target", 0)?.clone(),
            env,
            at,
            ports,
        )?;
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
            let expression_as = |engine: &mut Self,
                                 role: &str,
                                 index: usize,
                                 expected: Option<&Type>|
             -> Result<Expr> {
                let parsed = engine.p.expression_at(at, role, index)?.clone();
                let value = engine.rewrite(instance, &parsed, env, &[at])?;
                let types = engine
                    .model
                    .symbols
                    .iter()
                    .map(|(id, s)| (crate::specialize::symbol_name(*id), s.ty.clone()))
                    .collect();
                let actual = crate::expression::infer(
                    &value,
                    &types,
                    &engine.model.function_contracts(engine.p),
                    engine.c,
                    at,
                    expected,
                )?;
                if expected.is_some_and(|expected| actual != *expected)
                    || actual.quantity_scheme().is_none()
                {
                    return Err(invalid(
                        at,
                        "annotation expression has an incompatible physical type",
                    ));
                }
                Ok(value)
            };
            let expression = |engine: &mut Self, index: usize| -> Result<Expr> {
                if a.arguments.get(index).is_none() {
                    return Err(invalid(at, "annotation argument missing"));
                }
                expression_as(engine, "annotation.arguments", index, Some(&ty))
            };
            if kind == Shape::AccuracyGoal {
                if !self.model.symbols.contains_key(&target) {
                    return Err(invalid(
                        at,
                        "accuracy goal target is an existing scalar member",
                    ));
                }
                let members = a
                    .accuracy_goal
                    .as_ref()
                    .ok_or_else(|| invalid(at, "accuracy goal members"))?;
                if members.required_class
                    == pse_model::generated::enums::NumericalAccuracyClass::Unresolved
                {
                    return Err(invalid(
                        at,
                        "accuracy goal requests estimated or certified evidence",
                    ));
                }
                if members.resolution.is_none()
                    && members.criterion_lower.is_none()
                    && members.criterion_upper.is_none()
                {
                    return Err(invalid(at, "accuracy goal has resolution or a criterion"));
                }
                if members.use_policy
                    == pse_model::generated::enums::AccuracyGoalUse::RequireSatisfied
                    && members.criterion_lower.is_none()
                    && members.criterion_upper.is_none()
                {
                    return Err(invalid(
                        at,
                        "require_satisfied accuracy goal has a criterion",
                    ));
                }
                let sampled =
                    members.observation == pse_model::generated::enums::AccuracyObservation::Sample;
                if sampled != members.time.is_some() {
                    return Err(invalid(at, "only sample observations carry a sample time"));
                }
                let difference = difference_type(&ty, self.c.quantities, at)?;
                let time = members
                    .time
                    .as_ref()
                    .map(|_| expression_as(self, "annotation.accuracy_goal.time", 0, None))
                    .transpose()?;
                let resolution = members
                    .resolution
                    .as_ref()
                    .map(|_| {
                        expression_as(
                            self,
                            "annotation.accuracy_goal.resolution",
                            0,
                            Some(&difference),
                        )
                    })
                    .transpose()?;
                let criterion_lower = members
                    .criterion_lower
                    .as_ref()
                    .map(|_| {
                        expression_as(
                            self,
                            "annotation.accuracy_goal.criterion_lower",
                            0,
                            Some(&ty),
                        )
                    })
                    .transpose()?;
                let criterion_upper = members
                    .criterion_upper
                    .as_ref()
                    .map(|_| {
                        expression_as(
                            self,
                            "annotation.accuracy_goal.criterion_upper",
                            0,
                            Some(&ty),
                        )
                    })
                    .transpose()?;
                let id = pse_ids::named_id(at.as_id(), &target.to_string());
                let value = AnnotationValue::AccuracyGoal(Box::new(AccuracyGoal {
                    id,
                    quantity: ty.clone(),
                    subject: members.subject,
                    observation: members.observation,
                    time,
                    resolution,
                    criterion_lower,
                    criterion_upper,
                    required_class: members.required_class,
                    use_policy: members.use_policy,
                    refine: members.refine,
                    source: annotation_source(self.p, at),
                }));
                self.reserve(1)?;
                self.model.annotations.push(Annotation {
                    target,
                    value,
                    lineage: self.lineage(instance, row, &[at]),
                });
                continue;
            }
            if kind == Shape::EngineeringScale {
                if !self.model.symbols.contains_key(&target) {
                    return Err(invalid(
                        at,
                        "engineering scale target is an existing scalar member",
                    ));
                }
                let members = a
                    .engineering_scale
                    .as_ref()
                    .ok_or_else(|| invalid(at, "engineering scale members"))?;
                let expected = difference_type(&ty, self.c.quantities, at)?;
                let value = expression_as(
                    self,
                    "annotation.engineering_scale.value",
                    0,
                    Some(&expected),
                )?;
                let id = pse_ids::named_id(at.as_id(), &target.to_string());
                self.reserve(1)?;
                self.model.annotations.push(Annotation {
                    target,
                    value: AnnotationValue::EngineeringScale(EngineeringScale {
                        id,
                        quantity: expected,
                        kind: members.kind,
                        value,
                        source: annotation_source(self.p, at),
                    }),
                    lineage: self.lineage(instance, row, &[at]),
                });
                continue;
            }
            if kind == Shape::EngineeringDefault {
                if !self.model.symbols.contains_key(&target) {
                    return Err(invalid(
                        at,
                        "engineering default target is an existing scalar member",
                    ));
                }
                let path = &a
                    .engineering_default
                    .as_ref()
                    .ok_or_else(|| invalid(at, "engineering default selector"))?
                    .rule;
                let rule_id = self
                    .p
                    .names
                    .get(path.trim())
                    .copied()
                    .ok_or_else(|| invalid(at, "shared engineering rule path is unresolved"))?
                    .as_id();
                let rule = self
                    .model
                    .engineering_rules
                    .iter()
                    .find(|rule| rule.id == rule_id)
                    .ok_or_else(|| {
                        invalid(at, "selected constant is not a shared engineering rule")
                    })?;
                if !compatible_difference(&ty, &rule.quantity, self.c.quantities, at)? {
                    return Err(invalid(
                        at,
                        "shared engineering rule has an incompatible physical quantity",
                    ));
                }
                self.reserve(1)?;
                self.model.annotations.push(Annotation {
                    target,
                    value: AnnotationValue::EngineeringDefault {
                        rule_id,
                        source: annotation_source(self.p, at),
                    },
                    lineage: self.lineage(instance, row, &[at]),
                });
                continue;
            }
            let value = match a.kind {
                AnnotationKind::Start => AnnotationValue::Start(expression(self, 0)?),
                AnnotationKind::Nominal => AnnotationValue::Nominal(expression(self, 0)?),
                AnnotationKind::Bounds => {
                    AnnotationValue::Bounds(expression(self, 0)?, expression(self, 1)?)
                }
                AnnotationKind::Scale => {
                    AnnotationValue::Scale(a.scheme.ok_or_else(|| invalid(at, "scaling scheme"))?)
                }
                AnnotationKind::Report => AnnotationValue::Report(label(
                    self.p.static_at(at, "annotation.arguments", 0)?,
                    at,
                )?),
                AnnotationKind::Valid => AnnotationValue::Valid {
                    lower: expression(self, 0)?,
                    upper: expression(self, 1)?,
                    layer: pse_model::generated::enums::ModelingValidityLayer::Closure,
                },
                AnnotationKind::Check => {
                    let predicate = self.p.predicate_at(at, "annotation.arguments", 0)?.clone();
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
                AnnotationKind::Objective
                | AnnotationKind::Connectivity
                | AnnotationKind::AccuracyGoal
                | AnnotationKind::EngineeringScale
                | AnnotationKind::EngineeringDefault
                | AnnotationKind::EngineeringRule => {
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
        let scalar = Type::Quantity(Scheme::Concrete(
            self.c
                .quantities
                .neutral_dimensionless()
                .ok_or_else(|| invalid(at, "objective members need a neutral scalar type"))?,
        ));
        let number = |role: &str,
                      text: &Option<String>,
                      ty: &Type|
         -> Result<Option<(crate::specialize::Value, f64)>> {
            text.as_deref()
                .map(|_| {
                    let value = self.eval_field(
                        at,
                        env,
                        &format!("annotation.objective.{role}"),
                        0,
                        Some(ty),
                    )?;
                    let scalar = value.scalar(at)?;
                    Ok((value, scalar))
                })
                .transpose()
        };
        let weight = number("weight", &members.weight, &scalar)?;
        if weight
            .as_ref()
            .is_some_and(|(_, w)| !(w.is_finite() && *w > 0.0))
        {
            return Err(refuse(Refusal::InvalidWeight));
        }
        let normalization = number("normalization", &members.normalization, ty)?;
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
        let difference = Type::Quantity(Scheme::Concrete(
            Scheme::Delta(Box::new(scheme.clone()))
                .resolve_with_evidence(
                    self.c.quantities,
                    &std::collections::BTreeMap::new(),
                    self.c.preconditions,
                )
                .map_err(|e| invalid(at, e.to_string()))?,
        ));
        let absolute = number(
            "absolute_tolerance",
            &members.absolute_tolerance,
            &difference,
        )?;
        let relative = number("relative_tolerance", &members.relative_tolerance, &scalar)?;
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
    source: &Expr,
    env: &std::collections::BTreeMap<String, Type>,
) -> Result<Option<DeclarationId>> {
    let ExprKind::Path(mut path) = source.kind.clone() else {
        return Ok(None);
    };
    let Some(last) = path.segments.pop() else {
        return Ok(None);
    };
    if !last.indices.is_empty() {
        return Ok(None);
    }
    if let Some(id) = p.resolve(at, &dsl::render_expr(source)) {
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
    source: &Expr,
    env: &std::collections::BTreeMap<String, Type>,
) -> Result<Type> {
    if let Some(id) = target_declaration(p, c, at, source, env)? {
        let row = &p.declarations[&id];
        if let Some(e) = &row.value.equation {
            let mut local = env.clone();
            for (position, i) in e.indices.iter().enumerate() {
                let (Type::Set(element) | Type::Continuous(_, element)) =
                    crate::expression::index_domain_type(
                        p.static_at(id, "equation.indices.domain", position)?,
                        &local,
                        p,
                        c,
                        id,
                    )?
                else {
                    return Err(invalid(at, "annotation coordinate domain"));
                };
                local.insert(i.name.clone(), *element);
            }
            let equation = p.equation_at(id, "equation.expression", 0)?;
            let dsl::EquationKind::Relation { lhs, .. } = &equation.kind else {
                return Err(invalid(
                    at,
                    "annotation target must have a fixed physical row contract",
                ));
            };
            let Type::Quantity(s) = crate::expression::infer(lhs, &local, p, c, id, None)? else {
                return Err(invalid(at, "physical row annotation required"));
            };
            let quantity = Scheme::Delta(Box::new(s))
                .resolve_with_evidence(
                    c.quantities,
                    &std::collections::BTreeMap::new(),
                    c.preconditions,
                )
                .map_err(|e| invalid(at, e.to_string()))?;
            return Ok(Type::Quantity(Scheme::Concrete(quantity)));
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
    crate::expression::infer(source, env, p, c, at, None)
}

#[cfg(test)]
mod engineering_rule_tests {
    use super::*;

    fn checked_rule(quantity: &str, value: &str) -> Result<crate::CheckedPackage> {
        let (registry, _) = crate::kernel_types::physical();
        let preconditions = pse_quantity::PhysicalPreconditions::new(
            pse_quantity::generated::standard_preconditions(),
        )
        .unwrap();
        let text = format!(
            "package p {{ constant allowance:{quantity}={value} provenance(s, role.given); annotation engineering_rule p.allowance; def Root {{}} }}",
        );
        crate::check(
            &crate::kernel_types::try_source(&text)?,
            &crate::TypeContext {
                admissions: None,
                formula_authority: None,
                quantities: &registry,
                preconditions: &preconditions,
                scope: &crate::PhysicalScope::default(),
            },
        )
    }

    #[test]
    fn engineering_rule_self_difference_preserves_complete_additive_and_affine_contracts() {
        let (registry, names) = crate::kernel_types::physical();
        let component = registry.quantity_type(names["ComponentFlow"]).unwrap();
        assert_eq!(component.key.scale_kind, pse_quantity::ScaleKind::Point);
        assert_eq!(
            registry.kind(component.key.kind).unwrap().addition_kind,
            pse_quantity::QuantityAdditionKind::Additive
        );
        assert_ne!(names["ComponentFlow"], names["Flow"]);
        assert!(component.key.subject_kind.is_some());
        assert!(component.key.basis.is_some());
        for (declared, value, expected) in [
            ("Delta<ComponentFlow>", "1{mol/s}", "ComponentFlow"),
            ("Delta<Flow>", "1{mol/s}", "Flow"),
            ("DeltaTemperature", "0.1{K}", "DeltaTemperature"),
            ("Scalar", "0.001", "Scalar"),
        ] {
            let package = checked_rule(declared, value).unwrap();
            let rules = engineering_rules(&package).unwrap();
            assert_eq!(rules.len(), 1);
            let rule = &rules[0];
            let constant = package.names["p.allowance"];
            assert_eq!(rule.id, constant.as_id());
            assert_ne!(rule.marker.as_id(), rule.id);
            assert_eq!(rule.quantity, package.types[&constant]);
            assert_eq!(rule.value, package.constants[&constant].value);
            let crate::specialize::Value::Number { quantity, .. } = &rule.value else {
                panic!("an engineering rule must retain its admitted quantity value");
            };
            assert_eq!(*quantity, names[expected]);
            assert_eq!(
                rule.quantity
                    .quantity_scheme()
                    .unwrap()
                    .resolve(&package.quantities, &Default::default())
                    .unwrap(),
                *quantity
            );
        }
    }

    #[test]
    fn engineering_rule_self_difference_refuses_origin_sensitive_points_and_invalid_values() {
        for (quantity, value) in [("Temperature", "1{K}"), ("Pressure", "1{Pa}")] {
            let package = checked_rule(quantity, value).unwrap();
            let error = engineering_rules(&package).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("legal physical self-difference type"),
                "{error}"
            );
        }
        for value in ["0{mol/s}", "-1{mol/s}"] {
            let package = checked_rule("Delta<ComponentFlow>", value).unwrap();
            let error = engineering_rules(&package).unwrap_err();
            assert!(error.to_string().contains("finite and positive"), "{error}");
        }
        let package = checked_rule("Delta<ComponentFlow>", "1{mol/s} ± standard(0.1)").unwrap();
        let error = engineering_rules(&package).unwrap_err();
        assert!(error.to_string().contains("no uncertainty"), "{error}");
        let mut package = checked_rule("Delta<ComponentFlow>", "1{mol/s}").unwrap();
        let (_, names) = crate::kernel_types::physical();
        let constant = package.names["p.allowance"];
        package.constants.get_mut(&constant).unwrap().value = crate::specialize::Value::Number {
            bits: 1.0_f64.to_bits(),
            quantity: names["Flow"],
        };
        let error = engineering_rules(&package).unwrap_err();
        assert!(
            error.to_string().contains("matches its declared type"),
            "{error}"
        );
    }
}
