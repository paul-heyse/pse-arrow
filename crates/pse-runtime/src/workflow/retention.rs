// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit retained scientific lifecycle operations on the canonical substrate.

use super::{Runtime,WorkflowError};
use pse_columnar::CancellationToken;
pub use pse_operations::canonical_result_retention::ResultReclamationPage;

impl Runtime {
    /// Withdraw a terminal study's obligation to retain its point and summary
    /// results. This preserves scientific occurrence receipts; each run remains
    /// retained until explicitly retired, and live readers still protect it.
    pub async fn forget_study_results(&self,study:&str)->Result<(),WorkflowError> {
        self.operations()?.store().forget_study_results(study).await?;Ok(())
    }
    /// Withdraw derived analysis retention, including all selected source roots.
    /// Original method/configuration/input lineage remains an immutable receipt.
    pub async fn forget_analysis_results(&self,analysis:&str)->Result<(),WorkflowError> {
        self.operations()?.store().forget_analysis_results(analysis).await?;Ok(())
    }
    /// Explicitly retire one recovered terminal run and perform one bounded
    /// cleanup page. Repeat until `complete`; cancellation or process interruption
    /// leaves the recorded cursor ready to resume without scientific execution.
    pub async fn reclaim_run_results(&self,run:&str,cancel:&CancellationToken)->Result<ResultReclamationPage,WorkflowError> {
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        let operations=self.operations()?;
        operations.store().forget_run_results(run).await?;
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        Ok(operations.store().reclaim_result_page(run).await?)
    }
}
