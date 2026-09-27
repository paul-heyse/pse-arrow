// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Conservative owned-container accounting. Shared values may be counted more than once.
use crate::specialize::{Lineage, Value};
use crate::{CheckedPackage, Function, SpecializedModel, Type};
use pse_authoring::dsl::{Equation, EquationKind, Expr, ExprKind, Path, Predicate, PredicateKind};
use pse_model::HeapUsage;
use pse_quantity::scheme::Scheme;
use std::{collections::BTreeMap, mem::size_of};
fn map<K, V>(values: &BTreeMap<K, V>, mut value: impl FnMut(&K, &V) -> usize) -> usize {
    values.iter().fold(0usize, |sum, (k, v)| {
        sum.saturating_add(size_of::<(K, V)>() + 64)
            .saturating_add(value(k, v))
    })
}
fn scheme(value: &Scheme) -> usize {
    match value {
        Scheme::Variable(n) => n.capacity(),
        Scheme::Concrete(_) => 0,
        Scheme::Delta(v) | Scheme::Power(v, _) => size_of::<Scheme>() + scheme(v),
        Scheme::Product(a, b) | Scheme::Quotient(a, b) => {
            2 * size_of::<Scheme>() + scheme(a) + scheme(b)
        }
    }
}
fn ty(value: &Type) -> usize {
    match value {
        Type::Function { arguments, result } => {
            arguments.capacity() * size_of::<(String, Type)>()
                + arguments
                    .iter()
                    .map(|(n, t)| n.capacity() + ty(t))
                    .sum::<usize>()
                + size_of::<Type>()
                + ty(result)
        }
        Type::Quantity(v) => scheme(v),
        Type::Tuple(v) => v.capacity() * size_of::<Type>() + v.iter().map(ty).sum::<usize>(),
        Type::Set(v) | Type::Optional(v) | Type::Continuous(_, v) => size_of::<Type>() + ty(v),
        Type::Indexed { element, axes } => {
            size_of::<Type>() + ty(element) + axes.capacity() * size_of::<pse_ids::SemanticId>()
        }
        _ => 0,
    }
}
fn path(value: &Path) -> usize {
    value.segments.capacity() * size_of::<pse_authoring::dsl::PathSegment>()
        + value
            .segments
            .iter()
            .map(|s| {
                s.name.capacity()
                    + s.indices.capacity() * size_of::<Expr>()
                    + s.indices.iter().map(expression).sum::<usize>()
            })
            .sum::<usize>()
}
fn predicate(value: &Predicate) -> usize {
    size_of::<Predicate>()
        + match &value.kind {
            PredicateKind::Compare { lhs, rhs, .. } => expression(lhs) + expression(rhs),
            PredicateKind::In { expr, domain } => expression(expr) + path(domain),
            PredicateKind::Atom(e) => expression(e),
            PredicateKind::And(a, b) | PredicateKind::Or(a, b) => predicate(a) + predicate(b),
            PredicateKind::Not(p) => predicate(p),
            _ => 0,
        }
}
pub(crate) fn expression(value: &Expr) -> usize {
    size_of::<Expr>()
        + match &value.kind {
            ExprKind::Number(n) => n.unit.as_ref().map_or(0, String::capacity),
            ExprKind::Path(p) => path(p),
            ExprKind::Neg(e) => expression(e),
            ExprKind::Binary { lhs, rhs, .. } => expression(lhs) + expression(rhs),
            ExprKind::Call { args, .. } => {
                args.capacity() * size_of::<Expr>() + args.iter().map(expression).sum::<usize>()
            }
            ExprKind::NamedCall { name, args } => {
                name.capacity()
                    + args.capacity() * size_of::<Expr>()
                    + args.iter().map(expression).sum::<usize>()
            }
            ExprKind::Partial {
                function,
                wrt,
                args,
            } => {
                function.capacity()
                    + wrt.capacity() * size_of::<Path>()
                    + wrt.iter().map(path).sum::<usize>()
                    + args.capacity() * size_of::<Expr>()
                    + args.iter().map(expression).sum::<usize>()
            }
            ExprKind::Kernel { name, args } => {
                name.capacity()
                    + args.capacity() * size_of::<Expr>()
                    + args.iter().map(expression).sum::<usize>()
            }
            ExprKind::Reduce { binder, body, .. } => {
                binder.var.capacity()
                    + path(&binder.domain)
                    + binder.filter.as_deref().map_or(0, predicate)
                    + expression(body)
            }
            ExprKind::Fold {
                accumulator,
                item,
                binder,
                value,
                step,
            } => {
                accumulator.capacity()
                    + item.capacity()
                    + binder.var.capacity()
                    + path(&binder.domain)
                    + binder.filter.as_deref().map_or(0, predicate)
                    + expression(value)
                    + expression(step)
            }
            ExprKind::Derivative { body, wrt } => expression(body) + path(wrt),
            ExprKind::Conditional {
                guard,
                then,
                otherwise,
            } => predicate(guard) + expression(then) + expression(otherwise),
            ExprKind::Let { bindings, body } => {
                bindings.capacity() * size_of::<(String, Expr)>()
                    + bindings
                        .iter()
                        .map(|(n, e)| n.capacity() + expression(e))
                        .sum::<usize>()
                    + expression(body)
            }
        }
}
fn equation(value: &Equation) -> usize {
    size_of::<Equation>()
        + match &value.kind {
            EquationKind::Relation { lhs, rhs, .. } => expression(lhs) + expression(rhs),
            EquationKind::Conditional {
                guard,
                then,
                otherwise,
            } => predicate(guard) + equation(then) + equation(otherwise),
        }
}
fn lineage(v: &Lineage) -> usize {
    v.path.capacity()
        + (v.demand.capacity() + v.presets.capacity()) * size_of::<pse_ids::SemanticId>()
}
fn function(v: &Function) -> usize {
    v.validity.as_ref().map_or(0, predicate)
        + v.external
            .as_ref()
            .map_or(0, crate::external::External::retained_bytes)
        + v.variables
            .iter()
            .map(|s| s.capacity() + size_of::<String>() + 64)
            .sum::<usize>()
        + v.arguments.capacity() * size_of::<(String, Type)>()
        + v.arguments
            .iter()
            .map(|(n, v)| n.capacity() + ty(v))
            .sum::<usize>()
        + ty(&v.result)
        + v.body.as_ref().map_or(0, expression)
}
impl Value {
    /// Conservative owned value storage, including inline nodes and finite membership.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + match self {
                Self::Text(s) => s.capacity(),
                Self::Enum { member, .. } => member.capacity(),
                Self::Definition { bindings, .. } => {
                    map(bindings, |n, v| n.capacity() + v.retained_bytes())
                }
                Self::Row { fields, .. } => map(fields, |n, v| n.capacity() + v.retained_bytes()),
                Self::Set(v) | Self::Tuple(v) => {
                    v.capacity() * size_of::<Self>()
                        + v.iter().map(Self::retained_bytes).sum::<usize>()
                }
                _ => 0,
            }
    }
}
impl CheckedPackage {
    /// Conservative owned declaration, table, type and lookup storage.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .preconditions
                .declarations()
                .iter()
                .map(|p| {
                    size_of::<pse_quantity::PhysicalPrecondition>()
                        + p.operand_positions.capacity() * size_of::<u16>()
                })
                .sum::<usize>()
            + self.quantities.allocation_extent()
            + map(&self.quantity_names, |n, _| n.capacity())
            + self.lowered_functions.len() * (size_of::<pse_ids::SemanticId>() + 64)
            + map(&self.declarations, |_, v| v.heap_bytes())
            + map(&self.names, |n, _| n.capacity())
            + map(&self.children, |_, v| {
                v.capacity() * size_of::<pse_ids::SemanticId>()
            })
            + map(&self.types, |_, v| ty(v))
            + map(&self.functions, |_, v| function(v))
            + map(&self.members, |_, v| map(v, |n, _| n.capacity()))
            + map(&self.interfaces, |_, v| {
                v.len() * (size_of::<pse_ids::SemanticId>() + 64)
            })
            + map(&self.tables, |_, v| {
                v.keys.iter().map(ty).sum::<usize>()
                    + v.keys.capacity() * size_of::<Type>()
                    + ty(&v.result)
                    + v.columns.capacity() * size_of::<(String, Type)>()
                    + v.columns
                        .iter()
                        .map(|(n, v)| n.capacity() + ty(v))
                        .sum::<usize>()
                    + v.default.as_ref().map_or(0, Value::retained_bytes)
                    + map(&v.rows, |k, v| {
                        k.capacity() * size_of::<Value>()
                            + k.iter().map(Value::retained_bytes).sum::<usize>()
                            + v.retained_bytes()
                    })
                    + map(&v.origins, |k, _| {
                        k.capacity() * size_of::<Value>()
                            + k.iter().map(Value::retained_bytes).sum::<usize>()
                    })
            })
    }
}
impl SpecializedModel {
    /// Conservative owned specialization storage, excluding separately owned library math.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + map(&self.regimes, |_, r| {
                expression(&r.criterion)
                    + expression(&r.tolerance)
                    + r.alternatives.capacity() * size_of::<crate::specialize::Regime>()
                    + r.alternatives
                        .iter()
                        .map(|r| {
                            predicate(&r.eligibility)
                                + r.equations.capacity() * size_of::<crate::specialize::Row>()
                                + r.equations
                                    .iter()
                                    .map(|e| equation(&e.equation) + lineage(&e.lineage))
                                    .sum::<usize>()
                                + r.annotations.capacity()
                                    * size_of::<crate::annotation::Annotation>()
                                + r.annotations
                                    .iter()
                                    .map(|a| annotation(&a.value) + lineage(&a.lineage))
                                    .sum::<usize>()
                        })
                        .sum::<usize>()
            })
            + map(&self.fixtures, |_, f| {
                f.stages
                    .iter()
                    .map(|s| s.capacity() + size_of::<String>())
                    .sum::<usize>()
                    + f.integration.as_ref().map_or(0, |i| {
                        i.samples.capacity() * size_of::<f64>() + map(&i.quadratures, |_, _| 0)
                    })
                    + f.expected_failure.as_ref().map_or(0, HeapUsage::heap_bytes)
                    + map(&f.specifications, |p, _| p.capacity())
                    + f.oracle
                        .as_ref()
                        .map_or(0, pse_model::HeapUsage::heap_bytes)
            })
            + self.initial_equations.len() * 64
            + map(&self.integrated, |_, _| 0)
            + map(&self.integrals, |_, _| 0)
            + map(&self.derivatives, |_, v| lineage(&v.lineage))
            + map(&self.expectations, |_, v| {
                expression(&v.actual)
                    + expression(&v.expected)
                    + expression(&v.tolerance)
                    + expression(&v.relative_tolerance)
                    + ty(&v.ty)
                    + lineage(&v.lineage)
            })
            + map(&self.paths, |n, _| n.capacity())
            + map(&self.elastic, |_, v| {
                equation(&v.original.equation)
                    + lineage(&v.original.lineage)
                    + v.slacks.capacity() * size_of::<pse_ids::SemanticId>()
                    + expression(&v.penalty)
            })
            + map(&self.continuation, |_, v| {
                v.start.retained_bytes() + v.end.retained_bytes() + lineage(&v.lineage)
            })
            + map(&self.implicit, |_, v| match v {
                crate::specialize::Realization::Accelerated(id) => id.capacity(),
                _ => 0,
            })
            + map(&self.meshes, |_, m| {
                m.points.capacity() * size_of::<Value>()
                    + m.points.iter().map(Value::retained_bytes).sum::<usize>()
                    + m.derivative.capacity() * size_of::<Vec<(usize, f64)>>()
                    + m.derivative
                        .iter()
                        .map(|r| r.capacity() * size_of::<(usize, f64)>())
                        .sum::<usize>()
                    + m.integral.capacity() * size_of::<f64>()
                    + m.continuity.capacity() * size_of::<(usize, Vec<(usize, f64)>)>()
                    + m.continuity
                        .iter()
                        .map(|(_, v)| v.capacity() * size_of::<(usize, f64)>())
                        .sum::<usize>()
            })
            + map(&self.symbols, |_, s| {
                ty(&s.ty)
                    + s.expression.as_ref().map_or(0, expression)
                    + s.initial.as_ref().map_or(0, Value::retained_bytes)
                    + lineage(&s.lineage)
            })
            + self.equations.capacity() * size_of::<crate::specialize::Row>()
            + self
                .equations
                .iter()
                .map(|r| equation(&r.equation) + lineage(&r.lineage))
                .sum::<usize>()
            + map(&self.closures, |_, v| {
                ty(&v.ty)
                    + v.tolerance.retained_bytes()
                    + lineage(&v.lineage)
                    + v.terms.capacity() * size_of::<crate::specialize::Contribution>()
                    + v.terms
                        .iter()
                        .map(|v| expression(&v.expression) + lineage(&v.lineage))
                        .sum::<usize>()
            })
            + map(&self.instances, |_, v| {
                v.path.capacity()
                    + v.stages
                        .iter()
                        .map(|s| s.capacity() + size_of::<String>() + 64)
                        .sum::<usize>()
                    + (v.coordinates.capacity() + v.rows.capacity())
                        * size_of::<pse_ids::SemanticId>()
                    + map(&v.members, |n, _| n.capacity())
            })
            + map(&self.groups, |_, v| {
                v.instances.capacity() * size_of::<pse_ids::SemanticId>()
                    + v.body.coordinates.capacity()
                        * size_of::<(Type, pse_model::generated::enums::ModelingDeclarationKind)>()
                    + v.body.coordinates.iter().map(|(t, _)| ty(t)).sum::<usize>()
                    + map(&v.body.expressions, |_, e| expression(e))
                    + v.body.equations.capacity() * size_of::<Equation>()
                    + v.body.equations.iter().map(equation).sum::<usize>()
            })
            + map(&self.ports, |_, v| lineage(&v.lineage))
            + map(&self.connections, |_, v| lineage(&v.lineage))
            + map(&self.connectivity, |_, v| lineage(&v.lineage))
            + map(&self.functions, |n, v| n.capacity() + function(v))
            + self.annotations.capacity() * size_of::<crate::annotation::Annotation>()
            + self
                .annotations
                .iter()
                .map(|v| lineage(&v.lineage) + annotation(&v.value))
                .sum::<usize>()
    }
}

fn annotation(v: &crate::annotation::AnnotationValue) -> usize {
    use crate::annotation::AnnotationValue as A;
    match v {
        A::Start(e) | A::Nominal(e) => expression(e),
        A::Bounds(a, b) => expression(a) + expression(b),
        A::Scale(_) | A::Objective(_) => 0,
        A::Report(s) => s.capacity(),
        A::Valid {
            lower,
            upper,
            policy,
        } => expression(lower) + expression(upper) + policy.capacity(),
        A::Check(p) => predicate(p),
    }
}

impl Function {
    /// Conservative owned signature and expression storage.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>() + function(self)
    }
}
