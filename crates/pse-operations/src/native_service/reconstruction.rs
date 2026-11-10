// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit fresh replacement. Original stopped state is retained, never implicitly erased.

use super::{
    CanonicalOptions, CanonicalStore, Instant, NativeCanonicalService, NativeServiceError,
    NativeServiceSpec, Path, PathBuf, StoppedCanonicalState, StudyModuleReceipt,
};
use serde::{Deserialize, Serialize};

/// Exact original owned state and current empty target selected for reconstruction.
/// This document does not claim that authored inputs or scientific results were transferred.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconstructionIntent {
    operation: String,
    original_state: PathBuf,
    original_device: u64,
    original_inode: u64,
    original_nonce: String,
    target_schema_digest: String,
    target_module_sha256: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ReconstructionRecord {
    intent: ReconstructionIntent,
    target_state: PathBuf,
    target_invocation: String,
    target_device: u64,
    target_inode: u64,
    target_nonce: String,
}

/// Passive original-operation selection. A recorded target is not proof of initialization or liveness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReconstructionObservation {
    /// Original operation.
    pub operation: String,
    /// Private target selected before launch; observing it grants no ownership.
    pub target_state: PathBuf,
    /// Original physical invocation to reconcile, never a synthesized retry.
    pub target_invocation: String,
    /// Exact original target contract.
    pub target_schema_digest: String,
    /// Exact original prepared policy artifact.
    pub target_module_sha256: String,
}

impl ReconstructionRecord {
    fn observation(self) -> ReconstructionObservation {
        ReconstructionObservation {
            operation: self.intent.operation,
            target_state: self.target_state,
            target_invocation: self.target_invocation,
            target_schema_digest: self.intent.target_schema_digest,
            target_module_sha256: self.intent.target_module_sha256,
        }
    }
}

impl NativeCanonicalService {
    /// Read immutable reconstruction provenance on the replacement, even after old-state disposal.
    /// This reports the original selection, not current liveness or permission to reconstruct again.
    /// # Errors
    /// Private origin record belongs to another physical target.
    pub fn reconstruction_origin(
        &self,
    ) -> Result<Option<ReconstructionObservation>, NativeServiceError> {
        self.state.verify()?;
        self.records.verify(self.state())?;
        let path = self.state().join("reconstruction-origin.json");
        if !path.try_exists()? {
            return Ok(None);
        }
        let record: ReconstructionRecord = super::private_read(&path)?;
        if record.target_state != self.state.path
            || record.target_nonce != self.state.nonce
            || record.target_device != self.state.device
            || record.target_inode != self.state.inode
        {
            return Err(NativeServiceError::Refused(
                "reconstruction origin belongs to another target",
            ));
        }
        Ok(Some(record.observation()))
    }
}

/// A validated fresh live target plus the still-owned stopped original.
#[derive(Debug)]
pub struct ReconstructedCanonicalState {
    original: StoppedCanonicalState,
    target: NativeCanonicalService,
    intent: ReconstructionIntent,
}
impl ReconstructedCanonicalState {
    /// The original operation that produced this fresh replacement.
    pub fn intent(&self) -> &ReconstructionIntent {
        &self.intent
    }
    /// Live validated target. This contains no implied import of original authored inputs.
    pub fn target(&mut self) -> &mut NativeCanonicalService {
        &mut self.target
    }
    /// Original private state remains available for explicit input recovery or disposal.
    pub fn original(&self) -> &Path {
        self.original.state()
    }
    /// Transfer both owners. Disposing the old state remains a separate explicit operation.
    pub fn into_parts(self) -> (StoppedCanonicalState, NativeCanonicalService) {
        (self.original, self.target)
    }
    #[cfg(test)]
    pub(crate) fn suppress_ack_for_test(self) -> ReconstructionFailure {
        ReconstructionFailure {
            original: Box::new(self.original),
            target: Some(Box::new(self.target)),
            intent: Box::new(self.intent),
            source: Box::new(NativeServiceError::Refused(
                "successful reconstruction acknowledgement was lost; reconcile original owner",
            )),
        }
    }
}

