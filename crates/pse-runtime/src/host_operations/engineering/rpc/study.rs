// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated native adapters and original bounded study observation stream.

use super::*;
use crate::host_operations::engineering_contract::EngineeringObserveStudy;

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
pub(super) use admission_unary;

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

macro_rules! methods {
    () => {
        study::owned_unary!(
            browse_studies,
            StudyWorkbenchRequest,
            StudyWorkbenchPage,
            decode_study_workbench_request,
            browse_studies,
            encode_study_workbench_page
        );
        study::admission_unary!(
            prepare_study,
            EngineeringPrepareStudy,
            decode_engineering_prepare_study,
            prepare_study
        );
        study::admission_unary!(
            restore_study,
            EngineeringRestoreStudy,
            decode_engineering_restore_study,
            restore_study
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

        async fn read_prepared_study<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringOperationKey>,
        ) -> connectrpc::ServiceResult<
            impl connectrpc::Encodable<pb::StudyDefinition> + Send + use<'a>,
        > {
            self.authenticate(&context)?;
            let request = request.to_owned_message();
            let input = crate::engineering_projection::decode_engineering_operation_key(request)
                .map_err(study::projection_error)?;
            let output = self
                .engine
                .prepared_study_definition(&input)
                .await
                .map_err(transport)?;
            Ok(connectrpc::Response::new(study::accounted(
                output,
                crate::engineering_projection::encode_study_definition,
            )?))
        }

        async fn observe_study(
            &self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringObserveStudy>,
        ) -> connectrpc::ServiceResult<
            ServiceStream<impl connectrpc::Encodable<pb::StudyObservationPage> + Send + use<>>,
        > {
            self.authenticate(&context)?;
            let request = request.to_owned_message();
            let input = crate::engineering_projection::decode_engineering_observe_study(request)
                .map_err(study::projection_error)?;
            let stream =
                study::observations(self.engine.clone(), input, context.deadline()).await?;
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
pub(super) use methods;

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

pub(super) async fn observations(
    engine: Arc<EngineeringService>,
    input: EngineeringObserveStudy,
    deadline: Option<std::time::Instant>,
) -> Result<ServiceStream<AccountedResponse<pb::StudyObservationPage>>, ConnectError> {
    let observer = engine.observe_study(input).await.map_err(transport)?;
    let cancel = CancelSource::new().with_deadline(deadline);
    let stream = futures_util::stream::unfold(
        (engine, observer, cancel, false, false),
        |(engine, mut observer, cancel, follow, done)| async move {
            if done {
                observer.close();
                return None;
            }
            match engine
                .study_observation_page(&mut observer, follow, &cancel)
                .await
            {
                Ok(output) => {
                    let done = observation_complete(
                        output.value.status.header.terminal,
                        &output.value.sources,
                    );
                    let page = accounted(
                        output,
                        crate::engineering_projection::encode_study_observation_page,
                    );
                    let done = done || page.is_err();
                    Some((page, (engine, observer, cancel, true, done)))
                }
                Err(error) => {
                    observer.close();
                    Some((
                        Err(transport(error)),
                        (engine, observer, cancel, true, true),
                    ))
                }
            }
        },
    );
    Ok(Box::pin(stream))
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
