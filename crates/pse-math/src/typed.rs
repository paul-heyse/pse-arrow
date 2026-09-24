// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical admission precedes library normalization at every construction boundary.
use crate::{
    Function, MathError,
    guarded::{Comparison, CompiledBody, Condition, PreparedBody, Stage},
    library::{self, Optimization},
};
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use pse_quantity::{
    IndexSet, Opcode, QuantityRegistry, QuantityTypeId, Ratio, UnitId,
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
    pub(crate) quantity: QuantityTypeId,
    /// Actual lexical indices, separate from library arithmetic.
    pub(crate) indices: IndexSet,
    /// Source occurrence for domain diagnostics.
    pub(crate) source: SemanticId,
}
impl TypedValue {
    /// Physical type established by admission; callers cannot retag an existing value.
    pub const fn quantity(&self) -> QuantityTypeId {
        self.quantity
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
    /// Formal inputs plus barrier result slots.
    pub slots: usize,
    /// Construction operations, including repeated occurrences.
    pub occurrences: usize,
}
impl Default for BodyLimits {
    fn default() -> Self {
        Self {
            slots: library::MAX_FORMAL_SYMBOLS,
            occurrences: 16384,
        }
    }
}

/// A single specialization's typed construction context, never a model-wide graph.
pub struct BodyBuilder<'a> {
    registry: &'a QuantityRegistry,
    checker: &'a dyn InvariantChecker,
    limits: BodyLimits,
    inputs: usize,
    input_quantities: Vec<Option<QuantityTypeId>>,
    next_slot: usize,
    occurrences: usize,
    stages: Vec<Stage>,
    nonsmooth: bool,
    provider_order: DerivativeOrder,
    physical_only: bool,
    provider_cache:
        std::collections::HashMap<(pse_kernels::ProviderKey, Vec<Atom>), Vec<TypedValue>>,
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
    /// Zero/oversized limits or excessive input width.
    pub fn new(
        registry: &'a QuantityRegistry,
        checker: &'a dyn InvariantChecker,
        inputs: usize,
        limits: BodyLimits,
    ) -> Result<Self, MathError> {
        crate::initialize()?;
        if limits.slots == 0
            || limits.slots > library::MAX_FORMAL_SYMBOLS
            || inputs > limits.slots
            || limits.occurrences == 0
            || limits.occurrences > 16384
        {
            return Err(MathError::Limit("body limits"));
        }
        Ok(Self {
            registry,
            checker,
            limits,
            inputs,
            input_quantities: vec![None; inputs],
            next_slot: inputs,
            occurrences: 0,
            stages: vec![],
            nonsmooth: false,
            provider_order: DerivativeOrder::Second,
            physical_only: false,
            provider_cache: std::collections::HashMap::new(),
        })
    }
    /// A physical-only pass validates even empty finite bodies without constructing arithmetic.
    /// # Errors
    /// Invalid resource bounds.
    pub fn type_checking(
        registry: &'a QuantityRegistry,
        checker: &'a dyn InvariantChecker,
        limits: BodyLimits,
    ) -> Result<Self, MathError> {
        let mut builder = Self::new(registry, checker, 1, limits)?;
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
            return Err(MathError::Limit("body slots"));
        }
        let slot = self.next_slot;
        self.next_slot += 1;
        Ok(slot)
    }
    fn result(
        &mut self,
        request: OpRequest<'_>,
        args: &[&TypedValue],
    ) -> Result<(QuantityTypeId, IndexSet), MathError> {
        self.tick()?;
        let operands: Vec<_> = args
            .iter()
            .map(|v| Operand {
                quantity_type: v.quantity,
                indices: &v.indices,
            })
            .collect();
        let result = infer::infer_with_evidence(&request, &operands, self.registry, self.checker)?;
        // Composition-dependent conversions need an explicit provider/model operation.
        if !result.conversions.is_empty() {
            return Err(MathError::Contract(
                "physical input conversion requires an explicit model operation".into(),
            ));
        }
        Ok((result.result, result.indices))
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
            quantity: inferred.result,
            indices,
            source,
        })
    }
    /// Contextual literal with checked representation conversion.
    /// # Errors
    /// Nonfinite literal, ambiguous physical meaning or invalid unit conversion.
    pub fn literal(
        &mut self,
        value: f64,
        unit: UnitId,
        context: LiteralContext,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        if !value.is_finite() {
            return Err(MathError::Contract("nonfinite authored literal".into()));
        }
        let (quantity, indices) = self.result(OpRequest::Literal { unit, context }, &[])?;
        let ty = self.registry.quantity_type(quantity)?;
        let conversion = pse_quantity::convert_spec_for_type(
            self.registry.unit(unit)?,
            self.registry.unit(ty.canonical_unit)?,
            &ty.key,
        )?;
        let canonical = pse_quantity::convert_value(&conversion, value);
        if !canonical.is_finite() {
            return Err(MathError::Contract("nonfinite canonical literal".into()));
        }
        Ok(TypedValue {
            effects: BTreeSet::new(),
            atom: Atom::num(canonical),
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
        if self.physical_only {
            return Ok(());
        }
        let slot = self.materialize(value.atom.clone(), source)?;
        self.stages.push(Stage::Require {
            argument: slot,
            condition,
            order,
            source,
        });
        value.effects.insert(slot);
        value.atom = library::formal(slot)?;
        Ok(())
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
    /// Admit ordered physical arithmetic, then normalize with Symbolica.
    /// # Errors
    /// Incompatible physical contracts or unsupported real power facts.
    pub fn binary(
        &mut self,
        op: Binary,
        mut left: TypedValue,
        mut right: TypedValue,
        exponent: Option<Ratio>,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        if op == Binary::Pow
            && exponent.is_some_and(|r| r.den() == 1 && r.num().unsigned_abs() > 1024)
        {
            return Err(MathError::Limit("integral power degree"));
        }
        if !self.physical_only && op == Binary::Pow {
            if let Some(ratio) = exponent {
                let expected =
                    Atom::num(i64::from(ratio.num())) / Atom::num(i64::from(ratio.den()));
                if right.atom != expected {
                    return Err(MathError::Contract(
                        "power literal fact disagrees with its value".into(),
                    ));
                }
            }
        }
        let request = match op {
            Binary::Add => OpRequest::Add,
            Binary::Sub => OpRequest::Sub,
            Binary::Mul => OpRequest::Mul,
            Binary::Div => OpRequest::Div,
            Binary::Pow => OpRequest::Pow {
                exponent: exponent.map_or(Exponent::Symbolic, Exponent::Rational),
            },
        };
        let (quantity, indices) = self.result(request, &[&left, &right])?;
        if self.physical_only {
            return Ok(TypedValue {
                effects: left.effects.union(&right.effects).copied().collect(),
                atom: Atom::num(0),
                quantity,
                indices,
                source,
            });
        }
        if op == Binary::Div {
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
        let (quantity, indices) = self.result(OpRequest::Neg, &[&value])?;
        Ok(TypedValue {
            effects: value.effects.clone(),
            atom: if self.physical_only {
                Atom::num(0)
            } else {
                -value.atom
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
        mut value: TypedValue,
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
            Unary::Log10 => OpRequest::Transcendental(Opcode::Log10),
            Unary::Sin => OpRequest::Transcendental(Opcode::Sin),
            Unary::Cos => OpRequest::Transcendental(Opcode::Cos),
            Unary::Tan => OpRequest::Transcendental(Opcode::Tan),
        };
        let (quantity, indices) = self.result(request, &[&value])?;
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
            Unary::Log | Unary::Log10 => self.require(
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
                    quantity: value.quantity,
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
            Unary::Log10 => value.atom.log() / Atom::num(10).log(),
            Unary::Sqrt => value.atom.sqrt(),
            Unary::Abs => {
                return Err(MathError::Contract(
                    "absolute value must use its admitted branch".into(),
                ));
            }
            Unary::Sin => value.atom.sin(),
            Unary::Cos => value.atom.cos(),
            Unary::Tan => {
                let mut cosine = TypedValue {
                    effects: value.effects.clone(),
                    atom: value.atom.cos(),
                    quantity: value.quantity,
                    indices: value.indices.clone(),
                    source,
                };
                self.require(
                    &mut cosine,
                    Condition::Nonzero,
                    DerivativeOrder::Value,
                    source,
                )?;
                value.effects.extend(&cosine.effects);
                value.atom.sin() / cosine.atom
            }
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
        pse_quantity::admission::require_same_contract(
            left.quantity,
            right.quantity,
            self.registry,
        )?;
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
        self.tick()?;
        let spec = registration.spec();
        if inputs.len() != spec.inputs.len() {
            return Err(MathError::Contract("provider input arity".into()));
        }
        let cache_key = (spec.key(), inputs.iter().map(|v| v.atom.clone()).collect());
        if !self.physical_only {
            if let Some(outputs) = self.provider_cache.get(&cache_key) {
                for (value, port) in inputs.iter().zip(&spec.inputs) {
                    pse_quantity::admission::require_same_contract(
                        value.quantity,
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
        }
        let mut input_slots = vec![];
        for (value, port) in inputs.iter().zip(&spec.inputs) {
            pse_quantity::admission::require_same_contract(
                value.quantity,
                port.quantity,
                self.registry,
            )?;
            let ty = self.registry.quantity_type(port.quantity)?;
            let conversion = pse_quantity::convert_spec_for_type(
                self.registry.unit(ty.canonical_unit)?,
                self.registry.unit(port.unit)?,
                &ty.key,
            )?;
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
            outputs.push(TypedValue {
                effects: inputs
                    .iter()
                    .flat_map(|v| v.effects.iter().copied())
                    .chain([slot])
                    .collect(),
                atom: if self.physical_only {
                    Atom::num(0)
                } else {
                    library::formal(slot)? * Atom::num(conversion.scale)
                        + Atom::num(conversion.offset)
                },
                quantity: port.quantity,
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
        self.provider_order = self
            .provider_order
            .min(spec.derivatives)
            .min(spec.smoothness);
        if !self.physical_only {
            self.stages.push(Stage::Provider {
                spec: spec.clone(),
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
            comparison: guard.comparison,
            left: guard.left,
            right: guard.right,
            then: then_stages,
            otherwise: else_stages,
        });
        self.nonsmooth |= guard.variable;
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
        Self::type_checking(self.registry, self.checker, self.limits)
    }

    /// Validate smoothness and available provider derivatives independently of value execution.
    pub fn validate_order(&self, order: DerivativeOrder) -> Result<(), MathError> {
        if order > self.provider_order {
            return Err(MathError::Contract(
                "provider derivatives or phase smoothness insufficient".into(),
            ));
        }
        if self.nonsmooth && order > DerivativeOrder::Value {
            return Err(MathError::Contract(
                "variable nonsmooth operations have no C1/C2 neighborhood proof".into(),
            ));
        }
        Ok(())
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
        let (quantity, indices) = self.result(OpRequest::Reduce { kind, bound }, &[prototype])?;
        for term in terms {
            pse_quantity::admission::require_same_contract(
                prototype.quantity,
                term.quantity,
                self.registry,
            )?;
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
        body.set_quantities(
            self.input_quantities,
            outputs.iter().map(|v| v.quantity).collect(),
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
