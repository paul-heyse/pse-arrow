// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed coordinate evaluation with native source selection and finite products.
mod coordinates;
mod outer_guard;
use super::{
    AlgorithmContext, Arc, BTreeMap, BTreeSet, CompilerError, Inputs, JoinType, LogicalPlanBuilder,
    Outputs, Plans, ScalarValue, array_length, c, col, error, filter, invalid, join, lit, project,
    scalar,
};
use crate::passes::{
    native_outputs::{self, OutputRows, SourceRole},
    native_rows::{self, Keyed},
    p4::predicates::{
        IndexEvaluator,
        inventory::{Inventory, Origin, Support, unique},
    },
};
use pse_ids::{SemanticId, named_id};
use pse_mathir::{NodeId, Payload, ValueRef};
use pse_relations::{
    columnar::RelationRow,
    generated::{inferred as i, normalized as n},
};
use pse_templates::paths::Coordinate;

struct Extra {
    axes: Vec<Keyed<i::predicate_axes::Row>>,
    paths: Vec<Keyed<i::path_targets::Row>>,
    opaque: Vec<Keyed<n::template_property_requirements::Row>>,
}
#[derive(Clone)]
struct Read {
    seed: SemanticId,
    origin: Origin,
    source: SemanticId,
    node: Option<i64>,
    path: Option<i64>,
    index: Option<Vec<SemanticId>>,
    symbol: Option<SemanticId>,
    guard: Option<(SemanticId, i64)>,
}
struct Context<'a> {
    read: &'a Read,
    instance: &'a Keyed<n::instance_bindings::Row>,
}
struct Axis {
    domain: SemanticId,
    member: SemanticId,
    kind: pse_relations::generated::enums::DomainKind,
    subject: Option<SemanticId>,
    support: Support,
}
struct Target {
    owner: SemanticId,
    symbol: Option<SemanticId>,
    axes: Vec<Axis>,
    support: Support,
}

