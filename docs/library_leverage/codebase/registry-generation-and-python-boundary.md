# Registry, generation and declarations

This page covers `pse-schema` (the registry), `pse-codegen`, `pse-relations`, `pse-engine`, `pse-rules`, `pse-diagnostics`, `pse-model`, `pse-columnar` where it meets declarations, and the generated SurrealQL consumed by `pse-operations`. The file name predates the retirement of the Python boundary (commit `600d4df30` deleted `pse-py` and the Python package) and is kept so that links stay stable. Contracts are owned by
[§4 and §6](../../authoritative_design/sections/schema-and-relations.md#section-4),
[§21.5](../../authoritative_design/sections/workflows-and-results.md#section-21-5) and
[§23](../../authoritative_design/sections/operations-and-validation.md#section-23). The baseline is
on the [entry page](README.md). Codegen is not disfavoured; each generation hop is judged on its merits. Mechanisms were read statically and are *Interface-checked*; every value claim is *Proposed*, and nothing here is *Measured* except where a dated probe is cited. Counts come from `rg`, `wc` and two read-only reachability scripts, and they show shape.

**System picture.** The registry catalog is the single declaration for relations, enums, identities, documents and invariants. Codegen renders Rust with `quote`, `syn` and `prettyplease`, JSON Schema through schemars contexts, and SurrealQL, which is concatenated and substituted from Rust raw strings. The standard registry is baked into `pse-schema/src/generated/standard_registry.rs`. `pse-relations` holds an Arrow view, builder and codec module per relation (about 199k generated lines over 215 files). `pse-engine` is a governed DataFusion session. `pse-rules` lowers invariants to DataFusion plans.

Already strong: predicate validators lower declarations to ordinary DataFusion `Expr` and `PhysicalExpr`; physical reconciliation and invariant programs are native `LogicalPlan` work; the extension-type mechanism implements Arrow's `ExtensionType` and registers with DataFusion; the result pipeline uses `FileWriter`, `FileDecoder`, `RowConverter` and Arrow kernels directly; resources use `FairSpillPool` and `TrackConsumersPool`; refusal codes are a closed, declared set. Legitimately custom: framed identity, compatibility and migration semantics, and the strict unknown-field policy.

With the Python reader gone, the premise that every relation must have a uniform Arrow boundary form for a foreign reader no longer holds. Three Arrow forms still earn their keep: durable IPC result blocks with their witnesses, DataFusion relational admission, and `FieldCheckedBatch` as the admission capability. Several generated and engine forms now look like residue of the universal premise ([CBO-153](#cbo-153), [CBO-154](#cbo-154), [CBO-156](#cbo-156)). Prepaid columnar accounting is [CBO-001](cross-cutting-mechanisms.md#cbo-001) and IPC admission is [CBO-005](result-data-paths.md#cbo-005).

**Coverage on this page.** Traced: the `pse-schema` obligations, `field_contract` and `compatibility` modules; `pse-columnar` IPC and owned-buffer code; `pse-engine` entry points, `operation*`, `provider/*`, `resolution/*` and `capture` (caller and trait-implementation reachability); the canonical call modules of `pse-operations` (non-test bodies, long lines truncated); `pse-rules` `program.rs` and `references.rs`. Representative: `pse-schema` builder, products and standard registry; `pse-engine` cache, admission and scalar adapters (heads only, so [CBO-155](#cbo-155) rests on heads); `pse-codegen/src/codegen/surreal*.rs` (heads and sampled output); the diagnostics macro and both `vocabulary!` definitions. Skimmed or unread: `pse-model` generated rows, the `pse-codegen` Rust emitters beyond the registry emitter, `admission/derive.rs`, the generated retention and lifecycle SurrealQL functions, `canonical_staging`, and test modules throughout. Reachability claims are name-based or regex-based and show absence under their assumptions, not proof of non-use.

<a id="cbo-046"></a>

## CBO-046 — The canonical store program: SurrealQL in Rust strings with no semantic toolchain

- **Operation and consumers.** The store's transactional behaviour (admission, fencing, deadlines, retention, study readiness) is a SurrealQL program in the generated `surreal.surql` (about 4.7k lines) and its rollouts. Authors are the `pse-codegen` templates and the `pse-operations` call modules; every workflow caller depends on it.
- **Current mechanism.**
  - **Generation side.** The `surreal*.rs` generators total about 4.5k lines, about 3.8k of them inside `r#"…"#` strings. `.replace` chains fill 16 distinct `__NAME__` placeholders and `/* NAME */` markers in `surreal_execution`, `surreal_projects`, `surreal_research`, `surreal_study_workbench` and `surreal_artifacts`, and the last splices `format!`-built guard fragments. Nothing checks that every placeholder was consumed. `pse-codegen` does not depend on `surrealdb-syn`; parsing happens afterwards in `canonical_maintenance::parse_definitions` (tests and maintenance activation). The `THROW` code check in `surreal.rs::append` is a text scan (`ddl.split("THROW '")`). The preamble `deadline; publication; check_owner` is repeated by hand.
  - **Call side.** About 283 `fn::pse_` occurrences in non-test Rust restate each signature as text plus binds. Results return as `Value` or `Object` and are projected field by field. Most public store methods repeat one skeleton: `admit_activity`, `activity.scope`, `within_clock`, `ensure_writes`, `protected_query` with a string operation name, `.bind` per parameter, `take` or `ResultSlot`, a generated decoder, and `settle_exact` with a hand readback closure. Bind names for batches are rebuilt by `format!`.
  - Arity, parameter types, result shape, referenced fields and `THROW` identities are checked against nothing before a real server runs the code, by a search of `crates/` and `xtask` for `surrealdb_syn` and `surrealdb_sql`. The `xtask` `surrealql_tools` subcommand is parse-only and no recipe or host route to it was found.
  - Refusal codes are declared in `pse-schema/catalog/canonical_refusals.rs` and also spelled in the SQL literals. Inline SQL in `pse-operations` carries about 1,170 `THROW '` occurrences across 21 files, outside the generator check.
- **Why it matters.** A signature or field change means edits in the template, each caller's text, the binds, the projection and possibly the refusal catalog, and failures surface only in server-backed tests. A parse at generation time could reject bad text at the boundary that owns the bytes and could derive facts such as which functions skip the preamble. A declared operation table could generate the call skeleton with the signature (Proposed).
- **Patterns.** LOP-X015, LOP-02, LOP-X003, LOP-03.
- **Evidence.** `surreal.rs` (`relations`, `append`), `surreal_execution.rs` head, heads of `surreal_files`, `surreal_studies` and `surreal_study_workbench`, generated `surreal.surql` execution functions, `canonical_execution.rs` append path, `canonical_script.rs`, `canonical_maintenance.rs` head, `surrealql_tools.rs`, bodies of `canonical_studies`, `canonical_projects`, `canonical_artifacts`, `canonical_results`, `canonical_retention`, `canonical_lifecycle`, `canonical_analyses`, `canonical_project_imports` and `canonical_selection`.
- **Footprint.** Generators and non-test call modules inspected. `canonical_execution` beyond the append path, `knowledge.rs` (`stage_sql` and `activation_sql` built with `format!`) and `canonical_staging` are suspected and unread. The guarded-script split is [CBO-010](persistence-and-transport.md#cbo-010) and the response path is [CBO-066](persistence-and-transport.md#cbo-066).
- **Preservation constraints.** Byte-identical deterministic output and `SCHEMA_DIGEST`; exact `THROW` identity semantics and native `is_thrown` decoding; no coercion of opaque text into records; `surrealdb-syn` and `surrealdb-sql` staying out of the ordinary runtime closure (feature `canonical-maintenance`, ADR-0214) and the WASI closure; generated functions' `PERMISSIONS NONE`; the original-deadline semantic at each point in a transaction. A `DEFINE PARAM` route would add a second runtime representation of constants the registry owns.
- **Counterevidence and uncertainty.**
  - The AST has no type checker, so semantic checking means a local analysis or a generated contract table. Dynamic `object` parameters (`$row`, `$items`) limit how far signatures can describe payloads.
  - A generator or macro over a declarative function table may suffice without AST use. A template engine would change substitution syntax without checking SurrealQL.
  - Real-server tests are independent assurance and are not redundant. Keeping the text in a Rust module does put it under the generator's single output.
- **Status.** Current mechanism.
- **Research questions.**
  - What do the SurrealQL parse and AST types expose, stably, about function signatures and `THROW` expressions at the pinned server version?
  - Does any upstream tool offer typed bindings or schema-aware linting for stored functions?
  - Could one declared operation table generate signature, call skeleton, decoder and refusal inventory?

<a id="cbo-156"></a>

## CBO-156 — Recorded contracts, obligations and invariants: overlapping and unattached routes

- **Operation and consumers.** §4.3 describes durable opening that verifies a recorded witness and directional consumer admission; §4.6 says publication admission consumes declaration-owned keys and visible references through `pse_schema::obligations`. The registry declares 260 `InvariantSpec`s, largely generated unique and foreign-key integrity plus native row checks.
- **Current mechanism.**
  - **Recorded contracts.** `compatibility.rs` plus `compatibility/recorded.rs` (about 760 lines). By name-based reachability, `select_roots`, `admit_exact_write`, `verify_recorded`, `execution_schema` and `durable_execution_schema` are unreached in production. `VerifiedRecordedContract`, `RecordedProvider` and `DomainEnvironment::recorded` are used only by `schema_transform.rs` and tests. Result blocks use `ExpectedContract` and the file schema check in `AdmittedFile::open` instead.
  - **Obligations.** `ObligationTemplates` and `BoundObligation` are unseen from production; `validate/obligations.rs` is unreached for 169 of 192 lines; `EngineSession::bind_obligation*` is unreached. `RelationObligations` is nevertheless compiled and hydrated into the standard registry.
  - **Invariants.** `pse_rules::invariants::run_invariants` has one non-test caller, `xtask/src/codegen/physical.rs`. `pse_rules::references::admit_entities` (pure Rust) is the only production `pse-rules` path. `pse-engine/src/session/obligation.rs` reads `registry().invariants()` for declared-obligation dependencies and receipts. `result_batch::validate_keys` and physical `reconcile_units` check keys and units elsewhere.
- **Why it matters.** The architecture states guarantees through a route that is not the executing one, so a reader looks for key and witness enforcement in the wrong owner. Removing or wiring the unattached routes would also shrink hydrated product volume ([CBO-040](#cbo-040)) and the SQL-string assembly in `obligations.rs`. If the guarantee is wanted, there is a real gap: a durable file read does not compare recorded semantic meaning (Proposed).
- **Patterns.** LOP-X020, LOP-09, LOP-12.
- **Evidence.** `compatibility.rs`, the head of `recorded.rs` and its function list, `obligations.rs`, `validate/obligations.rs` function list, `field_contract.rs`, `pse-rules/src/invariants.rs` and `program.rs`, `native.rs` head, `obligation.rs` around `invariants()`, a reachability script and `rg` over the named crates.
- **Footprint.** How `obligation.rs` compiles its plans (the dependency half only) and whether `pse-operations` interpretation checks discharge the same meaning by another route were not traced.
- **Preservation constraints.** Actual key and reference refusal in publication; immutable result identity; refusal of incompatible history; candidate-scope distinctions (`Affected`, `Required`, `SidecarRelation`); findings as typed diagnostics; negative reads and unknown predicates retaining occurrence keys.
- **Counterevidence and uncertainty.**
  - The canonical store's versioned interpretation fingerprint may have replaced the witness deliberately, which would make §4.3 the stale part. ADR-0164 is proposed and in flux.
  - Tests exercise all of this machinery, so it is unattached, not broken. Integrity may legitimately differ per stage, and `compile_individual` or `bind_declared_queries` may be consumed through a route a literal search missed.
  - The reader's independence from the live registry is deliberate (§4.3), so any typed shared model must not couple recorded interpretation to current declarations.
- **Status.** Current promise (documentation) versus current mechanism (not attached).
- **Research questions.**
  - Which route should be the one executor of declared obligations, and what remains for `pse-rules` if it is not?
  - Does any durable read today need the recorded-meaning comparison?

<a id="cbo-154"></a>

## CBO-154 — Engine SQL, DML and export surfaces have no production door

- **Operation and consumers.** `EngineSession` and `EngineFactory` offer SQL, native commands, `INSERT`, isolated DML, PREPARE/EXECUTE retention, round providers, workspace and role inputs, provider capture, a frozen catalog hierarchy, the `Cache` logical node, obligation binding and `prepare_schema_transform`. Retained results leave the workstation as Arrow IPC (`export_ipc`) or cohort JSON lines; DataFusion is reachable only from Rust.
- **Current mechanism.**
  - **Unreached engine surface.** Production uses a narrow set: `prepare_rule_plan`, `prepare_many`, `execute_group`, `bind_declared_queries`, `candidate`, `candidate_checked`, `relation_plan`, `table_source` and factory plumbing. Production implementations of `Definition` and `Body` are all inside `pse-engine`; `Producer`, `CatalogSource` and `PrivateTableFactory` have test implementations or none. No production code builds a DML plan or the `Cache` node, and `ModelCache` appears never to be filled. Even with trait dispatch over-kept, 155 of 1,233 production functions (3,232 of 19,057 lines) are unreached, including most of `schema_transform.rs`, `roles.rs`, `computed.rs`, `commands/insert.rs` and `mutation.rs`. The registry declares zero `MigrationSpec`s and `declare_migration` is unreached.
  - **Real queries bypass it.** `pse-runtime/src/workflow/study_workbench.rs` builds `SessionContext::new_with_state(relation.session_state())` and aggregates over custom providers; `connected_results.rs` passes `session_state()` to scans; `xtask/src/development/evidence_query.rs` runs `SessionContext::sql` on fixed questions. These inherit configuration but not plan admission, roles or observation.
  - **No query or open-format export.** No SQL entry point exists in `pse-runtime/src` and `pse-project` exposes no query. No Parquet or CSV export was found in the workflow scope.
- **Why it matters.** Admission, the frozen catalog, the cache proof family and observation defend doors no product issues, while the analytic work that exists does not receive those defences. Two outcomes are visible: retire the unused surface, or route real queries through a small governed door, possibly with a read-only SQL or export command. The choice is a product decision (Proposed).
- **Patterns.** LOP-X020, LOP-12, LOP-11.
- **Evidence.** `operation.rs`, `operation/{ports,completion,ownership}.rs` heads, `provider/*`, `resolution.rs`, `capture.rs`, `native.rs`, `commands.rs`, `mutation.rs`, `round.rs`, `schema_transform.rs`, `arrow_stream.rs`, `cache_service/*`, a name-based reachability fixpoint, a full list of trait implementations, `study_workbench.rs`, `connected_results.rs` export path, `evidence_query.rs`.
- **Footprint.** About 8 to 10k lines including `session/cache*`, which name-based reach cannot separate. Every consumer of `session_state()` was not traced. The export absence search covers `pse-runtime/src` only; `pse-host` CLI surfaces were not inspected.
- **Preservation constraints.** Invariant SQL binding and refusal of volatile checks; immutable provider guarantees; spill and pool accounting for real queries; refusal of incompatible history; `Completion`'s no-restart-after-abandonment rule; lineage metadata, `.incomplete` staging and non-overwrite; original usability labels; exact IEEE values and the NaN and signed-zero semantics of any new format; bounded writers.
- **Counterevidence and uncertainty.**
  - Name-based analysis cannot prove liveness. It over-keeps trait methods and under-counts `sql`, `prepare` and `execute`, which stay reached by unrelated identifiers.
  - Tests are consumers of a kind (`pse-runtime/tests/native_repeated_analysis.rs` uses streaming). Some surface may be intended for a future analytic product or for Plan 59; plans were not read.
  - The bypass may be deliberate for providers whose owners already establish their fields. No consumer is recorded as asking for query or export; IPC plus external tools may suffice, and §19.2 says exact Arrow IPC remains the complete data path. Migration machinery waits for the first breaking change, and §4.1 keeps the declaration.
- **Status.** Current mechanism, with an open product question. The query and export door is Proposed.
- **Research questions.**
  - Can a read-only SQL option set and plan-level admission hooks carry admission on any session built from the configured state, in place of wrapping entry points?
  - How could a query or export stay inside the snapshot's protected custody and bounded memory? Compare [CBO-077](result-data-paths.md#cbo-077).

<a id="cbo-040"></a>

## CBO-040 — Registry products baked as Rust, JSONL and IPC

- **Operation and consumers.** `registry()` and `shared_registry()` hydrate the standard `Registry` once per process for every consumer, and `pse-codegen` itself. ADR-0210 records the design and is still `status: proposed`.
- **Current mechanism.**
  - `pse-codegen/src/codegen/rust/registry.rs` serializes `ResolvedRegistryProducts` into `pse-schema/src/generated/standard_registry.rs` (87,873 lines, about 4.75 MB), which ends in `hydrate()`.
  - Each sub-product has one encoding chosen by what its type supports: Rust constructors for model types; `native(start, end)` slices of a 16.8 MB deduplicated canonical-JSON pool (`standard-data.jsonl`) for serde-capable leaves, parsed back to Arrow `Field` at hydration; and fifteen `standard-N.arrow` IPC streams for self-description batches, read by `builder/products.rs::batch`.
  - The generated header explains that array-backed literals "exhausted the compilation memory cap", so the output format follows a build limit.
  - `pse-codegen` re-exports `pse-schema` (`pub use pse_schema::*`), and `pse-schema` compiles `mod generated;` unconditionally, so the generator depends on what it generates. A probe on 2026-10-27 recorded that a model field added together with its emitter breaks `just codegen` (258 × `E0063` in `standard_registry.rs`, nothing regenerated), and that the generated module is about 18 s (about 72%) of a full `pse-schema` build, with peak RSS under 2 GB ([evidence](../../design_review/evidence/library-leverage-registry-build-coupling-2026-10-27/README.md); not re-run at the current baseline). ADR-0210 records 304 ms against 30 ms, bounded by its fixture.
- **Why it matters.** With the Python product gone, an import-latency argument is weaker; build coupling and the size of the generated diff are now the stronger reasons. Hydrated products with no production consumer (`RelationObligations`, [CBO-156](#cbo-156)) could be dropped before any format change. All fields are parsed in processes that may need only a few relations. This is a prompt to measure, not a finding of slowness (Proposed).
- **Patterns.** LOP-X003, LOP-07, LOP-06.
- **Evidence.** `generated/standard_registry.rs` head and tail, `builder/products.rs` (`ResolvedRegistryProducts`, `hydrate`, `batch`, `decode`), `pse-schema/src/lib.rs`, `pse-codegen/src/lib.rs`, the registry emitter, `ls -la generated/`, ADR-0210. Three commits have touched the output since 2026-10-27 without changing its shape.
- **Footprint.** The `B04` timing test in `products.rs` was not read for results. The hydration of obligations and the arena (`resolved_contract.rs::hydrate`) is suspected and unread.
- **Preservation constraints.** Identical fingerprints and closure identities; deterministic bytes for `codegen-check`; fresh runtime owners and no restoration of proofs (§4.6); exact equivalence to source assembly.
- **Counterevidence and uncertainty.**
  - A prior decision chose this for startup reasons, generated Rust is type-checked, and JSON offsets may be the cheapest compile-time workaround. `&'static str` fields in specs may block simple zero-copy formats, and lazy hydration interacts with whole-registry fingerprints.
  - The registry product types derive no serde; only Arrow `Field` is serialized, and `HashMap` ordering of `Field` metadata is already handled on the JSON route by `pse_columnar::native_field::canonical_json`. A blob route must give the model its own serialization contract.
  - A blob needs complete-value admission, not only decoding: some formats ignore trailing input. A constant generated beside a blob can be stale with it, and layout, payload integrity and source-versus-hydrated equivalence are separate guarantees (reassessment, 2026-10-27).
- **Status.** Current mechanism. A single embedded data authority is Proposed.
- **Research questions.**
  - Can the products be serialized deterministically into one embedded blob, or can the generator be decoupled from the generated module first?
  - Does the runtime need documentation prose at all, and could it live apart from execution fields?
  - What does first-use construction cost?

<a id="cbo-153"></a>

## CBO-153 — Arrow projection generated for every relation regardless of consumers

- **Operation and consumers.** Each relation in the catalog receives a `pse-relations/src/generated` module with an `ArrowValue` impl, `RelationRow`, view, builder, `allocation_size` and validation entry points. Consumers are engine admission and result files for scientific and authored relations. The canonical store's rows travel as `pse_model` values to SurrealDB and do not use these forms.
- **Current mechanism.**
  - Generation is uniform. The 90 `runtime/canonical_*.rs` modules account for 52,234 of about 199k generated lines.
  - Seven of the 90 have a production consumer: `canonical_analyses`, `canonical_analysis_edges`, `canonical_analysis_nodes`, `canonical_artifacts`, `canonical_attempts`, `canonical_result_manifests` and `canonical_runs`. All seven are used for their `Builder` by `pse-runtime/src/workflow/{analyses,artifacts,connected_results}.rs`, projecting a fetched row or page of at most 64 rows into a `FieldCheckedBatch`. The other 83 are referenced only by generated code.
  - `generated/facts.rs` generates `decode` and `encode` over every relation (`FactBatch`) as a linear `if` chain over relation ids; its only non-generated user is `pse-engine/tests/typed_collection.rs`.
- **Why it matters.** Each declaration change regenerates forms with no reader, adding compile time and review surface, and the emitter must keep supporting shapes used only by unused modules. Consumer-selected emission could shrink the crate and would make the set of real Arrow boundaries visible (Proposed).
- **Patterns.** LOP-X020, LOP-02, LOP-X003.
- **Evidence.** most of `generated/runtime/solve_metrics.rs`, `generated/facts.rs` head, `pse-codegen/src/codegen/rust/semantic.rs` decode and encode emission, `columnar/collection.rs::encode_rows`, and a regex-and-brace import walker over all crates and xtask that expands nested `use` trees, aliases and wildcard imports.
- **Footprint.** Inspected: the emitted module shape and the consumer walk. `pse-codegen/src/codegen/rust/relation.rs` internals and a per-field census of the non-canonical families are suspected and unread. Whether the seven callers need Arrow at all was not traced.
- **Preservation constraints.** Arrow schemas, encoding fingerprints and `reference.schema_*` facts derive from declarations and stay available for every relation; durable witnesses and `no_shadow_structs` meaning survive. Only the codec and view projection would become selective.
- **Counterevidence and uncertainty.**
  - The walker is not a type resolver; a macro-assembled path would escape it, though none was found for these modules.
  - Uniform generation is cheap for the generator and gives one mental model. The seven consumers show a real pattern, so other canonical relations might gain one. An unused module can serve as declaration-level type checking.
- **Status.** Current mechanism.
- **Research questions.**
  - Which declaration facet or derived consumer list should select emission?
  - Does `FactBatch` need a production door ([CBO-154](#cbo-154)) or should it be test-only?

<a id="cbo-155"></a>

## CBO-155 — Field metadata re-established around DataFusion by a family of adapters

- **Operation and consumers.** Every declared extension or facet must survive planning and execution for admission at outputs and nested fields.
- **Current mechanism.**
  - `session/admission.rs` (`admit_plan`, `restore_semantic_fields*`, `derive.rs`, `FieldAdmissions`) checks and restores fields after optimization.
  - Adapters and UDFs include `scalar/preserve_field.rs`, `retain_metadata.rs`, `list_field.rs`, `list_concat`, `element.rs`, `structure.rs`, `nonnull.rs`, `checked_value`, and `aggregate.rs` with `aggregate/collection.rs` and `selection.rs` wrapping `array_agg`.
  - `field_transfer.rs` matches and swaps ordinary built-ins and states that "only proven gaps at this exact pin install physical adapters". `traversal.rs` adds a scoped plan graph for this evidence.
- **Why it matters.** The family is pin-coupled: each DataFusion upgrade means rechecking the gaps, and each adapter re-delegates every hook (`native_hooks` macros). Value would be a smaller surface, or a clearer statement of which fields need their semantics inside a plan rather than only at a materialized output (Proposed).
- **Patterns.** LOP-10, LOP-03, LOP-11.
- **Evidence.** Heads of `admission.rs`, `field_transfer.rs` and its dispatch, `aggregate.rs`, `scalar.rs`, `preserve_field.rs`, `retain_metadata.rs`, `traversal.rs`. Bodies are representative only.
- **Footprint.** `admission/derive.rs`, `obligation.rs`, `dependencies.rs`, `contract.rs`, `facts.rs` and `output.rs` are suspected and unread. `pse-engine` pins DataFusion 55.1.0 and 55.2.0 is in the local registry.
- **Preservation constraints.** A plan must not carry forged or lost scientific meaning. Restoration only adds missing annotation and never changes meaning (§4.4 directional admission). The one-DataFusion-version invariant.
- **Counterevidence and uncertainty.** Tests at this pin (`ordinary_native_functions_carry_input_fields...`) show the gaps are observed, not assumed. Upstream returns fields through `return_field_from_args`, so a later release may already close some, but which gaps remain was not checked. This may be a strong integration, not a wrong one.
- **Status.** Current mechanism.
- **Research questions.**
  - For each adapter, does the current return-field or field-metadata path, or a later release, preserve child metadata?
  - Which adapters could be offered upstream?

<a id="cbo-045"></a>

## CBO-045 — Two diagnostic-code authorities and macro noise

- **Operation and consumers.** Every public error's §23.2 code, read through `TypedDiagnostic` or `miette::Diagnostic::code`. `pse-kernels/src/lib.rs` records the miette code as an observation `failure_code`, and `pse-worker.rs::report` prints it beside `diagnostic_report`.
- **Current mechanism.**
  - `impl_diagnostic!` (`pse-diagnostics/src/lib.rs`, about 63 uses in 59 files) projects miette's `code()` from `TypedDiagnostic::diagnostic_code`, so the typed code is the authority. Callers repeat `forward(_) { None }, help(_) { None }, related(_) { None }, source(_) { None }`, about 103 `{ None }` blocks.
  - About 15 sites in 12 files derive `miette::Diagnostic` and declare a container code instead. `CanonicalError` has both, and they differ per variant: the container is `runtime::infrastructure` while its typed impl maps `Codec(_)` to `ValidationInvariant`. `square_response::Withheld` carries `pse::native::square_response`, which is not in the vocabulary and has no typed impl. Six checked files have derives and no typed impl. `FlightError` implements miette by hand with string literals.
  - `vocabulary!` is defined twice, in `pse-diagnostics/src/vocabulary.rs` (15 uses) and `pse-vocabulary/src/lib.rs` (10), with different `description`, `try_from = "String"` versus per-variant `serde(rename)`, `ALL` shape and JsonSchema. `pse-vocabulary` depends on `pse-diagnostics`, so the lower crate cannot import the upper macro.
  - `tests/governance/tests/error_taxonomy.rs` accepts a derive or the macro and does not compare the two authorities.
- **Why it matters.** A diagnostic consumer can read two codes for one failure, and the typed code is the architecture's authority. The `{ None }` boilerplate could shrink through macro defaults, a local change independent of the authority question (Proposed).
- **Patterns.** LOP-X007, LOP-02, LOP-09.
- **Evidence.** `pse-diagnostics/src/lib.rs` and both `vocabulary!` heads, `pse-operations/src/canonical.rs`, `square_response.rs`, `pse-kernels/src/lib.rs`, `pse-worker.rs::report`, `error_taxonomy.rs`; site counts by `rg`.
- **Footprint.** Sites by search. The `disposition()` map from `FailureClass` to `BoundaryClass` is a third place that was not re-read.
- **Preservation constraints.** Codes stay the typed `DiagnosticCode`; the facts and children projection; forwarding rules; a §23.2 code on every public error.
- **Counterevidence and uncertainty.**
  - §23.2 permits either route, and `disposition()` is a legitimate separate projection.
  - The derive cannot supply typed facts, so a small local derive is real maintenance and not obviously better than the macro. Some derive types may legitimately sit outside typed projection.
  - Merging the vocabularies may need a small leaf crate (an ADR-level step), and the differing serde routes affect wire form.
- **Status.** Current mechanism.
- **Research questions.**
  - Can a check compare the miette code with the typed code wherever both exist?
  - Could one leaf crate own `vocabulary!` without changing wire forms?

<a id="cbo-042"></a>

## CBO-042 — Document wire envelopes after the foreign reader retired

- **Operation and consumers.** Boundary-document versioning and schema publication for the CLI, host and schemas under `docs/generated/schema` (497 files).
- **Current mechanism.**
  - `pse-runtime/src/boundary_document_wire.rs` (678 lines) holds `Versioned<T, N>`, a `DeserializeSeed` header checker that forwards, and a manual `JsonSchema` implementation with `add_header`. Comments cite schemars 1.2.2 flatten behaviour and serde's flatten plus `deny_unknown_fields` limit; both remain real reasons.
  - The wire table (`boundary_documents.rs`, `for_each_boundary_document!`) counts FlatV1 155, Native 100, FlatV2 3 and ValueV1 1.
  - `pse-codegen/src/codegen/documents.rs` (410 lines) keeps independent input and output contract files through schemars contexts; the earlier name-renaming pass (`directional_names`, `rename_output`) no longer appears by search.
- **Why it matters.** The four wires were justified as historical contracts for a foreign consumer that has retired, and Rust is now the only reader. Fewer distinct wire shapes would mean fewer independent decisions (Proposed).
- **Patterns.** LOP-11, LOP-X020, LOP-10.
- **Evidence.** The head of `boundary_document_wire.rs` and the wire table; `documents.rs`; `pse_model::document::check_required_header`. ADR-0116 and ADR-0151 were not re-read.
- **Footprint.** Inspected: the head and the table only.
- **Preservation constraints.** Compatibility of existing flat wires and persisted forms; valid examples; domain fields named `version` stay legal; the header rule stays in `pse_model::document::check_required_header`.
- **Counterevidence and uncertainty.**
  - The separate header exists to protect domain `version` fields, and the flatten limits still hold.
  - Whether persisted forms pin the 155 FlatV1 entries was not established and needs a persistence check before any simplification.
  - Whether the about 260 exported schemas still have any consumer beyond the Rust owner and the generation that publishes them is open.
- **Status.** Current mechanism.
- **Research questions.**
  - Does any persisted or published form still require a FlatV1 header apart from the Rust reader?
  - Can a serde attribute or derive replace the hand-written header deserializer?

## Examined and not retained

- **Semantic description written as `json!` and read by a separate serde shape (`fingerprint::uncached_semantic_description`).** The reader side is not reached in production (see [CBO-156](#cbo-156)), so a typed shared model would be a minor change. It must also keep the reader independent of the live registry, and the canonical bytes of existing witnesses must not move.
- **Cache stack as over-featured for retention.** No production creation of the `Cache` node was found, so the stack is part of the unattached surface in [CBO-154](#cbo-154), not a live cache that is too large.
- **`pse-ids` framed hashing and the `Frame` catalog.** The custom serde `Serializer` is what makes identity independent of Rust type names and key order, and accumulated versioned frames are the frozen-identity contract. The open question is the hand frames beside it, in [CBO-039](authoring-and-identity.md#cbo-039).
- **`flight.rs` single-flight and `operation/completion.rs`.** Custom on purpose: completion-owned work with last-waiter cancel and poisoning. Drop-to-cancel libraries would release work early.
- **Plan observation (`observation.rs`, `trace.rs`, `graph.rs`, `comparison.rs`).** Uses `PgJsonVisitor`, `displayable(...).one_line()` and native metrics without parsing display text, in a bounded pool with a live consumer, and `datafusion-tracing` already covers spans.
- **`field_contract.rs` exact recursive comparison.** Its header explains why Arrow's `equals_datatype` and DataFusion's adapter do not establish identity.
- **`ext_metadata` hand-spelled strings, `validate_keys`, `extrema`, `data.rs` Parquet decode.** Hash-entering spellings justify avoiding a serializer's ordering; the others already use `RowConverter`, Arrow kernels or the Parquet reader directly.

## Retired opportunities

<a id="cbo-041"></a>

- **CBO-041 — Boundary documents transcoded through JSON, with checks in both languages.** Retired 2026-10-30: `600d4df30` deleted `crates/pse-py` and the Python package, and no `pse.codec` or Python wrapper remains in `git ls-files`, so the second-language check and the Rust-to-Python re-traversal are gone. JSON documents now cross only process and CLI boundaries. The surviving header owners are Rust-only. Successor: [CBO-042](registry-generation-and-python-boundary.md#cbo-042). Last described at [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-041).

<a id="cbo-043"></a>

- **CBO-043 — Two Python model systems and a custom schema-to-msgspec emitter.** Retired 2026-10-30: `600d4df30` deleted the Python side, and `pse-codegen/src/codegen/documents.rs` now publishes JSON schemas only, with no msgspec emitter or attrs row generator. Successor: [CBO-063](developer-tooling.md#cbo-063). Last described at [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-043).

<a id="cbo-044"></a>

- **CBO-044 — Python wrapper layer forwarding native handles accessor by accessor.** Retired 2026-10-30: `600d4df30` deleted the package, and `_modeling.py`, `_runs.py`, `_native.pyi` and `pse-py` are absent from `git ls-files`. Only stale comments remain, for example in `pse-schema/src/model/extension.rs`. Successor: [CBO-065](developer-tooling.md#cbo-065). Last described at [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-044).
