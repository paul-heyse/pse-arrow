// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded compile-time values. This interpreter selects structure; runtime math uses pse-math.
use crate::{CheckedPackage, DeclarationId, Result, Type, TypeContext, invalid};
use pse_authoring::{
    dsl::{self, BinaryOp, CompareOp, Expr, ExprKind, Predicate, PredicateKind},
    language::{StaticValue, parse_static},
};
use pse_ids::{FramedHasher, SemanticId};
use pse_model::generated::enums::ModelingDeclarationKind as Kind;
use pse_quantity::{QuantityTypeId, scheme::Scheme};
use std::collections::{BTreeMap, BTreeSet};

mod structure;

/// A fully explicit structural value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    /// Predicate.
    Boolean(bool),
    /// Exact integer.
    Integer(i64),
    /// Finite scalar with its complete physical type.
    Number {
        /// Bit pattern of the canonical `f64` value.
        bits: u64,
        /// Physical quantity type.
        quantity: QuantityTypeId,
    },
    /// Generated continuous coordinate; identity is independent of its floating value.
    Coordinate {
        /// Coordinate identity.
        id: SemanticId,
        /// Bit pattern of the canonical `f64` position.
        bits: u64,
        /// Physical quantity type of the axis.
        quantity: QuantityTypeId,
    },
    /// Textual label.
    Text(String),
    /// Entity identity and its concrete kind; the kind is content, not identity
    /// (ADR-0123 Outcome 2).
    Entity {
        /// The entity declaration, or a keyed row's derived identity.
        id: DeclarationId,
        /// The most-derived kind of the entity.
        kind: DeclarationId,
    },
    /// Value of an authored closed enumeration.
    Enum {
        /// Enumeration declaration.
        enumeration: DeclarationId,
        /// Member identity: renaming a member keeps it (ADR-0123 Outcome 2).
        member: SemanticId,
    },
    /// An opaque value of a declared identifier scheme, compared byte-exactly and never
    /// parsed (ADR-0123 Outcome 2).
    Identifier {
        /// Identifier scheme declaration.
        scheme: DeclarationId,
        /// The value as written.
        value: String,
    },
    /// Definition identity with named partial bindings.
    Definition {
        /// Definition declaration.
        id: DeclarationId,
        /// Bound arguments by name.
        bindings: BTreeMap<String, Value>,
    },
    /// A pure function declaration selected as structural data.
    Function(DeclarationId),
    /// Typed heterogeneous table row: positional cells and the column names they bind.
    Row {
        /// Table declaration.
        table: DeclarationId,
        /// Column names in column order, shared with the admitted table.
        names: std::sync::Arc<[String]>,
        /// Values in column order, shared with the admitted row.
        fields: std::sync::Arc<[Value]>,
    },
    /// Ordered finite set with no duplicates.
    Set(Vec<Value>),
    /// Ragged coordinate or table key.
    Tuple(Vec<Value>),
    /// A named quantity type of the physical document (ADR-0123 Outcome 6).
    QuantityType(QuantityTypeId),
    /// A named reference state of the physical document (ADR-0123 Outcome 6).
    ReferenceState(pse_quantity::ReferenceStateId),
    /// Explicit absence, never numeric zero.
    Missing,
}
impl Value {
    /// Stable framing excludes map allocation and backend-local IDs.
    pub fn frame(&self, h: &mut FramedHasher) {
        match self {
            Self::Boolean(v) => {
                h.str("bool").bool(*v);
            }
            Self::Integer(v) => {
                h.str("int").part(&v.to_le_bytes());
            }
            Self::Coordinate { id, bits, quantity } => {
                h.str("coordinate").id(id).u64(*bits).id(&quantity.as_id());
            }
            Self::Function(id) => {
                h.str("function").id(&id.as_id());
            }
            Self::Number { bits, quantity } => {
                h.str("number").u64(*bits).id(&quantity.as_id());
            }
            Self::Text(v) => {
                h.str("text").str(v);
            }
            Self::Entity { id, kind } => {
                h.str("entity").id(&id.as_id()).id(&kind.as_id());
            }
            Self::Enum {
                enumeration,
                member,
            } => {
                h.str("enum").id(&enumeration.as_id()).id(member);
            }
            Self::Identifier { scheme, value } => {
                h.str("identifier").id(&scheme.as_id()).str(value);
            }
            Self::Definition { id, bindings } => {
                h.str("definition").id(&id.as_id());
                for (n, v) in bindings {
                    h.str(n);
                    v.frame(h);
                }
            }
            // Columns are framed in name order, as a row's named fields always were.
            Self::Row {
                table,
                names,
                fields,
            } => {
                h.str("row").id(&table.as_id());
                let mut order = (0..names.len().min(fields.len())).collect::<Vec<_>>();
                order.sort_by(|a, b| names[*a].cmp(&names[*b]));
                for i in order {
                    h.str(&names[i]);
                    fields[i].frame(h);
                }
            }
            Self::Set(v) | Self::Tuple(v) => {
                h.str(if matches!(self, Self::Set(_)) {
                    "set"
                } else {
                    "tuple"
                })
                .u64(v.len() as u64);
                for x in v {
                    x.frame(h);
                }
            }
            Self::QuantityType(id) => {
                h.str("quantity-type").id(&id.as_id());
            }
            Self::ReferenceState(id) => {
                h.str("reference-state").id(&id.as_id());
            }
            Self::Missing => {
                h.str("missing");
            }
        }
    }
    /// Stable coordinate identity, preserving compound-key identity.
    pub fn identity(&self) -> SemanticId {
        if let Self::Entity { id, .. } = self {
            return id.as_id();
        }
        if let Self::Coordinate { id, .. } = self {
            return *id;
        }
        let mut h = FramedHasher::new(pse_ids::Frame::ModelingCoordinateV2);
        self.frame(&mut h);
        h.finish_id()
    }
    pub(crate) fn scalar(&self, at: DeclarationId) -> Result<f64> {
        match self {
            Self::Number { bits, .. } | Self::Coordinate { bits, .. } => Ok(f64::from_bits(*bits)),
            Self::Integer(v) => Ok(*v as f64),
            _ => Err(invalid(at, "numeric static value required")),
        }
    }
}
/// Immutable static environment supplied to specialization.
pub type Environment = BTreeMap<String, Value>;

