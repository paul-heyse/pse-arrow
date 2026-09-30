// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The operational relations: meaning and shape of the operational store
//! (ADR-0114 Outcomes 10 and 22; Plan 22 architecture §9).
//!
//! PostgreSQL owns what changes and Delta owns what is published. The registry owns the
//! meaning **and the shape** of every operational relation, the publication catalog's
//! included: the `pse_ops` DDL is generated from these declarations (`pse-codegen`
//! target `postgres`), so there is no second, hand-written description to compare.
//! Relation `runtime.operational_<t>` is table `pse_ops.<t>` ([`crate::store`]).
//!
//! What the declarations carry into the store:
//! - enumerations become PostgreSQL ENUM types, so a misspelled literal fails at PREPARE;
//! - entity identities (ADR-0115) become identity domains and typed ids;
//! - timestamps are `ts_us` (the microseconds `timestamptz` holds) and documents `json`;
//! - every CHECK is a named row check or field domain, rendered for DataFusion and
//!   PostgreSQL alike; unique keys and composite references are declared, never implied.
//!
//! There are no cascading deletes and no identity-minting defaults: deletes are explicit
//! and every identity is minted by the runtime (ADR-0114 Outcome 13). Defaults and access
//! paths are the store's `physical.sql`.
use super::declarations::{column, declaration, enumeration, identity};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, RelationDecl, SnapshotClass as S},
};
use arrow_schema::DataType as D;

fn text() -> T {
    T::native(D::Utf8)
}
/// Exact bytes; PostgreSQL `bytea`.
fn bytes() -> T {
    T::native(D::Binary)
}
fn real() -> T {
    T::native(D::Float64)
}
fn flag() -> T {
    T::native(D::Boolean)
}
fn int32() -> T {
    T::native(D::Int32)
}
fn int64() -> T {
    T::native(D::Int64)
}
fn json() -> T {
    T::json_document()
}
/// The `ts_us` logical type; PostgreSQL `timestamptz` at its microsecond precision.
fn ts() -> T {
    T::native(crate::model::extension::timestamp_micros_storage())
}
fn attempt_ref(name: &'static str) -> T {
    column(name, T::id()).with_fk("runtime.operational_attempts", "attempt_id")
}
fn publication_ref(name: &'static str) -> T {
    column(name, T::id()).with_fk("runtime.operational_publications", "publication_id")
}

