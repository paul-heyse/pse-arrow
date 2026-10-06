// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Units and the unit-conversion edge (blueprint §6.2, §8.2).
//!
//! Includes `Unit`, `convert_spec` and context-aware `convert_spec_for_type`.
//! [`CanonicalConversionPlan`] admits finite values at the physical binding boundary.
//! The mathematical body receives canonical coordinates after this admission.
//!
//! A unit is atomic (symbol, dimension, scale and offset authored) or defined: its
//! composition is authored and its dimension and scale are derived at admission
//! (ADR-0124). A unit literal composes atomic units with rational exponents; its
//! identity is [`unit_product_id`] over its canonical factors, so no spelling of a
//! composite unit needs registering.

use crate::Ratio;
use crate::ids::{QuantityTypeId, UnitId};
use crate::{DimensionVector as Dimension, QuantityError as Error};
use std::hash::{Hash, Hasher};

/// One canonical factor of a unit product: an atomic unit and its nonzero rational
/// exponent (ADR-0124).
#[derive(
    serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct UnitFactor {
    /// The atomic unit, or, in an authored composition, any declared unit.
    pub unit: UnitId,
    /// The nonzero rational exponent.
    pub exponent: Ratio,
}

/// A defined unit as authored: its composition only. Its dimension and scale are
/// derived when the registry admits it, so they have one authority (ADR-0124).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinedUnit {
    /// Registry identity; it must equal [`unit_product_id`] over the canonical factors.
    pub id: UnitId,
    /// The package's name for the row; a defined unit is never looked up by it.
    pub symbol: String,
    /// Authored factors, each naming an atomic or another defined unit.
    pub composition: Vec<UnitFactor>,
}

/// The identity of a canonical unit product (ADR-0124): the sole atomic factor itself
/// when the product is one unit with exponent one, otherwise a
/// `pse.quantity.unit-product.v1` derivation over the canonical factors. The factors
/// must be canonical (identity order, distinct units, nonzero exponents); spelling
/// never enters.
pub fn unit_product_id(factors: &[UnitFactor]) -> UnitId {
    if let [only] = factors
        && only.exponent == Ratio::ONE
    {
        return only.unit;
    }
    let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::QuantityUnitProductV1);
    h.u64(factors.len() as u64);
    for factor in factors {
        h.id(&factor.unit.as_id())
            .part(&factor.exponent.num().to_le_bytes())
            .part(&factor.exponent.den().to_le_bytes());
    }
    UnitId::from_id(h.finish_id())
}

/// Merge factors of the same unit, drop zero exponents and order by identity.
///
/// # Errors
/// An exponent sum that does not fit the canonical rational pair.
pub fn canonical_factors(
    factors: impl IntoIterator<Item = UnitFactor>,
) -> Result<Vec<UnitFactor>, Error> {
    let mut merged = std::collections::BTreeMap::<UnitId, Ratio>::new();
    for factor in factors {
        let slot = merged.entry(factor.unit).or_insert(Ratio::ZERO);
        *slot = slot.checked_add(factor.exponent)?;
    }
    Ok(merged
        .into_iter()
        .filter(|(_, exponent)| !exponent.is_zero())
        .map(|(unit, exponent)| UnitFactor { unit, exponent })
        .collect())
}

