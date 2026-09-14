// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P3 normalizes admitted source rows while retaining complete typed syntax and bindings.

mod demand_paths;
mod lower;
mod seeds;
mod units;
mod validate;

use crate::{
    CompilerError, InputBundle, PassContext, PassOutput, PassRecordDraft, PassStatus,
    mathir_relations::{Family, MathRows, RelationSink},
};
use pse_authoring::document::{
    DocumentBundle, OwnedDocumentSet, Rows,
    binding::{SourceExpression, bind_sources_owned},
};
use pse_ids::{CancellationToken, MemoryReserver, SemanticId};
use pse_relations::{RecordBatch, generated::authored};
use pse_schema::{
    Registry,
    model::{Cell, Namespace, PassSpec, RelationKey},
};
use std::{collections::BTreeMap, time::Instant};

/// Executable implementation bound to the exact registered P3 specification.
#[derive(Debug)]
pub struct P3 {
    spec: PassSpec,
}
impl P3 {
    /// Select the declared P3 implementation contract.
    /// # Errors
    /// Missing P3 declaration.
    pub fn new(registry: &Registry) -> Result<Self, CompilerError> {
        Ok(Self {
            spec: registry
                .pass("P3@1")
                .ok_or_else(|| invalid("P3 declaration missing"))?
                .clone(),
        })
    }
}
impl crate::Pass for P3 {
    fn spec(&self) -> &PassSpec {
        &self.spec
    }
    fn run<'a>(
        &'a self,
        ctx: &'a PassContext<'a>,
        inputs: &'a InputBundle,
    ) -> pse_catalog::provider::BoxFut<'a, Result<PassOutput, CompilerError>> {
        Box::pin(async move {
            let started = Instant::now();
            inputs.validate(&self.spec, ctx.registry)?;
            let inputs = inputs.rows(ctx.registry)?;
            let output = normalize_owned(
                &inputs,
                ctx.documents,
                ctx.registry,
                ctx.reserver,
                ctx.cancel,
            )?;
            let ports = self
                .spec
                .outputs
                .iter()
                .map(|port| {
                    let spec = ctx
                        .registry
                        .relation(&port.relation)
                        .ok_or_else(|| invalid("P3 output undeclared"))?;
                    let batch = output
                        .get(&spec.key)
                        .ok_or_else(|| invalid("P3 omitted an output"))?;
                    Ok((port.port, vec![batch.clone()]))
                })
                .collect::<Result<_, CompilerError>>()?;
            Ok(PassOutput {
                ports,
                findings: Vec::new(),
                record: PassRecordDraft {
                    pass_id: self.spec.id,
                    version: self.spec.version,
                    duration: started.elapsed(),
                    status: PassStatus::Ok,
                },
            })
        })
    }
}

