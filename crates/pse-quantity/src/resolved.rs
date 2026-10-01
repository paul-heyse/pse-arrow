// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Admitted physical contracts for named boundaries and anonymous intermediates.
//! Semantic monomials and contextual witnesses are independent of numerical coordinates.

use crate::infer::{
    BuiltInRule, Exponent, Inferred, InvariantChecker, OpRequest, Operand, OperandConversion,
    OperationSelection,
};
use crate::{
    DimensionVector, EntityKindId, IndexSet, KindFactor, QuantityError, QuantityRegistry,
    QuantityTypeId, QuantityTypeKey, Ratio, ScaleKind, UnitFactor, UnitId,
};
use std::collections::BTreeMap;

/// Scoped authority of one admitted scientific response formula. It never changes the
/// rules of ordinary arithmetic or overrides a matched registered operation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalFormulaAuthority(pse_ids::SemanticId);
impl PhysicalFormulaAuthority {
    /// Established by the declaration layer after checking the selected potential witness.
    pub const fn response(operation: pse_ids::SemanticId) -> Self {
        Self(operation)
    }
    /// The admitted response occurrence, including its selected scientific witness.
    pub const fn identity(&self) -> pse_ids::SemanticId {
        self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct FormulaEvidence {
    authority: PhysicalFormulaAuthority,
    operation: &'static str,
    operands: Vec<ResolvedPhysicalContract>,
    powers: Vec<Ratio>,
}

/// A declared scientific operation's checked output authorization. The declaration
/// layer establishes the semantic role; numeric lowering verifies this exact source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedOutputBoundary {
    operation: pse_ids::SemanticId,
    source: ResolvedPhysicalContract,
    target: ResolvedPhysicalContract,
    scale_bits: u64,
}
impl AdmittedOutputBoundary {
    /// Construct after operation-specific semantic admission, retaining both contracts.
    /// This is not an expected-type conversion: `operation` must identify an admitted
    /// coordinate, reconstruction, response, translation or transfer declaration.
    pub fn declared(
        operation: pse_ids::SemanticId,
        source: ResolvedPhysicalContract,
        target: QuantityTypeId,
        registry: &QuantityRegistry,
    ) -> Result<Self, QuantityError> {
        let target = ResolvedPhysicalContract::named(target, source.indices.clone(), registry)?;
        Self::declared_contract(operation, source, target)
    }
    /// Authorize an admitted anonymous target without manufacturing a public type ID.
    pub fn declared_contract(
        operation: pse_ids::SemanticId,
        source: ResolvedPhysicalContract,
        target: ResolvedPhysicalContract,
    ) -> Result<Self, QuantityError> {
        if source.representation.dimension != target.representation.dimension {
            return Err(refusal(
                "physical.operation_boundary",
                "declared physical operation result dimension differs from its formula",
            ));
        }
        let scale = checked_scale(source.representation.scale() / target.representation.scale())?;
        Ok(Self {
            operation,
            source,
            target,
            scale_bits: scale.to_bits(),
        })
    }
    /// Verify the exact admitted source and return the authorized output and coordinate scale.
    pub fn apply_contract(
        &self,
        source: &ResolvedPhysicalContract,
    ) -> Result<(ResolvedPhysicalContract, f64), QuantityError> {
        if source != &self.source {
            return Err(refusal(
                "physical.operation_boundary",
                "lowered formula differs from the admitted source contract",
            ));
        }
        Ok((self.target.clone(), f64::from_bits(self.scale_bits)))
    }
    /// Frame semantic authority together with both physical contracts and numerical scale.
    pub fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.id(&self.operation);
        self.source.frame(hash);
        self.target.frame(hash);
        hash.u64(self.scale_bits);
    }
}

/// Retained admission for one operation. Numerical lowering consumes these scales and
/// this result; it does not infer a different contract from the normalized expression.
#[derive(Clone, Debug)]
pub struct ResolvedInference {
    /// Original physical operands, retaining evidence even when exact factors cancel.
    pub operands: Vec<ResolvedPhysicalContract>,
    /// Complete admitted result, including anonymous physical intermediates.
    pub result: ResolvedPhysicalContract,
    /// Selected physical authority for this occurrence.
    pub selected: OperationSelection,
    /// Explicit registered conversions required by that authority.
    pub conversions: Vec<OperandConversion>,
    /// Multipliers applied to ordered operands before the numerical operation.
    pub operand_scales: Vec<f64>,
    /// Multiplier applied after the numerical operation.
    pub result_scale: f64,
}
impl ResolvedInference {
    /// Frame the actual admitted operation and numerical lowering, before normalization.
    pub fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.u64(self.operands.len() as u64);
        for operand in &self.operands {
            operand.frame(hash);
        }
        self.result.frame(hash);
        match &self.selected {
            OperationSelection::BuiltIn(rule) => {
                hash.str("builtin").str(rule.as_str());
            }
            OperationSelection::Registered {
                operation,
                operand_permutation,
            } => {
                hash.str("registered")
                    .id(&operation.as_id())
                    .u64(operand_permutation.len() as u64);
                for position in operand_permutation {
                    hash.u64(u64::from(*position));
                }
            }
        }
        hash.u64(self.conversions.len() as u64);
        for conversion in &self.conversions {
            hash.u64(u64::from(conversion.operand))
                .id(&conversion.conversion.as_id());
        }
        hash.u64(self.operand_scales.len() as u64);
        for scale in &self.operand_scales {
            hash.u64(scale.to_bits());
        }
        hash.u64(self.result_scale.to_bits());
    }
    /// Require a named result for a declaration boundary.
    pub fn named_result(self) -> Result<Inferred, QuantityError> {
        Ok(Inferred {
            result: self.result.require_named()?,
            indices: self.result.indices.clone(),
            selected: self.selected,
            conversions: self.conversions,
        })
    }
}

