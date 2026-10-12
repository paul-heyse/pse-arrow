// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure declaration phases. Retention and input publication belong to the compiler.
use super::*;
use crate::document::{DocumentPlan, DocumentTable, Documents};
use crate::expression::occurrences::{CheckedExpression, OccurrenceKey, ParsedFields};
pub mod expressions;

/// Structurally shared ordered signature state, preserving source-order visibility.
pub type Types = rpds::RedBlackTreeMapSync<DeclarationId, Type>;

/// Pure phase composition. The fresh implementation is the complete functional oracle.
pub trait Phases {
    /// Parse an exact declaration and its independently owned field attribution.
    fn parse(&self, row: &Declaration, documents: &dyn Documents) -> Result<Arc<ParsedFields>> {
        if let Some(parsed) = documents.parsed_fields(row.declaration_id)
            && parsed.matches(row, documents.field_spans(row.declaration_id))
        {
            return Ok(parsed.clone());
        }
        Ok(Arc::new(ParsedFields::parse(
            row,
            documents.field_spans(row.declaration_id),
        )?))
    }
    /// Enclosing duplicate, parent, inheritance and physical namespace obligations.
    fn inventory(
        &self,
        rows: &[Declaration],
        context: &TypeContext<'_>,
        documents: &dyn Documents,
    ) -> Result<InventoryProduct> {
        inventory(rows, context, documents)
    }
    /// Resolve and type one declaration using the actual preceding signature population.
    fn signature(
        &self,
        row: &Declaration,
        parsed: &Arc<ParsedFields>,
        environment: &ResolutionEnvironment,
        context: &TypeContext<'_>,
    ) -> Result<Signature> {
        let resolved = environment.resolve(row, context)?;
        let mut product = signature(row, parsed, &resolved, context)?;
        product.admit_reduced(&environment.namespace)?;
        Ok(product)
    }
    /// Admit one effective member population against exact inherited contracts.
    fn members(&self, input: &MemberInput) -> Result<MemberProduct> {
        input.check()
    }
    /// Complete enclosing graph, scientific data and physical obligations.
    fn finish(
        &self,
        package: CheckedPackage,
        context: &TypeContext<'_>,
        documents: &dyn Documents,
    ) -> Result<CheckedPackage> {
        finish_package(package, context, documents)
    }
    /// Bind occurrence attribution and lexical dependencies after complete admission.
    fn bind(&self, package: &mut CheckedPackage) -> Result<()> {
        crate::expression::occurrences::bind(package);
        Ok(())
    }
    /// Check mathematical fields and retain physical admissions after lexical binding.
    fn expressions(
        &self,
        package: &mut CheckedPackage,
        context: &TypeContext<'_>,
        _: &dyn Documents,
    ) -> Result<()> {
        crate::expression::check_all(package, context)
    }
}
/// Deliberately fresh complete checker control, with no retained compiler state.
#[derive(Debug)]
pub struct Fresh;
impl Phases for Fresh {}

pub(super) struct ParsedDocuments<'a> {
    pub(super) inner: &'a dyn Documents,
    pub(super) parsed: &'a BTreeMap<DeclarationId, Arc<ParsedFields>>,
}
impl Documents for ParsedDocuments<'_> {
    fn field_spans(
        &self,
        id: DeclarationId,
    ) -> Option<&BTreeMap<String, pse_authoring::SourceSpan>> {
        self.inner.field_spans(id)
    }
    fn parsed_fields(&self, id: DeclarationId) -> Option<&Arc<ParsedFields>> {
        self.parsed.get(&id)
    }
    fn preflight(&self, at: DeclarationId, bytes: usize) -> Result<()> {
        self.inner.preflight(at, bytes)
    }
    fn rows(&self, id: SemanticId) -> Option<usize> {
        self.inner.rows(id)
    }
    fn bytes(&self, id: SemanticId) -> usize {
        self.inner.bytes(id)
    }
    fn resolve(&self, source: SemanticId, path: &str) -> Option<SemanticId> {
        self.inner.resolve(source, path)
    }
    fn admit(&self, plan: &Arc<DocumentPlan>, document: SemanticId) -> Result<Arc<DocumentTable>> {
        self.inner.admit(plan, document)
    }
}

