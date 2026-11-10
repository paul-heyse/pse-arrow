// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admitted pure observation populations retain their compiler artifacts and result backing.
use super::*;
impl MathService {
    pub(crate) async fn evaluate_observation_batch(
        self: &Arc<Self>,
        case: Arc<ExecutableCase>,
        points: Vec<(pse_ids::SemanticId, pse_math::binding::CaseValues)>,
        driver: &crate::CancelSource,
    ) -> Result<Arc<pse_columnar::Leased<pse_math::assembly::CaseBatchOutcome>>, MathRuntimeError>
    {
        let driver = self.preparation_driver(driver)?;
        let bytes = case.assembly.batch_worker_bytes(points.len())?;
        let control = pse_diagnostics::Cancellation::default();
        let scope = pse_kernels::ExecutionScope::new(control.clone(), driver.deadline());
        let operation = self.job_retained_construction(
            1,
            crate::math::ConstructionEstimate::Opaque(bytes),
            control.clone(),
            scope.deadline(),
            move |flag| {
                if !Arc::ptr_eq(&flag, scope.cancellation()) {
                    return Err(pse_backend_native::ProblemError::Contract(
                        "observation batch cancellation owner changed".into(),
                    )
                    .into());
                }
                let mut worker = case.assembly.batch_worker(scope)?;
                let points = points
                    .iter()
                    .map(|(id, values)| pse_math::assembly::CaseBatchPoint { id: *id, values })
                    .collect::<Vec<_>>();
                let result = Arc::new(worker.constraints(&points)?);
                let bytes = result
                    .retained_bytes()
                    .checked_add(2 * size_of::<usize>())
                    .ok_or(MathRuntimeError::Limit("observation batch result"))?;
                Ok((result, bytes))
            },
        );
        tokio::pin!(operation);
        let (result, owner) = tokio::select! {
            result=&mut operation=>result?,
            ()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}
        };
        Ok(Arc::new(pse_columnar::Leased::new(result, owner)))
    }
}
