// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit retained scientific lifecycle operations on the canonical substrate.

use super::{Runtime, WorkflowError};
use pse_columnar::CancellationToken;
pub use pse_operations::canonical_history::{
    HistoryObligation, HistoryOwner, HistoryPurpose, HistoryReclamationPage,
};
pub use pse_operations::canonical_result_retention::ResultReclamationPage;

impl Runtime {
    /// Retain exact audit or unsettled-effect detail independently from payloads.
    pub async fn retain_history_obligation(
        &self,
        obligation: &HistoryObligation,
    ) -> Result<(), WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                self.operations()?
                    .store()
                    .retain_history_obligation(obligation)
                    .await?;
                Ok(())
            })
            .await
    }

    /// Release only the exact attributed obligation; other owners still protect detail.
    pub async fn release_history_obligation(
        &self,
        obligation: &HistoryObligation,
    ) -> Result<(), WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                self.operations()?
                    .store()
                    .release_history_obligation(obligation)
                    .await?;
                Ok(())
            })
            .await
    }

    /// Explicitly release detailed history after its payload and obligations settle.
    /// This is irreversible and preserves permanent replay and owner identities.
    pub async fn release_history(&self, owner: &HistoryOwner) -> Result<(), WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                self.operations()?.store().release_history(owner).await?;
                Ok(())
            })
            .await
    }

    /// Collect one bounded page after explicit history release. Repetition resumes
    /// the same owner; study collection includes its already settled member detail.
    pub async fn reclaim_history_page(
        &self,
        owner: &HistoryOwner,
        cancel: &CancellationToken,
    ) -> Result<HistoryReclamationPage, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                Ok(self
                    .operations()?
                    .store()
                    .reclaim_history_page(owner)
                    .await?)
            })
            .await
    }

    /// Withdraw a terminal study's obligation to retain its point and summary
    /// results. This preserves scientific occurrence receipts; each run remains
    /// retained until explicitly retired, and live readers still protect it.
    pub async fn forget_study_results(&self, study: &str) -> Result<(), WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                self.operations()?
                    .store()
                    .forget_study_results(study)
                    .await?;
                Ok(())
            })
            .await
    }
    /// Retire and remove one bounded page of analysis state; repeat until true.
    /// False includes pending settlement of the immutable creation window.
    pub async fn forget_analysis_results(&self, analysis: &str) -> Result<bool, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                Ok(self
                    .operations()?
                    .store()
                    .forget_analysis_results(analysis)
                    .await?)
            })
            .await
    }
    /// Explicitly retire one recovered terminal run and perform one bounded
    /// cleanup page. Repeat until `complete`; cancellation or process interruption
    /// leaves the recorded cursor ready to resume without scientific execution.
    pub async fn reclaim_run_results(
        &self,
        run: &str,
        cancel: &CancellationToken,
    ) -> Result<ResultReclamationPage, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                let operations = self.operations()?;
                operations.store().forget_run_results(run).await?;
                cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                Ok(operations.store().reclaim_result_page(run).await?)
            })
            .await
    }
}
