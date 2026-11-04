// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Retained native study admission, original canonical creation and protected passive consumers.

use super::*;
use crate::host_operations::engineering_contract::{
    EngineeringCohortRelease, EngineeringFreezeStudyCohort, EngineeringObserveStudy,
    EngineeringPrepareStudy, EngineeringPreparedStudy, EngineeringReadStudyResult,
    EngineeringRestoreStudy, EngineeringStudyCohortPage, EngineeringStudyHeaderPage,
    EngineeringStudyKey, EngineeringStudyResultPage, EngineeringStudyStatus,
    EngineeringSubmitStudy,
};
use crate::workflow::{
    FrozenStudyCohort, IssuedStudyIdentity, StudyCreationRequest, StudyDefinition,
};

#[derive(Debug)]
pub(super) struct PreparedStudy {
    pub(super) definition: StudyDefinition,
    selection: EngineeringPreparedStudy,
    _owner: Arc<dyn pse_columnar::PayloadOwner>,
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "finite original custody controls require valid local setup"
    )]
    use super::*;

    fn service() -> Arc<EngineeringService> {
        let runtime = crate::workflow::tests::runtime();
        let shared = runtime.shared.clone();
        EngineeringService::new(runtime, shared, 8).unwrap()
    }

    #[tokio::test]
    async fn engineering_study_capture_release_cannot_be_reopened_by_late_publication() {
        let service = service();
        let key = EngineeringOperationKey {
            operation: "original-capture".into(),
        };
        let (original, fresh) = service
            .issue(&key.operation, Kind::StudyCohort, &key)
            .unwrap();
        assert!(fresh);
        assert!(
            service
                .release_study_cohort(&key)
                .unwrap()
                .local_admission_closed
        );
        // The same publication boundary handles native completion after a pending capture
        // is released. Late outcomes are dropped instead of reopening local read admission.
        original.publish(Outcome::Retained).unwrap();
        assert!(matches!(
            original.completed().await.unwrap(),
            Outcome::Released
        ));
        assert!(
            service
                .release_study_cohort(&key)
                .unwrap()
                .local_admission_closed
        );
        assert!(service.study_cohort(&original).await.is_err());
        service.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn engineering_study_frontend_shutdown_joins_creation_without_requesting_study_cancel() {
        let service = service();
        let key = EngineeringOperationKey {
            operation: "original-study".into(),
        };
        let (original, fresh) = service.issue(&key.operation, Kind::Study, &key).unwrap();
        assert!(fresh);
        let (started, entered) = tokio::sync::oneshot::channel();
        let original_task = original.clone();
        let weak = Arc::downgrade(&service);
        service
            .spawn(original.clone(), async move {
                started
                    .send(())
                    .map_err(|_| infrastructure("creation entry observer disappeared"))?;
                loop {
                    let service = weak
                        .upgrade()
                        .ok_or_else(|| infrastructure("original service disappeared"))?;
                    if service.closing.load(Ordering::Acquire) {
                        break;
                    }
                    tokio::task::yield_now().await;
                }
                original_task.cancel.checkpoint().map_err(owner)?;
                Ok(Outcome::Retained)
            })
            .unwrap();
        entered.await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(10), service.shutdown())
            .await
            .unwrap()
            .unwrap();
        assert!(!original.cancel.token().is_cancelled());
        assert!(matches!(
            original.completed().await.unwrap(),
            Outcome::Retained
        ));
    }
}

/// Native response and its actual allocation/protection owner, retained through wire encoding.
#[derive(Debug)]
pub struct EngineeringStudyOutput<T> {
    /// Actual source-owned native response.
    pub value: T,
    /// Original retained native and encoding allocations.
    pub owner: Arc<dyn pse_columnar::PayloadOwner>,
    /// Complete encoded response ceiling, never a truncation request.
    pub maximum_bytes: usize,
}

