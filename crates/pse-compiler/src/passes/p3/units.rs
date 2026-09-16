// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Literal representation changes use actual unit definitions and explicit package choices.

use crate::{CompilerError, quantity_relations::PhysicalInventory};
use pse_authoring::dsl;
use pse_ids::SemanticId;
use pse_mathir::Payload;
use pse_quantity::{
    DimensionVector, QuantityRegistry, ScaleKind, Unit, UnitId, UnitSetId, convert_spec,
    convert_value,
};
use pse_relations::generated::{authored, normalized};
use pse_schema::Registry;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Units<'a> {
    physical: &'a QuantityRegistry,
    definitions: BTreeMap<UnitId, Unit>,
    packages: BTreeMap<SemanticId, UnitSetId>,
    pub derived: BTreeMap<UnitId, normalized::units::Row>,
    pub(super) dependencies: BTreeMap<UnitId, BTreeSet<UnitId>>,
    pub(super) source_uses: BTreeSet<UnitId>,
    pub(super) source_sets: BTreeSet<UnitSetId>,
}
impl<'a> Units<'a> {
    pub(super) fn new(
        physical: &'a PhysicalInventory,
        rows: &pse_authoring::document::Batches,
        registry: &Registry,
    ) -> Result<Self, CompilerError> {
        let physical = physical.quantities();
        let spec = registry
            .relation("authored.package_unit_sets")
            .ok_or_else(|| error("package unit selector undeclared"))?;
        let mut packages = BTreeMap::new();
        let batch = rows
            .get(&spec.id)
            .ok_or_else(|| error("package unit bindings absent"))?;
        for binding in authored::package_unit_sets::View::from_checked(batch)?.rows()? {
            let package = binding.package_id;
            let set = UnitSetId::from_id(binding.unit_set_id);
            if physical.unit_set(set).is_err() || packages.insert(package, set).is_some() {
                return Err(error("unknown or duplicate package unit set"));
            }
        }
        Ok(Self {
            physical,
            definitions: BTreeMap::new(),
            packages,
            derived: BTreeMap::new(),
            dependencies: BTreeMap::new(),
            source_uses: BTreeSet::new(),
            source_sets: BTreeSet::new(),
        })
    }
    fn units(&self) -> impl Iterator<Item = &Unit> {
        self.physical.units().chain(self.definitions.values())
    }
    pub(super) fn literal(
        &mut self,
        package: SemanticId,
        number: &dsl::Number,
    ) -> Result<Payload, CompilerError> {
        let Some(spelling) = &number.unit else {
            if let Ok(value) = number.value.to_string().parse::<i64>() {
                return Ok(Payload::IntConst { value });
            }
            let candidates = self
                .units()
                .filter(|unit| canonical_match(unit, DimensionVector::DIMENSIONLESS, 1.0))
                .cloned()
                .collect::<Vec<_>>();
            let [unit] = candidates.as_slice() else {
                return Err(error(
                    "a fractional unitless literal requires exactly one admitted dimensionless unit",
                ));
            };
            self.source_uses.insert(unit.id);
            return Ok(Payload::FloatConst {
                value: number.value,
                unit: unit.id,
            });
        };
        let set =
            self.packages.get(&package).copied().ok_or_else(|| {
                error("unit-bearing source requires an explicit package unit set")
            })?;
        let from = self.requested(package, spelling)?;
        if from.is_affine || from.reference_state.is_some() {
            return Ok(Payload::FloatConst {
                value: number.value,
                unit: from.id,
            });
        }
        let derived = self
            .physical
            .unit_set(set)?
            .derived_unit(from.dimension, self.physical)?;
        self.source_uses.extend(
            self.physical
                .unit_set(set)?
                .base
                .iter()
                .zip(from.dimension.exponents())
                .filter_map(|(unit, exponent)| (exponent.num() != 0).then_some(*unit).flatten()),
        );
        let target = self.target(package, set, derived.dimension, derived.scale_to_canonical)?;
        self.source_uses.insert(target.id);
        let conversion = convert_spec(&from, &target, ScaleKind::Difference)?;
        let value = convert_value(&conversion, number.value);
        if !value.is_finite() {
            return Err(error("literal normalization produced a non-finite value"));
        }
        Ok(Payload::FloatConst {
            value,
            unit: target.id,
        })
    }
    pub(super) fn requested(
        &mut self,
        package: SemanticId,
        spelling: &str,
    ) -> Result<Unit, CompilerError> {
        let set =
            self.packages.get(&package).copied().ok_or_else(|| {
                error("unit-bearing source requires an explicit package unit set")
            })?;
        let matches = self
            .units()
            .filter(|unit| unit.symbol == spelling)
            .cloned()
            .collect::<Vec<_>>();
        let unit = match matches.as_slice() {
            [unit] => Ok(unit.clone()),
            [] => self.compound(package, set, spelling),
            _ => Err(error("unit spelling is ambiguous")),
        }?;
        self.source_uses.insert(unit.id);
        self.source_sets.insert(set);
        Ok(unit)
    }
    fn compound(
        &mut self,
        package: SemanticId,
        set: UnitSetId,
        spelling: &str,
    ) -> Result<Unit, CompilerError> {
        let expression =
            dsl::parse_expr(spelling).map_err(|error| super::invalid(&error.to_string()))?;
        let mut dependencies = BTreeSet::new();
        let (dimension, scale) = compound_value(&expression, self, &mut dependencies)?;
        let id = UnitId::from_id(pse_ids::named_id(
            package,
            &format!("pse:p3:unit-expression:v1:{spelling}"),
        ));
        let unit = Unit {
            id,
            symbol: spelling.to_owned(),
            dimension,
            scale_to_canonical: scale,
            offset_to_canonical: 0.0,
            is_affine: false,
            reference_state: None,
        };
        unit.validate()?;
        if self.definitions.contains_key(&id) || self.physical.unit(id).is_ok() {
            return Err(error(
                "compound unit identity collides with a distinct admitted symbol",
            ));
        }
        let row = normalized::units::Row {
            unit_id: id.as_id(),
            symbol: spelling.to_owned(),
            name: format!("Authored unit {spelling}"),
            dimension: std::array::from_fn(|index| {
                let ratio = dimension.exponents()[index];
                pse_relations::generated::extension_values::ExtensionDimensionVectorItem {
                    num: ratio.num(),
                    den: ratio.den(),
                }
            }),
            scale_to_canonical: scale,
            offset_to_canonical: 0.0,
            is_affine: false,
            reference_state_id: None,
            system: "package".to_owned(),
            doc: "Derived from checked component-unit expressions.".to_owned(),
            package_id: package,
            unit_set_id: set.as_id(),
        };
        self.derived.insert(id, row);
        self.dependencies.insert(id, dependencies);
        self.definitions.insert(id, unit.clone());
        Ok(unit)
    }
    fn target(
        &mut self,
        package: SemanticId,
        set: UnitSetId,
        dimension: DimensionVector,
        scale: f64,
    ) -> Result<Unit, CompilerError> {
        // The package's explicit base-unit set determines its representation.
        // Equivalent authored spellings may coexist; neither declaration order
        // nor a global search for matching scales chooses the normalized unit.
        for base in self.physical.unit_set(set)?.base.iter().flatten() {
            let unit = self.physical.unit(*base)?;
            if canonical_match(unit, dimension, scale) {
                return Ok(unit.clone());
            }
        }
        let namespace = pse_ids::named_id(
            package,
            &format!("pse:p3:units:v1:{}", set.as_id().to_hex()),
        );
        let name = dimension
            .canonical_bytes()
            .iter()
            .flat_map(|byte| {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                [HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 15)]]
            })
            .map(char::from)
            .collect::<String>();
        let id = UnitId::from_id(pse_ids::named_id(namespace, &name));
        let unit = Unit {
            id,
            symbol: format!("p3_{}_{}", package.to_hex(), name),
            dimension,
            scale_to_canonical: scale,
            offset_to_canonical: 0.0,
            is_affine: false,
            reference_state: None,
        };
        unit.validate()?;
        if let Some(prior) = self
            .definitions
            .get(&id)
            .or_else(|| self.physical.unit(id).ok())
        {
            if !canonical_match(prior, dimension, scale) {
                return Err(error(
                    "derived unit identity conflicts with actual declared values",
                ));
            }
            return Ok(prior.clone());
        }
        let row = normalized::units::Row {
            unit_id: id.as_id(),
            symbol: unit.symbol.clone(),
            name: format!("Package unit {name}"),
            dimension: dimension.exponents().map(|ratio| {
                pse_relations::generated::extension_values::ExtensionDimensionVectorItem {
                    num: ratio.num(),
                    den: ratio.den(),
                }
            }),
            scale_to_canonical: scale,
            offset_to_canonical: 0.0,
            is_affine: false,
            reference_state_id: None,
            system: "package".to_owned(),
            doc: "Derived from the exact package base-unit definitions.".to_owned(),
            package_id: package,
            unit_set_id: set.as_id(),
        };
        self.derived.insert(id, row);
        let bases = self.physical.unit_set(set)?;
        self.dependencies.insert(
            id,
            bases
                .base
                .iter()
                .zip(dimension.exponents())
                .filter_map(|(unit, exponent)| (exponent.num() != 0).then_some(*unit).flatten())
                .collect(),
        );
        self.definitions.insert(id, unit.clone());
        Ok(unit)
    }
}
fn canonical_match(unit: &Unit, dimension: DimensionVector, scale: f64) -> bool {
    unit.dimension == dimension
        && unit.scale_to_canonical.to_bits() == scale.to_bits()
        && unit.offset_to_canonical.to_bits() == 0.0_f64.to_bits()
        && !unit.is_affine
        && unit.reference_state.is_none()
}
fn error(reason: &str) -> CompilerError {
    pse_authoring::AuthoringError::Contract {
        at: None,
        reason: reason.to_owned(),
    }
    .into()
}

