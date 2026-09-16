---
status: proposed
reviewed: 2026-09-14
depth: deep
blueprint_revision: 37
adrs: [ADR-0067, ADR-0065, ADR-0066, ADR-0031, ADR-0051, ADR-0039, ADR-0052, ADR-0053, ADR-0062]
evidence: Interface-checked
---

# Design review: DataFusion-integral semantics as the replacement for scripted checks

## 1. Decision and scope

**Revision note.** This revision replaces the first draft of the same day at the
maintainer's direction. The first draft was wrong in three ways that inverted the
design basis. It treated the repository's own blanket refusals (sealed function
inventory, foreign-provider refusal, read-only plans, closed rule algebra, PK-only
constraints, in-memory-only providers) as settled strengths rather than as bespoke
mechanisms to be measured against what the engine supplies intrinsically. It gated
engine mechanisms behind "a consumer" although blueprint §3.3.1 and ADR-0065's revisit
trigger say exactly that an absent consumer is not a prohibition. And it treated the
code-generation pipeline as a given although charter DM-58 lists code generation among
the machinery that has to justify itself. Each of those is corrected below; the
findings that were right (two live typers, repeated rescans, the derivation bracket)
are kept.

**Design basis for this review** (stated so the verdicts can be checked against it):

- Blueprint §3.3.1 / ADR-0065: "Any and all Arrow and DataFusion features ... are
  available ... An initial implementation subset, absent consumer, optional dependency or
  historical capability-map rejection is not an architectural prohibition. ... Semantic
  boundaries govern placement, not exclusion of entire API families."
- Maintainer direction (2026-09-14): a mechanism the engine provides intrinsically is
  preferred over a bespoke equivalent; a generated static projection is not the same as
  a programmatic one and carries staleness; and the question for each bespoke path is
  its tangible downside at scale (robustness, performance, traceability through logical
  planning, extensibility), not whether it can reach the minimum outcome.
- The charter, unchanged: DM-19/DM-31/DM-48 (recorded selection and dependencies),
  DM-45 (trust is an execution constraint), DM-44 (an extension carries its conformance
  obligation), DM-53/DM-54 (invariants and boundaries are tested). These are placement
  conditions the pivot has to carry, not reasons to withhold a mechanism.

**Decision: Revise the working tree; accept the maximal-native direction with the
placement conditions named in §7.** The blueprint already states most of the placement
rules (§3.3.1, §5.4, §14.2, §18.5, §22.3). The code lags them with blanket refusals, a
static generated substrate that sustains row-at-a-time scripting outside the planner,
bespoke store readers that restrict encodings and force full residency, and
optimizer-facing facts that are never derived from the evidence the system already
has. One gate fails on the working tree (G1, two live typers) and G7 is unresolved
because the blueprint claims placements the code refuses. The "simpler alternative"
the first draft selected was the bespoke-retaining one; §8 retracts it.

**Proposal:** the maintainer's request to characterize how the DataFusion mechanisms
in [`datafusion_semantic_analysis_types_udfs_capability_spec.md`](../../capability-maps/datafusion_semantic_analysis_types_udfs_capability_spec.md)
and [`datafusion_data_model_contracts_capability_spec.md`](../../capability-maps/datafusion_data_model_contracts_capability_spec.md)
can maximally replace hand-written checks, validators, generators and tests, making
relations and behaviors intrinsic to the data structure, its operations and its
execution rather than a separate scripted activity.

**Status:** both maps are **Interface-checked** against DataFusion 55.1.0 / Arrow
59.3.0. The code is **Implemented** on `wave2/semantic-compilation` at base `eebc82e`
plus the uncommitted Plan 05 working tree (622 changed files). Plan 05's checkpoint
records HP00 implemented, HP01 core implemented, HP02/HP04 ongoing, HP03/HP05/HP07
ongoing. This is a mid-pivot implementation.

**Reviewer:** Claude (design-review skill), with two read-only inventory
investigations used as leads; every citation below was read by the reviewer.

**Affected revisions:** blueprint revision 37; ADR-0065 and ADR-0067 (proposed);
Plan 05 (in-progress). No production code, blueprint or ADR is changed here.

**Observable outcome sought:** a compiler in which every relation, derived column,
change, reusable derived relation and policy is a native plan object or catalog object
the engine can inspect, optimize, explain and record; in which the type, quantity, key,
reference and domain contract is established once by the constructor and carried by
the engine; in which optimizer-facing facts (constraints, statistics, functional
dependencies, policy settings) are derived from executed obligations and bound
policies; and in which hand-written code is confined to specialized algorithms behind
complete contracts and to the trust boundary for foreign bytes.

**Baseline:** the load-bearing pieces exist: private `FieldCheckedBatch` and
`PreparedComputation`/`CompletedComputation` capabilities, one native invariant program
compiled from registry declarations, typed extension reconstruction from the registry
table, a DataFusion extension-type registry built from the same table, function-object
identity admission, a read-only `ConfigExtension` with `df_settings` read-back, truthful
`Exact`/`Unsupported` pushdown, a frozen ordered engine profile, and a programmatic
`relation_schema()` that even the generated code delegates to. What is in the way is
enumerated in §3.

**Supported scope and non-goals:** in scope are the construction, admission, catalog,
store-read, preparation, rule-lowering and generation boundaries of `pse-relations`,
`pse-catalog`, `pse-rules`, `pse-schema` and `pse-authoring`, and the test populations
that check them. Out of scope: numerical kernels (`pse-kernels` is an empty boundary),
solving, Python parity, P4–P10 pass bodies beyond their use of the common preparation
route. Publication/CAS protocol is unchanged and not re-inspected.

**Constraints and uncertainty:** several findings are already scheduled by Plan 05
(HP01, HP03, HP04, HP05, HP07) and are reported as *current state with the plan's
correction named*. No tests were run. One measurement was taken (§9).

### Method and coverage

- Read the charter, directive and template; blueprint D1–D14, §3.3.1–§3.3.3, §4.1–§4.6,
  §5.4, §7.6, §14.2–§14.5, §18.5, §20.1, §21.5, §22.2–§22.3, §24; Plan 05 in full;
  ADR-0013, 0025, 0031, 0037, 0048, 0051, 0065, 0067 front matter and decision text;
  register rows R-01, R-02, R-22, R-25; both capability maps in full; the prior
  capability review and its [deployment matrix](full-arrow-datafusion-capability-matrix-2026-09-14.md)
  in full; the wave-1 foundations review §6–§8.
- Read in full or at the cited grain: `pse-relations/src/ext/mod.rs`,
  `validate/{mod,field,values,bundle}.rs`, `columnar.rs`, `cells.rs`, `migrate.rs`,
  `registry_relations.rs`, one generated relation (`generated/authored/case_activations.rs`);
  `pse-catalog/src/session/{admission,preparation,physical_fields,roles,registry,functions,config,output,profile,candidate,computed,mod}.rs`,
  `snapshot_session.rs:53-260,296-450,700-795`, `scalar.rs`, `scalar/nonnull.rs`,
  `contract.rs`, `contract/registry.rs`, `computation.rs`,
  `provider/{table,pushdown,statistics,catalog,schema,list}.rs`, `inspection.rs`,
  `store/verify.rs`, `store/verify/{parquet,compact}.rs` heads, `store/open.rs:459-560`,
  `store/encode.rs` (grep), `store/membership.rs:271-311,376-395`;
  `pse-rules/src/{lib,invariants,validator}.rs`, `invariants/program.rs`,
  `plan/{lower,expr,head}.rs`, `strata/native_input.rs`, `strata/native_input/witness.rs`,
  `strata/constructed.rs:270-300`; `pse-schema/src/model/{invariant,relation,rule_validation,document}.rs`,
  `checks.rs`, `catalog/{invariants,inv}.rs`, `arrow.rs:50-145`;
  `pse-quantity/src/{admission,precondition}.rs`, `infer.rs:240-300`;
  `pse-authoring/src/change_set/exact.rs`, `p1/mod.rs:78-100`;
  `pse-compiler/src/validator.rs:115-155`, `passes/native_rows.rs:112-135`.
