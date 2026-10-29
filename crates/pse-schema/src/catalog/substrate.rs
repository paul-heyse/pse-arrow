// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Canonical substrate envelopes. Scientific payload shapes remain at their semantic owner.

use super::declarations::{canonical_declaration as declaration, column};
use crate::{
    RegistryBuilder,
    model::{FieldContract, Namespace, SnapshotClass},
};
use arrow_schema::DataType;

/// Maximum logical edits in one closed canonical source publication.
pub const SOURCE_PACKAGE_EDITS: usize = 8_192;

/// Exact schema/codec/structural-operation interpretation.
pub const INTERPRETATION: &str = "pse.substrate.v10";

/// Native schema admission generation; scientific row interpretation remains v10.
pub const SCHEMA_VERSION: u64 = 3;
/// Supported predecessor of native epoch admission, retained as an immutable migration input.
pub const PRE_EPOCH_SCHEMA_DIGEST: &str =
    "b6ee758d774a6510a9db894c9062cfe103439bdecc049efeaab8cd26581a21a3";
/// Raw bytes of the frozen generated predecessor declaration, distinct from its schema/codec digest.
pub const PRE_EPOCH_DECLARATION_SHA256: &str =
    "68eb820178f7cdc1cb042395b836c88cb9fbc5ead42a20d1d65099f4626d066b";
/// Exact native admission predecessor; its scientific interpretation is still v10.
pub const SCHEMA_V2_VERSION: u64 = 2;
/// Frozen schema/codec digest, never recomputed from the current generator.
pub const SCHEMA_V2_DIGEST: &str =
    "04c2b46a1ee5d153244089e816e1fc2586c1208a8fce5e48998f22a091c55bcc";
/// Raw bytes of the immutable schema-v2 declaration.
pub const SCHEMA_V2_DECLARATION_SHA256: &str =
    "4b082f4d8bdc6a17ff3ce00a1cf30c8569422a51e7678f1e9e6bc0c9871e1aae";

/// Self-contained scientific IPC admission, independent of derived indexes.
pub const RESULT_BLOCK_BYTES: usize = 32 * 1024 * 1024;
/// Preferred physical native IPC packing extent, flushed at acknowledgement boundaries.
pub const RESULT_FILE_TARGET_BYTES: usize = 16 * 1024 * 1024;
/// Native CBOR index/descriptor envelope admission per append.
pub const RESULT_INDEX_BYTES: usize = 128 * 1024;
/// Closed admitted-batch statistics envelope, independent of scientific payload size.
pub const RESULT_STATISTICS_BYTES: usize = 16 * 1024;
/// Codec version for exact statistics produced with admitted native result files.
pub const RESULT_STATISTICS_VERSION: u32 = 2;
/// Maximum complete encoded body held directly in the canonical transaction.
pub const RESULT_INLINE_BYTES: usize = 64 * 1024;
/// Target append envelope including inline body bytes, below actual RPC admission.
pub const RESULT_APPEND_TARGET_BYTES: usize = 2 * 1024 * 1024;
/// Maximum metadata rows in one artifact attachment transaction.
pub const ARTIFACT_APPEND_ROWS: usize = 64;
/// Complete uncompressed native RPC message admission.
pub const RESULT_MESSAGE_BYTES: usize = 4 * 1024 * 1024;
/// Maximum canonical metadata identity/string length at the response boundary.
pub const METADATA_ID_BYTES: usize = 4096;
/// Every valid index object contains at least the nonempty `key` field. The
/// minimal CBOR map `{key: "a"}` occupies seven bytes; other mandatory fields
/// only increase its extent. This ceiling protects loops, never chooses blocks.
pub const RESULT_INDEX_RECORDS: usize = RESULT_INDEX_BYTES / 7;

