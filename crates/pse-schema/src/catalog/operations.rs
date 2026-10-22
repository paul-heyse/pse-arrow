// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Scientific execution identities and lifecycle vocabulary shared by canonical workflows.
use super::declarations::{enumeration, identity};
use crate::RegistryBuilder;

pub(super) fn declare(builder: &mut RegistryBuilder) {
    enumeration(builder, "ConformanceExecution", ["native", "pure"]);
    enumeration(builder, "ConformancePresolve", ["auto", "off"]);
    enumeration(builder, "ScientificArtifactKind", ["final-state", "trajectory", "local-analysis"]);
    enumeration(builder, "ArtifactOwnerKind", ["user", "project", "audit"]);
    enumeration(builder, "LiveSimulationEventKind", ["sample", "attempt_finished", "selected", "recorded", "recording_unknown", "recording_closed", "joined"]);
    enumeration(
        builder,
        "CanonicalTerminalOutcome",
        ["succeeded", "failed", "partial", "cancelled"],
    );
    enumeration(
        builder,
        "CanonicalAttemptPhase",
        [
            "ingesting",
            "closed_awaiting_manifest",
            "closed_with_manifest",
            "terminal",
            "terminal_compacted",
        ],
    );
    enumeration(
        builder,
        "AttemptState",
        [
            "planned",
            "queued",
            "running",
            "completed",
            "partial",
            "failed",
            "cancelled",
            "stale",
            "superseded",
        ],
    );
    enumeration(builder, "StudyState", ["open", "concluded"]);
    enumeration(
        builder,
        "StudyPointState",
        ["pending", "assigned", "completed", "failed", "cancelled"],
    );
    declare_run_identity(builder);
    identity(
        builder,
        "attempt",
        "One actual scientific attempt of a run, with its immutable generation and observations",
    );
    identity(
        builder,
        "solution",
        "One qualified portable scientific seed from an admitted attempt",
    );
    identity(
        builder,
        "study",
        "One retained study coordinating distinct point occurrences",
    );
}
/// The identity scientific result rows and diagnostic findings reference.
pub(super) fn declare_run_identity(builder: &mut RegistryBuilder) {
    identity(
        builder,
        "run",
        "One immutable problem request, whose actual scientific executions are distinct attempts",
    );
}
