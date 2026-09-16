// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P0 resolves the actual exact-version package dependency graph before identity reuse.

use crate::AuthoringError;
use pse_ids::SemanticId;
use pse_relations::generated::{authored, normalized};
use std::collections::BTreeMap;

/// Resolve complete generated headers using native duplicate/missing/version joins,
/// followed by the named dependency-depth algorithm over the selected typed graph.
/// # Errors
/// Non-exact versions, missing/duplicate packages or dependencies, cycles or resources.
pub async fn resolve(
    packages: &pse_relations::columnar::FieldCheckedBatch,
    session: &pse_catalog::session::SnapshotSession,
    cancel: &pse_ids::CancellationToken,
) -> Result<pse_relations::columnar::FieldCheckedBatch, AuthoringError> {
    use datafusion::common::Column;
    use datafusion::logical_expr::{JoinType, LogicalPlanBuilder, col, lit};
    let registry = session.registry();
    let mut work = session.reserver().open("authoring:package-graph");
    work.try_grow(pse_ids::validation_extent(packages.batch())?)?;
    let headers = authored::packages::View::from_checked(packages)?.rows()?;
    for header in &headers {
        version(&header.version)?;
        for dependency in &header.dependencies {
            version(
                dependency
                    .version_req
                    .strip_prefix('=')
                    .unwrap_or(&dependency.version_req),
            )?;
        }
    }
    let bound = session.with_checked_role_inputs(
        BTreeMap::from([("package_dependency_headers".to_owned(), packages.clone())]),
        cancel,
    )?;
    let spec = registry
        .relation_by_id(authored::packages::RELATION_ID)
        .ok_or_else(|| contract("package declaration missing"))?;
    crate::change_set::stage::unique(&bound, "package_dependency_headers", spec, cancel).await?;
    let edges = dependency_edges(bound.scan_role("package_dependency_headers")?)?;
    let duplicate = LogicalPlanBuilder::from(edges.clone())
        .aggregate(
            vec![col("owner_id"), col("dependency_id")],
            vec![datafusion::functions_aggregate::expr_fn::count(lit(1_i64)).alias("__count")],
        )
        .and_then(|plan| plan.filter(col("__count").gt(lit(1_i64))))
        .and_then(LogicalPlanBuilder::build)
        .map_err(crate::change_set::plans::engine)?;
    refuse_rows(&bound, duplicate, "duplicate package dependency", cancel).await?;
    let targets = LogicalPlanBuilder::from(bound.scan_role("package_dependency_headers")?)
        .project(vec![col("package_id"), col("version")])
        .and_then(LogicalPlanBuilder::build)
        .map_err(crate::change_set::plans::engine)?;
    let missing = LogicalPlanBuilder::from(edges.clone())
        .join(
            targets.clone(),
            JoinType::LeftAnti,
            (
                vec![Column::from_name("dependency_id")],
                vec![Column::from_name("package_id")],
            ),
            None,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(crate::change_set::plans::engine)?;
    refuse_rows(&bound, missing, "package dependency target absent", cancel).await?;
    let mismatch = LogicalPlanBuilder::from(edges)
        .join(
            targets,
            JoinType::Inner,
            (
                vec![Column::from_name("dependency_id")],
                vec![Column::from_name("package_id")],
            ),
            None,
        )
        .and_then(|plan| plan.filter(col("required_version").not_eq(col("version"))))
        .and_then(LogicalPlanBuilder::build)
        .map_err(crate::change_set::plans::engine)?;
    refuse_rows(
        &bound,
        mismatch,
        "package exact version requirement differs",
        cancel,
    )
    .await?;
    let values = headers
        .iter()
        .map(|header| (header.package_id, header))
        .collect::<BTreeMap<_, _>>();
    let depth = depths(&values)?;
    let mut output = normalized::package_graph::Builder::with_registry(registry, headers.len())?;
    for package in headers {
        let mut dependencies = package
            .dependencies
            .into_iter()
            .map(|dependency| dependency.package_id)
            .collect::<Vec<_>>();
        dependencies.sort_unstable();
        output.push(normalized::package_graph::Row {
            package_id: package.package_id,
            version: package.version,
            content_hash: package.content_hash,
            depth: i64::from(depth[&package.package_id]),
            dependency_package_ids: dependencies,
            derivation_id: pse_ids::named_id(package.package_id, "pass:P0@1:package_graph"),
        })?;
    }
    Ok(output.finish()?)
}
async fn refuse_rows(
    session: &pse_catalog::session::SnapshotSession,
    plan: datafusion::logical_expr::LogicalPlan,
    reason: &str,
    cancel: &pse_ids::CancellationToken,
) -> Result<(), AuthoringError> {
    let plan = datafusion::logical_expr::LogicalPlanBuilder::from(plan)
        .limit(0, Some(1))
        .and_then(datafusion::logical_expr::LogicalPlanBuilder::build)
        .map_err(crate::change_set::plans::engine)?;
    if crate::change_set::plans::execute(session, plan, cancel)
        .await?
        .iter()
        .any(|batch| batch.num_rows() != 0)
    {
        return Err(contract(reason));
    }
    Ok(())
}

fn depths(
    packages: &BTreeMap<SemanticId, &authored::packages::Row>,
) -> Result<BTreeMap<SemanticId, u16>, AuthoringError> {
    let mut depths = BTreeMap::new();
    while depths.len() < packages.len() {
        let before = depths.len();
        for (id, package) in packages {
            if depths.contains_key(id) {
                continue;
            }
            let parents = package
                .dependencies
                .iter()
                .map(|dependency| depths.get(&dependency.package_id).copied())
                .collect::<Option<Vec<u16>>>();
            if let Some(parents) = parents {
                let depth = parents
                    .into_iter()
                    .max()
                    .map(|depth| {
                        depth
                            .checked_add(1)
                            .ok_or_else(|| contract("package dependency depth exceeds UInt16"))
                    })
                    .transpose()?
                    .unwrap_or(0);
                depths.insert(*id, depth);
            }
        }
        if depths.len() == before {
            return Err(contract("cyclic package dependency"));
        }
    }
    Ok(depths)
}

fn version(text: &str) -> Result<semver::Version, AuthoringError> {
    semver::Version::parse(text)
        .map_err(|error| contract(&format!("exact semantic version required: {error}")))
}
fn contract(reason: &str) -> AuthoringError {
    AuthoringError::Contract {
        at: None,
        reason: reason.to_owned(),
    }
}

fn dependency_edges(
    packages: datafusion::logical_expr::LogicalPlan,
) -> Result<datafusion::logical_expr::LogicalPlan, AuthoringError> {
    use datafusion::logical_expr::{LogicalPlanBuilder, col, lit};
    let get = datafusion::functions::core::expr_fn::get_field;
    let edges = LogicalPlanBuilder::from(packages)
        .project(vec![
            col("package_id").alias("owner_id"),
            col("dependencies"),
        ])
        .and_then(|plan| plan.unnest_column("dependencies"))
        .and_then(|plan| {
            plan.project(vec![
                col("owner_id"),
                get(col("dependencies"), "package_id").alias("dependency_id"),
                datafusion::functions::regex::expr_fn::regexp_replace(
                    get(col("dependencies"), "version_req"),
                    lit("^="),
                    lit(""),
                    None,
                )
                .alias("required_version"),
            ])
        })
        .and_then(LogicalPlanBuilder::build)
        .map_err(crate::change_set::plans::engine)?;
    Ok(edges)
}