impl EngineeringService {
    /// Read actual compact original study headers without scientific construction or dispatch.
    /// # Errors
    /// Original issuer/cursor, finite page, canonical read and pool failures propagate.
    pub async fn browse_studies(
        &self,
        input: crate::workflow::StudyWorkbenchRequest,
    ) -> Result<EngineeringStudyOutput<crate::workflow::StudyWorkbenchPage>, EngineeringServiceError>
    {
        input.validate_projection().map_err(owner)?;
        let reservation = Arc::new(
            datafusion::execution::memory_pool::MemoryConsumer::new(
                "engineering:study-workbench-response",
            )
            .register(&self.shared.pool()),
        );
        reservation
            .try_grow(
                pse_operations::canonical::MetadataResponse::Page
                    .allocation_bytes()
                    .checked_add(4 * (16 << 20))
                    .ok_or_else(|| resource("workbench response extent overflow"))?,
            )
            .map_err(owner)?;
        let value = pse_operations::canonical::response_ownership_scope(
            reservation.clone(),
            self.runtime
                .canonical_store()
                .map_err(owner)?
                .control_study_workbench_page(input),
        )
        .await
        .map_err(owner)?;
        Ok(EngineeringStudyOutput {
            value,
            owner: reservation,
            maximum_bytes: 16 << 20,
        })
    }

