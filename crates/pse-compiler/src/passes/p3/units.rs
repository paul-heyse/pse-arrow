// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Literal representation changes use actual unit definitions and explicit package choices.

use crate::{
    CompilerError,
    quantity_relations::{decode_unit_sets, decode_units},
};
use pse_authoring::dsl;
use pse_ids::SemanticId;
use pse_mathir::Payload;
use pse_quantity::{
    DimensionVector, ScaleKind, Unit, UnitId, UnitSet, UnitSetId, convert_spec, convert_value,
};
use pse_relations::{
    RecordBatch,
    generated::{authored, reference},
};
use pse_schema::{
    Registry,
    model::{Cell, RelationKey},
};
use std::collections::BTreeMap;

pub(super) struct Units {
    definitions: BTreeMap<UnitId, Unit>,
    sets: BTreeMap<UnitSetId, UnitSet>,
    packages: BTreeMap<SemanticId, UnitSetId>,
    pub derived: BTreeMap<UnitId, Vec<Cell>>,
}
impl Units {
    pub(super) fn new(
        inputs: &BTreeMap<RelationKey, RecordBatch>,
        rows: &pse_authoring::document::Rows,
        registry: &Registry,
    ) -> Result<Self, CompilerError> {
        let units = decode_units(inputs, registry)?;
        let sets = decode_unit_sets(inputs, registry, &units)?;
        let spec = registry
            .relation("authored.package_unit_sets")
            .ok_or_else(|| error("package unit selector undeclared"))?;
        let mut packages = BTreeMap::new();
        for row in rows.get(&spec.id).into_iter().flatten() {
            let binding = authored::package_unit_sets::Row::from_cells(row.clone())?;
            let package = binding.package_id;
            let set = UnitSetId::from_id(binding.unit_set_id);
            if !sets.contains_key(&set) || packages.insert(package, set).is_some() {
                return Err(error("unknown or duplicate package unit set"));
            }
        }
        Ok(Self {
            definitions: units,
            sets,
            packages,
            derived: BTreeMap::new(),
        })
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
                .definitions
                .values()
                .filter(|unit| canonical_match(unit, DimensionVector::DIMENSIONLESS, 1.0))
                .collect::<Vec<_>>();
            let [unit] = candidates.as_slice() else {
                return Err(error(
                    "a fractional unitless literal requires exactly one admitted dimensionless unit",
                ));
            };
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
            .sets
            .get(&set)
            .ok_or_else(|| error("package unit set missing"))?
            .derived_unit_from_units(from.dimension, &self.definitions)?;
        let target = self.target(package, set, derived.dimension, derived.scale_to_canonical)?;
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
            .definitions
            .values()
            .filter(|unit| unit.symbol == spelling)
            .cloned()
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [unit] => Ok(unit.clone()),
            [] => self.compound(package, set, spelling),
            _ => Err(error("unit spelling is ambiguous")),
        }
    }
    fn compound(
        &mut self,
        package: SemanticId,
        set: UnitSetId,
        spelling: &str,
    ) -> Result<Unit, CompilerError> {
        let expression =
            dsl::parse_expr(spelling).map_err(|error| super::invalid(&error.to_string()))?;
        let (dimension, scale) = compound_value(&expression, &self.definitions)?;
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
        if self.definitions.contains_key(&id) {
            return Err(error(
                "compound unit identity collides with a distinct admitted symbol",
            ));
        }
        let mut row = reference::units::Row {
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
        }
        .into_cells();
        row.extend([Cell::Id(package), Cell::Id(set.as_id())]);
        self.derived.insert(id, row);
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
        let candidates = self
            .definitions
            .values()
            .filter(|unit| canonical_match(unit, dimension, scale))
            .cloned()
            .collect::<Vec<_>>();
        match candidates.as_slice() {
            [unit] => return Ok(unit.clone()),
            [] => {}
            _ => {
                return Err(error(
                    "package representation has multiple matching unit declarations",
                ));
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
        if let Some(prior) = self.definitions.get(&id) {
            if !canonical_match(prior, dimension, scale) {
                return Err(error(
                    "derived unit identity conflicts with actual declared values",
                ));
            }
            return Ok(prior.clone());
        }
        let row = reference::units::Row {
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
        };
        let mut cells = row.into_cells();
        cells.extend([Cell::Id(package), Cell::Id(set.as_id())]);
        self.derived.insert(id, cells);
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
    units: &BTreeMap<UnitId, Unit>,
) -> Result<(DimensionVector, f64), CompilerError> {
    let (dimension, scale) = match &expr.kind {
        dsl::ExprKind::Number(value) if value.unit.is_none() => {
            (DimensionVector::DIMENSIONLESS, value.value)
        }
        dsl::ExprKind::Path(path)
            if path.segments.len() == 1 && path.segments[0].indices.is_empty() =>
        {
            let matches = units
                .values()
                .filter(|unit| unit.symbol == path.segments[0].name)
                .collect::<Vec<_>>();
            let [unit] = matches.as_slice() else {
                return Err(error("compound unit factor is missing or ambiguous"));
            };
            if unit.is_affine || unit.reference_state.is_some() {
                return Err(error(
                    "affine or datum-restricted unit cannot be a compound factor",
                ));
            }
            (unit.dimension, unit.scale_to_canonical)
        }
        dsl::ExprKind::Binary { op, lhs, rhs } => {
            let (left, scale) = compound_value(lhs, units)?;
            match op {
                dsl::BinaryOp::Mul => {
                    let (right, rscale) = compound_value(rhs, units)?;
                    (
                        left.mul(&right)
                            .map_err(pse_quantity::QuantityError::from)?,
                        scale * rscale,
                    )
                }
                dsl::BinaryOp::Div => {
                    let (right, rscale) = compound_value(rhs, units)?;
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
