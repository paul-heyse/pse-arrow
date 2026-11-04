// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed managed-helper workflow. Native interpreters own verdicts; the enclosing scope owns drain.
pub(crate) mod evidence;
pub(crate) mod phase;
mod report;
mod runner;
pub use evidence::{
    CapturedFile, EvidenceFile, EvidenceOmission, EvidenceRole, EvidenceRoot, EvidenceScope,
};
pub use report::{
    Action, Bounds, ContentIdentity, FixturePending, Input, Outcome, Pending, Phase, PhaseKind,
    Report, ReportObservation, ReportReference, TerminalRecord,
};
pub use runner::run;
