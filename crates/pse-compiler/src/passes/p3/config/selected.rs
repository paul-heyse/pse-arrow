// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected method roots reuse the same checked configuration and native products.
use super::{Configuration, invalid};
use crate::{
    CompilerError, InputBundle, PassContext,
    passes::native_outputs::{Sources, materialize},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::SemanticId;
use pse_relations::generated::{authored, normalized};
use pse_rules::strata::native_input::NativeInput;
use pse_schema::model::{PassSpec, RelationKey};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) struct SelectedRoot {
    pub instance: authored::instances::Row,
    pub source_relation: SemanticId,
    /// Exact native key from the retained P6 selection computation.
    pub source_key: pse_ids::ContentHash,
}

/// Construct selected contexts once and retain each output's actual source mapping.
/// Neither output equality nor a second execution establishes producer correspondence.
pub(crate) async fn configure_selected(
    inputs: &InputBundle,
    roots: &[SelectedRoot],
    pass: &PassSpec,
    session: &SnapshotSession,
    ctx: &PassContext<'_>,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    let registry = session.registry();
    let cancel = ctx.cancel;
    let mut checked = inputs.checked_rows(registry)?;
    let mut sources = Sources::from_inputs(inputs, registry)?;
    let batches = checked
        .values()
        .map(|batch| (batch.relation_id(), batch.clone()))
        .collect();
    let configuration = Configuration::build_selected(
        &batches,
        &sources,
        ctx.documents,
        session,
        ctx.physical()?,
        cancel,
        roots,
    )
    .await?;
    let expressions = checked
        .get(&normalized::expression_sources::RELATION_KEY)
        .ok_or_else(|| {
            invalid("selected configuration requires actual normalized expression sources")
        })?;
    let configured = configuration.emit(expressions, &sources).await?;
    let mut output = materialize(configured.output, &sources, pass, session, cancel).await?;
    let mut overrides = BTreeMap::new();
    for (key, input) in &output {
        sources.replace_native(*key, Arc::clone(input))?;
        checked.insert(*key, input.checked().clone());
        overrides.insert(*key, input.checked().clone());
    }
    let session = crate::passes::native_rows::workspace(session, &overrides, cancel)?;
    output.extend(super::super::products::emit(&checked, &sources, pass, &session, cancel).await?);
    if output.keys().any(|key| {
        !pass
            .outputs
            .iter()
            .any(|port| port.relation == key.qualified_name())
    }) {
        return Err(invalid(
            "selected configuration produced a relation outside the actual pass outputs",
        ));
    }
    Ok(output)
}
