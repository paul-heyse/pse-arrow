// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The meaning of the operational relations (ADR-0112 Outcome 10; Plan 22 architecture §9).
//!
//! PostgreSQL owns what changes and Delta owns what is published. The registry owns the
//! meaning of the operational relations: these declarations are the one authority, and the
//! `pse_ops` SQL migrations are their physical representation, checked column by column by
//! the migration-conformance test in `pse-operations`. Relation `runtime.operational_<t>`
//! is represented by table `pse_ops.<t>`. Enumerations are stored as text with a CHECK on
//! exactly these spellings; there are no database enum types.
//!
//! The publication catalog tables (workspaces, heads, members, settlements, reader leases
//! and retention marks) are declared with the catalog commit path.
use super::declarations::{column, enumeration, relation};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::{DataType as D, TimeUnit};

fn text() -> T {
    T::native(D::Utf8)
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
/// The `ts` logical type (blueprint §4.5); PostgreSQL `timestamptz`.
fn ts() -> T {
    T::native(D::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())))
}
fn attempt_ref(name: &'static str) -> T {
    column(name, T::id()).with_fk("runtime.operational_attempts", "attempt_id")
}

pub(super) fn declare(b: &mut RegistryBuilder) {
    declare_enumerations(b);
    declare_sources(b);
    declare_attempts(b);
    declare_jobs(b);
    declare_streams(b);
    declare_solutions(b);
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
    // The queue state of a job; each try runs as its own attempt.
    enumeration(
        b,
        "JobState",
        ["queued", "running", "completed", "failed", "cancelled"],
    );
    // What an attempt computes: an authored algebraic sequence, a simulation or a fit.
    enumeration(b, "AttemptKind", ["modeling", "simulation", "fit"]);
    enumeration(b, "StudyState", ["open", "completed", "cancelled"]);
    enumeration(
        b,
        "StudyPointState",
        ["pending", "assigned", "completed", "failed", "cancelled"],
    );
    // The native meaning of a stored seed's vectors (the portable warm-start payloads).
    enumeration(b, "StoredSeedKind", ["root", "nlp", "highs"]);
}

fn declare_sources(b: &mut RegistryBuilder) {
    relation(
        b,
        N::Runtime,
        "operational_source_bundles",
        S::Sidecar,
        &["bundle_hash"],
        vec![
            column("bundle_hash", T::hash()),
            column("manifest", text()),
            column("created_at", ts()),
        ],
        "Authored sources stored for job execution. `bundle_hash` is the package content hash of §6.1 over the bundle's path/text pairs; `manifest` is a JSON document naming the paths.",
    );
    relation(
        b,
        N::Runtime,
        "operational_source_documents",
        S::Sidecar,
        &["bundle_hash", "path"],
        vec![
            column("bundle_hash", T::hash())
                .with_fk("runtime.operational_source_bundles", "bundle_hash"),
            column("path", text()),
            column("content_hash", T::hash()),
            column("content", text()),
        ],
        "One authored document of a source bundle, keyed by its path; `content_hash` is the document content hash.",
    );
}

fn declare_attempts(b: &mut RegistryBuilder) {
    relation(
        b,
        N::Runtime,
        "operational_attempts",
        S::Sidecar,
        &["attempt_id"],
        vec![
            column("attempt_id", T::id()),
            column("run_id", T::id()),
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
            column("termination", text()).optional(),
            column("termination_detail", text()).optional(),
            column("created_at", ts()),
            column("updated_at", ts()),
            column("started_at", ts()).optional(),
            column("finished_at", ts()).optional(),
        ],
        "The durable attempt registry (DP-19). Identities are minted by the runtime. A running attempt holds exactly one lease; `cancel_requested` is the cancellation authority. `termination` is the typed termination code and `termination_detail` its versioned JSON detail. Published runtime.computation_runs and runtime.run_lineage are derived snapshots of a published attempt.",
    );
    relation(
        b,
        N::Runtime,
        "operational_attempt_transitions",
        S::Sidecar,
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
    );
}

