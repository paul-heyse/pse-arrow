// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Reserved typed analysis facts; package guards own all scientific consequences.
//!
//! A fact is a member of a registry namespace (ADR-0123 Outcome 1): the analysis route and
//! its derived dynamic flag, the selected objective level, and the selected
//! initialization stages. Expressions read a fact by its path `<namespace>.<member>`;
//! [`Fact::from_path`] is the one place that path is read, by the registry enumeration.
use crate::{
    Bindings, DeclarationId, Result, Type, invalid,
    specialize::{Environment, Value},
};
use pse_ids::SemanticId;
pub use pse_model::generated::enums::ModelingAnalysisRoute as Route;
/// The registry namespaces of reserved facts.
pub use pse_model::generated::enums::ModelingFactNamespace as FactNamespace;
use std::collections::BTreeMap;

/// A reserved fact. Every member of a namespace is a variant, so an undeclared fact is
/// unrepresentable rather than refused by its spelling.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fact {
    /// The selected analysis route, a [`Route`] member.
    Route,
    /// Whether the route integrates in time; always derived from the route.
    Dynamic,
    /// The zero-based objective level of a staged lexicographic step (ADR-0111).
    ObjectiveLevel,
    /// Whether the named initialization stage is selected.
    Stage(String),
}
impl Fact {
    /// The fact's namespace.
    pub fn namespace(&self) -> FactNamespace {
        match self {
            Self::Route | Self::Dynamic => FactNamespace::Analysis,
            Self::ObjectiveLevel => FactNamespace::Objective,
            Self::Stage(_) => FactNamespace::Stage,
        }
    }
    /// The member's name within its namespace.
    pub fn member(&self) -> &str {
        match self {
            Self::Route => "route",
            Self::Dynamic => "dynamic",
            Self::ObjectiveLevel => "level",
            Self::Stage(name) => name,
        }
    }
    /// The fact a namespace member names, or `None` for an undeclared member.
    pub fn new(namespace: FactNamespace, member: &str) -> Option<Self> {
        match (namespace, member) {
            (FactNamespace::Analysis, "route") => Some(Self::Route),
            (FactNamespace::Analysis, "dynamic") => Some(Self::Dynamic),
            (FactNamespace::Objective, "level") => Some(Self::ObjectiveLevel),
            (FactNamespace::Stage, name) if !name.is_empty() => Some(Self::Stage(name.into())),
            _ => None,
        }
    }
    /// The path an expression reads the fact by.
    pub fn path(&self) -> String {
        format!("{}.{}", self.namespace().as_str(), self.member())
    }
    /// The namespace a path's first segment names, if it is a reserved one.
    pub fn namespace_of(path: &str) -> Option<FactNamespace> {
        path.split_once('.')?.0.parse().ok()
    }
    /// The fact a path names; `None` outside the reserved namespaces or for an undeclared
    /// member.
    pub fn from_path(path: &str) -> Option<Self> {
        let (namespace, member) = path.split_once('.')?;
        Self::new(namespace.parse().ok()?, member)
    }
    /// Whether a same-layout mode may bind it: a mode selects `when` variants and stages,
    /// never the analysis route or an objective level (ADR-0119 Outcome 3).
    pub fn is_structural(&self) -> bool {
        self.namespace() == FactNamespace::Stage
    }
}
/// The built-in declaration of the analysis route enumeration.
fn route_id() -> DeclarationId {
    DeclarationId::from(pse_ids::named_id(
        SemanticId::NIL,
        "pse.modeling.analysis.route",
    ))
}
/// A route member's identity, derived from the built-in enumeration and the member name
/// as the named policy derives a package member's (ADR-0123 Outcome 2).
fn route_member(route: Route) -> SemanticId {
    pse_ids::named_id(route_id().as_id(), &format!("member:{}", route.as_str()))
}
fn value(route: Route) -> Value {
    Value::Enum {
        enumeration: route_id(),
        member: route_member(route),
    }
}
/// The route a member identity denotes.
fn route_of(member: SemanticId) -> Option<Route> {
    Route::ALL.into_iter().find(|route| route_member(*route) == member)
}
/// A route member read as `analysis.<route>`.
fn route_constant(path: &str) -> Option<Route> {
    let (namespace, member) = path.split_once('.')?;
    (namespace.parse::<FactNamespace>().ok()? == FactNamespace::Analysis)
        .then(|| member.parse().ok())
        .flatten()
}
pub(crate) fn constant(name: &str) -> Option<Value> {
    route_constant(name).map(value)
}
pub(crate) fn type_of(name: &str) -> Option<Type> {
    match Fact::from_path(name) {
        Some(Fact::Dynamic) => Some(Type::Boolean),
        Some(Fact::Route) => Some(Type::Enum(route_id())),
        _ => route_constant(name).map(|_| Type::Enum(route_id())),
    }
}
/// The selected objective level.
pub(crate) fn objective_level(input: &BTreeMap<Fact, Value>) -> Result<Option<usize>> {
    match input.get(&Fact::ObjectiveLevel) {
        None => Ok(None),
        Some(Value::Integer(level)) => usize::try_from(*level)
            .map(Some)
            .map_err(|_| invalid(SemanticId::NIL, "objective level must be nonnegative")),
        Some(_) => Err(invalid(
            SemanticId::NIL,
            "objective level requires an integer fact",
        )),
    }
}
/// The ambient facts an expression reads, by path: the route (steady by default), the
/// dynamic flag derived from it, and every selected stage. The objective level is not
/// ambient; it selects a transformation after instantiation.
pub(crate) fn facts(input: &BTreeMap<Fact, Value>) -> Result<Environment> {
    let route = match input.get(&Fact::Route) {
        None => Route::Steady,
        Some(Value::Enum {
            enumeration,
            member,
        }) if *enumeration == route_id() => route_of(*member)
            .ok_or_else(|| invalid(SemanticId::NIL, "unknown analysis route"))?,
        _ => {
            return Err(invalid(
                SemanticId::NIL,
                "analysis route requires its declared enum type",
            ));
        }
    };
    let dynamic = Value::Boolean(route != Route::Steady);
    if input.get(&Fact::Dynamic).is_some_and(|v| *v != dynamic) {
        return Err(invalid(
            SemanticId::NIL,
            "analysis.dynamic disagrees with the selected route",
        ));
    }
    let mut output = Environment::new();
    for (fact, value) in input {
        if let Fact::Stage(_) = fact {
            output.insert(fact.path(), value.clone());
        }
    }
    output.insert(Fact::Route.path(), value(route));
    output.insert(Fact::Dynamic.path(), dynamic);
    Ok(output)
}
impl Bindings {
    /// Select one mode. The dynamic Boolean is derived, never an independent decision.
    pub fn with_analysis(mut self, route: Route) -> Self {
        self.facts.insert(Fact::Route, value(route));
        self.facts
            .insert(Fact::Dynamic, Value::Boolean(route != Route::Steady));
        self
    }
    /// Select the objective level a staged lexicographic step optimizes (ADR-0111).
    pub fn with_objective_level(mut self, level: usize) -> Self {
        self.facts.insert(
            Fact::ObjectiveLevel,
            Value::Integer(i64::try_from(level).unwrap_or(i64::MAX)),
        );
        self
    }
    /// The selected objective level, if any.
    /// # Errors
    /// A negative or non-integer level.
    pub fn objective_level(&self) -> Result<Option<usize>> {
        objective_level(&self.facts)
    }
    /// Read and validate the selected mode, defaulting to steady analysis.
    pub fn analysis_route(&self) -> Result<Route> {
        route(&facts(&self.facts)?)
    }
}
/// The route of validated facts ([`facts`] always records one).
pub(crate) fn route(facts: &Environment) -> Result<Route> {
    let Some(Value::Enum { member, .. }) = facts.get(&Fact::Route.path()) else {
        return Err(invalid(SemanticId::NIL, "analysis route missing"));
    };
    route_of(*member).ok_or_else(|| invalid(SemanticId::NIL, "analysis route missing"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ADR-0123 Outcome 1: a fact is a member of a registry namespace. Paths are read by
    /// the enumeration, an undeclared member names no fact, and a mode binds only stages.
    #[test]
    fn analysis_facts_are_typed_namespaces() {
        assert_eq!(
            FactNamespace::ALL.map(FactNamespace::as_str),
            ["analysis", "objective", "stage"]
        );
        for fact in [
            Fact::Route,
            Fact::Dynamic,
            Fact::ObjectiveLevel,
            Fact::Stage("coast".into()),
        ] {
            assert_eq!(Fact::from_path(&fact.path()), Some(fact.clone()));
            assert_eq!(Fact::new(fact.namespace(), fact.member()), Some(fact.clone()));
        }
        assert_eq!(Fact::Route.path(), "analysis.route");
        assert_eq!(Fact::ObjectiveLevel.path(), "objective.level");
        for undeclared in [
            "analysis.speed",
            "objective.weight",
            "stage.",
            "phase.liquid",
            "route",
        ] {
            assert_eq!(Fact::from_path(undeclared), None, "{undeclared}");
        }
        assert_eq!(Fact::namespace_of("stage.x.y"), Some(FactNamespace::Stage));
        assert_eq!(Fact::namespace_of("scope.x"), None);
        assert!(Fact::Stage("s".into()).is_structural());
        assert!(!Fact::Route.is_structural() && !Fact::ObjectiveLevel.is_structural());
        // A route constant is read through the same namespace, typed by the route enum.
        assert_eq!(
            constant("analysis.integrated"),
            Some(value(Route::Integrated))
        );
        assert_eq!(type_of("analysis.dynamic"), Some(Type::Boolean));
        assert_eq!(type_of("stage.integrated"), None);
        // The ambient facts derive the dynamic flag and carry the selected stages only.
        let bindings = Bindings::default().with_analysis(Route::Integrated);
        let mut input = bindings.facts.clone();
        input.insert(Fact::Stage("coast".into()), Value::Boolean(true));
        input.insert(Fact::ObjectiveLevel, Value::Integer(1));
        let ambient = facts(&input).unwrap();
        assert_eq!(
            ambient.keys().map(String::as_str).collect::<Vec<_>>(),
            ["analysis.dynamic", "analysis.route", "stage.coast"]
        );
        assert_eq!(objective_level(&input).unwrap(), Some(1));
        input.insert(Fact::Dynamic, Value::Boolean(false));
        assert!(facts(&input).is_err());
        input.insert(Fact::Route, Value::Boolean(true));
        assert!(facts(&input).is_err());
    }
}
