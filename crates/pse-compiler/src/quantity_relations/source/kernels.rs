// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete admitted descriptors projected to the numerical port interface.
use super::{KernelContract, invalid};
use crate::{CompilerError, quantity_relations::kernel_contract::check_descriptor};
use pse_ids::{CancellationToken, SemanticId};
use pse_mathir::infer::KernelPort;
use pse_quantity::{DomainKind, QuantityRegistry, QuantityTypeId, UnitId};
use pse_relations::{columnar::FieldCheckedBatch, generated::reference};
use std::collections::BTreeMap;

pub(super) fn decode(
    batch: &FieldCheckedBatch,
    quantities: &QuantityRegistry,
    cancel: &CancellationToken,
) -> Result<BTreeMap<SemanticId, KernelContract>, CompilerError> {
    let mut result = BTreeMap::new();
    let view = reference::kernel_specs::View::from_checked(batch)?;
    for position in 0..view.len() {
        cancel.checkpoint()?;
        let descriptor = view.row(position)?;
        check_descriptor(&descriptor, quantities)?;
        let contract = KernelContract {
            inputs: descriptor
                .inputs
                .into_iter()
                .map(|port| {
                    kernel_port(
                        port.name,
                        port.quantity_type_id,
                        port.natural_unit_id,
                        &port.shape,
                    )
                })
                .collect::<Result<_, _>>()?,
            outputs: descriptor
                .outputs
                .into_iter()
                .map(|port| {
                    kernel_port(
                        port.name,
                        port.quantity_type_id,
                        port.natural_unit_id,
                        &port.shape,
                    )
                })
                .collect::<Result<_, _>>()?,
            parameters: descriptor
                .parameters
                .into_iter()
                .map(|port| {
                    kernel_port(
                        port.name,
                        port.quantity_type_id,
                        port.natural_unit_id,
                        &port.indexed_by,
                    )
                })
                .collect::<Result<_, _>>()?,
        };
        if result.insert(descriptor.kernel_id, contract).is_some() {
            return Err(invalid("duplicate kernel specification"));
        }
    }
    Ok(result)
}
fn kernel_port(
    name: String,
    ty: SemanticId,
    unit: SemanticId,
    shape: &[pse_relations::generated::enums::DomainKind],
) -> Result<KernelPort, CompilerError> {
    Ok(KernelPort {
        name,
        quantity_type: QuantityTypeId::from_id(ty),
        unit: UnitId::from_id(unit),
        shape: shape
            .iter()
            .map(|kind| {
                DomainKind::parse(kind.as_str())
                    .ok_or_else(|| invalid("kernel domain kind unknown"))
            })
            .collect::<Result<_, _>>()?,
    })
}
