// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit retained scientific lifecycle operations on the canonical substrate.

use super::{Runtime, WorkflowError};
use pse_columnar::CancellationToken;
pub use pse_operations::canonical_result_retention::ResultReclamationPage;

impl Runtime {
    /// Withdraw a terminal study's obligation to retain its point and summary
    /// results. This preserves scientific occurrence receipts; each run remains
    /// retained until explicitly retired, and live readers still protect it.
    pub async fn forget_study_results(&self, study: &str) -> Result<(), WorkflowError> {
        self.operations()?
            .store()
            .forget_study_results(study)
            .await?;
        Ok(())
    }
    /// Retire and remove one bounded page of analysis state; repeat until true.
    /// False includes pending settlement of the immutable creation window.
    pub async fn forget_analysis_results(&self, analysis: &str) -> Result<bool, WorkflowError> {
        Ok(self
            .operations()?
            .store()
            .forget_analysis_results(analysis)
            .await?)
    }
    /// Explicitly retire one recovered terminal run and perform one bounded
    /// cleanup page. Repeat until `complete`; cancellation or process interruption
    /// leaves the recorded cursor ready to resume without scientific execution.
    pub async fn reclaim_run_results(
        &self,
        run: &str,
        cancel: &CancellationToken,
    ) -> Result<ResultReclamationPage, WorkflowError> {
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        let operations = self.operations()?;
        operations.store().forget_run_results(run).await?;
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        Ok(operations.store().reclaim_result_page(run).await?)
    }
}