/// The dimension and scale of a product of admitted atomic units.
///
/// # Errors
/// Dimension overflow, or a scale that is not finite and positive.
pub(crate) fn derived_measure<'a>(
    factors: &[UnitFactor],
    atomic: impl Fn(UnitId) -> Result<&'a Unit, Error>,
) -> Result<(Dimension, f64), Error> {
    let mut dimension = Dimension::DIMENSIONLESS;
    let mut scale = 1.0_f64;
    for factor in factors {
        let unit = atomic(factor.unit)?;
        dimension = dimension.mul(&unit.dimension.pow(factor.exponent)?)?;
        let exponent = factor.exponent;
        scale *= if exponent.is_integer() {
            unit.scale_to_canonical.powi(i32::from(exponent.num()))
        } else {
            unit.scale_to_canonical
                .powf(f64::from(exponent.num()) / f64::from(exponent.den()))
        };
    }
    if !scale.is_finite() || scale <= 0.0 {
        return Err(Error::Registry {
            rule: "unit.derived_scale",
            subject: unit_product_id(factors).as_id(),
            detail: "unit product scale is not finite and positive".to_owned(),
        });
    }
    Ok((dimension, scale))
}

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
    /// `None` for an atomic unit; the canonical atomic factors of a defined or composed
    /// unit, whose dimension and scale were derived from them.
    pub definition: Option<Vec<UnitFactor>>,
}
impl Unit {
    /// Admit the numeric unit definition without guessing its physical kind.
    ///
    /// # Errors
    /// Rejects non-finite/zero/negative scales or offsets on non-affine units.
    pub fn validate(&self) -> Result<(), crate::QuantityError> {
        let detail = if self.symbol.is_empty() {
            Some("unit symbol is empty")
        } else if self.definition.is_some()
            && (self.is_affine || self.offset_to_canonical != 0.0 || self.reference_state.is_some())
        {
            Some("a unit product has no offset or datum restriction")
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
/// An admitted quantity representation and its checked canonical numeric operation.
///
/// Private fields prevent detached coefficients from establishing a physical contract.
/// Construction resolves the complete target contract from the selected registry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CanonicalConversionPlan {
    operation: CheckedRepresentationPlan,
    already_canonical: bool,
}

/// Registry-admitted representation coordinates for internal operators whose numerical
/// coordinate need not be the tolerance quantity's canonical storage unit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CheckedRepresentationPlan {
    quantity: QuantityTypeId,
    context: crate::QuantityTypeKey,
    source: UnitId,
    target: UnitId,
    scale_bits: u64,
    offset_bits: u64,
}

impl CanonicalConversionPlan {
    /// Admit a declared source representation from the selected registry.
    ///
    /// # Errors
    /// Missing declarations or incompatible dimension or reference restriction.
    pub fn registered(
        registry: &crate::QuantityRegistry,
        quantity: QuantityTypeId,
        source: UnitId,
    ) -> Result<Self, Error> {
        Self::admit(registry, quantity, registry.unit(source)?, false)
    }

    /// Admit a composed literal through the registry's unit algebra.
    ///
    /// # Errors
    /// Invalid factors or incompatible quantity representation.
    pub fn composed(
        registry: &crate::QuantityRegistry,
        quantity: QuantityTypeId,
        source: &crate::UnitProduct,
    ) -> Result<Self, Error> {
        Self::admit(registry, quantity, &registry.compose(source)?, false)
    }

    /// Admit a previously resolved unit, verifying its complete definition against the
    /// registry. Transient products are re-derived from their registered atomic factors.
    ///
    /// # Errors
    /// An altered or unresolvable definition, or incompatible quantity representation.
    pub fn resolved(
        registry: &crate::QuantityRegistry,
        quantity: QuantityTypeId,
        source: &Unit,
    ) -> Result<Self, Error> {
        let admitted = if let Some(factors) = &source.definition {
            registry.compose(&crate::UnitProduct::from_factors(
                factors
                    .iter()
                    .map(|factor| Ok((registry.unit(factor.unit)?.symbol.clone(), factor.exponent)))
                    .collect::<Result<Vec<_>, Error>>()?,
            )?)?
        } else {
            registry.unit(source.id)?.clone()
        };
        if &admitted != source {
            return Err(Error::Registry {
                rule: "conversion.source_definition",
                subject: source.id.as_id(),
                detail: "resolved source unit differs from its selected registry definition".into(),
            });
        }
        Self::admit(registry, quantity, &admitted, false)
    }

    /// Admit a magnitude already in the declared canonical representation. Its bits,
    /// including signed zero, are retained after the same finite-value validation.
    ///
    /// # Errors
    /// A quantity absent from the selected registry.
    pub fn canonical(
        registry: &crate::QuantityRegistry,
        quantity: QuantityTypeId,
    ) -> Result<Self, Error> {
        let target = registry.quantity_type(quantity)?;
        Self::admit(
            registry,
            quantity,
            registry.unit(target.canonical_unit)?,
            true,
        )
    }

    fn admit(
        registry: &crate::QuantityRegistry,
        quantity: QuantityTypeId,
        source: &Unit,
        already_canonical: bool,
    ) -> Result<Self, Error> {
        let target = registry.quantity_type(quantity)?;
        Ok(Self {
            operation: CheckedRepresentationPlan::admit(
                registry,
                quantity,
                source,
                target.canonical_unit,
            )?,
            already_canonical,
        })
    }

