// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated actual native inspection adapters retain original selected response ownership.

macro_rules! methods {
    () => {
        async fn inspect_modeling<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringInspectModeling>,
        ) -> connectrpc::ServiceResult<
            impl connectrpc::Encodable<pb::ModelingInspection> + Send + use<'a>,
        > {
            self.authenticate(&context)?;
            let request = request.to_owned_message();
            adoption::validate(&request)?;
            let input = crate::engineering_projection::decode_engineering_inspect_modeling(request)
                .map_err(study::projection_error)?;
            let cancel = CancelSource::new().with_deadline(context.deadline());
            let output = self
                .engine
                .inspect_modeling(input, &cancel)
                .await
                .map_err(transport)?;
            let message =
                crate::engineering_projection::encode_modeling_inspection(output.document)
                    .map_err(study::projection_error)?;
            Ok(connectrpc::Response::new(AccountedResponse {
                message,
                reservation: output.owner,
                maximum_bytes: 64 << 10,
            }))
        }

        async fn inspect_providers<'a>(
            &'a self,
            context: connectrpc::RequestContext,
            request: connectrpc::ServiceRequest<'_, pb::EngineeringInspectProviders>,
        ) -> connectrpc::ServiceResult<
            impl connectrpc::Encodable<pb::ProviderInspectionPage> + Send + use<'a>,
        > {
            self.authenticate(&context)?;
            let request = request.to_owned_message();
            adoption::validate(&request)?;
            let input =
                crate::engineering_projection::decode_engineering_inspect_providers(request)
                    .map_err(study::projection_error)?;
            let cancel = CancelSource::new().with_deadline(context.deadline());
            let output = self
                .engine
                .inspect_providers(input, &cancel)
                .map_err(transport)?;
            let message =
                crate::engineering_projection::encode_provider_inspection_page(output.page)
                    .map_err(study::projection_error)?;
            Ok(connectrpc::Response::new(AccountedResponse {
                message,
                reservation: output.owner,
                maximum_bytes: 64 << 10,
            }))
        }
    };
}
pub(super) use methods;
