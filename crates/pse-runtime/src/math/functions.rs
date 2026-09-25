// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared pure function projections and the existing compiler-issued artifact service.
use super::{ExecutableCase, MathRuntimeError, MathService, Workspace};
use pse_compiler::workspace::{Inputs, Profile};
use pse_engine::cache_service::flight::FlightCancellation;
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
        let owner = self.reserve("math:function-products", self.policy.workspace_bytes)?;
        let operation = self.job(1, self.policy.stack_bytes, control.clone(), move |flag| {
            let _lease = workspace.lease;
            let mut compiler = workspace
                .compiler
                .lock()
                .map_err(|_| MathRuntimeError::Infrastructure("compiler lock poisoned".into()))?;
            compiler.publish(inputs)?;
            Ok(compiler.prepare_functions(id, outputs, coordinates, order, profile, flag)?)
        });
        tokio::pin!(operation);
        let prepared = tokio::select! { result = &mut operation => result?, ()=driver.cancelled()=>{
            control.cancel();return Err(MathRuntimeError::Cancelled);
        }};
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
impl MathService {
    pub(crate) async fn validate_dynamic_partition(
        self: &Arc<Self>,
        workspace: Workspace,
        inputs: Inputs,
        id: SemanticId,
        rows: Vec<SemanticId>,
        columns: Vec<SemanticId>,
        cancel: &crate::CancelSource,
    ) -> Result<(), MathRuntimeError> {
        let control = FlightCancellation::default();
        let operation = self.job(
            1,
            pse_structural::incidence::MATCHING_STACK,
            control.clone(),
            move |flag| {
                let _lease = workspace.lease;
                let mut compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                compiler.publish(inputs)?;
                let analysis = compiler.analyze_dynamic_partition(id, rows, columns, flag)?;
                pse_backend_native::structural::admit(
                    &analysis,
                    pse_backend_native::structural::Mode::Roots,
                )?;
                Ok(())
            },
        );
        tokio::pin!(operation);
        tokio::select! {r=&mut operation=>r,()=cancel.cancelled()=>{control.cancel();Err(MathRuntimeError::Cancelled)}}
    }
}