/// Shared exact lexical/member population, including every absent lookup candidate.
/// Arithmetic source and byte offsets are owned separately by parsed occurrences.
#[derive(Clone, Debug, PartialEq)]
pub struct Namespace(Arc<CheckedPackage>);
impl Namespace {
    /// Freeze namespace metadata once per phase boundary, rather than cloning each prefix.
    pub fn new(package: &CheckedPackage) -> Self {
        let mut p = package.clone();
        p.expressions.clear();
        p.selection_closures.clear();
        p.selection_index = SelectionIndex::default();
        p.tables.clear();
        p.entities.clear();
        p.constants.clear();
        p.provenance.clear();
        p.attribute_provenance.clear();
        p.oracles.clear();
        p.test_only.clear();
        p.test_only_data.clear();
        for function in p.functions.values_mut() {
            function.body = None;
            function.validity = None;
            function.applicability.clear();
            function.applicability_uses.clear();
            function.physical_admissions.clear();
            function.envelopes.clear();
            function.validity_reads = Default::default();
        }
        for row in p.declarations.values_mut() {
            row.source_start = 0;
            row.source_end = 0;
            if let Some(binding) = &mut row.value.binding
                && let Some(expression) = &mut binding.expression
            {
                expression.clear();
            }
            if let Some(function) = &mut row.value.function {
                if let Some(body) = &mut function.body {
                    body.clear();
                }
                function.applicability.clear();
                function.validity = None;
            }
            row.value.equation = None;
            row.value.dataset = None;
            row.value.entity = None;
            row.value.constant = None;
            row.value.applicability = None;
            row.value.annotation = None;
            row.value.expectation = None;
        }
        Self(Arc::new(p))
    }
    /// Known namespace storage; shared physical context is retained by its existing owner.
    pub fn retained_bytes(&self) -> usize {
        self.0.retained_bytes()
    }
    /// Bind one occurrence using the existing lexical and physical-name owners.
    pub fn bind(&self, key: &OccurrenceKey, value: CheckedExpression) -> CheckedExpression {
        crate::expression::occurrences::bind_one(&self.0, key, value)
    }
}
/// Complete consumed lexical context and persistent preceding type population.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolutionEnvironment {
    /// Ordered namespace/import/member population; absence is represented exactly.
    pub namespace: Namespace,
    /// Source-order type state. Clones share tree nodes and updates copy only a path.
    pub types: Types,
}
/// Resolved names and scope arguments consumed by declaration-owned type construction.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedSignature {
    names: BTreeMap<String, Type>,
    variables: BTreeSet<String>,
    existing_type: Option<Type>,
    attribute_owner: bool,
    entity_kind: Option<DeclarationId>,
    entity_is_kind: bool,
}
impl ResolutionEnvironment {
    /// Resolve through the same visibility and physical name rules as the complete checker.
    /// # Errors
    /// Unknown or malformed scope parameter types.
    pub fn resolve(
        &self,
        row: &Declaration,
        context: &TypeContext<'_>,
    ) -> Result<ResolvedSignature> {
        let p = &self.namespace.0;
        let lookup = |id| self.types.get(&id);
        let mut names = p.named_types_with(row.declaration_id, &lookup);
        let mut variables = BTreeSet::new();
        let mut owner = Some(row.declaration_id);
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
        let entity_kind = row
            .value
            .entity
            .as_ref()
            .and_then(|v| p.resolve_with_types(row.declaration_id, &v.kind_name, &lookup));
        Ok(ResolvedSignature {
            names,
            variables,
            existing_type: lookup(row.declaration_id).cloned(),
            attribute_owner: row
                .parent_id
                .is_some_and(|parent| p.kinds.contains_key(&parent)),
            entity_is_kind: entity_kind.is_some_and(|kind| p.kinds.contains_key(&kind)),
            entity_kind,
        })
    }
    /// Conservative known retained tree extent. Shared nodes may be counted repeatedly.
    pub fn retained_bytes(&self) -> usize {
        self.namespace.retained_bytes().saturating_add(
            self.types
                .iter()
                .map(|(_, ty)| size_of::<(DeclarationId, Type)>() + 256 + crate::extent::ty(ty))
                .sum::<usize>(),
        )
    }
}
/// Checked type and callable body of one declaration, before global expression admission.
#[derive(Clone, Debug, PartialEq)]
pub struct Signature {
    /// A type assigned or retained by this declaration.
    pub ty: Option<Type>,
    /// Declaration-owned function contract and parsed body.
    pub function: Option<Function>,
}
impl Signature {
    /// Retain the existing reduced-law admission under this declaration's tracked
    /// signature owner; it consumes only its arguments and the exact slot population.
    /// # Errors
    /// Reduced-law arguments do not belong to their declared coordinate map.
    pub fn admit_reduced(&mut self, namespace: &Namespace) -> Result<()> {
        if let Some(function) = &mut self.function {
            crate::physical_operations::admit_reduced_law(&namespace.0, function)?;
        }
        Ok(())
    }
    /// Conservative known storage retained by one signature.
    pub fn retained_bytes(&self) -> usize {
        self.ty.as_ref().map_or(0, crate::extent::ty)
            + self.function.as_ref().map_or(0, |f| {
                size_of::<Function>() + f.body.as_ref().map_or(0, crate::expression::retained_bytes)
            })
            + 256
    }
}
impl ResolvedSignature {
    /// Conservative known resolved-name storage.
    pub fn retained_bytes(&self) -> usize {
        self.names
            .iter()
            .map(|(name, ty)| name.capacity() + crate::extent::ty(ty) + 128)
            .sum::<usize>()
            + self
                .variables
                .iter()
                .map(|name| name.capacity() + 64)
                .sum::<usize>()
            + 256
    }
}
/// Precharge a conservative persistent red-black insertion path, its rotations and type value.
pub fn type_update_bytes(entries: usize, ty: Option<&Type>) -> usize {
    let height = (usize::BITS - entries.saturating_add(1).leading_zeros()) as usize;
    height
        .saturating_mul(4)
        .saturating_add(8)
        .saturating_mul(size_of::<(DeclarationId, Type)>() + 256)
        .saturating_add(ty.map_or(0, crate::extent::ty))
}
/// Run the original declaration-owned rules against a resolved immutable environment.
/// # Errors
/// Exactly the original declaration signature refusals.
pub fn signature(
    row: &Declaration,
    parsed: &ParsedFields,
    resolved: &ResolvedSignature,
    context: &TypeContext<'_>,
) -> Result<Signature> {
    let id = row.declaration_id;
    let mut p = empty_package(context);
    p.declarations.insert(id, row.clone());
    p.expressions = parsed.expressions().clone();
    if let Some(ty) = &resolved.existing_type {
        p.types.insert(id, ty.clone());
    }
    let names = resolved.names.clone();
    let mut variables = resolved.variables.clone();
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
                if domain.is_discrete() && !matches!(p.types.get(&id), Some(Type::Quantity(_))) =>
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
        if !resolved.attribute_owner {
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
    if row.value.entity.is_some() {
        let kind = resolved
            .entity_kind
            .ok_or_else(|| invalid(id, "unknown entity kind"))?;
        if !resolved.entity_is_kind {
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
            .map(|_| p.expression_at(id, "function.body", 0).cloned())
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
                    .enumerate()
                    .map(|(position, _)| {
                        p.expression_at(id, "function.applicability", position)
                            .cloned()
                    })
                    .collect::<Result<_>>()?,
                applicability_uses: Vec::new(),
                prerequisites: Vec::new(),
                physical_admissions: BTreeMap::new(),
                physical_operation: None,
                reduction: None,
                validity: v
                    .validity
                    .as_ref()
                    .map(|_| p.predicate_at(id, "function.validity", 0).cloned())
                    .transpose()?,
                // Resolved once tables and kinds are admitted (`envelope::admit`).
                envelopes: Vec::new(),
                validity_reads: crate::envelope::Reads::default(),
                external: v
                    .external
                    .as_ref()
                    .map(|v| {
                        crate::external::External::check(
                            v,
                            id,
                            p.expression_at(id, "function.external.output", 0)?,
                        )
                    })
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
    Ok(Signature {
        ty: p.types.remove(&id),
        function: p.functions.remove(&id),
    })
}

/// Complete enclosing obligations retained conservatively where scientific dependencies are global.
/// # Errors
/// Full graph, data, provenance, physical and expression admission failures.
pub fn finish(
    package: CheckedPackage,
    context: &TypeContext<'_>,
    documents: &dyn Documents,
) -> Result<CheckedPackage> {
    finish_package(package, context, documents)
}

/// Shared source-order effective member state.
pub type Members = rpds::RedBlackTreeMapSync<DeclarationId, Arc<BTreeMap<String, DeclarationId>>>;
/// The already cycle-checked inheritance graph, shared by all member phase requests.
#[derive(Clone, Debug)]
pub struct Inheritance(Arc<DiGraph<DeclarationId, ()>>);
impl PartialEq for Inheritance {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
            || self.0.node_weights().eq(other.0.node_weights())
                && self
                    .0
                    .raw_edges()
                    .iter()
                    .map(|e| (e.source(), e.target()))
                    .eq(other.0.raw_edges().iter().map(|e| (e.source(), e.target())))
    }
}
impl Inheritance {
    pub(super) fn new(graph: DiGraph<DeclarationId, ()>) -> Self {
        Self(Arc::new(graph))
    }
    pub(super) fn id(&self, node: petgraph::graph::NodeIndex) -> DeclarationId {
        self.0[node]
    }
    fn reaches(&self, from: DeclarationId, to: DeclarationId) -> bool {
        let from = self.0.node_indices().find(|n| self.0[*n] == from);
        let to = self.0.node_indices().find(|n| self.0[*n] == to);
        match (from, to) {
            (Some(a), Some(b)) => petgraph::algo::has_path_connecting(self.0.as_ref(), a, b, None),
            _ => false,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
struct MemberContract {
    name: String,
    kind: pse_model::generated::enums::ModelingDeclarationKind,
    is_override: bool,
    default: bool,
    domain: Option<pse_model::generated::enums::ModelingVariableDomain>,
    indices: Vec<String>,
    binding: bool,
    equation: bool,
    connection: bool,
    function: Option<(Vec<(String, Type)>, Type)>,
    requires_implementation: bool,
    attribute_untyped: bool,
    excluded_kind_member: bool,
}
impl MemberContract {
    fn new(p: &CheckedPackage, row: &Declaration) -> Self {
        Self {
            name: row.name.clone(),
            kind: row.value.kind,
            is_override: row.is_override,
            default: row
                .value
                .function
                .as_ref()
                .is_some_and(|f| f.body.is_some() || f.external.is_some())
                || row
                    .value
                    .binding
                    .as_ref()
                    .is_some_and(|b| b.expression.is_some() || b.defined_by.is_some()),
            domain: row.value.binding.as_ref().and_then(|b| b.domain),
            indices: row.value.binding.as_ref().map_or_else(Vec::new, |b| {
                b.indices.iter().map(|i| i.domain.clone()).collect()
            }),
            binding: row.value.binding.is_some(),
            equation: row.value.equation.is_some(),
            connection: row.value.connection.is_some(),
            function: p
                .functions
                .get(&row.declaration_id)
                .map(|f| (f.arguments.clone(), f.result.clone())),
            requires_implementation: row
                .value
                .function
                .as_ref()
                .is_some_and(|f| f.body.is_none() && f.external.is_none()),
            attribute_untyped: row
                .value
                .attribute
                .as_ref()
                .is_some_and(|a| a.r#type.is_none()),
            excluded_kind_member: row.value.envelope.is_some() || row.value.requirement.is_some(),
        }
    }
}
/// Exact source contracts and lexical environments consumed by one effective-member check.
#[derive(Clone, Debug, PartialEq)]
pub struct MemberInput {
    id: DeclarationId,
    namespace: Namespace,
    types: Types,
    members: Members,
    inheritance: Inheritance,
    rows: BTreeMap<DeclarationId, MemberContract>,
    own: BTreeMap<String, DeclarationId>,
    inherited: BTreeMap<String, DeclarationId>,
    contracts: BTreeSet<DeclarationId>,
    parent: Option<DeclarationId>,
    kind_base: Option<DeclarationId>,
}
impl MemberInput {
    /// Owned member contract metadata, excluding the separately precharged shared trees.
    pub fn owned_contract_bytes(&self) -> usize {
        self.rows
            .values()
            .map(|row| {
                size_of::<MemberContract>()
                    + row.name.capacity()
                    + 256
                    + row
                        .indices
                        .iter()
                        .map(|name| name.capacity() + 64)
                        .sum::<usize>()
                    + row.function.as_ref().map_or(0, |(arguments, result)| {
                        crate::extent::ty(result)
                            + arguments
                                .iter()
                                .map(|(name, ty)| name.capacity() + crate::extent::ty(ty) + 128)
                                .sum::<usize>()
                    })
            })
            .sum::<usize>()
            + self
                .own
                .keys()
                .chain(self.inherited.keys())
                .map(|name| name.capacity() + 128)
                .sum::<usize>()
            + self.contracts.len() * 128
    }
    /// Declaration owning this effective namespace.
    pub fn declaration(&self) -> DeclarationId {
        self.id
    }
    pub(super) fn new(
        p: &CheckedPackage,
        id: DeclarationId,
        namespace: Namespace,
        types: Types,
        members: Members,
        inheritance: Inheritance,
        candidate: Option<EffectiveMemberCandidates>,
    ) -> Result<Self> {
        let row = &p.declarations[&id];
        let (own, inherited, contracts) =
            if row.value.kind == pse_model::generated::enums::ModelingDeclarationKind::EntityKind {
                (
                    p.children
                        .get(&id)
                        .into_iter()
                        .flatten()
                        .map(|child| (p.declarations[child].name.clone(), *child))
                        .collect(),
                    BTreeMap::new(),
                    BTreeSet::new(),
                )
            } else {
                candidate.ok_or_else(|| invalid(id, "interface candidate namespace absent"))?
            };
        let mut ids = own
            .values()
            .chain(inherited.values())
            .copied()
            .collect::<BTreeSet<_>>();
        ids.insert(id);
        if let Some(parent) = row.parent_id {
            ids.extend(
                p.members
                    .get(&parent)
                    .into_iter()
                    .flat_map(|m| m.values())
                    .copied(),
            );
        }
        if let Some(base) = p.kinds.get(&id).and_then(|k| k.base) {
            ids.extend(
                p.members
                    .get(&base)
                    .into_iter()
                    .flat_map(|m| m.values())
                    .copied(),
            );
        }
        let rows = ids
            .into_iter()
            .map(|id| (id, MemberContract::new(p, &p.declarations[&id])))
            .collect();
        Ok(Self {
            id,
            namespace,
            types,
            members,
            inheritance,
            rows,
            own,
            inherited,
            contracts,
            parent: row.parent_id,
            kind_base: p.kinds.get(&id).and_then(|k| k.base),
        })
    }
    fn resolve(&self, at: DeclarationId, name: &str) -> Option<DeclarationId> {
        self.namespace
            .0
            .resolve_with_members(at, name, &|id| self.types.get(&id), &|id| {
                self.members.get(&id).map(Arc::as_ref)
            })
    }
    /// Known retained phase extent, conservatively counting shared environments.
    pub fn retained_bytes(&self) -> usize {
        self.namespace.retained_bytes()
            + self.rows.len() * 512
            + self
                .types
                .iter()
                .map(|(_, ty)| crate::extent::ty(ty) + 256)
                .sum::<usize>()
            + self
                .members
                .iter()
                .map(|(_, m)| m.keys().map(|name| name.capacity() + 128).sum::<usize>() + 128)
                .sum::<usize>()
    }
    /// Admit effective inheritance, overrides, domains and signatures through the existing rules.
    /// # Errors
    /// Invalid inheritance/member contracts.
    pub fn check(&self) -> Result<MemberProduct> {
        use pse_model::generated::enums::ModelingDeclarationKind as Kind;
        let id = self.id;
        let owner = &self.rows[&id];
        let mut assigned = BTreeMap::new();
        if owner.kind == Kind::EntityKind {
            let mut members = self
                .kind_base
                .and_then(|base| self.members.get(&base))
                .map(|m| m.as_ref().clone())
                .unwrap_or_default();
            for (name, child) in &self.own {
                let row = &self.rows[child];
                if row.excluded_kind_member {
                    continue;
                }
                if row.attribute_untyped
                    && let Some(ty) = members.get(name).and_then(|id| self.types.get(id))
                {
                    assigned.insert(*child, ty.clone());
                }
                members.insert(name.clone(), *child);
            }
            return Ok(MemberProduct {
                members,
                contracts: None,
                types: assigned,
            });
        }
        let mut effective = self.inherited.clone();
        let projected = self.members.get(&id);
        for (name, member) in &self.own {
            let row = &self.rows[member];
            if let Some(base) = effective.get(name).copied() {
                let original = &self.rows[&base];
                if original.default && !row.is_override {
                    return Err(invalid(*member, "replacing a default requires override"));
                }
                if original.kind != row.kind {
                    return Err(invalid(*member, "interface member role differs"));
                }
                if original.binding && row.binding {
                    if original.domain != row.domain {
                        return Err(invalid(*member, "interface member domain differs"));
                    }
                    if original.indices.len() != row.indices.len() {
                        return Err(invalid(*member, "interface member index arity differs"));
                    }
                    for (a, b) in original.indices.iter().zip(&row.indices) {
                        let expected = self.resolve(base, a).map(|target| {
                            self.inherited
                                .iter()
                                .find(|(_, id)| **id == target)
                                .and_then(|(name, _)| projected.and_then(|m| m.get(name)))
                                .copied()
                                .unwrap_or(target)
                        });
                        let actual = self.resolve(*member, b);
                        if expected != actual || expected.is_none() && a != b {
                            return Err(invalid(*member, "interface member index domains differ"));
                        }
                    }
                }
                let refined = row.kind == Kind::Child
                    && match (self.types.get(&base), self.types.get(member)) {
                        (
                            Some(Type::Interface(expected)),
                            Some(Type::Interface(actual) | Type::Definition(actual)),
                        ) => self.inheritance.reaches(*expected, *actual),
                        _ => false,
                    };
                if self.types.get(&base) != self.types.get(member) && !refined {
                    return Err(invalid(*member, "interface member physical type differs"));
                }
                if let Some(signature) = &original.function {
                    let actual = row
                        .function
                        .as_ref()
                        .ok_or_else(|| invalid(*member, "function role required"))?;
                    if signature != actual {
                        return Err(invalid(*member, "interface function signature differs"));
                    }
                }
            } else if row.is_override {
                if owner.kind == Kind::Stage {
                    let parent = self
                        .parent
                        .ok_or_else(|| invalid(*member, "stage parent absent"))?;
                    let base = self
                        .members
                        .get(&parent)
                        .and_then(|m| m.get(name))
                        .copied()
                        .ok_or_else(|| invalid(*member, "stage override target absent"))?;
                    let original = &self.rows[&base];
                    if !(original.equation && row.equation || original.connection && row.connection)
                    {
                        return Err(invalid(
                            *member,
                            "stage overrides replace equations or connections of the same kind; variable identities remain stable",
                        ));
                    }
                } else {
                    return Err(invalid(*member, "override has no inherited member"));
                }
            }
            effective.insert(name.clone(), *member);
        }
        if owner.kind == Kind::Definition {
            for member in effective.values() {
                let row = &self.rows[member];
                if row.requires_implementation {
                    return Err(invalid(id, format!("missing implementation {}", row.name)));
                }
            }
        }
        Ok(MemberProduct {
            members: effective,
            contracts: Some(self.contracts.clone()),
            types: assigned,
        })
    }
}
/// Effective members and inherited attribute types, detached from compiler handles.
#[derive(Clone, Debug, PartialEq)]
pub struct MemberProduct {
    /// Complete effective name population.
    pub members: BTreeMap<String, DeclarationId>,
    /// Inherited contracts admitted for this owner.
    pub contracts: Option<BTreeSet<DeclarationId>>,
    /// Attribute types assigned by inherited contracts.
    pub types: BTreeMap<DeclarationId, Type>,
}
impl MemberProduct {
    /// Known owned maps and inherited type values.
    pub fn retained_bytes(&self) -> usize {
        self.members
            .keys()
            .map(|name| name.capacity() + 128)
            .sum::<usize>()
            + self
                .contracts
                .as_ref()
                .map_or(0, |contracts| contracts.len() * 128)
            + self
                .types
                .values()
                .map(|ty| crate::extent::ty(ty) + 128)
                .sum::<usize>()
            + 256
    }
}

type EffectiveMemberCandidates = (
    BTreeMap<String, DeclarationId>,
    BTreeMap<String, DeclarationId>,
    BTreeSet<DeclarationId>,
);

/// Cycle-checked enclosing namespace and preliminary candidate memberships.
#[derive(Clone, Debug, PartialEq)]
pub struct InventoryProduct {
    pub(super) package: CheckedPackage,
    pub(super) inheritance: Inheritance,
    pub(super) order: Vec<petgraph::graph::NodeIndex>,
    pub(super) candidates: BTreeMap<DeclarationId, EffectiveMemberCandidates>,
}
impl InventoryProduct {
    /// Known complete enclosing phase storage.
    pub fn retained_bytes(&self) -> usize {
        self.package.retained_bytes()
            + self.order.capacity() * size_of::<petgraph::graph::NodeIndex>()
            + self.candidates.len() * 512
    }
}
/// Run complete namespace and inheritance admission through the pure semantic owner.
/// # Errors
/// Duplicate/parent/inheritance/physical namespace refusals.
pub fn inventory(
    rows: &[Declaration],
    context: &TypeContext<'_>,
    documents: &dyn Documents,
) -> Result<InventoryProduct> {
    inventory_package(rows, context, documents)
}