/// Caller-held reconstruction custody, prepared synchronously before any cancellable work.
/// No async method consumes this owner; cancellation always returns control to its holder.
#[derive(Debug)]
pub struct ReconstructionController {
    original: StoppedCanonicalState,
    target: NativeCanonicalService,
    intent: ReconstructionIntent,
    validated: bool,
    #[cfg(test)]
    launch_pause: Option<(
        std::sync::Arc<tokio::sync::Notify>,
        std::sync::Arc<tokio::sync::Notify>,
    )>,
}
impl ReconstructionController {
    /// Original immutable operation selected before launch.
    pub fn intent(&self) -> &ReconstructionIntent {
        &self.intent
    }
    /// Private original remains preserved through every preparation/launch/recovery phase.
    pub fn original(&self) -> &Path {
        self.original.state()
    }
    /// Same prepared target for inspection or explicit physical settlement.
    pub fn target(&mut self) -> &mut NativeCanonicalService {
        &mut self.target
    }
    #[cfg(test)]
    pub(crate) fn pause_after_launch_for_test(
        &mut self,
    ) -> (
        std::sync::Arc<tokio::sync::Notify>,
        std::sync::Arc<tokio::sync::Notify>,
    ) {
        let hook = (
            std::sync::Arc::new(tokio::sync::Notify::new()),
            std::sync::Arc::new(tokio::sync::Notify::new()),
        );
        self.launch_pause = Some(hook.clone());
        hook
    }
    /// Launch the selected original target once and establish current readiness.
    /// # Errors
    /// Failure or cancellation retains both physical owners in this controller.
    pub async fn launch(&mut self, deadline: Instant) -> Result<(), NativeServiceError> {
        self.validated = false;
        validate_selection(&self.target, &self.original, &self.intent)?;
        self.target.launch(deadline).await?;
        #[cfg(test)]
        if let Some((entered, resume)) = &self.launch_pause {
            entered.notify_one();
            resume.notified().await;
        }
        validate_target(&self.target, &self.original, &self.intent)?;
        self.validated = true;
        Ok(())
    }
    /// Reconcile only the original target; never initialize or dispatch another service.
    /// # Errors
    /// Uncertain physical invocation or failed validation retains this controller.
    pub async fn reconcile(&mut self, deadline: Instant) -> Result<(), NativeServiceError> {
        self.validated = false;
        reconcile_target(&mut self.target, &self.original, &self.intent, deadline).await?;
        self.validated = true;
        Ok(())
    }
    /// Transfer validated readiness and both owners synchronously. A cancelled or
    /// failed async operation must be reconciled before this transfer succeeds.
    /// # Errors
    /// No completed validation, or current source/target identity differs.
    pub fn into_ready(self) -> Result<ReconstructedCanonicalState, ReconstructionFailure> {
        let checked = if self.validated {
            validate_target(&self.target, &self.original, &self.intent)
        } else {
            Err(NativeServiceError::Refused(
                "original reconstruction requires completed validation",
            ))
        };
        match checked {
            Ok(()) => Ok(ReconstructedCanonicalState {
                original: self.original,
                target: self.target,
                intent: self.intent,
            }),
            Err(source) => Err(ReconstructionFailure {
                original: Box::new(self.original),
                target: Some(Box::new(self.target)),
                intent: Box::new(self.intent),
                source: Box::new(source),
            }),
        }
    }
    /// Recover both actual owners for explicit physical settlement at any phase.
    pub fn into_parts(
        self,
    ) -> (
        StoppedCanonicalState,
        NativeCanonicalService,
        ReconstructionIntent,
    ) {
        (self.original, self.target, self.intent)
    }
}

/// Failure retains both original state and any partially launched replacement.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("fresh reconstruction remains unresolved: {source}")]
#[diagnostic(code(runtime::infrastructure))]
pub struct ReconstructionFailure {
    original: Box<StoppedCanonicalState>,
    target: Option<Box<NativeCanonicalService>>,
    intent: Box<ReconstructionIntent>,
    #[source]
    source: Box<NativeServiceError>,
}
impl ReconstructionFailure {
    /// Recover every original owner for explicit settlement, inspection or preservation.
    pub fn into_parts(
        self,
    ) -> (
        StoppedCanonicalState,
        Option<NativeCanonicalService>,
        ReconstructionIntent,
        NativeServiceError,
    ) {
        (
            *self.original,
            self.target.map(|target| *target),
            *self.intent,
            *self.source,
        )
    }
    /// Recover a prepared controller synchronously. Subsequent async work borrows it,
    /// so dropping a launch/reconciliation future never drops either physical owner.
    /// # Errors
    /// Preparation failed before any target was created; the original remains in this error.
    pub fn into_controller(mut self) -> Result<ReconstructionController, Self> {
        let Some(target) = self.target.take() else {
            return Err(self);
        };
        if let Err(source) = validate_selection(&target, &self.original, &self.intent) {
            self.target = Some(target);
            self.source = Box::new(source);
            return Err(self);
        }
        Ok(ReconstructionController {
            original: *self.original,
            target: *target,
            intent: *self.intent,
            validated: false,
            #[cfg(test)]
            launch_pause: None,
        })
    }
}