- Verified in the pinned DataFusion/Arrow source: `assert_expected_schema` compares
  names and types only (`expr/src/logical_plan/invariants.rs:114-124`);
  `Constraints::new_unverified` checks nothing (`common/src/functional_dependencies.rs:47-55`);
  `BinaryExpr::to_field` builds a metadata-free field (`expr/src/expr_schema.rs:558-572`);
  `Alias` merges metadata (`:518-528`); `ScalarFunctionExpr::evaluate` evaluates all
  children first (`physical-expr/src/scalar_function.rs:236-261`);
  `Analyzer::execute_and_check` runs an ordered observed list (`optimizer/src/analyzer/mod.rs:118-158`);
  `TableProvider::{insert_into,delete_from,update,truncate,merge_into}` exist
  (`session/src/table.rs:341-394`); `FunctionFactory` (`core/src/execution/context/mod.rs:2225`);
  `SessionContext::{register_batch,register_udtf,register_udf,register_catalog,register_table,register_catalog_list}`
  (`:536-2071`); `ExprPlanner`/`RelationPlanner`/`TypePlanner` (`expr/src/planner.rs:156,389,443`);
  `SchemaAdapterFactory` (`datasource/src/schema_adapter.rs:57`),
  `PhysicalExprAdapterFactory` (`physical-expr-adapter/src/schema_rewriter.rs:175`);
  `ArrowFormat`/`ArrowSource` for IPC listing (`datasource-arrow/src/{file_format,source}.rs`);
  `ParquetSource::{with_pushdown_filters,with_reorder_filters,with_enable_page_index,with_bloom_filter_on_read,with_metadata_size_hint}`
  (`datasource-parquet/src/source.rs:361-463`); `ListingTable::with_schema_adapter_factory`
  (`catalog-listing/src/table.rs:316`).
- Surveyed by grep across `crates/`: implementations of `return_field_from_args` (6),
  `AnalyzerRule` (0), `PhysicalOptimizerRule` (1), `preimage`/`evaluate_bounds`/
  `propagate_constraints`/`output_ordering`/`simplify` (0), `TableFunctionImpl`/
  `ViewTable`/`register_udtf` (0), `UserDefinedLogicalNode`/`ExtensionPlanner` (0),
  `mmap` (0), production callers of `validate_bundle` (0), production sites of
  `FieldCheckedBatch::admit`/`validate_batch` (25), `Cell`-path references in
  `pse-compiler` (631) against `LogicalPlanBuilder`/SQL references (171),
  builder-push sites in `pse-authoring` (223), `declare_rule` calls in `pse-schema`
  (31 across 16 files), `RelationDecl::new` (14).
- Counted: generated Rust in `pse-relations/src/generated`: 572 files, 561 relation
  modules, 435,539 lines against 3,789 hand-written lines and a 3,728-line generator;
  `pse-quantity/src/generated` 27,161 lines; `python/pse/contracts` 7,903 lines;
  `docs/generated` 13,591 lines. Tests: 894 functions in 200 files (823 Rust, 71
  Python), classified by the inventory into ten classes (structural conformance 158,
  field propagation 125, relational negative 138, plan admission 28, lifecycle 142,
  numerical 122, rewrite equivalence 22, governance 74, parity 15, other 70). 243
  hand-written check sites outside generated code and tests, in eight buckets.
- **Measured:** one build timing of the generated crate; see §9 for the command and
  conditions.
- **Not inspected:** P4–P10 pass bodies, `pse-compiler/src/driver` beyond its
  `FieldCheckedBatch::admit` site, the store publish/CAS path, `pse-mathir` beyond its
  quantity use, Python parity. **Attacked:** alias metadata forging (refused,
  `admission.rs:540-556`); candidate providers advertising keys (refused,
  `admission.rs:169-175`); caller-supplied `unique_sets` (refused,
  `contract/registry.rs:41-66`); a foreign `TableProvider` (refused, `admission.rs:187-189`);
  a UDF outside the sealed inventory (refused, `functions.rs:28-63`); a `Dml` plan
  (refused for every session, `admission.rs:97-106`); catalog registration on a sealed
  list (silently ignored, `provider/list.rs:22-30`). **Asserted, not attacked:**
  cancellation mid-stream, reservation release on failure, CAS conflict behavior.

## 2. Authority and lifecycle map

| Concept or fact | Semantic type and identity | Authority / owner | Revision boundary | Permitted update path | Derived representations and their liveness |
|---|---|---|---|---|---|
| Relation shape, key, roles, FK, quantity, nullability | `RelationSpec`/`ColumnSpec` (`model/relation.rs:223-262`), authored as Rust builder calls in `pse-schema/src/catalog/*` | Registry, once | Registry fingerprint | Rust edit, recompile `pse-schema` | **Programmatic:** assembled `Registry` object; `relation_schema()`/`field_for()` compute Arrow `Schema`/`Field` at call time (`arrow.rs:56-106`); `reference.schema_*` rows materialize the registry as relations (`registry_relations.rs`). **Static:** 561 generated Rust modules (Row/View/Builder/CellCodec/allocation), generated Python attrs classes, `docs/generated`; all three gated by `codegen-check` (ADR-0031/0051). The generated `schema()` itself delegates to `relation_schema()` (`case_activations.rs:220-223`). |
| Extension types | `ExtensionTypeSpec` table | Registry | `metadata_version` | Declaration edit | Arrow `ExtensionType` impls hand-enumerated by ordinal (`ext/mod.rs:150-160`); DataFusion factories built from the table (`registry.rs:22-58`); formatter |
| Relational invariant and inference rule | `InvariantSpec` → `RuleSpec` with `RulePlan`/`RuleExpr` bodies, authored as Rust (`declare_rule`, 31 calls) | Registry | Rule version | Rust edit, recompile | Native diagnostic union (`invariants/program.rs`); registry-side shape typer (`rule_validation.rs`) — *two typers* (F1); dead row engine `validate/bundle.rs` |
| Rule-plan output type/nullability | **Contested** between `rule_validation::shape` and DataFusion `to_field` + `pse-rules` `Column` contracts | — | — | — | F1 |
| Quantity contract of a computed column | *No route* (alias-introduced keys refused; no UDF calls `pse_quantity::infer`) | Unresolved | — | — | F3 |
| Catalog of relations | `SnapshotCatalogList` → `SnapshotCatalog` → `SnapshotSchema` over admitted `RelationTable`s (`provider/{list,catalog,schema}.rs`) | Session factory | Session | Sealed; `register_catalog` silently returns `None` | `information_schema` enabled but consumed only by `read_back_settings` |
| Store artifacts | IPC file per relation, optional Parquet, under `object_store` paths (§20.1) | Publication | Manifest checksum | `PutMode::Create` | Read by bespoke IPC framing parser and Parquet footer/Thrift preflight (`store/verify*`), decoded eagerly for every member at open (`open.rs:471-560`) |
| Function implementations | `Arc<ScalarUDF>` pointer set frozen pre-seal (`functions.rs`) | Session factory | Session | Refuses rebinding (`snapshot_session.rs:319-345`) | Plan admission walks pointers; name inventory hashed |
| Engine profile | Ordered native rule lists + settings hash (`profile.rs`, `config.rs`) | Session factory | Profile hash | New factory | `df_settings` read-back; the PSE derivation bracket is *excluded* (`profile.rs:19`) |
| Semantic policy | `PseOptions` constants `"four_valued"`, `"[]"` (`config.rs:36-52`) | Nominally the bound contract; actually literals | — | `set` refuses | `df_settings` shows placeholders |
| Field-checked batch, prepared/completed computation | Private capability types (`columnar.rs:45-70`, `preparation.rs:19-52`) | Constructors | Per batch / preparation | `admit` (row scan) or `finish`; `prepare`→`execute` | `into_checked_relation` rescans (F2) |

**Deliberately opaque behavior:** numerical kernels (none exist yet), the parser, the
store I/O, the pinned engine's rewrites. The engine's rewrites are governed by the
frozen profile plus a post-optimizer re-derivation and a physical repair rule; that
governance is F3.

