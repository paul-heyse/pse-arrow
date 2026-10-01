// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical admission precedes library normalization at every construction boundary.
#[path = "typed_witness.rs"]
mod witness;
pub use witness::ScientificWitness;
#[path = "typed_external.rs"]
mod external;
#[path = "typed_partial.rs"]
mod partial;
#[path = "typed_piecewise.rs"]
mod piecewise;
use crate::{
    Function, MathError,
    guarded::{Comparison, CompiledBody, Condition, PreparedBody, Stage},
    library::{self, Optimization},
};
use piecewise::branch_order;
pub(crate) use piecewise::proven_branch_order;
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use pse_quantity::{
    IndexSet, Opcode, QuantityRegistry, QuantityTypeId, Ratio, ResolvedInference,
    ResolvedPhysicalContract,
    infer::{self, Exponent, InvariantChecker, OpRequest, Operand},
    literal::LiteralContext,
};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use symbolica::atom::{Atom, AtomCore};

/// A physically admitted value. Arithmetic storage belongs to Symbolica.
#[derive(Clone, Debug)]
pub struct TypedValue {
    pub(crate) effects: BTreeSet<usize>,
    pub(crate) atom: Atom,
    /// Full physical result contract.
    pub(crate) quantity: ResolvedPhysicalContract,
    /// Actual lexical indices, separate from library arithmetic.
    pub(crate) indices: IndexSet,
    /// Source occurrence for domain diagnostics.
    pub(crate) source: SemanticId,
}
impl TypedValue {
    /// Physical type established by admission; callers cannot retag an existing value.
    pub fn quantity(&self) -> Result<QuantityTypeId, MathError> {
        Ok(self.quantity.require_named()?)
    }
    /// Complete physical contract, including anonymous intermediates.
    pub fn physical_contract(&self) -> &ResolvedPhysicalContract {
        &self.quantity
    }
    /// Free lexical indices established by physical inference.
    pub fn indices(&self) -> &IndexSet {
        &self.indices
    }
    /// Original source occurrence retained through library normalization.
    pub const fn source(&self) -> SemanticId {
        self.source
    }
}

fn check_power_degree(exponent: Ratio) -> Result<(), MathError> {
    if exponent.den() == 1 && exponent.num().unsigned_abs() > 1024 {
        return Err(MathError::Limit("integral power degree"));
    }
    Ok(())
}

/// Source arithmetic distinction retained until physical inference has completed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binary {
    /// Add compatible physical values.
    Add,
    /// Subtract, including point minus point producing a difference.
    Sub,
    /// Registered physical multiplication.
    Mul,
    /// Registered physical quotient.
    Div,
    /// Power with literal facts supplied separately.
    Pow,
}

/// Comparison of physically compatible operands, materialized before branch selection.
#[derive(Clone, Debug)]
pub struct Guard {
    effects: BTreeSet<usize>,
    left: usize,
    right: usize,
    comparison: Comparison,
    variable: bool,
}

/// Resource bound for a local body, independent of instance count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyLimits {
    /// Formal inputs plus barrier result slots. The process-global formal pool extends up
    /// to this allowance; the default is the initially registered chunk.
    pub slots: usize,
    /// Construction operations, including repeated occurrences.
    pub occurrences: usize,
}
impl Default for BodyLimits {
    fn default() -> Self {
        Self {
            slots: library::FORMAL_CHUNK,
            occurrences: 16384,
        }
    }
}

/// Opaque boundary separating explicit function arguments from its local body.
#[derive(Clone, Copy, Debug)]
pub struct FunctionScope(usize);

/// Opaque dependence on a checked domain; it cannot supply a numerical value.
#[derive(Clone, Copy, Debug)]
pub struct DomainAssumption(Option<usize>);

