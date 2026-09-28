---
title: Plan 22 store and typed-data execution packet
status: in-progress
date: 2026-09-28
parent: 22-solver-capabilities.md
adrs: [ADR-0114, ADR-0115, ADR-0116, ADR-0117]
review_sources:
  - ../design_review/reviews/design_review_typed-data-contracts_2026-09-28.md
---

# Plan 22 store and typed-data execution packet

This packet owns step progress, the binding execution decisions and the checkpoints for the
store and typed-data track of [Plan 22](22-solver-capabilities.md). It covers:
- D2;
- B1–B7;
- O5's missing incumbent stream, together with G8;
- O7, O8 and O9;
- the store documentation step;
- the W6 scoped qualification.

Packet definitions, sequencing and finding dispositions stay in the plan. The solver-scope
packets keep their progress in the [main execution packet](22-solver-capabilities-execution.md),
whose step E14 points here.

## Maintainer decisions (2026-09-28)

1. **Priority.** All remaining PostgreSQL and data-structuring scope in Plan 22 runs before the remaining solver scope. That scope is:
   - S1–S4, Y3–Y5, C3–C5 and N5;
   - M2 completion and M5;
   - the G4 and G6 remainders;
   - the solver docs pass.

   Q1 runs after both.
2. **Rules follow the design.** A repository rule that obstructs the better design is changed to fit it. New crates and dependencies are fine wherever they are the better design.
3. **Store contents are regenerable.** There is no data migration; a schema change resets the store (ADR-0114).
4. **Remote object stores are not a priority** for this locally run project. O8 qualifies the catalog protocol locally only, and a register row holds the remote trigger.
5. **Scoped qualification.** One runs at W6, before solver work resumes.
6. **Execution.** The three tracks run as parallel agents in worktrees and merge into `main` at packet boundaries.
7. **Model and case name declarations** (B3b's proposal). `model_id` names the model's specialized definition and `case_id` the case declaration, both typed `declaration`. The instance and the fit get their own columns, and the unused `model` and `case` identities go. No accepted ADR or architecture section defines these columns; the registry column documentation owns their meaning, so B3c implements the decision without a new record. Numerics identity bytes change, and cached results regenerate.
8. **Lean verification mid-track.** Merges are checked by compile checks and targeted units; use-case and journey suites wait for W6. From W5 on, single agents work in the main checkout to reuse the warm build.

## Binding execution decisions

| # | Decision | Source |
|---|---|---|
| X1 | **Platform vocabulary crate.** A new crate `pse-vocabulary` holds the ten `pse-schema` platform enums (Namespace, Authority, SnapshotClass, DerivationGranularity, Stability, ColumnRole, InvariantKind, Severity, Determinism, OperationEffect). `pse-schema` and `pse-model` both depend on it, and the generator re-exports these enums instead of copying them. The crate joins the `semantic` dependency-ceiling roots. The generator ceiling stays: it prevents a real bootstrap failure. Written as **ADR-0117**, which adds a crate; a short change-tier review appended to the typed-data review | maintainer 2026-09-28 |
| X2 | **Entity identity is a registry declaration kind.** `declare_identity` defines an identity; a column references it with `with_identity` (facet `pse.domain.identity`); a foreign key inherits it; each identity has at most one owner key. Typed ids are generated into `pse-model` through the macro moved to `pse-ids`. Serde and framing are unchanged, so no identity bytes move. Unowned identities are allowed (`run`, block, member). `source_bundle` wraps `ContentHash`: a recorded deviation from ADR-0115's "over SemanticId" wording | B1 design |
| X3 | **Logical types and facets.** JSON-document columns are the logical type `json` (jsonb). Operational timestamps are `ts_us`, microseconds. Unique keys and composite FKs are registry declarations. There is no `ON DELETE CASCADE`: deletes are explicit. A `finite` facet on float columns is rendered per target (the DataFusion validator and PostgreSQL CHECK), so the NaN and infinity guards are kept, not dropped | B1 design, refined |
| X4 | **Typed terminations.** `TerminationClass` (native, run_state, trajectory, runtime, rule) plus typed per-class columns, with a generated one-of rule. `RuntimeTermination` is a new registry enum. `rule` becomes the `DiagnosticCode` ENUM once B4 aliases it | B1 design |
| X5 | **Store switch and fingerprint.** B1 moves the store onto the generated DDL, adding interim casts to the sqlx statements that B2 deletes. The fingerprint is recorded as `COMMENT ON SCHEMA pse_ops` and is also a generated constant. `Store::open` creates the store or refuses it; `db-reset` is confirmed and destructive; `db-migrate` is deleted | B1 design |
| X6 | **Cornucopia integration.** Cornucopia 1.0.1 runs as an **xtask library dependency** under the workspace lockfile, never a `$PATH` CLI. `gen_fresh` loads the freshly generated DDL and `physical.sql`. xtask renders the crate manifest itself (workspace inheritance, `[lints] workspace = true`) and a `lib.rs` with a reasoned crate-level `allow` for the generated code, and re-prints every file with prettyplease for byte stability. `--check` exits non-zero when no server is reachable | B2 design |
| X7 | **Whole rows via composites.** Statements return table composites, and `types.mapping` maps `pse_ops.<table>` to the registry row, decoded by a generated `FromSql`. Projections carry hand-written nullability. Fallback if the B2.0 probe fails: a generated `rows.sql` of `--:` annotations, plus id wrapping at the repository boundary | B2 design |
| X8 | **One listener task per `Store`.** It runs on a dedicated connection, reconnects with backoff, re-issues `LISTEN`, and broadcasts `Resync`. Watchers re-read the authority on a notification, on `Resync` and on lag. It replaces `PgListener` | B2 design |
| X9 | **Publication identity and intents.** The publication attempt **is** the run's durable attempt id. A `publication_intents` table is registered before the first member write. It makes unpublished members reclaimable, and makes `ProvedNoncommit` provable before any effect | O8 design |
| X10 | **Catalog-owned retention.** `publication_windows` gives the catalog the change-data (`changes`) reason (T16). `workspaces.maintenance_epoch` plus a pse-catalog `ReadScope` replace the file-lease `Generation`, both as a cache key and as reader protection. `.pse-retention.lock` is deleted. `RetentionReason` loses `output`; `attempt` means a live intent's prefix | O8 design |
| X11 | **One member type.** A registry **named structure** `MemberDescriptor`, emitted once, replaces the control relation's member types in all 33 files. Member receipts go to v4, and v3 is refused as MigrationRequired | O8 design |
| X12 | **Export manifest.** A one-row Delta relation `runtime.publication_manifests` replaces `runtime.publications`. The export is a reader lease with a TTL, and it opens offline | O8 design |
| X13 | **Python settings are generated documents.** `SolveSettings`, `BackendSettings`, `DiffsolSettings` and `IdasSettings` become generated msgspec document types (ADR-0116 Outcome 7), not pyo3 keyword classes, so the enum types are visible to Python type checkers without pyo3 introspection workarounds | B4/B5 design |

**Rule changes included in the track**, following the standing direction:
- the `semantic` dependency-ceiling roots gain `pse-vocabulary`;
- AGENTS.md prime directive 2, `scripts/agent-hooks.py`, the `.claude/settings.json` deny rules, `.claude/rules/generated.md` and `tests/governance` `is_generated` gain `crates/pse-operations/src/generated/`, `crates/pse-operations-queries/` and `python/pse/_native.pyi` (TD10);
- REUSE, typos and taplo configuration cover the generated TOML;
- the governance regeneration test keeps covering the pure languages, while the query crate's regeneration check runs in `codegen-check` with a server.

## Tracks, waves and path ownership

The coordinator:
- reviews each merge;
- runs `just codegen` after every merge that touches the registry (generated conflicts are resolved by regeneration, never by hand);
- re-runs the packet's targeted tests on `main`;
- records the checkpoint here.

Environment:
- `PSE_SOLVER_IMAGE` is set for native tests (the local `pse-solvers:dev-local` image, see the main packet);
- PostgreSQL 18 runs locally for store work and query generation.

| Track | Packets | Owns while active |
|---|---|---|
| R — registry and store | B1, B2, O8, O7, O9, docs step | `crates/pse-schema`, `crates/pse-codegen`, `xtask/src/codegen*`, `crates/pse-ids` (except `derive.rs`), `crates/pse-model` and `crates/pse-relations` generated trees (coordinated), `crates/pse-operations*`, `crates/pse-catalog`, `crates/pse-runtime/src/workflow/{durable,worker,publication,run}.rs`, `crates/pse-py/src/workflow*`, `python/pse/_runs.py`, `_workflow.py`, `_inspection.py`, store recipes, doctor, protected-path lists |
| V — vocabularies and documents | B4, B5, then G8 with O5's incumbents | `crates/pse-vocabulary` (new), `crates/pse-diagnostics`, `crates/pse-backend-native` settings and events, `crates/pse-modeling/src/annotation.rs`, `crates/pse-math/src/numerics.rs`, `crates/pse-py/src/workflow/settings.rs`, `python/pse/contracts/documents/`, `python/pse/governance.py`; registry catalog modules for settings vocabularies (coordinated with R) |
| T — typing in math and modeling | B3a, B6, B7, B3b | `crates/pse-ids/src/derive.rs` and the frame call sites, `crates/pse-math` (except `numerics.rs`), `crates/pse-backend-native` presolve/transport/conditioning/HiGHS diagnostics, `crates/pse-modeling`, `crates/pse-compiler` |

| Wave | Track R — registry and store | Track V — vocabularies and documents | Track T — typing in math and modeling |
|---|---|---|---|
| W0 | D2: ADR-0117, the ceiling change, the new packet with X1–X13 | — | — |
| W1 | **B1** | — | **B3a** frame catalog → **B6** index spaces |
| W2 | **B2** | **B4** (after B1) | **B7** (after B1) |
| W3 | **O8** | **B5** (after B4) | — |
| W4 | **O7** (after O8 and B5) | **G8 + O5-incumbents** (after B2 and B5; merges after O7, which shares `worker.rs`) | **B3b** typed-id sweep (after O8) |
| W5 | **O9**, then the **docs step** | — | — |
| W6 | **Scoped qualification** (maintainer request, 2026-09-28), then solver scope resumes | — | — |

## Packet detail

Each sub-step compiles (`just check-package` / `just check`), runs its targeted tests
(`just unit-package <pkg> <filter>`; `just db-test`; native tests with `PSE_SOLVER_IMAGE`
set), and deletes what it replaces in the same change.

### D2 — decision (W0)
- ADR-0117 (`just adr-new`, then accepted) and a change-tier review note. Covers X1 and the ceiling change.
- `just adr-lint`.

### B1 — the registry owns the store's physical schema (W1)

| Step | Work | Tests |
|---|---|---|
| B1.0 | Packet-start probes: the registry check SQL parses in both dialects; multi-statement DDL runs in one transaction; the B2.0 Cornucopia probes (they decide X7) | — |
| B1.1 | Move `semantic_id_newtype!` from `pse-quantity/src/ids.rs` to `pse-ids/src/newtype.rs`, adding serde delegation and `const from_bytes`/`as_bytes`; `pse-quantity` uses it | `just unit-package pse-ids 'test(newtype)'` |
| B1.2 | `pse-schema`: identity and document facets in `model/field.rs`; `model/identity.rs`; `unique_keys` and `foreign_keys` in `model/relation.rs`; `declare_identity` and `resolve_identities` in `builder.rs`; integrity in `builder/integrity.rs`; `ts_us`; the `finite` facet; Arrow projection; self-description (`reference.schema_identities`, `schema_columns.identity_id`) | `identity_inherited_through_foreign_keys`, `conflicting_identity_refused`, `identity_owned_once`, `json_document_is_a_logical_type`, `unique_and_composite_keys_described` |
| B1.3 | Codegen: `rust/identities.rs` (typed ids into `pse-model`, `ArrowValue` into `pse-relations`); relation columns typed by identity; microsecond codecs; Python `NewType` ids | `identity_columns_render_typed_ids`, `microsecond_timestamps_render_codecs` |
| B1.4 | `Language::Postgres` → `crates/pse-operations/src/generated/` (`schema.sql`, `cornucopia.toml` mapping, `mod.rs`, `fingerprint.rs`); `rust/postgres.rs` generates `ToSql`/`FromSql` for store enums and ids behind a `postgres` feature on `pse-ids`/`pse-model`; pins `postgres-types =0.2.14` and `postgres-protocol`; xtask `Target::Postgres`; `codegen-postgres-check` | `postgres_value_mapping_round_trips`, `ddl_covers_store_relations_enums_identities`; `just family-check` |
| B1.5a | `catalog/operations.rs`: `ts_us`, `json`, identities, the seven catalog relations (`runtime.operational_*`), enums `RetentionPhase`, `SettlementOutcome` and X4's, keys and FKs, every CHECK ported and named, no `uuidv7()` defaults; typed termination in `durable.rs`; `just codegen` | `catalog_relations_registered`, `operational_timestamps_are_microseconds` |
| B1.5b | `physical.sql`; `src/schema.rs` (`open`, `schema_status`, `reset`, `SchemaMismatch`); interim sqlx casts; `pse-ops status\|reset`; justfile `db-reset` (confirmed) replacing `db-migrate`; doctor fingerprint check; `docs/dev/operational-store.md` | `generated_schema_creates_empty_store`, `store_schema_mismatch_refused`, `registry_enums_are_postgres_enums`, `misspelled_enum_literal_fails_prepare`, `row_invariants_generated_and_enforced`, the existing store tests; `just worker-test`; one `just db-reset` of the dev store |
| B1.6 | Protected paths (TD10) and the configuration listed under rule changes | `generated_path_edit_refused` (`just setup-test`), `just lint-agents` |

**Deletes:**
- both migrations, `MIGRATOR`, and the migrate API and error;
- `conformance_tests.rs` with its `CATALOG` exemption, and the pse-schema and arrow-schema dev-dependencies;
- `migrations_apply_to_empty_database`, `pse-ops migrate`, `db-migrate`, and the doctor's `_sqlx_migrations` code;
- the old macro, and the string termination codes.

### B2 — typed statements on tokio-postgres (W2)

| Step | Work | Tests |
|---|---|---|
| B2.0 | Scratch-crate probes: domain parameters and whole-row composites through Cornucopia; `params-only`; the `gen_fresh` URL; the lint surface of the generated code; `BinaryCopyInWriter` types; the ring `CryptoProvider`; `max_connections` | — |
| B2.1 | Pins: `tokio-postgres =0.7.18`, `deadpool-postgres =0.14.2`, `tokio-postgres-rustls =0.14.0`, `cornucopia =1.0.1` (xtask), `futures`. `Target::Queries` in `xtask/src/codegen/queries.rs` with X6 normalization. Generated crate `crates/pse-operations-queries/` registered. `queries/store.sql` | `store_queries_regenerate_identically`, `generated_query_rejects_misspelled_column_and_literal`, `normalized_manifest_passes_registration_rules` |
| B2.2 | Generated composite `FromSql` for store rows, with helpers in `pse-model/src/postgres.rs` | `postgres_value_mapping_round_trips` (extended) |
| B2.3 | `Store` on deadpool; `error.rs` classifies by `SqlState` constant (23514/23503 → `InvariantViolation`) and wraps driver errors; `testing.rs` rebuilt on tokio-postgres; store tests move to `#[tokio::test]` + `TestDatabase`; a nextest `store` test group | `sqlstate_classified_by_constant` |
| B2.4 | Repositories moved one module at a time — attempts → jobs → cancellation (with the X8 listener) → streams (binary COPY) → solutions → sources → catalog — with their consumers in `pse-runtime` and `pse-py` updated in the same step | All store, durable and worker tests, including `two_workers_never_claim_same_job`, `expired_lease_requeues_as_new_attempt`, `cancel_survives_listener_reconnect`, `listener_resyncs_after_connection_loss`, `progress_stream_complete_under_volume`, `stored_seed_reused_across_processes` |
| B2.5 | Remove sqlx entirely | `cargo tree -i sqlx` is empty; `just check`; `just family-check`; `just py-sync-native` plus the Python run units |

**Deletes:**
- the sqlx and `whoami` pins; `codec.rs`; all eight `FromRow` impls and the records that restate a row;
- the column macros, positional binds and `UNNEST` statements;
- string SQLSTATE matching, `#[sqlx::test]`, `PgListener`, and `Store::pool`/`from_pool`;
- the justfile `DATABASE_URL` export and the `_sqlx_test` backup exclusion;
- the ADR-0112 citations in the rewritten modules.

### B3a — frame catalog (W1, Track T)
- A `pse_ids::Frame` enum declares every production frame spelling (104 at B3a; 107 after `pse.ops.schema.v1` and the two durable-study frames). `FramedHasher::new`, `derive_id`, `derive_hash` and `pse-backend-native` `identity::of` take a `Frame`.
- Parameterized contexts (`tears.rs:23`, `fingerprint.rs:277`) become variants that render the identical string.
- The generated `facts.rs` frames come from the generator.
- **Tests:**
  - `frame_spellings_unique`;
  - `frame_spellings_unchanged`, which compares against a list captured at packet start;
  - the golden vectors, unchanged.

### B6 — typed index spaces (W1, Track T)
- **New dependencies:** `typed-index-collections =3.5.0`, `enum-map =3.1.0`.
- **Index types:** a `pse_math::index` module with newtypes for original and presolved rows and columns, instance-local slots, global columns, and typed `Triplet<R, C>`.
- **Boundaries converted:**
  - the presolve `Report` and pipeline (`presolve.rs:102-108`, `pipeline.rs:489-498, 748`);
  - `conditioning.rs:84` (KKT);
  - `pse-math` `sparse.rs:24`, `diagnostics.rs:49-50`, `assembly.rs:55-58, 262-270, 700-741`;
  - fitting `sparse.rs:22-23`.
- **Ranging key:** typed in `highs/diagnostics.rs:290, 462-467` and `transport.rs:529-531`.
- **enum-map:** for demand groups and `execution.rs:684`.
- **FFI and faer** convert at the adapters.
- **Tests:**
  - `compile_fail` doctests for cross-space indexing;
  - `ranging_unscaling_uses_typed_keys`;
  - the existing presolve, conditioning, assembly and HiGHS-diagnostics units (`just unit-package pse-math`, backend native units).

### B4 — vocabularies (W2, Track V)
1. **Create `pse-vocabulary`** (X1). Generalize the generator's alias rule (`rust/enums.rs:12-15`) from a name match against pse-quantity to a declared enum source (`EnumDecl::sourced`). This covers pse-quantity, pse-diagnostics (`DiagnosticCode`, `FailureClass`) and pse-vocabulary. Give `pse-diagnostics` and `pse-vocabulary` an optional `postgres` feature for any store column that uses them, and type X4's `rule` column.
2. **Registry enums for every settings enum carried by a backend or dynamics settings document.** The inventory is taken at packet start; it includes `IpoptLinearSolver`, `MumpsOrdering`, `SpralOrdering`, Spral scaling and pivot, Pardiso ordering and matching, `MuStrategy`, `HessianMode`, `ReusePolicy` and the HiGHS, POUNCE and KINSOL methods not yet in the registry. Behaviour such as `parallel()`, `mask()` and the Ipopt integer codes moves into adapter functions in `pse-backend-native`.
3. **Typed policy and provenance.** Add `ExtrapolationPolicy`, so `AnnotationValue::Valid { policy: ExtrapolationPolicy }` and its seven string comparisons go. Add `NumericalProvenanceField`, replacing `&'static str` in `pse-model` `numerics.rs:144` and the reads in `pse-math` `numerics.rs:110, 418, 455`.
4. **Tests:** `extrapolation_policy_typed`, `source_owned_vocabularies_have_one_rust_type`, `provenance_field_typed`, and the existing settings projection tests.

**Deletes:** the hand-written serde enums, the ten duplicate generated enums (twelve with the two pse-diagnostics ones), and the string comparisons.

### B5 — typed boundary documents (W3, Track V)

| Step | Work | Tests |
|---|---|---|
| B5.0 | Probes: nutype `derive_unchecked(schemars::JsonSchema)` with schemars 1.x; datamodel-code-generator determinism (msgspec output, `--disable-timestamp`); a schemars representation for the POUNCE `FeralConfig` projection | — |
| B5.1 | Pin `schemars =1.2.2`. `JsonSchema` on every backend and dynamics settings type, the solve controls, the job payload, `TerminationDetail` and `SourceManifest`, with the generator adding the derive to registry enums. Versioned envelopes | `backend_settings_schema_generated` |
| B5.2 | xtask `Target::Schemas`: JSON Schemas into `docs/generated/schema/`; Python msgspec document types into `python/pse/contracts/documents/`; `pse.governance` gains a msgspec branch | `test_documents_pass_any_lint` (py), `codegen-check` |
| B5.3 | X13: the Python settings classes become the generated document types; native entry points decode the envelope; the pyo3 `str` and `**fields` settings parameters are deleted; stubs regenerated | `test_backend_settings_typed`, `test_solve_settings_enum_types`, `test_solve_settings_backend_projection` |
| B5.4 | Validated scalar types (nutype, or `serde(try_from)` newtypes) for single-value setting domains; `admit_settings` keeps the cross-field rules | `invalid_tolerance_refused_at_decode` |
| B5.5 | Job payload v2, typed: `enqueue` takes the typed payload; the request identity is framed through `identity::of`; a `start` policy field (used by G8); v1 deleted. Typed `TerminationDetail` and `SourceManifest` | `job_request_identity_independent_of_key_order`, `termination_detail_versioned_and_typed`, `unknown_payload_version_refused` |
| B5.6 | Authoring JSON Schema through schemars on the generated document structs; delete `pse-codegen` `codegen/jsonschema.rs` if the output is at least as precise, otherwise record why it stays | `authoring_schema_equivalent_under_schemars` |

### B7 — modeling and compiler typed ids (W2, Track T)
- **Declarations:** package, case, model, declaration, root and instance on the registry columns that key them (`package_id` alone appears in 26 columns); definition, member and block as unowned identities.
- **Consumers:** the 31 `pse-modeling`, 6 `pse-compiler` and runtime-modeling multi-id signatures.
- **Tests:** a `compile_fail` doctest for a root/instance swap, plus the existing compiler, modeling and conformance units.
- **Expected effect:** authored-relation fingerprints change, so cached artifacts are invalidated; they are regenerable.

### O8 — the publication catalog replaces the Delta control relation (W3)

| Step | Work | Key tests |
|---|---|---|
| O8.1 | Registry named structures (X11): `MemberDescriptor` emitted once; an ast-grep migration of the 33 files; the conversion code at `delta/publication.rs:363-383`, `artifact/descriptor.rs:129-185` and `attempt.rs:409` deleted | `structure_name_is_presentation_only`, `named_structure_conflict_rejected`, the existing admission and selection units |
| O8.2 | Registry: member descriptor flattened into `operational_publication_members` with `PublicationMemberRole`; publication `kind`; unique `root_uri` and `maintenance_epoch`; the `conflict` settlement outcome; `publication_intents`; `publication_windows`; `RetentionReason` without `output`; `runtime.publication_manifests`. Then `just codegen` and `just db-reset` | the B1 registry tests, extended |
| O8.3 | Catalog statements as Cornucopia queries. Commit uses one lock order and compares the complete request (`IdentityReused`); intents; settlement (committed, proved_noncommit, conflict); reader-lease renewal and lapse; retention and maintenance under the advisory lock and the epoch; `testing::FaultProxy`, which drops the connection around `COMMIT` | `concurrent_head_advance_one_winner`, `commit_is_idempotent_per_attempt_and_settles`, `catalog_protects_published_versions`, `commit_refuses_retiring_inputs_and_retained_members`, `settle_distinguishes_committed_noncommit_conflict`, `reader_lease_renews_and_lapses`, `maintenance_bumps_epoch_before_effects`, `reclaimable_intents_are_fenced` |
| O8.4 | pse-catalog member-only API: `ReadScope`, `PublicationSelection`, `AdmitCandidate`, manifest write and open, `collect`/`remove_tables`/`remove_prefix` | `candidate_returns_admitted_record_with_actual_versions`, `manifest_round_trip`, `manifest_refuses_legacy_control_table`, `collect_keeps_protected_versions_on_memory_store`, `read_scope_is_a_cache_lookup_input` |
| O8.5 | **Cut-over.** Receipts v4; runtime publication on the durable attempt id; workspaces, open/open_head through `ReaderLeaseGuard`, export; maintenance orchestration and a `pse-publication` binary; pse-py and Python (`_runs.py`, `_workflow.py`, `_inspection.py`) | `concurrent_publishers_one_winner_no_lost_update` (two processes), `lost_ack_settles_via_catalog`, `maintenance_waits_for_reader_leases` (`memory://`), `catalog_protects_published_versions`, `exported_publication_opens_offline`, `publication_uses_durable_attempt_identity`, `legacy_workspace_root_refused`; port `authored_publication_resource` and the three Python publication tests |
| O8.6 | **Delete:** `publish.rs`; ticket `settle`/`observe`; `PublicationRoot`/`read_control`; `lease.rs`; the `leased.rs` leases; `retention.rs`; the control logic in `maintenance.rs`; `PublicationTarget`; the `runtime.publications` declaration and its fixture; the Python `PublicationRoot` and `pse.open(location, version)`; and the control-mechanism tests named in the O8 design. The surviving invariants are rewritten on the catalog path (`publication_each_object.rs`, `native_publication.rs`, `unified_sources.rs`, the xtask inspection fixture) | `rg "pse-retention\|runtime\.publications\|PublicationRoot\|lease::"` finds nothing |
| O8.7 | Local crash-safety of maintenance: FaultStore is extended with delete and list, and an interrupted deletion leaves the publication expiring until a rerun completes it | `interrupted_deletion_resumes`; the O8.5 lost-acknowledgement test |
| O8.8 | The architecture companion's store sections (§9.3, §9.5, §9.9); dispositions T02 and T16 resolved. Register rows: the automatic retention policy, and **remote object-store qualification** (S3-compatible), triggered by "a real need to publish to a remote object store". The maintainer decided on 2026-09-28 that remote stores are not a priority for this locally run project | — |

### O7 — studies across workers (W4)
- **Store.** A studies repository (`queries/studies.sql`): study, points and point transitions in the same transaction as the job and attempt. A point with a predecessor becomes claimable only once the predecessor has completed.
- **Payload.** Payload v3 adds a typed `StudyPointBinding`: `study_id`, `point_index`, `binding_hash`, an overlay document and the predecessor.
- **Publication.** The study registers **one publication intent** when it is created (X9).
  - Each point attempt writes its result members under that intent's prefix.
  - When every point is terminal, a finalization job commits one publication: the study summary relation plus every completed point's members.
  - A failed point is isolated.
  - A worker seeds from the predecessor's stored solution (`StartSource::Stored`).
- **Runtime and Python.** A durable `Runtime::start_study` returning a `StudyHandle`; `ModelingPackage.study(..., runtime=…)` for durable execution, alongside the ephemeral in-process path of the `Ephemeral` class; a `Runtime.studies()` listing.
- **Tests:** `study_parallel_workers_publish_once` (two worker processes), `failed_point_does_not_contaminate`, `predecessor_waits_and_seeds`, `study_cancel_stops_pending_points`.

### G8 + O5 incumbents — durable long solves (W4)
1. **Typed incumbent events.** A typed `IncumbentEvent` on the native `Event`: objective in original units with the export offset applied, dual bound, gap, nodes, and a throttled primal in original coordinates. SCIP's `watch_exec` (`scip.rs:359-389`) fetches the best solution; HiGHS supplies it from callback kinds 3 and 4.
2. **Durable sink.** The `Streamer` in `durable.rs` writes a `solutions` row (NLP primal, with the step's compatibility stamp) and an `incumbents` row, using binary COPY and a throttle.
3. **Resume.** `ClaimedJob` gains `parent_attempt`. `prepare_job` applies the payload `start` policy `ResumeFromParent`: the latest incumbent in the parent chain, passed through `with_stored_start`, then SCIP `inject` or a HiGHS sparse start.
4. **Tests** (solver image required):
   - `killed_worker_attempt_goes_stale_and_resumes_from_incumbent` (spawns pse-worker and kills it once an incumbent is stored);
   - `incumbent_stream_records_offset_objective`;
   - `highs_incumbents_streamed`;
   - `cross_process_cancel_stops_scip`.

### B3b — typed-id sweep (W4, Track T)
Converts the remaining workflow, runtime, pse-catalog (receipts) and pse-py signatures to typed ids, plus Python `attempt_id`/`publication_id` `NewType`s. Tested with `compile_fail` doctests.

### B3c — remaining typed-id sweep (W5)
B3b stayed out of the runtime files O7 owned. B3c types what is left:
- `run_id` on its registry columns (31) and through the runtime signatures (about 25), after checking the `run_lineage` wording that run identity "names the attempt";
- `WorkflowError::{EphemeralPublication.run_id, ExportLeaseExpired.publication}`;
- the O7 and O9 runtime entry points (`start_study`, `progress`, the study handles) and their Python counterparts (`AttemptId`, a `StudyId`, `case_id` as `DeclarationId` in `_modeling.py`);
- explicit identity owners in the registry, so a keyed projection such as the export manifest can declare its identity without becoming its owner;
- the no-op `.into()` calls B3b left in `publication.rs` and `reading.rs`, and the unused `run_id` parameter in `workflow/modeling/dynamics.rs`.

The model and case semantics follow the maintainer's decision on B3b's proposal (see the B3b checkpoint).

### O9 — query surface (W5)
- **Provider.** A read-only `TableProvider` per operational relation, in `pse-runtime` (the composition root). It runs generated statements with pushed-down typed filters (attempt, run, state, time range; `Inexact`) and builds batches with the `pse-relations` builders. It is bound through `EngineFactory` `with_provider` under `pse_ops`.
- **ADBC is not adopted:** it adds a C driver manager for tables the generated builders already serve. A short decision note is recorded.
- **Python:** `Runtime.jobs()`, `Runtime.studies()`, and `Runtime.progress(attempt, follow=True)` over the listener.
- **Tests:** `operational_tables_join_results_in_datafusion`, `provider_pushes_attempt_filter`, `test_progress_stream_python`.

### Docs step (W5)
- A `design:` revision (`PSE_DESIGN_EDIT=1`) moving the markers from "target" to "implemented": §0.6, §3.2, §3.3, §4.1, §4.2, §5.1, §5.3, §20.x, §21.x, §23.2, and D10's store.
- The architecture companion's "As implemented" notes for §9 and §12.
- `docs/dev/operational-store.md` rewritten.
- Plan 22 dispositions: TD01–TD11, T02 and T16 resolved with their tests.
- The owed O3–O6 architecture text is folded into this revision.

## Risks and packet-start checks
- **Cornucopia domains.** Domain typing through Cornucopia (B2.0), with X7's fallback.
- **Lint and build surface of generated code.** Cornucopia's output fails the workspace lints and `--no-default-features`. Handled by X6 normalization; clippy runs at Q1.
- **Dependency growth.** Lock growth and feature unification from Cornucopia, checked with `just family-check` and `cargo tree -d`.
- **Registry fingerprints.** A `FINGERPRINT` outside `runtime/operational_*`, `reference/schema_*` and the authored relations changing unexpectedly after `just codegen` is a signal to stop.
- **Store resets.** Each schema change needs `just db-reset`, and long-lived pools need a restart after one. Python `OperationalStore` tests fail until the dev store is reset.
- **O8 risks.** Settlement semantics change (`ProvedNoncommit` before any effect). Re-prepare reuse needs a stable `operation_id`. Lease lapse. Lock ordering between commit and maintenance. Editable DML tables lose cross-process caching (measure at Q1). `prepare_checkpoint` has no production caller, so re-home or delete it in O8.4.
- **Test infrastructure.** New pieces: FaultProxy, a two-process publisher test, and pse-worker kill tests.
- **Python API break.** No shim, by rule (O8.5, B5.3).

## Verification
- **Per packet:** compile checks plus the named targeted tests; `just codegen` / `codegen-check` wherever the registry or generators change (with the local PostgreSQL 18 running); `just db-test`; `just worker-test` and the native units with `PSE_SOLVER_IMAGE` set; `just py-sync-native` plus targeted `just py-test` when the Python surface changes; `just adr-lint` for D2; `just setup-test` and `just lint-agents` for B1.6; `just family-check` when a pinned dependency moves.
- **Exit checks:**
  - no `sqlx` in `cargo tree`;
  - no `FromRow`, migrations, `.pse-retention.lock` or `runtime.publications`;
  - no `FramedHasher::new("`;
  - no `**fields: object` in `_native.pyi`;
  - every B, O7–O9 and G8 test in the Plan 22 table passing, each reported with its command against the zero baseline.
- **W6 scoped qualification.** The maintainer requested it, so it runs as an explicit step once W5 lands. Checks over the crates this track touched (pse-ids, pse-vocabulary, pse-schema, pse-codegen, pse-model, pse-relations, pse-operations, pse-operations-queries, pse-catalog, pse-runtime, pse-math, pse-backend-native, pse-modeling, pse-compiler, pse-py, xtask) and `python/pse`:
  - formatting: `cargo fmt` for those crates, ruff format;
  - lint: `just clippy`, `just quality`;
  - generation and governance: `just codegen-check`, `just governance`, `just family-check`, `just adr-lint`;
  - store and native: `just db-test`, `just worker-test`, the native-acceptance conformance suite (including `authored_publication_resource`, currently unverified), the runtime native units;
  - Python: `just py-test` component and integration scopes for runs, publication, settings and studies;
  - docs: `just docs`.

  Each result is reported in the new packet with its command, conditions and failure count against the zero baseline. The full Q1 still runs after the solver scope.
- **Remote object stores** are out of this track; the register row above holds the trigger.

## Progress

| Step | Packets | State |
|---|---|---|
| W0 | D2: ADR-0117 accepted, with the review addendum; this packet written | complete (2026-09-28) |
| W1 | B1 (R); B3a → B6 (T) | complete: B3a and B6 in merge `8934a521`, B1 in merge `fa5a075c` |
| W2 | B2 (R); B4 (V); B7 (T) | complete: B4 in merge `ba5f9676`, B7 in merge `057ade3b`, B2 in merge `c664106e` |
| W3 | O8 (R); B5 (V) | complete: B5 in merge `d26bdebe`, O8 in merge `4487adfc` |
| W4 | O7 (R); G8 + O5 incumbents (V); B3b (T) | complete: G8 in merge `b80f8d4c`, O7 in merge `f5909d1c`, B3b in merge `3c6407ad` |
| W5 | O9; B3c; docs step | complete: O9 in merge `3a097532`, B3c in `4b9d16f7`/`3d878717`, docs step in `12321374`/`425a7b9c` (revision 67) and `0e727af5` (revision 68) |
| W6 | Scoped qualification | complete: see the W6 checkpoint; the run-identity fix `598b82ef` has not been run |

## Current checkpoint (2026-09-28)

W0 is complete (`48fd7f33`): ADR-0117 is accepted, and `just adr-lint` passes. W1 is running,
with B1 on track R and B3a then B6 on track T. Each track works in its own worktree; the
coordinator merges it into `main` after review and re-runs its targeted tests.

### W1, track T: B3a and B6 landed (merge `8934a521`)

**B3a** (`6d2f0c1e`). `pse_ids::Frame` declares all 104 production `derive_key` contexts in
one table, with spellings unchanged against the oracle captured beforehand
(`crates/pse-ids/tests/frame_spellings.txt`) and the golden vectors unchanged.
- `FramedHasher::new`, `derive_id`, `derive_hash`, the keyed `preimage` entry points and backend-native `identity::of` take a `Frame`.
- About 106 call sites moved.
- Deleted: `derive::context` and five test-only spellings.

**B6** (`db403e6c`). `pse_math::index` types now cover every boundary listed for the packet: original, presolved and reduced rows and columns; slots; global rows and columns; addends; typed entries and triplets.
- Presolve `Report` and pipeline: separate typed `jacobian` and `hessian` builders; the transformation hash is unchanged.
- The KKT assembly.
- The assembly-matrix entries and refill map.
- Assembly instances and rows.
- The diagnostics' parallel pairs.
- The fitting `Mapping`.
- HiGHS ranging: a typed `RangeFamily` in an `EnumMap`, with published spellings and order unchanged; the `"row_"`/`"column_cost_"` prefix matching is deleted.
- enum-map for the demand groups and compiled programs (`DerivativeOrder` derives `Enum` in `pse-kernels`).
- New pins: `typed-index-collections =3.5.0`, `enum-map =3.1.0`.

**Tests on `main` after the merge** (coordinator run, zero baseline):
- `just unit-package pse-math 'package(pse-math)'`: 75 passed, 0 failed.
- `cargo nextest run -p pse-ids -p pse-relations --lib --test golden_vectors --features pse-relations/force-validate -E 'package(pse-ids)'`: 41 passed, 0 failed.
- `just unit-native-package pse-backend-native pse-backend-native/native-solvers 'package(pse-backend-native)'` with `PSE_SOLVER_IMAGE` set: 176 passed, 0 failed.
- `just codegen-rust-contracts-check`: OK.

The agent's worktree runs covered the rest:
- `cargo test --doc` for `pse-math` (the `compile_fail` cross-space doctests, 5 passed) and `pse-backend-native` conditioning (2 passed);
- `pse-compiler` 96 passed; runtime fitting, diagnostics and native-analysis units 33 passed; runtime dynamics units 12 passed.

**Follow-ups:**
- The plan and review text say "119 spellings". The real catalog is 104 frames; the other literals are metadata and preimage version strings. The plan text and TD08 are corrected; the review and ADR-0115 keep their original wording.
- Listing the frame catalog in the generated docs (ADR-0115 Outcome 4) touches the generator track R owns. It moves to B3b.
- Still untyped: the dynamics oracle's `(row, coordinate)` support and the fitting oracle's coordinate and constraint tuples. They were outside B6's listed boundaries; B3b sweeps them.
- The Python ranging test (`test_modeling_kernel`) runs after `just py-sync-native`, once B1 lands.

### W1, track R: B1 landed (merge `fa5a075c`)

**Commits:** `2dd572dd` (B1.1), `9e6db304` (B1.2), `81b3b7a3` (B1.3, B1.4), `a20ee399` (B1.5), `1333fa81` (B1.6), `6fdfe17a` (fixtures).

**What landed:**
- **Identity macro.** It moved to `pse-ids` with serde delegation.
- **New registry declarations:** entity identities, inherited through foreign keys, with at most one owner; JSON documents; `ts_us` timestamps; unique keys; composite foreign keys; `reference.schema_identities`.
- **Store tables.** All 18 are in the registry, including the seven catalog tables. It also gains the four new enums, typed terminations per X4, and every former CHECK as a named row check.
- **`postgres` generator.** Its output is the `pse_ops` DDL, the fingerprint, the Cornucopia mapping, and the value mapping behind a `postgres` feature on `pse-ids` and `pse-model`.
- **Store lifecycle:**
  - the store is created from the schema or refused with `SchemaMismatch`;
  - `physical.sql`;
  - `pse-ops status|reset`;
  - `just db-reset` replacing `db-migrate`;
  - the doctor fingerprint check;
  - interim sqlx casts, which B2 deletes.
- **Protected paths:** TD10.
- **Merge resolution:** the fingerprint frame is `Frame::OpsSchemaV1`, registered in the spelling oracle as a deliberate post-capture addition.

**Deviations** (accepted by the coordinator):
1. **No `finite` facet** (amends X3). Registry `Float64` is already finite everywhere: the relational value check refuses non-finite values, and the Python contracts use `finite_float`. The generator therefore gives every `double precision` column a named finiteness CHECK, and array columns a CHECK on their elements. So incumbent `objective`, `dual_bound` and `gap` must now be finite; "no bound" is NULL.
2. **Former CHECKs are named row checks**, not invariants, because invariants are relational queries. Unique keys and foreign keys still generate invariants.
3. **Optional text checks** read `x IS NULL OR x <> ''`.
4. **The generated `mod.rs` exports `SCHEMA`**, so `pse-operations` keeps `pse-schema` as a dev-dependency only.
5. **Job, settlement and reader-lease ids** are minted by the runtime.
6. **`pse-ops status` exits 0** when no schema exists yet.
7. **The conformance fixture generator** now covers unique keys and composite references.
8. **Naming.** The registry follows X4 (`TerminationClass` plus typed columns); `TerminationCode` is the Rust type in `pse-operations`, although ADR-0114's text names a `TerminationCode` enum.

**B1.0 probe results:**
- **Check SQL portability.** All 39 row checks parse in the registry parser and in DataFusion 55.1, and apply in PostgreSQL 18.6.
- **Schema creation.** Multi-statement DDL runs as one transaction, and a misspelled ENUM literal fails at `PREPARE` and in partial indexes.
- **Cornucopia 1.0.1:**
  - a domain-typed result column arrives as its base type;
  - INSERT parameters into domain columns are typed as the domain, comparison parameters as the base type;
  - whole-row `SELECT a` and `RETURNING a` map through `types.mapping` to an external Rust type, with the composite fields keeping their domains. **X7 is feasible**, and a hand-written composite `FromSql` decodes NULL correctly, which Cornucopia's own struct cannot;
  - `params-only` makes `bind` private only where a params struct exists;
  - `gen_fresh` works with the peer-authenticated socket URL;
  - the generated manifest forces edition 2024 and rust-version 1.85, and defaults to sync `postgres` plus deadpool (normalized under X6);
  - a text parameter bound into an ENUM column needs an explicit cast.

**Tests on `main` after the merge** (coordinator run, zero baseline, no concurrent builds):

| Command | Result |
|---|---|
| `cargo nextest run -p pse-ids -p pse-relations --lib --test golden_vectors --features pse-relations/force-validate -E 'package(pse-ids)'` | 46 passed |
| `just unit-package pse-schema 'package(pse-schema) \| package(pse-codegen)' -p pse-codegen` | 60 passed |
| `just unit-package pse-relations 'package(pse-relations)'` | 34 passed |
| `just unit-package pse-math 'package(pse-math)'` | 75 passed |
| `just db-test` | 39 passed |
| `just unit-native-package pse-backend-native pse-backend-native/native-solvers 'package(pse-backend-native)'` | 176 passed |
| `just unit-native-package pse-runtime pse-runtime/native-solvers 'test(durable_tests) \| test(worker_tests)'` | 10 passed |
| `just worker-test` | 1 passed |
| Governance `no_shadow_structs`, `every_crate_registered`, `codegen_regeneration`, `error_taxonomy` | 9 passed |
| `just codegen-check` (relations, python, docs, postgres, bindgen) | exit 0 |
| `just family-check` | OK |
| `just db-status` | 18.6 supported; schema current |
| `just py-sync-native`, then `just py-unit-native` over `test_generated_contracts.py`, `test_any_lint.py` and the ranging test | 26 passed |
| `test_modeling_run.py`, `test_native_workflow.py`, `test_plan14_acceptance.py` | 20 passed |

All failed counts were 0; `PSE_SOLVER_IMAGE` was set for the native runs. `just check` compiles; its only warnings in touched files predate the track.

**Binding practice from W1 (shared target directory).** Cargo names a workspace crate's artifacts by its path relative to the workspace, so parallel worktrees sharing `/home/paul/pse-arrow/target` can link each other's crates. B1's first check picked up B3a's `pse-ids`.
- From W2 on, every worktree agent sets `CARGO_TARGET_DIR=<worktree>/target`.
- The coordinator re-runs each packet's tests on `main` with no concurrent builds, as it did for W1.

**Open, owned by B2:**
- the sqlx `migrate` feature;
- the ADR-0112 comment in the root `Cargo.toml`;
- the interim casts.

### W2 landed: B4 (`ba5f9676`), B7 (`057ade3b`), B2 (`c664106e`)

**B4 — vocabularies.**
- **`pse-vocabulary`** (ADR-0117) owns the ten platform vocabularies. The generator re-exports 28 source-owned vocabularies through a declared enum source (16 from pse-quantity, 2 from pse-diagnostics, 10 from pse-vocabulary), which removes twelve duplicate generated enums.
- **Store rule column.** `termination_rule` is the `DiagnosticCode` ENUM, mapped through an optional `postgres` feature on pse-diagnostics and pse-vocabulary.
- **Settings vocabularies.** 24 backend and dynamics settings enums are registry enums with unchanged serde spellings. Their native codes and option values moved into adapter functions.
- **Typed decisions.** `ExtrapolationPolicy` and `NumericalProvenanceField` replace string comparisons.
- **Deviations:**
  - `pse-vocabulary` depends on `pse-diagnostics` for its typed parse error, beyond ADR-0117's "serde and thiserror/miette only";
  - `OperationEffect`'s serde spelling now equals `as_str` (no persisted consumer);
  - the variant `PardisoMkl` is renamed `Pardisomkl`;
  - durable failures record `DiagnosticCode` and put the violated named rule in `termination_detail.rule`.
- **Identity bytes:**
  - unchanged for Ipopt, SCIP, controls and dynamics, pinned by `settings_identity_bytes_unchanged`;
  - **changed** for HiGHS, POUNCE, KINSOL and Clarabel, because the identity serializer frames Rust enum type names and those types were renamed for registry uniqueness. **Follow-up (B5):** settings identity must not depend on Rust type names.

**B7 — typed modeling identities.**
- **Declared identities:** `package` (owner `authored.packages`), `declaration` (owner `authored.modeling_declarations`), and unowned `instance`, `model` and `case`. Nested values may carry an identity.
- **Converted signatures:** pse-modeling 29 of 31 multi-id signatures and about 56 lineage parameters; pse-compiler 6 of 6; runtime modeling 12 of 13. A root/instance swap fails to compile (a `compile_fail` doctest). Identity bytes, serde and framing are unchanged.
- **Deviations:**
  - no separate `root`, `definition`, `member` or `block` identities: roots, definitions and members are declarations (`DeclarationId`) and blocks are instances (`InstanceId`), so the swap still fails to compile;
  - `source_id`/`fixture_id` in the modeling check and report tables also carry `declaration`.
- **Recorded:** lineage and solve rows derive model and case from the root instance or fit id by explicit conversion, which makes an existing conflation visible. That is for B3b or the docs step.
- **Moved to B3b:** the fit experiment ids and `alias`.

**B2 — typed statements.**
- **Query crate.** Statements are `.sql` files compiled by Cornucopia 1.0.1, used as an xtask library (`gen_fresh` against the local server), into `crates/pse-operations-queries`. The manifest and `lib.rs` are normalized by xtask; every other file is re-printed with prettyplease.
- **Row decoding.** Whole rows decode into registry rows through a generated composite `FromSql`.
- **Store runtime:**
  - deadpool and tokio-postgres with **Verified recycling**. A cancelled call can leave its statement running, and under `Fast` recycling the next caller queued behind it (`dropped_statement_does_not_block_the_pool`);
  - errors classified by `SqlState`, with 23514/23503 mapped to `InvariantViolation`;
  - one reconnecting listener task per store;
  - binary COPY for streams, source documents and publication members;
  - the store reads `PGUSER` when the URL names no user.
- **Wire change.** NOTIFY payloads and Python `attempt_id` strings are 32-character hex ids.
- **Deleted:** `codec.rs`, every `FromRow`, the row-restating records, positional binds, `UNNEST`, `PgListener`, `#[sqlx::test]` and the sqlx and `whoami` pins. `cargo tree -i sqlx` is empty.
- **Cornucopia limits** (worked around):
  - a `void` result is refused, so statements use `SELECT true AS … FROM pg_advisory_xact_lock(…)`;
  - a projection mixing a whole mapped row with another column generates uncompilable code, so catalog statements project columns instead.

**Merge resolutions:**
- **B7:** generated trees regenerated with `just codegen-bootstrap`; B4's `extrapolation_policy` helper takes a `DeclarationId`.
- **B2:**
  - the query crate had to be regenerated first with `cargo run -p xtask --no-default-features -- codegen --only queries`, because the full generator links `pse-runtime` and therefore the stale query crate. **This is the bootstrap order whenever the store schema changes;**
  - B4's typed rule assertions were re-applied to B2's rewritten store and worker tests.

**Tests on `main` after each merge** (coordinator runs, zero baseline, no concurrent builds; `PSE_SOLVER_IMAGE` set):

| After merge | Command | Result |
|---|---|---|
| B4 | `just unit-package pse-schema '… pse-codegen, pse-model, pse-vocabulary, pse-relations'` | 100 passed |
| B4 | pse-math with `extrapolation_policy_typed` | 77 passed |
| B4 | `pse-modeling \| pse-compiler` | 149 passed |
| B4 | `just db-test` | 39 passed |
| B4 | backend native units | 177 passed |
| B4 | runtime native units | 150 passed |
| B4 | `just worker-test` | 1 passed |
| B4 | pse-model `postgres` feature | 4 passed |
| B4 | Python (contracts, `Any` lint, settings projection; modeling run, native workflow, Plan 14 acceptance, modeling kernel) | 27 + 29 passed |
| B7 | schema and codegen | 63 passed |
| B7 | modeling, compiler, authoring | 161 passed |
| B7 | pse-math | 76 passed |
| B7 | `pse-modeling` doctests | passed |
| B7 | runtime native units | 150 passed |
| B7 | `just db-test` | 39 passed |
| B7 | `just worker-test` | 1 passed |
| B7 | `just unit-invariant-harness 'all()'` | 389 passed |
| B7 | `just codegen-check`, `just conformance-fixtures-check` | exit 0 |
| B7 | governance | 27 passed |
| B7 | Python, same selection | 27 + 29 passed |
| B2 | `just db-test` | 42 passed |
| B2 | runtime native units | 150 passed |
| B2 | `just worker-test` | 1 passed |
| B2 | xtask `codegen::queries` | 4 passed |
| B2 | pse-model `postgres` feature | 4 passed |
| B2 | pse-codegen | 14 passed |
| B2 | governance | 27 passed |
| B2 | `just codegen-check` (five targets) | exit 0 |
| B2 | `just family-check` | OK |
| B2 | Python, same selection | 27 + 29 passed |

All failed counts were 0. The dev store was reset once, after B4, and `just db-status` reports it current.

**Open follow-ups:**
- The settings identity serializer frames Rust type names (B5).
- The presolve `Pass` and `PolicyKind` vocabularies still cross the Python boundary as hand-written enums (B5).
- The model/case conflation (B3b or the docs step).
- Plan and ADR text: ADR-0115 lists root, definition, member and block identities; ADR-0117 limits the crate's dependencies; ADR-0114 names a `TerminationCode` enum. Record these in the docs step as deliberate deviations.

### W3, track V: B5 landed (merge `d26bdebe`)

**What landed:**
- **Settings module.** Every backend's settings type lives in an always-compiled `pse_backend_native::settings` module and is described by schemars 1.2.2. `BackendSettings` is tagged by `backend`, and every data-carrying choice by `kind`.
- **New documents:** a versioned `SolveSettings` (pse-runtime `math/settings.rs`); versioned `DiffsolSettings` and `IdasSettings`; `Controls`, with the time limit in seconds.
- **Job payload v2** carries the whole `SolveSettings` plus a `JobStart` (`Fresh` | `ResumeFromParent` | `StoredSolution`).
  - `enqueue` takes the typed job and frames the request identity from it.
  - A non-`Fresh` start is refused until G8.
  - v1, `JobProfile` and `JobPresolve` are deleted.
- **Stored documents.** `TerminationDetail` and `SourceManifest` are typed and versioned.
- **Schemas target.** `Target::Schemas` publishes seven document schemas plus the authoring schema to `docs/generated/schema/`, and generates frozen msgspec document types in `python/pse/contracts/documents/`. They come from a new closed emitter (`pse-codegen` `codegen/documents.rs`): every enumeration must be a registry vocabulary, and anything unmapped is a generation error.
  - `just codegen-schemas-check` is in the `codegen-check` group.
  - `pse.governance` lints the msgspec types.
- **Python settings (X13).** The Python settings are the generated types. The pyo3 keyword classes and `**fields` signatures are deleted, and the stubs are regenerated.
- **Validated scalars** (nutype 0.8 with `derive_unchecked` schemars): `Tolerance`, `Fraction`, `PositiveCount` and `FiniteBound`.
- **More registry vocabularies:** presolve `Pass` and `PolicyKind`, tear method, FERAL ordering and scaling.
- **Settings identity no longer depends on Rust type names.** It frames field names, serde spellings and exact float bits. Six frames are bumped: `backend.settings.v4`, `native.controls.v2`, `native.accuracy.v3`, `cone.layout.v3`, `explicit-conic.v3`, `durable.job_request.v2`. The new values are pinned by `settings_identity_is_type_name_independent`, and the frame oracle is updated.

**Deviations:**
- **Python emitter.** datamodel-code-generator was not adopted: it emits `Any`, duplicate enums, unfrozen structs and relies on external formatters. The repository's own closed emitter is used instead.
- **Authoring schema (B5.6).** It keeps its source emitter, because the schemars view describes structs after parsing and hydration, with required span and integrity columns and no `id` alias. The shared fields agree (`authoring_schema_equivalent_under_schemars`), and the reason is recorded on `jsonschema::generate`.
- **Settings JSON shapes changed:** `kind` tags, `Eta::Constant { value }`, untagged `OptionValue`, time limit in seconds.
- **Pyo3 classes that remain.** `SimulationSettings` and `ModelingFixturePolicy` take encoded documents as `bytes`.
- **Unchanged validation.** `NumericalPolicy` budgets keep `validate()`.
- **Version fields** are required.
- **ADR-0116 text** says the settings identity is unchanged and names `JobProfile`. Both are recorded for the docs step.

**Tests on `main` after the merge** (coordinator run, zero baseline, `PSE_SOLVER_IMAGE` set):

| Command | Result |
|---|---|
| Backend native units | 179 passed |
| Runtime native units | 154 passed |
| pse-ids golden and frames | 46 passed |
| schema, codegen, model, vocabulary | 76 passed |
| `cargo nextest run -p xtask -E 'test(codegen)'` | 24 passed |
| `just governance-tests` | 87 passed |
| `just db-test` | 42 passed |
| `just worker-test` | 1 passed |
| `just codegen-check` (seven targets) | exit 0 |
| `just family-check` | OK |
| `just py-unit-native` over the `Any` lint, contracts, settings projection and stub surface | 34 passed |
| Native workflow, modeling run, strategies, kernel and Plan 14 acceptance (pytest) | 32 passed |

The store schema did not change, so no reset was needed.

### W3, track R: O8 landed (merge `4487adfc`)

**Commits:** `edaff499` (O8.1), `03a0ba45` (O8.2), `77255ea8` (O8.3), `95515bf4` (O8.4), `d2add488` (O8.5, cut-over), `67936a44` (O8.6, deletion), `93161d64` (O8.7), `40b32c14` (O8.8).

The PostgreSQL catalog is now the only publication authority.

**Registry and catalog:**
- `MemberDescriptor` is one registry named structure, kept out of identity and fingerprint projections.
- The catalog statements, written for Cornucopia, own:
  - workspaces;
  - intents (X9);
  - a commit compare-and-set with a complete-request comparison, which refuses a second publication of the same attempt as `PublicationIdentityReused`;
  - settlement: committed, proved_noncommit and conflict with reason and head, where D9 settles a ticket with no registered intent as `ProvedNoncommit`;
  - reader leases that renew at a third of their TTL and cancel the reader on lapse;
  - change windows;
  - retention for the reasons publication, attempt and changes;
  - maintenance epochs and two-phase deletion.

**pse-catalog** keeps Delta member I/O:
- candidates that make nothing visible;
- member receipts v4, with v3 refused as MigrationRequired;
- `ReadScope` as the only cache key (X10);
- export manifests (X12);
- `collect`, `remove_tables` and `remove_prefix`.

**Runtime and Python:**
- The runtime publishes on the durable attempt id through an intent (X9).
- Python gains `register_workspace`, `workspace` and `head`; `Published`; `open` and `open_head` through a reader-lease guard; export and offline `open_export`.
- New `pse-publication` binary (export, retire, collect, reclaim), plus the `publication-test` and `pse-publication` recipes.

**Deleted:**
- the Delta control relation `runtime.publications`, together with `release_checkpoints`;
- `publish.rs`, `lease.rs`, `.pse-retention.lock`, `retention.rs`, ticket `settle`/`observe` and `PublicationRoot`;
- `prepare_checkpoint`, which had no caller and whose identity clashed with D1;
- the Python `pse.open(location, version)`, `PublicationRoot` and `PublicationRequest`;
- the control-mechanism tests.

The xtask inspection fixture is now an export manifest, so it needs PostgreSQL to generate.

**Additions and deviations:**
- **Retirement refinement.** Retiring a publication also removes inputs whose writer is already deleted, so a table written by a retired publication and read by a live one does not leak.
- **Retry without rewriting.** `ArtifactPlan::with_operation(attempt_id)` names the member writes, so a conflict loser re-prepares without rewriting members (the concurrency test confirms version 1 stays).
- **Commit grouping.** The O8.2 commit also carries statements, the repository and store tests, because the query crate needs them.
- **Also updated:** `docs/dev/native-workflow.md`.

**Merge resolution:**
- generated trees regenerated in bootstrap order;
- `test_modeling_run.py` combines O8's publication API with B5's typed settings;
- four B7 typed-id mismatches in the native-acceptance test support fixed (`tests/support/plan14.rs`, `acceptance/global_certification.rs`). The B7 verification had not compiled `pse-tests-conformance` with `native-acceptance`; that compile is now part of every native merge check.

**Tests on `main` after the merge** (zero baseline, `PSE_SOLVER_IMAGE` set):

| Command | Result |
|---|---|
| `just publication-test` | 9 passed |
| `just db-test` | 53 passed |
| `cargo nextest run -p pse-catalog` (force-validate) | 148 passed |
| `cargo nextest run -p pse-tests-engine -p pse-tests-lifecycle` | 16 passed |
| Runtime native units | 154 passed |
| `just worker-test` | 1 passed |
| `just native-test -E 'test(authored_publication_resource)'` | 1 passed (262 s) |
| `just governance-tests` | 87 passed |
| `just unit-invariant-harness 'all()'` | 411 passed |
| `just codegen-check`, `just conformance-fixtures-check`, `just family-check` | clean |

**Python:**
- The unit and component scopes and the durable publication tests pass, once the inspection fixture is built.
- `just inspection-fixture <dir>`, then `PSE_INSPECTION_PUBLICATION=<dir>` under `scripts/native-execution-env.sh`: 22 passed across `test_publication_streams`, `test_native_registry` and `test_native_caches`. The remaining unit and component tests: 144 passed.
- **Harness gap, for W6:** `just py-test` builds the fixture but runs pytest without the native library environment. `py-test` needs a native variant, or `native_exec`.

**Findings.** T02 is resolved by catalog reader leases held in short transactions; T16 by retention computed in catalog SQL, with windows, and with the lock file deleted. Evidence: `maintenance_waits_for_reader_leases`, `catalog_protects_published_versions`, `interrupted_deletion_resumes`.

### W4, track V: G8 and O5's incumbents landed (merge `b80f8d4c`)

**Commits:** `62231cb0`, `5f314f0c`, `431d4d39`, `6b05f58d`, `52d0cc2d`.

**What landed:**
- **Typed incumbent events.** The native `Event` carries a typed `IncumbentEvent`: objective in post-solve convention with the export offset, dual bound, gap, nodes, seconds and a throttled primal. Non-finite values are `None`.
  - SCIP reads incumbents from `SCIPgetBestSol` over the export's program columns, including the solution it holds at `initsol`, and only in the INITSOLVE to SOLVED stages (`SCIPgetGap` aborts earlier).
  - HiGHS emits one on callback kind 4, scaled to original coordinates.
  - Throttle: the first solution is captured immediately, then at most one per second, and the last incumbent always has a solution.
- **Durable sink.** The streamer stores each captured primal as a seed of its step (NLP seeds from SCIP, HiGHS seeds from HiGHS), plus an incumbents row, in one transaction. A new statement, `latest_in_attempt_chain`, follows `parent_attempt`.
- **Resume.** A claimed job honours `JobStart::ResumeFromParent` and `StoredSolution` through `with_stored_start` (`StartSource::Stored`, SCIP injection or HiGHS start), recorded as the try's `job.start` event. The "until G8" refusal is deleted.
- **Deleted:** `SolveReport::incumbents` and the duplicated bound metrics.

**Deviations:**
- HiGHS kind 3 is not an incumbent, because it fires for non-improving feasible solutions.
- The worker tests need `--memory-mib 8192` for SCIP.
- New `Runtime::work_once_with_result`.

**Open follow-ups:**
- durable incumbents are not published to Delta;
- the incumbents table has no `step` column;
- pse-py `ProgressEvent` does not expose the incumbent;
- captured solutions are never pruned;
- `StoredStart::Latest` may pick an incumbent capture;
- SCIP concurrent mode is untested;
- SCIP objectives for nonlinear objectives are epigraph values;
- architecture §5.3 still says the store stream is "not yet" built (docs step).

**Tests on `main` after the merge** (zero baseline, `PSE_SOLVER_IMAGE` set):

| Command | Result |
|---|---|
| `just check-solver-contracts`; `pse-tests-conformance` check with `native-acceptance,native-profiles,math-composition` | compile |
| Backend native units | 180 passed |
| Runtime native units | 157 passed |
| `just db-test` | 54 passed |
| `just worker-test` | 3 passed: `killed_worker_attempt_goes_stale_and_resumes_from_incumbent`, `cross_process_cancel_stops_scip`, end-to-end |
| `just publication-test` | 9 passed |
| `just native-test -E 'binary_id(pse-tests-conformance) and test(/coefficient_conic/)'` | 2 passed |
| `just codegen-check` | exit 0 |
| `just governance-tests` | 87 passed |

### W4, track R: O7 landed

**Commits:** `79661d4d`, `512f4444`, `353dd8e9`, `829b040e`, `8ba49bce`.

**Design as built:**
- **The study's own attempt.** `AttemptKind::study`, a *coordinating* attempt. It holds no lease and never runs; it stays `queued` while its points run, and ends as completed, partial, failed or cancelled in the transaction that makes its last point terminal. It is the attempt the study's one publication intent names (X9).
- **Finalization.** A job of its own (`AttemptKind::study_finalization`), created `waiting` when the study is created and released by the last-point transaction. It writes `runtime.study_outcomes` (one row per point) and commits the one publication.
- **Points.** Each point is a job with a typed `StudyPointBinding` in **job payload v3** (`JobPayload { version: 3, task }`: a modeling job with an optional binding, or a finalization).
  - A point with a predecessor waits (`JobState::waiting`, attempt `planned`) and is released when the predecessor completes.
  - If the predecessor fails or is cancelled, its dependents are cancelled transitively (`unattempted`, with a job error naming the predecessor).
  - A dependent point seeds from the predecessor's newest compatible stored solution (`latest_of_attempt`, `with_stored_start`), falling back to fresh with the reason recorded.
- **Member layout.** Point members go to `{prefix}points/{index}/{attempt}/…` (catalog `point_{index}`), and the summary to `{prefix}summary/` (catalog `study`); the publication kind is `relations`.
- **Cancellation.** Cancelling stops pending points, asks running tries to stop, and still publishes whatever completed.
- **Registry and store changes:**
  - `StudyState` becomes open, concluded, published.
  - `operational_studies` gains `attempt_id`, `publication_id` and `finalization_job`.
  - `operational_study_points` gains `predecessor` (composite self-reference, checked to be earlier) and `job_id`, and drops `attempt_id` and `result_ref`.
  - New `operational_study_point_members`.
  - New frames `DurableStudyRequestV1` and `DurableStudyPointBindingV1`.
- **Runtime and Python:** `Runtime::start_study`, `StudyHandle`, `Runtime.studies()`, `Runtime.work()` (an in-process worker loop), and a durable `ModelingPackage.study(..., runtime=…, workspace=…)` beside the ephemeral in-process path.

**Deliberate deviation from ADR-0114 Outcome 12** (accepted by the coordinator under the maintainer's standing direction):
- **What.** Legality now comes from two pure tables in `pse-operations` `lifecycle`, selected by the attempt kind's class: `TRANSITIONS` for executing attempts and `COORDINATING` for study attempts.
- **Why.** A study attempt run under a lease would go stale if its finalization crashed; its intent would then become reclaimable and the point members deletable.
- **Why the ADR's intent still holds.** One owner and a pure, testable authority remain.
- **Follow-up.** The docs step describes this in architecture §9.4.

**Other deviations:**
- `runtime.study_outcomes` is a new durable relation rather than a reuse of `runtime.modeling_studies`.
- A durable study needs the package's authored documents, so packages changed with `with_declarations`, `with_fit_data` or `with_limits` are refused.
- The v3 job request identity still uses the `DurableJobRequestV2` frame.

**Open issues** (for W6 or later):
- a crashed point try's partial member tables are never collected;
- a finalization that exhausts its retries leaves the study `concluded`, with no automatic recovery;
- the S15 10 000-point scale is unmeasured (creation is about six statements per point with no COPY, and each point transition serializes on the study row);
- a rare, retryable deadlock between a direct cancel of a waiting point and its predecessor's completion.

**Tests on `main` after the merge** (zero baseline, `PSE_SOLVER_IMAGE` set; the dev store was reset):

| Command | Result |
|---|---|
| `just db-test` | 59 passed |
| Runtime native units | 161 passed: `predecessor_waits_and_seeds`, `failed_point_does_not_contaminate`, `study_cancel_stops_pending_points` |
| `just worker-test` | 4 passed, including `study_parallel_workers_publish_once` with two worker processes |
| `just publication-test` | 9 passed |
| `just governance-tests` | 87 passed |
| schema and codegen | 69 passed |
| pse-ids frames | 46 passed |
| `just codegen-check` | exit 0 |
| `just family-check` | OK |
| Python unit and component (inspection fixture built) | 157 passed |
| Python `test_studies.py`, native workflow, modeling run, Plan 14 acceptance | 24 passed |

### W4, track T: B3b landed (in merge `3c6407ad`)

The merge is part of the maintainer's commit `3c6407ad`, which also carries the environment and
library-skill changes. B3b commits: through `9f7ef15b`.

**Typed:**
- **Registry.** Identities on the publication manifest, workspace, attempt and reader-lease columns; `declare_publications` declares the manifest's four identities.
- **pse-catalog.** Member receipts, write evidence and the publication ticket carry typed ids; the v4 receipt JSON is unchanged. `delta::manifest::publication_of` is the one place a manifest key becomes a `PublicationId`.
- **pse-runtime** (outside O7's files). `prepare_fit` takes a `FitId`; experiment simulations, modes and trajectories are keyed by `InstanceId`; steady coordinates are `TiVec<GlobalCol, (SemanticId, OriginalCol)>` and constraints `(GlobalRow, OriginalRow)`.
- **pse-backend-native.** `Oracle::support` returns `SupportEntry = Entry<OriginalRow, OriginalCol>`; the integrator, IDAS and derivative-diagnostics adapters convert at their edge. This closes the B6 follow-up on the dynamics and fitting oracles.
- **Frames.** The frame table is grouped by area and published as `docs/generated/frames.md` (ADR-0115 Outcome 4).
- **Python.** 22 annotations use the generated `NewType` ids (`RunId`, `AttemptId`, `PublicationId`, `WorkspaceId`, `FitId`, `InstanceId`).
- **`compile_fail` doctests:** an attempt where a publication belongs, a model where a case belongs, a coordinate where a function row belongs.

**Unchanged:** identity bytes, frame spellings, receipt JSON and the store schema. The registry
fingerprint and six contract fingerprints changed (`authored.fit_cases`,
`runtime.publication_manifests`, `runtime.fit_variables`, `fit_constraints`, `fit_observations`,
`response_sensitivities`); tables written before are regenerated.

**Model and case, made explicit.** Every place that sets a `model_id` or `case_id` now calls a
named derivation in `pse_model::lineage` (13 call sites). This shows that in every lineage row
`model_id` and `case_id` hold the same bytes, and that one model is named by different ids in
different places (a steady fit experiment's requirements use the experiment's instance, a
transient one the case declaration). Requirement rows are hashed into the numerics identity.
**Proposal, awaiting the maintainer:** the model is the specialized definition and the case is
the case declaration, both typed `declaration`; the instance and the fit get their own columns;
the unused `model` and `case` identities are dropped. It changes published column meanings and
the numerics identity bytes, so it needs a decision record.

**Merge resolution.** B3b's area-grouped frame table plus O7's two study frames; O7's study enums
beside B3b's typed-id imports in `_runs.py` and `_workflow.py`; generated trees regenerated in the
bootstrap order.

**Tests on `main` after the merge** (zero baseline, `PSE_SOLVER_IMAGE` set):

| Command | Result |
|---|---|
| `just check`; conformance with `native-acceptance,native-profiles,math-composition`; `just check-native-python` | compile; two unused-variable warnings, both from before this track (`pse-modeling` `expression.rs`, `pse-runtime` `workflow/modeling/dynamics.rs`), for W6 and B3c |
| `just unit-package pse-ids` | 39 passed |
| schema, codegen, model and vocabulary units | 80 passed |
| `just unit-package pse-math` | 76 passed |
| modeling and compiler units | 149 passed |
| `just unit-package pse-catalog` | 50 passed |
| `just db-test` | 59 passed |
| backend native units | 180 passed |
| runtime native units | 161 passed |
| `just worker-test` | 4 passed |

`just publication-test` ran after the O9 merge below; the governance and harness suites wait for W6.

### W5, track R: O9 landed (merge `3a097532`)

**Commits:** `bd780142`, `579c3d38`, `2dff880e`, `5acbe1d1`.

**Built:**
- **Providers.** `crates/pse-runtime/src/workflow/operational_tables.rs` serves 13 operational relations under `pse_ops` (attempts, attempt transitions, jobs, progress events and values, incumbents, solutions, studies, study points, workspaces, publications, publication members, settlements). Each runs a generated statement from `queries/tables.sql` (`pse_operations::tables`), pages in primary-key order after a keyset position, and builds batches with the generated builders.
- **Pushdown** is `Inexact` for identity and state equality or IN lists and for time comparisons and BETWEEN (DataFusion's OR chains for short IN lists count as one list), and `Unsupported` otherwise. A filter that cannot match runs no statement. Pushed limits and projections are honoured; no connection is held between pages.
- **Sessions.** `Runtime::operational_tables`, `with_operational_tables` and `query_session` (optionally with a run's results under `workspace.*` or from a publication's session). An ephemeral runtime registers none. Publication sessions do not include the providers by default, because an observed source disables the cache for the whole session.
- **Progress.** `Runtime::progress(attempt, follow, page, cancel)` merges progress events and incumbents in time order and follows the listener until the attempt ends; recording an incumbent also notifies.
- **Python.** `Runtime.jobs(states=, limit=)`, `Runtime.query(sql, result=, publication=)` returning a streaming `TableStream`, and `Runtime.progress(attempt_id, follow=True, page=256)` returning a closable `ProgressStream`. `ProgressEvent` gains `incumbent` (`pse.Incumbent`), `step`, `sequence` and `at`.
- **ADBC** is not adopted; the reason is in `docs/dev/operational-store.md` under "Query surface".

**Deviations:**
- Incumbents store `step`, `elapsed_seconds`, `phase`, `nodes` and `seconds`, so a followed stream returns complete typed incumbents. This closes G8's missing `step` column.
- `ProgressWatcher::next` takes and returns positions in both streams.
- A followed stream waits up to 5 seconds for its attempt to appear, since a run registers its attempt just after `start()` returns.
- The provider types are crate-private; the surface is the `Runtime` methods and `OPERATIONAL_SCHEMA`.

**Open issues:**
- Pages read at READ COMMITTED: a scan sees every row that existed throughout it, but not one snapshot.
- Statement performance at scale is unmeasured.
- Python checks only that Ipopt events carry no incumbent (the fixture package has no `Indicator` quantity); typed incumbents are exercised by the Rust SCIP and HiGHS tests.
- In the worktree, `reclaimable_intents_are_fenced` once failed dropping its test database (another role held a connection) and `catalog_protects_published_versions` once timed out at 120 s; both passed on rerun without changes. The failed run left a test database on the server.
- O9's entry points take `SemanticId` rather than typed ids (B3c).

**Tests on `main` after the merge** (zero baseline, `PSE_SOLVER_IMAGE` set; store regenerated with `just --yes db-reset`):

| Command | Result |
|---|---|
| `just check`; conformance with `native-acceptance,native-profiles,math-composition`; `just check-native-python` | compile; only the two known warnings |
| `just db-test` | 61 passed |
| runtime native units | 167 passed, including `operational_tables_join_results_in_datafusion` and `provider_pushes_attempt_filter` |
| `just worker-test` | 4 passed |
| `just publication-test` | 9 passed |

The Python surface (`test_progress_stream_python` and the O9 module) passed in the worktree
(84 passed across 11 modules); it was not rebuilt on `main`, and runs again at W6.

### W5: B3c and the docs step landed

**B3c** (`4b9d16f7`, `3d878717`; worked in the main checkout).
- **Explicit owners.** `FieldContract::with_owned_identity` declares ownership on the 12 owning keys. An owner must be its relation's single-column primary key and cannot be nested; a column that only carries an identity never owns it, so the export manifest carries `publication`.
- **Model and case** (decision 7). `model_id` is the root declaration the model specializes, and `case_id` that root when it is a case or a test; both are typed `declaration`. `instance_id` and `fit_id` are new columns. The `model` and `case` identities and `ModelId`/`CaseId` are deleted. `runtime.run_lineage` is at v2, `solve_runs` at v4 and `numerical_requirements` at v2. Producers derive the rows from `ModelingPreparation`'s `Solved` or `FitProblem`'s `Fitted` through `pse_model::lineage`.
  - **Coordinator-accepted choices.** A `test` root counts as a case. A fit names a model and case only when all its experiments share them, so `model_id` is optional (null for a fit across definitions). The implicit stage's trial hints carry the enclosing specialization's model and case, with the block's instance.
- **Run identity.** `run` is its own identity, not the attempt's: an ephemeral run has none, and a job's retries share its run. The `run` identity is declared on 31 `run_id` columns plus the findings table and the nested run links, and about 30 runtime signatures take `RunId`.
- **Bug fixed.** A worker's claimed try minted a fresh run id, so its result rows never matched `operational_attempts.run_id`. The try now runs under the stored run id; `worker_runs_an_authored_job_and_stores_its_seed` asserts it.
- **Other typed ids.** `WorkflowError` ids; the job's and study point's `case` as `DeclarationId`; `SeedOrigin::run`. On the Python side, `Runtime.progress(AttemptId)`, `Runtime.study(StudyId)`, the typed study and progress ids, and `_modeling.py` case and fixture ids as `DeclarationId`.
- **Deleted:** `manifest::publication_of`, the no-op conversions, and both pre-existing unused-variable warnings.
- **Bytes.** Numerics identity bytes, the registry fingerprint and the affected contract fingerprints changed. The store fingerprint is now `d12602d2…` (dev store reset). Frame spellings and golden vectors are unchanged. The invariant fixtures were regenerated, which also picked up the O7 and O9 store relations.

**B3c tests on `main`** (zero baseline, `PSE_SOLVER_IMAGE` set):

| Command | Result |
|---|---|
| `just check`; conformance with native features; `just check-native-python` | compile; five unused-`mut` warnings in `conformance.rs` and `pure.rs` predate B3c (W6) |
| schema, codegen, model and vocabulary units | 81 passed |
| pse-model doctests, including the `compile_fail` swap | 4 passed |
| pse-schema tests with pse-relations | 151 passed |
| modeling and compiler units; `just unit-package pse-catalog` | 149 and 50 passed |
| runtime native units; backend native units | 167 and 180 passed |
| `just db-test` / `just worker-test` / `just publication-test` | 61 / 4 / 9 passed |
| Python unit and component tests (12 touched modules); integration `test_studies`, `test_operational_queries`, `test_modeling_run`, `test_plan14_acceptance` | 79 and 9 passed |

**Docs step.**
- **Revision 67** (`12321374`, `425a7b9c`):
  - the typed-data markers now describe the system as built;
  - new blueprint §20.6 "Operational store and durable execution" holds the store contract and the O3–O9 and G8 text;
  - ADR wording deviations are recorded at their owning sections, with no ADR edited;
  - architecture companion notes for §9, §12 and §5.3;
  - `docs/dev/operational-store.md` rewritten;
  - TD01–TD11, T02 and T16 resolved with their tests;
  - register rows R-36 (automatic retention) and R-37 (remote object stores).
- **Revision 68** (`0e727af5`) adds B3c's model, case, run and ownership meanings to §5.1, §5.2 and §20.3.
- **Left for the solver docs pass:** stale G4–G7 claims in §3.3, §18.9 and §18.10.1.

**Next:** W6 scoped qualification.

### W6: scoped qualification

Run on `main` on 2026-09-28 against the zero baseline, with `PSE_SOLVER_IMAGE` set to the
local immutable image id. By maintainer direction (decision 8), use-case and journey suites
(the native-acceptance conformance runs, parity, the invariant harness) were not run.

**Fixes made to reach zero:**
- **Formatting** (`13f4f25b`): `cargo fmt`, taplo and ruff format over the track.
- **Rust lint** (`0acf0db7`, `b548e97d`, `50f5dc29`). 559 clippy findings resolved:
  - mechanical fixes, and about 270 missing docs;
  - dead code deleted, including the unwired dynamic-partition chain in `pse-compiler`;
  - `large_enum_variant` boxing and local type aliases;
  - reasoned `#[expect]`s, mainly `too_many_arguments` (32). About 12 of those pass the same root, instance, bindings and limits, and a request struct would remove them.
- **Python lint and types** (`65b649c6`, `83a1a255`). 167 ruff and 49 pyrefly findings resolved. Generated-file findings were fixed in the generators, and ids are typed at their source. Reasoned suppressions remain: S608 ×4 in the query tests, which would go if `Runtime.query` gains bind parameters; one overload break in a refusal test; SLF001 ×1; N999 ×4.
- **Repository lint** (`fc7abb12`, `1ae91c2f`, `7281ac97`, `44e3c2b0`):
  - a typos allowlist for native C symbols and deliberate test misspellings;
  - SPDX headers;
  - the thermodynamics handoff excluded from every checker (maintainer direction);
  - authored `packages/**` excluded from taplo (maintainer direction);
  - H1 titles on the two external PostgreSQL reviews so the book builds.

**Results:**

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `just clippy` (`clippy-default`, `clippy-no-default`) | both passed |
| `just quality` | every check passed except `lint-toml`, which failed only on the two `packages/reference/*/package.toml` files, since excluded (`7281ac97`, not re-run) |
| `just governance` (governance tests, all seven codegen checks, `family-check`) | passed |
| `just adr-lint`; `just docs`; `just docs-test` | passed; 144 chapters built; 8 passed |
| `just db-test` / `just worker-test` / `just publication-test` | 61 / 4 / 9 passed |
| runtime native units; `just unit-package pse-compiler` | 167 and 96 passed |
| Python unit and component tests; the Python modules the lint pass changed | 157 and 72 passed (lint agent's run) |
| `cargo nextest run -p pse-catalog -p pse-tests-engine -p pse-tests-lifecycle` (force-validate) | 151 passed, **13 failed** |

**The 13 failures** were a B3c regression. The public `declare_diagnostics` entry point
builds registries in the engine, lifecycle and pse-rules tests, and it did not declare the
`run` identity that `runtime.diagnostics_findings.run_id` now references (`UnknownReference
identity:run`). `598b82ef` gives the run identity its own declaration, used by the operational
catalog and by `declare_diagnostics`, as `declare_publications` does for its identities.
**Not re-run** at the maintainer's request. The next session runs that nextest command and the
pse-rules tests first.

**Excluded, for Q1:** native-acceptance conformance (`authored_publication_resource`
included), parity, the invariant harness, and performance at scale (S15, the O9 scans).

**Track state.** The store and typed-data track's functional scope is complete. Solver scope
resumes from the [main execution packet](22-solver-capabilities-execution.md).