    /// Apply the admitted operation with multiplication before addition, never `mul_add`.
    /// Already-canonical admission preserves the input representation exactly.
    ///
    /// # Errors
    /// A nonfinite source or a nonfinite converted result, with the physical operands.
    pub fn apply(&self, value: f64) -> Result<CanonicalMagnitude, Error> {
        let canonical = self.operation.apply_mode(value, self.already_canonical)?;
        Ok(CanonicalMagnitude {
            bits: canonical.to_bits(),
            quantity: self.quantity(),
        })
    }

    /// Quantity identity resolved from the complete admitted contract.
    pub const fn quantity(&self) -> QuantityTypeId {
        self.operation.quantity
    }
    /// Complete semantic contract retained by this plan.
    pub fn context(&self) -> &crate::QuantityTypeKey {
        &self.operation.context
    }
    /// Declared source representation unit.
    pub const fn source_unit(&self) -> UnitId {
        self.operation.source
    }
    /// Canonical representation unit of the target quantity.
    pub const fn canonical_unit(&self) -> UnitId {
        self.operation.target
    }
    /// Difference/uncertainty scaling factor, excluding any point offset.
    pub fn scale(&self) -> f64 {
        self.operation.scale()
    }
    /// Point offset; zero for differences.
    pub fn offset(&self) -> f64 {
        self.operation.offset()
    }
    /// Owned heap extent of the retained complete contract.
    pub fn heap_bytes(&self) -> usize {
        self.operation
            .context
            .shape
            .capacity()
            .saturating_mul(size_of::<crate::EntityKindId>())
    }
}

impl CheckedRepresentationPlan {
    pub(crate) fn registered(
        registry: &crate::QuantityRegistry,
        quantity: QuantityTypeId,
        source: UnitId,
        target: UnitId,
    ) -> Result<Self, Error> {
        Self::admit(registry, quantity, registry.unit(source)?, target)
    }

    fn admit(
        registry: &crate::QuantityRegistry,
        quantity: QuantityTypeId,
        source: &Unit,
        target: UnitId,
    ) -> Result<Self, Error> {
        let context = registry.quantity_type(quantity)?;
        let canonical = registry.unit(context.canonical_unit)?;
        let target = registry.unit(target)?;
        // Both representations must belong to the actual quantity; equal dimensions
        // between two arbitrary units alone cannot admit this operation.
        if target.dimension != canonical.dimension {
            return Err(Error::Registry {
                rule: "conversion.dimension",
                subject: target.id.as_id(),
                detail: format!("target unit has a different dimension from quantity {quantity}"),
            });
        }
        let spec = convert_spec_for_type(source, target, &context.key)?;
        Ok(Self {
            quantity,
            context: context.key.clone(),
            source: spec.from,
            target: spec.to,
            scale_bits: spec.scale.to_bits(),
            offset_bits: spec.offset.to_bits(),
        })
    }

    pub(crate) fn apply(&self, value: f64) -> Result<f64, Error> {
        self.apply_mode(value, false)
    }

    fn apply_mode(&self, value: f64, already_in_target: bool) -> Result<f64, Error> {
        if !value.is_finite() {
            return Err(Error::NonfiniteMagnitude {
                quantity: self.quantity,
                unit: self.source,
                value,
            });
        }
        let converted = if already_in_target {
            value
        } else {
            let scaled = value * self.scale();
            scaled + self.offset()
        };
        if !converted.is_finite() {
            return Err(Error::NonfiniteConversion {
                quantity: self.quantity,
                from: self.source,
                to: self.target,
                value,
                scale: self.scale(),
                offset: self.offset(),
            });
        }
        Ok(converted)
    }

    fn scale(&self) -> f64 {
        f64::from_bits(self.scale_bits)
    }
    fn offset(&self) -> f64 {
        f64::from_bits(self.offset_bits)
    }
}

impl Hash for CheckedRepresentationPlan {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.quantity.hash(state);
        self.context.kind.hash(state);
        self.context.basis.hash(state);
        self.context.reference_state.hash(state);
        self.context.scale_kind.hash(state);
        self.context.shape.hash(state);
        self.context.subject_kind.hash(state);
        self.source.hash(state);
        self.target.hash(state);
        self.scale_bits.hash(state);
        self.offset_bits.hash(state);
    }
}

