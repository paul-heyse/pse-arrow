// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated native adapters and original bounded study observation stream.

use super::*;
use crate::host_operations::engineering_contract::EngineeringObserveStudy;
use datafusion::execution::memory_pool::MemoryReservation;

// These expand only typed generated service callbacks. The composed source graph
// remains the sole method/effect contract; no scientific fields are mapped here.
macro_rules! owned_unary {
    ($method:ident, $input:ident, $output:ident, $decode:ident, $native:ident, $encode:ident) => {
        async fn $method<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::$input>,
        ) -> connectrpc::ServiceResult<impl connectrpc::Encodable<pb::$output> + Send + use<'a>> {
            self.authenticate(&context)?;
            let request = request.to_owned_message();
            let input =
                crate::engineering_projection::$decode(request).map_err(study::projection_error)?;
            let output = self.engine.$native(input).await.map_err(transport)?;
            Ok(connectrpc::Response::new(study::accounted(
                output,
                crate::engineering_projection::$encode,
            )?))
        }
    };
}
pub(super) use owned_unary;

macro_rules! admission_unary {
    ($method:ident, $input:ident, $decode:ident, $native:ident) => {
        async fn $method<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::$input>,
        ) -> connectrpc::ServiceResult<
            impl connectrpc::Encodable<pb::EngineeringPreparedStudy> + Send + use<'a>,
        > {
            self.authenticate(&context)?;
            let request = request.to_owned_message();
            let input =
                crate::engineering_projection::$decode(request).map_err(study::projection_error)?;
            let output = self.engine.$native(input).await.map_err(transport)?;
            let message = crate::engineering_projection::encode_engineering_prepared_study(output)
                .map_err(study::projection_error)?;
            Ok(connectrpc::Response::new(outgoing::Validated(message)))
        }
    };
}
macro_rules! original_unary {
    ($method:ident, $native:ident) => {
        async fn $method<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringStudyKey>,
        ) -> connectrpc::ServiceResult<
            impl connectrpc::Encodable<pb::EngineeringStudyStatus> + Send + use<'a>,
        > {
            self.authenticate(&context)?;
            let request = request.to_owned_message();
            let input = crate::engineering_projection::decode_engineering_study_key(request)
                .map_err(study::projection_error)?;
            let output = self.engine.$native(&input).await.map_err(transport)?;
            Ok(connectrpc::Response::new(study::accounted(
                output,
                crate::engineering_projection::encode_engineering_study_status,
            )?))
        }
    };
}
pub(super) use original_unary;

macro_rules! read_preparation {
    ($method:ident, $output:ident, $encode:ident) => {
        async fn $method<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringOperationKey>,
        ) -> connectrpc::ServiceResult<impl connectrpc::Encodable<pb::$output> + Send + use<'a>> {
            self.authenticate(&context)?;
            let input = crate::engineering_projection::decode_engineering_operation_key(
                request.to_owned_message(),
            )
            .map_err(study::projection_error)?;
            let output = self.engine.$method(&input).await.map_err(transport)?;
            Ok(connectrpc::Response::new(study::accounted(
                output,
                crate::engineering_projection::$encode,
            )?))
        }
    };
}

impl bindings::ExplicitStudyPreparationV2 for Implementation {
    admission_unary!(
        prepare_explicit_study,
        EngineeringPrepareExplicitStudy,
        decode_engineering_prepare_explicit_study,
        prepare_explicit_study
    );
    admission_unary!(
        restore_explicit_study,
        EngineeringRestoreExplicitStudy,
        decode_engineering_restore_explicit_study,
        restore_explicit_study
    );
    read_preparation!(
        read_prepared_explicit_study,
        ExplicitStudyDefinition,
        encode_explicit_study_definition
    );
}

