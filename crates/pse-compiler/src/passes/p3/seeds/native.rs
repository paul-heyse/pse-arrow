// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native property binding and ordered finite opaque-demand expansion.
use super::super::plans::{
    Plans, append, c, error, explode, filter, invalid, join, prefix, project, union,
};
use crate::{CompilerError, passes::native_outputs::Sources};
use datafusion::functions_nested::expr_fn::array_element;
use datafusion::{
    catalog::cte_worktable::CteWorkTable,
    common::ScalarValue,
    datasource::provider_as_source,
    functions::core::expr_fn::get_field,
    functions_aggregate::{count::count_distinct, expr_fn::count},
    functions_nested::expr_fn::array_length,
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::{
    SnapshotSession,
    output::{checked_literal, same_field_case},
    scalar,
};
use pse_ids::CancellationToken;
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{authored, normalized},
};
use pse_rules::strata::native_input::NativeInput;
use pse_schema::model::{AlgorithmSpec, RelationKey};
use std::{collections::BTreeMap, sync::Arc};

pub(in crate::passes::p3) async fn emit(
    inputs: &BTreeMap<RelationKey, FieldCheckedBatch>,
    sources: &Sources,
    pass: &AlgorithmSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    let mut plans = Plans::new(inputs, sources, pass, session, cancel)?;
    let expressions = expressions(&mut plans).await?;
    let opaque = opaque(&mut plans).await?;
    let seeds = union(vec![expressions, opaque])?;
    let seeds = validate_references(&mut plans, seeds).await?;
    let output = plans
        .output("normalized.property_demand_seeds", seeds, false)
        .await?;
    Ok(BTreeMap::from([(
        normalized::property_demand_seeds::RELATION_KEY,
        output,
    )]))
}

fn value(
    plans: &Plans<'_>,
    field: &str,
    value: Option<ScalarValue>,
) -> Result<Expr, CompilerError> {
    let spec = normalized::property_demand_seeds::spec(plans.session.registry())?;
    let column = spec
        .column(field)
        .ok_or_else(|| invalid("seed field undeclared"))?;
    let arrow = pse_schema::arrow::field_for(plans.session.registry(), column)?;
    let scalar = match value {
        Some(value) => value.cast_to(arrow.data_type()).map_err(error)?,
        None => ScalarValue::try_from(arrow.data_type()).map_err(error)?,
    };
    checked_literal(plans.session.registry(), column, scalar).map_err(error)
}
fn kind(plans: &Plans<'_>, name: &str) -> Result<Expr, CompilerError> {
    value(
        plans,
        "source_kind",
        Some(ScalarValue::Utf8(Some(name.to_owned()))),
    )
}
fn empty_index(plans: &Plans<'_>) -> Result<Expr, CompilerError> {
    Ok(plans
        .session
        .scalar_function("pse_index_tuple")?
        .call(vec![scalar::id_list(vec![])]))
}

