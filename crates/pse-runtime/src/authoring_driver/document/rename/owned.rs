// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Identity-bound source edits consume the actual parsed document child.
use super::{
    binding, contract, edits, entity, name_replacement, preserve_expression_bindings,
    target_replacements, valid_name, yaml_scalar,
};
use crate::authoring_driver::{DriverError, ParseBudget, document::OwnedDocumentSet};
use pse_columnar::CancellationToken;
use pse_engine::session::EngineSession;
use pse_ids::SemanticId;
use pse_relations::generated::enums::IdPolicy;
use std::collections::BTreeMap;

/// Rename one explicit identity and every bound expression/target reference.
/// `expected_name` is an exact before-image check on the selected source child.
/// The returned parser owner contains the new bytes and their generated relation rows.
/// # Errors
/// Stale name, named identity policy, shadowing/rebinding, source edits or resources.
pub async fn rename_documents(
    bundles: &OwnedDocumentSet,
    entity_id: SemanticId,
    expected_name: &str,
    new_name: &str,
    session: &EngineSession,
    cancel: &CancellationToken,
) -> Result<OwnedDocumentSet, DriverError> {
    let registry = session.registry();
    bundles.validate_registry(registry)?;
    valid_name(new_name)?;
    let mut work = pse_columnar::MemoryConsumer::new("authoring:rename").register(session.pool());
    work.try_grow(crate::authoring_driver::work::sources(bundles.bundles())?)?;
    let (before, bindings, mut completed) =
        crate::authoring_driver::p1::project_sources(bundles, session, cancel, &mut work).await?;
    let declaration = entity(&before, entity_id)?;
    if declaration.name != expected_name {
        return Err(contract(
            None,
            "rename before-image name differs from selected source",
        ));
    }
    let package = bundles
        .bundles()
        .iter()
        .find(|bundle| bundle.package.package_id == declaration.package_id)
        .ok_or_else(|| contract(None, "rename package absent"))?;
    if package.package.id_policy == IdPolicy::Named {
        return Err(DriverError::Authoring(
            pse_authoring::AuthoringError::RenameNamed {
                entity_id,
                qualified_name: declaration.qualified_name,
            },
        ));
    }
    if expected_name == new_name {
        return Ok(bundles.clone());
    }
    let mut replacements = BTreeMap::new();
    name_replacement(bundles.bundles(), entity_id, new_name, &mut replacements)?;
    for expression in bindings.expressions() {
        if let Some(text) = binding::rename_expression(expression, entity_id, new_name)? {
            replacements
                .entry(expression.document_id)
                .or_default()
                .push((expression.source_span, yaml_scalar(&text)?));
        }
    }
    target_replacements(
        bundles.bundles(),
        entity_id,
        new_name,
        &mut replacements,
        &mut super::TargetResolution {
            batches: &before,
            session,
            work: &mut work,
            completed: &mut completed,
            cancel,
        },
    )
    .await?;
    let edits = edits(bundles.bundles(), replacements)?;
    let updated = bundles.edit(
        &edits,
        registry,
        ParseBudget::default(),
        session.pool(),
        cancel,
    )?;
    let (after, updated_bindings, _) =
        crate::authoring_driver::p1::project_sources(&updated, session, cancel, &mut work).await?;
    super::native::preserved(&before, &after, session, cancel, &mut completed).await?;
    preserve_expression_bindings(bindings.expressions(), updated_bindings.expressions())?;
    Ok(updated)
}