/// Normalize complete admitted primitives and original source documents.
/// # Errors
/// Stale/missing sources, invalid references, physical representation ambiguity, exceeded
/// reservations/cancellation, or a malformed complete output contract.
pub fn normalize(
    inputs: &BTreeMap<RelationKey, RecordBatch>,
    documents: &[DocumentBundle],
    registry: &Registry,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, RecordBatch>, CompilerError> {
    let owned = pse_authoring::document::load_bundles_owned(documents, registry, reserver, cancel)?;
    normalize_owned(inputs, &owned, registry, reserver, cancel)
}

/// Normalize actual immutable source owners without a second source parse.
/// # Errors
/// The same semantic, source correspondence and resource failures as `normalize`.
pub fn normalize_owned(
    inputs: &BTreeMap<RelationKey, RecordBatch>,
    documents: &OwnedDocumentSet,
    registry: &Registry,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, RecordBatch>, CompilerError> {
    cancel.checkpoint()?;
    let mut work = reserver.open("P3:normalization");
    // Input buffers already retain their own leases. Forecast decoded cells from
    // visible values, so shared IPC backing is not charged once per field again.
    let row_bytes = inputs.values().try_fold(0_usize, |total, batch| {
        total
            .checked_add(pse_catalog::store::membership::validation_extent(batch)?)
            .ok_or_else(|| invalid("P3 workspace size overflow"))
    })?;
    // Source trees and binding inventories coexist with decoded rows until emission.
    let bytes = documents
        .bundles()
        .iter()
        .flat_map(|bundle| &bundle.documents)
        .try_fold(0_usize, |total, document| {
            total.checked_add(document.text.len())
        })
        .and_then(|bytes| bytes.checked_mul(64))
        .and_then(|bytes| bytes.checked_add(row_bytes))
        .ok_or_else(|| invalid("P3 workspace size overflow"))?;
    work.try_grow(bytes)
        .map_err(|_| CompilerError::ResourceLimit {
            consumer: "P3 normalization workspace".to_owned(),
            config_keys: vec!["memory_limit".to_owned()],
        })?;
    let rows = source_rows(inputs, registry)?;
    checked_documents(documents, &rows)?;
    let bindings = bind_sources_owned(documents, &rows, registry, reserver, cancel)?;
    let mut output = MathRows::new();
    for spec in registry
        .relations()
        .iter()
        .filter(|spec| spec.key.namespace == Namespace::Normalized)
    {
        output.insert(spec.key, Vec::new());
    }
    copy_primitives(&rows, &mut output, registry)?;
    let mut units = units::Units::new(inputs, &rows, registry)?;
    let mut offsets = BTreeMap::<&'static str, u64>::new();
    let target_context = pse_authoring::targets::TargetContext::from_rows(&rows)?;
    let mut demands = seeds::Demands::new(&rows)?;
    for source in bindings.expressions() {
        cancel.checkpoint()?;
        let prefix = family(source, registry)?;
        let offset = *offsets.get(prefix).unwrap_or(&0);
        let source_id = pse_ids::named_id(
            source.document_id,
            &format!("pse:p3:source:v1:{}", source.document_path),
        );
        let package = owner_package(source, &rows)?;
        let mut lowered = lower::lower(
            source,
            source_id,
            offset,
            &target_context,
            cancel,
            &mut units,
            package,
        )?;
        demands.source(
            source,
            source_id,
            &mut lowered,
            offset,
            cancel,
            &target_context,
        )?;
        validate::lowered(&lowered, source_id, offset, registry)?;
        let next = emit_source(
            source,
            source_id,
            prefix,
            offset,
            lowered,
            &mut output,
            registry,
        )?;
        offsets.insert(prefix, next);
    }
    demands.finish(&target_context, &mut output, registry, cancel)?;
    append(
        &mut output,
        registry,
        "normalized.units",
        units.derived.into_values().collect(),
    )?;
    output
        .into_iter()
        .map(|(key, rows)| {
            cancel.checkpoint()?;
            let spec = registry
                .relation(&key.qualified_name())
                .ok_or_else(|| invalid("normalized output schema missing"))?;
            Ok((
                key,
                pse_relations::cells::batch_from_cells_owned(
                    registry, spec, &rows, reserver, cancel,
                )?,
            ))
        })
        .collect()
}
fn emit_source(
    source: &SourceExpression,
    source_id: SemanticId,
    prefix: &'static str,
    offset: u64,
    lowered: lower::Lowered,
    output: &mut MathRows,
    registry: &Registry,
) -> Result<u64, CompilerError> {
    let derivation = pse_ids::named_id(source_id, "pass:P3@1");
    let mut sink = RelationSink::new(
        registry,
        Family::Normalized {
            prefix,
            offset,
            derivation,
            source_span: source.source_span,
        },
    );
    let nodes = lowered
        .graph
        .iter()
        .map(|(node, _)| node)
        .collect::<Vec<_>>();
    pse_mathir::relations::emit_untyped(&lowered.graph, &nodes, &mut sink)?;
    for (key, values) in sink.into_rows() {
        output.entry(key).or_default().extend(values);
    }
    append(
        output,
        registry,
        "normalized.expression_sources",
        vec![source_row(
            source,
            source_id,
            prefix,
            lowered.syntax,
            lowered.root,
            derivation,
            registry,
        )?],
    )?;
    append(
        output,
        registry,
        "normalized.predicate_nodes",
        lowered.predicates,
    )?;
    append(
        output,
        registry,
        "normalized.equation_nodes",
        lowered.equations,
    )?;
    append(
        output,
        registry,
        "normalized.expression_index_bindings",
        lowered.bindings,
    )?;
    offset
        .checked_add(
            u64::try_from(lowered.graph.len()).map_err(|_| invalid("node count overflow"))?,
        )
        .ok_or_else(|| invalid("node ordinal overflow"))
}

fn source_rows(
    inputs: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
) -> Result<Rows, CompilerError> {
    inputs
        .iter()
        .map(|(key, batch)| {
            let spec = registry
                .relation(&key.qualified_name())
                .filter(|spec| spec.key == *key)
                .ok_or_else(|| invalid("input schema version differs"))?;
            Ok((
                spec.id,
                pse_relations::cells::cells_from_batch(registry, spec, batch)?,
            ))
        })
        .collect()
}
fn copy_primitives(
    rows: &Rows,
    output: &mut MathRows,
    registry: &Registry,
) -> Result<(), CompilerError> {
    for spec in registry
        .relations()
        .iter()
        .filter(|spec| spec.key.namespace == Namespace::Authored)
    {
        if let Some(target) = registry.relation(&format!("normalized.{}", spec.key.name)) {
            let values = rows
                .get(&spec.id)
                .ok_or_else(|| invalid("P3 authored input relation absent"))?;
            output.insert(target.key, values.clone());
        }
    }
    let headers = rows
        .get(&authored::packages::RELATION_ID)
        .ok_or_else(|| invalid("P3 package headers absent"))?
        .iter()
        .cloned()
        .map(authored::packages::Row::from_cells)
        .collect::<Result<Vec<_>, _>>()?;
    let graph = pse_authoring::p0::resolve_headers(&headers, registry)?;
    append(
        output,
        registry,
        "normalized.package_graph",
        graph
            .rows
            .into_iter()
            .map(pse_relations::generated::normalized::package_graph::Row::into_cells)
            .collect(),
    )
}
fn family(source: &SourceExpression, registry: &Registry) -> Result<&'static str, CompilerError> {
    let relation = registry
        .relations()
        .iter()
        .find(|spec| spec.id == source.relation_id)
        .ok_or_else(|| invalid("source relation absent"))?;
    Ok(match relation.key.name {
        "template_display" => "display",
        "template_contributions" => "contribution",
        "template_guards" => "guard",
        _ => "template",
    })
}
fn owner_package(source: &SourceExpression, rows: &Rows) -> Result<SemanticId, CompilerError> {
    let templates = rows
        .get(&authored::templates::RELATION_ID)
        .ok_or_else(|| invalid("source templates absent"))?;
    let owners = templates
        .iter()
        .cloned()
        .map(authored::templates::Row::from_cells)
        .collect::<Result<Vec<_>, _>>()?;
    let owners = owners
        .iter()
        .filter(|row| row.template_id == source.owner_template_id)
        .collect::<Vec<_>>();
    let [owner] = owners.as_slice() else {
        return Err(invalid("source owner is missing or ambiguous"));
    };
    Ok(owner.package_id)
}
fn append(
    output: &mut MathRows,
    registry: &Registry,
    name: &str,
    rows: Vec<Vec<Cell>>,
) -> Result<(), CompilerError> {
    let spec = registry
        .relation(name)
        .ok_or_else(|| invalid("output declaration absent"))?;
    output
        .get_mut(&spec.key)
        .ok_or_else(|| invalid("undeclared output inventory"))?
        .extend(rows);
    Ok(())
}
fn source_row(
    source: &SourceExpression,
    id: SemanticId,
    family: &'static str,
    syntax: &'static str,
    root: u64,
    derivation: SemanticId,
    registry: &Registry,
) -> Result<Vec<Cell>, CompilerError> {
    let spec = registry
        .relations()
        .iter()
        .find(|spec| spec.id == source.relation_id)
        .ok_or_else(|| invalid("source relation absent"))?;
    let key = spec
        .primary_key
        .iter()
        .zip(&source.row_key)
        .map(|(name, value)| {
            let column = spec
                .columns
                .iter()
                .find(|column| column.name == *name)
                .ok_or_else(|| invalid("source key column absent"))?;
            let ty = registry
                .logical_type(&column.logical_type.name())
                .ok_or_else(|| invalid("source key type absent"))?;
            let mut parts = vec![
                Cell::Text((*name).to_owned()),
                Cell::Id(ty.id),
                Cell::Null,
                Cell::Null,
                Cell::Null,
                Cell::Null,
                Cell::Null,
                Cell::Null,
                Cell::Null,
            ];
            let (slot, value) = match value {
                Cell::Id(value) => (2, Cell::Id(*value)),
                Cell::Hash(value) => (3, Cell::Hash(*value)),
                Cell::Text(value) => (4, Cell::Text(value.clone())),
                Cell::Enum(value) => (4, Cell::Text((*value).to_owned())),
                Cell::I64(value) => (5, Cell::I64(*value)),
                Cell::U64(value) => (6, Cell::U64(*value)),
                Cell::Bool(value) => (7, Cell::Bool(*value)),
                Cell::List(values) => (8, Cell::List(values.clone())),
                _ => return Err(invalid("source key has an inadmissible value kind")),
            };
            parts[slot] = value;
            Ok(Cell::Struct(parts))
        })
        .collect::<Result<Vec<_>, CompilerError>>()?;
    Ok(vec![
        Cell::Id(id),
        Cell::Enum(family),
        Cell::Id(source.relation_id),
        Cell::List(key),
        Cell::Text(source.document_path.clone()),
        Cell::Id(source.owner_template_id),
        Cell::Enum(syntax),
        Cell::U64(root),
        Cell::Id(derivation),
        Cell::Struct(vec![
            Cell::Id(source.document_id),
            Cell::U64(u64::from(source.source_span.start)),
            Cell::U64(u64::from(source.source_span.end)),
        ]),
    ])
}
fn invalid(reason: &str) -> CompilerError {
    pse_authoring::AuthoringError::Contract {
        at: None,
        reason: reason.to_owned(),
    }
    .into()
}

fn checked_documents(documents: &OwnedDocumentSet, rows: &Rows) -> Result<(), CompilerError> {
    let parsed = documents.bundles();
    let expected = rows
        .get(&authored::documents::RELATION_ID)
        .ok_or_else(|| invalid("original document inventory input absent"))?;
    let actual = parsed
        .iter()
        .flat_map(|bundle| {
            bundle
                .rows
                .get(&authored::documents::RELATION_ID)
                .into_iter()
                .flatten()
        })
        .collect::<Vec<_>>();
    let literals = |values: &[Cell]| values.iter().map(Cell::literal_spec).collect::<Vec<_>>();
    let expected_set = expected
        .iter()
        .map(|row| literals(row))
        .collect::<std::collections::BTreeSet<_>>();
    let actual_set = actual
        .iter()
        .map(|row| literals(row))
        .collect::<std::collections::BTreeSet<_>>();
    if expected.len() != actual.len()
        || expected_set.len() != expected.len()
        || expected_set != actual_set
    {
        return Err(invalid(
            "complete original document inventory differs from the bound inputs",
        ));
    }
    for bundle in parsed {
        for (relation, supplied) in &bundle.rows {
            let actual = rows
                .get(relation)
                .ok_or_else(|| invalid("original source relation absent from inputs"))?;
            let actual = actual
                .iter()
                .map(|row| literals(row))
                .collect::<std::collections::BTreeSet<_>>();
            if supplied.iter().any(|row| !actual.contains(&literals(row))) {
                return Err(invalid(
                    "original typed source values differ from the bound inputs",
                ));
            }
        }
    }
    Ok(())
}
