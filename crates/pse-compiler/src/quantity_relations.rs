// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical declarations decoded from exact admitted primitive relation rows.
//! Hashes and fixture registries never supply missing declarations.
mod registry;
mod source;
pub use registry::decode_quantity_registry;
pub use source::{RelationSymbolSource, decode_symbol_source};

use crate::CompilerError;
use pse_ids::SemanticId;
use pse_quantity::{
    BaseDimension, DimensionVector, QuantityError, QuantityRegistryBuilder, Ratio, ReferenceState,
    ReferenceStateId, ReferenceStateKind, Unit, UnitId, UnitSet, UnitSetId,
};
use pse_relations::RecordBatch;
use pse_schema::{
    Registry,
    model::{Cell, RelationKey, RelationSpec},
};
use std::collections::BTreeMap;

/// Decode actual units, admitting each numeric definition and named datum dependency.
/// This does not require a neutral quantity or invent quantity kinds for a unit-only package.
///
/// # Errors
/// Missing/mismatched rows, duplicate identities/symbols or invalid physical definitions.
pub fn decode_units(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
) -> Result<BTreeMap<UnitId, Unit>, CompilerError> {
    let mut builder = QuantityRegistryBuilder::new();
    let mut units = BTreeMap::new();
    for row in read(rows, registry, "reference.units")? {
        let unit = unit(&row)?;
        builder.unit(unit.clone());
        units.insert(unit.id, unit);
    }
    let reference_spec = registry
        .relation("reference.units")
        .ok_or_else(|| invalid("unit contract absent"))?;
    let reference_rows = read(rows, registry, "reference.units")?;
    let declarations = reference_rows
        .iter()
        .map(|row| Ok((row.id("unit_id")?, row)))
        .collect::<Result<BTreeMap<_, _>, CompilerError>>()?;
    let mut normalized_ids = std::collections::BTreeSet::new();
    for row in read_optional(rows, registry, "normalized.units")? {
        let decoded = unit(&row)?;
        if !normalized_ids.insert(decoded.id) {
            return Err(invalid("duplicate normalized unit identity"));
        }
        if let Some(existing) = units.get(&decoded.id) {
            let original = declarations
                .get(&decoded.id.as_id())
                .ok_or_else(|| invalid("unit declaration absent"))?;
            if !same_unit(existing, &decoded)
                || reference_spec.columns.iter().any(|column| {
                    original.get(column.name).ok().map(Cell::literal_spec)
                        != row.get(column.name).ok().map(Cell::literal_spec)
                })
            {
                return Err(invalid(
                    "normalized/reference unit definitions disagree for the same ID",
                ));
            }
        } else {
            builder.unit(decoded.clone());
            units.insert(decoded.id, decoded);
        }
    }
    for state in decode_reference_states(rows, registry)? {
        builder.reference_state(state);
    }
    builder.build()?;
    Ok(units)
}
/// Decode base-unit sets and validate them against the supplied actual unit definitions.
///
/// # Errors
/// Missing or duplicate sets, invalid base dimensions, affine/datum base units, or bad rows.
pub fn decode_unit_sets(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    units: &BTreeMap<UnitId, Unit>,
) -> Result<BTreeMap<UnitSetId, UnitSet>, CompilerError> {
    let declared = decode_units(rows, registry)?;
    if declared.len() != units.len()
        || declared
            .iter()
            .any(|(id, left)| units.get(id).is_none_or(|right| !same_unit(left, right)))
    {
        return Err(invalid(
            "unit-set resolver units differ from the actual bound unit declarations",
        ));
    }
    let mut sets = BTreeMap::new();
    for row in read(rows, registry, "reference.unit_sets")? {
        let mut base = [None; BaseDimension::COUNT];
        for (dimension, name) in [
            (BaseDimension::Length, "length_unit_id"),
            (BaseDimension::Mass, "mass_unit_id"),
            (BaseDimension::Time, "time_unit_id"),
            (BaseDimension::Temperature, "temperature_unit_id"),
            (BaseDimension::Amount, "amount_unit_id"),
            (BaseDimension::Current, "current_unit_id"),
            (
                BaseDimension::LuminousIntensity,
                "luminous_intensity_unit_id",
            ),
            (BaseDimension::Currency, "currency_unit_id"),
        ] {
            base[usize::from(dimension.ordinal())] = row.optional_id(name)?.map(UnitId::from_id);
        }
        let set = UnitSet {
            id: UnitSetId::from_id(row.id("unit_set_id")?),
            base,
        };
        set.validate_units(units)?;
        if sets.insert(set.id, set).is_some() {
            return Err(invalid("duplicate unit-set identity"));
        }
    }
    Ok(sets)
}
fn unit(row: &Row<'_>) -> Result<Unit, CompilerError> {
    Ok(Unit {
        id: UnitId::from_id(row.id("unit_id")?),
        symbol: row.text("symbol")?.to_owned(),
        dimension: dimension(row.get("dimension")?)?,
        scale_to_canonical: row.number("scale_to_canonical")?,
        offset_to_canonical: row.number("offset_to_canonical")?,
        is_affine: row.boolean("is_affine")?,
        reference_state: row
            .optional_id("reference_state_id")?
            .map(ReferenceStateId::from_id),
    })
}
fn same_unit(left: &Unit, right: &Unit) -> bool {
    left.id == right.id
        && left.symbol == right.symbol
        && left.dimension == right.dimension
        && left.scale_to_canonical.to_bits() == right.scale_to_canonical.to_bits()
        && left.offset_to_canonical.to_bits() == right.offset_to_canonical.to_bits()
        && left.is_affine == right.is_affine
        && left.reference_state == right.reference_state
}

