// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P1 parses a complete package inventory and stages typed candidate changes.

use crate::change_set::{AuthoredReader, ChangeSet};
use crate::document::{DocumentBundle, Rows};
use crate::{AuthoringError, ParseBudget, SourceSpan};
use pse_ids::SemanticId;
use pse_relations::generated::authored;
use pse_schema::Registry;
use std::collections::BTreeMap;

/// Reparse the exact input documents, resolve P0 dependencies and bind targets before
/// staging a complete authored-package inventory against the supplied base.
///
/// The bundle list is the complete desired package universe, not a partial merge.
/// Other registry/runtime sidecars are retained; package relations include explicit empties.
/// # Errors
/// Syntax, identity, dependency, target, generated row and exact-base failures.
pub fn stage(
    bundles: &[DocumentBundle],
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    registry: &Registry,
) -> Result<ChangeSet, AuthoringError> {
    let parsed = bundles
        .iter()
        .map(|bundle| {
            let mut texts = BTreeMap::new();
            for document in &bundle.documents {
                if texts
                    .insert(document.path.clone(), document.text.clone())
                    .is_some()
                {
                    return Err(contract("duplicate original document path"));
                }
            }
            crate::document::load_package_texts(texts, registry, ParseBudget::default())
        })
        .collect::<Result<Vec<_>, _>>()?;
    stage_parsed(&parsed, base, header, registry, None)
}

/// Stage an immutable actual loader inventory while retaining work/output ownership.
/// # Errors
/// Exact source/base/target disagreement, cancellation or reservation refusal.
pub fn stage_owned(
    bundles: &crate::document::OwnedDocumentSet,
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    registry: &Registry,
    reserver: &dyn pse_ids::MemoryReserver,
    cancel: &pse_ids::CancellationToken,
) -> Result<crate::change_set::OwnedChangeSet, AuthoringError> {
    bundles.validate_registry(registry)?;
    cancel.checkpoint()?;
    let mut work = reserver.open("authoring:stage");
    work.try_grow(crate::work::sources(bundles.bundles())?)?;
    work.try_grow(crate::work::mul(registry.relations().len(), 2048)?)?;
    let mut source_ids = bundles
        .bundles()
        .iter()
        .flat_map(|bundle| bundle.rows.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    for document in registry.documents() {
        for section in &document.sections {
            source_ids.insert(
                registry
                    .relation(section.relation)
                    .ok_or_else(|| contract("source relation missing"))?
                    .id,
            );
        }
    }
    source_ids.extend([
        authored::packages::RELATION_ID,
        authored::documents::RELATION_ID,
        authored::entities::RELATION_ID,
    ]);
    for (id, batch) in base.relations()? {
        if source_ids.contains(&id) {
            work.try_grow(pse_ids::validation_extent(&batch)?)?;
        }
    }
    let changes = stage_parsed(
        bundles.bundles(),
        base,
        header,
        registry,
        Some((work.as_mut(), cancel)),
    )?;
    cancel.checkpoint()?;
    Ok(crate::change_set::OwnedChangeSet::new(changes, work))
}

fn stage_parsed(
    parsed: &[DocumentBundle],
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    registry: &Registry,
    target_work: TargetWork<'_>,
) -> Result<ChangeSet, AuthoringError> {
    crate::p0::resolve(parsed, registry)?;
    let mut rows = Rows::new();
    let mut spans = BTreeMap::<SemanticId, Vec<SourceSpan>>::new();
    for document in registry.documents() {
        for section in &document.sections {
            let relation = registry
                .relation(section.relation)
                .ok_or_else(|| contract("unknown document relation"))?;
            rows.entry(relation.id).or_default();
        }
    }
    for relation in [
        authored::packages::RELATION_ID,
        authored::documents::RELATION_ID,
        authored::entities::RELATION_ID,
    ] {
        rows.entry(relation).or_default();
    }
    for bundle in parsed {
        for (id, values) in &bundle.rows {
            rows.entry(*id).or_default().extend(values.iter().cloned());
        }
        for document in &bundle.documents {
            for (id, values) in &document.row_spans {
                spans.entry(*id).or_default().extend(values.iter().copied());
            }
        }
    }
    crate::document::binding::bind_parsed(parsed, &rows, registry)?;
    bind_targets_inner(&mut rows, &spans, registry, target_work)?;
    crate::change_set::stage_rows(base, header, &rows, registry)
}

/// Bind each retained target string to the complete actual current identity inventory.
/// # Errors
/// Any source row, instance/member ambiguity, domain mismatch or expansion failure.
pub fn bind_targets(
    rows: &mut Rows,
    spans: &BTreeMap<SemanticId, Vec<SourceSpan>>,
    registry: &Registry,
) -> Result<(), AuthoringError> {
    bind_targets_inner(rows, spans, registry, None)
}
type TargetWork<'a> = Option<(
    &'a mut dyn pse_ids::Reservation,
    &'a pse_ids::CancellationToken,
)>;
fn bind_targets_inner(
    rows: &mut Rows,
    spans: &BTreeMap<SemanticId, Vec<SourceSpan>>,
    registry: &Registry,
    mut work: TargetWork<'_>,
) -> Result<(), AuthoringError> {
    let context = crate::targets::TargetContext::from_rows(rows)?;
    let sources = [
        (
            authored::case_specs::RELATION_ID,
            authored::case_spec_targets::RELATION_ID,
        ),
        (
            authored::case_activations::RELATION_ID,
            authored::case_activation_targets::RELATION_ID,
        ),
        (
            authored::observations::RELATION_ID,
            authored::observation_targets::RELATION_ID,
        ),
    ];
    for (source, destination) in sources {
        let spec = registry
            .relations()
            .iter()
            .find(|spec| spec.id == source)
            .ok_or_else(|| contract("undeclared target source relation"))?;
        let [key] = spec.primary_key.as_slice() else {
            return Err(contract(
                "target source requires its declared single identity key",
            ));
        };
        let identity_column = spec
            .columns
            .iter()
            .position(|column| column.name == *key)
            .ok_or_else(|| contract("target source key column missing"))?;
        let target_column = spec
            .columns
            .iter()
            .position(|column| column.name == "target")
            .ok_or_else(|| contract("target source text column missing"))?;
        let mut targets = Vec::new();
        for (ordinal, row) in rows.get(&source).into_iter().flatten().enumerate() {
            let (Some(pse_schema::model::Cell::Id(id)), Some(pse_schema::model::Cell::Text(text))) =
                (row.get(identity_column), row.get(target_column))
            else {
                return Err(contract("malformed generated target source row"));
            };
            let at = spans
                .get(&source)
                .and_then(|spans| spans.get(ordinal))
                .copied()
                .unwrap_or(SourceSpan::head(SemanticId::NIL));
            let path = crate::targets::parse(text, at)?;
            let resolved = if let Some((reservation, cancel)) = &mut work {
                crate::targets::resolve_accounted(&path, &context, *id, *reservation, cancel)?
            } else {
                crate::targets::resolve(&path, &context, *id)?
            };
            targets.extend(
                resolved
                    .into_iter()
                    .map(crate::targets::TargetRow::into_cells),
            );
        }
        rows.insert(destination, targets);
    }
    Ok(())
}
fn contract(reason: &str) -> AuthoringError {
    AuthoringError::Contract {
        at: None,
        reason: reason.to_owned(),
    }
}
