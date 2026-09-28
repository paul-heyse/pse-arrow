// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Target Delta control facts; exact members replace custom manifest membership.
use super::declarations::{column, enumeration, relation};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    declare_retention(builder);
    declare_dependencies(builder);
    declare_artifacts(builder);
    declare_manifests(builder);
    builder.declare_artifact_profile("relations", std::collections::BTreeSet::new());
    relation(
        builder,
        N::Reference,
        "artifact_profiles",
        S::Model,
        &["kind"],
        vec![
            column("kind", T::native(arrow_schema::DataType::Utf8)),
            column("required_relations", T::list(T::id())),
        ],
        "Declared artifact completeness; every required relation must be selected even when empty.",
    );
    enumeration(builder, "MemberSelectionKind", ["full", "revision"]);
    enumeration(
        builder,
        "PublicationKind",
        [
            "relations",
            "source",
            "model",
            "case",
            "problem",
            "run",
            "diagnostics",
            "inspection",
        ],
    );
    builder.declare_relation(crate::model::RelationDecl::new(
        N::Runtime, "publications", 2, crate::model::Authority::Derived, S::Sidecar,
        "One native Delta control row selects exact members; native transactions index publication and attempt identities.",
    ).pk(&["workspace_id"]).granularity(crate::model::DerivationGranularity::Row)
        .columns(vec![
            T::key("workspace_id", T::id(), "Workspace control identity."),
            column("publication_id", T::id()),
            column("parent_publication_id", T::id()).optional(),
            column("attempt_id", T::id()),
            column("kind", T::enumeration("PublicationKind")),
            column("inputs", T::list(member())),
            column("members", T::list(member())),
        ]).checks(super::row_checks::for_relation(N::Runtime, "publications")));
}

/// The publication record (Plan 22 X12). Admitted candidates carry it before a catalog
/// commit makes them visible; an export writes it once as version 1 of a one-row Delta
/// table, with the export fields that let an offline reader open exactly those members.
fn declare_manifests(builder: &mut RegistryBuilder) {
    let micros = || T::native(crate::model::extension::timestamp_micros_storage());
    let exported = ["export_lease_id", "export_expires_at", "maintenance_epoch", "store_fingerprint"]
        .iter()
        .map(|field| format!("(\"exported_at\" IS NULL) = (\"{field}\" IS NULL)"))
        .collect::<Vec<_>>()
        .join(" AND ");
    builder.declare_relation(
        crate::model::RelationDecl::new(
            N::Runtime,
            "publication_manifests",
            1,
            crate::model::Authority::Derived,
            S::Sidecar,
            "One publication record: identity, workspace, parent, durable attempt, kind, the exact input and member vectors and the change windows read. The operational catalog is the authority for what is published; an export writes this row once as version 1 of a one-row Delta table together with the reader lease protecting its members, the workspace maintenance epoch and the operational store fingerprint, so an offline reader opens exactly those members.",
        )
        .pk(&["publication_id"])
        .granularity(crate::model::DerivationGranularity::Row)
        .columns(vec![
            T::key("publication_id", T::id(), "Publication identity."),
            column("workspace_id", T::id()),
            column("parent_publication_id", T::id()).optional(),
            column("attempt_id", T::id()),
            column("kind", T::enumeration("PublicationKind")),
            column("inputs", T::list(member())),
            column("members", T::list(member())),
            column("windows", T::list(window())),
            column("exported_at", micros()).optional(),
            column("export_lease_id", T::id()).optional(),
            column("export_expires_at", micros()).optional(),
            column("maintenance_epoch", T::nonnegative(i64::MAX)).optional(),
            column("store_fingerprint", T::hash()).optional(),
        ])
        .checks(
            [
                (
                    "parent_is_another_publication".into(),
                    "\"parent_publication_id\" IS DISTINCT FROM \"publication_id\"".into(),
                ),
                ("exported_together".into(), exported),
                (
                    "export_expires_after_export".into(),
                    "\"exported_at\" IS NULL OR \"export_expires_at\" > \"exported_at\"".into(),
                ),
            ]
            .into_iter()
            .collect(),
        ),
    );
}

