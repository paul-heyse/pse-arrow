// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Canonical substrate envelopes. Scientific payload shapes remain at their semantic owner.

use super::declarations::{column, declaration};
use crate::{
    RegistryBuilder,
    model::{FieldContract, Namespace, SnapshotClass},
};
use arrow_schema::DataType;

/// Physical access path owned by the canonical declaration route.
#[derive(Clone, Copy, Debug)]
pub struct NativeIndex {
    /// Unique name in the canonical database.
    pub name: &'static str,
    /// Declared canonical relation name.
    pub table: &'static str,
    /// Ordered declared columns.
    pub fields: &'static [&'static str],
    /// Whether the tuple is a native uniqueness constraint.
    pub unique: bool,
}
macro_rules! indexes {
    ($($name:ident: $table:ident [$($field:ident),+] $unique:literal),* $(,)?) => {
        /// Required access paths for bounded source, product and retention operations.
        pub const INDEXES: &[NativeIndex] = &[$(NativeIndex {
            name: stringify!($name), table: stringify!($table),
            fields: &[$(stringify!($field)),+], unique: $unique,
        }),*];
    };
}
indexes! {
    membership_selection: canonical_memberships [problem, scope, name, from_sequence] false,
    membership_active: canonical_memberships [problem, to_sequence] false,
    version_logical: canonical_versions [logical] false,
    revision_operation: canonical_revisions [operation] true,
    revision_sequence: canonical_revisions [problem, sequence] true,
    protection_revision: canonical_protections [revision, expires_at] false,
    root_revision: canonical_roots [revision] false,
    product_discovery: canonical_products [problem, producer] false,
    edge_source: canonical_edges [source_version] false,
    membership_pages: canonical_memberships [problem, key] false,
    membership_version: canonical_memberships [version] false,
    root_interval: canonical_roots [problem, sequence] false,
    protection_interval: canonical_protections [problem, sequence] false,
    reclaimed_interval: canonical_reclaimed_ranges [problem, from_sequence] false,
    edge_target: canonical_edges [target_scope, target_name] false,
    membership_logical: canonical_memberships [problem, logical, from_sequence] false,
    manifest_kind: canonical_version_manifests [kind] false,
    payload_blocks: canonical_payload_blocks [version, ordinal] true,
    edge_ordinal: canonical_edges [source_version, ordinal] true,
    stage_edits: canonical_staged_edits [stage, ordinal] true,
    staged_version: canonical_staged_edits [version, stage] false,
    stage_expiry: canonical_stages [activated, abandoned, expires_at] false,
    stage_cleanup: canonical_stages [problem, cleanup_complete, expires_at, key] false,
}

