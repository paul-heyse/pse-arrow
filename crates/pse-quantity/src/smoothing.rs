// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete scalar tolerance contexts for existing smooth operators (blueprint §8.2).
use crate::{
    Opcode, QuantityAdditionKind, QuantityError, QuantityRegistry, QuantityTypeId, ScaleKind,
    UnitId,
};

/// Resolve an actual declared scalar tolerance type from the first operand's complete key.
/// No dimension-compatible kind, basis or datum can replace that exact declaration.
///
/// # Errors
/// Missing type/context, unsupported opcode or dimensional input to `SafeLog`.
pub fn tolerance_type(
    opcode: Opcode,
    operand: QuantityTypeId,
    registry: &QuantityRegistry,
) -> Result<QuantityTypeId, QuantityError> {
    if !matches!(
        opcode,
        Opcode::SmoothMax
            | Opcode::SmoothMin
            | Opcode::SmoothAbs
            | Opcode::SafeSqrt
            | Opcode::SafeLog
    ) {
        return Err(QuantityError::InferencePrecondition {
            rule: "smooth.opcode",
            detail: "opcode has no smoothing tolerance contract".to_owned(),
        });
    }
    let input = registry.quantity_type(operand)?;
    let kind = registry.kind(input.key.kind)?;
    if opcode == Opcode::SafeLog && !kind.dimension.is_dimensionless() {
        return Err(QuantityError::InferencePrecondition {
            rule: "smooth.safe_log",
            detail: "SafeLog input and tolerance must be dimensionless".to_owned(),
        });
    }
    let mut key = input.key.clone();
    key.shape.clear();
    if kind.addition_kind == QuantityAdditionKind::OriginSensitive {
        key.scale_kind = ScaleKind::Difference;
    }
    registry.resolve_key(&key)
}

/// Admit epsilon and, when a unit is supplied, convert it to the first operand's
/// canonical representation coordinate using the actual complete tolerance context.
/// Bare epsilon already denotes that coordinate; it never selects a unit by a name.
///
/// # Errors
/// Nonpositive/nonfinite values, incompatible/missing units or absent complete types.
pub fn resolve_epsilon(
    opcode: Opcode,
    operand: QuantityTypeId,
    eps: f64,
    unit: Option<UnitId>,
    registry: &QuantityRegistry,
) -> Result<f64, QuantityError> {
    let tolerance = tolerance_type(opcode, operand, registry)?;
    let coordinate = registry.quantity_type(operand)?.canonical_unit;
    let conversion = crate::unit::CheckedRepresentationPlan::registered(
        registry, tolerance, unit.unwrap_or(coordinate), coordinate,
    )?;
    let converted = conversion.apply(eps)?;
    positive(opcode, converted)?;
    Ok(converted)
}
fn positive(opcode: Opcode, value: f64) -> Result<(), QuantityError> {
    if value <= 0.0 {
        return Err(QuantityError::StaticDomain {
            opcode,
            restriction: "a finite positive smoothing tolerance",
            value,
        });
    }
    Ok(())
}
