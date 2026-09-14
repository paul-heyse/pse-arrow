// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual registered columns replay through the same primitive mathematical callbacks.

use super::{MathRows, malformed};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_mathir::relations::{MathRelationSink, MathRelationSource};
use pse_mathir::{DomainRef, GuardRef, MathIrError, NodeId, Opcode, ValueRef};
use pse_quantity::{
    BoundIndexId, ConversionId, DomainId, InvariantId, OperationId, QuantityTypeId, ReductionKind,
    UnitId, WeightNormalization, infer::BuiltInRule,
};
use pse_relations::{RecordBatch, typed::CellCodec};
use pse_schema::{
    Registry,
    model::{Cell, Namespace, RelationKey, RelationSpec},
};
use std::collections::BTreeMap;

/// A structurally admitted compiled relation inventory; graph/inference checks follow replay.
#[derive(Debug)]
pub struct RelationSource<'a> {
    registry: &'a Registry,
    rows: MathRows,
}
impl<'a> RelationSource<'a> {
    /// Admit actual fields and values before converting any primitive callback.
    /// # Errors
    /// Unknown contracts, schema/value mismatches, or malformed row values.
    pub fn from_batches(
        rows: &BTreeMap<RelationKey, RecordBatch>,
        registry: &'a Registry,
    ) -> Result<Self, CompilerError> {
        let mut values = MathRows::new();
        for (key, batch) in rows {
            if key.name == "math_implicit_systems"
                || key.namespace != Namespace::Compiled
                || !(key.name.starts_with("math_") || key.name == "kernel_bindings")
            {
                continue;
            }
            let spec = registry
                .relation(&key.qualified_name())
                .filter(|spec| spec.key == *key)
                .ok_or_else(|| malformed("undeclared mathematical relation version"))?;
            values.insert(
                *key,
                pse_relations::cells::cells_from_batch(registry, spec, batch)?,
            );
        }
        Ok(Self {
            registry,
            rows: values,
        })
    }
}
impl MathRelationSource for RelationSource<'_> {
    fn read(&self, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
        for (key, rows) in &self.rows {
            let spec = self
                .registry
                .relation(&key.qualified_name())
                .ok_or_else(|| malformed("mathematical schema vanished"))?;
            for values in rows {
                replay(key.name, &Row { spec, values }, sink)?;
            }
        }
        Ok(())
    }
}
struct Row<'a> {
    spec: &'a RelationSpec,
    values: &'a [Cell],
}
impl Row<'_> {
    fn raw(&self, name: &str) -> Result<&Cell, MathIrError> {
        self.spec
            .columns
            .iter()
            .position(|column| column.name == name)
            .and_then(|index| self.values.get(index))
            .ok_or_else(|| malformed(format!("missing {}.{name}", self.spec.key.qualified_name())))
    }
    fn get<T: CellCodec>(&self, name: &str) -> Result<T, MathIrError> {
        value(self.raw(name)?)
    }
    fn node(&self, name: &str) -> Result<NodeId, MathIrError> {
        Ok(NodeId(self.get(name)?))
    }
    fn optional_node(&self, name: &str) -> Result<Option<NodeId>, MathIrError> {
        Ok(self.get::<Option<u64>>(name)?.map(NodeId))
    }
    fn enumeration<T>(&self, name: &str, parse: fn(&str) -> Option<T>) -> Result<T, MathIrError> {
        let Cell::Enum(name) = self.raw(name)? else {
            return Err(malformed("enum callback requires an admitted enum cell"));
        };
        parse(name).ok_or_else(|| malformed("unrecognized mathematical enum member"))
    }
    fn list(&self, name: &str) -> Result<&[Cell], MathIrError> {
        let Cell::List(values) = self.raw(name)? else {
            return Err(malformed("list callback requires an admitted list"));
        };
        Ok(values)
    }
}
fn value<T: CellCodec>(cell: &Cell) -> Result<T, MathIrError> {
    T::from_cell(cell.clone()).map_err(|error| malformed(error.to_string()))
}
fn tuple(cell: &Cell, count: usize) -> Result<&[Cell], MathIrError> {
    match cell {
        Cell::Struct(values) if values.len() == count => Ok(values),
        _ => Err(malformed("nested callback tuple shape differs")),
    }
}
fn replay(name: &str, row: &Row<'_>, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
    match name {
        "math_expr_nodes" => sink.expr_node(
            row.node("node_id")?,
            row.enumeration("opcode", Opcode::parse)?,
            row.get::<Option<SemanticId>>("quantity_type_id")?
                .map(QuantityTypeId::from_id),
            row.get("scope_instance_id")?,
            row.get("subtree_hash")?,
        ),
        "math_expr_args" => sink.expr_arg(
            row.node("parent_node_id")?,
            row.get("argument_ordinal")?,
            row.node("child_node_id")?,
        ),
        "math_symbol_refs" => sink.symbol_ref(
            row.node("node_id")?,
            ValueRef::ActualSymbol(row.get("symbol_id")?),
        ),
        "math_float_constants" => sink.float_constant(
            row.node("node_id")?,
            row.get("value")?,
            UnitId::from_id(row.get("unit_id")?),
        ),
        "math_int_constants" => sink.int_constant(row.node("node_id")?, row.get("value")?),
        "math_smooth_ops" => sink.smooth_op(row.node("node_id")?, row.get("eps")?),
        "math_conditionals" => sink.conditional(
            row.node("node_id")?,
            GuardRef::Math(row.node("guard_node_id")?),
        ),
        "math_kernel_calls" => sink.kernel_call(
            row.node("node_id")?,
            row.get("kernel_binding_id")?,
            row.get("output_ordinal")?,
        ),
        "math_implicit_refs" => sink.implicit_ref(
            row.node("node_id")?,
            row.get("implicit_system_id")?,
            row.get("unknown_ordinal")?,
        ),
        "math_unit_converts" => sink.unit_convert(
            row.node("node_id")?,
            row.get("scale")?,
            row.get("offset")?,
            UnitId::from_id(row.get("from_unit_id")?),
            UnitId::from_id(row.get("to_unit_id")?),
        ),
        "math_derivatives" | "math_integrals" | "math_broadcasts" | "math_reductions" => {
            domain_payload(name, row, sink)
        }
        "math_affine" => affine(row, sink),
        "math_weighted_means" => weighted(row, sink),
        "math_gathers" => gather(row, sink),
        "math_piecewise_linear" => piecewise(row, sink),
        "math_indexed_equations" => equation(row, sink),
        "math_free_indices" => sink.free_index(
            row.get("indexed_equation_id")?,
            BoundIndexId::from_id(row.get("bound_index_id")?),
            DomainId::from_id(row.get("domain_id")?),
            row.get("position")?,
        ),
        "math_quantity_selections" => selection(row, sink),
        "kernel_bindings" => kernel(row, sink),
        _ if row.values.is_empty() => Ok(()),
        _ => Err(malformed(format!(
            "{name} is not a source relation of the indexed mathematical graph"
        ))),
    }
}
fn domain_payload(
    name: &str,
    row: &Row<'_>,
    sink: &mut dyn MathRelationSink,
) -> Result<(), MathIrError> {
    match name {
        "math_derivatives" => sink.derivative(
            row.node("node_id")?,
            DomainRef::Actual(DomainId::from_id(row.get("wrt_domain_id")?)),
            row.get("order")?,
        ),
        "math_integrals" => sink.integral(
            row.node("node_id")?,
            DomainRef::Actual(DomainId::from_id(row.get("domain_id")?)),
            BoundIndexId::from_id(row.get("bound_index_id")?),
            row.get("quadrature_policy_id")?,
            row.get::<Option<u64>>("filter_node_id")?
                .map(|id| GuardRef::Math(NodeId(id))),
        ),
        "math_broadcasts" => sink.broadcast(
            row.node("node_id")?,
            DomainRef::Actual(DomainId::from_id(row.get("domain_id")?)),
            BoundIndexId::from_id(row.get("bound_index_id")?),
        ),
        "math_reductions" => sink.reduction(
            row.node("node_id")?,
            row.enumeration("kind", ReductionKind::parse)?,
            DomainRef::Actual(DomainId::from_id(row.get("domain_id")?)),
            BoundIndexId::from_id(row.get("bound_index_id")?),
            row.optional_node("filter_node_id")?.map(GuardRef::Math),
        ),
        _ => Err(malformed("unknown domain-bearing payload")),
    }
}