fn decode_reference_states(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
) -> Result<Vec<ReferenceState>, CompilerError> {
    read_optional(rows, registry, "reference.reference_states")?
        .into_iter()
        .map(|row| {
            Ok(ReferenceState {
                id: ReferenceStateId::from_id(row.id("reference_state_id")?),
                kind: ReferenceStateKind::parse(row.text("kind")?)
                    .ok_or_else(|| invalid("unknown reference state kind"))?,
                temperature: row.optional_number("temperature")?,
                pressure: row.optional_number("pressure")?,
                include_enthalpy_of_formation: row.boolean("include_enthalpy_of_formation")?,
                phase: row.optional_id("phase_id")?,
            })
        })
        .collect()
}
fn dimension(value: &Cell) -> Result<DimensionVector, CompilerError> {
    let Cell::List(values) = value else {
        return Err(invalid("dimension is not a fixed list"));
    };
    if values.len() != BaseDimension::COUNT {
        return Err(invalid("dimension has wrong width"));
    }
    let mut exponents = [Ratio::ZERO; BaseDimension::COUNT];
    for (out, cell) in exponents.iter_mut().zip(values) {
        let Cell::Struct(pair) = cell else {
            return Err(invalid("dimension exponent is not a ratio"));
        };
        let [Cell::I64(num), Cell::I64(den)] = pair.as_slice() else {
            return Err(invalid("dimension ratio fields differ"));
        };
        let num = i16::try_from(*num).map_err(|_| invalid("dimension numerator out of range"))?;
        let den = i16::try_from(*den).map_err(|_| invalid("dimension denominator out of range"))?;
        *out = Ratio::from_parts(num, den).map_err(QuantityError::from)?;
    }
    Ok(DimensionVector::new(exponents))
}
struct Row<'a> {
    spec: &'a RelationSpec,
    values: Vec<Cell>,
}
impl Row<'_> {
    fn get(&self, name: &str) -> Result<&Cell, CompilerError> {
        self.spec
            .columns
            .iter()
            .position(|column| column.name == name)
            .and_then(|index| self.values.get(index))
            .ok_or_else(|| invalid(format!("{} has no column {name}", self.spec.key)))
    }
    fn id(&self, name: &str) -> Result<SemanticId, CompilerError> {
        match self.get(name)? {
            Cell::Id(value) => Ok(*value),
            _ => Err(invalid(format!("{name} is not an ID"))),
        }
    }
    fn optional_id(&self, name: &str) -> Result<Option<SemanticId>, CompilerError> {
        match self.get(name)? {
            Cell::Null => Ok(None),
            _ => self.id(name).map(Some),
        }
    }
    fn text(&self, name: &str) -> Result<&str, CompilerError> {
        match self.get(name)? {
            Cell::Text(value) => Ok(value),
            Cell::Enum(value) => Ok(value),
            _ => Err(invalid(format!("{name} is not text/enum"))),
        }
    }
    fn number(&self, name: &str) -> Result<f64, CompilerError> {
        match self.get(name)? {
            Cell::F64(value) => Ok(*value),
            _ => Err(invalid(format!("{name} is not f64"))),
        }
    }
    fn optional_number(&self, name: &str) -> Result<Option<f64>, CompilerError> {
        match self.get(name)? {
            Cell::Null => Ok(None),
            _ => self.number(name).map(Some),
        }
    }
    fn enumeration<T>(
        &self,
        name: &str,
        parse: impl FnOnce(&str) -> Option<T>,
    ) -> Result<T, CompilerError> {
        parse(self.text(name)?).ok_or_else(|| invalid(format!("unknown {name} member")))
    }
    fn optional_enum<T>(
        &self,
        name: &str,
        parse: impl FnOnce(&str) -> Option<T>,
    ) -> Result<Option<T>, CompilerError> {
        match self.get(name)? {
            Cell::Null => Ok(None),
            _ => self.enumeration(name, parse).map(Some),
        }
    }
    fn list(&self, name: &str) -> Result<&[Cell], CompilerError> {
        match self.get(name)? {
            Cell::List(values) => Ok(values),
            _ => Err(invalid(format!("{name} is not a list"))),
        }
    }
    fn ids(&self, name: &str) -> Result<Vec<SemanticId>, CompilerError> {
        self.list(name)?
            .iter()
            .map(|cell| match cell {
                Cell::Id(id) => Ok(*id),
                _ => Err(invalid(format!("{name} member is not an ID"))),
            })
            .collect()
    }
    fn enum_list<T>(
        &self,
        name: &str,
        parse: impl Fn(&str) -> Option<T>,
    ) -> Result<Vec<T>, CompilerError> {
        self.list(name)?
            .iter()
            .map(|cell| match cell {
                Cell::Enum(value) => {
                    parse(value).ok_or_else(|| invalid(format!("unknown {name} member")))
                }
                _ => Err(invalid(format!("{name} member is not an enum"))),
            })
            .collect()
    }
    fn unsigned(&self, name: &str) -> Result<u64, CompilerError> {
        match self.get(name)? {
            Cell::U64(value) => Ok(*value),
            _ => Err(invalid(format!("{name} is not unsigned"))),
        }
    }
    fn optional_u16(&self, name: &str) -> Result<Option<u16>, CompilerError> {
        match self.get(name)? {
            Cell::Null => Ok(None),
            Cell::U64(value) => u16::try_from(*value)
                .map(Some)
                .map_err(|_| invalid(format!("{name} exceeds u16"))),
            _ => Err(invalid(format!("{name} is not an unsigned integer"))),
        }
    }
    fn boolean(&self, name: &str) -> Result<bool, CompilerError> {
        match self.get(name)? {
            Cell::Bool(value) => Ok(*value),
            _ => Err(invalid(format!("{name} is not boolean"))),
        }
    }
}
fn read<'a>(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &'a Registry,
    name: &str,
) -> Result<Vec<Row<'a>>, CompilerError> {
    let spec = registry
        .relation(name)
        .ok_or_else(|| invalid(format!("undeclared quantity relation {name}")))?;
    let batch = rows
        .get(&spec.key)
        .ok_or_else(|| invalid(format!("missing explicit quantity binding {name}")))?;
    Ok(
        pse_relations::cells::cells_from_batch(registry, spec, batch)?
            .into_iter()
            .map(|values| Row { spec, values })
            .collect(),
    )
}
fn read_optional<'a>(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &'a Registry,
    name: &str,
) -> Result<Vec<Row<'a>>, CompilerError> {
    let spec = registry
        .relation(name)
        .ok_or_else(|| invalid(format!("undeclared quantity relation {name}")))?;
    if rows.contains_key(&spec.key) {
        read(rows, registry, name)
    } else {
        Ok(vec![])
    }
}
fn invalid(detail: impl Into<String>) -> CompilerError {
    QuantityError::InferencePrecondition {
        rule: "quantity.relation_admission",
        detail: detail.into(),
    }
    .into()
}
