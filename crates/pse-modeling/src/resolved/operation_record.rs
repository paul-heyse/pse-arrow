// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Direct physical-operation metadata, preserving nominal scientific owners and typed roots.
use super::*;
use crate::portable::RefinementWire;
use serde::{Deserialize, Serialize};
pse_contract::projection_struct! {
/// Passive complete original reference anchor, including exact canonical magnitudes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchorRecord {
    projection: { name: "ModelingOperationAnchorRecord", rust_path: crate::resolved::operation_records::AnchorRecord,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    member: SemanticId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    values: [NodeIx; 2] => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    temperature: pse_quantity::unit::MagnitudeRecord => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    pressure: pse_quantity::unit::MagnitudeRecord => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Passive complete reference translation; physical restoration stays receipt-qualified.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationRecord {
    projection: { name: "ModelingOperationTranslationRecord", rust_path: crate::resolved::operation_records::TranslationRecord,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    source: pse_quantity::QuantityTypeId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    target: pse_quantity::QuantityTypeId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    source_anchor: DeclarationId => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    target_anchor: DeclarationId => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    component_kind: DeclarationId => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    state: [NodeIx; 2] => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    provenance: Vec<SemanticId> => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    anchors: Vec<AnchorRecord> => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
impl TranslationRecord {
    fn capture(value: &Translation) -> Self {
        Self {
            source: value.source,
            target: value.target,
            source_anchor: value.source_anchor,
            target_anchor: value.target_anchor,
            component_kind: value.component_kind,
            state: value.state,
            provenance: value.provenance.clone(),
            anchors: value
                .anchors
                .iter()
                .map(|anchor| AnchorRecord {
                    member: anchor.member,
                    values: anchor.values,
                    temperature: pse_quantity::unit::MagnitudeRecord::capture(anchor.temperature),
                    pressure: pse_quantity::unit::MagnitudeRecord::capture(anchor.pressure),
                })
                .collect(),
        }
    }
    fn restore(&self, at: DeclarationId) -> Result<Translation> {
        Ok::<_, crate::ModelingError>(Translation {
            source: self.source,
            target: self.target,
            source_anchor: self.source_anchor,
            target_anchor: self.target_anchor,
            component_kind: self.component_kind,
            state: self.state,
            provenance: self.provenance.clone(),
            anchors: self
                .anchors
                .iter()
                .map(|anchor| {
                    Ok::<_, crate::ModelingError>(Anchor {
                        member: anchor.member,
                        values: anchor.values,
                        temperature: anchor
                            .temperature
                            .restore()
                            .map_err(|error| invalid(at, error.to_string()))?,
                        pressure: anchor
                            .pressure
                            .restore()
                            .map_err(|error| invalid(at, error.to_string()))?,
                    })
                })
                .collect::<Result<_>>()?,
        })
    }
}
pse_contract::projection_tuple_enum! {
/// Passive original physical-operation attribution, not a physical admission constructor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OperationRecord {
    projection: { name: "ModelingPhysicalOperationRecord", rust_path: crate::resolved::operation_records::OperationRecord,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original coordinate-slot attribution.
    Coordinate {
        /// Original physical map declaration.
        map: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original coordinate slot.
        slot: DeclarationId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 1;
    /// Original reduced-law attribution.
    ReducedLaw {
        /// Original physical map declaration.
        map: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original reconstruction declaration.
        reconstruction: DeclarationId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 2;
    /// Original reconstruction and reference attribution.
    Reconstruction {
        /// Original physical map declaration.
        map: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original reconstruction declaration.
        reconstruction: DeclarationId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original reference declaration.
        reference: DeclarationId => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 3;
    /// Original response potential attribution.
    Response {
        /// Original potential declaration.
        potential: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 4;
    /// Complete original reference translation.
    ReferenceTranslation [/// Complete passive translation data.
        translation: TranslationRecord => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 5;
    /// Complete original reference-anchor mean attribution.
    ReferenceAnchorMean {
        /// Complete passive translation data.
        translation: TranslationRecord => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Whether the original source side was selected.
        source: bool => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original coefficient quantity.
        coefficient: pse_quantity::QuantityTypeId => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 6;
    /// Complete original directed transfer attribution.
    Transfer {
        /// Original optional source refinement.
        source: Option<RefinementWire> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original result refinement.
        result: RefinementWire => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Exact original signed orientation factor.
        factor: i8 => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original optional exchange declaration.
        exchange: Option<DeclarationId> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 7;
    /// Original transfer-magnitude attribution.
    TransferMagnitude {
        /// Original source refinement.
        source: RefinementWire => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 8;
}
}
impl OperationRecord {
    pub(crate) fn capture(value: &OperationMeaning) -> Self {
        match value {
            OperationMeaning::Coordinate { map, slot } => Self::Coordinate {
                map: *map,
                slot: *slot,
            },
            OperationMeaning::ReducedLaw {
                map,
                reconstruction,
            } => Self::ReducedLaw {
                map: *map,
                reconstruction: *reconstruction,
            },
            OperationMeaning::Reconstruction {
                map,
                reconstruction,
                reference,
            } => Self::Reconstruction {
                map: *map,
                reconstruction: *reconstruction,
                reference: *reference,
            },
            OperationMeaning::Response { potential } => Self::Response {
                potential: *potential,
            },
            OperationMeaning::ReferenceTranslation(value) => {
                Self::ReferenceTranslation(TranslationRecord::capture(value))
            }
            OperationMeaning::ReferenceAnchorMean {
                translation,
                source,
                coefficient,
            } => Self::ReferenceAnchorMean {
                translation: TranslationRecord::capture(translation),
                source: *source,
                coefficient: *coefficient,
            },
            OperationMeaning::Transfer {
                source,
                result,
                factor,
                exchange,
            } => Self::Transfer {
                source: source.as_ref().map(RefinementWire::capture),
                result: RefinementWire::capture(result),
                factor: *factor,
                exchange: *exchange,
            },
            OperationMeaning::TransferMagnitude { source } => Self::TransferMagnitude {
                source: RefinementWire::capture(source),
            },
        }
    }
    pub(crate) fn restore(&self, at: DeclarationId) -> Result<OperationMeaning> {
        Ok::<_, crate::ModelingError>(match self {
            Self::Coordinate { map, slot } => OperationMeaning::Coordinate {
                map: *map,
                slot: *slot,
            },
            Self::ReducedLaw {
                map,
                reconstruction,
            } => OperationMeaning::ReducedLaw {
                map: *map,
                reconstruction: *reconstruction,
            },
            Self::Reconstruction {
                map,
                reconstruction,
                reference,
            } => OperationMeaning::Reconstruction {
                map: *map,
                reconstruction: *reconstruction,
                reference: *reference,
            },
            Self::Response { potential } => OperationMeaning::Response {
                potential: *potential,
            },
            Self::ReferenceTranslation(value) => {
                OperationMeaning::ReferenceTranslation(value.restore(at)?)
            }
            Self::ReferenceAnchorMean {
                translation,
                source,
                coefficient,
            } => OperationMeaning::ReferenceAnchorMean {
                translation: translation.restore(at)?,
                source: *source,
                coefficient: *coefficient,
            },
            Self::Transfer {
                source,
                result,
                factor,
                exchange,
            } => OperationMeaning::Transfer {
                source: source.as_ref().map(|value| value.restore(at)).transpose()?,
                result: result.restore(at)?,
                factor: *factor,
                exchange: *exchange,
            },
            Self::TransferMagnitude { source } => OperationMeaning::TransferMagnitude {
                source: source.restore(at)?,
            },
        })
    }
}