fn declare_artifacts(builder: &mut RegistryBuilder) {
    enumeration(builder, "ArtifactReconstruction", ["none", "exact_release"]);
    relation(
        builder,
        N::Runtime,
        "artifact_descriptors",
        S::Sidecar,
        &["artifact_id"],
        vec![
            column("artifact_id", T::hash()),
            column("descriptor_version", T::nonnegative(i64::MAX)),
            column("profile", T::enumeration("PublicationKind")),
            column("profile_contract", T::hash()),
            column("requested_relations", T::list(T::id())),
            column("release_id", T::hash()),
            column("release_members", T::list(member())),
            column("semantic_identity", T::hash()),
            column(
                "implementation",
                T::structure(vec![
                    T::hash().with_name("source"),
                    T::hash().with_name("build"),
                    T::hash().with_name("registry"),
                    T::hash().with_name("algorithms"),
                ]),
            ),
            column("target_contract", T::hash()),
            column(
                "value_assumptions",
                T::list(T::structure(vec![
                    T::native(arrow_schema::DataType::Utf8).with_name("name"),
                    T::list(T::native(arrow_schema::DataType::UInt8)).with_name("canonical_value"),
                ])),
            ),
            column("reconstruction", T::enumeration("ArtifactReconstruction")),
        ],
        "Exact current-format artifact validity. Control members own output versions; local graph and Salsa handles are never persisted.",
    );
    relation(
        builder,
        N::Runtime,
        "release_checkpoints",
        S::Sidecar,
        &["consumer_id"],
        vec![
            column("consumer_id", T::id()),
            column("interpretation_version", T::nonnegative(i64::MAX)),
            column("admission_id", T::id()),
            column("base_release", T::hash()),
            column("target_release", T::hash()),
            column("base_members", T::list(member())),
            column("target_members", T::list(member())),
            column(
                "intervals",
                T::list(T::structure(vec![
                    T::native(arrow_schema::DataType::Utf8).with_name("table_uri"),
                    T::nonnegative(i64::MAX).with_name("from_version"),
                    T::nonnegative(i64::MAX).with_name("through_version"),
                ])),
            ),
        ],
        "Whole-release admission receipt and replay windows. A restarted compiler must admit its exact baseline before resuming.",
    );
}

fn declare_dependencies(builder: &mut RegistryBuilder) {
    enumeration(
        builder,
        "NativeDependencyEvidenceKind",
        [
            "absent",
            "present",
            "text",
            "identity",
            "identified_text",
            "fingerprint",
            "selection",
            "projection",
        ],
    );
    enumeration(
        builder,
        "NativeDependencyKind",
        [
            "operation",
            "input",
            "contract",
            "function",
            "rule",
            "setting",
            "policy",
            "provider",
            "scope",
            "observation",
        ],
    );
    relation(
        builder,
        N::Runtime,
        "native_dependencies",
        S::Sidecar,
        &["kind", "scope", "name"],
        vec![
            column("kind", T::enumeration("NativeDependencyKind")),
            column("scope", T::native(arrow_schema::DataType::Utf8)),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("evidence", dependency_evidence()),
        ],
        "Exact native input, implementation generation, policy and observation facts. Names alone never establish implementation equivalence.",
    );
}

fn dependency_evidence() -> T {
    let text = || T::native(arrow_schema::DataType::Utf8);
    let arms = [
        "text",
        "identity",
        "identified_text",
        "fingerprint",
        "selection",
        "projection",
    ];
    T::structure(vec![
        T::enumeration("NativeDependencyEvidenceKind").with_name("kind"),
        T::structure(vec![text().with_name("value")])
            .with_name("text")
            .optional(),
        T::structure(vec![T::id().with_name("value")])
            .with_name("identity")
            .optional(),
        T::structure(vec![
            T::id().with_name("identity"),
            text().with_name("text"),
        ])
        .with_name("identified_text")
        .optional(),
        T::structure(vec![T::hash().with_name("value")])
            .with_name("fingerprint")
            .optional(),
        member().with_name("selection").optional(),
        T::structure(vec![
            member().with_name("selection"),
            T::list(text()).with_name("columns"),
        ])
        .with_name("projection")
        .optional(),
    ])
    .with_alternative(
        &crate::model::TaggedAlternative::new("kind", arms.map(|name| (name.into(), name.into())))
            .with_unit("absent")
            .with_unit("present"),
    )
}