/// Admit one operation over full physical contracts. Registered matches are authoritative:
/// neither a failed prerequisite nor ambiguity can be rescued by dimension algebra.
pub fn infer_operation(
    request: &OpRequest<'_>,
    values: &[ResolvedPhysicalContract],
    expected: Option<QuantityTypeId>,
    registry: &QuantityRegistry,
    checker: &dyn InvariantChecker,
) -> Result<ResolvedInference, QuantityError> {
    infer_in_context(request, values, expected, registry, checker, None)
}

/// Admit an explicit partial through the same difference and quotient rules, retaining
/// the complete physical arguments and the derivative's numerical coordinate change.
pub fn infer_partial(
    value: &ResolvedPhysicalContract,
    arguments: &[ResolvedPhysicalContract],
    registry: &QuantityRegistry,
    checker: &dyn InvariantChecker,
) -> Result<ResolvedInference, QuantityError> {
    let mut result = value.clone();
    let mut raw_scale = value.representation.scale();
    for argument in arguments {
        let numerator = infer_operation(
            &OpRequest::Sub,
            &[result.clone(), result],
            None,
            registry,
            checker,
        )?
        .result;
        let denominator = infer_operation(
            &OpRequest::Sub,
            &[argument.clone(), argument.clone()],
            None,
            registry,
            checker,
        )?
        .result;
        result = infer_operation(
            &OpRequest::Div,
            &[numerator, denominator],
            None,
            registry,
            checker,
        )?
        .result;
        raw_scale = checked_scale(raw_scale / argument.representation.scale())?;
    }
    let result_scale = checked_scale(raw_scale / result.representation.scale())?;
    Ok(ResolvedInference {
        operands: std::iter::once(value.clone())
            .chain(arguments.iter().cloned())
            .collect(),
        result,
        selected: OperationSelection::BuiltIn(BuiltInRule::Partial),
        conversions: vec![],
        operand_scales: vec![],
        result_scale,
    })
}

