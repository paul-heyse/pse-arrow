// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected analytical science runs on the original admitted compiler lane.
use super::*;
use pse_compiler::workspace::{AnalyticalOperation, CompiledAnalyticalResult};
impl MathService {
    pub(crate) async fn analytical_member_for(
        self: &Arc<Self>,
        workspace: Workspace,
        frontier: Arc<modeling::OwnedModelingFrontier>,
        member: pse_ids::SemanticId,
        operation: AnalyticalOperation,
        limits: pse_math::jets::EvaluationLimits,
        driver: &crate::CancelSource,
    ) -> Result<Arc<pse_columnar::Leased<CompiledAnalyticalResult>>, MathRuntimeError> {
        let driver = self.preparation_driver(driver)?;
        let deadline = driver.deadline();
        let control = Cancellation::default();
        let work = self.compiler_job_retained(
            workspace,
            ConstructionEstimate::Known(0),
            control.clone(),
            deadline,
            move |authority, flag| {
                let scope = pse_kernels::ExecutionScope::new(flag.clone(), deadline);
                let result = authority.lock()?.prepare_analytical_member(
                    frontier.product(),
                    member,
                    operation,
                    limits,
                    &scope,
                )?;
                let bytes = result.retained_bytes();
                Ok((result, bytes))
            },
        );
        tokio::pin!(work);
        let (result, owner) = tokio::select! {result=&mut work=>result?,()=driver.cancelled()=>{control.cancel();let _=work.await;return Err(MathRuntimeError::Cancelled)}};
        Ok(Arc::new(pse_columnar::Leased::new(Arc::new(result), owner)))
    }
}
