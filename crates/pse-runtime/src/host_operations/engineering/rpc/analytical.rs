// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native selected mathematics and ordinary protected result reads share one custody route.
macro_rules! methods {
    () => {
        async fn execute_mathematical<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringExecuteMathematical>,
        ) -> connectrpc::ServiceResult<
            impl connectrpc::Encodable<pb::EngineeringRunStatus> + Send + use<'a>,
        > {
            self.authenticate(&context)?;
            let input = crate::engineering_projection::decode_engineering_execute_mathematical(
                request.to_owned_message(),
            )
            .map_err(study::projection_error)?;
            let output = self
                .engine
                .execute_mathematical(input)
                .await
                .map_err(transport)?;
            Ok(connectrpc::Response::new(
                crate::engineering_projection::encode_engineering_run_status(output)
                    .map_err(study::projection_error)?,
            ))
        }
    };
}
pub(super) use methods;