/// A finite canonical magnitude associated with its admitted quantity identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanonicalMagnitude {
    bits: u64,
    quantity: QuantityTypeId,
}
impl CanonicalMagnitude {
    /// Exact canonical magnitude bits.
    pub const fn bits(self) -> u64 {
        self.bits
    }
    /// Finite canonical numerical value.
    pub fn value(self) -> f64 {
        f64::from_bits(self.bits)
    }
    /// Admitted physical quantity identity.
    pub const fn quantity(self) -> QuantityTypeId {
        self.quantity
    }
}
/// Untrusted exact canonical magnitude data, separate from an admitted magnitude.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MagnitudeRecord {
    bits: u64,
    quantity: QuantityTypeId,
}
impl MagnitudeRecord {
    /// Capture an already admitted canonical value without unit conversion.
    pub fn capture(v: CanonicalMagnitude) -> Self {
        Self {
            bits: v.bits,
            quantity: v.quantity,
        }
    }
    /// Restore only during strict replay of a qualified selected numerical product.
    pub fn restore(self) -> Result<CanonicalMagnitude, crate::QuantityError> {
        crate::resolved::receipts::require_record(&self)?;
        if !f64::from_bits(self.bits).is_finite() {
            return Err(crate::QuantityError::InferencePrecondition {
                rule: "physical.magnitude_receipt",
                detail: "nonfinite canonical magnitude receipt".into(),
            });
        }
        Ok(CanonicalMagnitude {
            bits: self.bits,
            quantity: self.quantity,
        })
    }
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
            && self.definition == other.definition
            && self.scale_to_canonical.to_bits() == other.scale_to_canonical.to_bits()
            && self.offset_to_canonical.to_bits() == other.offset_to_canonical.to_bits()
    }
}
impl Eq for Unit {}

#[cfg(test)]
mod canonical_tests {
    use super::*;
    use crate::{
        BaseDimension, QuantityAdditionKind, QuantityKind, QuantityRegistry,
        QuantityRegistryBuilder, QuantityType, QuantityTypeKey, ScaleKind,
    };
    use pse_ids::SemanticId;

    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn unit(n: u8, symbol: &str, scale: f64, offset: f64) -> Unit {
        Unit {
            id: id(n).into(),
            symbol: symbol.into(),
            dimension: Dimension::base(BaseDimension::Temperature),
            scale_to_canonical: scale,
            offset_to_canonical: offset,
            is_affine: offset != 0.0,
            reference_state: None,
            definition: None,
        }
    }
    fn registry() -> QuantityRegistry {
        let mut b = QuantityRegistryBuilder::new();
        b.unit(unit(1, "K", 1.0, 0.0))
            .unit(unit(2, "degC", 1.0, 273.15))
            .unit(unit(3, "twiceK", 2.0, 0.0))
            .unit(unit(4, "hugeOrigin", 1.0, f64::MAX))
            .unit(unit(5, "twoRoundings", 1.0 + f64::EPSILON, -1.0))
            .unit(unit(6, "halfK", 0.5, 0.0))
            .kind(QuantityKind {
                id: id(20).into(),
                dimension: Dimension::base(BaseDimension::Temperature),
                extensive: false,
                addition_kind: QuantityAdditionKind::OriginSensitive,
                category: None,
                definition: None,
            });
        for (n, scale_kind) in [(30, ScaleKind::Point), (31, ScaleKind::Difference)] {
            b.quantity_type(QuantityType {
                id: id(n).into(),
                name: None,
                key: QuantityTypeKey {
                    kind: id(20).into(),
                    basis: None,
                    reference_state: None,
                    scale_kind,
                    shape: vec![],
                    subject_kind: None,
                },
                canonical_unit: id(1).into(),
                nominal_magnitude: None,
            });
        }
        b.build().unwrap()
    }

