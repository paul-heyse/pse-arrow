// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Demand-shaped preparation from complete locally owned source premises.
use super::*;
use crate::math::modeling::PreparedBasis;
use std::{future::Future, time::Instant};

fn preparation_time_limit() -> WorkflowError {
    crate::math::MathRuntimeError::Solve(pse_backend_native::ProblemError::Limit {
        kind: pse_backend_native::LimitKind::Time,
        detail: "scientific preparation deadline".into(),
    })
    .into()
}

/// A shared pure flight keeps its own charged completion owner when one waiter
/// cancels. Each waiter consumes its original enclosing operation clock.
async fn scoped<T>(
    cancel: &crate::CancelSource,
    deadline: Instant,
    operation: impl Future<Output = Result<T, WorkflowError>>,
) -> Result<T, WorkflowError> {
    let deadline = cancel
        .deadline()
        .map_or(deadline, |parent| parent.min(deadline));
    if cancel.token().is_cancelled() {
        return Err(crate::math::MathRuntimeError::Cancelled.into());
    }
    if Instant::now() >= deadline {
        return Err(preparation_time_limit());
    }
    tokio::select! {
        biased;
        () = cancel.cancelled() => Err(crate::math::MathRuntimeError::Cancelled.into()),
        () = tokio::time::sleep_until(deadline.into()) => Err(preparation_time_limit()),
        result = operation => result,
    }
}

impl ModelingPackage {
    /// Selection resolves exact local presence/absence and source ownership.
    /// It neither acquires nor renews publication permission.
    pub(super) async fn checked_selection_scoped(
        &self,
        roots: &[DeclarationId],
        cancel: &crate::CancelSource,
        deadline: Instant,
    ) -> Result<canonical::CheckedSelection, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let phase = cancel.child_with_deadline(Some(deadline));
                scoped(&phase, deadline, self.checked_selection(roots, &phase)).await
            })
            .await
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the complete local specialization returns its held basis"
    )]
    pub(super) async fn prepare_selected_basis(
        &self,
        selected: &mut canonical::CheckedSelection,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        cancel: &crate::CancelSource,
        deadline: Instant,
    ) -> Result<(ModelingPreparation, Arc<PreparedBasis>), WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let math = self.runtime.shared.math();
                if !selected
                    .admission
                    .dependencies
                    .matches(&self.revision.admitted)
                {
                    return Err(contract(
                        "selected source premises differ from the local receiver",
                    ));
                }
                let key = math.basis_key(&selected.admission, root, instance, bindings, limits)?;
                let basis = if let Some(basis) = math.modeling_cache.basis(&key)
                    && basis.source_premises == selected.admission.dependencies
                {
                    basis
                } else {
                    let workspace = self.pure_workspace()?;
                    let frontier = scoped(cancel, deadline, async {
                        Ok(math
                            .plan_frontier_keyed(
                                key.clone(),
                                workspace.clone(),
                                selected.admission.revision.clone(),
                                cancel,
                                deadline,
                            )
                            .await?)
                    })
                    .await?;
                    scoped(cancel, deadline, async {
                        Ok(math
                            .complete_frontier_keyed(
                                key,
                                selected.admission.dependencies.clone(),
                                workspace,
                                frontier,
                                cancel,
                                deadline,
                            )
                            .await?)
                    })
                    .await?
                };
                let model = basis
                    .bind(&selected.admission.revision, root, instance)?
                    .with_consumed_source_versions(
                        selected.admission.versions.clone(),
                        selected.admission.metadata.clone(),
                    );
                Ok((model, basis))
            })
            .await
    }
}