async fn expressions(plans: &mut Plans<'_>) -> Result<LogicalPlan, CompilerError> {
    let reads = plans.scan("provenance.property_read_occurrences", "read")?;
    let mappings = plans.scan("authored.template_symbol_properties", "mapping")?;
    let mapped = join(
        reads,
        mappings,
        JoinType::Inner,
        [c("read", "symbol_decl_id").eq(c("mapping", "symbol_decl_id"))],
    )?;
    let symbols = plans.scan("authored.template_symbols", "symbol")?;
    let mapped = join(
        mapped,
        symbols,
        JoinType::Left,
        [c("read", "symbol_decl_id").eq(c("symbol", "symbol_decl_id"))],
    )?;
    plans
        .require(
            &mapped,
            c("symbol", "symbol_decl_id").is_not_null(),
            "a property mapping has no actual symbol declaration",
        )
        .await?;
    let index = same_field_case(
        mapped.schema(),
        array_length(c("symbol", "indexed_by")).eq(lit(0_i64)),
        empty_index(plans)?,
        value(plans, "index", None)?,
    )
    .map_err(error)?;
    let guard_source = same_field_case(
        mapped.schema(),
        c("read", "guard_predicate_id").is_not_null(),
        c("read", "source_id"),
        value(plans, "guard_source_id", None)?,
    )
    .map_err(error)?;
    project(
        mapped,
        [
            c("read", "read_id").alias("seed_id"),
            c("read", "source_id").alias("source_id"),
            kind(plans, "expression")?.alias("source_kind"),
            c("mapping", "scope_selector_id").alias("scope_id"),
            c("mapping", "property_kind_id").alias("property_kind_id"),
            index.alias("index"),
            c("read", "guard_predicate_id").alias("guard_node_id"),
            guard_source.alias("guard_source_id"),
            c("read", "symbol_decl_id").alias("source_symbol_decl_id"),
            c("read", "read_node_id").alias("read_node_id"),
            plans
                .lists(vec![
                    c("read", "support"),
                    c("mapping", "support"),
                    c("symbol", "support"),
                ])?
                .alias("supports"),
        ],
    )
}

async fn validate_references(
    plans: &mut Plans<'_>,
    seeds: LogicalPlan,
) -> Result<LogicalPlan, CompilerError> {
    let scopes = plans.scan("authored.scopes", "scope")?;
    let seeds = join(
        seeds,
        scopes,
        JoinType::Left,
        [col("scope_id").eq(c("scope", "scope_id"))],
    )?;
    let properties = plans.scan("reference.property_kinds", "property")?;
    let seeds = join(
        seeds,
        properties,
        JoinType::Left,
        [col("property_kind_id").eq(c("property", "property_kind_id"))],
    )?;
    plans
        .require(
            &seeds,
            c("scope", "scope_id")
                .is_not_null()
                .and(c("property", "property_kind_id").is_not_null()),
            "property demand requires an actual scope and property kind",
        )
        .await?;
    let mut fields = normalized::property_demand_seeds::spec(plans.session.registry())?
        .columns
        .iter()
        .filter(|column| column.name() != "derivation_id")
        .map(|column| col(column.name()))
        .collect::<Vec<_>>();
    fields.push(
        plans
            .lists(vec![
                col("supports"),
                c("scope", "support"),
                c("property", "support"),
            ])?
            .alias("supports"),
    );
    project(seeds, fields)
}

#[expect(
    clippy::too_many_lines,
    reason = "opaque keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
