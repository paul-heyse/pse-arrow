// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Entities as typed records (ADR-0123 Outcome 2).
//!
//! An entity kind is a record schema: typed attributes with defaults, at most one refined
//! kind, and key attributes declared by one kind of its lineage. A refinement may bind an
//! inherited attribute, a function included, for itself and its own refinements. Data are
//! cells, typed once here at admission: [`typed`] is the only reading of a cell, so no
//! consumer re-reads an attribute value. Declared entities and the rows of keyed kinds
//! become [`Record`]s. A keyed row's identity is framed over the key-declaring kind and its
//! ordered, typed key values, defaults included ([`keyed_identity`]); the concrete
//! refinement is content, so key uniqueness holds across every refinement. Identifier
//! values are opaque, compared byte-exactly and unique within the admitted package closure
//! ([`ModelingIdentifierScope`]).
use crate::specialize::value::{self, Value, conforms};
use crate::{CheckedPackage, DeclarationId, Result, Type, TypeContext, invalid};
use pse_authoring::language::{Cell, CellSelected};
use pse_ids::FramedHasher;
use pse_model::generated::enums::{ModelingDeclarationKind as K, ModelingUncertaintyKind};
use std::collections::{BTreeMap, BTreeSet};

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
            | (Type::Optional(expected), Type::Optional(actual)) => {
                self.subsumes(expected, actual)
            }
            (Type::Optional(expected), actual) => self.subsumes(expected, actual),
            _ => expected == actual,
        }
    }
    /// The admitted record of an entity: a declared entity or a keyed row.
    pub fn record(&self, entity: DeclarationId) -> Option<&Record> {
        self.entities.get(&entity)
    }
    /// A keyed kind's key-declaring kind and its keys: name, type and typed default.
    pub(crate) fn keys(
        &self,
        kind: DeclarationId,
    ) -> Option<(DeclarationId, Vec<(String, Type, Option<Value>)>)> {
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

/// Type a cell against `ty` at the declaration `at` that holds it (ADR-0123 Outcome 1).
///
/// # Errors
/// A cell of another type, an unresolved path, an entity of another kind, an undeclared
/// member or scheme, absence where a value is required, or an uncertainty on a value that
/// is not numeric.
pub(crate) fn typed(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    at: DeclarationId,
    cell: &Cell,
    ty: &Type,
) -> Result<Typed> {
    let selected = cell
        .value
        .selected()
        .map_err(|e| invalid(at, e.to_string()))?;
    let (value, scale) = match (selected, ty) {
        (CellSelected::Missing, Type::Optional(_)) => (Value::Missing, None),
        (CellSelected::Missing, _) => {
            return Err(invalid(at, format!("missing value where {ty:?} is required")));
        }
        (_, Type::Optional(inner)) => return typed(p, c, at, cell, inner),
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
            let unit = pse_authoring::language::unit_product(v)
                .map_err(|e| invalid(at, e.to_string()))?;
            let number = pse_authoring::dsl::Number {
                value: v.magnitude,
                exact_integer: None,
                unit,
            };
            let (value, scale) = value::number(c, at, &number, Some(ty))?;
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
        (CellSelected::Reference(v), _) => (reference(p, at, &v.path, ty)?, None),
        (CellSelected::References(v), Type::Set(element)) => {
            let mut values = Vec::new();
            for path in &v.paths {
                let value = reference(p, at, &path.path, element)?;
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

fn kind_name(selected: CellSelected<'_>) -> &'static str {
    match selected {
        CellSelected::Boolean(_) => "a Boolean",
        CellSelected::Integer(_) => "an integer",
        CellSelected::Quantity(_) => "a quantity",
        CellSelected::Text(_) => "a text",
        CellSelected::Identifier(_) => "an identifier",
        CellSelected::Reference(_) => "a reference",
        CellSelected::References(_) => "a set",
        CellSelected::Missing => "a missing",
    }
}

/// A reference cell's path, resolved once against the expected type.
fn reference(p: &CheckedPackage, at: DeclarationId, path: &[String], ty: &Type) -> Result<Value> {
    let name = path.join(".");
    match ty {
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
            Ok(Value::Entity {
                id,
                kind: *actual,
            })
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
        _ => Err(invalid(at, format!("a reference cell where {ty:?} is expected"))),
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

/// Admit every kind, constant, declared entity and keyed row (ADR-0123 Outcome 2).
pub(crate) fn admit(p: &mut CheckedPackage, c: &TypeContext<'_>) -> Result<()> {
    admit_kinds(p, c)?;
    let constants = p
        .declarations
        .values()
        .filter_map(|row| row.value.constant.as_ref().map(|v| (row.declaration_id, v)))
        .map(|(id, constant)| {
            let ty = p
                .types
                .get(&id)
                .ok_or_else(|| invalid(id, "constant type absent"))?;
            Ok((id, typed(p, c, id, &constant.value, ty)?))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    p.constants = constants;
    let mut records = Vec::new();
    for row in p.declarations.values() {
        if let Some(entity) = &row.value.entity {
            records.push((row.declaration_id, declared(p, c, row.declaration_id, entity)?));
        }
    }
    for row in p.declarations.values() {
        if let Some(dataset) = &row.value.dataset
            && let Some(kind) = p
                .resolve(row.declaration_id, &dataset.target)
                .filter(|id| p.kinds.contains_key(id))
        {
            records.extend(rows(p, c, row.declaration_id, kind, dataset)?);
        }
    }
    for (id, record) in records {
        if let Some(previous) = p.entities.get(&id) {
            return Err(invalid(
                record.origin,
                format!(
                    "the key of this {} row is already admitted: {} (kind {}) and {} (kind {}) supply the same key of kind {}",
                    p.declarations[&record.kind].name,
                    p.declarations[&previous.origin].name,
                    p.declarations[&previous.kind].name,
                    p.declarations[&record.origin].name,
                    p.declarations[&record.kind].name,
                    p.kinds[&record.kind]
                        .key_kind
                        .map_or_else(String::new, |k| p.declarations[&k].name.clone()),
                ),
            ));
        }
        for value in record.values.values() {
            if let Value::Identifier { scheme, value } = value
                && let Err(holder) = p.identifiers.insert(*scheme, value, id)
            {
                return Err(invalid(
                    record.origin,
                    format!(
                        "identifier {} \"{value}\" is held by {} and by {}; identifier values are unique within the package closure",
                        p.declarations[scheme].name,
                        p.label(holder),
                        label(p, id, &record),
                    ),
                ));
            }
        }
        p.entities.insert(id, record);
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
        let mut own_keys = Vec::new();
        for child in p.children.get(&id).cloned().unwrap_or_default() {
            let row = &p.declarations[&child];
            let Some(attribute) = &row.value.attribute else {
                return Err(invalid(
                    child,
                    "an entity kind declares attributes and binds inherited ones only",
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
                    if let Some(owner) = inherited.and_then(|row| p.declarations[&row].parent_id)
                    {
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
                    if key {
                        if !key_type(&ty) {
                            return Err(invalid(
                                child,
                                "a key is a reference, an enumeration member, an integer, text, a Boolean, an identifier or a quantity",
                            ));
                        }
                        own_keys.push(name.clone());
                    }
                    if let Some(cell) = &attribute.value {
                        kind.defaults
                            .insert(name.clone(), typed(p, c, child, cell, &ty)?);
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
    typed(p, c, at, cell, ty)
}

/// A declared entity's record: supplied values, then kind bindings, defaults and absence.
fn declared(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    id: DeclarationId,
    entity: &pse_authoring::language::AuthoredModelingDeclarationsFieldValueEntity,
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
        supplied.insert(
            attribute.name.clone(),
            typed(p, c, id, &attribute.value, &p.types[&declaring])?,
        );
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
    let mut uncertainties = BTreeMap::new();
    for (name, declaring) in &schema.attributes {
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

/// The rows of a dataset whose target is a keyed kind. Row keys are the keys the dataset
/// does not bind, in declaration order; row values are the kind's other unbound
/// attributes, in lineage order. Trailing keys and values with a default, and trailing
/// optional values, may be omitted.
fn rows(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    dataset: DeclarationId,
    kind: DeclarationId,
    data: &pse_authoring::language::AuthoredModelingDeclarationsFieldValueDataset,
) -> Result<Vec<(DeclarationId, Record)>> {
    let schema = &p.kinds[&kind];
    let Some((key_kind, keys)) = p.keys(kind) else {
        return Err(invalid(
            dataset,
            format!(
                "dataset target {} is not a keyed kind",
                p.declarations[&kind].name
            ),
        ));
    };
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
            .insert(binding.name.clone(), typed(p, c, dataset, &binding.value, ty)?)
            .is_some()
        {
            return Err(invalid(dataset, format!("duplicate binding {}", binding.name)));
        }
    }
    let unbound = keys
        .iter()
        .filter(|(name, _, _)| !bound.contains_key(name))
        .collect::<Vec<_>>();
    let values = schema
        .attributes
        .iter()
        .filter(|(name, _)| !schema.keys.contains(name) && !schema.bound.contains_key(name))
        .collect::<Vec<_>>();
    let omittable = |cells: usize, total: usize, has_default: &dyn Fn(usize) -> bool| {
        cells <= total && (cells..total).all(has_default)
    };
    let mut output = Vec::new();
    for (index, row) in data.rows.iter().enumerate() {
        let at = || format!("row {index} of dataset {}", p.declarations[&dataset].name);
        if !omittable(row.keys.len(), unbound.len(), &|i| unbound[i].2.is_some()) {
            return Err(invalid(
                dataset,
                format!(
                    "{}: {} key cells where the unbound keys are {:?}",
                    at(),
                    row.keys.len(),
                    unbound.iter().map(|(name, _, _)| name).collect::<Vec<_>>()
                ),
            ));
        }
        if row.values.len() > values.len() {
            return Err(invalid(
                dataset,
                format!(
                    "{}: {} value cells where kind {} has the attributes {:?}",
                    at(),
                    row.values.len(),
                    p.declarations[&kind].name,
                    values.iter().map(|(name, _)| name).collect::<Vec<_>>()
                ),
            ));
        }
        let mut supplied = bound.clone();
        for (cell, (name, ty, _)) in row.keys.iter().zip(&unbound) {
            supplied.insert(name.clone(), typed(p, c, dataset, cell, ty)?);
        }
        for (cell, (name, declaring)) in row.values.iter().zip(&values) {
            supplied.insert(name.clone(), typed(p, c, dataset, cell, &p.types[declaring])?);
        }
        let record = record(p, kind, dataset, supplied)?;
        let key_values = keys
            .iter()
            .map(|(name, _, _)| record.values[name].clone())
            .collect::<Vec<_>>();
        output.push((keyed_identity(key_kind, &key_values), record));
    }
    // Two rows of one dataset with one key are refused as two datasets are.
    let mut seen = BTreeSet::new();
    for (id, _) in &output {
        if !seen.insert(*id) {
            return Err(invalid(
                dataset,
                format!(
                    "dataset {} supplies one key of kind {} twice",
                    p.declarations[&dataset].name, p.declarations[&key_kind].name
                ),
            ));
        }
    }
    Ok(output)
}