pub(super) fn declare(builder: &mut RegistryBuilder) {
    let text = || FieldContract::native(DataType::Utf8);
    let uint = || FieldContract::native(DataType::UInt64);
    let bytes = || FieldContract::native(DataType::Binary);
    let flag = || FieldContract::native(DataType::Boolean);
    let timestamp = || FieldContract::native(crate::model::extension::timestamp_micros_storage());
    let declarations = [
        (
            "canonical_interpretations",
            vec![
                column("key", text()),
                column("interpretation", text()),
                column("schema_digest", text()),
            ],
            "Installed schema, codec and structural operation interpretation.",
        ),
        (
            "canonical_problems",
            vec![
                column("key", text()),
                column("head", text()),
                column("sequence", uint()),
            ],
            "Linear problem head; guarded compare-and-set advances it.",
        ),
        (
            "canonical_revisions",
            vec![
                column("key", text()),
                column("problem", text()),
                column("sequence", uint()),
                column("parent", text()).optional(),
                column("operation", text()),
                column("request", bytes()),
                column("interpretation", text()),
            ],
            "Immutable authored revision and idempotent edit operation.",
        ),
        (
            "canonical_versions",
            vec![
                column("key", text()),
                column("logical", text()),
                column("kind", text()),
                column("payload", bytes()),
                column("interpretation", text()),
            ],
            "Immutable logical object version; payload codec belongs to the scientific declaration owner.",
        ),
        (
            "canonical_memberships",
            vec![
                column("key", text()),
                column("problem", text())
                    .with_fk("runtime.canonical_problems", "key")
                    .with_graph_endpoint(true),
                column("scope", text()),
                column("name", text()),
                column("logical", text()),
                column("version", text())
                    .with_fk("runtime.canonical_version_manifests", "key")
                    .with_graph_endpoint(false),
                column("from_sequence", uint()),
                column("to_sequence", uint()).optional(),
            ],
            "Changed membership interval and native problem-to-version graph edge; historical selection is explicit.",
        ),
        (
            "canonical_guards",
            vec![column("key", text()), column("generation", uint())],
            "Named conflict register; generation is separate from semantic eligibility.",
        ),
        (
            "canonical_edges",
            vec![
                column("key", text()),
                column("source_version", text()),
                column("ordinal", uint()),
                column("target_scope", text()),
                column("target_name", text()),
            ],
            "Authored structural reference resolved at a selected immutable revision.",
        ),
        (
            "canonical_version_manifests",
            vec![
                column("key", text()),
                column("logical", text()),
                column("kind", text()),
                column("interpretation", text()),
                column("payload_digest", text()),
                column("payload_len", uint()),
                column("block_count", uint()),
                column("reference_count", uint()),
                column("creator_stage", text()),
                column("closed", flag()),
            ],
            "Closed immutable source payload manifest; transport blocks do not change scientific meaning.",
        ),
        (
            "canonical_payload_blocks",
            vec![
                column("key", text()),
                column("version", text()),
                column("ordinal", uint()),
                column("payload", bytes()),
                column("digest", text()),
            ],
            "Bounded exact source payload transport block.",
        ),
        (
            "canonical_version_receipts",
            vec![column("key", text()), column("content_digest", text())],
            "Permanent immutable version identity fence, retained after payload reclamation.",
        ),
        (
            "canonical_stages",
            vec![
                column("key", text()),
                column("problem", text()),
                column("expected_head", text()).optional(),
                column("request_digest", text()),
                column("expires_at", timestamp()),
                column("closed", flag()),
                column("activated", flag()),
                column("edit_count", uint()),
                column("generation", uint()),
                column("abandoned", flag()),
                column("cleanup_complete", flag()),
            ],
            "Fenced bounded staging lease and permanent idempotent operation identity.",
        ),
        (
            "canonical_staged_edits",
            vec![
                column("key", text()),
                column("stage", text()),
                column("ordinal", uint()),
                column("logical", text()),
                column("scope", text()),
                column("name", text()),
                column("version", text()).optional(),
            ],
            "Source edit metadata activated atomically after immutable payload staging closes.",
        ),
        (
            "canonical_protections",
            vec![
                column("key", text()),
                column("problem", text()),
                column("revision", text()),
                column("sequence", uint()),
                column("expires_at", timestamp()),
                column("released", flag()),
            ],
            "Protected immutable read/preparation selection; expiry fences further use.",
        ),
        (
            "canonical_roots",
            vec![
                column("key", text()),
                column("problem", text()),
                column("revision", text()),
                column("sequence", uint()),
                column("owner_kind", text()),
                column("owner", text()),
            ],
            "Explicitly retained immutable revision root for a product, run, analysis or history.",
        ),
        (
            "canonical_reclaimed_ranges",
            vec![
                column("key", text()),
                column("problem", text()),
                column("from_sequence", uint()),
                column("to_sequence", uint()),
            ],
            "Bounded reclamation tombstone; receipt retention does not resurrect reclaimed source selection.",
        ),
        (
            "canonical_products",
            vec![
                column("key", text()),
                column("problem", text()),
                column("revision", text()),
                column("request", bytes()),
                column("payload", bytes()),
                column("dependencies", bytes()),
                column("producer", text()),
                column("interpretation", text()),
            ],
            "Portable admitted scientific description and complete dependency witness; contains no native handles.",
        ),
    ];
    for (name, columns, doc) in declarations {
        builder.declare_relation(declaration(
            Namespace::Runtime,
            name,
            1,
            SnapshotClass::Sidecar,
            &["key"],
            columns,
            doc,
        ));
    }
}
