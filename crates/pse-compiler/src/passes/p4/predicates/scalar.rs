// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{
    BTreeMap, CompilerError, Inventory, Keyed, NodeId, Payload, SemanticId, Support, Truth,
    ValueRef, invalid, inventory, n,
};
mod paths;
use crate::mathir_relations::{domain::DomainValue, syntax};
use pse_mathir::Opcode;
use pse_quantity::{QuantityTypeId, UnitId};
use pse_schema::math::TemplateValueKind;

#[derive(Clone, Debug)]
pub(super) enum Value {
    Unknown,
    Bool(bool),
    Integer(i128),
    Real {
        value: f64,
        unit: Option<UnitId>,
        quantity: Option<QuantityTypeId>,
    },
    Text(String),
    Enum(SemanticId, String),
    Id(SemanticId),
}
impl Value {
    pub(super) fn truth(self) -> Result<Truth, CompilerError> {
        match self {
            Self::Unknown => Ok(Truth::Unknown),
            Self::Bool(value) => Ok(value.into()),
            _ => Err(invalid(
                "predicate atom requires actual Boolean configuration",
            )),
        }
    }
    fn exact_real(&self) -> Result<f64, CompilerError> {
        match self {
            Self::Integer(value) => {
                let int = i64::try_from(*value)
                    .map_err(|_| invalid("integer is not exactly representable as Float64"))?;
                pse_quantity::numeric::exact_f64_from_i64(int)
                    .ok_or_else(|| invalid("integer is not exactly representable as Float64"))
            }
            Self::Real { value, .. } => Ok(*value),
            _ => Err(invalid("arithmetic requires a numeric operand")),
        }
    }
}
pub(super) struct Evaluation<'a, 'r> {
    pub inventory: &'a Inventory<'r>,
    pub graph: &'a pse_mathir::relations::LoadedMath,
    pub instance: &'a n::instance_bindings::Row,
    pub source_id: SemanticId,
    pub bindings: &'a BTreeMap<SemanticId, SemanticId>,
    pub support: &'a mut Support,
    pub cancel: &'a pse_ids::CancellationToken,
}
impl Evaluation<'_, '_> {
    pub(super) fn operand(&mut self, value: &syntax::Operand) -> Result<Value, CompilerError> {
        match value.selected()? {
            syntax::OperandSelected::EnumLiteral(value) => {
                Ok(Value::Enum(value.enum_id, value.member.clone()))
            }
            syntax::OperandSelected::Expression(value) => self.expression(NodeId(value.node_id)),
        }
    }
    pub(super) fn contains(
        &mut self,
        domain: &n::predicate_nodes::NormalizedPredicateNodesFieldValueInDomain,
        value: &Value,
    ) -> Result<Truth, CompilerError> {
        if matches!(value, Value::Unknown) {
            return Ok(Truth::Unknown);
        }
        let domain = self
            .inventory
            .domain(self.instance, &domain.domain_ref()?, self.support)?;
        let members = self.inventory.members(domain, self.support)?;
        for member in &members {
            self.support.insert(self.inventory.origin(member)?);
        }
        match value {
            Value::Id(id) => Ok(members.iter().any(|row| row.row.member_id == *id).into()),
            Value::Integer(value) => {
                let integer =
                    i64::try_from(*value).map_err(|_| invalid("coordinate exceeds Int64"))?;
                Ok(members
                    .iter()
                    .any(|row| {
                        row.row
                            .coordinate
                            .and_then(pse_quantity::numeric::exact_i64_from_f64)
                            == Some(integer)
                    })
                    .into())
            }
            _ => Err(invalid(
                "domain membership requires an actual member or exact coordinate",
            )),
        }
    }
    #[expect(
        clippy::too_many_lines,
        reason = "expression keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) fn expression(&mut self, old: NodeId) -> Result<Value, CompilerError> {
        let root = *self
            .graph
            .node_mapping
            .get(&old)
            .ok_or_else(|| invalid("predicate scalar node missing"))?;
        let reverse = self
            .graph
            .node_mapping
            .iter()
            .map(|(old, new)| (*new, *old))
            .collect::<BTreeMap<_, _>>();
        let mut values = BTreeMap::new();
        let mut pending = vec![(root, false)];
        while let Some((id, ready)) = pending.pop() {
            self.cancel.checkpoint()?;
            if values.contains_key(&id) {
                continue;
            }
            let node = self.graph.graph.node(id)?;
            if !ready {
                pending.push((id, true));
                pending.extend(
                    node.children
                        .iter()
                        .chain(node.payload.referenced_nodes().iter())
                        .map(|child| (*child, false)),
                );
                continue;
            }
            self.inventory.node_support(
                self.source_id,
                *reverse
                    .get(&id)
                    .ok_or_else(|| invalid("scalar node remap absent"))?,
                self.support,
            )?;
            let operand = |index: usize| -> Result<Value, CompilerError> {
                let child = node
                    .children
                    .get(index)
                    .ok_or_else(|| invalid("scalar operator arity differs"))?;
                values
                    .get(child)
                    .cloned()
                    .ok_or_else(|| invalid("scalar operand not evaluated"))
            };
            let value = match &node.payload {
                Payload::IntConst { value } => Value::Integer(i128::from(*value)),
                Payload::FloatConst { value, unit } => {
                    let unit_spec = self.inventory.physical.unit(*unit)?;
                    if unit_spec.dimension.is_dimensionless()
                        && !unit_spec.is_affine
                        && unit_spec.reference_state.is_none()
                    {
                        Value::Real {
                            value: *value * unit_spec.scale_to_canonical,
                            unit: None,
                            quantity: None,
                        }
                    } else {
                        Value::Real {
                            value: *value,
                            unit: Some(*unit),
                            quantity: None,
                        }
                    }
                }
                Payload::SymbolRef { symbol } => self.reference(symbol)?,
                Payload::PendingPath {
                    source_id,
                    path_id,
                    indices,
                } => {
                    let coordinates = indices
                        .iter()
                        .map(|node| {
                            values
                                .get(node)
                                .cloned()
                                .ok_or_else(|| invalid("path coordinate has no scalar value"))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    self.path(*source_id, *path_id, &coordinates)?
                }
                Payload::None => match node.opcode {
                    Opcode::Neg => match operand(0)? {
                        Value::Integer(value) => Value::Integer(
                            value
                                .checked_neg()
                                .ok_or_else(|| invalid("integer negation overflow"))?,
                        ),
                        Value::Real {
                            value,
                            unit,
                            quantity,
                        } => Value::Real {
                            value: -value,
                            unit,
                            quantity,
                        },
                        Value::Unknown => Value::Unknown,
                        _ => return Err(invalid("negation requires numeric scalar")),
                    },
                    Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Pow => {
                        arithmetic(node.opcode, &operand(0)?, &operand(1)?)?
                    }
                    _ => {
                        return Err(invalid(
                            "configuration predicate contains unsupported numerical operation",
                        ));
                    }
                },
                Payload::UnitConvert(spec) => match operand(0)? {
                    Value::Real {
                        value,
                        unit: Some(unit),
                        quantity,
                    } if unit == spec.from => Value::Real {
                        value: pse_quantity::unit::convert_value(spec, value),
                        unit: Some(spec.to),
                        quantity,
                    },
                    Value::Unknown => Value::Unknown,
                    _ => {
                        return Err(invalid(
                            "predicate unit conversion input coordinate differs",
                        ));
                    }
                },
                _ => {
                    return Err(invalid(
                        "configuration predicate contains unresolved numerical payload",
                    ));
                }
            };
            values.insert(id, value);
        }
        values
            .remove(&root)
            .ok_or_else(|| invalid("scalar root was not evaluated"))
    }
    fn reference(&mut self, reference: &ValueRef) -> Result<Value, CompilerError> {
        match reference {
            ValueRef::Index(index) => Ok(self
                .bindings
                .get(&index.as_id())
                .copied()
                .map_or(Value::Unknown, Value::Id)),
            ValueRef::ActualSymbol(_) | ValueRef::Domain(_) => Ok(Value::Unknown),
            ValueRef::Template {
                template_id,
                kind,
                name,
            } => {
                if *template_id != self.instance.template_id {
                    return Err(invalid(
                        "predicate configuration belongs to another template",
                    ));
                }
                if *kind == TemplateValueKind::Port {
                    return Ok(Value::Unknown);
                }
                self.config(self.instance.instance_id, kind.as_str(), name)
            }
        }
    }
    fn config(
        &mut self,
        owner: SemanticId,
        category: &str,
        name: &str,
    ) -> Result<Value, CompilerError> {
        // Both generated value structs are read directly. Their common registry
        // discriminator governs the scalar interpretation; no Cell mirror is built.
        macro_rules! scalar_value {
            ($value:expr, $selection:path) => {{
                use $selection as Selected;
                match $value.selected()? {
                    Selected::Boolean(arm) => Ok(Value::Bool(arm.value)),
                    Selected::Signed(arm) => Ok(Value::Integer(i128::from(arm.value))),
                    Selected::Unsigned(arm) => Ok(Value::Integer(i128::from(arm.value))),
                    Selected::Real(arm) => Ok(Value::Real {
                        value: arm.value,
                        unit: None,
                        quantity: None,
                    }),
                    Selected::Text(arm) => Ok(Value::Text(arm.value.clone())),
                    Selected::SemanticId(arm) => Ok(Value::Id(arm.value)),
                    Selected::Enum(arm) => Ok(Value::Enum(arm.enum_id, arm.member.clone())),
                    Selected::Quantity(arm) => Ok(Value::Real {
                        value: arm.value,
                        unit: Some(UnitId::from_id(arm.unit_id)),
                        quantity: Some(QuantityTypeId::from_id(arm.quantity_type_id)),
                    }),
                    Selected::Index(_) => {
                        Err(invalid("configuration value is not a predicate scalar"))
                    }
                }
            }};
        }
        if category == "feature" {
            let mut rows = self
                .inventory
                .feature_values
                .iter()
                .filter(|row| row.row.instance_id == owner && row.row.name == name);
            let Some(row) = rows.next() else {
                return Ok(Value::Unknown);
            };
            if rows.next().is_some() {
                return Err(invalid("feature repeats its semantic key"));
            }
            self.support.insert(self.inventory.origin(row)?);
            scalar_value!(&row.row.value, pse_relations::generated::inferred::instance_features::InferredInstanceFeaturesFieldValueSelected)
        } else {
            let mut rows = self.inventory.configuration.iter().filter(|row| {
                row.row.owner_id == owner
                    && row.row.category.as_str() == category
                    && row.row.name == name
            });
            let Some(row) = rows.next() else {
                return Ok(Value::Unknown);
            };
            if rows.next().is_some() {
                return Err(invalid("configuration repeats its semantic key"));
            }
            self.support.insert(self.inventory.origin(row)?);
            scalar_value!(&row.row.value, pse_relations::generated::normalized::config_values::NormalizedConfigValuesFieldValueSelected)
        }
    }
}
fn arithmetic(op: Opcode, left: &Value, right: &Value) -> Result<Value, CompilerError> {
    if matches!(left, Value::Unknown) || matches!(right, Value::Unknown) {
        return Ok(Value::Unknown);
    }
    if let (Value::Integer(a), Value::Integer(b)) = (left, right) {
        let value = match op {
            Opcode::Add => a.checked_add(*b),
            Opcode::Sub => a.checked_sub(*b),
            Opcode::Mul => a.checked_mul(*b),
            Opcode::Div if *b != 0 && a % b == 0 => a.checked_div(*b),
            Opcode::Pow => u32::try_from(*b)
                .ok()
                .and_then(|power| a.checked_pow(power)),
            _ => None,
        };
        if let Some(value) = value {
            return Ok(Value::Integer(value));
        }
        if op != Opcode::Div {
            return Err(invalid("exact integer predicate arithmetic overflow"));
        }
    }
    if matches!(left, Value::Real { unit: Some(_), .. })
        || matches!(right, Value::Real { unit: Some(_), .. })
    {
        return Err(invalid(
            "unit-bearing predicate arithmetic requires an explicit typed conversion/operation",
        ));
    }
    let a = left.exact_real()?;
    let b = right.exact_real()?;
    let value = match op {
        Opcode::Add => a + b,
        Opcode::Sub => a - b,
        Opcode::Mul => a * b,
        Opcode::Div => a / b,
        Opcode::Pow => a.powf(b),
        _ => return Err(invalid("unsupported scalar arithmetic")),
    };
    if !value.is_finite() {
        return Err(invalid("predicate arithmetic leaves its finite domain"));
    }
    Ok(Value::Real {
        value,
        unit: None,
        quantity: None,
    })
}
pub(super) fn compare(
    left: &Value,
    right: &Value,
    op: &str,
    physical: &pse_quantity::QuantityRegistry,
) -> Result<Truth, CompilerError> {
    if matches!(left, Value::Unknown) || matches!(right, Value::Unknown) {
        return Ok(Truth::Unknown);
    }
    let order = match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        (Value::Text(a), Value::Text(b)) => a.cmp(b),
        (Value::Id(a), Value::Id(b)) => a.cmp(b),
        (Value::Enum(a, x), Value::Enum(b, y)) if a == b => x.cmp(y),
        (
            Value::Real {
                value: a,
                unit: Some(ua),
                quantity: qa,
            },
            Value::Real {
                value: b,
                unit: Some(ub),
                quantity: qb,
            },
        ) => {
            if qa.is_some() && qb.is_some() && qa != qb {
                return Err(invalid(
                    "predicate compares incompatible complete quantity types",
                ));
            }
            let context = qa.or(*qb).ok_or_else(|| {
                invalid("quantity comparison has no declared kind/basis/reference context")
            })?;
            let quantity = physical.quantity_type(context)?;
            let spec = pse_quantity::unit::convert_spec_for_type(
                physical.unit(*ub)?,
                physical.unit(*ua)?,
                &quantity.key,
            )?;
            let b = pse_quantity::unit::convert_value(&spec, *b);
            a.partial_cmp(&b)
                .ok_or_else(|| invalid("nonfinite predicate comparison"))?
        }
        (
            Value::Integer(_) | Value::Real { unit: None, .. },
            Value::Integer(_) | Value::Real { unit: None, .. },
        ) => left
            .exact_real()?
            .partial_cmp(&right.exact_real()?)
            .ok_or_else(|| invalid("nonfinite predicate comparison"))?,
        _ => {
            return Err(invalid(
                "predicate operand logical/quantity contracts differ",
            ));
        }
    };
    let value = match op {
        "eq" => order.is_eq(),
        "not_eq" => !order.is_eq(),
        "lt" => order.is_lt(),
        "le" => !order.is_gt(),
        "gt" => order.is_gt(),
        "ge" => !order.is_lt(),
        _ => return Err(invalid("comparison operator is undeclared")),
    };
    Ok(value.into())
}
