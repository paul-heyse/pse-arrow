# Design review: typed data contracts — the operational store, generated schemas and domain typing

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | How pse-arrow types, declares and generates the data that crosses its boundaries. The review reassesses the PostgreSQL client stack (ADR-0112 Outcome 11) now that Core 3.1 no longer prefers runtime mechanisms over generated code. It also assesses every library proposed in the maintainer's two external reviews ([PostgreSQL options](../../external-review-postgresl-options.md); [typing and other enhancements](../../external-review-postgresl-typing-and-other-typing-enhancements.md)) against what the codebase already has. **In the boundary:** the operational store (`pse-operations`, its migrations, and its consumers `pse-runtime` `workflow/{durable,worker}.rs` and `pse-py`); the registry and generator (`pse-schema`, `pse-codegen`, `xtask` codegen, generated `pse-model`/`pse-relations`/`python/pse/contracts`); identities (`pse-ids`, `pse-quantity` typed ids); dense index spaces at coordinate boundaries (`pse-math` assembly/sparse, `pse-backend-native` presolve/transport/conditioning); backend settings and durable JSON documents (`pse-backend-native` `execution.rs`, `pse-runtime` worker payload); the Python settings boundary (`pse-py` `settings.rs`, `_native.pyi`). **Suppliers:** sqlx 0.9.0, tokio-postgres 0.7.18, postgres-types 0.2.14, Cornucopia 1.0.1, PostgreSQL 18.6, and the candidate typing libraries (slot 8). **Consumers:** Plan 22 packets O7–O9, G8 and the Python run surface. |
| Standard | Core **3.1** (AP-01–AP-06, DP-01–DP-24, G1–G9); process-simulator profile **1.1** (PS-01–PS-13, PS-G1–PS-G3); binding `pse-arrow` ([standard.toml](../design_principles/standard.toml)). Binding conflict K2 (the preference against generated projections) is resolved in Core 3.1, so generated code is judged on its merits here. |
| Tier / purpose | **Design tier, target purpose** (binding default). Authority text that blocks the target is recorded in slot 11 as a required change. |
| Reviewer / date | Agent review in the maintainer's session, 2026-09-28. Four read-only source surveys (store typing, registry/codegen, domain typing, library facts) supplied the counts and file references; the reviewer spot-checked the decisive ones (below) and ran one PostgreSQL probe. **Author review, not an independent review.** |
| Decisions | Current implementation: behavioural adequacy **fails** G1, G2, G3 and G8; architectural fitness **fails** G9 (AP-04, AP-05 violated). Overall on the current state: **Revise**. The proposed target in slot 9 is **Accept-scoped** at the *Proposed* level, pending ADR-0114 and the packet-start checks in slot 12. See [slot 12](#decision). |
| Disposition owner | [Plan 22 finding dispositions](../../plans/22-solver-capabilities.md#finding-dispositions), packets D1 and B1–B7. |

**Maintainer direction for this review (2026-09-28).**
1. Switch the store to rust-postgres/tokio-postgres if that is advantageous. sqlx's reconnecting listener, embedded migrations and per-test database harness are minor considerations against the typing benefits.
2. The operational store holds only regenerable results. Migrating stored data is not a design constraint.
3. The more programmatic and the less static the codebase, the better. Repository rules that limit a better design are changeable.

  The `from __future__ import annotations` ban was one such rule: `pse.governance` already resolved annotations. It was removed in this session, with `test_postponed_annotations_are_resolved_before_the_lint`.

**Functional target.** A process simulator whose data contracts are declared once and derived everywhere else:
- the registry declares meaning, and generates the Rust, Python, Arrow and PostgreSQL representations;
- SQL statements are checked against the schema before anything runs;
- identities, vocabularies and coordinate spaces are distinct types;
- durable documents have published, versioned schemas.

It follows the functional target of the [Plan 22 architecture](../../plans/22-solver-capabilities-architecture.md#1-target-drivers-and-scenarios); this review adds no solver capability.

**Drivers.**
- DP-01 and AP-04: one authority per fact; generated artifacts derived, never edited.
- DP-02 and DP-04: semantic distinctions are types, and identities are semantic and canonical.
- DP-03 and AP-05: invariants have an enforcement point, discovered before execution where practical.
- DP-24: durable documents are versioned and discoverable.
- DP-13 and DP-14: library first.
- AP-06: pure decisions stay testable without a database.

**Baseline.** Plan 22 O1–O6 as merged at `e734d696`:
- sqlx 0.9 with runtime-typed `query`/`query_as` and hand-written `FromRow`;
- two hand-written migrations (411 lines, 74 CHECKs) and a migration-conformance test;
- registry-declared `runtime.operational_*` relations generated into Rust rows and Arrow builders.

O7–O9 (studies, catalog, query surface) are not started. The seven catalog tables exist in DDL only.

**Not examined.**
- Property physics beyond PS-01's typing question (uom).
- Runtime behaviour of any library not yet linked (tokio-postgres, Cornucopia, schemars, nutype, typed-index-collections): *Interface-checked* at most.
- Performance: binary COPY versus `UNNEST` batches, and generation cost. All *Proposed* until Q1 measures them.

**Evidence base.**
- **Source read by the reviewer:**
  - `pse-operations` `lib.rs`, `codec.rs`, `lifecycle.rs`, `attempts.rs:1-200`, `jobs.rs:305-345`
  - `pse-runtime` `workflow/worker.rs:270-300, 415-440`
  - `pse-modeling` `annotation.rs:41-45, 179-181`
  - `python/pse/governance.py`
  - `tests/governance/tests/no_shadow_structs.rs`
  - blueprint §4.1–§4.3
  - ADR-0112
  - `Cargo.toml` pins; `justfile` store recipes
  - `cargo tree -e features -i serde_json`, which shows `preserve_order` and `float_roundtrip` unified in
- **Survey facts** (with file:line references, cited in the findings): statement and bind counts, the conformance test's coverage, registry/codegen structure, identity and index-space counts, settings fields.
- **Library facts:** crates.io, GitHub and Context7 for sqlx 0.9.0 (tag `v0.9.0`), Cornucopia (`main` at `b078d22`; CHANGELOG; `src/lib.rs`, `src/codegen/cargo.rs`, `src/type_registrar.rs`), rust-postgres (`1084ca8`), deadpool, refinery, tokio-postgres-rustls, datamodel-code-generator and the typing libraries. All *Interface-checked*.
- **Probe** (*Measured*, PostgreSQL 18.6, cluster `18/main`, inside rolled-back transactions, 2026-09-28): a misspelled literal compared with an ENUM-typed column fails at `PREPARE` (`invalid input value for enum`). The same literal against a text domain with a CHECK prepares, executes and silently returns zero rows. It fails only when inserted.

**Variation axes.**
- A new operational column, table or enum member (S19).
- A statement drifting from the schema (S20).
- A durable document changing version (S21).
- A new identity or coordinate space (S22).
- An invalid setting arriving from Python (S23).
- Replacing the store driver or a library major (S24; PSE-S06).
- Testing a pure rule without a database (S25; PSE-S05).

## 2. Decomposition, ownership and dependencies

| Component / responsibility | Decision or invariant hidden | Contract consumed / exposed | Dependency direction and reason | State/effect owner | Local test setup |
|---|---|---|---|---|---|
| Registry (`pse-schema`) and generator (`pse-codegen`, `xtask`) | Meaning of every durable relation, enum, invariant and document; how each representation is rendered | `Registry` → `GeneratedTree` (Rust, Python, Markdown targets); regeneration check (ADR-0031/0051) | Foundation; pure | None | Pure generation tests |
| Generated rows and enums (`pse-model`, `pse-relations`, `python/pse/contracts`) | Row shapes, enum spellings, Arrow codecs | Typed rows; `as_str`/`FromStr`; Arrow builders | Derived from the registry | None | Codec round trips |
| Operational store (`pse-operations`) | SQL, transactions, lifecycle legality, SQLSTATE mapping, catalog CAS | Repository functions over hand-written records (`AttemptRecord`, `JobRecord`, …); `Store`; `OperationsError` | Depends on `pse-model`, `pse-ids`, `pse-diagnostics`; `pse-runtime` depends on it | PostgreSQL `pse_ops` | 13 pure tests; 23 database tests (`#[sqlx::test]`) |
| Durable workflows (`pse-runtime` `durable.rs`, `worker.rs`) | Durability class, payload versions, termination codes | Store repositories; `ModelingJob` | Composition root | Attempts, jobs | 5 of 6 durable tests need a database |
| Identities (`pse-ids`, `pse-quantity` `ids.rs`) | Canonical identity and hash framing; typed physical-registry ids | `SemanticId`, `ContentHash`, `FramedHasher::new(&'static str)`; 13 `semantic_id_newtype!` ids | Foundation | None | Pure |
| Coordinate spaces (`pse-math` assembly/sparse; `pse-backend-native` presolve/transport/conditioning) | Mapping between original, presolved, instance-local and global coordinates | Raw `usize` maps and triplets | Math → backend adapters → FFI (`i32`/`usize`) | Per-attempt workspaces | Pure |
| Backend settings and documents (`pse-backend-native` `execution.rs`; `pse-runtime` worker) | Setting domains; versioned documents; request identity | serde structs with `deny_unknown_fields`; `admit_settings`; `SettingsDocument`; `enqueue(payload: Value)` | Settings owned by the backend crate | None | Pure |
| Python boundary (`pse-py`, `python/pse`) | Settings projection, run listing | pyo3 classes (`SolveSettings` with `str` fields; `BackendSettings(**fields: object)`); generated `_native.pyi` | Over `pse-runtime` | None | `py-unit` |

The composition root for durable work is `pse-runtime`. The consequential dependency for this
review is that `pse-operations` sits at the join of three independently maintained
descriptions of each table (slot 3). None of them is derived from another, and only one pair
is compared.

## 3. Contracts, authority and constraints

| Meaning / contract | Authoritative owner and update path | Consumer obligations / invariant | Enforcement and failure | Derived representations / evolution |
|---|---|---|---|---|
| Operational relation shape (tables, columns, nullability, keys) | Registry `catalog/operations.rs` for 11 tables; **DDL only** for the 7 catalog tables | Store and published rows agree | Conformance test: tables, columns, type family, nullability, single-column enum CHECKs, PK, declared FKs. It exempts the catalog tables and skips multi-column CHECKs, UNIQUE, composite FKs, defaults, indexes, grants and text-versus-jsonb | Hand-written migrations; hand-written records and `FromRow`; column macros; generated rows. **Four descriptions, one comparison** (TD01) |
| Statement correctness (column names, bind order and types, enum literals) | The SQL strings in the repository modules | — | Only by a database test that executes that statement (TD02) | None |
| Enum vocabularies in the store | Registry enums, stored as text (ADR-0112 Outcome 10) | Bind `as_str()`, parse with `FromStr` | Single-column CHECK; 12 inline literals in 9 statements, plus partial-index predicates, are unchecked | Generated `as_str`/`FromStr`; Python StrEnums |
| Termination, retention phase, settlement outcome | **None**: `String` codes and SQL literals | Consumers compare strings | CHECK on phase and outcome only; codes unchecked (TD03) | Publication outcome declared three times (Rust, SQL, Python) |
| Entity identities (run, attempt, job, publication, workspace, …) | Runtime-minted `SemanticId` (ADR-0112 Outcome 13) | The right id in the right parameter | None: one type for all entities (TD04) | 13 typed physical-registry ids exist in `pse-quantity` only |
| Coordinate spaces | Implicit in variable names | The right index in the right vector | None (TD05) | `Compatibility` stamp (runtime tag); `NumericalCoordinates` tag |
| Backend-setting vocabularies (linear solver, ordering, Hessian mode, reuse) | ADR-0113 Outcome 2 says registry enums; the code has hand-written serde enums | Python passes strings | serde parse | No Python enum types (TD06) |
| Durable documents (job payload, termination detail, source manifest, settings document) | Rust types where they exist; `json!` literals otherwise | Versioned where stated | Payload: one equality check on the version; detail and manifest: unversioned (TD07) | No JSON Schema; Python sees `**fields: object` |
| Hash-frame contexts | 119 distinct `"pse.*.vN"` literals at 90 call sites | Frame bumped when meaning changes | None (TD08) | — |

**Intended versus observed.**
- **ADR-0113 Outcome 2** is specified but not realized for the Ipopt linear solver and ordering (TD06).
- **Architecture companion §9.3 and §9.8** describe columns and dependencies the code does not have (TD12).
- **ADR-0112 Outcome 10** ("no database enum types that could drift") rested on hand-written DDL. With generation, the drift argument disappears, and the probe shows ENUM types add parse-time checking that text CHECKs cannot give.

## 4. Change scenarios and composition

S19–S25 are defined in the [architecture companion §1](../../plans/22-solver-capabilities-architecture.md#1-target-drivers-and-scenarios).
This slot records current and proposed responses.

| Scenario / stimulus and conditions | Expected response and change boundary | Edit/composition path | Observed or predicted impact | Acceptance and evidence |
|---|---|---|---|---|
| [S19](../../plans/22-solver-capabilities-architecture.md#s19) Add a column or enum member to an operational relation | One registry declaration; DDL, types and rows follow by regeneration; only statements that should use the column change | **Now:** registry → migration → record struct → `FromRow` → column macro → `row()` → tests. **Target:** registry → `just codegen` (DDL, enum type, rows, query crate) → the query files that use it | Now: six hand edits in one owner, two of them compared (TD01) | *Proposed*; B1 `generated_schema_creates_empty_store`, B2 `store_queries_regenerate_identically` |
| [S20](../../plans/22-solver-capabilities-architecture.md#s20) A statement drifts from the schema: misspelled column or enum literal, swapped binds | Rejected at code generation, never at run time; a silent zero-row match is impossible | **Now:** caught only if a database test executes the statement. **Target:** Cornucopia prepares every statement against the generated schema; ENUM literals fail at `PREPARE` (*Measured*); named parameters | Now: a misspelled `state = 'queud'` in a claim silently claims nothing (probe) | *Measured* (probe); B2 `generated_query_rejects_misspelled_column_and_literal` |
| [S21](../../plans/22-solver-capabilities-architecture.md#s21) A durable document changes (job payload v2, a settings field) | New version, published schema diff, typed Python types regenerated; an unknown version is refused | **Now:** edit the serde type and the `if version !=` check; no schema; Python unaware. **Target:** schemars schema + generated Python types; the typed enqueue API | Now: request identity varies with key order and with the build's serde features (TD07) | *Proposed*; B5 `job_request_identity_independent_of_key_order` |
| [S22](../../plans/22-solver-capabilities-architecture.md#s22) A caller passes a run id for an attempt id, or a presolved column for an original column | Compile error | Registry-declared entity ids; typed index spaces at coordinate boundaries | Now compiles (53 functions take ≥2 `SemanticId`; presolve maps are `Vec<usize>`) (TD04, TD05) | *Proposed*; B3/B6 `compile_fail` doctests |
| [S23](../../plans/22-solver-capabilities-architecture.md#s23) An invalid scalar setting (negative tolerance, `NaN` bound, unknown linear solver) arrives from Python | Rejected at the boundary with a typed cause, visible to Python type checkers beforehand | Generated Python types; validated scalar types; `admit_settings` keeps cross-field and environment rules | Now: strings and `**fields: object` cross; validation in 87 hand-written `validate*` functions | *Proposed*; B5 `python_backend_settings_typed` |
| [S24](../../plans/22-solver-capabilities-architecture.md#s24) Replace the store driver, or take a major of it | Only `pse-operations` and its generated query crate change | Repository API unchanged; generated crate regenerated | The switch proposed here is itself an instance; consumers see repository types, not driver types | *Proposed*; B2 |
| [S25](../../plans/22-solver-capabilities-architecture.md#s25) Test a pure store rule (legality, payload decoding, value mapping) without PostgreSQL | Pure unit tests | Lifecycle table (exists); generated `ToSql`/`FromSql` round trips; typed payload decoding | Now: lifecycle legality pure (a strength); decoding reachable only through a database | *Proposed*; B1 `postgres_value_mapping_round_trips` |
| Add an operational relation for O7 studies or the O8 catalog | One registry declaration, then the queries | As S19 | Now: the catalog tables live outside the registry and are exempt from conformance (TD01) | B1 declares them before O7/O8 |

## 5. Mechanisms and execution

| Stage / owner | Contract and mechanism | Inputs / dependencies | Effects and lifecycle | Reuse / equivalence / limits | Evidence or uncertainty |
|---|---|---|---|---|---|
| Schema generation (`pse-codegen` target `postgres`) | Registry → `pse_ops` DDL: ENUM types per registry enum, identity domains, tables, NOT NULL, PK, FK, CHECKs from registry invariants; schema fingerprint | Registry; a hand-written `physical.sql` for indexes, grants, defaults and append-only revokes, whose enum literals are checked when applied | None; files only | Byte-identical regeneration (ADR-0051) | *Proposed* |
| Store creation (`Store::open`) | Empty database → apply generated DDL + `physical.sql`, record the fingerprint; different fingerprint → typed `SchemaMismatch` naming `just db-reset`; never resets implicitly | Fingerprint | Creates `pse_ops` | No data migration while contents are regenerable (maintainer, 2026-09-28); versioned migrations return at register row R-35's trigger | *Proposed* |
| Query generation (Cornucopia 1.0.1 `gen_fresh`, called from `xtask`) | `.sql` query files with named parameters and declared nullability → the `pse-operations-queries` crate over tokio-postgres; `types.mapping` (generated from the registry) points each ENUM and identity domain at the pse-model or pse-ids type | Generated DDL + `physical.sql` loaded into a temporary database on the local PostgreSQL 18 server | Creates and drops a temporary database | Regenerated into a temporary directory and diffed | *Interface-checked* (`src/lib.rs` `gen_fresh`; `use-workspace-deps`) |
| Value mapping (`postgres` feature of `pse-ids` and `pse-model`) | `ToSql`/`FromSql` generated for registry enums (derive, by type name) and identities (uuid; domain accepted for parameters, base type for results); `ContentHash` ↔ `bytea(32)` | postgres-types only (no driver) | None | Pure round-trip tests | *Interface-checked* (postgres-types derive; PostgreSQL 18 `printtup.c` sends a domain's base type) |
| Driver and pool | tokio-postgres 0.7.18 + deadpool-postgres 0.14.2 (`prepare_cached`); `SqlState` constants; `BinaryCopyInWriter` for progress, values and incumbents; pipelining; TLS through tokio-postgres-rustls 0.14 (ring) | — | Connections per worker | — | *Interface-checked* |
| Notifications | A listener task on a dedicated connection re-issues `LISTEN` after reconnecting and signals a resynchronization; `cancel_requested` stays the authority (T03) | — | Reconnects with backoff | Notifications during an outage are lost either way; the re-read is mandatory | *Proposed* (bespoke, small) |
| Test databases | The existing `testing.rs` harness on tokio-postgres: create a database from the generated schema, drop it after | Local server | Per test | — | *Proposed* |

## 6. Architectural assessment and gates

| Foundation | Scenario and evidence / scope reason | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | The store owns SQL and transactions; the registry owns meaning; the runtime composes. No crossing beyond the store owner in S19–S25 | **satisfied** | — |
| AP-02 Stable contracts | Consumers use repository functions and records, never sqlx types; a driver switch stays inside `pse-operations` (S24) | **satisfied** | — |
| AP-03 Composition | New operational relations and statements compose through the repository modules; no workflow copies | **satisfied** | — |
| AP-04 Authoritative meaning | Four hand-kept descriptions of each table, one partial comparison (TD01); twelve vocabularies with two Rust types, and specified registry enums not realized (TD06); request identity over a non-canonical encoding (TD07) | **violated** | B1, B2, B4, B5 |
| AP-05 Explicit structure and constraints | Statement correctness implicit until execution, with a silent zero-row failure mode (TD02, *Measured*); coordinate spaces and entity identities implicit in names (TD04, TD05); stored codes as strings (TD03) | **violated** | B2, B3, B6 |
| AP-06 Local reasoning / testability | Lifecycle legality is a pure table with exhaustive tests; the `Ephemeral` class keeps library use database-free. Value decoding is testable only through a database (S25), which is a weakness, not a violation | **satisfied** | B1 makes decoding pure |

| Gate | Result | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | **fail** | Hand-written migrations, records, `FromRow` and column lists beside generated rows, with multi-column rules (`solutions_vectors_check` vs `SeedVectors`; `progress_values_one_value` vs registry `one_evidence_value`) compared nowhere (TD01). The hand-written `AttemptRecord` restates the generated row shape that blueprint §4.2 forbids; `no_shadow_structs` matches names only. ADR-0113's registry enums missing (TD06). Canonicalization of the job request identity undefined (TD07) | B1, B2, B4, B5 |
| G2 Semantic fidelity | **fail** | Termination code mixes six vocabularies in one `String`; retention phase and settlement outcome are literals (TD03); one identity type for every entity (TD04); nanosecond timestamps declared for microsecond columns; text and jsonb accepted as equal (TD09) | B1, B3 |
| G3 Validity | **fail** | A misspelled enum literal in a `WHERE` clause prepares and returns nothing (*Measured*); 165 positional binds, where two same-typed parameters can be swapped silently (TD02); CHECK and FK violations classified `Internal` (TD03) | B2 |
| G4 Hidden behaviour | **pass** | No inspection path changes meaning; store reset is explicit, never implicit (target) | — |
| G5 Consistency and recovery | **pass** | Lifecycle, leases and cancellation authority are sound (ADR-0112). TD02 threatens them (a sweep or claim that silently matches nothing), but no such defect was observed | B2 removes the threat |
| G6 Transformation and reuse | **pass** | Registry → generated projections are regeneration-checked. TD05 is a latent reinterpretation risk at coordinate maps, recorded under G2's intent rather than as an observed failure | B6 |
| G7 Truthful capability claims | **pass** | No capability claimed beyond its route. ADR-0113's unrealized enums are an authority divergence (G1), not a capability claim | — |
| G8 Library leverage | **fail** (minor) | SQLSTATE classified by string prefix where typed constants exist in the pinned ecosystem's alternative driver; a 301-line bespoke JSON Schema emitter (`pse-codegen` `codegen/jsonschema.rs`) with no stated reason (TD11) | B2, B5 |
| G9 Architectural fitness | **fail** | AP-04 and AP-05 violated | B1–B6 |
| PS-G1 Physical consistency | **pass** within scope | PS-01 is owned at run time by `pse-quantity` (dimension vectors, bases, reference states, gauge/absolute). Authored quantities are not known at Rust compile time, so compile-time dimensions (uom) cannot own them. The extrapolation policy (PS-02) is enforced at parse, but branched on as a string in six places (TD06) | B4 |
| PS-G2, PS-G3 | **not applicable** | No change to well-posedness or numerical-integrity contracts; TD05 touches diagnostics unscaling only | — |

## 7. Findings

| ID | Finding | Principles / gate / scenario | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| <a id="td01"></a>TD01 | **The store's physical schema has four hand-kept descriptions per table, reconciled only partly.** | DP-01, AP-04 · G1 · S19 | The DDL (`migrations/20260927000000_operational_store.sql`, `…120000_registry_relations.sql`: 411 lines, 74 CHECKs, registry enum lists restated) is hand-written. So are the eight `FromRow` impls (`attempts.rs:123`, `jobs.rs:202`, `solutions.rs:111`, …), the record structs, the column macros (`attempt_columns!`, `solution_columns!`) and `AttemptRecord::row()` (`attempts.rs:164-199`). The conformance test (`conformance_tests.rs:255-328`) compares only part of this. It exempts the 7 catalog tables through a hard-coded list (`:19-27`), and filters out multi-column CHECKs (`cardinality(conkey) = 1`). `no_shadow_structs` is name-based | S19 is six edits, two of them compared. Multi-column rules and catalog tables can drift with no signal. Blueprint §4.2's "nothing hand-written may restate a generated row shape" is violated undetected | The registry declares all `pse_ops` tables, invariants, identities and enums. A `postgres` generator emits the DDL. Cornucopia generates the row types from the schema. Migrations are replaced by fingerprint-checked creation while contents are regenerable | No `migrations/` directory, `FromRow` impl or column macro remains in `pse-operations`. A registry column change reaches DDL, rows and queries by `just codegen` alone |
| <a id="td02"></a>TD02 | **SQL statements are checked only by executing them.** | DP-03, DP-15, AP-05 · G3 · S20 | 68 production statements with 165 positional binds (`attempts` 36, `catalog` 37, `streams` 33, …). 12 enum literals in 9 statements (`jobs.rs:518-526, 663-664`; `attempts.rs:540, 628`; `catalog.rs:603, 676`), plus partial-index predicates. Probe: against a text CHECK column, a misspelled literal prepares and returns zero rows | A misspelled state in a claim or sweep silently matches nothing: jobs are never claimed, attempts never go stale. Swapping two same-typed binds writes wrong rows without rejection | ENUM types generated from the registry (literals fail at `PREPARE`, *Measured*). Cornucopia prepares every statement against the generated schema at generation. Named parameters | Generation fails for a query with a misspelled column or literal (a negative control in the `xtask` generator tests). No `.bind(` remains |
| <a id="td03"></a>TD03 | **Stored codes and outcomes are strings, and SQLSTATE is classified by string.** | DP-02, DP-21 · G2, G3 | `Termination.code: String` (`attempts.rs:40-45`) holds native termination spellings, solve states, `"unattempted"`, `"runtime.cancelled"`, `"runtime.infrastructure"` and diagnostic codes with `::`→`.` (`durable.rs:520-640`). Retention phase and settlement outcome are literals (`catalog.rs:189, 450-451, 516, 657-659`) with no registry enum. The publication outcome is declared in Rust (`pse-catalog` `delta/ticket.rs:28-36`), SQL and Python (`_runs.py:65-89`). `from_sqlx` matches code strings (`error.rs:201-239`) and maps 23514 (CHECK) and 23503 (FK) to `Internal` | A tool cannot classify a termination without parsing codes. An invariant violation reports as an internal defect | Registry enums `TerminationCode` (a termination class plus the source vocabulary's typed value), `RetentionPhase` and `SettlementOutcome`, as ENUM types. Typed `SqlState` classification, with 23514/23503 → a typed `InvariantViolation` | No string literal of these vocabularies in non-test `pse-operations` or `pse-runtime` code |
| <a id="td04"></a>TD04 | **Workflow and operational identities share one untyped `SemanticId`.** | DP-02, DP-04 · G2 · S22 | 277 hand-written `*_id: SemanticId` fields, and generated rows type every `*_id` as `SemanticId`. 53 functions take ≥2 `SemanticId` (pse-modeling 31, pse-runtime 13, pse-compiler 6, pse-operations 3), e.g. `catalog.rs:578 mark_expiring(workspace, publication)`. The typed-id mechanism exists but serves physical-registry ids only (`pse-quantity` `semantic_id_newtype!`, 13 types, ~501 uses) | Cross-entity swaps compile. The entity a foreign key names is carried by field names only | The registry declares an entity identity on each key column, and foreign-key columns inherit it. Typed ids are generated through one macro moved to `pse-ids`. Identity domains in PostgreSQL. Operational and workflow scope first (B3), modeling and compiler next (B7) | `compile_fail` doctests for a swap; no `SemanticId` in `pse-operations` public signatures |
| <a id="td05"></a>TD05 | **Dense index spaces at coordinate boundaries are raw `usize`, and one choice reads a string prefix.** | DP-02, PS-07 · G2 (latent G6) · S22 | Presolve `Report.columns/rows: Vec<usize>` maps presolved → original (`presolve.rs:102-108`, applied at `pipeline.rs:489-498`), and `dimensions` is a four-`usize` tuple. `conditioning.rs:84 kkt` takes (col, col) Hessian and (row, col) Jacobian triplets of one type. `assembly.rs:55-58` holds instance-local coordinates beside global columns. `transport.rs:529-531` picks row or variable nominals by key prefix `"row_"`/`"column_cost_"` (`highs/diagnostics.rs:290, 462-467`). `groups[demand as usize]` relies on push order (`assembly.rs:272-273, 700-741`) | An index from one space used in another yields plausible wrong numbers: mapped columns after presolve, unscaled ranging. F24 of the solver review showed the same conflation at the stamp level | typed-index-collections `TiVec`/`TiSlice` with a newtype per space at these boundaries; typed triplets; a typed ranging key; enum-map for enum-indexed storage. FFI and faer interiors keep `usize`/`i32`, converted at the adapter | Cross-space indexing fails to compile; the prefix match is gone |
| <a id="td06"></a>TD06 | **Decision vocabularies live outside the registry, and twelve vocabularies have two Rust types.** | DP-01, DP-02, PS-02 · G1 | ADR-0113 Outcome 2 names registry enums, but `IpoptLinearSolver` (`solve.rs:551`), `MumpsOrdering`/`SpralOrdering` (`ipopt/settings.rs:127-142`), `HessianMode` and `ReusePolicy` (`solve.rs:50,72`) are hand-written serde enums. `SolveSettings` takes six vocabularies as `str` (`pse-py` `settings.rs:73-155`). `AnnotationValue::Valid { policy: String }` (`annotation.rs:45`) is parsed at `:180` but branched on `"reject"`/`"extrapolate"` in six places (compiler `executable.rs:176`; runtime `results.rs:492-509`, `conformance.rs:766, 1308`, `dynamics.rs:1376`). Twelve source-owned vocabularies get a second generated enum (e.g. `DiagnosticCode`: hand-written 164 uses, generated 0) | The spec and the code disagree. Python cannot type-check the vocabularies. A new policy member means six string edits | Registry enums for these vocabularies, generated for Rust, Python and PostgreSQL. `ExtrapolationPolicy` typed in the kernel. The generator emits `pub use` of a source-owned vocabulary, as it already does for 16 quantity enums | No such literal comparisons remain; `_native.pyi` shows enum types |
| <a id="td07"></a>TD07 | **Durable boundary documents have no declared schema and no canonical identity.** | DP-04, DP-24, DP-21 · G1 · S21, S23 | The public `enqueue(payload_version, payload: serde_json::Value, …)` hashes `payload.to_string()` into the request identity (`worker.rs:279-291`). serde_json's `preserve_order` is enabled by feature unification from DataFusion, so the identity depends on key insertion order and on the build graph. The typed `ModelingJob` (`worker.rs:120-132`) is not the enqueue type. Version dispatch is one equality check (`:426`). The termination detail is `json!` literals with no version (`durable.rs:546, 569, 619`), though the registry calls it versioned. The source manifest is unversioned (`worker.rs:214`). The settings document is `settings: Value` (`execution.rs:549-558`). `BackendSettings(**fields: object)` is opaque to Python type checkers | Identical requests can get different identities. Documents change with no compatibility artifact. Python learns of invalid settings only inside Rust | Typed, versioned documents with serde + schemars 1.x; generated JSON Schemas; generated Python msgspec types; the typed enqueue; the request identity framed from the typed value | Identity test independent of key order; `enqueue` has no `Value` parameter; schemas under `docs/generated/schema/` |
| <a id="td08"></a>TD08 | **Hash-frame contexts are literals at 90 call sites.** | DP-04, DP-24 · G1 | `FramedHasher::new(&'static str)` (`derive.rs:104`); 119 distinct `"pse.*.vN"` strings; only three constants. Execution-packet item 13 records a key that changed under an unchanged frame version | A frame bump is a scattered edit; frames are neither discoverable nor guaranteed unique | One `Frame` enum in `pse-ids`: `FramedHasher::new(Frame::…)`, spellings unique by a unit test, listed in the generated docs | `FramedHasher::new("` has no callers |
| <a id="td09"></a>TD09 | **Operational timestamps claim nanosecond precision, and text and jsonb are conflated.** | DP-02 · G2 | The registry declares nanosecond timestamps (`catalog/operations.rs:38-40`); PostgreSQL stores microseconds. The conformance test accepts `text` or `jsonb` for Utf8 | Published operational rows imply precision that was never recorded | Microsecond timestamps for store-backed relations; a JSON-document logical type mapped to jsonb | Generated DDL and rows agree by construction |
| <a id="td10"></a>TD10 | **A generated file is not protected.** | DP-01 | `python/pse/_native.pyi` is generated by `just python-stubs` but is missing from AGENTS.md prime directive 2, `scripts/agent-hooks.py:83-84` and the `.claude/settings.json` deny rules | A hand edit is not blocked | Add it, and the new generated paths of B1 and B2, to all three | The hook refuses an edit |
| <a id="td11"></a>TD11 | **The authoring JSON Schema has a bespoke emitter.** | DP-13 · G8 | `pse-codegen` `codegen/jsonschema.rs` (301 lines, 117 `$defs`); nothing reads its output. With schemars adopted for Rust-owned documents (TD07) there would be two emitters | Two JSON Schema mechanisms | Derive `JsonSchema` on the generated authoring document structs, with registry facets emitted as `#[schemars(...)]` attributes, and delete the emitter if the output is at least as precise; otherwise record why the emitter stays | One emitter |
| <a id="td12"></a>TD12 | **Store documentation is stale.** | DP-24 | Architecture §9.3 lists `jobs.claimed_by`, progress `values jsonb` and solution `payload bytea`. §9.8 and ADR-0112 Outcome 11 say `pse-operations` depends on `pse-schema` (dev-only) and `pse-engine` (absent). ADR-0112 calls clorinde 2.0 current (archived 2026-05-21, merged into Cornucopia 1.0) | O7–O9 designed against the wrong shapes | Architecture companion corrected in this change; ADR-0114 supersedes ADR-0112 | — |

**Strengths that shape the correction.**
- The pure lifecycle transition table (`lifecycle.rs`) with exhaustive tests.
- Registry enums as the one spelling source, with typed decode failures (`codec.rs`).
- The registry's regeneration discipline (ADR-0051).
- The existing typed-id macro and `closed_enum!`.
- Typed backend settings with `deny_unknown_fields`, and admission with typed reasons.
- Exhaustive-match mirrors (`ProgressValue` ↔ `Metric`; `SeedVectors` ↔ `WarmPayload`).
- The `Ephemeral` durability class.

## 8. Library fit and ownership cost

| Capability / contract | Integration owner / exposed types | Candidate or current mechanism | Fit and limits | Coupling, lifecycle, test, upgrade/replacement cost | Bespoke code removed / recommendation |
|---|---|---|---|---|---|
| Typed statements against the schema | `pse-operations`; generated crate `pse-operations-queries` | **Cornucopia 1.0.1** (2026-08-14; Clorinde merged back in 1.0.0, 2026-05-20). Alternative: sqlx 0.9 `query!` with `.sqlx` | Cornucopia generates plain Rust over tokio-postgres with **no Cornucopia runtime dependency**. It offers named parameters, reusable row and parameter structs, `types.mapping` by PostgreSQL type (domains included), `use-workspace-deps`, and a library API (`gen_fresh`). **Limits:** nullability must be annotated (no inference, issue #226); no per-column mapping; no check mode (regenerate and diff); generation needs a server | Generated source committed and checked by our own regeneration, under the workspace lockfile. If upstream stops, the generated code still compiles. sqlx's alternative is opaque `.sqlx` metadata, a second check mechanism (`prepare --check` needs a live database, prints no diff, and sqlx-cli can no longer be installed `--locked`), positional binds and `sqlx::Type` impls on semantic crates | Removes `FromRow`, the codec parse helpers, column macros and positional binds. **Adopt** |
| Driver, pool, errors, bulk | `pse-operations` | **tokio-postgres 0.7.18**, **deadpool-postgres 0.14.2**; sqlx 0.9 today | Typed `SqlState`; typed binary COPY in and out; pipelining; Unix socket with peer auth (OS user via whoami). Limits: no reconnecting listener; per-test databases are ours | Bespoke parts: a listener task (small) and the existing `testing.rs` harness. **Maintainer: minor** | Removes string SQLSTATE matching and the `UNNEST` batches. **Adopt**; **remove sqlx** |
| Value mapping | `pse-ids`, `pse-model` (feature `postgres`) | **postgres-types 0.2.14** | Derives for ENUM by name, domains, transparent newtypes and composites. Light (`bytes`, `postgres-protocol`, `fallible-iterator`), with no driver | A feature-gated value-protocol dependency on semantic crates, as their serde derives already are. The orphan rule requires impls in the defining crates | **Adopt**, generated |
| TLS | `pse-operations` | **tokio-postgres-rustls 0.14.0** (rustls 0.23, `ring`) | Needed for remote stores (R-10 decided by ADR-0112) | — | **Adopt** |
| Migrations | — | refinery 0.9.2; sqlx migrate today | Contents are regenerable, so a fingerprint-checked create-or-refuse replaces migrations | — | **Defer** to register row R-35 (versioned migrations when contents must survive a schema change). Remove sqlx migrate |
| Dynamic SQL | — | SeaQuery 1.0.2 (binder now `sea-query-sqlx`) | Store statements are static; dynamic and analytical querying belongs to DataFusion (§3.3.1) | — | **Reject**; revisit if a store statement must be composed at run time |
| ORM / schema-from-database | — | Diesel 2.3 `print-schema`; SeaORM 2.0 entities | Makes the database or an ORM DSL the schema authority; the registry is the authority (D1) | — | **Reject** |
| Vectors / in-server code | — | pgvector 0.4.2; pgrx 0.19.3 | No vector data; no in-server computation | — | **Reject**; revisit on vector data, or an operation that must run inside PostgreSQL |
| Container databases | — | testcontainers 0.28 / -modules 0.15 (still needs 0.27) | The local PostgreSQL 18 server is required for generation anyway | — | **Reject**; revisit for an environment without a server |
| JSON Schema for Rust-owned documents | `pse-backend-native`, `pse-runtime`; generator | **schemars 1.2.2** (2020-12, serde-aware, syn 3) | Fits settings, job payload, termination detail and manifest; may replace the bespoke authoring emitter (TD11) | Derive on owned types | **Adopt** |
| Python types for those documents | `python/pse/contracts/` (generated) | **datamodel-code-generator** (msgspec `Struct` output; `--disable-timestamp` for deterministic bytes) or the `pse-codegen` Python target | msgspec is the package's JSON library. Governance needs a msgspec branch for the `Any` lint | Pinned in `[dependency-groups]`, run by `xtask` | **Adopt**; the choice is made at B5 start on output determinism |
| JSON Schema → Rust | — | typify 0.8 | No externally owned schema; still on schemars 0.8 internally | — | **Reject**; revisit when an external standard owns a schema |
| Validated scalars | Settings types | **nutype 0.8.0** | "Cannot construct invalid"; `finite` floats; typed error enums. Limits: its schemars support is 0.8 only; `derive_unchecked(schemars::JsonSchema)` with attribute passthrough must be confirmed | Proc macro | **Adopt** for single-value setting domains if the schemars-1 path works; otherwise hand-written `serde(try_from)` newtypes (≤ 6 types). Cross-field and environment rules stay in `admit_settings` |
| Struct validation | — | garde 0.23 | Cross-field rules are admission rules with typed reasons (DP-21); garde's errors are path and message strings | — | **Reject**; revisit if declarative constraint sets without typed reasons appear |
| Compile-time units | — | uom 0.38 | Dimensions fixed at compile time (issue #276); authored quantities are known only at run time; `pse-quantity` owns PS-01 with bases, reference states and gauge/absolute, which dimension algebra alone cannot express | — | **Reject**; revisit for a Rust-coded kernel library with fixed-dimension APIs |
| Typed dense indices | `pse-math`, `pse-backend-native` boundaries | **typed-index-collections 3.5.0** | `TiVec<K,V>`, `TiSlice<K,V>`, serde | Low | **Adopt** at coordinate boundaries (TD05). index_vec rejected (dormant since 2024) |
| Enum-indexed storage | Assembly and execution | **enum-map 3.1.0** | Arrays keyed by a fieldless enum | Low | **Adopt** at the named sites |
| Generational handles | — | slotmap 1.1 | Salsa owns authoring identities; no mutable object graph in Rust | — | **Reject** |
| Builders | — | bon 3.10 | Typestate builders. 80 functions take ≥7 parameters (24 `allow(too_many_arguments)`) | — | **Admissible, not scheduled.** Use it when a packet reshapes such a function into a request type |
| Boilerplate derives, serde adapters, closed-enum derives | — | derive_more 2.1; serde_with 3.24; strum 0.28 | Existing macros already generate `From`/`Display`; no serde-adapter need found; strum stays inside `pse-quantity` | — | **Not adopted** |
| Schema-first RPC | — | prost-build 0.14 | No protobuf boundary | — | **Reject**; revisit for a cross-language RPC boundary |
| Operational tables in DataFusion | O9 | ADBC 0.24 or generated Arrow builders | `pse-relations` already generates Arrow builders for every operational relation | — | Unchanged: O9 decides, now with the generated builders as the incumbent |

**External reviews, corrected.**
- Cornucopia 1.0 did absorb Clorinde; `cornucopia_async`/`_sync` are deprecated stubs.
- Cornucopia cannot target sqlx.
- The SeaQuery sqlx binder is `sea-query-sqlx` 0.9.
- sqlx 0.9 has per-column overrides in `sqlx.toml` (feature `sqlx-toml`).
- `cargo sqlx prepare --check` is not offline-capable (issue #4297).
- testcontainers-modules lags testcontainers 0.28.

## 9. Alternatives and tradeoffs

| Alternative | Scenarios served / change locality | Contracts, composition and test isolation | Meaning or machinery carried | Correctness / operational cost | Selection and revisit condition |
|---|---|---|---|---|---|
| Current baseline | S19: six hand edits; S20: runtime only | Repository API good; decoding needs a database | Four descriptions per table; hand-written migrations | Silent zero-row literals; positional binds | Rejected (TD01–TD03) |
| A. sqlx compile-time macros + registry-generated DDL and `sqlx::Type` impls | S19, S20 at build | Keeps sqlx's listener, migrations and test harness | `.sqlx` metadata beside the generated trees; `sqlx::Type` on `pse-ids`/`pse-model` pulls sqlx-core into semantic crates, or wrappers | Positional binds; string SQLSTATE; raw COPY; `prepare --check` needs a database, has no diff and cannot be installed `--locked`; better nullability inference | Viable. Not selected: the maintainer rates its conveniences minor, and B wins on generated source, named parameters, typed errors, typed COPY and lighter coupling |
| **B. Registry-generated schema + Cornucopia queries on tokio-postgres (selected)** | S19–S25 | Generated query crate behind the repository API; pure value-mapping tests | One regeneration discipline (ADR-0051) for DDL, rows and queries; two small bespoke parts (listener, test harness) | Named parameters; ENUM literals checked at `PREPARE`; typed SQLSTATE; binary COPY. Nullability annotated by hand (mitigated: registry-generated row declarations for whole-row statements, and every statement exercised by a store test) | **Selected.** Revisit if Cornucopia generation cannot run under the workspace lockfile, or if manual nullability causes a defect that store tests missed |
| C. Diesel `print-schema` or SeaORM entities | S19 generated from the database | ORM DSL | The database becomes the schema authority, a second authority beside the registry | — | Rejected (DP-01) |
| D. Simplest: keep sqlx runtime queries, widen the conformance test, add a literal lint | Partial S19 | Unchanged | Still four descriptions; a new lint | Does not check statements | Rejected. It re-expresses meaning, and new alignment lints are not wanted |

Why a new crate: Cornucopia's generated code uses crate-relative paths and its own
`GenericClient` glue. The new crate is fully generated and sits behind `pse-operations`,
so the seam is the generator's unit, not a new responsibility. Why feature-gated impls in
semantic crates: the orphan rule places `ToSql`/`FromSql` there. The dependency is the
value protocol (postgres-types), not a store or driver, so DP-17 holds, as it does for the
serde derives those crates already carry.

## 10. Verification

| Claim / scenario / risk | Evidence label | Reasoning, test or measurement | Conditions and expected result | Result or gap |
|---|---|---|---|---|
| ENUM literals are checked at `PREPARE`; text-domain literals are not | *Measured* | psql probe in rolled-back transactions | PostgreSQL 18.6, `18/main`, 2026-09-28 | ENUM: `invalid input value for enum` at `PREPARE`. Domain: prepared, zero rows; failed only on `INSERT` |
| Cornucopia generates a standalone crate, maps domains to user types, inherits workspace pins, exposes `gen_fresh` | *Interface-checked* | Source at `b078d22` and the docs | — | Confirmed. Nullability inference absent (#226) |
| tokio-postgres typed SQLSTATE, binary COPY, pipelining; no reconnection | *Interface-checked* | Source at `1084ca8`; issue #802 | — | Confirmed |
| Governance resolves postponed annotations | *Tested* | `just py-unit-native python/pse/tests/test_any_lint.py` with `PSE_SOLVER_IMAGE` set to the local image | Python 3.14.7, 2026-09-28 | 12 passed, 0 failed (baseline zero), including `test_postponed_annotations_are_resolved_before_the_lint` |
| TD01–TD12 current-state facts | Source-traced (*Implemented* paths read) | Surveys plus the reviewer's spot checks | `main` at `3b1730d1` | Counts ±10 % where the surveys say so |
| Benefits of B1–B7 | *Proposed* | Packet tests in Plan 22 | — | Open |
| Binary COPY throughput versus `UNNEST` | *Proposed* | Q1 measurement | Progress and incumbent streams at S14/S16 volume | Open |

## 11. Authority changes, exceptions and disposition

| Authority text | Conflict | Required change | Route |
|---|---|---|---|
| ADR-0112 Outcome 10 (enums stored as text; no database enum types), Outcome 11 (sqlx client, runtime-typed queries, `sqlx::migrate!`, `#[sqlx::test]`, dependencies), and its Options rows rejecting Cornucopia and `query!` for being generated | Blocks the selected design | **ADR-0114** (D22-13) supersedes ADR-0112, restating everything that stands and adding the generated schema, the Cornucopia query crate, the tokio-postgres stack, registry entity identities, create-or-refuse schema handling and the R-35 trigger | ADR + design review (adds a crate; changes an accepted decision). This review serves as its review (author review) |
| ADR-0113 Outcome 2 | Implementation divergence (TD06) | Implement as specified; no ADR change | Plan 22 B4 |
| Blueprint §4.1 (declaration kinds), §4.2 (generated trees), §20 and D10 (store representation) | New declaration kind (entity identity); new generated trees | `design:` edits after ADR-0114 is accepted, with a revision row | `PSE_DESIGN_EDIT=1` |
| AGENTS.md prime directive 2; `scripts/agent-hooks.py`; `.claude/settings.json` deny rules | New generated paths (`crates/pse-operations/src/generated/`, `crates/pse-operations-queries/`); `_native.pyi` missing (TD10) | Add them | Tooling, in B1/B2/B4 |
| `docs/dev/operational-store.md`; `db-migrate` recipe | Migrations replaced | `db-reset` (confirmed, destructive), `db-status` reporting the fingerprint | B1 |
| Register | Versioned migrations deferred | **R-35**: trigger "the first operational store or catalog whose contents must survive a schema change" | Register row in D1 |
| AGENTS.md *Invariants* and `.claude/rules/python.md`: ban on `from __future__ import annotations` | Rule without a capability reason (maintainer, 2026-09-28) | **Done in this session**: the ban, and its ast-grep rule `sgrules/no-future-annotations.yml`, removed; test added | — |
| Plan 22 plan, architecture companion and execution packet | Scope and sequencing | Amended in this change (D22-13, D1, B1–B7, S19–S25, §9 and §12, binding decisions 15–17) | Living plan |

No SHOULD deviation or MUST gap is recorded.

**Disposition.** [Plan 22](../../plans/22-solver-capabilities.md#finding-dispositions) owns
the status of TD01–TD12. Recommended owners:
- TD01, TD09 → B1
- TD02, TD03 → B1, B2
- TD04, TD08 → B3 (modeling scope B7)
- TD05 → B6
- TD06 → B4
- TD07, TD11 → B5
- TD10 → B1, B2, B4
- TD12 → D1 (done in the plan documents; ADR-0114 pending)

**Realized (2026-09-28, maintainer approval).** The single ADR-0114 proposed above was
written as three accepted records, each citing this review as its review:
- [ADR-0114](../../adr/0114-typed-operational-store.md): the store; it supersedes ADR-0112.
- [ADR-0115](../../adr/0115-registry-typed-identities-and-vocabularies.md): entity identities, vocabularies and frames.
- [ADR-0116](../../adr/0116-typed-boundary-documents.md): boundary documents; it supersedes ADR-0113.

Register row R-35 was added, and blueprint revision 66 amends the architecture sections.

## 12. Decision

<a id="decision"></a>

**Behavioural and semantic adequacy (current state).** Not adequate:
- **G1 fails:** hand-kept schema descriptions; ADR-0113's divergence; undefined request-identity canonicalization.
- **G2 fails:** string codes, one identity type, precision claims.
- **G3 fails:** a silent zero-row failure mode (*Measured*) and swappable positional binds.
- **G8 fails (minor):** string SQLSTATE; bespoke JSON Schema emitter.
- G4–G7 and the applicable profile gates pass.

**Architectural fitness (current state).** G9 fails. AP-04 and AP-05 are violated; AP-01, AP-02, AP-03 and AP-06 are satisfied.

**Overall.**
- **Revise** the current operational-store typing and the named typing gaps.
- **Accept-scoped** the target in slot 9 at the *Proposed* level, for the boundary in slot 1, with these packet-start checks:
  1. `xtask` calls Cornucopia's `gen_fresh` under the workspace lockfile. The generated manifest either inherits workspace MSRV and edition, or is normalized deterministically by the generator.
  2. Whole-row statements can share registry-generated row declarations across query files. Otherwise, a per-relation generated query file carries them.
  3. nutype's `derive_unchecked(schemars::JsonSchema)` path works with schemars 1.x, or the hand-written fallback is used.
  4. Deterministic Python type generation (datamodel-code-generator or the `pse-codegen` Python target).
- **Excluded:** versioned migrations (R-35), compile-time physical units, ORM or database-owned schemas, and generational handles.

Acceptance closes no implementation work.

| Priority | Change | Findings / scenarios | Acceptance evidence | Disposition owner |
|---|---|---|---|---|
| P1 | ADR-0114 and register row R-35 | TD12 | `just adr-lint` clean; ADR accepted | Plan 22 D1 |
| P2 | Registry owns the store's physical schema; typed identities generated; value mapping | TD01, TD03, TD04, TD09 · S19, S25 | B1 tests | Plan 22 B1 |
| P3 | tokio-postgres + Cornucopia typed queries; sqlx removed | TD02, TD03 · S20, S24 | B2 tests; the store tests on the new stack | Plan 22 B2 |
| P4 | Typed identities in consumers; frame catalog | TD04, TD08 · S22 | B3 tests | Plan 22 B3 |
| P5 | Vocabularies into the registry; one Rust type each | TD06, TD10 | B4 tests | Plan 22 B4 |
| P6 | Typed, schema-published documents; Python settings types; validated scalars | TD07, TD11 · S21, S23 | B5 tests | Plan 22 B5 |
| P7 | Typed index spaces at coordinate boundaries | TD05 · S22 | B6 tests | Plan 22 B6 |
| P8 | Modeling and compiler typed identities | TD04 · S22 | B7 tests | Plan 22 B7 |

O7, O8 and O9 build on B1 and B2, so the catalog, studies and query surface are written once
on the typed stack.

**What would falsify these conclusions.**
- Cornucopia generation cannot run reproducibly under the workspace lockfile.
- A typed-query defect that manual nullability produces escapes the store tests.
- Binary COPY and pipelining show no measurable benefit at S14/S16 volumes, which would weaken the driver case but not the generation case.
- Typed index spaces force conversions inside hot kernels rather than at adapters.

## Addendum: change review for ADR-0117

<a id="addendum-adr-0117"></a>

**Scope.** A change-tier review, conformance purpose, of
[ADR-0117](../../adr/0117-platform-vocabulary-crate.md). The ten `pse-schema` platform
vocabularies move into a new crate, `pse-vocabulary`, beneath `pse-schema` and `pse-model`,
so that the generator can re-export them (ADR-0115 Outcome 3, TD06). Standard Core 3.1.
Author review, 2026-09-28.

**Scenario.** A new platform vocabulary member, or a new vocabulary. It is declared once in
`pse-vocabulary`; the registry reads its `ALL`; `pse-model`, Python and PostgreSQL follow by
regeneration. Today the same change also produces a second Rust enum in `pse-model`, and code
where the copies meet needs conversions.

**Foundations affected.**
- **AP-04 (satisfied by the change):** one Rust type per vocabulary.
- **AP-01 (satisfied):** the vocabularies get an owner that is neither diagnostics nor identity.
- **DP-17 (satisfied):** the semantic ceiling keeps arrow out, and the generator ceiling keeps generated crates out of `pse-codegen`'s closure. Both stay in force, and the new crate joins the semantic roots.

**Findings.** None beyond TD06, which this change resolves for the ten vocabularies.

**Decision.** Accept at the Proposed level. Evidence is settled by Plan 22 B4
(`source_owned_vocabularies_have_one_rust_type`, `just family-check`, `every_crate_registered`).
Disposition: [Plan 22](../../plans/22-solver-capabilities.md#finding-dispositions) TD06, owner B4.