impl ReconstructionIntent {
    /// Original operation identity, reused for reconciliation rather than regenerated.
    pub fn operation(&self) -> &str {
        &self.operation
    }
    fn record_path(&self) -> PathBuf {
        self.original_state
            .join(format!("reconstruction-{}.json", self.operation))
    }
    fn validate(&self, original: &StoppedCanonicalState) -> Result<(), NativeServiceError> {
        original.service.state.verify()?;
        original.service.records.verify(original.state())?;
        if !uuid::Uuid::parse_str(&self.operation)
            .is_ok_and(|id| !id.is_nil() && id.to_string() == self.operation)
            || self.original_state != original.service.state.path
            || self.original_device != original.service.state.device
            || self.original_inode != original.service.state.inode
            || self.original_nonce != original.service.state.nonce
            || self.target_schema_digest != crate::generated::surreal::SCHEMA_DIGEST
            || self.target_module_sha256.len() != 64
            || !self
                .target_module_sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(NativeServiceError::Refused(
                "reconstruction differs from exact stopped original or current target",
            ));
        }
        Ok(())
    }
}

impl StoppedCanonicalState {
    /// Read an existing original selection without creating a target or probing a listener.
    /// # Errors
    /// Private request differs from the selected operation or authentic stopped source.
    pub fn reconstruction_status(
        &self,
        operation: uuid::Uuid,
    ) -> Result<Option<ReconstructionObservation>, NativeServiceError> {
        self.service.state.verify()?;
        self.service.records.verify(self.state())?;
        let path = self
            .state()
            .join(format!("reconstruction-{operation}.json"));
        if !path.try_exists()? {
            return Ok(None);
        }
        let record: ReconstructionRecord = super::private_read(&path)?;
        if record.intent.operation != operation.to_string()
            || record.intent.original_state != self.service.state.path
            || record.intent.original_device != self.service.state.device
            || record.intent.original_inode != self.service.state.inode
            || record.intent.original_nonce != self.service.state.nonce
        {
            return Err(NativeServiceError::Refused(
                "recorded reconstruction original selection differs",
            ));
        }
        Ok(Some(record.observation()))
    }
    /// Select a fresh empty canonical target while preserving this actual stopped original.
    /// No source files, authored models, result bodies or credentials are copied.
    /// # Errors
    /// Invalid operation, target artifact or original private ownership.
    pub fn empty_reconstruction_intent(
        &self,
        operation: uuid::Uuid,
        target_module_sha256: String,
    ) -> Result<ReconstructionIntent, NativeServiceError> {
        let intent = ReconstructionIntent {
            operation: operation.to_string(),
            original_state: self.service.state.path.clone(),
            original_device: self.service.state.device,
            original_inode: self.service.state.inode,
            original_nonce: self.service.state.nonce.clone(),
            target_schema_digest: crate::generated::surreal::SCHEMA_DIGEST.into(),
            target_module_sha256,
        };
        intent.validate(self)?;
        Ok(intent)
    }

