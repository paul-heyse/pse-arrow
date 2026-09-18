// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Extend the current target mathematical graph used by selected method realization.
use super::{Inputs, Realizer, invalid, resource};
use crate::passes::native_graph::{node_mapping, root_field};
use crate::{
    CompilerError,
    mathir_relations::{RelationSource, SourceFamily},
    passes::native_construction::{append, c, error, join, prefix, project},
};
use datafusion::arrow::array::FixedSizeBinaryArray;
use datafusion::{
    arrow::array::Array,
    logical_expr::{JoinType, LogicalPlan, col},
};
use pse_catalog::session::{output::declare_relation_output, scalar};
use pse_mathir::NodeId;
use pse_quantity::DomainId;
use pse_relations::{
    columnar::{FieldCheckedBatch, RelationRow},
    generated::compiled,
};
use std::collections::{BTreeMap, BTreeSet};

impl Realizer<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "load_input_graph keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) async fn load_input_graph(&mut self) -> Result<(), CompilerError> {
        let input_graph = self
            .inputs
            .iter()
            .filter(|(key, _)| {
                let mathematical = key.namespace == pse_schema::model::Namespace::Compiled
                    && !matches!(
                        key.name,
                        "method_parameter_bindings"
                            | "method_realizations"
                            | "kernel_output_symbols"
                    )
                    || key.namespace == pse_schema::model::Namespace::Inferred
                        && (key.name.starts_with("math_")
                            || matches!(key.name, "kernel_bindings" | "connection_equations"));
                mathematical
                    && self
                        .spec
                        .outputs
                        .iter()
                        .any(|port| port.relation == key.qualified_name())
            })
            .map(|(key, batch)| (*key, batch.clone()))
            .collect::<Inputs>();
        let extent = input_graph.values().try_fold(0usize, |sum, input| {
            sum.checked_add(pse_ids::validation_extent(input.batch())?)
                .ok_or_else(|| invalid("input_graph graph extent overflow"))
        })?;
        self.work
            .try_grow(
                extent
                    .checked_mul(3)
                    .ok_or_else(|| invalid("input_graph graph extent overflow"))?,
            )
            .map_err(|_| resource())?;
        let mut graph_rows = input_graph.clone();
        for name in [
            "math_dae_links",
            "math_implicit_systems",
            "math_equations",
            "math_objectives",
            "math_complementarity",
        ] {
            let spec = self
                .registry
                .relation(&format!("inferred.{name}"))
                .ok_or_else(|| invalid("input_graph graph side-table declaration absent"))?;
            let input = input_graph
                .get(&spec.key)
                .ok_or_else(|| invalid("input_graph graph side-table binding absent"))?;
            if !matches!(name, "math_dae_links" | "math_implicit_systems")
                && input.batch().num_rows() != 0
            {
                return Err(invalid(
                    "scalar solver objects cannot occur in an indexed input graph",
                ));
            }
            graph_rows.insert(
                spec.key,
                FieldCheckedBatch::concat_reserved(
                    self.registry,
                    spec,
                    &[],
                    self.reserver,
                    self.cancel,
                )?,
            );
        }
        let source =
            RelationSource::from_checked(&graph_rows, self.registry, SourceFamily::Inferred)?;
        let loaded = pse_mathir::relations::load_untyped(&source, &[])?;
        let mapping = loaded.node_mapping;
        self.graph = loaded.graph;
        self.equations = loaded.equations;
        self.kernels = loaded.kernel_bindings;
        self.selections = loaded.selections;
        self.work
            .try_grow(
                mapping
                    .len()
                    .checked_mul(2048)
                    .ok_or_else(|| invalid("native input_graph remap extent overflow"))?,
            )
            .map_err(|_| resource())?;
        let mapping_plan = node_mapping(&mapping)?;
        let mapped = self
            .session
            .prepare_rule_plan(mapping_plan, self.cancel)?
            .execute(self.cancel)
            .await?;
        let session = self.session.with_computation_roles(
            BTreeMap::from([("P7:actual-input_graph-node-map".to_owned(), mapped)]),
            self.cancel,
        )?;
        let session = session.with_checked_role_inputs(
            input_graph
                .iter()
                .map(|(key, input)| {
                    (
                        format!("P7:input_graph:{}", key.qualified_name()),
                        input.clone(),
                    )
                })
                .collect(),
            self.cancel,
        )?;
        for key in input_graph.keys() {
            self.cancel.checkpoint()?;
            let spec = self
                .registry
                .relation(&key.qualified_name())
                .ok_or_else(|| invalid("input_graph declaration absent"))?;
            let plan = session.scan_role(&format!("P7:input_graph:{}", key.qualified_name()))?;
            let mut plan = append(
                plan,
                [scalar::key(
                    spec.id,
                    spec.primary_key
                        .iter()
                        .map(|name| (*name, col(*name)))
                        .collect(),
                )
                .alias("__input_graph_source_key")],
            )?;
            let graph = key.namespace == pse_schema::model::Namespace::Inferred
                && (key.name.starts_with("math_") || key.name == "kernel_bindings")
                && !matches!(key.name, "math_dae_links" | "math_implicit_systems");
            if let Some(field) = root_field(key) {
                plan = join(
                    plan,
                    prefix(
                        session.scan_computation_role("P7:actual-input_graph-node-map")?,
                        "mapped",
                    )?,
                    JoinType::Left,
                    [col(field).eq(c("mapped", "old_node"))],
                )?;
                let required = session
                    .scalar_function("pse_require_nonnull")?
                    .call(vec![c("mapped", "new_node")]);
                let fields = spec
                    .columns
                    .iter()
                    .map(|column| {
                        if column.name() == field {
                            required.clone().alias(field)
                        } else {
                            col(column.name())
                        }
                    })
                    .chain([col("__input_graph_source_key")])
                    .collect::<Vec<_>>();
                plan = project(plan, fields)?;
            }
            // Attach only field annotations checked against the native result; the
            // source key remains in this same completion beside the remapped values.
            let value_plan = project(
                plan.clone(),
                spec.columns.iter().map(|column| col(column.name())),
            )?;
            let declared =
                declare_relation_output(value_plan, self.registry, spec).map_err(error)?;
            let mut fields = match declared {
                LogicalPlan::Projection(projection) => projection.expr,
                _ => {
                    return Err(invalid(
                        "declared input_graph projection is not a native projection",
                    ));
                }
            };
            fields.push(col("__input_graph_source_key"));
            plan = project(plan, fields)?;
            let complete = self
                .inventory
                .arguments
                .execute(plan, &session, self.cancel)
                .await?;
            for batch in complete.batches() {
                let values = FieldCheckedBatch::admit_owned_projection(
                    self.registry,
                    spec,
                    batch,
                    &(0..spec.columns.len()).collect::<Vec<_>>(),
                )?;
                let keys = batch
                    .batch()
                    .column(spec.columns.len())
                    .as_any()
                    .downcast_ref::<FixedSizeBinaryArray>()
                    .ok_or_else(|| invalid("input_graph source key storage differs"))?;
                let mut support = Vec::new();
                for row in 0..values.batch().num_rows() {
                    if keys.is_null(row) {
                        return Err(invalid("input_graph source key is null"));
                    }
                    let origin = self.source_position(
                        *key,
                        &crate::passes::native_rows::key_value(keys.value(row))?,
                    )?;
                    support.push(origin);
                    if graph {
                        for name in ["node_id", "parent_node_id"] {
                            if let Some(old) =
                                crate::passes::native_graph::ordinal(&values, name, row)?
                            {
                                let mapped = mapping.get(&NodeId(old)).ok_or_else(|| {
                                    invalid(
                                        "input_graph graph node is outside its actual admitted mapping",
                                    )
                                })?;
                                self.node_support.entry(*mapped).or_default().insert(origin);
                            }
                        }
                    }
                    for (name, output) in [
                        ("indexed_equation_id", &mut self.equation_support),
                        ("binding_id", &mut self.kernel_support),
                    ] {
                        if let Some(id) = crate::passes::native_graph::identity(&values, name, row)?
                        {
                            output.entry(id).or_default().insert(origin);
                        }
                    }
                }
                if graph {
                    continue;
                }
                if *key == compiled::symbols::RELATION_KEY {
                    for (position, symbol) in compiled::symbols::Row::rows(&values)?
                        .into_iter()
                        .enumerate()
                    {
                        if self.selected.contains(&symbol.owner_instance_id) {
                            return Err(invalid(
                                "selected instance already has realized predecessor symbols",
                            ));
                        }
                        self.symbol_support
                            .entry(symbol.symbol_id)
                            .or_default()
                            .insert(support[position]);
                        let key = (
                            symbol.owner_instance_id,
                            symbol.symbol_decl_id,
                            symbol.index.clone(),
                        );
                        if self.symbols.insert(key, symbol).is_some() {
                            return Err(invalid("input_graph symbol correspondence repeated"));
                        }
                    }
                } else {
                    let actual = support
                        .into_iter()
                        .map(|source| self.source_keys(&BTreeSet::from([source])))
                        .collect::<Result<Vec<_>, _>>()?;
                    self.output.append_checked(&values, |_, row| {
                        actual.get(row).cloned().ok_or_else(|| {
                            invalid("input_graph source/result correspondence absent")
                        })
                    })?;
                }
            }
        }
        self.input_groups(&input_graph)
    }
    fn input_groups(&mut self, input_graph: &Inputs) -> Result<(), CompilerError> {
        let groups: Vec<compiled::symbol_groups::Row> = super::inventory::decode(
            input_graph,
            self.registry,
            compiled::symbol_groups::spec(self.registry)?,
        )?;
        let members: Vec<compiled::symbol_group_members::Row> = super::inventory::decode(
            input_graph,
            self.registry,
            compiled::symbol_group_members::spec(self.registry)?,
        )?;
        for group in groups {
            let product = self
                .inventory
                .products
                .iter()
                .find(|row| row.product_id == group.product_id)
                .ok_or_else(|| {
                    invalid("input_graph group product absent from actual input facts")
                })?;
            let group_members = members
                .iter()
                .filter(|row| row.group_id == group.group_id)
                .map(|row| (row.tuple.clone(), row.symbol_id))
                .collect::<BTreeMap<_, _>>();
            let declarations = group_members
                .values()
                .map(|symbol| {
                    self.symbols
                        .values()
                        .find(|row| row.symbol_id == *symbol)
                        .map(|row| row.symbol_decl_id)
                        .ok_or_else(|| invalid("input_graph group member has no actual symbol"))
                })
                .collect::<Result<BTreeSet<_>, _>>()?;
            if declarations.len() == 1 {
                let declaration = *declarations
                    .first()
                    .ok_or_else(|| invalid("input_graph group declaration absent"))?;
                if self
                    .inventory
                    .symbols
                    .iter()
                    .any(|row| row.symbol_decl_id == declaration)
                    && group.group_id
                        == pse_templates::identity::symbol_group_id(
                            group.owner_instance_id,
                            declaration,
                        )
                {
                    self.groups.insert(
                        (group.owner_instance_id, declaration),
                        pse_templates::GroupBinding {
                            group: group.group_id,
                            axes: product
                                .domain_ids
                                .iter()
                                .copied()
                                .map(DomainId::from_id)
                                .collect(),
                            members: group_members,
                        },
                    );
                }
            }
        }
        Ok(())
    }
}