fn declare_retention(builder: &mut RegistryBuilder) {
    enumeration(
        builder,
        "ChangeKind",
        ["insert", "delete", "update_preimage", "update_postimage"],
    );
    relation(
        builder,
        N::Runtime,
        "change_events",
        S::Sidecar,
        &["table_uri", "commit_version", "kind", "row_key"],
        vec![
            column("table_uri", T::native(arrow_schema::DataType::Utf8)),
            column("relation_id", T::id()),
            column("contract_fingerprint", T::hash()),
            column("commit_version", T::nonnegative(i64::MAX)),
            column("kind", T::enumeration("ChangeKind")),
            column("row_key", T::row_key()),
            column(
                "committed_at",
                T::native(crate::model::extension::timestamp_storage()),
            )
            .optional(),
        ],
        "Typed native CDF identity; each event travels with its complete declared before/after row value.",
    );
    // Why a table version stays reachable (Plan 22 X10): a live publication selects it
    // (member or input); a live publication read it within a change window; or it lies
    // under the member prefix of a live, unpublished intent.
    enumeration(
        builder,
        "RetentionReason",
        ["publication", "attempt", "changes"],
    );
    let columns = vec![
        T::key(
            "table_uri",
            T::native(arrow_schema::DataType::Utf8),
            "Exact Delta table.",
        ),
        T::key(
            "from_version",
            T::nonnegative(i64::MAX),
            "First protected version, inclusive.",
        ),
        T::key(
            "through_version",
            T::nonnegative(i64::MAX),
            "Last protected version, inclusive.",
        ),
        T::key(
            "reason",
            T::enumeration("RetentionReason"),
            "Why data and history remain reachable.",
        ),
    ];
    builder.declare_relation(
        crate::model::RelationDecl::new(
            N::Runtime,
            "retained_versions",
            1,
            crate::model::Authority::Derived,
            S::Sidecar,
            "Native retention query outputs; ranges preserve each exact version and change window.",
        )
        .pk(&["table_uri", "from_version", "through_version", "reason"])
        .columns(columns)
        .granularity(crate::model::DerivationGranularity::Rule)
        .checks(
            [(
                "ordered_range".into(),
                "from_version <= through_version".into(),
            )]
            .into_iter()
            .collect(),
        ),
    );
    relation(
        builder,
        N::Runtime,
        "maintenance_outcomes",
        S::Sidecar,
        &["table_uri"],
        vec![
            column("table_uri", T::native(arrow_schema::DataType::Utf8)),
            column("delta_version", T::nonnegative(i64::MAX)),
            column(
                "deleted_files",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column("deleted_logs", T::nonnegative(i64::MAX)),
        ],
        "Actual native Delta maintenance outcomes; no bespoke data file deletion.",
    );
}
/// Durable products are consumer contracts, independent of stage execution inventory.
pub(super) fn declare_profiles(builder: &mut RegistryBuilder) {
    use std::collections::BTreeSet;
    let common = BTreeSet::from([
        "runtime.artifact_descriptors".to_owned(),
        "runtime.diagnostics_findings".to_owned(),
        "provenance.derivations".to_owned(),
    ]);
    builder.declare_artifact_profile(
        "run",
        BTreeSet::from([
            "runtime.artifact_descriptors".to_owned(),
            "authored.modeling_declarations".to_owned(),
            "runtime.run_lineage".to_owned(),
            "runtime.candidate_assessments".to_owned(),
            "runtime.modeling_checks".to_owned(),
        ]),
    );
    let mut source = common.clone();
    source.extend(
        builder
            .declared_relations()
            .iter()
            .filter(|relation| {
                matches!(
                    relation.authority,
                    crate::model::Authority::Authored | crate::model::Authority::Reference
                ) && relation.snapshot_class != S::Sidecar
            })
            .map(|relation| relation.key.qualified_name()),
    );
    builder.declare_artifact_profile("source", source.clone());
    builder.declare_artifact_profile("case", source);
    builder.declare_artifact_profile("diagnostics", common);
    builder.declare_artifact_profile(
        "inspection",
        BTreeSet::from(["runtime.artifact_descriptors".to_owned()]),
    );
}

/// One exact member selection: a qualified table name, its relation contract and the
/// exact Delta version (and optional revision slice) selected. The registry named
/// structure `MemberDescriptor` (Plan 22 X11): every relation that lists members
/// references this one declaration, and the generators emit it once.
pub(super) fn member() -> T {
    T::structure(vec![
        T::native(arrow_schema::DataType::Utf8)
            .with_name("catalog_name")
            .with_nullable(false),
        T::native(arrow_schema::DataType::Utf8)
            .with_name("schema_name")
            .with_nullable(false),
        T::native(arrow_schema::DataType::Utf8)
            .with_name("table_name")
            .with_nullable(false),
        T::id().with_name("relation_id").with_nullable(false),
        T::nonnegative(i64::from(u32::MAX))
            .with_name("relation_version")
            .with_nullable(false),
        T::hash()
            .with_name("contract_fingerprint")
            .with_nullable(false),
        T::native(arrow_schema::DataType::Utf8)
            .with_name("table_uri")
            .with_nullable(false),
        T::nonnegative(i64::MAX)
            .with_name("delta_version")
            .with_nullable(false),
        selection().with_name("selection"),
    ])
    .named("MemberDescriptor")
}

/// One inclusive version window of a Delta table: the registry named structure
/// `VersionWindow`.
fn window() -> T {
    T::structure(vec![
        T::native(arrow_schema::DataType::Utf8)
            .with_name("table_uri")
            .with_nullable(false),
        T::nonnegative(i64::MAX)
            .with_name("from_version")
            .with_nullable(false),
        T::nonnegative(i64::MAX)
            .with_name("through_version")
            .with_nullable(false),
    ])
    .named("VersionWindow")
}

fn selection() -> T {
    let alternative =
        crate::model::TaggedAlternative::new("kind", [("revision".into(), "revision".into())])
            .with_unit("full");
    T::structure(vec![
        T::enumeration("MemberSelectionKind").with_name("kind"),
        T::structure(vec![
            T::native(arrow_schema::DataType::Utf8).with_name("column"),
            T::id().with_name("revision_id"),
        ])
        .with_name("revision")
        .optional(),
    ])
    .with_alternative(&alternative)
}
