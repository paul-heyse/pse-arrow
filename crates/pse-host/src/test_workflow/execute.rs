// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed managed-helper workflow. Native interpreters own verdicts; the enclosing scope owns drain.
pub(crate) mod evidence;
pub(crate) mod phase;
pub(crate) mod progress;
mod report;
mod runner;
pub use evidence::{
    CapturedFile, EvidenceFile, EvidenceOmission, EvidenceRole, EvidenceRoot, EvidenceScope,
};
pub(crate) use report::original_report_accounts;
pub use report::{
    Action, Bounds, CompilerFailureAccount, CompilerFailureFact, ContentIdentity, FixturePending,
    GroupFailureAccount, Input, MAX_PREREQUISITE_FACT_ROWS, Outcome, Pending, Phase, PhaseKind,
    Report, ReportObservation, ReportReference, TerminalRecord, TestFailureAccount,
    TestFailureFact,
};
pub use runner::run;
pub(crate) use runner::run_managed;
