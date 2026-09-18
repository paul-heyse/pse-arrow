// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed source lowering with exact, native-selected source correspondences.

mod support;
use super::{invalid, lower, provenance, seeds, units};
use crate::{
    CompilerError,
    mathir_relations::{Family, RelationSink},
    passes::native_outputs::{OutputRows, SourceKey},
};
use pse_authoring::document::{
    Batches,
    binding::{OwnedSourceBindings, SourceExpression},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::{
    columnar::{Collection, FieldCheckedBatch},
    generated::{authored, normalized},
};
use pse_rules::strata::native_input::NativeInput;
use pse_schema::{
    Registry,
    model::{AlgorithmSpec, ExpressionOwnerKind, RelationKey},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use support::Support;

type Origins = BTreeSet<SourceKey>;

pub(super) async fn emit(
    bindings: &OwnedSourceBindings,
    batches: &Batches,
    construction: &provenance::Construction,
    physical: &crate::quantity_relations::PhysicalInventory,
    pass: &AlgorithmSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    let registry = session.registry();
    let mut support = Support::new(construction, session, cancel);
    let mut units = units::Units::new(physical, batches, registry)?;
    let mut work = session.reserver().open("P3:source-lowering");
    let mut offsets = BTreeMap::<&'static str, i64>::new();
    let mut output = OutputRows::new(registry, session.reserver(), cancel)?;
    let mut unit_origins = BTreeMap::<pse_quantity::UnitId, Origins>::new();
    output.ensure::<normalized::expression_sources::Row>()?;
    output.ensure::<normalized::expression_index_bindings::Row>()?;
    output.ensure::<normalized::predicate_nodes::Row>()?;
    output.ensure::<normalized::equation_nodes::Row>()?;
    output.ensure::<normalized::expression_paths::Row>()?;
    output.ensure::<normalized::property_path_demands::Row>()?;
    output.ensure::<pse_relations::generated::provenance::property_read_occurrences::Row>()?;

    for source in bindings.expressions() {
        cancel.checkpoint()?;
        if source.relation_id == authored::template_submodels::RELATION_ID {
            // Configuration consumes these bindings and records their actual source
            // rows in config_values/instance_bindings. No mathematical consumer uses
            // a second graph for parent parameters or Boolean configuration literals.
            continue;
        }
        let source_id = pse_ids::named_id(
            source.document_id,
            &format!("pse:p3:source:v1:{}", source.document_path),
        );
        let mut origins = support.expression(source, bindings).await?;
        let owner = support.owner(source.owner_template_id).await?;
        origins.insert(owner.1);
        let package = owner.0.package_id;
        let prefix = family(source, registry)?;
        let offset = *offsets.get(prefix).unwrap_or(&0);
        let mut lowered = lower::lower(
            source,
            source_id,
            offset,
            batches,
            cancel,
            &mut units,
            package,
            work.as_mut(),
        )?;
        let reads = seeds::syntax(
            &mut lowered,
            source_id,
            offset,
            session,
            cancel,
            work.as_mut(),
        )?;
        let mut unit_uses = std::mem::take(&mut units.source_uses);
        unit_uses.extend(
            lowered
                .literal_units
                .values()
                .flatten()
                .copied()
                .map(pse_quantity::UnitId::from_id),
        );
        for id in &unit_uses {
            origins.extend(support.unit(*id, package, &units).await?);
        }
        for set in std::mem::take(&mut units.source_sets) {
            origins.extend(support.unit_set(package, set).await?);
        }
        for id in &unit_uses {
            if units.derived.contains_key(id) {
                unit_origins.entry(*id).or_default().extend(origins.clone());
            }
        }
        let next = offset
            .checked_add(
                i64::try_from(lowered.graph.len())
                    .map_err(|_| invalid("source graph extent overflow"))?,
            )
            .ok_or_else(|| invalid("source graph ordinal overflow"))?;
        let columns = emit_source(
            source, source_id, prefix, offset, lowered, registry, session, cancel,
        )?;
        // Retain generated Arrow columns and exact per-row sources together.
        // The complete source result enters native witness matching once, after
        // construction, instead of retaining a computation per expression/table.
        for batch in columns.into_values().chain(reads.into_values()) {
            output.append_checked(&batch, |_, _| Ok(origins.clone()))?;
        }
        offsets.insert(prefix, next);
    }

    output.ensure::<normalized::units::Row>()?;
    for (id, row) in &units.derived {
        let mut actual = unit_origins.get(id).cloned().unwrap_or_default();
        actual.extend(support.unit(*id, row.package_id, &units).await?);
        if actual.is_empty() {
            return Err(invalid("derived unit lacks its actual definition sources"));
        }
        output.push(row.clone(), &actual)?;
    }

    let packages = batches
        .get(&authored::packages::RELATION_ID)
        .ok_or_else(|| invalid("package input absent"))?;
    let package_graph = pse_authoring::p0::resolve(packages, session, cancel).await?;
    for (key, supported) in support.packages(&package_graph).await?.columns {
        output.append_supported(key, supported)?;
    }
    provenance::materialize(output.finish()?, construction, pass, session, cancel).await
}

#[expect(
    clippy::too_many_arguments,
    reason = "emit_source keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
fn emit_source(
    source: &SourceExpression,
    source_id: SemanticId,
    prefix: &'static str,
    offset: i64,
    lowered: lower::Lowered,
    registry: &Registry,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, FieldCheckedBatch>, CompilerError> {
    let derivation = pse_ids::named_id(source_id, "pass:P3@1");
    let mut sink = RelationSink::new(
        registry,
        Family::Normalized {
            prefix,
            offset,
            derivation,
            source_span: source.source_span,
        },
        session.reserver(),
        cancel,
    );
    let nodes = lowered
        .graph
        .iter()
        .map(|(node, _)| node)
        .collect::<Vec<_>>();
    pse_mathir::relations::emit_untyped(&lowered.graph, &nodes, &mut sink)?;
    let mut output = sink.into_batches()?;
    let mut columns = Collection::new(registry, session.reserver(), cancel);
    columns.push(source_row(
        source,
        source_id,
        prefix,
        lowered.syntax,
        lowered.root,
        derivation,
    )?)?;
    for row in lowered.predicates {
        columns.push(row)?;
    }
    for row in lowered.equations {
        columns.push(row)?;
    }
    for row in lowered.bindings {
        columns.push(row)?;
    }
    for row in lowered.instance_paths {
        columns.push(row)?;
    }
    output.extend(columns.finish()?);
    Ok(output)
}

fn family(source: &SourceExpression, registry: &Registry) -> Result<&'static str, CompilerError> {
    let spec = registry
        .relation_by_id(source.relation_id)
        .ok_or_else(|| invalid("source relation absent"))?;
    Ok(match spec.key.name {
        "instance_equations" => "instance",
        "template_display" => "display",
        "template_contributions" => "contribution",
        "template_guards" => "guard",
        _ => "template",
    })
}

fn source_row(
    source: &SourceExpression,
    id: SemanticId,
    family: &str,
    syntax: &str,
    root: i64,
    derivation: SemanticId,
) -> Result<normalized::expression_sources::Row, CompilerError> {
    Ok(normalized::expression_sources::Row {
        source_id: id,
        family: family.parse()?,
        source_relation_id: source.relation_id,
        source_key: source.row_key,
        field_path: source.document_path.clone(),
        owner: match source.owner_kind {
            ExpressionOwnerKind::Template => {
                normalized::expression_sources::NormalizedExpressionSourcesFieldOwner::from_template(
                    normalized::expression_sources::NormalizedExpressionSourcesFieldOwnerTemplate {
                        template_id: source.owner_id,
                    },
                )
            }
            ExpressionOwnerKind::Instance => {
                normalized::expression_sources::NormalizedExpressionSourcesFieldOwner::from_instance(
                    normalized::expression_sources::NormalizedExpressionSourcesFieldOwnerInstance {
                        instance_id: source.owner_id,
                    },
                )
            }
        },
        syntax: syntax.parse()?,
        root_id: root,
        derivation_id: derivation,
        source_span: pse_relations::generated::extension_values::SourceSpan {
            document_id: source.document_id,
            start: i64::from(source.source_span.start),
            end: i64::from(source.source_span.end),
        },
    })
}