/// Artifact holder kinds and their shared attach/release fencing behavior.
pub const ARTIFACT_HOLDERS: &[(&str, &str, &str)] = &[
    (
        "study",
        "fn::pse_execution_v1::register($rpc_expiry,'study:'+$obligation.owner); IF (SELECT * FROM ONLY type::record('canonical_study_retirements',$obligation.owner))!=NONE { THROW 'PSE_REFUSAL:study_artifact_owner_retired'; };",
        "fn::pse_execution_v1::register($rpc_expiry,'study:'+$obligation.owner); IF (SELECT * FROM ONLY type::record('canonical_study_retirements',$obligation.owner))=NONE { THROW 'PSE_REFUSAL:study_input_release_requires_explicit_study_retirement'; };",
    ),
    (
        "analysis",
        "LET $analysis=SELECT * FROM ONLY type::record('canonical_analyses',$obligation.owner) FOR UPDATE; IF $analysis=NONE OR $analysis.retiring { THROW 'PSE_REFUSAL:analysis_artifact_owner_retired'; };",
        "LET $analysis=SELECT * FROM ONLY type::record('canonical_analyses',$obligation.owner) FOR UPDATE; IF $analysis=NONE OR !$analysis.retiring { THROW 'PSE_REFUSAL:analysis_input_release_requires_explicit_analysis_retirement'; };",
    ),
    ("user", "", ""),
    ("project", "", ""),
    ("audit", "", ""),
];

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
    ($($name:ident: $table:ident [$($field:ident $(.$path:ident)*),+] $unique:literal),* $(,)?) => {
        /// Required access paths for bounded source, product and retention operations.
        pub const INDEXES: &[NativeIndex] = &[$(NativeIndex {
            name: stringify!($name), table: stringify!($table),
            fields: &[$(concat!(stringify!($field) $(,".",stringify!($path))*)),+], unique: $unique,
        }),*];
    };
}
indexes! {
    file_intent: canonical_file_instances [intent] true,
    file_instance_owner: canonical_file_instances [owner, state, key] false,
    file_borrower_instance: canonical_file_borrowers [instance, released, key] false,
    file_borrower_owner: canonical_file_borrowers [owner_kind, owner, key] false,
    file_owner_active: canonical_file_owners [drained, key] false,
    manifest_ticket_selection: canonical_manifest_tickets [kind, selection, released, key] false,
    manifest_ticket_owner: canonical_manifest_tickets [owner, released, key] false,
    guard_incarnation: canonical_guards [incarnation] true,
    membership_selection: canonical_memberships [problem, scope, name, from_sequence] false,
    membership_active: canonical_memberships [problem, to_sequence] false,
    version_logical: canonical_versions [logical] false,
    revision_operation: canonical_revisions [operation] true,
    revision_sequence: canonical_revisions [problem, sequence] true,
    protection_revision: canonical_protections [revision, expires_at] false,
    root_revision: canonical_roots [revision] false,
    root_owner: canonical_roots [owner_kind, owner, key] false,
    product_revision_discovery: canonical_product_associations [problem, revision, key] false,
    product_discovery: canonical_product_associations [problem, producer, key] false,
    product_descriptor_association: canonical_product_associations [descriptor, key] false,
    product_content_association: canonical_product_associations [content, key] false,
    product_receipt_association: canonical_product_associations [receipt, key] false,
    edge_source: canonical_edges [source_version] false,
    membership_pages: canonical_memberships [problem, key] false,
    membership_scope_pages: canonical_memberships [problem, scope, key] false,
    membership_scope_head_pages: canonical_memberships [problem, scope, to_sequence, key] false,
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
    run_order: canonical_runs [problem, sequence] true,
    run_revision: canonical_runs [problem, revision, sequence] false,
    run_research_selection: canonical_runs [research_selection, sequence, key] false,
    run_research_dataset: canonical_runs [research_dataset, sequence, key] false,
    run_terminal_selection: canonical_runs [problem, terminal_class, sequence] false,
    attempt_run: canonical_attempts [run, generation] true,
    attempt_live: canonical_attempts [run, phase, generation] false,
    result_set_attempt: canonical_result_sets [attempt, name] true,
    result_batch_order: canonical_result_batches [result_set, ordinal] true,
    result_batch_inline_body: canonical_result_batches [inline_key, key] false,
    execution_operation_run: canonical_execution_operations [run, kind] false,
    execution_operation_compaction: canonical_execution_operations [run, minimal, key] false,
    history_run_locators: canonical_history_locators [run, key] false,
    history_study_locators: canonical_history_locators [study, key] false,
    history_obligation_owner: canonical_history_obligations [owner_kind, owner, released, key] false,
    history_identity_run: canonical_history_identities [run, category, key] false,
    run_source_cleanup: canonical_run_sources [run, key] false,
    problem_run_cleanup: canonical_problem_runs [run, key] false,
    result_block_range: canonical_result_blocks [result_set, output, partition, start] false,
    result_block_page: canonical_result_blocks [result_set, output, partition, ordinal] false,
    result_block_batch: canonical_result_blocks [batch, ordinal] true,
    result_output_range: canonical_result_block_outputs [result_set, output, partition, start] false,
    result_output_batch: canonical_result_block_outputs [batch, key] false,
    result_read_attempt: canonical_result_protections [attempt, key] false,
    result_read_run: canonical_result_protections [run, key] false,
    result_seed_eligibility: canonical_result_seeds [layout, preparation, backend, run_sequence, attempt_generation, step] false,
    result_seed_attempt: canonical_result_seeds [attempt, step] false,
    problem_run_edge: canonical_problem_runs [problem, run] true,
    run_source_edge: canonical_run_sources [run, revision] true,
    study_discovery: canonical_studies [active, terminal, key] false,
    study_run: canonical_studies [run] true,
    study_points_order: canonical_study_points [study, ordinal] true,
    study_point_run: canonical_study_points [run] true,
    study_point_occurrence: canonical_study_points [study, occurrence] true,
    study_candidates: canonical_study_points [study, settled, assigned, ordinal] false,
    study_ready_candidates: canonical_study_readiness [study, eligible, ordinal] false,
    study_ready_notifications: canonical_study_readiness [study, notify, ordinal] false,
    study_ready_order: canonical_study_readiness [study, ordinal] true,
    study_dependencies: canonical_study_dependencies [dependent, key] false,
    study_dependents: canonical_study_dependencies [predecessor, key] false,
    analysis_nodes: canonical_analysis_nodes [analysis, key] false,
    analysis_edges: canonical_analysis_edges [analysis, key] false,
    analysis_edge_source: canonical_analysis_edges [source, key] false,
    analysis_edge_target: canonical_analysis_edges [target, key] false,
    artifact_blocks: canonical_artifact_blocks [artifact, ordinal] true,
    artifact_obligations: canonical_artifact_obligations [artifact, key] false,
    artifact_owner: canonical_artifact_obligations [owner_kind, owner, key] false,
    artifact_reads: canonical_artifact_reads [artifact, key] false,
    artifact_staging_run: canonical_artifact_staging_inputs [run, key] false,
    artifact_staging_owner: canonical_artifact_staging_inputs [artifact, key] false,
    study_inputs: canonical_study_inputs [study, solution] true,
    analysis_inputs: canonical_analysis_inputs [analysis, key] false,
    analysis_input_artifact: canonical_analysis_inputs [artifact, analysis] false,
}