    /// Prepare one new private target and persist original selection before async launch.
    /// The caller retains the returned controller throughout every cancellable operation.
    /// # Errors
    /// Every failure returns the original and any created/partially launched target.
    pub fn prepare_empty_reconstruction(
        self,
        intent: ReconstructionIntent,
        mut spec: NativeServiceSpec,
        deadline: Instant,
    ) -> Result<ReconstructionController, ReconstructionFailure> {
        let original = self;
        let mut target = None;
        let result = (|| {
            let source = &original;
            intent.validate(source)?;
            super::deadline_left(deadline)?;
            if intent.record_path().try_exists()? {
                return Err(NativeServiceError::Refused(
                    "original reconstruction already recorded; reconcile its original target rather than create another",
                ));
            }
            spec.state_parent = spec.state_parent.canonicalize()?;
            if spec.state_parent.starts_with(source.state()) {
                return Err(NativeServiceError::Refused(
                    "replacement cannot be placed inside the original state",
                ));
            }
            if spec.module_source.clone().into_canonical()?.sha256 != intent.target_module_sha256 {
                return Err(NativeServiceError::Refused(
                    "selected reconstruction module differs from frozen target",
                ));
            }
            target = Some(NativeCanonicalService::new(spec)?);
            let next = target
                .as_mut()
                .ok_or(NativeServiceError::Refused("replacement owner is absent"))?;
            let receipt = StudyModuleReceipt::read(next.state())?;
            if receipt.sha256 != intent.target_module_sha256 {
                return Err(NativeServiceError::Refused(
                    "replacement module differs from frozen target",
                ));
            }
            let record = ReconstructionRecord {
                intent: intent.clone(),
                target_state: next.state().into(),
                target_invocation: next.intent().unit().into(),
                target_device: next.state.device,
                target_inode: next.state.inode,
                target_nonce: next.state.nonce.clone(),
            };
            super::private_write(&intent.record_path(), &record)?;
            std::fs::File::open(source.state())?.sync_all()?;
            super::private_write(&next.state().join("reconstruction-origin.json"), &record)?;
            std::fs::File::open(next.state())?.sync_all()?;
            Ok::<(), NativeServiceError>(())
        })();
        match (result, target) {
            (Ok(()), Some(target)) => Ok(ReconstructionController {
                original,
                target,
                intent,
                validated: false,
                #[cfg(test)]
                launch_pause: None,
            }),
            (result, target) => Err(ReconstructionFailure {
                original: Box::new(original),
                target: target.map(Box::new),
                intent: Box::new(intent),
                source: Box::new(result.err().unwrap_or(NativeServiceError::Refused(
                    "replacement readiness was not established",
                ))),
            }),
        }
    }
}

fn validate_selection(
    target: &NativeCanonicalService,
    original: &StoppedCanonicalState,
    intent: &ReconstructionIntent,
) -> Result<(), NativeServiceError> {
    intent.validate(original)?;
    target.state.verify()?;
    target.records.verify(target.state())?;
    let record: ReconstructionRecord = super::private_read(&intent.record_path())?;
    if record.intent != *intent
        || record.target_state != target.state()
        || record.target_invocation != target.intent().unit()
        || record.target_device != target.state.device
        || record.target_inode != target.state.inode
        || record.target_nonce != target.state.nonce
        || target.state().starts_with(original.state())
        || StudyModuleReceipt::read(target.state())?.sha256 != intent.target_module_sha256
    {
        return Err(NativeServiceError::Refused(
            "original replacement identity differs",
        ));
    }
    if super::private_read::<ReconstructionRecord>(
        &target.state().join("reconstruction-origin.json"),
    )? != record
    {
        return Err(NativeServiceError::Refused(
            "reconstruction origin differs from original selection",
        ));
    }
    Ok(())
}
fn validate_target(
    target: &NativeCanonicalService,
    original: &StoppedCanonicalState,
    intent: &ReconstructionIntent,
) -> Result<(), NativeServiceError> {
    validate_selection(target, original, intent)?;
    let ready = target.ready()?;
    if original
        .service
        .ready
        .as_ref()
        .is_some_and(|old| old.fence().issuer() == ready.fence().issuer())
    {
        return Err(NativeServiceError::Refused(
            "fresh replacement reused original issuing authority",
        ));
    }
    Ok(())
}

async fn reconcile_target(
    target: &mut NativeCanonicalService,
    original: &StoppedCanonicalState,
    intent: &ReconstructionIntent,
    deadline: Instant,
) -> Result<(), NativeServiceError> {
    validate_selection(target, original, intent)?;
    if target.unit.is_none() && target.launch_dispatched {
        let manager = target.manager.as_ref().ok_or(NativeServiceError::Refused(
            "original reconstruction manager is unavailable",
        ))?;
        target.unit = manager
            .reconcile_if_present(&target.intent, deadline)
            .await?;
    }
    target.wait_owned_listener(deadline).await?;
    let options = CanonicalOptions::from_state(target.state())?;
    let ready =
        crate::transport::operation_scope(deadline, CanonicalStore::connect_ready(&options))
            .await?;
    target.connections.push(ready.store().clone());
    target.ready = Some(ready);
    target.initialized = true;
    validate_target(target, original, intent)
}