fn compound_value(
    expr: &dsl::Expr,
    units: &Units<'_>,
    dependencies: &mut BTreeSet<UnitId>,
) -> Result<(DimensionVector, f64), CompilerError> {
    let (dimension, scale) = match &expr.kind {
        dsl::ExprKind::Number(value) if value.unit.is_none() => {
            (DimensionVector::DIMENSIONLESS, value.value)
        }
        dsl::ExprKind::Path(path)
            if path.segments.len() == 1 && path.segments[0].indices.is_empty() =>
        {
            let matches = units
                .units()
                .filter(|unit| unit.symbol == path.segments[0].name)
                .collect::<Vec<_>>();
            let [unit] = matches.as_slice() else {
                return Err(error(&format!(
                    "compound unit factor '{}' has {} matching declarations",
                    path.segments[0].name,
                    matches.len()
                )));
            };
            if unit.is_affine || unit.reference_state.is_some() {
                return Err(error(
                    "affine or datum-restricted unit cannot be a compound factor",
                ));
            }
            dependencies.insert(unit.id);
            (unit.dimension, unit.scale_to_canonical)
        }
        dsl::ExprKind::Binary { op, lhs, rhs } => {
            let (left, scale) = compound_value(lhs, units, dependencies)?;
            match op {
                dsl::BinaryOp::Mul => {
                    let (right, rscale) = compound_value(rhs, units, dependencies)?;
                    (
                        left.mul(&right)
                            .map_err(pse_quantity::QuantityError::from)?,
                        scale * rscale,
                    )
                }
                dsl::BinaryOp::Div => {
                    let (right, rscale) = compound_value(rhs, units, dependencies)?;
                    (
                        left.div(&right)
                            .map_err(pse_quantity::QuantityError::from)?,
                        scale / rscale,
                    )
                }
                dsl::BinaryOp::Pow => {
                    let exponent = rational(rhs)?;
                    (
                        left.pow(exponent)
                            .map_err(pse_quantity::QuantityError::from)?,
                        scale.powf(f64::from(exponent.num()) / f64::from(exponent.den())),
                    )
                }
                _ => {
                    return Err(error(
                        "unit expressions support only multiplication, division and rational powers",
                    ));
                }
            }
        }
        _ => return Err(error("unsupported compound unit syntax")),
    };
    if !scale.is_finite() || scale <= 0.0 {
        return Err(error("compound unit scale must remain finite and positive"));
    }
    Ok((dimension, scale))
}
fn rational(expr: &dsl::Expr) -> Result<pse_quantity::Ratio, CompilerError> {
    fn integer(expr: &dsl::Expr) -> Result<i32, CompilerError> {
        match &expr.kind {
            dsl::ExprKind::Number(value) if value.unit.is_none() => value
                .value
                .to_string()
                .parse()
                .map_err(|_| error("unit exponent must be an exact bounded integer ratio")),
            dsl::ExprKind::Neg(value) => integer(value)?
                .checked_neg()
                .ok_or_else(|| error("unit exponent overflow")),
            _ => Err(error("unit exponent must be an exact integer ratio")),
        }
    }
    let (num, den) = if let dsl::ExprKind::Binary {
        op: dsl::BinaryOp::Div,
        lhs,
        rhs,
    } = &expr.kind
    {
        (integer(lhs)?, integer(rhs)?)
    } else {
        (integer(expr)?, 1)
    };
    Ok(pse_quantity::Ratio::new(num, den).map_err(pse_quantity::QuantityError::from)?)
}
