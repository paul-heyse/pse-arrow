// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Direct physical-operation metadata, preserving nominal scientific owners and typed roots.
use super::*;
use crate::portable::RefinementWire;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AnchorRecord {
    member: SemanticId,
    values: [NodeIx; 2],
    temperature: pse_quantity::unit::MagnitudeRecord,
    pressure: pse_quantity::unit::MagnitudeRecord,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TranslationRecord {
    source: pse_quantity::QuantityTypeId,
    target: pse_quantity::QuantityTypeId,
    source_anchor: DeclarationId,
    target_anchor: DeclarationId,
    component_kind: DeclarationId,
    state: [NodeIx; 2],
    provenance: Vec<SemanticId>,
    anchors: Vec<AnchorRecord>,
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum OperationRecord {
    Coordinate {
        map: DeclarationId,
        slot: DeclarationId,
    },
    ReducedLaw {
        map: DeclarationId,
        reconstruction: DeclarationId,
    },
    Reconstruction {
        map: DeclarationId,
        reconstruction: DeclarationId,
        reference: DeclarationId,
    },
    Response {
        potential: DeclarationId,
    },
    ReferenceTranslation(TranslationRecord),
    ReferenceAnchorMean {
        translation: TranslationRecord,
        source: bool,
        coefficient: pse_quantity::QuantityTypeId,
    },
    Transfer {
        source: Option<RefinementWire>,
        result: RefinementWire,
        factor: i8,
        exchange: Option<DeclarationId>,
    },
    TransferMagnitude {
        source: RefinementWire,
    },
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
