// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared pure function projections and the existing compiler-issued artifact service.
use super::{ExecutableCase, MathRuntimeError, MathService};
use std::sync::Arc;
impl MathService {
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
            Vec::new(),
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