/// The single operation authority with optional declaration-owned scientific formula
/// scope. A scope permits dimensionally consistent response sums, retaining both terms
/// until the owning output authorization supplies their public physical role.
pub fn infer_in_context(
    request: &OpRequest<'_>,
    values: &[ResolvedPhysicalContract],
    expected: Option<QuantityTypeId>,
    registry: &QuantityRegistry,
    checker: &dyn InvariantChecker,
    authority: Option<&PhysicalFormulaAuthority>,
) -> Result<ResolvedInference, QuantityError> {
    let named = values
        .iter()
        .map(|value| {
            value.named.map(|quantity_type| Operand {
                quantity_type,
                indices: &value.indices,
            })
        })
        .collect::<Option<Vec<_>>>();
    let multiplicative = matches!(
        request,
        OpRequest::Mul
            | OpRequest::Div
            | OpRequest::Pow {
                exponent: Exponent::Rational(_)
            }
            | OpRequest::Sqrt
    );
    let raw_representation = || -> Result<UnitRepresentation, QuantityError> {
        let powers: Vec<Ratio> = match request {
            OpRequest::Mul => vec![Ratio::ONE, Ratio::ONE],
            OpRequest::Div => vec![Ratio::ONE, Ratio::new(-1, 1)?],
            OpRequest::Pow {
                exponent: Exponent::Rational(power),
            } => vec![*power],
            OpRequest::Sqrt => vec![Ratio::new(1, 2)?],
            _ => {
                return values
                    .first()
                    .map(|value| value.representation.clone())
                    .ok_or_else(|| refusal("physical.arity", "operation needs an operand"));
            }
        };
        let mut factors = Vec::new();
        let mut dimension = DimensionVector::DIMENSIONLESS;
        let mut scale = 1.0;
        for (value, power) in values.iter().zip(powers) {
            dimension = dimension.mul(&value.representation.dimension.pow(power)?)?;
            scale *= value
                .representation
                .scale()
                .powf(f64::from(power.num()) / f64::from(power.den()));
            for factor in &value.representation.factors {
                factors.push(UnitFactor {
                    unit: factor.unit,
                    exponent: factor.exponent.checked_mul(power)?,
                });
            }
        }
        Ok(UnitRepresentation {
            factors: crate::unit::canonical_factors(factors)?,
            dimension,
            scale_bits: checked_scale(scale)?.to_bits(),
        })
    };
    let registered = if let Some(operands) = &named {
        crate::infer::has_registered(request, operands, registry)?
    } else {
        false
    };
    let mut result = if registered {
        let admitted = crate::infer::registered_contract(
            request,
            named.as_ref().ok_or_else(|| {
                refusal(
                    "physical.registered",
                    "registered operands require named contracts",
                )
            })?,
            registry,
            checker,
        )?;
        let representation = match registry.resolve_key(&admitted.key) {
            Ok(id) => UnitRepresentation::named(
                registry.quantity_type(id)?.canonical_unit,
                admitted.key.scale_kind,
                registry,
            )?,
            Err(_) => raw_representation()?,
        };
        let contract = ResolvedPhysicalContract::from_key(
            &admitted.key,
            admitted.indices,
            representation,
            registry,
        )?;
        ResolvedInference {
            operands: values.to_vec(),
            result: contract,
            selected: admitted.selected,
            conversions: admitted.conversions,
            operand_scales: vec![1.0; values.len()],
            result_scale: 1.0,
        }
    } else if matches!(request, OpRequest::Add | OpRequest::Sub)
        && authority.is_some()
        && values.len() == 2
        && !values[0].same_meaning(&values[1])
        && values[0]
            .named
            .zip(values[1].named)
            .is_none_or(|(left, right)| {
                registry
                    .quantity_type(left)
                    .ok()
                    .zip(registry.quantity_type(right).ok())
                    .is_none_or(|(left, right)| left.key.kind != right.key.kind)
            })
    {
        if values[0].representation.dimension != values[1].representation.dimension
            || values[0].indices != values[1].indices
            || values[0].axes != values[1].axes
        {
            return Err(refusal(
                "physical.response_sum",
                "response terms differ in dimensions or actual index contract",
            ));
        }
        let mut contract = values[0].clone();
        contract.named = None;
        contract.kinds.clear();
        contract.pure_number = false;
        contract.qualified.extend(values[1].qualified.clone());
        contract.qualifiers()?;
        contract.qualified.sort();
        contract.qualified.dedup();
        contract.formula = Some(Box::new(FormulaEvidence {
            authority: authority
                .ok_or_else(|| refusal("physical.response_scope", "response authority is absent"))?
                .clone(),
            operation: if matches!(request, OpRequest::Add) {
                "add"
            } else {
                "sub"
            },
            operands: values.to_vec(),
            powers: vec![],
        }));
        ResolvedInference {
            operands: values.to_vec(),
            result: contract,
            selected: OperationSelection::BuiltIn(BuiltInRule::PhysicalFormula),
            conversions: vec![],
            operand_scales: vec![1.0; values.len()],
            result_scale: 1.0,
        }
    } else if multiplicative {
        let terms = match request {
            OpRequest::Mul if values.len() == 2 => {
                vec![(&values[0], Ratio::ONE), (&values[1], Ratio::ONE)]
            }
            OpRequest::Div if values.len() == 2 => {
                vec![(&values[0], Ratio::ONE), (&values[1], Ratio::new(-1, 1)?)]
            }
            OpRequest::Pow {
                exponent: Exponent::Rational(power),
            } if values.len() == 2 && values[1].pure_number && values[1].indices.is_empty() => {
                vec![(&values[0], *power)]
            }
            OpRequest::Sqrt if values.len() == 1 => vec![(&values[0], Ratio::new(1, 2)?)],
            _ => {
                return Err(refusal(
                    "physical.power",
                    "multiplication needs two operands; an exact power needs a neutral scalar exponent",
                ));
            }
        };
        // Pure scalar scaling preserves point semantics; it does not manufacture an
        // affine product between two physical factors.
        let preserved = match request {
            OpRequest::Mul | OpRequest::Div
                if values[1].pure_number && values[1].indices.is_empty() =>
            {
                Some(0)
            }
            OpRequest::Mul if values[0].pure_number && values[0].indices.is_empty() => Some(1),
            _ => None,
        };
        let contract = if let Some(position) = preserved {
            values[position].clone()
        } else {
            ResolvedPhysicalContract::monomial(&terms, registry)?.0
        };
        let rule = if preserved.is_some() {
            if values.iter().any(|value| {
                value
                    .named
                    .is_some_and(|id| registry.discrete_category(id).ok().flatten().is_some())
            }) {
                BuiltInRule::DiscreteScaling
            } else {
                BuiltInRule::NeutralScaling
            }
        } else {
            BuiltInRule::Chain
        };
        ResolvedInference {
            operands: values.to_vec(),
            result: contract,
            selected: OperationSelection::BuiltIn(rule),
            conversions: vec![],
            operand_scales: vec![1.0; values.len()],
            result_scale: 1.0,
        }
    } else if matches!(request, OpRequest::Add | OpRequest::Sub) && named.is_some() {
        let (key, indices) = crate::infer::additive_contract(
            matches!(request, OpRequest::Sub),
            named.as_ref().ok_or_else(|| {
                refusal("physical.additive", "named additive operands are absent")
            })?,
            registry,
        )?;
        let representation = match registry.resolve_key(&key) {
            Ok(id) => UnitRepresentation::named(
                registry.quantity_type(id)?.canonical_unit,
                key.scale_kind,
                registry,
            )?,
            Err(_) => values[0].representation.clone(),
        };
        ResolvedInference {
            operands: values.to_vec(),
            result: ResolvedPhysicalContract::from_key(&key, indices, representation, registry)?,
            selected: OperationSelection::BuiltIn(BuiltInRule::Addition),
            conversions: vec![],
            operand_scales: vec![1.0; values.len()],
            result_scale: 1.0,
        }
    } else if let Some(operands) = named {
        let admitted = crate::infer::infer_named(request, &operands, registry, checker)?;
        ResolvedInference {
            operands: values.to_vec(),
            result: ResolvedPhysicalContract::named(admitted.result, admitted.indices, registry)?,
            selected: admitted.selected,
            conversions: admitted.conversions,
            operand_scales: vec![1.0; values.len()],
            result_scale: 1.0,
        }
    } else {
        let first = values
            .first()
            .ok_or_else(|| refusal("physical.arity", "operation needs an operand"))?;
        let (contract, rule) = match request {
            OpRequest::Neg | OpRequest::Abs if values.len() == 1 => {
                (first.clone(), BuiltInRule::Unary)
            }
            OpRequest::Add | OpRequest::Sub | OpRequest::Conditional
                if values.len() == 2 && first.same_meaning(&values[1]) =>
            {
                (first.clone(), BuiltInRule::Addition)
            }
            OpRequest::FiniteReduce { kind, .. }
                if values.len() == 1
                    && first.indices.is_empty()
                    && (*kind != crate::ReductionKind::Prod || first.pure_number) =>
            {
                (first.clone(), BuiltInRule::Reduction)
            }
            OpRequest::Reduce { kind, bound }
                if values.len() == 1
                    && (*kind != crate::ReductionKind::Prod || first.pure_number) =>
            {
                let mut contract = first.clone();
                if !contract.indices.contains(bound) {
                    return Err(refusal("physical.reduction", "bound index is absent"));
                }
                let removed = IndexSet::try_from_iter([*bound])
                    .map_err(|_| refusal("physical.reduction", "invalid binder"))?;
                contract.indices = contract.indices.difference(&removed);
                let position = contract
                    .axes
                    .iter()
                    .position(|kind| *kind == bound.kind)
                    .ok_or_else(|| refusal("physical.reduction", "bound axis is absent"))?;
                contract.axes.remove(position);
                contract.named = None;
                (contract.resolve_named(registry)?.0, BuiltInRule::Reduction)
            }
            OpRequest::Broadcast { index } if values.len() == 1 => {
                let mut contract = first.clone();
                if !contract
                    .indices
                    .insert(*index)
                    .map_err(|_| refusal("physical.broadcast", "binder conflict"))?
                {
                    return Err(refusal("physical.broadcast", "binder already present"));
                }
                contract.axes.push(index.kind);
                contract.named = None;
                (contract.resolve_named(registry)?.0, BuiltInRule::Broadcast)
            }
            _ => {
                return Err(refusal(
                    "physical.anonymous_operation",
                    "operation requires matching complete contracts or an explicit named boundary",
                ));
            }
        };
        ResolvedInference {
            operands: values.to_vec(),
            result: contract,
            selected: OperationSelection::BuiltIn(rule),
            conversions: vec![],
            operand_scales: vec![1.0; values.len()],
            result_scale: 1.0,
        }
    };
    if multiplicative {
        result.result_scale =
            checked_scale(raw_representation()?.scale() / result.result.representation.scale())?;
    } else if matches!(
        request,
        OpRequest::Add
            | OpRequest::Sub
            | OpRequest::Conditional
            | OpRequest::Neg
            | OpRequest::Abs
            | OpRequest::Reduce { .. }
            | OpRequest::FiniteReduce { .. }
            | OpRequest::Broadcast { .. }
    ) {
        for (scale, value) in result.operand_scales.iter_mut().zip(values) {
            *scale =
                checked_scale(value.representation.scale() / result.result.representation.scale())?;
        }
    }
    if let Some(expected) = expected {
        let (contract, scale) = result.result.at_boundary(expected, registry)?;
        result.result = contract;
        result.result_scale = checked_scale(result.result_scale * scale)?;
    }
    Ok(result)
}

