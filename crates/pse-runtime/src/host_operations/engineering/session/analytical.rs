// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mathematical native execution retains the same issued run and original reader as solves.
use super::*;
use crate::host_operations::engineering_contract::EngineeringExecuteMathematical;
#[cfg(test)]
mod tests;
impl EngineeringService {
    /// Submit or reconcile exactly one original selected mathematical request.
    /// Original `ReadResult` reads the frozen mathematical relation after source retirement.
    pub async fn execute_mathematical(
        self: &Arc<Self>,
        input: EngineeringExecuteMathematical,
    ) -> Result<EngineeringRunStatus, EngineeringServiceError> {
        input.validate_projection().map_err(owner)?;
        if input.operation == input.source_operation {
            return Err(request("mathematical operation cannot depend on itself"));
        }
        let key = EngineeringRunKey {
            operation: input.operation.clone(),
            run: input.run,
            attempt: input.attempt,
        };
        if matches!(
            self.runtime.durability(),
            crate::workflow::Durability::Durable(_)
        ) && let Some((_owner, _record, _attempt, stored, _preparation)) =
            self.find_original_record(&key).await?
        {
            if stored.submission != fingerprint(&input)? {
                return Err(request(
                    "original mathematical submission belongs to another envelope",
                ));
            }
            return self.run_status(&key).await;
        }
        let (operation, fresh) = self.issue_with_issuance(
            &input.operation,
            Kind::Run,
            &input,
            Issuance::Run(key.clone()),
        )?;
        if fresh {
            let service = self.clone();
            let original = operation.clone();
            self.spawn(operation.clone(), async move {
                let source = service.operation(&input.source_operation, Kind::Source)?;
                let Outcome::Source { package, .. } = source.dependency(&original.cancel).await?
                else {
                    return Err(request("original mathematical source unavailable"));
                };
                if package.revision_id() != input.expected_local_revision {
                    return Err(request("original mathematical source parent is stale"));
                }
                let prepared = package
                    .prepare_operation(
                        &service.runtime.preparation_services(),
                        &OperationSource::of(&package),
                        &input.preparation,
                        RuntimeOperationRequest::Mathematical(input.request),
                        None,
                        &original.cancel,
                    )
                    .await
                    .map_err(owner)?;
                let preparation = prepared.identity().map_err(owner)?;
                let identity = crate::workflow::IssuedRunIdentity {
                    run: pse_model::generated::identities::RunId::from_id(input.run),
                    attempt: pse_model::generated::identities::AttemptId::from_id(input.attempt),
                    operation: input.operation,
                    submission: original.fingerprint,
                };
                Ok(
                    match prepared
                        .start_issued(
                            &service.runtime.execution_services(),
                            identity,
                            &original.cancel,
                        )
                        .await
                        .map_err(owner)?
                    {
                        crate::workflow::IssuedRunStart::Started(handle) => Outcome::Run {
                            handle,
                            preparation,
                        },
                        crate::workflow::IssuedRunStart::Retained { .. } => Outcome::Retained,
                    },
                )
            })?;
        }
        operation.completed().await?;
        self.run_status(&key).await
    }
}
