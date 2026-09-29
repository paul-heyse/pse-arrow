// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Definition admission and effective interface membership precede instantiation.
use crate::{Declaration, Result, Selected, Type, TypeContext, invalid};
use petgraph::{
    algo::{kosaraju_scc, toposort},
    graph::DiGraph,
};
use pse_authoring::dsl;
use pse_ids::SemanticId;
use pse_model::generated::identities::DeclarationId;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Resolved function contract and its declaration-owned body.
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    /// Generic finite reduction generated from a checked lexical reduction.
    pub reduction: Option<FiniteReduction>,
    /// Strict authored domain predicate over explicit arguments, checked before any function evaluation.
    pub validity: Option<dsl::Predicate>,
    /// A checked foreign implementation; factories are supplied only by the runtime.
    pub external: Option<crate::external::External>,
    /// Claimed boundary agreement, proved by the mathematical consumer before derivative admission.
    pub continuity: Option<u8>,
    /// Declaration identity.
    pub id: DeclarationId,
    /// Universally quantified physical types.
    pub variables: BTreeSet<String>,
    /// Ordered explicit arguments.
    pub arguments: Vec<(String, Type)>,
    /// Declared result.
    pub result: Type,
    /// A default-free interface requirement has no body.
    pub body: Option<dsl::Expr>,
}
/// The retained physical meaning of a finite reduction, independent of its cardinality.
#[derive(Clone, Debug, PartialEq)]
pub struct FiniteReduction {
    /// Scalar mathematical reduction, independent of scientific meaning.
    pub kind: pse_quantity::ReductionKind,
    /// Source set's entity kind, retained after coordinate enumeration.
    pub domain: Option<pse_quantity::EntityKindId>,
    /// Independently checked element contract, including for an empty set.
    pub prototype: pse_quantity::QuantityTypeId,
}
/// Checked package inventory. Every map is keyed by semantic identity, never backend handles.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckedPackage {
    pub(crate) quantities: Arc<pse_quantity::QuantityRegistry>,
    pub(crate) preconditions: Arc<pse_quantity::PhysicalPreconditions>,
    pub(crate) quantity_names: Arc<BTreeMap<String, pse_quantity::QuantityTypeId>>,
    pub(crate) lowered_functions: BTreeSet<DeclarationId>,
    /// Registry-generated authoritative declarations.
    pub(crate) declarations: BTreeMap<DeclarationId, Declaration>,
    /// Qualified names for public lookup.
    pub(crate) names: BTreeMap<String, DeclarationId>,
    /// Lexical child declarations.
    pub(crate) children: BTreeMap<DeclarationId, Vec<DeclarationId>>,
    /// Checked member types.
    pub(crate) types: BTreeMap<DeclarationId, Type>,
    /// Checked function signatures/bodies.
    pub(crate) functions: BTreeMap<DeclarationId, Function>,
    /// Effective members, including inherited interface defaults and overrides.
    pub(crate) members: BTreeMap<DeclarationId, BTreeMap<String, DeclarationId>>,
    /// Admitted immutable table values, indexed by complete typed keys.
    pub(crate) tables: BTreeMap<DeclarationId, crate::data::Table>,
    /// Full interface extension/implementation closure.
    pub(crate) interfaces: BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
}
impl CheckedPackage {
    /// The immutable physical environment used to admit this package.
    pub fn context(&self) -> TypeContext<'_> {
        TypeContext {
            quantities: &self.quantities,
            preconditions: &self.preconditions,
            names: &self.quantity_names,
        }
    }
    /// Look up an entry point in the admitted inventory. Expression visibility uses `resolve`.
    pub fn entry(&self, qualified_name: &str) -> Option<DeclarationId> {
        self.names.get(qualified_name).copied()
    }
    /// Read the admitted declarations without exposing mutation of checked state.
    pub fn declarations(&self) -> impl Iterator<Item = &Declaration> {
        self.declarations.values()
    }
    /// The admitted declaration with this identity.
    pub fn declaration(&self, id: DeclarationId) -> Option<&Declaration> {
        self.declarations.get(&id)
    }
    fn qualified_name(&self, mut id: DeclarationId) -> Option<String> {
        let mut parts = Vec::new();
        loop {
            let row = self.declarations.get(&id)?;
            parts.push(row.name.as_str());
            match row.parent_id {
                Some(parent) => id = parent,
                None => break,
            }
        }
        parts.reverse();
        Some(parts.join("."))
    }
    pub(crate) fn preset_definition(&self, mut id: DeclarationId) -> Result<DeclarationId> {
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert(id) {
                return Err(invalid(id, "recursive preset bindings"));
            }
            let row = &self.declarations[&id];
            if row.value.kind != pse_model::generated::enums::ModelingDeclarationKind::Preset {
                if matches!(
                    row.value.kind,
                    pse_model::generated::enums::ModelingDeclarationKind::Definition
                        | pse_model::generated::enums::ModelingDeclarationKind::Case
                        | pse_model::generated::enums::ModelingDeclarationKind::Test
                ) {
                    return Ok(id);
                }
                return Err(invalid(id, "preset target must be a definition"));
            }
            let source = row
                .value
                .binding
                .as_ref()
                .and_then(|b| b.expression.as_deref())
                .ok_or_else(|| invalid(id, "preset target missing"))?;
            let name = match pse_authoring::language::parse_static(source)
                .map_err(|e| invalid(id, e.to_string()))?
            {
                pse_authoring::language::StaticValue::Apply { name, .. } => name,
                pse_authoring::language::StaticValue::Expression(dsl::Expr {
                    kind: dsl::ExprKind::NamedCall { name, .. },
                    ..
                }) => name,
                _ => return Err(invalid(id, "preset requires a definition application")),
            };
            id = self
                .resolve(id, &name)
                .ok_or_else(|| invalid(id, "preset definition absent"))?;
        }
    }
    /// A unique source member, including guarded contracts without activating them.
    pub(crate) fn declared_member(
        &self,
        owner: DeclarationId,
        name: &str,
    ) -> Option<DeclarationId> {
        if let Some(member) = self.members.get(&owner).and_then(|m| m.get(name)) {
            return Some(*member);
        }
        // Guarded declarations have a source contract even before their guard is
        // selected. Keep them out of the effective map: specialization alone owns
        // activation. Multiple possible declarations require an unambiguous contract.
        fn visit(
            p: &CheckedPackage,
            id: DeclarationId,
            name: &str,
            found: &mut BTreeSet<DeclarationId>,
        ) {
            if p.declarations[&id].value.kind
                != pse_model::generated::enums::ModelingDeclarationKind::When
            {
                return;
            }
            for child in p.children.get(&id).into_iter().flatten() {
                if p.declarations[child].name == name {
                    found.insert(*child);
                }
                visit(p, *child, name, found);
            }
        }
        let mut found = BTreeSet::new();
        for member in self
            .members
            .get(&owner)
            .into_iter()
            .flat_map(|m| m.values())
        {
            visit(self, *member, name, &mut found);
        }
        if found.len() == 1 {
            found.first().copied()
        } else {
            None
        }
    }

    /// Resolve lexical names, explicit imports, and names within the owning package.
    pub fn resolve(&self, mut owner: DeclarationId, name: &str) -> Option<DeclarationId> {
        if let Some(id) = name
            .strip_prefix("f_")
            .and_then(|hex| SemanticId::parse_hex(hex).ok())
            .map(DeclarationId::from)
            .filter(|id| self.lowered_functions.contains(id))
        {
            return Some(id);
        }
        loop {
            for child in self.children.get(&owner).into_iter().flatten() {
                let row = &self.declarations[child];
                if let Some(import) = &row.value.import {
                    let alias = import.alias.as_deref().unwrap_or(&row.name);
                    if name == alias {
                        return self.names.get(&row.name).copied();
                    }
                    if let Some(tail) = name.strip_prefix(alias).and_then(|s| s.strip_prefix('.'))
                        && let Some(id) = self.names.get(&format!("{}.{tail}", row.name))
                    {
                        return Some(*id);
                    }
                }
            }
            if let Some(id) = self.declared_member(owner, name) {
                return Some(id);
            }
            if let Some(id) = self
                .qualified_name(owner)
                .and_then(|scope| self.names.get(&format!("{scope}.{name}")))
            {
                return Some(*id);
            }
            if let Some(id) = self
                .children
                .get(&owner)
                .and_then(|v| v.iter().find(|id| self.declarations[id].name == name))
            {
                return Some(*id);
            }
            let Some(parent) = self.declarations.get(&owner).and_then(|r| r.parent_id) else {
                break;
            };
            owner = parent;
        }
        // Absolute spelling is available within the owning package only. Loading
        // another package never grants visibility into it.
        let package = self.declarations.get(&owner)?;
        if name == package.name
            || name
                .strip_prefix(&package.name)
                .is_some_and(|s| s.starts_with('.'))
        {
            self.names.get(name).copied()
        } else {
            None
        }
    }

    pub(crate) fn named_types(&self, owner: DeclarationId) -> BTreeMap<String, Type> {
        let mut names = self
            .names
            .iter()
            .filter(|(name, id)| self.resolve(owner, name) == Some(**id))
            .filter_map(|(name, id)| self.types.get(id).map(|ty| (name.clone(), ty.clone())))
            .collect::<BTreeMap<_, _>>();
        let mut chain = Vec::new();
        let mut node = Some(owner);
        while let Some(id) = node {
            chain.push(id);
            node = self.declarations[&id].parent_id;
        }
        if let Some(package) = chain.last().and_then(|id| self.declarations.get(id)) {
            let prefix = format!("{}.", package.name);
            for (name, quantity) in self.quantity_names.iter() {
                if let Some(local) = name.strip_prefix(&prefix)
                    && !local.contains('.')
                {
                    let ty = Type::Quantity(pse_quantity::scheme::Scheme::Concrete(*quantity));
                    names.insert(local.into(), ty.clone());
                    names.insert(name.clone(), ty);
                }
            }
        }
        for owner in chain.into_iter().rev() {
            for id in self.children.get(&owner).into_iter().flatten() {
                let row = &self.declarations[id];
                if let Some(import) = &row.value.import {
                    let alias = import.alias.as_deref().unwrap_or(&row.name);
                    let prefix = format!("{}.", row.name);
                    for (name, quantity) in self.quantity_names.iter() {
                        if let Some(tail) = name.strip_prefix(&prefix)
                            && !tail.contains('.')
                        {
                            names.insert(
                                format!("{alias}.{tail}"),
                                Type::Quantity(pse_quantity::scheme::Scheme::Concrete(*quantity)),
                            );
                        }
                    }
                    for (name, id) in &self.names {
                        if let Some(tail) = name.strip_prefix(&prefix)
                            && let Some(ty) = self.types.get(id)
                        {
                            names.insert(format!("{alias}.{tail}"), ty.clone());
                        }
                    }
                }
            }
            for (name, id) in self.members.get(&owner).into_iter().flatten() {
                if let Some(ty) = self.types.get(id) {
                    names.insert(name.clone(), ty.clone());
                }
            }
            for id in self.children.get(&owner).into_iter().flatten() {
                if let Some(ty) = self.types.get(id) {
                    names.insert(self.declarations[id].name.clone(), ty.clone());
                }
            }
        }
        names
    }
}
/// Validate declarations and definitions independently of root instantiation.
/// # Errors
/// Malformed IR, names/types/defaults, recursive ownership/extensions, or invalid expressions.
pub fn check(rows: &[Declaration], context: &TypeContext<'_>) -> Result<CheckedPackage> {
    check_declarations(rows, context).map_err(|error| error.located(rows))
}
fn check_declarations(rows: &[Declaration], context: &TypeContext<'_>) -> Result<CheckedPackage> {
    let mut p = CheckedPackage {
        quantities: Arc::new(context.quantities.clone()),
        preconditions: Arc::new(context.preconditions.clone()),
        quantity_names: Arc::new(context.names.clone()),
        lowered_functions: BTreeSet::new(),
        declarations: BTreeMap::new(),
        names: BTreeMap::new(),
        children: BTreeMap::new(),
        types: BTreeMap::new(),
        functions: BTreeMap::new(),
        members: BTreeMap::new(),
        tables: BTreeMap::new(),
        interfaces: BTreeMap::new(),
    };
    for row in rows {
        if row.declaration_id.as_id() == SemanticId::NIL {
            return Err(invalid(row.declaration_id, "nil declaration identity"));
        }
        row.value
            .selected()
            .map_err(|e| invalid(row.declaration_id, e.to_string()))?;
        if p.declarations
            .insert(row.declaration_id, row.clone())
            .is_some()
        {
            return Err(invalid(row.declaration_id, "duplicate identity"));
        }
    }
    let mut graph = DiGraph::<DeclarationId, ()>::new();
    let nodes = p
        .declarations
        .keys()
        .map(|id| (*id, graph.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    let mut siblings = BTreeSet::new();
    for row in rows {
        {
            // ADR-0104: constraint forms belong to a definition's own rows (possibly guarded),
            // never to an implicit residual, a regime, a stage overlay or an alternative.
            use pse_model::generated::enums::ModelingDeclarationKind as Kind;
            let form = matches!(
                row.value.kind,
                Kind::Sos1
                    | Kind::Sos2
                    | Kind::Atmost
                    | Kind::Atleast
                    | Kind::Exactly
                    | Kind::Piecewise
                    | Kind::Logic
                    | Kind::Complementarity
            ) || row
                .value
                .equation
                .as_ref()
                .is_some_and(|e| e.condition.is_some());
            let mut owner = row.parent_id.and_then(|id| p.declarations.get(&id));
            while let Some(guard) = owner.filter(|r| r.value.kind == Kind::When) {
                owner = guard.parent_id.and_then(|id| p.declarations.get(&id));
            }
            let owner = owner.map(|r| r.value.kind);
            if form && !matches!(owner, Some(Kind::Definition | Kind::Test | Kind::Case)) {
                return Err(invalid(
                    row.declaration_id,
                    "constraint forms belong to definitions, tests and cases",
                ));
            }
        }
        if let Some(scope) = &row.value.scope {
            use pse_model::generated::enums::ModelingDeclarationKind as Kind;
            if scope.selection.is_some() && row.value.kind != Kind::Implicit
                || scope.eligibility.is_some() && row.value.kind != Kind::Regime
            {
                return Err(invalid(
                    row.declaration_id,
                    "implicit selection or regime eligibility owner",
                ));
            }
            let parent_kind = row
                .parent_id
                .and_then(|id| p.declarations.get(&id))
                .map(|r| r.value.kind);
            if row.value.kind == Kind::Alternative
                && (parent_kind != Some(Kind::Disjunction)
                    || !scope.parameters.is_empty()
                    || !scope.bases.is_empty()
                    || !scope.type_parameters.is_empty())
                || row.value.kind == Kind::Disjunction
                    && (!matches!(
                        parent_kind,
                        Some(Kind::Definition | Kind::Test | Kind::Case | Kind::Alternative)
                    ) || !scope.parameters.is_empty()
                        || !scope.bases.is_empty()
                        || !scope.type_parameters.is_empty())
            {
                return Err(invalid(
                    row.declaration_id,
                    "disjunctions belong to definitions and own their alternatives",
                ));
            }
            if row.value.kind == Kind::Regime
                && (row.parent_id.is_none_or(|id| {
                    p.declarations
                        .get(&id)
                        .is_none_or(|row| row.value.kind != Kind::Implicit)
                }) || !scope.parameters.is_empty()
                    || !scope.bases.is_empty()
                    || !scope.type_parameters.is_empty())
            {
                return Err(invalid(
                    row.declaration_id,
                    "regimes are alternatives over their implicit owner's shared unknowns",
                ));
            }
            if (scope.fixture.is_some() || scope.oracle.is_some())
                && !matches!(
                    row.value.kind,
                    pse_model::generated::enums::ModelingDeclarationKind::Test
                        | pse_model::generated::enums::ModelingDeclarationKind::Case
                )
            {
                return Err(invalid(
                    row.declaration_id,
                    "fixture/oracle metadata requires a test or case",
                ));
            }
            if let Some(fixture) = &scope.fixture {
                use pse_model::generated::enums::ModelingFixtureExecution as Execution;
                let execution = fixture.execution.unwrap_or(Execution::Steady);
                if (execution == Execution::Integrated) != fixture.integration.is_some()
                    || execution != Execution::Initialized
                        && (!fixture.stages.is_empty() || fixture.initialization.is_some())
                    || fixture.stages.iter().any(|s| s.is_empty())
                    || fixture
                        .expected_failure
                        .as_ref()
                        .is_some_and(|e| e.rule.trim().is_empty())
                {
                    return Err(invalid(
                        row.declaration_id,
                        "fixture execution metadata disagrees with its route",
                    ));
                }
                // ADR-0119 Outcome 3: modes and events belong to an integrated fixture; a
                // mode's facts select `when` variants and stages, never the analysis route
                // or an objective level; a terminal event neither resets nor changes mode.
                let mode_names = fixture
                    .modes
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<BTreeSet<_>>();
                if !fixture.modes.is_empty() && execution != Execution::Integrated
                    || mode_names.len() != fixture.modes.len()
                    || mode_names.contains("")
                    || fixture.modes.iter().any(|m| {
                        m.facts
                            .iter()
                            .map(|f| f.name.as_str())
                            .collect::<BTreeSet<_>>()
                            .len()
                            != m.facts.len()
                            || m.facts.iter().any(|f| {
                                f.name.starts_with("analysis.") || f.name.starts_with("objective.")
                            })
                            || m.events.iter().any(|e| match &e.next {
                                Some(next) => !mode_names.contains(next.as_str()),
                                None => !e.reset.is_empty(),
                            })
                    })
                {
                    return Err(invalid(
                        row.declaration_id,
                        "fixture modes need the integrated route, unique names and structural facts; each event names a declared successor or is terminal without resets",
                    ));
                }
                for expression in fixture.modes.iter().flat_map(|m| &m.events).flat_map(|e| {
                    [&e.guard, &e.tolerance]
                        .into_iter()
                        .chain(e.reset.iter().flat_map(|r| [&r.target, &r.expression]))
                }) {
                    dsl::parse_expr(expression)
                        .map_err(|e| invalid(row.declaration_id, e.to_string()))?;
                }
                if let Some(policy) = &fixture.initialization
                    && (!policy.initial_step.is_finite()
                        || policy.initial_step <= 0.
                        || policy.initial_step > 1.
                        || !policy.minimum_step.is_finite()
                        || policy.minimum_step <= 0.
                        || policy.minimum_step > policy.initial_step
                        || !policy.growth.is_finite()
                        || policy.growth <= 1.
                        || policy.maximum_attempts <= 0
                        || policy.maximum_attempts > 100_000
                        || !policy.time_limit_seconds.is_finite()
                        || policy.time_limit_seconds <= 0.)
                {
                    return Err(invalid(
                        row.declaration_id,
                        "invalid bounded initialization fixture policy",
                    ));
                }
                if let Some(integration) = &fixture.integration {
                    if integration.quadrature_relative_tolerance.is_some()
                        != !integration.quadratures.is_empty()
                        || integration
                            .quadrature_relative_tolerance
                            .is_some_and(|v| !v.is_finite() || v <= 0.)
                        || integration.samples.is_empty()
                        || !integration.relative_tolerance.is_finite()
                        || integration.relative_tolerance <= 0.
                        || !integration.normalized_absolute_tolerance.is_finite()
                        || integration.normalized_absolute_tolerance <= 0.
                    {
                        return Err(invalid(
                            row.declaration_id,
                            "integration fixture requires samples and positive finite tolerances",
                        ));
                    }
                    // ADR-0119 Outcome 2: one value per interval of each scheduled input.
                    if integration
                        .schedules
                        .iter()
                        .any(|s| s.times.is_empty() || s.values.len() != s.times.len() + 1)
                    {
                        return Err(invalid(
                            row.declaration_id,
                            "a scheduled input declares its change times and one value per interval",
                        ));
                    }
                    for expression in integration
                        .samples
                        .iter()
                        .chain(std::iter::once(&integration.initial_step))
                        .chain(
                            integration
                                .quadratures
                                .iter()
                                .map(|q| &q.absolute_tolerance),
                        )
                        .chain(
                            integration
                                .schedules
                                .iter()
                                .flat_map(|s| s.times.iter().chain(&s.values)),
                        )
                    {
                        dsl::parse_expr(expression)
                            .map_err(|e| invalid(row.declaration_id, e.to_string()))?;
                    }
                }
            }
            if scope.oracle.as_ref().is_some_and(|o| {
                o.reference.trim().is_empty()
                    || o.revision.trim().is_empty()
                    || o.reference.len() > 1024
                    || o.revision.len() > 1024
                    || o.reference.starts_with('/')
                    || o.reference.contains("skill://")
            }) {
                return Err(invalid(
                    row.declaration_id,
                    "oracle requires a portable reference and immutable source revision",
                ));
            }
        }
        if !siblings.insert((row.parent_id, row.name.clone())) {
            return Err(invalid(
                row.declaration_id,
                format!("duplicate member {}", row.name),
            ));
        }
        if let Some(parent) = row.parent_id {
            let Some(node) = nodes.get(&parent) else {
                return Err(invalid(row.declaration_id, "absent parent"));
            };
            graph.add_edge(*node, nodes[&row.declaration_id], ());
            p.children
                .entry(parent)
                .or_default()
                .push(row.declaration_id);
        }
    }
    let order = toposort(&graph, None)
        .map_err(|c| invalid(graph[c.node_id()], "recursive declaration ownership"))?;
    for values in p.children.values_mut() {
        values.sort_by_key(|id| (p.declarations[id].ordinal, *id));
    }
    let mut full = BTreeMap::<DeclarationId, String>::new();
    for node in order {
        let id = graph[node];
        let row = &p.declarations[&id];
        let name = row.parent_id.map_or_else(
            || row.name.clone(),
            |parent| format!("{}.{}", full[&parent], row.name),
        );
        p.names.insert(name.clone(), id);
        full.insert(id, name);
        let ty = match row
            .value
            .selected()
            .map_err(|e| invalid(id, e.to_string()))?
        {
            Selected::EntityKind(_) => Some(Type::Entity(id)),
            Selected::Enum(_) => Some(Type::Enum(id)),
            Selected::Table(_) => Some(Type::Table(id)),
            Selected::Definition(_)
            | Selected::Case(_)
            | Selected::Test(_)
            | Selected::Implicit(_)
            | Selected::Set(_)
            | Selected::Preset(_) => Some(Type::Definition(id)),
            Selected::Interface(_) => Some(Type::Interface(id)),
            _ => None,
        };
        if let Some(ty) = ty {
            p.types.insert(id, ty);
        }
    }
    for (name, quantity) in context.names {
        context
            .quantities
            .quantity_type(*quantity)
            .map_err(|e| invalid(SemanticId::NIL, e.to_string()))?;
        if p.names.contains_key(name) {
            return Err(invalid(
                p.names[name],
                "physical alias conflicts with a declaration",
            ));
        }
    }
    for row in rows {
        if let Some(import) = &row.value.import {
            let target = p.names.get(&row.name).copied().ok_or_else(|| {
                invalid(
                    row.declaration_id,
                    format!("imported package {} is absent", row.name),
                )
            })?;
            if p.declarations[&target].value.kind
                != pse_model::generated::enums::ModelingDeclarationKind::Package
            {
                return Err(invalid(
                    row.declaration_id,
                    "import target is not a package",
                ));
            }
            let alias = import.alias.as_deref().unwrap_or(&row.name);
            if p.children
                .get(&row.parent_id.unwrap_or(row.declaration_id))
                .into_iter()
                .flatten()
                .any(|id| {
                    *id != row.declaration_id && {
                        let other = &p.declarations[id];
                        other.name == alias
                            || other.value.import.as_ref().and_then(|v| v.alias.as_deref())
                                == Some(alias)
                    }
                })
            {
                return Err(invalid(
                    row.declaration_id,
                    "import alias conflicts with a local name",
                ));
            }
        }
    }
    // Coordinate types are needed even when an indexed declaration precedes its axis.
    for row in rows {
        if let Some(v) = &row.value.continuous {
            let ty = context.resolve(
                &v.type_name,
                &BTreeSet::new(),
                &p.named_types(row.declaration_id),
                row.declaration_id,
            )?;
            if !matches!(ty, Type::Quantity(_)) {
                return Err(invalid(
                    row.declaration_id,
                    "continuous axis requires a physical type",
                ));
            }
            p.types.insert(
                row.declaration_id,
                Type::Continuous(row.declaration_id, Box::new(ty)),
            );
        }
    }
    // Interface and definition dependencies are explicit graphs, never Salsa recovery.
    let mut inheritance = DiGraph::<DeclarationId, ()>::new();
    let inode = p
        .declarations
        .keys()
        .map(|id| (*id, inheritance.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    for row in rows {
        if let Some(scope) = &row.value.scope {
            for name in &scope.bases {
                let base = p.resolve(row.declaration_id, name).ok_or_else(|| {
                    invalid(row.declaration_id, format!("unknown interface {name}"))
                })?;
                if !matches!(p.types.get(&base), Some(Type::Interface(_))) {
                    return Err(invalid(row.declaration_id, "base must be an interface"));
                }
                inheritance.add_edge(inode[&base], inode[&row.declaration_id], ());
            }
        }
    }
    if let Some(cycle) = kosaraju_scc(&inheritance)
        .into_iter()
        .find(|c| c.len() > 1 || c.first().is_some_and(|n| inheritance.contains_edge(*n, *n)))
    {
        return Err(invalid(
            inheritance[cycle[0]],
            format!(
                "recursive interface extension: {}",
                cycle
                    .iter()
                    .map(|n| p.declarations[&inheritance[*n]].name.clone())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ),
        ));
    }
    for row in rows {
        let id = row.declaration_id;
        let mut names = p.named_types(id);
        let mut variables = BTreeSet::new();
        let mut owner = Some(id);
        while let Some(at) = owner {
            if let Some(scope) = &p.declarations[&at].value.scope {
                variables.extend(scope.type_parameters.iter().cloned());
                for arg in &scope.parameters {
                    let ty = context.resolve(&arg.type_name, &variables, &names, at)?;
                    names.insert(arg.name.clone(), ty);
                }
            }
            owner = p.declarations[&at].parent_id;
        }
        if row.value.kind == pse_model::generated::enums::ModelingDeclarationKind::Alternative {
            // An alternative is referenced by its binary indicator (ADR-0104).
            p.types
                .insert(id, crate::indicator_type(context.quantities, id)?);
        }
        if let Some(v) = &row.value.binding {
            if !v.type_name.is_empty() {
                p.types
                    .insert(id, context.resolve(&v.type_name, &variables, &names, id)?);
            }
            // ADR-0103: exactly a variable declares a domain, and a discrete one is physical.
            let variable =
                row.value.kind == pse_model::generated::enums::ModelingDeclarationKind::Variable;
            match v.domain {
                None if variable => return Err(invalid(id, "variable declares no domain")),
                Some(_) if !variable => {
                    return Err(invalid(id, "only a variable declares a domain"));
                }
                Some(domain)
                    if domain.is_discrete()
                        && !matches!(p.types.get(&id), Some(Type::Quantity(_))) =>
                {
                    return Err(invalid(
                        id,
                        "a discrete domain requires a physical quantity type",
                    ));
                }
                _ => {}
            }
        }
        if let Some(v) = &row.value.accumulator {
            p.types
                .insert(id, context.resolve(&v.type_name, &variables, &names, id)?);
        }
        if let Some(v) = &row.value.continuous {
            let ty = context.resolve(&v.type_name, &variables, &names, id)?;
            if !matches!(ty, Type::Quantity(_)) {
                return Err(invalid(id, "continuous axis requires a physical type"));
            }
            p.types.insert(id, Type::Continuous(id, Box::new(ty)));
        }
        if let Some(v) = &row.value.entity {
            let kind = p
                .resolve(id, &v.kind_name)
                .ok_or_else(|| invalid(id, "unknown entity kind"))?;
            if !matches!(p.types.get(&kind), Some(Type::Entity(_))) {
                return Err(invalid(id, "entity requires a declared kind"));
            }
            p.types.insert(id, Type::Entity(kind));
        }
        if let Some(v) = &row.value.function {
            variables.extend(v.type_parameters.iter().cloned());
            let mut argument_names = BTreeSet::new();
            let arguments: Vec<(String, Type)> = v
                .arguments
                .iter()
                .map(|a| {
                    if !argument_names.insert(a.name.clone()) {
                        return Err(invalid(id, "duplicate formal argument"));
                    }
                    Ok((
                        a.name.clone(),
                        context.resolve(&a.type_name, &variables, &names, id)?,
                    ))
                })
                .collect::<Result<_>>()?;
            let result = context.resolve(&v.return_type, &variables, &names, id)?;
            let body = v
                .body
                .as_ref()
                .map(|b| dsl::parse_expr(b).map_err(|e| invalid(id, e.to_string())))
                .transpose()?;
            p.types.insert(
                id,
                Type::Function {
                    arguments: arguments.clone(),
                    result: Box::new(result.clone()),
                },
            );
            p.functions.insert(
                id,
                Function {
                    reduction: None,
                    validity: v
                        .validity
                        .as_ref()
                        .map(|p| dsl::parse_predicate(p).map_err(|e| invalid(id, e.to_string())))
                        .transpose()?,
                    external: v
                        .external
                        .as_ref()
                        .map(|v| crate::external::External::check(v, id))
                        .transpose()?,
                    continuity: v
                        .continuity
                        .map(|v| {
                            u8::try_from(v)
                                .ok()
                                .filter(|v| *v <= 2)
                                .ok_or_else(|| invalid(id, "piecewise derivative order 0..=2"))
                        })
                        .transpose()?,
                    id,
                    variables: variables.clone(),
                    arguments,
                    result,
                    body,
                },
            );
        }
        if let Some(values) = &row.value.enumeration
            && (values.members.is_empty()
                || values.members.iter().any(String::is_empty)
                || values.members.iter().collect::<BTreeSet<_>>().len() != values.members.len())
        {
            return Err(invalid(id, "enumeration requires distinct named members"));
        }
        if let Some(table) = &row.value.table {
            if !matches!(
                table.missing_policy.as_str(),
                "required" | "optional" | "default"
            ) || (table.missing_policy == "default") != table.default_value.is_some()
            {
                return Err(invalid(id, "invalid table absence policy"));
            }
            if table.columns.is_empty() {
                context.resolve(&table.value_type, &variables, &names, id)?;
            }
            for column in &table.columns {
                context.resolve(&column.type_name, &variables, &names, id)?;
            }
            for key in &table.keys {
                context.resolve(&key.type_name, &variables, &names, id)?;
            }
        }
    }
    for node in toposort(&inheritance, None)
        .map_err(|c| invalid(inheritance[c.node_id()], "interface cycle"))?
    {
        let id = inheritance[node];
        let row = &p.declarations[&id];
        let Some(scope) = &row.value.scope else {
            continue;
        };
        let own = p
            .children
            .get(&id)
            .into_iter()
            .flatten()
            .map(|child| (p.declarations[child].name.clone(), *child))
            .collect::<BTreeMap<_, _>>();
        let mut effective = BTreeMap::new();
        let mut contracts = BTreeSet::new();
        for base in &scope.bases {
            let base = p
                .resolve(id, base)
                .ok_or_else(|| invalid(id, "unknown interface"))?;
            contracts.insert(base);
            contracts.extend(p.interfaces.get(&base).into_iter().flatten().copied());
            for (name, member) in p.members.get(&base).into_iter().flatten() {
                if let Some(previous) = effective.insert(name.clone(), *member)
                    && previous != *member
                    && !own
                        .get(name)
                        .is_some_and(|id| p.declarations[id].is_override)
                {
                    // A diamond may expose both an ancestor declaration and its
                    // already checked refinement. Keep the more specific owner,
                    // independently of the order in which bases are listed.
                    let previous_owner = p.declarations[&previous].parent_id;
                    let member_owner = p.declarations[member].parent_id;
                    if let (Some(a), Some(b)) = (
                        previous_owner.and_then(|owner| inode.get(&owner)),
                        member_owner.and_then(|owner| inode.get(&owner)),
                    ) {
                        if a != b && petgraph::algo::has_path_connecting(&inheritance, *a, *b, None)
                        {
                            continue;
                        }
                        if a != b && petgraph::algo::has_path_connecting(&inheritance, *b, *a, None)
                        {
                            effective.insert(name.clone(), previous);
                            continue;
                        }
                    }
                    return Err(invalid(
                        id,
                        format!("competing defaults for {name}; explicit override required"),
                    ));
                }
            }
        }
        let mut projected = effective.clone();
        projected.extend(own.clone());
        // Resolve inherited sibling names against the complete candidate scope;
        // validation must not depend on the lexical order of member names.
        p.members.insert(id, projected.clone());
        let inherited = effective.clone();
        for (name, member) in own {
            if let Some(base) = effective.get(&name).copied() {
                let base_row = &p.declarations[&base];
                let has_default = base_row
                    .value
                    .function
                    .as_ref()
                    .is_some_and(|f| f.body.is_some() || f.external.is_some())
                    || base_row
                        .value
                        .binding
                        .as_ref()
                        .is_some_and(|b| b.expression.is_some() || b.defined_by.is_some());
                if has_default && !p.declarations[&member].is_override {
                    return Err(invalid(member, "replacing a default requires override"));
                }
                let actual_row = &p.declarations[&member];
                if base_row.value.kind != actual_row.value.kind {
                    return Err(invalid(member, "interface member role differs"));
                }
                if let (Some(expected), Some(actual)) =
                    (&base_row.value.binding, &actual_row.value.binding)
                {
                    if expected.domain != actual.domain {
                        return Err(invalid(member, "interface member domain differs"));
                    }
                    if expected.indices.len() != actual.indices.len() {
                        return Err(invalid(member, "interface member index arity differs"));
                    }
                    for (a, b) in expected.indices.iter().zip(&actual.indices) {
                        let expected_domain = p.resolve(base, &a.domain).map(|target| {
                            inherited
                                .iter()
                                .find(|(_, value)| **value == target)
                                .and_then(|(name, _)| projected.get(name))
                                .copied()
                                .unwrap_or(target)
                        });
                        let actual_domain = p.resolve(member, &b.domain);
                        if expected_domain != actual_domain
                            || (expected_domain.is_none() && a.domain != b.domain)
                        {
                            return Err(invalid(member, "interface member index domains differ"));
                        }
                    }
                }
                // Child instances are constructed once and cannot be replaced through
                // the inherited interface. Their visible contract may therefore be
                // refined to an extending interface or an implementing definition.
                // Input parameters, variable quantities and function signatures remain
                // invariant; narrowing those would reject a formerly valid binding.
                let child_refinement = actual_row.value.kind
                    == pse_model::generated::enums::ModelingDeclarationKind::Child
                    && match (p.types.get(&base), p.types.get(&member)) {
                        (
                            Some(Type::Interface(expected)),
                            Some(Type::Interface(actual) | Type::Definition(actual)),
                        ) => petgraph::algo::has_path_connecting(
                            &inheritance,
                            inode[expected],
                            inode[actual],
                            None,
                        ),
                        _ => false,
                    };
                if p.types.get(&base) != p.types.get(&member) && !child_refinement {
                    return Err(invalid(member, "interface member physical type differs"));
                }
                if let Some(f) = p.functions.get(&base) {
                    let actual = p
                        .functions
                        .get(&member)
                        .ok_or_else(|| invalid(member, "function role required"))?;
                    if f.arguments != actual.arguments || f.result != actual.result {
                        return Err(invalid(member, "interface function signature differs"));
                    }
                }
            } else if p.declarations[&member].is_override {
                if row.value.kind == pse_model::generated::enums::ModelingDeclarationKind::Stage {
                    let parent = row
                        .parent_id
                        .ok_or_else(|| invalid(member, "stage parent absent"))?;
                    let base = p
                        .children
                        .get(&parent)
                        .into_iter()
                        .flatten()
                        .find(|id| p.declarations[id].name == name)
                        .copied()
                        .ok_or_else(|| invalid(member, "stage override target absent"))?;
                    if p.declarations[&base].value.equation.is_none()
                        || p.declarations[&member].value.equation.is_none()
                    {
                        return Err(invalid(
                            member,
                            "stage overrides replace equations; variable identities remain stable",
                        ));
                    }
                } else {
                    return Err(invalid(member, "override has no inherited member"));
                }
            }
            effective.insert(name, member);
        }
        if matches!(
            row.value
                .selected()
                .map_err(|e| invalid(id, e.to_string()))?,
            Selected::Definition(_)
        ) {
            for member in effective.values() {
                let declaration = &p.declarations[member];
                if declaration
                    .value
                    .function
                    .as_ref()
                    .is_some_and(|f| f.body.is_none() && f.external.is_none())
                {
                    return Err(invalid(
                        id,
                        format!("missing implementation {}", declaration.name),
                    ));
                }
            }
        }
        p.interfaces.insert(id, contracts);
        p.members.insert(id, effective);
    }
    let mut calls = DiGraph::<DeclarationId, ()>::new();
    let function_nodes = p
        .functions
        .keys()
        .map(|id| (*id, calls.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    for (id, function) in &p.functions {
        let domain = function.validity.as_ref().map(|predicate| dsl::Expr {
            kind: dsl::ExprKind::Conditional {
                guard: Box::new(predicate.clone()),
                then: Box::new(dsl::Expr {
                    kind: dsl::ExprKind::Number(dsl::Number {
                        value: 0.,
                        exact_integer: Some(0),
                        unit: None,
                    }),
                    span: Default::default(),
                }),
                otherwise: Box::new(dsl::Expr {
                    kind: dsl::ExprKind::Number(dsl::Number {
                        value: 0.,
                        exact_integer: Some(0),
                        unit: None,
                    }),
                    span: Default::default(),
                }),
            },
            span: Default::default(),
        });
        for body in function.body.iter().chain(domain.iter()) {
            body.walk(|expr| {
                let name = match &expr.kind {
                    dsl::ExprKind::NamedCall { name, .. } => Some(name),
                    dsl::ExprKind::Partial { function, .. } => Some(function),
                    _ => None,
                };
                if let Some(target) = name
                    .and_then(|name| p.resolve(*id, name))
                    .and_then(|id| function_nodes.get(&id))
                {
                    calls.add_edge(function_nodes[id], *target, ());
                }
            });
        }
    }
    toposort(&calls, None).map_err(|cycle| {
        invalid(
            calls[cycle.node_id()],
            "recursive package function expansion",
        )
    })?;
    for row in rows {
        if row.value.kind == pse_model::generated::enums::ModelingDeclarationKind::Preset {
            let base = p.preset_definition(row.declaration_id)?;
            if let Some(interfaces) = p.interfaces.get(&base).cloned() {
                p.interfaces.insert(row.declaration_id, interfaces);
            }
        }
    }
    crate::data::admit(&mut p, context)?;
    crate::expression::check_all(&p, context)?;
    Ok(p)
}

impl CheckedPackage {
    /// Project one root's declaration dependencies after whole-package admission.
    /// Equality of this projection backdates unrelated edits in the compiler workspace.
    /// # Errors
    /// Root identity is absent.
    pub fn select(&self, root: DeclarationId) -> Result<Self> {
        if !self.declarations.contains_key(&root) {
            return Err(invalid(root, "selected root absent"));
        }
        let mut selected = BTreeSet::new();
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            if !selected.insert(id) {
                continue;
            }
            let row = &self.declarations[&id];
            if let Some(parent) = row.parent_id {
                pending.push(parent);
            }
            pending.extend(
                self.children
                    .get(&id)
                    .into_iter()
                    .flatten()
                    .filter(|child| self.declarations[child].value.import.is_some())
                    .copied(),
            );
            if row.value.kind != pse_model::generated::enums::ModelingDeclarationKind::Package {
                pending.extend(self.children.get(&id).into_iter().flatten().copied());
                pending.extend(
                    self.members
                        .get(&id)
                        .into_iter()
                        .flat_map(|m| m.values())
                        .copied(),
                );
            }
            let mut texts = Vec::new();
            if row.value.import.is_some()
                && let Some(target) = self.names.get(&row.name)
            {
                pending.push(*target);
            }
            if let Some(v) = &row.value.scope {
                texts.extend(v.bases.iter().map(String::as_str));
                if let Some(selection) = &v.selection {
                    texts.push(&selection.criterion);
                    texts.push(&selection.tolerance);
                }
                texts.extend(v.eligibility.as_deref());
                if let Some(fixture) = &v.fixture {
                    if let Some(integration) = &fixture.integration {
                        texts.extend(integration.samples.iter().map(String::as_str));
                        texts.push(&integration.initial_step);
                        for q in &integration.quadratures {
                            texts.push(&q.target);
                            texts.push(&q.absolute_tolerance);
                        }
                        for s in &integration.schedules {
                            texts.push(&s.target);
                            texts.extend(s.times.iter().chain(&s.values).map(String::as_str));
                        }
                    }
                    for e in fixture.modes.iter().flat_map(|m| &m.events) {
                        texts.push(&e.guard);
                        texts.push(&e.tolerance);
                        for r in &e.reset {
                            texts.push(&r.target);
                            texts.push(&r.expression);
                        }
                    }
                    for s in &fixture.specifications {
                        texts.push(&s.target);
                        texts.extend(s.expression.as_deref());
                    }
                }
                for p in &v.parameters {
                    texts.push(&p.type_name);
                    texts.extend(p.default_value.as_deref());
                }
            }
            if let Some(v) = &row.value.binding {
                texts.push(&v.type_name);
                texts.extend(v.expression.as_deref());
                texts.extend(v.defined_by.as_deref());
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
            }
            if let Some(v) = &row.value.function {
                if let Some(external) = &v.external {
                    texts.push(&external.output);
                }
                texts.extend(v.body.as_deref());
                texts.extend(v.validity.as_deref());
                texts.push(&v.return_type);
                texts.extend(v.arguments.iter().map(|a| a.type_name.as_str()));
            }
            if let Some(v) = &row.value.table {
                texts.push(&v.value_type);
                texts.extend(v.default_value.as_deref());
                texts.extend(v.keys.iter().map(|k| k.type_name.as_str()));
                for dataset in self.declarations.values() {
                    if dataset
                        .value
                        .dataset
                        .as_ref()
                        .is_some_and(|v| self.resolve(dataset.declaration_id, &v.table) == Some(id))
                    {
                        pending.push(dataset.declaration_id);
                    }
                }
            }
            if let Some(v) = &row.value.dataset {
                texts.push(&v.table);
                for r in &v.rows {
                    texts.extend(r.keys.iter().chain(&r.values).map(String::as_str));
                }
            }
            if let Some(v) = &row.value.entity {
                texts.push(&v.kind_name);
                texts.extend(v.attributes.iter().map(|a| a.expression.as_str()));
            }
            if let Some(v) = &row.value.equation {
                texts.push(&v.expression);
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
                texts.extend(v.condition.as_ref().map(|c| c.variable.as_str()));
            }
            if let Some(v) = &row.value.ordered_set {
                texts.extend([v.member.as_str(), v.weight.as_str()]);
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
            }
            if let Some(v) = &row.value.cardinality {
                texts.extend([v.count.as_str(), v.member.as_str()]);
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
            }
            if let Some(v) = &row.value.piecewise {
                texts.extend([
                    v.output.as_str(),
                    v.input.as_str(),
                    v.abscissa.as_str(),
                    v.ordinate.as_str(),
                ]);
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
            }
            if let Some(v) = &row.value.logic {
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
            }
            if let Some(v) = &row.value.complementarity {
                texts.extend([v.first.as_str(), v.second.as_str()]);
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
            }
            if let Some(v) = &row.value.realization {
                texts.extend(v.argument.as_deref());
                texts.extend(v.function.as_deref());
            }
            if let Some(v) = &row.value.guard {
                texts.push(&v.predicate);
            }
            if let Some(v) = &row.value.requirement {
                texts.push(&v.predicate);
            }
            if let Some(v) = &row.value.accumulator {
                texts.push(&v.type_name);
                texts.push(&v.tolerance);
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
            }
            if let Some(v) = &row.value.contribution {
                texts.push(&v.target);
                texts.push(&v.expression);
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
            }
            if let Some(v) = &row.value.connection {
                texts.push(&v.from);
                texts.push(&v.to);
            }
            if let Some(v) = &row.value.annotation {
                texts.push(&v.target);
                texts.extend(v.arguments.iter().map(String::as_str));
                if let Some(o) = &v.objective {
                    texts.extend(
                        [
                            &o.weight,
                            &o.normalization,
                            &o.absolute_tolerance,
                            &o.relative_tolerance,
                        ]
                        .into_iter()
                        .flatten()
                        .map(String::as_str),
                    );
                }
            }
            if let Some(v) = &row.value.continuous {
                texts.extend([v.type_name.as_str(), v.lower.as_str(), v.upper.as_str()]);
            }
            if let Some(v) = &row.value.discretization {
                texts.extend([
                    v.target.as_str(),
                    v.scheme.as_str(),
                    v.elements.as_str(),
                    v.order.as_str(),
                ]);
            }
            if let Some(v) = &row.value.realization {
                texts.push(&v.target);
            }
            if let Some(v) = &row.value.relaxation {
                texts.extend([v.target.as_str(), v.nominal.as_str()]);
            }
            if let Some(v) = &row.value.continuation {
                texts.extend([v.target.as_str(), v.start.as_str(), v.end.as_str()]);
            }
            if let Some(v) = &row.value.expectation {
                texts.extend([v.actual.as_str(), v.expected.as_str(), v.tolerance.as_str()]);
                texts.extend(v.relative_tolerance.as_deref());
            }
            for text in texts {
                pending.extend(dependency_paths(self, id, text));
            }
            if let Some(ty) = self.types.get(&id) {
                type_dependencies(ty, &mut pending);
            }
            if let Some(f) = self.functions.get(&id) {
                for (_, ty) in &f.arguments {
                    type_dependencies(ty, &mut pending);
                }
                type_dependencies(&f.result, &mut pending);
            }
            if let Some(table) = self.tables.get(&id) {
                for ty in &table.keys {
                    type_dependencies(ty, &mut pending);
                }
                for (_, ty) in &table.columns {
                    type_dependencies(ty, &mut pending);
                }
                type_dependencies(&table.result, &mut pending);
            }
        }
        let mut p = self.clone();
        p.declarations.retain(|id, _| selected.contains(id));
        p.names.retain(|_, id| selected.contains(id));
        p.types.retain(|id, _| selected.contains(id));
        p.functions.retain(|id, _| selected.contains(id));
        p.tables.retain(|id, _| selected.contains(id));
        p.children.retain(|id, _| selected.contains(id));
        for members in p.children.values_mut() {
            members.retain(|id| selected.contains(id));
        }
        p.members.retain(|id, _| selected.contains(id));
        for members in p.members.values_mut() {
            members.retain(|_, id| selected.contains(id));
        }
        p.interfaces.retain(|id, _| selected.contains(id));
        for interfaces in p.interfaces.values_mut() {
            interfaces.retain(|id| selected.contains(id));
        }
        Ok(p)
    }
}

fn dependency_paths(
    p: &CheckedPackage,
    owner: DeclarationId,
    text: &str,
) -> BTreeSet<DeclarationId> {
    use pse_authoring::{
        dsl::{Equation, EquationKind, Expr, ExprKind},
        language::{StaticValue, parse_static},
    };
    fn path(
        p: &CheckedPackage,
        owner: DeclarationId,
        path: &dsl::Path,
        out: &mut BTreeSet<DeclarationId>,
    ) {
        if let Some(id) = (1..=path.segments.len()).rev().find_map(|end| {
            let prefix = path.segments[..end]
                .iter()
                .map(|segment| segment.name.as_str())
                .collect::<Vec<_>>()
                .join(".");
            p.resolve(owner, &prefix)
        }) {
            out.insert(id);
        }
    }
    fn expression(
        p: &CheckedPackage,
        owner: DeclarationId,
        e: &Expr,
        out: &mut BTreeSet<DeclarationId>,
    ) {
        for value in e.free_paths() {
            path(p, owner, value, out);
        }
        e.walk(|node| call(p, owner, node, out));
    }
    fn call(
        p: &CheckedPackage,
        owner: DeclarationId,
        node: &Expr,
        out: &mut BTreeSet<DeclarationId>,
    ) {
        let name = match &node.kind {
            ExprKind::NamedCall { name, .. } => Some(name),
            ExprKind::Partial { function, .. } => Some(function),
            _ => None,
        };
        if let Some(name) = name {
            if let Some(id) = p.resolve(owner, name) {
                out.insert(id);
            }
            if let Ok(callee) = dsl::parse_expr(name) {
                for value in callee.free_paths() {
                    path(p, owner, value, out);
                }
            }
        }
    }
    fn predicate(
        p: &CheckedPackage,
        owner: DeclarationId,
        value: &dsl::Predicate,
        out: &mut BTreeSet<DeclarationId>,
    ) {
        for value in value.paths() {
            path(p, owner, value, out);
        }
        value.walk_expressions(|node| call(p, owner, node, out));
    }
    fn syntax(
        p: &CheckedPackage,
        owner: DeclarationId,
        value: &StaticValue,
        out: &mut BTreeSet<DeclarationId>,
    ) {
        match value {
            StaticValue::Expression(e) => expression(p, owner, e, out),
            StaticValue::Set(values) | StaticValue::Tuple(values) => {
                for v in values {
                    syntax(p, owner, v, out);
                }
            }
            StaticValue::Comprehension {
                body,
                bindings,
                filter,
            } => {
                for (_, v) in bindings {
                    syntax(p, owner, v, out);
                }
                syntax(p, owner, body, out);
                if let Some(filter) = filter {
                    predicate(p, owner, filter, out);
                }
            }
            StaticValue::Apply { name, arguments } => {
                if let Some(id) = p.resolve(owner, name) {
                    out.insert(id);
                }
                for (_, v) in arguments {
                    syntax(p, owner, v, out);
                }
            }
            StaticValue::Text(_) => {}
        }
    }
    fn equation(
        p: &CheckedPackage,
        owner: DeclarationId,
        value: &Equation,
        out: &mut BTreeSet<DeclarationId>,
    ) {
        match &value.kind {
            EquationKind::Relation { lhs, rhs, .. } => {
                expression(p, owner, lhs, out);
                expression(p, owner, rhs, out);
            }
            EquationKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                predicate(p, owner, guard, out);
                equation(p, owner, then, out);
                equation(p, owner, otherwise, out);
            }
        }
    }
    let mut out = BTreeSet::new();
    if let Some(id) = p.resolve(owner, text) {
        out.insert(id);
    }
    if let Ok(value) = parse_static(text) {
        syntax(p, owner, &value, &mut out);
    } else if let Ok(value) = dsl::parse_equation(text) {
        equation(p, owner, &value, &mut out);
    } else if let Ok(value) = dsl::parse_predicate(text) {
        predicate(p, owner, &value, &mut out);
    }
    out
}
fn type_dependencies(ty: &Type, out: &mut Vec<DeclarationId>) {
    match ty {
        Type::Entity(id)
        | Type::Enum(id)
        | Type::Table(id)
        | Type::Row(id)
        | Type::Definition(id)
        | Type::Interface(id) => out.push(*id),
        Type::Function { arguments, result } => {
            for (_, ty) in arguments {
                type_dependencies(ty, out);
            }
            type_dependencies(result, out);
        }
        Type::Set(inner) | Type::Optional(inner) => type_dependencies(inner, out),
        Type::Tuple(values) => {
            for v in values {
                type_dependencies(v, out);
            }
        }
        Type::Indexed { element, axes } => {
            type_dependencies(element, out);
            out.extend(axes);
        }
        _ => {}
    }
}
