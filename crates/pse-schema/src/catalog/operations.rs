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
    declare_studies(b);
    declare_catalog(b);
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
    // Which typed column names an attempt's termination (X4): the last native solver
    // termination, the last run state when no native termination was reported, a
    // trajectory termination, a runtime-owned outcome, or a violated named rule.
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
    // The outcome of settling an uncertain commit acknowledgement.
    enumeration(b, "SettlementOutcome", ["committed", "proved_noncommit"]);
}

fn declare_identities(b: &mut RegistryBuilder) {
    identity(b, "attempt", "One durable attempt: a single try of a run");
    identity(
        b,
        "run",
        "A run: the logical request its attempts try, minted by the runtime",
    );
    identity(b, "job", "One durable job claimed by workers");
    identity(b, "solution", "One stored reusable solution (a warm-start seed)");
    identity(b, "study", "One study coordinated across its points");
    identity(
        b,
        "source_bundle",
        "An authored source bundle, content-addressed by its package content hash",
    );
    identity(b, "workspace", "One publication workspace in the catalog");
    identity(b, "publication", "One committed publication in the catalog");
    identity(b, "settlement", "One settlement of an uncertain commit acknowledgement");
    identity(b, "reader_lease", "One reader lease protecting a publication");
}

fn declare_sources(b: &mut RegistryBuilder) {
    b.declare_relation(store(
        "operational_source_bundles",
        &["bundle_hash"],
        vec![
            column("bundle_hash", T::hash()).with_identity("source_bundle"),
            column("manifest", json()),
            column("created_at", ts()),
        ],
        "Authored sources stored for job execution. `bundle_hash` is the package content hash of §6.1 over the bundle's path/text pairs; `manifest` is a JSON document naming the paths.",
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
                column("content", text()),
            ],
            "One authored document of a source bundle, keyed by its path; `content_hash` is the document content hash.",
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
    std::iter::once(format!("(\"termination_class\" IS NULL AND {})", absent(None)))
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
                column("attempt_id", T::id()).with_identity("attempt"),
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
                column("termination_rule", text()).optional(),
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
        .check("one_termination", one_termination())
        .check("termination_rule_nonempty", absent_or_nonempty("termination_rule")),
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
                column("job_id", T::id()).with_identity("job"),
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
                column("at", ts()),
                column("objective", real()),
                // No bound yet, or an unbounded relaxation, is an absent bound: values
                // are finite like every registry Float64.
                column("dual_bound", real()).optional(),
                column("gap", real()).optional(),
                column("solution_id", T::id())
                    .with_fk("runtime.operational_solutions", "solution_id")
                    .optional(),
            ],
            "Improving feasible points and the bound at that time, numbered by the producer; `solution_id` names the stored point when it is kept for resumption.",
        )
        .check("seq_nonnegative", "\"seq\" >= 0")
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
                column("solution_id", T::id()).with_identity("solution"),
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
                // The final barrier parameter of an NLP seed's interior-point producer.
                column("barrier", real()).optional(),
                column("basis_columns", codes()).optional(),
                column("basis_rows", codes()).optional(),
                attempt_ref("created_by").optional(),
                column("created_at", ts()),
            ],
            "Reusable seeds in original source coordinates, keyed by the coordinate-compatibility stamp (the layout stamp) and the preparation identity. `kind` fixes which vectors are present; an NLP seed may carry the final barrier parameter of its interior-point producer (authored objective units); basis codes keep the native integer statuses. The seed's content identity enters the lineage of every result it seeds (F25).",
        )
        .check("vectors", SEED_VECTORS)
        .check("barrier_positive", "\"barrier\" IS NULL OR \"barrier\" > 0"),
    );
}

