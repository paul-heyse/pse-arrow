// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete quantity inference (blueprint §8.3).

use crate::literal::{LiteralContext, resolve_literal};
use crate::{
    BoundIndexRef, ConversionId, DimensionVector, DomainKind, IncompatibilityReason, IndexSet,
    InvariantId, Opcode, OperationId, QuantityAdditionKind, QuantityError, QuantityOperation,
    QuantityRegistry, QuantityScaleRule, QuantityShapeRule, QuantityType, QuantityTypeId,
    QuantityTypeKey, Ratio, ReductionKind, ScaleKind, UnitConvertSpec, UnitId, WeightNormalization,
    admission, convert_spec_for_type,
};

/// One ordered operand, including binder identity rather than only index shape.
#[derive(Clone, Copy, Debug)]
pub struct Operand<'a> {
    /// Admitted complete quantity contract.
    pub quantity_type: QuantityTypeId,
    /// Actual free binders.
    pub indices: &'a IndexSet,
}
/// Exponent information established by the expression driver.
#[derive(Clone, Copy, Debug)]
pub enum Exponent {
    /// An exact rational literal.
    Rational(Ratio),
    /// A nonconstant exponent; requires a dimensionless base.
    Symbolic,
}
/// The complete request; payload-specific facts are explicit inputs.
#[derive(Clone, Debug)]
pub enum OpRequest<'a> {
    /// Per-occurrence literal typing.
    Literal {
        /// Authored unit.
        unit: UnitId,
        /// Expected complete contract.
        context: LiteralContext,
    },
    /// Ordered addition.
    Add,
    /// Ordered subtraction.
    Sub,
    /// Negation.
    Neg,
    /// Absolute value.
    Abs,
    /// Product.
    Mul,
    /// Quotient.
    Div,
    /// Exponentiation.
    Pow {
        /// Resolved exponent facts.
        exponent: Exponent,
    },
    /// Square root.
    Sqrt,
    /// Declared dimensionless function.
    Transcendental(Opcode),
    /// Ordered affine constant and terms; coefficients have already supplied signs.
    Affine {
        /// One sign (-1 or +1) per term operand.
        term_signs: &'a [i8],
        /// Whether operand zero is the explicit typed constant.
        has_constant: bool,
    },
    /// A smooth scalar operation with its explicit dimensional regularization.
    Smooth {
        /// One of the five smooth opcodes.
        opcode: Opcode,
        /// Positive finite smoothing parameter in the operation's declared units.
        eps: f64,
    },
    /// A group read after the domain source has admitted the actual coordinate tuple.
    Gather {
        /// Complete declared group type.
        group_type: QuantityTypeId,
        /// Binders in the group's declared axis order.
        coordinates: &'a [BoundIndexRef],
    },
    /// Ordered mean: operands are weight,value pairs.
    WeightedMean {
        /// Normalization semantics.
        normalization: WeightNormalization,
        /// Named unit-sum proof prerequisite.
        certified_invariant: Option<InvariantId>,
    },
    /// Reduction over a known binder.
    Reduce {
        /// Reduction operation.
        kind: ReductionKind,
        /// Bound identity and domain.
        bound: BoundIndexRef,
    },
    /// Add a binder explicitly.
    Broadcast {
        /// Binder to add.
        index: BoundIndexRef,
    },
    /// Branch types; the driver separately establishes Boolean guard semantics.
    Conditional,
    /// Explicit checked unit edge.
    UnitConvert {
        /// Exact edge coefficients.
        spec: UnitConvertSpec,
    },
    /// Binding input/output contracts, resolved by identity before invocation.
    KernelCall {
        /// Complete expected inputs.
        declared_inputs: &'a [QuantityTypeId],
        /// Complete output.
        declared_output: QuantityTypeId,
    },
    /// An already-resolved implicit unknown.
    ImplicitRef {
        /// Unknown's complete type.
        unknown: QuantityTypeId,
    },
    /// Continuous-domain differentiation.
    Derivative {
        /// Coordinate unit.
        domain_unit: UnitId,
        /// Actual domain kind.
        domain_kind: DomainKind,
        /// Positive derivative order.
        order: u8,
    },
    /// Continuous-domain integration.
    Integral {
        /// Coordinate unit.
        domain_unit: UnitId,
        /// Exact lexical integration binder and its domain.
        bound: BoundIndexRef,
    },
    /// Explicit piecewise input/output contracts.
    PiecewiseLinear {
        /// Input coordinate type.
        input: QuantityTypeId,
        /// Output value type.
        output: QuantityTypeId,
    },
}
crate::closed_enum! {
    /// Built-in physical rules; registered compositions retain their operation identity.
    pub enum BuiltInRule {
        /// Contextual literal resolution.
        Literal => "literal",
        /// Complete additive/origin-sensitive rule.
        Addition => "addition",
        /// Preserve a unary operand contract.
        Unary => "unary",
        /// Explicitly designated neutral scalar scaling.
        NeutralScaling => "neutral_scaling",
        /// Ordered affine type composition.
        Affine => "affine",
        /// Explicit smooth-operator contracts.
        Smooth => "smooth",
        /// Group type with explicit binder coordinates.
        Gather => "gather",
        /// Identical value contracts under declared normalization.
        WeightedMean => "weighted_mean",
        /// Bind one index without losing remaining identities.
        Reduction => "reduction",
        /// Add a binder explicitly.
        Broadcast => "broadcast",
        /// Identical branch contracts.
        Conditional => "conditional",
        /// Exact declared unit edge.
        UnitConvert => "unit_convert",
        /// Complete declared binding contracts.
        Binding => "binding",
    }
}
/// Evidence of the physical rule selected for this node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationSelection {
    /// A closed built-in rule.
    BuiltIn(BuiltInRule),
    /// A unique registered rule and its original operand order.
    Registered {
        /// Operation identity.
        operation: OperationId,
        /// Original operand ordinal for each matched slot.
        operand_permutation: Vec<u16>,
    },
}
/// A selected conversion, with its original operand ordinal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperandConversion {
    /// Original operand position.
    pub operand: u16,
    /// Explicit registered conversion.
    pub conversion: ConversionId,
}
/// The complete result and derivation dependencies.
#[derive(Clone, Debug)]
pub struct Inferred {
    /// Resolved complete result contract.
    pub result: QuantityTypeId,
    /// Remaining actual free binders.
    pub indices: IndexSet,
    /// Physical rule selected.
    pub selected: OperationSelection,
    /// Explicit conversions required before the selected operation.
    pub conversions: Vec<OperandConversion>,
}
/// Checks an invariant against actual facts applicable to this exact operation request.
/// Implementations must inspect operation/operand/registry scope and their validated fact
/// source; matching a declared identity or an unscoped set of IDs is insufficient.
pub trait InvariantChecker {
    /// Whether all facts read by this checker remain immutable for its borrowed
    /// lifetime. Mutable or observation-based implementations must keep the default.
    fn immutable(&self) -> bool {
        false
    }

