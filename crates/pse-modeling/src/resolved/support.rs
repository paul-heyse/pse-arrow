// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Resolver-owned physical products. Numerical construction consumes these exact admissions.
use super::*;
use pse_quantity::{CanonicalConversionPlan, PhysicalPreconditions, QuantityRegistry, ResolvedInference, ResolvedPhysicalContract, infer::{Exponent, OpRequest}, literal::LiteralContext, scheme::Scheme};

/// A literal's actual admitted representation conversion, independent of numerical construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralSupport {
    /// Resolver-issued canonical conversion, retaining its complete admitted target.
    pub conversion: CanonicalConversionPlan,
}

/// One reached node's complete semantic type and actual physical operation, when applicable.
#[derive(Clone, Debug)]
pub struct NodeSupport {
    /// Nominal refinements remain present, including actual transfer owners.
    pub ty: Type,
    /// Complete resolved physical contract, including representation and lexical indices.
    pub contract: ResolvedPhysicalContract,
    /// Actual arithmetic admission, never a claimed expected type in its place.
    pub operation: Option<ResolvedInference>,
    /// Actual literal conversion, if this node is a source literal.
    pub literal: Option<LiteralSupport>,
    /// Actual declared member boundary, separate from arithmetic and literal admission.
    pub boundary: Option<OutputSupport>,
}

/// The physical quantity owner retains the ordinary result boundary admission.
pub use pse_quantity::AdmittedContractBoundary as OutputSupport;

/// Immutable support for one exact reached resolved syntax product.
#[derive(Clone, Debug)]
pub struct PhysicalSupport {
    syntax: ContentHash,
    nodes: BTreeMap<NodeIx, NodeSupport>,
    order: Vec<NodeIx>,
    outputs: Vec<OutputSupport>,
    premises: pse_quantity::premises::PhysicalPremises,
}
impl PhysicalSupport {
    /// Prepare a scalar reached graph with its actual physical context and contextual literals.
    /// Application/indexed products must be supplied by their owning producer before that cutover.
    pub fn prepare_scalar(syntax: &ResolvedSyntax, registry: &QuantityRegistry, facts: &PhysicalPreconditions, expected: &[Type], literal_contracts: &BTreeMap<NodeIx, Type>) -> Result<Self> {
        prepare_scalar(syntax, registry, facts, expected, literal_contracts)
    }
    /// Exact syntax whose node coordinates this product admits.
    pub fn syntax(&self) -> ContentHash { self.syntax }
    /// Require an actual reached admission. An omitted or unrelated node is not an expected type.
    pub fn node(&self, slot: NodeIx) -> Result<&NodeSupport> {
        self.nodes.get(&slot).ok_or_else(|| invalid(SemanticId::NIL, "required resolved physical node is absent"))
    }
    /// Actual resolver read values, issued by the synchronous successful preparation.
    pub fn premises(&self) -> &pse_quantity::premises::PhysicalPremises { &self.premises }
    /// Require the actual result boundary at this ordered requested output.
    pub fn output(&self, index: usize) -> Result<&OutputSupport> {self.outputs.get(index).ok_or_else(|| invalid(SemanticId::NIL, "required resolved output boundary is absent"))}
    /// Known retained support extent. Shared premises are conservatively charged to this product.
    pub fn retained_bytes(&self) -> usize {
        self.nodes.values().fold(size_of::<Self>()
            .saturating_add(self.nodes.len().saturating_mul(size_of::<(NodeIx, NodeSupport)>().saturating_add(96)))
            .saturating_add(self.order.capacity().saturating_mul(size_of::<NodeIx>()))
            .saturating_add(self.outputs.capacity().saturating_mul(size_of::<OutputSupport>()))
            .saturating_add(self.outputs.iter().fold(0usize, |bytes, output| bytes.saturating_add(output.heap_bytes())))
            .saturating_add(self.premises.retained_bytes()), |bytes, value| bytes
                .saturating_add(extent::ty(&value.ty)).saturating_add(value.contract.heap_bytes())
                .saturating_add(value.operation.as_ref().map_or(0, extent::inference))
                .saturating_add(value.literal.as_ref().map_or(0, |literal| literal.conversion.heap_bytes()))
                .saturating_add(value.boundary.as_ref().map_or(0, |boundary| boundary.heap_bytes())))
    }
    /// Frame actual products in reached traversal order, independent of source or arena ordinals.
    pub fn frame(&self, hash: &mut FramedHasher) {
        hash.str("resolved_physical_support").hash(&self.syntax);
        self.premises.frame(hash);
        hash.u64(self.order.len() as u64);
        for slot in &self.order {
            if let Some(support) = self.nodes.get(slot) {
                frame_type(&support.ty, hash); support.contract.frame(hash);
                match &support.operation {Some(operation) => {hash.str("operation"); operation.frame(hash);} None => {hash.str("value");}}
                match &support.literal {
                    Some(literal) => {hash.str("literal_conversion").id(&literal.conversion.quantity().as_id()).id(&literal.conversion.source_unit().as_id()).id(&literal.conversion.canonical_unit().as_id()).u64(literal.conversion.scale().to_bits()).u64(literal.conversion.offset().to_bits());}
                    None => {hash.str("no_literal_conversion");}
                }
                match &support.boundary {Some(boundary) => {hash.str("member_boundary"); boundary.frame(hash);} None => {hash.str("no_member_boundary");}}
            }
        }
        hash.u64(self.outputs.len() as u64);
        for output in &self.outputs {output.frame(hash);}
    }
}

