// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Completed native evidence, independent of raw output and physical settlement.
use crate::{
    extraction::{CompletedReport, ExtractionError, NextestSummary, OutputStream},
    managed::streams::ChunkObserver,
};
use std::path::PathBuf;
/// Failure to consume the required original completed report.
#[derive(Debug, thiserror::Error)]
#[error("native completed report: {error}")]
pub struct ObservationError {
    /// Original decode, identity or availability failure.
    #[source]
    pub error: ExtractionError,
}
pse_diagnostics::impl_diagnostic! {
    ObservationError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(this) { Some(&this.error) }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
/// Exact failed test's address in a native completed artifact, not a stream event offset.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct FailureReference {
    /// Original selected identity.
    pub identity: crate::discovery::TestIdentity,
    /// Native report path containing the binary/test entry.
    pub report: PathBuf,
}
/// Owner-constructed native completed facts.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct Observation {
    /// Original completed native artifact; never substituted with XML-safe text for raw bytes.
    pub report: PathBuf,
    /// Native exit plus exact population and report reconciliation.
    pub summary: NextestSummary,
    /// Native failure artifact addresses.
    pub failures: Vec<FailureReference>,
}
/// Required completed-report consumer. Raw stdout/stderr are retained without interpretation.
#[derive(Debug)]
pub struct CompletedObserver(CompletedReport);
impl CompletedObserver {
    /// Bind one private report and captured population.
    pub fn new(report: CompletedReport) -> Self {
        Self(report)
    }
    /// Require the exact source fixture owner's settled result for a borrowed service.
    pub fn with_fixture_outcome(self, path: PathBuf) -> Self {
        Self(self.0.with_fixture_outcome(path))
    }
    /// Called after original wait and required raw output drainage.
    pub fn finish(
        self,
        child_exit: i32,
        capture_complete: bool,
    ) -> (Observation, Option<ObservationError>) {
        let report = self.0.path().to_owned();
        let (summary, failures, error) = self.0.finish(child_exit, capture_complete);
        (
            Observation {
                report,
                summary,
                failures: failures
                    .into_iter()
                    .map(|failure| FailureReference {
                        identity: failure.identity,
                        report: failure.report,
                    })
                    .collect(),
            },
            error.map(|error| ObservationError { error }),
        )
    }
}
impl ChunkObserver for CompletedObserver {
    type Error = ObservationError;
    fn observe(
        &mut self,
        _stream: OutputStream,
        _offset: u64,
        _bytes: &[u8],
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
