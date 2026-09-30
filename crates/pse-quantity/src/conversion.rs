// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registered quantity conversions (blueprint §6.2, §8.4).
use crate::{CanonicalConversionPlan, CanonicalMagnitude, ConversionId, ConversionKind, QuantityError, QuantityRegistry, QuantityTypeId, ScaleKind};
use pse_ids::SemanticId;
/// One explicit conversion; applying it is separate from declaring it.
#[derive(Clone, Debug)]
pub struct ConversionRule {
    /// Registry identity.
    pub id: ConversionId,
    /// Complete input contract.
    pub from: QuantityTypeId,
    /// Complete output contract.
    pub to: QuantityTypeId,
    /// Scale, affine, or a parameterized kernel.
    pub kind: ConversionKind,
    /// Required only for a kernel conversion.
    pub kernel: Option<SemanticId>,
    /// Named parameter dependencies, never implicit inputs.
    pub required_parameters: Vec<String>,
    /// Resolved multiplicative coefficient; absent for a kernel.
    pub scale: Option<f64>,
    /// Resolved additive coefficient; absent for a kernel.
    pub offset: Option<f64>,
}

// Semantic equality preserves every declared IEEE bit, including signed zero.
impl PartialEq for ConversionRule {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.from == other.from
            && self.to == other.to
            && self.kind == other.kind
            && self.kernel == other.kernel
            && self.required_parameters == other.required_parameters
            && self.scale.map(f64::to_bits) == other.scale.map(f64::to_bits)
            && self.offset.map(f64::to_bits) == other.offset.map(f64::to_bits)
    }
}
impl Eq for ConversionRule {}

/// A datum translation whose origin is the selected reference's typed condition.
/// Input and output are canonical magnitudes; representation conversion happens first.
/// The coefficients are derived, never a second authored pressure/temperature origin.
#[derive(Clone, Debug)]
pub struct DatumConversionPlan {
    source: CanonicalConversionPlan,
    target: CanonicalConversionPlan,
    scale: f64,
    offset: f64,
}
impl DatumConversionPlan {
    /// Resolve a pressure or temperature datum change from the selected declarations.
    /// Material caloric reference changes require scientific anchors and composition,
    /// and cannot use this condition-origin operation.
    /// # Errors
    /// Different kind/basis/scale/shape/subject, an absent origin, or invalid coefficients.
    pub fn admit(registry: &QuantityRegistry, from: QuantityTypeId, to: QuantityTypeId) -> Result<Self, QuantityError> {
        let a = registry.quantity_type(from)?;
        let b = registry.quantity_type(to)?;
        let mut expected = a.key.clone();
        expected.reference_state = b.key.reference_state;
        if expected != b.key {
            return Err(datum_error(from, "a datum change must retain kind, basis, scale, shape and subject"));
        }
        // Resolve origins even for differences: this establishes that these datums
        // describe the actual measured kind, rather than erasing an enthalpy datum.
        let origin = |ty: &crate::QuantityType| -> Result<f64, QuantityError> {
            let Some(reference) = ty.key.reference_state else { return Ok(0.0); };
            let reference = registry.reference_state(reference)?;
            let mut matches = [reference.pressure, reference.temperature].into_iter().flatten()
                .filter_map(|condition| match registry.quantity_type(condition.quantity_type) {
                    Ok(contract) if contract.key.kind == ty.key.kind && contract.key.reference_state.is_none() => Some(Ok(condition)),
                    Ok(_) => None,
                    Err(error) => Some(Err(error)),
                });
            let condition = matches.next().transpose()?.ok_or_else(|| datum_error(from, "the datum has no absolute condition for this quantity kind"))?;
            if matches.next().is_some() { return Err(datum_error(from, "the datum has ambiguous conditions for this kind")); }
            let value = CanonicalConversionPlan::registered(registry, condition.quantity_type, condition.unit)?.apply(condition.value)?.value();
            let unit = registry.unit(registry.quantity_type(condition.quantity_type)?.canonical_unit)?;
            let value = value * unit.scale_to_canonical + unit.offset_to_canonical;
            if !value.is_finite() { return Err(datum_error(from, "the datum condition overflows in absolute representation")); }
            Ok(value)
        };
        let (a_origin, b_origin) = (origin(a)?, origin(b)?);
        let a_unit = registry.unit(a.canonical_unit)?;
        let b_unit = registry.unit(b.canonical_unit)?;
        let scale = a_unit.scale_to_canonical / b_unit.scale_to_canonical;
        let offset = if a.key.scale_kind == ScaleKind::Difference { 0.0 } else {
            (a_unit.offset_to_canonical + a_origin - b_unit.offset_to_canonical - b_origin) / b_unit.scale_to_canonical
        };
        if !scale.is_finite() || scale <= 0.0 || !offset.is_finite() {
            return Err(datum_error(from, "derived datum coefficients are not finite"));
        }
        Ok(Self { source: CanonicalConversionPlan::canonical(registry, from)?, target: CanonicalConversionPlan::canonical(registry, to)?, scale, offset })
    }
    /// Multiplicative coefficient between the canonical representations.
    pub const fn scale(&self) -> f64 { self.scale }
    /// Derived origin shift; zero for differences.
    pub const fn offset(&self) -> f64 { self.offset }
    /// Apply to the checked source quantity, preserving multiply-then-add arithmetic.
    /// # Errors
    /// Wrong source quantity or overflow.
    pub fn apply(&self, value: CanonicalMagnitude) -> Result<CanonicalMagnitude, QuantityError> {
        if value.quantity() != self.source.quantity() {
            return Err(datum_error(value.quantity(), "datum conversion received a different source contract"));
        }
        let converted = value.value() * self.scale + self.offset;
        if !converted.is_finite() {
            return Err(QuantityError::NonfiniteConversion { quantity: self.target.quantity(), from: self.source.canonical_unit(), to: self.target.canonical_unit(), value: value.value(), scale: self.scale, offset: self.offset });
        }
        self.target.apply(converted)
    }
}
fn datum_error(quantity: QuantityTypeId, detail: &str) -> QuantityError {
    QuantityError::Registry { rule: "conversion.datum_origin", subject: quantity.as_id(), detail: detail.into() }
}
