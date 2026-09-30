// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Entities as typed records (ADR-0123 Outcome 2).
//!
//! An entity kind is a record schema: typed attributes with defaults, at most one refined
//! kind, and key attributes declared by one kind of its lineage. A refinement may bind an
//! inherited attribute, a function included, for itself and its own refinements. Data are
//! cells, typed once here at admission: `typed` is the only reading of a cell, so no
//! consumer re-reads an attribute value. Declared entities and the rows of keyed kinds
//! become [`Record`]s. A keyed row's identity is framed over the key-declaring kind and its
//! ordered, typed key values, defaults included ([`keyed_identity`]); the concrete
//! refinement is content, so key uniqueness holds across every refinement. Identifier
//! values are opaque, compared byte-exactly and unique within the admitted package closure
//! ([`ModelingIdentifierScope`]).
//!
//! Keyed rows are admitted in phases that do not depend on declaration order (ADR-0123
//! Outcome 3): every row's identity is formed from its key cells first, so a cell may
//! reference any keyed row by its kind and key cells (`kind[keys]`), its own kind's rows
//! included. The references are then resolved against the admitted identities, and the
//! rows a kind's rows reference within that kind form an acyclic lineage.
use crate::specialize::value::{self, Value, conforms};
use crate::{CheckedPackage, DeclarationId, Result, Type, TypeContext, invalid};
use pse_authoring::language::{Cell, CellSelected};
use pse_ids::FramedHasher;
use pse_model::generated::enums::{ModelingDeclarationKind as K, ModelingUncertaintyKind};
use std::collections::{BTreeMap, BTreeSet};

type KeyField = (String, Type, Option<Value>);

/// A cell's uncertainty in its value's canonical unit, or as a fraction for a relative one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Uncertainty {
    /// What the magnitude states.
    pub kind: ModelingUncertaintyKind,
    /// Nonnegative and finite.
    pub magnitude: f64,
}

/// A cell typed at admission.
#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    /// The typed value.
    pub value: Value,
    /// Its declared uncertainty, if any.
    pub uncertainty: Option<Uncertainty>,
}

/// A checked entity kind.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Kind {
    /// Whether this kind itself prohibits records; a refinement declares its own marker.
    pub is_abstract: bool,
    /// The one kind this kind refines.
    pub base: Option<DeclarationId>,
    /// Every attribute and its declaring row, inherited ones first, in declaration order.
    pub attributes: Vec<(String, DeclarationId)>,
    /// Typed defaults by attribute, inherited ones included.
    pub defaults: BTreeMap<String, Typed>,
    /// Values bound for the kind and its refinements: the most-derived binding of each
    /// attribute and the binding row.
    pub bound: BTreeMap<String, (Typed, DeclarationId)>,
    /// The kind that declares the keys; `None` for a kind without keys.
    pub key_kind: Option<DeclarationId>,
    /// The key attributes, in declaration order.
    pub keys: Vec<String>,
    /// Whether the kind or an ancestor carries the provenance facet, so its entities may be
    /// sources (ADR-0123 Outcome 5).
    pub provenance: bool,
    /// Whether the kind or an ancestor carries the release facet: its entities are releases
    /// of the software an oracle's values come from (Plan 23 H6).
    pub release: bool,
    /// Validity envelopes, inherited ones first, each bounded by two attributes of every
    /// entity of the kind (ADR-0123 Outcome 4).
    pub envelopes: Vec<crate::envelope::Envelope>,
    /// Derived attributes, inherited ones first: each name and its declaring row. No entity
    /// supplies, and no kind binds, a derived attribute (Plan 23 D0).
    pub derived: Vec<(String, DeclarationId)>,
    /// Unique attributes, inherited ones first: each name and its declaring row. Its values
    /// are distinct across every entity of the declaring kind (Plan 23 D0).
    pub unique: Vec<(String, DeclarationId)>,
    /// Requirements every entity of the kind satisfies, inherited ones first (Plan 23 D0).
    pub requirements: Vec<DeclarationId>,
}

/// An admitted entity: a declared entity or a row of a keyed kind.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    /// The most-derived kind: content, not identity.
    pub kind: DeclarationId,
    /// The declaration it was admitted from: the entity, or the dataset of a keyed row.
    pub origin: DeclarationId,
    /// Every attribute's typed value.
    pub values: BTreeMap<String, Value>,
    /// The uncertainties that values declare, in canonical units.
    pub uncertainties: BTreeMap<String, Uncertainty>,
}

/// Opaque identifier values, each held by one entity of the admitted package closure.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelingIdentifierScope {
    values: BTreeMap<(DeclarationId, String), DeclarationId>,
}
impl ModelingIdentifierScope {
    /// Conservative owned identifier index storage.
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .values
                .keys()
                .map(|(_, value)| {
                    value.capacity() + size_of::<(DeclarationId, String, DeclarationId)>() + 64
                })
                .sum::<usize>()
    }
    /// The entity holding `value` of `scheme`; values are compared byte-exactly.
    pub fn entity(&self, scheme: DeclarationId, value: &str) -> Option<DeclarationId> {
        self.values.get(&(scheme, value.to_owned())).copied()
    }
    /// Record `value` for `entity`, or return the entity already holding it.
    fn insert(
        &mut self,
        scheme: DeclarationId,
        value: &str,
        entity: DeclarationId,
    ) -> std::result::Result<(), DeclarationId> {
        match self.values.insert((scheme, value.to_owned()), entity) {
            Some(holder) if holder != entity => Err(holder),
            _ => Ok(()),
        }
    }
    /// Every value of `scheme` and the entity holding it.
    pub(crate) fn of(&self, scheme: DeclarationId) -> impl Iterator<Item = (&str, DeclarationId)> {
        self.values
            .range((scheme, String::new())..)
            .take_while(move |((s, _), _)| *s == scheme)
            .map(|((_, value), entity)| (value.as_str(), *entity))
    }
    pub(crate) fn retain(&mut self, keep: impl Fn(DeclarationId) -> bool) {
        self.values.retain(|_, entity| keep(*entity));
    }
}

/// The identity of a keyed row: the key-declaring kind and the ordered, typed key values,
/// defaults included. An entity key is framed by the entity's identity alone, so moving a
/// referenced row between refinements re-keys nothing.
pub fn keyed_identity(key_kind: DeclarationId, keys: &[Value]) -> DeclarationId {
    let mut h = FramedHasher::new(pse_ids::Frame::ModelingKeyedEntityV1);
    h.id(&key_kind.as_id()).u64(keys.len() as u64);
    for key in keys {
        match key {
            Value::Entity { id, .. } => {
                h.str("entity").id(&id.as_id());
            }
            other => other.frame(&mut h),
        }
    }
    DeclarationId::from(h.finish_id())
}

impl CheckedPackage {
    /// The release an oracle's values come from (Plan 23 H6): the oracle entity itself when
    /// its kind carries the release facet, else the first release its attributes reference,
    /// in declaration order.
    pub fn release_of(&self, oracle: DeclarationId) -> Option<DeclarationId> {
        let record = self.record(oracle)?;
        let kind = self.kinds.get(&record.kind)?;
        if kind.release {
            return Some(oracle);
        }
        kind.attributes
            .iter()
            .find_map(|(name, _)| match record.values.get(name) {
                Some(Value::Entity { id, kind })
                    if self.kinds.get(kind).is_some_and(|k| k.release) =>
                {
                    Some(*id)
                }
                _ => None,
            })
    }
    /// Whether `kind` is `ancestor` or one of its refinements.
    pub(crate) fn refines(&self, mut kind: DeclarationId, ancestor: DeclarationId) -> bool {
        for _ in 0..=self.kinds.len() {
            if kind == ancestor {
                return true;
            }
            match self.kinds.get(&kind).and_then(|k| k.base) {
                Some(base) => kind = base,
                None => return false,
            }
        }
        false
    }
    /// Whether a value of type `actual` is admitted where `expected` is: the same type, or
    /// a refinement of an expected entity kind, through sets and optionals.
    pub(crate) fn subsumes(&self, expected: &Type, actual: &Type) -> bool {
        match (expected, actual) {
            (Type::Entity(expected), Type::Entity(actual)) => self.refines(*actual, *expected),
            (Type::Set(expected), Type::Set(actual))
            | (Type::Optional(expected), Type::Optional(actual)) => self.subsumes(expected, actual),
            (Type::Optional(expected), actual) => self.subsumes(expected, actual),
            _ => expected == actual,
        }
    }
    /// The admitted record of an entity: a declared entity or a keyed row.
    pub fn record(&self, entity: DeclarationId) -> Option<&Record> {
        self.entities.get(&entity)
    }
    /// A keyed kind's key-declaring kind and its keys: name, type and typed default.
    pub(crate) fn keys(&self, kind: DeclarationId) -> Option<(DeclarationId, Vec<KeyField>)> {
        let record = self.kinds.get(&kind)?;
        let key_kind = record.key_kind?;
        let keys = record
            .keys
            .iter()
            .map(|name| {
                let declaring = record
                    .attributes
                    .iter()
                    .find(|(attribute, _)| attribute == name)
                    .map(|(_, row)| *row)?;
                Some((
                    name.clone(),
                    self.types.get(&declaring)?.clone(),
                    record.defaults.get(name).map(|typed| typed.value.clone()),
                ))
            })
            .collect::<Option<Vec<_>>>()?;
        Some((key_kind, keys))
    }
    /// The row of `kind` with the complete `keys`, when one is admitted with a concrete
    /// kind refining `kind`.
    pub(crate) fn keyed_row(&self, kind: DeclarationId, keys: &[Value]) -> Option<Value> {
        let (key_kind, _) = self.keys(kind)?;
        let id = keyed_identity(key_kind, keys);
        let record = self.entities.get(&id)?;
        self.refines(record.kind, kind).then_some(Value::Entity {
            id,
            kind: record.kind,
        })
    }
    /// The entity holding `value` of identifier `scheme` within the admitted closure.
    pub fn identified(&self, scheme: DeclarationId, value: &str) -> Option<DeclarationId> {
        self.identifiers.entity(scheme, value)
    }
    fn label(&self, entity: DeclarationId) -> String {
        self.entities
            .get(&entity)
            .map_or_else(|| entity.to_string(), |record| label(self, entity, record))
    }
}
/// An entity's name, or its kind and dataset for a keyed row.
fn label(p: &CheckedPackage, entity: DeclarationId, record: &Record) -> String {
    if entity == record.origin {
        p.declarations[&entity].name.clone()
    } else {
        format!(
            "a {} row of dataset {}",
            p.declarations[&record.kind].name, p.declarations[&record.origin].name
        )
    }
}