    /// Establish the named prerequisite for these operands or return an error.
    ///
    /// # Errors
    /// Rejects absent, inapplicable, or violated facts.
    fn check(
        &self,
        invariant: InvariantId,
        request: &OpRequest<'_>,
        operation: Option<&QuantityOperation>,
        operands: &[Operand<'_>],
        registry: &QuantityRegistry,
    ) -> Result<(), QuantityError>;
}
/// Conservative checker used when the caller supplies no applicable facts.
#[derive(Debug)]
pub struct NoInvariantFacts;
impl InvariantChecker for NoInvariantFacts {
    fn immutable(&self) -> bool {
        true
    }
    fn check(
        &self,
        _: InvariantId,
        _: &OpRequest<'_>,
        _: Option<&QuantityOperation>,
        _: &[Operand<'_>],
        _: &QuantityRegistry,
    ) -> Result<(), QuantityError> {
        Err(invariant(
            "operation.unestablished_invariant",
            "no applicable fact checker was supplied",
        ))
    }
}

/// Infer with no established invariant facts.
///
/// # Errors
/// Rejects incompatible contracts, ambiguous rules, or unestablished prerequisites.
pub fn infer(
    request: &OpRequest<'_>,
    operands: &[Operand<'_>],
    registry: &QuantityRegistry,
) -> Result<Inferred, QuantityError> {
    infer_with_evidence(request, operands, registry, &NoInvariantFacts)
}
/// Infer using a checker of actual facts scoped to this operation and its dependencies.
/// Declaring an invariant or hashing its declaration does not establish it.
///
/// # Errors
/// Rejects incompatible contracts, ambiguous rules, or unestablished prerequisites.
#[expect(
    clippy::too_many_lines,
    reason = "exhaustive request dispatch preserves visible operator-family contracts"
)]
pub fn infer_with_evidence(
    request: &OpRequest<'_>,
    operands: &[Operand<'_>],
    registry: &QuantityRegistry,
    checker: &dyn InvariantChecker,
) -> Result<Inferred, QuantityError> {
    for operand in operands {
        let ty = registry.quantity_type(operand.quantity_type)?;
        let mut expected = ty.key.shape.clone();
        expected.sort_unstable();
        let mut actual: Vec<_> = operand.indices.iter().map(|index| index.kind).collect();
        actual.sort_unstable();
        if expected != actual {
            return Err(incompatible(IncompatibilityReason::IndexMismatch, operands));
        }
    }
    match request {
        OpRequest::Literal { unit, context } => {
            count(operands, 0)?;
            Ok(built(
                resolve_literal(*unit, *context, registry)?,
                IndexSet::new(),
                BuiltInRule::Literal,
            ))
        }
        OpRequest::Add | OpRequest::Sub => {
            additive(matches!(request, OpRequest::Sub), operands, registry)
        }
        OpRequest::Neg | OpRequest::Abs => {
            count(operands, 1)?;
            Ok(built(
                operands[0].quantity_type,
                operands[0].indices.clone(),
                BuiltInRule::Unary,
            ))
        }
        OpRequest::Mul | OpRequest::Div => {
            count(operands, 2)?;
            let neutral = registry.neutral_dimensionless();
            let left_neutral =
                neutral == Some(operands[0].quantity_type) && operands[0].indices.is_empty();
            let right_neutral =
                neutral == Some(operands[1].quantity_type) && operands[1].indices.is_empty();
            let keep = if right_neutral {
                Some(0)
            } else if left_neutral && matches!(request, OpRequest::Mul) {
                Some(1)
            } else {
                None
            };
            if let Some(position) = keep {
                return Ok(built(
                    operands[position].quantity_type,
                    operands[position].indices.clone(),
                    BuiltInRule::NeutralScaling,
                ));
            }
            registered(request, operands, registry, checker)
        }
        OpRequest::Affine {
            term_signs,
            has_constant,
        } => affine(operands, term_signs, *has_constant, registry),
        OpRequest::Smooth { opcode, eps } => smooth(operands, *opcode, *eps, registry, checker),
        OpRequest::Gather {
            group_type,
            coordinates,
        } => {
            count(operands, 0)?;
            let ty = registry.quantity_type(*group_type)?;
            let shape: Vec<_> = coordinates.iter().map(|index| index.kind).collect();
            if ty.key.shape != shape {
                return Err(invariant(
                    "gather.shape",
                    "coordinate axes disagree with the group type",
                ));
            }
            let indices = IndexSet::try_from_iter(coordinates.iter().copied())
                .map_err(|_| invariant("gather.binders", "conflicting coordinate binders"))?;
            if indices.len() != coordinates.len() {
                return Err(invariant(
                    "gather.binders",
                    "a binder occurs on multiple axes",
                ));
            }
            Ok(built(*group_type, indices, BuiltInRule::Gather))
        }
        OpRequest::WeightedMean {
            normalization,
            certified_invariant,
        } => weighted_mean(
            operands,
            *normalization,
            *certified_invariant,
            registry,
            checker,
        ),
        OpRequest::Conditional => {
            count(operands, 2)?;
            same(operands, registry)?;
            Ok(built(
                operands[0].quantity_type,
                operands[0].indices.clone(),
                BuiltInRule::Conditional,
            ))
        }
        OpRequest::Broadcast { index } => {
            count(operands, 1)?;
            let mut key = registry
                .quantity_type(operands[0].quantity_type)?
                .key
                .clone();
            let mut indices = operands[0].indices.clone();
            if indices.contains_index(index.bound_index) {
                return Err(invariant("broadcast.new_index", "binder already present"));
            }
            indices
                .insert(*index)
                .map_err(|_| invariant("broadcast.index", "binder conflict"))?;
            key.shape.push(index.kind);
            Ok(built(
                registry.resolve_key(&key)?,
                indices,
                BuiltInRule::Broadcast,
            ))
        }
        OpRequest::Reduce { kind, bound } => {
            validate_reduction(operands, *kind, *bound, registry)?;
            if *kind == ReductionKind::Sum
                && !match_rules(request, operands, registry, false)?.is_empty()
            {
                // An actual kind-matched contraction is authoritative, including a
                // refusal or ambiguity. A failed contract must not fall back.
                registered(request, operands, registry, checker)
            } else {
                reduce(operands, *kind, *bound, registry)
            }
        }
        OpRequest::UnitConvert { spec } => unit_convert(operands, *spec, registry),
        OpRequest::KernelCall {
            declared_inputs,
            declared_output,
        } => {
            count(operands, declared_inputs.len())?;
            for (expected, actual) in declared_inputs.iter().zip(operands) {
                admission::require_same_contract(*expected, actual.quantity_type, registry)?;
            }
            registry.quantity_type(*declared_output)?;
            let indices = common_indices(operands)?;
            Ok(built(*declared_output, indices, BuiltInRule::Binding))
        }
        OpRequest::ImplicitRef { unknown } => {
            count(operands, 0)?;
            let ty = registry.quantity_type(*unknown)?;
            if !ty.key.shape.is_empty() {
                return Err(invariant(
                    "implicit.indices",
                    "indexed unknown requires explicit binder facts",
                ));
            }
            Ok(built(*unknown, IndexSet::new(), BuiltInRule::Binding))
        }
        OpRequest::PiecewiseLinear { input, output } => {
            count(operands, 1)?;
            admission::require_same_contract(*input, operands[0].quantity_type, registry)?;
            registry.quantity_type(*output)?;
            Ok(built(
                *output,
                operands[0].indices.clone(),
                BuiltInRule::Binding,
            ))
        }
        OpRequest::Pow { .. }
        | OpRequest::Sqrt
        | OpRequest::Transcendental(_)
        | OpRequest::Derivative { .. }
        | OpRequest::Integral { .. } => registered(request, operands, registry, checker),
    }
}
fn built(result: QuantityTypeId, indices: IndexSet, rule: BuiltInRule) -> Inferred {
    Inferred {
        result,
        indices,
        selected: OperationSelection::BuiltIn(rule),
        conversions: vec![],
    }
}
fn invariant(rule: &'static str, detail: &str) -> QuantityError {
    QuantityError::InferencePrecondition {
        rule,
        detail: detail.to_owned(),
    }
}
fn count(operands: &[Operand<'_>], expected: usize) -> Result<(), QuantityError> {
    if operands.len() == expected {
        Ok(())
    } else {
        Err(invariant(
            "operation.arity",
            "operand count disagrees with request",
        ))
    }
}
fn incompatible(reason: IncompatibilityReason, operands: &[Operand<'_>]) -> QuantityError {
    QuantityError::Incompatible {
        reason,
        operands: operands
            .iter()
            .map(|operand| (operand.quantity_type, operand.indices.clone()))
            .collect(),
        hint: None,
    }
}
fn common_indices(operands: &[Operand<'_>]) -> Result<IndexSet, QuantityError> {
    let Some(first) = operands.first() else {
        return Ok(IndexSet::new());
    };
    if operands.iter().all(|x| x.indices == first.indices) {
        Ok(first.indices.clone())
    } else {
        Err(incompatible(IncompatibilityReason::IndexMismatch, operands))
    }
}
fn same(operands: &[Operand<'_>], registry: &QuantityRegistry) -> Result<(), QuantityError> {
    if let Some(first) = operands.first() {
        for other in &operands[1..] {
            admission::require_same_contract(first.quantity_type, other.quantity_type, registry)?;
        }
    }
    common_indices(operands)?;
    Ok(())
}
fn additive(
    subtract: bool,
    operands: &[Operand<'_>],
    registry: &QuantityRegistry,
) -> Result<Inferred, QuantityError> {
    count(operands, 2)?;
    let left = registry.quantity_type(operands[0].quantity_type)?;
    let right = registry.quantity_type(operands[1].quantity_type)?;
    for (equal, reason) in [
        (
            left.key.kind == right.key.kind,
            IncompatibilityReason::KindMismatch,
        ),
        (
            left.key.basis == right.key.basis,
            IncompatibilityReason::BasisMismatch,
        ),
        (
            left.key.reference_state == right.key.reference_state,
            IncompatibilityReason::DatumMismatch,
        ),
        (
            left.key.subject_kind == right.key.subject_kind,
            IncompatibilityReason::SubjectMismatch,
        ),
    ] {
        if !equal {
            return Err(incompatible(reason, operands));
        }
    }
    let indices = common_indices(operands)?;
    let mut key = left.key.clone();
    if registry.kind(key.kind)?.addition_kind == QuantityAdditionKind::OriginSensitive {
        key.scale_kind = match (subtract, left.key.scale_kind, right.key.scale_kind) {
            (false, ScaleKind::Point, ScaleKind::Point) => {
                return Err(incompatible(
                    IncompatibilityReason::PointPlusPoint,
                    operands,
                ));
            }
            (true, ScaleKind::Difference, ScaleKind::Point) => {
                return Err(incompatible(
                    IncompatibilityReason::DifferenceMinusPoint,
                    operands,
                ));
            }
            (true, ScaleKind::Point, ScaleKind::Point)
            | (_, ScaleKind::Difference, ScaleKind::Difference) => ScaleKind::Difference,
            _ => ScaleKind::Point,
        };
    }
    Ok(built(
        registry.resolve_key(&key)?,
        indices,
        BuiltInRule::Addition,
    ))
}
fn weighted_mean(
    operands: &[Operand<'_>],
    normalization: WeightNormalization,
    certificate: Option<InvariantId>,
    registry: &QuantityRegistry,
    checker: &dyn InvariantChecker,
) -> Result<Inferred, QuantityError> {
    if operands.is_empty() || !operands.len().is_multiple_of(2) {
        return Err(invariant(
            "weighted_mean.pairs",
            "nonempty weight/value pairs required",
        ));
    }
    if normalization == WeightNormalization::CertifiedUnitSum {
        let id = certificate.ok_or_else(|| {
            invariant("weighted_mean.certificate", "unit-sum invariant is absent")
        })?;
        checker.check(
            id,
            &OpRequest::WeightedMean {
                normalization,
                certified_invariant: certificate,
            },
            None,
            operands,
            registry,
        )?;
    }
    let values: Vec<_> = operands.iter().skip(1).step_by(2).copied().collect();
    same(&values, registry)?;
    for pair in operands.as_chunks::<2>().0 {
        let weight = registry.quantity_type(pair[0].quantity_type)?;
        if !registry.kind(weight.key.kind)?.dimension.is_dimensionless() {
            return Err(invariant(
                "weighted_mean.weight",
                "weight must be dimensionless",
            ));
        }
        if !pair[0].indices.is_empty() && pair[0].indices != pair[1].indices {
            return Err(incompatible(IncompatibilityReason::IndexMismatch, operands));
        }
    }
    Ok(built(
        values[0].quantity_type,
        values[0].indices.clone(),
        BuiltInRule::WeightedMean,
    ))
}
fn validate_reduction(
    operands: &[Operand<'_>],
    kind: ReductionKind,
    bound: BoundIndexRef,
    registry: &QuantityRegistry,
) -> Result<(), QuantityError> {
    count(operands, 1)?;
    let ty = registry.quantity_type(operands[0].quantity_type)?;
    if operands[0].indices.get(bound.bound_index) != Some(&bound) {
        return Err(incompatible(IncompatibilityReason::IndexMismatch, operands));
    }
    if kind == ReductionKind::Sum
        && registry.kind(ty.key.kind)?.addition_kind == QuantityAdditionKind::OriginSensitive
        && ty.key.scale_kind == ScaleKind::Point
    {
        return Err(incompatible(IncompatibilityReason::SumOfPoints, operands));
    }
    if kind == ReductionKind::Prod && !registry.kind(ty.key.kind)?.dimension.is_dimensionless() {
        return Err(invariant(
            "reduction.product",
            "product requires a registered finite-cardinality rule or neutral dimensionless body",
        ));
    }
    Ok(())
}
fn reduce(
    operands: &[Operand<'_>],
    kind: ReductionKind,
    bound: BoundIndexRef,
    registry: &QuantityRegistry,
) -> Result<Inferred, QuantityError> {
    validate_reduction(operands, kind, bound, registry)?;
    let ty = registry.quantity_type(operands[0].quantity_type)?;
    let bound_set = IndexSet::try_from_iter([bound])
        .map_err(|_| invariant("reduction.index", "binder conflict"))?;
    let indices = operands[0].indices.difference(&bound_set);
    let mut key = ty.key.clone();
    let Some(position) = key.shape.iter().position(|kind| *kind == bound.kind) else {
        return Err(invariant(
            "reduction.shape",
            "bound kind absent from quantity shape",
        ));
    };
    key.shape.remove(position);
    Ok(built(
        registry.resolve_key(&key)?,
        indices,
        BuiltInRule::Reduction,
    ))
}
fn unit_convert(
    operands: &[Operand<'_>],
    spec: UnitConvertSpec,
    registry: &QuantityRegistry,
) -> Result<Inferred, QuantityError> {
    count(operands, 1)?;
    let ty = registry.quantity_type(operands[0].quantity_type)?;
    if spec.from != ty.canonical_unit {
        return Err(QuantityError::UnitConvertMismatch {
            declared_from: spec.from,
            operand_unit: ty.canonical_unit,
            to: spec.to,
        });
    }
    let expected =
        convert_spec_for_type(registry.unit(spec.from)?, registry.unit(spec.to)?, &ty.key)?;
    if !spec.same_contract(&expected) {
        return Err(invariant(
            "unit_convert.coefficients",
            "declared edge disagrees with actual units and scale kind",
        ));
    }
    // Representation-unit changes do not silently change the physical quantity contract.
    Ok(built(
        ty.id,
        operands[0].indices.clone(),
        BuiltInRule::UnitConvert,
    ))
}

fn request_opcode(request: &OpRequest<'_>) -> Opcode {
    match request {
        OpRequest::Literal { .. } => Opcode::Const,
        OpRequest::Add => Opcode::Add,
        OpRequest::Sub => Opcode::Sub,
        OpRequest::Neg => Opcode::Neg,
        OpRequest::Abs => Opcode::Abs,
        OpRequest::Mul => Opcode::Mul,
        OpRequest::Div => Opcode::Div,
        OpRequest::Pow { .. } => Opcode::Pow,
        OpRequest::Sqrt => Opcode::Sqrt,
        OpRequest::Transcendental(opcode) | OpRequest::Smooth { opcode, .. } => *opcode,
        OpRequest::Affine { .. } => Opcode::Affine,
        OpRequest::Gather { .. } => Opcode::Gather,
        OpRequest::WeightedMean { .. } => Opcode::WeightedMean,
        OpRequest::Reduce { kind, .. } => match kind {
            ReductionKind::Sum => Opcode::SumOver,
            ReductionKind::Prod => Opcode::ProdOver,
            ReductionKind::Min => Opcode::MinOver,
            ReductionKind::Max => Opcode::MaxOver,
        },
        OpRequest::Broadcast { .. } => Opcode::Broadcast,
        OpRequest::Conditional => Opcode::Conditional,
        OpRequest::UnitConvert { .. } => Opcode::UnitConvert,
        OpRequest::KernelCall { .. } => Opcode::KernelCall,
        OpRequest::ImplicitRef { .. } => Opcode::ImplicitRef,
        OpRequest::Derivative { .. } => Opcode::Derivative,
        OpRequest::Integral { .. } => Opcode::Integral,
        OpRequest::PiecewiseLinear { .. } => Opcode::PiecewiseLinear,
    }
}
struct Match<'a> {
    rule: &'a QuantityOperation,
    types: Vec<QuantityTypeId>,
    permutation: Vec<u16>,
}
fn match_rules<'a>(
    request: &OpRequest<'_>,
    operands: &[Operand<'_>],
    registry: &'a QuantityRegistry,
    swapped: bool,
) -> Result<Vec<Match<'a>>, QuantityError> {
    let mut found = Vec::new();
    for rule in registry.operations_for(request_opcode(request)) {
        if let Some(required) = registry.reduction_domain(rule.id) {
            let actual = match request {
                OpRequest::Reduce { bound, .. } | OpRequest::Integral { bound, .. } => {
                    Some(bound.kind)
                }
                _ => None,
            };
            if actual != Some(required) {
                continue;
            }
        }
        if rule.input_kinds.len() != operands.len() {
            continue;
        }
        let mut permutation = (0..operands.len())
            .map(u16::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| invariant("operation.arity", "too many operands"))?;
        if swapped {
            permutation.reverse();
        }
        let mut types: Vec<_> = permutation
            .iter()
            .map(|position| operands[usize::from(*position)].quantity_type)
            .collect();
        let mut matches = true;
        for conversion in &rule.input_conversions {
            let declared = registry.conversion(conversion.conversion)?;
            let slot = usize::from(conversion.operand);
            if types[slot] != declared.from {
                matches = false;
                break;
            }
            types[slot] = declared.to;
        }
        if !matches {
            continue;
        }
        for (ty, kind) in types.iter().zip(&rule.input_kinds) {
            if registry.quantity_type(*ty)?.key.kind != *kind {
                matches = false;
                break;
            }
        }
        if matches {
            found.push(Match {
                rule,
                types,
                permutation,
            });
        }
    }
    Ok(found)
}
fn registered(
    request: &OpRequest<'_>,
    operands: &[Operand<'_>],
    registry: &QuantityRegistry,
    checker: &dyn InvariantChecker,
) -> Result<Inferred, QuantityError> {
    let opcode = request_opcode(request);
    let ordered = match_rules(request, operands, registry, false)?;
    let swapped = if ordered.is_empty() && opcode == Opcode::Mul && operands.len() == 2 {
        match_rules(request, operands, registry, true)?
    } else {
        vec![]
    };
    let selected = if ordered.len() == 1 {
        &ordered[0]
    } else if ordered.is_empty() && swapped.len() == 1 {
        &swapped[0]
    } else {
        return Err(QuantityError::OperationUnsupported {
            opcode,
            input_kinds: operands
                .iter()
                .map(|x| {
                    registry
                        .quantity_type(x.quantity_type)
                        .map(|ty| ty.key.kind)
                })
                .collect::<Result<_, _>>()?,
            ordered_matches: ordered.len(),
            swapped_matches: swapped.len(),
        });
    };
    let types = selected
        .types
        .iter()
        .map(|id| registry.quantity_type(*id))
        .collect::<Result<Vec<_>, _>>()?;
    let dimension = dimension_for(request, &types, registry)?;
    if registry.kind(selected.rule.result_kind)?.dimension != dimension {
        return Err(invariant(
            "operation.result_dimension",
            "registered result kind disagrees with actual dimension algebra",
        ));
    }
    let rule = selected.rule;
    let aligned: Vec<_> = selected
        .permutation
        .iter()
        .zip(&selected.types)
        .map(|(position, quantity_type)| Operand {
            quantity_type: *quantity_type,
            indices: operands[usize::from(*position)].indices,
        })
        .collect();
    for invariant in &selected.rule.precondition_invariants {
        checker.check(*invariant, request, Some(selected.rule), &aligned, registry)?;
    }
    let (key, indices) = compose_key(request, rule, &types, &aligned)?;
    let result = registry
        .resolve_key(&key)
        .map_err(|_| QuantityError::UnregisteredResultType {
            opcode,
            operation: rule.id,
            requested: format!(
                "kind {} with declared basis/reference/scale/shape/subject",
                key.kind
            ),
        })?;
    let conversions = rule
        .input_conversions
        .iter()
        .map(|conversion| OperandConversion {
            operand: selected.permutation[usize::from(conversion.operand)],
            conversion: conversion.conversion,
        })
        .collect();
    Ok(Inferred {
        result,
        indices,
        selected: OperationSelection::Registered {
            operation: rule.id,
            operand_permutation: selected.permutation.clone(),
        },
        conversions,
    })
}
fn compose_key(
    request: &OpRequest<'_>,
    rule: &QuantityOperation,
    types: &[&QuantityType],
    aligned: &[Operand<'_>],
) -> Result<(QuantityTypeKey, IndexSet), QuantityError> {
    let basis = tag(
        &types.iter().map(|ty| ty.key.basis).collect::<Vec<_>>(),
        rule.basis_rule.as_str(),
        rule.basis_source,
        rule.result_basis,
    )?;
    let reference_state = tag(
        &types
            .iter()
            .map(|ty| ty.key.reference_state)
            .collect::<Vec<_>>(),
        rule.reference_rule.as_str(),
        rule.reference_source,
        rule.result_reference_state,
    )?;
    let subject_kind = tag(
        &types
            .iter()
            .map(|ty| ty.key.subject_kind)
            .collect::<Vec<_>>(),
        rule.subject_rule.as_str(),
        rule.subject_source,
        rule.result_subject_kind,
    )?;
    let scale_kind = match rule.scale_rule {
        QuantityScaleRule::Preserve => {
            types[usize::from(rule.scale_source.ok_or_else(|| {
                invariant("operation.scale_source", "missing preserved scale source")
            })?)]
            .key
            .scale_kind
        }
        QuantityScaleRule::Point => ScaleKind::Point,
        QuantityScaleRule::Difference => ScaleKind::Difference,
        QuantityScaleRule::Addition | QuantityScaleRule::WeightedMean => {
            return Err(invariant(
                "operation.scale_policy",
                "addition/weighted-mean policies require their built-in request",
            ));
        }
    };
    let (shape, indices) = compose_shape(request, rule, types, aligned)?;
    let key = QuantityTypeKey {
        kind: rule.result_kind,
        basis,
        reference_state,
        scale_kind,
        shape,
        subject_kind,
    };
    Ok((key, indices))
}

fn compose_shape(
    request: &OpRequest<'_>,
    rule: &QuantityOperation,
    types: &[&QuantityType],
    aligned: &[Operand<'_>],
) -> Result<(Vec<DomainKind>, IndexSet), QuantityError> {
    Ok(match rule.shape_rule {
        QuantityShapeRule::Preserve => {
            let source = usize::from(rule.shape_source.ok_or_else(|| {
                invariant("operation.shape_source", "missing preserved shape source")
            })?);
            (
                types[source].key.shape.clone(),
                aligned[source].indices.clone(),
            )
        }
        QuantityShapeRule::SameIndices => {
            let indices = common_indices(aligned)?;
            (
                types
                    .first()
                    .map_or_else(Vec::new, |ty| ty.key.shape.clone()),
                indices,
            )
        }
        QuantityShapeRule::Scalar => {
            if aligned.iter().any(|operand| !operand.indices.is_empty()) {
                return Err(invariant(
                    "operation.scalar_shape",
                    "scalar result cannot discard free indices",
                ));
            }
            (vec![], IndexSet::new())
        }
        QuantityShapeRule::ReduceBoundIndex => {
            let (OpRequest::Integral { bound, .. } | OpRequest::Reduce { bound, .. }) = request
            else {
                return Err(invariant(
                    "operation.index_policy",
                    "reduction policy requires an explicit bound index",
                ));
            };
            count(aligned, 1)?;
            if aligned[0].indices.get(bound.bound_index) != Some(bound) {
                return Err(incompatible(IncompatibilityReason::IndexMismatch, aligned));
            }
            let mut shape = types[0].key.shape.clone();
            let position = shape
                .iter()
                .position(|kind| *kind == bound.kind)
                .ok_or_else(|| {
                    invariant(
                        "operation.index_policy",
                        "integration domain kind absent from body shape",
                    )
                })?;
            shape.remove(position);
            let bound_set = IndexSet::try_from_iter([*bound])
                .map_err(|_| invariant("operation.index_policy", "integration binder conflict"))?;
            (shape, aligned[0].indices.difference(&bound_set))
        }
        QuantityShapeRule::ExplicitBroadcast => {
            return Err(invariant(
                "operation.index_policy",
                "index manipulation requires its explicit built-in request",
            ));
        }
    })
}

fn tag<T: Copy + Eq>(
    values: &[Option<T>],
    policy: &str,
    source: Option<u16>,
    declared: Option<T>,
) -> Result<Option<T>, QuantityError> {
    match policy {
        "preserve" => Ok(values[usize::from(source.ok_or_else(|| {
            invariant(
                "operation.component_source",
                "missing preserved component source",
            )
        })?)]),
        "require_equal" | "registered_conversion" => {
            let first = values.first().copied().flatten();
            if values.iter().all(|value| *value == first) {
                Ok(first)
            } else {
                Err(invariant(
                    "operation.component_equality",
                    "operand semantic components disagree",
                ))
            }
        }
        "cancel" => {
            let first = values.first().copied().flatten();
            if first.is_some() && values.iter().all(|value| *value == first) {
                Ok(None)
            } else {
                Err(invariant(
                    "operation.component_cancellation",
                    "cancellation requires matching non-absent tags",
                ))
            }
        }
        "declared_result" => Ok(declared),
        _ => Err(invariant(
            "operation.component_policy",
            "unsupported component policy",
        )),
    }
}
fn dimension_for(
    request: &OpRequest<'_>,
    types: &[&QuantityType],
    registry: &QuantityRegistry,
) -> Result<DimensionVector, QuantityError> {
    let dimensions = types
        .iter()
        .map(|ty| registry.kind(ty.key.kind).map(|kind| kind.dimension))
        .collect::<Result<Vec<_>, _>>()?;
    match (request, dimensions.as_slice()) {
        (OpRequest::Mul, [a, b]) => Ok(a.mul(b)?),
        (OpRequest::Div, [a, b]) => Ok(a.div(b)?),
        (
            OpRequest::Sqrt
            | OpRequest::Smooth {
                opcode: Opcode::SafeSqrt,
                ..
            },
            [a],
        ) => Ok(a.root(2)?),
        (OpRequest::Pow { exponent }, [base, power]) => {
            if !power.is_dimensionless() {
                return Err(invariant(
                    "power.exponent",
                    "exponent must be dimensionless",
                ));
            }
            match exponent {
                Exponent::Rational(ratio) => Ok(base.pow(*ratio)?),
                Exponent::Symbolic if base.is_dimensionless() => Ok(*base),
                Exponent::Symbolic => Err(invariant(
                    "power.base",
                    "symbolic exponent requires dimensionless base",
                )),
            }
        }
        (
            OpRequest::Transcendental(_)
            | OpRequest::Smooth {
                opcode: Opcode::SafeLog,
                ..
            },
            [input],
        ) if input.is_dimensionless() => Ok(DimensionVector::DIMENSIONLESS),
        (
            OpRequest::Derivative {
                domain_unit, order, ..
            },
            [body],
        ) if *order > 0 => Ok(body.div(
            &registry
                .unit(*domain_unit)?
                .dimension
                .pow(Ratio::new(i32::from(*order), 1)?)?,
        )?),
        (
            OpRequest::Reduce {
                kind: ReductionKind::Sum,
                ..
            },
            [body],
        ) => Ok(*body),
        (OpRequest::Integral { domain_unit, .. }, [body]) => {
            Ok(body.mul(&registry.unit(*domain_unit)?.dimension)?)
        }
        _ => Err(invariant(
            "operation.dimension",
            "request operands do not satisfy dimensional preconditions",
        )),
    }
}

fn affine(
    operands: &[Operand<'_>],
    signs: &[i8],
    has_constant: bool,
    registry: &QuantityRegistry,
) -> Result<Inferred, QuantityError> {
    count(operands, signs.len() + usize::from(has_constant))?;
    if operands.is_empty() || signs.iter().any(|sign| !matches!(sign, -1 | 1)) {
        return Err(invariant(
            "affine.terms",
            "nonempty operands and signed terms are required",
        ));
    }
    let first = operands[0];
    let mut result = built(
        first.quantity_type,
        first.indices.clone(),
        BuiltInRule::Affine,
    );
    let start = usize::from(!has_constant);
    for (position, sign) in signs.iter().enumerate().skip(start) {
        let term = operands[position + usize::from(has_constant)];
        let prior = Operand {
            quantity_type: result.result,
            indices: &result.indices,
        };
        result = additive(*sign < 0, &[prior, term], registry)?;
    }
    result.selected = OperationSelection::BuiltIn(BuiltInRule::Affine);
    Ok(result)
}
fn smooth(
    operands: &[Operand<'_>],
    opcode: Opcode,
    eps: f64,
    registry: &QuantityRegistry,
    checker: &dyn InvariantChecker,
) -> Result<Inferred, QuantityError> {
    if !eps.is_finite() || eps <= 0.0 {
        return Err(QuantityError::StaticDomain {
            opcode,
            restriction: "a finite positive smoothing parameter",
            value: eps,
        });
    }
    let first = operands.first().ok_or_else(|| {
        invariant(
            "smooth.operand",
            "smoothing requires an actual first operand",
        )
    })?;
    crate::smoothing::resolve_epsilon(opcode, first.quantity_type, eps, None, registry)?;
    match opcode {
        Opcode::SmoothMax | Opcode::SmoothMin => {
            count(operands, 2)?;
            same(operands, registry)?;
            Ok(built(
                operands[0].quantity_type,
                operands[0].indices.clone(),
                BuiltInRule::Smooth,
            ))
        }
        Opcode::SmoothAbs => {
            count(operands, 1)?;
            Ok(built(
                operands[0].quantity_type,
                operands[0].indices.clone(),
                BuiltInRule::Smooth,
            ))
        }
        Opcode::SafeSqrt | Opcode::SafeLog => registered(
            &OpRequest::Smooth { opcode, eps },
            operands,
            registry,
            checker,
        ),
        _ => Err(invariant(
            "smooth.opcode",
            "opcode has no smooth-parameter contract",
        )),
    }
}