/// One complete contextual factor. Cancellation requires equality of its full key,
/// rather than equality of dimensions or unqualified kind factors.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct QualifiedFactor {
    /// Kind, basis, datum, point/difference and subject; axes live on the value contract.
    pub key: QuantityTypeKey,
    /// Exact signed power of this context.
    pub exponent: Ratio,
}

/// Numerical coordinates of a physical value; a product has no invented public unit ID.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitRepresentation {
    factors: Vec<UnitFactor>,
    dimension: DimensionVector,
    scale_bits: u64,
}
impl UnitRepresentation {
    pub(crate) fn named(
        unit: UnitId,
        _scale: ScaleKind,
        registry: &QuantityRegistry,
    ) -> Result<Self, QuantityError> {
        let unit = registry.unit(unit)?;
        if unit.is_affine || unit.offset_to_canonical != 0.0 {
            return Err(refusal(
                "physical.representation",
                "canonical storage coordinates must be linear",
            ));
        }
        Ok(Self {
            factors: unit.definition.clone().unwrap_or_else(|| {
                vec![UnitFactor {
                    unit: unit.id,
                    exponent: Ratio::ONE,
                }]
            }),
            dimension: unit.dimension,
            scale_bits: unit.scale_to_canonical.to_bits(),
        })
    }
    /// Declared atomic unit factors in canonical identity order.
    pub fn factors(&self) -> &[UnitFactor] {
        &self.factors
    }
    /// Physical dimension, a check rather than semantic authority.
    pub const fn dimension(&self) -> DimensionVector {
        self.dimension
    }
    /// Finite positive numerical scale to SI coordinates.
    pub fn scale(&self) -> f64 {
        f64::from_bits(self.scale_bits)
    }
}