fn affine(row: &Row<'_>, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
    let terms = row
        .list("terms")?
        .iter()
        .map(|cell| {
            let t = tuple(cell, 2)?;
            Ok((value(&t[0])?, NodeId(value(&t[1])?)))
        })
        .collect::<Result<Vec<_>, MathIrError>>()?;
    sink.affine(
        row.node("node_id")?,
        row.get("constant")?,
        row.get::<Option<SemanticId>>("constant_quantity_type_id")?
            .map(QuantityTypeId::from_id),
        row.get::<Option<SemanticId>>("constant_unit_id")?
            .map(UnitId::from_id),
        &terms,
    )
}
fn weighted(row: &Row<'_>, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
    let pairs = row
        .list("pairs")?
        .iter()
        .map(|cell| {
            let t = tuple(cell, 2)?;
            Ok((NodeId(value(&t[0])?), NodeId(value(&t[1])?)))
        })
        .collect::<Result<Vec<_>, MathIrError>>()?;
    sink.weighted_mean(
        row.node("node_id")?,
        &pairs,
        row.enumeration("normalization", WeightNormalization::parse)?,
        row.get::<Option<SemanticId>>("unit_sum_invariant_id")?
            .map(InvariantId::from_id),
    )
}
fn gather(row: &Row<'_>, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
    let coordinates = row
        .list("coordinate_map")?
        .iter()
        .map(|cell| {
            let t = tuple(cell, 2)?;
            Ok((BoundIndexId::from_id(value(&t[0])?), value(&t[1])?))
        })
        .collect::<Result<Vec<_>, MathIrError>>()?;
    sink.gather(row.node("node_id")?, row.get("group_id")?, &coordinates)
}
fn piecewise(row: &Row<'_>, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
    let points = row
        .list("breakpoints")?
        .iter()
        .map(|cell| {
            let t = tuple(cell, 2)?;
            Ok((value(&t[0])?, value(&t[1])?))
        })
        .collect::<Result<Vec<_>, MathIrError>>()?;
    sink.piecewise_linear(
        row.node("node_id")?,
        &points,
        QuantityTypeId::from_id(row.get("input_quantity_type_id")?),
        QuantityTypeId::from_id(row.get("output_quantity_type_id")?),
    )
}
fn equation(row: &Row<'_>, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
    sink.indexed_equation(
        row.get("indexed_equation_id")?,
        row.get("owner_instance_id")?,
        row.get("equation_decl_id")?,
        &row.get::<String>("qualified_name")?,
        row.get("product_id")?,
        row.optional_node("filter_node_id")?,
        row.node("body_node_id")?,
        row.enumeration("sense", pse_mathir::equation::Sense::parse)?,
        row.optional_node("lower_node_id")?,
        row.optional_node("upper_node_id")?,
        row.get::<Option<SemanticId>>("residual_quantity_type_id")?
            .map(QuantityTypeId::from_id),
        row.get("law_instance_id")?,
        row.get("derivation_id")?,
    )
}
fn selection(row: &Row<'_>, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
    let conversions = row
        .list("conversions")?
        .iter()
        .map(|cell| {
            let t = tuple(cell, 2)?;
            Ok((value(&t[0])?, ConversionId::from_id(value(&t[1])?)))
        })
        .collect::<Result<Vec<_>, MathIrError>>()?;
    let builtin = match row.raw("builtin_rule")? {
        Cell::Null => None,
        Cell::Enum(name) => Some(
            BuiltInRule::parse(name).ok_or_else(|| malformed("unknown built-in quantity rule"))?,
        ),
        _ => return Err(malformed("built-in quantity rule must be an enum")),
    };
    sink.quantity_selection(
        row.node("node_id")?,
        row.get::<Option<SemanticId>>("operation_id")?
            .map(OperationId::from_id),
        builtin,
        &row.get::<Vec<u16>>("operand_permutation")?,
        &conversions,
        row.get("deferred_static_check")?,
    )
}
fn kernel(row: &Row<'_>, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
    let parameters = row
        .list("parameter_bindings")?
        .iter()
        .map(|cell| {
            let t = tuple(cell, 4)?;
            Ok((
                value(&t[0])?,
                value(&t[1])?,
                value(&t[2])?,
                value::<Option<SemanticId>>(&t[3])?.map(UnitId::from_id),
            ))
        })
        .collect::<Result<Vec<_>, MathIrError>>()?;
    let inputs = row
        .list("input_bindings")?
        .iter()
        .map(|cell| {
            let t = tuple(cell, 2)?;
            Ok((value(&t[0])?, NodeId(value(&t[1])?)))
        })
        .collect::<Result<Vec<_>, MathIrError>>()?;
    sink.kernel_binding(
        row.get("binding_id")?,
        row.get("kernel_id")?,
        row.get("scope_instance_id")?,
        &parameters,
        &inputs,
    )
}
