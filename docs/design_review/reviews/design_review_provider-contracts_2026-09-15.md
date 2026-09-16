---
title: DataFusion provider contracts as the governing data architecture
date: 2026-09-15
status: proposed
scope: provider hierarchy, operation policies, execution, publication, and inspection
evidence: Interface-checked; targeted characterization probes, with conditions in section 9
plan: docs/plans/05-native-logical-plan-hard-pivot.md
---

# DataFusion provider contracts as the governing data architecture

## 1. Decision and scope

**Direction:** adopt the DataFusion provider hierarchy as the common framework for
every product data operation. Catalog, schema, table, function/factory, planning,
execution and publication boundaries implement standard policies derived from one
set of declarations. An implementation difficulty is work to complete, not grounds
for retaining an independent pathway. The maintainer stipulated this direction on
2026-09-15, including changing existing policies where they obstruct it.

**Review decision: Revise.** The direction has substantial, concrete architectural
value. The current implementation and the supplied capability map do not yet specify
or enforce the complete contract. The highest-priority changes are a coherent
namespace/binding model, extensible provider admission, and explicit command and
publication semantics. Section 11 supplies dependency-ordered implementation packets.

**Observable outcome:** a relation or operation introduced through a declaration and
its actual implementation becomes consistently discoverable, bindable, executable,
inspectable and subject to the applicable policies through Rust, SQL and Python.
The same input role resolves to the same owner through each interface. A policy
change has one definition, an explicit scope and an explainable effect on preparation,
execution and reuse. Compilation, validation, source changes and persistence use this
framework as fully as analytical queries.

**Status:** Proposed architecture; Interface-checked against the locally resolved
DataFusion 55.1.0 contracts; Implemented mechanisms and Tested gaps identified below.
This is a design review, not implementation acceptance. No performance improvement
has been measured for the proposed architecture.

### Method and coverage

- Read the complete supplied [provider capability map](../../capability-maps/datafusion_provider_contracts.md),
  the design charter/directive/template, relevant blueprint §3.3.3 and §5.4 contracts,
  Plan 05 HP02–HP13, and the earlier foundations review's construction and gate findings.
- Inspected actual catalog/schema/table implementations, source-role binding, plan
  admission, native preparation, common producer completion, pinned reopening, the
  compiler restoration boundary and the Rust table inspection reader. The recent P0
  role-collision correction is context, not a substitute for reviewing the common cause.
- Resolved `/apache/datafusion` through Context7. Queried hierarchy, scan/pushdown/DML
  and resolution/planning topics. Context7's main-branch examples are discovery
  evidence; the pinned source takes precedence where they differ.
- Ran `just metadata`, then `cargo metadata --locked --offline --format-version 1`:
  the recipe omits dependencies, so the latter was needed for resolved features.
  [Recorded context](../evidence/provider-contracts-context-2026-09-15.json) includes
  registry checksums, features and hashes of inspected implementation/source files.
  Rust is 1.98.1; DataFusion is 55.1.0, Arrow 59.3.0, object_store 0.13.2.
- Inspected the installed, version-matched `datafusion-session`, `datafusion-catalog`,
  `datafusion-expr`, core planner/context and datasource source. Official web
  documentation was also consulted. Versioned docs.rs/crates.io requests were not
  retrievable through the web tool; Lib.rs returned 403. No installed DataFusion-only
  skill or connected library-enrichment service was available. The design-review,
  Context7 and library-research skills supplied the workflow; no missing tool result
  is represented as evidence.
- Executed focused characterization probes, retained under `docs/design_review/evidence/`.
  They attack namespace lookup, metadata admission, root mutation semantics, view
  inlining and SQL DDL timing. They establish those observations only.
- Did not audit every numerical kernel, every file-format provider or an external
  catalog implementation. Did not execute remote-concurrency, multi-table DML,
  publication-failure or target performance experiments. These are explicit work
  packets and verification subjects, not implied passes.

The working tree contains extensive ongoing implementation. Source citations below
refer to the inspected working tree, not HEAD alone. No predecessor graph, legacy
store or old-versus-new equivalence campaign is needed for this design.

### Exact-release corrections to the capability map

These corrections affect implementation choices, not eligibility of the hierarchy.

| Topic | Interface-checked result | Design implication |
|---|---|---|
| Trait location | The six catalog/provider/function contracts are defined in `datafusion-session`; catalog crates re-export them | Use supported public imports and the one resolved family |
| Projection signature | Installed 55.1.0 `datafusion-session/src/table.rs:185` still takes `Option<&Vec<usize>>`; `ScanArgs::projection` takes a slice. The map and current online upgrade guide say the older method takes a slice | Compile against the actual pin; normalize both entry points into one scan implementation. Do not silently upgrade to reconcile documentation |
| Root registration | `CatalogProviderList::register_catalog` returns `Option`, has no error channel, and promises replacement | A sealed implementation returning `None` without registering is not a conforming rejection protocol |
| Schema registration defaults | Pinned `SchemaProvider` defaults produce execution errors; catalog/table defaults commonly use not-implemented errors | Standard application diagnostics need explicit translation. Error variant uniformity is not supplied by the traits |
| Structured scans | `ScanArgs` has projection, filters, limit and statistics requests; `ScanResult` currently contains an execution plan | No current preferred-ordering field or general guarantee/result certificate is supplied |
| Remote resolution | Default `Async*::resolve` caches referenced lookups, including missing lookups during construction; it awaits them in loops | It is not a batch/parallel resolver or an atomic remote snapshot protocol. Add bounded parallelism and backend revision binding where required |
| Cached enumeration | Resolved async providers enumerate the successfully resolved subset, not necessarily the entire remote namespace | Declare full versus referenced-subset coverage; absence outside that scope cannot prove global absence |
| Metadata | `SchemaProvider::table_type` can avoid constructing a table; columns and other metadata can still require `table()` | A cheap table-type override does not make all introspection cheap |
| Constraints/statistics | Constraints are optimizer claims, not validation; statistics requests are optional hints | Advertise only established facts and measured/cached statistics with explicit precision |
| DML | INSERT/DELETE/UPDATE/TRUNCATE/MERGE have native provider hooks and UInt64 `count` output conventions | A count is not proof of atomic commit, durable visibility or exactly-once execution |
| Views | `ViewTable::new` does not analyze or validate its plan. Native scan construction can inline `get_logical_plan()` | Table `scan` alone cannot be the universal policy/validation boundary |