pub(crate) struct Evaluator<'a, 'b> {
    pub selections: Option<&'a crate::scientific_selection::Collector>,
    pub package: &'a CheckedPackage,
    pub physical: &'a TypeContext<'b>,
    pub at: DeclarationId,
    pub env: &'a Environment,
    pub limit: usize,
    pub stack: Vec<DeclarationId>,
    /// Who reads admitted data: admission, a test fixture or a production root, which
    /// reads no test-only data (ADR-0123 Outcome 5).
    pub reader: crate::provenance::Reader<'a>,
}
impl Evaluator<'_, '_> {
    pub(crate) fn text(&mut self, text: &str, expected: Option<&Type>) -> Result<Value> {
        let value = if matches!(expected, Some(Type::Integer)) && text.trim().parse::<i64>().is_ok()
        {
            Value::Integer(
                text.trim()
                    .parse::<i64>()
                    .map_err(|e| invalid(self.at, e.to_string()))?,
            )
        } else {
            let syntax = parse_static(text).map_err(|e| invalid(self.at, e.to_string()))?;
            self.syntax(&syntax, expected, 0)?
        };
        if let Some(expected) = expected
            && !conforms(&value, expected, self.package)
        {
            return Err(invalid(
                self.at,
                format!("static value does not satisfy {expected:?}"),
            ));
        }
        Ok(value)
    }
    fn syntax(
        &mut self,
        syntax: &StaticValue,
        expected: Option<&Type>,
        depth: usize,
    ) -> Result<Value> {
        if depth > 64 {
            return Err(invalid(self.at, "static expansion depth"));
        }
        match syntax {
            StaticValue::Text(s) => Ok(Value::Text(s.clone())),
            StaticValue::Set(values) | StaticValue::Tuple(values) => {
                if values.len() > self.limit {
                    return Err(invalid(self.at, "finite membership budget"));
                }
                let member = if let Some(Type::Set(t)) = expected {
                    Some(t.as_ref())
                } else {
                    None
                };
                let tuple = if let Some(Type::Tuple(types)) = expected {
                    Some(types)
                } else {
                    None
                };
                if let Some(types) = tuple
                    && types.len() != values.len()
                {
                    return Err(invalid(self.at, "tuple arity"));
                }
                let values = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        self.syntax(v, tuple.and_then(|t| t.get(i)).or(member), depth + 1)
                    })
                    .collect::<Result<Vec<_>>>()?;
                if matches!(syntax, StaticValue::Set(_)) {
                    if values.iter().collect::<BTreeSet<_>>().len() != values.len() {
                        return Err(invalid(self.at, "duplicate finite member"));
                    }
                    Ok(Value::Set(values))
                } else {
                    Ok(Value::Tuple(values))
                }
            }
            StaticValue::Comprehension {
                body,
                bindings,
                filter,
            } => {
                let mut environments = vec![self.env.clone()];
                for (name, source) in bindings {
                    let mut next = Vec::new();
                    for env in environments {
                        let mut evaluator = Evaluator {
                            package: self.package,
                            physical: self.physical,
                            at: self.at,
                            env: &env,
                            limit: self.limit,
                            stack: self.stack.clone(),
                            reader: self.reader,
                            selections: self.selections,
                        };
                        let Value::Set(values) = evaluator.syntax(source, None, depth + 1)? else {
                            return Err(invalid(
                                self.at,
                                "comprehension domain must be a finite set",
                            ));
                        };
                        if next.len().saturating_add(values.len()) > self.limit {
                            return Err(invalid(self.at, "comprehension extent exceeds budget"));
                        }
                        for value in values {
                            let mut local = env.clone();
                            local.insert(name.clone(), value);
                            next.push(local);
                        }
                    }
                    environments = next;
                }
                let element = if let Some(Type::Set(t)) = expected {
                    Some(t.as_ref())
                } else {
                    None
                };
                let mut values = Vec::new();
                let mut seen = BTreeSet::new();
                for env in environments {
                    let mut evaluator = Evaluator {
                        package: self.package,
                        physical: self.physical,
                        at: self.at,
                        env: &env,
                        limit: self.limit,
                        stack: self.stack.clone(),
                        reader: self.reader,
                        selections: self.selections,
                    };
                    if let Some(filter) = filter
                        && !evaluator.predicate(filter)?
                    {
                        continue;
                    }
                    let value = evaluator.syntax(body, element, depth + 1)?;
                    if seen.insert(value.clone()) {
                        values.push(value);
                    }
                }
                Ok(Value::Set(values))
            }
            StaticValue::Apply { name, arguments } => {
                let mut value = self.reference(name, depth + 1)?;
                let Value::Definition { id, bindings } = &mut value else {
                    return Err(invalid(self.at, "definition reference required"));
                };
                let mut supplied = BTreeSet::new();
                for (n, expression) in arguments {
                    if !supplied.insert(n) {
                        return Err(invalid(self.at, "duplicate binding"));
                    }
                    let target = self.package.preset_definition(*id)?;
                    let parameter = self.package.declarations[&target]
                        .value
                        .scope
                        .as_ref()
                        .and_then(|s| s.parameters.iter().find(|p| p.name == *n))
                        .ok_or_else(|| invalid(self.at, format!("unknown argument {n}")))?;
                    let ty = self.physical.resolve(
                        &parameter.r#type,
                        &BTreeSet::new(),
                        &self.package.named_types(*id),
                        *id,
                    )?;
                    let value = self.syntax(expression, Some(&ty), depth + 1)?;
                    if !conforms(&value, &ty, self.package) {
                        return Err(invalid(self.at, "constructor argument type"));
                    }
                    bindings.insert(n.clone(), value);
                }
                Ok(value)
            }
            StaticValue::Expression(e) => self.expr(e, expected, depth + 1),
        }
    }
    /// The declarations' types visible at `at`; at admission a kind's name types its
    /// extent (Plan 23 D0).
    fn named_types(&self) -> BTreeMap<String, Type> {
        let mut types = self.package.named_types(self.at);
        if matches!(self.reader, crate::provenance::Reader::Admission(_)) {
            crate::entity::extents(self.package, self.at, &mut types);
        }
        types
    }
    fn reference(&mut self, name: &str, depth: usize) -> Result<Value> {
        if depth > 64 {
            return Err(invalid(self.at, "static reference depth"));
        }
        if let Some(v) = self.env.get(name) {
            crate::scientific_selection::retain_context(self.package, v, self.selections);
            return Ok(v.clone());
        }
        let Some(id) = self.package.resolve(self.at, name) else {
            // ADR-0123 Outcome 6: a physical name the owning package sees.
            return match self.package.physical_name(self.at, name) {
                Some(pse_quantity::PhysicalName::QuantityType(id)) => Ok(Value::QuantityType(id)),
                Some(pse_quantity::PhysicalName::ReferenceState(id)) => {
                    Ok(Value::ReferenceState(id))
                }
                None => Err(invalid(self.at, format!("missing static binding {name}"))),
            };
        };
        if self.stack.contains(&id) {
            return Err(invalid(
                id,
                format!(
                    "recursive static binding {}",
                    self.stack
                        .iter()
                        .map(|id| self.package.declarations[id].name.clone())
                        .collect::<Vec<_>>()
                        .join(" -> ")
                ),
            ));
        }
        self.stack.push(id);
        let row = &self.package.declarations[&id];
        let saved_at = self.at;
        self.at = id;
        let value = match row
            .value
            .selected()
            .map_err(|e| invalid(id, e.to_string()))?
        {
            _ if self.package.functions.contains_key(&id) => Ok(Value::Function(id)),
            // ADR-0123 Outcome 5: an entity or a constant that references test-only data is
            // test-only, and a root outside a test fixture reads none.
            crate::Selected::Entity(_) => match self.package.types.get(&id) {
                Some(Type::Entity(kind)) => self
                    .reader
                    .read(
                        self.package,
                        saved_at,
                        self.package.is_test_only(id),
                        || format!("entity {} referencing test-only data", row.name),
                    )
                    .map(|()| Value::Entity { id, kind: *kind }),
                _ => Err(invalid(id, "entity kind")),
            },
            // ADR-0123 Outcome 2: a typed constant is admitted data.
            crate::Selected::Constant(_) => self
                .reader
                .read(
                    self.package,
                    saved_at,
                    self.package.is_test_only(id),
                    || {
                        format!(
                            "constant {} {}",
                            row.name,
                            crate::provenance::supplied_by(self.package, id)
                        )
                    },
                )
                .and_then(|()| {
                    self.package
                        .constants
                        .get(&id)
                        .map(|typed| typed.value.clone())
                        .ok_or_else(|| invalid(id, "constant not admitted"))
                }),
            // Plan 23 D0: at admission, a kind denotes its admitted extent.
            crate::Selected::EntityKind(_)
                if matches!(self.reader, crate::provenance::Reader::Admission(_)) =>
            {
                Ok(Value::Set(crate::entity::extent(self.package, id)))
            }
            crate::Selected::Definition(_)
            | crate::Selected::Interface(_)
            | crate::Selected::Preset(_) => Ok(Value::Definition {
                id,
                bindings: BTreeMap::new(),
            }),
            crate::Selected::Set(b) | crate::Selected::ScopeValue(b) => {
                let source = b
                    .expression
                    .as_ref()
                    .ok_or_else(|| invalid(id, "static value requires a definition"))?;
                self.text(source, self.package.types.get(&id))
            }
            _ => Err(invalid(
                id,
                format!(
                    "runtime value {name} ({}) cannot select model structure",
                    row.value.kind.as_str()
                ),
            )),
        };
        self.at = saved_at;
        self.stack.pop();
        if let Ok(value) = &value {
            crate::scientific_selection::retain_context(self.package, value, self.selections);
        }
        value
    }
    /// A named member of a static value: a definition's member, a row's column, an
    /// entity's attribute or a reference state's condition. An entity's attributes are
    /// read from its record, typed once at admission (ADR-0123 Outcome 2).
    fn member(&mut self, value: Value, name: &str) -> Result<Value> {
        crate::scientific_selection::retain_context(self.package, &value, self.selections);
        let numeric_owner = match &value {
            Value::Entity { id, .. } => Some(*id),
            _ => None,
        };
        let result = match value {
            Value::Definition { id, bindings } => self.definition_member(id, &bindings, name)?,
            Value::Row { names, fields, .. } => names
                .iter()
                .position(|n| n == name)
                .and_then(|i| fields.get(i))
                .cloned()
                .ok_or_else(|| invalid(self.at, "unknown table column"))?,
            Value::Entity { id, kind } => self
                .package
                .record(id)
                .and_then(|record| record.values.get(name))
                .cloned()
                .ok_or_else(|| {
                    let derived = self
                        .package
                        .kinds
                        .get(&kind)
                        .is_some_and(|k| k.derived.iter().any(|(n, _)| n == name));
                    invalid(
                        id,
                        if derived {
                            format!(
                                "derived attribute {name} of kind {} is evaluated once every table is admitted; a table's derived column does not read it",
                                self.package.declarations[&kind].name
                            )
                        } else {
                            format!(
                                "unknown attribute {name} of kind {}",
                                self.package.declarations[&kind].name
                            )
                        },
                    )
                })?,
            Value::ReferenceState(id) => self.reference_condition(id, name)?,
            Value::Missing => {
                return Err(invalid(self.at, "missing optional row must be guarded"));
            }
            _ => return Err(invalid(self.at, "static value has no named member")),
        };
        crate::scientific_selection::retain_context(self.package, &result, self.selections);
        if matches!(
            &result,
            Value::Number { .. } | Value::Integer(_) | Value::Coordinate { .. }
        ) && let Some(id) = numeric_owner
            && let Some(collector) = self.selections
        {
            collector.record_numeric(id);
        }
        Ok(result)
    }
    /// The row of a keyed kind with the given key values: the key-declaring kind's keys in
    /// order, trailing keys with a default omitted (ADR-0123 Outcome 2).
    fn keyed(&mut self, kind: DeclarationId, indices: &[Expr], depth: usize) -> Result<Value> {
        let name = || self.package.declarations[&kind].name.clone();
        let (_, keys) = self
            .package
            .keys(kind)
            .ok_or_else(|| invalid(self.at, format!("kind {} has no keys", name())))?;
        if indices.len() > keys.len() || keys[indices.len()..].iter().any(|k| k.2.is_none()) {
            return Err(invalid(
                self.at,
                format!(
                    "a {} row is looked up by its keys {:?}",
                    name(),
                    keys.iter().map(|k| &k.0).collect::<Vec<_>>()
                ),
            ));
        }
        let mut values = Vec::new();
        for (index, (_, ty, default)) in keys.iter().enumerate() {
            values.push(match indices.get(index) {
                Some(e) => {
                    let v = self.expr(e, Some(ty), depth + 1)?;
                    if !conforms(&v, ty, self.package) {
                        return Err(invalid(self.at, format!("{} key type", name())));
                    }
                    v
                }
                None => default
                    .clone()
                    .ok_or_else(|| invalid(self.at, "key default"))?,
            });
        }
        let row = self
            .package
            .keyed_row(kind, &values)
            .ok_or_else(|| invalid(self.at, format!("no {} row has these keys", name())))?;
        // ADR-0123 Outcome 5: a root outside a test fixture reads no test-only row.
        crate::scientific_selection::retain_context(self.package, &row, self.selections);
        if let Value::Entity { id, .. } = &row {
            let origin = self.package.record(*id).map(|r| r.origin);
            self.reader.read(
                self.package,
                self.at,
                self.package.is_test_only(*id),
                || {
                    format!(
                        "{} {}",
                        crate::data::display(self.package, &row),
                        origin.map_or_else(String::new, |origin| {
                            crate::provenance::supplied_by(self.package, origin)
                        })
                    )
                },
            )?;
        }
        Ok(row)
    }
    /// A reference state's typed condition, in its quantity type's canonical unit
    /// (ADR-0123 Outcome 6).
    pub(crate) fn reference_condition(
        &self,
        state: pse_quantity::ReferenceStateId,
        attribute: &str,
    ) -> Result<Value> {
        let quantities = self.physical.quantities;
        let reference = quantities
            .reference_state(state)
            .map_err(|e| invalid(self.at, e.to_string()))?;
        let condition = match attribute {
            "temperature" => reference.temperature,
            "pressure" => reference.pressure,
            _ => {
                return Err(invalid(
                    self.at,
                    format!("a reference state has no attribute {attribute}"),
                ));
            }
        }
        .ok_or_else(|| {
            invalid(
                self.at,
                format!("reference state {} declares no {attribute}", reference.name),
            )
        })?;
        let expected = self.package.reference_attribute_type(attribute, self.at)?;
        if condition.quantity_type != expected {
            return Err(invalid(self.at, "reference condition type"));
        }
        let value = quantities
            .reference_condition(&condition)
            .map_err(|e| invalid(self.at, e.to_string()))?;
        Ok(Value::Number {
            bits: value.to_bits(),
            quantity: condition.quantity_type,
        })
    }
    /// The complete type of a static arithmetic result, inferred at its root.
    fn typed_result(&mut self, e: &Expr, result: f64, expected: Option<&Type>) -> Result<Value> {
        let mut env = self
            .env
            .iter()
            .filter_map(|(n, v)| value_type(v).map(|t| (n.clone(), t)))
            .collect::<BTreeMap<_, _>>();
        // At admission, a reduction over a kind ranges over its extent (Plan 23 D0).
        if matches!(self.reader, crate::provenance::Reader::Admission(_)) {
            e.walk(|node| {
                if let ExprKind::Reduce { binder, .. } | ExprKind::Fold { binder, .. } = &node.kind
                {
                    let name = dsl::render_path(&binder.domain);
                    if let Some(kind) = self
                        .package
                        .resolve(self.at, &name)
                        .filter(|id| self.package.kinds.contains_key(id))
                    {
                        env.entry(name)
                            .or_insert_with(|| Type::Set(Box::new(Type::Entity(kind))));
                    }
                }
            });
        }
        // The operands are static values: an optional member a guard found present has its
        // present type, as the guard's refinement typed it (Plan 23 D0).
        let mut presence_paths = BTreeSet::new();
        e.walk(|node| {
            if let ExprKind::NamedCall { name, args } = &node.kind
                && matches!(name.as_str(),"present" | "require_present")
                && let [
                    Expr {
                        kind: ExprKind::Path(path),
                        ..
                    },
                ] = args.as_slice()
            {
                presence_paths.insert(dsl::render_path(path));
            }
        });
        for path in e.paths() {
            if path.segments.len() > 1 && path.segments.iter().all(|s| s.indices.is_empty()) {
                let name = dsl::render_path(path);
                if !env.contains_key(&name)
                    && !presence_paths.contains(&name)
                    && let Ok(value) = self.expr(
                        &Expr {
                            kind: ExprKind::Path(path.clone()),
                            span: dsl::Span::default(),
                        },
                        None,
                        0,
                    )
                    && !matches!(value, Value::Missing | Value::Entity { .. })
                    && let Some(ty) = value_type(&value)
                {
                    env.insert(name, ty);
                }
            }
        }
        let ty = crate::expression::infer(e, &env, self.package, self.physical, self.at, expected)?;
        match ty {
            Type::Quantity(Scheme::Concrete(quantity)) => Ok(Value::Number {
                bits: result.to_bits(),
                quantity,
            }),
            Type::Integer if result.fract() == 0.0 && result.abs() < i64::MAX as f64 => {
                Ok(Value::Integer(result as i64))
            }
            _ => Err(invalid(self.at, "static arithmetic result type")),
        }
    }
    /// The value of a multiplicative chain: exact when every factor is an integer.
    fn chain_value(&mut self, e: &Expr, depth: usize) -> Result<Numeric> {
        if depth > 64 {
            return Err(invalid(self.at, "static expression depth"));
        }
        let ExprKind::Binary { op, lhs, rhs } = &e.kind else {
            return Ok(match self.expr(e, None, depth)? {
                Value::Integer(value) => Numeric::Integer(value),
                other => Numeric::Real(other.scalar(self.at)?),
            });
        };
        if !crate::expression::chain_operation(*op, rhs) {
            return Ok(Numeric::Real(self.expr(e, None, depth)?.scalar(self.at)?));
        }
        let a = self.chain_value(lhs, depth + 1)?;
        let b = if *op == BinaryOp::Pow {
            let expected = matches!(a, Numeric::Integer(_)).then_some(Type::Integer);
            match self.expr(rhs, expected.as_ref(), depth + 1)? {
                Value::Integer(value) => Numeric::Integer(value),
                other => Numeric::Real(other.scalar(self.at)?),
            }
        } else {
            self.chain_value(rhs, depth + 1)?
        };
        if let (Numeric::Integer(x), Numeric::Integer(y)) = (a, b) {
            let result = match op {
                BinaryOp::Mul => x.checked_mul(y),
                BinaryOp::Div if y != 0 && x.checked_rem(y) == Some(0) => x.checked_div(y),
                BinaryOp::Pow => u32::try_from(y).ok().and_then(|y| x.checked_pow(y)),
                _ => None,
            };
            return result.map(Numeric::Integer).ok_or_else(|| {
                invalid(self.at, "integer arithmetic overflow or nonintegral result")
            });
        }
        let (x, y) = (a.real(), b.real());
        Ok(Numeric::Real(match op {
            BinaryOp::Mul => x * y,
            BinaryOp::Div => x / y,
            _ => x.powf(y),
        }))
    }
    pub(crate) fn expr(
        &mut self,
        e: &Expr,
        expected: Option<&Type>,
        depth: usize,
    ) -> Result<Value> {
        if depth > 64 {
            return Err(invalid(self.at, "static expression depth"));
        }
        match &e.kind {
            ExprKind::Number(n) => Ok(number(self.physical, self.at, n, expected)?.0),
            ExprKind::Path(path) => {
                let name = dsl::render_path(path);
                if name == "true" {
                    return Ok(Value::Boolean(true));
                }
                if name == "false" {
                    return Ok(Value::Boolean(false));
                }
                // Plan 23 D0: explicit absence, never numeric zero.
                if name == "missing" {
                    return Ok(Value::Missing);
                }
                if let Some(v) = self.env.get(&name) {
                    return Ok(v.clone());
                }
                if let Some(v) = crate::analysis::constant(&name) {
                    return Ok(v);
                }
                let first = path
                    .segments
                    .first()
                    .ok_or_else(|| invalid(self.at, "empty static path"))?;
                if let Some(position) = path
                    .segments
                    .iter()
                    .position(|segment| !segment.indices.is_empty())
                {
                    let selected = &path.segments[position];
                    let table_name = path.segments[..=position]
                        .iter()
                        .map(|segment| segment.name.as_str())
                        .collect::<Vec<_>>()
                        .join(".");
                    let id = self
                        .package
                        .resolve(self.at, &table_name)
                        .ok_or_else(|| invalid(self.at, "unknown table"))?;
                    let value = if self.package.kinds.contains_key(&id) {
                        self.keyed(id, &selected.indices, depth)?
                    } else {
                        let table = self
                            .package
                            .tables
                            .get(&id)
                            .ok_or_else(|| invalid(id, "table data not admitted"))?;
                        if selected.indices.len() != table.keys.len() {
                            return Err(invalid(id, "table key arity"));
                        }
                        let keys = selected
                            .indices
                            .iter()
                            .zip(&table.keys)
                            .map(|(e, key)| {
                                let v = self.expr(e, Some(&key.ty), depth + 1)?;
                                if !conforms(&v, &key.ty, self.package) {
                                    return Err(invalid(id, "table key type"));
                                }
                                crate::scientific_selection::retain_context(
                                    self.package,
                                    &v,
                                    self.selections,
                                );
                                Ok(v)
                            })
                            .collect::<Result<Vec<_>>>()?;
                        // ADR-0123 Outcome 3: a required table refuses a key outside its
                        // completeness before reading a row.
                        table.lookup(self.package, id, self.at, keys, self.reader)?
                    };
                    crate::scientific_selection::retain_context(
                        self.package,
                        &value,
                        self.selections,
                    );
                    let mut value = value;
                    for segment in &path.segments[position + 1..] {
                        if !segment.indices.is_empty() {
                            return Err(invalid(id, "invalid table row member access"));
                        }
                        value = self.member(value, &segment.name)?;
                    }
                    return Ok(value);
                }
                if path.segments.len() > 1 {
                    let definition_prefix = (1..path.segments.len()).rev().find_map(|count| {
                        let prefix = path.segments[..count]
                            .iter()
                            .map(|s| s.name.as_str())
                            .collect::<Vec<_>>()
                            .join(".");
                        let definition = self.env.get(&prefix).cloned().or_else(|| {
                            let id = self.package.resolve(self.at, &prefix)?;
                            matches!(
                                self.package.declarations[&id].value.kind,
                                Kind::Definition | Kind::Interface | Kind::Preset
                            )
                            .then(|| Value::Definition {
                                id,
                                bindings: Environment::new(),
                            })
                        })?;
                        matches!(definition, Value::Definition { .. })
                            .then_some((count, definition))
                    });
                    // A lexical binding shadows a declaration of its name, as in typing (Plan 23
                    // D0: a kind's attribute named like a declaration).
                    let lexical = self.env.contains_key(&first.name);
                    // A qualified declaration path is resolved before entity attribute traversal.
                    // Package names are scopes, not runtime values to be evaluated.
                    if definition_prefix.is_none()
                        && !lexical
                        && self.package.resolve(self.at, &name).is_some()
                    {
                        return self.reference(&name, depth + 1);
                    }
                    let enum_name = path.segments[..path.segments.len() - 1]
                        .iter()
                        .map(|segment| segment.name.as_str())
                        .collect::<Vec<_>>()
                        .join(".");
                    if !lexical
                        && let Some(id) = self.package.resolve(self.at, &enum_name)
                        && self.package.declarations[&id].value.enumeration.is_some()
                    {
                        let member = &path.segments[path.segments.len() - 1].name;
                        return crate::entity::enum_member(self.package, id, member, id);
                    }
                    // A qualified physical name, `<package>.<Name>`, before its attributes.
                    let physical_prefix = (2..path.segments.len()).rev().find_map(|count| {
                        let prefix = path.segments[..count]
                            .iter()
                            .map(|s| s.name.as_str())
                            .collect::<Vec<_>>()
                            .join(".");
                        match self.package.physical_name(self.at, &prefix)? {
                            pse_quantity::PhysicalName::ReferenceState(id) => {
                                Some((count, Value::ReferenceState(id)))
                            }
                            pse_quantity::PhysicalName::QuantityType(id) => {
                                Some((count, Value::QuantityType(id)))
                            }
                        }
                    });
                    // A qualified entity is an immutable record, just as a lexical
                    // entity binding is. Resolve its longest declaration prefix before
                    // traversing attributes; the enclosing package is only a scope.
                    let entity_prefix = if lexical {
                        None
                    } else {
                        (2..path.segments.len()).rev().find_map(|count| {
                            let prefix = path.segments[..count]
                                .iter()
                                .map(|s| s.name.as_str())
                                .collect::<Vec<_>>()
                                .join(".");
                            let id = self.package.resolve(self.at, &prefix)?;
                            matches!(self.package.types.get(&id), Some(Type::Entity(_)))
                                .then_some((count, prefix))
                        })
                    };
                    let entity_prefix = entity_prefix
                        .map(|(count, prefix)| {
                            self.reference(&prefix, depth + 1).map(|v| (count, v))
                        })
                        .transpose()?;
                    let (count, mut value) =
                        match definition_prefix.or(physical_prefix).or(entity_prefix) {
                            Some(value) => value,
                            None => (1, self.reference(&first.name, depth + 1)?),
                        };
                    for segment in &path.segments[count..] {
                        if !segment.indices.is_empty() {
                            return Err(invalid(
                                self.at,
                                "attribute values do not have coordinates",
                            ));
                        }
                        value = self.member(value, &segment.name)?;
                    }
                    return Ok(value);
                }
                self.reference(&name, depth + 1)
            }
            ExprKind::Neg(e) => {
                if expected == Some(&Type::Integer)
                    && let ExprKind::Number(n) = &e.kind
                    && n.unit.is_none()
                {
                    return n
                        .integer()
                        .and_then(i128::checked_neg)
                        .and_then(|v| i64::try_from(v).ok())
                        .map(Value::Integer)
                        .ok_or_else(|| invalid(self.at, "exact bounded integer required"));
                }
                let v = self.expr(e, expected, depth + 1)?;
                match v {
                    Value::Integer(v) => v
                        .checked_neg()
                        .map(Value::Integer)
                        .ok_or_else(|| invalid(self.at, "integer overflow")),
                    Value::Number { bits, quantity } | Value::Coordinate { bits, quantity, .. } => {
                        Ok(Value::Number {
                            bits: (-f64::from_bits(bits)).to_bits(),
                            quantity,
                        })
                    }
                    _ => Err(invalid(self.at, "numeric negation")),
                }
            }
            ExprKind::Binary { op, lhs, rhs }
                if crate::expression::chain_operation(*op, rhs)
                    && expected != Some(&Type::Integer) =>
            {
                // A maximal product, quotient and exact-power subtree is evaluated as a
                // whole and typed once at its root (ADR-0124); its inner nodes have no
                // type of their own.
                let result = match self.chain_value(e, depth)? {
                    Numeric::Integer(value) => return Ok(Value::Integer(value)),
                    Numeric::Real(value) => value,
                };
                if !result.is_finite() {
                    return Err(invalid(self.at, "nonfinite static arithmetic"));
                }
                self.typed_result(e, result, expected)
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let additive = matches!(op, BinaryOp::Add | BinaryOp::Sub);
                let operand_expected = if additive || expected == Some(&Type::Integer) {
                    expected
                } else {
                    None
                };
                let a = self.expr(lhs, operand_expected, depth + 1)?;
                let right_expected = if matches!(a, Value::Integer(_)) {
                    Some(Type::Integer)
                } else if additive {
                    value_type(&a)
                } else {
                    None
                };
                let b = self.expr(rhs, right_expected.as_ref(), depth + 1)?;
                if let (Value::Integer(x), Value::Integer(y)) = (&a, &b) {
                    let result = match op {
                        BinaryOp::Add => x.checked_add(*y),
                        BinaryOp::Sub => x.checked_sub(*y),
                        BinaryOp::Mul => x.checked_mul(*y),
                        BinaryOp::Div if *y != 0 && x.checked_rem(*y) == Some(0) => {
                            x.checked_div(*y)
                        }
                        BinaryOp::Pow => u32::try_from(*y).ok().and_then(|y| x.checked_pow(y)),
                        _ => None,
                    };
                    return result.map(Value::Integer).ok_or_else(|| {
                        invalid(self.at, "integer arithmetic overflow or nonintegral result")
                    });
                }
                let x = a.scalar(self.at)?;
                let y = b.scalar(self.at)?;
                let result = match op {
                    BinaryOp::Add => x + y,
                    BinaryOp::Sub => x - y,
                    BinaryOp::Mul => x * y,
                    BinaryOp::Div => x / y,
                    BinaryOp::Pow => x.powf(y),
                };
                if !result.is_finite() {
                    return Err(invalid(self.at, "nonfinite static arithmetic"));
                }
                self.typed_result(e, result, expected)
            }
            ExprKind::Call { function, args } => {
                if args.len() != 1 {
                    return Err(invalid(self.at, "static primitive arity"));
                }
                let value = self.expr(&args[0], None, depth + 1)?.scalar(self.at)?;
                let result = match function.as_str() {
                    "abs" => value.abs(),
                    "sqrt" => value.sqrt(),
                    "exp" => value.exp(),
                    "log" => value.ln(),
                    "sin" => value.sin(),
                    "cos" => value.cos(),
                    _ => return Err(invalid(self.at, "unsupported static primitive")),
                };
                if !result.is_finite() {
                    return Err(invalid(self.at, "static primitive domain violation"));
                }
                let mut types = self.named_types();
                types.extend(
                    self.env
                        .iter()
                        .filter_map(|(n, v)| value_type(v).map(|t| (n.clone(), t))),
                );
                let Type::Quantity(s) = crate::expression::infer(
                    e,
                    &types,
                    self.package,
                    self.physical,
                    self.at,
                    expected,
                )?
                else {
                    return Err(invalid(self.at, "physical primitive result"));
                };
                let quantity = s
                    .resolve_with_evidence(
                        self.physical.quantities,
                        &BTreeMap::new(),
                        self.physical.preconditions,
                    )
                    .map_err(|e| invalid(self.at, e.to_string()))?;
                Ok(Value::Number {
                    bits: result.to_bits(),
                    quantity,
                })
            }
            ExprKind::Reduce { kind, binder, body } => {
                if *kind == dsl::ReduceKind::Integral {
                    return Err(invalid(
                        self.at,
                        "continuous integral is not a static requirement",
                    ));
                }
                let Value::Set(values) =
                    self.reference(&dsl::render_path(&binder.domain), depth + 1)?
                else {
                    return Err(invalid(self.at, "finite reduction set required"));
                };
                if values.len() > self.limit {
                    return Err(invalid(self.at, "static reduction extent"));
                }
                let mut types = self.named_types();
                types.extend(
                    self.env
                        .iter()
                        .filter_map(|(n, v)| value_type(v).map(|t| (n.clone(), t))),
                );
                let ty = crate::expression::infer(
                    e,
                    &types,
                    self.package,
                    self.physical,
                    self.at,
                    expected,
                )?;
                let Type::Set(element) = crate::expression::infer(
                    &Expr {
                        kind: ExprKind::Path(binder.domain.clone()),
                        span: dsl::Span::default(),
                    },
                    &types,
                    self.package,
                    self.physical,
                    self.at,
                    None,
                )?
                else {
                    return Err(invalid(self.at, "finite reduction domain type required"));
                };
                types.insert(binder.var.clone(), *element);
                let prototype = crate::expression::infer(
                    body,
                    &types,
                    self.package,
                    self.physical,
                    self.at,
                    None,
                )?;
                let prototype_quantity = if let Type::Quantity(s) = &prototype {
                    Some(
                        s.resolve_with_evidence(
                            self.physical.quantities,
                            &BTreeMap::new(),
                            self.physical.preconditions,
                        )
                        .map_err(|e| invalid(self.at, e.to_string()))?,
                    )
                } else {
                    None
                };
                let product = *kind == dsl::ReduceKind::Prod;
                let mut result = match &ty {
                    Type::Integer => Value::Integer(i64::from(product)),
                    Type::Quantity(s) => Value::Number {
                        bits: if product {
                            1.0_f64.to_bits()
                        } else {
                            0.0_f64.to_bits()
                        },
                        quantity: s
                            .resolve_with_evidence(
                                self.physical.quantities,
                                &BTreeMap::new(),
                                self.physical.preconditions,
                            )
                            .map_err(|e| invalid(self.at, e.to_string()))?,
                    },
                    _ => return Err(invalid(self.at, "numeric reduction required")),
                };
                let nested_limit = self.limit / values.len().max(1);
                for value in values {
                    let mut env = self.env.clone();
                    env.insert(binder.var.clone(), value);
                    let mut evaluator = Evaluator {
                        package: self.package,
                        physical: self.physical,
                        at: self.at,
                        env: &env,
                        limit: nested_limit,
                        stack: self.stack.clone(),
                        reader: self.reader,
                        selections: self.selections,
                    };
                    if let Some(filter) = &binder.filter
                        && !evaluator.predicate(filter)?
                    {
                        continue;
                    }
                    let value = evaluator.expr(body, Some(&prototype), depth + 1)?;
                    match (&mut result, value) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            *a = if product {
                                a.checked_mul(b)
                            } else {
                                a.checked_add(b)
                            }
                            .ok_or_else(|| invalid(self.at, "static reduction integer overflow"))?
                        }
                        (
                            Value::Number { bits, .. },
                            Value::Number {
                                bits: b,
                                quantity: q,
                            },
                        ) if prototype_quantity == Some(q) => {
                            let value = if product {
                                f64::from_bits(*bits) * f64::from_bits(b)
                            } else {
                                f64::from_bits(*bits) + f64::from_bits(b)
                            };
                            if !value.is_finite() {
                                return Err(invalid(self.at, "nonfinite static reduction"));
                            }
                            *bits = value.to_bits();
                        }
                        _ => return Err(invalid(self.at, "static reduction element type")),
                    }
                }
                Ok(result)
            }
            ExprKind::Fold {
                accumulator,
                item,
                binder,
                value,
                step,
            } => {
                let Value::Set(members) =
                    self.reference(&dsl::render_path(&binder.domain), depth + 1)?
                else {
                    return Err(invalid(self.at, "fold requires finite membership"));
                };
                if members.len() > self.limit {
                    return Err(invalid(self.at, "static fold extent"));
                }
                let mut types = self.named_types();
                types.extend(
                    self.env
                        .iter()
                        .filter_map(|(n, v)| value_type(v).map(|t| (n.clone(), t))),
                );
                let ty = crate::expression::infer(
                    e,
                    &types,
                    self.package,
                    self.physical,
                    self.at,
                    expected,
                )?;
                let limit = self.limit / members.len().max(1);
                let mut result = None;
                for member in members {
                    let mut env = self.env.clone();
                    env.insert(binder.var.clone(), member);
                    let mut evaluator = Evaluator {
                        package: self.package,
                        physical: self.physical,
                        at: self.at,
                        env: &env,
                        limit,
                        stack: self.stack.clone(),
                        reader: self.reader,
                        selections: self.selections,
                    };
                    if let Some(filter) = &binder.filter
                        && !evaluator.predicate(filter)?
                    {
                        continue;
                    }
                    let current = evaluator.expr(value, Some(&ty), depth + 1)?;
                    result = Some(if let Some(previous) = result {
                        env.insert(accumulator.clone(), previous);
                        env.insert(item.clone(), current);
                        Evaluator {
                            package: self.package,
                            physical: self.physical,
                            at: self.at,
                            env: &env,
                            limit,
                            stack: self.stack.clone(),
                            reader: self.reader,
                            selections: self.selections,
                        }
                        .expr(step, Some(&ty), depth + 1)?
                    } else {
                        current
                    });
                }
                if result.is_none()
                    && matches!(ty, Type::Set(_))
                    && let ExprKind::NamedCall { name, args } = &step.kind
                {
                    let plain = |e: &Expr, n: &str| {
                        matches!(&e.kind,ExprKind::Path(p)
                        if p.segments.len()==1 && p.segments[0].name==n && p.segments[0].indices.is_empty())
                    };
                    if name == "union"
                        && args.len() == 2
                        && plain(&args[0], accumulator)
                        && plain(&args[1], item)
                    {
                        return Ok(Value::Set(Vec::new()));
                    }
                }
                result.ok_or_else(|| invalid(self.at, "fold requires at least one selected member"))
            }
            ExprKind::Let { bindings, body } => {
                let mut env = self.env.clone();
                for (name, value) in bindings {
                    let value = Evaluator {
                        package: self.package,
                        physical: self.physical,
                        at: self.at,
                        env: &env,
                        limit: self.limit,
                        stack: self.stack.clone(),
                        reader: self.reader,
                        selections: self.selections,
                    }
                    .expr(value, None, depth + 1)?;
                    env.insert(name.clone(), value);
                }
                Evaluator {
                    package: self.package,
                    physical: self.physical,
                    at: self.at,
                    env: &env,
                    limit: self.limit,
                    stack: self.stack.clone(),
                    reader: self.reader,
                    selections: self.selections,
                }
                .expr(body, expected, depth + 1)
            }
            ExprKind::Conditional {
                guard,
                then,
                otherwise,
            } => {
                let selected = self.predicate(guard)?;
                self.expr(if selected { then } else { otherwise }, expected, depth + 1)
            }
            ExprKind::NamedCall { name, args } => self.builtin(name, args, depth + 1),
            _ => Err(invalid(
                self.at,
                "only finite static expressions may select structure",
            )),
        }
    }
    fn builtin(&mut self, name: &str, args: &[Expr], depth: usize) -> Result<Value> {
        if matches!(name, "implements" | "provides") {
            if args.len() != 2 {
                return Err(invalid(self.at, "capability query arity"));
            }
            let target = self.expr(&args[0], None, depth + 1)?;
            let Value::Definition { id, .. } = target else {
                return Err(invalid(self.at, "capability target"));
            };
            let ExprKind::Path(path) = &args[1].kind else {
                return Err(invalid(self.at, "capability name"));
            };
            let member = dsl::render_path(path);
            let present = if name == "implements" {
                self.package
                    .resolve(self.at, &member)
                    .is_some_and(|contract| {
                        contract == id
                            || self
                                .package
                                .interfaces
                                .get(&id)
                                .is_some_and(|v| v.contains(&contract))
                    })
            } else {
                self.package
                    .members
                    .get(&id)
                    .is_some_and(|v| v.contains_key(&member))
            };
            return Ok(Value::Boolean(present));
        }
        if matches!(name, "tuple" | "set_of") {
            if args.len() > self.limit {
                return Err(invalid(self.at, "structural constructor budget"));
            }
            let mut values = Vec::new();
            for arg in args {
                let value = self.expr(arg, None, depth + 1)?;
                if name == "tuple" || !values.contains(&value) {
                    values.push(value);
                }
            }
            return Ok(if name == "tuple" {
                Value::Tuple(values)
            } else {
                Value::Set(values)
            });
        }
        if name == "selection_compatible" {
            if args.len() != 3 {
                return Err(invalid(self.at, "selection_compatible arity"));
            }
            let Value::Set(records) = self.expr(&args[0], None, depth + 1)? else {
                return Err(invalid(self.at, "selection records required"));
            };
            let context = self.expr(&args[1], None, depth + 1)?;
            let function = self.expr(&args[2], None, depth + 1)?;
            let mut slots = BTreeMap::new();
            for record in records {
                let slot = self.structural_call(
                    function.clone(),
                    &[record.clone(), context.clone()],
                    depth + 1,
                )?;
                if !matches!(slot, Value::Tuple(_)) {
                    return Err(invalid(self.at, "selection slot tuple required"));
                }
                if slots
                    .insert(slot, record.clone())
                    .is_some_and(|previous| previous != record)
                {
                    return Ok(Value::Boolean(false));
                }
            }
            return Ok(Value::Boolean(true));
        }
        if name == "selection_closure" {
            if args.len() != 3 {
                return Err(invalid(self.at, "selection_closure arity"));
            }
            let Value::Set(roots) = self.expr(&args[0], None, depth + 1)? else {
                return Err(invalid(self.at, "selection roots are a finite set"));
            };
            let context = self.expr(&args[1], None, depth + 1)?;
            let Value::Function(id) = self.expr(&args[2], None, depth + 1)? else {
                return Err(invalid(self.at, "selection dependency function required"));
            };
            let function = self
                .package
                .functions
                .get(&id)
                .ok_or_else(|| invalid(self.at, "selection dependency function is admitted"))?;
            if function.arguments.len() != 2 || function.external.is_some() {
                return Err(invalid(
                    id,
                    "selection dependencies use a two-argument authored function",
                ));
            }
            let body = function
                .body
                .as_ref()
                .ok_or_else(|| invalid(id, "selection dependency function has a body"))?;
            let closure = crate::scientific_selection::close(
                roots,
                context,
                self.at,
                self.limit,
                |record, context| {
                    let env = BTreeMap::from([
                        (function.arguments[0].0.clone(), record.clone()),
                        (function.arguments[1].0.clone(), context.clone()),
                    ]);
                    if !conforms(record, &function.arguments[0].1, self.package)
                        || !conforms(context, &function.arguments[1].1, self.package)
                    {
                        return Err(invalid(id, "selection dependency argument type"));
                    }
                    let mut evaluator = Evaluator {
                        package: self.package,
                        physical: self.physical,
                        at: id,
                        env: &env,
                        limit: self.limit,
                        stack: self.stack.clone(),
                        reader: self.reader,
                        selections: self.selections,
                    };
                    if let Some(validity) = &function.validity
                        && !evaluator.predicate(validity)?
                    {
                        return Err(invalid(id, "selection dependency function domain"));
                    }
                    let value = evaluator.expr(body, Some(&function.result), depth + 1)?;
                    if !conforms(&value, &function.result, self.package) {
                        return Err(invalid(id, "selection dependency result type"));
                    }
                    let Value::Set(records) = value else {
                        return Err(invalid(id, "selection dependencies return a finite set"));
                    };
                    Ok(records)
                },
            )?;
            if let Some(selections) = self.selections {
                let occurrence = crate::scientific_selection::SelectionOccurrence {
                    declaration: self.at,
                    dependencies: id,
                    expression: args
                        .iter()
                        .map(dsl::render_expr)
                        .collect::<Vec<_>>()
                        .join(","),
                    roots: closure.roots.clone(),
                    context: closure.context.clone(),
                };
                selections.record(occurrence, closure.clone());
            }
            return Ok(Value::Set(closure.records));
        }
        if name == "keys" {
            if args.len() != 1 {
                return Err(invalid(self.at, "keys arity"));
            }
            let ExprKind::Path(path) = &args[0].kind else {
                return Err(invalid(self.at, "keys requires a table declaration"));
            };
            let id = self
                .package
                .resolve(self.at, &dsl::render_path(path))
                .ok_or_else(|| invalid(self.at, "unknown table"))?;
            let table = self
                .package
                .tables
                .get(&id)
                .ok_or_else(|| invalid(self.at, "keys requires an admitted table"))?;
            if table.rows.len() > self.limit {
                return Err(invalid(self.at, "table key extent exceeds budget"));
            }
            return Ok(Value::Set(
                table
                    .keys_read(self.package, id, self.at)?
                    .map(|k| {
                        if k.len() == 1 {
                            k[0].clone()
                        } else {
                            Value::Tuple(k.clone())
                        }
                    })
                    .collect(),
            ));
        }
        let values = args
            .iter()
            .enumerate()
            .map(|(i, e)| {
                self.expr(
                    e,
                    if name == "at" && i == 1 {
                        Some(&Type::Integer)
                    } else {
                        None
                    },
                    depth + 1,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        match (name, values.as_slice()) {
            ("at", [Value::Tuple(values), Value::Integer(index)]) => usize::try_from(*index)
                .ok()
                .and_then(|i| values.get(i))
                .cloned()
                .ok_or_else(|| invalid(self.at, "tuple coordinate out of bounds")),
            ("require_present", [Value::Missing]) => Err(invalid(self.at,"required optional value is absent")),
            ("require_present", [value]) => Ok(value.clone()),
            ("present", [value]) => Ok(Value::Boolean(!matches!(value, Value::Missing))),
            ("size", [Value::Set(values)]) => Ok(Value::Integer(values.len() as i64)),
            ("union", [Value::Set(a), Value::Set(b)]) => {
                let mut result = a.clone();
                for v in b {
                    if !result.contains(v) {
                        result.push(v.clone());
                    }
                }
                if result.len() > self.limit {
                    return Err(invalid(self.at, "set budget"));
                }
                Ok(Value::Set(result))
            }
            ("product", [Value::Set(a), Value::Set(b)]) => {
                let count = a
                    .len()
                    .checked_mul(b.len())
                    .ok_or_else(|| invalid(self.at, "product overflow"))?;
                if count > self.limit {
                    return Err(invalid(self.at, "product budget"));
                }
                Ok(Value::Set(
                    a.iter()
                        .flat_map(|x| {
                            b.iter()
                                .map(move |y| Value::Tuple(vec![x.clone(), y.clone()]))
                        })
                        .collect(),
                ))
            }
            _ => {
                if let Ok(function @ Value::Function(_)) = self.reference(name, depth + 1) {
                    return self.structural_call(function, &values, depth + 1);
                }
                if let Ok(path) = dsl::parse_expr(name)
                    && matches!(path.kind, ExprKind::Path(_))
                    && let Ok(function @ Value::Function(_)) = self.expr(&path, None, depth + 1)
                {
                    return self.structural_call(function, &values, depth + 1);
                }
                let id = self
                    .package
                    .resolve(self.at, name)
                    .ok_or_else(|| invalid(self.at, format!("unknown static operation {name}")))?;
                if values.is_empty()
                    && matches!(
                        self.package.types.get(&id),
                        Some(Type::Definition(_) | Type::Interface(_))
                    )
                {
                    Ok(Value::Definition {
                        id,
                        bindings: BTreeMap::new(),
                    })
                } else {
                    Err(invalid(self.at, "unsupported static operation"))
                }
            }
        }
    }
    fn structural_call(
        &mut self,
        selected: Value,
        values: &[Value],
        depth: usize,
    ) -> Result<Value> {
        let Value::Function(id) = selected else {
            return Err(invalid(self.at, "structural callback function required"));
        };
        if depth > 64 || self.stack.contains(&id) {
            return Err(invalid(id, "recursive structural callback"));
        }
        let function = self
            .package
            .functions
            .get(&id)
            .ok_or_else(|| invalid(id, "admitted callback required"))?;
        if function.external.is_some()
            || function.arguments.len() != values.len()
            || !matches!(
                function.result,
                Type::Tuple(_) | Type::Set(_) | Type::Boolean | Type::Integer
            )
        {
            return Err(invalid(
                id,
                "structural callbacks return finite structural values",
            ));
        }
        let env = function
            .arguments
            .iter()
            .zip(values)
            .map(|((name, ty), value)| {
                if !conforms(value, ty, self.package) {
                    return Err(invalid(id, "structural callback argument type"));
                }
                Ok((name.clone(), value.clone()))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let mut stack = self.stack.clone();
        stack.push(id);
        let mut evaluator = Evaluator {
            package: self.package,
            physical: self.physical,
            at: id,
            env: &env,
            limit: self.limit,
            stack,
            reader: self.reader,
            selections: self.selections,
        };
        if let Some(validity) = &function.validity
            && !evaluator.predicate(validity)?
        {
            return Err(invalid(id, "structural callback domain"));
        }
        let body = function
            .body
            .as_ref()
            .ok_or_else(|| invalid(id, "structural callback body required"))?;
        let value = evaluator.expr(body, Some(&function.result), depth + 1)?;
        if !conforms(&value, &function.result, self.package) {
            return Err(invalid(id, "structural callback result type"));
        }
        Ok(value)
    }
    pub(crate) fn predicate(&mut self, p: &Predicate) -> Result<bool> {
        match &p.kind {
            PredicateKind::Bool(v) => Ok(*v),
            PredicateKind::Atom(e) => match self.expr(e, Some(&Type::Boolean), 0)? {
                Value::Boolean(v) => Ok(v),
                _ => Err(invalid(self.at, "Boolean condition required")),
            },
            PredicateKind::Not(p) => Ok(!self.predicate(p)?),
            PredicateKind::And(a, b) => Ok(self.predicate(a)? && self.predicate(b)?),
            PredicateKind::Or(a, b) => Ok(self.predicate(a)? || self.predicate(b)?),
            PredicateKind::Compare { op, lhs, rhs } => {
                let a = self.expr(lhs, None, 0)?;
                let b = self.expr(rhs, value_type(&a).as_ref(), 0)?;
                let order = match (&a, &b) {
                    (Value::Integer(a), Value::Integer(b)) => Some(a.cmp(b)),
                    (
                        Value::Number { .. } | Value::Coordinate { .. },
                        Value::Number { .. } | Value::Coordinate { .. },
                    ) => a.scalar(self.at)?.partial_cmp(&b.scalar(self.at)?),
                    _ => Some(a.cmp(&b)),
                }
                .ok_or_else(|| invalid(self.at, "unordered static comparison"))?;
                Ok(match op {
                    CompareOp::Eq => order.is_eq(),
                    CompareOp::NotEq => !order.is_eq(),
                    CompareOp::Lt => order.is_lt(),
                    CompareOp::Le => !order.is_gt(),
                    CompareOp::Gt => order.is_gt(),
                    CompareOp::Ge => !order.is_lt(),
                })
            }
            PredicateKind::In { expr, domain } => {
                let value = self.expr(expr, None, 0)?;
                // A declared set, or a set-valued member such as an entity's attribute.
                let domain = self.expr(
                    &Expr {
                        kind: ExprKind::Path(domain.clone()),
                        span: dsl::Span::default(),
                    },
                    None,
                    0,
                )?;
                match domain {
                    Value::Set(v) => Ok(v.contains(&value)),
                    _ => Err(invalid(self.at, "membership requires finite set")),
                }
            }
            PredicateKind::Null => Err(invalid(self.at, "unknown static predicate")),
        }
    }
}
/// A number literal typed against `expected` and converted to its type's canonical unit,
/// with the scale that conversion applies to a difference: the one reading of a literal
/// shared by static evaluation and data cells (ADR-0123 Outcome 1).
pub(crate) fn number(
    physical: &TypeContext<'_>,
    at: DeclarationId,
    n: &dsl::Number,
    expected: Option<&Type>,
) -> Result<(Value, f64)> {
    if n.unit.is_none() && matches!(expected, Some(Type::Integer)) {
        return n
            .integer()
            .and_then(|v| i64::try_from(v).ok())
            .map(|v| (Value::Integer(v), 1.))
            .ok_or_else(|| invalid(at, "exact bounded integer required"));
    }
    let literal_type = crate::expression::number_type(n, physical, at, expected)?;
    let quantity = literal_type
        .quantity_scheme()
        .ok_or_else(|| invalid(at, "concrete literal type required"))?
        .resolve_with_evidence(
            physical.quantities,
            &BTreeMap::new(),
            physical.preconditions,
        )
        .map_err(|error| invalid(at, error.to_string()))?;
    let conversion = if let Some(unit) = &n.unit {
        pse_quantity::CanonicalConversionPlan::composed(physical.quantities, quantity, unit)
    } else {
        pse_quantity::CanonicalConversionPlan::canonical(physical.quantities, quantity)
    }
    .map_err(|e| invalid(at, e.to_string()))?;
    let value = conversion
        .apply(n.value)
        .map_err(|e| invalid(at, e.to_string()))?;
    Ok((
        Value::Number {
            bits: value.bits(),
            quantity: value.quantity(),
        },
        conversion.scale(),
    ))
}
/// A static chain value before its root is typed.
#[derive(Clone, Copy)]
enum Numeric {
    Integer(i64),
    Real(f64),
}
impl Numeric {
    fn real(self) -> f64 {
        match self {
            Self::Integer(value) => value as f64,
            Self::Real(value) => value,
        }
    }
}
pub(crate) fn value_type(value: &Value) -> Option<Type> {
    match value {
        Value::Boolean(_) => Some(Type::Boolean),
        Value::Integer(_) => Some(Type::Integer),
        Value::Number { quantity, .. } | Value::Coordinate { quantity, .. } => {
            Some(Type::Quantity(Scheme::Concrete(*quantity)))
        }
        Value::Text(_) => Some(Type::Text),
        Value::Entity { kind, .. } => Some(Type::Entity(*kind)),
        Value::Enum { enumeration, .. } => Some(Type::Enum(*enumeration)),
        Value::Identifier { scheme, .. } => Some(Type::Identifier(*scheme)),
        Value::Definition { id, .. } => Some(Type::Definition(*id)),
        Value::Set(v) => v
            .first()
            .and_then(value_type)
            .map(|t| Type::Set(Box::new(t))),
        Value::Row { table, .. } => Some(Type::Row(*table)),
        Value::Tuple(v) => v
            .iter()
            .map(value_type)
            .collect::<Option<Vec<_>>>()
            .map(Type::Tuple),
        Value::QuantityType(_) => Some(Type::QuantityType),
        Value::ReferenceState(_) => Some(Type::ReferenceState),
        Value::Function(_) | Value::Missing => None,
    }
}

pub(crate) fn conforms(value: &Value, ty: &Type, p: &CheckedPackage) -> bool {
    match (value, ty) {
        (Value::Function(id), Type::Function { .. }) => p.types.get(id) == Some(ty),
        (Value::Missing, Type::Optional(_)) => true,
        (value, Type::Optional(inner)) => conforms(value, inner, p),
        (Value::Set(values), Type::Set(element)) => values.iter().all(|v| conforms(v, element, p)),
        (Value::Tuple(values), Type::Tuple(types)) => {
            values.len() == types.len() && values.iter().zip(types).all(|(v, t)| conforms(v, t, p))
        }
        (Value::Definition { id, .. }, Type::Interface(contract)) => {
            id == contract || p.interfaces.get(id).is_some_and(|s| s.contains(contract))
        }
        (Value::Definition { id, .. }, Type::Definition(contract)) => {
            matches!((p.preset_definition(*id), p.preset_definition(*contract)),
                (Ok(actual), Ok(expected)) if actual == expected)
        }
        // A member of a refined kind is a member of every kind it refines.
        (Value::Entity { kind, .. }, Type::Entity(expected)) => p.refines(*kind, *expected),
        _ => value_type(value).as_ref() == Some(ty),
    }
}
