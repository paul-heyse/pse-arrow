// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact expression inputs and declaration-owned physical admission products.
use super::*;
use crate::expression::{
    self,
    admission::{AdmissionRecorder, ExpressionAdmissions, Owner},
};

/// One immutable candidate and its shared lexical/type/data comparison population.
/// Arithmetic fields are compared through their actual consumed owners below.
#[derive(Clone, Debug)]
pub struct Environment {
    package: Arc<CheckedPackage>,
    metadata: Arc<CheckedPackage>,
    declarations: Arc<BTreeMap<DeclarationId, Declaration>>,
    bodies: Arc<BTreeMap<DeclarationId, Arc<Body>>>,
}
impl PartialEq for Environment {
    fn eq(&self, other: &Self) -> bool {
        (Arc::ptr_eq(&self.metadata, &other.metadata) || self.metadata == other.metadata)
            && (Arc::ptr_eq(&self.declarations, &other.declarations)
                || self.declarations == other.declarations)
    }
}
impl Environment {
    /// Freeze once per candidate. No full-package clone is made per declaration.
    /// # Errors
    /// Resource admission refuses the candidate.
    pub fn new(package: &CheckedPackage, documents: &dyn Documents) -> Result<Self> {
        documents.preflight(
            DeclarationId::from_id(SemanticId::NIL),
            package.retained_bytes().saturating_mul(8),
        )?;
        let mut metadata = package.clone();
        metadata.expressions.clear();
        metadata.declarations.clear();
        metadata.selection_closures.clear();
        metadata.selection_index = SelectionIndex::default();
        for function in metadata.functions.values_mut() {
            function.body = None;
            function.validity = None;
            function.applicability.clear();
            function.applicability_uses.clear();
            function.physical_admissions.clear();
            function.envelopes.clear();
            function.validity_reads = Default::default();
            // The generated external body is owned by the selected function; its other
            // authored contract fields remain in the exact declaration metadata.
            function.external = None;
        }
        let mut declarations = BTreeMap::new();
        let mut bodies = BTreeMap::new();
        for (id, row) in &package.declarations {
            let mut row = row.clone();
            row.source_start = 0;
            row.source_end = 0;
            // Restore the generated typed row before comparison: JSON number equality
            // is not the semantic owner for scientific cells (including signed zero).
            // A value that cannot round trip remains completely conservative.
            let normalized = match serde_json::to_value(&row) {
                Ok(mut value) => {
                    for (key, field) in package.occurrences_of(*id) {
                        let path = key.role.split('.').collect::<Vec<_>>();
                        if let Some(payload) = value.get_mut("value") {
                            mask_field(payload, &path, key.position, &field.text, false);
                        }
                    }
                    match serde_json::from_value(value) {
                        Ok(row) => row,
                        Err(_) => row.clone(),
                    }
                }
                Err(_) => row.clone(),
            };
            declarations.insert(*id, normalized);
            let fields = package
                .occurrences_of(*id)
                .map(|(key, field)| {
                    let mut field = field.clone();
                    field.source = pse_authoring::SourceSpan::head(SemanticId::NIL);
                    field.physical_admissions.clear();
                    (key.clone(), field)
                })
                .collect();
            let function = package.functions.get(id).map(|function| {
                let mut function = function.clone();
                function.physical_admissions.clear();
                function
            });
            bodies.insert(
                *id,
                Arc::new(Body {
                    declaration: row,
                    fields,
                    function,
                }),
            );
        }
        Ok(Self {
            package: Arc::new(package.clone()),
            metadata: Arc::new(metadata),
            declarations: Arc::new(declarations),
            bodies: Arc::new(bodies),
        })
    }
    /// Include each actual shared candidate root once, including older snapshots
    /// still borrowed by unchanged input phases.
    pub fn collect_retained_inputs<'a>(&'a self, census: &mut RetainedInputs<'a>) {
        use pse_model::HeapUsage;
        if census.include_shared(&self.package) { census.add(self.package.retained_bytes()); }
        if census.include_shared(&self.metadata) { census.add(self.metadata.retained_bytes()); }
        if census.include_shared(&self.declarations) {
            census.add(self.declarations.values().map(|row| row.owned_bytes() + 128).sum());
        }
        if census.include_shared(&self.bodies) {
            census.add(self.bodies.len().saturating_mul(size_of::<(DeclarationId, Arc<Body>)>() + 128));
            for body in self.bodies.values() { body.collect_retained_inputs(census); }
        }
    }
    /// Compare the shared enclosing population once per candidate, then reuse its
    /// immutable owner so per-declaration input equality does not rescan N rows N times.
    pub fn reuse_metadata(&mut self, previous: &Self) {
        if self == previous {
            self.metadata = previous.metadata.clone();
            self.declarations = previous.declarations.clone();
        }
        for (id, body) in Arc::make_mut(&mut self.bodies) {
            if let Some(prior) = previous.bodies.get(id)
                && body == prior
            {
                *body = prior.clone();
            }
        }
    }
}
// Only an exact parser-owned role/ordinal and byte-equal source string can be masked.
// Flattened multidimensional roles are deliberately left conservative when the parser
// role does not name a unique JSON array coordinate. Labels, types and scientific cells
// are never removed by text search or by a guessed expression-field name.
fn mask_field(
    value: &mut serde_json::Value,
    path: &[&str],
    position: usize,
    text: &str,
    indexed: bool,
) -> bool {
    match value {
        serde_json::Value::String(source) if path.is_empty() && source == text => {
            source.clear();
            true
        }
        serde_json::Value::Object(fields) if !path.is_empty() => fields
            .get_mut(path[0])
            .is_some_and(|next| mask_field(next, &path[1..], position, text, indexed)),
        serde_json::Value::Array(values) => {
            if let Some(index) = path.first().and_then(|part| part.parse::<usize>().ok()) {
                values
                    .get_mut(index)
                    .is_some_and(|next| mask_field(next, &path[1..], position, text, indexed))
            } else if !indexed {
                values
                    .get_mut(position)
                    .is_some_and(|next| mask_field(next, path, position, text, true))
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Exact mathematical fields reachable from one declaration, plus the shared
/// enclosing namespace population. Absence, ordered imports and type changes are
/// represented by the complete metadata; source attribution remains a parse/bind input.
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    environment: Environment,
    declaration: DeclarationId,
    owners: BTreeSet<DeclarationId>,
    bodies: BTreeMap<DeclarationId, Arc<Body>>,
}
#[derive(Debug, PartialEq)]
struct Body {
    declaration: Declaration,
    fields: BTreeMap<OccurrenceKey, CheckedExpression>,
    function: Option<Function>,
}
/// One owner's bound references, including generated function roots whose syntax
/// has no separately authored occurrence. Resolution is performed by the phase owner.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceInput {
    environment: Environment,
    declaration: DeclarationId,
    body: Arc<Body>,
}
impl Body {
    fn collect_retained_inputs<'a>(self: &'a Arc<Self>, census: &mut RetainedInputs<'a>) {
        use pse_model::HeapUsage;
        if census.include_shared(self) {
            census.add(self.declaration.owned_bytes());
            census.add(self.fields.iter().map(|(key, field)| expression::occurrences::field_retained_bytes(key, field, true)).sum());
            census.add(self.function.as_ref().map_or(0, |function| size_of::<Function>() + crate::extent::function(function)));
        }
    }
}
impl Environment {
    /// Capture the one exact source/body owner consumed by reference resolution.
    /// # Errors
    /// The declaration is absent from this candidate.
    pub fn references(&self, declaration: DeclarationId) -> Result<ReferenceInput> {
        let body = self
            .bodies
            .get(&declaration)
            .cloned()
            .ok_or_else(|| invalid(declaration, "expression reference owner absent"))?;
        Ok(ReferenceInput {
            environment: self.clone(),
            declaration,
            body,
        })
    }
}
impl ReferenceInput {
    /// Include the exact environment and source body retained by this input.
    pub fn collect_retained_inputs<'a>(&'a self, census: &mut RetainedInputs<'a>) {
        self.environment.collect_retained_inputs(census);
        self.body.collect_retained_inputs(census);
    }
    /// Resolve generated roots through the same complete lexical namespace as normal
    /// occurrence binding. The caller retains this actual work as an incremental phase.
    pub fn resolve(&self) -> BTreeSet<DeclarationId> {
        let package = &self.environment.package;
        let id = self.declaration;
        let mut dependencies = BTreeSet::new();
        dependencies.extend(self.body.declaration.parent_id);
        for field in self.body.fields.values() {
            dependencies.extend(field.dependencies.iter().copied());
            dependencies.extend(field.context.iter().copied());
        }
        if let Some(function) = &self.body.function {
            for root in expression::inventory::function(function) {
                for path in expression::source_paths(root.expression) {
                    for end in (1..=path.segments.len()).rev() {
                        if let Some(target) = package.resolve_segments(id, &path.segments[..end]) {
                            dependencies.insert(target);
                            break;
                        }
                    }
                }
            }
        }
        dependencies
    }
}
impl Input {
    /// Include this input's own dependency maps and each shared source owner once.
    pub fn collect_retained_inputs<'a>(&'a self, census: &mut RetainedInputs<'a>) {
        self.environment.collect_retained_inputs(census);
        census.add(self.owners.len().saturating_mul(size_of::<DeclarationId>() + 128));
        census.add(self.bodies.len().saturating_mul(size_of::<(DeclarationId, Arc<Body>)>() + 128));
        for body in self.bodies.values() { body.collect_retained_inputs(census); }
    }
    /// Capture actual bound dependencies and lexical ancestor guards before retaining
    /// any cloned body inventory. Generated function roots use the same source resolver.
    /// # Errors
    /// The candidate's precharged resource allowance is exhausted.
    pub fn new(
        environment: &Environment,
        declaration: DeclarationId,
        references: &BTreeMap<DeclarationId, Arc<BTreeSet<DeclarationId>>>,
        documents: &dyn Documents,
    ) -> Result<Self> {
        let mut pending = vec![declaration];
        let mut owners = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !owners.insert(id) {
                continue;
            }
            if let Some(dependencies) = references.get(&id) {
                pending.extend(dependencies.iter().copied());
            }
        }
        // Each normalized body has one shared owner in this candidate. Transitive
        // dependency chains retain handles, never repeated deep body/prefix clones.
        documents.preflight(declaration, owners.len().saturating_mul(256))?;
        let bodies = owners
            .iter()
            .filter_map(|id| environment.bodies.get(id).map(|body| (*id, body.clone())))
            .collect();
        Ok(Self {
            environment: environment.clone(),
            declaration,
            owners,
            bodies,
        })
    }
    /// Actual declaration identity, never a body hash or database-local query id.
    pub fn declaration(&self) -> DeclarationId {
        self.declaration
    }
    /// Run the existing pure checker against the retained exact candidate, with one
    /// recorder for its consumed field/function owners.
    /// # Errors
    /// Exactly the existing expression and physical-admission refusals.
    pub fn check(&self, context: &TypeContext<'_>) -> Result<Product> {
        let recorder =
            AdmissionRecorder::for_declarations(&self.environment.package, &self.owners)?;
        let context = TypeContext {
            admissions: Some(&recorder),
            formula_authority: context.formula_authority.clone(),
            quantities: context.quantities,
            preconditions: context.preconditions,
            scope: context.scope,
        };
        expression::check_declaration(
            self.environment.package.as_ref(),
            &context,
            self.declaration,
        )?;
        Ok(Product(recorder.into_products()))
    }
}
/// Immutable physical admissions produced by one declaration's actual type checking.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Product(BTreeMap<Owner, ExpressionAdmissions>);
impl Product {
    /// Conservative retained products, including full physical schemes and inference.
    pub fn retained_bytes(&self) -> usize {
        self.0
            .iter()
            .map(|(owner, products)| {
                256 + match owner {
                    Owner::Field(key) => key.role.capacity(),
                    Owner::Function(_) => 0,
                } + products
                    .values()
                    .map(|value| value.retained_bytes() + 128)
                    .sum::<usize>()
            })
            .sum()
    }
    /// Merge contextual consumers exactly as the original complete recorder did.
    /// # Errors
    /// A shared occurrence is admitted under conflicting physical contexts.
    pub fn merge(&mut self, other: Self, at: DeclarationId) -> Result<()> {
        for (owner, products) in other.0 {
            let entries = self.0.entry(owner).or_default();
            for (coordinate, admission) in products {
                if let Some(previous) = entries.get(&coordinate) {
                    if previous != &admission {
                        return Err(invalid(
                            at,
                            "one expression occurrence has conflicting physical contexts",
                        ));
                    }
                } else {
                    entries.insert(coordinate, admission);
                }
            }
        }
        Ok(())
    }
    /// Install only after every declaration and shared physical context has admitted.
    /// # Errors
    /// An admission refers to an owner outside the checked inventory.
    pub fn install(self, package: &mut CheckedPackage) -> Result<()> {
        for (owner, products) in self.0 {
            match owner {
                Owner::Field(key) => {
                    package
                        .expressions
                        .get_mut(&key)
                        .ok_or_else(|| invalid(key.declaration, "checked field inventory absent"))?
                        .physical_admissions = products
                }
                Owner::Function(id) => {
                    package
                        .functions
                        .get_mut(&id)
                        .ok_or_else(|| invalid(id, "checked function inventory absent"))?
                        .physical_admissions = products
                }
            }
        }
        Ok(())
    }
}
