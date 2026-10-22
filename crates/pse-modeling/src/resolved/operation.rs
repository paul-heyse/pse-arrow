// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected physical-operation meaning with direct roots instead of retained source ASTs.
//! Quantity-owned admission facts remain separate from this nominal semantic descriptor.
use super::*;
#[path = "operation_record.rs"]
mod record;
use crate::PhysicalOperation as OriginalOperation;
pub(crate) use record::OperationRecord;

/// Rewritten source expressions supplied by the actual contextual-operation producer.
#[derive(Debug)]
pub(crate) enum OperationRoots {
    None,
    Reference {
        state: [NodeIx; 2],
        anchors: Vec<[NodeIx; 2]>,
    },
}

/// One admitted component anchor and its directly constructed state expressions.
#[derive(Clone, Debug, PartialEq)]
pub struct Anchor {
    member: SemanticId,
    values: [NodeIx; 2],
    temperature: pse_quantity::CanonicalMagnitude,
    pressure: pse_quantity::CanonicalMagnitude,
}
/// Complete selected translation between scientific reference states and their anchors.
#[derive(Clone, Debug, PartialEq)]
pub struct Translation {
    source: pse_quantity::QuantityTypeId,
    target: pse_quantity::QuantityTypeId,
    source_anchor: DeclarationId,
    target_anchor: DeclarationId,
    component_kind: DeclarationId,
    state: [NodeIx; 2],
    provenance: Vec<SemanticId>,
    anchors: Vec<Anchor>,
}

