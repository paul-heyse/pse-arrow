// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Units and the unit-conversion edge (blueprint §6.2, §8.2).
//!
//! Includes `Unit`, `convert_spec` and context-aware `convert_spec_for_type`.
//! `convert_value` applies `(v * scale) + offset` at the physical binding boundary.
//! The mathematical body receives canonical coordinates after this admission.

use crate::ids::UnitId;

/// A resolved unit-conversion edge: `to = (from * scale) + offset` (blueprint §7.2, §8.2).
///
/// The offset is nonzero only when a *point* quantity is converted out of an affine unit
/// (°C or °F) — a difference converts by scale alone. Gauge datum changes use a
/// registered physical conversion, separately from representation units (ADR-0058). §7.2
/// makes the consequence explicit: natural-unit ports carry visible `UnitConvert` edges rather than an implicit reinterpretation.
///
/// The two `f64` fields are compared through [`pse_ids::canonical_f64_bits`] wherever two
/// specs must be judged equal, never with `==` (ADR-0030); the derived `PartialEq` here is
/// the ordinary IEEE comparison and is intended for tests and debugging.
///
/// ```
/// use pse_ids::SemanticId;
/// use pse_quantity::{UnitConvertSpec, UnitId};
///
/// // °C to K, as a point quantity: scale 1, offset 273.15.
/// let celsius = UnitId::from_id(SemanticId::from_bytes([1; 16]));
/// let kelvin = UnitId::from_id(SemanticId::from_bytes([2; 16]));
/// let spec = UnitConvertSpec {
///     from: celsius,
///     to: kelvin,
///     scale: 1.0,
///     offset: 273.15,
/// };
/// assert_eq!(spec.from, celsius);
/// ```
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UnitConvertSpec {
    /// The unit the value is in before the conversion.
    pub from: UnitId,
    /// The unit the value is in after the conversion.
    pub to: UnitId,
    /// The multiplicative factor, applied first.
    pub scale: f64,
    /// The additive offset, applied second; zero for a difference conversion.
    pub offset: f64,
}

/// A unit anchored to the canonical SI dimension basis (blueprint §6.2).
#[derive(Clone, Debug)]
pub struct Unit {
    /// Registry identity.
    pub id: UnitId,
    /// Unique package-resolved symbol.
    pub symbol: String,
    /// SI exponents.
    pub dimension: crate::DimensionVector,
    /// Positive finite multiplier to SI.
    pub scale_to_canonical: f64,
    /// Finite offset to SI; zero for a non-affine unit.
    pub offset_to_canonical: f64,
    /// Whether point conversion may apply an offset.
    pub is_affine: bool,
    /// Optional datum restriction of the spelling; representation conversion preserves it.
    pub reference_state: Option<crate::ReferenceStateId>,
}
impl Unit {
    /// Admit the numeric unit definition without guessing its physical kind.
    ///
    /// # Errors
    /// Rejects non-finite/zero/negative scales or offsets on non-affine units.
    pub fn validate(&self) -> Result<(), crate::QuantityError> {
        let detail = if self.symbol.is_empty() {
            Some("unit symbol is empty")
        } else if !self.scale_to_canonical.is_finite() || self.scale_to_canonical <= 0.0 {
            Some("unit scale must be finite and positive")
        } else if !self.offset_to_canonical.is_finite() {
            Some("unit offset must be finite")
        } else if !self.is_affine && self.offset_to_canonical != 0.0 {
            Some("non-affine unit has an offset")
        } else {
            None
        };
        if let Some(detail) = detail {
            return Err(crate::QuantityError::Registry {
                rule: "unit.definition",
                subject: self.id.as_id(),
                detail: detail.to_owned(),
            });
        }
        Ok(())
    }
}
/// Resolve a unit edge in either direction; offsets apply only to points.
///
/// # Errors
/// Rejects malformed definitions, unequal dimensions or non-finite coefficients.
pub fn convert_spec(
    from: &Unit,
    to: &Unit,
    scale_kind: crate::ScaleKind,
) -> Result<UnitConvertSpec, crate::QuantityError> {
    if from.reference_state.is_some() || to.reference_state.is_some() {
        return Err(crate::QuantityError::InferencePrecondition {
            rule: "unit.reference_context",
            detail: "datum-restricted units require a complete quantity context".to_owned(),
        });
    }
    representation_conversion(from, to, scale_kind)
}
/// Resolve representation units within a complete physical quantity context (ADR-0058).
/// A datum change is a separate registered quantity conversion, applied exactly once.
///
/// # Errors
/// Rejects a unit whose reference restriction differs from the actual quantity context,
/// malformed units, incompatible dimensions or non-finite coefficients.
pub fn convert_spec_for_type(
    from: &Unit,
    to: &Unit,
    context: &crate::QuantityTypeKey,
) -> Result<UnitConvertSpec, crate::QuantityError> {
    for unit in [from, to] {
        if unit.reference_state.is_some() && unit.reference_state != context.reference_state {
            return Err(crate::QuantityError::InferencePrecondition {
                rule: "unit.reference_context",
                detail: format!(
                    "unit {} is restricted to a different reference state",
                    unit.id
                ),
            });
        }
    }
    representation_conversion(from, to, context.scale_kind)
}
fn representation_conversion(
    from: &Unit,
    to: &Unit,
    scale_kind: crate::ScaleKind,
) -> Result<UnitConvertSpec, crate::QuantityError> {
    from.validate()?;
    to.validate()?;
    if from.dimension != to.dimension {
        return Err(crate::QuantityError::Registry {
            rule: "conversion.dimension",
            subject: from.id.as_id(),
            detail: format!("target unit {} has a different dimension", to.id),
        });
    }
    let scale = from.scale_to_canonical / to.scale_to_canonical;
    let offset = match scale_kind {
        crate::ScaleKind::Point => {
            (from.offset_to_canonical - to.offset_to_canonical) / to.scale_to_canonical
        }
        crate::ScaleKind::Difference => 0.0,
    };
    if !scale.is_finite() || scale <= 0.0 || !offset.is_finite() {
        return Err(crate::QuantityError::Registry {
            rule: "conversion.coefficients",
            subject: from.id.as_id(),
            detail: "unit conversion is not representable with finite coefficients".to_owned(),
        });
    }
    Ok(UnitConvertSpec {
        from: from.id,
        to: to.id,
        scale,
        offset,
    })
}
/// Apply the ordered two-rounding conversion of blueprint §7.3.
/// No `mul_add` or target-specific contraction is permitted.
pub fn convert_value(spec: &UnitConvertSpec, value: f64) -> f64 {
    let scaled = value * spec.scale;
    scaled + spec.offset
}
impl UnitConvertSpec {
    /// Compare the actual declared conversion, preserving signed zero (ADR-0030).
    pub fn same_contract(&self, other: &Self) -> bool {
        self.from == other.from
            && self.to == other.to
            && pse_ids::canonical_f64_bits(self.scale) == pse_ids::canonical_f64_bits(other.scale)
            && pse_ids::canonical_f64_bits(self.offset) == pse_ids::canonical_f64_bits(other.offset)
    }
}

// Semantic equality preserves every declared IEEE bit, including signed zero.
impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.symbol == other.symbol
            && self.dimension == other.dimension
            && self.is_affine == other.is_affine
            && self.reference_state == other.reference_state
            && self.scale_to_canonical.to_bits() == other.scale_to_canonical.to_bits()
            && self.offset_to_canonical.to_bits() == other.offset_to_canonical.to_bits()
    }
}
impl Eq for Unit {}
