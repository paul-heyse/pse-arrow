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
            // The generated typed projection follows parser-owned field roles and
            // ordinals. Unknown/mismatching coordinates retain their complete value;
            // scientific cells never cross a JSON number representation.
            let mut normalized = row.clone();
            let expected = package
                .occurrences_of(*id)
                .map(|(key, field)| ((key.role.as_str(), key.position), field.text.as_str()))
                .collect::<BTreeMap<_, _>>();
            normalized
                .value
                .mask_source_expressions(|_, role, position, text| {
                    expected
                        .get(&(role, position))
                        .is_some_and(|expected| *expected == text)
                });
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
        if census.include_shared(&self.package) {
            self.package.collect_retained_inputs(census);
        }
        if census.include_shared(&self.metadata) {
            self.metadata.collect_retained_inputs(census);
        }
        if census.include_shared(&self.declarations) {
            census.add(
                self.declarations
                    .values()
                    .map(|row| row.owned_bytes() + 128)
                    .sum(),
            );
        }
        if census.include_shared(&self.bodies) {
            census.add(
                self.bodies
                    .len()
                    .saturating_mul(size_of::<(DeclarationId, Arc<Body>)>() + 128),
            );
            for body in self.bodies.values() {
                body.collect_retained_inputs(census);
            }
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
            census.add(
                self.fields
                    .iter()
                    .map(|(key, field)| {
                        expression::occurrences::field_retained_bytes(key, field, true)
                    })
                    .sum(),
            );
            census.add(self.function.as_ref().map_or(0, |function| {
                size_of::<Function>() + crate::extent::function(function)
            }));
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
        census.add(
            self.owners
                .len()
                .saturating_mul(size_of::<DeclarationId>() + 128),
        );
        census.add(
            self.bodies
                .len()
                .saturating_mul(size_of::<(DeclarationId, Arc<Body>)>() + 128),
        );
        for body in self.bodies.values() {
            body.collect_retained_inputs(census);
        }
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

#[cfg(test)]
mod tests {
    fn mask(
        value: &mut pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValue,
        role: &str,
        position: usize,
        expected: &str,
    ) -> bool {
        value.mask_source_expressions(|_, selected, ordinal, text| {
            selected == role && ordinal == position && text == expected
        }) != 0
    }

    #[test]
    fn typed_mask_obeys_exact_role_ordinal_text_and_conservative_fallback() {
        let rows = crate::kernel_types::source(
            "package p {def D(first:Scalar=1,second:Scalar=2) {} fn law(x:Scalar)->Scalar=x;}",
        );
        let mut value = rows
            .iter()
            .find(|row| row.name == "D")
            .unwrap()
            .value
            .clone();
        let original = value.clone();
        assert!(!mask(&mut value, "scope.parameters.default_value", 1, "1"));
        assert!(!mask(&mut value, "scope.parameters.default_value", 2, "2"));
        assert!(!mask(&mut value, "scope.parameters.unknown", 1, "2"));
        assert!(!mask(&mut value, "unknown.expression", 0, "1"));
        assert_eq!(value, original);
        assert!(mask(&mut value, "scope.parameters.default_value", 1, "2"));
        let scope = value.scope.as_ref().unwrap();
        assert_eq!(scope.parameters[0].default_value.as_deref(), Some("1"));
        assert_eq!(scope.parameters[1].default_value.as_deref(), Some(""));
        assert_eq!(scope.parameters[1].name, "second");
        assert_eq!(
            scope.parameters[1].r#type,
            original.scope.as_ref().unwrap().parameters[1].r#type
        );
        let mut function = rows
            .iter()
            .find(|row| row.name == "law")
            .unwrap()
            .value
            .clone();
        function.function.as_mut().unwrap().applicability = vec!["first".into(), "second".into()];
        assert!(mask(&mut function, "function.body", 0, "x"));
        assert!(mask(&mut function, "function.applicability", 1, "second"));
        let law = function.function.unwrap();
        assert_eq!(law.body.as_deref(), Some(""));
        assert_eq!(law.applicability, ["first", ""]);
        assert_eq!(law.arguments[0].name, "x");
    }

    #[test]
    fn typed_mask_preserves_scientific_float_bits_and_unmasked_metadata() {
        let rows = crate::kernel_types::source(
            "package p {constant c:Scalar=-0.0 provenance(origin, role.published); fn law(x:Scalar)->Scalar=x;}",
        );
        let original = rows
            .iter()
            .find(|row| row.name == "c")
            .unwrap()
            .value
            .clone();
        for bits in [
            0.0_f64.to_bits(),
            (-0.0_f64).to_bits(),
            0x7ff8_0000_0000_0042,
        ] {
            let mut value = original.clone();
            value
                .constant
                .as_mut()
                .unwrap()
                .value
                .value
                .quantity
                .as_mut()
                .unwrap()
                .magnitude = f64::from_bits(bits);
            let before = value.clone();
            assert!(!mask(
                &mut value,
                "constant.value.value.quantity.magnitude",
                0,
                "-0.0"
            ));
            assert!(!mask(&mut value, "unrecognized", 0, "-0.0"));
            assert_eq!(
                value
                    .constant
                    .as_ref()
                    .unwrap()
                    .value
                    .value
                    .quantity
                    .as_ref()
                    .unwrap()
                    .magnitude
                    .to_bits(),
                bits
            );
            assert_eq!(value, before);
        }
        let mut first = rows
            .iter()
            .find(|row| row.name == "law")
            .unwrap()
            .value
            .clone();
        let mut second = first.clone();
        second.function.as_mut().unwrap().body = Some("x*2".into());
        assert!(mask(&mut first, "function.body", 0, "x"));
        assert!(mask(&mut second, "function.body", 0, "x*2"));
        assert_eq!(first, second);
        second.function.as_mut().unwrap().arguments[0].name = "changed".into();
        assert_ne!(first, second);
    }
}
