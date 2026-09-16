// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The `pse.manifest.v2` envelope, declared (blueprint §20.2).
//!
//! This declaration controls the generated Rust/Python `Manifest` types and reference
//! documentation (ADR-0051, ADR-0060). The manifest's own checksum is held
//! by the ref, so there is no field for it here: a document cannot contain its own digest.
//!
//! Evidence, timestamps and alternative encodings are in the envelope but out of logical
//! membership (§5.3 step 7). The snapshot-kind profile, not an optional field's presence,
//! decides which dependencies are required.

use crate::builder::RegistryBuilder;
use crate::model::{ManifestField, ManifestRustBinding, ManifestSpec, ManifestType};

/// Declares the manifest envelope.
pub fn declare(builder: &mut RegistryBuilder) {
    builder.declare_manifest(ManifestSpec::new(
        ManifestSpec::VERSION,
        ManifestSpec::MEMBERSHIP_PROFILE,
        fields(),
    ));
}

/// The `pse.manifest.v2` fields, in blueprint §20.2 order.
fn fields() -> Vec<ManifestField> {
    let mut out = identity_fields();
    out.extend(membership_fields());
    out.extend(dependency_fields());
    out
}

/// The envelope's own identity: version, kind, snapshot ID, profile, timestamp, registry.
fn identity_fields() -> Vec<ManifestField> {
    vec![
        ManifestField::new(
            "manifest_version",
            ManifestType::Text,
            "Always `pse.manifest.v2`. A different value is a different envelope, not a variant.",
        ),
        ManifestField::new(
            "snapshot_kind",
            ManifestType::Text,
            "`model`, `case`, `stage` or `run`. The kind's profile decides which dependencies are \
             required.",
        )
        .with_rust(ManifestRustBinding::SnapshotKind),
        ManifestField::new(
            "snapshot_id",
            ManifestType::Hash,
            "The `pse.snapshot.v2` frame digest of the membership below (blueprint §5.3 step 7).",
        )
        .with_rust(ManifestRustBinding::SnapshotId),
        ManifestField::new(
            "membership_profile",
            ManifestType::Text,
            "Always `pse.snapshot.v2`: the profile the `snapshot_id` was computed under.",
        ),
        ManifestField::new(
            "created_at",
            ManifestType::Timestamp,
            "When the manifest was written. Excluded from logical membership.",
        ),
        ManifestField::new(
            "schema_registry_fingerprint",
            ManifestType::Hash,
            "The registry these relations were written under.",
        ),
    ]
}

/// The semantic membership and the packages it was authored from.
fn membership_fields() -> Vec<ManifestField> {
    vec![
        ManifestField::new(
            "relations",
            ManifestType::list(relation_member()),
            "The complete semantic membership, including required empty relations.",
        ),
        ManifestField::new(
            "packages",
            ManifestType::list(ManifestType::Struct(vec![
                ManifestField::new("package_id", ManifestType::Id, "The package."),
                ManifestField::new("version", ManifestType::Text, "Its resolved version."),
                ManifestField::new("logical_hash", ManifestType::Hash, "Its logical digest.")
                    .with_rust(ManifestRustBinding::LogicalHash),
            ])),
            "Every package whose content this snapshot depends on.",
        ),
    ]
}