/// A store relation declaration; the caller adds checks, keys and references.
fn store(
    name: &'static str,
    keys: &[&'static str],
    columns: Vec<T>,
    doc: &'static str,
) -> RelationDecl {
    declaration(N::Runtime, name, 1, S::Sidecar, keys, columns, doc)
}

/// `"<column>" <> ''`: required text is nonempty.
fn nonempty(column: &str) -> String {
    format!("\"{column}\" <> ''")
}

/// `"<column>" IS NULL OR "<column>" <> ''`: optional text is absent or nonempty. Row
/// checks must be true (a null predicate is a violation), so absence is explicit.
fn absent_or_nonempty(column: &str) -> String {
    format!("\"{column}\" IS NULL OR \"{column}\" <> ''")
}

pub(super) fn declare(b: &mut RegistryBuilder) {
    declare_enumerations(b);
    declare_identities(b);
    declare_sources(b);
    declare_attempts(b);
    declare_jobs(b);
    declare_streams(b);
    declare_solutions(b);
    declare_catalog(b);
    declare_studies(b);
}

fn declare_enumerations(b: &mut RegistryBuilder) {
    // DP-19: the durable lifecycle. Legality of a change is owned by the one transition
    // table in `pse-operations`, never by these spellings or by SQL.
    enumeration(
        b,
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
    // The queue state of a job; each try runs as its own attempt. A waiting job is not
    // claimable until the work it depends on releases it (a study point behind its
    // predecessor, a study's finalization behind its last point).
    enumeration(
        b,
        "JobState",
        [
            "waiting",
            "queued",
            "running",
            "completed",
            "failed",
            "cancelled",
        ],
    );
    // What an attempt computes: an authored algebraic sequence, a simulation or a fit; a
    // study, which coordinates its points' attempts and is the attempt its publication
    // names; or a study's finalization, which publishes it.
    enumeration(
        b,
        "AttemptKind",
        [
            "modeling",
            "simulation",
            "fit",
            "study",
            "study_finalization",
        ],
    );
    // A study is open while its points run, concluded once every point is terminal (its
    // attempt has ended and its finalization is queued), published once its one
    // publication is committed.
    enumeration(b, "StudyState", ["open", "concluded", "published"]);
    // A point is pending until a worker claims its job, assigned while a try runs, then
    // completed (its result members are written), failed, or cancelled (by the study, or
    // because its predecessor did not complete).
    enumeration(
        b,
        "StudyPointState",
        ["pending", "assigned", "completed", "failed", "cancelled"],
    );
    // The native meaning of a stored seed's vectors (the portable warm-start payloads).
    enumeration(b, "StoredSeedKind", ["root", "nlp", "highs"]);
    // Where a stored solution came from (Plan 22 I13): the accepted output seed of a step,
    // or a point captured from a search's incumbent stream, pruned with that stream.
    enumeration(b, "StoredSolutionOrigin", ["output", "incumbent"]);
    // Which typed column names an attempt's termination (X4): the last native solver
    // termination, the last run state when no native termination was reported, a
    // trajectory termination, a runtime-owned outcome, or the diagnostic code of a
    // violated rule (a `DiagnosticCode`).
    enumeration(
        b,
        "TerminationClass",
        ["native", "run_state", "trajectory", "runtime", "rule"],
    );
    // Terminations the durable runtime owns: work stopped by cancellation or by
    // infrastructure, work never attempted, a fit without a solve, a run whose
    // completion could not be assessed.
    enumeration(
        b,
        "RuntimeTermination",
        [
            "cancelled",
            "infrastructure",
            "unattempted",
            "constant_evaluation",
            "unassessed",
        ],
    );
    // Two-phase deletion of a publication: no new reader leases, then files removed.
    enumeration(b, "RetentionPhase", ["expiring", "deleted"]);
    // The outcome of settling an uncertain commit acknowledgement: the publication is
    // visible; nothing was or can be committed by that request as prepared (the head is
    // still its expected parent); or it conflicts with the committed history (the head
    // moved, or the attempt was published as another publication).
    enumeration(
        b,
        "SettlementOutcome",
        ["committed", "proved_noncommit", "conflict"],
    );
    // Whether a publication member is an output it publishes or an input it read.
    enumeration(b, "PublicationMemberRole", ["output", "input"]);
}

fn declare_identities(b: &mut RegistryBuilder) {
    declare_publication_identities(b);
    declare_run_identity(b);
    identity(b, "job", "One durable job claimed by workers");
    identity(
        b,
        "solution",
        "One stored reusable solution (a warm-start seed)",
    );
    identity(b, "study", "One study coordinated across its points");
    identity(
        b,
        "source_bundle",
        "An authored source bundle, content-addressed by its package content hash",
    );
    identity(
        b,
        "settlement",
        "One settlement of an uncertain commit acknowledgement",
    );
}

/// The run identity, which the diagnostic findings and every result relation reference.
pub(super) fn declare_run_identity(b: &mut RegistryBuilder) {
    identity(
        b,
        "run",
        "One run: an execution of a solve, simulation, fit or study, minted by the runtime before any effect. A durable run's tries are its attempts, and a retried job's attempts share its run; result rows name the run, the store and the publication name the attempt",
    );
}

/// The identities a publication record references (`runtime.publication_manifests`).
/// A registry that declares the publication contracts without the store declares these
/// alone ([`super::declare_publications`]).
pub(super) fn declare_publication_identities(b: &mut RegistryBuilder) {
    identity(b, "attempt", "One durable attempt: a single try of a run");
    identity(b, "workspace", "One publication workspace in the catalog");
    identity(
        b,
        "publication",
        "One publication: registered as an intent before its first member write and committed at most once",
    );
    identity(
        b,
        "reader_lease",
        "One reader lease protecting a publication",
    );
}

fn declare_sources(b: &mut RegistryBuilder) {
    b.declare_relation(store(
        "operational_source_bundles",
        &["bundle_hash"],
        vec![
            column("bundle_hash", T::hash()).with_owned_identity("source_bundle"),
            column("manifest", json()),
            column("created_at", ts()),
        ],
        "Authored sources stored for job execution. `bundle_hash` is the package content hash of §6.1 over the bundle's path/content pairs; `manifest` is a JSON document naming the paths.",
    ));
    b.declare_relation(
        store(
            "operational_source_documents",
            &["bundle_hash", "path"],
            vec![
                column("bundle_hash", T::hash())
                    .with_fk("runtime.operational_source_bundles", "bundle_hash"),
                column("path", text()),
                column("content_hash", T::hash()),
                column("content", bytes()),
            ],
            "One authored document of a source bundle, keyed by its path: its exact bytes, whose kind the path declares (a text document's are UTF-8, a data document's Parquet; ADR-0125), and `content_hash`, the document content hash.",
        )
        .check("path_nonempty", nonempty("path")),
    );
}

/// The termination columns, one per `TerminationClass` member.
const TERMINATIONS: [(&str, &str); 5] = [
    ("native", "termination_native"),
    ("run_state", "termination_run_state"),
    ("trajectory", "termination_trajectory"),
    ("runtime", "termination_runtime"),
    ("rule", "termination_rule"),
];

/// Exactly the column the class selects is present, and none without a class (X4).
fn one_termination() -> String {
    let absent = |selected: Option<&str>| {
        TERMINATIONS
            .iter()
            .map(|(_, column)| {
                if Some(*column) == selected {
                    format!("\"{column}\" IS NOT NULL")
                } else {
                    format!("\"{column}\" IS NULL")
                }
            })
            .collect::<Vec<_>>()
            .join(" AND ")
    };
    std::iter::once(format!(
        "(\"termination_class\" IS NULL AND {})",
        absent(None)
    ))
    .chain(TERMINATIONS.iter().map(|(class, column)| {
        format!(
            "(\"termination_class\" = '{class}' AND {})",
            absent(Some(column))
        )
    }))
    .collect::<Vec<_>>()
    .join(" OR ")
}

fn declare_attempts(b: &mut RegistryBuilder) {
    b.declare_relation(
        store(
            "operational_attempts",
            &["attempt_id"],
            vec![
                column("attempt_id", T::id()).with_owned_identity("attempt"),
                column("run_id", T::id()).with_identity("run"),
                column("kind", T::enumeration("AttemptKind")),
                column("request_identity", T::hash()),
                column("preparation_identity", T::hash()).optional(),
                column("state", T::enumeration("AttemptState")),
                column("state_version", int32()),
                column("parent_attempt", T::id())
                    .with_fk("runtime.operational_attempts", "attempt_id")
                    .optional(),
                column("worker", text()).optional(),
                column("lease_expires_at", ts()).optional(),
                column("heartbeat_at", ts()).optional(),
                column("cancel_requested", flag()),
                column("cancel_requested_at", ts()).optional(),
                column("termination_class", T::enumeration("TerminationClass")).optional(),
                column("termination_native", T::enumeration("NativeTermination")).optional(),
                column("termination_run_state", T::enumeration("NativeRunState")).optional(),
                column("termination_trajectory", T::enumeration("TrajectoryTermination"))
                    .optional(),
                column("termination_runtime", T::enumeration("RuntimeTermination")).optional(),
                column("termination_rule", T::enumeration("DiagnosticCode")).optional(),
                column("termination_detail", json()).optional(),
                column("created_at", ts()),
                column("updated_at", ts()),
                column("started_at", ts()).optional(),
                column("finished_at", ts()).optional(),
            ],
            "The durable attempt registry (DP-19). Identities are minted by the runtime. A running attempt holds exactly one lease; `cancel_requested` is the cancellation authority. The termination is typed: `termination_class` selects exactly one of the typed termination columns, and `termination_detail` is its versioned JSON detail. Published runtime.computation_runs and runtime.run_lineage are derived snapshots of a published attempt.",
        )
        .check("state_version_nonnegative", "\"state_version\" >= 0")
        .check(
            "parent_is_another_attempt",
            "\"parent_attempt\" IS DISTINCT FROM \"attempt_id\"",
        )
        .check("worker_nonempty", absent_or_nonempty("worker"))
        // A running attempt always has an owner and a lease; nothing else holds a lease.
        .check(
            "running_holds_lease",
            "(\"state\" = 'running') = (\"lease_expires_at\" IS NOT NULL)",
        )
        .check(
            "running_has_worker",
            "\"state\" <> 'running' OR \"worker\" IS NOT NULL",
        )
        .check(
            "cancel_request_timed",
            "\"cancel_requested\" = (\"cancel_requested_at\" IS NOT NULL)",
        )
        .check("one_termination", one_termination()),
    );
    b.declare_relation(
        store(
            "operational_attempt_transitions",
            &["attempt_id", "seq"],
            vec![
                attempt_ref("attempt_id"),
                column("seq", int32()),
                column("from_state", T::enumeration("AttemptState")).optional(),
                column("to_state", T::enumeration("AttemptState")),
                column("actor", text()).optional(),
                column("reason", text()).optional(),
                column("at", ts()),
            ],
            "Append-only audit of every attempt state change, written with the change; sequence 0 records creation and has no previous state.",
        )
        .check("seq_nonnegative", "\"seq\" >= 0")
        .check(
            "creation_has_no_previous_state",
            "(\"seq\" = 0) = (\"from_state\" IS NULL)",
        ),
    );
}

fn declare_jobs(b: &mut RegistryBuilder) {
    b.declare_relation(
        store(
            "operational_jobs",
            &["job_id"],
            vec![
                column("job_id", T::id()).with_owned_identity("job"),
                attempt_ref("attempt_id"),
                column("idempotency_key", text()),
                column("payload_version", int32()),
                column("payload", json()),
                column("priority", int32()),
                column("state", T::enumeration("JobState")),
                column("tries", int32()),
                column("max_tries", int32()),
                column("backoff_base_us", int64()),
                column("backoff_cap_us", int64()),
                column("available_at", ts()),
                column("enqueued_at", ts()),
                column("updated_at", ts()),
                column("last_error", text()).optional(),
            ],
            "Durable work claimed with FOR UPDATE SKIP LOCKED. `attempt_id` is the current try; each try runs as a new attempt. `payload` is a JSON document of format `payload_version`; a worker refuses versions it does not know. Backoff is in microseconds, doubled per retry up to the cap.",
        )
        // Each attempt is the current try of at most one job; an enqueue is idempotent.
        .unique("attempt_id", &["attempt_id"])
        .unique("idempotency_key", &["idempotency_key"])
        .check("idempotency_key_nonempty", nonempty("idempotency_key"))
        .check("payload_version_positive", "\"payload_version\" > 0")
        .check("tries_nonnegative", "\"tries\" >= 0")
        .check("max_tries_positive", "\"max_tries\" > 0")
        .check("tries_within_max", "\"tries\" <= \"max_tries\"")
        .check("backoff_base_nonnegative", "\"backoff_base_us\" >= 0")
        .check(
            "backoff_cap_covers_base",
            "\"backoff_cap_us\" >= \"backoff_base_us\"",
        ),
    );
}

fn declare_streams(b: &mut RegistryBuilder) {
    b.declare_relation(
        store(
            "operational_progress_events",
            &["attempt_id", "seq"],
            vec![
                attempt_ref("attempt_id"),
                column("seq", int64()),
                column("step", int32()),
                column("at", ts()),
                column("elapsed_seconds", real()),
                column("phase", text()),
            ],
            "Live progress of a durable attempt, numbered by its producer and retained by policy instead of a fixed event cap. `step` is the step of the run; `elapsed_seconds` is measured from the step's admitted execution start.",
        )
        .check("seq_nonnegative", "\"seq\" >= 0")
        .check("step_nonnegative", "\"step\" >= 0")
        .check("elapsed_nonnegative", "\"elapsed_seconds\" >= 0")
        .check("phase_nonempty", nonempty("phase")),
    );
    // The one-value rule is the `one_evidence_value` row check shared with
    // runtime.solve_metrics (`row_checks`).
    b.declare_relation(
        store(
            "operational_progress_values",
            &["attempt_id", "seq", "name"],
            vec![
                attempt_ref("attempt_id"),
                column("seq", int64()),
                column("name", text()),
                column("kind", T::enumeration("NativeMetricKind")),
                // A nonfinite real is recorded as unavailable `nonfinite`, never stored
                // (every registry Float64 is finite; the DDL states it per column).
                column("real", real()).optional(),
                column("integer", int64()).optional(),
                column("boolean", flag()).optional(),
                column("text", text()).optional(),
                column("unavailable", T::enumeration("EvidenceUnavailableReason")).optional(),
            ],
            "Typed values of one progress event, in the value vocabulary of runtime.solve_metrics: exactly the selected value field is populated, and numeric values are stored exactly. Publication snapshots them into runtime.solve_metrics as namespace `event.<seq>.<phase>`.",
        )
        .foreign_key(
            "progress_event",
            &["attempt_id", "seq"],
            "runtime.operational_progress_events",
            &["attempt_id", "seq"],
        )
        .check("name_nonempty", nonempty("name")),
    );
    b.declare_relation(
        store(
            "operational_incumbents",
            &["attempt_id", "seq"],
            vec![
                attempt_ref("attempt_id"),
                column("seq", int64()),
                column("step", int32()),
                column("at", ts()),
                column("elapsed_seconds", real()),
                column("phase", text()),
                column("objective", real()),
                // No bound yet, or an unbounded relaxation, is an absent bound: values
                // are finite like every registry Float64.
                column("dual_bound", real()).optional(),
                column("gap", real()).optional(),
                // What the search reported with the incumbent; absent when the native
                // value was not a count or a finite duration.
                column("nodes", int64()).optional(),
                column("seconds", real()).optional(),
                column("solution_id", T::id())
                    .with_fk("runtime.operational_solutions", "solution_id")
                    .optional(),
            ],
            "Improving feasible points of a branch-and-bound search and the bound at that time, numbered by the producer, with the progress context of the event that reported each: the step of the run, the phase, and `elapsed_seconds` from the step's admitted execution start. `nodes` and `seconds` are the search's node count and native running time then; `solution_id` names the stored point when it is kept for resumption.",
        )
        .check("seq_nonnegative", "\"seq\" >= 0")
        .check("step_nonnegative", "\"step\" >= 0")
        .check("elapsed_nonnegative", "\"elapsed_seconds\" >= 0")
        .check("phase_nonempty", nonempty("phase"))
        .check("nodes_nonnegative", "\"nodes\" IS NULL OR \"nodes\" >= 0")
        .check("seconds_nonnegative", "\"seconds\" IS NULL OR \"seconds\" >= 0")
        .check("gap_nonnegative", "\"gap\" IS NULL OR \"gap\" >= 0"),
    );
}

/// The vectors a seed kind carries (the former `solutions_vectors_check`): a root seed is
/// a primal only; an NLP seed is a primal with optional paired bound duals, row duals and
/// barrier; a HiGHS seed holds any of a primal, paired column/row duals and a paired
/// basis, and at least one.
const SEED_VECTORS: &str = "(\"kind\" = 'root' AND \"primal\" IS NOT NULL AND \"lower_bound_duals\" IS NULL AND \"upper_bound_duals\" IS NULL AND \"column_duals\" IS NULL AND \"row_duals\" IS NULL AND \"barrier\" IS NULL AND \"basis_columns\" IS NULL AND \"basis_rows\" IS NULL) OR (\"kind\" = 'nlp' AND \"primal\" IS NOT NULL AND (\"lower_bound_duals\" IS NULL) = (\"upper_bound_duals\" IS NULL) AND \"column_duals\" IS NULL AND \"basis_columns\" IS NULL AND \"basis_rows\" IS NULL) OR (\"kind\" = 'highs' AND \"lower_bound_duals\" IS NULL AND \"upper_bound_duals\" IS NULL AND \"barrier\" IS NULL AND (\"column_duals\" IS NULL) = (\"row_duals\" IS NULL) AND (\"basis_columns\" IS NULL) = (\"basis_rows\" IS NULL) AND (\"primal\" IS NOT NULL OR \"column_duals\" IS NOT NULL OR \"basis_columns\" IS NOT NULL))";

fn declare_solutions(b: &mut RegistryBuilder) {
    let vector = || T::list(real());
    let codes = || T::list(int32());
    b.declare_relation(
        store(
            "operational_solutions",
            &["solution_id"],
            vec![
                column("solution_id", T::id()).with_owned_identity("solution"),
                column("compatibility_stamp", T::hash()),
                column("preparation_identity", T::hash()),
                column("kind", T::enumeration("StoredSeedKind")),
                column("origin", T::enumeration("StoredSolutionOrigin")),
                column("backend", T::enumeration("NativeBackend")),
                column("profile_stamp", T::hash()),
                column("data_stamp", T::hash()),
                column("primal", vector()).optional(),
                column("lower_bound_duals", vector()).optional(),
                column("upper_bound_duals", vector()).optional(),
                column("column_duals", vector()).optional(),
                column("row_duals", vector()).optional(),
                // The final barrier parameter of an NLP seed's interior-point producer.
                column("barrier", real()).optional(),
                column("basis_columns", codes()).optional(),
                column("basis_rows", codes()).optional(),
                attempt_ref("created_by").optional(),
                column("created_at", ts()),
            ],
            "Reusable seeds in original source coordinates, keyed by the coordinate-compatibility stamp (the layout stamp) and the preparation identity. `kind` fixes which vectors are present; an NLP seed may carry the final barrier parameter of its interior-point producer (authored objective units); basis codes keep the native integer statuses. `origin` is `output` for a step's accepted output seed, the only kind the newest-compatible lookup returns, and `incumbent` for a point captured from an attempt's incumbent stream: a capture belongs to its attempt and expires with that stream unless a queued job's start or a waiting study point still names it. The seed's content identity enters the lineage of every result it seeds (F25).",
        )
        .check("vectors", SEED_VECTORS)
        .check(
            "capture_has_attempt",
            "\"origin\" <> 'incumbent' OR \"created_by\" IS NOT NULL",
        )
        .check("barrier_positive", "\"barrier\" IS NULL OR \"barrier\" > 0"),
    );
}

fn job_ref(name: &'static str) -> T {
    column(name, T::id()).with_fk("runtime.operational_jobs", "job_id")
}

fn declare_studies(b: &mut RegistryBuilder) {
    b.declare_relation(
        store(
            "operational_studies",
            &["study_id"],
            vec![
                column("study_id", T::id()).with_owned_identity("study"),
                attempt_ref("attempt_id"),
                column("publication_id", T::id())
                    .with_fk("runtime.operational_publication_intents", "publication_id"),
                job_ref("finalization_job"),
                column("definition", json()),
                column("state", T::enumeration("StudyState")),
                column("created_at", ts()),
                column("updated_at", ts()),
            ],
            "Studies coordinated across workers (Plan 22 O7). `attempt_id` is the study's own attempt: it holds no lease, stays queued while the points run and ends when the last point is terminal; it is the attempt the study's one publication names, and `publication_id` is that publication's intent, registered at creation. `finalization_job` waits until the last point is terminal and then publishes the study. `definition` is the study's versioned JSON definition.",
        )
        // A study owns its attempt, its intent and its finalization job.
        .unique("attempt_id", &["attempt_id"])
        .unique("publication_id", &["publication_id"])
        .unique("finalization_job", &["finalization_job"]),
    );
    b.declare_relation(
        store(
            "operational_study_points",
            &["study_id", "point_index"],
            vec![
                column("study_id", T::id()).with_fk("runtime.operational_studies", "study_id"),
                column("point_index", int32()),
                column("binding_hash", T::hash()),
                column("predecessor", int32()).optional(),
                job_ref("job_id"),
                column("state", T::enumeration("StudyPointState")),
                column("updated_at", ts()),
            ],
            "One study point: its value bindings by hash, the earlier point whose stored solution seeds it, its job (whose current attempt is the point's try) and its state. A point with a predecessor waits until the predecessor completed, and is cancelled when the predecessor fails or is cancelled.",
        )
        // Each job runs one point; a binding appears once per study.
        .unique("job_id", &["job_id"])
        .unique("binding", &["study_id", "binding_hash"])
        .foreign_key(
            "predecessor",
            &["study_id", "predecessor"],
            "runtime.operational_study_points",
            &["study_id", "point_index"],
        )
        .check("point_index_nonnegative", "\"point_index\" >= 0")
        .check(
            "predecessor_is_earlier",
            "\"predecessor\" IS NULL OR (\"predecessor\" >= 0 AND \"predecessor\" < \"point_index\")",
        ),
    );
    b.declare_relation(
        store(
            "operational_study_point_members",
            &[
                "study_id",
                "point_index",
                "catalog_name",
                "schema_name",
                "table_name",
            ],
            vec![
                column("study_id", T::id()).with_fk("runtime.operational_studies", "study_id"),
                column("point_index", int32()),
                column("catalog_name", text()),
                column("schema_name", text()),
                column("table_name", text()),
                column("relation_id", T::id()),
                column("relation_version", int64()),
                column("contract_fingerprint", T::hash()),
                column("table_uri", text()),
                column("delta_version", int64()),
                column("selection_kind", T::enumeration("MemberSelectionKind")),
                column("revision_column", text()).optional(),
                column("revision_id", T::id()).optional(),
            ],
            "The result members a completed study point wrote under its study's publication intent, one registry `MemberDescriptor` per row, recorded in the transaction that completes the point. The study's publication commits them together with its summary.",
        )
        .foreign_key(
            "point",
            &["study_id", "point_index"],
            "runtime.operational_study_points",
            &["study_id", "point_index"],
        )
        .check("catalog_name_nonempty", nonempty("catalog_name"))
        .check("schema_name_nonempty", nonempty("schema_name"))
        .check("table_name_nonempty", nonempty("table_name"))
        .check("table_uri_nonempty", nonempty("table_uri"))
        .check("relation_version_nonnegative", "\"relation_version\" >= 0")
        .check("delta_version_nonnegative", "\"delta_version\" >= 0")
        .check("one_selection", ONE_SELECTION),
    );
}

/// A member selects the full table, or one revision by a named column (the former
/// `one_selection` check of the catalog's members).
const ONE_SELECTION: &str = "(\"selection_kind\" = 'full' AND \"revision_column\" IS NULL AND \"revision_id\" IS NULL) OR (\"selection_kind\" = 'revision' AND \"revision_column\" IS NOT NULL AND \"revision_column\" <> '' AND \"revision_id\" IS NOT NULL)";

fn declare_catalog(b: &mut RegistryBuilder) {
    b.declare_relation(
        store(
            "operational_workspaces",
            &["workspace_id"],
            vec![
                column("workspace_id", T::id()).with_owned_identity("workspace"),
                column("name", text()),
                column("root_uri", text()),
                column("maintenance_epoch", int64()),
                column("created_at", ts()),
            ],
            "A publication workspace: a named root under which members are written. Its head row is created with it. `maintenance_epoch` advances before every maintenance effect (retirement, collection), so a reader's cache scope never outlives the maintenance it was read under (Plan 22 X10).",
        )
        .unique("name", &["name"])
        .unique("root_uri", &["root_uri"])
        .check("name_nonempty", nonempty("name"))
        .check("root_uri_nonempty", nonempty("root_uri"))
        .check("maintenance_epoch_nonnegative", "\"maintenance_epoch\" >= 0"),
    );
    b.declare_relation(
        store(
            "operational_publication_intents",
            &["publication_id"],
            vec![
                column("publication_id", T::id()).with_owned_identity("publication"),
                column("workspace_id", T::id())
                    .with_fk("runtime.operational_workspaces", "workspace_id"),
                attempt_ref("attempt_id"),
                column("member_prefix", text()),
                column("prepared_at", ts()),
                column("abandoned_at", ts()).optional(),
                column("reclaimed_at", ts()).optional(),
            ],
            "Publication intents (Plan 22 X9), registered before the first member write: the publication identity, its durable attempt and the prefix every member it writes lives under. An intent without a publication is unpublished: reclaimable once abandoned, once its attempt is published as another publication, or once its attempt is stale or superseded. An abandoned intent never commits.",
        )
        .unique("member_prefix", &["member_prefix"])
        .unique("identity", &["publication_id", "workspace_id", "attempt_id"])
        .check("member_prefix_nonempty", nonempty("member_prefix"))
        .check(
            "reclaimed_after_abandoned",
            "\"reclaimed_at\" IS NULL OR \"abandoned_at\" IS NOT NULL",
        ),
    );
    b.declare_relation(
        store(
            "operational_publications",
            &["publication_id"],
            vec![
                column("publication_id", T::id())
                    .with_fk("runtime.operational_publication_intents", "publication_id"),
                column("workspace_id", T::id())
                    .with_fk("runtime.operational_workspaces", "workspace_id"),
                publication_ref("parent_publication").optional(),
                attempt_ref("attempt_id"),
                column("kind", T::enumeration("PublicationKind")),
                column("committed_at", ts()),
            ],
            "Immutable publication records. Each commits its registered intent; the attempt identity is unique, so publication is idempotent per attempt and settlement queries this relation.",
        )
        .unique("attempt_id", &["attempt_id"])
        .foreign_key(
            "intent",
            &["publication_id", "workspace_id", "attempt_id"],
            "runtime.operational_publication_intents",
            &["publication_id", "workspace_id", "attempt_id"],
        )
        .check(
            "parent_is_another_publication",
            "\"parent_publication\" IS DISTINCT FROM \"publication_id\"",
        ),
    );
    b.declare_relation(store(
        "operational_publication_heads",
        &["workspace_id"],
        vec![
            column("workspace_id", T::id())
                .with_fk("runtime.operational_workspaces", "workspace_id"),
            publication_ref("publication_id").optional(),
            column("advanced_at", ts()),
        ],
        "One head per workspace, created with the workspace, so the compare-and-set commit always locks an existing row. An absent head means nothing has been published yet.",
    ));
    b.declare_relation(
        store(
            "operational_publication_members",
            &[
                "publication_id",
                "role",
                "catalog_name",
                "schema_name",
                "table_name",
            ],
            vec![
                publication_ref("publication_id"),
                column("role", T::enumeration("PublicationMemberRole")),
                column("catalog_name", text()),
                column("schema_name", text()),
                column("table_name", text()),
                column("relation_id", T::id()),
                column("relation_version", int64()),
                column("contract_fingerprint", T::hash()),
                column("table_uri", text()),
                column("delta_version", int64()),
                column("selection_kind", T::enumeration("MemberSelectionKind")),
                column("revision_column", text()).optional(),
                column("revision_id", T::id()).optional(),
            ],
            "The members of a publication, one registry `MemberDescriptor` per row: an output it publishes or an input it read, each naming an exact Delta version of a table, the relation contract it was written under and the rows selected (the full table or one revision).",
        )
        .check("catalog_name_nonempty", nonempty("catalog_name"))
        .check("schema_name_nonempty", nonempty("schema_name"))
        .check("table_name_nonempty", nonempty("table_name"))
        .check("table_uri_nonempty", nonempty("table_uri"))
        .check("relation_version_nonnegative", "\"relation_version\" >= 0")
        .check("delta_version_nonnegative", "\"delta_version\" >= 0")
        .check("one_selection", ONE_SELECTION),
    );
    b.declare_relation(
        store(
            "operational_publication_windows",
            &["publication_id", "table_uri", "from_version"],
            vec![
                publication_ref("publication_id"),
                column("table_uri", text()),
                column("from_version", int64()),
                column("through_version", int64()),
            ],
            "Change-data windows a publication read: every version of the table from `from_version` through `through_version` (inclusive) stays reachable while the publication is live (retention reason `changes`, finding T16).",
        )
        .check("table_uri_nonempty", nonempty("table_uri"))
        .check("from_version_nonnegative", "\"from_version\" >= 0")
        .check("ordered_window", "\"from_version\" <= \"through_version\""),
    );
    b.declare_relation(
        store(
            "operational_settlements",
            &["settlement_id"],
            vec![
                column("settlement_id", T::id()).with_owned_identity("settlement"),
                attempt_ref("attempt_id"),
                column("outcome", T::enumeration("SettlementOutcome")),
                publication_ref("publication_id").optional(),
                column("reason", text()).optional(),
                publication_ref("conflict_head").optional(),
                column("settled_at", ts()),
            ],
            "Settlement inquiries after an uncertain commit acknowledgement, and their outcome: a committed outcome names its publication; a conflict names why and the head it met.",
        )
        .check(
            "committed_names_publication",
            "(\"outcome\" = 'committed') = (\"publication_id\" IS NOT NULL)",
        )
        .check(
            "conflict_has_reason",
            "(\"outcome\" = 'conflict') = (\"reason\" IS NOT NULL)",
        )
        .check("reason_nonempty", absent_or_nonempty("reason"))
        .check(
            "conflict_head_of_conflict",
            "\"conflict_head\" IS NULL OR \"outcome\" = 'conflict'",
        ),
    );
    b.declare_relation(
        store(
            "operational_reader_leases",
            &["lease_id"],
            vec![
                column("lease_id", T::id()).with_owned_identity("reader_lease"),
                publication_ref("publication_id"),
                column("head_of", T::id())
                    .with_fk("runtime.operational_workspaces", "workspace_id")
                    .optional(),
                column("holder", text()),
                column("acquired_at", ts()),
                column("expires_at", ts()),
                column("released_at", ts()).optional(),
            ],
            "Reader leases: taken, renewed and released in short transactions; no reader holds a database session while it reads Delta files (finding T02). A lease that resolved a workspace head records that workspace in `head_of`; an export is a lease held by `export:<destination>`.",
        )
        .check("holder_nonempty", nonempty("holder"))
        .check(
            "expires_after_acquired",
            "\"expires_at\" > \"acquired_at\"",
        ),
    );
    b.declare_relation(
        store(
            "operational_retention_marks",
            &["publication_id"],
            vec![
                publication_ref("publication_id"),
                column("phase", T::enumeration("RetentionPhase")),
                column("marked_at", ts()),
                column("deleted_at", ts()).optional(),
            ],
            "Two-phase deletion state. A publication without a mark is live; `expiring` refuses new leases; `deleted` records that the member files were removed.",
        )
        .check(
            "deleted_when_marked_deleted",
            "(\"phase\" = 'deleted') = (\"deleted_at\" IS NOT NULL)",
        ),
    );
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "assertions over the platform registry")]

    use crate::model::IdentityBase;

    #[test]
    fn catalog_relations_registered() {
        let registry = crate::registry().unwrap();
        let tables = crate::store::relations(registry)
            .into_iter()
            .map(|(table, _)| table)
            .collect::<Vec<_>>();
        for table in [
            "attempt_transitions",
            "attempts",
            "incumbents",
            "jobs",
            "progress_events",
            "progress_values",
            "publication_heads",
            "publication_intents",
            "publication_members",
            "publication_windows",
            "publications",
            "reader_leases",
            "retention_marks",
            "settlements",
            "solutions",
            "source_bundles",
            "source_documents",
            "studies",
            "study_point_members",
            "study_points",
            "workspaces",
        ] {
            assert!(tables.contains(&table), "{table} is not a store relation");
        }
        assert_eq!(tables.len(), 21);
        // Identities are owned by the keys declared their owners and inherited through
        // references.
        let owner = |name: &str| {
            registry
                .identity(name)
                .and_then(|identity| identity.owner.as_ref())
                .map(|owner| (owner.relation.clone(), owner.column.clone()))
        };
        assert_eq!(
            owner("workspace"),
            Some((
                "runtime.operational_workspaces".to_owned(),
                "workspace_id".to_owned()
            ))
        );
        assert_eq!(owner("run"), None);
        // A publication is owned by its intent, registered before any member write
        // (Plan 22 X9); the committed publication inherits it through its reference.
        assert_eq!(
            owner("publication"),
            Some((
                "runtime.operational_publication_intents".to_owned(),
                "publication_id".to_owned()
            ))
        );
        // The one-row export manifest types its key without owning the identity.
        assert_eq!(
            registry
                .relation("runtime.publication_manifests")
                .and_then(|spec| spec.column("publication_id"))
                .and_then(crate::model::FieldContract::identity),
            Some("publication")
        );
        assert_eq!(
            registry
                .identity("source_bundle")
                .map(|identity| identity.base),
            Some(IdentityBase::ContentHash)
        );
        let carried = |relation: &str, column: &str| {
            registry
                .relation(relation)
                .and_then(|spec| spec.column(column))
                .and_then(crate::model::FieldContract::identity)
        };
        assert_eq!(
            carried("runtime.operational_publication_heads", "publication_id"),
            Some("publication")
        );
        assert_eq!(
            carried("runtime.operational_progress_values", "attempt_id"),
            Some("attempt")
        );
        assert_eq!(
            carried("runtime.operational_source_documents", "bundle_hash"),
            Some("source_bundle")
        );
        // Every former CHECK is a named, declared constraint.
        let attempts = registry.relation("runtime.operational_attempts").unwrap();
        for check in [
            "running_holds_lease",
            "running_has_worker",
            "cancel_request_timed",
            "one_termination",
        ] {
            assert!(attempts.checks.contains_key(check), "{check}");
        }
        let values = registry
            .relation("runtime.operational_progress_values")
            .unwrap();
        assert!(values.checks.contains_key("one_evidence_value"));
        assert_eq!(
            values.foreign_keys[0].target,
            "runtime.operational_progress_events"
        );
        // The catalog's O8 declarations: the flattened member descriptor with its role and
        // selection rule, the settlement conflict, the windows and the intent reference.
        let members = registry
            .relation("runtime.operational_publication_members")
            .unwrap();
        assert!(members.checks.contains_key("one_selection"));
        assert_eq!(
            members.column("role").unwrap().enum_name(),
            Some("PublicationMemberRole")
        );
        let publications = registry
            .relation("runtime.operational_publications")
            .unwrap();
        assert_eq!(
            publications.column("kind").unwrap().enum_name(),
            Some("PublicationKind")
        );
        assert_eq!(
            publications.foreign_keys[0].target,
            "runtime.operational_publication_intents"
        );
        let workspaces = registry.relation("runtime.operational_workspaces").unwrap();
        assert!(workspaces.column("maintenance_epoch").is_some());
        assert!(
            workspaces
                .unique_keys
                .iter()
                .any(|key| key.columns == ["root_uri"])
        );
        let settlement = registry.enum_spec("SettlementOutcome").unwrap();
        assert!(settlement.members.iter().any(|m| m.name == "conflict"));
        let reasons = registry.enum_spec("RetentionReason").unwrap();
        assert!(reasons.members.iter().all(|m| m.name != "output"));
        assert!(
            registry
                .relation("runtime.operational_publication_windows")
                .unwrap()
                .checks
                .contains_key("ordered_window")
        );
        // The O7 declarations: a study owns its coordinating attempt, its publication
        // intent and its finalization job; a point references its job and, by a composite
        // self-reference, its predecessor; completed points record their result members.
        let studies = registry.relation("runtime.operational_studies").unwrap();
        for key in ["attempt_id", "publication_id", "finalization_job"] {
            assert!(
                studies
                    .unique_keys
                    .iter()
                    .any(|unique| unique.columns == [key]),
                "{key}"
            );
        }
        assert_eq!(
            carried("runtime.operational_studies", "attempt_id"),
            Some("attempt")
        );
        assert_eq!(
            carried("runtime.operational_studies", "publication_id"),
            Some("publication")
        );
        assert_eq!(
            carried("runtime.operational_study_points", "job_id"),
            Some("job")
        );
        let points = registry
            .relation("runtime.operational_study_points")
            .unwrap();
        assert!(points.checks.contains_key("predecessor_is_earlier"));
        assert!(points.foreign_keys.iter().any(|reference| {
            reference.target == "runtime.operational_study_points"
                && reference.columns == ["study_id", "predecessor"]
        }));
        let point_members = registry
            .relation("runtime.operational_study_point_members")
            .unwrap();
        assert!(point_members.checks.contains_key("one_selection"));
        assert_eq!(
            point_members.foreign_keys[0].target,
            "runtime.operational_study_points"
        );
        let kinds = registry.enum_spec("AttemptKind").unwrap();
        assert!(kinds.members.iter().any(|m| m.name == "study"));
        let jobs = registry.enum_spec("JobState").unwrap();
        assert!(jobs.members.iter().any(|m| m.name == "waiting"));
    }

    #[test]
    fn operational_timestamps_are_microseconds() {
        let registry = crate::registry().unwrap();
        let micros = crate::model::extension::timestamp_micros_storage();
        let mut timestamps = 0;
        for (_, spec) in crate::store::relations(registry) {
            for column in &spec.columns {
                if let arrow_schema::DataType::Timestamp(..) = column.data_type() {
                    assert_eq!(
                        column.data_type(),
                        micros,
                        "{}.{} is not ts_us",
                        spec.qualified_name(),
                        column.name()
                    );
                    assert_eq!(column.value_type().type_name().unwrap(), "ts_us");
                    timestamps += 1;
                }
            }
        }
        assert!(timestamps > 20);
    }
}
