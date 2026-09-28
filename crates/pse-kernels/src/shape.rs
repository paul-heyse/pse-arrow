// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Logical indexed arrays map to the existing ordered scalar ABI without inventing coordinates.
use super::*;
use std::collections::BTreeSet;
/// One logical argument or result array. Sparse or ragged coordinates are explicit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderShape {
    /// Semantic logical argument/result identity.
    pub id: SemanticId,
    /// Ordered domain identities.
    pub axes: Vec<SemanticId>,
    /// Ordered coordinate tuples; no membership is inferred from observed values.
    pub coordinates: Vec<Vec<SemanticId>>,
    /// Scalar port IDs in coordinate order.
    pub cells: Vec<SemanticId>,
}
/// Logical shape metadata; unlisted scalar ports retain their scalar meaning.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProviderShapes {
    /// Logical shapes over the provider's input ports.
    pub inputs: Vec<ProviderShape>,
    /// Logical shapes over the provider's output ports.
    pub outputs: Vec<ProviderShape>,
}
impl ProviderShapes {
    pub(super) fn validate(&self, inputs: &[Port], outputs: &[Port]) -> Result<(), ProviderError> {
        for (shapes, ports) in [(&self.inputs, inputs), (&self.outputs, outputs)] {
            let mut names = BTreeSet::new();
            let mut assigned = BTreeSet::new();
            if shapes.len() > 4096 {
                return Err(ProviderError::Limit("logical provider ports"));
            }
            for s in shapes {
                if !names.insert(s.id)
                    || s.axes.is_empty()
                    || s.axes.len() > 16
                    || s.cells.len() > 4096
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
