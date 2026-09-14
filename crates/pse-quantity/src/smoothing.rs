// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete scalar tolerance contexts for existing smooth operators (blueprint §8.2).
use crate::{
    Opcode, QuantityAdditionKind, QuantityError, QuantityRegistry, QuantityTypeId, ScaleKind,
    UnitId, convert_spec_for_type, convert_value,
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
    positive(opcode, eps)?;
    let tolerance = registry.quantity_type(tolerance_type(opcode, operand, registry)?)?;
    let converted = if let Some(unit) = unit {
        let input = registry.quantity_type(operand)?;
        let spec = convert_spec_for_type(
            registry.unit(unit)?,
            registry.unit(input.canonical_unit)?,
            &tolerance.key,
        )?;
        convert_value(&spec, eps)
    } else {
        eps
    };
    positive(opcode, converted)?;
    Ok(converted)
}
fn positive(opcode: Opcode, value: f64) -> Result<(), QuantityError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(QuantityError::StaticDomain {
            opcode,
            restriction: "a finite positive smoothing tolerance",
            value,
        });
    }
    Ok(())
}