async fn opaque(plans: &mut Plans<'_>) -> Result<LogicalPlan, CompilerError> {
    let requirements = plans.scan("authored.template_property_requirements", "requirement")?;
    let expressions = plans.scan("normalized.expression_sources", "guard")?;
    let guarded = join(
        requirements,
        expressions,
        JoinType::Left,
        [
            c("guard", "source_relation_id")
                .eq(plans.sid(authored::template_property_requirements::RELATION_ID)?),
            c("guard", "source_key").eq(c("requirement", "source_token")),
        ],
    )?;
    plans
        .require(
            &guarded,
            c("requirement", "guard")
                .is_null()
                .eq(c("guard", "source_id").is_null())
                .and(
                    c("guard", "source_id")
                        .is_null()
                        .or(c("guard", "syntax").eq(lit("predicate"))),
                ),
            "opaque property guard must have its exact predicate source",
        )
        .await?;
    let guarded = project(
        guarded,
        [
            c("requirement", "requirement_id").alias("requirement_id"),
            c("requirement", "template_id").alias("template_id"),
            c("requirement", "scope_selector_id").alias("scope_id"),
            c("requirement", "property_kind_id").alias("property_kind_id"),
            c("requirement", "index_domain_bindings").alias("axes"),
            c("guard", "source_id").alias("guard_source_id"),
            c("guard", "root_id").alias("guard_node_id"),
            plans
                .lists(vec![c("requirement", "support"), c("guard", "support")])?
                .alias("supports"),
        ],
    )?;
    let axes = checked_axes(plans, guarded.clone()).await?;
    let seed = project(
        guarded,
        [
            col("requirement_id"),
            col("scope_id"),
            col("property_kind_id"),
            col("guard_source_id"),
            col("guard_node_id"),
            array_length(col("axes")).alias("axis_count"),
            lit(0_i64).alias("cursor"),
            empty_index(plans)?.alias("index"),
            col("supports"),
        ],
    )?;
    let seed = plans.session.derive_plan_fields(seed, plans.cancel)?;
    let name = "p3_opaque_property_tuples";
    let work = LogicalPlanBuilder::scan(
        name,
        provider_as_source(Arc::new(CteWorkTable::new(
            name,
            Arc::new(seed.schema().as_arrow().clone()),
        ))),
        None,
    )
    .and_then(LogicalPlanBuilder::build)
    .map_err(error)?;
    let work = join(
        work,
        prefix(axes, "axis")?,
        JoinType::Inner,
        [
            col("requirement_id").eq(c("axis", "requirement_id")),
            col("cursor").eq(c("axis", "position")),
        ],
    )?;
    let member_tuple = plans
        .session
        .scalar_function("pse_index_tuple")?
        .call(vec![scalar::id_list(vec![c("axis", "member_id")])]);
    let step = project(
        work,
        [
            col("requirement_id"),
            col("scope_id"),
            col("property_kind_id"),
            col("guard_source_id"),
            col("guard_node_id"),
            col("axis_count"),
            (col("cursor") + lit(1_i64)).alias("cursor"),
            plans.ids(vec![col("index"), member_tuple])?.alias("index"),
            plans
                .lists(vec![col("supports"), c("axis", "supports")])?
                .alias("supports"),
        ],
    )?;
    let closure = LogicalPlanBuilder::from(seed)
        .to_recursive_query(name.to_owned(), step, false)
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let complete = filter(closure, col("cursor").eq(col("axis_count")))?;
    let identity = scalar::key(pse_ids::SemanticId::NIL, vec![("index", col("index"))]);
    project(
        complete,
        [
            scalar::named_id(col("requirement_id"), identity).alias("seed_id"),
            col("requirement_id").alias("source_id"),
            kind(plans, "opaque_operation")?.alias("source_kind"),
            col("scope_id"),
            col("property_kind_id"),
            col("index"),
            col("guard_node_id"),
            col("guard_source_id"),
            value(plans, "source_symbol_decl_id", None)?.alias("source_symbol_decl_id"),
            value(plans, "read_node_id", None)?.alias("read_node_id"),
            col("supports"),
        ],
    )
}

/// Every axis is checked before expansion, including an empty member inventory.
#[expect(
    clippy::too_many_lines,
    reason = "checked_axes keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