**Identity behavior:** unchanged by the proposal and found sound in the prior review.

## 3. The bespoke boundaries, their origins, and what the engine supplies instead

This table is the core of the revision. Each row is a place where the code either
refuses an engine mechanism or re-implements one. The origin column names the ADR or
blueprint section that established the rule and the purpose it served; the next
column states what the blueprint says *now*; the last two columns state the native
mechanism that serves the same purpose and the tangible downside of keeping the
bespoke path as the codebase scales.

| # | Bespoke boundary in the working tree | Origin and purpose | Blueprint now | Native mechanism serving the purpose | Downside of the bespoke path at scale |
|---|---|---|---|---|---|
| 1 | Any `TableProvider` other than the four PSE types is refused as "foreign" (`admission.rs:187-189`) | ADR-0048 (superseded); §5.4 "only retained provider types"; purpose: a scan must read an admitted owner (DM-45, DM-31) | §3.3.1: providers eligible; matrix row "Custom/listing/external TableProviders: eligible, capture versions or bind immutable sources" | `ViewTable::get_logical_plan()` lets admission recurse into a view's definition, so a view over admitted scans *is* admitted by construction; a `TableFunctionImpl` returning such a view is the same; a genuinely external provider (`ListingTable` over foreign files) is a declared observation input (DM-13, DM-28) with its version captured, not a refusal | **Traceability/extensibility:** every reusable derived relation (§14.5 closure report, §15.6 reports, per-run diagnostics) must be a Rust function; none is discoverable in the catalog, composable in a user query, or optimized with its consumer. Every parameterized relation family (template instantiation over a domain product, P5 port members, P7 expansion) is a Rust loop rather than a table function whose output plan the engine can push into. |
| 2 | `Dml`/`Ddl`/`Copy`/`Statement` refused for every session (`admission.rs:97-106`) | ADR-0013 (superseded) D10 four roles; D13 cases/results never mutate the model; purpose: immutable snapshots | §5.4: "DML, DDL, MERGE, defaults, Copy and Statement remain eligible for private authoring/import/attempt work with declared effects ... Governance checks these boundaries rather than forbidding native node families." The `no_mutating_providers` governance test is correctly scoped to `RelationTable` only. | A *workspace* provider type implementing `insert_into`/`delete_from`/`update`/`merge_into` (pinned `session/src/table.rs:341-394`); change-set application becomes `LogicalPlan::Dml` over the staged candidate relations with `merge_into` supplying keyed insert/update/delete under one plan; publication stays the separate commit boundary (§22.2) | **Traceability/robustness:** the change-set engine (`pse-authoring/src/change_set/{apply,stage,plans,base}.rs`, `pse-catalog/src/store/changes/*`) implements preimage checks, operation ordering and key semantics in Rust with no plan, no EXPLAIN, no dependency capture at binding, and its own ordering rules to test (`validate_envelope`, `check_change_base`, …). A native DML plan is explainable, its inputs are captured by the same route as every other plan, and its ordering semantics are the engine's, not a second definition. |
| 3 | Function objects outside the sealed pointer inventory refused (`functions.rs:28-63`); rebinding a name refused (`snapshot_session.rs:319-345`) | §14.2 rule 5 function-registry hash in the memo key; §3.3.1 "Registration finishes before a session is sealed"; purpose: reproducibility and recorded selection (DM-19, DM-31, DM-48) | §3.3.1: `FunctionFactory` and planners "can compile or project the one registry/operation authority"; matrix rows 50/57/58: configuration-aware, async and factory functions eligible | The plan enumerates its own functions; `ScalarUDFImpl: DynEq + DynHash` plus the kernel digest of §18.5 give each implementation an identity; the dependency descriptor records what the plan *actually uses*. `PlanOrigin` (already present) selects policy: compiler plans require digest-bearing implementations; analytics plans accept any and record it. Registration through the engine's own `register_udf`/`FunctionFactory` pre-seal; `with_updated_config` for configuration-dependent kernels. | **Extensibility:** a parameterized function (a conversion with captured coefficients, a kernel bound to a parameter set) cannot be constructed at plan-build time; coefficients are forced into literal arguments or into bespoke code. The pointer walk is a second inventory beside the engine's `FunctionRegistry`, and `with_scalar_functions` forks a whole `SessionState` to add one function. |
| 4 | Closed rule algebra: `RulePlan`, 17 `RuleExprOp`s (no arithmetic, CASE, window, set operation), 5 aggregate kinds; registry-side typer `rule_validation.rs` (692 lines); rules authored as Rust (`declare_rule` × 31) | ADR-0048/0053 (superseded); purpose: a bounded, typed, testable rule language | §14.2: "Rule declarations are a frontend to native plans, not a closed runtime interpreter"; §22.3: a new diagnostic check is a `RulePlanSpec` with *no code change*; matrix row 54: compiler SQL eligible as single-source frontend | Rule bodies as SQL text rows in `reference.rule_specs` (data, not Rust), planned by `SqlToRel` with `ExprPlanner`/`RelationPlanner`/`TypePlanner` for PSE constructs (`pse.` extension literals, quantity operators); type checking is `TypeCoercion` against the programmatic catalog at registry load; heads admitted by `declare_relation_output` | **Extensibility/robustness:** each new operator is three coordinated edits (enum, registry typer, lowering); a new check recompiles `pse-schema` and regenerates 435k lines; the blueprint's own litmus test (§22.3, "no code change") is not met; two typers can disagree (F1). |
| 5 | Constraints: PK only; `unique_sets` always empty (`contract/registry.rs:17-35`) | §5.4: Unique only where P2 validates; purpose: never advertise an unverified FD | §5.4 says "`Unique` where P2 validates a `unique` invariant" — the code never populates it | Derive `Constraints` (and thus `FunctionalDependencies`) from the completed invariant program: an `InvariantReport` with zero errors for a `unique` invariant is exactly the evidence; the provider minted from that snapshot advertises it | **Performance/traceability:** the optimizer treats every join on a validated-unique column as many-to-many, cannot eliminate `DISTINCT`/`GROUP BY` over validated keys, and the "total lookup" premise of §4.6 has no native FD to rest on. |
| 6 | Statistics: exact row count only (`provider/statistics.rs`) | §5.4: min/max of key columns Exact from the footer; `statistics_requests` answered selectively | Same section; not implemented | Parquet footer statistics through `ParquetSource`/`ListingTable::with_collect_stat`; for IPC, column min/max/null-count computed once at admission and cached; `ScanArgs::statistics_requests` honored | **Performance:** join ordering and filter selectivity use `default_filter_selectivity`; at compiled-graph scale (equations × symbols × derivations) join order dominates. Unmeasured; a hypothesis with a clear mechanism. |
| 7 | `Exact` pushdown only for key/enum/reference equality, `IN`, `IS NULL` (`pushdown.rs`) | §5.4 differentially tested subset; purpose: never claim exactness the scan does not deliver | Same; matrix: "Parquet row groups/pages, RowFilter: selective reads eligible" | For file-backed sources, `ParquetSource` with `pushdown_filters`/`reorder_filters`/page index/bloom filters uses the engine's `PruningPredicate` for any expressible predicate — no PSE classifier; for in-memory sources the residual filter costs the same as an exact one | **Performance/robustness:** there is no I/O pruning at all today because there is no file-backed provider; the classifier is a second predicate grammar whose every extension needs a `pushdown_vs_unpruned` case. |
| 8 | Store readers: hand-written IPC framing parser (`verify.rs:100-140`), Parquet footer reader restricted to uncompressed PLAIN (`verify/parquet.rs:26-29`), a 267-line Thrift compact-protocol allocation preflight (`verify/compact.rs`), every member decoded eagerly at open (`open.rs:471-560`), no mmap | ADR-0045/0046, DM-45: bounded admission of untrusted bytes before allocation | §5.4: "Arrow IPC files ... hot path, zero-copy mmap"; §20.1: selective reads eligible; matrix: "current import reads/concatenates complete artifact" | `object_store` (already the I/O layer) + `ListingTable` with `ArrowFormat` (IPC) and `ParquetSource` with `metadata_size_hint`, page index, bloom filters, projection and predicate pushdown; `PhysicalExprAdapterFactory` for declared schema evolution; the memory pool accounts operator allocations. Own artifacts written under `PutMode::Create` with a recorded checksum are trusted by provenance; bounded preflight stays for *foreign* bytes only (§10). | **Robustness/performance:** a bespoke Thrift scanner and IPC framing parser are precisely the code the upstream readers already harden; the PLAIN/uncompressed restriction forbids dictionary/RLE encoding and page indexes, inflating artifacts and precluding pruning; residency at open equals snapshot size regardless of the query; every reopen re-verifies the process's own artifacts as if hostile. |
| 9 | `PseOptions` entries are literal placeholders (`config.rs:36-52`) | §14.2 rule 5: policy exposed as `datafusion.pse.*` so it enters `df_settings` and the settings hash | Same section: entries are "generated views of the bound contract values" | Construct `PseOptions` from the session's actual bound policies (case policy relations, §6.10; kernel outcome policies); `with_updated_config` for policy-dependent UDFs; `set` may still refuse arbitrary override | **Reproducibility:** two sessions with different numerical or null policies hash identically; a policy-dependent kernel marked `Immutable` folds under the wrong policy. |
| 10 | Semantic derivation as a bracket around the analyzer/optimizer, plus a physical repair rule (`admission.rs:29-33`, `preparation.rs:114-135`, `physical_fields.rs`) | §4.4 "Candidate field/layout safety is checked before use"; purpose: refuse forged metadata | §14.2 rule 5: the ordered analyzer list is the recorded profile | A named `AnalyzerRule` after `TypeCoercion` inside `EngineRules.analyzers` (hashed, ordered, observed); `ExprPlanner`/`TypePlanner` for construction-time typing of PSE literals and types | **Traceability:** the derivation that changes every plan's fields is outside the recorded profile and the observer; metadata loss by a rewrite is repaired rather than refused or declared (F3). |
| 11 | `register_catalog` on the sealed list returns `None` silently (`provider/list.rs:22-30`); `SchemaProvider` mutation unsupported | §5.4 "registration methods reject calls after the session is sealed" | Same | Register through `SessionContext::{register_catalog,register_table}` *before* seal; after seal, refuse loudly through the platform entry point | **Hidden behavior:** a caller cannot distinguish "registered" from "ignored" (G4, minor). |
| 12 | Discovery through bespoke `TableReader`/Python `_inspection.py`; `information_schema` used only for settings | ADR-0061 immutable inspection | §5.4: catalog queryable by SQL; §4.1: registry stored as `reference.schema_*` so the platform can query its own schema | Expose `reference.schema_*` in the catalog so `information_schema.columns ⋈ reference.schema_columns` answers "what does this column mean" for any tool or agent (DM-55) | **Extensibility/staleness:** a second inspection API to keep in sync with the registry; agents need PSE-specific API knowledge. |