/// Prepare scalar support before numerical construction. Unimplemented application products
/// are explicit refusals; this private pause point has no alternate production consumer.
pub(super) fn prepare_scalar(
    syntax: &ResolvedSyntax,
    registry: &QuantityRegistry,
    facts: &PhysicalPreconditions,
    expected: &[Type],
    literal_contracts: &BTreeMap<NodeIx, Type>,
) -> Result<PhysicalSupport> {
    if expected.len() != syntax.outputs.len() {return Err(invalid(SemanticId::NIL, "resolved output contract arity"));}
    let identity = syntax.identity()?;
    let ((nodes, order, outputs), premises) = pse_quantity::premises::capture(registry, facts, || {
        let scope = crate::PhysicalScope::default();
        let context = crate::TypeContext {admissions: None, formula_authority: None, quantities: registry, preconditions: facts, scope: &scope};
        let mut producer = Producer {syntax, context, literal_contracts, nodes: BTreeMap::new(), order: Vec::new(), active: std::collections::BTreeSet::new(), predicates: BTreeMap::new(), depth: 0};
        let mut outputs = Vec::with_capacity(expected.len());
        for (root, expected) in syntax.outputs.iter().zip(expected) {
            let value = producer.expression(*root, Some(expected))?;
            if value.ty.physical_refinement() != expected.physical_refinement() {return Err(invalid(producer.at(*root), "resolved output changes its required nominal physical role"));}
            outputs.push(OutputSupport::prepare(&value.contract, &producer.contract(expected, producer.at(*root))?, registry).map_err(|error| invalid(producer.at(*root), error.to_string()))?);
        }
        for predicate in &syntax.validity {producer.predicate(*predicate)?;}
        Ok::<_, crate::ModelingError>((producer.nodes, producer.order, outputs))
    })?;
    let premises = premises.ok_or_else(|| invalid(SemanticId::NIL, "resolved scalar support lacks a complete actual physical premise capture"))?;
    Ok(PhysicalSupport {syntax: identity, nodes, order, outputs, premises})
}