/// One immutable physical expression contract. Named identities are retained only when
/// the complete contract resolves to a declaration; intermediates never allocate IDs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResolvedPhysicalContract {
    named: Option<QuantityTypeId>,
    kinds: Vec<KindFactor>,
    qualified: Vec<QualifiedFactor>,
    axes: Vec<EntityKindId>,
    indices: IndexSet,
    representation: UnitRepresentation,
    pure_number: bool,
    formula: Option<Box<FormulaEvidence>>,
}
/// The common basis, reference state and subject kind of a contract's qualified factors.
type Qualifiers = (
    Option<crate::BasisId>,
    Option<crate::ReferenceStateId>,
    Option<EntityKindId>,
);
impl ResolvedPhysicalContract {
    /// Conservative owned extent of the retained contract, excluding its inline value.
    pub fn heap_bytes(&self) -> usize {
        self.kinds.capacity() * size_of::<KindFactor>()
            + self.qualified.capacity() * size_of::<QualifiedFactor>()
            + self
                .qualified
                .iter()
                .map(|factor| factor.key.shape.capacity() * size_of::<EntityKindId>())
                .sum::<usize>()
            + self.axes.capacity() * size_of::<EntityKindId>()
            + self.indices.len() * 2048
            + self.representation.factors.capacity() * size_of::<UnitFactor>()
            + self.formula.as_ref().map_or(0, |formula| {
                size_of::<FormulaEvidence>()
                    + formula.operands.capacity() * size_of::<Self>()
                    + formula.operands.iter().map(Self::heap_bytes).sum::<usize>()
                    + formula.powers.capacity() * size_of::<Ratio>()
            })
    }
    /// Stable structural identity, independent of a diagnostic or symbolic rendering.
    pub fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.u64(u64::from(self.named.is_some()));
        if let Some(id) = self.named {
            hash.id(&id.as_id());
        }
        hash.u64(self.kinds.len() as u64);
        for factor in &self.kinds {
            hash.id(&factor.kind.as_id())
                .part(&factor.exponent.num().to_le_bytes())
                .part(&factor.exponent.den().to_le_bytes());
        }
        hash.u64(self.qualified.len() as u64);
        for factor in &self.qualified {
            let key = &factor.key;
            hash.id(&key.kind.as_id()).str(key.scale_kind.as_str());
            for id in [
                key.basis.map(|id| id.as_id()),
                key.reference_state.map(|id| id.as_id()),
                key.subject_kind.map(|id| id.as_id()),
            ] {
                hash.u64(u64::from(id.is_some()));
                if let Some(id) = id {
                    hash.id(&id);
                }
            }
            hash.part(&factor.exponent.num().to_le_bytes())
                .part(&factor.exponent.den().to_le_bytes());
        }
        hash.u64(self.axes.len() as u64);
        for axis in &self.axes {
            hash.id(&axis.as_id());
        }
        hash.u64(self.indices.len() as u64);
        for index in self.indices.iter() {
            hash.id(&index.bound_index.as_id())
                .id(&index.domain.as_id())
                .id(&index.kind.as_id());
        }
        hash.u64(self.representation.factors.len() as u64);
        for factor in &self.representation.factors {
            hash.id(&factor.unit.as_id())
                .part(&factor.exponent.num().to_le_bytes())
                .part(&factor.exponent.den().to_le_bytes());
        }
        hash.part(&self.representation.dimension.canonical_bytes())
            .u64(self.representation.scale_bits)
            .u64(u64::from(self.pure_number));
        hash.bool(self.formula.is_some());
        if let Some(formula) = &self.formula {
            hash.id(&formula.authority.identity())
                .str(formula.operation)
                .u64(formula.operands.len() as u64);
            for operand in &formula.operands {
                operand.frame(hash);
            }
            hash.u64(formula.powers.len() as u64);
            for power in &formula.powers {
                hash.part(&power.num().to_le_bytes())
                    .part(&power.den().to_le_bytes());
            }
        }
    }
    /// Resolve a declared quantity with its actual free lexical indices.
    pub fn named(
        quantity: QuantityTypeId,
        indices: IndexSet,
        registry: &QuantityRegistry,
    ) -> Result<Self, QuantityError> {
        let ty = registry.quantity_type(quantity)?;
        let mut expected = ty.key.shape.clone();
        expected.sort_unstable();
        let mut actual: Vec<_> = indices.iter().map(|index| index.kind).collect();
        actual.sort_unstable();
        if expected != actual {
            return Err(refusal(
                "physical.indices",
                "free binders disagree with declared ordered axes",
            ));
        }
        Self::from_key(
            &ty.key,
            indices,
            UnitRepresentation::named(ty.canonical_unit, ty.key.scale_kind, registry)?,
            registry,
        )
    }

    pub(crate) fn from_key(
        key: &QuantityTypeKey,
        indices: IndexSet,
        representation: UnitRepresentation,
        registry: &QuantityRegistry,
    ) -> Result<Self, QuantityError> {
        let named = registry.resolve_key(key).ok();
        let pure_number = named.is_some_and(|id| {
            registry.neutral_dimensionless() == Some(id)
                || registry.discrete_category(id).ok().flatten().is_some()
        });
        let mut scalar_key = key.clone();
        scalar_key.shape.clear();
        Ok(Self {
            named,
            kinds: if pure_number {
                vec![]
            } else {
                registry.kind_monomial(key.kind)?
            },
            qualified: if pure_number {
                vec![]
            } else {
                vec![QualifiedFactor {
                    key: scalar_key,
                    exponent: Ratio::ONE,
                }]
            },
            axes: key.shape.clone(),
            indices,
            representation,
            pure_number,
            formula: None,
        })
    }

    /// Declared public identity, absent for a semantic intermediate.
    pub const fn named_id(&self) -> Option<QuantityTypeId> {
        self.named
    }
    /// Require a public named boundary; dimensions alone cannot supply one.
    pub fn require_named(&self) -> Result<QuantityTypeId, QuantityError> {
        self.named.ok_or_else(|| {
            refusal(
                "physical.named_boundary",
                "anonymous intermediate requires an admitted named result boundary",
            )
        })
    }
    /// Canonical monomial over declared base kinds.
    pub fn kind_factors(&self) -> &[KindFactor] {
        &self.kinds
    }
    /// Contextual factors still carried by this expression.
    pub fn qualified_factors(&self) -> &[QualifiedFactor] {
        &self.qualified
    }
    /// Ordered declared group axes.
    pub fn axes(&self) -> &[EntityKindId] {
        &self.axes
    }
    /// Actual lexical binder identities, independent of algebraic cancellation.
    pub fn indices(&self) -> &IndexSet {
        &self.indices
    }
    /// Numerical coordinate representation.
    pub fn representation(&self) -> &UnitRepresentation {
        &self.representation
    }
    /// Whether admission established a neutral/count/indicator coefficient.
    pub const fn is_pure_number(&self) -> bool {
        self.pure_number
    }

    /// Admission at an expected named boundary, including the explicit numerical scale
    /// from the current coordinates to that declaration's canonical coordinates.
    pub fn at_boundary(
        &self,
        expected: QuantityTypeId,
        registry: &QuantityRegistry,
    ) -> Result<(Self, f64), QuantityError> {
        if let Some(actual) = self.named {
            crate::admission::require_same_contract(expected, actual, registry)?;
        } else if !self.matches_key(&registry.quantity_type(expected)?.key, registry)? {
            return Err(refusal(
                "physical.named_boundary",
                "semantic factors or contextual obligations do not match the expected quantity",
            ));
        }
        let target = Self::named(expected, self.indices.clone(), registry)?;
        let scale = checked_scale(self.representation.scale() / target.representation.scale())?;
        Ok((target, scale))
    }
    /// Admit a complete function contract, which can itself describe an anonymous
    /// product rather than a public quantity name.
    pub fn at_contract_boundary(
        &self,
        expected: &Self,
        registry: &QuantityRegistry,
    ) -> Result<(Self, f64), QuantityError> {
        if let Some(id) = expected.named {
            return self.at_boundary(id, registry);
        }
        if !self.same_meaning(expected) {
            return Err(refusal(
                "physical.contract_boundary",
                "complete anonymous argument or result contract differs",
            ));
        }
        Ok((
            expected.clone(),
            checked_scale(self.representation.scale() / expected.representation.scale())?,
        ))
    }

    /// Equality of physical meaning and actual binders, independent of numerical units.
    pub fn same_meaning(&self, other: &Self) -> bool {
        self.kinds == other.kinds
            && self.qualified == other.qualified
            && self.axes == other.axes
            && self.indices == other.indices
            && self.pure_number == other.pure_number
            && self.formula == other.formula
    }

    fn matches_key(
        &self,
        key: &QuantityTypeKey,
        registry: &QuantityRegistry,
    ) -> Result<bool, QuantityError> {
        if self.formula.is_some() {
            return Ok(false);
        }
        if self.axes != key.shape {
            return Ok(false);
        }
        let neutral = registry
            .neutral_dimensionless()
            .and_then(|id| registry.quantity_type(id).ok());
        if self.kinds.is_empty() {
            return Ok(self.qualified.is_empty()
                && neutral.is_some_and(|ty| {
                    let mut candidate = ty.key.clone();
                    candidate.shape = self.axes.clone();
                    candidate == *key
                }));
        }
        if registry.kind_monomial(key.kind)? != self.kinds {
            return Ok(false);
        }
        let kind = registry.kind(key.kind)?;
        let (basis, reference, subject) = self.qualifiers()?;
        let mut qualifiers = (basis, reference, subject);
        if let Some(definition) = &kind.definition {
            if definition
                .basis
                .is_some_and(|declared| basis.is_some_and(|carried| declared != carried))
                || definition
                    .reference_state
                    .is_some_and(|declared| reference.is_some_and(|carried| declared != carried))
                || definition
                    .subject_kind
                    .is_some_and(|declared| subject.is_some_and(|carried| declared != carried))
            {
                return Ok(false);
            }
            qualifiers = (
                definition.basis.or(basis),
                definition.reference_state.or(reference),
                definition.subject_kind.or(subject),
            );
        }
        if (key.basis, key.reference_state, key.subject_kind) != qualifiers {
            return Ok(false);
        }
        let expected_scale = if let Some(definition) = &kind.definition {
            definition.scale_kind
        } else if let [factor] = self.qualified.as_slice()
            && factor.key.kind == key.kind
            && factor.exponent == Ratio::ONE
        {
            factor.key.scale_kind
        } else if kind.addition_kind == crate::QuantityAdditionKind::OriginSensitive {
            return Ok(false);
        } else {
            ScaleKind::Point
        };
        Ok(key.scale_kind == expected_scale)
    }

    fn qualifiers(&self) -> Result<Qualifiers, QuantityError> {
        fn common<T: Copy + Eq>(
            values: impl Iterator<Item = Option<T>>,
            rule: &'static str,
        ) -> Result<Option<T>, QuantityError> {
            let mut selected = None;
            for value in values.flatten() {
                if selected.is_some_and(|prior| prior != value) {
                    return Err(refusal(
                        rule,
                        "incompatible contextual factors require an explicit physical operation",
                    ));
                }
                selected = Some(value);
            }
            Ok(selected)
        }
        Ok((
            common(self.qualified.iter().map(|f| f.key.basis), "physical.basis")?,
            common(
                self.qualified.iter().map(|f| f.key.reference_state),
                "physical.reference",
            )?,
            common(
                self.qualified.iter().map(|f| f.key.subject_kind),
                "physical.subject",
            )?,
        ))
    }

    fn resolve_named(mut self, registry: &QuantityRegistry) -> Result<(Self, f64), QuantityError> {
        let candidates = registry
            .quantity_types()
            .filter_map(|ty| match self.matches_key(&ty.key, registry) {
                Ok(true) => Some(Ok(ty.id)),
                Ok(false) => None,
                Err(error) => Some(Err(error)),
            })
            .collect::<Result<Vec<_>, _>>()?;
        if candidates.len() > 1 {
            return Err(refusal(
                "physical.named_ambiguity",
                "several declared quantities match the complete intermediate contract",
            ));
        }
        if let Some(id) = candidates.first() {
            return self.at_boundary(*id, registry);
        }
        self.named = None;
        Ok((self, 1.0))
    }

    /// Compose an admitted product/power while retaining qualified obligations and all
    /// free binders. Called only after registered-operation selection has not refused.
    pub(crate) fn monomial(
        terms: &[(&Self, Ratio)],
        registry: &QuantityRegistry,
    ) -> Result<(Self, f64), QuantityError> {
        let mut kinds = BTreeMap::<crate::QuantityKindId, Ratio>::new();
        let mut qualified = BTreeMap::<QuantityTypeKey, Ratio>::new();
        let mut unit_factors = Vec::new();
        let mut axes = Vec::new();
        let mut indices = IndexSet::new();
        let mut all_qualifiers = Vec::new();
        let mut dimension = DimensionVector::DIMENSIONLESS;
        let mut scale = 1.0;
        for (value, exponent) in terms {
            dimension = dimension.mul(&value.representation.dimension.pow(*exponent)?)?;
            scale *= value
                .representation
                .scale()
                .powf(f64::from(exponent.num()) / f64::from(exponent.den()));
            for factor in &value.qualified {
                if factor.key.scale_kind == ScaleKind::Point
                    && factor.key.reference_state.is_some()
                    && !registry.kind(factor.key.kind)?.dimension.is_dimensionless()
                {
                    return Err(refusal(
                        "physical.affine_product",
                        "a nonzero-datum point requires an explicit physical product operation",
                    ));
                }
            }
            all_qualifiers.extend(value.qualified.iter().cloned());
            for factor in &value.kinds {
                let slot = kinds.entry(factor.kind).or_insert(Ratio::ZERO);
                *slot = slot.checked_add(factor.exponent.checked_mul(*exponent)?)?;
            }
            for factor in &value.qualified {
                let slot = qualified.entry(factor.key.clone()).or_insert(Ratio::ZERO);
                *slot = slot.checked_add(factor.exponent.checked_mul(*exponent)?)?;
            }
            unit_factors.extend(
                value
                    .representation
                    .factors
                    .iter()
                    .map(|factor| {
                        Ok(UnitFactor {
                            unit: factor.unit,
                            exponent: factor.exponent.checked_mul(*exponent)?,
                        })
                    })
                    .collect::<Result<Vec<_>, QuantityError>>()?,
            );
            // Axes are attached to actual binders, never removed by factor cancellation.
            let mut remaining = value.indices.iter().copied().collect::<Vec<_>>();
            for axis in &value.axes {
                let position = remaining
                    .iter()
                    .position(|index| index.kind == *axis)
                    .ok_or_else(|| {
                        refusal("physical.axes", "ordered axis has no matching free binder")
                    })?;
                let index = remaining.remove(position);
                if indices.insert(index).map_err(|_| {
                    refusal("physical.binders", "conflicting lexical binder domains")
                })? {
                    axes.push(*axis);
                }
            }
            if !remaining.is_empty() {
                return Err(refusal("physical.axes", "free binder has no declared axis"));
            }
        }
        let mut result = Self {
            named: None,
            kinds: kinds
                .into_iter()
                .filter(|(_, exponent)| !exponent.is_zero())
                .map(|(kind, exponent)| KindFactor { kind, exponent })
                .collect(),
            qualified: all_qualifiers,
            axes,
            indices,
            representation: UnitRepresentation {
                factors: crate::unit::canonical_factors(unit_factors)?,
                dimension,
                scale_bits: checked_scale(scale)?.to_bits(),
            },
            pure_number: false,
            formula: None,
        };
        // Check before cancellation: an unlike datum/basis/subject cannot disappear.
        result.qualifiers()?;
        result.qualified = qualified
            .into_iter()
            .filter(|(_, exponent)| !exponent.is_zero())
            .map(|(key, exponent)| QualifiedFactor { key, exponent })
            .collect();
        result.pure_number = result.kinds.is_empty() && result.qualified.is_empty();
        if let Some(authority) = terms
            .iter()
            .find_map(|(value, _)| value.formula.as_ref().map(|formula| &formula.authority))
        {
            if terms.iter().any(|(value, _)| {
                value
                    .formula
                    .as_ref()
                    .is_some_and(|formula| &formula.authority != authority)
            }) {
                return Err(refusal(
                    "physical.response_scope",
                    "scientific formulas from different declarations cannot mix before their boundaries",
                ));
            }
            result.formula = Some(Box::new(FormulaEvidence {
                authority: authority.clone(),
                operation: "product",
                operands: terms.iter().map(|(value, _)| (*value).clone()).collect(),
                powers: terms.iter().map(|(_, power)| *power).collect(),
            }));
            result.pure_number = false;
        }
        result.resolve_named(registry)
    }
}

pub(crate) fn checked_scale(scale: f64) -> Result<f64, QuantityError> {
    if !scale.is_finite() || scale <= 0.0 {
        Err(refusal(
            "physical.coordinate_scale",
            "expression coordinate conversion is not finite and positive",
        ))
    } else {
        Ok(scale)
    }
}
pub(crate) fn refusal(rule: &'static str, detail: &str) -> QuantityError {
    QuantityError::InferencePrecondition {
        rule,
        detail: detail.into(),
    }
}