fn declare_studies(b: &mut RegistryBuilder) {
    b.declare_relation(store(
        "operational_studies",
        &["study_id"],
        vec![
            column("study_id", T::id()).with_identity("study"),
            column("definition", json()),
            column("state", T::enumeration("StudyState")),
            column("created_at", ts()),
            column("updated_at", ts()),
        ],
        "Study coordination state; `definition` is the study's versioned JSON definition.",
    ));
    b.declare_relation(
        store(
            "operational_study_points",
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
        )
        // Each attempt runs at most one point; a binding appears once per study.
        .unique("attempt_id", &["attempt_id"])
        .unique("binding", &["study_id", "binding_hash"])
        .check("point_index_nonnegative", "\"point_index\" >= 0"),
    );
}

fn declare_catalog(b: &mut RegistryBuilder) {
    b.declare_relation(
        store(
            "operational_workspaces",
            &["workspace_id"],
            vec![
                column("workspace_id", T::id()).with_identity("workspace"),
                column("name", text()),
                column("root_uri", text()),
                column("created_at", ts()),
            ],
            "A publication workspace: a named root under which members are written. Its head row is created with it.",
        )
        .unique("name", &["name"])
        .check("name_nonempty", nonempty("name"))
        .check("root_uri_nonempty", nonempty("root_uri")),
    );
    b.declare_relation(
        store(
            "operational_publications",
            &["publication_id"],
            vec![
                column("publication_id", T::id()).with_identity("publication"),
                column("workspace_id", T::id())
                    .with_fk("runtime.operational_workspaces", "workspace_id"),
                publication_ref("parent_publication").optional(),
                attempt_ref("attempt_id"),
                column("committed_at", ts()),
            ],
            "Immutable publication records. The attempt identity is unique: publication is idempotent per attempt, and settlement queries this relation.",
        )
        .unique("attempt_id", &["attempt_id"])
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
            &["publication_id", "member"],
            vec![
                publication_ref("publication_id"),
                column("member", text()),
                column("table_uri", text()),
                column("delta_version", int64()),
                column("contract_fingerprint", T::hash()),
            ],
            "The members of a publication: each names an exact Delta version of a table and the contract fingerprint it was written under.",
        )
        .check("member_nonempty", nonempty("member"))
        .check("table_uri_nonempty", nonempty("table_uri"))
        .check("delta_version_nonnegative", "\"delta_version\" >= 0"),
    );
    b.declare_relation(
        store(
            "operational_settlements",
            &["settlement_id"],
            vec![
                column("settlement_id", T::id()).with_identity("settlement"),
                attempt_ref("attempt_id"),
                column("outcome", T::enumeration("SettlementOutcome")),
                publication_ref("publication_id").optional(),
                column("settled_at", ts()),
            ],
            "Settlement inquiries after an uncertain commit acknowledgement, and their outcome; a committed outcome names its publication.",
        )
        .check(
            "committed_names_publication",
            "(\"outcome\" = 'committed') = (\"publication_id\" IS NOT NULL)",
        ),
    );
    b.declare_relation(
        store(
            "operational_reader_leases",
            &["lease_id"],
            vec![
                column("lease_id", T::id()).with_identity("reader_lease"),
                publication_ref("publication_id"),
                column("holder", text()),
                column("acquired_at", ts()),
                column("expires_at", ts()),
                column("released_at", ts()).optional(),
            ],
            "Reader leases: taken in a short transaction and released when the read is done; no reader holds a database session while it reads Delta files (finding T02).",
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
            "publication_members",
            "publications",
            "reader_leases",
            "retention_marks",
            "settlements",
            "solutions",
            "source_bundles",
            "source_documents",
            "studies",
            "study_points",
            "workspaces",
        ] {
            assert!(tables.contains(&table), "{table} is not a store relation");
        }
        assert_eq!(tables.len(), 18);
        // Identities are owned by their keys and inherited through references.
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
        assert_eq!(
            registry.identity("source_bundle").map(|identity| identity.base),
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
        assert_eq!(values.foreign_keys[0].target, "runtime.operational_progress_events");
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