impl bindings::ExpandedStudyPreparationV2 for Implementation {
    admission_unary!(
        prepare_expanded_study,
        EngineeringPrepareExpandedStudy,
        decode_engineering_prepare_expanded_study,
        prepare_expanded_study
    );
    admission_unary!(
        restore_expanded_study,
        EngineeringRestoreExpandedStudy,
        decode_engineering_restore_expanded_study,
        restore_expanded_study
    );
    read_preparation!(
        read_prepared_expanded_study,
        ExpandedStudyDefinition,
        encode_expanded_study_definition
    );
}

macro_rules! control_methods {
    () => {
        study::owned_unary!(
            browse_studies,
            StudyWorkbenchRequest,
            StudyWorkbenchPage,
            decode_study_workbench_request,
            browse_studies,
            encode_study_workbench_page
        );
        study::owned_unary!(
            submit_study,
            EngineeringSubmitStudy,
            EngineeringStudyStatus,
            decode_engineering_submit_study,
            submit_study,
            encode_engineering_study_status
        );
        study::original_unary!(study_status, study_status);
        study::original_unary!(cancel_study, cancel_study);
        study::owned_unary!(
            freeze_study_cohort,
            EngineeringFreezeStudyCohort,
            StudyCohortReceipt,
            decode_engineering_freeze_study_cohort,
            freeze_study_cohort,
            encode_study_cohort_receipt
        );
        study::owned_unary!(
            read_study_headers,
            EngineeringStudyHeaderPage,
            StudyCohortHeaderPage,
            decode_engineering_study_header_page,
            study_cohort_headers,
            encode_study_cohort_header_page
        );
        study::owned_unary!(
            read_study_members,
            EngineeringStudyCohortPage,
            StudyCohortMemberPage,
            decode_engineering_study_cohort_page,
            study_cohort_members,
            encode_study_cohort_member_page
        );

        async fn observe_study(
            &self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringObserveStudy>,
        ) -> connectrpc::ServiceResult<
            ServiceStream<impl connectrpc::Encodable<pb::StudyObservationPage> + Send + use<>>,
        > {
            self.authenticate(&context)?;
            let owner = context
                .extensions()
                .get::<ingress::RequestOwner>()
                .ok_or_else(|| ConnectError::internal("engineering ingress custody missing"))?;
            let request = request.to_owned_message();
            let input = crate::engineering_projection::decode_engineering_observe_study(request)
                .map_err(study::projection_error)?;
            let stream = study::observations(
                self.engine.clone(),
                input,
                owner.0.clone(),
                context.deadline(),
            );
            Ok(connectrpc::Response::new(stream))
        }

        async fn read_study_result<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringReadStudyResult>,
        ) -> connectrpc::ServiceResult<
            impl connectrpc::Encodable<pb::EngineeringStudyResultPage> + Send + use<'a>,
        > {
            self.authenticate(&context)?;
            let request = request.to_owned_message();
            let input =
                crate::engineering_projection::decode_engineering_read_study_result(request)
                    .map_err(study::projection_error)?;
            let cancel = CancelSource::new().with_deadline(context.deadline());
            let output = self
                .engine
                .read_study_result(input, &cancel)
                .await
                .map_err(transport)?;
            Ok(connectrpc::Response::new(study::accounted(
                output,
                crate::engineering_projection::encode_engineering_study_result_page,
            )?))
        }

        async fn release_study_cohort<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringOperationKey>,
        ) -> connectrpc::ServiceResult<
            impl connectrpc::Encodable<pb::EngineeringCohortRelease> + Send + use<'a>,
        > {
            self.authenticate(&context)?;
            let request = request.to_owned_message();
            let input = crate::engineering_projection::decode_engineering_operation_key(request)
                .map_err(study::projection_error)?;
            let output = self
                .engine
                .release_study_cohort(&input)
                .map_err(transport)?;
            let message = crate::engineering_projection::encode_engineering_cohort_release(output)
                .map_err(study::projection_error)?;
            Ok(connectrpc::Response::new(outgoing::Validated(message)))
        }
    };
}
pub(super) use control_methods;

