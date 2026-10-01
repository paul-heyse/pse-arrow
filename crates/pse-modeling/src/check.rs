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

/// An expected failure names exactly one typed lineage (Plan 23 H5): a rejected validity
/// predicate, or the members a structural refusal or a diagnostic finding concerns. A form
/// or data layer predicate names its form and the arguments it constrains, and a data layer
/// one the parameter set whose envelope rejects; a closure range belongs to members, not to
/// a form, so it names only the members it bounds.
fn expected_failure_shape(
    at: DeclarationId,
    expected: &pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeFixtureExpectedFailure,
) -> Result<()> {
    use pse_model::generated::enums::ModelingValidityLayer as Layer;
    let parses = |text: &String| {
        dsl::parse_expr(text)
            .map(|_| ())
            .map_err(|e| invalid(at, e.to_string()))
    };
    if let Some(applicability) = &expected.applicability {
        if expected.validity.is_some() || !expected.members.is_empty() {
            return Err(invalid(
                at,
                "an expected failure names exactly one typed lineage",
            ));
        }
        return applicability
            .sets
            .iter()
            .chain(&applicability.variables)
            .try_for_each(parses);
    }
    match (&expected.validity, expected.members.is_empty()) {
        (Some(validity), true) => {
            if validity.variables.is_empty() {
                return Err(invalid(
                    at,
                    "an expected validity failure names the arguments or members it constrains",
                ));
            }
            match (validity.layer, &validity.form) {
                (Layer::Closure, None) if validity.sets.is_empty() => {}
                (Layer::Closure, _) => {
                    return Err(invalid(
                        at,
                        "an expected closure-layer rejection names only the members its range bounds",
                    ));
                }
                (_, None) => {
                    return Err(invalid(
                        at,
                        "an expected form or data layer rejection names its form",
                    ));
                }
                (Layer::Data, Some(_)) if validity.sets.is_empty() => {
                    return Err(invalid(
                        at,
                        "an expected data-layer failure names the parameter set whose envelope rejects",
                    ));
                }
                _ => {}
            }
            validity
                .sets
                .iter()
                .chain(&validity.variables)
                .try_for_each(parses)
        }
        (None, false) => expected.members.iter().try_for_each(parses),
        _ => Err(invalid(
            at,
            "an expected failure names exactly one lineage: validity(...) or members(...)",
        )),
    }
}
/// Resolved function contract and its declaration-owned body.
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    /// Checked typed claim applications authored on this form.
    pub applicability: Vec<dsl::Expr>,
    /// Actual selected claims retained through numerical lowering.
    pub applicability_uses: Vec<crate::applicability::Use>,
    /// Generated argument indices whose demanded effects precede the returned value.
    /// These prerequisites retain evidence without making the argument's number the result.
    pub prerequisites: Vec<usize>,
    /// Occurrence-owned concrete admissions and unresolved generic obligations.
    pub physical_admissions: crate::expression::admission::ExpressionAdmissions,
    /// Declaration-owned authorization retained through specialization and lowering.
    pub physical_operation: Option<crate::PhysicalOperation>,
    /// Generic finite reduction generated from a checked lexical reduction.
    pub reduction: Option<FiniteReduction>,
    /// Strict authored domain predicate over explicit arguments, checked before any function
    /// evaluation: the form layer, which never extrapolates (ADR-0123 Outcome 4).
    pub validity: Option<dsl::Predicate>,
    /// The data layer: the generated guards of the envelopes its row and entity arguments
    /// carry (ADR-0123 Outcome 4). A rejecting guard is checked like the form layer's
    /// predicate; specialization selects the consumer's policy.
    pub envelopes: Vec<crate::envelope::Guard>,
    /// What the form layer's predicate reads once specialized: its parameter sets and the
    /// arguments it constrains (Plan 23 H5).
    pub validity_reads: crate::envelope::Reads,
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
    pub prototype: pse_quantity::ResolvedPhysicalContract,
}
/// Checked package inventory. Every map is keyed by semantic identity, never backend handles.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckedPackage {
    pub(crate) expressions: crate::expression::occurrences::Occurrences,
    pub(crate) selection_closures: crate::scientific_selection::Selections,
    pub(crate) physical_admissions:
        BTreeMap<DeclarationId, crate::expression::admission::ExpressionAdmissions>,
    pub(crate) quantities: Arc<pse_quantity::QuantityRegistry>,
    pub(crate) preconditions: Arc<pse_quantity::PhysicalPreconditions>,
    pub(crate) scope: Arc<crate::PhysicalScope>,
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
    /// Child coordinate contracts mechanically derived from their analysis-owned policy.
    pub(crate) temporal: BTreeMap<DeclarationId, (DeclarationId, DeclarationId, String)>,
    /// Admitted immutable table values, indexed by complete typed keys.
    pub(crate) tables: BTreeMap<DeclarationId, crate::data::Table>,
    /// Full interface extension/implementation closure.
    pub(crate) interfaces: BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
    /// Checked entity kinds: refinement, attribute schema, keys and bindings (ADR-0123
    /// Outcome 2).
    pub(crate) kinds: BTreeMap<DeclarationId, crate::entity::Kind>,
    /// Admitted entity records by identity: declared entities and keyed rows.
    pub(crate) entities: BTreeMap<DeclarationId, crate::entity::Record>,
    /// Identifier values, each held by one entity of the admitted closure.
    pub(crate) identifiers: crate::entity::ModelingIdentifierScope,
    /// Typed constants.
    pub(crate) constants: BTreeMap<DeclarationId, crate::entity::Typed>,
    /// The checked provenance of every dataset and constant (ADR-0123 Outcome 5).
    pub(crate) provenance: BTreeMap<DeclarationId, crate::provenance::Provenance>,
    /// Explicit attribute origins, which override a record's default origin for attribution.
    pub(crate) attribute_provenance:
        BTreeMap<(DeclarationId, String), crate::provenance::Provenance>,
    /// The source entity each test names as its oracle.
    pub(crate) oracles: BTreeMap<DeclarationId, DeclarationId>,
    /// Test-only entities, keyed rows and constants; table rows carry their own taint.
    pub(crate) test_only: BTreeSet<DeclarationId>,
    /// Test-only datasets and constants: their role is test-only, or their lineage reaches
    /// test-only data (ADR-0123 Outcome 5).
    pub(crate) test_only_data: BTreeSet<DeclarationId>,
}
impl CheckedPackage {
    /// Immutable context-dependent scientific selection products admitted with this package.
    pub fn selection_closures(&self) -> &crate::scientific_selection::Selections {
        &self.selection_closures
    }
    /// The immutable physical environment used to admit this package.
    pub fn context(&self) -> TypeContext<'_> {
        TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &self.quantities,
            preconditions: &self.preconditions,
            scope: &self.scope,
        }
    }
    /// The top-level package declaration owning `id`.
    fn package_of(&self, mut id: DeclarationId) -> Option<&Declaration> {
        loop {
            let row = self.declarations.get(&id)?;
            match row.parent_id {
                Some(parent) => id = parent,
                None => return Some(row),
            }
        }
    }
    /// The physical name `name` denotes at `at`: a name of the physical document the
    /// owning package sees, unqualified or as `<package>.<Name>` (ADR-0123 Outcome 6).
    pub(crate) fn physical_name(
        &self,
        at: DeclarationId,
        name: &str,
    ) -> Option<pse_quantity::PhysicalName> {
        let package = self.package_of(at)?;
        if !self.scope.sees(package.document_id) {
            return None;
        }
        let local = match &self.scope.package {
            Some(prefix) => name
                .strip_prefix(prefix.as_str())
                .and_then(|tail| tail.strip_prefix('.'))
                .unwrap_or(name),
            None => name,
        };
        self.quantities.physical_name(local)
    }
    /// The physical name bindings the admitted source resolves with: each top-level
    /// package that sees the physical names, and each name, qualified by that package
    /// (ADR-0123 Outcome 8).
    pub fn physical_bindings(&self) -> BTreeMap<String, SemanticId> {
        let mut bindings = BTreeMap::new();
        for package in self
            .declarations
            .values()
            .filter(|row| row.parent_id.is_none() && self.scope.sees(row.document_id))
        {
            for (name, value) in self.quantities.physical_names() {
                let id = match value {
                    pse_quantity::PhysicalName::QuantityType(id) => id.as_id(),
                    pse_quantity::PhysicalName::ReferenceState(id) => id.as_id(),
                };
                bindings.insert(format!("{}.{name}", package.name), id);
            }
        }
        bindings
    }
    /// The one quantity type every reference state states `attribute` in, which types
    /// `state.temperature` and `state.pressure` (ADR-0123 Outcome 6).
    pub(crate) fn reference_attribute_type(
        &self,
        attribute: &str,
        at: DeclarationId,
    ) -> Result<pse_quantity::QuantityTypeId> {
        let types = self
            .quantities
            .reference_states()
            .filter_map(|state| match attribute {
                "temperature" => state.temperature,
                "pressure" => state.pressure,
                _ => None,
            })
            .map(|condition| condition.quantity_type)
            .collect::<BTreeSet<_>>();
        match types.into_iter().collect::<Vec<_>>().as_slice() {
            [one] => Ok(*one),
            [] => Err(invalid(
                at,
                format!("no reference state declares a {attribute}"),
            )),
            _ => Err(invalid(
                at,
                format!("reference states state {attribute} in different quantity types"),
            )),
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
    /// The admitted rows of the table declared as `id`.
    pub fn table(&self, id: DeclarationId) -> Option<&crate::data::Table> {
        self.tables.get(&id)
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
    /// A source member with one physical contract, without activating guards.
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
        let first = found.first().copied()?;
        if found.len() == 1 {
            return Some(first);
        }
        let first_row = &self.declarations[&first];
        let ty = self.types.get(&first)?;
        ty.quantity_scheme()?;
        let indices = crate::expression::member_indices(first_row);
        // A representative supplies only the common source type and coordinate
        // contract. Effective member identity still comes from the active guard.
        found
            .iter()
            .all(|id| {
                let row = &self.declarations[id];
                row.value.kind == first_row.value.kind
                    && self.types.get(id) == Some(ty)
                    && crate::expression::member_indices(row)
                        .iter()
                        .map(|(_, domain)| *domain)
                        .eq(indices.iter().map(|(_, domain)| *domain))
            })
            .then_some(first)
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
            // Untyped declarations cannot contribute to this environment. Avoid
            // resolving their names again for every declaration in the closure.
            .filter(|(_, id)| self.types.contains_key(*id))
            .filter(|(name, id)| self.resolve(owner, name) == Some(**id))
            .filter_map(|(name, id)| self.types.get(id).map(|ty| (name.clone(), ty.clone())))
            .collect::<BTreeMap<_, _>>();
        let mut chain = Vec::new();
        let mut node = Some(owner);
        while let Some(id) = node {
            chain.push(id);
            node = self.declarations[&id].parent_id;
        }
        // ADR-0123 Outcome 6: a package whose manifest depends on the declaring package
        // sees the physical names unqualified and as `<package>.<Name>`.
        if let Some(package) = chain.last().and_then(|id| self.declarations.get(id))
            && self.scope.sees(package.document_id)
        {
            for (name, value) in self.quantities.physical_names() {
                let ty = match value {
                    pse_quantity::PhysicalName::QuantityType(id) => {
                        Type::Quantity(pse_quantity::scheme::Scheme::Concrete(id))
                    }
                    pse_quantity::PhysicalName::ReferenceState(_) => Type::ReferenceState,
                };
                if let Some(prefix) = &self.scope.package {
                    names.insert(format!("{prefix}.{name}"), ty.clone());
                }
                names.insert(name.into(), ty);
            }
        }
        for owner in chain.into_iter().rev() {
            // A nominal coordinate slot is addressed relative to every visible lexical
            // owner exactly as ordinary declaration lookup addresses it.
            if let Some(prefix) = self
                .names
                .iter()
                .find_map(|(name, id)| (*id == owner).then(|| format!("{name}.")))
            {
                for (name, id) in &self.names {
                    if let Some(relative) = name.strip_prefix(&prefix)
                        && self.resolve(owner, relative) == Some(*id)
                        && let Some(ty) = self.types.get(id)
                    {
                        names.insert(relative.to_owned(), ty.clone());
                    }
                }
            }
            for id in self.children.get(&owner).into_iter().flatten() {
                let row = &self.declarations[id];
                if let Some(import) = &row.value.import {
                    let alias = import.alias.as_deref().unwrap_or(&row.name);
                    let prefix = format!("{}.", row.name);
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
/// Validate declarations and definitions independently of root instantiation, without
/// package data documents.
/// # Errors
/// Malformed IR, names/types/defaults, recursive ownership/extensions, or invalid expressions.
pub fn check(rows: &[Declaration], context: &TypeContext<'_>) -> Result<CheckedPackage> {
    check_with(rows, context, &crate::document::NoDocuments)
}
/// Validate declarations and definitions whose datasets may name package data documents,
/// which `documents` supplies and admits (ADR-0125).
/// # Errors
/// As [`check`], and the refusals of a data document's admission.
pub fn check_with(
    rows: &[Declaration],
    context: &TypeContext<'_>,
    documents: &dyn crate::document::Documents,
) -> Result<CheckedPackage> {
    check_declarations(rows, context, documents).map_err(|error| error.located(rows))
}
fn check_declarations(
    rows: &[Declaration],
    context: &TypeContext<'_>,
    documents: &dyn crate::document::Documents,
) -> Result<CheckedPackage> {
    let mut p = CheckedPackage {
        expressions: crate::expression::occurrences::collect(rows)?,
        selection_closures: BTreeMap::new(),
        physical_admissions: BTreeMap::new(),
        quantities: Arc::new(context.quantities.clone()),
        preconditions: Arc::new(context.preconditions.clone()),
        scope: Arc::new(context.scope.clone()),
        lowered_functions: BTreeSet::new(),
        declarations: BTreeMap::new(),
        names: BTreeMap::new(),
        children: BTreeMap::new(),
        types: BTreeMap::new(),
        functions: BTreeMap::new(),
        members: BTreeMap::new(),
        temporal: BTreeMap::new(),
        tables: BTreeMap::new(),
        interfaces: BTreeMap::new(),
        kinds: BTreeMap::new(),
        entities: BTreeMap::new(),
        identifiers: crate::entity::ModelingIdentifierScope::default(),
        constants: BTreeMap::new(),
        provenance: BTreeMap::new(),
        attribute_provenance: BTreeMap::new(),
        oracles: BTreeMap::new(),
        test_only: BTreeSet::new(),
        test_only_data: BTreeSet::new(),
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
            if (scope.selection.is_some() || scope.branch.is_some() || scope.operational.is_some())
                && row.value.kind != Kind::Implicit
                || scope.eligibility.is_some() && row.value.kind != Kind::Regime
            {
                return Err(invalid(
                    row.declaration_id,
                    "implicit selection or regime eligibility owner",
                ));
            }
            if [
                scope.selection.is_some(),
                scope.branch.is_some(),
                scope.operational.is_some(),
            ]
            .into_iter()
            .filter(|v| *v)
            .count()
                > 1
            {
                return Err(invalid(
                    row.declaration_id,
                    "one implicit selection declaration required",
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
            // ADR-0123 Outcome 5: facets belong to entity kinds.
            if !scope.facets.is_empty()
                && row.value.kind
                    != pse_model::generated::enums::ModelingDeclarationKind::EntityKind
            {
                return Err(invalid(row.declaration_id, "facets belong to entity kinds"));
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
                // The integrated and shooting routes take the fixture's integration controls.
                if matches!(execution, Execution::Integrated | Execution::Shooting)
                    != fixture.integration.is_some()
                    || execution != Execution::Initialized
                        && (!fixture.stages.is_empty() || fixture.initialization.is_some())
                    || fixture.stages.iter().any(|s| s.is_empty())
                    // Expected diagnostics describe a solved point of the steady or
                    // initialized route.
                    || !fixture.diagnostics.is_empty()
                        && !matches!(execution, Execution::Steady | Execution::Initialized)
                    || fixture
                        .diagnostics
                        .iter()
                        .any(|d| d.rule.trim().is_empty() || d.members.is_empty())
                {
                    return Err(invalid(
                        row.declaration_id,
                        "fixture execution metadata disagrees with its route",
                    ));
                }
                if let Some(expected) = &fixture.expected_failure {
                    expected_failure_shape(row.declaration_id, expected)?;
                }
                // ADR-0110 Outcome 5: a shooting fixture declares its method and controls,
                // schedules held free; single shooting has no inner nodes, multiple shooting
                // at least one. Only a schedule held free has bounds.
                let schedules = fixture
                    .integration
                    .iter()
                    .flat_map(|i| &i.schedules)
                    .collect::<Vec<_>>();
                let shooting = execution == Execution::Shooting;
                if shooting != fixture.shooting.is_some()
                    || shooting != schedules.iter().any(|s| s.free)
                    || schedules
                        .iter()
                        .any(|s| !s.free && (s.lower.is_some() || s.upper.is_some()))
                    || fixture.shooting.as_ref().is_some_and(|s| {
                        s.nodes.is_empty()
                            != (s.method == pse_model::generated::enums::ShootingMethod::Single)
                    })
                {
                    return Err(invalid(
                        row.declaration_id,
                        "a shooting fixture declares its method, the inner nodes of multiple shooting and schedules held free as its controls; only a shooting fixture holds a schedule free, with bounds",
                    ));
                }
                for expression in fixture.shooting.iter().flat_map(|s| &s.nodes).chain(
                    schedules
                        .iter()
                        .flat_map(|s| s.lower.iter().chain(&s.upper)),
                ) {
                    dsl::parse_expr(expression)
                        .map_err(|e| invalid(row.declaration_id, e.to_string()))?;
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
                            .map(|f| (f.namespace, f.name.as_str()))
                            .collect::<BTreeSet<_>>()
                            .len()
                            != m.facts.len()
                            || m.facts.iter().any(|f| {
                                crate::analysis::Fact::new(f.namespace, &f.name)
                                    .is_none_or(|fact| !fact.is_structural())
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
                // ADR-0119: an execution policy states at least one setting; allowances
                // are positive; a pure fixture starts no solver, so its policy holds only
                // specialization allowances. The consumer validates the derivative step and
                // tolerance with the run's derivative policy.
                if let Some(policy) = &fixture.policy {
                    let solver = policy.backend.is_some()
                        || policy.presolve.is_some()
                        || policy.derivative_step.is_some()
                        || policy.derivative_tolerance.is_some()
                        || policy.derivative_cells.is_some()
                        || policy.foreign_bytes.is_some()
                        || !policy.native_options.is_empty();
                    let mut names = BTreeSet::new();
                    for option in &policy.native_options {
                        use pse_authoring::language::CellSelected;
                        let primitive = match option.value.value.selected() {
                            Ok(CellSelected::Boolean(_) | CellSelected::Text(_)) => true,
                            Ok(CellSelected::Integer(value)) => i32::try_from(value.value).is_ok(),
                            Ok(CellSelected::Quantity(value)) => {
                                value.magnitude.is_finite()
                                    && value.unit.as_ref().is_none_or(Vec::is_empty)
                            }
                            _ => false,
                        };
                        if option.name.is_empty()
                            || !names.insert(&option.name)
                            || option.value.uncertainty.is_some()
                            || !primitive
                        {
                            return Err(invalid(
                                row.declaration_id,
                                format!(
                                    "native option {} requires a unique name and an exact Boolean, bounded integer, finite unitless real or text",
                                    option.name
                                ),
                            ));
                        }
                    }
                    let allowances = [
                        policy.derivative_cells,
                        policy.items,
                        policy.body_occurrences,
                        policy.body_slots,
                        policy.foreign_bytes,
                    ];
                    if !solver && allowances.iter().all(Option::is_none)
                        || allowances.iter().flatten().any(|n| *n <= 0)
                        || solver && execution == Execution::Pure
                    {
                        return Err(invalid(
                            row.declaration_id,
                            "a fixture policy states at least one setting and positive allowances; a pure fixture's policy holds only specialization allowances",
                        ));
                    }
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
        }
        {
            // ADR-0123 Outcome 4: a property package (a definition) or an analysis (a test or
            // a case) selects the data layer's extrapolation policy, once per scope. The form
            // layer never extrapolates; the closure layer's consumer states its policy on its
            // validity annotation. An entity kind declares its envelopes as declarations; a
            // table declares its own inline.
            use pse_model::generated::enums::ModelingDeclarationKind as Kind;
            let parent = row
                .parent_id
                .and_then(|id| p.declarations.get(&id))
                .map(|r| r.value.kind);
            if row.value.envelope.is_some() && parent != Some(Kind::EntityKind) {
                return Err(invalid(
                    row.declaration_id,
                    "an envelope declaration belongs to an entity kind; a table declares its envelopes inline",
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
            Selected::IdentifierScheme => Some(Type::Identifier(id)),
            Selected::Boundary(_) => Some(Type::Boundary(id)),
            Selected::CoordinateMap(_) => Some(Type::CoordinateMap(id)),
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
    // ADR-0123 Outcome 6: a package that sees the physical names declares none of them;
    // the same name in both places is ambiguous and refused.
    for row in rows {
        let Some(package) = row.parent_id.and_then(|id| p.declarations.get(&id)) else {
            continue;
        };
        if package.parent_id.is_some() || !context.scope.sees(package.document_id) {
            continue;
        }
        let local = row
            .value
            .import
            .as_ref()
            .and_then(|import| import.alias.as_deref())
            .unwrap_or(&row.name);
        if context.quantities.physical_name(local).is_some() {
            return Err(invalid(
                row.declaration_id,
                format!(
                    "ambiguous name {local}: declared by package {} and by the physical document",
                    package.name
                ),
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
                &v.r#type,
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
    crate::temporal::admit(&mut p, context)?;
    crate::physical_operations::admit_signatures(&mut p, context)?;
    // Interface and definition dependencies are explicit graphs, never Salsa recovery.
    let mut inheritance = DiGraph::<DeclarationId, ()>::new();
    let inode = p
        .declarations
        .keys()
        .map(|id| (*id, inheritance.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    // ADR-0123 Outcome 2: an entity kind refines at most one entity kind, and only a kind is
    // the base of a kind; interfaces and definitions extend interfaces. One graph and one
    // cycle check serve both.
    use pse_model::generated::enums::ModelingDeclarationKind as DeclarationKind;
    for row in rows {
        if row.value.kind == DeclarationKind::EntityKind {
            p.kinds
                .insert(row.declaration_id, crate::entity::Kind::default());
        }
    }
    for row in rows {
        if let Some(scope) = &row.value.scope {
            let kind = row.value.kind == DeclarationKind::EntityKind;
            if kind && scope.bases.len() > 1 {
                return Err(invalid(
                    row.declaration_id,
                    "an entity kind refines at most one kind",
                ));
            }
            for name in &scope.bases {
                let base = p
                    .resolve(row.declaration_id, name)
                    .ok_or_else(|| invalid(row.declaration_id, format!("unknown base {name}")))?;
                match (kind, p.kinds.contains_key(&base)) {
                    (true, true) => {
                        if let Some(record) = p.kinds.get_mut(&row.declaration_id) {
                            record.base = Some(base);
                        }
                    }
                    (true, false) => {
                        return Err(invalid(
                            row.declaration_id,
                            format!(
                                "only an entity kind is the base of an entity kind; {name} is not one"
                            ),
                        ));
                    }
                    (false, true) => {
                        return Err(invalid(
                            row.declaration_id,
                            format!("entity kind {name} is the base of entity kinds only"),
                        ));
                    }
                    (false, false) if !matches!(p.types.get(&base), Some(Type::Interface(_))) => {
                        return Err(invalid(row.declaration_id, "base must be an interface"));
                    }
                    (false, false) => {}
                }
                inheritance.add_edge(inode[&base], inode[&row.declaration_id], ());
            }
            // A stage replaces equations of its definition, its own or inherited, so the
            // definition's effective members are resolved before its stages.
            if row.value.kind == DeclarationKind::Stage
                && let Some(parent) = row.parent_id
                && let Some(owner) = inode.get(&parent)
            {
                inheritance.add_edge(*owner, inode[&row.declaration_id], ());
            }
        }
    }
    if let Some(cycle) = kosaraju_scc(&inheritance)
        .into_iter()
        .find(|c| c.len() > 1 || c.first().is_some_and(|n| inheritance.contains_edge(*n, *n)))
    {
        let kinds = cycle.iter().all(|n| p.kinds.contains_key(&inheritance[*n]));
        return Err(invalid(
            inheritance[cycle[0]],
            format!(
                "recursive {}: {}",
                if kinds {
                    "kind refinement"
                } else {
                    "interface extension"
                },
                cycle
                    .iter()
                    .map(|n| p.declarations[&inheritance[*n]].name.clone())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ),
        ));
    }
    let inheritance_order = toposort(&inheritance, None)
        .map_err(|c| invalid(inheritance[c.node_id()], "interface cycle"))?;
    // Project declaration-owned candidate namespaces before resolving signatures.
    // Nominal types can name inherited boundaries; the same candidates receive the
    // full override and signature validation below, without a second name authority.
    let mut candidate_scopes = BTreeMap::new();
    for node in &inheritance_order {
        let id = inheritance[*node];
        let row = &p.declarations[&id];
        let Some(scope) = &row.value.scope else {
            continue;
        };
        if row.value.kind == DeclarationKind::EntityKind {
            continue;
        }
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
        p.interfaces.insert(id, contracts.clone());
        candidate_scopes.insert(id, (own, effective, contracts));
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
                    let ty = context.resolve(&arg.r#type, &variables, &names, at)?;
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
            if let Some(ty) = &v.r#type {
                p.types
                    .insert(id, context.resolve(ty, &variables, &names, id)?);
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
                .insert(id, context.resolve(&v.r#type, &variables, &names, id)?);
        }
        if let Some(v) = &row.value.inventory_balance {
            let ty = context.resolve(&v.r#type, &variables, &names, id)?;
            if ty.quantity_scheme().is_none() {
                return Err(invalid(id, "inventory balance requires a physical type"));
            }
            p.types.insert(id, ty);
        }
        // ADR-0123 Outcome 2: attributes belong to entity kinds; a binding takes its
        // inherited attribute's type when the kind's members are known.
        if let Some(v) = &row.value.attribute {
            if !row
                .parent_id
                .is_some_and(|parent| p.kinds.contains_key(&parent))
            {
                return Err(invalid(id, "an attribute belongs to an entity kind"));
            }
            if let Some(ty) = &v.r#type {
                p.types
                    .insert(id, context.resolve(ty, &variables, &names, id)?);
            }
        }
        if let Some(v) = &row.value.constant {
            p.types
                .insert(id, context.resolve(&v.r#type, &variables, &names, id)?);
        }
        if let Some(v) = &row.value.continuous {
            let ty = context.resolve(&v.r#type, &variables, &names, id)?;
            if !matches!(ty, Type::Quantity(_)) {
                return Err(invalid(id, "continuous axis requires a physical type"));
            }
            p.types.insert(id, Type::Continuous(id, Box::new(ty)));
        }
        if let Some(v) = &row.value.entity {
            let kind = p
                .resolve(id, &v.kind_name)
                .ok_or_else(|| invalid(id, "unknown entity kind"))?;
            if !p.kinds.contains_key(&kind) {
                return Err(invalid(id, "entity requires a declared kind"));
            }
            p.types.insert(id, Type::Entity(kind));
        }
        if row.value.applicability.is_some() {
            crate::applicability::signature(&mut p, context, row, &names)?;
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
                        context.resolve(&a.r#type, &variables, &names, id)?,
                    ))
                })
                .collect::<Result<_>>()?;
            let result = context.resolve(&v.return_type, &variables, &names, id)?;
            let body = v
                .body
                .as_ref()
                .map(|b| p.expression(id, b).cloned())
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
                    applicability: v
                        .applicability
                        .iter()
                        .map(|e| p.expression(id, e).cloned())
                        .collect::<Result<_>>()?,
                    applicability_uses: Vec::new(),
                    prerequisites: Vec::new(),
                    physical_admissions: BTreeMap::new(),
                    physical_operation: None,
                    reduction: None,
                    validity: v
                        .validity
                        .as_ref()
                        .map(|source| p.predicate(id, source).cloned())
                        .transpose()?,
                    // Resolved once tables and kinds are admitted (`envelope::admit`).
                    envelopes: Vec::new(),
                    validity_reads: crate::envelope::Reads::default(),
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
                || values
                    .members
                    .iter()
                    .any(|m| m.name.is_empty() || m.member_id == SemanticId::NIL)
                || values
                    .members
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<BTreeSet<_>>()
                    .len()
                    != values.members.len()
                || values
                    .members
                    .iter()
                    .map(|m| m.member_id)
                    .collect::<BTreeSet<_>>()
                    .len()
                    != values.members.len())
        {
            return Err(invalid(
                id,
                "enumeration requires distinct named members with distinct identities",
            ));
        }
    }
    for id in p.functions.keys().copied().collect::<Vec<_>>() {
        let mut function = p.functions[&id].clone();
        crate::physical_operations::admit_reduced_law(&p, &mut function)?;
        p.functions.insert(id, function);
    }
    for node in inheritance_order {
        let id = inheritance[node];
        let row = &p.declarations[&id];
        let Some(_) = &row.value.scope else {
            continue;
        };
        // A kind's members are its refined kind's and its own; a binding names an inherited
        // attribute and has its type (ADR-0123 Outcome 2). `entity::admit` checks the rest.
        if row.value.kind == DeclarationKind::EntityKind {
            let mut members = p
                .kinds
                .get(&id)
                .and_then(|k| k.base)
                .and_then(|base| p.members.get(&base))
                .cloned()
                .unwrap_or_default();
            for child in p.children.get(&id).cloned().unwrap_or_default() {
                // An envelope bounds the kind's attributes and a requirement constrains its
                // entities; neither is a member (ADR-0123 Outcome 4, Plan 23 D0).
                if p.declarations[&child].value.envelope.is_some()
                    || p.declarations[&child].value.requirement.is_some()
                {
                    continue;
                }
                let name = p.declarations[&child].name.clone();
                if p.declarations[&child]
                    .value
                    .attribute
                    .as_ref()
                    .is_some_and(|a| a.r#type.is_none())
                    && let Some(ty) = members.get(&name).and_then(|m| p.types.get(m)).cloned()
                {
                    p.types.insert(child, ty);
                }
                members.insert(name, child);
            }
            p.members.insert(id, members);
            continue;
        }
        let (own, mut effective, contracts) = candidate_scopes
            .remove(&id)
            .ok_or_else(|| invalid(id, "interface candidate namespace absent"))?;
        let projected = p.members.get(&id).cloned().unwrap_or_default();
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
                    // The target is a member of the stage's definition, declared there or
                    // inherited from an interface it implements.
                    let base = p
                        .members
                        .get(&parent)
                        .and_then(|members| members.get(&name))
                        .copied()
                        .ok_or_else(|| invalid(member, "stage override target absent"))?;
                    let original = &p.declarations[&base].value;
                    let replacement = &p.declarations[&member].value;
                    if !((original.equation.is_some() && replacement.equation.is_some())
                        || (original.connection.is_some() && replacement.connection.is_some()))
                    {
                        return Err(invalid(
                            member,
                            "stage overrides replace equations or connections of the same kind; variable identities remain stable",
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
    crate::contextual::admit_translations(&mut p, context)?;
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
        let translation = match function.physical_operation.as_ref() {
            Some(crate::PhysicalOperation::ReferenceTranslation(translation)) => Some(translation),
            _ => None,
        };
        if let Some(translation) = translation {
            for anchor in [translation.source_anchor, translation.target_anchor] {
                if let Some(target) = function_nodes.get(&anchor) {
                    calls.add_edge(function_nodes[id], *target, ());
                }
            }
        }
        let conditions = translation
            .into_iter()
            .flat_map(|translation| [&translation.temperature, &translation.pressure]);
        for body in function.body.iter().chain(domain.iter()).chain(conditions) {
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
    crate::entity::admit(&mut p, context, documents)?;
    crate::provenance::admit(&mut p)?;
    crate::contextual::admit_translation_provenance(&mut p)?;
    // Tables, then the kinds' derived attributes, which may read them; requirements read
    // both (Plan 23 D0). A dataset naming a data document has its rows admitted by
    // `documents` (ADR-0125).
    let requirements = crate::data::admit(&mut p, context, documents)?;
    crate::entity::derive(&mut p, context)?;
    crate::data::verify_rows(&p, &requirements)?;
    crate::entity::verify(&p)?;
    crate::envelope::admit(&mut p, context)?;
    crate::applicability::admit(&p, context)?;
    crate::expression::check_all(&mut p, context)?;
    crate::expression::occurrences::bind(&mut p);
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
            pending.extend(
                self.expressions
                    .iter()
                    .filter(|(key, _)| key.declaration == id)
                    .flat_map(|(_, expression)| expression.dependencies.iter().copied()),
            );
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
            let mut types: Vec<&Vec<pse_authoring::language::TypeNode>> = Vec::new();
            // Cells name declarations only as reference paths (ADR-0123 Outcome 1).
            let mut cells: Vec<&pse_authoring::language::Cell> = Vec::new();
            // Completeness names declared sets and enumerations by path (Outcome 3).
            let mut sets: Vec<&Vec<String>> = Vec::new();
            // Provenance and oracles name sources, roles and lineage by path (Outcome 5).
            let mut paths: Vec<&Vec<String>> = Vec::new();
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
                texts.extend(v.branch.as_deref());
                if let Some(operation) = &v.operational {
                    texts.extend(operation.anchors.iter().map(|a| a.expression.as_str()));
                    texts.extend(operation.neighborhood.as_deref());
                }
                texts.extend(v.eligibility.as_deref());
                // A test depends on its oracle source (ADR-0123 Outcome 5).
                paths.extend(v.oracle.iter());
                if let Some(fixture) = &v.fixture {
                    if let Some(expected) = &fixture.expected_failure
                        && let Some(v) = &expected.applicability
                    {
                        texts.extend([v.claim.as_str(), v.form.as_str()]);
                        texts.extend(v.sets.iter().map(String::as_str));
                    }
                    if let Some(integration) = &fixture.integration {
                        texts.extend(integration.samples.iter().map(String::as_str));
                        texts.push(&integration.initial_step);
                        for q in &integration.quadratures {
                            texts.push(&q.target);
                            texts.push(&q.absolute_tolerance);
                        }
                        for s in &integration.schedules {
                            texts.push(&s.target);
                            texts.extend(
                                s.times
                                    .iter()
                                    .chain(&s.values)
                                    .chain(s.lower.iter().chain(&s.upper))
                                    .map(String::as_str),
                            );
                        }
                    }
                    if let Some(shooting) = &fixture.shooting {
                        texts.extend(shooting.nodes.iter().map(String::as_str));
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
                    types.push(&p.r#type);
                    texts.extend(p.default_value.as_deref());
                }
            }
            if let Some(v) = &row.value.binding {
                types.extend(v.r#type.as_ref());
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
                texts.extend(v.applicability.iter().map(String::as_str));
                types.push(&v.return_type);
                types.extend(v.arguments.iter().map(|a| &a.r#type));
            }
            if let Some(v) = &row.value.applicability {
                texts.extend([v.owner.as_str(), v.evidence.as_str()]);
                texts.extend(v.predicate.as_deref());
                texts.extend(v.lower.as_deref());
                texts.extend(v.upper.as_deref());
                texts.extend(
                    v.alternatives
                        .iter()
                        .chain(&v.dependencies)
                        .map(String::as_str),
                );
                types.extend(v.arguments.iter().map(|a| &a.r#type));
            }
            if let Some(v) = &row.value.permission {
                texts.extend(v.targets.iter().map(String::as_str));
            }
            if let Some(v) = &row.value.coordinate_map {
                types.extend(v.arguments.iter().map(|a| &a.r#type));
                texts.extend(v.validity.as_deref());
                pending.extend(self.children.get(&id).into_iter().flatten().copied());
            }
            if let Some(v) = &row.value.coordinate_slot {
                texts.push(&v.expression);
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
            }
            if let Some(v) = &row.value.reconstruction {
                texts.extend([
                    v.map.as_str(),
                    v.reference.as_str(),
                    v.normalization.as_str(),
                ]);
                types.push(&v.return_type);
                types.extend(v.arguments.iter().map(|a| &a.r#type));
            }
            if let Some(v) = &row.value.response {
                texts.push(&v.body);
                types.push(&v.return_type);
                types.extend(v.arguments.iter().map(|a| &a.r#type));
            }
            if let Some(v) = &row.value.reference_translation {
                texts.extend([
                    v.source_anchor.as_str(),
                    v.target_anchor.as_str(),
                    v.temperature.as_str(),
                    v.pressure.as_str(),
                ]);
                types.push(&v.return_type);
                types.extend(v.arguments.iter().map(|a| &a.r#type));
                paths.extend(provenance_paths(&v.provenance));
            }
            if let Some(v) = &row.value.boundary {
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
            }
            if let Some(v) = &row.value.exchange {
                texts.extend([v.from.as_str(), v.to.as_str()]);
                texts.extend(v.indices.iter().map(|i| i.domain.as_str()));
            }
            if let Some(v) = &row.value.table {
                types.extend(v.value_type.as_ref());
                cells.extend(v.default_value.as_ref());
                types.extend(v.keys.iter().map(|k| &k.r#type));
                types.extend(v.columns.iter().map(|c| &c.r#type));
                texts.extend(v.columns.iter().filter_map(|c| c.derived.as_deref()));
                texts.extend(v.requirements.iter().map(String::as_str));
                sets.extend(v.complete_over.iter().filter_map(|e| e.set.as_ref()));
                types.extend(v.envelopes.iter().map(|e| &e.r#type));
            }
            if let Some(v) = &row.value.envelope {
                types.push(&v.r#type);
            }
            // A table or a kind depends on the datasets that supply its rows, a kind also on
            // those of its refinements.
            if row.value.table.is_some() || self.kinds.contains_key(&id) {
                for dataset in self.declarations.values() {
                    if dataset.value.dataset.as_ref().is_some_and(|v| {
                        self.resolve(dataset.declaration_id, &v.target)
                            .is_some_and(|target| target == id || self.refines(target, id))
                    }) {
                        pending.push(dataset.declaration_id);
                    }
                }
            }
            if let Some(kind) = self.kinds.get(&id) {
                pending.extend(kind.base);
            }
            if let Some(v) = &row.value.dataset {
                texts.push(&v.target);
                paths.extend(provenance_paths(&v.provenance));
                sets.extend(v.complete_over.iter().filter_map(|e| e.set.as_ref()));
                cells.extend(v.bindings.iter().map(|b| &b.value));
                for r in &v.rows {
                    cells.extend(r.keys.iter().chain(&r.values));
                }
            }
            if let Some(v) = &row.value.entity {
                texts.push(&v.kind_name);
                cells.extend(v.attributes.iter().map(|a| &a.value));
                paths.extend(v.provenance.iter().flat_map(provenance_paths));
                paths.extend(
                    v.attributes
                        .iter()
                        .flat_map(|a| a.provenance.iter().flat_map(provenance_paths)),
                );
            }
            if let Some(v) = &row.value.attribute {
                types.extend(v.r#type.as_ref());
                cells.extend(v.value.as_ref());
                texts.extend(v.derived.as_deref());
            }
            if let Some(v) = &row.value.constant {
                types.push(&v.r#type);
                cells.push(&v.value);
                paths.extend(provenance_paths(&v.provenance));
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
                types.push(&v.r#type);
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
                types.push(&v.r#type);
                texts.extend([v.lower.as_str(), v.upper.as_str()]);
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
            if let Some(v) = &row.value.temporal {
                texts.extend([v.target.as_str(), v.axis.as_str()]);
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
            for set in sets {
                pending.extend(self.resolve(id, &set.join(".")));
            }
            // A role depends on its enumeration.
            for path in paths {
                pending.extend(self.resolve(id, &path.join(".")));
                if let Some((_, owner)) = path.split_last()
                    && !owner.is_empty()
                {
                    pending.extend(self.resolve(id, &owner.join(".")));
                }
            }
            for path in cells.into_iter().flat_map(cell_paths) {
                pending.extend(self.resolve(id, &path));
                // A qualified enumeration member depends on its enumeration.
                if let Some((owner, _)) = path.rsplit_once('.') {
                    pending.extend(self.resolve(id, owner));
                }
            }
            // A declared type depends on the declarations its paths name (ADR-0123
            // Outcome 1): each joined path is resolved once, never re-parsed.
            for path in types
                .into_iter()
                .flat_map(|nodes| pse_authoring::language::type_paths(nodes))
            {
                pending.extend(self.resolve(id, &path));
            }
            if let Some(ty) = self.types.get(&id) {
                type_dependencies(ty, &mut pending);
            }
            if let Some(f) = self.functions.get(&id) {
                if let Some(body) = &f.body {
                    pending.extend(dependency_paths(self, id, &dsl::render_expr(body)));
                }
                for (_, ty) in &f.arguments {
                    type_dependencies(ty, &mut pending);
                }
                type_dependencies(&f.result, &mut pending);
            }
            if let Some(table) = self.tables.get(&id) {
                for key in &table.keys {
                    type_dependencies(&key.ty, &mut pending);
                }
                for column in &table.columns {
                    type_dependencies(&column.ty, &mut pending);
                }
                type_dependencies(&table.result, &mut pending);
            }
        }
        let mut p = self.clone();
        p.declarations.retain(|id, _| selected.contains(id));
        p.names.retain(|_, id| selected.contains(id));
        p.types.retain(|id, _| selected.contains(id));
        p.expressions
            .retain(|key, _| selected.contains(&key.declaration));
        p.functions.retain(|id, _| selected.contains(id));
        p.physical_admissions.retain(|id, _| selected.contains(id));
        p.tables.retain(|id, _| selected.contains(id));
        p.children.retain(|id, _| selected.contains(id));
        for members in p.children.values_mut() {
            members.retain(|id| selected.contains(id));
        }
        p.members.retain(|id, _| selected.contains(id));
        p.temporal.retain(|target, (policy, axis, _)| {
            selected.contains(target) && selected.contains(policy) && selected.contains(axis)
        });
        for members in p.members.values_mut() {
            members.retain(|_, id| selected.contains(id));
        }
        p.interfaces.retain(|id, _| selected.contains(id));
        for interfaces in p.interfaces.values_mut() {
            interfaces.retain(|id| selected.contains(id));
        }
        p.kinds.retain(|id, _| selected.contains(id));
        p.constants.retain(|id, _| selected.contains(id));
        p.provenance.retain(|id, _| selected.contains(id));
        p.attribute_provenance
            .retain(|(id, _), _| selected.contains(id));
        p.oracles.retain(|id, _| selected.contains(id));
        // A record stays with the declaration that admitted it: its entity or its dataset.
        p.entities
            .retain(|_, record| selected.contains(&record.origin));
        let entities = p.entities.keys().copied().collect::<BTreeSet<_>>();
        p.identifiers.retain(|entity| entities.contains(&entity));
        p.test_only
            .retain(|id| entities.contains(id) || p.constants.contains_key(id));
        p.test_only_data.retain(|id| selected.contains(id));
        Ok(p)
    }
}
/// The paths a provenance names: its source, its role and its lineage entries.
fn provenance_paths(
    provenance: &pse_authoring::language::ModelingProvenance,
) -> impl Iterator<Item = &Vec<String>> {
    [&provenance.source, &provenance.role]
        .into_iter()
        .chain(provenance.lineage.iter().map(|entry| &entry.path))
}
/// The paths a cell names: its reference, its set's references, an identifier's scheme,
/// and a keyed-row reference's target with the paths of its key cells.
fn cell_paths(cell: &pse_authoring::language::Cell) -> Vec<String> {
    use pse_authoring::language::CellSelected;
    match cell.value.selected() {
        Ok(CellSelected::Reference(v)) => vec![v.path.join(".")],
        Ok(CellSelected::References(v)) => v
            .paths
            .iter()
            .flat_map(|p| {
                std::iter::once(p.path.join(".")).chain(
                    p.keys
                        .iter()
                        .flatten()
                        .filter_map(|key| pse_authoring::language::key_cell_value(key).ok())
                        .flat_map(|cell| cell_paths(&cell)),
                )
            })
            .collect(),
        Ok(CellSelected::Identifier(v)) => vec![v.scheme.join(".")],
        Ok(CellSelected::Row(v)) => std::iter::once(v.target.join("."))
            .chain(
                v.keys
                    .iter()
                    .filter_map(|key| pse_authoring::language::key_cell_value(key).ok())
                    .flat_map(|cell| cell_paths(&cell)),
            )
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn dependency_paths(
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
        | Type::Identifier(id)
        | Type::Table(id)
        | Type::Row(id)
        | Type::Definition(id)
        | Type::Boundary(id)
        | Type::CoordinateMap(id)
        | Type::Interface(id) => out.push(*id),
        Type::Reconstruction { declaration, map } => out.extend([*declaration, *map]),
        Type::RefinedQuantity { refinement, .. } => match refinement {
            crate::PhysicalRefinement::Coordinate { map, slot } => out.extend([*map, *slot]),
            crate::PhysicalRefinement::ReducedLaw {
                map,
                reconstruction,
            } => out.extend([*map, *reconstruction]),
            crate::PhysicalRefinement::Transfer { boundary, .. } => match boundary {
                crate::BoundaryRef::Declared(id)
                | crate::BoundaryRef::Bound {
                    declaration: id, ..
                } => out.push(*id),
            },
        },
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