async fn checked_axes(
    plans: &mut Plans<'_>,
    requirements: LogicalPlan,
) -> Result<LogicalPlan, CompilerError> {
    let expanded = explode(requirements, col("axes"), "position")?;
    let binding = array_element(col("axes"), col("position") + lit(1_i64));
    let expanded = append(
        expanded,
        [
            get_field(binding.clone(), "index_name").alias("index_name"),
            get_field(binding, "domain_id").alias("domain_id"),
        ],
    )?;
    let duplicates = LogicalPlanBuilder::from(expanded.clone())
        .aggregate(
            [col("requirement_id"), col("index_name")],
            [count(lit(1_i64)).alias("count")],
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    plans
        .require(
            &duplicates,
            col("count").eq(lit(1_i64)),
            "opaque operation repeats an index binding",
        )
        .await?;
    let domains = plans.scan("normalized.domains", "domain")?;
    let expanded = join(
        expanded,
        domains,
        JoinType::Left,
        [col("domain_id").eq(c("domain", "domain_id"))],
    )?;
    let templates = plans.scan("authored.template_domains", "template")?;
    let expanded = join(
        expanded,
        templates,
        JoinType::Left,
        [
            col("template_id").eq(c("template", "template_id")),
            col("index_name").eq(c("template", "name")),
        ],
    )?;
    plans
        .require(
            &expanded,
            c("domain", "domain_id")
                .is_not_null()
                .and(c("template", "name").is_not_null())
                .and(c("domain", "kind").eq(c("template", "kind")))
                .and(c("domain", "continuous").eq(c("template", "continuous")))
                .and(c("domain", "continuous").eq(lit(false))),
            "opaque index requires a finite domain matching its template declaration",
        )
        .await?;
    let members = plans.scan("normalized.domain_members", "member")?;
    let stats = LogicalPlanBuilder::from(members.clone())
        .aggregate(
            [c("member", "domain_id")],
            [
                count(lit(1_i64)).alias("count"),
                count_distinct(c("member", "member_id")).alias("identities"),
                count_distinct(c("member", "ordinal")).alias("ordinals"),
            ],
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let expanded = join(
        expanded,
        prefix(stats, "stats")?,
        JoinType::Left,
        [col("domain_id").eq(c("stats", "member:domain_id"))],
    )?;
    plans
        .require(
            &expanded,
            c("stats", "count")
                .gt(lit(0_i64))
                .and(c("stats", "count").eq(c("stats", "identities")))
                .and(c("stats", "count").eq(c("stats", "ordinals"))),
            "opaque index domain has no finite unique member inventory",
        )
        .await?;
    let expanded = join(
        expanded,
        members,
        JoinType::Inner,
        [col("domain_id").eq(c("member", "domain_id"))],
    )?;
    let axes = project(
        expanded,
        [
            col("requirement_id"),
            col("position"),
            c("member", "member_id").alias("member_id"),
            plans
                .lists(vec![
                    c("domain", "support"),
                    c("template", "support"),
                    c("member", "support"),
                ])?
                .alias("supports"),
        ],
    )?;
    plans.retain(axes).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_property_seed_plan_keeps_typed_index_through_union() {
        let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
        let inputs = [
            "provenance.property_read_occurrences",
            "authored.template_symbol_properties",
            "authored.template_symbols",
            "authored.scopes",
            "reference.property_kinds",
            "authored.template_property_requirements",
            "normalized.expression_sources",
            "normalized.domains",
            "authored.template_domains",
            "normalized.domain_members",
        ]
        .into_iter()
        .map(|name| {
            let relation = registry.relation(name).unwrap();
            (
                relation.key,
                FieldCheckedBatch::concat(&registry, relation, &[]).unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
        let cancel = CancellationToken::new();
        let (session, sources) = crate::passes::native_test::session(
            &registry,
            inputs.clone(),
            &pse_ids::FixedBudget::new(256 << 20),
            &cancel,
        )
        .unwrap();
        let mut plans = Plans::new(
            &inputs,
            &sources,
            registry.algorithm("P3@1").unwrap(),
            &session,
            &cancel,
        )
        .unwrap();
        let expressions = expressions(&mut plans).await.unwrap();
        let opaque = opaque(&mut plans).await.unwrap();
        let seeds = validate_references(&mut plans, union(vec![expressions, opaque]).unwrap())
            .await
            .unwrap();
        let prepared = plans.session.prepare(seeds, &cancel).unwrap();
        assert_eq!(
            prepared
                .optimized_plan()
                .schema()
                .field_with_unqualified_name("index")
                .unwrap()
                .metadata()
                .get("ARROW:extension:name")
                .map(String::as_str),
            Some("pse.index_tuple")
        );
        let complete = prepared.execute(&cancel).await.unwrap();
        assert!(complete.batches().iter().all(|batch| batch.num_rows() == 0));
    }
}