**Absence and uncertainty:** unchanged from the code's four-valued rule contract; the
residual-obligation model still has one capability for "established by construction"
and "established by completed obligation" and none for "unresolved" (F2).

## 4. Derivation and execution design

| Stage | Blueprint target | Working tree | Maximal-native placement |
|---|---|---|---|
| Registry → schema | Programmatic (`relation_schema`) | Programmatic **and** 435k generated lines **and** Python attrs **and** docs, all diff-gated | Programmatic only, plus whichever static projection has a demonstrated consumer (§7 F6) |
| Registry → catalog | `CatalogProviderList` of snapshots (§5.4) | Immutable inventory of admitted `RelationTable`s | Same, plus views/table functions over admitted scans and `reference.schema_*` exposed |
| Raw candidate → field-checked | Physical safety + field contract once | Row×column `Cell` decode (`validate/mod.rs:133-166`) | Physical safety + field contract; local predicates as native findings branch |
| Rule declaration → plan | Frontend to native plans | Closed `RulePlan` enum lowered by `pse-rules` with a parallel typer | SQL text (data) planned by `SqlToRel` + PSE planners against the catalog |
| Analysis | Named PSE analyzer in the frozen list | Bracket + post-optimizer re-derivation + physical repair | Named `AnalyzerRule`; post-optimizer pass asserts |
| Computed quantity | Field-aware function from one authority | No route | Function family delegating to `pse_quantity::infer_with_evidence` |
| Output declaration | Plan projects into the declared relation | `declare_relation_output` | Same |
| Execution → completion | Executor mints completion | `PreparedComputation::execute` | Same |
| Completion → relation | Established facts carried | Full rescan (`preparation.rs:353`) | `from_declared_completion` (F2) |
| Invariants → optimizer facts | Unique where validated; stats from footers | PK only; row count only | Constraints/FDs/statistics derived from `InvariantReport` and footers (F5) |
| Change set | Native DML in a private workspace (§5.4) | Bespoke apply/stage engine | `merge_into`/`insert_into` plans on a workspace provider (F4) |
| Open / reopen | Lazy, pruned, mmap IPC (§5.4, §20.1) | Eager full decode of every member through bespoke readers | `ListingTable`/`ArrowFormat`/`ParquetSource` over `object_store` with statistics and pruning; bounded preflight only for foreign bytes (F7) |
| Kernel batch evaluation | Generated field-aware UDF from `KernelSpec` (§18.5) | None (no kernels) | Runtime-constructed `ScalarUDF` from the `KernelSpec` row + boxed body: the adapter is generic code parameterized by the declaration, not generated Rust |
| Policy | `datafusion.pse.*` views of bound values | Literal placeholders | Bound per session from policy relations |

**Relationship structures, provider selection, boundary contracts, coherent
publication:** unchanged from the first draft where not covered above: field metadata
survives `Alias`, is dropped by `BinaryExpr`, copied-then-stripped by type-only `Cast`,
lost by two pinned physical operators (repaired, §10).

### The generated substrate, measured

The generated Rust is not a side artifact; it is the substrate on which the scripted
path runs.

| Fact | Evidence |
|---|---|
| The runtime schema derivation is already the authority even inside generated code | generated `schema()` calls `pse_schema::arrow::relation_schema` (`case_activations.rs:220-223`); `field_for()` computes every semantic key (`arrow.rs:85-106`) |
| What the generated code adds beyond that | per relation: `Row` struct, `CellCodec`, `ArrowValue`, borrowed `View`, `Builder`, allocation-size arithmetic, `validate`, `check_declaration` (`case_activations.rs`, 560+ lines for a four-column relation) |
| Scale | 561 relation modules, 572 files, 435,539 lines; generator 3,728 lines; Python attrs 7,903 lines; docs 13,591 lines |
| Who consumes the typed API | `pse-compiler`: 631 `Cell`-path references vs 171 plan-builder references; `pse-authoring`: 223 builder-push sites; `pse-rules`: 45 vs 130 |
| Staleness controls the substrate requires | `codegen-check` (ADR-0051), committed generated trees (ADR-0031), `no_shadow_structs`, `GENERATED.sha256` in Python, the "bootstrap starts no engine" constraint of §3.3.3 (which exists only because generation runs upstream of the engine) |
| What the typed API buys | compile-time column-name checks inside hand-written Rust that builds rows — which is the path the pivot exists to delete; plan-time resolution (`col("x")` against the `DFSchema` at preparation, before any data, with an inspectable plan) is the programmatic equivalent |
| What it costs at build time | see §9 (Measured) |