struct Producer<'a> {
    syntax: &'a ResolvedSyntax,
    context: crate::TypeContext<'a>,
    literal_contracts: &'a BTreeMap<NodeIx, Type>,
    nodes: BTreeMap<NodeIx, NodeSupport>,
    order: Vec<NodeIx>,
    active: std::collections::BTreeSet<NodeIx>,
    predicates: BTreeMap<PredicateIx, bool>,
    depth: usize,
}
impl Producer<'_> {
    fn at(&self, slot: NodeIx) -> DeclarationId { self.syntax.node_sources.get(&slot).map_or_else(|| SemanticId::NIL.into(), |source| source.field.declaration) }
    fn contract(&self, ty: &Type, at: DeclarationId) -> Result<ResolvedPhysicalContract> {
        ty.quantity_scheme().ok_or_else(|| invalid(at, "resolved numerical value has no physical scheme"))?
            .resolve_contract_with_evidence(self.context.quantities, &BTreeMap::new(), self.context.preconditions).map_err(|error| invalid(at, error.to_string()))
    }
    fn value(&self, ty: Type, at: DeclarationId) -> Result<NodeSupport> { let contract = self.contract(&ty, at)?; Ok(NodeSupport {ty, contract, operation: None, literal: None, boundary: None}) }
    fn alias(value: NodeSupport) -> NodeSupport {NodeSupport {ty: value.ty, contract: value.contract, operation: None, literal: None, boundary: None}}
    fn operation(&self, request: OpRequest<'_>, operands: &[NodeSupport], at: DeclarationId) -> Result<NodeSupport> {
        let refinement = if matches!(request, OpRequest::Conditional) {
            if operands.len() != 2 || operands[0].ty.physical_refinement() != operands[1].ty.physical_refinement() {return Err(invalid(at, "resolved conditional changes a nominal physical role"));}
            operands[0].ty.physical_refinement().cloned()
        } else {crate::contextual::arithmetic_refinement(&request, &operands.iter().map(|value| value.ty.clone()).collect::<Vec<_>>(), &self.context, at)?};
        let operation = crate::expression::admission::admit_operation(&request, &operands.iter().map(|value| value.contract.clone()).collect::<Vec<_>>(), self.context.quantities, self.context.preconditions, self.context.formula_authority.as_ref(), at)?;
        if !operation.conversions.is_empty() {return Err(invalid(at, "resolved input conversion requires an explicit admitted model operation"));}
        let quantity = Scheme::from_contract(operation.result.clone());
        let ty = match refinement { Some(refinement) => Type::RefinedQuantity {quantity, refinement}, None => Type::Quantity(quantity) };
        Ok(NodeSupport {ty, contract: operation.result.clone(), operation: Some(operation), literal: None, boundary: None})
    }
    fn expression(&mut self, slot: NodeIx, expected: Option<&Type>) -> Result<NodeSupport> {
        if let Some(support) = self.nodes.get(&slot) {return Ok(support.clone());}
        if self.depth >= 128 || !self.active.insert(slot) {return Err(invalid(self.at(slot), "resolved physical support is cyclic or exceeds its depth bound"));}
        self.depth += 1;
        let result = self.expression_inner(slot, expected);
        self.depth -= 1;
        self.active.remove(&slot);
        let support = result?;
        self.nodes.insert(slot, support.clone()); self.order.push(slot);
        Ok(support)
    }
    fn expression_inner(&mut self, slot: NodeIx, expected: Option<&Type>) -> Result<NodeSupport> {
        let syntax = self.syntax;
        let node = syntax.nodes.get(slot).ok_or_else(|| invalid(self.at(slot), "resolved physical node outside its owner"))?;
        let at = self.at(slot);
        match node {
            Node::Number(number) => {
                let registry = self.context.quantities;
                let unit = if let Some(product) = &number.unit {registry.compose(product).map_err(|error| invalid(at, error.to_string()))?}
                    else {let quantity = registry.neutral_dimensionless().ok_or_else(|| invalid(at, "neutral literal type absent"))?; registry.unit(registry.quantity_type(quantity).map_err(|error| invalid(at, error.to_string()))?.canonical_unit).map_err(|error| invalid(at, error.to_string()))?.clone()};
                let explicit = self.literal_contracts.get(&slot).or_else(|| if number.unit.is_none() {None} else {expected});
                let context = if let Some(explicit) = explicit {LiteralContext::Explicit {quantity_type: self.contract(explicit, at)?.require_named().map_err(|error| invalid(at, error.to_string()))?}}
                    else if number.unit.is_none() {LiteralContext::Explicit {quantity_type: registry.neutral_dimensionless().ok_or_else(|| invalid(at, "neutral literal type absent"))?}}
                    else {LiteralContext::Free};
                let operation = crate::expression::admission::admit_operation(&OpRequest::Literal {unit: &unit, context}, &[], registry, self.context.preconditions, None, at)?;
                let conversion = CanonicalConversionPlan::resolved(registry, operation.result.require_named().map_err(|error| invalid(at, error.to_string()))?, &unit).map_err(|error| invalid(at, error.to_string()))?;
                conversion.apply(number.value).map_err(|error| invalid(at, error.to_string()))?;
                let quantity = Scheme::from_contract(operation.result.clone());
                let ty = match explicit.or(expected).and_then(Type::physical_refinement).filter(|refinement| matches!(refinement, crate::PhysicalRefinement::Transfer {..})) {
                    Some(refinement) => Type::RefinedQuantity {quantity, refinement: refinement.clone()}, None => Type::Quantity(quantity),
                };
                Ok(NodeSupport {ty, contract: operation.result.clone(), operation: Some(operation), literal: Some(LiteralSupport {conversion}), boundary: None})
            }
            Node::Reference(Reference::Symbol(symbol)) => {
                let symbol = syntax.symbols.at(*symbol).ok_or_else(|| invalid(at, "resolved physical symbol absent"))?;
                if symbol.local_role() == LocalRole::Member && symbol.definition.is_none() {return Err(invalid(at, "required resolved numerical member definition is absent"));}
                if symbol.local_role() != LocalRole::Member && symbol.definition.is_some() {return Err(invalid(at, "resolved input role cannot also own a member definition"));}
                let mut result = self.value(symbol.ty.clone(), at)?;
                if let Some(definition) = symbol.definition {
                    let value = self.expression(definition, Some(&symbol.ty))?;
                    if value.ty.physical_refinement() != symbol.ty.physical_refinement() {return Err(invalid(at, "resolved member definition changes its required nominal physical role"));}
                    result.boundary = Some(OutputSupport::prepare(&value.contract, &result.contract, self.context.quantities).map_err(|error| invalid(at, error.to_string()))?);
                }
                Ok(result)
            }
            Node::Reference(Reference::Lexical(binder)) => match syntax.binders.get(*binder) {
                Some(Binding::Value(value)) => self.expression(*value, expected).map(Self::alias),
                _ => Err(invalid(at, "lexical coordinate requires its complete indexed support product")),
            },
            Node::Neg(value) => {let value = self.expression(*value, expected)?; self.operation(OpRequest::Neg, &[value], at)}
            Node::Binary {op, lhs, rhs} => {
                let additive = matches!(op, BinaryOp::Add | BinaryOp::Sub);
                let left = self.expression(*lhs, if additive {expected} else {None})?;
                let right = self.expression(*rhs, if additive {Some(&left.ty)} else {None})?;
                let request = match op {BinaryOp::Add => OpRequest::Add, BinaryOp::Sub => OpRequest::Sub, BinaryOp::Mul => OpRequest::Mul, BinaryOp::Div => OpRequest::Div, BinaryOp::Pow => OpRequest::Pow {exponent: exponent(syntax, *rhs).map_or(Exponent::Symbolic, Exponent::Rational)}};
                self.operation(request, &[left, right], at)
            }
            Node::Call {function, args} => {
                use pse_quantity::functions::{Implementation, UnaryFunction};
                match function.implementation() {
                    Implementation::Unary(function) if args.len() == 1 => {
                        let value = self.expression(args[0], None)?;
                        let request = match function {UnaryFunction::Exp => OpRequest::Transcendental(pse_quantity::Opcode::Exp), UnaryFunction::Log => OpRequest::Transcendental(pse_quantity::Opcode::Log), UnaryFunction::Sqrt => OpRequest::Sqrt, UnaryFunction::Abs => OpRequest::Abs, UnaryFunction::Sin => OpRequest::Transcendental(pse_quantity::Opcode::Sin), UnaryFunction::Cos => OpRequest::Transcendental(pse_quantity::Opcode::Cos)};
                        self.operation(request, &[value], at)
                    }
                    Implementation::Extremum {..} if args.len() == 2 => {let left = self.expression(args[0], None)?; let right = self.expression(args[1], None)?; self.operation(OpRequest::Conditional, &[left, right], at)}
                    _ => Err(invalid(at, "resolved builtin constructor or arity unavailable")),
                }
            }
            Node::Conditional {guard, then, otherwise} => {self.predicate(*guard)?; let yes = self.expression(*then, expected)?; let no = self.expression(*otherwise, Some(&yes.ty))?; self.operation(OpRequest::Conditional, &[yes, no], at)}
            Node::Let {bindings, body} => {
                for binder in bindings {let Some(Binding::Value(value)) = syntax.binders.get(*binder) else {return Err(invalid(at, "resolved let requires a value binding"));}; self.expression(*value, None)?;}
                self.expression(*body, expected).map(Self::alias)
            }
            Node::Apply {..} | Node::Partial {..} | Node::Provider {..} | Node::Reference(Reference::Indexed {..}) | Node::Reduce {..} | Node::Fold {..} | Node::Derivative {..} => Err(invalid(at, "resolved application requires its selected function/provider/indexed support product")),
        }
    }
    fn predicate(&mut self, slot: PredicateIx) -> Result<()> {
        if let Some(complete) = self.predicates.get(&slot) {return if *complete {Ok(())} else {Err(invalid(SemanticId::NIL, "resolved physical guard cycle"))};}
        self.predicates.insert(slot, false);
        if self.depth >= 128 {return Err(invalid(SemanticId::NIL, "resolved physical guard depth exceeded"));}
        self.depth += 1;
        let result = self.predicate_inner(slot);
        self.depth -= 1;
        if result.is_ok() {self.predicates.insert(slot, true);}
        result
    }
    fn predicate_inner(&mut self, slot: PredicateIx) -> Result<()> {
        let syntax = self.syntax;
        match syntax.predicates.get(slot).ok_or_else(|| invalid(SemanticId::NIL, "resolved guard outside its owner"))? {
            ResolvedPredicate::Compare {lhs, rhs, ..} => {let left = self.expression(*lhs, None)?; let right = self.expression(*rhs, Some(&left.ty))?; if !left.contract.same_meaning(&right.contract) || left.contract.indices() != right.contract.indices() || left.ty.physical_refinement() != right.ty.physical_refinement() {return Err(invalid(self.at(*lhs), "resolved comparison complete physical contracts differ"));}}
            ResolvedPredicate::And(lhs, rhs) | ResolvedPredicate::Or(lhs, rhs) => {self.predicate(*lhs)?; self.predicate(*rhs)?;}
            ResolvedPredicate::Not(value) => self.predicate(*value)?,
            ResolvedPredicate::Bool(_) => {}
            ResolvedPredicate::In {..} | ResolvedPredicate::Atom(_) | ResolvedPredicate::Null => return Err(invalid(SemanticId::NIL, "resolved guard requires its complete Boolean or finite membership product")),
        }
        Ok(())
    }
}
fn exponent(syntax: &ResolvedSyntax, slot: NodeIx) -> Option<pse_quantity::Ratio> {
    exponent_at(syntax, slot, 0)
}
fn exponent_at(syntax: &ResolvedSyntax, slot: NodeIx, depth: usize) -> Option<pse_quantity::Ratio> {
    if depth >= 128 {return None;}
    match syntax.nodes.get(slot)? {
        Node::Number(number) if number.unit.is_none() => number.integer().and_then(|integer| i16::try_from(integer).ok()).and_then(|integer| pse_quantity::Ratio::new(i32::from(integer), 1).ok()),
        Node::Neg(value) => exponent_at(syntax, *value, depth + 1)?.checked_mul(pse_quantity::Ratio::new(-1, 1).ok()?).ok(),
        Node::Binary {op: BinaryOp::Div, lhs, rhs} => {
            let numerator = exponent_at(syntax, *lhs, depth + 1)?;
            let denominator = exponent_at(syntax, *rhs, depth + 1)?;
            if numerator.den() != 1 || denominator.den() != 1 {return None;}
            pse_quantity::Ratio::new(i32::from(numerator.num()), i32::from(denominator.num())).ok()
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct EmptyNames;
    impl Names for EmptyNames {
        fn selection(&self, _: &dsl::Path) -> Option<Selection> {None}
        fn function(&self, _: &dsl::Path) -> Option<FunctionIx> {None}
        fn provider(&self, _: &str) -> Option<ProviderSelection> {None}
    }
    fn syntax(text: &str) -> ResolvedSyntax {
        let source = crate::expression::occurrences::OccurrenceKey {declaration: SemanticId::from_bytes([1;16]).into(), role: "function.body".into(), position: 0};
        let mut resolver = Resolver::new(ResolvedSyntax::default(), &EmptyNames, source, 100);
        resolver.output(&dsl::parse_expr(text).unwrap()).unwrap();
        resolver.finish()
    }
    #[test]
    fn resolved_support_retains_actual_operations_literal_conversion_and_read_premises() {
        let (registry, _) = crate::kernel_types::physical();
        let facts = PhysicalPreconditions::new(vec![]).unwrap();
        let scalar = registry.neutral_dimensionless().unwrap();
        let syntax = syntax("sqrt(2^2)+1");
        let support = PhysicalSupport::prepare_scalar(&syntax, &registry, &facts, &[Type::Quantity(Scheme::Concrete(scalar))], &BTreeMap::new()).unwrap();
        assert_eq!(support.syntax(), syntax.identity().unwrap());
        let root = support.node(syntax.outputs[0]).unwrap();
        assert!(root.operation.is_some());
        assert_eq!(root.contract.require_named().unwrap(), scalar);
        assert!(support.nodes.values().any(|node| node.literal.as_ref().is_some_and(|literal| literal.conversion.quantity() == scalar)));
        assert!(support.premises().matches(&registry, &facts));
        assert!(support.node(NodeIx::from(syntax.nodes.len())).is_err());
    }
    #[test]
    fn resolved_support_checks_original_branches_and_required_output_contracts() {
        let (registry, names) = crate::kernel_types::physical();
        let facts = PhysicalPreconditions::new(vec![]).unwrap();
        let scalar = Type::Quantity(Scheme::Concrete(registry.neutral_dimensionless().unwrap()));
        let length = Type::Quantity(Scheme::Concrete(names["Length"]));
        assert!(PhysicalSupport::prepare_scalar(&syntax("1"), &registry, &facts, std::slice::from_ref(&length), &BTreeMap::new()).is_err());
        let original = syntax("if false then 1 {m} else 1 {s}");
        assert!(PhysicalSupport::prepare_scalar(&original, &registry, &facts, &[length], &BTreeMap::new()).is_err());
        assert!(PhysicalSupport::prepare_scalar(&syntax("1"), &registry, &facts, &[], &BTreeMap::new()).is_err());
        let mut cycle = syntax("1");
        let root=cycle.outputs[0];
        cycle.nodes[root] = Node::Neg(root);
        assert!(PhysicalSupport::prepare_scalar(&cycle, &registry, &facts, &[scalar], &BTreeMap::new()).is_err());
    }
}
