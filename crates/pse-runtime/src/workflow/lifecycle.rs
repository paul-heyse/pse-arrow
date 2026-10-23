// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public current-owner lifecycle composition; host resource authority stays in Python.
use super::{Runtime, WorkflowError};
pub use pse_operations::canonical_lifecycle::{
    LifecycleBlocker, LifecycleBytes, LifecycleExplanation, LifecycleKind, LifecycleRelease,
    LifecycleSelection, LifecycleTarget,
};
impl Runtime {
    /// Select exact current retention-owner coordinates without acquiring scientific payloads.
    pub async fn lifecycle_select(
        &self,
        target: &LifecycleTarget,
    ) -> Result<LifecycleSelection, WorkflowError> {
        self.scope_activity(async { Ok(self.canonical_store()?.lifecycle_select(target).await?) })
            .await
    }
    /// Read original references/protections. This document is never a deletion grant.
    pub async fn lifecycle_explain(
        &self,
        selection: &LifecycleSelection,
    ) -> Result<LifecycleExplanation, WorkflowError> {
        self.scope_activity(async {
            Ok(self.canonical_store()?.lifecycle_explain(selection).await?)
        })
        .await
    }
    /// Explicitly route an exact-generation release through the original owner.
    /// Project/draft visibility and physical file collection remain distinct commands.
    pub async fn lifecycle_release(
        &self,
        selection: &LifecycleSelection,
    ) -> Result<LifecycleRelease, WorkflowError> {
        self.scope_activity(async {
            Ok(self.canonical_store()?.lifecycle_release(selection).await?)
        })
        .await
    }
}
