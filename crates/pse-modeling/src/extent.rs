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
fn state_key(value: &crate::specialize::StateKey) -> usize {
    value.name.capacity()
        + value.indices.capacity() * size_of::<Value>()
        + value
            .indices
            .iter()
            .map(Value::retained_bytes)
            .sum::<usize>()
}
fn selections(values: &crate::scientific_selection::Selections) -> usize {
    map(values, |occurrence, closure| {
        occurrence.expression.capacity()
            + occurrence.roots.capacity() * size_of::<Value>()
            + occurrence
                .roots
                .iter()
                .map(Value::retained_bytes)
                .sum::<usize>()
            + occurrence.context.retained_bytes()
            + (closure.roots.capacity() + closure.records.capacity()) * size_of::<Value>()
            + closure
                .roots
                .iter()
                .chain(&closure.records)
                .map(Value::retained_bytes)
                .sum::<usize>()
            + closure.context.retained_bytes()
            + map(&closure.edges, |record, dependencies| {
                record.retained_bytes()
                    + dependencies
                        .iter()
                        .map(|value| size_of::<Value>() + 64 + value.retained_bytes())
                        .sum::<usize>()
            })
    })
}
pub(crate) fn scheme(value: &Scheme) -> usize {
    match value {
        Scheme::Variable(n) => n.capacity(),
        Scheme::Concrete(_) => 0,
        Scheme::Resolved(contract) => {
            size_of::<pse_quantity::ResolvedPhysicalContract>() + contract.heap_bytes()
        }
        Scheme::Delta(v) | Scheme::Power(v, _) => size_of::<Scheme>() + scheme(v),
        Scheme::Product(a, b) | Scheme::Quotient(a, b) => {
            2 * size_of::<Scheme>() + scheme(a) + scheme(b)
        }
    }
}
pub(crate) fn ty(value: &Type) -> usize {
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
        Type::Quantity(v) | Type::RefinedQuantity { quantity: v, .. } => scheme(v),
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
pub(crate) fn predicate(value: &Predicate) -> usize {
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
            ExprKind::Number(n) => n
                .unit
                .as_ref()
                .map_or(0, pse_quantity::UnitProduct::retained_bytes),
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
pub(crate) fn equation(value: &Equation) -> usize {
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
    v.applicability.capacity() * size_of::<Expr>()
        + v.prerequisites.capacity() * size_of::<usize>()
        + v.applicability.iter().map(expression).sum::<usize>()
        + v.applicability_uses.capacity() * size_of::<crate::applicability::Use>()
        + v.applicability_uses
            .iter()
            .map(|usage| {
                usage.node.retained_bytes()
                    + usage.predicates.capacity() * size_of::<Predicate>()
                    + usage.predicates.iter().map(predicate).sum::<usize>()
                    + usage.inputs.capacity() * size_of::<Expr>()
                    + usage.inputs.iter().map(expression).sum::<usize>()
            })
            .sum::<usize>()
        + v.physical_admissions
            .iter()
            .map(|(occurrence, admission)| {
                occurrence.syntax.capacity() + admission.retained_bytes() + 64
            })
            .sum::<usize>()
        + v.reduction
            .as_ref()
            .map_or(0, |reduction| reduction.prototype.heap_bytes())
        + v.validity.as_ref().map_or(0, predicate)
        + v.envelopes.capacity() * size_of::<crate::envelope::Guard>()
        + v.envelopes
            .iter()
            .map(|g| {
                predicate(&g.predicate)
                    + g.carrier.capacity()
                    + g.envelope.axis.capacity()
                    + g.envelope.lower.capacity()
                    + g.envelope.upper.capacity()
                    + ty(&g.envelope.ty)
                    + g.arguments
                        .iter()
                        .map(|a| a.capacity() + size_of::<String>())
                        .sum::<usize>()
            })
            .sum::<usize>()
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
                Self::Identifier { value, .. } => value.capacity(),
                Self::Definition { bindings, .. } => {
                    map(bindings, |n, v| n.capacity() + v.retained_bytes())
                }
                // Row cells are shared with the admitted table; each value counts them.
                Self::Row { names, fields, .. } => {
                    names.iter().map(String::capacity).sum::<usize>()
                        + fields.iter().map(Self::retained_bytes).sum::<usize>()
                }
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
            + selections(&self.selection_closures)
            + crate::expression::occurrences::retained_bytes(&self.expressions)
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
            + self.scope.package.as_ref().map_or(0, String::capacity)
            + self
                .scope
                .documents
                .as_ref()
                .map_or(0, |d| d.len() * (size_of::<pse_ids::SemanticId>() + 32))
            + self.lowered_functions.len() * (size_of::<pse_ids::SemanticId>() + 64)
            + map(&self.declarations, |_, v| v.heap_bytes())
            + map(&self.names, |n, _| n.capacity())
            + map(&self.children, |_, v| {
                v.capacity() * size_of::<pse_ids::SemanticId>()
            })
            + map(&self.types, |_, v| ty(v))
            + map(&self.physical_admissions, |_, admissions| {
                map(admissions, |occurrence, admission| {
                    occurrence.syntax.capacity() + admission.retained_bytes()
                })
            })
            + map(&self.functions, |_, v| function(v))
            + map(&self.members, |_, v| map(v, |n, _| n.capacity()))
            + map(&self.temporal, |_, (_, _, argument)| argument.capacity())
            + map(&self.interfaces, |_, v| {
                v.len() * (size_of::<pse_ids::SemanticId>() + 64)
            })
            + map(&self.tables, |_, v| v.retained_bytes(ty))
            + map(&self.kinds, |_, k| {
                k.attributes.capacity() * size_of::<(String, pse_ids::SemanticId)>()
                    + k.attributes
                        .iter()
                        .map(|(n, _)| n.capacity() + size_of::<(String, pse_ids::SemanticId)>())
                        .sum::<usize>()
                    + map(&k.defaults, |n, t| n.capacity() + t.value.retained_bytes())
                    + map(&k.bound, |n, (t, _)| {
                        n.capacity() + t.value.retained_bytes()
                    })
                    + k.keys.iter().map(String::capacity).sum::<usize>()
                    + k.keys.capacity() * size_of::<String>()
                    + k.envelopes.capacity() * size_of::<crate::envelope::Envelope>()
                    + k.envelopes
                        .iter()
                        .map(|envelope| {
                            envelope.axis.capacity()
                                + envelope.lower.capacity()
                                + envelope.upper.capacity()
                                + ty(&envelope.ty)
                        })
                        .sum::<usize>()
                    + (k.derived.capacity() + k.unique.capacity())
                        * size_of::<(String, pse_ids::SemanticId)>()
                    + k.derived
                        .iter()
                        .chain(&k.unique)
                        .map(|(name, _)| name.capacity())
                        .sum::<usize>()
                    + k.requirements.capacity() * size_of::<pse_ids::SemanticId>()
            })
            + map(&self.entities, |_, r| {
                map(&r.values, |n, v| n.capacity() + v.retained_bytes())
                    + map(&r.uncertainties, |n, _| n.capacity() + 16)
            })
            + map(&self.constants, |_, t| t.value.retained_bytes())
            + self.identifiers.retained_bytes()
            + map(&self.provenance, |_, p| {
                p.role.facets.len() * 64
                    + p.lineage.capacity()
                        * size_of::<(
                            pse_model::generated::enums::ModelingLineageKind,
                            pse_ids::SemanticId,
                        )>()
            })
            + map(&self.attribute_provenance, |(_, name), p| {
                name.capacity()
                    + p.role.facets.len() * 64
                    + p.lineage.capacity()
                        * size_of::<(
                            pse_model::generated::enums::ModelingLineageKind,
                            pse_ids::SemanticId,
                        )>()
            })
            + (self.test_only.len() + self.test_only_data.len()) * 80
            + map(&self.oracles, |_, _| 0)
    }
}
impl SpecializedModel {
    /// Conservative owned specialization storage, excluding separately owned library math.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + selections(&self.selection_closures)
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
                        i.samples.capacity() * size_of::<f64>()
                            + map(&i.quadratures, |_, _| 0)
                            + i.schedules.capacity()
                                * size_of::<crate::specialize::ScheduleFixture>()
                            + i.schedules
                                .iter()
                                .map(|s| {
                                    (s.times.capacity() + s.values.capacity()) * size_of::<f64>()
                                })
                                .sum::<usize>()
                    })
                    + f.shooting
                        .as_ref()
                        .map_or(0, |s| s.nodes.capacity() * size_of::<f64>())
                    + f.modes.capacity() * size_of::<crate::specialize::FixtureMode>()
                    + f.modes
                        .iter()
                        .map(|m| {
                            m.name.capacity()
                                + map(&m.facts, |n, _| n.member().len())
                                + m.events.capacity() * size_of::<crate::specialize::FixtureEvent>()
                                + m.events
                                    .iter()
                                    .map(|e| e.name.capacity() + map(&e.reset, |_, _| 0))
                                    .sum::<usize>()
                        })
                        .sum::<usize>()
                    + f.expected_failure.as_ref().map_or(0, |e| match &e.lineage {
                        crate::specialize::ExpectedLineage::Applicability {
                            sets,
                            variables,
                            ..
                        } => {
                            sets.capacity() * size_of::<pse_ids::SemanticId>()
                                + variables.capacity() * size_of::<String>()
                                + variables.iter().map(String::capacity).sum::<usize>()
                        }
                        crate::specialize::ExpectedLineage::Validity {
                            sets,
                            variables,
                            members,
                            ..
                        } => {
                            (sets.capacity() + members.capacity())
                                * size_of::<pse_ids::SemanticId>()
                                + variables.capacity() * size_of::<u32>()
                        }
                        crate::specialize::ExpectedLineage::Members(members) => {
                            members.len() * (size_of::<pse_ids::SemanticId>() + 32)
                        }
                    })
                    + map(&f.specifications, |p, _| p.capacity())
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
            + map(&self.state_specifications, |_, v| {
                map(&v.coordinates, |key, _| state_key(key))
                    + v.reconstructions.capacity() * size_of::<(crate::specialize::Row, Value)>()
                    + v.reconstructions
                        .iter()
                        .map(|(row, tolerance)| {
                            equation(&row.equation)
                                + lineage(&row.lineage)
                                + tolerance.retained_bytes()
                        })
                        .sum::<usize>()
                    + map(&v.transports, |key, observation| {
                        state_key(key)
                            + expression(&observation.expression)
                            + ty(&observation.ty)
                            + observation.tolerance.retained_bytes()
                    })
                    + lineage(&v.lineage)
            })
            + map(&self.material_ports, |_, v| {
                map(&v.coordinates, |key, _| state_key(key)) + lineage(&v.lineage)
            })
            + map(&self.inventory_balances, |_, v| {
                expression(&v.inventory)
                    + expression(&v.flux)
                    + ty(&v.ty)
                    + v.tolerance.retained_bytes()
                    + map(&v.transfers, |_, transfer| expression(transfer))
                    + lineage(&v.lineage)
            })
            + map(&self.inventory_initial_conditions, |_, v| {
                lineage(&v.lineage)
            })
            + map(&self.connections, |_, v| {
                lineage(&v.lineage)
                    + v.bindings.capacity()
                        * size_of::<(pse_ids::SemanticId, pse_ids::SemanticId)>()
                    + v.rows.capacity() * size_of::<pse_ids::SemanticId>()
            })
            + map(&self.connectivity, |_, v| lineage(&v.lineage))
            + map(&self.functions, |n, v| n.capacity() + function(v))
            + self.objectives.members.capacity() * size_of::<crate::specialize::ObjectiveMember>()
            + self
                .objectives
                .members
                .iter()
                .map(|m| lineage(&m.lineage))
                .sum::<usize>()
            + self.objectives.levels.capacity() * size_of::<crate::specialize::ObjectiveLevel>()
            + self
                .objectives
                .levels
                .iter()
                .map(|l| l.members.capacity() * size_of::<usize>())
                .sum::<usize>()
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
        A::Valid { lower, upper, .. } => expression(lower) + expression(upper),
        A::Check(p) => predicate(p),
    }
}

impl Function {
    /// Conservative owned signature and expression storage.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>() + function(self)
    }
}
