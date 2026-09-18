// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{
    AlgorithmInputs, BTreeMap, CancellationToken, CompilerError, Inputs, Inventory, Keyed, NodeId,
    Registry, RelationRow, SemanticId, SnapshotSession, Support, input_role, invalid, n,
};
use crate::passes::native_rows::{self, engine};
use datafusion::{
    arrow::array::{Array, FixedSizeBinaryArray, Int64Array},
    common::ScalarValue,
    logical_expr::{LogicalPlanBuilder, col, lit},
};
use n::expression_sources::NormalizedExpressionSourcesFieldOwnerSelected as Owner;
use pse_catalog::session::{output::checked_literal, scalar};

pub(super) async fn rows<T: RelationRow>(
    arguments: &mut AlgorithmInputs,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<Vec<Keyed<T>>, CompilerError> {
    let spec = T::relation(registry)?;
    native_rows::keyed_rows(
        arguments,
        session.scan_role(&input_role(spec.key))?,
        session,
        registry,
        cancel,
    )
    .await
}

// The shared evaluator accepts each pass's declared subset. Any operation that
// consumes a missing declaration still fails at its exact typed lookup.
pub(super) async fn available<T: RelationRow>(
    inputs: &Inputs,
    arguments: &mut AlgorithmInputs,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<Vec<Keyed<T>>, CompilerError> {
    if inputs.contains_key(&T::relation(registry)?.key) {
        rows(arguments, session, registry, cancel).await
    } else {
        Ok(Vec::new())
    }
}

pub(super) async fn node_origins(
    inputs: &Inputs,
    arguments: &mut AlgorithmInputs,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<BTreeMap<(String, NodeId), Support>, CompilerError> {
    let mut result: BTreeMap<_, Support> = BTreeMap::new();
    for key in inputs.keys() {
        if key.namespace != pse_schema::model::Namespace::Normalized {
            continue;
        }
        let Some((family, _)) = ["template", "instance", "display", "contribution", "guard"]
            .into_iter()
            .find_map(|family| {
                registry
                    .relations()
                    .iter()
                    .find(|spec| {
                        spec.key.namespace == pse_schema::model::Namespace::Compiled
                            && pse_schema::catalog::expr_family::target_name(family, spec.key.name)
                                == Some(key.name)
                    })
                    .map(|spec| (family, spec))
            })
        else {
            continue;
        };
        let spec = registry
            .relation(&key.qualified_name())
            .ok_or_else(|| invalid("node source declaration absent"))?;
        let Some(node) = spec
            .column("node_id")
            .or_else(|| spec.column("parent_node_id"))
        else {
            continue;
        };
        let plan = LogicalPlanBuilder::from(session.scan_role(&input_role(*key))?)
            .project([
                col(node.name()).alias("source_node"),
                scalar::key(
                    spec.id,
                    spec.primary_key
                        .iter()
                        .map(|name| (*name, col(*name)))
                        .collect(),
                )
                .alias("source_key"),
            ])
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        let completed = arguments.execute(plan, session, cancel).await?;
        for batch in completed.batches() {
            let nodes = batch
                .column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .ok_or_else(|| invalid("source node is not Int64"))?;
            let keys = batch
                .column(1)
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .ok_or_else(|| invalid("source node key is not a typed key"))?;
            for row in 0..batch.num_rows() {
                if nodes.is_null(row) || keys.is_null(row) || nodes.value(row) < 0 {
                    return Err(invalid("source node occurrence is null"));
                }
                result
                    .entry((family.to_owned(), NodeId(nodes.value(row))))
                    .or_default()
                    .insert((*key, native_rows::key_value(keys.value(row))?));
            }
        }
    }
    Ok(result)
}

impl Inventory<'_> {
    pub(crate) async fn source_instances(
        &mut self,
        source: &n::expression_sources::Row,
        cancel: &CancellationToken,
    ) -> Result<Vec<Keyed<n::instance_bindings::Row>>, CompilerError> {
        let spec = n::instance_bindings::Row::relation(self.registry)?;
        let plan = self.session.scan_role(&input_role(spec.key))?;
        let (name, identity) = match source.owner.selected()? {
            Owner::Instance(value) => ("instance_id", value.instance_id),
            Owner::Template(value) => ("template_id", value.template_id),
        };
        let field = spec
            .column(name)
            .ok_or_else(|| invalid("instance identity field absent"))?;
        let value = checked_literal(
            self.registry,
            field,
            ScalarValue::FixedSizeBinary(16, Some(identity.as_bytes().to_vec())),
        )
        .map_err(engine)?;
        let condition = col(name).eq(value);
        let plan = LogicalPlanBuilder::from(plan)
            .filter(condition)
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        native_rows::keyed_rows(
            &mut self.arguments,
            plan,
            &self.session,
            self.registry,
            cancel,
        )
        .await
    }

    /// DataFusion constructs the finite product; every member and its source key
    /// leave that same execution together. The empty-axis product contains one tuple.
    pub(crate) async fn tuples(
        &mut self,
        domains: &[SemanticId],
        cancel: &CancellationToken,
    ) -> Result<Vec<(Vec<SemanticId>, Support)>, CompilerError> {
        self.tuples_filtered(domains, &BTreeMap::new(), cancel)
            .await
    }
    pub(crate) async fn tuples_filtered(
        &mut self,
        domains: &[SemanticId],
        fixed: &BTreeMap<usize, SemanticId>,
        cancel: &CancellationToken,
    ) -> Result<Vec<(Vec<SemanticId>, Support)>, CompilerError> {
        let spec = n::domain_members::Row::relation(self.registry)?;
        let mut plan = LogicalPlanBuilder::empty(true)
            .project([lit(true).alias("unit")])
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        for (axis, domain) in domains.iter().enumerate() {
            let field = spec
                .column("domain_id")
                .ok_or_else(|| invalid("member domain field absent"))?;
            let value = checked_literal(
                self.registry,
                field,
                ScalarValue::FixedSizeBinary(16, Some(domain.as_bytes().to_vec())),
            )
            .map_err(engine)?;
            let members =
                LogicalPlanBuilder::from(self.session.scan_role(&input_role(spec.key))?)
                    .filter(col("domain_id").eq(value).and(
                        if let Some(member) = fixed.get(&axis) {
                            col("member_id").eq(checked_literal(
                                self.registry,
                                spec.column("member_id")
                                    .ok_or_else(|| invalid("member identity field absent"))?,
                                ScalarValue::FixedSizeBinary(16, Some(member.as_bytes().to_vec())),
                            )
                            .map_err(engine)?)
                        } else {
                            lit(true)
                        },
                    ))
                    .map_err(engine)?
                    .project([
                        col("member_id").alias(format!("member_{axis}")),
                        scalar::key(
                            spec.id,
                            spec.primary_key
                                .iter()
                                .map(|name| (*name, col(*name)))
                                .collect(),
                        )
                        .alias(format!("key_{axis}")),
                    ])
                    .and_then(LogicalPlanBuilder::build)
                    .map_err(engine)?;
            plan = LogicalPlanBuilder::from(plan)
                .cross_join(members)
                .and_then(LogicalPlanBuilder::build)
                .map_err(engine)?;
        }
        if !domains.is_empty() {
            plan = LogicalPlanBuilder::from(plan)
                .sort(
                    (0..domains.len()).map(|axis| col(format!("member_{axis}")).sort(true, false)),
                )
                .and_then(LogicalPlanBuilder::build)
                .map_err(engine)?;
        }
        let completed = self.arguments.execute(plan, &self.session, cancel).await?;
        let mut result = Vec::new();
        for batch in completed.batches() {
            for row in 0..batch.num_rows() {
                cancel.checkpoint()?;
                let mut tuple = Vec::with_capacity(domains.len());
                let mut support = Support::new();
                for axis in 0..domains.len() {
                    let members = batch
                        .column(1 + 2 * axis)
                        .as_any()
                        .downcast_ref::<FixedSizeBinaryArray>()
                        .ok_or_else(|| invalid("native tuple member is not an identity"))?;
                    let keys = batch
                        .column(2 + 2 * axis)
                        .as_any()
                        .downcast_ref::<FixedSizeBinaryArray>()
                        .ok_or_else(|| invalid("native tuple source key is not a typed key"))?;
                    if members.is_null(row) || keys.is_null(row) {
                        return Err(invalid("native tuple occurrence is null"));
                    }
                    tuple.push(
                        SemanticId::try_from_slice(members.value(row))
                            .map_err(|_| invalid("native tuple identity width differs"))?,
                    );
                    support.insert((spec.key, native_rows::key_value(keys.value(row))?));
                }
                result.push((tuple, support));
            }
        }
        Ok(result)
    }
}
