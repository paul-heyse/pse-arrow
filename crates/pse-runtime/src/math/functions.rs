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
        let driver = self.preparation_driver(driver)?;
        let owner = self.shared_product(
            vec![1, Arc::as_ptr(&prepared.plan) as usize],
            Arc::new(prepared.clone()),
            lease,
            Vec::new(),
        )?;
        let plan = Arc::new(prepared.plan.as_ref().clone().with_owner(owner.clone()));
        let layout = self.reserve(
            "case_operation_layout",
            plan.operation_layout_allocation_bound()?,
        )?;
        let artifacts = self
            .prepare_artifacts(prepared.artifacts.iter(), &driver)
            .await?;
        let _ = self.preparation_driver(&driver)?;
        #[cfg(test)]
        self.preparation_effects
            .function_assemblies
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let _span = tracing::info_span!("pse.case.function_assembly", pse.owner.stage = true).entered();
        let assembly =
            Arc::new(plan.assemble(artifacts.iter().map(|a| a.program.clone()).collect())?);
        let _ = self.preparation_driver(&driver)?;
        Ok(Arc::new(ExecutableCase {
            assembly,
            _artifacts: artifacts,
            _layout: layout,
            _owner: owner,
        }))
    }
}
