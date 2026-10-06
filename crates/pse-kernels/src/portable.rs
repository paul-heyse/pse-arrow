// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Opaque immutable provider descriptions. Native factories stay attempt-owned.
use super::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct PortWire {
    id: SemanticId,
    quantity: QuantityTypeId,
    unit: UnitId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ShapeWire {
    id: SemanticId,
    axes: Vec<SemanticId>,
    coordinates: Vec<Vec<SemanticId>>,
    cells: Vec<SemanticId>,
}
impl From<&ProviderShape> for ShapeWire {
    fn from(v: &ProviderShape) -> Self {
        Self {
            id: v.id,
            axes: v.axes.clone(),
            coordinates: v.coordinates.clone(),
            cells: v.cells.clone(),
        }
    }
}
impl ShapeWire {
    fn restore(&self) -> ProviderShape {
        ProviderShape {
            id: self.id,
            axes: self.axes.clone(),
            coordinates: self.coordinates.clone(),
            cells: self.cells.clone(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SpecWire {
    id: SemanticId,
    revision: ContentHash,
    data: ContentHash,
    inputs: Vec<PortWire>,
    outputs: Vec<PortWire>,
    shapes: [Vec<ShapeWire>; 2],
    derivative_source: DerivativeSource,
    derivatives: u8,
    smoothness: u8,
}
/// Untrusted decoded provider data; it contains no callback, factory or native handle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRecord {
    version: u32,
    key: ContentHash,
    spec: SpecWire,
}
impl ProviderRecord {
    /// Complete retained provider descriptor fields, excluding attempt-local factories.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.spec.inputs.capacity() * size_of::<PortWire>()
            + self.spec.outputs.capacity() * size_of::<PortWire>()
            + self
                .spec
                .shapes
                .iter()
                .map(|v| v.capacity() * size_of::<ShapeWire>())
                .sum::<usize>()
            + self
                .spec
                .shapes
                .iter()
                .flat_map(|v| v.iter())
                .map(|v| {
                    size_of::<ShapeWire>()
                        + v.axes.capacity() * size_of::<SemanticId>()
                        + v.cells.capacity() * size_of::<SemanticId>()
                        + v.coordinates.capacity() * size_of::<Vec<SemanticId>>()
                        + v.coordinates
                            .iter()
                            .map(|v| v.capacity() * size_of::<SemanticId>())
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
    /// Capture already admitted exact provider meaning, without worker construction.
    pub fn capture(value: &AdmittedProvider) -> Self {
        let s = value.spec();
        let ports = |values: &[Port]| {
            values
                .iter()
                .map(|v| PortWire {
                    id: v.id,
                    quantity: v.quantity,
                    unit: v.unit,
                })
                .collect()
        };
        Self {
            version: 1,
            key: s.key().0,
            spec: SpecWire {
                id: s.id,
                revision: s.revision,
                data: s.data,
                inputs: ports(&s.inputs),
                outputs: ports(&s.outputs),
                shapes: [
                    s.shapes.inputs.iter().map(ShapeWire::from).collect(),
                    s.shapes.outputs.iter().map(ShapeWire::from).collect(),
                ],
                derivative_source: s.derivative_source,
                derivatives: s.derivatives as u8,
                smoothness: s.smoothness as u8,
            },
        }
    }
    /// Restore inside the caller-qualified physical receipt session. Port admission is
    /// retained exactly; execution still needs an independently matching factory.
    pub fn restore(&self) -> Result<AdmittedProvider, ProviderError> {
        pse_quantity::resolved::receipts::require_record(self)
            .map_err(|e| ProviderError::Contract(e.to_string()))?;
        if self.version != 1 {
            return Err(ProviderError::Contract(
                "unsupported provider record".into(),
            ));
        }
        let order = |v| match v {
            0 => Ok(DerivativeOrder::Value),
            1 => Ok(DerivativeOrder::First),
            2 => Ok(DerivativeOrder::Second),
            _ => Err(ProviderError::Contract(
                "invalid provider derivative receipt".into(),
            )),
        };
        let ports = |values: &[PortWire]| {
            values
                .iter()
                .map(|v| Port {
                    id: v.id,
                    quantity: v.quantity,
                    unit: v.unit,
                })
                .collect()
        };
        let s = &self.spec;
        let spec = ProviderSpec {
            id: s.id,
            revision: s.revision,
            data: s.data,
            inputs: ports(&s.inputs),
            outputs: ports(&s.outputs),
            shapes: ProviderShapes {
                inputs: s.shapes[0].iter().map(ShapeWire::restore).collect(),
                outputs: s.shapes[1].iter().map(ShapeWire::restore).collect(),
            },
            derivative_source: s.derivative_source,
            derivatives: order(s.derivatives)?,
            smoothness: order(s.smoothness)?,
        };
        if spec.key().0 != self.key {
            return Err(ProviderError::Contract(
                "provider record identity differs".into(),
            ));
        }
        Ok(AdmittedProvider(spec))
    }
}