pub(super) fn accounted<N, M>(
    output: crate::host_operations::engineering::EngineeringStudyOutput<N>,
    encode: impl FnOnce(N) -> Result<M, pse_contract_proto::support::BoundaryProjectionError>,
) -> Result<AccountedResponse<M>, ConnectError> {
    let message = encode(output.value).map_err(projection_error)?;
    Ok(AccountedResponse {
        message,
        reservation: output.owner,
        maximum_bytes: output.maximum_bytes,
    })
}

pub(super) fn projection_error(
    error: pse_contract_proto::support::BoundaryProjectionError,
) -> ConnectError {
    // The maintained validator owns the canonical bounded violation details.
    // Preserve them exactly once at the common ingress/outgoing presentation seam.
    let details = error
        .wire_validation()
        .map(|failure| failure.clone().into_connect_error().details)
        .unwrap_or_default();
    let mut response = transport(EngineeringServiceError::Owner(Box::new(
        crate::projection_diagnostics::ProjectionDiagnosticError::from(error),
    )));
    response.details.extend(details);
    response
}

pub(super) fn observations(
    engine: Arc<EngineeringService>,
    input: EngineeringObserveStudy,
    input_owner: Arc<MemoryReservation>,
    deadline: Option<std::time::Instant>,
) -> ServiceStream<AccountedResponse<pb::StudyObservationPage>> {
    use futures_util::{TryStreamExt as _, stream};

    // ConnectRPC can drop setup or the response body. Admit only from a polled
    // response stream, then let the service join any admitted native capture.
    let open = stream::once(async move {
        let native = engine.clone();
        let observer =
            settle_study_observation(&engine, input_owner.clone(), deadline, async move {
                native.observe_study(input, deadline).await
            })
            .await?
            .ok_or_else(study_observation_deadline_error)?;
        Ok::<_, ConnectError>(stream::unfold(
            Some((engine, observer, input_owner, false)),
            move |state| async move {
                let (engine, mut observer, input_owner, follow) = state?;
                let next = async {
                    if follow {
                        wait_for_study_observation(deadline).await?;
                    }
                    let native = engine.clone();
                    settle_study_observation(&engine, input_owner.clone(), deadline, async move {
                        let page = native.study_observation_page(&mut observer).await?;
                        Ok((observer, page))
                    })
                    .await
                };
                let (observer, output) = match next.await {
                    Ok(output) => output,
                    Err(error) => return Some((Err(error), None)),
                };
                let done = observation_complete(
                    output.value.status.header.terminal,
                    &output.value.sources,
                );
                let page = accounted(
                    output,
                    crate::engineering_projection::encode_study_observation_page,
                );
                let done = done || page.is_err();
                Some((
                    page,
                    (!done).then_some((engine, observer, input_owner, true)),
                ))
            },
        ))
    });
    Box::pin(open.try_flatten())
}

async fn settle_study_observation<T: Send + 'static>(
    engine: &EngineeringService,
    input_owner: Arc<MemoryReservation>,
    deadline: Option<std::time::Instant>,
    work: impl Future<Output = Result<T, EngineeringServiceError>> + Send + 'static,
) -> Result<T, ConnectError> {
    check_study_observation_deadline(deadline)?;
    // An admitted poll can issue a capture. Let it settle under its original
    // clock. A discarded success drops through its normal custody release.
    let task = async move {
        if check_study_observation_deadline(deadline).is_err() {
            return Ok(None);
        }
        work.await.map(Some)
    };
    // The probe observes the actual request grant without retaining it. Scope
    // it inside the joined task: the detached transport waiter owns no probe.
    #[cfg(all(test, feature = "solver-kinsol"))]
    let task =
        crate::workflow::StudyObserver::with_ingress_probe(Arc::downgrade(&input_owner), task);
    let value = engine
        .observe_owned(input_owner, task)
        .await
        .map_err(transport)?;
    check_study_observation_deadline(deadline)?;
    value.ok_or_else(study_observation_deadline_error)
}

