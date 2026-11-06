// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owner-controlled restoration of selected checked function meanings.
//! The wire is untrusted data until the enclosing numerical product is qualified.
use crate::{DeclarationId, Function, Result, Type, invalid};
use pse_authoring::dsl::{Expr, Predicate};
use pse_ids::{ContentHash, SemanticId};
use pse_quantity::resolved::receipts::SchemeRecord;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pse_contract::projection_tuple_enum! {
/// Passive complete static value facts; only qualified record restoration admits use.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValueWire {
    projection: { name: "ModelingStaticValueFacts", rust_path: crate::portable::ValueWire,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original boolean value.
    Boolean [/// Exact boolean payload.
        value: bool => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 1;
    /// Original exact structural integer.
    Integer [/// Exact signed payload.
        value: i64 => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 2;
    /// Physical scalar data, with finite admission deferred to its native owner.
    Number {
        /// Original binary64 bits.
        bits: u64 => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [], representation: pse_contract::boundary_projection::Scalar::F64Bits };
        /// Original nominal physical type.
        quantity: pse_quantity::QuantityTypeId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 3;
    /// Physical coordinate data with its original semantic member identity.
    Coordinate {
        /// Original coordinate identity.
        id: SemanticId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original binary64 bits.
        bits: u64 => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [], representation: pse_contract::boundary_projection::Scalar::F64Bits };
        /// Original nominal physical type.
        quantity: pse_quantity::QuantityTypeId => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 4;
    /// Original uninterpreted text.
    Text [/// Original UTF-8 payload.
        value: String => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 5;
    /// Entity member and its nominal kind.
    Entity {
        /// Original entity declaration.
        id: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original entity-kind declaration.
        kind: DeclarationId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 6;
    /// Member of an original package enumeration.
    Enum {
        /// Original enumeration declaration.
        enumeration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original member identity.
        member: SemanticId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 7;
    /// Opaque value of its original identifier scheme.
    Identifier {
        /// Original scheme declaration.
        scheme: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original exact identifier value.
        value: String => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 8;
    /// Definition reference with the complete original static binding map.
    Definition {
        /// Original definition declaration.
        id: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original named bindings; no resolution occurs on adoption.
        bindings: BTreeMap<String, ValueWire> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 9;
    /// Original selected function reference.
    Function [/// Original function declaration.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 10;
    /// Original ordered table row.
    Row {
        /// Original table declaration.
        table: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original field names in row order.
        names: Vec<String> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original field values in the same order.
        fields: Vec<ValueWire> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 11;
    /// Original ordered finite set data.
    Set [/// Retained native member order.
        values: Vec<ValueWire> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 12;
    /// Original ordered product data.
    Tuple [/// Retained native component order.
        values: Vec<ValueWire> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 13;
    /// Passive reference to a physical type.
    QuantityType [/// Original physical type identity.
        quantity: pse_quantity::QuantityTypeId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 14;
    /// Passive reference to a physical datum.
    ReferenceState [/// Original reference-state identity.
        reference: pse_quantity::ReferenceStateId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 15;
    /// Explicit original absence.
    Missing => 16;
}
}
impl ValueWire {
    fn capture(v: &crate::specialize::Value) -> Self {
        use crate::specialize::Value as V;
        match v {
            V::Boolean(v) => Self::Boolean(*v),
            V::Integer(v) => Self::Integer(*v),
            V::Number { bits, quantity } => Self::Number {
                bits: *bits,
                quantity: *quantity,
            },
            V::Coordinate { id, bits, quantity } => Self::Coordinate {
                id: *id,
                bits: *bits,
                quantity: *quantity,
            },
            V::Text(v) => Self::Text(v.clone()),
            V::Entity { id, kind } => Self::Entity {
                id: *id,
                kind: *kind,
            },
            V::Enum {
                enumeration,
                member,
            } => Self::Enum {
                enumeration: *enumeration,
                member: *member,
            },
            V::Identifier { scheme, value } => Self::Identifier {
                scheme: *scheme,
                value: value.clone(),
            },
            V::Definition { id, bindings } => Self::Definition {
                id: *id,
                bindings: bindings
                    .iter()
                    .map(|(n, v)| (n.clone(), Self::capture(v)))
                    .collect(),
            },
            V::Function(v) => Self::Function(*v),
            V::Row {
                table,
                names,
                fields,
            } => Self::Row {
                table: *table,
                names: names.to_vec(),
                fields: fields.iter().map(Self::capture).collect(),
            },
            V::Set(v) => Self::Set(v.iter().map(Self::capture).collect()),
            V::Tuple(v) => Self::Tuple(v.iter().map(Self::capture).collect()),
            V::QuantityType(v) => Self::QuantityType(*v),
            V::ReferenceState(v) => Self::ReferenceState(*v),
            V::Missing => Self::Missing,
        }
    }
    fn restore(&self, at: DeclarationId) -> Result<crate::specialize::Value> {
        use crate::specialize::Value as V;
        Ok(match self {
            Self::Boolean(v) => V::Boolean(*v),
            Self::Integer(v) => V::Integer(*v),
            Self::Number { bits, quantity } => {
                if !f64::from_bits(*bits).is_finite() {
                    return Err(invalid(at, "nonfinite structural receipt"));
                }
                V::Number {
                    bits: *bits,
                    quantity: *quantity,
                }
            }
            Self::Coordinate { id, bits, quantity } => {
                if !f64::from_bits(*bits).is_finite() {
                    return Err(invalid(at, "nonfinite coordinate receipt"));
                }
                V::Coordinate {
                    id: *id,
                    bits: *bits,
                    quantity: *quantity,
                }
            }
            Self::Text(v) => V::Text(v.clone()),
            Self::Entity { id, kind } => V::Entity {
                id: *id,
                kind: *kind,
            },
            Self::Enum {
                enumeration,
                member,
            } => V::Enum {
                enumeration: *enumeration,
                member: *member,
            },
            Self::Identifier { scheme, value } => V::Identifier {
                scheme: *scheme,
                value: value.clone(),
            },
            Self::Definition { id, bindings } => V::Definition {
                id: *id,
                bindings: bindings
                    .iter()
                    .map(|(n, v)| Ok((n.clone(), v.restore(at)?)))
                    .collect::<Result<_>>()?,
            },
            Self::Function(v) => V::Function(*v),
            Self::Row {
                table,
                names,
                fields,
            } => V::Row {
                table: *table,
                names: names.clone().into(),
                fields: fields
                    .iter()
                    .map(|v| v.restore(at))
                    .collect::<Result<Vec<_>>>()?
                    .into(),
            },
            Self::Set(v) => V::Set(v.iter().map(|v| v.restore(at)).collect::<Result<_>>()?),
            Self::Tuple(v) => V::Tuple(v.iter().map(|v| v.restore(at)).collect::<Result<_>>()?),
            Self::QuantityType(v) => V::QuantityType(*v),
            Self::ReferenceState(v) => V::ReferenceState(*v),
            Self::Missing => V::Missing,
        })
    }
    fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + match self {
                Self::Text(v) | Self::Identifier { value: v, .. } => v.capacity(),
                Self::Definition { bindings, .. } => bindings
                    .iter()
                    .map(|(n, v)| 64 + n.capacity() + v.retained_bytes())
                    .sum(),
                Self::Row { names, fields, .. } => {
                    names.capacity() * size_of::<String>()
                        + fields.capacity() * size_of::<ValueWire>()
                        + names
                            .iter()
                            .map(|n| size_of::<String>() + n.capacity())
                            .sum::<usize>()
                        + fields.iter().map(Self::retained_bytes).sum::<usize>()
                }
                Self::Set(v) | Self::Tuple(v) => {
                    v.capacity() * size_of::<ValueWire>()
                        + v.iter().map(Self::retained_bytes).sum::<usize>()
                }
                _ => 0,
            }
    }
}
pse_contract::projection_tuple_enum! {
/// Passive original boundary attribution, without a physical boundary authorization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BoundaryWire {
    projection: { name: "ModelingBoundaryFacts", rust_path: crate::portable::BoundaryWire,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original unbound boundary declaration.
    Declared [/// Original boundary identity.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 1;
    /// Original bound instance and exact static coordinates.
    Bound {
        /// Original instance identity.
        instance: crate::InstanceId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original boundary declaration.
        declaration: DeclarationId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original ordered coordinate data.
        coordinates: Vec<ValueWire> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 2;
}
}
pse_contract::projection_enum! {
/// Passive complete nominal refinement facts, restored after whole-record membership.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RefinementWire {
    projection: { name: "ModelingRefinementFacts", rust_path: crate::portable::RefinementWire,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original named coordinate refinement.
    Coordinate {
        /// Original physical coordinate map.
        map: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original coordinate slot.
        slot: DeclarationId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 1;
    /// Original reduced-law refinement.
    ReducedLaw {
        /// Original physical coordinate map.
        map: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original reconstruction declaration.
        reconstruction: DeclarationId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 2;
    /// Original directed transfer refinement.
    Transfer {
        /// Original complete boundary attribution.
        boundary: BoundaryWire => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Whether the original direction is into that boundary.
        enters_boundary: bool => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 3;
}
}
impl RefinementWire {
    pub(crate) fn capture(v: &crate::PhysicalRefinement) -> Self {
        use crate::PhysicalRefinement as R;
        match v {
            R::Coordinate { map, slot } => Self::Coordinate {
                map: *map,
                slot: *slot,
            },
            R::ReducedLaw {
                map,
                reconstruction,
            } => Self::ReducedLaw {
                map: *map,
                reconstruction: *reconstruction,
            },
            R::Transfer {
                boundary,
                direction,
            } => Self::Transfer {
                boundary: match boundary {
                    crate::BoundaryRef::Declared(v) => BoundaryWire::Declared(*v),
                    crate::BoundaryRef::Bound {
                        instance,
                        declaration,
                        coordinates,
                    } => BoundaryWire::Bound {
                        instance: *instance,
                        declaration: *declaration,
                        coordinates: coordinates.iter().map(ValueWire::capture).collect(),
                    },
                },
                enters_boundary: *direction == crate::TransferDirection::Into,
            },
        }
    }
    pub(crate) fn restore(&self, at: DeclarationId) -> Result<crate::PhysicalRefinement> {
        use crate::PhysicalRefinement as R;
        Ok(match self {
            Self::Coordinate { map, slot } => R::Coordinate {
                map: *map,
                slot: *slot,
            },
            Self::ReducedLaw {
                map,
                reconstruction,
            } => R::ReducedLaw {
                map: *map,
                reconstruction: *reconstruction,
            },
            Self::Transfer {
                boundary,
                enters_boundary,
            } => R::Transfer {
                boundary: match boundary {
                    BoundaryWire::Declared(v) => crate::BoundaryRef::Declared(*v),
                    BoundaryWire::Bound {
                        instance,
                        declaration,
                        coordinates,
                    } => crate::BoundaryRef::Bound {
                        instance: *instance,
                        declaration: *declaration,
                        coordinates: coordinates
                            .iter()
                            .map(|v| v.restore(at))
                            .collect::<Result<_>>()?,
                    },
                },
                direction: if *enters_boundary {
                    crate::TransferDirection::Into
                } else {
                    crate::TransferDirection::OutOf
                },
            },
        })
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + match self {
                Self::Transfer {
                    boundary: BoundaryWire::Bound { coordinates, .. },
                    ..
                } => coordinates
                    .iter()
                    .map(ValueWire::retained_bytes)
                    .sum::<usize>(),
                _ => 0,
            }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct TranslationWire {
    source: pse_quantity::QuantityTypeId,
    target: pse_quantity::QuantityTypeId,
    source_anchor: DeclarationId,
    target_anchor: DeclarationId,
    temperature: Expr,
    pressure: Expr,
    component_kind: DeclarationId,
    provenance: Vec<SemanticId>,
    anchors: Vec<AnchorWire>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnchorWire {
    member: SemanticId,
    values: [Expr; 2],
    temperature: pse_quantity::unit::MagnitudeRecord,
    pressure: pse_quantity::unit::MagnitudeRecord,
}
impl TranslationWire {
    fn capture(v: &crate::contextual::ReferenceTranslation) -> Self {
        Self {
            source: v.source,
            target: v.target,
            source_anchor: v.source_anchor,
            target_anchor: v.target_anchor,
            temperature: v.temperature.clone(),
            pressure: v.pressure.clone(),
            component_kind: v.component_kind,
            provenance: v.provenance.clone(),
            anchors: v
                .anchors
                .iter()
                .map(|v| AnchorWire {
                    member: v.member,
                    values: v.values.clone(),
                    temperature: pse_quantity::unit::MagnitudeRecord::capture(v.temperature),
                    pressure: pse_quantity::unit::MagnitudeRecord::capture(v.pressure),
                })
                .collect(),
        }
    }
    fn restore(&self, at: DeclarationId) -> Result<crate::contextual::ReferenceTranslation> {
        Ok(crate::contextual::ReferenceTranslation {
            source: self.source,
            target: self.target,
            source_anchor: self.source_anchor,
            target_anchor: self.target_anchor,
            temperature: self.temperature.clone(),
            pressure: self.pressure.clone(),
            component_kind: self.component_kind,
            provenance: self.provenance.clone(),
            anchors: self
                .anchors
                .iter()
                .map(|v| {
                    Ok(crate::contextual::ReferenceAnchorPair {
                        member: v.member,
                        values: v.values.clone(),
                        temperature: v
                            .temperature
                            .restore()
                            .map_err(|e| invalid(at, e.to_string()))?,
                        pressure: v
                            .pressure
                            .restore()
                            .map_err(|e| invalid(at, e.to_string()))?,
                    })
                })
                .collect::<Result<_>>()?,
        })
    }
    fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + crate::expression::retained_bytes(&self.temperature)
            + crate::expression::retained_bytes(&self.pressure)
            + self.provenance.capacity() * size_of::<SemanticId>()
            + self.anchors.capacity() * size_of::<AnchorWire>()
            + self
                .anchors
                .iter()
                .map(|v| {
                    size_of::<AnchorWire>()
                        + v.values
                            .iter()
                            .map(crate::expression::retained_bytes)
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum OperationWire {
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
        witness: String,
        potential: Option<DeclarationId>,
    },
    ReferenceTranslation(TranslationWire),
    ReferenceAnchorMean {
        translation: TranslationWire,
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
impl OperationWire {
    fn capture(v: &crate::PhysicalOperation) -> Self {
        use crate::PhysicalOperation as O;
        match v {
            O::Coordinate { map, slot } => Self::Coordinate {
                map: *map,
                slot: *slot,
            },
            O::ReducedLaw {
                map,
                reconstruction,
            } => Self::ReducedLaw {
                map: *map,
                reconstruction: *reconstruction,
            },
            O::Reconstruction {
                map,
                reconstruction,
                reference,
            } => Self::Reconstruction {
                map: *map,
                reconstruction: *reconstruction,
                reference: *reference,
            },
            O::Response { witness, potential } => Self::Response {
                witness: witness.clone(),
                potential: *potential,
            },
            O::ReferenceTranslation(v) => Self::ReferenceTranslation(TranslationWire::capture(v)),
            O::ReferenceAnchorMean {
                translation,
                source,
                coefficient,
            } => Self::ReferenceAnchorMean {
                translation: TranslationWire::capture(translation),
                source: *source,
                coefficient: *coefficient,
            },
            O::Transfer {
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
            O::TransferMagnitude { source } => Self::TransferMagnitude {
                source: RefinementWire::capture(source),
            },
        }
    }
    fn restore(&self, at: DeclarationId) -> Result<crate::PhysicalOperation> {
        use crate::PhysicalOperation as O;
        Ok(match self {
            Self::Coordinate { map, slot } => O::Coordinate {
                map: *map,
                slot: *slot,
            },
            Self::ReducedLaw {
                map,
                reconstruction,
            } => O::ReducedLaw {
                map: *map,
                reconstruction: *reconstruction,
            },
            Self::Reconstruction {
                map,
                reconstruction,
                reference,
            } => O::Reconstruction {
                map: *map,
                reconstruction: *reconstruction,
                reference: *reference,
            },
            Self::Response { witness, potential } => O::Response {
                witness: witness.clone(),
                potential: *potential,
            },
            Self::ReferenceTranslation(v) => O::ReferenceTranslation(v.restore(at)?),
            Self::ReferenceAnchorMean {
                translation,
                source,
                coefficient,
            } => O::ReferenceAnchorMean {
                translation: translation.restore(at)?,
                source: *source,
                coefficient: *coefficient,
            },
            Self::Transfer {
                source,
                result,
                factor,
                exchange,
            } => O::Transfer {
                source: source.as_ref().map(|v| v.restore(at)).transpose()?,
                result: result.restore(at)?,
                factor: *factor,
                exchange: *exchange,
            },
            Self::TransferMagnitude { source } => O::TransferMagnitude {
                source: source.restore(at)?,
            },
        })
    }
    fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + match self {
                Self::Response { witness, .. } => witness.capacity(),
                Self::ReferenceTranslation(v)
                | Self::ReferenceAnchorMean { translation: v, .. } => v.retained_bytes(),
                Self::Transfer { source, result, .. } => {
                    source.as_ref().map_or(0, RefinementWire::retained_bytes)
                        + result.retained_bytes()
                }
                Self::TransferMagnitude { source } => source.retained_bytes(),
                _ => 0,
            }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ReductionWire {
    kind: pse_quantity::ReductionKind,
    domain: Option<pse_quantity::EntityKindId>,
    prototype: pse_quantity::resolved::receipts::ContractRecord,
}
pse_contract::projection_tuple_enum! {
/// Passive complete type facts. Resolved physical leaves stay receipt-qualified.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TypeWire {
    projection: { name: "ModelingTypeFacts", rust_path: crate::portable::TypeWire,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Retained boolean role.
    Boolean => 1;
    /// Retained exact integer role.
    Integer => 2;
    /// Retained text role.
    Text => 3;
    /// A value naming a physical type.
    QuantityType => 4;
    /// A value naming a physical datum.
    ReferenceState => 5;
    /// Retained applicability-claim role.
    Applicability => 6;
    /// Retained declared boundary role.
    Boundary [/// Original boundary declaration.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 7;
    /// Retained physical coordinate-map role.
    CoordinateMap [/// Original map declaration.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 8;
    /// Retained reconstruction and its original map.
    Reconstruction {
        /// Original reconstruction declaration.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original coordinate-map declaration.
        map: DeclarationId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 9;
    /// Complete retained physical type scheme data.
    Quantity [/// Original passive scheme record.
        scheme: SchemeRecord => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 10;
    /// Complete retained scheme and its nominal refinement.
    Refined {
        /// Original passive scheme record.
        quantity: SchemeRecord => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original refinement facts.
        refinement: RefinementWire => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 11;
    /// Retained nominal entity-kind role.
    Entity [/// Original entity-kind declaration.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 12;
    /// Retained package-enumeration role.
    Enum [/// Original enumeration declaration.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 13;
    /// Retained opaque identifier role.
    Identifier [/// Original identifier-scheme declaration.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 14;
    /// Retained finite-set element type.
    Set [/// Original complete element type.
        element: Box<TypeWire> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 15;
    /// Retained continuous-domain element type.
    Continuous [/// Original domain declaration.
        domain: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original complete element type.
        element: Box<TypeWire> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 16;
    /// Retained ordered product type.
    Tuple [/// Original component types in native order.
        elements: Vec<TypeWire> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 17;
    /// Retained table role.
    Table [/// Original table declaration.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 18;
    /// Retained row role.
    Row [/// Original table declaration.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 19;
    /// Retained definition role.
    Definition [/// Original definition declaration.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 20;
    /// Retained interface role.
    Interface [/// Original interface declaration.
        declaration: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 21;
    /// Retained complete function signature.
    Function {
        /// Original named arguments in declaration order.
        arguments: Vec<(String, TypeWire)> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original complete result type.
        result: Box<TypeWire> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 22;
    /// Retained explicit optional type.
    Optional [/// Original contained type.
        element: Box<TypeWire> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 23;
    /// Retained indexed type and coordinate-kind declarations.
    Indexed {
        /// Original complete element type.
        element: Box<TypeWire> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original coordinate kinds in axis order.
        axes: Vec<DeclarationId> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 24;
}
}
impl TypeWire {
    fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + match self {
                Self::Quantity(v) => v.retained_bytes(),
                Self::Refined {
                    quantity,
                    refinement,
                } => quantity.retained_bytes() + refinement.retained_bytes(),
                Self::Set(v) | Self::Continuous(_, v) | Self::Optional(v) => v.retained_bytes(),
                Self::Tuple(v) => {
                    v.capacity() * size_of::<TypeWire>()
                        + v.iter().map(Self::retained_bytes).sum::<usize>()
                }
                Self::Function { arguments, result } => {
                    arguments.capacity() * size_of::<(String, TypeWire)>()
                        + result.retained_bytes()
                        + arguments
                            .iter()
                            .map(|(n, t)| n.capacity() + t.retained_bytes())
                            .sum::<usize>()
                }
                Self::Indexed { element, axes } => {
                    element.retained_bytes() + axes.capacity() * size_of::<DeclarationId>()
                }
                _ => 0,
            }
    }
    fn capture(v: &Type) -> Self {
        match v {
            Type::Boolean => Self::Boolean,
            Type::Integer => Self::Integer,
            Type::Text => Self::Text,
            Type::QuantityType => Self::QuantityType,
            Type::ReferenceState => Self::ReferenceState,
            Type::Applicability => Self::Applicability,
            Type::Boundary(v) => Self::Boundary(*v),
            Type::CoordinateMap(v) => Self::CoordinateMap(*v),
            Type::Reconstruction { declaration, map } => Self::Reconstruction {
                declaration: *declaration,
                map: *map,
            },
            Type::Quantity(v) => Self::Quantity(SchemeRecord::capture(v)),
            Type::RefinedQuantity {
                quantity,
                refinement,
            } => Self::Refined {
                quantity: SchemeRecord::capture(quantity),
                refinement: RefinementWire::capture(refinement),
            },
            Type::Entity(v) => Self::Entity(*v),
            Type::Enum(v) => Self::Enum(*v),
            Type::Identifier(v) => Self::Identifier(*v),
            Type::Set(v) => Self::Set(Box::new(Self::capture(v))),
            Type::Continuous(id, v) => Self::Continuous(*id, Box::new(Self::capture(v))),
            Type::Tuple(v) => Self::Tuple(v.iter().map(Self::capture).collect()),
            Type::Table(v) => Self::Table(*v),
            Type::Row(v) => Self::Row(*v),
            Type::Definition(v) => Self::Definition(*v),
            Type::Interface(v) => Self::Interface(*v),
            Type::Function { arguments, result } => Self::Function {
                arguments: arguments
                    .iter()
                    .map(|(n, t)| (n.clone(), Self::capture(t)))
                    .collect(),
                result: Box::new(Self::capture(result)),
            },
            Type::Optional(v) => Self::Optional(Box::new(Self::capture(v))),
            Type::Indexed { element, axes } => Self::Indexed {
                element: Box::new(Self::capture(element)),
                axes: axes.clone(),
            },
        }
    }
    fn restore(&self, at: DeclarationId) -> Result<Type> {
        Ok(match self {
            Self::Boolean => Type::Boolean,
            Self::Integer => Type::Integer,
            Self::Text => Type::Text,
            Self::QuantityType => Type::QuantityType,
            Self::ReferenceState => Type::ReferenceState,
            Self::Applicability => Type::Applicability,
            Self::Boundary(v) => Type::Boundary(*v),
            Self::CoordinateMap(v) => Type::CoordinateMap(*v),
            Self::Reconstruction { declaration, map } => Type::Reconstruction {
                declaration: *declaration,
                map: *map,
            },
            Self::Quantity(v) => Type::Quantity(
                v.restore()
                    .map_err(|error| crate::record_membership_error(at, error))?,
            ),
            Self::Refined {
                quantity,
                refinement,
            } => Type::RefinedQuantity {
                quantity: quantity
                    .restore()
                    .map_err(|error| crate::record_membership_error(at, error))?,
                refinement: refinement.restore(at)?,
            },
            Self::Entity(v) => Type::Entity(*v),
            Self::Enum(v) => Type::Enum(*v),
            Self::Identifier(v) => Type::Identifier(*v),
            Self::Set(v) => Type::Set(Box::new(v.restore(at)?)),
            Self::Continuous(id, v) => Type::Continuous(*id, Box::new(v.restore(at)?)),
            Self::Tuple(v) => Type::Tuple(v.iter().map(|v| v.restore(at)).collect::<Result<_>>()?),
            Self::Table(v) => Type::Table(*v),
            Self::Row(v) => Type::Row(*v),
            Self::Definition(v) => Type::Definition(*v),
            Self::Interface(v) => Type::Interface(*v),
            Self::Function { arguments, result } => Type::Function {
                arguments: arguments
                    .iter()
                    .map(|(n, t)| Ok((n.clone(), t.restore(at)?)))
                    .collect::<Result<_>>()?,
                result: Box::new(result.restore(at)?),
            },
            Self::Optional(v) => Type::Optional(Box::new(v.restore(at)?)),
            Self::Indexed { element, axes } => Type::Indexed {
                element: Box::new(element.restore(at)?),
                axes: axes.clone(),
            },
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct UseWire {
    node: pse_model::applicability::Node,
    predicates: Vec<Predicate>,
    inputs: Vec<Expr>,
}
pse_contract::projection_struct! {
/// Passive complete original envelope read attribution; decoding performs no read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadsWire {
    projection: { name: "ModelingEnvelopeReadFacts", rust_path: crate::portable::ReadsWire,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    sets: Vec<SemanticId> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    variables: Vec<u32> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
impl From<&crate::envelope::Reads> for ReadsWire {
    fn from(v: &crate::envelope::Reads) -> Self {
        Self {
            sets: v.sets.clone(),
            variables: v.variables.clone(),
        }
    }
}
impl ReadsWire {
    pub(crate) fn restore(&self) -> crate::envelope::Reads {
        crate::envelope::Reads {
            sets: self.sets.clone(),
            variables: self.variables.clone(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct GuardWire {
    owner: DeclarationId,
    axis: String,
    ty: TypeWire,
    lower: String,
    upper: String,
    carrier: String,
    extent: pse_model::generated::enums::ModelingEnvelopeExtent,
    arguments: Vec<String>,
    predicate: Predicate,
    reads: ReadsWire,
}
impl GuardWire {
    fn capture(v: &crate::envelope::Guard) -> Self {
        Self {
            owner: v.envelope.owner,
            axis: v.envelope.axis.clone(),
            ty: TypeWire::capture(&v.envelope.ty),
            lower: v.envelope.lower.clone(),
            upper: v.envelope.upper.clone(),
            carrier: v.carrier.clone(),
            extent: v.extent,
            arguments: v.arguments.clone(),
            predicate: v.predicate.clone(),
            reads: ReadsWire::from(&v.reads),
        }
    }
    fn restore(&self, at: DeclarationId) -> Result<crate::envelope::Guard> {
        Ok(crate::envelope::Guard {
            envelope: crate::envelope::Envelope {
                owner: self.owner,
                axis: self.axis.clone(),
                ty: self.ty.restore(at)?,
                lower: self.lower.clone(),
                upper: self.upper.clone(),
            },
            carrier: self.carrier.clone(),
            extent: self.extent,
            arguments: self.arguments.clone(),
            predicate: self.predicate.clone(),
            reads: self.reads.restore(),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ShapeWire {
    argument: SemanticId,
    axes: Vec<SemanticId>,
    coordinates: Vec<Vec<SemanticId>>,
    start: usize,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ExternalWire {
    implementation: String,
    revision: ContentHash,
    data: ContentHash,
    output: Expr,
    derivative_source: crate::external::DerivativeSource,
    derivatives: u8,
    smoothness: u8,
    shapes: Vec<ShapeWire>,
}
impl ExternalWire {
    fn capture(v: &crate::external::External) -> Self {
        Self {
            implementation: v.implementation.clone(),
            revision: v.revision,
            data: v.data,
            output: v.output.clone(),
            derivative_source: v.derivative_source,
            derivatives: v.derivatives,
            smoothness: v.smoothness,
            shapes: v
                .shapes
                .iter()
                .map(|v| ShapeWire {
                    argument: v.argument,
                    axes: v.axes.clone(),
                    coordinates: v.coordinates.clone(),
                    start: v.start,
                })
                .collect(),
        }
    }
    fn restore(&self) -> crate::external::External {
        crate::external::External {
            implementation: self.implementation.clone(),
            revision: self.revision,
            data: self.data,
            output: self.output.clone(),
            derivative_source: self.derivative_source,
            derivatives: self.derivatives,
            smoothness: self.smoothness,
            shapes: self
                .shapes
                .iter()
                .map(|v| crate::external::ArgumentShape {
                    argument: v.argument,
                    axes: v.axes.clone(),
                    coordinates: v.coordinates.clone(),
                    start: v.start,
                })
                .collect(),
        }
    }
}
/// Opaque selected checked function meaning. Authoritative checked types have no
/// wire deserializer; proof/role authority is consumed by separate strict receipts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionRecord {
    version: u32,
    id: DeclarationId,
    operation: Option<OperationWire>,
    reduction: Option<ReductionWire>,
    continuity: Option<u8>,
    applicability: Vec<Expr>,
    uses: Vec<UseWire>,
    prerequisites: Vec<usize>,
    admissions: Vec<(
        crate::expression::admission::ExpressionOccurrence,
        crate::expression::admission::PhysicalAdmissionRecord,
    )>,
    validity: Option<Predicate>,
    envelopes: Vec<GuardWire>,
    reads: ReadsWire,
    external: Option<ExternalWire>,
    variables: BTreeSet<String>,
    arguments: Vec<(String, TypeWire)>,
    result: TypeWire,
    body: Option<Expr>,
}
impl FunctionRecord {
    /// Conservative owned-container accounting for this selected portable meaning.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.applicability.capacity() * size_of::<Expr>()
            + self.uses.capacity() * size_of::<UseWire>()
            + self.admissions.capacity()
                * size_of::<(
                    crate::expression::admission::ExpressionOccurrence,
                    crate::expression::admission::PhysicalAdmissionRecord,
                )>()
            + self.envelopes.capacity() * size_of::<GuardWire>()
            + self.arguments.capacity() * size_of::<(String, TypeWire)>()
            + self
                .operation
                .as_ref()
                .map_or(0, OperationWire::retained_bytes)
            + self.reduction.as_ref().map_or(0, |v| {
                size_of::<ReductionWire>() + v.prototype.retained_bytes()
            })
            + self
                .applicability
                .iter()
                .map(crate::expression::retained_bytes)
                .sum::<usize>()
            + self
                .uses
                .iter()
                .map(|v| {
                    size_of::<UseWire>()
                        + v.node.retained_bytes()
                        + v.predicates.capacity() * size_of::<Predicate>()
                        + v.inputs.capacity() * size_of::<Expr>()
                        + v.predicates
                            .iter()
                            .map(crate::extent::predicate)
                            .sum::<usize>()
                        + v.inputs
                            .iter()
                            .map(crate::expression::retained_bytes)
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.prerequisites.capacity() * size_of::<usize>()
            + self
                .admissions
                .iter()
                .map(|(_, a)| {
                    size_of::<crate::expression::admission::ExpressionOccurrence>()
                        + a.retained_bytes()
                })
                .sum::<usize>()
            + self.validity.as_ref().map_or(0, crate::extent::predicate)
            + self
                .envelopes
                .iter()
                .map(|v| {
                    size_of::<GuardWire>()
                        + v.axis.capacity()
                        + v.ty.retained_bytes()
                        + v.lower.capacity()
                        + v.upper.capacity()
                        + v.carrier.capacity()
                        + v.arguments
                            .iter()
                            .map(|v| size_of::<String>() + v.capacity())
                            .sum::<usize>()
                        + crate::extent::predicate(&v.predicate)
                        + v.reads.sets.capacity() * size_of::<SemanticId>()
                        + v.reads.variables.capacity() * size_of::<u32>()
                })
                .sum::<usize>()
            + self.reads.sets.capacity() * size_of::<SemanticId>()
            + self.reads.variables.capacity() * size_of::<u32>()
            + self
                .variables
                .iter()
                .map(|v| size_of::<String>() + 64 + v.capacity())
                .sum::<usize>()
            + self
                .arguments
                .iter()
                .map(|(n, t)| size_of::<String>() + n.capacity() + t.retained_bytes())
                .sum::<usize>()
            + self.result.retained_bytes()
            + self
                .body
                .as_ref()
                .map_or(0, crate::expression::retained_bytes)
            + self.external.as_ref().map_or(0, |v| {
                size_of::<ExternalWire>()
                    + v.implementation.capacity()
                    + crate::expression::retained_bytes(&v.output)
                    + v.shapes.capacity() * size_of::<ShapeWire>()
                    + v.shapes
                        .iter()
                        .map(|s| {
                            size_of::<ShapeWire>()
                                + s.axes.capacity() * size_of::<SemanticId>()
                                + s.coordinates.capacity() * size_of::<Vec<SemanticId>>()
                                + s.coordinates
                                    .iter()
                                    .map(|v| v.capacity() * size_of::<SemanticId>())
                                    .sum::<usize>()
                        })
                        .sum::<usize>()
            })
    }
    /// Capture complete selected checked meaning without checking or specializing it.
    pub fn capture(v: &Function) -> Self {
        Self {
            version: 2,
            id: v.id,
            operation: v.physical_operation.as_ref().map(OperationWire::capture),
            reduction: v.reduction.as_ref().map(|v| ReductionWire {
                kind: v.kind,
                domain: v.domain,
                prototype: pse_quantity::resolved::receipts::ContractRecord::capture(&v.prototype),
            }),
            continuity: v.continuity,
            applicability: v.applicability.clone(),
            uses: v
                .applicability_uses
                .iter()
                .map(|v| UseWire {
                    node: v.node.clone(),
                    predicates: v.predicates.clone(),
                    inputs: v.inputs.clone(),
                })
                .collect(),
            prerequisites: v.prerequisites.clone(),
            admissions: v
                .physical_admissions
                .iter()
                .map(|(o, a)| {
                    (
                        *o,
                        crate::expression::admission::PhysicalAdmissionRecord::capture(a),
                    )
                })
                .collect(),
            validity: v.validity.clone(),
            envelopes: v.envelopes.iter().map(GuardWire::capture).collect(),
            reads: ReadsWire::from(&v.validity_reads),
            external: v.external.as_ref().map(ExternalWire::capture),
            variables: v.variables.clone(),
            arguments: v
                .arguments
                .iter()
                .map(|(n, t)| (n.clone(), TypeWire::capture(t)))
                .collect(),
            result: TypeWire::capture(&v.result),
            body: v.body.clone(),
        }
    }
    /// Restore inside a qualified strict numerical session, retaining every original
    /// occurrence product; no expression checking, specialization or proof runs here.
    pub fn restore(&self) -> Result<Function> {
        pse_quantity::resolved::receipts::require_record(self)
            .map_err(|error| crate::record_membership_error(self.id, error))?;
        if self.version != 2 {
            return Err(invalid(self.id, "unsupported function record"));
        }
        let admissions = self
            .admissions
            .iter()
            .map(|(o, a)| Ok((*o, a.restore(self.id)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        if admissions.len() != self.admissions.len() {
            return Err(invalid(self.id, "duplicate function admission occurrence"));
        }
        let function = Function {
            applicability: self.applicability.clone(),
            applicability_uses: self
                .uses
                .iter()
                .map(|v| crate::applicability::Use {
                    node: v.node.clone(),
                    predicates: v.predicates.clone(),
                    inputs: v.inputs.clone(),
                })
                .collect(),
            prerequisites: self.prerequisites.clone(),
            physical_admissions: admissions,
            physical_operation: self
                .operation
                .as_ref()
                .map(|v| v.restore(self.id))
                .transpose()?,
            reduction: self
                .reduction
                .as_ref()
                .map(|v| {
                    v.prototype
                        .restore()
                        .map(|prototype| crate::FiniteReduction {
                            kind: v.kind,
                            domain: v.domain,
                            prototype,
                        })
                        .map_err(|e| invalid(self.id, e.to_string()))
                })
                .transpose()?,
            validity: self.validity.clone(),
            envelopes: self
                .envelopes
                .iter()
                .map(|v| v.restore(self.id))
                .collect::<Result<_>>()?,
            validity_reads: self.reads.restore(),
            external: self.external.as_ref().map(ExternalWire::restore),
            continuity: self.continuity,
            id: self.id,
            variables: self.variables.clone(),
            arguments: self
                .arguments
                .iter()
                .map(|(n, t)| Ok((n.clone(), t.restore(self.id)?)))
                .collect::<Result<_>>()?,
            result: self.result.restore(self.id)?,
            body: self.body.clone(),
        };
        let roots = crate::expression::inventory::function(&function)
            .iter()
            .map(|root| root.expression)
            .collect::<Vec<_>>();
        let node_counts = crate::expression::inventory::node_counts(&roots);
        for occurrence in function.physical_admissions.keys() {
            match occurrence {
                crate::expression::admission::ExpressionOccurrence::FiniteReduction
                    if function.reduction.is_some() => {}
                crate::expression::admission::ExpressionOccurrence::Node { body, position }
                    if node_counts
                        .get(*body)
                        .is_some_and(|count| *position < *count) => {}
                _ => {
                    return Err(invalid(
                        self.id,
                        "function admission coordinate outside retained inventory",
                    ));
                }
            }
        }
        if function.reduction.is_some()
            && !function
                .physical_admissions
                .contains_key(&crate::expression::admission::ExpressionOccurrence::FiniteReduction)
        {
            return Err(invalid(
                self.id,
                "finite reduction admission coordinate absent",
            ));
        }
        Ok(function)
    }
}

pse_contract::projection_struct! {
/// Opaque complete static value in a caller-qualified immutable scientific recipe.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueRecord {
    projection: { name: "ModelingValueRecord", rust_path: crate::portable::ValueRecord,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    value: ValueWire => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
impl ValueRecord {
    /// Capture the actual selected typed value; no text representation is used.
    pub fn capture(value: &crate::specialize::Value) -> Self {
        Self {
            value: ValueWire::capture(value),
        }
    }
    /// Require exact qualified record membership before reconstructing the typed value.
    pub fn restore(&self, at: DeclarationId) -> Result<crate::specialize::Value> {
        pse_quantity::resolved::receipts::require_record(self)
            .map_err(|error| crate::record_membership_error(at, error))?;
        self.value.restore(at)
    }
    /// Known retained wire extent.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(self.value.retained_bytes())
    }
}
pse_contract::projection_struct! {
/// Opaque full nominal type, including refinements and every indexed/static role.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeRecord {
    projection: { name: "ModelingTypeRecord", rust_path: crate::portable::TypeRecord,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    value: TypeWire => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
impl TypeRecord {
    /// Capture an actual producer-owned complete type.
    pub fn capture(value: &Type) -> Self {
        Self {
            value: TypeWire::capture(value),
        }
    }
    /// Require the qualified record before nested quantity-owner contract restoration.
    pub fn restore(&self, at: DeclarationId) -> Result<Type> {
        pse_quantity::resolved::receipts::require_record(self)
            .map_err(|error| crate::record_membership_error(at, error))?;
        self.value.restore(at)
    }
    /// Known retained wire extent.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(self.value.retained_bytes())
    }
}

#[cfg(test)]
#[path = "portable_projection_tests.rs"]
mod projection_tests;