    /// Admit the full actual StudyRequest vocabulary against one exact native source.
    /// # Errors
    /// Original admission, source/context, finite resource and conflicting-request failures propagate.
    pub async fn prepare_study(
        self: &Arc<Self>,
        input: EngineeringPrepareStudy,
    ) -> Result<EngineeringPreparedStudy, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        if input.operation == input.source_operation {
            return Err(request(
                "study preparation cannot depend on its own operation",
            ));
        }
        let source = self.operation(&input.source_operation, Kind::Source)?;
        let extent = crate::workflow::bounded_study_document_extent(&input.request, 16 << 20)
            .map_err(owner)?;
        let reservation = self
            .shared
            .math()
            .reserve(
                "engineering:study-preparation-request",
                extent
                    .checked_mul(8)
                    .ok_or_else(|| resource("study request extent overflow"))?,
            )
            .map_err(owner)?;
        let (operation, fresh) = self.issue(&input.operation, Kind::StudyPreparation, &input)?;
        if fresh {
            let service = self.clone();
            let original = operation.clone();
            self.spawn(operation.clone(), async move {
                let Outcome::Source { package, selection } =
                    source.dependency(&original.cancel).await?
                else {
                    return Err(request("study source admission is unavailable"));
                };
                if package.revision_id() != input.expected_source {
                    return Err(request(
                        "study preparation names another exact source parent",
                    ));
                }
                let admitted = package
                    .admit_study(
                        &service.runtime.execution_services(),
                        &input.request.inputs,
                        &input.request.output,
                        &original.cancel,
                    )
                    .await
                    .map_err(owner)?;
                let definition = service
                    .runtime
                    .record_study(&admitted, &original.cancel)
                    .await
                    .map_err(owner)?;
                let identity =
                    pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &definition)
                        .map_err(owner)?;
                let extent = crate::workflow::bounded_study_document_extent(&definition, 128 << 20)
                    .map_err(owner)?;
                let retained = service
                    .shared
                    .math()
                    .reserve(
                        "engineering:prepared-study",
                        extent
                            .checked_mul(4)
                            .ok_or_else(|| resource("prepared study extent overflow"))?,
                    )
                    .map_err(owner)?;
                let selection = EngineeringPreparedStudy {
                    operation: input.operation,
                    source: Some(selection),
                    preparation: identity,
                    occurrences: u64::try_from(definition.point_count()).map_err(owner)?,
                };
                Ok(Outcome::PreparedStudy(Arc::new(PreparedStudy {
                    definition,
                    selection,
                    _owner: Arc::new((reservation, retained)),
                })))
            })?;
        }
        match operation.completed().await? {
            Outcome::PreparedStudy(prepared) => Ok(prepared.selection.clone()),
            Outcome::Failed(error) => Err(EngineeringServiceError::Original(error)),
            _ => Err(infrastructure(
                "study preparation published an incompatible outcome",
            )),
        }
    }

    /// Return the complete actual portable definition for exact original recovery.
    /// Complete responses exceeding the declared wire allowance refuse before cloning.
    /// # Errors
    /// Missing admission, changed operation kind or resource/extent refusal propagates.
    pub async fn prepared_study_definition(
        &self,
        key: &EngineeringOperationKey,
    ) -> Result<EngineeringStudyOutput<StudyDefinition>, EngineeringServiceError> {
        key.validate_projection().map_err(owner)?;
        let operation = self.operation(&key.operation, Kind::StudyPreparation)?;
        let Outcome::PreparedStudy(prepared) = operation.completed().await? else {
            return Err(request("original native study preparation is unavailable"));
        };
        let extent = crate::workflow::bounded_study_document_extent(&prepared.definition, 16 << 20)
            .map_err(owner)?;
        let encoding = self
            .shared
            .math()
            .reserve(
                "engineering:portable-study-definition",
                extent
                    .checked_mul(4)
                    .and_then(|value| value.checked_add(4096))
                    .ok_or_else(|| resource("portable study response extent overflow"))?,
            )
            .map_err(owner)?;
        Ok(EngineeringStudyOutput {
            value: prepared.definition.clone(),
            owner: Arc::new((prepared, encoding)),
            maximum_bytes: 16 << 20,
        })
    }

    /// Retain exact original portable science for native interrupted-creation reconciliation.
    /// This is retention, not proof of execution eligibility: native start rechecks the complete
    /// canonical source, physical context, bindings, numerical policy and seed prerequisites.
    /// # Errors
    /// Changed immutable identity, invalid native roles or resource refusal propagates.
    pub async fn restore_study(
        self: &Arc<Self>,
        input: EngineeringRestoreStudy,
    ) -> Result<EngineeringPreparedStudy, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        input.definition.validate_roles().map_err(owner)?;
        let identity =
            pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &input.definition)
                .map_err(owner)?;
        if identity != input.expected_preparation {
            return Err(request(
                "portable study differs from the exact original immutable preparation",
            ));
        }
        let extent = crate::workflow::bounded_study_document_extent(&input.definition, 128 << 20)
            .map_err(owner)?;
        let reservation = self
            .shared
            .math()
            .reserve(
                "engineering:restored-study",
                extent
                    .checked_mul(4)
                    .ok_or_else(|| resource("restored study extent overflow"))?,
            )
            .map_err(owner)?;
        let (operation, fresh) = self.issue(&input.operation, Kind::StudyPreparation, &input)?;
        if fresh {
            operation.publish(Outcome::PreparedStudy(Arc::new(PreparedStudy {
                selection: EngineeringPreparedStudy {
                    operation: input.operation,
                    source: None,
                    preparation: identity,
                    occurrences: u64::try_from(input.definition.point_count()).map_err(owner)?,
                },
                definition: input.definition,
                _owner: reservation,
            })))?;
        }
        match operation.completed().await? {
            Outcome::PreparedStudy(prepared) => Ok(prepared.selection.clone()),
            Outcome::Failed(error) => Err(EngineeringServiceError::Original(error)),
            _ => Err(infrastructure(
                "study restoration published an incompatible outcome",
            )),
        }
    }

    /// Submit the exact native creation owner, retaining deterministic original coordinates.
    /// # Errors
    /// Changed original envelopes/science or unavailable exact native preparations refuse.
    pub async fn submit_study(
        self: &Arc<Self>,
        input: EngineeringSubmitStudy,
    ) -> Result<EngineeringStudyOutput<EngineeringStudyStatus>, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        let key = EngineeringStudyKey {
            operation: input.operation.clone(),
            study: input.study,
        };
        if let Some((_, original, summary)) = self.original_study(&key).await? {
            let issued = original
                .issued
                .as_ref()
                .ok_or_else(|| request("native study has no caller issuance"))?;
            if issued.submission != fingerprint(&input)?
                || original.definition != input.expected_preparation
            {
                return Err(request(
                    "original canonical study belongs to another complete submit envelope",
                ));
            }
            if summary
                .as_ref()
                .is_some_and(|header| header.active || header.terminal || header.cancelled)
            {
                return self.study_status(&key).await;
            }
            // An acknowledged parent without complete activation is an original creation prefix.
            // Reconciliation below needs the exact original definition; absence never starts new science.
        }
        let preparation = self.operation(&input.preparation_operation, Kind::StudyPreparation)?;
        let (operation, fresh) = self.issue(&input.operation, Kind::Study, &input)?;
        let issued = IssuedStudyIdentity {
            study: input.study,
            operation: input.operation.clone(),
            submission: operation.fingerprint,
        };
        {
            let mut retained = operation
                .issued_study
                .lock()
                .map_err(|_| infrastructure("issued study identity lock poisoned"))?;
            if retained
                .as_ref()
                .is_some_and(|original| original != &issued)
            {
                return Err(request(
                    "original operation has another issued study identity",
                ));
            }
            *retained = Some(issued.clone());
        }
        if fresh {
            let service = self.clone();
            let original = operation.clone();
            self.spawn(operation.clone(), async move {
                let Outcome::PreparedStudy(prepared) =
                    preparation.dependency(&original.cancel).await?
                else {
                    return Err(request("native study preparation is unavailable"));
                };
                if prepared.selection.preparation != input.expected_preparation {
                    return Err(request(
                        "exact immutable study preparation identity differs",
                    ));
                }
                let handle = service
                    .runtime
                    .start_issued_study(prepared.definition.clone(), issued, &original.cancel)
                    .await
                    .map_err(owner)?;
                Ok(Outcome::Study(handle))
            })?;
        }
        operation.completed().await?;
        self.study_status(&key).await
    }

    async fn original_study(
        &self,
        key: &EngineeringStudyKey,
    ) -> Result<
        Option<(
            Arc<datafusion::execution::memory_pool::MemoryReservation>,
            StudyCreationRequest,
            Option<pse_operations::canonical_studies::StudySummary>,
        )>,
        EngineeringServiceError,
    > {
        key.validate_projection().map_err(owner)?;
        let reservation = Arc::new(
            datafusion::execution::memory_pool::MemoryConsumer::new(
                "engineering:original-study-header",
            )
            .register(&self.shared.pool()),
        );
        reservation
            .try_grow(2 * pse_operations::canonical::MetadataResponse::Header.allocation_bytes())
            .map_err(owner)?;
        let store = self.runtime.canonical_store().map_err(owner)?;
        let Some(parent) = pse_operations::canonical::response_ownership_scope(
            reservation.clone(),
            store.canonical_run(&format!(
                "run:{}",
                IssuedStudyIdentity::parent_run_for(key.study)
            )),
        )
        .await
        .map_err(owner)?
        else {
            return Ok(None);
        };
        let original = StudyCreationRequest::read(parent.request.as_slice()).map_err(owner)?;
        if original.study != key.study
            || original
                .issued
                .as_ref()
                .is_none_or(|issued| issued.study != key.study || issued.operation != key.operation)
        {
            return Err(request(
                "canonical study belongs to another original issued request",
            ));
        }
        let summary = pse_operations::canonical::response_ownership_scope(
            reservation.clone(),
            store.canonical_study_summary(&key.study.to_string()),
        )
        .await
        .map_err(owner)?;
        if summary
            .as_ref()
            .is_some_and(|header| header.run != parent.key)
        {
            return Err(request(
                "canonical study parent differs from its original creation request",
            ));
        }
        Ok(Some((reservation, original, summary)))
    }

    /// Read only the original compact canonical header, including incomplete creation knowledge.
    /// # Errors
    /// Absent original registration or a foreign operation/study pairing refuses.
    pub async fn study_status(
        &self,
        key: &EngineeringStudyKey,
    ) -> Result<EngineeringStudyOutput<EngineeringStudyStatus>, EngineeringServiceError> {
        key.validate_projection().map_err(owner)?;
        let maximum_bytes = 64 << 10;
        let encoding = self
            .shared
            .math()
            .reserve("engineering:study-status-encoding", 4 * maximum_bytes)
            .map_err(owner)?;
        let mut local_known = false;
        let failure = match self.operation(&key.operation, Kind::Study) {
            Ok(operation) => match operation.snapshot()? {
                Outcome::Study(handle) if handle.study_id() != key.study => {
                    return Err(request(
                        "original operation has another native study identity",
                    ));
                }
                outcome => {
                    let retained = operation
                        .issued_study
                        .lock()
                        .map_err(|_| infrastructure("issued study identity lock poisoned"))?;
                    if retained.as_ref().is_none_or(|issued| {
                        issued.study != key.study || issued.operation != key.operation
                    }) {
                        return Err(request(
                            "original operation has another issued study identity",
                        ));
                    }
                    local_known = true;
                    if let Outcome::Failed(error) = outcome {
                        Some(failure(error.as_ref()))
                    } else {
                        None
                    }
                }
            },
            Err(EngineeringServiceError::Request(_)) => None,
            Err(error) => return Err(error),
        };
        let (reservation, canonical) = match self.original_study(key).await? {
            Some((reservation, _, canonical)) => (reservation, canonical),
            None if local_known => {
                let reservation = Arc::new(
                    datafusion::execution::memory_pool::MemoryConsumer::new(
                        "engineering:pending-study-status",
                    )
                    .register(&self.shared.pool()),
                );
                reservation.try_grow(4096).map_err(owner)?;
                (reservation, None)
            }
            None => {
                return Err(request(
                    "original canonical study registration is unavailable; absence does not establish no effect",
                ));
            }
        };
        Ok(EngineeringStudyOutput {
            value: EngineeringStudyStatus {
                operation: key.operation.clone(),
                study: key.study,
                canonical,
                failure,
            },
            owner: Arc::new((reservation, encoding)),
            maximum_bytes,
        })
    }

    /// Cooperatively cancel the exact original canonical study without asserting settlement.
    /// # Errors
    /// Foreign identity, missing original parent or original cancellation refusal propagates.
    pub async fn cancel_study(
        &self,
        key: &EngineeringStudyKey,
    ) -> Result<EngineeringStudyOutput<EngineeringStudyStatus>, EngineeringServiceError> {
        self.study_status(key).await?;
        match self.operation(&key.operation, Kind::Study) {
            Ok(operation) => {
                operation.cancel.cancel();
                // The original creation owner settles issued canonical effects before returning.
                // A lost ACK is not permission to race a second cancellation/creation owner.
                operation.completed().await?;
            }
            Err(EngineeringServiceError::Request(_)) => {}
            Err(error) => return Err(error),
        }
        if self.study_status(key).await?.value.canonical.is_some() {
            self.runtime
                .study(key.study)
                .cancel()
                .await
                .map_err(owner)?;
        }
        self.study_status(key).await
    }

    /// Open the existing bounded passive observer, preserving all original cursor checks.
    /// # Errors
    /// A changed study selection, source context, cursor or native capture refusal propagates.
    pub async fn observe_study(
        &self,
        input: EngineeringObserveStudy,
    ) -> Result<crate::workflow::StudyObserver, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        self.study_status(&input.original).await?;
        if input.request.study != input.original.study.to_string() {
            return Err(request("study observer selects another original study"));
        }
        self.runtime
            .observe_study(input.request, input.cursor)
            .await
            .map_err(owner)
    }

    /// Consume one actual native observer page with its original cursor/capture custody.
    /// # Errors
    /// Original capture/history/continuity and the observer's own finite clock/resource failures
    /// remain distinct from scientific execution and never cancel the study.
    pub async fn study_observation_page(
        &self,
        observer: &mut crate::workflow::StudyObserver,
        follow: bool,
        cancel: &CancelSource,
    ) -> Result<
        EngineeringStudyOutput<crate::workflow::StudyObservationPage>,
        EngineeringServiceError,
    > {
        cancel.checkpoint().map_err(owner)?;
        let encoding = self
            .shared
            .math()
            .reserve("engineering:study-observation-encoding", 4 * (4 << 20))
            .map_err(owner)?;
        let value = if follow {
            observer.follow(std::time::Duration::from_millis(100)).await
        } else {
            observer.poll().await
        }
        .map_err(owner)?;
        cancel.checkpoint().map_err(owner)?;
        let native = value.payload_owner();
        Ok(EngineeringStudyOutput {
            value,
            owner: Arc::new((native, encoding)),
            maximum_bytes: 4 << 20,
        })
    }

    /// Retain the actual native frozen capture rather than granting authority from its receipt.
    /// # Errors
    /// Incomplete cuts, overlarge populations and original capture failures refuse.
    pub async fn freeze_study_cohort(
        self: &Arc<Self>,
        input: EngineeringFreezeStudyCohort,
    ) -> Result<EngineeringStudyOutput<crate::workflow::StudyCohortReceipt>, EngineeringServiceError>
    {
        input.validate_projection().map_err(owner)?;
        let maximum_bytes = 64 << 10;
        let encoding = self
            .shared
            .math()
            .reserve(
                "engineering:study-capture-acknowledgment",
                4 * maximum_bytes,
            )
            .map_err(owner)?;
        let (operation, fresh) = self.issue(&input.operation, Kind::StudyCohort, &input)?;
        if fresh {
            let service = self.clone();
            self.spawn(operation.clone(), async move {
                let cohort = service
                    .runtime
                    .freeze_study_cohort(input.request)
                    .await
                    .map_err(owner)?;
                Ok(Outcome::StudyCohort(Arc::new(cohort)))
            })?;
        }
        let cohort = self.study_cohort(&operation).await?;
        Ok(EngineeringStudyOutput {
            value: cohort.receipt().clone(),
            owner: Arc::new((cohort, encoding)),
            maximum_bytes,
        })
    }

    async fn study_cohort(
        &self,
        operation: &Operation,
    ) -> Result<Arc<FrozenStudyCohort>, EngineeringServiceError> {
        match operation.completed().await? {
            Outcome::StudyCohort(cohort) => Ok(cohort),
            Outcome::Released => Err(request("original study capture local admission is closed")),
            Outcome::Failed(error) => Err(EngineeringServiceError::Original(error)),
            _ => Err(infrastructure(
                "cohort operation published an incompatible outcome",
            )),
        }
    }

    /// Close local capture admission without claiming physical cleanup or revoking escaped reads.
    /// The actual native capture owner remains with every in-flight/encoded response until release.
    /// # Errors
    /// Missing/foreign operation identity or poisoned custody locks refuse.
    pub fn release_study_cohort(
        &self,
        input: &EngineeringOperationKey,
    ) -> Result<EngineeringCohortRelease, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        let operation = self.operation(&input.operation, Kind::StudyCohort)?;
        let mut outcome = operation
            .outcome
            .lock()
            .map_err(|_| infrastructure("cohort custody lock poisoned"))?;
        *outcome = Outcome::Released;
        operation.cancel.cancel();
        operation.changed.notify_waiters();
        Ok(EngineeringCohortRelease {
            operation: input.operation.clone(),
            local_admission_closed: true,
        })
    }

    /// Read original frozen header facts with their protection/allocation through encoding.
    /// # Errors
    /// A foreign cursor, absent live capability or native finite-page refusal propagates.
    pub async fn study_cohort_headers(
        &self,
        input: EngineeringStudyHeaderPage,
    ) -> Result<
        EngineeringStudyOutput<crate::workflow::StudyCohortHeaderPage>,
        EngineeringServiceError,
    > {
        input.validate_projection().map_err(owner)?;
        let operation = self.operation(&input.operation, Kind::StudyCohort)?;
        let cohort = self.study_cohort(&operation).await?;
        let maximum_bytes = 1 << 20;
        let encoding = self
            .shared
            .math()
            .reserve("engineering:study-header-encoding", 4 * maximum_bytes)
            .map_err(owner)?;
        let value = cohort.header_page(input.request).await.map_err(owner)?;
        let page_owner = value.payload_owner();
        Ok(EngineeringStudyOutput {
            value,
            owner: Arc::new((cohort, page_owner, encoding)),
            maximum_bytes,
        })
    }

    /// Read every original sibling classification, including unavailable and failed members.
    /// # Errors
    /// A foreign cursor, absent live capability or native finite-page refusal propagates.
    pub async fn study_cohort_members(
        &self,
        input: EngineeringStudyCohortPage,
    ) -> Result<
        EngineeringStudyOutput<crate::workflow::StudyCohortMemberPage>,
        EngineeringServiceError,
    > {
        input.validate_projection().map_err(owner)?;
        let operation = self.operation(&input.operation, Kind::StudyCohort)?;
        let cohort = self.study_cohort(&operation).await?;
        let maximum_bytes = 4 << 20;
        let encoding = self
            .shared
            .math()
            .reserve("engineering:study-member-encoding", 4 * maximum_bytes)
            .map_err(owner)?;
        let value = cohort.member_page(input.request).await.map_err(owner)?;
        let page_owner = value.payload_owner();
        Ok(EngineeringStudyOutput {
            value,
            owner: Arc::new((cohort, page_owner, encoding)),
            maximum_bytes,
        })
    }

    /// Read an exact original captured manifest through the native protected result reader.
    /// # Errors
    /// Unavailable siblings, changed manifests, partial output, cancellation and resource refusal
    /// remain original failures. No unavailable member is silently omitted from the cohort.
    pub async fn read_study_result(
        &self,
        input: EngineeringReadStudyResult,
        cancel: &CancelSource,
    ) -> Result<EngineeringStudyOutput<EngineeringStudyResultPage>, EngineeringServiceError> {
        use datafusion::arrow::ipc::writer::StreamWriter;
        input.validate_projection().map_err(owner)?;
        if input.end < input.start || input.end - input.start > 4096 {
            return Err(request(
                "selected original study result admits at most 4096 complete rows",
            ));
        }
        cancel.checkpoint().map_err(owner)?;
        let operation = self.operation(&input.operation, Kind::StudyCohort)?;
        let cohort = self.study_cohort(&operation).await?;
        let reservation = Arc::new(
            datafusion::execution::memory_pool::MemoryConsumer::new(
                "engineering:selected-study-result-response",
            )
            .register(&self.shared.pool()),
        );
        let base = input
            .maximum_bytes
            .checked_mul(2)
            .and_then(|value| value.checked_add(4096))
            .ok_or_else(|| resource("selected study response extent overflow"))?;
        reservation.try_grow(base).map_err(owner)?;
        let mut reader = cohort
            .result(
                input.position,
                &input.relation,
                input.start,
                input.end,
                cancel.token(),
            )
            .await
            .map_err(owner)?;
        let run = reader.selection().run().key.clone();
        let attempt = reader.selection().attempt().key.clone();
        let manifest = reader.selection().manifest().key.clone();
        let mut sink = pse_ids::BoundedSink::new(Vec::new(), input.maximum_bytes);
        let mut rows = 0u64;
        {
            let mut writer =
                StreamWriter::try_new(&mut sink, reader.schema().as_ref()).map_err(owner)?;
            while let Some(batch) = reader.next_batch().await.map_err(owner)? {
                cancel.checkpoint().map_err(owner)?;
                let working = pse_columnar::ipc_file::writer_extent(
                    std::slice::from_ref(&batch),
                    input.maximum_bytes,
                )
                .map_err(owner)?;
                let extent = base
                    .checked_add(working)
                    .ok_or_else(|| resource("study writer extent overflow"))?;
                reservation
                    .try_grow(extent.saturating_sub(reservation.size()))
                    .map_err(owner)?;
                writer.write(&batch).map_err(owner)?;
                rows = rows
                    .checked_add(u64::try_from(batch.num_rows()).map_err(owner)?)
                    .ok_or_else(|| resource("study selected row count overflow"))?;
            }
            cancel.checkpoint().map_err(owner)?;
            writer.finish().map_err(owner)?;
        }
        let capture = cohort.receipt().capture.clone();
        Ok(EngineeringStudyOutput {
            value: EngineeringStudyResultPage {
                capture,
                position: input.position,
                run,
                attempt,
                manifest,
                relation: input.relation,
                start: input.start,
                end: input.end,
                rows,
                ipc: sink.into_inner(),
            },
            owner: Arc::new((cohort, reservation)),
            maximum_bytes: input.maximum_bytes,
        })
    }
}
