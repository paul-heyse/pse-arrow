// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Coefficient values and source keys come from the same completed native join.
use super::super::super::super::outputs::Support;
use super::{
    BTreeSet, CompilerError, Context, Declaration, ExprGraph, Opcode, Output, Payload, Reservation,
    SemanticId, compiled, invalid,
};
use crate::passes::native_outputs::Sources;
use datafusion::arrow::array::{Array, FixedSizeBinaryArray, Float64Array, ListArray};
use pse_ids::{IndexTuple, symbol_instance_id};
use pse_relations::generated::{enums, extension_values};

pub(super) struct Coefficient {
    group: SemanticId,
    element_member: SemanticId,
    species_member: SemanticId,
    element_id: SemanticId,
    species_id: SemanticId,
    count: Option<f64>,
    mw: Option<f64>,
    support: Support,
}
pub(super) fn read(
    complete: &pse_catalog::session::CompletedComputation,
    sources: &Sources,
    registry: &pse_schema::Registry,
) -> Result<Vec<Coefficient>, CompilerError> {
    let mut rows = Vec::new();
    for owned in complete.batches() {
        let batch = owned.batch();
        let ids = |field: &str, row: usize| -> Result<SemanticId, CompilerError> {
            let values = array::<FixedSizeBinaryArray>(batch, field)?;
            if values.is_null(row) {
                return Err(invalid("coefficient identity is null"));
            }
            SemanticId::try_from_slice(values.value(row))
                .map_err(|_| invalid("coefficient identity width differs"))
        };
        let numbers = |field: &str, row: usize| -> Result<Option<f64>, CompilerError> {
            let values = array::<Float64Array>(batch, field)?;
            Ok((!values.is_null(row)).then(|| values.value(row)))
        };
        for row in 0..batch.num_rows() {
            let mut keys = Vec::new();
            for (field, name) in [
                ("element_source", "normalized.domain_members"),
                ("species_member_source", "normalized.domain_members"),
                (
                    "element_eligible_source",
                    "inferred.domain_eligible_members",
                ),
                (
                    "species_eligible_source",
                    "inferred.domain_eligible_members",
                ),
                ("species_source", "normalized.species"),
            ] {
                let values = array::<FixedSizeBinaryArray>(batch, field)?;
                if values.is_null(row) {
                    return Err(invalid("coefficient source key is null"));
                }
                let spec = registry
                    .relation(name)
                    .ok_or_else(|| invalid("coefficient source declaration absent"))?;
                keys.push((
                    spec.key,
                    crate::passes::native_rows::key_value(values.value(row))?,
                ));
            }
            let compositions = array::<ListArray>(batch, "composition_keys")?;
            if compositions.is_null(row) {
                return Err(invalid("species composition inventory is absent"));
            }
            let values = compositions.value(row);
            let values = values
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .ok_or_else(|| invalid("composition key storage differs"))?;
            for value in values {
                let key = value.ok_or_else(|| invalid("composition source key is null"))?;
                keys.push((
                    pse_relations::generated::normalized::species_elements::RELATION_KEY,
                    crate::passes::native_rows::key_value(key)?,
                ));
            }
            rows.push(Coefficient {
                group: ids("group_id", row)?,
                element_member: ids("element_member", row)?,
                species_member: ids("species_member", row)?,
                element_id: ids("element_id", row)?,
                species_id: ids("species_id", row)?,
                count: numbers("count", row)?,
                mw: numbers("molecular_weight", row)?,
                support: sources.locate(keys)?,
            });
        }
    }
    Ok(rows)
}
fn array<'a, T: Array + 'static>(
    batch: &'a pse_relations::RecordBatch,
    name: &str,
) -> Result<&'a T, CompilerError> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<T>())
        .ok_or_else(|| invalid(format!("coefficient field {name} has unexpected storage")))
}
pub(super) fn emit(
    context: &Context<'_, '_>,
    declaration: &Declaration,
    actual: &[Coefficient],
    graph: &mut ExprGraph,
    output: &mut Output<'_>,
    work: &mut dyn Reservation,
) -> Result<(), CompilerError> {
    let members = &context.inputs.domain_eligible_members;
    let elements = members
        .iter()
        .filter(|row| row.domain_id == declaration.element.domain.as_id())
        .map(|row| {
            output.use_row(row);
            row.member_id
        })
        .collect::<BTreeSet<_>>();
    let species = members
        .iter()
        .filter(|row| row.domain_id == declaration.species.domain.as_id())
        .map(|row| {
            output.use_row(row);
            row.member_id
        })
        .collect::<BTreeSet<_>>();
    let expected = elements
        .len()
        .checked_mul(species.len())
        .ok_or_else(|| invalid("element coefficient product overflow"))?;
    if actual.len() != expected || expected == 0 {
        return Err(invalid(
            "element coefficient join is incomplete: every actual species needs explicit composition",
        ));
    }
    work.try_grow(
        expected
            .checked_mul(16384)
            .ok_or_else(|| invalid("element output reservation overflow"))?,
    )
    .map_err(pse_catalog::CatalogError::from)?;
    let base = output.active.clone();
    let mut seen = BTreeSet::new();
    output.push(declaration.header.clone())?;
    output.push(compiled::symbol_groups::Row {
        group_id: declaration.group,
        owner_instance_id: declaration.owner,
        name: format!("element_coefficients:{}", declaration.group.to_hex()),
        product_id: declaration.product,
    })?;
    let mut next_ordinal =
        output
            .rows::<compiled::symbols::Row>()?
            .iter()
            .try_fold(0u64, |next, row| {
                Ok::<_, CompilerError>(
                    next.max(
                        row.ordinal
                            .checked_add(1)
                            .ok_or_else(|| invalid("symbol ordinal overflow"))?,
                    ),
                )
            })?;
    for row in actual {
        context.ctx.cancel.checkpoint()?;
        output.active = base.clone();
        output.active.extend(row.support.iter().cloned());
        if row.group != declaration.group
            || !elements.contains(&row.element_member)
            || !species.contains(&row.species_member)
            || !seen.insert((row.element_member, row.species_member))
        {
            return Err(invalid(
                "element coefficient tuple is duplicate or outside the actual product",
            ));
        }
        let count = row.count.unwrap_or(0.0);
        if !count.is_finite() || count < 0.0 {
            return Err(invalid(
                "actual element count must be finite and nonnegative",
            ));
        }
        let molecular_weight = if declaration.mass {
            let value = row.mw.ok_or_else(|| {
                invalid("mass Element projection needs actual kg/mol molecular weight")
            })?;
            if !value.is_finite() || value <= 0.0 {
                return Err(invalid(
                    "mass Element projection molecular weight must be positive and finite",
                ));
            }
            Some(value)
        } else {
            None
        };
        let value = molecular_weight.map_or(count, |mw| count / mw);
        if !value.is_finite() {
            return Err(invalid("element coefficient is not finite"));
        }
        let index = vec![row.element_member, row.species_member];
        let symbol = symbol_instance_id(declaration.owner, declaration.group, IndexTuple(&index));
        let node = graph.insert_typed(
            Opcode::Const,
            Payload::FloatConst {
                value,
                unit: declaration.unit,
            },
            &[],
            declaration.scalar,
            Some(declaration.owner),
        )?;
        output.record_graph(graph, [node])?;
        let unbounded = extension_values::Bound {
            kind: enums::BoundKind::Unbounded,
            value: None,
        };
        output.push(compiled::symbols::Row {
            symbol_id: symbol,
            ordinal: next_ordinal,
            owner_instance_id: declaration.owner,
            symbol_decl_id: declaration.group,
            qualified_name: format!("element_coefficient:{}", symbol.to_hex()),
            index: index.clone(),
            quantity_type_id: declaration.scalar.as_id(),
            unit_id: declaration.unit.as_id(),
            role: enums::SymbolRole::Expression,
            solver_type: enums::SolverVariableType::Continuous,
            semantic_role: enums::VariableSemanticRole::ReportingOnly,
            lifecycle: enums::VariableLifecycle::GeneratedSemantic,
            default_lower: unbounded.clone(),
            default_upper: unbounded,
            default_initial: None,
            derivation_id: declaration.group,
        })?;
        next_ordinal = next_ordinal
            .checked_add(1)
            .ok_or_else(|| invalid("symbol ordinal overflow"))?;
        output.push(compiled::symbol_group_members::Row {
            group_id: declaration.group,
            tuple: index.clone(),
            symbol_id: symbol,
        })?;
        output.push(compiled::symbol_expressions::Row {
            symbol_id: symbol,
            node_id: node.0,
            derivation_id: declaration.group,
        })?;
        output.push(compiled::expression_roots::Row {
            owner_id: symbol,
            role: enums::ExpressionRootRole::SymbolExpression,
            ordinal: 0,
            node_id: node.0,
            derivation_id: declaration.group,
        })?;
        output.push(compiled::element_projection_coefficients::Row {
            group_id: declaration.group,
            index,
            symbol_id: symbol,
            node_id: node.0,
            element_id: row.element_id,
            species_id: row.species_id,
            count,
            molecular_weight,
            value,
            derivation_id: declaration.group,
        })?;
    }
    output.active = base;
    for row in actual {
        output.active.extend(row.support.iter().cloned());
    }
    Ok(())
}