/// Everything the result's meaning depends on besides its own rows.
fn dependency_fields() -> Vec<ManifestField> {
    vec![
        admission_binding(),
        ManifestField::new(
            "compiler",
            ManifestType::Struct(vec![
                ManifestField::new("version", ManifestType::Text, "The compiler version."),
                ManifestField::new(
                    "passes",
                    ManifestType::list(ManifestType::Struct(vec![
                        ManifestField::new("pass_id", ManifestType::Id, "The pass."),
                        ManifestField::new("version", ManifestType::Text, "Its version."),
                    ])),
                    "The passes that produced this snapshot, with their versions.",
                ),
            ]),
            "What compiled it.",
        ),
        ManifestField::new(
            "engine_profile",
            ManifestType::optional(ManifestType::Struct(vec![
                ManifestField::new("engine_profile_id", ManifestType::Id, "The profile."),
                ManifestField::new("content_hash", ManifestType::Hash, "Its digest."),
            ])),
            "The declared relational engine (blueprint §14.2 rule 5). Required for a stage whose \
             pass executes plans.",
        ),
        ManifestField::new(
            "numerical_policy",
            ManifestType::optional(ManifestType::Struct(vec![
                ManifestField::new("policy_id", ManifestType::Id, "The policy."),
                ManifestField::new("content_hash", ManifestType::Hash, "Its digest."),
            ])),
            "The numerical policy in force (blueprint §6.11).",
        ),
        ManifestField::new(
            "toolchain",
            ManifestType::Struct(vec![
                ManifestField::new(
                    "lockfile_hash",
                    ManifestType::Hash,
                    "The resolved dependency graph.",
                ),
                ManifestField::new(
                    "canonicalization",
                    ManifestType::Text,
                    "Always `pse.canon.v2` (ADR-0045).",
                ),
            ]),
            "What built it.",
        ),
        ManifestField::new(
            "kernels",
            ManifestType::list(ManifestType::Struct(vec![
                ManifestField::new("kernel_id", ManifestType::Id, "The kernel."),
                ManifestField::new("version", ManifestType::Text, "Its version."),
                ManifestField::new("digest", ManifestType::Hash, "Its artifact digest."),
            ])),
            "Every kernel whose implementation the result depends on.",
        ),
        ManifestField::new(
            "semantic_parents",
            ManifestType::list(ManifestType::Struct(vec![
                ManifestField::new(
                    "role",
                    ManifestType::Text,
                    "The input binding this parent supplied, for example `input.typed_math`.",
                ),
                ManifestField::new("snapshot_id", ManifestType::Hash, "The parent snapshot.")
                    .with_rust(ManifestRustBinding::SnapshotId),
            ])),
            "The snapshots this one derives from. A duplicate role is an error.",
        )
        .with_rust(ManifestRustBinding::SnapshotParents),
        ManifestField::new(
            "evidence",
            ManifestType::list(ManifestType::Struct(vec![
                ManifestField::new(
                    "kind",
                    ManifestType::Text,
                    "What the evidence is, for example `datafusion_proto`.",
                ),
                ManifestField::new(
                    "canonical",
                    ManifestType::Bool,
                    "Always false for plan bytes: noncanonical evidence (ADR-0044).",
                ),
                ManifestField::new(
                    "encoding_checksum",
                    ManifestType::Hash,
                    "The digest of the stored bytes.",
                )
                .with_rust(ManifestRustBinding::EncodingChecksum),
                ManifestField::new("codec_version", ManifestType::Text, "The platform codec."),
                ManifestField::new("path", ManifestType::Text, "Where it is stored."),
            ])),
            "Diagnostic artifacts. Excluded from logical membership.",
        ),
    ]
}

/// One `relations[]` member.
fn relation_member() -> ManifestType {
    ManifestType::Struct(vec![
        ManifestField::new(
            "port",
            ManifestType::Text,
            "The member port; `<namespace>/<relation id>` for a model or case member.",
        ),
        ManifestField::new("namespace", ManifestType::Text, "The relation's namespace."),
        ManifestField::new("relation_id", ManifestType::Id, "The relation."),
        ManifestField::new("name", ManifestType::Text, "The relation name."),
        ManifestField::new("version", ManifestType::U32, "The schema version.")
            .with_rust(ManifestRustBinding::SchemaVersion),
        ManifestField::new(
            "logical_hash",
            ManifestType::Hash,
            "The `pse.canon.v2` logical digest: what the relation means.",
        )
        .with_rust(ManifestRustBinding::LogicalHash),
        ManifestField::new("rows", ManifestType::U64, "The row count."),
        ManifestField::new(
            "encodings",
            ManifestType::list(ManifestType::Struct(vec![
                ManifestField::new(
                    "format",
                    ManifestType::Text,
                    "`arrow_ipc_file` or `parquet`.",
                )
                .with_rust(ManifestRustBinding::EncodingFormat),
                ManifestField::new("writer_version", ManifestType::Text, "What wrote it."),
                ManifestField::new(
                    "encoding_checksum",
                    ManifestType::Hash,
                    "The digest of the stored bytes: whether this object is intact. Never \
                     substituted for the logical hash (ADR-0045).",
                )
                .with_rust(ManifestRustBinding::EncodingChecksum),
                ManifestField::new("bytes", ManifestType::U64, "The stored size."),
                ManifestField::new("path", ManifestType::Text, "Where it is stored."),
            ])),
            "One entry per stored encoding of the same logical relation.",
        ),
    ])
}

fn admission_binding() -> ManifestField {
    ManifestField::new(
        "admission_binding",
        ManifestType::optional(ManifestType::Struct(vec![
            ManifestField::new(
                "encoding_checksum",
                ManifestType::Hash,
                "Checksum of the complete immutable admission-binding artifact.",
            )
            .with_rust(ManifestRustBinding::EncodingChecksum),
        ])),
        "Checksum of the immutable exact parent and invocation bindings. Required for \
         context-dependent artifacts; excluded from logical membership (ADR-0067).",
    )
}
