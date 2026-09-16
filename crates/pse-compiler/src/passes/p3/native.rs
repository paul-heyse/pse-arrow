// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native source selection retains its exact keys and typed algorithm arguments.
use crate::{
    CompilerError,
    passes::{
        native_outputs::{SourceKey, Sources},
        native_rows::{AlgorithmInputs, keyed_rows, scan},
    },
};
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_relations::columnar::RelationRow;

pub(super) async fn rows<T: RelationRow>(
    arguments: &mut AlgorithmInputs,
    sources: &Sources,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<Vec<(T, SourceKey)>, CompilerError> {
    let registry = session.registry();
    let spec = T::relation(registry)?;
    let (port, _) = sources
        .get(&spec.key)
        .ok_or_else(|| super::invalid("P3 source relation has no unique immutable role"))?;
    Ok(keyed_rows::<T>(
        arguments,
        scan(session, spec, "source")?,
        session,
        registry,
        cancel,
    )
    .await?
    .into_iter()
    .map(|value| {
        (
            value.row,
            SourceKey {
                relation: spec.key,
                port: port.clone(),
                key: value.key,
            },
        )
    })
    .collect())
}