/// The complete consumed nominal meaning. No source witness name or rendered syntax remains.
#[derive(Clone, Debug, PartialEq)]
pub enum OperationMeaning {
    /// Coordinate selected from one declared state map.
    Coordinate {
        /// Actual state-map declaration.
        map: DeclarationId,
        /// Declared coordinate slot.
        slot: DeclarationId,
    },
    /// Reduced law derived from one state reconstruction.
    ReducedLaw {
        /// Actual state-map declaration.
        map: DeclarationId,
        /// Reconstruction supplying the reduced law.
        reconstruction: DeclarationId,
    },
    /// Declared reconstruction evaluated in an explicit reference state.
    Reconstruction {
        /// Actual state-map declaration.
        map: DeclarationId,
        /// Selected reconstruction declaration.
        reconstruction: DeclarationId,
        /// Actual reference-state declaration.
        reference: DeclarationId,
    },
    /// Response authorized by the selected scientific potential.
    Response {
        /// Actual potential declaration, independent of its source alias.
        potential: DeclarationId,
    },
    /// Translation carrying its actual state roots and component anchors.
    ReferenceTranslation(Translation),
    /// Component mean evaluated at one side of a reference translation.
    ReferenceAnchorMean {
        /// Complete translation whose anchors are consumed.
        translation: Translation,
        /// Whether the mean belongs to the source reference side.
        source: bool,
        /// Actual registered coefficient contract.
        coefficient: pse_quantity::QuantityTypeId,
    },
    /// Signed transfer retaining its original and resulting boundary meaning.
    Transfer {
        /// Original boundary refinement when this is a transformed transfer.
        source: Option<crate::PhysicalRefinement>,
        /// Exact resulting boundary refinement.
        result: crate::PhysicalRefinement,
        /// Declared orientation multiplier.
        factor: i8,
        /// Explicit local exchange authorizing a boundary crossing.
        exchange: Option<DeclarationId>,
    },
    /// Magnitude observed from a transfer without erasing its boundary provenance.
    TransferMagnitude {
        /// Original transfer refinement whose magnitude is observed.
        source: crate::PhysicalRefinement,
    },
}
impl Translation {
    fn capture(
        source: &crate::contextual::ReferenceTranslation,
        roots: OperationRoots,
        at: DeclarationId,
    ) -> Result<Self> {
        let OperationRoots::Reference { state, anchors } = roots else {
            return Err(invalid(
                at,
                "selected reference operation lacks its direct contextual roots",
            ));
        };
        if source.anchors.is_empty() || source.anchors.len() != anchors.len() {
            return Err(invalid(at, "selected reference anchor/root arity differs"));
        }
        Ok(Self {
            source: source.source,
            target: source.target,
            source_anchor: source.source_anchor,
            target_anchor: source.target_anchor,
            component_kind: source.component_kind,
            state,
            provenance: source.provenance.clone(),
            anchors: source
                .anchors
                .iter()
                .zip(anchors)
                .map(|(source, values)| Anchor {
                    member: source.member,
                    values,
                    temperature: source.temperature,
                    pressure: source.pressure,
                })
                .collect(),
        })
    }
    fn frame(
        &self,
        nodes: &BTreeMap<NodeIx, usize>,
        hash: &mut FramedHasher,
        at: DeclarationId,
    ) -> Result<()> {
        hash.id(&self.source.as_id())
            .id(&self.target.as_id())
            .id(&self.source_anchor.as_id())
            .id(&self.target_anchor.as_id())
            .id(&self.component_kind.as_id());
        for root in self.state {
            frame_root(root, nodes, hash, at)?;
        }
        hash.u64(self.provenance.len() as u64);
        for id in &self.provenance {
            hash.id(id);
        }
        hash.u64(self.anchors.len() as u64);
        for anchor in &self.anchors {
            hash.id(&anchor.member);
            for value in [anchor.temperature, anchor.pressure] {
                hash.id(&value.quantity().as_id()).u64(value.bits());
            }
            for root in anchor.values {
                frame_root(root, nodes, hash, at)?;
            }
        }
        Ok(())
    }
}
fn frame_root(
    root: NodeIx,
    nodes: &BTreeMap<NodeIx, usize>,
    hash: &mut FramedHasher,
    at: DeclarationId,
) -> Result<()> {
    let ordinal = nodes.get(&root).ok_or_else(|| {
        invalid(
            at,
            "physical-operation root is absent from the same canonical framing walk",
        )
    })?;
    hash.u64(*ordinal as u64);
    Ok(())
}
impl OperationMeaning {
    pub(crate) fn capture(
        source: &OriginalOperation,
        roots: OperationRoots,
        at: DeclarationId,
    ) -> Result<Self> {
        match source {
            OriginalOperation::ReferenceTranslation(source) => {
                return Ok(Self::ReferenceTranslation(Translation::capture(
                    source, roots, at,
                )?));
            }
            OriginalOperation::ReferenceAnchorMean {
                translation,
                source,
                coefficient,
            } => {
                return Ok(Self::ReferenceAnchorMean {
                    translation: Translation::capture(translation, roots, at)?,
                    source: *source,
                    coefficient: *coefficient,
                });
            }
            _ => {}
        }
        if !matches!(roots, OperationRoots::None) {
            return Err(invalid(
                at,
                "noncontextual operation received unrelated contextual roots",
            ));
        }
        Ok(match source {
            OriginalOperation::Coordinate { map, slot } => Self::Coordinate {
                map: *map,
                slot: *slot,
            },
            OriginalOperation::ReducedLaw {
                map,
                reconstruction,
            } => Self::ReducedLaw {
                map: *map,
                reconstruction: *reconstruction,
            },
            OriginalOperation::Reconstruction {
                map,
                reconstruction,
                reference,
            } => Self::Reconstruction {
                map: *map,
                reconstruction: *reconstruction,
                reference: *reference,
            },
            OriginalOperation::Response {
                potential: Some(potential),
                ..
            } => Self::Response {
                potential: *potential,
            },
            OriginalOperation::Response {
                potential: None, ..
            } => {
                return Err(invalid(
                    at,
                    "selected response lacks its actual scientific potential",
                ));
            }
            OriginalOperation::Transfer {
                source,
                result,
                factor,
                exchange,
            } => Self::Transfer {
                source: source.clone(),
                result: result.clone(),
                factor: *factor,
                exchange: *exchange,
            },
            OriginalOperation::TransferMagnitude { source } => Self::TransferMagnitude {
                source: source.clone(),
            },
            OriginalOperation::ReferenceTranslation(_)
            | OriginalOperation::ReferenceAnchorMean { .. } => {
                return Err(invalid(at, "contextual operation roots were not consumed"));
            }
        })
    }
    pub(crate) fn frame(
        &self,
        layout: &CanonicalLayout,
        hash: &mut FramedHasher,
        at: DeclarationId,
    ) -> Result<()> {
        let nodes = layout
            .nodes
            .iter()
            .copied()
            .enumerate()
            .map(|(ordinal, slot)| (slot, ordinal))
            .collect();
        match self {
            Self::Coordinate { map, slot } => {
                hash.str("coordinate").id(&map.as_id()).id(&slot.as_id());
            }
            Self::ReducedLaw {
                map,
                reconstruction,
            } => {
                hash.str("reduced-law")
                    .id(&map.as_id())
                    .id(&reconstruction.as_id());
            }
            Self::Reconstruction {
                map,
                reconstruction,
                reference,
            } => {
                hash.str("reconstruction")
                    .id(&map.as_id())
                    .id(&reconstruction.as_id())
                    .id(&reference.as_id());
            }
            Self::Response { potential } => {
                hash.str("response").id(&potential.as_id());
            }
            Self::ReferenceTranslation(value) => {
                hash.str("reference-translation");
                value.frame(&nodes, hash, at)?;
            }
            Self::ReferenceAnchorMean {
                translation,
                source,
                coefficient,
            } => {
                hash.str("reference-anchor-mean")
                    .bool(*source)
                    .id(&coefficient.as_id());
                translation.frame(&nodes, hash, at)?;
            }
            Self::Transfer {
                source,
                result,
                factor,
                exchange,
            } => {
                hash.str("transfer").bool(source.is_some());
                if let Some(source) = source {
                    source.frame(hash);
                }
                result.frame(hash);
                hash.part(&factor.to_le_bytes()).bool(exchange.is_some());
                if let Some(exchange) = exchange {
                    hash.id(&exchange.as_id());
                }
            }
            Self::TransferMagnitude { source } => {
                hash.str("transfer-contribution");
                source.frame(hash);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_response_requires_the_actual_potential_and_drops_only_its_source_alias() {
        let at = DeclarationId::from_id(SemanticId::from_bytes([4; 16]));
        let selected = |name: &str, potential| OriginalOperation::Response {
            witness: name.into(),
            potential,
        };
        assert!(
            OperationMeaning::capture(&selected("missing", None), OperationRoots::None, at)
                .is_err()
        );
        assert_eq!(
            OperationMeaning::capture(&selected("source_name", Some(at)), OperationRoots::None, at)
                .unwrap(),
            OperationMeaning::capture(&selected("renamed", Some(at)), OperationRoots::None, at)
                .unwrap()
        );
        assert!(
            OperationMeaning::capture(
                &selected("source_name", Some(at)),
                OperationRoots::Reference {
                    state: [NodeIx::from(0); 2],
                    anchors: vec![]
                },
                at
            )
            .is_err()
        );
    }
}