/// What a keyed-row reference cell resolves against (Plan 23 KR5).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Rows<'a> {
    /// No keyed row is admitted where the cell is typed: a kind's default or binding.
    Unavailable,
    /// Row identities are being formed: a reference is its identity and named kind,
    /// resolved in the next phase.
    Identity,
    /// Every keyed row's identity and concrete kind.
    Admitted(&'a BTreeMap<DeclarationId, DeclarationId>),
}

/// Type a cell against `ty` at the declaration `at` that holds it (ADR-0123 Outcome 1).
///
/// # Errors
/// A cell of another type, an unresolved path, an entity of another kind, an undeclared
/// member or scheme, absence where a value is required, a keyed row that is not admitted,
/// or an uncertainty on a value that is not numeric.
pub(crate) fn typed(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
    cell: &Cell,
    ty: &Type,
    rows: Rows<'_>,
) -> Result<Typed> {
    let selected = cell
        .value
        .selected()
        .map_err(|e| invalid(at, e.to_string()))?;
    let (value, scale) = match (selected, ty) {
        (CellSelected::Missing, Type::Optional(_)) => (Value::Missing, None),
        (CellSelected::Missing, _) => {
            return Err(invalid(
                at,
                format!("missing value where {ty:?} is required"),
            ));
        }
        (_, Type::Optional(inner)) => return typed(p, c, at, cell, inner, rows),
        (CellSelected::Row(v), Type::Entity(expected)) => {
            (keyed_reference(p, c, at, v, *expected, rows)?, None)
        }
        (CellSelected::Boolean(v), Type::Boolean) => (Value::Boolean(v.value), None),
        (CellSelected::Integer(v), Type::Integer) => (Value::Integer(v.value), Some(1.)),
        (CellSelected::Integer(v), Type::Quantity(_)) => {
            let number = pse_authoring::dsl::Number {
                // Exact to 2^53; a larger integer keeps its exact value beside it.
                value: v.value as f64,
                exact_integer: Some(i128::from(v.value))
                    .filter(|n| n.unsigned_abs() > 9_007_199_254_740_991),
                unit: None,
            };
            let (value, scale) = value::number(c, at, &number, Some(ty))?;
            (value, Some(scale))
        }
        (CellSelected::Quantity(v), Type::Quantity(_)) => {
            let unit =
                pse_authoring::language::unit_product(v).map_err(|e| invalid(at, e.to_string()))?;
            let number = pse_authoring::dsl::Number {
                value: v.magnitude,
                exact_integer: None,
                unit,
            };
            // A refusal names the value as written and the quantity it is not.
            let (value, scale) = value::number(c, at, &number, Some(ty)).map_err(|e| {
                crate::data::context(
                    e,
                    &format!(
                        "{} (actual quantity {}) is not a {}",
                        pse_authoring::language::render_cell(cell).unwrap_or_default(),
                        crate::expression::number_type(&number, c, at, None).map_or_else(
                            |_| "not uniquely determined from the written unit".into(),
                            |actual| quantity_name(c, &actual)
                        ),
                        quantity_name(c, ty)
                    ),
                )
            })?;
            (value, Some(scale))
        }
        (CellSelected::Text(v), Type::Text) => (Value::Text(v.value.clone()), None),
        (CellSelected::Identifier(v), Type::Identifier(scheme)) => {
            let name = v.scheme.join(".");
            if p.resolve(at, &name) != Some(*scheme) {
                return Err(invalid(
                    at,
                    format!(
                        "identifier of scheme {name} where scheme {} is expected",
                        p.declarations[scheme].name
                    ),
                ));
            }
            // Opaque: kept byte for byte, never trimmed, normalized or checked.
            (
                Value::Identifier {
                    scheme: *scheme,
                    value: v.value.clone(),
                },
                None,
            )
        }
        (CellSelected::Reference(v), _) => {
            let value = reference(p, c, at, &v.path, ty)?;
            let scale = matches!(value, Value::Number { .. }).then_some(1.);
            (value, scale)
        }
        (CellSelected::References(v), Type::Set(element)) => {
            let mut values = Vec::new();
            for path in &v.paths {
                let value = reference(p, c, at, &path.path, element)?;
                if values.contains(&value) {
                    return Err(invalid(at, "duplicate member of a set cell"));
                }
                values.push(value);
            }
            (Value::Set(values), None)
        }
        (selected, _) => {
            return Err(invalid(
                at,
                format!("{} cell where {ty:?} is expected", kind_name(selected)),
            ));
        }
    };
    if !conforms(&value, ty, p) {
        return Err(invalid(
            at,
            format!("{} cell does not satisfy {ty:?}", kind_name(selected)),
        ));
    }
    let uncertainty = match (&cell.uncertainty, scale) {
        (None, _) => None,
        (Some(_), None) => {
            return Err(invalid(at, "an uncertainty qualifies a numeric value only"));
        }
        (Some(u), Some(scale)) => {
            if !u.magnitude.is_finite() || u.magnitude < 0. {
                return Err(invalid(at, "an uncertainty is finite and nonnegative"));
            }
            Some(Uncertainty {
                kind: u.kind,
                magnitude: if u.kind == ModelingUncertaintyKind::Relative {
                    u.magnitude
                } else {
                    // A difference converts by the unit's scale alone.
                    u.magnitude * scale.abs()
                },
            })
        }
    };
    Ok(Typed { value, uncertainty })
}

/// A quantity type as a refusal names it: its physical-document name where it has one.
fn quantity_name(c: &TypeContext<'_>, ty: &Type) -> String {
    let Type::Quantity(pse_quantity::scheme::Scheme::Concrete(id)) = ty else {
        return format!("{ty:?}");
    };
    c.quantities
        .quantity_type(*id)
        .ok()
        .and_then(|t| t.name.clone())
        .unwrap_or_else(|| format!("quantity type {}", id.as_id()))
}