/// A single specialization's typed construction context, never a model-wide graph.
#[derive(Clone)]
pub struct BodyBuilder<'a> {
    context: &'a crate::SymbolicContext,
    registry: &'a QuantityRegistry,
    checker: &'a dyn InvariantChecker,
    limits: BodyLimits,
    inputs: usize,
    input_quantities: Vec<Option<QuantityTypeId>>,
    next_slot: usize,
    occurrences: usize,
    stages: Vec<Stage>,
    provider_order: DerivativeOrder,
    physical_only: bool,
    admissions: Vec<ResolvedInference>,
    output_authorizations: Vec<pse_quantity::AdmittedOutputBoundary>,
    formula_authority: Option<pse_quantity::PhysicalFormulaAuthority>,
    provider_cache: std::collections::HashMap<
        (pse_kernels::ProviderKey, Vec<Atom>, Vec<usize>),
        Vec<TypedValue>,
    >,
}
impl std::fmt::Debug for BodyBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BodyBuilder")
            .field("inputs", &self.inputs)
            .field("slots", &self.next_slot)
            .finish_non_exhaustive()
    }
}
impl<'a> BodyBuilder<'a> {
    /// Create a bounded context. Every input is physically checked before use.
    /// # Errors
    /// Zero limits, or more inputs than the slot allowance.
    pub fn new(
        context: &'a crate::SymbolicContext,
        registry: &'a QuantityRegistry,
        checker: &'a dyn InvariantChecker,
        inputs: usize,
        limits: BodyLimits,
    ) -> Result<Self, MathError> {
        if limits.slots == 0 || limits.occurrences == 0 {
            return Err(MathError::Limit("body limits"));
        }
        if inputs > limits.slots {
            return Err(MathError::SlotLimit {
                required: inputs,
                available: limits.slots,
            });
        }
        Ok(Self {
            context,
            registry,
            checker,
            limits,
            inputs,
            input_quantities: vec![None; inputs],
            admissions: vec![],
            output_authorizations: vec![],
            formula_authority: None,
            next_slot: inputs,
            occurrences: 0,
            stages: vec![],
            provider_order: DerivativeOrder::Second,
            physical_only: false,
            provider_cache: std::collections::HashMap::new(),
        })
    }
    /// A physical-only pass validates even empty finite bodies without constructing arithmetic.
    /// # Errors
    /// Invalid resource bounds.
    pub fn type_checking(
        context: &'a crate::SymbolicContext,
        registry: &'a QuantityRegistry,
        checker: &'a dyn InvariantChecker,
        limits: BodyLimits,
    ) -> Result<Self, MathError> {
        let mut builder = Self::new(context, registry, checker, 1, limits)?;
        builder.physical_only = true;
        Ok(builder)
    }
    fn tick(&mut self) -> Result<(), MathError> {
        self.occurrences += 1;
        if self.occurrences > self.limits.occurrences {
            return Err(MathError::Limit("body occurrences"));
        }
        Ok(())
    }
    fn slot(&mut self) -> Result<usize, MathError> {
        if self.next_slot >= self.limits.slots {
            return Err(MathError::SlotLimit {
                required: self.next_slot.saturating_add(1),
                available: self.limits.slots,
            });
        }
        let slot = self.next_slot;
        self.next_slot += 1;
        Ok(slot)
    }
    fn admit(
        &mut self,
        request: OpRequest<'_>,
        args: &[&TypedValue],
    ) -> Result<ResolvedInference, MathError> {
        self.tick()?;
        let values = args
            .iter()
            .map(|value| value.quantity.clone())
            .collect::<Vec<_>>();
        let result = pse_quantity::resolved::infer_in_context(
            &request,
            &values,
            None,
            self.registry,
            self.checker,
            self.formula_authority.as_ref(),
        )?;
        if !result.conversions.is_empty() {
            return Err(MathError::Contract(
                "physical input conversion requires an explicit model operation".into(),
            ));
        }
        self.admissions.push(result.clone());
        Ok(result)
    }
    /// Consume a checker-owned operation without reconstructing physical algebra.
    /// The exact operand check prevents an admission from being replayed after a change
    /// of representation, subject, datum, axes or bound-index identity.
    fn consume_admission(
        &mut self,
        admission: ResolvedInference,
        args: &[&TypedValue],
    ) -> Result<ResolvedInference, MathError> {
        self.tick()?;
        if admission.operands.len() != args.len()
            || admission
                .operands
                .iter()
                .zip(args)
                .any(|(expected, actual)| expected != &actual.quantity)
        {
            return Err(MathError::Contract(
                "checked operation operands differ from lowering occurrence".into(),
            ));
        }
        if !admission.conversions.is_empty() {
            return Err(MathError::Contract(
                "physical input conversion requires an explicit model operation".into(),
            ));
        }
        self.admissions.push(admission.clone());
        Ok(admission)
    }
    /// Read the currently scoped declaration-owned formula authority for specialization.
    pub fn physical_formula_authority(&self) -> Option<&pse_quantity::PhysicalFormulaAuthority> {
        self.formula_authority.as_ref()
    }
    fn result(
        &mut self,
        request: OpRequest<'_>,
        args: &[&TypedValue],
    ) -> Result<(ResolvedPhysicalContract, IndexSet), MathError> {
        let result = self.admit(request, args)?;
        Ok((result.result.clone(), result.result.indices().clone()))
    }
    /// Admit the declared result boundary and explicitly change numerical coordinates.
    pub fn named_boundary(
        &mut self,
        mut value: TypedValue,
        expected: QuantityTypeId,
    ) -> Result<TypedValue, MathError> {
        let (contract, scale) = value.quantity.at_boundary(expected, self.registry)?;
        if scale != 1.0 {
            value.atom *= Atom::num(scale);
        }
        value.quantity = contract;
        Ok(value)
    }
    /// Consume a complete function signature, including an admitted anonymous result.
    pub fn contract_boundary(
        &mut self,
        mut value: TypedValue,
        expected: &ResolvedPhysicalContract,
    ) -> Result<TypedValue, MathError> {
        let (contract, scale) = value
            .quantity
            .at_contract_boundary(expected, self.registry)?;
        if scale != 1.0 {
            value.atom *= Atom::num(scale);
        }
        value.quantity = contract;
        Ok(value)
    }
    /// Consume a checked declaration-owned physical role transition exactly as admitted.
    pub fn authorized_boundary(
        &mut self,
        mut value: TypedValue,
        authorization: &pse_quantity::AdmittedOutputBoundary,
    ) -> Result<TypedValue, MathError> {
        let (contract, scale) = authorization.apply_contract(&value.quantity)?;
        if scale != 1.0 {
            value.atom *= Atom::num(scale);
        }
        value.quantity = contract;
        self.output_authorizations.push(authorization.clone());
        Ok(value)
    }
    /// Frame all admitted physical operations and declared role transitions.
    pub fn frame_admissions(&self, hash: &mut pse_ids::FramedHasher) {
        hash.str("physical-admissions-v1")
            .u64(self.admissions.len() as u64);
        for admission in &self.admissions {
            admission.frame(hash);
        }
        hash.u64(self.output_authorizations.len() as u64);
        for authorization in &self.output_authorizations {
            authorization.frame(hash);
        }
    }
    /// Versioned identity of the admitted physical operation stream.
    pub fn admission_identity(&self) -> pse_ids::ContentHash {
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::MathResolvedAdmissionsV2);
        self.frame_admissions(&mut hash);
        hash.finish_hash()
    }
    /// Enter a declaration-owned scientific formula scope, returning the prior scope.
    pub fn physical_formula_scope(
        &mut self,
        authority: Option<pse_quantity::PhysicalFormulaAuthority>,
    ) -> Option<pse_quantity::PhysicalFormulaAuthority> {
        std::mem::replace(&mut self.formula_authority, authority)
    }
    /// Bind a complete physical contract to one reusable formal input.
    /// # Errors
    /// Unknown type, incompatible shape or a slot outside the input layout.
    pub fn input(
        &mut self,
        slot: usize,
        quantity: QuantityTypeId,
        indices: IndexSet,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        self.tick()?;
        if !self.physical_only && slot >= self.inputs {
            return Err(MathError::Contract("input slot out of range".into()));
        }
        // Axis order is the group binding's contract; index-set identity is order independent.
        let inferred = infer::infer_with_evidence(
            &OpRequest::Neg,
            &[Operand {
                quantity_type: quantity,
                indices: &indices,
            }],
            self.registry,
            self.checker,
        )?;
        if !self.physical_only {
            let mut key = self.registry.quantity_type(quantity)?.key.clone();
            key.shape.clear();
            let scalar = self.registry.resolve_key(&key)?;
            if self.input_quantities[slot].is_some_and(|previous| previous != scalar) {
                return Err(MathError::Contract(
                    "one formal input has incompatible physical uses".into(),
                ));
            }
            self.input_quantities[slot] = Some(scalar);
        }
        Ok(TypedValue {
            effects: BTreeSet::new(),
            atom: if self.physical_only {
                Atom::num(0)
            } else {
                library::formal(slot)?
            },
            quantity: ResolvedPhysicalContract::named(
                inferred.result,
                indices.clone(),
                self.registry,
            )?,
            indices,
            source,
        })
    }
    /// Contextual literal in its composed unit (ADR-0124) with checked representation
    /// conversion.
    /// # Errors
    /// Nonfinite literal, ambiguous physical meaning or invalid unit conversion.
    pub fn literal(
        &mut self,
        value: f64,
        unit: &pse_quantity::Unit,
        context: LiteralContext,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let (quantity, indices) = self.result(OpRequest::Literal { unit, context }, &[])?;
        let canonical = pse_quantity::CanonicalConversionPlan::resolved(
            self.registry,
            quantity.require_named()?,
            unit,
        )?
        .apply(value)?
        .value();
        Ok(TypedValue {
            effects: BTreeSet::new(),
            // Preserve integral authored facts in the same exact coefficient domain
            // used by physical rational exponents. Other literals stay binary64;
            // approximate decimals are never guessed to be rational numbers.
            atom: if canonical.fract() == 0.0
                && canonical >= f64::from(i16::MIN)
                && canonical <= f64::from(i16::MAX)
            {
                Atom::num(canonical as i32)
            } else {
                Atom::num(canonical)
            },
            quantity,
            indices,
            source,
        })
    }
    /// Preserve a pre-normalization condition and its dependency barrier.
    /// # Errors
    /// The bounded body has no remaining barrier slot.
    pub fn require(
        &mut self,
        value: &mut TypedValue,
        condition: Condition,
        order: DerivativeOrder,
        source: SemanticId,
    ) -> Result<(), MathError> {
        self.obligation(value, condition, order, source, None)
    }
    fn obligation(
        &mut self,
        value: &mut TypedValue,
        condition: Condition,
        order: DerivativeOrder,
        source: SemanticId,
        lineage: Option<Arc<pse_model::diagnostic::ValidityLineage>>,
    ) -> Result<(), MathError> {
        if self.physical_only {
            return Ok(());
        }
        let slot = self.materialize(value.atom.clone(), source)?;
        self.stages.push(Stage::Require {
            argument: slot,
            condition,
            order,
            source,
            lineage,
        });
        value.effects.insert(slot);
        value.atom = library::formal(slot)?;
        Ok(())
    }
    /// Keep an already checked domain barrier even when the returned expression simplifies away.
    pub fn with_assumption(
        &self,
        mut value: TypedValue,
        assumption: &DomainAssumption,
    ) -> TypedValue {
        value.effects.extend(assumption.0);
        value
    }
    /// Evaluate a physically checked predicate of an authored form only for domain
    /// admission, at value order; a rejection names `lineage` (ADR-0123 Outcome 4).
    /// Predicate-local providers and branches do not supply mathematical derivatives.
    pub fn domain<T>(
        &mut self,
        lineage: Arc<pse_model::diagnostic::ValidityLineage>,
        build: T,
    ) -> Result<DomainAssumption, MathError>
    where
        T: FnOnce(&mut Self) -> Result<TypedValue, MathError>,
    {
        let source = lineage.source;
        let parent = std::mem::take(&mut self.stages);
        let cache = std::mem::take(&mut self.provider_cache);
        let order = self.provider_order;
        let predicate = build(self).and_then(|value| self.materialize(value.atom, source));
        let stages = std::mem::replace(&mut self.stages, parent);
        self.provider_cache = cache;
        self.provider_order = order;
        let argument = predicate?;
        if self.physical_only {
            return Ok(DomainAssumption(None));
        }
        let token = self.slot()?;
        self.stages.push(Stage::Domain {
            stages,
            argument,
            token,
            lineage,
        });
        Ok(DomainAssumption(Some(token)))
    }
    /// Bind evidence to library-owned predicate evaluation and actual numeric inputs.
    /// The effect token survives cancellation, normalization and differentiation.
    pub fn applicability<T>(
        &mut self,
        plan: Arc<pse_model::applicability::Node>,
        build: T,
    ) -> Result<DomainAssumption, MathError>
    where
        T: FnOnce(&mut Self) -> Result<(Vec<TypedValue>, Vec<TypedValue>), MathError>,
    {
        let source = plan.claim.form;
        let parent = std::mem::take(&mut self.stages);
        let cache = std::mem::take(&mut self.provider_cache);
        let order = self.provider_order;
        let values = build(self).and_then(|(predicates, inputs)| {
            let predicates = predicates
                .into_iter()
                .map(|v| self.materialize(v.atom, source))
                .collect::<Result<Vec<_>, _>>()?;
            let inputs = inputs
                .into_iter()
                .map(|v| self.materialize(v.atom, source))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((predicates, inputs))
        });
        let stages = std::mem::replace(&mut self.stages, parent);
        self.provider_cache = cache;
        self.provider_order = order;
        let (predicates, inputs) = values?;
        if self.physical_only {
            return Ok(DomainAssumption(None));
        }
        let token = self.slot()?;
        self.stages.push(Stage::Applicability {
            stages,
            predicates,
            inputs,
            token,
            plan,
        });
        Ok(DomainAssumption(Some(token)))
    }
    /// Preserve an authored validity interval as original-domain obligations,
    /// independent of symbolic simplification and requested derivative order.
    /// A rejection names `lineage`, the closure layer's (Plan 23 H5).
    pub fn within_range(
        &mut self,
        mut value: TypedValue,
        lower: TypedValue,
        upper: TypedValue,
        lineage: Arc<pse_model::diagnostic::ValidityLineage>,
    ) -> Result<TypedValue, MathError> {
        let source = lineage.source;
        let mut lo = self.binary(Binary::Sub, value.clone(), lower, None, source)?;
        let mut hi = self.binary(Binary::Sub, upper, value.clone(), None, source)?;
        self.obligation(
            &mut lo,
            Condition::Nonnegative,
            DerivativeOrder::Value,
            source,
            Some(lineage.clone()),
        )?;
        self.obligation(
            &mut hi,
            Condition::Nonnegative,
            DerivativeOrder::Value,
            source,
            Some(lineage),
        )?;
        value.effects.extend(lo.effects);
        value.effects.extend(hi.effects);
        Ok(value)
    }
    /// Retain an authored shared binding as one evaluated block output.
    /// Physical meaning, source attribution and ordered domain effects are preserved.
    /// # Errors
    /// The bounded body has no remaining value slot.
    pub fn bind(&mut self, value: TypedValue) -> Result<TypedValue, MathError> {
        if matches!(
            value.atom.as_view(),
            symbolica::atom::AtomView::Num(_) | symbolica::atom::AtomView::Var(_)
        ) {
            // Constants and existing slots already have bounded representations.
            // Preserve their physical contract and effects without an alias block.
            return Ok(value);
        }
        self.independent(value)
    }
    /// Bind a fresh function argument even when two arguments share a value.
    /// Explicit partials must distinguish f(a, b) evaluated at a == b.
    pub fn independent(&mut self, mut value: TypedValue) -> Result<TypedValue, MathError> {
        if !self.physical_only {
            let slot = self.materialize(value.atom, value.source)?;
            value.atom = library::formal(slot)?;
        }
        Ok(value)
    }
    /// Begin a pure function body after its independently bound explicit arguments.
    pub fn function_scope(&self) -> FunctionScope {
        FunctionScope(self.stages.len())
    }

    /// Differentiate an admitted pure body using Symbolica, retaining its original obligations.
    /// Arguments must be independently bound before `scope`; repeated arguments request
    /// higher derivatives. Branches and external calls need their own derivative contract.
    /// # Errors
    /// Unsupported effect, missing complete derivative type, or a bounded symbolic refusal.
    pub fn partial(
        &mut self,
        scope: FunctionScope,
        value: TypedValue,
        arguments: &[TypedValue],
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        if arguments.is_empty() || arguments.len() > 8 || scope.0 > self.stages.len() {
            return Err(MathError::Limit("function partial order or scope"));
        }
        let arguments_contracts = arguments
            .iter()
            .map(|argument| argument.quantity.clone())
            .collect::<Vec<_>>();
        let admission = pse_quantity::resolved::infer_partial(
            &value.quantity,
            &arguments_contracts,
            self.registry,
            self.checker,
        )?;
        self.admissions.push(admission.clone());
        self.partial_checked(scope, value, arguments, admission, source)
    }
    /// Differentiate a checked physical occurrence without recomputing its contract.
    pub fn partial_admitted(
        &mut self,
        scope: FunctionScope,
        value: TypedValue,
        arguments: &[TypedValue],
        admission: ResolvedInference,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let operands = std::iter::once(&value).chain(arguments).collect::<Vec<_>>();
        let admission = self.consume_admission(admission, &operands)?;
        self.partial_checked(scope, value, arguments, admission, source)
    }
    fn partial_checked(
        &mut self,
        scope: FunctionScope,
        value: TypedValue,
        arguments: &[TypedValue],
        admission: ResolvedInference,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        if arguments.is_empty() || arguments.len() > 8 || scope.0 > self.stages.len() {
            return Err(MathError::Limit("function partial order or scope"));
        }
        let quantity = admission.result;
        let scale = admission.result_scale;
        if !self.physical_only
            && self.stages[scope.0..]
                .iter()
                .any(|s| matches!(s, Stage::Provider { .. }))
        {
            let atom = self.external_partial(scope, &value.atom, arguments, source)?;
            return Ok(TypedValue {
                atom: if scale == 1.0 {
                    atom
                } else {
                    atom * Atom::num(scale)
                },
                quantity,
                indices: value.indices,
                effects: value.effects,
                source,
            });
        }
        if !self.physical_only
            && self.stages[scope.0..]
                .iter()
                .any(|s| matches!(s, Stage::Branch { .. }))
        {
            let output = self.slot()?;
            let stages = piecewise::partial_paths(
                &self.stages[scope.0..],
                std::collections::BTreeMap::new(),
                &value.atom,
                arguments,
                output,
                source,
                self.limits.occurrences,
                &mut 0,
                &mut self.next_slot,
                self.limits.slots,
            )?;
            piecewise::strengthen_guards(&mut self.stages[scope.0..], arguments.len());
            self.stages.extend(stages);
            return Ok(TypedValue {
                atom: library::formal(output)? * Atom::num(scale),
                quantity,
                indices: value.indices,
                effects: value.effects,
                source,
            });
        }
        let atom = if self.physical_only {
            value.atom.clone()
        } else {
            self.shared_partial(scope, &value.atom, arguments, source)?
        };
        // A partial evaluates derivatives even when its caller asks only for values.
        // Keep every original guard and strengthen the local derivative-order guards.
        for stage in &mut self.stages[scope.0..] {
            if let Stage::Require { order, .. } = stage {
                *order = match (*order, arguments.len()) {
                    (DerivativeOrder::Second, 1) => DerivativeOrder::First,
                    _ => DerivativeOrder::Value,
                };
            }
        }
        Ok(TypedValue {
            atom: if scale == 1.0 {
                atom
            } else {
                atom * Atom::num(scale)
            },
            quantity,
            indices: value.indices,
            effects: value.effects,
            source,
        })
    }
    fn materialize(&mut self, atom: Atom, source: SemanticId) -> Result<usize, MathError> {
        if self.physical_only {
            return Ok(0);
        }
        let slot = self.slot()?;
        self.stages.push(Stage::Block {
            expressions: vec![atom],
            outputs: vec![slot],
            source,
        });
        Ok(slot)
    }
    /// Admit one ordered physical arithmetic operation, then normalize with Symbolica.
    /// # Errors
    /// Incompatible physical contracts or unsupported real power facts.
    pub fn binary(
        &mut self,
        op: Binary,
        left: TypedValue,
        right: TypedValue,
        exponent: Option<Ratio>,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let request = match op {
            Binary::Add => OpRequest::Add,
            Binary::Sub => OpRequest::Sub,
            Binary::Mul => OpRequest::Mul,
            Binary::Div => OpRequest::Div,
            Binary::Pow => OpRequest::Pow {
                exponent: exponent.map_or(Exponent::Symbolic, Exponent::Rational),
            },
        };
        let admission = self.admit(request, &[&left, &right])?;
        self.combine(op, left, right, exponent, admission, source)
    }
    /// Lower one binary occurrence using the concrete admission supplied by checking.
    pub fn binary_admitted(
        &mut self,
        op: Binary,
        left: TypedValue,
        right: TypedValue,
        exponent: Option<Ratio>,
        admission: ResolvedInference,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let admission = self.consume_admission(admission, &[&left, &right])?;
        self.combine(op, left, right, exponent, admission, source)
    }
    /// Build one node from its retained physical admission.
    fn combine(
        &mut self,
        op: Binary,
        mut left: TypedValue,
        mut right: TypedValue,
        exponent: Option<Ratio>,
        admission: ResolvedInference,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        if op == Binary::Pow
            && let Some(exponent) = exponent
        {
            check_power_degree(exponent)?;
        }
        let quantity = admission.result;
        let indices = quantity.indices().clone();
        for (value, scale) in [&mut left, &mut right]
            .into_iter()
            .zip(&admission.operand_scales)
        {
            if *scale != 1.0 {
                value.atom = &value.atom * Atom::num(*scale);
            }
        }
        if !self.physical_only
            && op == Binary::Pow
            && let Some(ratio) = exponent
        {
            let expected = Atom::num(i64::from(ratio.num())) / Atom::num(i64::from(ratio.den()));
            if right.atom != expected {
                return Err(MathError::Contract(
                    "power literal fact disagrees with its value".into(),
                ));
            }
        }
        if self.physical_only {
            return Ok(TypedValue {
                effects: left.effects.union(&right.effects).copied().collect(),
                atom: Atom::num(0),
                quantity,
                indices,
                source,
            });
        }
        if op == Binary::Div
            && !(matches!(right.atom.as_view(), symbolica::atom::AtomView::Num(_))
                && !right.atom.is_zero())
        {
            self.require(
                &mut right,
                Condition::Nonzero,
                DerivativeOrder::Value,
                source,
            )?;
        }
        if op == Binary::Pow {
            match exponent {
                Some(r) if r.den() == 1 => {
                    if r.num() <= 0 {
                        self.require(
                            &mut left,
                            Condition::Nonzero,
                            DerivativeOrder::Value,
                            source,
                        )?;
                    } else {
                        // Do not construct arbitrarily large exact constants during CAS admission.
                        let slot = self.materialize(left.atom.clone(), source)?;
                        left.atom = library::formal(slot)?;
                    }
                }
                // The initial general real-power profile is strictly positive.
                _ => self.require(
                    &mut left,
                    Condition::Positive,
                    DerivativeOrder::Value,
                    source,
                )?,
            }
        }
        let atom = match op {
            Binary::Add => &left.atom + &right.atom,
            Binary::Sub => &left.atom - &right.atom,
            Binary::Mul => &left.atom * &right.atom,
            Binary::Div => &left.atom / &right.atom,
            Binary::Pow => left.atom.pow(&right.atom),
        };
        let atom = if admission.result_scale == 1.0 {
            atom
        } else {
            atom * Atom::num(admission.result_scale)
        };
        Ok(TypedValue {
            effects: left.effects.union(&right.effects).copied().collect(),
            atom,
            quantity,
            indices,
            source,
        })
    }
    /// Physically admitted unary negation.
    /// # Errors
    /// Invalid physical input or exhausted body budget.
    pub fn negate(
        &mut self,
        value: TypedValue,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let admission = self.admit(OpRequest::Neg, &[&value])?;
        self.negate_checked(value, admission, source)
    }
    /// Lower unary negation from its occurrence-owned admission.
    pub fn negate_admitted(
        &mut self,
        value: TypedValue,
        admission: ResolvedInference,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let admission = self.consume_admission(admission, &[&value])?;
        self.negate_checked(value, admission, source)
    }
    fn negate_checked(
        &mut self,
        value: TypedValue,
        admission: ResolvedInference,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let quantity = admission.result;
        let indices = quantity.indices().clone();
        Ok(TypedValue {
            effects: value.effects.clone(),
            atom: if self.physical_only {
                Atom::num(0)
            } else {
                -value.atom * Atom::num(admission.operand_scales[0] * admission.result_scale)
            },
            quantity,
            indices,
            source,
        })
    }
    /// Built-in library functions with original domain obligations.
    /// # Errors
    /// Unsupported arity/function, missing physical rule or invalid domain profile.
    pub fn unary(
        &mut self,
        function: Function,
        value: TypedValue,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let crate::Implementation::Unary(function) = function.implementation() else {
            return Err(MathError::Contract(
                "function needs a distinct admitted constructor".into(),
            ));
        };
        use crate::UnaryFunction as Unary;
        let request = match function {
            Unary::Sqrt => OpRequest::Sqrt,
            Unary::Abs => OpRequest::Abs,
            Unary::Exp => OpRequest::Transcendental(Opcode::Exp),
            Unary::Log => OpRequest::Transcendental(Opcode::Log),
            Unary::Sin => OpRequest::Transcendental(Opcode::Sin),
            Unary::Cos => OpRequest::Transcendental(Opcode::Cos),
        };
        let admission = self.admit(request, &[&value])?;
        self.unary_checked(function, value, admission, source)
    }
    /// Lower a primitive function from its retained physical operation.
    pub fn unary_admitted(
        &mut self,
        function: Function,
        value: TypedValue,
        admission: ResolvedInference,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let crate::Implementation::Unary(function) = function.implementation() else {
            return Err(MathError::Contract(
                "function needs a distinct admitted constructor".into(),
            ));
        };
        let admission = self.consume_admission(admission, &[&value])?;
        self.unary_checked(function, value, admission, source)
    }
    fn unary_checked(
        &mut self,
        function: crate::UnaryFunction,
        mut value: TypedValue,
        admission: ResolvedInference,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        use crate::UnaryFunction as Unary;
        let quantity = admission.result;
        let indices = quantity.indices().clone();
        if admission.operand_scales[0] != 1.0 {
            value.atom = &value.atom * Atom::num(admission.operand_scales[0]);
        }

        if self.physical_only {
            return Ok(TypedValue {
                effects: value.effects.clone(),
                atom: Atom::num(0),
                quantity,
                indices,
                source,
            });
        }
        match function {
            Unary::Exp => {
                // Symbolica 3.0.0's exp/log normalization can discard an exponent
                // whose logarithmic factors do not simplify. Keep the exponent as
                // an evaluated dependency; Numerica still owns exp and its jets.
                let slot = self.materialize(value.atom.clone(), source)?;
                value.atom = library::formal(slot)?;
            }
            Unary::Log => self.require(
                &mut value,
                Condition::Positive,
                DerivativeOrder::Value,
                source,
            )?,
            Unary::Sqrt => {
                self.require(
                    &mut value,
                    Condition::Nonnegative,
                    DerivativeOrder::Value,
                    source,
                )?;
                self.require(
                    &mut value,
                    Condition::Positive,
                    DerivativeOrder::First,
                    source,
                )?;
            }
            Unary::Abs => {
                let zero = TypedValue {
                    effects: value.effects.clone(),
                    atom: Atom::num(0),
                    quantity: value.quantity.clone(),
                    indices: value.indices.clone(),
                    source,
                };
                let guard = self.compare(Comparison::Lt, &value, &zero, source)?;
                let negative = value.clone();
                return self.conditional(
                    guard,
                    |builder| builder.negate(negative, source),
                    |_| Ok(value),
                    source,
                );
            }
            _ => {}
        }
        let atom = match function {
            Unary::Exp => value.atom.exp(),
            Unary::Log => value.atom.log(),
            Unary::Sqrt => value.atom.sqrt(),
            Unary::Abs => {
                return Err(MathError::Contract(
                    "absolute value must use its admitted branch".into(),
                ));
            }
            Unary::Sin => value.atom.sin(),
            Unary::Cos => value.atom.cos(),
        };
        let atom = if admission.result_scale == 1.0 {
            atom
        } else {
            atom * Atom::num(admission.result_scale)
        };
        Ok(TypedValue {
            effects: value.effects.clone(),
            atom,
            quantity,
            indices,
            source,
        })
    }
    /// Admit and materialize a scalar comparison.
    /// # Errors
    /// Incompatible physical contracts or exhausted body resources.
    pub fn compare(
        &mut self,
        comparison: Comparison,
        left: &TypedValue,
        right: &TypedValue,
        source: SemanticId,
    ) -> Result<Guard, MathError> {
        if !left.quantity.same_meaning(&right.quantity) {
            return Err(MathError::Contract(
                "comparison physical contracts differ".into(),
            ));
        }
        if left.indices != right.indices {
            return Err(MathError::Contract("comparison binder mismatch".into()));
        }
        Ok(Guard {
            effects: left.effects.union(&right.effects).copied().collect(),
            left: self.materialize(left.atom.clone(), source)?,
            right: self.materialize(right.atom.clone(), source)?,
            comparison,
            variable: true,
        })
    }
    /// A specialization-time Boolean guard. Both branches still receive physical checking.
    pub fn fixed_guard(&mut self, selected: bool, source: SemanticId) -> Result<Guard, MathError> {
        Ok(Guard {
            effects: BTreeSet::new(),
            left: self.materialize(Atom::num(i32::from(selected)), source)?,
            right: self.materialize(Atom::num(1), source)?,
            comparison: Comparison::Eq,
            variable: false,
        })
    }
    /// Admit an executable physical provider and schedule one coherent multi-output call.
    pub fn provider(
        &mut self,
        registration: &pse_kernels::AdmittedProvider,
        inputs: &[TypedValue],
        source: SemanticId,
    ) -> Result<Vec<TypedValue>, MathError> {
        self.provider_partial(registration, inputs, &[], source)
    }
    /// Select supplied first or second local partials and compose any remaining outer jets.
    /// Unit offsets disappear and physical delta-unit chain factors remain explicit.
    pub fn provider_partial(
        &mut self,
        registration: &pse_kernels::AdmittedProvider,
        inputs: &[TypedValue],
        partial: &[usize],
        source: SemanticId,
    ) -> Result<Vec<TypedValue>, MathError> {
        self.tick()?;
        let spec = registration.spec();
        let remaining = (spec.derivatives.min(spec.smoothness) as usize)
            .checked_sub(partial.len())
            .ok_or_else(|| {
                MathError::Contract("external partial exceeds supplied derivative order".into())
            })?;
        if partial.iter().any(|i| *i >= inputs.len()) {
            return Err(MathError::Contract("external partial coordinate".into()));
        }
        let available = match remaining {
            0 => DerivativeOrder::Value,
            1 => DerivativeOrder::First,
            _ => DerivativeOrder::Second,
        };
        if inputs.len() != spec.inputs.len() {
            return Err(MathError::Contract("provider input arity".into()));
        }
        let cache_key = (
            spec.key(),
            inputs.iter().map(|v| v.atom.clone()).collect(),
            partial.to_vec(),
        );
        if !self.physical_only
            && let Some(outputs) = self.provider_cache.get(&cache_key)
        {
            for (value, port) in inputs.iter().zip(&spec.inputs) {
                pse_quantity::admission::require_same_contract(
                    value.quantity()?,
                    port.quantity,
                    self.registry,
                )?;
            }
            if inputs.windows(2).any(|w| w[0].indices != w[1].indices) {
                return Err(MathError::Contract(
                    "provider lexical indices differ".into(),
                ));
            }
            return Ok(outputs
                .iter()
                .cloned()
                .map(|mut value| {
                    value
                        .effects
                        .extend(inputs.iter().flat_map(|v| v.effects.iter().copied()));
                    value.source = source;
                    value.indices = inputs
                        .first()
                        .map_or_else(IndexSet::new, |v| v.indices.clone());
                    value
                })
                .collect());
        }
        let mut input_slots = vec![];
        let mut input_scales = vec![];
        for (value, port) in inputs.iter().zip(&spec.inputs) {
            pse_quantity::admission::require_same_contract(
                value.quantity()?,
                port.quantity,
                self.registry,
            )?;
            let ty = self.registry.quantity_type(port.quantity)?;
            let conversion = pse_quantity::convert_spec_for_type(
                self.registry.unit(ty.canonical_unit)?,
                self.registry.unit(port.unit)?,
                &ty.key,
            )?;
            input_scales.push(conversion.scale);
            input_slots.push(self.materialize(
                &value.atom * Atom::num(conversion.scale) + Atom::num(conversion.offset),
                source,
            )?);
        }
        let mut slots = vec![];
        let mut outputs = vec![];
        for port in &spec.outputs {
            let slot = if self.physical_only { 0 } else { self.slot()? };
            slots.push(slot);
            let ty = self.registry.quantity_type(port.quantity)?;
            let conversion = pse_quantity::convert_spec_for_type(
                self.registry.unit(port.unit)?,
                self.registry.unit(ty.canonical_unit)?,
                &ty.key,
            )?;
            use pse_quantity::scheme::{Scheme, Substitution};
            let mut scheme = Scheme::Concrete(port.quantity);
            for i in partial {
                scheme = Scheme::Quotient(
                    Box::new(Scheme::Delta(Box::new(scheme))),
                    Box::new(Scheme::Delta(Box::new(Scheme::from_contract(
                        inputs[*i].quantity.clone(),
                    )))),
                );
            }
            let quantity = scheme
                .resolve_contract_with_evidence(self.registry, &Substitution::new(), self.checker)
                .map_err(|e| MathError::Contract(e.to_string()))?;
            let scale = partial
                .iter()
                .fold(conversion.scale, |s, i| s * input_scales[*i]);
            if !scale.is_finite() {
                return Err(MathError::Contract("external partial unit scale".into()));
            }
            outputs.push(TypedValue {
                effects: inputs
                    .iter()
                    .flat_map(|v| v.effects.iter().copied())
                    .chain([slot])
                    .collect(),
                atom: if self.physical_only {
                    Atom::num(0)
                } else {
                    library::formal(slot)? * Atom::num(scale)
                        + Atom::num(if partial.is_empty() {
                            conversion.offset
                        } else {
                            0.
                        })
                },
                quantity,
                indices: inputs
                    .first()
                    .map_or_else(IndexSet::new, |v| v.indices.clone()),
                source,
            });
        }
        if inputs.windows(2).any(|w| w[0].indices != w[1].indices) {
            return Err(MathError::Contract(
                "provider lexical indices differ".into(),
            ));
        }
        self.provider_order = self.provider_order.min(available);
        if !self.physical_only {
            self.stages.push(Stage::Provider {
                spec: spec.clone(),
                partial: partial.to_vec(),
                inputs: input_slots,
                outputs: slots,
                source,
            });
            self.provider_cache.insert(cache_key, outputs.clone());
        }
        Ok(outputs)
    }
    /// Construct independent lazy branch regions. No optimizer sees across their barriers.
    /// # Errors
    /// Either branch is physically invalid or the branch result contracts differ.
    pub fn conditional<T, E>(
        &mut self,
        guard: Guard,
        then: T,
        otherwise: E,
        source: SemanticId,
    ) -> Result<TypedValue, MathError>
    where
        T: FnOnce(&mut Self) -> Result<TypedValue, MathError>,
        E: FnOnce(&mut Self) -> Result<TypedValue, MathError>,
    {
        if self.physical_only {
            let a = then(self)?;
            let b = otherwise(self)?;
            let (quantity, indices) = self.result(OpRequest::Conditional, &[&a, &b])?;
            return Ok(TypedValue {
                effects: BTreeSet::new(),
                atom: Atom::num(0),
                quantity,
                indices,
                source,
            });
        }
        let parent = std::mem::take(&mut self.stages);
        let parent_providers = std::mem::take(&mut self.provider_cache);
        let then_value = then(self)?;
        let result_slot = self.slot()?;
        self.stages.push(Stage::Block {
            expressions: vec![then_value.atom.clone()],
            outputs: vec![result_slot],
            source,
        });
        let then_stages = std::mem::take(&mut self.stages);
        self.provider_cache.clear();
        let else_value = otherwise(self)?;
        let (quantity, indices) =
            self.result(OpRequest::Conditional, &[&then_value, &else_value])?;
        self.stages.push(Stage::Block {
            expressions: vec![else_value.atom],
            outputs: vec![result_slot],
            source,
        });
        let effects = then_value
            .effects
            .union(&else_value.effects)
            .copied()
            .chain(guard.effects)
            .chain([result_slot])
            .collect();
        let else_stages = std::mem::replace(&mut self.stages, parent);
        self.provider_cache = parent_providers;
        self.stages.push(Stage::Branch {
            continuity: if guard.variable {
                DerivativeOrder::Value
            } else {
                DerivativeOrder::Second
            },
            comparison: guard.comparison,
            left: guard.left,
            right: guard.right,
            then: then_stages,
            otherwise: else_stages,
        });
        Ok(TypedValue {
            effects,
            atom: library::formal(result_slot)?,
            quantity,
            indices,
            source,
        })
    }
    /// Create a bounded physical pass over the same registry and invariant evidence.
    /// # Errors
    /// Invalid analysis resource bounds.
    pub fn physical_pass(&self) -> Result<BodyBuilder<'a>, MathError> {
        Self::type_checking(self.context, self.registry, self.checker, self.limits)
    }

    /// Validate smoothness and available provider derivatives independently of value execution.
    pub fn validate_order(&self, order: DerivativeOrder) -> Result<(), MathError> {
        if order > self.provider_order {
            return Err(MathError::Contract(
                "provider derivatives or phase smoothness insufficient".into(),
            ));
        }
        if order > branch_order(&self.stages) {
            return Err(MathError::Contract(
                "variable nonsmooth operations have no C1/C2 neighborhood proof".into(),
            ));
        }
        Ok(())
    }

    /// Reduce scalar cells after enumeration, retaining the consumed physical domain.
    /// The independent prototype prevents an empty set from hiding an invalid contract.
    pub fn finite_reduce(
        &mut self,
        kind: pse_quantity::ReductionKind,
        domain: Option<pse_quantity::EntityKindId>,
        prototype: &ResolvedPhysicalContract,
        terms: &[TypedValue],
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let admission = pse_quantity::resolved::infer_in_context(
            &OpRequest::FiniteReduce { kind, domain },
            std::slice::from_ref(prototype),
            None,
            self.registry,
            self.checker,
            self.formula_authority.as_ref(),
        )?;
        self.finite_reduce_admitted(kind, prototype, terms, admission, source)
    }
    /// Enumerated reduction consumes the independently checked prototype operation.
    pub fn finite_reduce_admitted(
        &mut self,
        kind: pse_quantity::ReductionKind,
        prototype: &ResolvedPhysicalContract,
        terms: &[TypedValue],
        admission: ResolvedInference,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let prototype_value = TypedValue {
            quantity: prototype.clone(),
            indices: IndexSet::new(),
            atom: Atom::num(0),
            effects: BTreeSet::new(),
            source,
        };
        let admission = self.consume_admission(admission, &[&prototype_value])?;
        if !prototype.indices().is_empty() {
            return Err(MathError::Contract(
                "finite reduction prototype retains free indices".into(),
            ));
        }
        let terms = terms
            .iter()
            .map(|term| {
                if !term.indices.is_empty() {
                    return Err(MathError::Contract(
                        "finite reduction cell retains free indices".into(),
                    ));
                }
                self.contract_boundary(term.clone(), prototype)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let atom = if self.physical_only {
            Atom::num(0)
        } else {
            match kind {
                pse_quantity::ReductionKind::Sum => terms.iter().fold(Atom::num(0), |sum, term| {
                    sum + &term.atom * Atom::num(admission.operand_scales[0])
                }),
                pse_quantity::ReductionKind::Prod => {
                    terms.iter().fold(Atom::num(1), |product, term| {
                        product * &term.atom * Atom::num(admission.operand_scales[0])
                    })
                }
                _ => {
                    return Err(MathError::Contract(
                        "finite source reduction requires sum or product".into(),
                    ));
                }
            }
        };
        Ok(TypedValue {
            atom: atom * Atom::num(admission.result_scale),
            indices: admission.result.indices().clone(),
            quantity: admission.result,
            source,
            effects: terms
                .iter()
                .flat_map(|term| term.effects.iter().copied())
                .collect(),
        })
    }
    /// Reduce admitted finite term occurrences without deduplicating a bag.
    /// The prototype is independently type checked, including when there are no members.
    /// # Errors
    /// Invalid physical reduction, mismatching term contracts or exceeded local width.
    pub fn reduce(
        &mut self,
        kind: pse_quantity::ReductionKind,
        bound: pse_quantity::BoundIndexRef,
        prototype: &TypedValue,
        terms: &[TypedValue],
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let admission = self.admit(OpRequest::Reduce { kind, bound }, &[prototype])?;
        self.reduce_checked(kind, prototype, terms, admission, source)
    }
    /// Lower a bound-index reduction using the actual binder's retained admission.
    pub fn reduce_admitted(
        &mut self,
        kind: pse_quantity::ReductionKind,
        prototype: &TypedValue,
        terms: &[TypedValue],
        admission: ResolvedInference,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let admission = self.consume_admission(admission, &[prototype])?;
        self.reduce_checked(kind, prototype, terms, admission, source)
    }
    fn reduce_checked(
        &mut self,
        kind: pse_quantity::ReductionKind,
        prototype: &TypedValue,
        terms: &[TypedValue],
        admission: ResolvedInference,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let quantity = admission.result;
        let indices = quantity.indices().clone();
        for term in terms {
            if !prototype.quantity.same_meaning(&term.quantity) {
                return Err(MathError::Contract(
                    "reduction physical contracts differ".into(),
                ));
            }
            if term.indices != prototype.indices {
                return Err(MathError::Contract(
                    "reduction occurrence binders differ".into(),
                ));
            }
        }
        let mut effects: BTreeSet<_> = terms
            .iter()
            .flat_map(|v| v.effects.iter().copied())
            .collect();
        let atom = if self.physical_only {
            Atom::num(0)
        } else {
            match kind {
                pse_quantity::ReductionKind::Sum => terms
                    .iter()
                    .fold(Atom::num(0), |sum, term| sum + &term.atom),
                pse_quantity::ReductionKind::Prod => terms
                    .iter()
                    .fold(Atom::num(1), |product, term| product * &term.atom),
                pse_quantity::ReductionKind::Min | pse_quantity::ReductionKind::Max => {
                    let mut terms = terms.iter();
                    let mut value = terms
                        .next()
                        .ok_or_else(|| MathError::Contract("empty min/max reduction".into()))?
                        .clone();
                    for term in terms {
                        value = self.extremum(
                            kind == pse_quantity::ReductionKind::Min,
                            value,
                            term.clone(),
                            source,
                        )?;
                    }
                    effects.extend(value.effects);
                    value.atom
                }
            }
        };
        Ok(TypedValue {
            effects,
            atom,
            quantity,
            indices,
            source,
        })
    }
    /// Exact value minimum or maximum, retaining a lazy selection barrier.
    /// # Errors
    /// Physical mismatch, or insufficient control storage.
    pub fn extremum(
        &mut self,
        minimum: bool,
        left: TypedValue,
        right: TypedValue,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let guard = if minimum {
            self.compare(Comparison::Le, &left, &right, source)?
        } else {
            self.compare(Comparison::Le, &right, &left, source)?
        };
        self.conditional(guard, |_| Ok(left), |_| Ok(right), source)
    }
    /// Produce an immutable, physically admitted symbolic specification.
    pub fn prepare(mut self, outputs: &[TypedValue]) -> Result<PreparedBody, MathError> {
        if self.physical_only {
            return Err(MathError::Contract(
                "physical-only analysis is not executable".into(),
            ));
        }
        let output_slots = outputs
            .iter()
            .map(|value| self.materialize(value.atom.clone(), value.source))
            .collect::<Result<Vec<_>, _>>()?;
        let mut body = PreparedBody::new(
            self.inputs,
            self.next_slot,
            output_slots,
            self.stages,
            self.provider_order,
        )?;
        body.set_effects(outputs.iter().map(|v| v.effects.clone()).collect());
        body.set_occurrences(self.occurrences);
        body.set_quantities(
            self.input_quantities,
            outputs
                .iter()
                .map(TypedValue::quantity)
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(body)
    }

    /// Compile a physically admitted result with the standard bounded worker profile.
    /// # Errors
    /// Unsupported smooth profile, resource limit or library refusal.
    pub fn finish(
        self,
        outputs: &[TypedValue],
        order: DerivativeOrder,
        options: Optimization,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<CompiledBody, MathError> {
        self.validate_order(order)?;
        let body = self.prepare(outputs)?;
        body.compile(
            &(0..body.output_count()).collect::<Vec<_>>(),
            &(0..body.input_count()).collect::<Vec<_>>(),
            order,
            options,
            crate::jets::EvaluationLimits::default(),
            cancelled,
        )
    }
}
