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
        ["relations", "source", "model", "case", "problem", "run"],
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
    enumeration(
        builder,
        "RetentionReason",
        ["publication", "output", "attempt", "changes"],
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
/// Profile membership follows domain output signatures, never predecessor stages or
/// a stored execution graph. Partial checkpoints explicitly use `relations`.
pub(super) fn declare_profiles(builder: &mut RegistryBuilder) {
    let outputs = |algorithms: &[&str]| {
        builder
            .declared_algorithms()
            .iter()
            .filter(|algorithm| algorithms.contains(&algorithm.name))
            .flat_map(|algorithm| {
                algorithm
                    .outputs
                    .iter()
                    .map(|output| output.relation.clone())
            })
            .collect::<std::collections::BTreeSet<_>>()
    };
    let source = outputs(&["source"]);
    let model = outputs(&["source", "P3"]);
    let problem = outputs(&["source", "P3", "P4", "P5", "P6", "P7", "P8", "P9", "P10"]);
    builder.declare_artifact_profile("source", source.clone());
    builder.declare_artifact_profile("case", source);
    builder.declare_artifact_profile("model", model);
    builder.declare_artifact_profile("problem", problem);
    builder.declare_artifact_profile(
        "run",
        [
            "runtime.numerical_programs",
            "runtime.jacobian_coordinates",
            "runtime.solver_outcomes",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    );
}

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