async fn wait_for_study_observation(
    deadline: Option<std::time::Instant>,
) -> Result<(), ConnectError> {
    let wait = tokio::time::sleep(std::time::Duration::from_millis(100));
    match deadline {
        Some(deadline) => tokio::time::timeout_at(deadline.into(), wait)
            .await
            .map_err(|_| study_observation_deadline_error()),
        None => Ok(wait.await),
    }
}

fn check_study_observation_deadline(
    deadline: Option<std::time::Instant>,
) -> Result<(), ConnectError> {
    if deadline.is_some_and(|deadline| std::time::Instant::now() >= deadline) {
        return Err(study_observation_deadline_error());
    }
    Ok(())
}

fn study_observation_deadline_error() -> ConnectError {
    ConnectError::deadline_exceeded(
        "study observation allowance elapsed; the original study remains under its retained owner",
    )
}

fn observation_complete(terminal: bool, sources: &[crate::workflow::StudyObservedSource]) -> bool {
    // A terminal canonical header does not exhaust the original bounded history reader.
    // Continue until each selected native source explicitly reports its own exhaustion.
    terminal
        && sources
            .iter()
            .all(|source| source.position.history_complete)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn engineering_study_observation_expiry_preserves_admitted_native_outcomes() {
        use connectrpc::ErrorCode;
        use datafusion::execution::memory_pool::MemoryConsumer;
        use std::{
            sync::atomic::{AtomicBool, Ordering},
            time::{Duration, Instant},
        };

        let (engine, _spill) = super::super::super::session::tests::service();
        let input = Arc::new(
            MemoryConsumer::new("controlled observation input").register(&engine.ingress_pool()),
        );
        let polled = Arc::new(AtomicBool::new(false));
        let marker = polled.clone();
        let refused =
            settle_study_observation(&engine, input.clone(), Some(Instant::now()), async move {
                marker.store(true, Ordering::Release);
                Ok(())
            })
            .await;
        assert!(matches!(refused, Err(error) if error.code == ErrorCode::DeadlineExceeded));
        assert!(!polled.load(Ordering::Acquire));

        let deadline = Instant::now() + Duration::from_secs(1);
        let after_deadline = (deadline + Duration::from_millis(25)).into();
        let settled = Arc::new(AtomicBool::new(false));
        let marker = settled.clone();
        let (completed, failed) = tokio::join!(
            settle_study_observation(&engine, input.clone(), Some(deadline), async move {
                tokio::time::sleep_until(after_deadline).await;
                marker.store(true, Ordering::Release);
                Ok(())
            }),
            settle_study_observation(&engine, input, Some(deadline), async move {
                tokio::time::sleep_until(after_deadline).await;
                Err::<(), _>(EngineeringServiceError::Request(
                    "original capture refused".into(),
                ))
            }),
        );
        assert!(settled.load(Ordering::Acquire));
        assert!(matches!(completed, Err(error) if error.code == ErrorCode::DeadlineExceeded));
        assert!(
            matches!(failed, Err(error) if error.code == ErrorCode::InvalidArgument
            && error.message.as_deref().is_some_and(|message| message.contains("original capture refused"))
            && error.source_arc().is_some())
        );
        engine.shutdown().await.unwrap();
    }

    #[test]
    fn engineering_study_stream_preserves_terminal_history_across_bounded_pages() {
        let mut source = crate::workflow::StudyObservedSource {
            position: crate::workflow::StudyObservationPosition {
                run: "original-run".into(),
                attempt: "original-attempt".into(),
                attempt_generation: 1,
                metadata_revision: None,
                source_generation: None,
                source_sequence: 0,
                batch: 0,
                offset: 0,
                durable_sequence: None,
                history_complete: false,
            },
            continuity: crate::workflow::StudyObservationContinuity::CatchUp,
            events: Vec::new(),
            checkpoint: None,
            coverage: None,
            history_refusal: None,
        };
        assert!(!observation_complete(true, std::slice::from_ref(&source)));
        source.position.history_complete = true;
        assert!(!observation_complete(false, std::slice::from_ref(&source)));
        assert!(observation_complete(true, std::slice::from_ref(&source)));
    }
}
