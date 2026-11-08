// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Passive rollout premises held by the original physical service owner.

use super::{
    Instant, NativeCanonicalService, NativeServiceError, Path, StudyModuleReceipt, UnitObservation,
};
use crate::{
    canonical::maintenance::{CatalogObservation, CatalogRead, FrozenRollout},
    generated::surreal::{INTERPRETATION, SCHEMA_DIGEST},
    transport::operation_scope,
};
use pse_host::surrealkit::VerifiedRelease;
use serde::Serialize;

/// The original v4 server instance, authenticated target and frozen rollout.
///
/// Only the live native service owner can observe this binding. A retained
/// observation can describe the premise of a preview. It cannot authorize a
/// later operation without revalidation by the exact maintenance owner.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct OwnedRolloutPreview {
    /// Original native service invocation and selected process.
    listener: UnitObservation,
    /// Exact prepared server recipe proved by the running executable.
    server_recipe: String,
    /// Authenticated issuer, original marker and installed declarations.
    catalog: CatalogObservation,
    /// Current generated target interpretation.
    target_interpretation: &'static str,
    /// Current generated target declaration and codec digest.
    target_schema_digest: &'static str,
    /// Exact installed readonly module verified against the owned receipt.
    loaded_module_sha256: String,
    /// Prepared SurrealKit input generation validated by the host.
    tool_input_sha256: String,
    /// Exact executable from that prepared generation.
    tool_binary_sha256: String,
    /// Checked producer-owned manifest and ordered frozen step identities.
    rollout: FrozenRollout,
}

impl NativeCanonicalService {
    /// Observe the exact rollout target using the live application and listener owner.
    ///
    /// Bracket the Viewer catalog observation with actual listener ownership
    /// checks. This path reads the selected state and installed definitions
    /// without changing admission, credentials, module or database contents.
    pub async fn preview_frozen_rollout(
        &self,
        manifest: &Path,
        tool: &VerifiedRelease,
        deadline: Instant,
    ) -> Result<OwnedRolloutPreview, NativeServiceError> {
        self.state.verify()?;
        self.records.verify(&self.state.path)?;
        let store = self.ready()?.store();
        let rollout = FrozenRollout::read(manifest)?;
        let listener = self.wait_owned_listener(deadline).await?;
        let module = StudyModuleReceipt::read(&self.state.path)?;
        let catalog = operation_scope(deadline, async {
            store.validate_study_module(&module).await?;
            CatalogRead::from_state(
                &self.state.path,
                Some(store.database()),
                store.issuing_store()?,
            )?
            .inspect_v4()
            .await
        })
        .await?;
        let after = self.wait_owned_listener(deadline).await?;
        self.state.verify()?;
        self.records.verify(&self.state.path)?;
        if after != listener
            || catalog.target.endpoint != store.endpoint()
            || catalog.target.namespace != store.namespace()
            || catalog.marker.interpretation != INTERPRETATION
            || catalog.marker.study_module_sha256 != module.sha256
        {
            return Err(NativeServiceError::Refused(
                "native rollout listener, selected target or loaded module changed",
            ));
        }
        Ok(OwnedRolloutPreview {
            listener,
            server_recipe: self.server_recipe.clone(),
            catalog,
            target_interpretation: INTERPRETATION,
            target_schema_digest: SCHEMA_DIGEST,
            loaded_module_sha256: module.sha256.clone(),
            tool_input_sha256: tool.input_sha256().to_owned(),
            tool_binary_sha256: tool.binary_sha256().to_owned(),
            rollout,
        })
    }
}
