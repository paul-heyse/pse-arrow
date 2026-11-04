// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Logical indexed arrays map to the existing ordered scalar ABI without inventing coordinates.
use super::*;
use std::collections::BTreeSet;

const MAXIMUM_LOGICAL_PORTS: usize = 4096;
const MINIMUM_SHAPE_AXES: usize = 1;
const MAXIMUM_SHAPE_AXES: usize = 16;
const MAXIMUM_SHAPE_CELLS: usize = MAXIMUM_SCALAR_PORTS;
const LOGICAL_PORTS_RULE: pse_contract::boundary_projection::LocalRule =
    pse_contract::boundary_projection::LocalRule::Length {
        min: 0,
        max: MAXIMUM_LOGICAL_PORTS as u64,
    };
const SHAPE_AXES_RULE: pse_contract::boundary_projection::LocalRule =
    pse_contract::boundary_projection::LocalRule::Length {
        min: MINIMUM_SHAPE_AXES as u64,
        max: MAXIMUM_SHAPE_AXES as u64,
    };
const SHAPE_CELLS_RULE: pse_contract::boundary_projection::LocalRule =
    pse_contract::boundary_projection::LocalRule::Length {
        min: 0,
        max: MAXIMUM_SHAPE_CELLS as u64,
    };
pse_contract::projection_struct! {
/// One logical argument or result array. Sparse or ragged coordinates are explicit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderShape {
    projection: { name: "KernelProviderShape", rust_path: crate::ProviderShape,
        version: 1, direction: pse_contract::boundary_projection::Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Semantic logical argument/result identity.
    pub id: SemanticId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Ordered domain identities.
    pub axes: Vec<SemanticId> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Output, rules: [SHAPE_AXES_RULE] };
    /// Ordered coordinate tuples; no membership is inferred from observed values.
    pub coordinates: Vec<Vec<SemanticId>> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Output, rules: [SHAPE_CELLS_RULE] };
    /// Scalar port IDs in coordinate order.
    pub cells: Vec<SemanticId> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Output, rules: [SHAPE_CELLS_RULE] };
}
}
pse_contract::projection_struct! {
/// Logical shape metadata; unlisted scalar ports retain their scalar meaning.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProviderShapes {
    projection: { name: "KernelProviderShapes", rust_path: crate::ProviderShapes,
        version: 1, direction: pse_contract::boundary_projection::Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Logical shapes over the provider's input ports.
    pub inputs: Vec<ProviderShape> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [LOGICAL_PORTS_RULE] };
    /// Logical shapes over the provider's output ports.
    pub outputs: Vec<ProviderShape> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Output, rules: [LOGICAL_PORTS_RULE] };
}
}
impl ProviderShapes {
    pub(super) fn validate(&self, inputs: &[Port], outputs: &[Port]) -> Result<(), ProviderError> {
        for (shapes, ports) in [(&self.inputs, inputs), (&self.outputs, outputs)] {
            let mut names = BTreeSet::new();
            let mut assigned = BTreeSet::new();
            if shapes.len() > MAXIMUM_LOGICAL_PORTS {
                return Err(ProviderError::Limit("logical provider ports"));
            }
            for s in shapes {
                if !names.insert(s.id)
                    || s.axes.len() < MINIMUM_SHAPE_AXES
                    || s.axes.len() > MAXIMUM_SHAPE_AXES
                    || s.cells.len() > MAXIMUM_SHAPE_CELLS
                    || s.cells.len() != s.coordinates.len()
                    || s.coordinates.iter().any(|c| c.len() != s.axes.len())
                    || s.coordinates.iter().collect::<BTreeSet<_>>().len() != s.coordinates.len()
                {
                    return Err(ProviderError::Contract(
                        "invalid logical provider shape".into(),
                    ));
                }
                let mut physical = None;
                for id in &s.cells {
                    let p = ports.iter().find(|p| p.id == *id).ok_or_else(|| {
                        ProviderError::Contract(
                            "logical provider cell is not in the scalar ABI".into(),
                        )
                    })?;
                    if !assigned.insert(*id) {
                        return Err(ProviderError::Contract(
                            "scalar cell belongs to multiple logical ports".into(),
                        ));
                    }
                    let ty = (p.quantity, p.unit);
                    if physical.replace(ty).is_some_and(|prior| prior != ty) {
                        return Err(ProviderError::Contract(
                            "indexed provider cells have different physical contracts".into(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
    pub(super) fn frame(&self, h: &mut pse_ids::FramedHasher) {
        for shapes in [&self.inputs, &self.outputs] {
            h.u64(shapes.len() as u64);
            for s in shapes {
                h.id(&s.id).u64(s.axes.len() as u64);
                for a in &s.axes {
                    h.id(a);
                }
                h.u64(s.cells.len() as u64);
                for (c, id) in s.coordinates.iter().zip(&s.cells) {
                    for x in c {
                        h.id(x);
                    }
                    h.id(id);
                }
            }
        }
    }
    /// Conservative retained metadata bytes, separate from callback scratch.
    pub fn retained_bytes(&self) -> usize {
        self.inputs
            .iter()
            .chain(&self.outputs)
            .map(|s| {
                size_of::<ProviderShape>()
                    + s.axes.capacity() * size_of::<SemanticId>()
                    + s.cells.capacity() * size_of::<SemanticId>()
                    + s.coordinates.capacity() * size_of::<Vec<SemanticId>>()
                    + s.coordinates
                        .iter()
                        .map(|c| c.capacity() * size_of::<SemanticId>())
                        .sum::<usize>()
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_contract::boundary_projection::ValidateProjection;

    #[test]
    fn logical_shape_projection_preserves_native_limits_and_refusal_categories() {
        let mut shape = ProviderShape {
            id: pse_ids::named_id(SemanticId::NIL, "logical-port"),
            axes: vec![pse_ids::named_id(SemanticId::NIL, "axis"); MAXIMUM_SHAPE_AXES],
            coordinates: Vec::new(),
            cells: Vec::new(),
        };
        shape.validate_projection().unwrap();
        ProviderShapes {
            inputs: vec![shape.clone()],
            outputs: Vec::new(),
        }
        .validate(&[], &[])
        .unwrap();
        shape.axes.push(shape.axes[0]);
        assert!(shape.validate_projection().is_err());
        assert!(matches!(
            ProviderShapes { inputs: vec![shape.clone()], outputs: Vec::new() }.validate(&[], &[]),
            Err(ProviderError::Contract(reason)) if reason == "invalid logical provider shape"
        ));
        shape.axes.clear();
        assert!(shape.validate_projection().is_err());

        let shapes = ProviderShapes {
            inputs: vec![shape; MAXIMUM_LOGICAL_PORTS + 1],
            outputs: Vec::new(),
        };
        assert!(shapes.validate_projection().is_err());
        assert!(matches!(
            shapes.validate(&[], &[]),
            Err(ProviderError::Limit("logical provider ports"))
        ));
    }
}
