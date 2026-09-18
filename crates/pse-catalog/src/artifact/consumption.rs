// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native projection analysis establishes consumed fields before any simplifying
//! optimizer can erase a read. Unknown extensions and policy requirements remain whole.
use super::{ArtifactPlan, dependencies, invalid};
use crate::{CatalogError, session::SnapshotSession};
use datafusion::{
    common::{TableReference, tree_node::TreeNodeRecursion},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder},
    optimizer::OptimizerRule,
};
use pse_ids::CancellationToken;
use pse_relations::generated::runtime::{native_dependencies as deps, publications};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub(super) fn columns(
    artifact: &ArtifactPlan,
) -> Result<BTreeMap<String, BTreeSet<String>>, CatalogError> {
    if !artifact.session.effective_policy()?.requirements.is_empty() {
        return Ok(BTreeMap::new());
    }
    for output in artifact.outputs.values() {
        if projection_is_opaque(&output.plan)? {
            return Ok(BTreeMap::new());
        }
    }
    let mut columns: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for output in artifact.outputs.values() {
        // Only native projection analysis, never constant-folding/filter elimination.
        let projected = datafusion::optimizer::optimize_projections::OptimizeProjections::new()
            .rewrite(output.plan.clone(), &artifact.session.bound_state()?)
            .map_err(crate::session::engine)?
            .data;
        projected
            .apply_with_subqueries(|node| {
                if let LogicalPlan::TableScan(scan) = node {
                    // Native view expansion replaces the selected view with its
                    // actual retained providers. Resolve their registered lineage,
                    // including aliases, exactly as whole-input capture does.
                    for binding in artifact.session.source_bindings(node).map_err(|error| {
                        datafusion::common::DataFusionError::External(Box::new(error))
                    })? {
                        let Some(member) = &binding.selection else {
                            continue;
                        };
                        let Some(spec) = artifact
                            .session
                            .registry()
                            .relation_by_id(member.relation_id)
                        else {
                            continue;
                        };
                        let fields = columns
                            .entry(binding.reference.to_quoted_string())
                            .or_default();
                        fields.extend(
                            scan.projected_schema
                                .fields()
                                .iter()
                                .map(|field| field.name().clone()),
                        );
                        for predicate in &scan.filters {
                            fields.extend(
                                predicate
                                    .column_refs()
                                    .into_iter()
                                    .map(|column| column.name.clone()),
                            );
                        }
                        // Equality includes row membership, multiplicity and every key, even
                        // when a count/negative read emits no original payload fields.
                        fields.extend(spec.primary_key.iter().map(|name| (*name).to_owned()));
                    }
                }
                Ok(TreeNodeRecursion::Continue)
            })
            .map_err(crate::session::engine)?;
    }
    Ok(columns)
}

fn projection_is_opaque(plan: &LogicalPlan) -> Result<bool, CatalogError> {
    // One extension already requires whole-input evidence. Do not expand its
    // shared producer graph after that decision; native Stop also propagates
    // from subqueries and prevents traversal of later siblings.
    plan.apply_with_subqueries(|node| {
        Ok(if matches!(node, LogicalPlan::Extension(_)) {
            TreeNodeRecursion::Stop
        } else {
            TreeNodeRecursion::Continue
        })
    })
    .map(|recursion| recursion == TreeNodeRecursion::Stop)
    .map_err(crate::session::engine)
}

