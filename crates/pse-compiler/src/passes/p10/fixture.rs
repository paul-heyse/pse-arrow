// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit complete predecessor fixture registry (ADR-0056); no invented P4–P9 outputs.
mod model;
mod physical;
use super::{Context, invalid};
use crate::CompilerError;
pub use model::{model_inputs, physical_package};
pub use physical::{physical_rows, unit_rows};
use pse_mathir::NodeId;
use pse_quantity::{QuantityKindId, QuantityTypeId};
use pse_relations::RecordBatch;
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, ColumnSpec, Determinism, InputPort, LogicalType as T, Namespace,
        OutputPort, PassDecl, PortSource, RelationDecl, RelationKey, SnapshotClass,
    },
};
use std::collections::BTreeMap;

/// Complete explicit input rows for the fixture expression `1 + 2`.
///
/// The returned context is also present as the actual required input relation. The
/// expected canonical result is one integer constant with value three.
/// # Errors
/// Missing fixture declarations, malformed physical rows or graph emission failure.
pub fn arithmetic_inputs(
    registry: &Registry,
) -> Result<(BTreeMap<RelationKey, RecordBatch>, Context), CompilerError> {
    let mut rows = physical_rows(registry)?;
    let pass = registry
        .pass("P10@1")
        .ok_or_else(|| invalid("P10 fixture pass undeclared"))?;
    for port in &pass.inputs {
        let spec = registry
            .relation(&port.relation)
            .ok_or_else(|| invalid("P10 fixture input undeclared"))?;
        if let std::collections::btree_map::Entry::Vacant(entry) = rows.entry(spec.key) {
            entry.insert(pse_relations::cells::batch_from_cells(registry, spec, &[])?);
        }
    }
    let mut graph = pse_mathir::ExprGraph::new();
    let one = graph.int_const(1)?;
    let two = graph.int_const(2)?;
    let root = graph.add(one, two)?;
    let mut sink = crate::mathir_relations::RelationSink::new(
        registry,
        crate::mathir_relations::Family::Compiled,
    );
    pse_mathir::relations::emit_untyped(&graph, &[root], &mut sink)?;
    for (key, values) in sink.into_rows() {
        let spec = registry
            .relation(&key.qualified_name())
            .ok_or_else(|| invalid("P10 math relation undeclared"))?;
        rows.insert(
            key,
            pse_relations::cells::batch_from_cells(registry, spec, &values)?,
        );
    }
    let neutral = pse_ids::SemanticId::from_bytes([31; 16]);
    let context = Context {
        neutral: Some(QuantityTypeId::from_id(neutral)),
        boolean: None,
        roots: vec![root],
    };
    let spec = registry
        .relation("reference.p10_fixture_context")
        .ok_or_else(|| invalid("P10 context undeclared"))?;
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(
            registry,
            spec,
            &[vec![
                Cell::U64(0),
                Cell::Id(neutral),
                Cell::Null,
                Cell::List(vec![Cell::U64(root.0)]),
            ]],
        )?,
    );
    Ok((rows, context))
}