pub(super) fn declare(builder: &mut RegistryBuilder) {
    let text = || FieldContract::native(DataType::Utf8);
    let uint = || FieldContract::native(DataType::UInt64);
    let bytes = || FieldContract::native(DataType::Binary);
    let flag = || FieldContract::native(DataType::Boolean);
    let timestamp = || FieldContract::native(crate::model::extension::timestamp_micros_storage());
    let declarations = [
        (
            "canonical_control_actors",
            vec![column("key", text()).canonical_record_key(),column("password",text()).canonical_readonly(),column("enabled",flag()),column("browse",flag()),column("cancel",flag())],
            "Privilegedly issued database-scoped control actors. No record user can select credentials or modify grants directly.",
            crate::model::CanonicalRelation {retention:crate::model::CanonicalRetention::Durable,backup:crate::model::CanonicalBackup::Required,inline_vertex:false,enforced:false,generation_references:false},
        ),
        (
            "canonical_control_commands",
            vec![column("key",text()).canonical_record_key(),column("issuing_store",text()).canonical_readonly(),column("actor",text()).canonical_readonly(),column("operation_id",text()).canonical_readonly(),column("study",text()).canonical_readonly(),column("receipt",bytes()).canonical_readonly()],
            "Immutable actor/store-scoped cancellation intent and exact decision retained through database retirement; absence never authorizes mutation replay.",
            crate::model::CanonicalRelation {retention:crate::model::CanonicalRetention::Receipt,backup:crate::model::CanonicalBackup::Required,inline_vertex:false,enforced:false,generation_references:false},
        ),
        (
            "canonical_maintenance_operations",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::MaintenanceOperation),
                column("operation", text()),
                column("generation", text()),
                column("phase", text()),
                column("source", text()),
                column("target", text()),
                column("kind", text()),
                column("after", text()),
                column("next", text()),
                column("count", uint()),
                column("complete", flag()),
            ],
            "Current fenced privileged maintenance generation and immutable exact-operation page effects. Historical installation remains separately privileged; page replay never repeats authority rotation or placement mutation.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Receipt, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_maintenance_activations",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::MaintenanceActivation),
                column("input_sha256", text()),
                column("predecessor", bytes()),
                column("interpretation", text()),
                column("schema_digest", text()),
                column("study_module_sha256", text()),
                column("issuing_store", text()),
            ],
            "Exact immutable schema activation effect receipt under the original rollout nonce; marker and issuing identity publication are atomic with this acknowledgement.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Receipt, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_interpretations",
            vec![
                column("key", text()),
                column("interpretation", text()),
                column("schema_digest", text()),
                column("study_module_sha256", text()),
                column("issuing_store", text()),
                column("payload_incarnation", text()),
                column("schema_version", uint()),
                column("schema_epoch", text()),
                column("admission_open", flag()),
            ],
            "Installed schema, codec and structural operation interpretation with transactional epoch admission.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_file_admission",
            vec![
                column("key", text()),
                column("storage_incarnation", text()),
                column("frozen", flag()),
                column("gc_frozen", flag()),
            ],
            "Selected file placement incarnation and jointly locked admission/collection freeze; ordinary opening never provisions missing placement.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::LiveAuthority, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_file_owners",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::FileOwner),
                column("issuing_store", text()),
                column("storage_incarnation", text()),
                column("boot", text()),
                column("pid_namespace", text()),
                column("pid", uint()),
                column("start", text()),
                column("invocation", text()).optional(),
                column("unit", text()).optional(),
                column("group", text()).optional(),
                column("group_inode", uint()).optional(),
                column("drained", flag()),
            ],
            "Authenticated actual Linux process owner; boot, PID namespace, PID and start bind every file capability. Silence and finite deadlines never prove drain.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::LiveAuthority, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_file_instances",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::FileInstance),
                column("intent", text()),
                column("request_digest", text()),
                column("owner", text()),
                column("owner_kind", text()),
                column("selection", text()),
                column("issuing_store", text()),
                column("storage_incarnation", text()),
                column("digest", text()),
                column("payload_bytes", uint()),
                column("encoding", text()),
                column("batch_count", uint()),
                column("schema", text()),
                column("interpretation", text()),
                column("state", FieldContract::enumeration("CanonicalFileState")),
            ],
            "Never-reused physical file instance reserved before file work. Original unresolved intent protects pending/sealed publication; collecting and collected are irreversible.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_file_borrowers",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::FileBorrower),
                column("instance", text()),
                column("owner_kind", text()),
                column("owner", text()),
                column("released", flag()),
            ],
            "Independent exact physical-file borrower and permanent released attribution; logical result/artifact lifetimes arbitrate with collection under the instance guard.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_manifest_tickets",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ManifestTicket),
                column("kind", text()),
                column("selection", text()),
                column("run", text()).optional(),
                column("attempt", text()).optional(),
                column("manifest", text()).optional(),
                column("owner", text()),
                column("storage_incarnation", text()),
                column("released", flag()),
            ],
            "Nonexpiring exact immutable selection read ticket covering future unread files; only actual authenticated owner death or completed reader drain releases it.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::LiveAuthority, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_problems",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::Problem),
                column("head", text()),
                column("sequence", uint()),
            ],
            "Linear problem head; guarded compare-and-set advances it.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: true, enforced: false, generation_references: false },
        ),
        (
            "canonical_revisions",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision),
                column("problem", text()),
                column("sequence", uint()),
                column("parent", text()).optional(),
                column("operation", text()),
                column("request", bytes()),
                column("interpretation", text()),
            ],
            "Immutable authored revision and idempotent edit operation.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: true, enforced: false, generation_references: false },
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
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: true, enforced: false, generation_references: false },
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
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: true, generation_references: false },
        ),
        (
            "canonical_guards",
            vec![
                column("key", text()),
                column("generation", uint()),
                column("incarnation", text()).optional(),
                column("analysis_creation_closed_through", uint()).optional(),
                column("artifact_creation_closed_through", uint()).optional(),
                column("protection_closed_through", uint()).optional(),
                column("source_reclaim_after", text()).optional(),
            ],
            "Named conflict register; generation is separate from semantic eligibility.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::LiveAuthority, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
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
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
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
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
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
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_version_receipts",
            vec![column("key", text()), column("content_digest", text())],
            "Permanent immutable version identity fence, retained after payload reclamation.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Receipt, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
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
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
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
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
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
                column("grant_deadline", timestamp()).optional(),
                column("grant_nonce", text()).optional(),
                column("authority_incarnation", text()).optional(),
            ],
            "Protected immutable read/preparation selection; expiry fences further use.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::LiveAuthority, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
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
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
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
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Receipt, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_products",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::Product),
                column("problem", text()),
                column("revision", text()),
                column("request", bytes()),
                column("payload", bytes()),
                column("dependencies", bytes()),
                column("producer", text()),
                column("producer_class", FieldContract::enumeration("CanonicalProducerClass")),
                column("interpretation", text()),
            ],
            "Portable admitted scientific description and complete dependency witness; contains no native handles.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Derived, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_body_contents",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::BodyContent),
                column("version", text()),
                column("digest", text()),
                column("bytes", uint()),
                column("producer", text()),
                column("interpretation", text()),
                column("format", text()),
            ],
            "Immutable body bytes and complete reconstruction meaning, independent of selection receipts.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Derived, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_selection_receipts",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::SelectionReceipt),
                column("version", text()),
                column("digest", text()),
                column("bytes", uint()),
                column("interpretation", text()),
                column("format", text()),
            ],
            "Immutable complete selected and acquired predicates; not permission to publish.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Derived, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_product_associations",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProductAssociation),
                column("issuing_store", text()),
                column("problem", text()),
                column("revision", text()),
                column("request", bytes()),
                column("payload", bytes()),
                column("dependencies", bytes()),
                column("content", text()),
                column("receipt", text()),
                column("descriptor", text()),
                column("producer", text()),
                column("producer_class", FieldContract::enumeration("CanonicalProducerClass")),
                column("interpretation", text()),
            ],
            "Original immutable publication provenance; payload is a bounded physical descriptor, never scientific body bytes.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Derived, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_product_descriptors",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProductDescriptor),
                column("issuing_store", text()),
                column("problem", text()),
                column("request", bytes()),
                column("payload", bytes()),
                column("dependencies", bytes()),
                column("content", text()),
                column("receipt", text()),
                column("producer", text()),
                column("producer_class", FieldContract::enumeration("CanonicalProducerClass")),
                column("interpretation", text()),
                column("association", text()).optional(),
            ],
            "Fully checked exact descriptor with the lowest currently rooted association; hashes only index equality.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Derived, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_runs",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::Run),
                column("purpose", FieldContract::enumeration("CanonicalRunPurpose")).canonical_readonly().with_canonical_assertion("($value='execution' AND $this.admission_study=NONE AND $this.admission_point=NONE) OR ($value='study_admission' AND $this.admission_study!=NONE AND $this.admission_point!=NONE)"),
                column("admission_study", text()).with_canonical_key(crate::model::CanonicalKeyKind::Study).canonical_readonly().with_fk("runtime.canonical_studies", "key").optional(),
                column("admission_point", text()).with_canonical_key(crate::model::CanonicalKeyKind::StudyPoint).canonical_readonly().with_fk("runtime.canonical_study_points", "key").optional(),
                column("research_selection", text()).optional(),
                column("research_dataset", text()).optional(),
                column("problem", text()).with_canonical_key(crate::model::CanonicalKeyKind::Problem),
                column("revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision),
                column("source_sequence", uint()),
                column("sequence", uint()),
                column("request", bytes()),
                column("source_selection", bytes()),
                column("attestation", bytes()),
                column("interpretation", text()),
                column("current_generation", uint()),
                column("current_attempt", text()).with_canonical_key(crate::model::CanonicalKeyKind::Attempt).optional(),
                column("cancelled", flag()),
                column("terminal_attempt", text()).with_canonical_key(crate::model::CanonicalKeyKind::Attempt).optional(),
                column(
                    "terminal_class",
                    FieldContract::enumeration("CanonicalTerminalOutcome"),
                )
                .optional(),
            ],
            "One semantic execution, exact input selection and executable provenance; retry attempts retain this run identity.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: true, enforced: false, generation_references: false },
        ),
        (
            "canonical_attempts",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::Attempt),
                column("run", text()).with_canonical_key(crate::model::CanonicalKeyKind::Run).canonical_readonly(),
                column("generation", uint()).canonical_readonly(),
                column("claim_operation", text()).with_canonical_key(crate::model::CanonicalKeyKind::Operation).canonical_readonly(),
                column("request", bytes()).canonical_readonly(),
                column("worker", text()).canonical_readonly(),
                column("expires_at", timestamp()),
                column("phase", FieldContract::enumeration("CanonicalAttemptPhase")).with_canonical_initial("ingesting").with_canonical_transition_exception("type::record('canonical_runs',$this.run).purpose='study_admission' AND $before IN ['ingesting','closed_awaiting_manifest','closed_with_manifest'] AND $value='terminal'").with_canonical_assertion("(($value='ingesting' AND $this.close_generation=NONE AND $this.closed_manifest=NONE AND $this.outcome=NONE AND $this.completion=NONE AND $this.seed_manifest=NONE) OR ($value='closed_awaiting_manifest' AND $this.close_generation!=NONE AND $this.closed_manifest=NONE AND $this.outcome=NONE AND $this.completion=NONE AND $this.seed_manifest=NONE) OR ($value='closed_with_manifest' AND $this.close_generation!=NONE AND $this.closed_manifest!=NONE AND $this.outcome=NONE AND $this.completion=NONE AND $this.seed_manifest=NONE) OR ($value IN ['terminal','terminal_compacted'] AND (($value='terminal_compacted' AND $this.completion=NONE) OR ($value='terminal' AND ((type::record('canonical_runs',$this.run).purpose='study_admission' AND $this.outcome='failed' AND $this.completion=NONE) OR ($this.completion!=NONE AND !(type::record('canonical_runs',$this.run).purpose='study_admission' AND $this.outcome='failed'))))) AND $this.outcome IN ['succeeded','failed','partial','cancelled'] AND (type::record('canonical_runs',$this.run).purpose='study_admission' OR ($this.close_generation!=NONE AND $this.closed_manifest!=NONE))))"),
                column("close_generation", uint()).optional(),
                column(
                    "outcome",
                    FieldContract::enumeration("CanonicalTerminalOutcome"),
                ).with_canonical_assertion("$before=NONE OR $before=$value")
                .optional(),
                column("closed_manifest", text()).with_canonical_assertion("($value=NONE OR $value=$this.key) AND ($before=NONE OR $before=$value)").optional(),
                column("completion", bytes()).with_canonical_assertion("($this.phase!='terminal_compacted' OR $value=NONE) AND ($before=NONE OR $before=$value OR ($value=NONE AND $this.phase='terminal_compacted'))").optional(),
                column("seed_manifest", text()).with_canonical_seal("$before.phase IN ['terminal','terminal_compacted']").with_canonical_seal_refusal(super::canonical_refusals::RefusalCode::ExecutionTerminalSeedManifestImmutable.wire()).optional(),
                column("finalization_study", text()).with_canonical_key(crate::model::CanonicalKeyKind::Study).canonical_readonly().optional(),
            ],
            "Fenced actual execution attempt; closure authority is distinct from the revoked worker generation.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_result_sets",
            vec![
                column("key", text()),
                column("attempt", text()),
                column("name", text()),
                column("interpretation", text()),
                column("next_ordinal", uint()),
                column("row_count", uint()),
            ],
            "Private ordered scientific result membership; closure freezes the contiguous batch selection.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_inline_payloads",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::InlinePayload),
                column("attempt", text()),
                column("digest", text()),
                column("encoding", text()),
                column("schema", text()),
                column("interpretation", text()),
                column("payload_bytes", uint()),
                column("batch_count", uint()),
                column("body", bytes()),
            ],
            "One immutable complete bounded encoded body shared by exact logical batch members.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_result_batches",
            vec![
                column("key", text()),
                column("attempt", text()),
                column("result_set", text()),
                column("ordinal", uint()),
                column("digest", text()),
                column("row_count", uint()),
                column("payload_binding", text()),
                column("placement", FieldContract::enumeration("CanonicalPayloadPlacement")),
                column("inline_key", text()).optional(),
                column("file_instance", text()).optional(),
                column("payload_index", uint()),
                column("payload_bytes", uint()),
                column("schema", text()),
                column("encoding", text()),
            ],
            "Immutable bounded ingestion receipt; repeating the identity with different bytes is refused.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_result_manifests",
            vec![
                column("key", text()),
                column("attempt", text()),
                column("generation", uint()),
                column("digest", text()),
                column("descriptors", bytes()),
            ],
            "Closed exact result-set descriptor with frozen batch extents and coverage; does not imply scientific success.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_execution_operations",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::Operation),
                column("run", text()).with_canonical_key(crate::model::CanonicalKeyKind::Run),
                column("attempt", text()).with_canonical_key(crate::model::CanonicalKeyKind::Attempt).optional(),
                column("kind", text()),
                column("request", bytes()),
                column("result", bytes()),
                column("issuing_store", text()),
                column("study", text()).with_canonical_key(crate::model::CanonicalKeyKind::Study).optional(),
                column("point", text()).with_canonical_key(crate::model::CanonicalKeyKind::StudyPoint).optional(),
                column("minimal", flag()),
                column("disposition", text()),
            ],
            "Immutable lifecycle effect identity and settlement receipt after an uncertain acknowledgment.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Receipt, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_history_retirements",
            vec![
                column("key", text()),
                column("owner_kind", text()),
                column("owner", text()),
                column("issuing_store", text()),
                column("outcome", text()),
                column("revision", text()).canonical_readonly(),
                column("generation", uint()).canonical_readonly(),
                column("complete", flag()),
                column("after_locator", text()).optional(),
            ],
            "Permanent explicit detailed-history release and truthful minimal outcome; bounded collection cursor never grants replay or resurrection.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Receipt, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_history_locators",
            vec![
                column("key", text()),
                column("study", text()),
                column("run", text()),
                column("point", text()).optional(),
                column("issuing_store", text()),
            ],
            "Permanent exact point and summary owner attribution created atomically with study membership; opaque identities remain fenced after detail deletion.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_history_identities",
            vec![
                column("key", text()),
                column("category", text()),
                column("identity", text()),
                column("run", text()),
                column("issuing_store", text()),
            ],
            "Permanent original attempt, manifest, seed and effect-free run owner attribution; no successful numerical result is invented for absent effects.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_history_obligations",
            vec![
                column("key", text()),
                column("owner_kind", text()),
                column("owner", text()),
                column("purpose", text()),
                column("attribution", text()),
                column("issuing_store", text()),
                column("released", flag()),
            ],
            "Independent attributed audit or unsettled-effect history obligation; exact released tombstones reject delayed re-acquisition and identity reuse.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_history_obligation_counts",
            vec![column("key", text()), column("count", uint())],
            "Transactionally maintained bounded owner-obligation lookup; counts cannot substitute for immutable obligation attribution.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_result_blocks",
            vec![
                column("key", text()),
                column("result_set", text()),
                column("batch", text()),
                column("output", text()),
                column("partition", text()),
                column("ordinal", uint()),
                column("start", uint()),
                column("end", uint()),
                column("rows", uint()),
                column("columns", uint()),
                column("coordinate_min", FieldContract::native(DataType::Float64)).optional(),
                column("coordinate_max", FieldContract::native(DataType::Float64)).optional(),
                column("payload_bytes", uint()),
                column("payload_digest", text()),
                column("statistics", bytes()).optional(),
                column("interpretation", text()),
            ],
            "Independently decodable exact scientific Arrow block; indexed range metadata is derived from its admitted coordinates.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Derived, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_result_block_outputs",
            vec![
                column("key", text()),
                column("batch", text()),
                column("result_set", text()),
                column("output", text()),
                column("partition", text()),
                column("start", uint()),
                column("end", uint()),
                column("coordinate_min", FieldContract::native(DataType::Float64)).optional(),
                column("coordinate_max", FieldContract::native(DataType::Float64)).optional(),
                column("interpretation", text()),
            ],
            "Derived native output-group index references exact original IPC rows; dense arrays never create per-element database cells.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Derived, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_result_protections",
            vec![
                column("key", text()),
                column("run", text()),
                column("attempt", text()),
                column("manifest", text()),
            ],
            "Exact result read association to the same-key revision protection; result reclamation checks its live protection under the retention guard.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::LiveAuthority, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_result_retirements",
            vec![
                column("key", text()),
                column("run", text()),
                column("after_generation", uint()),
                column("complete", flag()),
            ],
            "Explicit irreversible withdrawal of a run's scientific payloads; bounded cleanup resumes without reopening scientific execution or deleting immutable lifecycle receipts.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Receipt, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_retirements",
            vec![column("key", text()), column("study", text())],
            "Explicit withdrawal of a terminal study's result retention obligation; its occurrence and scientific outcome receipts remain immutable.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Receipt, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_artifacts",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::Artifact),
                column("kind", text()),
                column("revision", text()),
                column("primary_problem", text()),
                column("primary_authority", text()),
                column("creation_nonce", text()),
                column("creation_expires_at", uint()),
                column("creation_request_digest", text()),
                column("interpretation", text()),
                column("provenance", bytes()),
                column("block_count", uint()),
                column("payload_bytes", uint()),
                column("active", flag()),
                column("retiring", flag()),
            ],
            "Independent selected scientific closure; finite original creation authority, owned copied content and source roots survive producer collection.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: true, enforced: false, generation_references: false },
        ),
        (
            "canonical_artifact_blocks",
            vec![
                column("key", text()),
                column("artifact", text()),
                column("ordinal", uint()),
                column("relation", text()),
                column("file_instance", text()),
                column("file_digest", text()),
                column("file_batch_index", uint()),
                column("payload_bytes", uint()),
                column("schema", text()),
                column("encoding", text()),
                column("digest", text()),
                column("original_digest", text()),
                column("rows", uint()),
            ],
            "One bounded artifact-owned capsule or standalone IPC block; original and copied encoding digests are distinct provenance.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_artifact_obligations",
            vec![
                column("key", text()),
                column("artifact", text()),
                column("owner_kind", FieldContract::enumeration("CanonicalArtifactHolderKind")),
                column("owner", text()),
            ],
            "Independent attributed study, user, project, analysis or audit obligation; only the matching owner releases it.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_artifact_releases",
            vec![
                column("key", text()),
                column("artifact", text()),
                column("owner_kind", FieldContract::enumeration("CanonicalArtifactHolderKind")),
                column("owner", text()),
            ],
            "Permanent exact obligation release; a delayed attach cannot recreate a released owner lifetime.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Receipt, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_artifact_reads",
            vec![column("key", text()), column("artifact", text())],
            "Artifact read association to the same-key finite canonical source protection grant; collection joins actual grant settlement.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::LiveAuthority, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_artifact_staging_inputs",
            vec![
                column("key", text()),
                column("artifact", text()),
                column("run", text()),
                column("attempt", text()),
                column("manifest", text()),
            ],
            "Protective exact producer selection during staged or uncertain scientific transfer; removed only after activation or exact creation settlement.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_inputs",
            vec![
                column("key", text()),
                column("study", text()),
                column("solution", text()),
                column("artifact", text()),
            ],
            "Exact external seed selection admitted before study activation; one obligation per distinct selected solution preserves original occurrence policy.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_analyses",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::Analysis),
                column("revision", text()),
                column("method", text()),
                column("configuration", bytes()),
                column("input_digest", text()),
                column("primary_problem", text()),
                column("primary_authority", text()),
                column("creation_nonce", text()),
                column("creation_request_digest", text()),
                column("creation_expires_at", uint()),
                column("interpretation", text()),
                column("node_count", uint()),
                column("edge_count", uint()),
                column("active", flag()),
                column("retiring", flag()),
            ],
            "Derived analysis method, exact configuration and source lineage; activation follows complete graph membership and never changes authored problem authority.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: true, enforced: false, generation_references: false },
        ),
        (
            "canonical_analysis_nodes",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::AnalysisNode),
                column("analysis", text()),
                column("semantic", text()),
                column("kind", text()),
            ],
            "Selected semantic objects within one immutable derived analysis graph.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: true, enforced: false, generation_references: false },
        ),
        (
            "canonical_analysis_edges",
            vec![
                column("key", text()),
                column("analysis", text()),
                column("source", text())
                    .with_fk("runtime.canonical_analysis_nodes", "key")
                    .with_graph_endpoint(true),
                column("target", text())
                    .with_fk("runtime.canonical_analysis_nodes", "key")
                    .with_graph_endpoint(false),
                column("kind", text()),
                column("evidence_artifact", text()).optional(),
                column("evidence_block", uint()).optional(),
                column("evidence_row", uint()).optional(),
                column("evidence_field", text()).optional(),
            ],
            "Method-scoped incidence, dependency, topology or quantitative evidence edge; each relation meaning is explicit.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: true, generation_references: false },
        ),
        (
            "canonical_analysis_inputs",
            vec![
                column("key", text()),
                column("analysis", text())
                    .with_fk("runtime.canonical_analyses", "key")
                    .with_graph_endpoint(true),
                column("artifact", text())
                    .with_fk("runtime.canonical_artifacts", "key")
                    .with_graph_endpoint(false),
            ],
            "Independent scientific input artifact consumed by a retained derived analysis; explicit attributed obligations survive original producer collection.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: true, generation_references: false },
        ),
        (
            "canonical_result_seeds",
            vec![
                column("key", text()),
                column("batch", text()),
                column("result_set", text()),
                column("attempt", text()),
                column("run", text()),
                column("layout", text()),
                column("preparation", text()),
                column("profile", text()),
                column("data", text()),
                column("backend", text()),
                column("step", uint()),
                column("batch_count", uint()),
                column("first_ordinal", uint()),
                column("run_sequence", uint()),
                column("attempt_generation", uint()),
                column("payload_bytes", uint()),
                column("digest", text()),
            ],
            "Indexed exact scientific seed descriptor; immutable chunks belong to the closed result selection, and completion independently grants eligibility.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Derived, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_problem_runs",
            vec![
                column("key", text()),
                column("problem", text())
                    .with_fk("runtime.canonical_problems", "key")
                    .with_graph_endpoint(true),
                column("run", text())
                    .with_fk("runtime.canonical_runs", "key")
                    .with_graph_endpoint(false),
            ],
            "Native problem-to-run relationship; execution identity remains distinct from authored revision identity.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: true, generation_references: false },
        ),
        (
            "canonical_run_sources",
            vec![
                column("key", text()),
                column("run", text())
                    .with_fk("runtime.canonical_runs", "key")
                    .with_graph_endpoint(true),
                column("revision", text())
                    .with_fk("runtime.canonical_revisions", "key")
                    .with_graph_endpoint(false),
            ],
            "Exact selected source revision of a run; supports connected provenance traversal.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: true, generation_references: false },
        ),
        (
            "canonical_studies",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::Study),
                column("run", text()).with_canonical_key(crate::model::CanonicalKeyKind::Run).canonical_readonly(),
                column("problem", text()).with_canonical_key(crate::model::CanonicalKeyKind::Problem).canonical_readonly(),
                column("revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision).canonical_readonly(),
                column("metadata", bytes()).canonical_readonly(),
                column("physical_revision", text()).canonical_readonly().optional(),
                column("physical_identity", text()).canonical_readonly().optional(),
                column("context", text()).canonical_readonly().optional(),
                column("interpretation", text()).canonical_readonly(),
                column("point_count", uint()).canonical_readonly(),
                column("next_ordinal", uint()).with_canonical_assertion("$value>=0dec AND $value<=18446744073709551615dec AND math::floor($value)=$value AND $value<=$this.point_count AND ($before=NONE OR $value>=$before) AND (!$this.active OR $value=$this.point_count)"),
                column("dependency_count", uint()).canonical_readonly(),
                column("installed_dependencies", uint()).with_canonical_assertion("$value>=0dec AND $value<=18446744073709551615dec AND math::floor($value)=$value AND $value<=$this.dependency_count AND ($before=NONE OR $value>=$before) AND (!$this.active OR ($value=$this.dependency_count AND $this.next_ordinal=$this.point_count))"),
                column("active", flag()),
                column("cancelled", flag()),
                column("generation", uint()),
                column("terminal", flag()),
            ],
            "Immutable study selection and bounded staged occurrence membership; activation follows complete admission.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_capture_gate",
            vec![column("key", text()).canonical_record_key(), column("incarnation", text()).canonical_mutable(), column("capture", text()).canonical_mutable().optional(), column("owner", text()).canonical_mutable().optional(), column("request", bytes()).canonical_mutable().optional(), column("issued_expires_at", uint()).canonical_mutable()],
            "Immutable compact capture ownership and exact facts; pending, interrupted and abandoned state remains backed up and protective until authenticated fenced recovery or actual scan drain.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_captures",
            vec![column("key", text()).canonical_record_key(), column("owner", text()).canonical_readonly(), column("request", bytes()).canonical_readonly(), column("incarnation", text()).canonical_readonly(), column("issued_expires_at", uint()).canonical_readonly(), column("state", FieldContract::enumeration("CanonicalStudyCaptureState")).with_canonical_initial("pending").with_canonical_transition_exception("$before=NONE AND $value='released' AND $this.incarnation='unissued' AND $this.issued_expires_at=0dec"), column("header_count", uint()).with_canonical_seal("$before.state!='pending'").with_canonical_seal_refusal(super::canonical_refusals::RefusalCode::StudyCaptureIdentityReused.wire()), column("member_count", uint()).with_canonical_seal("$before.state!='pending'").with_canonical_seal_refusal(super::canonical_refusals::RefusalCode::StudyCaptureIdentityReused.wire())],
            "Immutable compact capture ownership and exact facts; pending, interrupted and abandoned state remains backed up and protective until authenticated fenced recovery or actual scan drain.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_capture_headers",
            vec![column("key", text()).canonical_record_key(), column("capture", text()).canonical_readonly(), column("ordinal", uint()).canonical_readonly(), column("body", bytes()).canonical_readonly()],
            "Immutable compact capture ownership and exact facts; pending, interrupted and abandoned state remains backed up and protective until authenticated fenced recovery or actual scan drain.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_capture_members",
            vec![column("key", text()).canonical_record_key(), column("capture", text()).canonical_readonly(), column("ordinal", uint()).canonical_readonly(), column("body", bytes()).canonical_readonly(), column("study", text()).canonical_readonly(), column("run", text()).canonical_readonly(), column("attempt", text()).canonical_readonly().optional(), column("manifest", text()).canonical_readonly().optional()],
            "Immutable compact capture ownership and exact facts; pending, interrupted and abandoned state remains backed up and protective until authenticated fenced recovery or actual scan drain.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_capture_protections",
            vec![column("key", text()).canonical_record_key(), column("capture", text()).canonical_readonly(), column("study", text()).canonical_readonly(), column("run", text()).canonical_readonly(), column("attempt", text()).canonical_readonly(), column("manifest", text()).canonical_readonly(), column("problem", text()).canonical_readonly(), column("revision", text()).canonical_readonly()],
            "Immutable compact capture ownership and exact facts; pending, interrupted and abandoned state remains backed up and protective until authenticated fenced recovery or actual scan drain.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_admission_outcomes",
            vec![column("key", text()).canonical_record_key().canonical_readonly(), column("projection", bytes()).canonical_readonly()],
            "Compact immutable admitted case/binding identity or explicit absence/refusal, without scientific descriptors; lazy admission publishes it with its outcome.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_installations",
            vec![column("key", text()).canonical_record_key().canonical_readonly(), column("digest", text()).canonical_readonly(), column("generation", uint()).canonical_readonly(), column("phase", FieldContract::enumeration("CanonicalStudyInstallationPhase")).with_canonical_initial("nodes"), column("node_cursor", uint()).with_canonical_assertion("$value>=0dec AND math::floor($value)=$value AND ($before=NONE OR $value>=$before)"), column("edge_cursor", uint()).with_canonical_assertion("$value>=0dec AND math::floor($value)=$value AND ($before=NONE OR $value>=$before)")],
            "Exact immutable population identity and installed frontier, including interrupted installation; retained for replay and backup.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_installation_pages",
            vec![column("key", text()).canonical_record_key().canonical_readonly(), column("study", text()).canonical_readonly().canonical_readonly(), column("digest", text()).canonical_readonly(), column("generation", uint()).canonical_readonly(), column("phase", FieldContract::enumeration("CanonicalStudyInstallationPhase")).canonical_readonly().with_canonical_assertion("$value IN ['nodes','edges']"), column("cursor", uint()).canonical_readonly(), column("next_cursor", uint()).canonical_readonly(), column("request", bytes()).canonical_readonly().canonical_readonly(), column("issued_expires_at", uint()).canonical_readonly().canonical_readonly(), column("state", FieldContract::enumeration("CanonicalStudyInstallationPageState")).with_canonical_initial("issued")],
            "Immutable exact committed node/edge page acknowledgement under its original issued clock; interrupted installations remain resumable without recreating rows.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_points",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::StudyPoint),
                column("study", text()).with_canonical_key(crate::model::CanonicalKeyKind::Study),
                column("admission_run", text()).with_canonical_key(crate::model::CanonicalKeyKind::Run).canonical_readonly().with_fk("runtime.canonical_runs", "key").optional(),
                column("variant_problem", text()).with_canonical_key(crate::model::CanonicalKeyKind::Problem).canonical_readonly().optional(),
                column("variant_operation", text()).with_canonical_key(crate::model::CanonicalKeyKind::Operation).canonical_readonly().optional(),
                column("ordinal", uint()),
                column("occurrence", uint()),
                column("seed_solution", text()).optional(),
                column("policy", bytes()),
                column("descriptor", bytes()),
                column("admission", bytes()).optional(),
                column("facts", bytes()),
                column("outcome", bytes()).optional(),
                column("run", text()).with_canonical_key(crate::model::CanonicalKeyKind::Run),
                column("revision", uint()),
                column("assigned", flag()),
                column("settled", flag()),
                column("attempt", text()).with_canonical_key(crate::model::CanonicalKeyKind::Attempt).optional(),
                column("start", bytes()).optional(),
            ],
            "Distinct requested occurrence, immutable operation and policy, fenced observations and actual selected start. Query flags are derived from the shared policy owner.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: true, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_dependencies",
            vec![
                column("key", text()),
                column("study", text()),
                column("dependent", text())
                    .with_fk("runtime.canonical_study_points", "key")
                    .with_graph_endpoint(true),
                column("predecessor", text())
                    .with_fk("runtime.canonical_study_points", "key")
                    .with_graph_endpoint(false),
                column("kind", text()),
            ],
            "Native occurrence dependency edge; ordering, scientific usability and selected seed roles retain distinct meanings.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Durable, backup: crate::model::CanonicalBackup::Required, inline_vertex: false, enforced: true, generation_references: false },
        ),
        (
            "canonical_study_readiness",
            vec![
                column("key", text()),
                column("point", text()),
                column("study", text()),
                column("ordinal", uint()),
                column("source_revision", uint()),
                column("pending", uint()),
                column("eligible", flag()),
                column("premises", bytes()).optional(),
                column("dirty", flag()),
                column("notify", flag()),
                column("notify_revision", uint()),
                column("notify_after", text()),
            ],
            "Derived discovery only: complete observed premise revisions, pending predecessor observations and pure-policy eligibility. Durable bounded outgoing notification never grants a scientific claim.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Derived, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
        (
            "canonical_study_readiness_cursors",
            vec![
                column("key", text()),
                column("study", text()),
                column("revision", uint()),
                column("rebuild_pending", flag()),
                column("rebuild_after", uint()).optional(),
                column("notify_after", uint()).optional(),
                column("verify_after", uint()).optional(),
                column("pending_operation", text()).optional(),
            ],
            "Derived per-study round-robin notification and resumable rebuild/verification cursors. Scientific state transitions never write this row.",
            crate::model::CanonicalRelation { retention: crate::model::CanonicalRetention::Derived, backup: crate::model::CanonicalBackup::Recovered, inline_vertex: false, enforced: false, generation_references: false },
        ),
    ];
    for (name, columns, doc, policy) in declarations {
        builder.declare_relation(
            declaration(
                Namespace::Runtime,
                name,
                1,
                SnapshotClass::Sidecar,
                &["key"],
                columns,
                doc,
            )
            .with_canonical(policy),
        );
    }
}