fn kind_name(selected: CellSelected<'_>) -> &'static str {
    match selected {
        CellSelected::Boolean(_) => "a Boolean",
        CellSelected::Integer(_) => "an integer",
        CellSelected::Quantity(_) => "a quantity",
        CellSelected::Text(_) => "a text",
        CellSelected::Identifier(_) => "an identifier",
        CellSelected::Reference(_) => "a reference",
        CellSelected::References(_) => "a set",
        CellSelected::Row(_) => "a keyed-row reference",
        CellSelected::Missing => "a missing",
    }
}

/// A keyed-row reference `kind[keys]`: the key-declaring kind's keys in order, trailing
/// keys with a default omitted, as a lookup names them (Plan 23 KR5). Its identity is the
/// row's; its concrete kind must refine both the named kind and the expected one.
fn keyed_reference(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
    reference: &pse_authoring::language::CellRow,
    expected: DeclarationId,
    rows: Rows<'_>,
) -> Result<Value> {
    let path = reference.target.join(".");
    let named = p
        .resolve(at, &path)
        .filter(|id| p.kinds.contains_key(id))
        .ok_or_else(|| invalid(at, format!("{path} is not an entity kind")))?;
    let Some((key_kind, keys)) = p.keys(named) else {
        return Err(invalid(at, format!("kind {path} has no keys")));
    };
    if !p.refines(named, expected) && !p.refines(expected, named) {
        return Err(invalid(
            at,
            format!(
                "a {path} row is not an entity of kind {}",
                p.declarations[&expected].name
            ),
        ));
    }
    if reference.keys.len() > keys.len()
        || keys[reference.keys.len()..].iter().any(|k| k.2.is_none())
    {
        return Err(invalid(
            at,
            format!(
                "a {path} row is referenced by its keys {:?}",
                keys.iter().map(|k| &k.0).collect::<Vec<_>>()
            ),
        ));
    }
    let mut values = Vec::with_capacity(keys.len());
    for (index, (_, ty, default)) in keys.iter().enumerate() {
        values.push(match reference.keys.get(index) {
            Some(key) => {
                let cell = pse_authoring::language::key_cell_value(key)
                    .map_err(|e| invalid(at, e.to_string()))?;
                typed(p, c, at, &cell, ty, rows)?.value
            }
            None => default.clone().ok_or_else(|| invalid(at, "key default"))?,
        });
    }
    let id = keyed_identity(key_kind, &values);
    let kind = match rows {
        Rows::Unavailable => {
            return Err(invalid(
                at,
                format!(
                    "a keyed-row reference to {path} is resolved against admitted rows; a kind's default or binding names none"
                ),
            ));
        }
        Rows::Identity => named,
        Rows::Admitted(index) => {
            let kind = *index.get(&id).ok_or_else(|| {
                invalid(
                    at,
                    format!(
                        "no {path} row has the keys [{}]",
                        values
                            .iter()
                            .map(|v| crate::data::display(p, v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
            })?;
            if !p.refines(kind, named) || !p.refines(kind, expected) {
                return Err(invalid(
                    at,
                    format!(
                        "the {path} row [{}] is of kind {}, not an entity of kind {}",
                        values
                            .iter()
                            .map(|v| crate::data::display(p, v))
                            .collect::<Vec<_>>()
                            .join(", "),
                        p.declarations[&kind].name,
                        p.declarations[&expected].name
                    ),
                ));
            }
            kind
        }
    };
    Ok(Value::Entity { id, kind })
}

/// A reference cell's path, resolved once against the expected type.
fn reference(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
    path: &[String],
    ty: &Type,
) -> Result<Value> {
    let name = path.join(".");
    match ty {
        Type::Quantity(_) => {
            let (attribute, owner) = path
                .split_last()
                .ok_or_else(|| invalid(at, "empty quantity path"))?;
            let Some(pse_quantity::PhysicalName::ReferenceState(state)) =
                p.physical_name(at, &owner.join("."))
            else {
                return Err(invalid(
                    at,
                    format!("{name} is not a named reference-state condition"),
                ));
            };
            value::Evaluator {
                package: p,
                physical: c,
                at,
                env: &BTreeMap::new(),
                limit: crate::data::EVALUATION_LIMIT,
                stack: Vec::new(),
                reader: crate::provenance::Reader::Admission(None),
            }
            .reference_condition(state, attribute)
        }
        // A member by name, bare or qualified by its enumeration.
        Type::Enum(enumeration) => {
            let (member, owner) = path
                .split_last()
                .ok_or_else(|| invalid(at, "empty member path"))?;
            if !owner.is_empty() && p.resolve(at, &owner.join(".")) != Some(*enumeration) {
                return Err(invalid(
                    at,
                    format!(
                        "{name} is not a member of {}",
                        p.declarations[enumeration].name
                    ),
                ));
            }
            enum_member(p, *enumeration, member, at)
        }
        Type::Entity(kind) => {
            let id = p
                .resolve(at, &name)
                .ok_or_else(|| invalid(at, format!("unknown entity {name}")))?;
            let Some(Type::Entity(actual)) = p.types.get(&id) else {
                return Err(invalid(at, format!("{name} is not an entity")));
            };
            if p.declarations[&id].value.kind != K::Entity || !p.refines(*actual, *kind) {
                return Err(invalid(
                    at,
                    format!(
                        "{name} is not an entity of kind {}",
                        p.declarations[kind].name
                    ),
                ));
            }
            Ok(Value::Entity { id, kind: *actual })
        }
        Type::QuantityType | Type::ReferenceState => {
            let value = match p.physical_name(at, &name) {
                Some(pse_quantity::PhysicalName::QuantityType(id)) => Value::QuantityType(id),
                Some(pse_quantity::PhysicalName::ReferenceState(id)) => Value::ReferenceState(id),
                None => return Err(invalid(at, format!("unknown physical name {name}"))),
            };
            if !conforms(&value, ty, p) {
                return Err(invalid(at, format!("{name} is not a {ty:?}")));
            }
            Ok(value)
        }
        Type::Function { .. } | Type::Definition(_) | Type::Interface(_) => {
            let id = p
                .resolve(at, &name)
                .ok_or_else(|| invalid(at, format!("unknown declaration {name}")))?;
            let value = if p.functions.contains_key(&id) {
                Value::Function(id)
            } else {
                Value::Definition {
                    id,
                    bindings: BTreeMap::new(),
                }
            };
            if !conforms(&value, ty, p) {
                return Err(invalid(at, format!("{name} does not satisfy {ty:?}")));
            }
            Ok(value)
        }
        _ => Err(invalid(
            at,
            format!("a reference cell where {ty:?} is expected"),
        )),
    }
}

/// The member of `enumeration` named `member`.
pub(crate) fn enum_member(
    p: &CheckedPackage,
    enumeration: DeclarationId,
    member: &str,
    at: DeclarationId,
) -> Result<Value> {
    p.declarations[&enumeration]
        .value
        .enumeration
        .as_ref()
        .and_then(|e| e.members.iter().find(|m| m.name == member))
        .map(|m| Value::Enum {
            enumeration,
            member: m.member_id,
        })
        .ok_or_else(|| {
            invalid(
                at,
                format!(
                    "{member} is not a member of {}",
                    p.declarations[&enumeration].name
                ),
            )
        })
}

/// A key is a reference, an enumeration member, an integer, text, a Boolean, an
/// identifier or a quantity: a value with a byte-exact framing.
fn key_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Entity(_)
            | Type::Enum(_)
            | Type::Integer
            | Type::Text
            | Type::Boolean
            | Type::Identifier(_)
            | Type::Quantity(_)
    )
}

/// Admit every kind, constant, declared entity and keyed row (ADR-0123 Outcomes 2–3).
///
/// Phase 1 forms every keyed row's identity from its key cells; phase 2 types every
/// constant, entity attribute and row value against those identities, so references need
/// no declaration order; the rows each kind's rows reference within their own kind must
/// then be acyclic.
pub(crate) fn admit(
    p: &mut CheckedPackage,
    c: &TypeContext<'_>,
    documents: &dyn crate::document::Documents,
) -> Result<()> {
    // Refinements copy their inherited schema, defaults and bindings. Charge the
    // declaration payload for every inheriting owner before constructing those copies.
    for id in p.kinds.keys().copied() {
        let mut cursor = Some(id);
        let mut bytes = size_of::<Kind>();
        while let Some(owner) = cursor {
            for child in p.children.get(&owner).into_iter().flatten() {
                bytes = bytes
                    .saturating_add(
                        pse_model::HeapUsage::owned_bytes(&p.declarations[child]).saturating_mul(4),
                    )
                    .saturating_add(4 * (size_of::<Typed>() + size_of::<Type>() + 64));
            }
            cursor = p.kinds[&owner].base;
        }
        documents.preflight(id, bytes)?;
    }
    admit_kinds(p, c)?;
    // Budget expanded records, keys, identifier indexes and phase-local copies before
    // admitting the first row. Default payload is multiplied by row count, not source size.
    for row in p.declarations.values() {
        let (kind, count, payload) = if row.value.entity.is_some() {
            let Some(Type::Entity(kind)) = p.types.get(&row.declaration_id) else {
                continue;
            };
            (*kind, 1, 0)
        } else if let Some(data) = &row.value.dataset {
            let Some(kind) = p
                .resolve(row.declaration_id, &data.target)
                .filter(|kind| p.kinds.contains_key(kind))
            else {
                continue;
            };
            if let Some(path) = &data.document {
                let document = documents.resolve(row.document_id, path).ok_or_else(|| {
                    invalid(row.declaration_id, "package data document is not supplied")
                })?;
                (
                    kind,
                    documents.rows(document).unwrap_or(0),
                    documents.bytes(document),
                )
            } else {
                (kind, data.rows.len(), 0)
            }
        } else {
            continue;
        };
        let schema = &p.kinds[&kind];
        let per_record = size_of::<Record>()
            + 6 * (size_of::<DeclarationId>() + 64)
            + schema
                .attributes
                .iter()
                .map(|(name, _)| size_of::<(String, Value)>() + name.capacity() + 64)
                .sum::<usize>()
            + schema
                .defaults
                .values()
                .map(|value| value.value.retained_bytes())
                .sum::<usize>()
            + schema
                .bound
                .values()
                .map(|(value, _)| value.value.retained_bytes())
                .sum::<usize>();
        documents.preflight(
            row.declaration_id,
            count
                .saturating_mul(per_record)
                .saturating_mul(4)
                .saturating_add(payload.saturating_mul(4)),
        )?;
    }
    // Identifier-bearing declared entities precede document key resolution. Their row
    // references are revalidated against the complete identity index below.
    let declared_identifiers = p
        .declarations
        .values()
        .filter_map(|row| {
            row.value
                .entity
                .as_ref()
                .map(|entity| (row.declaration_id, entity))
        })
        .map(|(id, entity)| Ok((id, declared(p, c, id, entity, Rows::Identity)?)))
        .collect::<Result<Vec<_>>>()?;
    for (id, record) in &declared_identifiers {
        register_identifiers(p, *id, record)?;
    }
    let datasets = p
        .declarations
        .values()
        .filter_map(|row| {
            let dataset = row.value.dataset.as_ref()?;
            let kind = p
                .resolve(row.declaration_id, &dataset.target)
                .filter(|id| p.kinds.contains_key(id))?;
            Some((row.declaration_id, kind, dataset))
        })
        .collect::<Vec<_>>();
    // Phase 1: identities from keys.
    let mut index = BTreeMap::<DeclarationId, (DeclarationId, DeclarationId)>::new();
    let mut document_keys = BTreeMap::new();
    for (dataset, kind, data) in &datasets {
        let ids = if let Some(path) = &data.document {
            let known = index.iter().map(|(id, (kind, _))| (*id, *kind)).collect();
            let keys = document_rows(p, c, *dataset, *kind, data, path, documents, &known, true)?;
            let layout = layout(p, c, *dataset, *kind, data, Rows::Identity)?;
            let ids = keys
                .keys
                .iter()
                .map(|keys| {
                    complete_document_keys(&layout, keys)
                        .map(|keys| keyed_identity(layout.key_kind, &keys))
                })
                .collect::<Result<Vec<_>>>()?;
            document_keys.insert(*dataset, keys);
            ids
        } else {
            identities(p, c, *dataset, *kind, data)?
        };
        for id in ids {
            if let Some((previous_kind, previous)) = index.insert(id, (*kind, *dataset)) {
                return Err(invalid(
                    *dataset,
                    format!(
                        "the key of this {} row is already admitted: {} (kind {}) and {} (kind {}) supply the same key of kind {}",
                        p.declarations[kind].name,
                        p.declarations[&previous].name,
                        p.declarations[&previous_kind].name,
                        p.declarations[dataset].name,
                        p.declarations[kind].name,
                        p.kinds[kind]
                            .key_kind
                            .map_or_else(String::new, |k| p.declarations[&k].name.clone()),
                    ),
                ));
            }
        }
    }
    let admitted = index
        .iter()
        .map(|(id, (kind, _))| (*id, *kind))
        .collect::<BTreeMap<_, _>>();
    let resolved = Rows::Admitted(&admitted);
    // Phase 2: values typed against the admitted identities.
    let constants = p
        .declarations
        .values()
        .filter_map(|row| row.value.constant.as_ref().map(|v| (row.declaration_id, v)))
        .map(|(id, constant)| {
            let ty = p
                .types
                .get(&id)
                .ok_or_else(|| invalid(id, "constant type absent"))?;
            Ok((id, typed(p, c, id, &constant.value, ty, resolved)?))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    p.constants = constants;
    let mut records = Vec::new();
    for row in p.declarations.values() {
        if let Some(entity) = &row.value.entity {
            records.push((
                row.declaration_id,
                declared(p, c, row.declaration_id, entity, resolved)?,
            ));
        }
    }
    for (dataset, kind, data) in &datasets {
        if let Some(path) = &data.document {
            let document = document_rows(
                p, c, *dataset, *kind, data, path, documents, &admitted, false,
            )?;
            let layout = layout(p, c, *dataset, *kind, data, resolved)?;
            let identities = |table: &crate::document::DocumentTable| {
                table
                    .keys
                    .iter()
                    .map(|keys| {
                        complete_document_keys(&layout, keys)
                            .map(|keys| keyed_identity(layout.key_kind, &keys))
                    })
                    .collect::<Result<Vec<_>>>()
            };
            if identities(&document)? != identities(&document_keys[dataset])? {
                return Err(invalid(
                    *dataset,
                    "document key identities changed between identity and value admission",
                ));
            }
            records.extend(document_records(
                p, c, *dataset, *kind, data, &document, resolved,
            )?);
        } else {
            records.extend(rows(p, c, *dataset, *kind, data, resolved)?);
        }
    }
    for (id, record) in records {
        register_identifiers(p, id, &record)?;
        p.entities.insert(id, record);
    }
    // ADR-0123 Outcome 4: every entity orders the bounds of its kind's envelopes.
    for (id, record) in &p.entities {
        for envelope in &p.kinds[&record.kind].envelopes {
            let bound = |name: &str| record.values.get(name).cloned().unwrap_or(Value::Missing);
            crate::envelope::ordered(
                envelope,
                &bound(&envelope.lower),
                &bound(&envelope.upper),
                record.origin,
                || {
                    crate::data::display(
                        p,
                        &Value::Entity {
                            id: *id,
                            kind: record.kind,
                        },
                    )
                },
            )?;
        }
    }
    lineage(p)
}

fn register_identifiers(p: &mut CheckedPackage, id: DeclarationId, record: &Record) -> Result<()> {
    for value in record.values.values() {
        if let Value::Identifier { scheme, value } = value {
            if p.identifiers.entity(*scheme, value) == Some(id) {
                continue;
            }
            if let Err(holder) = p.identifiers.insert(*scheme, value, id) {
                let holder = p
                    .declarations
                    .get(&holder)
                    .map_or_else(|| p.label(holder), |d| d.name.clone());
                return Err(invalid(
                    record.origin,
                    format!(
                        "identifier {} \"{value}\" is held by {} and by {}; identifier values are unique within the package closure",
                        p.declarations[scheme].name,
                        holder,
                        label(p, id, record)
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// The rows of each key-declaring kind that its rows reference form an acyclic lineage: a
/// fitted or derived row may name the row it came from, never, through others, itself
/// (ADR-0123 Outcome 3). A cycle is refused with its rows named.
fn lineage(p: &CheckedPackage) -> Result<()> {
    let family = |id: &DeclarationId| {
        let record = p.entities.get(id)?;
        (record.origin != *id).then_some(())?;
        p.kinds.get(&record.kind)?.key_kind
    };
    let mut graph = petgraph::graph::DiGraph::<DeclarationId, ()>::new();
    let nodes = p
        .entities
        .keys()
        .filter(|id| family(id).is_some())
        .map(|id| (*id, graph.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    fn references(value: &Value, out: &mut Vec<DeclarationId>) {
        match value {
            Value::Entity { id, .. } => out.push(*id),
            Value::Set(values) | Value::Tuple(values) => {
                for v in values {
                    references(v, out);
                }
            }
            _ => {}
        }
    }
    for (id, node) in &nodes {
        let mut targets = Vec::new();
        for value in p.entities[id].values.values() {
            references(value, &mut targets);
        }
        for target in targets {
            if let Some(to) = nodes.get(&target).filter(|_| family(&target) == family(id)) {
                graph.add_edge(*to, *node, ());
            }
        }
    }
    if let Some(cycle) = petgraph::algo::kosaraju_scc(&graph)
        .into_iter()
        .find(|c| c.len() > 1 || c.first().is_some_and(|n| graph.contains_edge(*n, *n)))
    {
        let mut rows = cycle
            .iter()
            .map(|n| {
                let id = graph[*n];
                crate::data::display(
                    p,
                    &Value::Entity {
                        id,
                        kind: p.entities[&id].kind,
                    },
                )
            })
            .collect::<Vec<_>>();
        rows.sort();
        return Err(invalid(
            p.entities[&graph[cycle[0]]].origin,
            format!("rows reference one another in a cycle: {}", rows.join(", ")),
        ));
    }
    Ok(())
}

/// Check each kind after the kind it refines: attributes, keys, defaults and bindings.
fn admit_kinds(p: &mut CheckedPackage, c: &TypeContext<'_>) -> Result<()> {
    let depth = |p: &CheckedPackage, mut kind: DeclarationId| {
        let mut depth = 0_usize;
        while let Some(base) = p.kinds.get(&kind).and_then(|k| k.base) {
            kind = base;
            depth += 1;
        }
        depth
    };
    let mut order = p.kinds.keys().copied().collect::<Vec<_>>();
    order.sort_by_key(|kind| (depth(p, *kind), *kind));
    for id in order {
        let base = p.kinds[&id].base;
        let mut kind = base.map(|b| p.kinds[&b].clone()).unwrap_or_default();
        kind.base = base;
        // ADR-0123 Outcome 5: a refinement inherits the provenance facet, and a release is a
        // source (Plan 23 H6).
        let facet = |facet| {
            p.declarations[&id]
                .value
                .scope
                .as_ref()
                .is_some_and(|scope| scope.facets.contains(&facet))
        };
        kind.provenance |= facet(pse_model::generated::enums::ModelingKindFacet::Provenance);
        kind.release |= facet(pse_model::generated::enums::ModelingKindFacet::Release);
        kind.is_abstract = facet(pse_model::generated::enums::ModelingKindFacet::Abstract);
        if kind.release && !kind.provenance {
            return Err(invalid(
                id,
                format!(
                    "kind {} declares releases, which are sources: it or a kind it refines carries the provenance facet",
                    p.declarations[&id].name
                ),
            ));
        }
        let mut own_keys = Vec::new();
        let mut envelopes = Vec::new();
        for child in p.children.get(&id).cloned().unwrap_or_default() {
            let row = &p.declarations[&child];
            if row.value.envelope.is_some() {
                envelopes.push(child);
                continue;
            }
            // Plan 23 D0: a requirement every entity of the kind and its refinements satisfies.
            if row.value.requirement.is_some() {
                kind.requirements.push(child);
                continue;
            }
            let Some(attribute) = &row.value.attribute else {
                return Err(invalid(
                    child,
                    "an entity kind declares attributes, envelopes and requirements and binds inherited attributes only",
                ));
            };
            let name = row.name.clone();
            let inherited = kind
                .attributes
                .iter()
                .find(|(attribute, _)| *attribute == name)
                .map(|(_, row)| *row);
            match (&attribute.r#type, attribute.key) {
                (Some(_), key) => {
                    if let Some(owner) = inherited.and_then(|row| p.declarations[&row].parent_id) {
                        return Err(invalid(
                            child,
                            format!(
                                "attribute {name} is already declared by kind {}; a refinement binds it",
                                p.declarations[&owner].name
                            ),
                        ));
                    }
                    if row.is_override {
                        return Err(invalid(child, "override binds an inherited attribute"));
                    }
                    let ty = p
                        .types
                        .get(&child)
                        .cloned()
                        .ok_or_else(|| invalid(child, "attribute type absent"))?;
                    if attribute.storage.len() > 1
                        || attribute.storage.iter().any(|s| s.name != name)
                    {
                        return Err(invalid(
                            child,
                            "an attribute declares its storage once, under its own name",
                        ));
                    }
                    if attribute.derived.is_some() && !attribute.storage.is_empty() {
                        return Err(invalid(
                            child,
                            "a derived attribute is evaluated at admission and declares no document storage",
                        ));
                    }
                    crate::data::storage(p, c, child, &name, &ty, attribute.storage.first())?;
                    if key {
                        if !key_type(&ty) {
                            return Err(invalid(
                                child,
                                "a key is a reference, an enumeration member, an integer, text, a Boolean, an identifier or a quantity",
                            ));
                        }
                        own_keys.push(name.clone());
                    }
                    // Plan 23 D0: a unique attribute compares values as a key does.
                    if attribute.unique {
                        let mut inner = &ty;
                        while let Type::Optional(value) = inner {
                            inner = value;
                        }
                        if !key_type(inner) {
                            return Err(invalid(
                                child,
                                format!(
                                    "unique attribute {name} is a reference, an enumeration member, an integer, text, a Boolean, an identifier or a quantity"
                                ),
                            ));
                        }
                        kind.unique.push((name.clone(), child));
                    }
                    // Plan 23 D0: a derived attribute carries its expression and no default.
                    if attribute.derived.is_some() {
                        if key || attribute.value.is_some() {
                            return Err(invalid(
                                child,
                                format!("derived attribute {name} is neither a key nor defaulted"),
                            ));
                        }
                        kind.derived.push((name.clone(), child));
                    }
                    if let Some(cell) = &attribute.value {
                        kind.defaults.insert(
                            name.clone(),
                            typed(p, c, child, cell, &ty, Rows::Unavailable)?,
                        );
                    }
                    kind.attributes.push((name, child));
                }
                (None, false) => {
                    let Some(declaring) = inherited else {
                        return Err(invalid(
                            child,
                            format!("{name} binds no attribute inherited by this kind"),
                        ));
                    };
                    if kind.keys.contains(&name) {
                        return Err(invalid(
                            child,
                            format!("key {name} is supplied by each row, not bound by a kind"),
                        ));
                    }
                    if let Some((_, derived)) = kind.derived.iter().find(|(n, _)| *n == name) {
                        let owner = p.declarations[derived].parent_id.unwrap_or(*derived);
                        return Err(invalid(
                            child,
                            format!(
                                "attribute {name} is derived by kind {}; a kind binds none",
                                p.declarations[&owner].name
                            ),
                        ));
                    }
                    if attribute.unique
                        || attribute.derived.is_some()
                        || !attribute.storage.is_empty()
                    {
                        return Err(invalid(
                            child,
                            "a binding declares neither uniqueness nor a derivation",
                        ));
                    }
                    if let Some((_, previous)) = kind.bound.get(&name)
                        && !row.is_override
                    {
                        let owner = p.declarations[previous].parent_id.unwrap_or(*previous);
                        return Err(invalid(
                            child,
                            format!(
                                "rebinding {name}, bound by kind {}, requires override",
                                p.declarations[&owner].name
                            ),
                        ));
                    }
                    let ty = p.types[&declaring].clone();
                    let cell = attribute
                        .value
                        .as_ref()
                        .ok_or_else(|| invalid(child, "a binding carries its value"))?;
                    let value = bound(p, c, child, id, cell, &ty)?;
                    kind.bound.insert(name, (value, child));
                }
                (None, true) => return Err(invalid(child, "a key declares its type")),
            }
        }
        if !own_keys.is_empty() {
            if let Some(owner) = kind.key_kind {
                return Err(invalid(
                    id,
                    format!(
                        "the keys of a refinement lineage are declared by one kind, here {}",
                        p.declarations[&owner].name
                    ),
                ));
            }
            kind.key_kind = Some(id);
            kind.keys = own_keys;
        }
        // ADR-0123 Outcome 4: an envelope is data, bounded by two of the kind's attributes,
        // declared or inherited; a refinement inherits its kind's envelopes.
        let what = format!("kind {}", p.declarations[&id].name);
        for child in envelopes {
            let row = &p.declarations[&child];
            let declared = row
                .value
                .envelope
                .as_ref()
                .ok_or_else(|| invalid(child, "envelope payload"))?;
            if let Some(previous) = kind.envelopes.iter().find(|e| e.axis == row.name) {
                let owner = p.declarations[&previous.owner]
                    .parent_id
                    .unwrap_or(previous.owner);
                return Err(invalid(
                    child,
                    format!(
                        "{what} redeclares envelope {}, declared by kind {}",
                        row.name, p.declarations[&owner].name
                    ),
                ));
            }
            let ty = c.resolve(
                &declared.r#type,
                &BTreeSet::new(),
                &p.named_types(child),
                child,
            )?;
            let envelope = crate::envelope::resolve(
                c,
                child,
                &what,
                &row.name,
                ty,
                &declared.lower,
                &declared.upper,
                |name| {
                    kind.attributes
                        .iter()
                        .find(|(attribute, _)| attribute == name)
                        .and_then(|(_, declaration)| p.types.get(declaration).cloned())
                },
            )?;
            kind.envelopes.push(envelope);
        }
        // Envelope bounds are ordered as each entity is admitted, before any derivation.
        if let Some((name, row)) = kind.derived.iter().find(|(name, _)| {
            kind.envelopes
                .iter()
                .any(|e| e.lower == *name || e.upper == *name)
        }) {
            return Err(invalid(
                *row,
                format!("derived attribute {name} bounds no envelope"),
            ));
        }
        p.kinds.insert(id, kind);
    }
    Ok(())
}

/// A kind-level binding. A function bound to a function-typed attribute may take, where
/// the attribute takes an entity of an ancestor kind, an entity of any kind between that
/// ancestor and the binding kind: a call through a static reference passes an entity of
/// the binding kind or a refinement, and specialization checks the actual value.
fn bound(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
    kind: DeclarationId,
    cell: &Cell,
    ty: &Type,
) -> Result<Typed> {
    if let Type::Function { arguments, result } = ty
        && let Ok(CellSelected::Reference(reference)) = cell.value.selected()
        && cell.uncertainty.is_none()
    {
        let name = reference.path.join(".");
        let id = p
            .resolve(at, &name)
            .filter(|id| p.functions.contains_key(id))
            .ok_or_else(|| invalid(at, format!("{name} is not a function")))?;
        let function = &p.functions[&id];
        let admitted = function.variables.is_empty()
            && function.result == **result
            && function.arguments.len() == arguments.len()
            && function
                .arguments
                .iter()
                .zip(arguments)
                .all(|((name, actual), (declared, expected))| {
                    name == declared
                        && (actual == expected
                            || matches!((actual, expected), (Type::Entity(actual), Type::Entity(expected))
                                if p.refines(*actual, *expected) && p.refines(kind, *actual)))
                });
        if !admitted {
            return Err(invalid(
                at,
                format!("function {name} does not have the bound attribute's signature {ty:?}"),
            ));
        }
        return Ok(Typed {
            value: Value::Function(id),
            uncertainty: None,
        });
    }
    typed(p, c, at, cell, ty, Rows::Unavailable)
}

/// A declared entity's record: supplied values, then kind bindings, defaults and absence.
fn declared(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    id: DeclarationId,
    entity: &pse_authoring::language::AuthoredModelingDeclarationsFieldValueEntity,
    rows: Rows<'_>,
) -> Result<Record> {
    let Some(Type::Entity(kind)) = p.types.get(&id).cloned() else {
        return Err(invalid(id, "entity requires a declared kind"));
    };
    let schema = &p.kinds[&kind];
    if schema.key_kind.is_some() {
        return Err(invalid(
            id,
            format!(
                "rows of keyed kind {} come from datasets",
                p.declarations[&kind].name
            ),
        ));
    }
    let mut supplied = BTreeMap::new();
    for attribute in &entity.attributes {
        let declaring = attribute_row(p, schema, kind, &attribute.name, id)?;
        if supplied.contains_key(&attribute.name) {
            return Err(invalid(
                id,
                format!("duplicate entity attribute {}", attribute.name),
            ));
        }
        let value = typed(p, c, id, &attribute.value, &p.types[&declaring], rows)
            .map_err(|e| crate::data::context(e, &format!("attribute {}", attribute.name)))?;
        supplied.insert(attribute.name.clone(), value);
    }
    record(p, kind, id, supplied)
}

/// The declaring row of an attribute a row or entity may supply: declared by the kind or
/// an ancestor, and not bound by a kind.
fn attribute_row(
    p: &CheckedPackage,
    schema: &Kind,
    kind: DeclarationId,
    name: &str,
    at: DeclarationId,
) -> Result<DeclarationId> {
    let declaring = schema
        .attributes
        .iter()
        .find(|(attribute, _)| attribute == name)
        .map(|(_, row)| *row)
        .ok_or_else(|| {
            invalid(
                at,
                format!(
                    "unknown attribute {name} of kind {}",
                    p.declarations[&kind].name
                ),
            )
        })?;
    if let Some((_, binding)) = schema.bound.get(name) {
        let owner = p.declarations[binding].parent_id.unwrap_or(*binding);
        return Err(invalid(
            at,
            format!(
                "attribute {name} is bound by kind {}",
                p.declarations[&owner].name
            ),
        ));
    }
    if schema.derived.iter().any(|(derived, _)| derived == name) {
        let owner = p.declarations[&declaring].parent_id.unwrap_or(declaring);
        return Err(invalid(
            at,
            format!(
                "attribute {name} is derived by kind {}; no entity supplies it",
                p.declarations[&owner].name
            ),
        ));
    }
    Ok(declaring)
}

/// A complete record: each attribute's supplied value, else its kind binding, else its
/// default, else absence where the attribute is optional.
fn record(
    p: &CheckedPackage,
    kind: DeclarationId,
    origin: DeclarationId,
    mut supplied: BTreeMap<String, Typed>,
) -> Result<Record> {
    let schema = &p.kinds[&kind];
    let mut values = BTreeMap::new();
    if schema.is_abstract {
        return Err(invalid(
            origin,
            format!(
                "abstract kind {} cannot supply an entity or keyed row; instantiate a concrete refinement",
                p.declarations[&kind].name
            ),
        ));
    }
    let mut uncertainties = BTreeMap::new();
    for (name, declaring) in &schema.attributes {
        // A derived attribute's value is evaluated once every table is admitted.
        if schema.derived.iter().any(|(derived, _)| derived == name) {
            continue;
        }
        let typed = if let Some(value) = supplied.remove(name) {
            value
        } else if let Some((value, _)) = schema.bound.get(name) {
            value.clone()
        } else if let Some(value) = schema.defaults.get(name) {
            value.clone()
        } else if matches!(p.types.get(declaring), Some(Type::Optional(_))) {
            Typed {
                value: Value::Missing,
                uncertainty: None,
            }
        } else {
            return Err(invalid(
                origin,
                format!(
                    "missing attribute {name} of kind {}",
                    p.declarations[&kind].name
                ),
            ));
        };
        if let Some(uncertainty) = typed.uncertainty {
            uncertainties.insert(name.clone(), uncertainty);
        }
        values.insert(name.clone(), typed.value);
    }
    Ok(Record {
        kind,
        origin,
        values,
        uncertainties,
    })
}

/// The key and value layout of a dataset whose target is a keyed kind. Row keys are the
/// keys the dataset does not bind, in declaration order; row values are the kind's other
/// unbound attributes, in lineage order. Trailing keys and values with a default, and
/// trailing optional values, may be omitted.
struct Layout<'a> {
    key_kind: DeclarationId,
    keys: Vec<(String, Type, Option<Value>)>,
    bound: BTreeMap<String, Typed>,
    values: Vec<&'a (String, DeclarationId)>,
}
impl Layout<'_> {
    fn unbound(&self) -> impl Iterator<Item = &(String, Type, Option<Value>)> {
        self.keys
            .iter()
            .filter(|(name, _, _)| !self.bound.contains_key(name))
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "identity and value phases share the document adapter"
)]
fn document_rows(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    dataset: DeclarationId,
    kind: DeclarationId,
    data: &pse_authoring::language::AuthoredModelingDeclarationsFieldValueDataset,
    path: &str,
    documents: &dyn crate::document::Documents,
    identities: &BTreeMap<DeclarationId, DeclarationId>,
    keys_only: bool,
) -> Result<std::sync::Arc<crate::document::DocumentTable>> {
    if !data.rows.is_empty() {
        return Err(invalid(
            dataset,
            "a dataset supplies inline rows or a document, not both",
        ));
    }
    let schema = &p.kinds[&kind];
    documents.preflight(
        dataset,
        schema.attributes.len().saturating_mul(
            p.entities
                .len()
                .saturating_add(identities.len())
                .saturating_mul(2 * size_of::<DeclarationId>() + 64)
                .saturating_add(p.identifiers.retained_bytes()),
        ),
    )?;
    let layout = layout(
        p,
        c,
        dataset,
        kind,
        data,
        if keys_only {
            Rows::Identity
        } else {
            Rows::Admitted(identities)
        },
    )?;
    let label = format!(
        "dataset {} of kind {}",
        p.declarations[&dataset].name, p.declarations[&kind].name
    );
    let slot = |name: &str, declaring: DeclarationId| {
        let attribute = p.declarations[&declaring]
            .value
            .attribute
            .as_ref()
            .ok_or_else(|| invalid(dataset, "attribute declaration absent"))?;
        let storage = crate::data::storage(
            p,
            c,
            declaring,
            name,
            &p.types[&declaring],
            attribute.storage.first(),
        )?;
        let mut slot = crate::data::document_slot(
            p,
            dataset,
            &label,
            name,
            &p.types[&declaring],
            storage,
            Some(identities),
        )?;
        slot.default = p.kinds[&kind]
            .defaults
            .get(name)
            .map(|value| value.value.clone())
            .or_else(|| slot.optional.then_some(Value::Missing));
        if let crate::document::Target::Entity { deferred, .. } = &mut slot.target {
            *deferred = keys_only;
        }
        Ok(slot)
    };
    let keys = layout
        .unbound()
        .map(|(name, _, _)| {
            let declaring = p.kinds[&kind]
                .attributes
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, id)| *id)
                .ok_or_else(|| invalid(dataset, "key declaration absent"))?;
            slot(name, declaring)
        })
        .collect::<Result<Vec<_>>>()?;
    let values = if keys_only {
        Vec::new()
    } else {
        layout
            .values
            .iter()
            .map(|(name, declaring)| slot(name, *declaring))
            .collect::<Result<Vec<_>>>()?
    };
    let plan = std::sync::Arc::new(crate::document::DocumentPlan {
        dataset,
        label,
        keys,
        values,
        keys_only,
    });
    let document = documents
        .resolve(p.declarations[&dataset].document_id, path)
        .ok_or_else(|| {
            invalid(
                dataset,
                format!("package data document {path} is not supplied"),
            )
        })?;
    documents.admit(&plan, document)
}

fn complete_document_keys(layout: &Layout<'_>, supplied: &[Value]) -> Result<Vec<Value>> {
    let mut supplied = supplied.iter();
    layout
        .keys
        .iter()
        .map(|(name, _, _)| {
            layout
                .bound
                .get(name)
                .map(|t| t.value.clone())
                .or_else(|| supplied.next().cloned())
                .ok_or_else(|| invalid(layout.key_kind, format!("document omits key {name}")))
        })
        .collect()
}

fn document_records(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    dataset: DeclarationId,
    kind: DeclarationId,
    data: &pse_authoring::language::AuthoredModelingDeclarationsFieldValueDataset,
    document: &crate::document::DocumentTable,
    rows: Rows<'_>,
) -> Result<Vec<(DeclarationId, Record)>> {
    let layout = layout(p, c, dataset, kind, data, rows)?;
    document
        .keys
        .iter()
        .zip(&document.values)
        .map(|(keys, values)| {
            let keys = complete_document_keys(&layout, keys)?;
            let supplied = layout
                .keys
                .iter()
                .map(|(name, _, _)| name)
                .zip(&keys)
                .chain(layout.values.iter().map(|(name, _)| name).zip(values))
                .map(|(name, value)| {
                    (
                        name.clone(),
                        Typed {
                            value: value.clone(),
                            uncertainty: None,
                        },
                    )
                })
                .collect();
            Ok((
                keyed_identity(layout.key_kind, &keys),
                record(p, kind, dataset, supplied)?,
            ))
        })
        .collect()
}
fn layout<'a>(
    p: &'a CheckedPackage,
    c: &TypeContext<'_>,
    dataset: DeclarationId,
    kind: DeclarationId,
    data: &pse_authoring::language::AuthoredModelingDeclarationsFieldValueDataset,
    rows: Rows<'_>,
) -> Result<Layout<'a>> {
    let schema = &p.kinds[&kind];
    if schema.is_abstract {
        return Err(invalid(
            dataset,
            format!(
                "abstract kind {} cannot supply an entity or keyed row",
                p.declarations[&kind].name
            ),
        ));
    }
    let Some((key_kind, keys)) = p.keys(kind) else {
        return Err(invalid(
            dataset,
            format!(
                "dataset target {} is not a keyed kind",
                p.declarations[&kind].name
            ),
        ));
    };
    if !data.complete_over.is_empty() {
        return Err(invalid(
            dataset,
            "completeness is claimed for a table's rows, not a keyed kind's",
        ));
    }
    let mut bound = BTreeMap::new();
    for binding in &data.bindings {
        let Some((_, ty, _)) = keys.iter().find(|(name, _, _)| *name == binding.name) else {
            return Err(invalid(
                dataset,
                format!(
                    "binding {} names no key of kind {}",
                    binding.name, p.declarations[&key_kind].name
                ),
            ));
        };
        if bound
            .insert(
                binding.name.clone(),
                typed(p, c, dataset, &binding.value, ty, rows)?,
            )
            .is_some()
        {
            return Err(invalid(
                dataset,
                format!("duplicate binding {}", binding.name),
            ));
        }
    }
    let values = schema
        .attributes
        .iter()
        .filter(|(name, _)| {
            !schema.keys.contains(name)
                && !schema.bound.contains_key(name)
                && !schema.derived.iter().any(|(derived, _)| derived == name)
        })
        .collect::<Vec<_>>();
    Ok(Layout {
        key_kind,
        keys,
        bound,
        values,
    })
}
/// The name of one dataset row in a refusal.
fn row_label(p: &CheckedPackage, dataset: DeclarationId, index: usize) -> String {
    format!("row {index} of dataset {}", p.declarations[&dataset].name)
}
/// The complete key values of one row, bound keys included, in the key-declaring kind's
/// order; trailing keys the row omits take their defaults.
fn key_values(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    dataset: DeclarationId,
    layout: &Layout<'_>,
    index: usize,
    cells: &[Cell],
    rows: Rows<'_>,
) -> Result<Vec<Value>> {
    let unbound = layout.unbound().collect::<Vec<_>>();
    if cells.len() > unbound.len() || unbound[cells.len()..].iter().any(|k| k.2.is_none()) {
        return Err(invalid(
            dataset,
            format!(
                "{}: {} key cells where the unbound keys are {:?}",
                row_label(p, dataset, index),
                cells.len(),
                unbound.iter().map(|(name, _, _)| name).collect::<Vec<_>>()
            ),
        ));
    }
    let mut supplied = cells.iter().zip(&unbound);
    layout
        .keys
        .iter()
        .map(|(name, ty, default)| {
            if let Some(value) = layout.bound.get(name) {
                return Ok(value.value.clone());
            }
            match supplied.next() {
                Some((cell, _)) => {
                    let key = typed(p, c, dataset, cell, ty, rows)?;
                    if key.uncertainty.is_some() {
                        return Err(invalid(
                            dataset,
                            format!(
                                "{}: key {name} carries no uncertainty",
                                row_label(p, dataset, index)
                            ),
                        ));
                    }
                    Ok(key.value)
                }
                None => default
                    .clone()
                    .ok_or_else(|| invalid(dataset, "key default")),
            }
        })
        .collect()
}
/// Phase 1: the identity of every row of a dataset whose target is a keyed kind, formed
/// from its key cells alone.
fn identities(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    dataset: DeclarationId,
    kind: DeclarationId,
    data: &pse_authoring::language::AuthoredModelingDeclarationsFieldValueDataset,
) -> Result<Vec<DeclarationId>> {
    let layout = layout(p, c, dataset, kind, data, Rows::Identity)?;
    let mut seen = BTreeSet::new();
    let mut output = Vec::with_capacity(data.rows.len());
    for (index, row) in data.rows.iter().enumerate() {
        let keys = key_values(p, c, dataset, &layout, index, &row.keys, Rows::Identity)?;
        let id = keyed_identity(layout.key_kind, &keys);
        // Two rows of one dataset with one key are refused as two datasets are.
        if !seen.insert(id) {
            return Err(invalid(
                dataset,
                format!(
                    "dataset {} supplies one key of kind {} twice",
                    p.declarations[&dataset].name, p.declarations[&layout.key_kind].name
                ),
            ));
        }
        output.push(id);
    }
    Ok(output)
}
/// Phase 2: the records of a dataset whose target is a keyed kind, every cell typed
/// against the admitted row identities.
fn rows(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    dataset: DeclarationId,
    kind: DeclarationId,
    data: &pse_authoring::language::AuthoredModelingDeclarationsFieldValueDataset,
    rows: Rows<'_>,
) -> Result<Vec<(DeclarationId, Record)>> {
    let layout = layout(p, c, dataset, kind, data, rows)?;
    let mut output = Vec::with_capacity(data.rows.len());
    for (index, row) in data.rows.iter().enumerate() {
        if row.values.len() > layout.values.len() {
            return Err(invalid(
                dataset,
                format!(
                    "{}: {} value cells where kind {} has the attributes {:?}",
                    row_label(p, dataset, index),
                    row.values.len(),
                    p.declarations[&kind].name,
                    layout
                        .values
                        .iter()
                        .map(|(name, _)| name)
                        .collect::<Vec<_>>()
                ),
            ));
        }
        let keys = key_values(p, c, dataset, &layout, index, &row.keys, rows)?;
        let mut supplied = layout
            .keys
            .iter()
            .zip(&keys)
            .map(|((name, _, _), value)| {
                (
                    name.clone(),
                    layout.bound.get(name).cloned().unwrap_or(Typed {
                        value: value.clone(),
                        uncertainty: None,
                    }),
                )
            })
            .collect::<BTreeMap<_, _>>();
        for (cell, (name, declaring)) in row.values.iter().zip(&layout.values) {
            let value = typed(p, c, dataset, cell, &p.types[declaring], rows).map_err(|e| {
                crate::data::context(
                    e,
                    &format!("{}: attribute {name}", row_label(p, dataset, index)),
                )
            })?;
            supplied.insert(name.clone(), value);
        }
        let record = record(p, kind, dataset, supplied)?;
        output.push((keyed_identity(layout.key_kind, &keys), record));
    }
    Ok(output)
}

/// The names a kind is visible by at `at`, each typed as the finite set of the kind's
/// admitted entities. An admission-time expression (a derived attribute or column, a kind or
/// row requirement) ranges over a kind by its name; a specialization root does not, so its
/// model never depends on which entities a closure admits (Plan 23 D0).
pub(crate) fn extent_types(p: &CheckedPackage, at: DeclarationId) -> BTreeMap<String, Type> {
    let mut types = p.named_types(at);
    extents(p, at, &mut types);
    types
        .into_iter()
        .filter(|(_, ty)| matches!(ty, Type::Set(element) if matches!(**element, Type::Entity(_))))
        .filter(|(name, _)| {
            p.resolve(at, name)
                .is_some_and(|id| p.kinds.contains_key(&id))
        })
        .collect()
}

/// Retype each name in `types` that denotes a kind at `at` as the set of its admitted
/// entities (Plan 23 D0).
pub(crate) fn extents(p: &CheckedPackage, at: DeclarationId, types: &mut BTreeMap<String, Type>) {
    for (name, ty) in types.iter_mut() {
        if let Type::Entity(kind) = ty
            && p.kinds.contains_key(kind)
            && p.resolve(at, name) == Some(*kind)
        {
            *ty = Type::Set(Box::new(Type::Entity(*kind)));
        }
    }
}

/// The admitted entities of `kind` and its refinements, in identity order.
pub(crate) fn extent(p: &CheckedPackage, kind: DeclarationId) -> Vec<Value> {
    p.entities
        .iter()
        .filter(|(_, record)| p.refines(record.kind, kind))
        .map(|(id, record)| Value::Entity {
            id: *id,
            kind: record.kind,
        })
        .collect()
}

/// What an expression of `kind` binds: the visible kinds' extents, then the kind's
/// attributes by name and `self`, the entity itself.
fn kind_types(
    p: &CheckedPackage,
    kind: DeclarationId,
    at: DeclarationId,
) -> BTreeMap<String, Type> {
    let mut env = extent_types(p, at);
    for (name, row) in &p.kinds[&kind].attributes {
        if let Some(ty) = p.types.get(row) {
            env.insert(name.clone(), ty.clone());
        }
    }
    env.insert("self".into(), Type::Entity(kind));
    env
}

/// The values an expression of an entity's kind reads: its attributes and `self`.
fn record_values(id: DeclarationId, record: &Record) -> value::Environment {
    let mut env = record.values.clone();
    env.insert(
        "self".into(),
        Value::Entity {
            id,
            kind: record.kind,
        },
    );
    env
}

/// The admitted entities of `kind` and its refinements with their records.
fn members<'p>(
    p: &'p CheckedPackage,
    kind: DeclarationId,
) -> impl Iterator<Item = (DeclarationId, &'p Record)> + 'p {
    p.entities
        .iter()
        .filter(move |(_, record)| p.refines(record.kind, kind))
        .map(|(id, record)| (*id, record))
}

/// One derived attribute: its declaring row and kind, name, type and expression.
struct Derivation {
    row: DeclarationId,
    kind: DeclarationId,
    name: String,
    ty: Type,
    expression: pse_authoring::dsl::Expr,
}

/// Derive every entity's derived attributes once every table is admitted (Plan 23 D0).
///
/// Each derived attribute and each kind requirement is typed first, with the entity's
/// attributes, `self` and the visible kinds' extents bound. Derived attributes are then
/// evaluated once per entity of their kind and its refinements, in the order of the
/// attribute names their expressions mention; a derivation that mentions itself, directly or
/// through others, is refused with the attributes named. An entity whose derivation reads
/// test-only data is test-only, and so is whatever references it.
pub(crate) fn derive(p: &mut CheckedPackage, c: &TypeContext<'_>) -> Result<()> {
    let mut derivations = Vec::new();
    for row in p.declarations.values() {
        let at = row.declaration_id;
        let kind = row.parent_id.filter(|parent| p.kinds.contains_key(parent));
        if let Some(requirement) = &row.value.requirement
            && let Some(kind) = kind
        {
            let predicate = pse_authoring::dsl::parse_predicate(&requirement.predicate)
                .map_err(|e| invalid(at, e.to_string()))?;
            crate::expression::predicate(&predicate, &kind_types(p, kind, at), p, c, at).map_err(
                |e| {
                    crate::data::context(
                        e,
                        &format!(
                            "requirement `{}` of kind {}",
                            requirement.predicate, p.declarations[&kind].name
                        ),
                    )
                },
            )?;
        }
        let Some(expression) = row
            .value
            .attribute
            .as_ref()
            .and_then(|a| a.derived.as_ref())
        else {
            continue;
        };
        let (Some(kind), Some(ty)) = (kind, p.types.get(&at).cloned()) else {
            return Err(invalid(at, "a derived attribute belongs to an entity kind"));
        };
        let what = || {
            format!(
                "derived attribute {} of kind {}",
                row.name, p.declarations[&kind].name
            )
        };
        let expression =
            pse_authoring::dsl::parse_expr(expression).map_err(|e| invalid(at, e.to_string()))?;
        let actual =
            crate::expression::infer(&expression, &kind_types(p, kind, at), p, c, at, Some(&ty))
                .map_err(|e| crate::data::context(e, &what()))?;
        if !p.subsumes(&ty, &actual) {
            return Err(invalid(
                at,
                format!("{} is {actual:?}, not its declared {ty:?}", what()),
            ));
        }
        derivations.push(Derivation {
            row: at,
            kind,
            name: row.name.clone(),
            ty,
            expression,
        });
    }
    let mut tainted = BTreeSet::new();
    for index in derivation_order(p, &derivations)? {
        let derivation = &derivations[index];
        let physical = p.context();
        let mut values = Vec::new();
        for (id, record) in members(p, derivation.kind) {
            let env = record_values(id, record);
            let read = std::cell::Cell::new(false);
            let value = value::Evaluator {
                package: p,
                physical: &physical,
                at: derivation.row,
                env: &env,
                limit: crate::data::EVALUATION_LIMIT,
                stack: Vec::new(),
                reader: crate::provenance::Reader::Admission(Some(&read)),
            }
            .expr(&derivation.expression, Some(&derivation.ty), 0)
            .map_err(|e| {
                crate::data::context(
                    e,
                    &format!(
                        "derived attribute {} of {}",
                        derivation.name,
                        crate::data::display(
                            p,
                            &Value::Entity {
                                id,
                                kind: record.kind
                            }
                        )
                    ),
                )
            })?;
            if !conforms(&value, &derivation.ty, p) {
                return Err(invalid(
                    record.origin,
                    format!(
                        "derived attribute {} of {} is not a {:?}",
                        derivation.name,
                        crate::data::display(
                            p,
                            &Value::Entity {
                                id,
                                kind: record.kind
                            }
                        ),
                        derivation.ty
                    ),
                ));
            }
            if read.get() {
                tainted.insert(id);
            }
            values.push((id, value));
        }
        for (id, value) in values {
            if let Some(record) = p.entities.get_mut(&id) {
                record.values.insert(derivation.name.clone(), value);
            }
        }
    }
    crate::provenance::taint_derived(p, tainted);
    Ok(())
}

/// Derived attributes ordered by the attribute names their expressions mention: a mention
/// of a derived attribute's name, as an attribute of the entity or of any other, orders that
/// derivation first. The order is conservative; a cycle is refused with its attributes named.
fn derivation_order(p: &CheckedPackage, derivations: &[Derivation]) -> Result<Vec<usize>> {
    let mut graph = petgraph::graph::DiGraph::<usize, ()>::new();
    let nodes = (0..derivations.len())
        .map(|index| graph.add_node(index))
        .collect::<Vec<_>>();
    for (to, derivation) in derivations.iter().enumerate() {
        let mentions = derivation
            .expression
            .paths()
            .into_iter()
            .flat_map(|path| path.segments.iter().map(|s| s.name.as_str()))
            .collect::<BTreeSet<_>>();
        for (from, other) in derivations.iter().enumerate() {
            if mentions.contains(other.name.as_str()) {
                graph.add_edge(nodes[from], nodes[to], ());
            }
        }
    }
    petgraph::algo::toposort(&graph, None)
        .map(|order| order.into_iter().map(|n| graph[n]).collect())
        .map_err(|_| {
            let cycle = petgraph::algo::kosaraju_scc(&graph)
                .into_iter()
                .find(|c| c.len() > 1 || c.first().is_some_and(|n| graph.contains_edge(*n, *n)))
                .unwrap_or_default();
            let mut names = cycle
                .iter()
                .map(|n| {
                    let d = &derivations[graph[*n]];
                    format!("{}.{}", p.declarations[&d.kind].name, d.name)
                })
                .collect::<Vec<_>>();
            names.sort();
            invalid(
                cycle
                    .first()
                    .map_or(derivations[0].row, |n| derivations[graph[*n]].row),
                format!(
                    "derived attributes {} derive from one another",
                    names.join(", ")
                ),
            )
        })
}

/// Every entity satisfies the requirements of its kind and its ancestors, and no two
/// entities of a kind share a value of an attribute it declares unique (Plan 23 D0). A
/// refusal names the entities, the kind and the violated clause.
pub(crate) fn verify(p: &CheckedPackage) -> Result<()> {
    let physical = p.context();
    for (kind, schema) in &p.kinds {
        let kind_name = &p.declarations[kind].name;
        let own = |row: &DeclarationId| p.declarations[row].parent_id == Some(*kind);
        for row in schema.requirements.iter().filter(|row| own(row)) {
            let requirement = p.declarations[row]
                .value
                .requirement
                .as_ref()
                .ok_or_else(|| invalid(*row, "requirement payload"))?;
            let predicate = pse_authoring::dsl::parse_predicate(&requirement.predicate)
                .map_err(|e| invalid(*row, e.to_string()))?;
            for (id, record) in members(p, *kind) {
                let env = record_values(id, record);
                let holds = value::Evaluator {
                    package: p,
                    physical: &physical,
                    at: *row,
                    env: &env,
                    limit: crate::data::EVALUATION_LIMIT,
                    stack: Vec::new(),
                    reader: crate::provenance::Reader::Admission(None),
                }
                .predicate(&predicate)?;
                if !holds {
                    return Err(invalid(
                        record.origin,
                        format!(
                            "{} of kind {kind_name} violates the requirement `{}`: {}",
                            crate::data::display(
                                p,
                                &Value::Entity {
                                    id,
                                    kind: record.kind
                                }
                            ),
                            requirement.predicate,
                            requirement.message
                        ),
                    ));
                }
            }
        }
        for (name, _) in schema.unique.iter().filter(|(_, row)| own(row)) {
            let mut seen = BTreeMap::<&Value, DeclarationId>::new();
            for (id, record) in members(p, *kind) {
                let Some(value) = record.values.get(name).filter(|v| **v != Value::Missing) else {
                    continue;
                };
                if let Some(previous) = seen.insert(value, id) {
                    let entity = |id: DeclarationId| {
                        crate::data::display(
                            p,
                            &Value::Entity {
                                id,
                                kind: p.entities[&id].kind,
                            },
                        )
                    };
                    return Err(invalid(
                        record.origin,
                        format!(
                            "{} and {} share {name} = {}, which kind {kind_name} declares unique",
                            entity(previous),
                            entity(id),
                            crate::data::display(p, value)
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}
