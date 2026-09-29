// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared pure function projections and the existing compiler-issued artifact service.
use super::{ExecutableCase, MathRuntimeError, MathService, Workspace};
use pse_columnar::flight::FlightCancellation;
use pse_compiler::workspace::{Inputs, Profile};
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use std::sync::Arc;
impl MathService {
    /// Atomically prepare selected outputs/coordinates from an immutable model revision.
    #[expect(
        clippy::too_many_arguments,
        reason = "selected function projection binds ordered outputs and coordinates atomically"
    )]
    pub async fn prepare_functions_revision(
        self: &Arc<Self>,
        workspace: Workspace,
        inputs: Inputs,
        id: SemanticId,
        outputs: Vec<SemanticId>,
        coordinates: Vec<SemanticId>,
        order: DerivativeOrder,
        profile: Profile,
        driver: &crate::CancelSource,
    ) -> Result<Arc<ExecutableCase>, MathRuntimeError> {
        let control = FlightCancellation::default();
        let foreign = self.policy.foreign_bytes;
        let operation = self.job_retained(
            1,
            super::WITHIN_WORKSPACE,
            control.clone(),
            move |flag| {
                let _lease = workspace.lease;
                let mut compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                compiler.publish(inputs)?;
                let prepared =
                    compiler.prepare_functions(id, outputs, coordinates, order, profile, flag)?;
                let bytes = prepared
                    .plan
                    .retained_bytes()
                    .checked_add(foreign)
                    .ok_or(MathRuntimeError::Limit("function product extent"))?;
                Ok((prepared, bytes))
            },
        );
        tokio::pin!(operation);
        let (prepared, lease) = tokio::select! { result = &mut operation => result?, ()=driver.cancelled()=>{
            control.cancel();return Err(MathRuntimeError::Cancelled);
        }};
        self.assemble_functions(prepared, lease, driver).await
    }
    pub(super) async fn assemble_functions(
        self: &Arc<Self>,
        prepared: pse_compiler::workspace::PreparedFunctions,
        lease: Arc<pse_columnar::AllocationLease>,
        driver: &crate::CancelSource,
    ) -> Result<Arc<ExecutableCase>, MathRuntimeError> {
        let owner = self.shared_product(
            vec![1, Arc::as_ptr(&prepared.plan) as usize],
            Arc::new(prepared.clone()),
            lease,
        )?;
        let mut artifacts = Vec::new();
        for request in prepared.artifacts.iter() {
            if driver.token().is_cancelled() {
                return Err(MathRuntimeError::Cancelled);
            }
            artifacts.push(self.artifact(request.clone()).await?);
        }
        let plan = Arc::new(prepared.plan.as_ref().clone().with_owner(owner.clone()));
        let _span = tracing::info_span!("pse.case.function_assembly").entered();
        let assembly =
            Arc::new(plan.assemble(artifacts.iter().map(|a| a.program.clone()).collect())?);
        Ok(Arc::new(ExecutableCase {
            assembly,
            _artifacts: artifacts,
            _owner: owner,
        }))
    }
}