#[expect(
    clippy::too_many_lines,
    reason = "build keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) async fn build(
    plans: &mut Plans<'_>,
    inputs: &Inputs,
    ctx: &AlgorithmContext<'_>,
    outputs: &mut Outputs,
) -> Result<(), CompilerError> {
    let mut evaluator = IndexEvaluator::new(
        &plans.session,
        inputs,
        ctx.registry,
        ctx.reserver,
        ctx.cancel,
        ctx.physical()?,
    )
    .await?;
    let seeds = evaluator
        .inventory
        .rows::<n::property_demand_seeds::Row>(ctx.cancel)
        .await?;
    let paths = evaluator
        .inventory
        .rows::<n::property_path_demands::Row>(ctx.cancel)
        .await?;
    let extra = Extra {
        axes: evaluator.inventory.rows(ctx.cancel).await?,
        paths: evaluator.inventory.rows(ctx.cancel).await?,
        opaque: evaluator.inventory.rows(ctx.cancel).await?,
    };
    let mut reads = Vec::new();
    for seed in seeds {
        let guard = match (seed.row.guard_source_id, seed.row.guard_node_id) {
            (None, None) => None,
            (Some(source), Some(node)) => Some((source, node)),
            _ => {
                return Err(invalid(
                    "read guard source and predicate must be present together",
                ));
            }
        };
        reads.push(Read {
            seed: seed.row.seed_id,
            origin: evaluator.inventory.origin(&seed)?,
            source: seed.row.source_id,
            node: seed.row.read_node_id,
            path: None,
            index: seed.row.index,
            symbol: seed.row.source_symbol_decl_id,
            guard,
        });
    }
    for path in paths {
        reads.push(Read {
            seed: path.row.demand_id,
            origin: evaluator.inventory.origin(&path)?,
            source: path.row.source_id,
            node: Some(path.row.read_node_id),
            path: Some(path.row.path_id),
            index: None,
            symbol: None,
            guard: path
                .row
                .guard_predicate_id
                .map(|node| (path.row.source_id, node)),
        });
    }
    let mut output = OutputRows::new(ctx.registry, ctx.reserver, ctx.cancel)?;
    output.ensure::<i::demand_read_keys::Row>()?;
    output.ensure::<i::demand_index_maps::Row>()?;
    output.ensure::<i::read_coordinate_failures::Row>()?;
    for relation in [
        n::domain_members::RELATION_KEY,
        n::instance_domain_bindings::RELATION_KEY,
        i::path_targets::RELATION_KEY,
    ] {
        let (port, _) = plans
            .sources
            .get(&relation)
            .ok_or_else(|| invalid("read coordinate scope lacks an actual source"))?;
        for target in [
            i::demand_read_keys::RELATION_KEY,
            i::read_coordinate_failures::RELATION_KEY,
        ] {
            output.read_scope(
                target,
                SourceRole {
                    relation,
                    port: port.clone(),
                },
            )?;
        }
    }
    for read in reads {
        ctx.cancel.checkpoint()?;
        let source = evaluator
            .inventory
            .sources
            .iter()
            .find(|row| row.row.source_id == read.source)
            .cloned();
        let instances = if let Some(source) = &source {
            evaluator
                .inventory
                .source_instances(&source.row, ctx.cancel)
                .await?
        } else {
            let declaration = unique(
                &extra.opaque,
                |row| row.requirement_id == read.source,
                "opaque requirement",
            )?;
            let template = plans.sid(declaration.row.template_id)?;
            let plan =
                LogicalPlanBuilder::from(evaluator.inventory.scan::<n::instance_bindings::Row>()?)
                    .filter(col("template_id").eq(template))
                    .and_then(LogicalPlanBuilder::build)
                    .map_err(error)?;
            native_rows::keyed_rows(
                &mut evaluator.inventory.arguments,
                plan,
                &evaluator.inventory.session,
                ctx.registry,
                ctx.cancel,
            )
            .await?
        };
        let (outer, outer_support) = outer_guard::lookup(
            &mut evaluator.inventory,
            source.as_ref().map(|source| &source.row),
            ctx.cancel,
        )
        .await?;
        for instance in instances {
            let context = Context {
                read: &read,
                instance: &instance,
            };
            let assignments =
                coordinates::assignments(&mut evaluator, &extra, &context, ctx.cancel).await?;
            for (bindings, mut support) in assignments {
                support.insert(read.origin);
                support.insert(evaluator.inventory.origin(&instance)?);
                support.extend(outer_support.iter().copied());
                if let Some(source) = &source {
                    support.insert(evaluator.inventory.origin(source)?);
                }
                let guard_index = coordinates::guard_index(&extra, &read, &bindings)?;
                let Some(targets) = coordinates::targets(&evaluator, &extra, &context, &bindings)?
                else {
                    let bound = bindings.keys().copied().collect::<Vec<_>>();
                    let index = bindings.values().copied().collect::<Vec<_>>();
                    let id = named_id(
                        read.seed,
                        &format!(
                            "pse:read-coordinate-failure:v1:{}:{}:{}",
                            instance.row.instance_id.to_hex(),
                            identities(&bound)?,
                            identities(&index)?
                        ),
                    );
                    output.push(
                        i::read_coordinate_failures::Row {
                            read_id: id,
                            seed_id: read.seed,
                            requester_instance_id: instance.row.instance_id,
                            source_id: read.source,
                            read_node_id: read.node,
                            bound_index_ids: bound,
                            source_index: index,
                            guard_source_id: read.guard.map(|(source, _)| source),
                            guard_node_id: read.guard.map(|(_, node)| node),
                            guard_index,
                            outer_guard_source_id: outer.map(|(source, _)| source),
                            outer_guard_node_id: outer.map(|(_, node)| node),
                            derivation_id: SemanticId::NIL,
                        },
                        &plans.sources.locate(support)?,
                    )?;
                    continue;
                };
                for target in targets {
                    let mut support = support.clone();
                    support.extend(target.support.iter().copied());
                    support.extend(
                        target
                            .axes
                            .iter()
                            .flat_map(|axis| axis.support.iter().copied()),
                    );
                    let index = target
                        .axes
                        .iter()
                        .map(|axis| axis.member)
                        .collect::<Vec<_>>();
                    let id = named_id(
                        read.seed,
                        &format!(
                            "pse:read-tuple:v1:{}:{}:{}:{}",
                            instance.row.instance_id.to_hex(),
                            target.owner.to_hex(),
                            identities(&index)?,
                            identities(&guard_index)?
                        ),
                    );
                    output.push(
                        i::demand_read_keys::Row {
                            read_id: id,
                            seed_id: read.seed,
                            requester_instance_id: instance.row.instance_id,
                            owner_instance_id: target.owner,
                            symbol_decl_id: target.symbol,
                            index,
                            guard_source_id: read.guard.map(|(source, _)| source),
                            guard_node_id: read.guard.map(|(_, node)| node),
                            guard_index: guard_index.clone(),
                            outer_guard_source_id: outer.map(|(source, _)| source),
                            outer_guard_node_id: outer.map(|(_, node)| node),
                            derivation_id: SemanticId::NIL,
                        },
                        &plans.sources.locate(support.iter().copied())?,
                    )?;
                    map_targets(
                        plans,
                        &mut evaluator.inventory,
                        &target,
                        id,
                        support,
                        &mut output,
                        ctx,
                    )
                    .await?;
                }
            }
        }
    }
    for (key, input) in native_outputs::materialize(
        output.finish()?,
        &plans.sources,
        plans.pass,
        &plans.session,
        ctx.cancel,
    )
    .await?
    {
        plans.session = native_rows::workspace(
            &plans.session,
            &BTreeMap::from([(key, input.checked().clone())]),
            ctx.cancel,
        )?;
        plans.sources.replace_native(key, Arc::clone(&input))?;
        outputs.insert(key, input);
    }
    Ok(())
}

