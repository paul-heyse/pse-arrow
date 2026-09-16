// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete P6 winner, scope, state and provision correspondence is a native join.

use datafusion::{
    functions_aggregate::expr_fn::count,
    logical_expr::{JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::{
    columnar::RelationRow,
    generated::{inferred, reference},
};
use pse_schema::Registry;

use crate::{
    CompilerError,
    passes::native_rows::{AlgorithmInputs, column, engine, join, project, scan},
};

pub(super) struct Candidate {
    pub requirement: inferred::property_requirements::Row,
    pub resolution: inferred::method_resolutions::Row,
    pub resolution_key: pse_ids::ContentHash,
    pub scope: inferred::state_scopes::Row,
    pub state: inferred::instances::Row,
    pub method: reference::method_specs::Row,
    pub provision: reference::method_provisions::Row,
}

pub(super) async fn candidates(
    owner: &mut AlgorithmInputs,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<Vec<Candidate>, CompilerError> {
    let requirements = scan(
        session,
        inferred::property_requirements::spec(registry)?,
        "q",
    )?;
    let resolutions = scan(session, inferred::method_resolutions::spec(registry)?, "r")?;
    crate::passes::native_rows::reject(
        join(
            resolutions.clone(),
            requirements.clone(),
            JoinType::LeftAnti,
            &[("r.requirement_id", "q.requirement_id")],
        )?,
        session,
        cancel,
        "method resolution has no actual property requirement",
    )
    .await?;
    let joined = join(
        requirements.clone(),
        resolutions,
        JoinType::Inner,
        &[("q.requirement_id", "r.requirement_id")],
    )?;
    let joined = join(
        joined,
        scan(session, inferred::state_scopes::spec(registry)?, "c")?,
        JoinType::Inner,
        &[("q.state_scope_id", "c.state_scope_id")],
    )?;
    let joined = join(
        joined,
        scan(session, inferred::instances::spec(registry)?, "s")?,
        JoinType::Inner,
        &[("c.state_instance_id", "s.instance_id")],
    )?;
    let joined = join(
        joined,
        scan(session, reference::method_specs::spec(registry)?, "m")?,
        JoinType::Inner,
        &[("r.method_id", "m.method_id")],
    )?;
    let joined = join(
        joined,
        scan(session, reference::method_provisions::spec(registry)?, "p")?,
        JoinType::Inner,
        &[
            ("r.method_id", "p.method_id"),
            ("q.property_kind_id", "p.property_kind_id"),
        ],
    )?;
    let fields = [
        ("q", inferred::property_requirements::spec(registry)?),
        ("r", inferred::method_resolutions::spec(registry)?),
        ("c", inferred::state_scopes::spec(registry)?),
        ("s", inferred::instances::spec(registry)?),
        ("m", reference::method_specs::spec(registry)?),
        ("p", reference::method_provisions::spec(registry)?),
    ];
    let joined = LogicalPlanBuilder::from(joined)
        .project(fields.into_iter().flat_map(|(alias, spec)| {
            spec.columns.iter().map(move |field| {
                column(alias, field.name()).alias(format!("{alias}_{}", field.name()))
            })
        }))
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let retained = owner
        .retain_plan(joined, session, "p9_selected_method_candidates", cancel)
        .await?;
    let joined = retained.scan_computation_role("p9_selected_method_candidates")?;
    let counts = LogicalPlanBuilder::from(joined.clone())
        .aggregate(
            [col("q_requirement_id").alias("requirement_id")],
            [count(lit(1_i64)).alias("matches")],
        )
        .map_err(engine)?
        .alias("n")
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let counts = join(
        requirements,
        counts,
        JoinType::Left,
        &[("q.requirement_id", "n.requirement_id")],
    )?;
    let violations = LogicalPlanBuilder::from(counts)
        .filter(
            column("n", "matches")
                .is_null()
                .or(column("n", "matches").not_eq(lit(1_i64))),
        )
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    crate::passes::native_rows::reject(violations, &retained, cancel,
        "each property requirement needs exactly one actual resolution, state, method and provision").await?;

    // Every projected row sequence shares this complete unique requirement ordering.
    let joined = LogicalPlanBuilder::from(joined)
        .sort([col("q_requirement_id").sort(true, false)])
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let requirements = selected::<inferred::property_requirements::Row>(
        owner, &joined, "q", &retained, registry, cancel,
    )
    .await?;
    let resolutions = selected::<inferred::method_resolutions::Row>(
        owner, &joined, "r", &retained, registry, cancel,
    )
    .await?;
    let scopes =
        selected::<inferred::state_scopes::Row>(owner, &joined, "c", &retained, registry, cancel)
            .await?;
    let states =
        selected::<inferred::instances::Row>(owner, &joined, "s", &retained, registry, cancel)
            .await?;
    let methods =
        selected::<reference::method_specs::Row>(owner, &joined, "m", &retained, registry, cancel)
            .await?;
    let provisions = selected::<reference::method_provisions::Row>(
        owner, &joined, "p", &retained, registry, cancel,
    )
    .await?;
    let resolution_keys = owner
        .keys(
            LogicalPlanBuilder::from(joined.clone())
                .project([pse_catalog::session::scalar::key(
                    inferred::method_resolutions::RELATION_ID,
                    inferred::method_resolutions::spec(registry)?
                        .primary_key
                        .iter()
                        .map(|name| (*name, col(format!("r_{name}"))))
                        .collect(),
                )
                .alias("source_key")])
                .map_err(engine)?
                .build()
                .map_err(engine)?,
            &retained,
            cancel,
        )
        .await?
        .into_iter()
        .map(|key| key.ok_or_else(|| super::invalid("selected resolution key is null")))
        .collect::<Result<Vec<_>, _>>()?;
    let expected = requirements.len();
    if [
        resolutions.len(),
        resolution_keys.len(),
        scopes.len(),
        states.len(),
        methods.len(),
        provisions.len(),
    ]
    .into_iter()
    .any(|count| count != expected)
    {
        return Err(super::invalid(
            "native method projections have inconsistent cardinalities",
        ));
    }
    Ok(requirements
        .into_iter()
        .zip(resolutions)
        .zip(scopes)
        .zip(states)
        .zip(methods)
        .zip(provisions)
        .zip(resolution_keys)
        .map(
            |(
                (((((requirement, resolution), scope), state), method), provision),
                resolution_key,
            )| Candidate {
                requirement,
                resolution,
                resolution_key,
                scope,
                state,
                method,
                provision,
            },
        )
        .collect())
}

async fn selected<T: RelationRow>(
    owner: &mut AlgorithmInputs,
    joined: &LogicalPlan,
    alias: &str,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<Vec<T>, CompilerError> {
    owner
        .rows(
            LogicalPlanBuilder::from(joined.clone())
                .project(
                    T::relation(registry)?
                        .columns
                        .iter()
                        .map(|field| col(format!("{alias}_{}", field.name())).alias(field.name())),
                )
                .map_err(engine)?
                .build()
                .map_err(engine)?,
            session,
            registry,
            cancel,
        )
        .await
}

/// Select parameter mapping declarations for exactly the actually selected methods.
pub(super) fn mappings(
    session: &SnapshotSession,
    registry: &Registry,
) -> Result<LogicalPlan, CompilerError> {
    let mappings = scan(
        session,
        reference::method_state_parameters::spec(registry)?,
        "a",
    )?;
    let resolutions = LogicalPlanBuilder::from(scan(
        session,
        inferred::method_resolutions::spec(registry)?,
        "r",
    )?)
    .project([column("r", "method_id")])
    .map_err(engine)?
    .filter(col("method_id").is_not_null())
    .map_err(engine)?
    .distinct()
    .map_err(engine)?
    .alias("r")
    .map_err(engine)?
    .build()
    .map_err(engine)?;
    let selected = join(
        mappings,
        resolutions,
        JoinType::LeftSemi,
        &[("a.method_id", "r.method_id")],
    )?;
    project(
        selected,
        reference::method_state_parameters::spec(registry)?,
        "a",
    )
}
