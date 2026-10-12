// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Canonical substrate envelopes. Scientific payload shapes remain at their semantic owner.

use super::declarations::{column, declaration};
use crate::{
    RegistryBuilder,
    model::{FieldContract, Namespace, SnapshotClass},
};
use arrow_schema::DataType;

/// Exact schema/codec/structural-operation interpretation.
pub const INTERPRETATION: &str = "pse.substrate.v5";

/// Self-contained scientific IPC admission, independent of derived indexes.
pub const RESULT_BLOCK_BYTES: usize = 512 * 1024;
/// Native CBOR index/descriptor envelope admission per append.
pub const RESULT_INDEX_BYTES: usize = 128 * 1024;
/// Complete uncompressed native WebSocket message admission.
pub const RESULT_MESSAGE_BYTES: usize = 4 * 1024 * 1024;
/// Every valid index object contains at least the nonempty `key` field. The
/// minimal CBOR map `{key: "a"}` occupies seven bytes; other mandatory fields
/// only increase its extent. This ceiling protects loops, never chooses blocks.
pub const RESULT_INDEX_RECORDS: usize = RESULT_INDEX_BYTES / 7;

/// Relations whose endpoints exist before insertion, including first publication.
/// Study dependencies intentionally allow predecessors on later ingestion pages.
pub const ENFORCED_RELATIONS: &[&str] = &[
    "canonical_memberships",
    "canonical_analysis_edges",
    "canonical_analysis_inputs",
    "canonical_problem_runs",
    "canonical_run_sources",
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
    guard_incarnation: canonical_guards [incarnation] true,
    membership_selection: canonical_memberships [problem, scope, name, from_sequence] false,
    membership_active: canonical_memberships [problem, to_sequence] false,
    version_logical: canonical_versions [logical] false,
    revision_operation: canonical_revisions [operation] true,
    revision_sequence: canonical_revisions [problem, sequence] true,
    protection_revision: canonical_protections [revision, expires_at] false,
    root_revision: canonical_roots [revision] false,
    root_owner: canonical_roots [owner_kind, owner, key] false,
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
    run_terminal_selection: canonical_runs [problem, terminal_class, sequence] false,
    attempt_run: canonical_attempts [run, generation] true,
    attempt_live: canonical_attempts [run, terminal, generation] false,
    result_set_attempt: canonical_result_sets [attempt, name] true,
    result_batch_order: canonical_result_batches [result_set, ordinal] true,
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
    result_row_selection: canonical_result_rows [result_set, output, partition, row] false,
    result_row_batch: canonical_result_rows [batch, key] false,
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
            "canonical_maintenance_activations",
            vec![column("key", text()),column("input_sha256", text()),column("predecessor", bytes()),column("interpretation", text()),column("schema_digest", text()),column("issuing_store", text())],
            "Exact immutable schema activation effect receipt under the original rollout nonce; marker and issuing identity publication are atomic with this acknowledgement.",
        ),
        (
            "canonical_interpretations",
            vec![
                column("key", text()),
                column("interpretation", text()),
                column("schema_digest", text()),
                column("issuing_store", text()),
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
            vec![
                column("key", text()),
                column("generation", uint()),
                column("incarnation", text()).optional(),
                column("analysis_creation_closed_through", uint()).optional(),
                column("artifact_creation_closed_through", uint()).optional(),
                column("protection_closed_through", uint()).optional(),
            ],
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
                column("grant_deadline", timestamp()).optional(),
                column("grant_nonce", text()).optional(),
                column("authority_incarnation", text()).optional(),
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
        (
            "canonical_body_contents",
            vec![
                column("key", text()),
                column("version", text()),
                column("digest", text()),
                column("bytes", uint()),
                column("producer", text()),
                column("interpretation", text()),
                column("format", text()),
            ],
            "Immutable body bytes and complete reconstruction meaning, independent of selection receipts.",
        ),
        (
            "canonical_selection_receipts",
            vec![
                column("key", text()),
                column("version", text()),
                column("digest", text()),
                column("bytes", uint()),
                column("interpretation", text()),
                column("format", text()),
            ],
            "Immutable complete selected and acquired predicates; not permission to publish.",
        ),
        (
            "canonical_product_associations",
            vec![
                column("key", text()),
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
                column("interpretation", text()),
            ],
            "Original immutable publication provenance; payload is a bounded physical descriptor, never scientific body bytes.",
        ),
        (
            "canonical_product_descriptors",
            vec![
                column("key", text()),
                column("issuing_store", text()),
                column("problem", text()),
                column("request", bytes()),
                column("payload", bytes()),
                column("dependencies", bytes()),
                column("content", text()),
                column("receipt", text()),
                column("producer", text()),
                column("interpretation", text()),
                column("association", text()).optional(),
            ],
            "Fully checked exact descriptor with the lowest currently rooted association; hashes only index equality.",
        ),
        (
            "canonical_runs",
            vec![
                column("key", text()),
                column("problem", text()),
                column("revision", text()),
                column("source_sequence", uint()),
                column("sequence", uint()),
                column("request", bytes()),
                column("source_selection", bytes()),
                column("attestation", bytes()),
                column("interpretation", text()),
                column("current_generation", uint()),
                column("current_attempt", text()).optional(),
                column("cancelled", flag()),
                column("terminal_attempt", text()).optional(),
                column("terminal_class", text()).optional(),
            ],
            "One semantic execution, exact input selection and executable provenance; retry attempts retain this run identity.",
        ),
        (
            "canonical_attempts",
            vec![
                column("key", text()),
                column("run", text()),
                column("generation", uint()),
                column("claim_operation", text()),
                column("request", bytes()),
                column("worker", text()),
                column("expires_at", timestamp()),
                column("ingestion_open", flag()),
                column("closed", flag()),
                column("close_generation", uint()).optional(),
                column("terminal", flag()),
                column("outcome", text()).optional(),
                column("closed_manifest", text()).optional(),
                column("completion", bytes()).optional(),
                column("seed_manifest", text()).optional(),
                column("finalization_study", text()).optional(),
            ],
            "Fenced actual execution attempt; closure authority is distinct from the revoked worker generation.",
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
        ),
        (
            "canonical_result_batches",
            vec![
                column("key", text()),
                column("attempt", text()),
                column("result_set", text()),
                column("ordinal", uint()),
                column("digest", text()),
                column("payload", bytes()),
                column("row_count", uint()),
            ],
            "Immutable bounded ingestion receipt; repeating the identity with different bytes is refused.",
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
        ),
        (
            "canonical_execution_operations",
            vec![
                column("key", text()),
                column("run", text()),
                column("attempt", text()).optional(),
                column("kind", text()),
                column("request", bytes()),
                column("result", bytes()),
                column("issuing_store", text()),
                column("study", text()).optional(),
                column("point", text()).optional(),
                column("minimal", flag()),
                column("disposition", text()),
            ],
            "Immutable lifecycle effect identity and settlement receipt after an uncertain acknowledgment.",
        ),
        (
            "canonical_history_retirements",
            vec![column("key", text()),column("owner_kind", text()),column("owner", text()),column("issuing_store", text()),column("outcome", text()),column("complete", flag()),column("after_locator", text()).optional()],
            "Permanent explicit detailed-history release and truthful minimal outcome; bounded collection cursor never grants replay or resurrection.",
        ),
        (
            "canonical_history_locators",
            vec![column("key", text()),column("study", text()),column("run", text()),column("point", text()).optional(),column("issuing_store", text())],
            "Permanent exact point and summary owner attribution created atomically with study membership; opaque identities remain fenced after detail deletion.",
        ),
        (
            "canonical_history_identities",
            vec![column("key", text()),column("category", text()),column("identity", text()),column("run", text()),column("issuing_store", text())],
            "Permanent original attempt, manifest, seed and effect-free run owner attribution; no successful numerical result is invented for absent effects.",
        ),
        (
            "canonical_history_obligations",
            vec![column("key", text()),column("owner_kind", text()),column("owner", text()),column("purpose", text()),column("attribution", text()),column("issuing_store", text()),column("released", flag())],
            "Independent attributed audit or unsettled-effect history obligation; exact released tombstones reject delayed re-acquisition and identity reuse.",
        ),
        (
            "canonical_history_obligation_counts",
            vec![column("key", text()),column("count", uint())],
            "Transactionally maintained bounded owner-obligation lookup; counts cannot substitute for immutable obligation attribution.",
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
                column("interpretation", text()),
            ],
            "Independently decodable exact scientific Arrow block; indexed range metadata is derived from its admitted coordinates.",
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
        ),
        (
            "canonical_result_rows",
            vec![
                column("key", text()),
                column("result_set", text()),
                column("attempt", text()),
                column("batch", text()),
                column("relation_id", FieldContract::id()),
                column("run_id", FieldContract::id().with_identity("run")),
                column("owner", FieldContract::id()),
                column("step", FieldContract::native(DataType::Int64)).optional(),
                column("experiment_id", FieldContract::id()).optional(),
                column("output", text()),
                column("partition", text()),
                column("row", uint()),
                column("offset", uint()),
                column("interpretation", text()),
            ],
            "One exact typed scientific row locator in its original immutable attempt/set/block; output and partition are derived access selectors, never scientific key authority.",
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
        ),
        (
            "canonical_study_retirements",
            vec![column("key", text()), column("study", text())],
            "Explicit withdrawal of a terminal study's result retention obligation; its occurrence and scientific outcome receipts remain immutable.",
        ),
        (
            "canonical_artifacts",
            vec![
                column("key", text()),
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
        ),
        (
            "canonical_artifact_blocks",
            vec![column("key", text()), column("artifact", text()), column("ordinal", uint()),
                column("relation", text()), column("payload", bytes()), column("digest", text()),
                column("original_digest", text()), column("rows", uint())],
            "One bounded artifact-owned capsule or standalone IPC block; original and copied encoding digests are distinct provenance.",
        ),
        (
            "canonical_artifact_obligations",
            vec![column("key", text()), column("artifact", text()), column("owner_kind", text()), column("owner", text())],
            "Independent attributed study, user, project, analysis or audit obligation; only the matching owner releases it.",
        ),
        (
            "canonical_artifact_releases",
            vec![column("key", text()), column("artifact", text()), column("owner_kind", text()), column("owner", text())],
            "Permanent exact obligation release; a delayed attach cannot recreate a released owner lifetime.",
        ),
        (
            "canonical_artifact_reads",
            vec![column("key", text()), column("artifact", text())],
            "Artifact read association to the same-key finite canonical source protection grant; collection joins actual grant settlement.",
        ),
        (
            "canonical_artifact_staging_inputs",
            vec![column("key", text()), column("artifact", text()), column("run", text()), column("attempt", text()), column("manifest", text())],
            "Protective exact producer selection during staged or uncertain scientific transfer; removed only after activation or exact creation settlement.",
        ),
        (
            "canonical_study_inputs",
            vec![column("key", text()), column("study", text()), column("solution", text()), column("artifact", text())],
            "Exact external seed selection admitted before study activation; one obligation per distinct selected solution preserves original occurrence policy.",
        ),
        (
            "canonical_analyses",
            vec![
                column("key", text()),
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
        ),
        (
            "canonical_analysis_nodes",
            vec![
                column("key", text()),
                column("analysis", text()),
                column("semantic", text()),
                column("kind", text()),
            ],
            "Selected semantic objects within one immutable derived analysis graph.",
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
        ),
        (
            "canonical_studies",
            vec![
                column("key", text()),
                column("run", text()),
                column("problem", text()),
                column("revision", text()),
                column("metadata", bytes()),
                column("interpretation", text()),
                column("point_count", uint()),
                column("next_ordinal", uint()),
                column("active", flag()),
                column("cancelled", flag()),
                column("generation", uint()),
                column("terminal", flag()),
            ],
            "Immutable study selection and bounded staged occurrence membership; activation follows complete admission.",
        ),
        (
            "canonical_study_points",
            vec![
                column("key", text()),
                column("study", text()),
                column("ordinal", uint()),
                column("occurrence", uint()),
                column("seed_solution", text()).optional(),
                column("policy", bytes()),
                column("descriptor", bytes()),
                column("facts", bytes()),
                column("outcome", bytes()).optional(),
                column("run", text()),
                column("revision", uint()),
                column("assigned", flag()),
                column("settled", flag()),
                column("attempt", text()).optional(),
                column("start", bytes()).optional(),
            ],
            "Distinct requested occurrence, immutable operation and policy, fenced observations and actual selected start. Query flags are derived from the shared policy owner.",
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
            ],
            "Derived per-study round-robin notification and resumable rebuild/verification cursors. Scientific state transitions never write this row.",
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
