// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native expectation calls retain original operation and finite response custody.
macro_rules! methods {
    () => {
        async fn read_expectation_samples<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringReadExpectationSamples>,
        ) -> connectrpc::ServiceResult<
            impl connectrpc::Encodable<pb::EngineeringExpectationSamples> + Send + use<'a>,
        > {
            self.authenticate(&context)?;
            let input = crate::engineering_projection::decode_engineering_read_expectation_samples(
                request.to_owned_message(),
            )
            .map_err(study::projection_error)?;
            let cancel = CancelSource::new().with_deadline(context.deadline());
            let output = self
                .engine
                .read_expectation_samples(input, &cancel)
                .await
                .map_err(transport)?;
            let message =
                crate::engineering_projection::encode_engineering_expectation_samples(output.page)
                    .map_err(study::projection_error)?;
            Ok(connectrpc::Response::new(AccountedResponse {
                message,
                reservation: output.owner,
                maximum_bytes: 16 << 20,
            }))
        }
        async fn execute_expectation<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringExecuteExpectation>,
        ) -> connectrpc::ServiceResult<
            impl connectrpc::Encodable<pb::ExpectationReport> + Send + use<'a>,
        > {
            self.authenticate(&context)?;
            let input = crate::engineering_projection::decode_engineering_execute_expectation(
                request.to_owned_message(),
            )
            .map_err(study::projection_error)?;
            let cancel = CancelSource::new().with_deadline(context.deadline());
            let output = self
                .engine
                .execute_expectation(input, &cancel)
                .await
                .map_err(transport)?;
            let message = crate::engineering_projection::encode_expectation_report(output.report)
                .map_err(study::projection_error)?;
            Ok(connectrpc::Response::new(AccountedResponse {
                message,
                reservation: output.owner,
                maximum_bytes: 16 << 20,
            }))
        }
        async fn read_expectation<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringReadExpectation>,
        ) -> connectrpc::ServiceResult<
            impl connectrpc::Encodable<pb::ExpectationReport> + Send + use<'a>,
        > {
            self.authenticate(&context)?;
            let input = crate::engineering_projection::decode_engineering_read_expectation(
                request.to_owned_message(),
            )
            .map_err(study::projection_error)?;
            let cancel = CancelSource::new().with_deadline(context.deadline());
            let output = self
                .engine
                .read_expectation(input, &cancel)
                .await
                .map_err(transport)?;
            let message = crate::engineering_projection::encode_expectation_report(output.report)
                .map_err(study::projection_error)?;
            Ok(connectrpc::Response::new(AccountedResponse {
                message,
                reservation: output.owner,
                maximum_bytes: 16 << 20,
            }))
        }
    };
}
pub(super) use methods;
