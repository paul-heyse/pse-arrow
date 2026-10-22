// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete selected function products with direct typed edges and declared argument order.
use super::*;
use crate::{expression::admission::PhysicalAdmission, specialize::finite::ReductionSelection};
use std::collections::BTreeMap;

/// One actually selected hard data envelope, independent of source argument spellings.
#[derive(Clone, Debug)]
pub struct FunctionEnvelope {
    /// Actual declaring scientific owner.
    pub owner: DeclarationId,
    /// Complete expected physical type.
    pub ty: Type,
    /// Direct original guard.
    pub predicate: PredicateIx,
    /// Actual data and argument positions consumed by the guard.
    pub reads: crate::envelope::Reads,
}
/// One complete scientific applicability consumer and its resolved observations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceUse {
    /// Actual claims, permissions, unions and required dependencies.
    pub node: pse_model::applicability::Node,
    /// Ordered original predicates.
    pub predicates: Vec<PredicateIx>,
    /// Direct observation expressions.
    pub inputs: Vec<NodeIx>,
}
/// An external contract retains complete implementation, shape and derivative semantics.
#[derive(Clone, Debug)]
pub struct FunctionExternal {
    /// Actual selected implementation name, part of its public descriptor contract.
    pub implementation: String,
    /// Complete registered implementation content.
    pub revision: ContentHash,
    /// Actual bound data content.
    pub data: ContentHash,
    /// Exact selected output ordinal.
    pub output: usize,
    /// Actual derivative provenance.
    pub derivative_source: crate::external::DerivativeSource,
    /// Supplied derivative order.
    pub derivatives: u8,
    /// Declared continuity order.
    pub smoothness: u8,
    /// Actual ordered finite input gathers.
    pub shapes: Vec<crate::external::ArgumentShape>,
}
/// The complete typed call argument before finite scalar flattening.
#[derive(Clone, Debug, PartialEq)]
pub struct ArgumentLayout {
    /// Actual selected declared type, including an empty indexed argument's axes.
    pub ty: Type,
    /// Half-open scalar formal range. Static and empty indexed arguments have zero width.
    pub start: usize,
    /// Number of scalar formals contributed by this actual argument.
    pub len: usize,
    /// Ordered actual tuples; the range ordinal corresponds to this same tuple order.
    pub tuples: Vec<Vec<crate::specialize::Value>>,
    /// Complete ordered actual axis occurrences, including ragged prefix contexts.
    pub domains: Vec<(usize, Vec<crate::specialize::Value>, Carrier)>,
    /// Complete actual static meaning, never a rendered parameter name.
    pub static_value: Option<crate::specialize::Value>,
}
/// A consumed selection's complete semantic payload, independently of source spelling.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionMeaning {
    /// Actual authored dependency callable.
    pub dependencies: DeclarationId,
    /// Complete roots, context and admitted closure edges.
    pub closure: crate::scientific_selection::SelectionClosure,
    /// Exact admitted selected records; every required record must be present.
    pub records: Vec<crate::entity::Record>,
}
impl SelectionMeaning {
    pub(crate) fn capture(
        selections: crate::scientific_selection::Selections,
        package: &crate::CheckedPackage,
    ) -> Result<Vec<Self>> {
        let mut meanings = BTreeMap::new();
        for (occurrence, closure) in selections {
            let records = closure
                .records
                .iter()
                .map(|value| {
                    let crate::specialize::Value::Entity { id, .. } = value else {
                        return Err(invalid(
                            occurrence.declaration,
                            "scientific closure contains a nonrecord",
                        ));
                    };
                    package.record(*id).cloned().ok_or_else(|| {
                        invalid(
                            occurrence.declaration,
                            "required selected scientific record is absent",
                        )
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let key = (
                occurrence.dependencies,
                closure.roots.clone(),
                closure.context.clone(),
                closure.records.clone(),
                closure.edges.clone(),
            );
            let meaning = Self {
                dependencies: occurrence.dependencies,
                closure,
                records,
            };
            if meanings.get(&key).is_some_and(|prior| prior != &meaning) {
                return Err(invalid(
                    occurrence.declaration,
                    "one scientific selection has conflicting actual records",
                ));
            }
            meanings.insert(key, meaning);
        }
        Ok(meanings.into_values().collect())
    }
    fn frame(&self, hash: &mut FramedHasher) {
        hash.id(&self.dependencies.as_id());
        self.closure.frame(hash);
        hash.u64(self.records.len() as u64);
        for record in &self.records {
            hash.id(&record.kind.as_id())
                .id(&record.origin.as_id())
                .u64(record.values.len() as u64);
            for (name, value) in &record.values {
                hash.str(name);
                value.frame(hash);
            }
            hash.u64(record.uncertainties.len() as u64);
            for (name, uncertainty) in &record.uncertainties {
                hash.str(name)
                    .str(uncertainty.kind.as_str())
                    .u64(uncertainty.magnitude.to_bits());
            }
        }
    }
}
/// Executable function meaning; no normalized textual body is retained here.
#[derive(Clone, Debug)]
pub struct FunctionProduct {
    /// Complete consumed semantic identity, issued once by this producer.
    pub identity: ContentHash,
    /// Reached applicability effects, prepared once from this owner and child products.
    pub has_applicability_effects: bool,
    /// Original source owner for diagnostics and declared physical authorizations.
    pub declaration: DeclarationId,
    /// One immutable direct syntax owner.
    pub syntax: ResolvedSyntax,
    /// Complete actual physical support, prepared before final function identity.
    pub physical: Arc<PhysicalSupport>,
    /// Ordered actual formal slots, including unused declared arguments.
    pub formals: Vec<SymbolIx>,
    /// Ordered nominal call-boundary arguments, before any declared body authorization.
    pub signature: Vec<Type>,
    /// Actual argument shapes and static selections before scalar flattening.
    pub arguments: Vec<ArgumentLayout>,
    /// Complete selected scientific semantics with no source text.
    pub selections: Vec<SelectionMeaning>,
    /// Actual contextual literal contracts issued during specialization.
    pub literals: BTreeMap<NodeIx, Type>,
    /// Complete nominal declared result.
    pub result: Type,
    /// Direct body, absent only for an external or synthesized finite-reduction implementation.
    pub body: Option<NodeIx>,
    /// Actual strict form guard.
    pub validity: Option<PredicateIx>,
    /// Original hard data guards in source order.
    pub envelopes: Vec<FunctionEnvelope>,
    /// Complete declared evidence consumers.
    pub applicability: Vec<EvidenceUse>,
    /// Argument effects that precede the returned value.
    pub prerequisites: Vec<usize>,
    /// Source-owned physical obligations attached to the exact selected typed operations.
    pub admissions: BTreeMap<NodeIx, PhysicalAdmission>,
    /// Synthesized finite-reduction admission, distinct even when its carrier is empty.
    pub reduction_admission: Option<PhysicalAdmission>,
    /// Independently admitted reduction prototype and physical domain.
    pub reduction: Option<crate::FiniteReduction>,
    /// Exact declared carrier and actually selected members, when source specialization issued it.
    pub reduction_selection: Option<ReductionSelection>,
    /// Declared physical role authorization, never inferred from a numerical signature.
    pub physical_operation: Option<OperationMeaning>,
    /// Exact quantity-owned nominal authorization issued by the source operation producer.
    pub output_boundary: Option<pse_quantity::AdmittedOutputBoundary>,
    /// Exact declared body views at each nominal scalar input ordinal.
    pub body_inputs: Vec<Option<BodyInputSupport>>,
    /// Form-layer data and argument lineage.
    pub validity_reads: crate::envelope::Reads,
    /// Actual foreign implementation contract.
    pub external: Option<FunctionExternal>,
    /// Actual claimed piecewise continuity.
    pub continuity: Option<u8>,
}
impl FunctionProduct {
    /// Seal a graph already emitted by the actual specialization constructor.
    /// This validates ownership and framing; it grants no physical admission authority.
    pub(crate) fn seal(mut self) -> Result<Arc<Self>> {
        if self.formals.len() != self.signature.len() {
            return Err(invalid(
                self.declaration,
                "constructed function signature/formal arity differs",
            ));
        }
        for (ordinal, slot) in self.formals.iter().enumerate() {
            let root = self.syntax.outputs.get(ordinal).ok_or_else(|| {
                invalid(
                    self.declaration,
                    "constructed function lost its ordered formal root",
                )
            })?;
            if !matches!(self.syntax.nodes.get(*root),Some(Node::Reference(Reference::Symbol(actual))) if actual==slot)
            {
                return Err(invalid(
                    self.declaration,
                    "constructed function formal root differs from its ordered scalar signature",
                ));
            }
        }
        for slot in &self.formals {
            if self.syntax.symbols.at(*slot).is_none() {
                return Err(invalid(
                    self.declaration,
                    "constructed function formal is outside its local owner",
                ));
            }
        }
        if self.body.is_none() && self.external.is_none() && self.reduction.is_none() {
            return Err(invalid(
                self.declaration,
                "constructed function lacks its selected implementation",
            ));
        }
        if self.external.is_some() && (self.body.is_some() || self.reduction.is_some()) {
            return Err(invalid(
                self.declaration,
                "external function has a second executable implementation",
            ));
        }
        if self.reduction.is_some()
            && (self.body.is_some()
                || self.reduction_admission.is_none()
                || self.reduction_selection.is_none())
        {
            return Err(invalid(
                self.declaration,
                "finite reduction lacks its actual prototype/admission/selection product",
            ));
        }
        self.has_applicability_effects = !self.applicability.is_empty()
            || !self.prerequisites.is_empty()
            || construction::effects(
                &self.syntax.arena,
                &self.syntax.symbols,
                &self.syntax.outputs,
                &self.syntax.validity,
            )?;
        self.identity = self.frame_identity()?;
        Ok(Arc::new(self))
    }
    /// Actual formal root issued in the function's declared scalar order.
    pub fn formal_root(&self, ordinal: usize) -> Result<NodeIx> {
        self.formals
            .get(ordinal)
            .and_then(|_| self.syntax.outputs.get(ordinal))
            .copied()
            .ok_or_else(|| {
                invalid(
                    self.declaration,
                    "formal root is outside the actual ordered signature",
                )
            })
    }
    fn frame_identity(&self) -> Result<ContentHash> {
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedFunctionV1);
        let layout = self.syntax.canonical_layout()?;
        // Form validity retains its actual declaring owner. Cross-instance reuse of
        // this same contract remains lawful; cross-declaration guarded sharing would
        // first need an explicit attributed-validity role in the numerical stage.
        hash.bool(self.validity.is_some());
        if self.validity.is_some() {
            hash.id(&self.declaration.as_id());
        }
        hash.hash(&layout.identity);
        self.physical.frame(&mut hash);
        hash.u64(self.arguments.len() as u64);
        for argument in &self.arguments {
            frame_type(&argument.ty, &mut hash);
            hash.u64(argument.start as u64)
                .u64(argument.len as u64)
                .u64(argument.tuples.len() as u64);
            for tuple in &argument.tuples {
                hash.u64(tuple.len() as u64);
                for value in tuple {
                    value.frame(&mut hash);
                }
            }
            hash.u64(argument.domains.len() as u64);
            for (position, prefix, carrier) in &argument.domains {
                hash.u64(*position as u64).u64(prefix.len() as u64);
                for value in prefix {
                    value.frame(&mut hash);
                }
                frame_type(&carrier.element, &mut hash);
                hash.u64(carrier.members.len() as u64);
                for value in carrier.members.iter() {
                    value.frame(&mut hash);
                }
                match carrier.kind {
                    CarrierKind::OrderedSet => {
                        hash.str("ordered-set");
                    }
                    CarrierKind::Continuous { domain } => {
                        hash.str("continuous").id(&domain.as_id());
                    }
                }
            }
            hash.bool(argument.static_value.is_some());
            if let Some(value) = &argument.static_value {
                value.frame(&mut hash);
            }
        }
        hash.u64(self.selections.len() as u64);
        for meaning in &self.selections {
            meaning.frame(&mut hash);
        }
        hash.u64(self.signature.len() as u64);
        for ty in &self.signature {
            frame_type(ty, &mut hash);
        }
        hash.u64(self.formals.len() as u64);
        for slot in &self.formals {
            frame_type(&self.syntax.symbols[*slot].ty, &mut hash);
        }
        frame_type(&self.result, &mut hash);
        hash.u64(self.prerequisites.len() as u64);
        for ordinal in &self.prerequisites {
            if *ordinal >= self.formals.len() {
                return Err(invalid(
                    self.declaration,
                    "function prerequisite outside actual signature",
                ));
            }
            hash.u64(*ordinal as u64);
        }
        if self.body_inputs.len() != self.formals.len() {
            return Err(invalid(
                self.declaration,
                "function body-input authorization arity differs",
            ));
        }
        hash.u64(self.body_inputs.len() as u64);
        for input in &self.body_inputs {
            hash.bool(input.is_some());
            if let Some(input) = input {
                hash.id(&input.relationship.identity());
                input.boundary.frame(&mut hash);
            }
        }
        self.validity_reads.frame(&mut hash);
        hash.u64(self.envelopes.len() as u64);
        for guard in &self.envelopes {
            hash.id(&guard.owner.as_id());
            frame_type(&guard.ty, &mut hash);
            guard.reads.frame(&mut hash);
        }
        hash.u64(self.applicability.len() as u64);
        for usage in &self.applicability {
            usage.node.frame(&mut hash);
            hash.u64(usage.predicates.len() as u64)
                .u64(usage.inputs.len() as u64);
        }
        hash.bool(self.continuity.is_some());
        if let Some(order) = self.continuity {
            hash.u64(order.into());
        }
        hash.bool(self.physical_operation.is_some());
        if let Some(operation) = &self.physical_operation {
            operation.frame(&layout, &mut hash, self.declaration)?;
        }
        hash.bool(self.output_boundary.is_some());
        if let Some(boundary) = &self.output_boundary {
            boundary.frame(&mut hash);
        }
        hash.bool(self.reduction.is_some());
        if let Some(reduction) = &self.reduction {
            hash.str(reduction.kind.as_str())
                .bool(reduction.domain.is_some());
            if let Some(domain) = reduction.domain {
                hash.id(&domain.as_id());
            }
            reduction.prototype.frame(&mut hash);
        }
        hash.bool(self.reduction_selection.is_some());
        if let Some(selection) = &self.reduction_selection {
            frame_type(&selection.carrier.element, &mut hash);
            hash.u64(selection.carrier.members.len() as u64);
            for value in selection.carrier.members.iter() {
                value.frame(&mut hash);
            }
            hash.u64(selection.selected.len() as u64);
            for value in selection.selected.iter() {
                value.frame(&mut hash);
            }
        }
        hash.bool(self.external.is_some());
        if let Some(external) = &self.external {
            hash.str(&external.implementation)
                .hash(&external.revision)
                .hash(&external.data)
                .u64(external.output as u64)
                .str(external.derivative_source.as_str())
                .u64(external.derivatives.into())
                .u64(external.smoothness.into());
            hash.u64(external.shapes.len() as u64);
            for shape in &external.shapes {
                hash.id(&shape.argument)
                    .u64(shape.start as u64)
                    .u64(shape.axes.len() as u64);
                for axis in &shape.axes {
                    hash.id(axis);
                }
                hash.u64(shape.coordinates.len() as u64);
                for tuple in &shape.coordinates {
                    hash.u64(tuple.len() as u64);
                    for value in tuple {
                        hash.id(value);
                    }
                }
            }
        }
        // Attach proofs to the exact operation ordinal issued by this same framing walk.
        let ordinals = layout
            .nodes
            .iter()
            .copied()
            .enumerate()
            .map(|(ordinal, slot)| (slot, ordinal))
            .collect::<BTreeMap<_, _>>();
        let mut admissions = self
            .admissions
            .iter()
            .map(|(slot, admission)| {
                ordinals
                    .get(slot)
                    .copied()
                    .map(|ordinal| (ordinal, admission))
                    .ok_or_else(|| {
                        invalid(
                            self.declaration,
                            "function admission is outside its reached typed graph",
                        )
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        admissions.sort_by_key(|(ordinal, _)| *ordinal);
        hash.u64(admissions.len() as u64);
        for (ordinal, admission) in admissions {
            hash.u64(ordinal as u64);
            admission.frame(&mut hash);
        }
        let mut literals = self
            .literals
            .iter()
            .map(|(slot, ty)| {
                ordinals
                    .get(slot)
                    .copied()
                    .map(|ordinal| (ordinal, ty))
                    .ok_or_else(|| {
                        invalid(
                            self.declaration,
                            "function literal contract is outside its reached typed graph",
                        )
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        literals.sort_by_key(|(ordinal, _)| *ordinal);
        hash.u64(literals.len() as u64);
        for (ordinal, ty) in literals {
            hash.u64(ordinal as u64);
            frame_type(ty, &mut hash);
        }
        hash.bool(self.reduction_admission.is_some());
        if let Some(admission) = &self.reduction_admission {
            admission.frame(&mut hash);
        }
        Ok(hash.finish_hash())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn selected(body: &str, formal_names: (&str, &str)) -> crate::SpecializedModel {
        let (registry, _) = crate::kernel_types::physical();
        let facts = pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
        let scope = crate::PhysicalScope::default();
        let context = crate::TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &registry,
            preconditions: &facts,
            scope: &scope,
        };
        let text = format!(
            "package p {{fn pick({}:Scalar,{}:Scalar)->Scalar={body}; def Root {{var x:Scalar; var y:Scalar; eq e:pick(x,y)==1;}}}}",
            formal_names.0, formal_names.1
        );
        let package =
            crate::check(&crate::kernel_types::try_source(&text).unwrap(), &context).unwrap();
        crate::specialize(
            &package,
            package.names["p.Root"],
            SemanticId::from_bytes([9; 16]).into(),
            &crate::Bindings::default(),
            crate::Limits::default(),
        )
        .unwrap()
    }
    fn identity(body: &str, names: (&str, &str)) -> ContentHash {
        let model = selected(body, names);
        model
            .resolved
            .functions
            .iter()
            .find(|selection| selection.product.formals.len() == 2)
            .unwrap()
            .product
            .identity
    }
    #[test]
    fn selected_function_products_preserve_formal_ordinals_and_ignore_source_spelling() {
        assert_eq!(
            identity("a+b", ("a", "b")),
            identity("left+right", ("left", "right"))
        );
        assert_ne!(identity("a", ("a", "b")), identity("b", ("a", "b")));
        assert_ne!(identity("a+b", ("a", "b")), identity("a-b", ("a", "b")));
    }
    #[test]
    fn form_validity_keeps_declaring_owner_while_unguarded_aliases_share() {
        let plain = selected("a+b", ("a", "b"));
        let original = plain
            .resolved
            .functions
            .iter()
            .find(|selection| selection.product.formals.len() == 2)
            .unwrap()
            .product
            .as_ref();
        let mut alias = original.clone();
        alias.declaration = DeclarationId::from_id(SemanticId::from_bytes([77; 16]));
        assert_eq!(
            original.frame_identity().unwrap(),
            alias.frame_identity().unwrap()
        );
        let (registry, _) = crate::kernel_types::physical();
        let facts = pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
        let scope = crate::PhysicalScope::default();
        let context = crate::TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &registry,
            preconditions: &facts,
            scope: &scope,
        };
        let source = "package p {fn pick(a:Scalar,b:Scalar)->Scalar valid(a>0)=a+b; def Root {var x:Scalar;var y:Scalar;eq e:pick(x,y)==1;}}";
        let package =
            crate::check(&crate::kernel_types::try_source(source).unwrap(), &context).unwrap();
        let model = crate::specialize(
            &package,
            package.names["p.Root"],
            SemanticId::from_bytes([9; 16]).into(),
            &crate::Bindings::default(),
            crate::Limits::default(),
        )
        .unwrap();
        let guarded = model
            .resolved
            .functions
            .iter()
            .find(|selection| selection.product.formals.len() == 2)
            .unwrap()
            .product
            .as_ref();
        assert!(guarded.validity.is_some());
        let mut alias = guarded.clone();
        alias.declaration = DeclarationId::from_id(SemanticId::from_bytes([77; 16]));
        assert_ne!(
            guarded.frame_identity().unwrap(),
            alias.frame_identity().unwrap()
        );
        let mut moved = guarded.clone();
        for source in moved.syntax.node_sources.values_mut() {
            source.span.start = source.span.start.saturating_add(1);
            source.span.end = source.span.end.saturating_add(1);
        }
        assert_eq!(
            guarded.frame_identity().unwrap(),
            moved.frame_identity().unwrap(),
            "current source ranges remain in the receiver occurrence owner"
        );
    }
    #[test]
    fn selected_function_admissions_map_to_the_actual_lexical_inventory() {
        let model = selected("sum+sum where sum=a+b", ("a", "b"));
        let product = &model
            .resolved
            .functions
            .iter()
            .find(|selection| selection.product.formals.len() == 2)
            .unwrap()
            .product;
        assert!(product.admissions.len() >= 2);
        assert!(product.admissions.keys().all(|slot| {
            product.syntax.node_sources[slot]
                .coordinate
                .is_some_and(|coordinate| coordinate.stage == InventoryStage::Original)
        }));
        let roots = product.syntax.select(vec![product.body.unwrap()], vec![]);
        assert!(Arc::ptr_eq(&roots.arena, &product.syntax.arena));
        assert!(std::ptr::eq(
            &roots.symbols[product.formals[0]].ty,
            &product.syntax.symbols[product.formals[0]].ty
        ));
    }
}
