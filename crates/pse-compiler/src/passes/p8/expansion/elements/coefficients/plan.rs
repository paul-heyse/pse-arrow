// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Standard logical joins retain source identity fields and nullable absence explicitly.
use crate::CompilerError;
use datafusion::functions_aggregate::expr_fn::array_agg;
use datafusion::{
    common::{Column, DataFusionError, NullEquality},
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder},
};
use pse_catalog::session::{SnapshotSession, scalar};
use pse_schema::Registry;

pub(super) fn members(
    session: &SnapshotSession,
    registry: &Registry,
) -> Result<LogicalPlan, CompilerError> {
    let groups = scan(session, registry, "compiled.element_projection_groups", "g")?;
    let element = scan(session, registry, "normalized.domain_members", "e")?;
    let species = scan(session, registry, "normalized.domain_members", "j")?;
    let element_eligible = scan(session, registry, "inferred.domain_eligible_members", "ee")?;
    let species_eligible = scan(session, registry, "inferred.domain_eligible_members", "je")?;
    let compositions = scan(session, registry, "normalized.species_elements", "c")?;
    let definitions = scan(session, registry, "normalized.species", "s")?;
    let known = LogicalPlanBuilder::from(scan(
        session,
        registry,
        "normalized.species_elements",
        "known",
    )?)
    .aggregate(
        [column("known", "species_id")],
        [array_agg(source_key(
            registry,
            "normalized.species_elements",
            "known",
        )?)
        .alias("composition_keys")],
    )
    .map_err(engine)?
    .alias("known")
    .map_err(engine)?
    .build()
    .map_err(engine)?;
    let joined = join(
        groups,
        element,
        JoinType::Inner,
        &[("g.element_domain_id", "e.domain_id")],
    )?;
    let joined = join(
        joined,
        element_eligible,
        JoinType::Inner,
        &[
            ("e.domain_id", "ee.domain_id"),
            ("e.member_id", "ee.member_id"),
        ],
    )?;
    let joined = join(
        joined,
        species,
        JoinType::Inner,
        &[("g.species_domain_id", "j.domain_id")],
    )?;
    let joined = join(
        joined,
        species_eligible,
        JoinType::Inner,
        &[
            ("j.domain_id", "je.domain_id"),
            ("j.member_id", "je.member_id"),
        ],
    )?;
    let joined = join(
        joined,
        definitions,
        JoinType::Inner,
        &[("j.ref_entity_id", "s.species_id")],
    )?;
    let joined = join(
        joined,
        known,
        JoinType::Inner,
        &[("s.species_id", "known.species_id")],
    )?;
    let joined = join(
        joined,
        compositions,
        JoinType::Left,
        &[
            ("e.ref_entity_id", "c.element_id"),
            ("s.species_id", "c.species_id"),
        ],
    )?;
    LogicalPlanBuilder::from(joined)
        .project(vec![
            column("g", "group_id").alias("group_id"),
            column("e", "member_id").alias("element_member"),
            column("j", "member_id").alias("species_member"),
            column("e", "ref_entity_id").alias("element_id"),
            column("s", "species_id").alias("species_id"),
            column("c", "count").alias("count"),
            column("s", "mw").alias("molecular_weight"),
            source_key(registry, "normalized.domain_members", "e")?.alias("element_source"),
            source_key(registry, "normalized.domain_members", "j")?.alias("species_member_source"),
            source_key(registry, "inferred.domain_eligible_members", "ee")?
                .alias("element_eligible_source"),
            source_key(registry, "inferred.domain_eligible_members", "je")?
                .alias("species_eligible_source"),
            source_key(registry, "normalized.species", "s")?.alias("species_source"),
            column("known", "composition_keys"),
        ])
        .map_err(engine)?
        .build()
        .map_err(engine)
}
fn scan(
    session: &SnapshotSession,
    registry: &Registry,
    name: &str,
    alias: &str,
) -> Result<LogicalPlan, CompilerError> {
    let key = registry
        .relation(name)
        .ok_or_else(|| super::invalid("element projection relation absent"))?
        .key;
    LogicalPlanBuilder::scan(
        session.table_reference(&key)?,
        session.table_source(&key)?,
        None,
    )
    .map_err(engine)?
    .alias(alias)
    .map_err(engine)?
    .build()
    .map_err(engine)
}
fn join(
    left: LogicalPlan,
    right: LogicalPlan,
    kind: JoinType,
    keys: &[(&str, &str)],
) -> Result<LogicalPlan, CompilerError> {
    let left_keys = keys
        .iter()
        .map(|(left, _)| Column::from_qualified_name(*left))
        .collect::<Vec<_>>();
    let right_keys = keys
        .iter()
        .map(|(_, right)| Column::from_qualified_name(*right))
        .collect::<Vec<_>>();
    LogicalPlanBuilder::from(left)
        .join_detailed(
            right,
            kind,
            (left_keys, right_keys),
            None,
            NullEquality::NullEqualsNothing,
        )
        .map_err(engine)?
        .build()
        .map_err(engine)
}
fn column(alias: &str, name: &str) -> Expr {
    Expr::Column(Column::new(Some(alias), name))
}
fn engine(error: DataFusionError) -> CompilerError {
    pse_rules::errmap::classify(error, pse_catalog::PlanOrigin::RuleCompiler).into()
}

fn source_key(registry: &Registry, name: &str, alias: &str) -> Result<Expr, CompilerError> {
    let spec = registry
        .relation(name)
        .ok_or_else(|| super::invalid("element coefficient source declaration absent"))?;
    Ok(scalar::key(
        spec.id,
        spec.primary_key
            .iter()
            .map(|name| (*name, column(alias, name)))
            .collect(),
    ))
}