pub(super) fn projection(
    member: publications::RuntimePublicationsFieldMembersItem,
    columns: BTreeSet<String>,
) -> Result<deps::RuntimeNativeDependenciesFieldEvidenceProjection, CatalogError> {
    use publications::RuntimePublicationsFieldMembersItemSelectionSelected as Selected;
    let selection = match member.selection.selected()? {
        Selected::Full => {
            deps::RuntimeNativeDependenciesFieldEvidenceProjectionSelectionSelection::from_full()
        }
        Selected::Revision(value) => {
            deps::RuntimeNativeDependenciesFieldEvidenceProjectionSelectionSelection::from_revision(
                deps::RuntimeNativeDependenciesFieldEvidenceProjectionSelectionSelectionRevision {
                    column: value.column.clone(),
                    revision_id: value.revision_id,
                },
            )
        }
    };
    Ok(deps::RuntimeNativeDependenciesFieldEvidenceProjection {
        columns: columns.into_iter().collect(),
        selection: deps::RuntimeNativeDependenciesFieldEvidenceProjectionSelection {
            catalog_name: member.catalog_name,
            schema_name: member.schema_name,
            table_name: member.table_name,
            relation_id: member.relation_id,
            relation_version: member.relation_version,
            contract_fingerprint: member.contract_fingerprint,
            table_uri: member.table_uri,
            delta_version: member.delta_version,
            selection,
        },
    })
}
fn member(
    value: &deps::RuntimeNativeDependenciesFieldEvidenceProjection,
) -> Result<publications::RuntimePublicationsFieldMembersItem, CatalogError> {
    use deps::RuntimeNativeDependenciesFieldEvidenceProjectionSelectionSelectionSelected as Selected;
    let source = &value.selection;
    let selection = match source.selection.selected()? {
        Selected::Full => publications::RuntimePublicationsFieldMembersItemSelection::from_full(),
        Selected::Revision(value) => {
            publications::RuntimePublicationsFieldMembersItemSelection::from_revision(
                publications::RuntimePublicationsFieldMembersItemSelectionRevision {
                    column: value.column.clone(),
                    revision_id: value.revision_id,
                },
            )
        }
    };
    Ok(publications::RuntimePublicationsFieldMembersItem {
        catalog_name: source.catalog_name.clone(),
        schema_name: source.schema_name.clone(),
        table_name: source.table_name.clone(),
        relation_id: source.relation_id,
        relation_version: source.relation_version,
        contract_fingerprint: source.contract_fingerprint,
        table_uri: source.table_uri.clone(),
        delta_version: source.delta_version,
        selection,
    })
}
/// Build exact native value comparisons for changed versions and metadata equality
/// for everything else. No digest or projection label can establish equivalence.
pub(super) async fn comparison(
    mut session: SnapshotSession,
    current: Vec<deps::Row>,
    previous: &[deps::Row],
    cancel: &CancellationToken,
) -> Result<(SnapshotSession, LogicalPlan), CatalogError> {
    let mut current = current;
    let mut previous = previous.to_vec();
    let mut comparisons = Vec::new();
    for (index, row) in current.iter_mut().enumerate() {
        let mut matches = previous
            .iter_mut()
            .filter(|old| old.kind == row.kind && old.scope == row.scope && old.name == row.name);
        let Some(old) = matches.next() else {
            continue;
        };
        if matches.next().is_some() {
            continue;
        }
        // Keep duplicate and malformed facts visible to the complete metadata difference.
        row.evidence.selected()?;
        old.evidence.selected()?;
        let (Some(new_projection), Some(old_projection)) =
            (&mut row.evidence.projection, &mut old.evidence.projection)
        else {
            continue;
        };
        let mut normalized_new = new_projection.clone();
        let mut normalized_old = old_projection.clone();
        normalized_new.selection.delta_version = 0;
        normalized_old.selection.delta_version = 0;
        if normalized_new != normalized_old {
            continue;
        }
        if new_projection.selection.delta_version == old_projection.selection.delta_version {
            continue;
        }
        // A complete bounded native CDF interval proves the same multiset delta
        // with fewer decoded rows. History/contract refusal uses exact endpoint
        // comparison; never interpret unavailable CDF as no change.
        if new_projection.selection.delta_version > old_projection.selection.delta_version {
            let selected = member(new_projection)?;
            let reference = datafusion::common::ResolvedTableReference {
                catalog: selected.catalog_name.clone().into(),
                schema: selected.schema_name.clone().into(),
                table: selected.table_name.clone().into(),
            };
            match session
                .change_plan(&reference, old_projection.selection.delta_version, cancel)
                .await
            {
                Ok((bound, changes)) => {
                    crate::cache_service::metrics::record(&bound.bound_state()?, |metrics| {
                        &metrics.cdf_comparisons
                    });
                    let difference = changed_values(&changes, &new_projection.columns)?;
                    comparisons.push(violation(difference, &row.name)?);
                    session = bound;
                    new_projection.selection.delta_version = 0;
                    old_projection.selection.delta_version = 0;
                    continue;
                }
                Err(_) => {
                    cancel.checkpoint()?;
                }
            }
        }
        let mut plans = Vec::new();
        crate::cache_service::metrics::record(&session.bound_state()?, |metrics| {
            &metrics.endpoint_comparisons
        });
        for (side, projected) in [
            ("current", &*new_projection),
            ("previous", &*old_projection),
        ] {
            plans.push(endpoint_plan(&mut session, projected, side, index).await?);
        }
        let right = plans
            .pop()
            .ok_or_else(|| invalid("previous consumed input absent"))?;
        let left = plans
            .pop()
            .ok_or_else(|| invalid("current consumed input absent"))?;
        comparisons.push(violation(
            dependencies::difference(left, right).map_err(crate::session::engine)?,
            &row.name,
        )?);
        new_projection.selection.delta_version = 0;
        old_projection.selection.delta_version = 0;
    }
    let (bound, expected) =
        dependencies::bind(&session, "expected_dependencies", &current, cancel)?;
    let (bound, actual) = dependencies::bind(&bound, "previous_dependencies", &previous, cancel)?;
    let mut difference = violation(
        dependencies::difference(expected.plan().clone(), actual.plan().clone())
            .map_err(crate::session::engine)?,
        "dependency metadata",
    )?;
    for comparison in comparisons {
        difference = LogicalPlanBuilder::from(difference)
            .union(comparison)
            .and_then(LogicalPlanBuilder::build)
            .map_err(crate::session::engine)?;
    }
    Ok((bound, difference))
}

