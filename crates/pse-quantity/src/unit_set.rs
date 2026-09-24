// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Package base units and derived units (blueprint §6.2, §8.2).
use crate::registry::QuantityRegistry;
use crate::{BaseDimension, DimensionVector, QuantityError, Unit, UnitId, UnitSetId};
use std::collections::BTreeMap;

/// A package's ordered choice of base units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitSet {
    /// Registry identity.
    pub id: UnitSetId,
    /// Units in `BaseDimension` order; current, light and currency are optional.
    pub base: [Option<UnitId>; BaseDimension::COUNT],
}
/// A derived dimension and its scale to SI; it has no independent registry identity.
#[derive(Clone, Copy, Debug)]
pub struct DerivedUnit {
    /// The requested dimension.
    pub dimension: DimensionVector,
    /// Multiplicative scale to SI; derived units never have an offset.
    pub scale_to_canonical: f64,
}
impl UnitSet {
    /// Create and validate a base-unit set.
    ///
    /// # Errors
    /// Rejects missing required units, unknown units, affine units and wrong dimensions.
    pub fn new(
        id: UnitSetId,
        base: [Option<UnitId>; BaseDimension::COUNT],
        registry: &QuantityRegistry,
    ) -> Result<Self, QuantityError> {
        let set = Self { id, base };
        set.validate(registry)?;
        Ok(set)
    }
    /// Check each base selection against the actual admitted unit definitions.
    ///
    /// # Errors
    /// Rejects invalid selections as described by [`Self::new`].
    pub fn validate(&self, registry: &QuantityRegistry) -> Result<(), QuantityError> {
        self.validate_lookup(&|id| registry.unit(id))
    }
    /// Validate a unit-only package without inventing physical quantity declarations.
    ///
    /// # Errors
    /// Same actual unit, base dimension and required-axis checks as [`Self::validate`].
    pub fn validate_units(&self, units: &BTreeMap<UnitId, Unit>) -> Result<(), QuantityError> {
        self.validate_lookup(&|id| unit(units, id))
    }
    fn validate_lookup<'a>(
        &self,
        lookup: &impl Fn(UnitId) -> Result<&'a Unit, QuantityError>,
    ) -> Result<(), QuantityError> {
        for dimension in BaseDimension::ALL {
            let Some(id) = self.base[usize::from(dimension.ordinal())] else {
                if dimension.ordinal() < 5 {
                    return Err(self.error("missing required base unit"));
                }
                continue;
            };
            let unit = lookup(id)?;
            unit.validate()?;
            if unit.is_affine || unit.offset_to_canonical != 0.0 || unit.reference_state.is_some() {
                return Err(self.error("base unit must be non-affine"));
            }
            if unit.dimension != DimensionVector::base(*dimension) {
                return Err(self.error("base unit has the wrong dimension"));
            }
        }
        Ok(())
    }
    /// Compute the product of the selected base scales with the declared rational powers.
    ///
    /// # Errors
    /// Rejects unavailable base units and overflow/underflow in the derived scale.
    pub fn derived_unit(
        &self,
        dimension: DimensionVector,
        registry: &QuantityRegistry,
    ) -> Result<DerivedUnit, QuantityError> {
        self.derived_lookup(dimension, &|id| registry.unit(id))
    }
    /// Compute package-derived units using explicit unit-only declarations.
    ///
    /// # Errors
    /// Rejects invalid/missing base units and nonfinite derived scales.
    pub fn derived_unit_from_units(
        &self,
        dimension: DimensionVector,
        units: &BTreeMap<UnitId, Unit>,
    ) -> Result<DerivedUnit, QuantityError> {
        self.derived_lookup(dimension, &|id| unit(units, id))
    }
    fn derived_lookup<'a>(
        &self,
        dimension: DimensionVector,
        lookup: &impl Fn(UnitId) -> Result<&'a Unit, QuantityError>,
    ) -> Result<DerivedUnit, QuantityError> {
        self.validate_lookup(lookup)?;
        let mut scale = 1.0;
        for base in BaseDimension::ALL {
            let exponent = dimension.exponent(*base);
            if exponent.is_zero() {
                continue;
            }
            let id = self.base[usize::from(base.ordinal())]
                .ok_or_else(|| self.error("derived unit requires an absent optional base unit"))?;
            scale *= lookup(id)?
                .scale_to_canonical
                .powf(f64::from(exponent.num()) / f64::from(exponent.den()));
            if !scale.is_finite() || scale <= 0.0 {
                return Err(self.error("derived unit scale is not finite and positive"));
            }
        }
        Ok(DerivedUnit {
            dimension,
            scale_to_canonical: scale,
        })
    }
    fn error(&self, detail: &str) -> QuantityError {
        QuantityError::Registry {
            rule: "unit_set.base_units",
            subject: self.id.as_id(),
            detail: detail.to_owned(),
        }
    }
}

fn unit(units: &BTreeMap<UnitId, Unit>, id: UnitId) -> Result<&Unit, QuantityError> {
    units.get(&id).ok_or(QuantityError::UnknownId {
        kind: "unit",
        id: id.as_id(),
    })
}