fn declare_jobs(b: &mut RegistryBuilder) {
    relation(
        b,
        N::Runtime,
        "operational_jobs",
        S::Sidecar,
        &["job_id"],
        vec![
            column("job_id", T::id()),
            attempt_ref("attempt_id"),
            column("idempotency_key", text()),
            column("payload_version", int32()),
            column("payload", text()),
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
    );
}

fn declare_streams(b: &mut RegistryBuilder) {
    relation(
        b,
        N::Runtime,
        "operational_progress_events",
        S::Sidecar,
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
    );
    relation(
        b,
        N::Runtime,
        "operational_progress_values",
        S::Sidecar,
        &["attempt_id", "seq", "name"],
        vec![
            attempt_ref("attempt_id"),
            column("seq", int64()),
            column("name", text()),
            column("kind", T::enumeration("NativeMetricKind")),
            column("real", real()).optional(),
            column("integer", int64()).optional(),
            column("boolean", flag()).optional(),
            column("text", text()).optional(),
            column("unavailable", T::enumeration("EvidenceUnavailableReason")).optional(),
        ],
        "Typed values of one progress event, in the value vocabulary of runtime.solve_metrics: exactly the selected value field is populated, and numeric values are stored exactly. Publication snapshots them into runtime.solve_metrics as namespace `event.<seq>.<phase>`.",
    );
    relation(
        b,
        N::Runtime,
        "operational_incumbents",
        S::Sidecar,
        &["attempt_id", "seq"],
        vec![
            attempt_ref("attempt_id"),
            column("seq", int64()),
            column("at", ts()),
            column("objective", real()),
            column("dual_bound", real()).optional(),
            column("gap", real()).optional(),
            column("solution_id", T::id())
                .with_fk("runtime.operational_solutions", "solution_id")
                .optional(),
        ],
        "Improving feasible points and the bound at that time, numbered by the producer; `solution_id` names the stored point when it is kept for resumption.",
    );
}

fn declare_solutions(b: &mut RegistryBuilder) {
    let vector = || T::list(real());
    let codes = || T::list(int32());
    relation(
        b,
        N::Runtime,
        "operational_solutions",
        S::Sidecar,
        &["solution_id"],
        vec![
            column("solution_id", T::id()),
            column("compatibility_stamp", T::hash()),
            column("preparation_identity", T::hash()),
            column("kind", T::enumeration("StoredSeedKind")),
            column("backend", T::enumeration("NativeBackend")),
            column("profile_stamp", T::hash()),
            column("data_stamp", T::hash()),
            column("primal", vector()).optional(),
            column("lower_bound_duals", vector()).optional(),
            column("upper_bound_duals", vector()).optional(),
            column("column_duals", vector()).optional(),
            column("row_duals", vector()).optional(),
            column("basis_columns", codes()).optional(),
            column("basis_rows", codes()).optional(),
            attempt_ref("created_by").optional(),
            column("created_at", ts()),
        ],
        "Reusable seeds in original source coordinates, keyed by the coordinate-compatibility stamp (the layout stamp) and the preparation identity. `kind` fixes which vectors are present; basis codes keep the native integer statuses. The seed's content identity enters the lineage of every result it seeds (F25).",
    );
}

fn declare_studies(b: &mut RegistryBuilder) {
    relation(
        b,
        N::Runtime,
        "operational_studies",
        S::Sidecar,
        &["study_id"],
        vec![
            column("study_id", T::id()),
            column("definition", text()),
            column("state", T::enumeration("StudyState")),
            column("created_at", ts()),
            column("updated_at", ts()),
        ],
        "Study coordination state; `definition` is the study's versioned JSON definition.",
    );
    relation(
        b,
        N::Runtime,
        "operational_study_points",
        S::Sidecar,
        &["study_id", "point_index"],
        vec![
            column("study_id", T::id()).with_fk("runtime.operational_studies", "study_id"),
            column("point_index", int32()),
            column("binding_hash", T::hash()),
            column("state", T::enumeration("StudyPointState")),
            attempt_ref("attempt_id").optional(),
            column("result_ref", text()).optional(),
            column("updated_at", ts()),
        ],
        "One study point: its value bindings by hash, its claim state and the attempt that ran it.",
    );
}
