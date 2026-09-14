// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual kernel port contracts; binding callback validation occurs on their use by P10.
use super::{KernelContract, invalid, read};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_mathir::infer::KernelPort;
use pse_quantity::{DomainKind, QuantityRegistry, QuantityTypeId, UnitId};
use pse_relations::RecordBatch;
use pse_schema::{
    Registry,
    model::{Cell, LogicalType, RelationKey},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn decode(
    rows: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
    quantities: &QuantityRegistry,
) -> Result<BTreeMap<SemanticId, KernelContract>, CompilerError> {
    let mut result = BTreeMap::new();
    for row in read(rows, registry, "reference.kernel_specs")? {
        let ports = |name: &str, shape: &str| -> Result<Vec<KernelPort>, CompilerError> {
            let column = row
                .spec
                .columns
                .iter()
                .find(|column| column.name == name)
                .ok_or_else(|| invalid("kernel port column absent"))?;
            let LogicalType::List(item) = &column.logical_type else {
                return Err(invalid("kernel port layout is not list"));
            };
            let LogicalType::Struct(fields) = item.as_ref() else {
                return Err(invalid("kernel port layout is not struct"));
            };
            let mut names = BTreeSet::new();
            row.list(name)?
                .iter()
                .map(|cell| {
                    let Cell::Struct(values) = cell else {
                        return Err(invalid("kernel port value is not struct"));
                    };
                    let get = |name: &str| {
                        fields
                            .iter()
                            .position(|(field, _, _)| *field == name)
                            .and_then(|index| values.get(index))
                            .ok_or_else(|| invalid("kernel port field absent"))
                    };
                    let Cell::Text(name) = get("name")? else {
                        return Err(invalid("kernel port name invalid"));
                    };
                    if name.is_empty() || !names.insert(name.clone()) {
                        return Err(invalid("kernel port name empty/repeated"));
                    }
                    let Cell::Id(ty) = get("quantity_type_id")? else {
                        return Err(invalid("kernel port quantity absent"));
                    };
                    let Cell::Id(unit) = get("natural_unit_id")? else {
                        return Err(invalid("kernel port unit absent"));
                    };
                    let Cell::List(dimensions) = get(shape)? else {
                        return Err(invalid("kernel shape absent"));
                    };
                    let shape = dimensions
                        .iter()
                        .map(|cell| match cell {
                            Cell::Enum(value) => DomainKind::parse(value)
                                .ok_or_else(|| invalid("kernel domain kind unknown")),
                            _ => Err(invalid("kernel domain kind is not enum")),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let quantity_type = QuantityTypeId::from_id(*ty);
                    let unit = UnitId::from_id(*unit);
                    let contract = quantities.quantity_type(quantity_type)?;
                    if contract.key.shape != shape {
                        return Err(invalid("kernel shape differs from complete port quantity"));
                    }
                    pse_quantity::convert_spec_for_type(
                        quantities.unit(unit)?,
                        quantities.unit(contract.canonical_unit)?,
                        &contract.key,
                    )?;
                    Ok(KernelPort {
                        name: name.clone(),
                        quantity_type,
                        unit,
                        shape,
                    })
                })
                .collect()
        };
        let contract = KernelContract {
            inputs: ports("inputs", "shape")?,
            outputs: ports("outputs", "shape")?,
            parameters: ports("parameters", "indexed_by")?,
        };
        if result.insert(row.id("kernel_id")?, contract).is_some() {
            return Err(invalid("duplicate kernel specification"));
        }
    }
    Ok(result)
}