async fn endpoint_plan(
    session: &mut SnapshotSession,
    projected: &deps::RuntimeNativeDependenciesFieldEvidenceProjection,
    side: &str,
    index: usize,
) -> Result<LogicalPlan, CatalogError> {
    let selected = member(projected)?;
    let state = Arc::new(session.bound_state()?);
    let provider =
        crate::delta::publication::selected_provider(&selected, session.registry(), state)
            .await
            .map_err(crate::session::engine)?;
    let name = TableReference::full("reuse", "inputs", format!("{side}_{index}"));
    let spec = session
        .registry()
        .relation_by_id(selected.relation_id)
        .ok_or_else(|| invalid("consumed input contract absent"))?;
    // Internal exact selected providers, not arbitrary external source admission.
    let mut binding = crate::provider::binding::TableBinding::new(
        name.clone(),
        provider.clone(),
        Some(spec.key),
        None,
    );
    binding.selection = Some(selected);
    session
        .bindings
        .insert(
            crate::provider::binding::BindingKey::Native(name.clone()),
            binding,
        )
        .map_err(crate::session::engine)?;
    let plan = LogicalPlanBuilder::scan(
        name,
        datafusion::datasource::provider_as_source(provider),
        None,
    )
    .and_then(|builder| {
        builder.project(
            projected
                .columns
                .iter()
                .map(|name| Expr::Column(datafusion::common::Column::from_name(name))),
        )
    })
    .and_then(LogicalPlanBuilder::build)
    .map_err(crate::session::engine)?;
    Ok(plan)
}
fn changed_values(changes: &LogicalPlan, columns: &[String]) -> Result<LogicalPlan, CatalogError> {
    use datafusion::functions::core::expr_fn::get_field;
    use datafusion::logical_expr::lit;
    // Qualify nested inputs before projecting a leaf with the same name as its
    // parent struct (a perfectly valid user column named "value").
    let input = |name: &str| {
        Expr::Column(datafusion::common::Column::new(
            Some(TableReference::bare("__pse_cdf_input")),
            name,
        ))
    };
    let images = |kinds: [&str; 2]| {
        LogicalPlanBuilder::from(changes.clone())
            .alias("__pse_cdf_input")?
            .filter(
                get_field(input("change"), "kind")
                    .in_list(kinds.into_iter().map(lit).collect(), false),
            )?
            .project(
                columns
                    .iter()
                    .map(|name| get_field(input("value"), name.clone()).alias(name)),
            )?
            .build()
    };
    dependencies::difference(
        images(["insert", "update_postimage"]).map_err(crate::session::engine)?,
        images(["delete", "update_preimage"]).map_err(crate::session::engine)?,
    )
    .map_err(crate::session::engine)
}
fn violation(plan: LogicalPlan, name: &str) -> Result<LogicalPlan, CatalogError> {
    LogicalPlanBuilder::from(plan)
        .project([datafusion::logical_expr::lit(name).alias("dependency")])
        .and_then(LogicalPlanBuilder::build)
        .map_err(crate::session::engine)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opaque_projection_stops_before_expanding_shared_producers() {
        use crate::session::contract::ExecutionContract;
        use datafusion::logical_expr::Union;
        use pse_schema::model::provider::OperationEffect;

        let mut plan = LogicalPlanBuilder::empty(true).build().unwrap();
        assert!(!projection_is_opaque(&plan).unwrap());
        // A tree walk would expand over a trillion occurrences, despite only
        // forty native shared producer owners. The first extension suffices.
        for _ in 0..40 {
            let input = Arc::new(ExecutionContract::plan(
                plan,
                None,
                [OperationEffect::Read].into_iter().collect(),
            ));
            plan = LogicalPlan::Union(Union {
                inputs: vec![Arc::clone(&input), Arc::clone(&input)],
                schema: Arc::clone(input.schema()),
            });
        }
        assert!(projection_is_opaque(&plan).unwrap());
    }

    #[tokio::test]
    async fn projected_changes_preserve_nulls_and_bag_multiplicity() {
        let context = datafusion::prelude::SessionContext::new();
        let changes = context.sql("SELECT named_struct('kind', kind) AS change, named_struct('id', id, 'value', value) AS value FROM (VALUES ('update_preimage',1,CAST(NULL AS BIGINT)),('update_postimage',1,7)) AS edits(kind,id,value)").await.unwrap().into_unoptimized_plan();
        for (columns, empty) in [
            (vec!["id".into()], true),
            (vec!["id".into(), "value".into()], false),
        ] {
            let plan = changed_values(&changes, &columns).unwrap();
            let result = context
                .execute_logical_plan(plan)
                .await
                .unwrap()
                .collect()
                .await
                .unwrap();
            assert_eq!(result.iter().all(|batch| batch.num_rows() == 0), empty);
        }
        let changes = context.sql("SELECT named_struct('kind', kind) AS change, named_struct('id', id) AS value FROM (VALUES ('insert',CAST(NULL AS BIGINT)),('insert',NULL),('delete',NULL)) AS edits(kind,id)").await.unwrap().into_unoptimized_plan();
        let result = context
            .execute_logical_plan(changed_values(&changes, &["id".into()]).unwrap())
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();
        assert!(result.iter().any(|batch| batch.num_rows() > 0));
    }
}