// JSON is the existing scalar identity encoding. No generic relation rows are built.
fn identities(ids: &[SemanticId]) -> Result<String, CompilerError> {
    serde_json::to_string(&ids.iter().map(|id| id.to_hex()).collect::<Vec<_>>())
        .map_err(|error| invalid(error.to_string()))
}

async fn map_targets(
    plans: &mut Plans<'_>,
    inventory: &mut Inventory<'_>,
    target: &Target,
    read: SemanticId,
    support: Support,
    output: &mut OutputRows<'_>,
    ctx: &AlgorithmContext<'_>,
) -> Result<(), CompilerError> {
    let requirement = plans.scan("inferred.requirement_keys", "candidate")?;
    let mut matched = filter(
        requirement,
        array_length(c("candidate", "index")).eq(lit(i64::try_from(target.axes.len())
            .map_err(|_| invalid("requirement axis extent overflow"))?)),
    )?;
    for (axis, position) in target.axes.iter().zip(0_i64..) {
        let alias = format!("candidate_axis_{position}");
        let axes = plans.scan("inferred.requirement_key_axes", &alias)?;
        let subject = if let Some(subject) = axis.subject {
            c(&alias, "subject_id").eq(plans.sid(subject)?)
        } else {
            c(&alias, "subject_id")
                .is_null()
                .and(c(&alias, "domain_id").eq(plans.sid(axis.domain)?))
                .and(c(&alias, "member_id").eq(plans.sid(axis.member)?))
        };
        matched = join(
            matched,
            axes,
            JoinType::Inner,
            [
                c("candidate", "requirement_id").eq(c(&alias, "requirement_id")),
                c(&alias, "position").eq(lit(position)),
                c(&alias, "kind").eq(lit(axis.kind.as_str())),
                subject,
            ],
        )?;
    }
    let spec = i::requirement_keys::Row::relation(ctx.registry)?;
    let matched = project(
        matched,
        spec.columns
            .iter()
            .map(|field| c("candidate", field.name()).alias(field.name())),
    )?;
    let rows = native_rows::keyed_rows::<i::requirement_keys::Row>(
        &mut inventory.arguments,
        matched,
        &plans.session,
        ctx.registry,
        ctx.cancel,
    )
    .await?;
    for requirement in rows {
        let mut sources = support.clone();
        sources.insert(inventory.origin(&requirement)?);
        output.push(
            i::demand_index_maps::Row {
                read_id: read,
                requirement_id: requirement.row.requirement_id,
                derivation_id: SemanticId::NIL,
            },
            &plans.sources.locate(sources)?,
        )?;
    }
    Ok(())
}