Pinned sources for these rows are listed in §9. The
[official catalog guide](https://datafusion.apache.org/library-user-guide/catalogs.html)
establishes the hierarchy and its extension model. The
[current 55 upgrade guide](https://datafusion.apache.org/library-user-guide/upgrading/55.0.0.html)
is useful discovery evidence but disagrees with the installed projection signature.

## 2. Authority and lifecycle map

The hierarchy becomes the common access and execution framework. Existing canonical
semantic declarations remain the authority for meaning; provider objects are their
bound implementations. A second editable registry of provider policy would recreate
the duplication this pivot is intended to remove.

| Concept | Semantic identity | Authority / owner | Revision boundary | Update path | Derived representations |
|---|---|---|---|---|---|
| Relation definition | Existing declared relation/version and semantic fields | `pse-schema` registry | Definition revision | Declaration and generator | Arrow schema, metadata, obligation plans |
| Operation definition | Existing pass/operation identity, input/output roles and effects | Extend the existing operation declaration, not a second dispatch roster | Operation version | Declaration plus actual native implementation | Plans, factories, capability and inspection rows |
| Provider implementation binding | Actual implementation plus contract/version and captured dependencies | Assembly binding retained by the operation context | Binding generation | Explicit construction/rebinding | Native trait object and descriptive capability view |
| Scoped policy | Typed policy identity, scope and combination rule | Existing canonical declarations/policy relations, extended where needed | Policy revision | Validated command | Effective immutable policy and explanation |
| Namespace generation | Catalog/schema/table names, aliases, coverage and selected owners | One resolution result built from authoritative refs/declarations | One operation's resolved generation | Explicit refresh or namespace command | DataFusion hierarchy, SQL names, inspection views |
| Table binding | Relation version + instance/port + source revision + admission state | Exact candidate, completed or admitted owner | Binding lifetime | New binding; no identity inferred from table name | `TableProvider`, `TableSource`, typed algorithm view |
| Attempt workspace | Attempt ID, base generation, proposed changes and resources | One operation execution | Attempt lifetime | Declared DML/DDL/native execution | Private providers, streams and temporary state |
| Published result | Coherent manifest/ref plus declared output ports | Existing conditional publication protocol | Commit boundary | Consume complete outcome, then publish | Read-only snapshot catalog and Python handles |
| Observation | Operation, provider, policy and attempt linkage | Actual execution/inspection measurement | Observation event | Append observation | EXPLAIN, diagnostics, metrics and provenance tables |

**Identity:** SQL names and display aliases locate bindings; they are not semantic
IDs. Rename preserves the applicable entity identity and changes only the declared
name mapping. Before/after roles and two versions of one schema remain distinct.
Catalog snapshots need exact manifest/encoding context, not a snapshot ID alone.
Aliases such as `head` are resolved once per operation. Caches retain exact owners
and their dependency scope; hashes can accelerate lookup but cannot create authority.

**Namespaces:** the seven domain namespaces remain a declaration category, not a
restriction on the native schema hierarchy. Internal operation scopes, candidate
bindings, completed results, runtime attempts and metadata need real catalog/schema
entries. Generate their names from scoped binding identities and maintain reverse
mapping for diagnostics. Do not require functions to invent collision-resistant
prefix strings. The current `packages` collision is a concrete example of that cost.

**Opaque implementation:** parsing, graph algorithms, quantity inference, numerical
kernels, solver callbacks and object-store primitives can remain ordinary code.
Their product-level invocations, inputs, outputs, effects and results belong to the
same provider/operation framework. Their implementation language is not an exception.

## 3. Semantic contracts and invariants

### Standardization at every native boundary

The following is the required design surface, not a claim that every native trait
already implements it. Share policy declarations and evaluation mechanisms; preserve
the distinct semantics of each native interface.

| Native level | Standard rules and policies | Mechanism and boundary | Observable refusal / evidence |
|---|---|---|---|
| `CatalogProviderList` | Name normalization, catalog replacement, alias resolution, metadata-generation selection, default catalog, visibility and enumeration coverage | One operation-scoped resolution/assembly; native registration semantics remain faithful; application command boundary governs modifications | Missing catalog versus failed resolution distinguished; exact selected generation recorded |
| `CatalogProvider` | Schema naming, ownership scope, replacement, cascade, immutable published scope versus private assembly, metadata budgets | Generated schema inventory and selected policy; cheap synchronous lookup over resolved metadata | Nonempty drop without permitted cascade fails; retained readers keep their resolved owners |
| `SchemaProvider` | Table naming, duplicate refusal, owner metadata, full/subset enumeration, `table_exist` coherence, cheap `table_type`, resolution errors and cache lifetime | Single immutable table-entry metadata source; atomic insert-if-absent in mutable workspaces | Duplicate table errors; unknown lookup is not backend failure; unavailable is not an empty table |
| `TableProvider` metadata | Semantic schema, admission state, PK/unique facts, defaults, view definition, table kind, statistics precision, binding provenance | Project from the bound relation/operation contract and established facts | Candidate duplicates remain visible; unknown facts are not optimizer guarantees |
| `TableProvider` planning | Predicate support, projection/order/duplicates, filter-only columns, limit, statistics requests, scan purpose and actual backend | `scan` and `scan_with_args` share one planner; predicate classification and translation share one implementation | Exact/Inexact/Unsupported applies per expression; unsupported hints retain native residual work |
| Provider DML | Supported write operations, base generation, validation, affected-row accounting, conflict/retry semantics, candidate/result visibility | Native mutation methods construct explicit attempt-owned execution plans; shared completion/publication consumes all outputs | Unsupported operation or stale base fails before authoritative visibility; no count-only success receipt |
| `TableProviderFactory` | Format/options/schema policy, source identity, allowed declared effects, resolution resources and implementation/version | Bind `CreateExternalTable` through the same operation context; use explicit declared schema where product semantics require it | No inferred physical schema silently becomes a declared semantic relation |
| `TableFunctionImpl` / views | Typed parameters, selected source generation, dependencies, effects, metadata and output schema | Pure provider construction; expensive work in scheduled execution; actual function/view binding retained even if inlined | Same name with another implementation is another binding; invocation does not silently choose a new model revision |
| `Session` / native planners | Effective policy, actual UDF/UDAF/UDWF/higher-order/table functions, analyzers, logical/physical optimizers, codecs and source bindings | Freeze one effective `SessionState` per prepared operation; inspect all plans, including scalar-only and extension plans | Cross-catalog policy conflict is explicit; native defaults do not imply reproducibility or validity |
| `ExecutionPlan` / `DataSource` | Partitioning, ordering, boundedness, stream schema, cancellation, memory/spill, per-execution state, statistics and observation | Native operators and sources under one runtime; custom operators where algorithms require them | Late failure remains failed/partial; no early batch establishes whole-result validity |
| `DataSink` / publication operation | Durable output, complete multi-table obligations, manifests, conditional refs, idempotency and uncertain outcomes | Existing storage primitives behind a registered effectful operation and one commit protocol | Storage write failure, rejected publication and uncertain commit are distinct |
| Rust/Python inspection | Same binding lookup, metadata, capability errors, field retention, batch limits and exported-owner lifetime | Provider-backed owned streams and generated projections of the same operation/result contract | Closing a handle stops unread work; retained arrays keep leases; inspection cannot compile or publish |

**Shared registry mechanics:** use one consistent name-validation/lookup policy,
deterministic enumeration where the application promises it, `Arc` ownership and
atomic updates within mutable assembly scopes. Preserve catalog replacement versus
table duplicate refusal; a shared map implementation must not erase that distinction.
Default catalog/schema resolution belongs to the operation's captured configuration.
Standardize the native `is`/`downcast_ref` helpers for diagnostic or implementation
adaptation, while ordinary policy selection uses explicit bindings and capabilities.
Concrete Rust types must not become a second business-policy dispatch system.

### Policy combination is explicit

One generic last-writer-wins map is insufficient. Standardize these combination rules:

1. **Semantic requirements and established facts:** requirements compose; proofs
   remain tied to exact owners and premises. A child scope cannot override a failed
   invariant or declare an unproved key valid.
2. **Effects and supported operations:** the requested purpose, inherited policy and
   actual implementation jointly determine admissibility. A narrower scope may
   restrict a capability; a boolean cannot manufacture an implementation. Reads,
   writes, external-state reads and nondeterminism are separate dimensions.
3. **Defaults and strategy selection:** define which settings may be overridden and
   their root → catalog → schema → table → invocation precedence. SQL identifier
   normalization happens once; quoted identifiers retain their declared meaning.
4. **Resources:** all scopes charge the common runtime. Child allocations cannot
   exceed their selected parent budget. Thread/partition/spill settings are composed
   deliberately, not independently recreated at every provider.
5. **Cross-catalog plans:** resolve a single effective operation context from all
   participating bindings. Reject incompatible semantic/effect requirements or
   require an explicit conversion/selection. Do not mutate session settings while
   scanning different tables.
6. **Observation:** record effective values, their declaration origins, chosen
   implementations and relevant dependencies. Policy views are generated projections
   of those bindings, not a second configuration interface.

This uses native configuration extensions where appropriate, but catalog traits do
not supply automatic hierarchical policy inheritance. PSE must implement the above
composition once and bind its result into native sessions/providers.

### Absence, validity and completeness

| Distinction | Required representation and handling |
|---|---|
| Missing / failed lookup | `None` only for absence within declared resolution coverage; failure remains a structured error |
| Absent optional input / present empty relation | No binding versus a binding with zero rows; never interchangeable |
| Not yet materialized / empty result | Known operation/output state versus completed zero-row table; lookup does not run the producer |
| Candidate / established field values / established relational facts / admitted result | Retain actual private owners and completed obligations; metadata alone cannot promote a state |
| Unknown capability / unsupported / supported / failed invocation | Distinct capability and operation outcomes; no speculative write used as a capability probe |
| Partial stream / complete outcome / published revision | Separate lifecycle states; only complete admitted bundles cross publication |
| Unknown remote generation / coherent captured generation | Record the uncertainty or require backend revision support; an in-memory cache alone does not establish coherence |

Native constraints primarily represent primary/unique keys. PSE quantity, foreign-key,
coverage, source-correspondence and lifecycle obligations remain native diagnostic
plans or contracted algorithms under the common operation framework. They are not
supplied by `constraints()` or `schema()`.

## 4. Derivation and execution design

### One framework, with each responsibility at its native level

```text
canonical relation / operation / policy declarations
                    |
explicit resolve + bind operation (metadata I/O and admission are recorded)
                    |
CatalogProviderList -> CatalogProvider -> SchemaProvider -> TableProvider
                    |                         |              |
             scoped generation          metadata views   scan / DML / view
                    |                         |              |
                    +---- frozen native Session + bindings --+
                                      |
                 analyze + policy/semantic obligations + optimize
                                      |
                   ExecutionPlan / DataSource / DataSink
                        |                         |
                  owned read stream         private attempt output
                                                  |
                                  complete obligations + conditional publish
                                                  |
                                       new snapshot provider generation
```

The hierarchy does not contain per-schema analyzer or optimizer hooks. Those rules
are assembled in the native session from the participating scopes. Likewise,
`RuntimeEnv`, planners and codecs cooperate with the hierarchy; they are not children
of `SchemaProvider` in the Rust trait API. This placement covers scalar-only plans,
view inlining, table functions and custom physical nodes that would evade policies
implemented only inside a table scan.

### Binding, metadata and admission

Use one resolved binding inventory for SQL lookup, direct `LogicalPlanBuilder`
construction, metadata enumeration, provenance and typed algorithm access. Native
`TableSource` is derived from the same bound `TableProvider`. A direct scan may be
efficient; it must not introduce a second naming or admission route.

Generalize the current actual-owner admission mechanism to actual registered provider
bindings. A binding retains its real implementation, immutable metadata, established
facts, dependencies, resource owner and permitted operations. Native external
providers, views and table-function outputs enter through this same boundary.
Do not replace the current concrete-type allowlist with a freely settable `trusted`
flag or an unaudited `ProviderCapabilities` sidecar. Capabilities are descriptive
projections of a bound implementation contract; the invoked methods remain decisive.

Validation rules apply at the earliest sound boundary and remain associated with
the exact immutable output. Pure row-local checks may use the existing reusable
semantic UDF mechanism. Keys/FKs/coverage and late stream obligations need whole
relation or bundle completion. A validation UDF cannot make a publication effect
safe merely by appearing in a logical plan.

### Reads, changes, compilation and publication

| Operation family | Target native route | Required lifecycle |
|---|---|---|
| Analytical query / diagnostics | Bound providers plus ordinary native plans and UDFs | Complete read context; residual diagnostics share source owners |
| Source ingress and parsing | Source provider/factory plus contracted parsing/lowering operator | Bytes and schema/version explicit; original parse owner reused; candidate output remains uncommitted |
| Rename and ChangeSet | Native before/after providers and relational change plans; DML for applicable updates | Base generation and complete source correspondence retained; invalid edits never publish |
| P0–P10 compilation | Existing semantic algorithms bound as native operations/operators; provider-backed inputs and named results | Explicit compile command, attempt-scoped execution, one invocation per operation; no compile-on-inspection |
| Rule recursion | Native recursive/worktable providers within the same attempt | Monotone/stratum semantics, conflicts, absence dependencies and resource limits remain explicit |
| Quantity/MathIR/solver/backend | Typed projections into specialized implementations behind native operator contracts | Whole coupled problem retained; analytical pruning cannot alter a solver problem; callbacks are internal execution |
| Import / cold reopening | Explicit provider-resolution/admission operation with shared traversal context | Verify current artifacts and unresolved correspondence before exposing an admitted handle; no repair/publication |
| INSERT/DELETE/UPDATE/TRUNCATE/MERGE | Actual provider mutation hooks using private candidate/attempt scopes | Stage all related outputs, check complete obligations, then publish conditionally |
| DDL / defaults / Copy / external tables | Native logical command + provider/factory/sink mechanism | Command admission before effects; defaults derive from canonical declarations; namespace and durable effects recorded separately |
| Persistence / export | DataSink or custom publication ExecutionPlan using current storage protocol | Output objects and manifest first, conditional visibility last; explicit uncertain commit and retry semantics |
| Rust/Python table inspection | Same resolved table binding and owned provider stream | No source compilation, selected-method resolution or new authoritative facts during inspection |

**Multi-output compilation:** an `ExecutionPlan` returns one stream schema. Do not
flatten the P3–P10 bundle into an untyped universal table to imitate that signature.
One explicit operation owns its private output catalog and typed ports; its execution
stream reports the operation outcome. On completion, the output providers bind the
actual results from that invocation. Reads of those providers do not rerun the
compiler. Publication validates and exposes the whole declared bundle together.
Extend the existing pass/operation declarations and completion ownership for this
purpose, then delete the competing producer dispatch boundary after consumers move.

**DDL timing:** upstream `SessionContext::sql_with_options` calls
`execute_logical_plan`; DDL handlers may change namespaces immediately. The common
frontend therefore constructs a native logical command first, checks effective
policies and binds an attempt, and only then invokes the effectful handler. EXPLAIN
and validation use the prepared command without running it. Blanket removal of
`with_allow_ddl(false)` from the current facade would not implement this design.

**Native root mutability:** the list trait has no fallible registration API. Keep a
conforming list as private assembly/command state; expose fallible application
commands for policy validation and durable visibility. Prepared operations capture
resolved source and metadata generations and do not re-resolve mutable aliases.
Any native extension that accesses or changes a session catalog participates in the
declared operation effects. Instrument namespace generation changes and reject
undeclared changes before admitting an outcome. Do not claim that the raw trait
object is immutable, an authorization boundary or a transaction. The current silent
no-op implementation must be deleted. Tests in §9 are required before this boundary
is accepted.

**Remote resolution:** reuse the `Async*` contracts for discovery, with explicit
backend revision tokens or coherent metadata snapshots. Resolve dependencies in
bounded parallel batches where useful; retain the same generation for the operation.
Do not build a remote service until a consumer needs it. The local resolver and a
controlled adversarial fake backend establish the standard lifecycle first. A source
without snapshot support can be used under a declared live-observation contract;
strict reproducible model admission must capture the actual input or refuse the
unestablished guarantee.

**Materialization and resource ownership:** reuse immutable metadata and source
owners across forks; materialize at actual fan-out, validation, publication or
foreign-boundary needs. Full `SessionState` and catalog-map reconstruction on each
small role addition is not the target. Native execution still owns mutable operators
and per-attempt buffers. Array export keeps existing leases; provider adoption is not
a reason to copy buffers or keep completed execution workspaces alive indefinitely.

## 5. Representative journeys

### Ordinary extension: a diagnostic relation over a heater graph

Declare the diagnostic's output schema and operation, then bind its actual native
plan over a selected P10 generation. The schema provider obtains name, table kind,
metadata and capability views from that declaration/binding. Rust, SQL and Python
resolve the same provider. Quantity checks and source dependencies come from shared
construction rules. No new table-class branch, Python row DTO, naming convention or
producer roster is needed. A new algorithm is added only if its semantics require one.

### Meaningful change: update the heater's selected configuration

Resolve the current base once. Bind before/after tables in a private authoring
catalog. A native update/change operation creates new source candidates; the shared
validation and compilation operations consume those candidates. Policy and source
changes invalidate exactly the dependent prepared results. Conditional publication
exposes the complete new revision. The original reader retains its original graph.
There is no need to retain a discarded predecessor architecture or its data objects.

### Boundary: factory-created data and cold Python inspection

A declared factory resolves an external source under an explicit input/capture policy.
Physical Arrow fields do not automatically supply missing quantity or identity
meaning. The shared boundary derives or checks the declared semantic fields before
model admission. On later cold open, one resolution/admission owner supplies Rust
and Python provider handles. Metadata and table reads use that admitted generation;
they do not repair missing outputs. A retained sliced Python array remains valid
after stream, snapshot and store handles close, and the final drop releases its lease.

### Interruption: multi-table mutation followed by uncertain publication

Execute a command against a captured base and stage all affected relations privately.
Cancellation before completion releases private output and leaves the published ref
unchanged. A crash after writing objects but before publication does not make them a
valid visible revision. A timeout during conditional publication is resolved through
the operation/ref receipt, not by assuming rollback or repeating an external effect.
Concurrent stale-base commands fail explicitly. Table DML count rows are linked to
this outcome; they cannot independently report a durable commit.

### Adversarial journey: an apparently harmless view or UDF

A view exposes a logical plan and is inlined; a filter folds away; a scalar-only plan
has no table scan. Required policies, implementation dependencies and effects remain
bound to the prepared operation before those changes. Only valid property transfers
survive. If a stable/volatile function reads mutable external state, bind/capture that
state or execute under its declared observation/effect policy. A mandatory action is
an explicit command, never a scalar side effect whose execution an optimizer may
omit, duplicate or reorder.

## 6. Acceptance gates

These gates assess the full requested provider-governed behavior. Existing narrower
paths have useful safeguards; that does not establish the enlarged contract.

| Gate | Result | Evidence or gap | Required action |
|---|---|---|---|
| G1 — Authority | Unresolved | Actual snapshot owners exist, but the common policy/binding authority and native-root mutation reconciliation are not implemented | PR01–PR03: one binding generation and explicit command ownership |
| G2 — Semantic fidelity | Unresolved | Current semantic field UDFs/admission are useful; the full native-provider/view/factory path lacks a common preservation contract | PR03–PR05: derived semantic properties and negative conversion tests |
| G3 — Validity | Unresolved | Candidates correctly withhold constraints; replacing closed admission and adding write providers needs end-to-end obligations | PR03, PR05–PR06: established facts and complete bundle validation |
| G4 — Hidden behavior | Unresolved | Snapshot reads currently reject mutations; full DDL/factory/function/operator effects cannot be governed by scan callbacks alone | PR04–PR06: explicit preparation, effect declaration and command execution |
| G5 — Consistency and recovery | Unresolved | Existing conditional publication remains valuable; no complete provider DML/multi-table protocol is implemented | PR06–PR07: attempt isolation, stale-base refusal, uncertain-commit recovery |
| G6 — Transformation and reuse | Unresolved | Current preparation captures roles before optimization; new provider policies, view bindings and remote absence scopes need equivalent dependency coverage | PR03–PR04, PR07: exact inputs/implementations and complete invalidation |
| G7 — Truthful capability claims | Fail for the full target | Role SQL lookup and ordinary information-schema access fail; admitted provider implementations are closed; root registration silently does nothing | PR02–PR03, PR08: coherent discovery, truthful native contracts and general registered bindings |

There is no aggregate score. **Revise** follows from the failed and unresolved gates.
The maintainer's selected direction remains the basis for the corrections.

## 7. Principle findings

Verdicts apply to the stated finding/scope, not every use of a principle across the
repository. Findings are grouped by cause and ranked by correctness/authority,
semantic duplication, then unmeasured cost.

| Finding | Principle IDs and verdict | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|
| F1 — Provider identity and hierarchical lookup are split | DM-19, DM-43, DM-55: Violated for named-role discovery | `provider/catalog.rs:18` enumerates only `Namespace::ALL`; `session/roles.rs:141` creates `roles.inputs`; `snapshot_session.rs:637` reconstructs catalogs from separate maps. Probe: direct role scan succeeds, SQL lookup fails | A bound input is present to one consumer and absent to another; internal names collide with real pass ports | One scoped binding inventory; real operation schemas and generated role names; direct scans use the same entries | Same provider owner and values via role, SQL, metadata and Python; repeated versions and empty inputs remain distinct |
| F2 — Admission is a concrete implementation roster | DM-19, DM-43, DM-44: Violated for the enlarged provider target; DM-45: Unresolved for its replacement | `session/admission.rs:170` checks pointer membership, then branches on `RelationTable`, `CandidateTable`, `ComputedTable`, `IndexedTable` and rejects others. `read_back_settings` at `snapshot_session.rs:851` has a special bypass | A conforming view/factory/provider cannot participate without another core type branch; ordinary metadata is rejected | Bind actual registered provider implementations and semantic/effect contracts; generate metadata providers within that same admission model | Add a provider and table function without editing admission; forged facts, foreign bindings and hidden effects still fail |
| F3 — Root registration violates the native contract | DM-19, DM-43, DM-59: Violated | `provider/list.rs:24` returns `None` and leaves the list unchanged; upstream `catalog.rs:200` specifies insertion/replacement | Native code receives the same return value as successful first registration, then lookup says absent | Conforming private assembly list plus fallible application commands; capture read generations and delete silent no-op | Register/replace return exact prior owners; unauthorized publication refused; existing prepared readers retain selected bindings |
| F4 — Full operation effects have no common provider lifecycle yet | DM-22, DM-28–30, DM-41: Unresolved | `admission.rs:97` rejects all DML/DDL/Copy/Statement in the only session route; `computation.rs:25` invokes a separate opaque producer contract; upstream context handles DDL during `sql()` | Simply enabling native commands could apply effects before PSE validation; keeping current separation leaves provider standardization incomplete | Common native operation preparation and purpose policies; native DML/DDL/factory/sink routes and one completion/publication protocol | EXPLAIN has no effects; invalid/late-failing changes do not publish; count/receipt agree after conditional commit |
| F5 — Scan-only enforcement loses view and non-table obligations | DM-07, DM-24, DM-28, DM-31: Unresolved for the proposal | `datafusion-expr` builder `scan_with_filters_inner:525` inlines a provider logical plan; scalar-only/extension plans need no scan. Map §12 describes hierarchy but no enclosing enforcement path | A required action or policy implemented only in `scan` may never run, while a prepared query still executes | Bind obligations/effects/dependencies before lowering; session rules and execution completion supplement provider callbacks | Inlining, constant folding, zero-row plans and custom nodes preserve required policy or explicitly refuse |
| F6 — Metadata and capability descriptions need one derivation | DM-02, DM-07, DM-43, DM-52: Unresolved | Map §9.H suggests independent capability booleans; current `statistics.rs:10` only supplies row count; `table.rs:143` ignores statistics requests; ordinary metadata probe fails | Sidecars can promise unavailable mutations or unproved constraints; blueprint statistical claims can exceed actual implementation | Derive metadata and capability views from bindings; separate requirement/support/established fact; implement or explicitly report unavailable statistics | Method/metadata conformance; candidate duplicate detection; requested statistics and precision reflect actual source |
| F7 — A cached remote tree is not a coherent remote generation | DM-14, DM-31–32, DM-35: Unresolved | Map §8 implies coherence; pinned `async.rs:342` walks live lookups and caches referenced subsets, without backend snapshot or transaction | Two tables may capture different remote revisions; absent/unresolved entries may be treated as globally absent | Version-bound resolver, explicit coverage, negative dependencies and bounded request coordination | Backend changes between lookups; mixed capture rejected or reported under explicit observation semantics |
| F8 — Rebinding/materialization has avoidable structural duplication | DM-26, DM-37–39, DM-56: Unresolved for cost; Implemented mechanisms identified | Every role addition calls `refresh_sources`; `snapshot_session.rs:637` rebuilds maps/state; `preparation.rs:249` collects every batch; `producer.rs:124` starts a new physical inventory during restoration | Fine-grained native composition and cold admission can repeatedly rebuild shared context; a provider rewrite that repeats this pattern misses its benefit | Share resolved immutable generations, explicit streaming/materialization boundaries and traversal-scoped owners | Measure rebindings, producer calls, parses, copies, first batch, total latency and peak/residual memory |
| F9 — Rust/Python inspection retains separate table resolution and slicing semantics | DM-41, DM-44, DM-55–56: Unresolved for the new target | `inspection.rs:30` resolves snapshot members directly; `next_batch` slices the loaded relation rather than consuming a provider stream | New source kinds or provider policies would require coordinated inspection changes or diverge from query behavior | Resolve once through common bindings; retain the existing lease/close guarantees in the provider stream adapter | Identical metadata/values/refusals across interfaces; partial drain, close, cancellation and final export-owner release |

**Implemented strengths to retain:** immutable admitted owners and actual registry
checks in `RelationTable::new`; constraint-free candidates; native filter operators
in `provider/table.rs:184`; shared scan planner for both entry points; exact role
owners captured before optimization; private completion before publication; and
exported array leases. These mechanisms prevent real invalid states and need to
survive their integration into the broader framework.

| Applicable retained contract | Principle verdict within inspected scope | Enforcement and rejected state |
|---|---|---|
| Actual source owner and declaration coupling | DM-02, DM-07: Satisfied for current admitted table construction | `RelationTable::new` rejects a foreign registry context and ambiguous relation membership |
| Candidate facts are not admitted keys | DM-07: Satisfied for current candidate providers | `CandidateTable` advertises no constraints; current plan admission rejects a candidate claiming them |
| Read-only snapshot purpose | DM-20: Satisfied for the inspected SQL/plan entry points | Current snapshot facade rejects native mutation families before execution; the larger purpose model remains unresolved |
| Scan preparation builds native operators | DM-26: Satisfied for the inspected admitted in-memory scan | `native_scan` constructs source/filter/limit plans without evaluating the predicate over rows |

**Applicability:** all twelve principle groups bear on this cross-cutting change,
with depth on authority, providers, effects, validity, reuse and boundaries. DM-05,
DM-25 and DM-38 remain compatible: semantic intent stays declared, while DataFusion
is the selected framework and specialized algorithms implement native operation
contracts. Numerical approximation details, solver fidelity and unrelated domain
rules were not reviewed; they remain applicable product requirements, not automatic
passes or exceptions. No score or blanket enumeration of all sixty principles is used.

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication / extension locality | Correctness / operational risk | Cost | Performance evidence | Decision |
|---|---|---|---|---|---|
| Current hierarchy plus separate role maps, inspection and producer paths | Repeated admission/naming/dispatch adaptations remain | Known discovery defects; new providers require core edits | Lowest immediate change, growing extension cost | Current source only; no comparative benchmark | Does not satisfy the stipulated target |
| A second universal provider-policy DSL and wrapper trait mirroring every DataFusion method | Moves duplication into a parallel API and interpreter | Capabilities and native behavior can disagree; large abstraction surface | High implementation and maintenance burden | None | Reject; no demonstrated need for a second engine contract |
| Native traits plus one declared binding/policy model, shared operation lifecycle and thin generated adapters | One semantic declaration and actual implementation; native extension points remain available | Requires explicit command/publication and lifecycle conformance | Broad but dependency-ordered pivot; reuses existing semantic and storage mechanisms | Benefits remain hypotheses to measure | Selected; the simpler coherent architecture meeting the requested scope |

**Concrete benefits expected:** uniform lookup eliminates the observed role mismatch;
common capability binding removes admission-type edits for each provider; standard
mutation semantics make ChangeSets and DML share validation/publication; provider
metadata enables uniform inspection; shared generations can eliminate repeated
resolution. Correctness mechanisms are specified above. Latency/memory improvements
remain Proposed until the target measurements in §9 exist.

The useful shared core is naming and resolution, bound metadata and capabilities,
policy composition, preparation/completion, and conditional publication. A generic
registry container is optional implementation detail, not a new semantic authority.
Use upstream providers/operators directly when they satisfy these contracts. Custom
scan/mutation planners and numerical kernels remain ordinary code where required.

## 9. Verification and measurement plan

### Evidence inventory

Local dependency citations refer to the installed source identified and hashed in
[the context record](../evidence/provider-contracts-context-2026-09-15.json):

| Evidence | Inspected locations | Claim strength |
|---|---|---|
| U1 — hierarchy and registration | `datafusion-session-55.1.0/src/catalog.rs:133,200`; `schema.rs:37` | Interface-checked |
| U2 — scans, metadata, writes and factories | `datafusion-session-55.1.0/src/table.rs:52,185,210,317,341,428,524,566,601` | Interface-checked |
| U3 — native planning/effects | `datafusion-session-55.1.0/src/session.rs:80`; `datafusion-55.1.0/src/execution/context/mod.rs:642,686`; `physical_planner.rs:750` | Interface-checked; DDL timing also probed |
| U4 — async resolution | `datafusion-catalog-55.1.0/src/async.rs:188,261,333` | Interface-checked; remote coherence not tested |
| U5 — views | `datafusion-catalog-55.1.0/src/view.rs:44,75,98`; `datafusion-expr-55.1.0/src/logical_plan/builder.rs:513` | Interface-checked; inlining also probed |
| U6 — execution and sinks | `datafusion-datasource-55.1.0/src/source.rs:128`; `sink.rs:50`; current PSE `session/preparation.rs:249` | Interface-checked / Implemented, not full lifecycle-tested here |
| C1 — current application | Source expressions cited in §7 and working-tree hashes in context record | Implemented; guarantees limited to inspected paths |
| P1 — executable probes | [Probe source](../evidence/provider-contracts-probe.rs) and [receipt](../evidence/provider-contracts-probes-2026-09-15.md) | Tested characterization only; see receipt for command, mode, baseline and counts |

The prior implementation's P0 regression test completed during task transition:
`just test-package pse-authoring --test p0 --no-fail-fast --status-level fail
--final-status-level fail`, default/force-validation/baseline 0, **2 passed, 0
failed/skipped**, 0.627 s, run `f91e8775-c782-4f6b-aa94-66a94bffb710`. This does not
qualify the engineering workflow or this proposed architecture. Further implementation
runs were paused for the review.

### Target conformance matrix

| Claim / risk | Evidence label now | Test / analysis | Conditions and required result |
|---|---|---|---|
| One coherent hierarchy | Proposed; current contrary probes | Role/SQL/Python/metadata resolution conformance | Two versions, two ports of one schema, quoted names, alias changes, empty and absent inputs; same exact binding everywhere |
| Faithful registry contracts | Proposed; current root defect probed | Root/catalog replacement, table duplicate, missing removal and cascade tests | Atomic insert semantics under concurrency; previous provider returned correctly; unchanged captured read generation |
| Complete metadata | Proposed | Standard information schema plus generated PSE policy/capability/lineage views | Full/subset coverage explicit; cheap metadata performs no row execution; errors never become missing/empty |
| Extensible admission | Proposed | Native view, table function, file provider and a custom provider through one binding API | No new concrete-type branch; forged keys/fields, wrong context or hidden dependencies rejected |
| Correct scan negotiation | Existing implementation inspected; wider target Proposed | Same immutable data with pushdown enabled and Unsupported reference | Values and multiplicities agree for nulls, duplicates, repeated/reordered/empty projections, filter-only columns, Exact/Inexact and LIMIT; deliberate over-pruning is detected |
| Fact transfer and validation | Proposed | Projection/join/union/view and candidate diagnostic tests | Unproved uniqueness never optimizes away duplicates; dropped join/key premises remove dependent guarantees; metadata cannot perform a unit conversion |
| Preparation does not publish | Proposed; upstream timing probed | EXPLAIN, prepare/drop and rejected-command tests | DDL, DML, table factories, scalar-only and extension operations produce no unauthorized visible effect |
| Multi-output and DML lifecycle | Proposed | Insert/delete/update/truncate/merge plus P3 multi-port publication | Whole bundle validation; no early visible port; one UInt64 count result where native DML requires it, tied to real outcome |
| Repeated execution/retry | Proposed | Fresh attempts, cancellation and partial drain | Operators reset or are rebuilt; no hidden reuse of consumed stream/workspace; duplicate execution cannot repeat a committed logical command silently |
| Coherent resolver | Proposed | Controlled backend changes between lookups, duplicate references, failures and cancellation | Exact revision capture or explicit live-observation semantics; bounded concurrency; negative-read scope retained |
| Safe reuse | Proposed | Independently change provider implementation, policy, default, absent candidate, metadata generation and external input | Every meaningful change invalidates affected preparation/results; alias spelling alone is not authority |
| Ownership and resources | Proposed | Pending streams, exception, partial drain, Python retained slice, final release, spill/refusal | One runtime budget; no per-provider hidden pool; no leaked attempt state; actual exported owners retain leases |
| Full product workflow | Proposed | Real source-authored heater/mixer × FTPx/FcTP through compile, persist, cold Rust and Python | Independent equation, quantity, selected-method, contribution/exclusion and provenance expectations; all operations use target bindings |
| Extension locality | Proposed | Add one new diagnostic/provider and one scoped policy | One authoritative addition plus genuinely new implementation/tests; no repeated rule, roster or Python policy edits |
| Target cost | Proposed | Instrument small edits, large/skewed sources, repeated views and cold/warm reopening | Record actual thread/partition/batch/memory/spill settings; measure resolution, parses, rebindings, plan phases, first/last batch, materializations, copies, publication, reopening, peak and final memory |

Use mathematical and lifecycle expectations, not predecessor equivalence. Run the
repository's zero-baseline, force-validation checks appropriate to each packet.
Broader Rust/Python/parity and terminal Plan 05 gates remain required at delivery;
this review does not substitute focused characterization for them.

## 10. Exceptions and unresolved decisions

**No product data-operation exception is accepted here.** All currently identified
families have a route through provider-backed bindings and native operation/execution
contracts. A missing implementation is an open packet. Expensive restructuring,
non-relational inner loops or existing policy are insufficient reasons for exclusion.

| Boundary / decision | Resolution proposed | Evidence / limit | Owner and revisit trigger |
|---|---|---|---|
| Root trait lacks typed rejection | Keep native registration faithful in private assembly/command scope; enforce policy at fallible operation boundaries; capture read bindings and record mutations | U1; raw trait is not an immutable capability API. Concurrency and extension-side mutation checks remain unverified | Catalog/runtime owner; before any public mutable provider or session-catalog extension is supported |
| Registry/code generation bootstrap | The minimal declarations and build-time generator may run before a runtime hierarchy exists; generated/runtime relations then enter it | A bootstrap dependency cycle cannot be solved by requiring the generated engine to generate itself; this is tooling, not a separate product data engine | Schema/tooling owner; when generation can reuse an already-built standard provider operation without a cycle |
| Native FFI callbacks and OS I/O primitives | Remain internals of contracted native execution; operation entry and outcome stay in the framework | Re-entering a planner for every callback/syscall would change execution mechanics without adding semantic coverage | Runtime/backend owner; any new independently callable product operation must receive a framework binding |
| Remote source without snapshot support | Explicit observation/capture contract; refuse an unsupported reproducible snapshot claim | Async caching does not establish multi-object remote atomicity | Catalog/import owner; first remote source integration |
| Numerical/adaptive algorithm layout | Typed inputs/results and native operation contract; specialized internal state | No exclusion from provider-governed invocation, validation, resources or publication | Compiler/solver owner; first target workload exposing an unrepresentable boundary |

No SHOULD deviation is proposed merely to preserve existing architecture. If a
future product operation is alleged to be fundamentally incompatible, document the
required semantics, attempted native extension/operator route, failing executable
example and smallest excluded boundary. The maintainer reviews that exception;
it must not grow into an alternative data engine.

## 11. Decision and implementation changes

**Decision: Revise the current design and implementation to deliver the stipulated
provider-governed architecture.** Adopt the target in §§2–4 and implement the packets
below. Close the failed/unresolved gates with their named evidence before claiming
acceptance. The hierarchy standardizes all supported operations; existing contracts
are retained when they express required semantics, and their mechanisms are rewritten
where needed.

### Policy and authority changes

| Current rule / assumption | Required disposition | Authority change / affected surface |
|---|---|---|
| Seven namespaces are the entire provider schema universe | Keep domain namespaces; permit generated operation, binding and metadata scopes | Proposed revision to blueprint §5.4 and Plan 05 HP02/HP11 |
| General sessions are exclusively snapshot/read-only paths | Define read, candidate, authoring, execution and publication purposes in one framework | Extend ADR-0067 or create the scoped follow-on decision before implementation; blueprint §3.3.3, §5.4, §14.3.1 |
| Concrete PSE provider types are admission authority | Bind real implementations and their complete semantic/effect contracts | `session/admission.rs`, provider registration and operation declarations |
| Native root registration may silently fail | Remove the no-op; use faithful assembly semantics and fallible commands | `provider/list.rs`, assembly lifecycle and generated diagnostics |
| Every catalog schema lookup must fit the current eager local loader | Keep cheap resolved lookup; allow explicit async resolution/admission and lazy execution | Blueprint §5.4/§20; no hidden network failure as absence |
| Statistics/pushdown claims are fixed prose | Derive capabilities/facts; expand implementation when justified; report unsupported requests honestly | Reconcile blueprint §5.4 statistical claims and capability map with current source |
| Separate Rust/Python inspection table handling | Use common binding and stream semantics; preserve exported-owner guarantees | Plan 05 HP11, inspection code and generated Python boundary |
| Provider policies imply a restricted function/library list | Remove any such stale interpretation | Full native library eligibility remains; effects are purpose-specific contracts, not library prohibitions |
| Earlier implementation packages are fixed despite the newer target | Rebase remaining Plan 05 work on these packets and delete superseded paths | Plan 05 remains the execution plan; this review is its provider-design input, not a parallel execution plan |

Blueprint and accepted ADR files were not edited during this review. The maintainer
has selected the direction; the ADR/design revision work below records the concrete
semantics and evidence rather than reopening eligibility.

### Dependency-ordered handoff packets

All packets are hard pivots. Delete the replaced consumers/path in the packet that
installs its target, preserve useful domain algorithms through complete native
contracts, and retain no legacy runtime stores or compatibility switches.

| Packet / dependency | Owned responsibility and likely files | Implementation outcome | Deletion / acceptance oracle |
|---|---|---|---|
| **PR00 — reconcile design authority**; first | Proposed ADR-0067 or scoped follow-on; blueprint design revision; Plan 05; provider map | Record purpose/scoping, root mutation, multi-output operations, effect and publication semantics; reconcile exact-release API differences | Delete stale instructions/claims; independent G1–G7 design assessment and declaration ownership agreed |
| **PR01 — declare binding and policy semantics**; PR00 | `pse-schema` operation/policy contracts and generators; catalog binding/context | One relation-instance/role identity, metadata generation, purpose and typed policy-combination model; actual implementation bindings | No independently editable capability booleans or new competing dispatch registry; generated contract checks and conflict cases |
| **PR02 — build the complete provider hierarchy**; PR01 | `provider/{list,catalog,schema}.rs`, `session/{roles,snapshot_session}.rs` | Generic scopes, exact quoted naming, faithful mutation semantics, metadata coverage; SQL and direct scans resolve identical owners | Delete separate semantic source/role/computation inventories and repeated name policies after their consumers move; invert the current lookup/root probes into target acceptance |
| **PR03 — common provider admission and metadata**; PR02 | `session/admission.rs`, provider bindings/metadata, table/factory registration | Extensible actual-owner bindings, established facts, generated capabilities, native views/functions/file providers and ordinary information schema | Delete concrete admission roster and fixed settings-query bypass; forged metadata/provider tests plus native extension added without core edits |
| **PR04 — bind policy through all preparation**; PR03 | `session/{factory,preparation,functions,profile}.rs`, plan/codec boundaries | One effective native session; source/effect/implementation dependencies captured before rewriting; semantic rules cover inlined and scalar-only plans | Delete provider-local policy interpretations and unsupported preparation shortcuts; inlining/folding/codec/context tests |
| **PR05 — standard scan and owned execution**; PR03–PR04 | `provider/{table,pushdown,statistics}.rs`, candidate/computed/indexed sources, native streams | One scan planner, truthful hints/properties, reusable native sources, explicit materialization and resource ownership | Merge duplicated scan behavior where semantics match; retain candidate/admitted distinction; scan differential and lifecycle matrix passes |
| **PR06 — command and publication lifecycle**; PR04–PR05 | Catalog computation/publication, native mutation providers, authoring changes, command frontend | DDL/DML/Copy/factories are explicit operations; private multi-table state; complete obligations; conditional durable visibility | Delete separate mutation/validation/publication decisions; interruption, stale-base, duplicate attempt and uncertain-commit tests |
| **PR07 — integrate actual compiler and cold resolution**; PR02–PR06 | Compiler Driver/validator, P0–P10 bindings, rules/native algorithms, `store/pinned*` | Same provider operation framework for source admission, compilation, reuse and reopening; shared traversal/invocation owners | Delete separate producer-dispatch/result adaptation and per-stage context reconstruction once target consumers run; source-authored heater/mixer workflow and negative provenance cases |
| **PR08 — unify inspection and extension conformance**; PR03, PR05, PR07 | Rust inspection, `pse-py` stream/handles, generated Python contracts, metadata views | Same hierarchy, metadata and owned streams in Rust/SQL/Python; purpose/capability/plan explanations queryable | Delete direct parallel table-resolution/slicing path; cold read-only admission, partial drain and final array-owner tests |
| **PR09 — measure, remove remainder, close gates**; PR07–PR08 | Target workloads, deletion inventory, Plan 05 status, review evidence | Resolve measured duplication; complete operation coverage and final independent G1–G7 review | Zero legacy production consumers; named full Rust/Python/product gates, target cost receipts and exception census |

**Implementation sequencing:** PR02–PR04 provide the missing foundation before
further isolated functional patches. The P0 role rename already in progress is a
small correction, not the namespace design to propagate. Keep the source-authored
engineering workflow as the integration guide throughout PR05–PR08. Do not first
build a hypothetical remote platform or perform a feature-by-feature wrapper rewrite
without real consumers.

**Closure criterion:** every callable product data operation has a declared native
route, an actual implementation binding, effective policy, complete dependency/effect
contract, accountable resources and an observable outcome. No parallel provider,
inspection, compiler or mutation pathway survives merely to preserve old behavior.
Exceptions require the narrow evidence in §10. Target correctness and cost are
demonstrated by §9, and each gate is settled independently.