/// Build the platform registry extended with only explicit P10 fixture context and ports.
/// The production registry remains unchanged. Every predecessor is an actual pinned or
/// explicit external fixture binding, never a stub pass result.
///
/// # Errors
/// Registry structural, reference, enum, invariant or pass-graph admission failure.
pub fn registry() -> Result<Registry, CompilerError> {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare(&mut builder);
    declare_context(&mut builder);
    let relations = builder.declared_relations().to_vec();
    let mut inputs = vec![];
    let mut outputs = vec![];
    for relation in &relations {
        let qualified = relation.key.qualified_name();
        let math = relation.key.namespace == Namespace::Compiled
            && (relation.key.name.starts_with("math_") || relation.key.name == "kernel_bindings");
        let required = math
            || matches!(
                qualified.as_str(),
                "reference.units"
                    | "reference.unit_sets"
                    | "reference.quantity_kinds"
                    | "reference.bases"
                    | "reference.reference_states"
                    | "reference.quantity_types"
                    | "reference.conversion_rules"
                    | "reference.quantity_operations"
                    | "reference.kernel_specs"
                    | "reference.p10_fixture_context"
                    | "authored.domains"
                    | "authored.continuous_domains"
                    | "authored.domain_members"
                    | "authored.instances"
                    | "authored.template_symbols"
                    | "authored.template_domains"
                    | "authored.instance_domain_bindings"
                    | "normalized.domain_products"
                    | "normalized.units"
                    | "inferred.valid_index_tuples"
                    | "compiled.symbols"
                    | "compiled.symbol_groups"
                    | "compiled.symbol_group_members"
            );
        if required {
            inputs.push(InputPort {
                port: if qualified == "normalized.units" {
                    "normalized_units"
                } else {
                    relation.key.name
                },
                relation: qualified.clone(),
                source: PortSource::Pinned,
                required: true,
            });
        }
        if math {
            outputs.push(OutputPort {
                port: relation.key.name,
                relation: qualified,
            });
        }
    }
    let mut importer_inputs = vec![];
    let mut importer_outputs = vec![];
    for input in &inputs {
        let relation = relations
            .iter()
            .find(|relation| relation.key.qualified_name() == input.relation)
            .ok_or_else(|| invalid("P10 fixture importer relation undeclared"))?;
        if relation.snapshot_class == SnapshotClass::Model {
            importer_inputs.push(input.clone());
        } else {
            importer_outputs.push(OutputPort {
                port: input.port,
                relation: input.relation.clone(),
            });
        }
    }
    builder.declare_pass(
        PassDecl::new("FixtureP10Inputs", "1", Determinism::Deterministic)
            .inputs(importer_inputs)
            .outputs(importer_outputs)
            .diagnostics(vec!["compile.math", "runtime.resource_limit"]),
    );
    builder.declare_pass(
        PassDecl::new("P10", "1", Determinism::Deterministic)
            .inputs(inputs)
            .outputs(outputs)
            .diagnostics(vec!["compile.math", "runtime.resource_limit"]),
    );
    Ok(builder.build()?)
}
fn declare_context(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Reference,
            "p10_fixture_context",
            1,
            Authority::Reference,
            SnapshotClass::Model,
            "ADR-0056 explicit P10 invocation context",
        )
        .pk(&["context_id"])
        .columns(vec![
            ColumnSpec::key("context_id", T::U8, "The only context row has ordinal zero"),
            ColumnSpec::payload(
                "neutral_quantity_type_id",
                T::id(),
                "Explicit neutral scalar",
            )
            .optional()
            .with_fk("reference.quantity_types", "quantity_type_id"),
            ColumnSpec::payload("boolean_quantity_kind_id", T::id(), "Explicit Boolean kind")
                .optional()
                .with_fk("reference.quantity_kinds", "quantity_kind_id"),
            ColumnSpec::payload(
                "root_node_ids",
                T::list(T::U64),
                "Explicit ordered source storage roots",
            ),
        ]),
    );
}
/// Read the complete explicit context from its actual one-row fixture input.
/// # Errors
/// Missing/duplicate context, wrong ordinal or malformed typed field values.
pub fn context(
    inputs: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
) -> Result<Context, CompilerError> {
    let spec = registry
        .relation("reference.p10_fixture_context")
        .ok_or_else(|| invalid("P10 fixture context is undeclared"))?;
    let batch = inputs
        .get(&spec.key)
        .ok_or_else(|| invalid("P10 fixture context input is absent"))?;
    let rows = pse_relations::cells::cells_from_batch(registry, spec, batch)?;
    let [row] = rows.as_slice() else {
        return Err(invalid("P10 requires exactly one explicit context row"));
    };
    let get = |name: &str| {
        spec.columns
            .iter()
            .position(|column| column.name == name)
            .and_then(|index| row.get(index))
            .ok_or_else(|| invalid("P10 context field absent"))
    };
    if get("context_id")? != &Cell::U64(0) {
        return Err(invalid("P10 context ordinal must be zero"));
    }
    let id = |name: &str| match get(name)? {
        Cell::Null => Ok(None),
        Cell::Id(id) => Ok(Some(*id)),
        _ => Err(invalid("P10 context identity is malformed")),
    };
    let Cell::List(values) = get("root_node_ids")? else {
        return Err(invalid("P10 roots are not an ordered list"));
    };
    let roots = values
        .iter()
        .map(|cell| match cell {
            Cell::U64(id) => Ok(NodeId(*id)),
            _ => Err(invalid("P10 root ordinal is malformed")),
        })
        .collect::<Result<_, _>>()?;
    Ok(Context {
        neutral: id("neutral_quantity_type_id")?.map(QuantityTypeId::from_id),
        boolean: id("boolean_quantity_kind_id")?.map(QuantityKindId::from_id),
        roots,
    })
}
