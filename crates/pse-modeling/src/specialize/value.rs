// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded compile-time values. This interpreter selects structure; runtime math uses pse-math.
use crate::{CheckedPackage, Result, Type, TypeContext, invalid};
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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    /// Predicate.
    Boolean(bool),
    /// Exact integer.
    Integer(i64),
    /// Finite scalar with its complete physical type.
    Number { bits: u64, quantity: QuantityTypeId },
    /// Generated continuous coordinate; identity is independent of its floating value.
    Coordinate {
        id: SemanticId,
        bits: u64,
        quantity: QuantityTypeId,
    },
    /// Textual label.
    Text(String),
    /// Entity identity and kind identity.
    Entity { id: SemanticId, kind: SemanticId },
    /// Value of an authored closed enumeration.
    Enum {
        enumeration: SemanticId,
        member: String,
    },
    /// Definition identity with named partial bindings.
    Definition {
        id: SemanticId,
        bindings: BTreeMap<String, Value>,
    },
    /// A pure function declaration selected as structural data.
    Function(SemanticId),
    /// Typed heterogeneous table row.
    Row {
        table: SemanticId,
        fields: BTreeMap<String, Value>,
    },
    /// Ordered finite set with no duplicates.
    Set(Vec<Value>),
    /// Ragged coordinate or table key.
    Tuple(Vec<Value>),
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
                h.str("function").id(id);
            }
            Self::Number { bits, quantity } => {
                h.str("number").u64(*bits).id(&quantity.as_id());
            }
            Self::Text(v) => {
                h.str("text").str(v);
            }
            Self::Entity { id, kind } => {
                h.str("entity").id(id).id(kind);
            }
            Self::Enum {
                enumeration,
                member,
            } => {
                h.str("enum").id(enumeration).str(member);
            }
            Self::Definition { id, bindings } => {
                h.str("definition").id(id);
                for (n, v) in bindings {
                    h.str(n);
                    v.frame(h);
                }
            }
            Self::Row { table, fields } => {
                h.str("row").id(table);
                for (n, v) in fields {
                    h.str(n);
                    v.frame(h);
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
            Self::Missing => {
                h.str("missing");
            }
        }
    }
    /// Stable coordinate identity, preserving compound-key identity.
    pub fn identity(&self) -> SemanticId {
        if let Self::Entity { id, .. } | Self::Coordinate { id, .. } = self {
            return *id;
        }
        let mut h = FramedHasher::new(pse_ids::Frame::ModelingCoordinateV1);
        self.frame(&mut h);
        h.finish_id()
    }
    pub(crate) fn scalar(&self, at: SemanticId) -> Result<f64> {
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
    pub package: &'a CheckedPackage,
    pub physical: &'a TypeContext<'b>,
    pub at: SemanticId,
    pub env: &'a Environment,
    pub limit: usize,
    pub stack: Vec<SemanticId>,
}
impl Evaluator<'_, '_> {
    pub fn text(&mut self, text: &str, expected: Option<&Type>) -> Result<Value> {
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
        if let Some(expected) = expected {
            if !conforms(&value, expected, self.package) {
                return Err(invalid(
                    self.at,
                    format!("static value does not satisfy {expected:?}"),
                ));
            }
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
                        &parameter.type_name,
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
    fn reference(&mut self, name: &str, depth: usize) -> Result<Value> {
        if depth > 64 {
            return Err(invalid(self.at, "static reference depth"));
        }
        if let Some(v) = self.env.get(name) {
            return Ok(v.clone());
        }
        let id = self
            .package
            .resolve(self.at, name)
            .ok_or_else(|| invalid(self.at, format!("missing static binding {name}")))?;
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
            crate::Selected::Function(_) => Ok(Value::Function(id)),
            crate::Selected::Entity(e) => {
                let kind = self
                    .package
                    .resolve(id, &e.kind_name)
                    .ok_or_else(|| invalid(id, "entity kind"))?;
                Ok(Value::Entity { id, kind })
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
        value
    }
    pub fn expr(&mut self, e: &Expr, expected: Option<&Type>, depth: usize) -> Result<Value> {
        if depth > 64 {
            return Err(invalid(self.at, "static expression depth"));
        }
        match &e.kind {
            ExprKind::Number(n) => {
                if n.unit.is_none() && matches!(expected, Some(Type::Integer)) {
                    return n
                        .integer()
                        .and_then(|v| i64::try_from(v).ok())
                        .map(Value::Integer)
                        .ok_or_else(|| invalid(self.at, "exact bounded integer required"));
                }
                let ty = crate::expression::infer(
                    e,
                    &BTreeMap::new(),
                    self.package,
                    self.physical,
                    self.at,
                    expected,
                )?;
                let Type::Quantity(Scheme::Concrete(quantity)) = ty else {
                    return Err(invalid(self.at, "concrete literal type required"));
                };
                let mut value = n.value;
                if let Some(unit) = &n.unit {
                    let source = self
                        .physical
                        .quantities
                        .unit_by_symbol(unit)
                        .ok_or_else(|| invalid(self.at, "literal unit"))?;
                    let target = self
                        .physical
                        .quantities
                        .quantity_type(quantity)
                        .map_err(|e| invalid(self.at, e.to_string()))?;
                    let conversion = pse_quantity::convert_spec_for_type(
                        source,
                        self.physical
                            .quantities
                            .unit(target.canonical_unit)
                            .map_err(|e| invalid(self.at, e.to_string()))?,
                        &target.key,
                    )
                    .map_err(|e| invalid(self.at, e.to_string()))?;
                    value = pse_quantity::convert_value(&conversion, value);
                }
                Ok(Value::Number {
                    bits: value.to_bits(),
                    quantity,
                })
            }
            ExprKind::Path(path) => {
                let name = dsl::render_path(path);
                if name == "true" {
                    return Ok(Value::Boolean(true));
                }
                if name == "false" {
                    return Ok(Value::Boolean(false));
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
                        .map(|(e, ty)| {
                            let v = self.expr(e, Some(ty), depth + 1)?;
                            if !conforms(&v, ty, self.package) {
                                return Err(invalid(id, "table key type"));
                            }
                            Ok(v)
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let value = table
                        .rows
                        .get(&keys)
                        .cloned()
                        .or_else(|| table.default.clone())
                        .or_else(|| table.optional.then_some(Value::Missing))
                        .ok_or_else(|| invalid(id, "required table value absent"))?;
                    if path.segments.len() == position + 1 {
                        return Ok(value);
                    }
                    if path.segments.len() == position + 2
                        && path.segments[position + 1].indices.is_empty()
                        && let Value::Row { fields, .. } = value
                    {
                        return fields
                            .get(&path.segments[position + 1].name)
                            .cloned()
                            .ok_or_else(|| invalid(id, "unknown table column"));
                    }
                    return Err(invalid(id, "invalid table row member access"));
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
                    // A qualified declaration path is resolved before entity attribute traversal.
                    // Package names are scopes, not runtime values to be evaluated.
                    if definition_prefix.is_none() && self.package.resolve(self.at, &name).is_some()
                    {
                        return self.reference(&name, depth + 1);
                    }
                    let enum_name = path.segments[..path.segments.len() - 1]
                        .iter()
                        .map(|segment| segment.name.as_str())
                        .collect::<Vec<_>>()
                        .join(".");
                    if let Some(id) = self.package.resolve(self.at, &enum_name)
                        && let Some(values) = &self.package.declarations[&id].value.enumeration
                    {
                        let member = &path.segments[path.segments.len() - 1].name;
                        if values.members.contains(member) {
                            return Ok(Value::Enum {
                                enumeration: id,
                                member: member.clone(),
                            });
                        }
                        return Err(invalid(id, "unknown enumeration member"));
                    }
                    let (count, mut value) = match definition_prefix {
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
                        value = match value {
                            Value::Definition { id, bindings } => {
                                self.definition_member(id, &bindings, &segment.name)?
                            }
                            Value::Row { fields, .. } => fields
                                .get(&segment.name)
                                .cloned()
                                .ok_or_else(|| invalid(self.at, "unknown table column"))?,
                            Value::Entity { id, kind } => {
                                let member = self
                                    .package
                                    .members
                                    .get(&kind)
                                    .and_then(|m| m.get(&segment.name))
                                    .copied()
                                    .ok_or_else(|| invalid(id, "unknown entity attribute"))?;
                                let entity = self.package.declarations[&id]
                                    .value
                                    .entity
                                    .as_ref()
                                    .ok_or_else(|| invalid(id, "entity payload"))?;
                                let explicit = entity
                                    .attributes
                                    .iter()
                                    .find(|a| a.name == segment.name)
                                    .map(|a| a.expression.as_str());
                                let source = explicit
                                    .or_else(|| {
                                        self.package.declarations[&member]
                                            .value
                                            .binding
                                            .as_ref()
                                            .and_then(|b| b.expression.as_deref())
                                    })
                                    .ok_or_else(|| invalid(id, "missing entity attribute"))?;
                                if self.stack.contains(&member) {
                                    return Err(invalid(member, "recursive entity attribute"));
                                }
                                self.stack.push(member);
                                let saved = self.at;
                                self.at = if explicit.is_some() { id } else { member };
                                let result = self.text(source, self.package.types.get(&member));
                                self.at = saved;
                                self.stack.pop();
                                result?
                            }
                            Value::Missing => {
                                return Err(invalid(
                                    self.at,
                                    "missing optional row must be guarded",
                                ));
                            }
                            _ => return Err(invalid(self.at, "static value has no named member")),
                        };
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
                let env = self
                    .env
                    .iter()
                    .filter_map(|(n, v)| value_type(v).map(|t| (n.clone(), t)))
                    .collect();
                let ty = crate::expression::infer(
                    e,
                    &env,
                    self.package,
                    self.physical,
                    self.at,
                    expected,
                )?;
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
                let mut types = self.package.named_types(self.at);
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
                let mut types = self.package.named_types(self.at);
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
                let mut types = self.package.named_types(self.at);
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
                        }
                        .expr(step, Some(&ty), depth + 1)?
                    } else {
                        current
                    });
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
                    .rows
                    .keys()
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
    pub fn predicate(&mut self, p: &Predicate) -> Result<bool> {
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
                let domain = self.reference(&dsl::render_path(domain), 0)?;
                match domain {
                    Value::Set(v) => Ok(v.contains(&value)),
                    _ => Err(invalid(self.at, "membership requires finite set")),
                }
            }
            PredicateKind::Null => Err(invalid(self.at, "unknown static predicate")),
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
        _ => value_type(value).as_ref() == Some(ty),
    }
}
