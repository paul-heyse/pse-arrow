// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Scientific execution identities and lifecycle vocabulary shared by canonical workflows.
use super::declarations::{enumeration, identity};
use crate::RegistryBuilder;
use crate::model::{EnumDecl, EnumMember, EnumProjection, EnumProjectionMember};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    for (name,members,transitions) in [
        ("CanonicalStudyCaptureState", &["pending","frozen","abandoned","releasing","released"][..], &[("pending","frozen"),("pending","abandoned"),("abandoned","releasing"),("frozen","releasing"),("releasing","released")][..]),
        ("CanonicalStudyInstallationPhase", &["nodes","edges","active"][..], &[("nodes","edges"),("nodes","active"),("edges","active")][..]),
        ("CanonicalStudyInstallationPageState", &["issued","committed"][..], &[("issued","committed")][..]),
    ] {
        builder.declare_enum(EnumDecl::platform(name,members.iter().map(|name|EnumMember::new(name,name)).collect()).with_transitions(transitions));
    }
    enumeration(builder, "CanonicalPayloadPlacement", ["inline", "sealed"]);
    enumeration(builder, "ConformanceExecution", ["native", "pure"]);
    enumeration(builder, "ConformancePresolve", ["auto", "off"]);
    enumeration(
        builder,
        "ScientificArtifactKind",
        ["final-state", "trajectory", "local-analysis"],
    );
    enumeration(
        builder,
        "ScientificArtifactFamily",
        ["modeling", "simulation", "fit"],
    );
    enumeration(builder, "ArtifactOwnerKind", ["user", "project", "audit"]);
    enumeration(
        builder,
        "LiveSimulationEventKind",
        [
            "sample",
            "attempt_finished",
            "selected",
            "recorded",
            "recording_unknown",
            "recording_closed",
            "joined",
        ],
    );
    builder.declare_enum(
        EnumDecl::platform(
            "CanonicalTerminalOutcome",
            ["succeeded", "failed", "partial", "cancelled"]
                .into_iter()
                .map(|name| EnumMember::new(name, name))
                .collect(),
        )
        .with_projection(EnumProjection {
            method: "attempt_state",
            target: "AttemptState",
            input: None,
            members: [
                ("succeeded", "completed"),
                ("failed", "failed"),
                ("partial", "partial"),
                ("cancelled", "cancelled"),
            ]
            .into_iter()
            .map(|(source, value)| EnumProjectionMember {
                source,
                value,
                when_true: None,
            })
            .collect(),
        })
        .with_projection(EnumProjection {
            method: "study_point_state",
            target: "StudyPointState",
            input: Some("partial_usable"),
            members: [
                ("succeeded", "completed", "completed"),
                ("failed", "failed", "failed"),
                ("partial", "failed", "completed"),
                ("cancelled", "cancelled", "cancelled"),
            ]
            .into_iter()
            .map(|(source, value, when_true)| EnumProjectionMember {
                source,
                value,
                when_true: Some(when_true),
            })
            .collect(),
        }),
    );
    builder.declare_enum(
        EnumDecl::platform(
            "CanonicalAttemptPhase",
            [
                "ingesting",
                "closed_awaiting_manifest",
                "closed_with_manifest",
                "terminal",
                "terminal_compacted",
            ]
            .into_iter()
            .map(|name| EnumMember::new(name, name))
            .collect(),
        )
        .with_transitions(&[
            ("ingesting", "closed_awaiting_manifest"),
            ("ingesting", "closed_with_manifest"),
            ("closed_awaiting_manifest", "closed_with_manifest"),
            ("closed_with_manifest", "terminal"),
            ("terminal", "terminal_compacted"),
        ])
        .with_predicate("ingestion_open", &["ingesting"])
        .with_predicate(
            "closed",
            &[
                "closed_awaiting_manifest",
                "closed_with_manifest",
                "terminal",
                "terminal_compacted",
            ],
        )
        .with_predicate("terminal", &["terminal", "terminal_compacted"]),
    );
    enumeration(
        builder,
        "CanonicalProjectPublicationKind",
        ["import", "fork", "publish"],
    );
    enumeration(
        builder,
        "CanonicalArtifactHolderKind",
        super::substrate::ARTIFACT_HOLDERS
            .iter()
            .map(|holder| holder.0),
    );
    builder.declare_enum(
        EnumDecl::platform(
            "CanonicalProducerClass",
            ["general", "library_mathematics", "local_scientific_replay"]
                .into_iter()
                .map(|name| EnumMember::new(name, name))
                .collect(),
        )
        .with_predicate(
            "scientific",
            &["library_mathematics", "local_scientific_replay"],
        ),
    );
    enumeration(
        builder,
        "CanonicalRunPurpose",
        ["execution", "study_admission"],
    );
    enumeration(builder, "CanonicalFileOwnerKind", ["result", "artifact"]);
    enumeration(
        builder,
        "CanonicalFileState",
        [
            "pending",
            "sealed",
            "attached",
            "abandoned",
            "collecting",
            "collected",
        ],
    );
    enumeration(
        builder,
        "CanonicalPayloadEncoding",
        ["arrow-ipc-file", "opaque-json", "opaque-bytes"],
    );
    enumeration(
        builder,
        "CanonicalHistoryOwnerKind",
        [
            "run", "study", "point", "analysis", "user", "project", "audit",
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