Charter DM-52 permits generating mechanical artifacts; DM-58 requires each generator
to show its leverage. The leverage of the Rust generation is compile-time typing for
row-scripted passes. Once passes are plans (Plan 05's direction; HP03–HP10), the
consumers of `Row`/`View`/`Builder` are the specialized algorithms at a handful of
boundaries, which read columns by name with a typed downcast. The Python attrs classes
are a separate decision: pyarrow `Schema` plus the registered extension types is the
programmatic contract; the attrs projection is justified only if a Python consumer
needs native classes (§21.5 asks for them; that request should be re-examined against
the same staleness cost).

Two further points on "programmatic". First, the registry declarations themselves are
Rust builder calls compiled into `pse-schema`; a package can add *rows* (quantity
types, methods, templates) but not *relations*. The logical endpoint of the
maintainer's direction is relation declarations as data assembled at load, with the
registry fingerprint (already in every manifest) as the version — **Proposed**, outside
both maps, with obligations under DM-51. Second, `SchemaAdapter`/`PhysicalExprAdapter`
are the engine's schema-evolution mechanism at scan time; `migrate.rs` re-implements
evolution as a row interpreter over `Cell`s (HP05 already targets this).

## 5. Representative journeys

### Ordinary extension: a new relation with a quantity column and a foreign key

Today: one `RelationDecl` in Rust; `just codegen` emits roughly 775 lines; `pse-relations`
and every dependent crate recompile; CI diffs the generated tree; the invariants derive
mechanically (`builder/integrity.rs`). Under the maximal-native placement: the same
declaration; the catalog serves the schema at session build; a plan referencing the
new column types at preparation; no generated Rust; `docs/generated/relations` becomes
a query over `reference.schema_*` (or stays generated as a documentation projection
with its own consumer).

### Ordinary extension: a new diagnostic check

Today: a `RuleDecl` in Rust in `pse-schema/src/catalog`, recompile, regenerate. §22.3
promises "no code change". Under the placement: a SQL row in `reference.rule_specs`
whose head is the violating-key projection, planned and type-checked against the
catalog when the registry loads, with PSE planners resolving `pse.` literals and
quantity operators. The registry-side typer disappears with the closed enum (F1).

### Meaningful change: applying a change set

Today: staged rows → `validate_envelope` → bespoke apply → re-admission → P2. Under the
placement: staged relations bound as a workspace provider; one `merge_into` plan per
relation (keyed insert/update/delete with the staged ordinal as ordering); EXPLAIN shows
the change; the plan's inputs are captured by the common route; P2 runs its native
program over the workspace; publication unchanged.

### Boundary: reopening a large compiled snapshot for one diagnostic

Today: every member decoded, verified and admitted at open; residency equals the
snapshot; the diagnostic then scans in memory. Under the placement: a `ListingTable`
per relation over the store's `object_store`; projection and predicate pushdown
prune files/row groups/pages; statistics come from footers; only the diagnostic's
columns are read; the manifest checksum is the trust evidence for the process's own
artifacts.

### Boundary: a filter over a unit-converted quantity

`WHERE T_kelvin > 300` where `T_kelvin = pse_qconvert(T_celsius, …)`: without
`simplify`/`preimage` on the conversion function the predicate sits above the
function and nothing below prunes. With an affine `preimage` returning the half-open
input interval and a retained residual predicate (§14.2.1), the scan prunes. The
soundness obligation is the map's §8: rounding, representability, direction. This is
a consumer that exists as soon as natural-unit conversions are inserted (HP10), not a
speculative hook.

### Interruption or failure: an unregistered function in a compiler plan

Today: refused as "outside the retained implementation inventory". Under the
placement: the same plan in `PlanOrigin::RuleCompiler` is refused with the
reproducibility reason (no digest-bearing identity); in `PlanOrigin::Analytics` it is
accepted and its identity recorded in the dependency descriptor. The refusal keeps
its purpose and loses its blanket.

## 6. Acceptance gates

| Gate | Verdict | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | **Fail** (in-scope; correction scheduled by HP03) | Rule output typing has two live implementations (`rule_validation.rs:23-79` vs `pse-rules/plan/{expr,head}.rs`), neither derived from the other; PK/FK integrity has a native program and a retained, exported, tested row engine (`validate/bundle.rs`; only reference `tests/admission.rs:307`). | F1 |
| G2 — Semantic fidelity | Unresolved | No silent loss found; no route exists to derive a computed quantity; policy values are placeholders, so two policies are indistinguishable to the engine. | F3, F5 |
| G3 — Validity | Pass, inspected scope | Private capabilities; refused forgery; candidates never advertise keys. Removing the blanket refusals (F4) must keep this: views admitted by recursion into their definition; DML only on workspace providers; external providers declared as observations. | F4 placement conditions |
| G4 — Hidden behavior | Pass, inspected scope; two patterns flagged | Preparation does not execute rows; volatility detected; settings read back. Ambient `pse_schema::registry()` in UDF field derivation (`exact.rs:60`); silent no-op `register_catalog` (`list.rs:22-30`). | F9, row 11 |
| G5 — Consistency and recovery | Not applicable to this scope | Publication/CAS unchanged and not re-inspected. | — |
| G6 — Transformation and reuse | Unresolved | Derivation bracket outside the hashed profile; optimizer facts (constraints, statistics, FDs, policy) not derived from established evidence, so rewrites cannot use what the system already knows and the settings hash omits policy. | F3, F5 |
| G7 — Truthful capability claims | **Unresolved** | Blueprint §5.4 claims Unique-where-validated constraints, footer statistics, IPC mmap, DML in private workspaces and Parquet pruning; the code refuses or omits each. The blueprint is normative-forward, so these are *Proposed*, but blanket refusals are the opposite of "eligible". The proposal's test-elimination claim has no route (F8). | F4–F7, F8 |

## 7. Principle findings

| Finding | Principle IDs | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|
| **F1: The same semantic fact has two implementations twice over** — rule typing (registry shape checker vs engine derivation at lowering) and relational integrity (native program vs dead row engine). | DM-02, DM-52, DM-56; G1 | `model/rule_validation.rs:23-79` vs `pse-rules/plan/expr.rs:56-110`, `head.rs:55-90`; `validate/bundle.rs` exported at `validate/mod.rs:20`, only referenced by `tests/admission.rs:307`. | A rule accepted at registry build can be refused at lowering with the wrong error class; every nullability refinement is written twice; the test suite asserts PK/FK semantics the product never runs. | Delete `bundle.rs` and its test now. Replace the closed algebra and its typer with SQL text planned by the engine against the catalog (§3 row 4); what remains registry-side is what only the registry knows (head columns exist, key types, stratification). | Governance grep for the removed symbols; a rule with registry/engine typing disagreement becomes unwritable. |
| **F2: Established facts are re-established by repetition** — row scans at every boundary including after native completion; constructor replay for internal results. | DM-26, DM-37, DM-38, DM-22; §4.6, §14.3.1 | `validate_batch` decodes rows×columns via `cell_at` (`validate/mod.rs:133-166`), called from `FieldCheckedBatch::admit` (`columnar.rs:63`) at 25 production sites including `preparation.rs:353` after `declare_relation_output` already proved the field contract (`output.rs:29-80`), `invariants.rs:119`, `native_state.rs:45`, `witness.rs:302`, `provenance.rs:45`, `p3.rs:204`, `commit.rs:439`; the decode is budgeted separately (`membership.rs:376-379`); `ConstructedRelation::validate` re-runs the constructor and compares by value (`constructed.rs:278-300`). The external-stage replay (`compiler/src/validator.rs:124-152`) is the sanctioned boundary reconstruction, not an instance. | O(rows×cols) decode plus a `Cell` allocation per value plus a reservation per boundary; a full re-execution to prove an internal result; residual predicates fail as row errors after materialization rather than as findings. | `admit_raw` for foreign bytes; `from_declared_completion` for native results whose plan ends in `declare_relation_output` and whose five residual local predicates (`values.rs`, `local_values.rs`) ran as a native findings branch generated from `ExtensionTypeSpec`/`ColumnSpec`; delete the replay once `NativeInput` (`strata/native_input.rs:44-56`) is the sole carrier. | `target_work_accounting`: zero `cell_at` and zero re-executions on native paths; metamorphic differential native-branch vs `validate_cell` on adversarial batches until the row path is deleted. |
| **F3: Semantic derivation is a bracket around the engine, not a stage in it; no route derives a computed quantity.** | DM-06, DM-22, DM-24, DM-31, DM-48; G2, G6 | `derive_native_plan` before `Analyzer` and after `Optimizer` (`admission.rs:29-33`, `preparation.rs:114-135`); `SemanticFields` physical repair (`physical_fields.rs:9-13`); `profile.rs:19` excludes the bracket from the hashed profile; no `impl AnalyzerRule`; `admit_expression` refuses alias-introduced quantity keys (`admission.rs:540-556`); `pse_quantity::infer` has no engine consumer. | Two builds with different derivation logic share a profile hash; metadata loss by a rewrite is repaired, not refused or declared; the first kernel UDF will re-implement quantity inference. | Register `pse.semantic_derivation.v1` as an `AnalyzerRule` after `TypeCoercion` in `EngineRules.analyzers`; post-optimizer pass asserts (refuses) except the two §10 exceptions; one quantity-aware function family whose `return_field_from_args` delegates to `pse_quantity::infer_with_evidence`, with `ExprPlanner`/`TypePlanner` so `T + ΔT` in SQL text lowers to it; the analyzer refuses bare `BinaryExpr` over quantity-tagged fields; `simplify`/`preimage` on the conversion members with the §14.2.1 obligations. | Profile hash changes with the rule version; observer records the rule; `T_col + P_col` with different kinds fails at analysis; `pse_qmul(h, n)` derives energy flow; filter above a conversion prunes with the residual retained and full-result differential equal. |
| **F4: Blanket refusals where the blueprint prescribes placement** — foreign providers, DML, function objects, catalog registration (§3 rows 1, 2, 3, 11). | DM-04, DM-19, DM-28, DM-45, DM-57, DM-58; G7 | `admission.rs:97-106,187-189`; `functions.rs:28-63`; `snapshot_session.rs:319-345`; `list.rs:22-30`; against §3.3.1, §5.4 and matrix rows "Custom/listing/external TableProviders", "DML, MERGE, COPY", "FunctionFactory", "TableFunctionImpl". | Every derived relation family, change application and parameterized function is bespoke Rust invisible to planning, discovery and dependency capture; the purposes the refusals served (owned sources, immutable snapshots, recorded selection) are served by placement, so the refusals buy nothing the placement does not. | Replace each refusal with its placement rule: (1) a scan is admitted if its provider is a PSE owner, or a view/table-function output whose `get_logical_plan()` recurses to admitted scans, or a declared external observation with captured version; (2) `Dml` admitted only when every target is a workspace provider type; (3) function identity recorded per `PlanOrigin` policy instead of pointer-refused; (4) registration pre-seal through the engine APIs, loud refusal after. | Negative tests at each placement boundary (foreign external provider without declaration → refused; DML on `RelationTable` → refused; compiler plan with digest-less function → refused); positive tests for view, table function, workspace merge, analytics closure. |
| **F5: Optimizer-facing facts are not derived from the evidence the system already has** — constraints, statistics, pushdown grammar, policy settings (§3 rows 5, 6, 7, 9). | DM-23, DM-24, DM-31, DM-32, DM-48; G6 | `contract/registry.rs:17-35` (`unique_sets` always empty), `provider/statistics.rs` (row count only), `pushdown.rs` (closed grammar), `config.rs:36-52` (placeholders); against §5.4 and §14.2 rule 5. | The optimizer is blind to validated uniqueness and to value distributions; no I/O pruning exists; two policies hash the same; policy-dependent folding is wrong. | Mint `Constraints`/FDs from `InvariantReport`; compute or read statistics once per admitted artifact; drop the PSE classifier for file-backed sources in favor of the engine's pruning; bind `PseOptions` from policy relations and use `with_updated_config`. | `EXPLAIN` shows `DISTINCT` eliminated over a validated-unique key; join order changes with statistics (Measured); settings hash differs across policies; `pushdown_vs_unpruned` over a `ParquetSource`. |
| **F6: Generated Rust is the substrate of the scripted path** (and the extension bindings are hand-enumerated). | DM-56, DM-58, DM-52, DM-38, DM-50; §22.3 | §4 table above: 435,539 generated lines; typed API consumed by 631 `Cell`-path references in `pse-compiler` and 223 builder-push sites in `pse-authoring`; four staleness controls; rules as Rust; `ext/mod.rs:150-160` ordinal literals. | Every relation change regenerates and recompiles hundreds of thousands of lines; row-scripted passes have no plan, no EXPLAIN, no captured dependencies, no optimization across their boundaries; §22.3's "no code change" promise is unmet; the bootstrap-without-engine constraint exists only to serve generation. | Make the programmatic path the only Rust path: registry → `Schema` at session build; passes as plans; specialized algorithms read `RecordBatch` columns by name at their boundary; keep a static projection only where a consumer is shown (candidates: Python attrs per §21.5, re-examined; documentation). Generate the extension bindings if they stay. Author rules as data (row 4). | Generated-line count and build time (Measured, §9) fall to the retained projections; every remaining generated artifact names its consumer; `no_shadow_structs` retained. |
| **F7: Store readers are bespoke and restrict encodings; open is eager and unpruned** (§3 row 8). | DM-36, DM-37, DM-38, DM-45, DM-58 | `verify.rs:100-140`, `verify/parquet.rs:26-29`, `verify/compact.rs` (267 lines), `open.rs:471-560`, no mmap anywhere; against §5.4 and §20.1. | Hand-maintained Thrift and IPC parsers; artifacts without dictionary/RLE/page index; residency equals snapshot size; own artifacts re-verified as hostile every reopen; no selective reads. | `ListingTable` with `ArrowFormat`/`ParquetSource` over the existing `object_store`, statistics from footers, `PhysicalExprAdapterFactory` for declared evolution, pool-accounted decoding; keep bounded preflight as the declared *foreign-bytes import* path only (§10). | Open time and resident bytes for a large snapshot before/after (Measured); a foreign-bytes fixture still refused at the import boundary; full-result differential between memory and file providers. |
| **F8: "Eliminate most of the test scope" is unbacked; what shrinks is the population whose validators are deleted.** | DM-53, DM-54, DM-59, DM-60 | 894 tests; the inventory places 421 in the structural/field/relational band and 301 in lifecycle/numerical/rewrite/parity; `pushdown_vs_unpruned`, `relational_expansion_p2`, `memo_dependencies`, `invariant_fixtures` (1,200 generated fixtures) are the evidence for the engine surfaces that check nothing (`new_unverified`, name/type-only schema assertion, metadata-free `BinaryExpr`). | Removing the second band leaves the construction rules unverified. | Reframe as a population change: one test per transfer rule; generated conformance suites per contract family (extension types, invariants — already generated, UDF field derivation, providers); differential/metamorphic/lifecycle unchanged; delete implementation-shaped tests with their code. | Every surviving test names its contract; governance check on deleted-symbol references. |
| **F9: Ambient global registry inside function field derivation and generated admission.** | DM-28, DM-02, DM-20; G4 | `exact.rs:60`; generated `validate()`/`try_from_batch` (`case_activations.rs:227-229,327-333`); sessions bind `Arc<Registry>` by pointer. | A session on a fixture registry derives fields through a different authority than admitted its inputs. | Function instances capture `Arc<Registry>` at registration; delete registry-less generated entry points (moot under F6). | Fixture-registry session exercising every UDF's `return_field_from_args` with the global accessor unavailable. |
| **F10: The invariant program re-binds by value what the session owns by pointer.** | DM-26, DM-31 | `run_invariants` calls `validate_bindings` (`snapshot_session.rs:365-391`, `retained != batch`) and iterates a caller map for scope (`program.rs:45-60`). | O(data) equality scan per run; two sources for scope. | Session-only API; scope from `input_keys()`/roles. | No batch equality on the invariant path. |
| **F11: Test code restates declared facts instead of deriving them.** | DM-02, DM-52 | `banned_patterns.rs:35-50` restates `clippy.toml:48-53` paths and reasons; `registry_assembles.rs:245-262` restates all 11 extension storages; `no_mutating_providers.rs:47-55` encodes read-only as seven method-name strings; `conservation_source.rs:40-58` hard-codes eight quantity-type IDs. (The 1,200 invariant fixtures are regenerated from the registry and are not an instance.) | Second hand-maintained copies of facts the configuration or registry owns — the same defect the maintainer wants out of production code. | Parse `clippy.toml`; render `EXTENSION_TYPES`; behavioral test for read-only providers (`insert_into` on `RelationTable` returns the default unsupported error — and *succeeds* on a workspace provider under F4); compute IDs from the loaded package. | Literals removed; a declaration change alters the expectation without a test edit. |

**Applicability:** Groups 1–2, 4–7, 9, 11–12 carry the findings. Group 3 (identity) was
found sound in the prior review and is untouched. Group 8 applies through F5–F7 as
mechanisms with a stated performance hypothesis and a measurement plan; no speedup is
claimed. Group 10 applies through F3(i) and F5 (profile and settings completeness).

### Every mechanism in both maps, placed

No row is "deferred". Each names the bespoke equivalent, its downside at scale, and
the placement condition the charter attaches (an obligation, not a gate).

| Mechanism (map §) | Bespoke equivalent today | Downside at scale | Placement and obligation |
|---|---|---|---|
| Registry → `Field` metadata (S§3, S§5) | — (this is the declaration surface) | — | Keep; the single authority |
| `ExtensionType` + `DFExtensionType` registry (S§3) | Hand-enumerated bindings | Three edits per new type | Generate or construct from the table |
| `return_field_from_args` (S§6) | `pse-rules` `Column` contracts for calls | Second typer | Sole route for computed fields; delegates to `pse_quantity::infer` |
| `AnalyzerRule` (S§2) | Derivation bracket + physical repair | Outside profile; repair not refusal | Named rule after `TypeCoercion`; equivalence statement per rewrite (DM-24) |
| `ExprPlanner`/`RelationPlanner`/`TypePlanner` (D§2.5 via `SqlToRel`) | Closed `RuleExpr`/`RulePlan` enums + registry typer | Three edits per operator; rules as Rust | Rules as SQL data; planners resolve PSE literals/types/quantity operators; the registry keeps only what only it knows |
| `simplify`/`preimage`/intervals/ordering (S§7–§8) | Nothing; filters sit above conversions | No pruning through conversions; no bound reasoning for solver limits | Implement on conversion/affine members with exact half-open/rounding proof and retained residual (§14.2.1); intervals only with a sound enclosure argument; ordering only where strict |
| `is_strict`, `with_updated_config`, `Volatility` (S§6) | `requires_fresh_execution` walk | Policy-dependent folding wrong | Derive from `KernelSpec` propagation policy and bound `PseOptions` |
| Struct-valued kernel UDF, `struct_field_mapping` (S§9) | Nothing (no kernels) | Repeated EOS work across properties | First kernel batch route; mapping only for pass-through fields |
| Aggregate UDF (S§10) | 5 closed aggregate kinds; MathIR `WeightedMean` | Enum growth; no basis-aware reductions in plans | Built-ins first (`array_agg` ORDER BY for `collect_ordered`); UDAF for `WeightedMean` with partial-state and null/empty contracts |
| Window UDF (S§11) | None in rules | No precedence/tie/trajectory windows | Built-ins first; custom evaluator only with frame/peer contracts |
| `TableProvider` full surface (D§1) | Four PSE types; in-memory only; foreign refused | Rows 1, 5, 6, 7, 8 above | Owners, workspaces, views, table functions, external observations; constraints/statistics from evidence |
| `ViewTable` (D§2) | Rust report functions | Not discoverable/composable/optimizable | Admitted by recursion into `get_logical_plan()` |
| `TableFunctionImpl` (D§2.4) | Rust expansion loops (P5/P7) | Plan-invisible expansion | Returns a view over admitted scans; literal selectors; output schema explicit |
| Catalog hierarchy, `register_*` (D§3) | Sealed maps; silent no-op | Hidden behavior | Register pre-seal through engine APIs; refuse loudly after |
| `information_schema` + `reference.schema_*` (D§3.4–§3.5) | Bespoke inspection API | Second surface; agent knowledge | Expose the registry relations; discovery by query (DM-55) |
| `ConfigExtension` (D§4) | Placeholder constants | Policy invisible | Bind from policy relations; keep override refusal |
| `SchemaAdapter`/`PhysicalExprAdapter` (D§1.2, matrix row 52) | `migrate.rs` row interpreter | Materialize old+new; no scan-time evolution | Declared evolution applied at scan |
| `ListingTable`/`ArrowFormat`/`ParquetSource` (D§1, matrix "Sources") | Bespoke readers; eager load | Row 8 above | Own artifacts by provenance; foreign bytes through bounded import |
| DML/MERGE (D§1.7) | Change-set engine | Row 2 above | Workspace providers only; publication unchanged |
| `LogicalPlan::Extension`/`ExtensionPlanner` (S§14.2 r8) | Plan fragments with materialization at algorithm edges | Lost optimization and stitched provenance across every algorithm boundary | Wrap specialized algorithms (SCC, tear, matching) as extension nodes exposing inputs/expressions and pushdown behavior |
| `FunctionFactory` (matrix row 58) | Pre-seal Rust registration only | Kernels/rules can't be declared as source | Frontend to the one `KernelSpec`/rule inventory, pre-seal |
| Async UDF / external sources (matrix rows 56–57) | Not used | — | Declared effects, captured versions; never pure |
| `datafusion-proto`/Substrait/SQL unparser (matrix) | Proto codec present | — | Derived representations; re-admit on decode |
| Arrow kernels, builders, views (matrix "Arrow representations") | Generated typed builders and `Cell` codecs | F2, F6 | Runtime builders from `DataType`; kernels over batches; views by name at algorithm boundaries |

**Optional maturity assessment:** omitted; the gates carry the decision.

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication and extension locality | Correctness and operational risks | Implementation / maintenance cost | Performance evidence | Why selected or rejected |
|---|---|---|---|---|---|
| **Current working tree** | One declaration per relation/invariant (good); two typers; dead engine; three static projections with staleness gates; bespoke change, read and discovery paths | Drift between typers; repair not refusal; blanket refusals contradict the blueprint; optimizer blind | Already paid; ongoing regeneration of 435k lines per schema change; every derived family is new Rust | None measured; residency and rescans are structural | Reject as the end state |
| **Retain the bespoke boundaries, tidy the checks** (the first draft's "simpler alternative") | Keeps every refusal and generator; removes only rescans and one typer | Keeps every downside in §3; every future relation family, change and function stays bespoke | Lower initial delta, unbounded ongoing cost | None | **Rejected and retracted**; it was a bare-minimum reading |
| **Maximal-native with placement conditions** (selected) | One authority per fact; engine derives, records and optimizes; rules and relations as data; static projections only with a consumer | Placement conditions must be enforced (workspace-only DML, declared external sources, digest-bearing functions in compiler plans, sound preimage proofs); the engine's unverified surfaces stay covered by differential tests | Substantial, mostly subtractive in Rust (bundle, typer, replays, change engine, readers, most of the generated tree) plus one analyzer, one function family, planners, workspace provider, listing providers, evidence-derived constraints/statistics | Hypotheses with mechanisms (F5–F7) and a measurement plan; one measurement taken (§9) | Selected: it is what §3.3.1, §5.4, §14.2, §18.5 and §22.3 already say, applied |
| Build a proof DSL / second optimizer | Another meaning system | High | High | None | Reject (unchanged from prior reviews) |

**Abstractions justified by current needs:** the registry (the single declaration
surface; its own relations make it queryable); the private capability types; one
analyzer rule; one quantity function family; PSE planners for SQL rules; a workspace
provider type; listing providers over the existing object store. Each has a consumer
named in §5.

**What remains ordinary code:** quantity algebra, MathIR canonicalization, the
document parser, kernels when they exist, the commit protocol, the bounded preflight
for foreign bytes, the two pinned-engine metadata workarounds.

## 9. Verification and measurement plan

| Claim or risk | Evidence label | Test / analysis / benchmark | Conditions and expected result | Current result or remaining gap |
|---|---|---|---|---|
| Generated crate build cost | **Measured** | `cargo build -p pse-relations --offline` after a warm dependency build, dev profile, this workstation | Wall time for the 435k-line crate alone | See the result recorded below |
| Only validated facts reach the optimizer (F5) | Proposed | `Constraints` minted from `InvariantReport`; `EXPLAIN` shows `DISTINCT` elimination | Candidates still advertise nothing | Not implemented |
| Placement conditions hold after removing refusals (F4) | Proposed | Negative tests per boundary (foreign provider undeclared; DML on `RelationTable`; digest-less function in compiler plan) | Refused with the placement reason | Not implemented |
| View/table-function admission by recursion | Proposed | View over admitted scans accepted; view over an undeclared external scan refused | — | Not implemented |
| Change set as `merge_into` equals the bespoke engine on the current fixtures (differential, kept until deletion) | Proposed | Same inputs, same resulting relations and receipts | — | Not implemented |
| Lazy pruned open (F7) | Hypothesis → Measured | Open time and resident bytes for a large compiled snapshot, memory vs listing providers, recorded settings | Residency proportional to the query, not the snapshot | Unmeasured |
| Statistics change join order (F5) | Hypothesis → Measured | Representative P8/P10 joins with and without footer statistics | Plan and time recorded | Unmeasured |
| Preimage prunes through affine conversion (F3) | Proposed | Full-result differential with pruning disabled; endpoint/rounding cases | Identical results; fewer rows decoded | Not implemented |
| Derivation in the profile (F3) | Proposed | Hash changes with rule version; observer records the rule | — | Not implemented |
| No rescan or replay on native completion (F2) | Proposed | `target_work_accounting` | Zero `cell_at`, zero re-executions | Not implemented |
| Pushdown exactness | Tested | `tests/engine/tests/pushdown_vs_unpruned.rs` | Identical rows/multiplicities/nulls | Retain; extend to file-backed providers |
| Registry/engine typing cannot disagree (F1) | Proposed | Removal | — | Not implemented |
| Test expectations derive from declarations (F11) | Proposed | Rewrite four tests | — | Not implemented |

**Measured, 2026-09-14:** `cargo build -p pse-relations --offline`, dev profile, warm
dependency cache, only `pse-relations` recompiled (the crate is 438,866 lines of which
435,539 are generated), on the maintainer's workstation with the pinned toolchain:

| Quantity | Value |
|---|---|
| Wall time | 39.3 s |
| CPU time (user) | 3 min 32 s |
| CPU time (sys) | 5.5 s |

This is the incremental cost paid by every dependent crate's rebuild whenever any
relation declaration changes. It measures one crate in one profile; it is not a
whole-workspace or CI figure, and no claim is made about the runtime cost of the
generated code.

**Cost accounting:** the proposal changes validation (decode/allocation per boundary),
preparation (one analyzer pass; SQL planning of rules at registry load), open/reopen
(lazy vs eager), build (generated lines), and change application (plan vs engine).
Construction of numerics, publication and recovery are untouched.

## 10. Exceptions and unresolved decisions

**Exception 1 — pinned-engine metadata repair.** Principle IDs: DM-24, DM-42. Scope:
`SingleRowListArrayBuilder` dropping nested child metadata on scalar list expansion;
`CrossJoinExec::build_batch` scalarizing build-side nested values; repaired by
`SemanticFields` (`physical_fields.rs`). Reason: upstream defects at 55.1.0; refusing
would exclude CASE-over-list and Cartesian products. Compensating controls:
`missing_metadata_only` proves the layout is otherwise identical; the rule is a named
`PhysicalOptimizerRule` in the frozen physical list. Evidence: Implemented. Owner:
maintainer. Revisit: DataFusion major bump, or F3a landing.

**Exception 2 — bounded preflight for foreign bytes.** Principle IDs: DM-45, DM-58.
Scope: the Thrift/IPC preflight (`verify/compact.rs`, `verify.rs`) is retained *only*
for the declared foreign-bytes import path, not for the process's own artifacts read
through listing providers. Reason: parquet 59.3 allocates from collection counts
before reading; own artifacts are trusted by the manifest checksum written under
`PutMode::Create`. Compensating controls: the import path is a distinct entry point
with its own tests. Evidence: Implemented (the preflight), Proposed (the split).
Owner: maintainer. Revisit: an upstream metadata size limit in the pinned parquet
reader.

**Unresolved decisions for the author:** (1) which static projections keep a consumer
(Python attrs classes per §21.5; documentation) once Rust generation is withdrawn; (2)
whether relation declarations themselves move from Rust builder calls to data assembled
at load (the endpoint of the programmatic direction, with DM-51 obligations); (3)
whether residual local predicates execute as a findings branch or as checked kernels
(findings recommended for DM-47/DM-08 uniformity).

## 11. Decision and implementation changes

**Decision: Revise.** The maximal-native direction is accepted; it is what the
blueprint already prescribes. The working tree must shed its blanket refusals, its
second typers, its rescans and replays, its bespoke readers and change engine, and the
static generated substrate that keeps passes off the planner, under the placement
conditions named in F4 and the obligations named in the mechanism table.

**Reason:** the strongest evidence is that the programmatic path already exists at
every layer (the runtime schema derivation that even generated code calls, the native
invariant program, the private completion route, the frozen profile with settings
read-back) and that the blueprint's own placement rules are more permissive than the
code. The strongest limitation is that none of the optimizer-facing facts, none of the
derived relation families, and none of the change or read paths use the engine yet.

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 (authority) | Delete `validate/bundle.rs` and its test; retire the closed rule algebra and registry typer in favor of SQL rule text planned by the engine with PSE planners (F1, §3 row 4) | DM-02, DM-52, DM-16 | No second typer; §22.3 "no code change" met for a new check | Governance grep; a new rule fixture authored as data only |
| 1 (placement) | Replace the four blanket refusals with placement rules: views/table functions by recursion, DML on workspace providers only, function identity per `PlanOrigin`, loud post-seal registration refusal (F4) | DM-04, DM-19, DM-28, DM-45 | Negative tests per boundary pass; a view, a table function, a merge and an analytics closure each prepare and execute | The negative tests |
| 1 (derivation) | Named `AnalyzerRule` in the profile; post-optimizer assertion; quantity function family delegating to `pse_quantity::infer`; planners for SQL text (F3) | DM-06, DM-22, DM-24, DM-31 | Profile hash covers the rule; `T + P` refused; `h·n` derived | Profile-hash and observer tests; generated field-derivation conformance |
| 2 (evidence to optimizer) | Constraints/FDs from `InvariantReport`; statistics from footers or one-time admission; policy-bound `PseOptions`; engine pruning for file-backed sources (F5) | DM-23, DM-31, DM-48 | `EXPLAIN` evidence; settings hash differs across policies | `pushdown_vs_unpruned` over `ParquetSource`; hash tests |
| 2 (substrate) | Withdraw Rust generation to the projections with a named consumer; passes as plans; algorithms read batches by name (F6); `from_declared_completion` and native findings branch (F2) | DM-56, DM-58, DM-26, DM-38 | Generated lines and build time fall; zero `cell_at` on native paths | `target_work_accounting`; retained `no_shadow_structs` |
| 2 (store) | Listing providers over the existing object store for own artifacts; bounded preflight confined to foreign import (F7) | DM-36, DM-37, DM-45 | Open/residency measured; foreign fixture still refused | Differential memory vs file providers |
| 3 (verification) | Test population reframed (F8); expectations derived from declarations (F11); session-registry capture in functions (F9); session-only invariant API (F10) | DM-53, DM-54, DM-02, DM-28, DM-26 | Every surviving test names its contract | Governance checks |

**Final check:** the claims match the evidence: every "eligible" in the blueprint and
the matrix is matched here with a placement condition and a named downside of the
bespoke path; no mechanism is withheld for lack of a consumer; the one gate failure
and the two unresolved gates name the decisions the author has to make; and the
performance statements are labeled hypotheses with a measurement plan, except the one
build timing that was actually taken.
