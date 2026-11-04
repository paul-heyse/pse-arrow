// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed managed-helper workflow. Native interpreters own verdicts; the enclosing scope owns drain.
pub(crate) mod phase;
pub(crate) mod evidence;
mod report;
mod runner;
pub use report::{
    Action, Bounds, FixturePending, Input, ContentIdentity, Outcome, Pending, Phase, PhaseKind, Report,
    ReportObservation, ReportReference, TerminalRecord,
};
pub use runner::run;
pub use evidence::{CapturedFile, EvidenceFile, EvidenceOmission, EvidenceRole, EvidenceRoot, EvidenceScope};