    #[test]
    fn canonical_admission_distinguishes_source_nonfinite_and_both_overflows() {
        let registry = registry();
        let scaled =
            CanonicalConversionPlan::registered(&registry, id(30).into(), id(3).into()).unwrap();
        for input in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                matches!(scaled.apply(input), Err(Error::NonfiniteMagnitude { quantity, unit, .. })
                if quantity == id(30).into() && unit == id(3).into())
            );
        }
        assert!(
            matches!(scaled.apply(f64::MAX), Err(Error::NonfiniteConversion { value, scale, offset, .. })
            if value == f64::MAX && scale == 2.0 && offset == 0.0)
        );
        assert_eq!(
            scaled.apply(f64::MAX / 2.0).unwrap().bits(),
            f64::MAX.to_bits()
        );
        let shifted =
            CanonicalConversionPlan::registered(&registry, id(30).into(), id(4).into()).unwrap();
        assert!(
            matches!(shifted.apply(f64::MAX), Err(Error::NonfiniteConversion { scale, offset, .. })
            if scale == 1.0 && offset == f64::MAX)
        );
        assert_eq!(shifted.apply(0.0).unwrap().bits(), f64::MAX.to_bits());
    }

    #[test]
    fn canonical_admission_preserves_two_roundings_signed_zero_and_subnormals() {
        let registry = registry();
        let rounding =
            CanonicalConversionPlan::registered(&registry, id(30).into(), id(5).into()).unwrap();
        let input = 1.0 - f64::EPSILON;
        let converted = rounding.apply(input).unwrap().value();
        assert_eq!(
            converted.to_bits(),
            ((input * (1.0 + f64::EPSILON)) - 1.0).to_bits()
        );
        assert_ne!(
            converted.to_bits(),
            input.mul_add(rounding.scale(), rounding.offset()).to_bits()
        );
        let canonical = CanonicalConversionPlan::canonical(&registry, id(30).into()).unwrap();
        let represented =
            CanonicalConversionPlan::registered(&registry, id(30).into(), id(1).into()).unwrap();
        assert_eq!(canonical.apply(-0.0).unwrap().bits(), (-0.0_f64).to_bits());
        assert_eq!(represented.apply(-0.0).unwrap().bits(), 0.0_f64.to_bits());
        assert_ne!(canonical, represented);
        assert_eq!(canonical.apply(f64::from_bits(1)).unwrap().bits(), 1);
        assert!(canonical.apply(f64::INFINITY).is_err());
        let half =
            CanonicalConversionPlan::registered(&registry, id(30).into(), id(6).into()).unwrap();
        assert_eq!(half.apply(f64::from_bits(2)).unwrap().bits(), 1);
        assert_eq!(half.apply(f64::from_bits(1)).unwrap().bits(), 0);
    }

    #[test]
    fn canonical_admission_applies_affine_origins_only_to_points_and_retains_scale() {
        let registry = registry();
        let point =
            CanonicalConversionPlan::registered(&registry, id(30).into(), id(2).into()).unwrap();
        let interval =
            CanonicalConversionPlan::registered(&registry, id(31).into(), id(2).into()).unwrap();
        assert_eq!(point.apply(80.0).unwrap().value(), 353.15);
        assert_eq!(interval.apply(80.0).unwrap().value(), 80.0);
        assert_eq!(interval.apply(80.0).unwrap().quantity(), id(31).into());
        let scaled =
            CanonicalConversionPlan::registered(&registry, id(31).into(), id(3).into()).unwrap();
        assert_eq!(scaled.scale(), 2.0);
        assert_eq!(scaled.canonical_unit(), id(1).into());
        assert_eq!(scaled.context().scale_kind, ScaleKind::Difference);
    }

    #[test]
    fn canonical_admission_rederives_resolved_sources_and_checks_datum_restrictions() {
        let registry = registry();
        let mut forged = registry.unit(id(2).into()).unwrap().clone();
        forged.scale_to_canonical = 100.0;
        assert!(matches!(
            CanonicalConversionPlan::resolved(&registry, id(30).into(), &forged),
            Err(Error::Registry {
                rule: "conversion.source_definition",
                ..
            })
        ));
        let mut b = registry.to_builder();
        b.reference_state(crate::ReferenceState {
            id: id(40).into(),
            name: "different".into(),
            kind: crate::ReferenceStateKind::Custom,
            temperature: None,
            pressure: None,
            include_enthalpy_of_formation: false,
            subject: None,
        });
        let mut restricted = unit(7, "datumK", 1.0, 0.0);
        restricted.reference_state = Some(id(40).into());
        b.unit(restricted);
        let registry = b.build().unwrap();
        assert!(matches!(
            CanonicalConversionPlan::registered(&registry, id(30).into(), id(7).into()),
            Err(Error::InferencePrecondition {
                rule: "unit.reference_context",
                ..
            })
        ));
    }

    #[test]
    fn canonical_admission_accepts_transient_unit_products_without_global_registration() {
        let registry = registry();
        let product = crate::UnitProduct::from_factors([
            ("twiceK".into(), Ratio::new(2, 1).unwrap()),
            ("K".into(), Ratio::new(-1, 1).unwrap()),
        ])
        .unwrap();
        let source = registry.compose(&product).unwrap();
        assert!(registry.unit(source.id).is_err());
        let composed =
            CanonicalConversionPlan::composed(&registry, id(30).into(), &product).unwrap();
        let resolved =
            CanonicalConversionPlan::resolved(&registry, id(30).into(), &source).unwrap();
        assert_eq!(composed, resolved);
        assert_eq!(composed.apply(3.0).unwrap().value(), 12.0);
    }

    #[test]
    fn canonical_reference_conditions_and_smoothing_use_checked_admission() {
        let registry = registry();
        let condition = crate::ReferenceCondition {
            value: f64::MAX,
            quantity_type: id(30).into(),
            unit: id(3).into(),
        };
        assert!(matches!(
            registry.reference_condition(&condition),
            Err(Error::NonfiniteConversion { .. })
        ));
        assert_eq!(
            registry
                .reference_condition(&crate::ReferenceCondition {
                    value: 25.0,
                    unit: id(2).into(),
                    ..condition
                })
                .unwrap(),
            298.15
        );
        assert!(matches!(
            crate::smoothing::resolve_epsilon(
                crate::Opcode::SmoothMax,
                id(30).into(),
                f64::MAX,
                Some(id(3).into()),
                &registry,
            ),
            Err(Error::NonfiniteConversion { .. })
        ));
        assert_eq!(
            crate::smoothing::resolve_epsilon(
                crate::Opcode::SmoothMax,
                id(30).into(),
                2.0,
                Some(id(2).into()),
                &registry,
            )
            .unwrap(),
            2.0
        );
        assert!(matches!(
            crate::smoothing::resolve_epsilon(
                crate::Opcode::SmoothMax,
                id(30).into(),
                f64::from_bits(1),
                Some(id(6).into()),
                &registry,
            ),
            Err(Error::StaticDomain { .. })
        ));
        assert!(matches!(
            crate::smoothing::resolve_epsilon(
                crate::Opcode::SmoothMax,
                id(30).into(),
                f64::NAN,
                None,
                &registry,
            ),
            Err(Error::NonfiniteMagnitude { .. })
        ));
    }

    #[test]
    fn canonical_smoothing_retains_operand_coordinates_when_tolerance_canonical_unit_differs() {
        let registry = registry();
        let mut b = QuantityRegistryBuilder::new();
        for unit in registry.units() {
            b.unit(unit.clone());
        }
        for kind in registry.kinds() {
            b.kind(kind.clone());
        }
        for quantity in registry.quantity_types() {
            let mut quantity = quantity.clone();
            if quantity.key.scale_kind == ScaleKind::Difference {
                quantity.canonical_unit = id(3).into();
            }
            b.quantity_type(quantity);
        }
        let registry = b.build().unwrap();
        let tolerance =
            CanonicalConversionPlan::registered(&registry, id(31).into(), id(2).into()).unwrap();
        assert_eq!(tolerance.apply(2.0).unwrap().value(), 1.0);
        // The smoothing algorithm consumes first-operand K coordinates, even though
        // the admitted tolerance type stores canonical magnitudes in twiceK.
        assert_eq!(
            crate::smoothing::resolve_epsilon(
                crate::Opcode::SmoothMax,
                id(30).into(),
                2.0,
                Some(id(2).into()),
                &registry,
            )
            .unwrap(),
            2.0
        );
        assert_eq!(
            crate::smoothing::resolve_epsilon(
                crate::Opcode::SmoothMax,
                id(30).into(),
                2.0,
                None,
                &registry,
            )
            .unwrap(),
            2.0
        );
    }
}
