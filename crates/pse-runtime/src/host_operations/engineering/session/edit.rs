// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original operation custody for explicit source-file edits.
use super::*;

impl EngineeringService {
    /// Retain an exact-parent edit before performing the native parsed reload/delta.
    /// # Errors
    /// Changed same-identity requests, stale parents and original owner refusals propagate.
    pub async fn edit_source(
        self: &Arc<Self>,
        input: EngineeringEditSource,
    ) -> Result<EngineeringSourceOutcome, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        if input.operation == input.source_operation {
            return Err(request("source edit cannot depend on its own operation"));
        }
        // Capture an existing receiver before inserting the dependent operation.
        // Every edge therefore points to an earlier admission; forward references
        // cannot create a cycle between concurrently submitted edits.
        let receiver = self.operation(&input.source_operation, Kind::Source)?;
        let bytes = input.delta.edits.iter().try_fold(4096usize, |size, edit| {
            let bytes = match &edit.change {
                SourceFileChange::Add(bytes) | SourceFileChange::Replace(bytes) => bytes.len(),
                SourceFileChange::Remove => 0,
            };
            size.checked_add(bytes)
                .and_then(|value| value.checked_add(edit.path.len() + 1024))
                .ok_or_else(|| resource("source edit request extent overflow"))
        })?;
        let reservation = datafusion::execution::memory_pool::MemoryConsumer::new(
            "engineering:source-edit-request",
        )
        .register(&self.shared.pool());
        reservation.try_grow(bytes).map_err(owner)?;
        let (operation, fresh) = self.issue(&input.operation, Kind::Source, &input)?;
        if fresh {
            let service = self.clone();
            let original = operation.clone();
            self.spawn(operation.clone(), async move {
                let _reservation = reservation;
                let outcome = receiver.dependency(&original.cancel).await?;
                let Outcome::Source { package, .. } = outcome else {
                    return Err(request("source edit receiver is unavailable"));
                };
                let package = package
                    .with_file_delta(
                        &service.runtime.source_services(),
                        &input.delta,
                        &original.cancel,
                    )
                    .await
                    .map_err(owner)?;
                let source = OperationSource::of(&package);
                let selection = EngineeringSourceSelection {
                    operation: input.operation,
                    local_revision: package.revision_id().into(),
                    source_revision: source.revision.as_id(),
                    physical_context: source.physical_context,
                };
                Ok(Outcome::Source {
                    package: Arc::new(package),
                    selection,
                })
            })?;
        }
        source_outcome(operation.snapshot()?)
    }
}
