---
title: Schema-first pivot of the existing codebase to native DataFusion and Delta
status: done
date: 2026-09-16
adrs: [ADR-0065, ADR-0066, ADR-0068, ADR-0069]
phase: 1
---

# Schema-first pivot of the existing codebase to native DataFusion and Delta

**Completed through Plan 09 — 2026-09-18.**
[Plan 09](09-native-caching-and-pivot-completion.md) implemented and qualified every
carried E00–E10 obligation, including the remaining caller/resource/recovery work,
legacy deletions and Q01–Q14/G1–G7 acceptance. Its final receipts are authoritative
for completion; the earlier source and planning checkpoints below remain historical.
No additional simulator functionality or predecessor data migration was included.

## Historical implementation checkpoint — 2026-09-17

**Implementation remains incomplete.** This is the architecture-only successor to
Plan 07. The schema, native invariant/rule, provider/compiler/publication and
Rust/Python caller replacements are implemented. The predecessor driver, closed rule
algebra, stores and product handles are deleted. Typed dependencies, conservative
reuse, bounded CDF, local reader protection and native maintenance are implemented,
with remaining caller/resource/recovery coverage and qualification work below.
**No SP package, E step or terminal architecture gate is certified closed.**

**Current checkpoint — 2026-09-17:** implementation is paused at the maintainer's
request to update this document. The next implementation action is to finish the
known cache lint and maintenance-fixture API fix, then complete the remaining
caller/declaration/deletion and resource audits. Earlier workspace compilation and
the latest isolated unit receipts do not certify the latest whole tree. Integration
qualification has not started for this completion pass.

Use [current remaining scope](#current-remaining-scope--2026-09-17) and
[the restart sequence](#implementation-sequence-and-deletion-checkpoint) as the
execution checklist. SP sections retain their target requirements with current
status summaries; [implementation history](#implementation-history) records earlier
source states, including superseded restart instructions.

**Maintainer-directed sequence:** complete all implementation and deletions first;
then run integration/current-function/terminal tests and fix the resulting failures.
Compilation, generation and focused unit checks support implementation. Repeated
integration campaigns between architecture replacements are no longer the workflow.

The functional guideposts are the existing process-model outcomes. There is no
compatibility layer, historical-object preservation, dual-write rollout or new
simulator scope. Useful derived graph algorithms remain eligible; predecessor graph
or store authority does not. Local DataFusion/Arrow and Delta skills remain the
library references; do not use Context7 for those families.

## Context

### Governing inputs and scope

- [Schema engineering review](../design_review/reviews/design_review_schema-engineering-typed-values_2026-09-16.md),
  findings **SE-F01–SE-F11** in this plan.
- [Unified DataFusion/Delta design review](../design_review/reviews/design_review_unified-datafusion-delta_2026-09-15.md),
  findings **UD-F01–UD-F14**.
- [Plan 07 completion review](../design_review/reviews/design_review_unified-datafusion-delta-completion_2026-09-16.md):
  current implementation, surviving caller closures and incomplete gates.
- [Provider capability map](../capability-maps/datafusion_provider_contracts.md),
  [DataFusion skill](../../.codex/skills/datafusion/SKILL.md) and
  [Delta Lake skill](../../.codex/skills/deltalake/SKILL.md), with exact source as tie-breaker.
- Blueprint §§3.3.3, 4–6, 14, 18, 20–22 and the existing decision records are inputs;
  SP00 records affected decisions before implementation changes. This plan does not
  silently amend accepted ADRs or the blueprint.

**In scope:** replace the architecture of existing authoring/editing, normalization,
P0–P10 responsibilities, native inference and symbolic construction, numerical
evaluation, the implemented Ipopt operator, provider/policy/inspection, persistence,
publication, change/reuse and Python/Arrow boundaries. Complete the schema review's
enhancements, including their application to declarations already present for future
functions; changing those declarations is not implementing those functions.

**Out of scope:** new process-model families, new property/kernel mathematics,
P11–P15 case/discretization/structural/scaling/initialization behavior, a new
source-to-solve simulator, expanded derivative/Hessian capabilities, generated Pyomo,
NL/SOL, new solver backends, remote catalog deployment and distributed execution.
An existing unsupported capability remains explicitly unsupported. Its stub or
declaration is not an excuse to retain old runtime objects. Remove obsolete API
placeholders; retain useful future declarations only under the new contract basis.

Existing functions remain usable through the target architecture. Their independent
domain assertions, source diagnostics, typed outcomes and resource/cancellation
behavior are the functional guideposts. Old bytes, object graphs, pass IDs, old store
layouts and predecessor-equivalence campaigns are not acceptance requirements.

### Present implementation and research boundary

**Implemented at this checkpoint:** product compiler and Rust/Python publication
paths use native plans and exact Delta selections. The predecessor custom catalog/
snapshot/stage/memo lifecycle and closed `RulePlan`/`RuleExpr` authority are deleted.
Native invariant/rule declarations, coherent math, generated numerical values,
typed keys/spans/support, actual provider admission and finite algorithm children
are implemented. Typed dependencies, conservative reuse, CDF and native maintenance
are also implemented. Remaining route/resource/recovery coverage, compile/static
closure and final qualification are detailed in the current E00–E10 checklist.

**Interface-checked:** DataFusion 55.1.0, Arrow/Parquet 59.3.0, object_store 0.13.2,
Delta `58f07cd62bfbce3649a7e1c87c696288068ae184`, kernel
`8ba063f8f84fec222000f66d40d70911d7c79675`. Research used the two local skills,
pinned Cargo sources, provider map and `just metadata`; no Context7 was used.
The [planning evidence](../design_review/evidence/schema-first-native-data-pivot-2026-09-16.json)
records selected hashes, canonical API references and coverage limits.

At plan creation, no runtime probe or benchmark established the proposed schema
changes. Existing component receipts in the completion review established their
recorded scope only. `just doctor` then reported **1 freshness failure, baseline 0**,
and a stale editable extension. Later `just py-sync` succeeded, but the latest startup
diagnostic again reports an outdated environment; refresh remains on the restart
checklist. Planning skill scans produced 28 DataFusion and 4 Delta advisory
syntax matches, including tests; these were leads, not defect counts.

### Completion outcomes for this plan

| ID | Architecture outcome | Required observable result |
|---|---|---|
| A01 | One recursive field/value contract | Layouts, local predicates, references, bindings and diagnostics derive from one declaration; invalid alternatives/collections cannot be silently admitted |
| A02 | Loss-aware execution/durable boundary | Exact field identity and explicit conversions; durable declarations describe their meaning without an identical in-memory registry; mismatches refuse |
| A03 | Structure carries correspondence | Typed keys/support/spans, ordered children/tagged payloads and numerical vectors replace JSON keys, ordinal side vectors and encoded column names |
| A04 | Native plans own data work | Existing transformations/validation/inference/evaluation execute through native plans/functions/extensions, without a closed parallel relational algebra |
| A05 | Complete common provider contract | Rust/SQL/Python, views, factories and no-scan plans use actual native configuration, providers, policy, effects and resources |
| A06 | Delta is the sole durable authority | Existing product callers publish/open exact members through one validating route and explicit recovery; old refs/manifests/CAS/stage stores are deleted |
| A07 | Target change/reuse/retention | Native dependency and CDF queries, qualified exact reuse, reader protection and Delta maintenance replace old memo/restoration mechanisms |
| A08 | Existing functions and extensibility qualify | Fresh-source current-function journeys and one ordinary schema/invariant/provider extension pass; no new simulator behavior is required |

## Decisions

### 1. One field tree, native types and a small set of domain declarations

Use Arrow `Field`/`DataType` as the physical type vocabulary. Refactor the current
top-level `ColumnSpec` plus `LogicalType::Struct` child tuples into one recursive
field contract capable of attaching domain meaning at every path. Keep named domain
types, alternatives, references and collection semantics as declarations; generate
their Arrow fields and metadata. Do not create a second editable field tree or a
general expression language to describe Arrow's own type system.

The authoritative contract supplies: field names and nullability, native storage,
domain type/version, alternatives, range/finite constraints, keys/references,
quantity binding, list order/cardinality/uniqueness and meaningful defaults.
Metadata, Rust/Python accessors, native predicate templates and docs are projections.
Immutable registry code may construct native `Expr`/`LogicalPlan` at binding time;
no runtime `RulePlan`/`RuleExpr` interpreter is retained for general relational work.

PSE's canonical stored ordinals/counts/versions use `Int64` with domain range checks;
FFI `i32`/`usize` narrowing occurs explicitly at its boundary. This is **not a ban on
Arrow unsigned types**: arbitrary native inputs remain eligible, with exact conversion
or explicit unsupported-durability rejection. Keep 16/32-byte semantic IDs/hashes
where their meaning requires them. Dictionary encoding is a physical option, not enum
identity; choose native string enum values with generated membership checks for the
canonical durable shape. Preserve exact UTC nanoseconds through the enabled Delta
feature and test the resulting table protocol.

### 2. Coupled values are typed structures, not nullable-column conventions

- `TaggedAlternative` defines a discriminator and optional arm structs; exactly the
  selected arm exists, its required children exist, other arms are absent, and unknown
  tags fail. An absent outer optional value is distinct from a present invalid value.
- Apply it to existing rule literals, configuration/feature values, kernel outcomes,
  incidence linearity/coefficient, equation bounds and implemented operation outcomes.
  Reuse one arm declaration for layouts, checks, Rust enum access and Python types.
- Make per-row quantities actual `quantity_value` structures, with value, quantity
  type and explicit unit/reference requirements. Do not invent unit IDs from missing
  data or attach one static field annotation to heterogeneous rows.
- Represent owned ordered children as lists and correlated entries as
  `list<struct<...>>`. Independent sets need membership/reference checks rather than
  artificial positional pairing. For an implicit system, a named square requirement
  may compare independent equation/variable cardinalities; do not imply a pairing
  solely because the lists have equal lengths.
- Null collection, empty collection, absent arm, unknown truth, unavailable numerical
  value and failed operation have distinct declared meanings. Required-child checks
  apply to visible children of a valid parent; masked storage is not a visible value.

### 3. Native validation, with explicit layers and no silent row dropping

Generate local scalar/struct predicates as native expressions and portable Delta
CHECK SQL where supported. Use Arrow structural checks and Delta's actual validation
plan before writing new validation kernels. Generate set-level PK/FK/closure and
nested-reference checks as native violation queries over exact providers. Keep a
domain UDF only for meaning not expressible with the native functions available at
the pin; it receives typed values and returns a typed value or structured failure.

Admission must reject invalid rows, including UNKNOWN when validity requires TRUE;
a `Filter` that merely removes malformed rows is not validation. Defaults apply only
to omitted values under the declaration, not indiscriminately to explicit NULL.
SQL `get_column_default` alone is insufficient for Arrow/programmatic writes.
Provider constraints/functional dependencies are advertised only after they are
established for the actual immutable input, never used as proof of their own truth.

Native field inference and full-field UDF hooks remain primary. One shared domain
transfer mechanism replaces metadata-only wrapper duplication. It does not recreate
a closed native-function allowlist or implement generic Arrow typing again. A native
operation may produce an ordinary unqualified Arrow field; it may not falsely claim
a PSE domain contract. Domain-changing arithmetic still needs its actual quantity,
basis, guard and approximation rules. A struct by itself does not prove those rules.

### 4. Durable declarations describe both meaning and the actual storage

Generate Delta `StructField` metadata and table properties for relation identity,
contract version/fingerprint and named layout conversion. Preserve native-compatible
semantic annotations, but **never attach an Arrow extension descriptor to an
incompatible physical storage type**. Current `pse.semantic_id` expects
`FixedSizeBinary(16)`; Delta stores `Binary`. Current enum storage similarly differs.
Store the logical descriptor and conversion version in generated PSE metadata where
necessary, and restore the valid execution extension only after checked conversion.
Same-storage extensions can retain their valid `ARROW:extension:*` annotations.

Cold opening checks stored declaration, selected contract and available implementation
explicitly. Matching contracts open; unknown/mismatched contracts refuse with typed
diagnostics. This pivot recreates development data and provides no historical
migration reader. Durable metadata must be intelligible without registered PSE UDFs;
that does not imply an external engine can implement every domain algorithm.

The exact admission guard compares names, field structure, nullability and relevant
contract metadata. Named conversions may deliberately change approved storage
details, such as Delta's list-item name or view-array representation. Validate these
against a generated conversion contract. Do not require raw equality across a known
lossless physical conversion, and do not use positional casts or permissive schema
evolution as a substitute for exact domain admission.

### 5. Typed identity and provenance have an explicit scope

Generate a stable key token from the declared ordered primary-key values, relation
identity and key-contract version, using the existing canonical framing machinery
where useful. It identifies a row within a relation; it is not a hash of all mutable
payload values or a Delta version. Keep actual key columns authoritative and visible.
Derived tokens have explicit collision/consistency checks, not assumed uniqueness.
An empty declared key means a declared singleton, with a scoped constant token.

References use typed `(selection, relation_id, row_key)` bindings, with selection
inherited once from an enclosing exact publication where appropriate. They cannot
borrow the same key from a different revision/catalog. Support and absence domains
remain typed data. JSON becomes optional diagnostic rendering, never join identity.
Spans and support travel with source/derived rows instead of parallel positional vectors.

### 6. Complete value reshaping; tune physical layouts with evidence

For every existing expression family, retain a node relation with stable node identity,
ordered `children`, opcode/tagged payload and declared quantity/guard information.
Delete canonical ordinal-edge and per-opcode payload side tables and their duplicated
joins. Preserve canonicalization/hash-consing only as useful derived algorithms; no
old graph object or old hash byte sequence is a migration obligation. Native UNNEST
and views expose edges/arms where consumers need relations, without another authority.

Numerical results use a declared scenario/program contract with residual and sparse
Jacobian vectors and a typed coordinate relation. Dimension-sized `FixedSizeList`
is appropriate for a fixed prepared program, including an explicit zero-dimension
policy; persisted cross-program data uses a variable list plus validated dimensions
when a single fixed width would be false. Coordinates declare uniqueness, range,
order and program identity. The existing Ipopt adapter consumes contiguous slices
and checked index conversions; it gains no new mathematical capability here.

Measure expression scans and callback allocations before selecting final physical
packing, then tune the **one** target representation. Keep hot discriminators as
top-level columns and use native views/projections. The pinned Delta scan does not
establish nested expression projection into file reads. A measured issue may justify
a narrow native integration improvement or generated physical projection; it does not
justify retaining the old canonical side-table/graph pathway. Contract simplification
is required; performance improvement is not claimed before measurement.

### Additional schema opportunities found during planning

These build on SE-F01–SE-F11; they are **Proposed**, supported by the named inspected
interfaces/callers. They are included in implementation and qualification below.

| ID | Evidence and opportunity | Planned correction and benefit | Verification |
|---|---|---|---|
| N01 — Recursive references and collections | `model/relation.rs` permits one scalar-column FK; nested struct children in `model/logical_type.rs` are only `(name,type,nullable)`; `delta/admission.rs` iterates top-level `spec.columns` | Recursive field paths, composite FK mappings, target scope, null policy and list cardinality/order/uniqueness declared once; native UNNEST/anti-join checks replace hand-maintained closure variants | Missing nested/composite target, null/empty cases, duplicates and wrong revision rejected; declared valid order retained |
| N02 — Physical extension compatibility | `model/extension.rs` declares fixed/dictionary storage; `delta.rs` converts it; Python `extensions.py.template` rejects a different storage type | Generate execution descriptor and durable logical descriptor from the same type; never label Binary as an incompatible fixed-size Arrow extension | Raw external Arrow read succeeds without PSE registration; registered consumer does not deserialize invalid storage; exact target restoration succeeds |
| N03 — Recursive nullability and exactness | Durable guard uses `equals_datatype`; native struct compatibility permits missing/extra fields; pinned Delta validation skips some paths through list/map elements and excludes FixedSizeList in its collection check | One exact nested contract plus explicit physical adaptations; fill native validation gaps using native higher-order/list plans where possible, narrowly scoped domain checks otherwise | Renamed/reordered/extra children, null struct parents, lists of structs, nested lists and fixed lists tested through all write/read paths |
| N04 — Publication and attempt values | `catalog/publication.rs` uses two nullable revision-selector fields; `publication_plan.rs` sets both absent and stores attempt text in commit metadata | Tagged member selection (`full` or typed revision slice), declarative current-artifact member profiles, and typed attempt/member outcomes/input identity; fewer ambiguous states in retry and cleanup | Half-selected revision impossible; missing profile member refused; matching retry resolves actual versions and changed-input retry refuses |
| N05 — Operation result contracts | `pse-backend-native/src/native/output.rs` hand-builds status, nullable diagnostics and parallel value/dual lists; `pse-compiler/src/records.rs` creates positional diagnostic cells | Generate existing numerical/command results and structured diagnostic evidence; couple variable/value/dual and constraint/residual entries or bind their typed program order explicitly | Swapped/truncated result vectors and inconsistent status/payload fail; existing cancelled/failed/finite outcomes remain distinguishable |
| N06 — Native defaults and derived facts | Provider map §8.4 shows defaults apply to SQL INSERT only; `terminal_attempts.rs` declares stored count-equals-list-length checks | One native input-normalization plan for omitted/defaulted fields; derive cheap counts in native views or qualified Delta generated columns instead of storing independently editable copies | SQL/Arrow writes agree for omitted vs explicit-null input; forged derived count cannot become authority |
| N07 — Arrow-native values beyond the closed cell catalog | `pse-relations/src/cells.rs` decodes/rebuilds batches as `Vec<Vec<Cell>>`; `LogicalType` rejects otherwise-native types | Native fields/batches internally, `ScalarValue` for native literals, generated typed access at parser/FFI boundaries; remove generic runtime Cell conversion and closed native type restrictions | Existing transforms run without batch→Cell→batch; supported Map/native types execute without inventing a domain extension; unsupported durability is an explicit boundary result |
| N08 — Native metadata and semantic dependencies | `session/config.rs` duplicates selected native settings; provider hooks/constraints and old memo interpret related facts separately | Read native configuration/introspection from actual bindings; generate domain views and typed input/policy/function/absence dependencies; advertise proven optimizer metadata once | Sentinel config survives, metadata matches scan, no-scan/view policy holds, changed dependency invalidates exact reuse |

N01 also replaces ad hoc `invariant_closure.rs` list-reference declarations where
they state a general field relationship. Truly domain-specific closure rules remain
native query definitions. N06 removes redundant meanings, not useful materialized
indexes or metrics whose actual consumer justifies their cost.

### Exact-pin capability decisions and cautions

| Library capability / canonical API | Use in this plan | Inspected evidence / limitation |
|---|---|---|
| `datafusion_expr::expr::Expr`, `ExprSchemable`, native builders | Native typing, nested expressions, checks, normalization and transformations | DataFusion skill `topics/expressions.md`, `api/datafusion_expr.expr_schema.md`; no parallel type interpreter |
| `datafusion_expr::udf::ScalarUDFImpl` | Actual full return fields, coercion/nullability/volatility, valid simplification; `struct_field_mapping` for surviving constructors | Skill `traits/ScalarUDFImpl.md`, `api/datafusion_expr.udf.md`; field mapping propagates ordering, not arbitrary lineage or domain proof |
| `datafusion_common::nested_struct::{cast_column,adapt_batch_to_schema,validate_struct_compatibility}` | Explicit compatible native adaptation after exact admission | Skill API page: compatibility allows missing/extra fields; **not** the exact schema validator |
| `datafusion_common::unnest::UnnestOptions`, higher-order array functions | Typed child/reference/support queries; deliberate null/empty/list behavior | Skill unnest API and SQL-function catalog; qualifier and ordinal meaning must survive |
| `datafusion_common::functional_dependencies::Constraints` | Publish proven PK/unique properties to the optimizer | Native constraints are declarations, not row validation; keep nullable-unique semantics exact |
| `datafusion_session::{catalog,schema,table}` provider traits, native registries/views/factories | Complete native hierarchy and common admission for actual bindings | Provider map; `prepare_insert` routes Rust/native Arrow input through native SQL INSERT default planning; consumed statistics use physical/file paths rather than relying on the unused provider statistics hook |
| `buoyant_kernel::schema::{StructType,StructField}` / Arrow conversions | Declared Delta fields and semantic metadata | Skill schema API and pinned `kernel/src/engine/arrow_conversion/mod.rs`; physical types still differ |
| `deltalake_core::operations::{write,constraints,create}` | Native validated writes, CHECK constraints, creation/properties and generated columns where qualified | Actual `DeltaTable` builders; no `DeltaOps`; constraint addition **does validate existing rows** at this pin, contrary to the skill topic's summary |
| Delta validation and generated-column execution | Reuse native null/check machinery; fill only demonstrated gaps | Pinned `delta_datafusion/data_validation.rs`, `operations/write/generated_columns.rs`; nested/fixed-list limits above and SQL NULL behavior require tests |
| Delta scan, DML, `DeltaExtensionPlanner` | Shared actual session, pruning, mutation hooks and visible native input plans | Skill DataFusion seam and provider map; public builder composition is used, private Delta planners are not copied |
| `deltalake_core::kernel::transaction::CommitProperties` | Actual effect identity and observed-version reconciliation | App transaction/attempt metadata alone is **not** proof a retried member write is skipped; qualify the complete operation |
| `CdfLoadBuilder`, `DeltaCdfTableProvider` | Typed version-range semantic change input | Preserve change type/version; handle preimages/postimages explicitly; CDF is not the reuse engine |
| `OptimizeBuilder`, checkpoints, `VacuumBuilder::with_keep_versions` | Native maintenance over live selected versions and reader pins | Data-file retention does not by itself retain control logs/checkpoints or prevent reader races |
| Delta codecs; native logical/provider descriptors | Supported exact rebinding and re-planning | Generic Delta logical codec arms/current physical scan coverage remain limited; refuse unsupported arms before calling them |

**Considered, not mandatory new implementations:** native Map is eligible, but it
does not enforce semantic key uniqueness by itself; Union is eligible for transient
execution but is not a Delta/Parquet durable layout; Variant has unqualified write/
query routes at the pin and is not the default for diagnostic evidence. Unity,
Flight/distributed execution, row tracking, column mapping/type widening and clustering
remain eligible where an actual current consumer warrants them. Feature enumeration
is not proof of writer support. Do not enable every table feature or build an adapter
for every remote system merely to demonstrate eligibility. Use native facilities for
the operations the current codebase actually needs.

### Policy and contract placement across the native hierarchy

This is a coverage obligation, not a mandate to write a wrapper at every level.
Prefer native implementations; add only PSE domain meaning and explicit admission.

| Native level | Standardized schema/policy responsibility | Closure |
|---|---|---|
| Runtime / object stores | Shared memory/spill/cache/task ownership, registered storage, explicit IO/backend guarantees and observed resources | SP08/SP12; Q09/Q13/Q14 |
| Session / configuration / functions | Actual native options/extensions and function implementations, domain defaults/requirements, effect/resource bounds | SP06/SP08; Q07/Q09 |
| Catalog list / catalog | Exact workspace/publication selection, native name registration/replacement and immutable invocation bindings | SP08/SP09/SP11; Q09/Q10/Q12 |
| Schema / async resolution | Names, owner/type/discovery, complete versus partial metadata, absence versus error, refresh into a new binding | SP08/SP11; Q09/Q12 |
| Table / source / view | Generated schema/defaults, admitted constraints, exact member selection, safe view inlining, scans and mutations | SP06–SP09; Q01/Q07–Q10 |
| Factories / table functions | Parameterized native opening/plans under the same contract; effectful work executes at its declared boundary | SP08/SP11; Q09/Q12 |
| Fields / expressions / analyzer / optimizer | Native typing/coercion and proven domain transfer, generated checks, dependencies and valid transformations | SP01–SP06/SP10; Q01–Q07/Q11 |
| Query / extension / physical planning | Real child graphs, actual composed Delta/PSE planners, explicit effects, truthful partition/order/statistics and cancellation | SP05/SP08/SP10; Q06/Q09/Q11 |
| Datasource / format / sink | Native Arrow/Parquet/Delta read/write, schema adaptation, residual filters/projection/limits and consumed statistics; no bespoke file-selection engine | SP07–SP09/SP12; Q08–Q10/Q13 |
| Delta table / transaction / maintenance | Generated features/properties/CHECKs, actual versions, member/root reconciliation, CDF and live-reference retention | SP07/SP09/SP12; Q07/Q08/Q10/Q13 |
| Streams / FFI / Python | Owned typed batches, backpressure/release/cancel, explicit conversion and capability diagnostics | SP05/SP11; Q06/Q12/Q14 |
| Metadata / EXPLAIN / serialization | Read-only projections of actual bindings/contracts/outcomes; supported exact descriptors and re-planning, no hidden effects | SP08/SP11; Q09/Q12/Q14 |

## Plan

### Execution direction — implementation before integration qualification

The maintainer's current instruction is to complete the remaining architecture
scope and its caller/schema/module deletions before running integration suites
again. During this cut, use compilation, generation and focused unit checks to
develop the target directly. Do not run repeated engine, end-to-end, conformance,
Python component or full-package integration campaigns between replacements.
Existing integration failures remain visible work for final qualification; they
do not require preserving predecessor execution paths or interrupting the pivot
to keep intermediate workflows green. Do not increase timeouts to establish a
passing receipt. Run integration and terminal architecture qualification only
after the implementation and deletion inventory is complete.

### Current remaining scope — 2026-09-17

**Historical checkpoint:** all scope below was subsequently completed by Plan 09.

**Evidence boundary:** this is a documentation checkpoint, not a completion receipt.
Implementation is paused. The completed work and named checks below supersede the
earlier interrupted invariant checkpoint. Current compile/static closure remains
open; no integration, compiler/storage/solver journey or broad qualification run
has started during this continuation. E10 is **not started**. Complete all remaining
implementation, caller replacement and deletions before integration tests, including
journeys housed in library test modules. Only isolated units support this phase.

#### Implemented foundations to retain and build upon

These are implemented changes with preceding scoped evidence, not closed packages:

- Recursive native Arrow field declarations, exact execution/storage comparisons,
  compatible extension descriptors, signed local integer/range contracts and generated
  Rust/Python/JSON-schema projections. Native external unsigned values remain eligible.
- Tagged configuration/features, quantity values, source owners, method declarations/
  provisions/producers, kernel outcomes, incidence, equation bounds and publication
  selections. Normalized expression-index domains, predicates and equation syntax now
  have complete generated alternatives. Comparisons use native lists of exactly two
  operands, avoiding the failed fixed-size-list Delta representation.
- Explicit singleton keys, pinned Arrow-row typed key tokens, nested/composite references,
  selected quantity validation, co-located source spans and algorithm/configuration
  support, and generated row/execution diagnostic evidence.
- Coherent expression-family rows with ordered children and tagged payloads; generated
  numerical programs, vectors, Jacobian coordinates and solver outcomes; existing
  evaluator/Ipopt consumers; exact selected-program dimension/coordinate admission.
- Native declared SQL row checks shared by candidate diagnostics, provider requirements
  and persisted Delta CHECK; registry-free declared-table reconstruction; explicit
  layout/contract identity; declared publication roots and exact retained member slices.
- Caller-native session configuration preservation, real Delta/PSE extension planner
  composition, native validating member writes and conditional complete-vector root
  publication/reconciliation. Product paths use the target boundary; the complete
  fixture/route audit and end-to-end qualification remain open.
- Native SQL invariants and all 130 native SQL inference declarations; complete
  method, transfer, conservation and case-target values; typed finite algorithms with
  actual requirement/value children. The closed rule algebra and compiler controller
  no longer provide an alternative execution path.
- Exact publication handles and owned Arrow streams in Rust/Python, registry-owned
  artifact completeness profiles, typed member-attempt/dependency receipts, guarded
  retained-generation reuse, CDF and coordinated local native maintenance.

Do not rebuild these changes or revive their deleted representations. Remaining work
is complete route/resource coverage, final deletion audit and qualification.

| Step | Packages | Current implementation state | Remaining completion boundary |
|---|---|---|---|
| E00 | SP00 | Current-function architecture acceptance command implemented; obsolete simulator command deleted | Finish operating guidance, decision reconciliation and complete fixture/deletion inventory |
| E01 | SP01–SP02 | Exact fields, signed values, complete alternatives and principal consumers implemented | Audit residual dependent optionals, collection contracts and generated remnants |
| E02 | SP03 | Typed keys/references/spans/support/diagnostics implemented; six native support units pass | Audit all selected-input/owner routes and complete fixed-point/producer-release coverage |
| E03 | SP04–SP05 | Coherent math and numerical contract/consumer cut implemented | Finish resource/effect/owner audit; numerical and cost qualification in E10 |
| E04 | SP06/SP10 | Native invariants/rules/field transfer and native insert defaults implemented; three production Cell conversions removed | Complete default/derived-value/function coverage and fixture audit; qualify durable routes in E10 |
| E05 | SP08 | Native hierarchy and common execution/effect boundary implemented; seven fixture surfaces now use common native preparation | Finish all-route admission/resource/cancellation audit and compile/static closure |
| E06 | SP07/SP09 | Complete artifact profiles, exact members and native member/root recovery implemented; cold unknown-generation reuse refuses | Finish recovery/history fixtures and qualify actual Delta outcomes in E10 |
| E07 | SP10 | Native compiler/source-edit composition implemented; Driver/stage/memo/producer records/change envelopes deleted | Finish residual conversion/dependency/caller audit and target fixture coverage |
| E08 | SP11 | Legacy stores/identities/handles and predecessor-only fixtures deleted; preceding all-target checks passed | Compile latest fixture/refactor edits; finish whole-workspace audit, generated/stub closure and ownership qualification |
| E09 | SP12 | Typed dependencies, guarded reuse, bounded CDF, leases and native retention/reclamation implemented; CDF/retention projection units pass | Finish recovery/history/resource coverage; latest reclamation fixture is unrun and has a known API mismatch |
| E10 | SP13 | Not started for this completion pass; historical/scoped receipts only | After all implementation/deletions: integration fixes, measurements, extension proof, Q01–Q14 and G1–G7 |

#### Latest completed changes and verification — 2026-09-17

**Implemented — native defaults and value transfer:**
`SnapshotSession::prepare_insert` binds a caller's native input through DataFusion's
insert planner. Native planning owns quoted column names, omitted defaults, explicit
NULL and coercion; the planning-only view is inlined and does not become a persistent
namespace. The original session still owns admission and source dependencies, with
no implicit execution-purpose upgrade. `WritableTable` forwards native constraints
and defaults. Generated `ArrowValue` now supports direct typed array conversion;
rule selection literals and the P7 contribution/P8 law-participation transfers no
longer round-trip through `Cell`. This mechanical codec does not grant domain validity.

**Implemented — native support and lifecycle projections:** filter/UNION and UNNEST
witness tests extend existing aggregate/window/anti/outer-join coverage. Bounded CDF
normalization retains all four change images, typed before/after keys and values,
signed commit versions and UTC nanosecond timestamps. Publication reachability and
retention unions use native queries and checked-value admission for declared fields,
including the reason tag. Empty membership and duplicate retention rows are covered.

**Implemented — remaining fixture routing:** the common helper in
`crates/pse-catalog/tests/support/native_execution.rs` prepares actual scan and DML
target providers while preserving the caller's native state. `coherent_math`,
`composite_reference_checks`, `declared_row_checks`, `native_value_contracts`,
`tagged_delta_values`, `unified_delta_dml` and `support/source_syntax` use it for PSE
execution. Raw upstream/registry-free contract probes remain useful where that is
their explicit subject; they do not certify a product-route bypass. Stored native
expression fixtures decode the recorded expression format. Independent scientific
and contract assertions remain; these integration fixtures have not executed.

**Implemented, unqualified — maintenance fixture:**
`crates/pse-catalog/tests/native_artifact_lifecycle.rs` now derives retention from the
actual publication, combines explicit retention, and adds an unpublished member
after a stale-parent rejection, native retirement/reclamation, rejected writer retry
and unchanged visible root assertions. It also retains exact reuse/cold refusal,
bounded CDF and active-reader checks. The latest expansion is **unrun** and has the
known call-site mismatch below. Control checkpoint/history pruning and additional
interrupted/empty-attempt cases still need coverage before E10.

**Tested:** baseline **0**, default Nextest/test profile, explicit
`pse-relations/force-validate`; isolated small in-memory units only. Excluded tests
are positive-selection exclusions, not ignored failures or integration receipts.

| Command | Result | Execution / run ID |
|---|---|---|
| `just unit-package pse-catalog 'test(session::commands::insert::tests::)'` | 2 passed, 0 failed, 92 excluded | 0.025 s; `2ba61d52-d76c-4474-ae27-0394e7893f07` |
| `just unit-package pse-rules 'test(plan::trace::tests::)'` | 6 passed, 0 failed, 7 excluded | 0.025 s; `44be6668-61ef-45ff-b566-75e4ab2e61cb` |
| `just unit-package pse-catalog 'test(delta::changes::tests::)'` | 1 passed, 0 failed, 94 excluded | 0.758 s; `59296432-74f6-4307-b73a-101e44925599` |
| `just unit-package pse-catalog 'test(delta::retention::tests::)'` | 1 passed, 0 failed, 95 excluded | 0.260 s; `86bb8025-54fc-4888-94b9-431f474b5430` |

Logs are local continuation receipts at `/tmp/pse-native-insert-units.log`,
`/tmp/pse-native-support-units.log`, `/tmp/pse-native-cdf-unit.log` and
`/tmp/pse-native-retention-unit.log`. They are not committed acceptance artifacts.

**Interface-checked, preceding source states:** `just check` passed workspace/all
targets in the dev profile in 1m09s before the latest transfer/projection/fixture and
lint refactors; `just check-library pse-compiler` passed after the three typed Arrow
conversion changes (1m36s including lock wait). `just check-package pse-catalog`
passed in 32.56s after fixture routing, before the latest lifecycle/refactor edits;
it reported helper-visibility warnings subsequently corrected in source. `just fmt`
and an editable `just py-sync` completed earlier; later edits require final refresh.
The upstream `proc-macro-error2 2.0.1` future-incompatibility warning remains visible.
These receipts do not establish current workspace, feature, Python or lint closure.

#### Known unfinished code and static checkpoint

1. **Observed failing gate:** latest `just clippy` (workspace/all targets, dev profile,
   `-D warnings`, baseline **0**) stops on one `clippy::type_complexity` finding in
   `crates/pse-catalog/src/session/cache.rs`, `CacheStore`. The same finding is reported
   for library and library-test compilation. Factor the entry/container type without
   changing weak completion ownership. Receipt: `/tmp/pse-native-pivot-clippy.log`.
   This is the first current blocking finding, **not** a total remaining workspace
   lint count: earlier compiler/rule fixture findings and unvisited crates still
   require reconciliation. The no-default-features branch was not reached.
2. **Source-inspected API mismatch:** `prepare_maintenance` now takes
   `&RelationPlan`; the new `ReclaimUnpublished` call in
   `crates/pse-catalog/tests/native_artifact_lifecycle.rs` still passes `retention`
   by value. Fix that call, then compile the full latest target tree. Do not describe
   the expanded fixture as compile-checked or executed.
3. **Open resource coverage:** review transient dependency-receipt decoding and
   growth, including JSON decode before allocation reservation; verify effect,
   cancellation/settlement and owner lifetimes across all remaining routes. Native
   Arrow batch accounting and leases are implemented, but they do not prove every
   transient allocation or metadata observation is accounted for.
4. **Environment/generation closure:** the latest startup diagnostic reports an
   outdated environment after the earlier successful `just py-sync`. Refresh through
   the repository recipe when implementation resumes, then regenerate affected
   outputs/stubs and complete static/feature checks. No environment repair, new build
   or test was run for this documentation-only checkpoint.

#### E00 — Finish scope and deletion ownership

**Implemented:** the target-only `architecture-acceptance` recipe and native inspection/
engineering entry points; the obsolete simulator command is deleted.

- Finish the existing-function fixture/oracle inventory and operating guidance. Keep
  all acceptance requirements within implemented process-model functionality.
- Reconcile final schema/identity/Python/publication changes with decision and generated
  governance inventories. ADR-0068/ADR-0069 remain **proposed** in their front matter;
  do not report acceptance or amend accepted decisions as routine cleanup.
- Update stale repository guidance that still points solely to Plan 05 and predecessor
  authority. The maintainer's Plan 08 authorization already governs this hard pivot.
- Close the caller/declaration/generator/fixture deletion inventory. Preserve useful
  independent scientific assertions; retain no predecessor-only layout or API tests.

#### E01 — Finish schema and generated-declaration audit

**Implemented:** complete selected-method outcomes, conservation subjects/coordinates/
participation, case/activation/observation targets, and their principal consumers.
Closed rule reflection and predecessor publication/stage/memo declarations are deleted.

- Audit remaining dependent optional fields, local integer roles, collection
  correspondence, quantity/reference declarations and stale flat-layout fixture
  constructors. Retain legitimate optional facts and native unsigned external values.
- Verify no orphan declarations, generators or exports remain for deleted authority.
  Regenerate affected surfaces from the authoritative declarations; do not re-encode
  removed representations or expand future simulator scope.

#### E02 — Complete identity, reference and support coverage

**Implemented:** exact typed keys/references, source spans, generated diagnostic
values and native support recovery. Six isolated native support tests cover aggregate,
window, anti/outer join, filter/UNION and UNNEST behavior.

- Audit exact publication/member/revision selection through compiler, rule, diagnostic
  and Python readers. A relation name or row token must not satisfy a reference to a
  different selection. Use the same candidate member vector for nested/composite FK,
  quantities and source-support admission; retain absence and ambiguity semantics.
- Complete remaining fixed-point and owned-support fixture coverage. Retain useful
  algorithm lookup structures; remove positional reconstruction or detached support
  authority if the caller audit finds any. Extend isolated units only for uncovered
  behavior; operator-unit success is not producer-release or publication certification.
- Close owned stream/producer-release behavior with E08; qualify Q03/Q04 in E10.

#### E03 — Finish math/numerical resource and boundary audit

**Implemented:** coherent expression rows, generated numerical programs/vectors/
coordinates/outcomes, exact selected-program admission and the existing native
algorithm/evaluator/Ipopt consumers.

- Audit remaining boundaries for exact program identity, vector dimensions/order,
  coordinate contracts, quantity/scoping and child order/multiplicity. Preserve
  existing symbolic semantics, limited-memory Ipopt behavior and explicit linked/
  unlinked capability outcomes. No new solver or derivative family is in scope.
- Finish effect, cancellation/settlement and owner-lifetime coverage with E05/E08.
  Keep transient graphs only where they serve an actual algorithm.
- Reserve actual numerical/derivative/failure journeys, callback allocation and
  nested-scan measurements for E10 after implementation and deletion closure.

#### E04 — Finish native invariant, transfer and default coverage

**Implemented:** native invariant declarations/binding/generated reflection; all 130
native SQL inference rules; deletion of `RulePlan`/`RuleExpr` and their type/truth
lowering/reflection. Actual function identity drives semantic field transfer; redundant
SQL-name twins are removed. `prepare_insert` uses native defaults/coercion, and direct
`ArrowValue` conversions replace three production Cell round-trips.

- Finish caller/fixture coverage for exact bound inputs, ordered offending keys,
  severity/identity and typed diagnostics. Missing inputs, false/unknown required
  checks and ambiguous selections must refuse through the common native boundary.
- Preserve domain four-valued truth, support/multiplicity, stratification and finite
  incomplete-result semantics. Retain the specialized fixed-point operator only for
  these domain semantics, with actual native children and dependencies.
- Audit semantic transfer across used projections, filters, joins, grouping, windows,
  UNION, UNNEST, casts and struct/map/array/lambda functions. Remove duplicate metadata
  restoration with no distinct role; retain demonstrated physical-boundary adapters
  and domain checks. Complete necessary field/coercion/nullability/volatility/order/
  simplification hooks without restricting arbitrary ordinary native functions.
- Finish write-caller coverage for omitted defaults versus explicit NULL and for
  derived values. Use the implemented native insert route, native expressions and
  views where appropriate; add no parallel default evaluator or derived-field store.
  The two in-memory insert units establish local behavior; durable SQL/Rust/Arrow
  parity remains a final Q07 obligation.
- Audit remaining production Cell conversions: typed parser/algorithm/FFI boundaries
  can remain where needed; relational reassembly without such a role must go.

#### E05 — Finish common provider routing and resource coverage

**Implemented:** the actual native catalog-list/catalog/schema/table hierarchy,
caller-state preservation, lazy native commands, real operation children and common
requirements/effects/ownership. Root `OperationNode`/`NativeOperation` callbacks are
removed. Seven additional fixture surfaces now use common native preparation.

- Audit direct Rust/SQL/Python reads and writes, publication open, inlined views,
  no-scan plans, quoted names, factories, parameterized sources and async resolution.
  Keep actual native inventories authoritative; remove any residual mirrored settings,
  hidden default sessions or product admission bypasses found by the audit.
- Finish native write/DML fixture routing across the workspace. Distinguish intentional
  upstream/registry-free probes from product flows that must use common preparation.
  Preserve actual caller functions/rules/configuration, pools and object stores.
- Complete effects, cancellation and committed-effect settlement coverage, including
  nested Delta/domain/solver extensions without an extension-name/downcast roster.
- Audit truthful constraints/defaults, pushdown/residuals, statistics/order/partition
  propagation, streaming/backpressure and budgets. Review dependency-receipt decoding
  allocations and growth, not just Arrow batch reservations and cache buffers.

#### E06 — Finish recovery coverage for declared Delta artifacts

**Implemented:** exact declarations and table policy, artifact completeness profiles,
complete candidate vectors, retained member slices, typed member attempts, native
commit/application-transaction reconciliation and conditional root publication.
Version-2 control transactions index exact attempt/publication versions; matching rows
are still verified. Cold unknown-generation reuse conservatively refuses.

- Audit native DML/table/schema-operation coverage and cold declaration/profile
  admission. Ordinary writes require the current exact contract; explicit schema
  changes cannot silently infer or replace it. Apply local/nested/domain/numerical
  requirements to the entire candidate vector before control publication.
- Finish independent failure/retry fixtures for provisioned, written-but-unpublished,
  rejected, unresolved and acknowledged/lost-response outcomes. Keep actual input,
  policy/function/provider and retained-operation identity in the recovery proof.
  A plan fingerprint or attempt label alone is insufficient; do not invent portable
  identity for opaque implementations to make a cold reuse test pass.
- Complete control transaction-index checkpoint/history-pruning coverage with E09.
  Native transaction retention and row verification are implemented, but recovery
  after actual log cleanup has not been qualified. Execute these journeys only in E10.

#### E07 — Finish native compiler and source-edit caller audit

**Implemented:** native source/edit/rename and typed finite algorithm composition,
real pre/postcondition children, invocation caching and exact consumed dependencies.
`Driver`, `PipelineRequest`, `StageDag`, memo/producer records/validator, P0/P1/P2
adapters, authoring candidate envelopes and change-operation schemas are deleted.

- Finish the source/fixture/consumer audit for existing normalization, inference,
  property demand, contribution, equation and math outcomes. Preserve parsing and
  useful typed algorithms; remove residual relational Cell reassembly where native
  expressions or generated Arrow values suffice.
- Verify source edits retain exact before-images and identity-bound reference edits,
  and that every algorithm carries actual argument/requirement dependencies into
  the common preparation boundary. Keep target end-to-end fixtures ready for E10.
- Confirm no old pass records, bundles, scheduler exports, memo/hints or cold-open
  stage replay remain reachable. Use explicit fresh computation whenever exact reuse
  is unqualified; no predecessor fallback is allowed.

#### E08 — Finish Rust/Python ownership and deletion closure

**Implemented:** target publication/native-operation handles and Arrow streams;
replacement of product `Store`/`Snapshot`/`Manifest` handles, snapshot codecs and
inspection/build/engineering callers; deletion of predecessor catalog store/control/
ref/manifest/encoding/membership/sidecar modules and predecessor-only fixtures.
Preceding engine/conformance/lifecycle/catalog all-target compilation succeeded.

- Fix the known latest maintenance fixture call and cache lint, then finish compilation
  and strict linting for the full latest tree. Earlier checks predate the newest
  fixture/refactor changes and do not cover every remaining compiler/rule lint.
- Audit Rust/Python exports, logical/provider descriptor codecs, inspectors, test
  support, manifests and generated remnants. Unknown versions/extension arms must
  refuse; fresh reconstruction/EXPLAIN must not mutate or resurrect old identities.
- Finish owned-stream/producer/runtime drop, cancellation, abandonment and external
  Arrow fixture coverage. Qualify separate-process Rust/Python cold open only in E10.
- Refresh editable extension/stubs and regenerate affected declarations. Close
  quality/import/documentation drift on current source. Retain native providers or
  sessions that serve the target; a `Snapshot` substring alone is no deletion rule.

#### E09 — Finish native change, reuse and maintenance coverage

**Implemented:** typed exact dependencies including absence/settings/policy/function/
provider/metadata coverage; native guarded retained-generation reuse; bounded CDF;
local read/write/maintenance leases; native publication/explicit retention queries;
Delta optimize/checkpoint/vacuum/log cleanup and unpublished-attempt retirement.
CDF image and retention projection units pass; actual Delta lifecycle is unqualified.

- Finish dependency consumer/resource auditing. Keep unknown/volatile observations
  explicitly fresh, preserve empty/unmatched dependencies, and refuse unqualified
  cold-generation reuse. Missing receipt/history evidence requires recomputation or
  refusal, never an empty-diff or invented identity assumption.
- Finish the expanded reclamation fixture and coverage for interrupted/empty attempts,
  control identity indices surviving checkpoints/log pruning, retained CDF ranges and
  active-reader/data/history protection. The newest combined fixture is unrun and
  has a known by-value retention argument to fix before compilation.
- Confirm every read/stream/command retains ownership through actual execution and
  maintenance uses the same local exclusion boundary. Refuse unsupported remote
  destructive-coordination guarantees; do not introduce a remote platform.
- Audit native retention defaults, receipt lifetime/growth and cleanup of target
  versions. No predecessor data retention, custom Parquet deletion or shortened
  retention workaround is part of this plan. Reserve actual Delta/CDF/reuse/recovery
  and compaction/reclamation journeys for E10.

#### E10 — Final qualification after implementation and deletions

1. Establish that E00–E09 implementation and deletion obligations are complete by
   source/caller/declaration/fixture inventory; compile and regenerate the target.
2. Execute the final current-function integration journeys: fresh source/edit to
   existing compiled math, implemented numerical/solver cases, target publication,
   cold Rust/Python/Arrow access, exact dependencies/CDF/reuse and cleanup.
3. Fix failures in the target implementation or obsolete fixtures while retaining
   independent useful assertions. Do not restore old code, disable valid checks,
   accept a warning baseline or raise timeouts to manufacture a pass. Rerun affected
   gates after fixes; avoid duplicate whole-suite runs without a relevant reason.
4. Prove ordinary extension locality: one tagged value/reference/invariant and one
   native expression/provider/parameterized relation through the same declaration,
   admission, persistence, effects and inspection boundaries, without new dispatch.
5. Measure small/scaled planning/execution, Delta IO/replay/file growth, nested/coherent
   scans, callback allocations and peak/retained memory. Profile actual target
   bottlenecks. Earlier multi-minute small fixtures are unresolved failures of the
   current performance envelope, not an accepted target or a DataFusion benchmark.
6. Run final zero-baseline quality, generation, governance, Python, documentation,
   required feature/build and linked/unlinked checks. Record named commands, actual
   source/feature/profile boundaries and counts. Reassess Q01–Q14 and G1–G7 independently.

#### Implementation sequence and deletion checkpoint

Resume in this order; the current user request pauses execution at documentation:

1. **Restore compile/static closure.** Fix `CacheStore` type complexity and the final
   `prepare_maintenance` reference argument. Refresh the stale environment through
   `just py-sync` when needed. Compile the latest workspace/fixture source, finish
   remaining strict lint findings and reach the required feature branches. An early
   catalog lint failure is not evidence that later compiler/rule targets are clean.
2. **Complete E01–E09 coverage and any resulting implementation.** Audit source/field/
   default/derived-value/support routes, native preparation and actual dependencies;
   finish resource/effect/cancellation/owner coverage and the recovery/history/
   reclamation fixture gaps. Use compilation, generation and isolated unit tests only.
3. **Close deletion, generation and governance inventory.** Verify no predecessor
   caller/schema/export/generator/fixture remains reachable; retain independent domain
   assertions. Refresh target generated contracts, Python stubs and operating guidance.
   Reconcile E00 decisions without fabricating ADR acceptance. Record the final
   current-function oracle and ordinary-extension coverage before entering E10.
4. **Only after all implementation and deletions are actioned, execute E10.** Run one
   consolidated current-function integration/architecture campaign; fix target failures
   and rerun affected gates when relevant. Finish measurements and independent Q/G
   decisions. No raised timeouts, legacy fallback or future simulator prerequisite.

| Remaining closure | Owning step | Concrete completion boundary |
|---|---|---|
| Latest compile/lint/environment gaps | E05/E08 | Known cache lint and maintenance call fixed; current target/feature compilation and lint receipts, regenerated extension/stubs |
| Native default/derived-field/transfer and support route audit | E01–E05/E07 | All current consumers use native contracts; necessary domain/algorithm adapters have a distinct role; uncovered isolated cases addressed |
| Remaining raw product/fixture preparation paths | E05/E08 | Product reads/writes/DML use common admission; intentional upstream/registry-free probes explicitly distinguished |
| Dependency allocations, metadata observations, effects and ownership | E03/E05/E09 | Exact facts/freshness retained and transient resources/cancellation/lifetimes accounted for across current routes |
| Member/root recovery, checkpoint/history and reclamation fixtures | E06/E09 | Latest combined fixture compiles; interrupted/empty attempts, identity-index history cleanup and retained CDF cases have independent oracles, ready for E10 |
| Residual legacy references and generated/governance/docs drift | E00/E01/E08 | Caller/declaration/generator/export/fixture audit closed; target guidance and decision status accurate |
| Cold ownership, durable behavior, costs and terminal gates | E10 | Fresh integration/measurement/quality receipts and independent Q01–Q14/G1–G7 decisions after implementation closure |

**Already deleted:** closed RulePlan/RuleExpr and reflection, root OperationNode/
NativeOperation callbacks, Driver/controller/stage graph/memo/records/validator, source
candidate/change envelopes, predecessor catalog store/snapshot/manifest/ref/control/
encoding modules, Python Store/Snapshot/Manifest handles, source operation receipts,
producer stage bundles/records and snapshot hash/membership/revision sidecars.
Predecessor-only stage/memo/terminal/sidecar/ref/manifest fixtures and the obsolete
simulator acceptance entry point are also removed. Three production Cell conversion
sites now use typed Arrow values. Retained mechanical codecs and useful typed
parser/algorithm/FFI boundaries are not a second relational authority.

These deletions are implemented facts, not proof of whole-workspace audit or end-to-end
behavior. Useful domain algorithms, native providers, Arrow ownership and exact field
contracts remain. No integration test ran for this documentation update.

### Milestones and dependencies

| Milestone | Packages | Exit |
|---|---|---|
| M1 — coherent schema basis | SP00–SP05 | Existing declarations and consumers use exact recursive contracts, typed values/keys/support, coherent math and numerical structures |
| M2 — native validation and durability | SP06–SP09 | Common native admission and self-describing Delta tables/publications work with schema-level negative and recovery cases |
| M3 — predecessor removal | SP10–SP11, using M2 | Existing authoring/compiler/numerical/Python paths run on the target; old store/driver/algebra/handles and callers are absent |
| M4 — complete architecture lifecycle | SP12–SP13 | Target change/retention/reconstruction, extension proof, cost evidence and independent architecture gates close |

Sequence is dependency-ordered work by one implementation stream, not separate
releases. SP08–SP11 form one caller-replacement cut: new internals may be built before
callers switch, but no compatibility bridge, dual write or mixed-authority product is
an accepted intermediate deliverable. Temporary compile failures during a coupled
schema/caller change are preferable to preserving predecessor forms.

The SP package bullets below retain the original target requirements and exits; they
are not a list of unfinished actions. Their status lines and the current E00–E10
checklist distinguish implementation already landed from remaining work.

### SP00 — Freeze current-function scope and record changed contracts

**Depends on:** none. **Status:** Acceptance command and native entry points implemented; final oracle/deletion inventory, operating guidance and decision reconciliation remain in E00.

- Inventory executable current functions and their source fixtures/independent
  assertions. Include existing authoring/edit/rename, normalization/P0–P10 outcomes,
  inference/support, scalar evaluation and the linked/unlinked native solver cases.
  Do not count a declared schema or stub as an implemented simulator function.
- Record schema/identity/metadata/Python/commit changes in an ADR and explicit blueprint
  revision as required; use proposed ADR-0068 coherently or supersede affected accepted
  decisions with a new record. Include signed ordinal and key framing changes affected
  by ADR-0050. No acceptance status is fabricated and no historical-data migration is
  required. Update operating guidance to this narrowed execution scope.
- Specify the architecture acceptance command by reusing ordinary tests/xtask; replace
  the current terminal command's dependency on nonexistent future simulator targets.
  Preserve the future simulator inventory in Plan 07 as unscheduled work, not a gate
  that this architecture plan must satisfy.
- Add exact-pin/evidence labels to the schema design essay (SE-F11). Freeze reproducible
  synthetic nested-value, expression and existing numerical measurement fixtures.
- Refresh the local environment before tests; record current failures without adopting
  baselines. Preserve unrelated dirty work.

**Surfaces:** decision/governance docs, schema-generation manifests, existing test/xtask
entry points. **Exit:** A01–A08 fixture/oracle and deletion inventory exists; decisions
precede affected code; no new simulator behavior is on the execution checklist.

### SP01 — Exact recursive field contracts and boundary safety

**Depends on:** SP00. **Status:** Exact recursive field foundations implemented; remaining declaration/route audit and Q01/Q08 qualification are E01/E05/E06/E08/E10.

- Introduce the recursive field declaration using native Arrow types plus named domain
  facets. Replace bare nested tuples/top-level-only semantics; derive metadata rather
  than storing a second editable copy. Expose native fields without a closed list of
  eligible engine types.
- Generate exact execution/storage field comparison and named physical conversion
  descriptions. Refuse renamed, duplicate, missing, extra or semantically mismatched
  fields; allow only documented reversible view/list/storage adaptations.
- Replace the `equals_datatype` admission shortcut before broader layout changes.
  Check nested parent masks and required visible children. Use native adapters after
  admission, never as permission to relabel values.
- Update Rust/Python/Arrow schema generators and regeneration bootstrap together.

**Surfaces:** `pse-schema/src/{model,arrow,delta,codegen}`, `pse-relations` field
admission, `pse-catalog/src/delta/layout.rs`, Python extension generator.
**Delete:** duplicate nested shape/metadata interpretation and permissive durable
guard. **Exit:** architecture schema negative tests reject renamed children and false
extension storage; all generated field projections agree, including nested paths.

### SP02 — Canonical scalar types, alternatives and quantities

**Depends on:** SP01. **Status:** Signed/value/collection and complete coupled-value consumers implemented; residual field/fixture/generated-remnant audit remains in E01/E04/E08.

- Convert PSE durable ordinals/counts/versions and ordinal/span extensions to signed
  canonical storage with generated nonnegative/width bounds. Keep checked external
  unsigned support and checked Ipopt index narrowing; do not cast silently.
- Declare tagged alternatives and apply them to all SE-F01 sites and existing config/
  feature values. Fold discriminated payloads into generated value types and accessors.
- Replace `PerRow` sibling naming with typed quantity structures and explicit unit/
  reference obligations. Preserve homogeneous column-level quantities where correct.
- Declare collection null/empty/order/cardinality/uniqueness and implicit-system
  cardinality relationships; generate their checks, not another validation DSL.
- Generate compatible execution and durable enum/identity/extension representations.

**Surfaces:** `pse-schema/src/model/{relation,field,extension}`, relevant
`catalog/s6_*` declarations, `pse-ids` framing, generated Rust/Python consumers.
**Delete:** tag/payload writer conventions, sibling quantity rules, unsigned-only PSE
ordinal conversions and duplicated alternative validators.
**Exit:** wrong arm, unknown tag, hidden/visible null, negative/out-of-range ordinal,
wrong unit and inconsistent cardinality fixtures have one declared rejection contract.

### SP03 — Typed keys, references, support, spans and diagnostics

**Depends on:** SP02. **Status:** Typed keys/references/spans/diagnostics and native support implemented; six support units pass. Complete selected-input and ownership coverage in E02/E05/E08, then qualify in E10.

- Generate row-key computation from actual primary-key declarations; keep key values
  and selection scope explicit. Specify collision/consistency and key-contract version
  behavior; an unchanged key survives payload edits, while its selected revision changes.
- Generate nested/composite reference violation queries with appropriate native
  UNNEST/anti-join, exact scope and explicit null/empty treatment. Use actual target
  unique keys rather than assuming any referenced column is unique.
- Replace JSON `row_key`/`undecided` identity and per-row key encoders with typed tokens
  and support references. Render diagnostic JSON only at a display boundary if useful.
- Emit source spans on authored rows and carry support with derived results. Replace
  coefficient/index/ownership reconstruction by projections, joins and nested values
  while the necessary facts are already available in the plan.
- Use generated diagnostic builders and typed evidence; eliminate positional nine-cell
  records and fields whose only meaning is encoded inside diagnostic JSON text.

**Surfaces:** `pse-rules/src/{derivations,invariants,plan}`, catalog scalar encoders,
`pse-authoring/src/{document,p1}`, compiler records/coefficient/index/native outputs,
schema provenance/diagnostic declarations.
**Delete:** semantic JSON key codec and joins, row-span side vectors, general per-row
support relocation and positional diagnostic assembly.
**Exit:** shuffled/sliced/chunked input preserves spans/support; wrong-revision and
nested FK cases fail; native provenance plans contain no JSON-key encoding.

### SP04 — Coherent expression rows and native symbolic access

**Depends on:** SP02–SP03. **Status:** Coherent expression rows and consumer replacement implemented; residual native boundary/resource audit and final semantics/cost qualification remain in E03/E07/E10.

- Make each existing expression-family node contain ordered children and one declared
  tagged payload, retaining explicit references/quantities/guards actually supported.
  Generate opcode arity, child FK, payload and cycle/shape obligations as appropriate.
- Change all existing math construction, canonicalization, quantity analysis,
  template lowering, scalar evaluation and inspection consumers in the same schema cut.
  Use native list/struct access and derived edge views for relational consumers.
- Keep derived graph/sparse layouts only where a real algorithm consumes them; release
  their producers and never persist predecessor graph authority.
- Measure opcode-only, arm-only and whole-node scans with the pinned Delta route;
  account for file bytes, planning time and peak/retained memory. Keep physical tuning
  and any narrow projection integration within the one new authoritative representation.

**Surfaces:** `pse-schema/src/catalog/{s6_9_math,expr_family}`, math codegen,
`pse-mathir`, existing compiler/template/quantity/numerics consumers and Python views.
**Delete:** canonical `*_expr_args` and per-opcode payload relation families, old
load/sink join APIs and fixtures whose only assertion is their former layout.
**Exit:** existing expression meanings, operand order/multiplicity and canonicalization
oracles pass with one coherent node value; every old consumer is replaced or deleted.

### SP05 — Type the existing numerical and operation outputs

**Depends on:** SP02–SP04. **Status:** Generated numerical programs/vectors/coordinates/outcomes and native consumers implemented; remaining effect/owner/resource coverage is E03/E05/E08 and actual numerical qualification is E10.

- Generate the current evaluation program contract: program/dimension identity,
  residual vector, sparse Jacobian vector and typed coordinate relation. Validate
  coordinate ranges/order/uniqueness and vector lengths at preparation.
- Expose vector outputs through native expressions/physical execution and adapt the
  existing Ipopt callback to contiguous typed buffers, with no per-callback replanning.
- Generate current solver status/diagnostic/iteration/result schemas; explicitly bind
  values/duals/residuals to variable/constraint identity or the exact program ordering.
  Define unavailable/nonfinite/partial values without conflating them with success.
- Preserve the existing limited-memory policy, actual FFI, cancellation/settlement,
  ownership and linked/unlinked capability refusal. Qualify allocation/release behavior
  separately from numerical correctness, using current analytic problems only.

**Surfaces:** `pse-numerics/src/expressions.rs`, native backend output/workspace,
runtime/numerical declarations and generators.
**Delete:** `residual_<n>`/`jacobian_<r>_<c>` contract names, separate positional mapping
conventions and handwritten outcome schemas.
**Exit:** existing derivative/solve/failure/cancellation cases pass; schema/dimension
negatives refuse; callback allocation and retained-memory measurements are recorded.
No case builder, new derivative family or additional backend is implemented.

### SP06 — Native invariant plans and one domain transfer mechanism

**Depends on:** SP01–SP05. **Status:** Native row checks, relational invariants, rule binding, semantic transfer and native insert defaults implemented; closed rule algebra/reflection deleted. Remaining function/default/fixture coverage and qualification are E04/E10.

- Lower alternative/range/enum/collection/default/reference declarations to native
  expressions and violation plans. Generate portable CHECK SQL where possible and a
  bounded domain function only where native functionality cannot carry the meaning.
- Remove the requirement that `InvariantSpec` name a `RulePlan` declaration. Bind
  native plan definitions to actual providers; retain domain invariant identity,
  severity, typed violating keys and structured diagnostics.
- Unify omitted-column defaults and generated/derived fields for SQL and Arrow writes.
  Use views for cheap derived counts; use Delta generated columns only with qualified
  deterministic expressions and write-route parity, not as a second authority.
- Replace metadata-only function twins with native inference plus the single domain
  transfer implementation. Exercise projection/filter/join/group/window/UNION/unnest/
  casts and used higher-order/struct/map functions; preserve native function eligibility.
- Add valid full-field, coercion, nullability, volatility, ordering and simplification
  hooks to genuinely domain-specific functions; delete redundant wrapper behavior.

**Surfaces:** schema invariant/rule declarations, `pse-rules` invariant construction,
`pse-catalog/src/session/{admission,scalar,physical_fields,output}`, Delta contract code.
**Delete:** general rule-expression truth/type interpreter and duplicated transfer
branches as consumers move; retain only domain truth/support/termination semantics.
**Exit:** native plans enforce declared checks with structured failure; portable
checks run without PSE UDF registration; arbitrary ordinary native functions do not
require a metadata-preserving twin; no invalid row is silently filtered away.

### SP07 — Persist the complete declared Delta contract

**Depends on:** SP01–SP06. **Status:** Declared member/root tables, policies, durable fields and cold CHECK reconstruction implemented; complete route audit and final cold/durable qualification remain in E05/E06/E08/E10.

- Generate Delta fields/metadata, table identity/version/fingerprint/layout properties,
  native CHECK constraints and required feature declarations from the authoritative schema.
- Use public create/write/constraint/metadata builders under the actual session.
  Explicitly select schema mode: ordinary writes require the current exact contract;
  deliberate target schema changes are explicit commands, not automatic inference.
- Reconcile native field metadata and physical adaptation at cold open. Test raw
  external Arrow reading and valid registered extension restoration, without requiring
  a separately published Python Delta release incompatible with the pinned Rust stack.
- Reuse native Delta nullability/constraints/generated expressions; qualify uncovered
  nested collection paths and values, UTC nanoseconds, dictionaries, widths and masks.
- Replace contract-fingerprint-in-UDF-name as the only durable identity. Make local
  predicates queryable/describable even when a residual domain function is needed.

**Surfaces:** `pse-schema/src/delta.rs`, catalog `delta/{layout,contract,write,dml,provider}`,
table creation/opening and generated contracts.
**Delete:** metadata stripping, unnecessary integer casts, the monolithic opaque
CHECK for predicates now expressed natively and implicit schema-change routes.
**Exit:** target-only cold tables expose/reconcile their contract; mismatches and
invalid rows reject consistently across Rust/SQL/Arrow entry points.

### SP08 — One actual native hierarchy, configuration and execution boundary

**Depends on:** SP06–SP07. **Status:** Native hierarchy, caller state, composed planners and common execution boundary implemented; root callbacks deleted. Remaining route/resource/cancellation audit and qualification are E05/E08/E10.

- Use native catalog-list/catalog/schema/table registries and views for actual bound
  providers; common metadata derives from those objects. Cover names/aliases/quoted
  names, immutable replacement, absent/error distinction, async resolution, factories
  and parameterized relations without a second name inventory.
- Preserve caller `SessionStateBuilder` settings/extensions/functions/rules and shared
  resources; apply explicit domain overrides using native configuration, with conflicts
  reported. Replace manually mirrored native settings as authority.
- Route Rust/SQL/Python, direct publication reads, view inlining and no-scan plans
  through common admission. Restrict raw assembly helpers to internal use.
- Compose Delta and all implemented PSE extension planners normally, including the
  existing solver where built. Declare effects through the operation's contract and
  actual native path, not an ever-growing downcast roster or generic callback engine.
- Use native scans, pruning, statistics propagation and stream/backpressure properties;
  publish only proven constraints/exactness. Preserve cancel/settlement/ownership.

**Surfaces:** catalog session/providers/metadata/resolution, `pse-runtime` session
factory, implemented native planners and Python runtime assembly.
**Delete:** duplicate inventories and default-session fallbacks. Root-only
`OperationNode` dispatch and public unadmitted product execution helpers disappear
with their callers in the complete SP08–SP11 cut; SP08 is not independently accepted
while those callers still require the predecessor boundary.
**Exit:** sentinel configuration/rules/functions survive; policy/effects apply to nested
operations and all interfaces; EXPLAIN/discovery/reconstruction remain non-mutating.

### SP09 — Typed Delta attempts, exact publication and recovery

**Depends on:** SP03, SP07–SP08. **Status:** Exact profiles/roots/members and native member/root attempt recovery implemented, including conservative cold-generation refusal; recovery/history coverage and actual Delta qualification remain in E06/E09/E10.

- Generate exact member selectors, current-artifact completeness profiles and typed
  attempt/member outcomes. Profiles describe existing source/normalized/compiled and
  numerical artifacts only; absent future simulator routes remain unsupported.
- Compose actual native writes and unchanged exact members into one candidate vector.
  Validate local, nested/composite and declared domain obligations against that vector.
  Preserve useful checkpoints, not a transaction for every former pass.
- Bind attempt identity to the exact inputs/contracts/policies/operation, record actual
  committed versions and reconcile member writes before retry. Reject identity reuse
  with changed inputs; distinguish committed, unpublished, rejected and unresolved.
- Publish the complete vector by qualified conditional Delta control commit; validate
  expected parent, concurrent first create and post-commit acknowledgment failures.
- Expose native DML and required table/schema operations through the same validating
  route; opaque output bytes use a typed relation/provider boundary where needed.

**Surfaces:** schema publication/attempt/dependency declarations, catalog Delta plans,
commands/control/recovery and old publication callers.
**Delete:** custom publication journal/ref/CAS semantics when the SP08–SP11 caller cut
lands; no bridge translating old manifests into target roots.
**Exit:** complete current-artifact publication, member reuse/slices and failure-after-
each-effect tests pass, with exact cold reopening and no blind retry of unknown effects.

### SP10 — Move existing authoring and compilation to native plans

**Depends on:** SP03–SP04, SP06, SP08–SP09. **Status:** Native compiler/source-edit/algorithm composition implemented; Driver/PipelineRequest/StageDag/memo/producer authority and closed rule algebra deleted. Residual caller/conversion/dependency audit remains in E04/E07 before E10.

- Keep useful parsing and domain algorithms, but express existing correspondence,
  defaults, selection, normalization, topology/property-demand/contribution/math
  transformations as native plans/functions over exact providers.
- Replace source/change candidate envelopes with typed rows and native command plans;
  preserve identity-bound rename, stale-before-image refusal and structured spans.
- Bind native rule/inference plans directly. Keep the one specialized native fixed-point
  physical algorithm only where native recursion cannot express its required truth,
  multiplicity/support, stratification and finite incomplete-result behavior.
- Replace `Driver`/`PipelineRequest`/`StageDag` execution authority with composition of
  native plans and explicit output contracts. No new stage interpreter or scheduler.
  Remove batch→Cell→batch transformations where no parser/algorithm/FFI boundary requires
  materialization. A bounded typed algorithm layout remains allowed.
- Replace memo callers with exact dependency-bound native results; initially recompute
  conservatively where reuse is not yet qualified, without retaining old stage hints.

**Surfaces:** `pse-authoring`, `pse-compiler/{driver,passes,inputs,records,...}`,
`pse-rules`, relevant math/template/quantity/material consumers.
**Delete:** closed `RulePlan`/`RuleExpr` algebra and type/trace mirrors, old pass/snapshot
controller, stage restore/hints, procedural relational reassembly and predecessor graphs.
**Exit:** fresh current source cases produce independently checked existing facts and
mathematics solely through target providers; no old snapshot or stage sidecar is input.

### SP11 — Replace remaining consumers and remove the old store completely

**Depends on:** SP05, SP08–SP10; part of the same caller-replacement cut. **Status:** Target Rust/Python publication handles and codecs implemented; predecessor store/modules/handles and predecessor-only fixtures deleted. Latest compile/lint, full caller/deletion audit, generation and cold ownership qualification remain in E08/E10.

- Make Rust/SQL/Python expose the same target publications, existing compiled/numerical
  data, settings, support and diagnostics. Generate Python value/extension classes;
  keep actual owned Arrow streams and deliberate small diagnostic materialization.
- Replace old store/head/manifest/hash handles with exact target publication and native
  operation handles. Replace snapshot codecs with qualified logical/provider descriptors
  and fresh planning; refuse unknown codec versions/unsupported extension arms.
- Move fixture builders, xtask, tests and inspectors to fresh target data; retain
  independent scientific/domain assertions already exercised, not layout goldens.
- Delete old store/snapshot/control/encoding/membership/sidecar/context/ref modules,
  predecessor readers/writers, schemas used only by those modules, obsolete exports,
  tests/generators/dependencies and compatibility-shaped API surfaces. Remove crate
  boundaries only if they no longer serve target responsibilities, with SP00 decisions.

**Surfaces:** catalog store/snapshot/inspection/codec closures, `pse-py`, Python codec/
inspection/interfaces, xtask and test-support/fixture tooling.
**Exit:** separate OS processes query the same exact target data in Rust/Python;
streams outlive dropped producer handles correctly; caller/reachability audit finds no
old runtime authority. This exit is mandatory before any architecture completion claim.

### SP12 — Native semantic change, exact reuse and protected maintenance

**Depends on:** SP09–SP11. **Status:** Typed dependency/reuse/CDF/reader protection and native maintenance implemented, with isolated projection units. Resource and interrupted/history/reclamation coverage remain in E09; durable lifecycle qualification is E10.

- Declare exact input selections, contract/policy/function/provider versions, absence
  scopes and external observations as typed dependencies. Use native queries to decide
  exact reuse and conservative invalidation. No serialized physical plan or object hash
  alone certifies semantic reuse.
- Enable and bind Delta CDF for the current editable relations where change processing
  consumes it. Normalize insert/delete/update pre/postimages with commit-version bounds
  and typed keys. A missing/expired history range causes explicit full recomputation or
  refusal according to the requested operation, never an assumed empty change set.
- Use native Delta optimization/checkpoint/log replay/cleanup. Derive retained members
  from live publications, attempts, explicit retained outputs and active readers; protect
  both data and required control/history logs. Coordinate pin acquisition and destructive
  maintenance for the supported local execution/storage envelope.
- Qualify cleanup after interrupted attempts and old target versions. No indefinite
  historical retention or recreated custom file-maintenance protocol. Unsupported remote
  reader-exclusion guarantees refuse destructive maintenance; no remote platform is built.

**Surfaces:** target dependency/attempt/reader relations, native change/reuse views,
Delta commands/CDF/maintenance and common resource ownership.
**Delete:** any remaining artifact memo/context restoration/invalidation duplicates
and bespoke Parquet maintenance. **Exit:** existing edit workflows, exact reuse and
clean target recomputation agree; compaction is semantically neutral; cleanup cannot
remove live data/log versions; no unsupported backend silently weakens the guarantee.

### SP13 — Qualify the architectural pivot and close deletion gates

**Depends on:** SP00–SP12. **Status:** Not started for this completion pass. Final qualification follows all implementation/deletions; E10/Q01–Q14/G1–G7 remain open. Earlier scoped receipts are not terminal acceptance.

- Run the architecture terminal journey over fresh existing source inputs, current
  transformations/math and implemented numerical cases, target publication, cold
  Rust/Python reading, change/reuse and cleanup. Do not add future simulator fixtures
  as prerequisites or accept mock future functionality as evidence.
- Prove ordinary extensions: one new tagged value/reference/invariant, one native
  expression/provider/parameterized relation, using the same metadata, validation,
  effects, persistence and inspection contracts without new dispatch inventories.
- Measure small/scaled nested data, coherent expression scans, native compile/planning,
  Delta IO/log replay, callback allocations/time, peak/retained memory and file growth.
  Fix demonstrated bottlenecks through native facilities; publish limitations honestly.
- Audit all replaced caller closures and generated remnants, then run current-source
  quality/generation/governance/Python/docs/feature checks against baseline zero.
  Remove obsolete assertions by replacing their useful coverage, not by suppressing
  legitimate failures or freezing warning baselines.
- Reassess G1–G7 independently for A01–A08. Record future simulator work separately;
  no gate is closed by merely excluding an existing implemented function.

**Exit:** all verification rows below have fresh named receipts; all package exits and
deletion obligations hold; architecture review can Accept its explicit current-function
scope. Plan 07's future simulator acceptance remains unclaimed.

### Replacement and deletion ledger

| Predecessor mechanism | Replacement | Owning cut | Current boundary |
|---|---|---|---|
| Top-level-only fields, bare child tuples and closed physical types | Recursive native Arrow contracts/domain facets | SP01–SP02 | Implemented; residual declaration/generated audit |
| Tag/payload and quantity conventions, unsigned ordinal conversion cascade | Generated complete alternatives/quantities/signed ranges | SP02 | Principal consumers replaced; residual fixture/collection audit |
| JSON keys, parallel spans and positional support/diagnostics | Typed scoped references and co-located spans/support | SP03 | Predecessor representations deleted; all-route/ownership qualification open |
| Canonical expression edge/payload tables and rejoin APIs | Coherent rows with native derived views | SP04 | Replacement/deletion implemented; semantics/cost qualification open |
| Encoded numerical names and handwritten parallel result schemas | Generated program/vector/coordinate/outcome contracts | SP05 | Replacement/deletion implemented; numerical/resource qualification open |
| Metadata-only function twins and closed invariant/rule/type machinery | Native plans/functions and common domain checks/transfer | SP06/SP10 | Algebra/reflection and redundant twins deleted; used-function/default/derived-value coverage audit |
| Metadata-stripped Delta layout and monolithic UDF-only CHECK | Self-describing fields/properties/native checks | SP07 | Implemented; durable/cold route qualification open |
| Root callback dispatch, settings mirrors and unadmitted routes | Actual native hierarchy and common preparation | SP08 | Root callbacks deleted; remaining route/resource audit |
| Publish journal/ref/CAS and member retry assumptions | Typed native Delta attempts/exact reconciliation | SP08–SP11 | Old authority deleted; history/recovery fixture coverage and qualification open |
| Stage driver, memo/hints and relational Cell reassembly | Native plans, explicit typed algorithms and dependencies | SP10–SP12 | Driver/stage/memo deleted; three more Cell sites replaced; residual conversion/caller audit |
| Store/manifests/sidecars/context/membership and Python snapshot handles | Exact publication/provider handles and owned streams | SP11 | Product authority/handles/codecs deleted or replaced; final exports/generators/cold ownership audit |
| Predecessor-only fixtures and simulator-only terminal prerequisite | Fresh current-function architecture qualification | Each cut; SP13 | Old fixtures/command deleted; target fixture expansion and acceptance campaign remain unrun |

Every row includes callers, generated schema artifacts, fixtures and exports. SP13
audits absence; it is not a holding area for deletions postponed from earlier cuts.

## Verification

### Architecture acceptance matrix

The final acceptance matrix remains **Proposed**; no row is terminal-certified. The
current checkpoint records **Tested** isolated units and **Interface-checked** earlier
source states only. Baseline is **0**. Final receipts must name command, feature/
profile, source state and failure/skip counts. Selection follows existing functional
outcomes, not predecessor implementation forms or future simulator features.

| ID | Proof obligation / attack | Owning packages |
|---|---|---|
| Q01 | Exact nested fields: renamed/extra/missing/reordered children, false metadata, view/list physical adaptations, valid parent masks and visible-child nulls | SP01/SP07 |
| Q02 | Every alternative arm and invalid combination; negative/range ordinals; quantity/reference distinctions; null/empty/ordered/unique collections | SP02 |
| Q03 | Row-key stability under chunk/order changes and payload edits; collision/consistency rejection; scalar/composite/nested FK and wrong-selection attacks | SP03 |
| Q04 | Source spans/support survive filter/join/UNION/window/unnest and producer release; no positional reconstruction or JSON join identity | SP03/SP06/SP10 |
| Q05 | Existing expression meanings, operand order/multiplicity, payload/arity/closure, canonicalization and scalar numerical consumer after coherent-row cut | SP04 |
| Q06 | Numerical program/schema/coordinate dimensions and actual existing solver derivatives/status/duals/cancellation/panic/ownership; measured callback allocations | SP05 |
| Q07 | Native CHECK constraints in a PSE-free session, UNKNOWN rejection, no silent filtering, defaults for omitted vs NULL, identical SQL/Rust/Arrow write outcomes | SP06/SP07 |
| Q08 | Exact durable declaration/properties/layout round trip, raw external Arrow readability, compatible extension restoration and mismatch refusal; timestamp protocol | SP07 |
| Q09 | Actual config/UDF/rule/planner sentinels; hierarchy discovery/quoted names/absence; view/no-scan/factory/nested operation policies; truthful constraints/pushdown/stats | SP08 |
| Q10 | Complete current-artifact member vector, slices/unchanged reuse, concurrent first-create/stale parent, failure after each member/root commit, retry identity/lost acknowledgment | SP09 |
| Q11 | Fresh-source current authoring/edit/normalization/compilation/inference journey through target only; independent facts/diagnostics; no old snapshots | SP10/SP11 |
| Q12 | Separate-process Rust/Python cold open, raw/registered Arrow consumers, stream abandonment/drop/cancel, target descriptor replanning and unsupported-codec refusal | SP11 |
| Q13 | Data/policy/function/absence changes, qualified exact reuse and full recomputation oracle, CDF update pairs/range gaps, compaction and data/log reader-pin races | SP12 |
| Q14 | Ordinary extension locality, complete runtime caller/declaration/fixture deletion, reproducible cost/resource envelope and final zero-baseline quality | SP13 |

Reuse existing test binaries where they fit. Add ordinary targets such as
`schema_contracts`, `native_architecture` and cold architecture Python tests only
where necessary; SP00 records their final names. Do not create a parallel test platform.

| Command surface | When / meaning |
|---|---|
| `just doctor`, `just metadata`, `just family-check` | Environment/dependency inspection when relevant; no runtime completion claim |
| `just check-library <package>`, `just check` | Compile during coupled replacements as useful; temporary failures are fixed by completing the target callers |
| `just unit-package <package> '<nextest-filter>'` | Isolated **unit** checks during implementation; explicit force validation. Inspect the selected behavior: a library test that performs a compiler/storage/solver journey belongs to final qualification |
| `just codegen-bootstrap`, `just codegen`, `just conformance-fixtures` | Regenerate after declarations/generators agree; fixture generation is allowed, executing conformance suites waits for the completed cut |
| `just codegen-check`, static quality/lint commands | Check declarations, generated output and code quality at useful boundaries; no broad integration prerequisite between cuts |
| Engine, conformance, package integration, linked solver and Python component suites | **Only after all implementation and deletions are complete**; retain useful independent functional assertions while replacing old fixtures/callers now |
| `just py-sync`, Python stub generation | Rebuild the target boundary when needed; this does not imply running `just py-test` during implementation |
| `just ci-fast`, `just governance`, `just py-test`, `just quality`, required feature checks, `just adr-lint`, `just lint-agents`, `just docs` | Final current-source checks, baseline zero; identify overlap and avoid redundant complete runs without intervening relevant changes |
| `just architecture-acceptance <new-output-directory>` | Implemented target-only campaign with provenance, commands, logs, exits and timings; not run for this completion pass. Only after all implementation/deletions; independent Q/G decisions remain required |
| Focused ordinary benchmarks / `just bench-smoke` | Final named latency/resource measurements; smoke alone is not performance evidence |

### Traceability: every schema-review finding

| Schema review | Implementation | Required evidence / refinement |
|---|---|---|
| SE-F01 alternatives | SP02/SP06 | Q02/Q07; include N01/N03 nested contracts and all currently declared sites |
| SE-F02 expression rows | SP04/SP10 | Q05/Q11; fold canonical rows, measure physical access; no historical hash/graph preservation requirement |
| SE-F03 semantic values/transfer | SP02/SP06/SP08 | Q02/Q04/Q09; no new function allowlist; structure does not by itself validate domain arithmetic |
| SE-F04 durable declaration | SP01/SP07 | Q01/Q08; compatible physical extension descriptors per N02; exact match or refuse, no migration reader |
| SE-F05 signed local integers | SP02/SP07 | Q02/Q08 plus join/cast measurements; external unsigned Arrow remains eligible |
| SE-F06 typed row identity | SP03/SP12 | Q03/Q04/Q13; actual PK remains authoritative; references include selection scope |
| SE-F07 numerical output | SP05 | Q06; current analytic/solver cases only, typed dimensions and allocation measurements |
| SE-F08 structural correspondences | SP03/SP05/SP10 | Q04/Q06/Q11; generated diagnostics and typed outcome bindings |
| SE-F09 durable exactness | SP01/SP07 | Q01; native compatibility helper is not a substitute for full-field contract comparison |
| SE-F10 native local checks | SP06/SP07 | Q07; exploit Delta validation, cover demonstrated nested gaps, residual UDFs only for actual domain meaning |
| SE-F11 evidence labels | SP00/SP13 | Pin/source labels and actual measurements; no claimed Delta nested-I/O optimization without proof |

### Traceability: additional opportunities

| Opportunity | Packages | Acceptance |
|---|---|---|
| N01 recursive references/collections | SP01–SP03/SP06 | Q01–Q03/Q07 |
| N02 compatible extension descriptors | SP01–SP02/SP07 | Q01/Q08/Q12 |
| N03 recursive nullability/exactness | SP01/SP06–SP07 | Q01/Q02/Q07/Q08 |
| N04 publication/attempt values | SP02/SP09/SP12 | Q02/Q10/Q13 |
| N05 operation result contracts | SP02/SP03/SP05 | Q02/Q04/Q06 |
| N06 defaults and derived facts | SP06–SP08 | Q07/Q09 |
| N07 Arrow-native values | SP01/SP06/SP10 | Q01/Q07/Q09/Q11 |
| N08 metadata and dependencies | SP08/SP12 | Q09/Q13 |

### Traceability: prior unified-pivot work, without simulator expansion

| Plan 07 / original finding | Disposition here |
|---|---|
| UD00 target/oracles; UD-F14 | SP00 current-function authority/acceptance; future simulator requirements remain unscheduled |
| UD01 runtime/dependencies; UD-F05/F06/F13 | SP08/SP13 native composition, complete actual configuration and feature/build qualification |
| UD02 schemas/providers; UD-F02/F03 | SP01–SP08 strengthened schema-first implementation |
| UD03 Delta lifecycle; UD-F01/F04/F07 | SP07/SP09/SP11 exact durability/recovery and complete predecessor deletion |
| UD04 authoring | SP03/SP10 current parsing/editing/normalization only |
| UD05 inference/math; UD-F08/F09 | SP03/SP04/SP06/SP10 existing domain behavior on the new architecture |
| UD06 new problem/structure/initialization | **Excluded**; schemas may be corrected, no new runtime implementation |
| UD07 evaluation/Ipopt; UD-F08/F11 | SP05/SP08 preserve and restructure what exists; no new case-to-solve workflow, derivatives or simulator coverage |
| UD08 generated Pyomo/NL | **Excluded**; no backend implementation or qualification |
| UD09 reuse/CDF/retention; UD-F09/F12 | SP09/SP12 for current artifacts/operations and their complete target lifecycle |
| UD10 inspection/Python/codecs; UD-F10/F11 | SP08/SP11 target-only current data and owned boundaries |
| UD11 simulator/cost | Replace with SP13 architecture/current-function/extension/cost acceptance; do not claim simulator completion |
| UD12 deletion/gates | SP13 final audit after deletions in owning cuts; no carryover legacy entitlement |

### Independent terminal gates

All are currently **Unresolved for this in-progress plan**; this does not reverse the
completion review's observed G1 failure or schema review's observed G2 failure.

| Gate | Architecture-only closing condition |
|---|---|
| G1 authority | One recursive declaration and one Delta product lifecycle; no competing mutable store, field meaning, rule algebra or inspector |
| G2 fidelity | Exact nested/value/quantity/identity/math/numerical/Python meaning preserved for existing functions; explicit physical conversions |
| G3 validity | Every current write/execute/open route enforces required local, nested, relational and artifact-profile obligations |
| G4 effects | Native plans expose actual dependencies/effects/resources; no hidden callback execution or mutation during inspection/reconstruction |
| G5 recovery | Member/root outcomes, retries, cancellation, cold opening and cleanup/pins are qualified at actual effect boundaries |
| G6 transformation/reuse | Native rewrites, multiplicity/support, numerical policy and exact dependency-based reuse hold under independent oracles |
| G7 capability | Current-function routes and ordinary extensions actually work; future simulator functions remain explicitly unavailable and unclaimed |

## Implementation history

These receipts are chronological evidence for their named source states. They are
not a cumulative passing certification of the current tree. The invariant and native
caller replacements subsequently landed; the current checkpoint records later checks
and the latest unfinished compile/static work.
Use the current remaining-scope checklist above for restart and completion decisions;
historical “Remaining” and “in progress” notes are retained as provenance only.

### Earlier native pivot continuation receipts — 2026-09-17

The following continuation notes preserve their original source-state boundaries.
Their open-work statements are historical; use the current E00–E10 checklist above.

#### Native dependencies and maintenance continuation — 2026-09-17

**Implemented:** `runtime.native_dependencies` captures exact selected inputs,
including empty/unmatched sources, native settings with explicit absence, policy
facts, ordered rules, function registrations, provider owners and metadata lookup
coverage. Actual opaque implementations are tied to the retained ArtifactPlan
generation. A fresh/cold generation refuses reuse even when all names match;
unknown extensions and volatile/observed inputs require explicit fresh execution.
An owned execution contract can declare an extension's real effects without adding
an extension-name roster. Derived scopes retain their consumed providers rather than
all ambient providers, and conflicting selections refuse.

**Implemented:** member receipts persist those generated typed dependency rows.
Selected dependency receipts are exposed through DataFusion's native StreamingTable
and PartitionStream contracts, with no IO during preparation. Native grouped counts
and null-safe bidirectional EXCEPT detect changes and multiplicity differences.
`ArtifactPlan::prepare_reuse` gates the actual selected output scan on that query;
expired or missing commit evidence requires recomputation. This provides retained-
generation reuse; no portable identity is invented for unknown native implementations.

**Implemented:** exact Delta views, writable scans/commands, CDF providers, physical
plans and streams retain local read ownership. The physical adapter preserves child
properties, modern statistics, filter/limit pushdown and input ordering. Maintenance
uses a location-checked exclusive lease to read control facts without reacquiring a
shared lock. Native publication-retention queries expand both members and inputs;
typed unions include explicit output, attempt and CDF intervals. Referenced/current
versions and active readers prevent reclamation. Unreferenced member attempts can
be retired through a native metadata transaction, deleted through native Delta, and
vacuumed under the declared expiration policy. No custom data-file deletion exists.

**Implemented:** publication contract version 2 uses native application transactions
to index attempt/publication identities to their actual control version. Reconciliation
still compares the exact stored row. Transactions are not assumed to deduplicate
replay. Expiring transaction identities is refused for control tables; retired control
logs can be pruned while identity indices survive checkpoints. Native member retirement
also records an empty/provisioned attempt, since DELETE alone can be a no-op.

**Implemented:** `just architecture-acceptance <new-directory>` runs the final
current-function campaign and preserves source provenance, exact commands, exit
codes, timings and logs. The obsolete simulator recipe/xtask entry point and an
unreachable OperationInput source file are deleted. The new native artifact lifecycle
integration fixture covers retained-generation reuse, cold-generation refusal,
typed CDF and reader-blocked maintenance; it has been compiled, not executed.

**Tested**, baseline zero, default Nextest/test profile, explicit
`pse-relations/force-validate`, isolated units only:

- `just unit-package pse-catalog
  'test(delta::leased::tests::) | test(delta::lease::tests::)'`: 3 passed,
  0 failed, 87 excluded, 0.011 s; run `3eafff2d-504b-45c5-bfd1-89e3768abc14`.
- `just unit-package pse-catalog
  'test(artifact::dependencies::tests::) | test(session::cache::freshness_tests::) | test(delta::attempt::tests::)'`:
  3 passed, 0 failed, 89 excluded, 0.797 s; run
  `aae7f84e-db78-4803-b720-e95b937d641b`. In-memory queries check NULL changes,
  generation changes, reordering, missing/duplicate facts, and extension freshness.

**Interface-checked:** workspace/all-target `just check` passed in 21.28 s before
the final fixture/retirement edits, with one subsequently removed qualification
warning. `just check-package pse-catalog` then passed in 1m28s including Cargo lock
wait, covering the new lifecycle fixture before the final retirement edit.
`just family-check` passes the pinned families and evidence locks. Contract and
conformance fixture generators completed successfully. No integration campaign ran.

**Open:** finish shared-default/derived-value and source-support route auditing,
retention/reclamation failure fixtures and extension-locality coverage, then final
native/Python/quality closure. E10 still owns actual Delta/compiler/solver journeys,
cost measurements and independent Q01–Q14/G1–G7 decisions. Skill syntax scans flag
the lease's statistics hook because the scan recognizes the deprecated hook only;
the implementation uses native `statistics_from_inputs`/`child_stats_requests`.
Strict native write schema mode remains intentional. Vacuum reads the declared native table retention duration through its default builder path; no shorter override is installed.

#### Publication and lifecycle continuation — 2026-09-17

**Implemented:** scientific engine fixtures now use native source/edit/algorithm plans
and exact Delta publications. Legacy stage/memo/terminal/sidecar/manifest/ref fixtures
are removed; independent scientific equations, incidence, support and numerical
assertions remain. The lifecycle fault matrix now injects failures into real native
Delta writes and checks complete control/member visibility. Catalog fixtures use the
common session/publication boundary. No integration fixture has executed.

**Implemented:** declared artifact completeness is registry-owned, reflected in
`reference.artifact_profiles`, and participates in registry identity. `source`/`case`
require source-projection outputs, `model` adds normalization outputs, `problem` adds
the existing semantic/math outputs, and `run` requires the existing numerical program,
coordinates and outcome. Partial checkpoints explicitly use `relations`; they retain
all member-level checks. Planning, control admission and cold open check profiles.

**Implemented:** native member writes record the complete request, exact selections,
contract and session policies plus the retained immutable operation identity. Recovery
reads actual Delta actions and requires the matching application transaction before
returning a committed version without re-executing its child. Provisioned-but-unwritten,
rejected and unresolved cases are distinguished. Opaque implementations currently
require the same retained ArtifactPlan; safe cold reuse and observation classification
remain E09 work. No plan fingerprint or attempt label establishes semantic equivalence.

**Implemented:** exact source dependencies now survive private derived scopes; selected
empty inputs remain dependencies. Native bounded CDF emits typed change context and a
complete before/after value. Maintenance has a real typed retention-query child and
uses Delta optimize, checkpoint, `with_keep_versions`, vacuum and checkpoint-aware log
cleanup. Shared writer/control leases and exclusive local maintenance leases now
coordinate with existing publication readers. Retained CDF ranges select Lite vacuum,
since Full vacuum can remove CDC payloads outside the active file set. All reader paths,
retention derivation, cold reuse and interruption behavior still require closure.

**Tested**, baseline zero, default Nextest/test profile, explicit
`pse-relations/force-validate`, isolated units only:

- `just unit-package pse-schema
  'test(builder::tests::artifact_profiles_are_declared_reflected_and_identity_bearing)'`:
  1 passed, 0 failed, 30 excluded; 0.004 s execution.
- `just unit-package pse-catalog
  'test(delta::attempt::tests::) | test(delta::lease::tests::) | test(delta::admission::tests::complete_artifact_profiles_refuse_missing_members_but_allow_explicit_partial_collections)'`:
  4 passed, 0 failed, 85 excluded; 0.740 s execution. These check exact request
  comparison, missing profile members and local reader/writer exclusion using a
  temporary lock file; they do not execute Delta/compiler/solver journeys.
- `just unit-package pse-ids
  'test(contract::tests::matching_schema_is_accepted_and_extra_contextual_keys_are_refused)'`:
  1 passed, 0 failed, 74 excluded; 0.002 s execution, before the profile additions.

**Interface-checked:** `just check-package pse-tests-engine` (11.48 s),
`just check-package pse-tests-conformance` (2m09s), and
`just check-package pse-tests-lifecycle` (1m01s) succeeded before the latest
profile/maintenance changes. `just check-package pse-catalog` succeeded after native
fixture API closure (0.52 s). `just check-library pse-catalog` succeeded after the
maintenance implementation (16.51 s); later typed-CDF edits still require compilation.
`just py-sync` succeeded before the latest UUID dependency and schema changes; refresh
it again at final Python closure. All generated trees are changed only by generators.

#### Native source and fixture continuation — 2026-09-17

**Implemented:** `AlgorithmSpec` signatures replace producer graphs. Their arguments
include actual native pre/postcondition dependencies. Requirements and error findings
are real plan children around shared invocation results. Typed source edit/rename plans
replace authoring envelopes. Python fixtures now open `PublicationRoot` and inspect
exact three-part names. Their independent schema, lifetime, budget, cancellation and
engineering assertions remain. The shared scientific Rust harness now composes native
plans and captures Arrow relations; its cold checks use Delta control/member versions.
Remaining legacy Rust fixtures still require replacement; workspace compilation is open.

**Implemented:** schema generators prune old catalog/manifest, change operation, stage
record, revision and snapshot/hash diagnostic/derivation contracts. Native derivations
identify their algorithm directly. Extra predecessor schema identity stamps are refused
rather than silently excluded from admission/canonicalization.

**Tested:** `just unit-package pse-compiler
 'test(native::tests::native_requirements_gate_actual_arguments_and_results)'` passed
1, failed 0 (baseline 0), 27 excluded, default Nextest/test profile and explicit force
validation; 2.127 s execution. Duplicate actual inputs prevent algorithm invocation;
duplicate outputs are refused; planning invokes no algorithm. This receipt precedes
the latest diagnostic/revision schema cleanup. No integration journey was executed.

**Interface-checked:** compiler product source-edit compilation and authoring all-target
compilation passed before the latest diagnostic/revision cleanup. `just py-sync`
completed editable native build and metadata-derived stubs before that cleanup.
`just codegen-contracts` regenerated the latest Rust/Python/docs contracts. Current
compile and isolated canonical metadata checks are being refreshed; do not infer a
whole-workspace or cold runtime pass from these receipts.

#### Earlier E01/E04 cut — native values and query authority

**Implemented:** native SQL invariants replace invariant-to-rule compilation throughout
schema declarations, registry reflection and invariant execution. Binding checks exact
selected provider inputs and ordered diagnostic keys. Pure generated contracts now
reflect query/input/key declarations. The predecessor store consumer has been deleted.

**Implemented:** case/activation/observation targets carry complete tagged members;
method resolutions carry one outcome and selected method reference; transfer port/member
selectors form one optional value. P7/P9 and authoring consumers use those declarations.
`pse_checked_value` establishes local value contracts inside native expressions and
refuses implicit storage or quantity reinterpretation. Cross-relation obligations remain
provider admission requirements.

**Implemented:** all 130 inference rules now have ordinary SQL source declarations in
`crates/pse-schema/src/catalog/native_rules/`, explicit input scopes, domain truth and
stratum/conflict policies. The product `RulePlan`/`RuleExpr` algebra, lowering/type
mirrors, nested custom closure and reflection tables are deleted. Registry inspection
stores native query text. The outer finite fixed-point operator remains for its domain
semantics. Source witnesses are derived from native logical plans, preserving typed
keys, aggregate membership and exact-input anti-join absence. Temporary conversion
executables/source helpers have been removed.

**Interface-checked:** `just check-library pse-schema`, `just check-library pse-rules`
and `just check-library pse-compiler` succeed. `just codegen-contracts` regenerated
Rust/Python/docs declarations without executing package/compiler/storage workflows.
Subsequent source edits still require consolidated regeneration and lint checks.

**Tested**, baseline zero, default nextest/test profile with explicit
`pse-relations/force-validate`:

- `just unit-package pse-rules 'test(strata::demand_planning::every_native_rule_binds_with_its_exact_input_fields)'`:
  1 passed, 0 failed, 5 excluded by the positive selection; all 130 native queries
  and their support plans bind against empty selected providers (10.544 s execution).
- `just unit-package pse-rules 'test(plan::trace::tests::)'`: aggregate and anti-join
  and window witness tests: 3 passed, 0 failed, 6 excluded (0.015 s execution),
  using tiny in-memory inputs only. Window recovery retains conservative complete
  partition dependencies without recomputing window values over witness fanout.
- Earlier focused invariant binding/duplicate-key tests: 3 passed, 0 failed;
  checked-value kernel tests: 2 passed, 0 failed. These are isolated unit receipts,
  not terminal compiler/publication certification.

**Implemented — subsequent native boundary cut:** conservation subjects, coordinates,
participation and law defaults now use complete generated values. Ordinary native
`min`, `max`, `array_agg`, `named_struct` and `array_element` expressions share actual
function-identity field transfer; redundant SQL-name twins are removed. Necessary
pinned physical adapters remain. Native rule SQL projection chains are simplified;
all 130 query/support bindings still pass the isolated empty-provider binding test.
Outer-join witnesses distinguish actual positive keys from unmatched-side absence.

**Implemented — execution and ownership:** native execution contracts contain their
requirement and value plans as real children. Native namespace commands are deferred
until execution. Publication reads now use the admitted session and owned stream;
raw publication-state/provider read bypasses are removed. Local shared reader leases
are acquired before exact table resolution and retained through lazy execution.
The exclusive maintenance caller and complete retention authority are still open.

**Implemented — native table policy and change:** declared Delta properties participate
in registry reflection, fingerprints, generated contracts and cold metadata admission.
Defaults enable CDF and disable automatic expired-log cleanup. Bounded native CDF
reads retain change metadata, check both endpoint contracts and every intervening
commit, preserve revision predicates and refuse missing history. This route has
compile evidence only; its final Delta journey remains deferred to E10.

**Implemented — invocation cache:** the default native `CacheFactory` now inserts a
deferred logical node. Actual physical siblings share one computation, pool-accounted
buffers and native `SpillManager` storage within an invocation. The cache index uses
weak completion owners; retaining the session does not retain completed buffers.
Caller factories are preserved. This is a foundation for E07, not a replacement of
the subsequently deleted Driver/stage memo and not durable reuse certification.

**Tested**, baseline zero, default nextest/test profile with explicit
`pse-relations/force-validate`, isolated unit selections only:

- `just unit-package pse-schema 'test(builder::tests::native_table_policy_changes_identity_and_survives_cold_metadata)'`:
  1 passed, 0 failed, 32 filtered out; policy changes affect identity, cold metadata
  agrees, and duplicate/reserved properties refuse (0.003 s execution).
- `just unit-package pse-catalog 'test(delta::lease::tests::)'`:
  1 passed, 0 failed, 145 filtered out; cloned reader lifetime, exclusive exclusion
  and cancellation, using one temporary lock file (0.003 s execution).
- `just unit-package pse-catalog 'test(session::cache::tests::)'`:
  4 passed, 0 failed, 146 filtered out (0.016 s execution), covering lazy shared
  execution, native spill, fresh invocation, released pool reservations, resuming
  an abandoned reader without restarting its input, and caller factory preservation.
- Preceding boundary units: native field transfer 1 passed; requirement barrier
  2 passed; native support 4 passed. All used tiny in-memory operators. The latest
  all-rule binding receipt is 1 passed, 0 failed, 9 filtered out (12.273 s execution).

**Interface-checked:** `just check-library pse-catalog` succeeds after the native
cache/CDF implementation, including the completed selected cache unit run.
`just codegen-contracts` regenerated all three contract surfaces after table policy.
The unit recipe explicitly selects `pse-relations` so force-validation is available
when testing a lower-level package such as `pse-schema`; the positive test filter
still determines the only tests executed.

**Implemented — fixture source cut:** every `pse-rules` test target now declares native
SQL and exact input scopes; schema registry admission tests now inspect native query,
outcome, dependency and invariant declarations. Closed-algebra-only tests are removed;
independent row, support, conflict, NULL, overflow and recursive cancellation assertions
remain in native fixtures. **Interface-checked:** `just check-package pse-rules` (2.00 s)
and `just check-package pse-schema` (30.81 s) compile all their test sources with explicit
force-validation. No integration test was executed.

**Implemented — native hierarchy ownership:** native schema providers now own table
implementations and their established facts. Semantic roles hold only native table
locators. Namespace forks copy catalog/schema state while sharing immutable table and
buffer owners. The parallel binding-owned provider map and catalog reconstruction are
removed. Aliases cannot replace a table's established semantic facts. Native command
results retain their actual updated caller state, including `SET` configuration, with
weak completion references that avoid a state/configuration ownership cycle.

**Tested**, baseline zero, default nextest/test profile with explicit
`pse-relations/force-validate`, isolated units:

- `just unit-package pse-catalog 'test(provider::binding::tests::)'`: 4 passed,
  0 failed, 151 filtered out (0.015 s); native lookup, alias ownership, removal,
  quoted names, immutable replacement and private native namespace mutation.
- `just unit-package pse-catalog 'test(session::commands::deferred::tests::)'`:
  1 passed, 0 failed, 155 filtered out (0.007 s); actual native `SET`, caller opaque
  extension preservation, original-state isolation and owner release.
- `just unit-package pse-catalog 'test(session::scalar::checked_value::tests::)'`:
  3 passed, 0 failed, 148 filtered out (5.497 s); already typed enum/extension
  meanings cannot be retagged as a different meaning.
- The latest all-130-rule binding unit: 1 passed, 0 failed, 10 filtered out
  (12.184 s), after that stricter value-meaning check.

**Interface-checked:** `just check-library pse-compiler` (57.63 s including Cargo lock
wait) after native hierarchy ownership; `just check-library pse-catalog` (13.45 s)
after command-state completion.

**Implemented — remaining native operation routes:** private in-memory DML now enters
through actual native `TableProvider` write hooks at any logical depth. Its physical
operator exposes the before-image and write input as real children, delegates expression
and write semantics to DataFusion, validates the private result, and replaces only its
native namespace generation. The predecessor `PreparedMutation`, its deferred target
shim and its root preparation dispatch are deleted. Providers without a private factory
retain their own truthful native mutation hooks, including declared Delta operations.
Metadata discovery is a lazy native table source over the asynchronous catalog contract;
it no longer implements or invokes `NativeOperation`. Its completed namespace cannot
retain its own resolving provider.

**Tested**, baseline zero, default nextest/test profile with explicit
`pse-relations/force-validate`:

- `just unit-package pse-catalog 'test(session::mutation::native::tests::)'`:
  1 passed, 0 failed, 156 filtered out (0.010 s); one-row native update, real source
  child, affected-row count, private replacement and preserved earlier source.
- `just unit-package pse-catalog 'test(session::resolution::tests::)'`:
  1 passed, 0 failed, 157 filtered out (0.009 s); empty in-memory metadata source,
  lazy opening, native scan and absence of a producer/result ownership cycle.
  A trivial-cast warning in that test fixture was subsequently removed.

**Interface-checked:** `just check-library pse-catalog` after private DML (16.61 s)
and native metadata resolution (17.68 s). Generic `OperationNode`/`NativeOperation`
remains reachable only through the predecessor compiler production/store paths;
those paths and their preparation fallback still require the E07/E08 deletion.

**Implemented — native compiler cut and current caller boundary:** `Driver`, its
commit/history/restore controller, stage DAG/key/registry, memo, pass records and
compiler restoration validators are deleted. Reusable domain algorithms consume
`RelationFacts`, preserving exact source selections. `native::model::compile` now
composes their real logical dependencies; the native physical boundary consumes the
actual executed children. Typed heterogeneous Arrow tuples expose relation outputs
through native UNNEST. Multiple arguments from one tuple share one actual producer
child and the caller's native invocation cache. `ArtifactPlan` declares named outputs
and composes their Delta writes with exact retained selections into one publication.
This is implementation, not end-to-end compiler qualification: source/edit commands,
remaining declaration cleanup, obligations and target fixture/caller closure remain.

**Implemented — Python and store hard cut:** Python `Publication`/`pse.open` now select
an exact Delta control URI/version and literal catalog/schema/table components.
`Store`, `Snapshot`, mutable-ref/manifest-hash open and store-only settings are removed
from the Python product boundary. The owned Arrow stream remains. Native codecs now
encode exact selected member descriptors and rebind against the receiving native
hierarchy, with no predecessor manifest. The catalog `store/**`, snapshot, computation,
source-production, old table/pushdown/statistics adapters and root `NativeOperation`
executor are deleted. Applicable native field checks, pool accounting, provider
hierarchy and settlement handling remain. The generator, xtask, remaining Rust/Python
fixtures and obsolete schema/error inventories still need closure; intermediate
whole-workspace compilation is expected to fail at those deleted API callers.

**Interface-checked:** `just check-library pse-runtime` succeeded (55.78 s), after
native compiler composition. `cargo check -p pse-py --lib --offline` succeeded
(1m56s), refreshing only existing pinned dependency edges while compiling the new
Python publication boundary. A then-unused `SnapshotPort` enum arm was subsequently
deleted with the store cut. Current store/codecs edits still require compile closure.

**Tested**, baseline zero, default nextest/test profile, explicit force-validation:
`just unit-package pse-compiler
 'test(native::tests::native_tuple_selects_typed_members_and_preserves_empty_relations)'`:
1 passed, 0 failed, 26 excluded (0.792 s execution). This isolated one-row/empty-tuple
unit checks native extraction, complete typed fields and shared-child structure; it
does not execute a compiler or Delta workflow.

**Implemented — native effects and fixture generation:** Delta write/publication
nodes now use the generic native execution contract for effects; the preparation
roster no longer enumerates those node types. `just conformance-fixtures` regenerated
current declarations and removed obsolete closed-rule fixture families; it is pure
fixture construction, not invariant or integration execution.

Remaining E04 work includes defaults/derived-field unification, wider source-support
operator coverage and the remaining workspace fixture/caller audit. All old rule
APIs must stay deleted. All E05–E09 compiler/provider/publication/store/lifecycle cuts
remain open as detailed below. No package is terminal-certified.

### Implementation checkpoint — 2026-09-16

**Implementation is partial; no SP00–SP13 package or terminal gate is closed.**

**Implemented — scope and authority:** ADR-0069 and blueprint revision 40 precede
the contract changes. AGENTS.md names Plan 08. The
[current-function oracle inventory](08-current-function-oracles.md) identifies existing
functional assertions and deletion boundaries. The schema essay names exact pins and
evidence limits. The architecture acceptance command is specified; its terminal
implementation, complete measurement fixtures and remaining SP00 closure stay open.

**Implemented — exact boundary:** `pse-schema::field_contract` compares native Arrow
fields recursively, rejecting duplicate paths and changed names/order/nullability/
metadata. Relation admission and DurableLayout share this checker; the former's
independent field comparison and the latter's `equals_datatype` shortcut are deleted.
Delta scan admission permits the named recursive view-array adaptation. Known PSE
extensions cannot claim incompatible physical storage.

**Implemented — durable declarations:** Delta fields preserve a versioned native
execution descriptor, with schema annotations kept separate from field annotations.
Nested list-item metadata survives in that descriptor even though Delta's ArrayType
has no field-metadata slot. Incompatible Arrow extension annotations are removed from
changed storage; compatible ones remain. Cold decode reconstructs the execution schema
without a registry, validates its physical declaration and compares it with the requested
contract before projecting values. Unknown/missing/forged descriptors refuse. No reader
for the previous metadata-stripped layout is retained.

**Implemented — value conversion:** decode checks the inverse native Arrow cast for
exact values when storage changes. This catches float rounding in addition to native
integer overflow/identity-width errors, while respecting masked struct parents and
signed zero. Sorted-map durability remains explicitly unsupported until order semantics
are qualified; native Arrow map eligibility is unaffected.

**Implemented — native registry declarations:** `FieldContract` owns one native Arrow
`Field`; nested fields carry their own role, reference, quantity and extension facets.
The closed `LogicalType` enum, `ColumnSpec` struct and bare tuple declarations are
deleted, with production consumers, generators and test fixtures changed together.
Execution metadata is derived from the declaration at every depth, with conflicting
or orphan derived facets rejected during registry assembly. Arbitrary native types and
external annotations remain eligible for declaration; a missing language codec reports an operation
error instead of inventing PSE semantics for bare binary or dictionary values.

Registry rows and fingerprints include canonical native field serialization, including
nested facets and custom annotations. Rust/Python/JSON projections preserve nullable
list members, and generated reference tables expose nested paths. Exact boundary
checks include dictionary ordering through native union/dictionary/run-end containers;
valid zero-width fixed lists remain eligible. Native Arrow field sizing replaces the
closed recursive type-size interpreter in authoring allocation.

**Remaining boundary:** SP01 qualification is in progress. Native invariant lowering,
tagged alternatives, typed quantities, signed canonical ordinals, table properties,
defaults, complete publication profiles and every later SP02–SP13 replacement/deletion
obligation remain open. Stored field descriptors do not establish PK/FK application
validity or complete SP07. Existing RulePlan/RuleExpr/Cell and compiler/store lifecycles
remain scheduled for their owning replacement cuts; adapting their field consumers
does not close their deletion obligations. In particular, the current batch/value
admission routes still decode through Cell and therefore do not yet execute every
newly declarable native type; the declaration tests do not certify that later cut.

**Tested — Rust:** all commands below use the default Nextest profile and explicit
`pse-relations/force-validate`; baseline **0**. No skipped tests are counted as passing.

| Command | Result | Run ID |
|---|---|---|
| `just test-package pse-catalog --test unified_delta_contracts --test unified_delta_dml -p pse-schema -p pse-relations --test field_contract --no-fail-fast` | 30 passed, 0 failed/skipped; 9.971 s | `2b1f3e18-c9b0-468a-b106-8b5b61b01f41` |
| `just test-package pse-relations --no-fail-fast` | 28 passed, 0 failed/skipped; 1.274 s | `f48d67b3-d8f8-48c6-ba74-30120a8b1cbf` |
| `just test-package pse-catalog --lib --no-fail-fast` | 113 passed, 0 failed/skipped; 5.607 s | `dfb16664-d20f-48e1-b881-79d7602a5bae` |
| `just governance` | 59 passed, 0 failed/skipped; 3.131 s; generation and family checks pass | `8fd850ad-c10b-459b-9d61-46395d3f2645` |

The last three receipts precede the final cold descriptor reconstruction helper;
the 30-test boundary receipt covers that helper and its live Delta consumer. Catalog
unit tests still exercise surviving legacy machinery and do not certify its deletion.
The additional selected relations package makes the validation feature available to
schema tests; the first schema-only invocation failed feature selection before tests.

**Tested — supporting checks:** `just adr-lint` passes 69 records, index and 31 register
rows. `just family-check` confirms the existing pinned families after enabling native
Arrow-schema serialization. `just codegen-check` matches all three schema targets and
pinned Ipopt bindings. Skill syntax scans found 0 DataFusion advisory matches on changed
boundary files and 2 Delta schema-mode advisories on existing writes. Those writes
intentionally retain Delta's strict default: the pinned explicit modes are Merge and
Overwrite, neither of which is appropriate for exact contract admission.

**Corrections and open quality:** an old cold-open fixture claimed non-null output
from a nullable UDF; it now uses the actual stored declaration and tests rejection of
the false one. The first py-sync attempt raced with the module addition and failed;
a subsequent rebuild and doctor succeeded. Final-source refresh/check receipts follow
below. The initial package lint findings were corrected. The upstream
`proc-macro-error2` future-compatibility warning and mdBook's large search-index warning
remain open; no zero-warning terminal claim is made. The unclosed proposed-ADR example
was fixed, leaving `just docs` at 0 errors and 1 warning, baseline 0.

**Tested — final boundary source:**
`cargo clippy -p pse-schema -p pse-relations -p pse-catalog --lib --locked -- -D warnings`
(default dev profile) exits 0 with **0 project lint findings, baseline 0**; Cargo still
reports the upstream future-compatibility warning above. `just py-sync` rebuilt the
editable extension and regenerated its actual API stub successfully; `just doctor`
reports the environment ready with 0 blocking failures. `just lint-agents` passes
22 files, 75 path references and 57 recipe references. No Python runtime suite,
benchmark or full workspace terminal gate was run for this boundary cut.

Package-specific Cargo cleanup removed 91.4 GiB of regenerable schema/relations/catalog
artifacts after free disk fell to 1.5 GiB. No source, user data or development stores
were removed by that cleanup. Existing legacy code/data deletion remains tied to its
replacement cuts, not to this build-space repair.

### Native declaration qualification — 2026-09-16

**Tested — baseline 0:**

| Command | Mode and result |
|---|---|
| `just test-package pse-schema -p pse-relations -p pse-catalog -p pse-rules -p pse-authoring -p pse-compiler --no-fail-fast --status-level fail --final-status-level fail` | Default Nextest, `pse-relations/force-validate`: **466 passed, 0 failed/skipped**, 41.123 s; run `e5de7f72-42c1-4deb-8b80-39c7111117a5` |
| `just check` | Workspace/all targets/default dev profile: **0 compile errors**; upstream `proc-macro-error2` future-compatibility warning remains |
| `cargo clippy -p pse-schema -p pse-relations -p pse-catalog -p pse-rules -p pse-authoring --all-targets --locked -- -D warnings` | Default dev profile, all selected targets: **0 project lint findings**; same upstream warning |

The final source also checks dictionary ordering in declaration equality, ordered/hash
collections and forged descriptor admission, so an unchanged fingerprint cannot bypass
the exact comparison. This uses the native field as authority; no parallel physical
schema is stored.

The first 464-test run had four failures: two fixtures deliberately removed the newly
required nested role, one compared a bound nested field to its unbound physical input,
and an alias-parser test included registry size in a fixed 2 MiB overhead limit. The
corrected oracles require retained nested roles, bound child contracts and bounded alias
overhead against the same current documents without aliases, with complete claim release.
No predecessor schema or compatibility reader was restored.

`just clippy` remains unsuccessful: its latest full invocation reported **66 compiler
findings**, chiefly long functions and argument counts in the remaining pass/driver
code, plus a repeated rule-fixture closure finding. The latter and the compiler's one
needless borrow were corrected; full workspace Clippy has not been rerun after those
corrections. The no-default-features arm was not reached. These are open work with a
zero baseline, not waived findings or terminal quality acceptance.

**Tested — supporting checks:** `just quality` passes Python format/lint/types/import
boundaries and repository checks, including 14 setup/tooling tests, with 0 failures
against baseline 0. `just py-sync` refreshes the editable native extension and actual
API stub; `just doctor` reports 0 blocking issues. `just codegen-check` matches all three
schema targets and pinned Ipopt bindings on the final source. `just py-test` runs against
a fresh native store with force validation and reports **80 passed, 0 failed/skipped**
(unit/component, 18.95 s), baseline 0. These Python checks still exercise surviving
legacy handles; they do not establish their SP11 replacement. `just docs` builds with
0 errors and the existing large search-index warning, baseline 0.
Pinned skill syntax scans of the changed schema/boundary scope report 0 DataFusion and
0 Delta advisory matches. They do not prove behavioral completeness.

**Engine diagnostic:** the first targeted default-profile run hit the 120-second timeout
in `cold_stage_reopen_retains_selected_policy_and_rejects_missing_invocation`. It was
interrupted to switch to the existing CI profile: 3 passed, 1 timeout, 1 SIGINT failure,
4 not run; run `30665fc2-1bc8-4661-abcc-ff1de5ec846c`. No timeout setting or test assertion
was relaxed. The CI-profile run (`7c8fa571-4ebd-4602-bd20-f7b9f2b4858b`)
executed all 9 tests: **8 passed, 1 failed**, 781.472 s, baseline 0. The failure
was cold source replay: native Delta scans exposed `Utf8View` while the logical
contract named `Utf8`. Admission now verifies the exact versioned durable descriptor
and reconstructs its execution declaration before admitting that named adaptation.
Renamed descriptors, invalid versions and unqualified execution view fields refuse.
The follow-up command `just test-package pse-tests-engine --test unified_sources
-p pse-catalog --lib --test unified_delta_contracts --test unified_delta_dml --profile ci
--no-fail-fast --status-level fail --final-status-level fail` passed **138 tests,
0 failed/skipped**, force validation, baseline 0, 33.562 s; run
`320608b2-fc20-4130-b385-e8f13282410a`. This qualifies the corrected replay alongside
catalog and native Delta admission/DML. The other 8 engine cases passed in the preceding
CI run. These receipts precede the subsequent SP02 string-enum cut below.

### Canonical string-enum cut — 2026-09-16

**Implemented:** `pse.enum` stores the declared member spelling as native `Utf8`
in execution and Delta. Bound kinds carry an explicit nested enum field. Generated
Rust enums, Python `StrEnum` values, JSON membership and Arrow extension descriptors
all derive from the same domain. Composite-extension enum references resolve at
registry assembly and participate in relation and compiled-contract fingerprints.

The generated dictionary builders/readers and the physical-dictionary-to-`BoundKind`
guess are deleted. Authoring bindings, rule outcomes, diagnostic severity and existing
consumers use actual string values. Rule-outcome CASE branches construct checked
literals and use the common native `with_metadata` mechanism; an alias cannot invent
enum meaning. External Arrow dictionaries remain eligible under their actual contract;
a dictionary carrying the former `pse.enum` storage is rejected. No migration reader
or compatibility route was added.

Python composite storage constructs actual nested extension types, preserves child
metadata and checks exact storage schemas during IPC reconstruction. Generated
extension equality, inequality and hashing include serialized binding metadata:
PyArrow's default equality compares class/name/storage without that parameter.
The regression covers distinct domain identities, native schema equality, nested enum
binding and refusal of forged child metadata or a mismatched nested extension binding. Ordinary string
values retain string meaning; their physical shape does not imply a PSE enum.

**Tested — baseline 0:**

| Command | Mode and result |
|---|---|
| `just test-package pse-schema -p pse-ids -p pse-relations -p pse-catalog -p pse-rules -p pse-authoring -p pse-compiler --no-fail-fast --status-level fail --final-status-level fail` | Default Nextest, `pse-relations/force-validate`: **575 passed, 0 failed/skipped**, 39.989 s; run `7b3de102-0fe2-4d39-afc2-ad4c351ca504` |
| `.venv/bin/python -m pytest python/pse/tests/test_extension_round_trip.py python/pse/tests/test_generated_contracts.py -q` | Python unit contracts: **19 passed, 0 failed/skipped**, 0.69 s |
| `just py-test` | Unit/component, 32 workers, fresh generated native store with force validation: **82 passed, 0 failed/skipped**, 19.32 s |
| `just test-package pse-tests-engine --test native_normalization --test native_template_graph --test native_change_batches --test unified_sources --profile ci --no-fail-fast --status-level fail --final-status-level fail` | CI Nextest, force validation: **9 passed, 0 failed/skipped**, 763.422 s; run `0b50e0a1-7211-4059-bee0-03c6fc44f8bf` |
| `just check` | Workspace/all targets/default dev profile: **0 compile errors** |
| `cargo clippy -p pse-schema -p pse-ids -p pse-relations -p pse-catalog -p pse-rules -p pse-authoring --all-targets --locked -- -D warnings` | Selected packages/all targets/default dev profile: **0 project findings** |
| `just quality` | Python format/lint/types/import boundaries and repository checks: **0 failures**, including 14 setup/tooling tests |
| `just codegen-check` | All three schema targets and pinned Ipopt bindings match |
| `just fmt-check` | Rust/TOML format checks pass |
| `just py-sync`; `just doctor` | Editable native extension/API stub current; **0 blocking issues** |

The initial package run reported **31 failures of 575**, baseline 0. They exposed
remaining dictionary assumptions in authoring binding literals and rule-outcome CASE
annotation. Both caller paths were replaced before the passing rerun. A separate IPC
probe exposed nested extension reconstruction losing its declared storage; the generator
was corrected and the negative/round-trip regression now passes. A further parameter
probe showed native PyArrow equality accepting different enum bindings; the generated
equality and inequality methods now compare canonical serialized metadata as well as
exact native storage, with matching hashing. No assertion or admission check was
weakened to recover those receipts.

**Interface-checked:** native `datafusion_expr::expr::Expr`/`ExprSchemable` and checked
literal fields from the pinned DataFusion skill; native Delta constraints remain the
SP06/SP07 integration target. PyArrow `ExtensionType.__eq__` was inspected directly in
the installed PyArrow 25.0.1 `pyarrow/types.pxi` and probed through field and IPC APIs. The changed
DataFusion schema/boundary scope has 0 advisory skill-scan matches. Syntax scans are
not semantic proofs. The upstream `proc-macro-error2` future-compatibility warning and
the earlier full-workspace Clippy findings remain open; scoped checks do not close them.

**Remaining:** SP02 signed ordinals/counts/versions, tagged alternatives, quantity
structures and collection facets. Portable native enum CHECK/violation plans remain
part of SP06/SP07; this cut establishes canonical storage and existing membership
admission, not completion of the native invariant replacement. SP03–SP13 and the
complete compiler/store/Python caller deletion remain open. The targeted CI-profile
engine journey passes all 9 cases. No full-workspace terminal gate, cost measurement,
linked solver receipt or G1–G7 closure is claimed.

### Signed extensions and native value predicates — 2026-09-16

**Implemented:** `pse.ordinal_ref` and `pse.source_span.start/end` now use canonical
`Int64`. The recursive `IntegerRange` facet declares inclusive bounds on native fields;
ordinal references require `[0, i64::MAX]`, and source byte offsets retain the parser's
checked 32-bit bound. Rust/Python validators, JSON bounds, field fingerprints and
native predicates derive from that facet. Generated rows, authoring hydration and math
IR/compiler consumers use the signed extension representation. The parser's own byte
offset type remains a checked input boundary. Previous unsigned extension storage and
its readers are deleted; no compatibility path exists.

**Implemented:** one native expression construction path supplies publication local
violation queries and Delta local checks. It expresses enum membership, integer bounds,
finite real values, binary/list widths, visible nested nullability, source-span ordering,
bound alternatives, normalized dimension exponents and the existing quantity-sibling
obligation through DataFusion functions, operators and higher-order predicates. Map
keys/values, key uniqueness and list-view children participate recursively. Null parents
mask otherwise invalid descendants. The monolithic `AssertRelation`/Delta `Check`
Cell callbacks and the redundant pre-write filtering guard are deleted from these
routes. Other Cell consumers and the general RulePlan machinery remain open work.

Declared member tables persist their identity, version, namespace, fingerprint and
native CHECK SQL as Delta table properties, in addition to exact field descriptors.
Creation establishes constraints before the first data write; invalid first data can
leave an empty declared version 0, reported as a committed metadata operation, without
publishing the member. Existing writes reconcile actual stored fields and properties.
`DeclaredCheck::open` reconstructs recorded fields and predicates without a registry;
it checks self-consistency, not agreement with an independent registry or publication
admission. Cold scalar writes use only native Delta/DataFusion functions.

**Interface-checked limitation and bounded adapter:** at Delta `58f07cd6`, CHECK parsing
uses `GenericDialect`, which cannot parse SQL lambdas. Caller dialect settings do not
change that parser. Only collection-containing fields use a generated adapter that
executes a compiled native DataFusion expression; it has no Cell conversion or row
validator. The actual collection expression and its explicit DuckDB parsing dialect
are persisted alongside the CHECK. Missing adapter binding fails closed. Cold reopening
parses those stored expressions using a clone of the actual caller state and binds
them without changing its runtime, planner, configuration or other functions.

The pinned DataFusion unparser's dot-field rendering also loses lambda scope on SQL
round trip. Its native dialect override hook now emits explicit `get_field` calls;
the expression itself is unchanged. This was found by compiling every declared
collection predicate from its stored SQL, including nested structs. Native binary
width checks use built-in hex encoding and string length because this pin's
`octet_length` does not accept binary input. Their allocation cost is not measured.

Delta's `WriteBuilder::with_configuration` exists, but first-write validation uses
only generated-column expressions when no snapshot exists; it does not consume the
new configuration's CHECK predicates. Its internal create builder also lacks the
public create operation's custom-property acceptance option. The explicit declared
member-table create is therefore intentional. The publication control table's atomic
initialization/recovery and complete policy integration remain SP07/SP09 work; this
receipt does not claim every Delta table uses the new declaration route.

Pinned references: `datafusion_expr::expr::Expr`,
`datafusion_functions_nested::array_any_match::array_any_match`,
`datafusion_sql::unparser::Unparser` and its `Dialect` hook in the DataFusion skill;
`deltalake_core::operations::{create,write}` and
`deltalake_core::delta_datafusion::planner::DeltaPlanner` in the Delta skill. Exact
Cargo sources for Delta's `delta_datafusion/expr.rs`, `operations/write/execution.rs`
and DataFusion's `unparser/expr.rs` resolve the parser and first-write behavior.

**Tested — baseline 0:**

| Command | Mode and result |
|---|---|
| `just test-package pse-schema -p pse-ids -p pse-relations -p pse-catalog -p pse-rules -p pse-authoring -p pse-compiler --no-fail-fast --status-level fail --final-status-level fail` | Default Nextest / `pse-relations/force-validate`: **586 passed, 0 failed/skipped**, 41.692 s, `da5749c7-e8ff-483c-9f58-f06145f51e26` |
| `just test-package pse-catalog --no-fail-fast --status-level fail --final-status-level fail` | Default / force-validation after the final metadata-key separation: **202 passed, 0 failed/skipped**, 33.211 s, `72857c14-0418-49a4-80b4-e9ea7c913961` |
| `just test-package pse-catalog --lib delta::predicates --no-fail-fast --status-level fail --final-status-level fail` | Default / force-validation edge cases: **3 passed, 0 failed**, 114 unrelated tests filtered; 0.016 s, `4155c715-61ad-446d-80b6-671ba24cbbff` |
| `just py-test` | Unit/component, 32 workers, fresh generated native store with force validation: **83 passed, 0 failed/skipped**, 33.49 s |
| `just test-package pse-tests-engine --test native_normalization --test native_template_graph --test native_change_batches --test unified_sources --profile ci --no-fail-fast --status-level fail --final-status-level fail` | CI / force-validation: **9 passed, 0 failed/skipped** (2 slow), 782.316 s, `3fbb9b4b-16d4-42fa-b4f7-2eb51bf7a727` |
| `just test-package pse-tests-engine --test unified_sources --profile ci --no-fail-fast --status-level fail --final-status-level fail` | CI / force-validation after the final metadata-key separation: **1 passed, 0 failed/skipped**, 3.411 s, `edc0f220-f682-40e4-8574-e1b948d75c3f` |
| `just check` | Workspace/all targets/default dev profile: **0 compile errors** |
| `cargo clippy -p pse-schema -p pse-ids -p pse-relations -p pse-catalog -p pse-rules -p pse-authoring --all-targets --locked -- -D warnings` | Selected packages/all targets/default dev profile: **0 project findings** |
| `cargo clippy -p pse-catalog --all-targets --locked -- -D warnings` | Final metadata-key separation / default dev profile: **0 project findings** |
| `just quality` | Python format/lint/types/import boundaries and repository checks: **0 failures**, including 14 setup/tooling tests |
| `just codegen-check` | All three schema targets and pinned Ipopt bindings match |
| `just fmt-check`; `git diff --check` | Rust/TOML formatting and whitespace checks pass |
| `just py-sync`; `just doctor` | Editable extension and compiled API stubs refreshed after the final Rust/test edits; **0 blocking issues** |

The native CHECK probe initially failed on unsupported binary length/cast rendering,
lambda SQL scope and first-create write mode. The final cold-open regression uses a
field named `dialect`, establishing that field-expression properties cannot shadow the
parsing-dialect property. Earlier probes also exposed fixtures that replaced
the built-in function registry or supplied schema-level execution metadata to a raw
Delta writer. These were corrected without weakening domain assertions. Four Clippy
findings from source-span test indexing were corrected with checked signed-to-`usize`
conversion. The upstream `proc-macro-error2 2.0.1` future-compatibility warning and prior
full-workspace compiler Clippy findings remain open. The scoped check is not a whole
workspace lint receipt. The seven-package, Python and four-binary engine receipts
precede the final property-key namespace refinement; the final catalog suite and
cold-source engine rerun qualify that change, with another clean catalog Clippy run
and editable extension refresh. Source normalization/template cases still exercise
the predecessor compiler lifecycle; their success does not close its replacement.

Skill syntax scans over `pse-catalog/src/delta` found 5 DataFusion and 2 Delta advisory
matches: a test-only default session, missing statistics on three command outcomes,
the physical-input bridge's deliberately unsupported filter pushdown and two writes
without schema evolution enabled. Delta exposes only merge/overwrite schema modes;
omitting a mode is intentional for exact ordinary writes. Command statistics and
complete common provider policy remain later-plan work. These are syntax observations,
not measured performance or semantic defect counts.

**Remaining:** ordinary PSE unsigned counts/versions/ordinals, general tagged
alternatives, typed quantities and declared collection facets; typed keys/support and
coherent expression/numerical rows; common native invariant/domain transfer; full Delta
table policies, root publication/recovery and the compiler/store/Python caller cut.
The old product store, Driver/stage/memo paths and closed rule representation are still
present. SP00–SP13 and G1–G7 are not certified complete. No new simulator behavior was
added, and no measurement or linked-solver acceptance is claimed.

### Signed metadata and tagged configuration — 2026-09-16

**Implemented:** canonical signed storage now covers registry versions and declaration
ordinals, publication member/control versions and command outcomes, package depth,
change-operation/staged-row ordinals, case sampling counts and ordinals, runtime
attempt/iteration/kernel-outcome positions and terminal evidence counts/positions.
Existing width bounds are generated from `IntegerRange`. Declaration materialization,
fingerprints, generated Rust/Python/JSON contracts, native list-length expressions and
the existing authoring/compiler/publication callers use the new representation.

`PublicationRoot` and PSE Delta command results now use `Int64`. Conversion to the
pinned kernel's unsigned `Version` rejects negative values; conversion back rejects
overflow. A write whose result cannot be represented retains its actual committed
version in `MutationError::Committed`. DataFusion's required native DML `UInt64`
`count` result remains its external protocol. Full-width unsigned literal/configuration
values and random seeds remain eligible; this cut adds no unsigned compatibility
reader for replaced PSE fields.

**Implemented:** `TaggedAlternative` binds a required discriminator to distinct optional
struct arms on the native field tree. Its metadata contains the selection relationship;
payload types remain the Arrow children. Registry admission checks arm shape, complete
enum coverage and exact metadata. Generated Rust constructors and borrowed enum
selectors, generated Python post-initialization validation, JSON Schema `oneOf`, imported
Arrow admission and native DataFusion/Delta predicates all project that declaration.
Unknown tags, missing selected arms and overlapping arms fail; null outer values mask
their children.

Selection itself requires no UDF. Configuration's index arm still uses the existing
persisted native-collection expression adapter because this Delta pin cannot parse
the required lambda SQL. No new configuration validator UDF is introduced.

Configuration, feature candidates, resolved features and their generated provenance
values now use this declared shape. Scalar payloads have a required `value` child;
the enum arm carries its dictionary identity and member together. The quantity arm
uses the existing typed quantity structure with required value, quantity type and unit.
Existing unit/dimension/reference checks remain active. Compiler parsing constructs
generated arms; scalar consumers use generated selectors, and native feature/selector/
balance queries read the corresponding children. The manually assembled empty value,
the duplicated configuration `RulePlan` shape rules and the field-by-field feature-value
copy are deleted. No old-shape reader is retained.

Normalized domain and instance references now update the actual field metadata.
The earlier native-field cut had left one caller mutating the temporary reference
returned by `fk()`, so generated domain members and child bindings still referenced
the authored-only inventory. The declaration now targets the complete normalized
inventory, with a regression checking both the declaration and execution metadata.
Numeric-domain checks also consume the new quantity arm's value.

**Interface-checked:** DataFusion 55.1.0's `datafusion_session::table::TableProvider`
documents unsigned native DML count results. Its
`datafusion_functions_nested::length::ArrayLength` returns `UInt64`; the PSE list-length
projection now uses a native checked `Int64` cast. Delta `58f07cd6` and kernel `8ba063f8`
retain their unsigned external version interface. These checks used the pinned local
skills and Cargo sources. Context7's upstream attrs documentation confirms that
`__attrs_post_init__` runs after field validators; actual generated construction is
tested against the pinned attrs 26.1.0/cattrs 26.2.0 environment.

**Tested — focused contracts, baseline 0:**
`just test-package pse-schema --test native_field_declarations -p pse-relations --test tagged_values -p pse-catalog --test tagged_delta_values --no-fail-fast --status-level fail --final-status-level fail`
passed **15 tests, 0 failed/skipped**, default Nextest with force validation, 0.954 s,
run `a1c99695-d843-414a-8944-c07a65741623`. This includes a cold raw Delta writer with
only native functions enforcing selection, child ranges and null-parent visibility
on a scalar tagged fixture,
without a PSE CHECK UDF or registry. Invalid writes leave the committed version unchanged.

**Tested — current source, baseline 0:**

| Command | Mode and result |
|---|---|
| `just test-package pse-schema -p pse-ids -p pse-relations -p pse-catalog -p pse-rules -p pse-authoring -p pse-templates -p pse-compiler --no-fail-fast --status-level fail --final-status-level fail` | Default Nextest / force validation after all reference/quantity and regression corrections: **617 passed, 0 failed/skipped**, 45.759 s, `6b5f2978-2cc2-4797-8497-4da35604ba7e` |
| `just py-test` | Unit/component, 32 workers, fresh native inspection fixtures after the reference/quantity corrections: **84 passed, 0 failed/skipped**, 29.33 s |
| `just test-package pse-schema -p pse-relations --test native_field_declarations --no-fail-fast --status-level fail --final-status-level fail` | Default / force validation, including the reference correction: **13 passed, 0 failed/skipped**, 0.968 s, `a97d43fe-11f4-49cd-b55b-01a1bcf5fcc2` |
| `just test-package pse-tests-engine --test native_normalization --test native_template_graph --test native_change_batches --test unified_sources --test terminal_attempts --profile ci --no-fail-fast --status-level fail --final-status-level fail` | CI / force validation: **16 passed, 0 failed/skipped** (2 slow), 908.103 s, `2bc50ab7-c805-4e44-a158-843b370f3a1e`; precedes the final equivalent generated-validator cleanup and reference/quantity corrections |
| `just test-package pse-tests-engine --test native_engineering_workflows --profile ci --no-fail-fast --status-level fail --final-status-level fail` | CI / force validation after the reference/quantity corrections: **4 passed, 0 failed/skipped** (4 slow), 1346.628 s, `30fb677b-2679-43df-a6c5-6db53fa0e98b`. Heater/mixer FTPx and FcTP source-to-P10 assertions; existing one-worker, 32 GiB runtime per case and two-case test group. |
| `just check` | Workspace/all targets/default dev profile: **0 compile errors or project warnings** |
| `cargo clippy -p pse-schema -p pse-relations -p pse-catalog -p pse-rules -p pse-authoring -p pse-templates --all-targets --locked -- -D warnings` | Selected packages/all targets/default dev profile: **0 project findings**, after generator corrections |
| `PS1='' just quality` | Python format/lint/types/import boundaries and repository checks: **0 failures**, including 14 setup/tooling tests; empty prompt avoids an unrelated system Bash startup failure on the recipe's comment line |
| `just codegen-check`; `just fmt-check`; `git diff --check` | Regeneration, Rust/TOML formatting and whitespace checks pass |
| `just py-sync`; `PS1='' just doctor` | Editable native extension and compiled API stubs refreshed after the reference/quantity corrections; final environment check reports **0 blocking issues**. A final sync refreshed uv's source fingerprint after the regression-test edit; system Bash still prints its unrelated unset-prompt startup diagnostic. |

The first generated-check refactor emitted empty optional-value guards; large signed
bound literals and a nested single predicate also triggered Clippy findings. The
generator now omits empty checks, formats bounds readably and composes parent visibility
with the single predicate. The final eight-package receipt includes those corrections.
The upstream `proc-macro-error2 2.0.1` future-compatibility warning and full-workspace
Clippy qualification remain open; the selected-package receipt is not a workspace
lint receipt. No measurement or linked-solver acceptance is claimed.

Initial validation found an unsigned-only sort-oracle fixture, old dynamic Cell
writers/comparisons, and native queries selecting the former flat configuration
children. These were updated with the caller cut. The generated Python cross-field
validator initially violated the exception-message lint; the generator was corrected.
No baseline was introduced and no domain assertion was weakened.

The expanded heater/mixer run initially failed all four cases at P3 publication
because of the stale normalized foreign keys described above. A new regression's
first assertion incorrectly treated execution metadata as registry-declaration
metadata; it now checks the execution `pse.semantic.fk` projection explicitly.
The schema-only test command also requires selecting `pse-relations` for the recipe's
force-validation feature; the corrected command is recorded above.
The final heater/mixer rerun passes all four cases. Its duration is a test-run receipt,
not a controlled performance comparison; architecture cost qualification remains open.

**Remaining:** signed expression/graph IDs and the other ordinary ordinal/count sites;
alternatives outside configuration/features; general typed quantities, explicit
collection semantics, keys/references/support, coherent expression/numerical rows,
common invariant/domain transfer, full table/root policies and the provider/compiler/
store/Python replacement/deletion. The predecessor store, Driver/stage/memo paths and
general RulePlan representation still exist. SP00–SP13 and G1–G7 remain uncertified;
this receipt adds no simulator behavior or cost measurement.

### Remaining execution sequence — 2026-09-16

This was the checkpoint at which the detailed E00–E10 sequence was authorized.
The maintained implementation state and remaining work now live in
[current remaining scope](#current-remaining-scope--2026-09-17). Earlier receipt
paragraphs labelled “Remaining” describe their source boundary; they are not the
current backlog and must not cause completed schema cuts to be implemented again.

### Schema and publication contract continuation — 2026-09-16

**Implemented; package acceptance remains open.** The continuation applies E01
schema foundations and bounded E05/E06 changes needed by their consumers:

- Operator, arity, equation-sense and template-reference declarations now belong to
  `pse-schema::math`. `pse-schema` no longer depends on `pse-mathir`; math consumes
  the schema crate. The old `opspec.rs` and its re-exports are removed. This makes
  generated coherent math rows possible without a dependency cycle; it does not
  establish that the graph replacement has happened.
- `TaggedAlternative` supports payload-free alternatives and multiple tags sharing
  a payload shape. Generated Rust constructors/selectors, Python checks, JSON Schema
  and native predicates follow the same declaration. Publication selection is now
  `full` or `revision { column, revision_id }`; the two independent nullable fields
  are deleted. Revision columns must carry the semantic identity declaration.
- Kernel outcomes contain a selected result: success carries a typed quantity;
  failure variants share a required reason/quantity/unit payload. Incidence contains
  a linear coefficient arm or a payload-free nonlinear alternative. These changes
  restructure existing declarations and add no numerical execution functionality.
- `QuantityContract::PerRow`, its sibling naming rule, registry metadata flag and
  corresponding generator/admission checks are deleted. Heterogeneous values use
  `QuantityValue`; homogeneous field quantities remain. Cross-relation unit/quantity
  agreement still belongs to the uncompleted reference/invariant work.
- Native list fields declare order, cardinality and uniqueness. Lists retain Arrow
  nullability and child nullability, so absent and empty are distinct. Package
  dependencies are a set; implicit-system lists preserve order and prohibit repeated
  members. Rust/Python/JSON checks and native `array_length`/`array_distinct` predicates
  are generated. Equal cardinalities between related collections remain open.
- Ordinary derived-structure counts and positions in `s6_12_derived` use bounded
  signed fields. Expression IDs and other remaining ordinal sites are still open.
- `SessionFactory::from_builder` retains the actual native configuration, including
  opaque extensions. Its separate settings/budget arguments and replacement of the
  caller's configuration are removed. The convenience constructor explicitly builds
  its selected configuration before delegating. Tests follow the actual configuration
  through nested execution and policy overrides.
- Root and member opens verify persisted declared Delta properties and native CHECK.
  Root initialization installs the schema and CHECK before its first data write.
  An initialized empty control table is not a publication. Initial publication is
  version 1; callers use the actual returned version. Concurrent initialization and
  lost initialization responses reconcile the real declared table; publication
  identity applies only to the subsequent data transaction. Existing member/root
  retries continue to check exact requests. This does not close member-attempt reuse.
- Publication input verification transfers the native Arrow input column into the
  identical member contract; its generic Cell reconstruction is removed.

**Pinned capability evidence — Interface-checked and exercised by the tests below:**
the DataFusion skill's `datafusion_functions_nested.set_ops.md`,
`datafusion_functions_nested.length.md`, `datafusion.execution.session_state.md` and
`datafusion_execution.config.md` establish native list predicates and configuration
ownership. The Delta skill's create/write builder references and pinned upstream
`operations/write/mod.rs` establish that checks must precede the first data write.
Capability-gap scans of the edited catalog paths found ordinary mutation nodes with
unknown statistics, a physical-child bridge without filter pushdown, tiny test
sessions and native writes with schema mode unset. No merge/overwrite schema mode is
selected: exact field admission and contract verification require schema refusal.
The mutation commands retain their real inputs and execution effects; no optimization
claim is inferred from these syntactic hints.

**Corrections found during validation:** `serde_json/preserve_order` feature unification
changed tagged metadata fingerprints and JSON Schema alternative ordering. Both now
sort object keys explicitly, while preserving array order. A raw cold-writer fixture
initially used the declared storage schema rather than Delta's reconstructed physical
schema; it now takes that schema directly from the loaded Delta table.

**Remaining:** E00–E10 and SP00–SP13 are not certified. In particular, coherent math
rows, the remaining signed ordinals and alternatives, typed keys/references/support,
dimensioned numerical rows, common invariant/domain transfer, complete table policies,
member-attempt reconciliation, provider/compiler/store/Python caller replacement,
native change/reuse/retention, legacy deletion and terminal architecture measurements
remain open. The predecessor graph/store, Driver/stage/memo and RulePlan/RuleExpr
paths still exist. There is no compatibility layer for the contracts replaced here.

#### Continuation verification

**Tested, baseline 0; no terminal architectural acceptance:**

| Command | Actual evidence and limits |
|---|---|
| `just test-package pse-schema -p pse-relations -p pse-mathir -p pse-catalog --no-fail-fast` | Default Nextest with force validation: **434 passed, 0 failed/skipped**, 42.611 s. Includes generated typed alternatives/quantities, native collection predicates, cold collection CHECK, exact member/root admission, concurrent initializers and lost-response reconciliation. |
| `just test-package pse-tests-engine --test unified_sources --profile ci --no-fail-fast` | CI / force validation: **1 passed, 0 failed/skipped**, 3.613 s; `33376d2e-0a9a-4a82-8a0d-6b891723753b`. Exact source reopening, reparsing and support queries; no broader simulator qualification. |
| `just py-sync`, `just py-test` | Editable native extension/stubs rebuilt; Python 3.14.7 unit/component suite: **86 passed, 0 failed**, 21.57 s. |
| `just quality` | Python/repository checks: **0 findings**; includes **14 setup tests passed**. |
| `just governance` | Default force-validation governance: **69 passed, 0 failed/skipped**, 3.972 s; `a1e805f8-1c4d-4528-a978-d7ff87dc2fa4`. All three schema generation targets, Ipopt bindings and pinned dependency-family checks pass. |
| `just check` | Workspace, all targets, locked default compilation succeeded. Cargo still emits the upstream `proc-macro-error2` future-incompatibility notice. |
| `just clippy` | Default workspace/all-target pass fails with **65 compiler lint errors**, baseline 0. The no-default-features pass was not reached. |

The first 434-test run had one invalid cold-writer fixture and the first governance
run had one cross-feature JSON ordering mismatch; their causes and corrections are
described above. Scoped Clippy for the four changed packages passed with `-D warnings`.
Compiler findings are not an accepted baseline or a
completion waiver. Linked-solver, full engine, feature-matrix, cost and terminal
architecture gates are not certified by this continuation.

### Native nested references and row predicates — 2026-09-16

**Implemented; package acceptance remains open.** Native occurrence plans now walk
declared structs, lists, list views and maps. They mask parents before extracting
children and preserve visible occurrence multiplicity. The previous source-span-only
walker and publication's top-level-only FK loop are replaced by this shared traversal.
Foreign-key validation uses native anti joins against the exact selected target in
the same catalog. Conflicting selected target revisions refuse; a visible unresolved
reference violates admission. Null and empty containers make no reference claim.
This does not implement composite mappings or typed diagnostic/key output yet.

Typed scalar `QuantityValue` occurrences now join the selected quantity-type and unit
relations, including the type's canonical unit. Missing definitions, unequal dimensions,
non-scalar type shapes and incompatible reference-state restrictions refuse admission.
An ambient table outside the selected publication cannot satisfy these obligations.
This is reference/representation admission, not a second numerical conversion engine;
the complete native physical-type domain invariants remain part of E04.

Relations can declare named **native DataFusion SQL** row predicates. The registry's
own relation rows, fingerprints, exact Arrow schema metadata and generated inspection
interfaces carry the declaration. Actual caller state binds and type-checks the
predicates; non-Boolean and volatile expressions refuse. Publication admission and
persisted Delta CHECK both require true, so SQL NULL cannot bypass them. Cold Delta
reconstruction verifies the recorded checks against the table configuration before
binding; extra and removed CHECK constraints both refuse exact admission. This adds
no parallel expression language or row interpreter.

Compiled and inferred implicit-system declarations now require equal unknown/equation
list cardinalities through this shared native check. Existing ordered uniqueness stays
on each list's field declaration. Rust/Python value constructors continue to establish
their documented local field contracts; cross-field SQL and cross-relation checks run
at native candidate/write admission. Replacing every predecessor caller with that
boundary remains mandatory in E05–E08.

**Interface-checked:** DataFusion 55.1.0 skill references for
`datafusion_common::unnest::UnnestOptions`, native `map_keys`/`map_values`,
`datafusion_functions::core::expr_fn::named_struct`, native binary expressions and
`SessionState::create_logical_expr`; Delta commit `58f07cd6` create/write/constraint
interfaces. Capability-gap scans found the previously recorded mutation statistics,
test-session and schema-evolution hints; no new runtime workaround was introduced.

**Corrections found during validation:** `map_entries` normalizes entry child names
at this pin; paired native UNNEST of keys and values plus `named_struct` preserves the
declared names and correlation. The initial Delta retry fixture reopened no snapshot
after the failed data write had already created an empty declared table. It now loads
the actual table before retry. Native validation errors contain the invalid-row preview,
not the named constraint, so the test asserts the actual validation error and unchanged
data version rather than an invented diagnostic field.

**Remaining:** this closes neither E01/E02/E04 nor SP00–SP13. Signed graph IDs,
coherent math/numerical rows, other alternatives, exact/composite keys and references,
structured support/diagnostics, common domain transfer and rule replacement, complete
table policies, member-attempt reuse, the provider/compiler/store/Python caller cut,
native reuse/retention and terminal deletion/measurement gates remain open. No new
simulator function, legacy reader or compatibility path was added.

#### Native reference/predicate verification

**Tested, baseline 0:** focused catalog library plus native row-check integration
(`just test-package pse-catalog --lib --test declared_row_checks --no-fail-fast`)
passed **130 tests, 0 failures/skips**, default Nextest with force validation,
12.273 s. This includes cold registry-free CHECK enforcement, failed first writes,
removed-constraint refusal, quoted nested names, null/empty containers, list-view
slices, map/struct masks, exact catalog/revision scope and quantity mismatches.

**Tested, baseline 0; final continuation evidence:**

| Command | Actual evidence and limits |
|---|---|
| `just test-package pse-schema -p pse-relations -p pse-mathir -p pse-catalog --no-fail-fast` | Default Nextest / force validation: **446 passed, 0 failed/skipped**, 44.925 s; `9efa03f1-b2ad-4735-9e94-f2aabb0ead43`. |
| `just test-package pse-catalog --test declared_row_checks --test unified_delta_contracts --no-fail-fast` | Default / force validation, after enforcing the complete CHECK set: **24 passed, 0 failed/skipped**, 14.677 s; `79da1100-4d67-46f7-8fbe-f08c39b9df48`. |
| `just governance` | Default / force validation: **69 passed, 0 failed/skipped**, 3.919 s; `e9e032e0-b3bf-44c1-a1e6-df0f3c281977`. Regeneration and family checks pass. |
| `just test-package pse-tests-engine --test unified_sources --profile ci --no-fail-fast` | CI / force validation: **1 passed, 0 failed/skipped**, 3.960 s; `4f494617-b0fc-4820-80eb-c77fac1b8dd9`. |
| `just test-package pse-tests-engine --test memo_dependencies --test terminal_attempts --profile ci --no-fail-fast` | CI / force validation, updated registry-copy fixtures: **11 passed, 0 failed/skipped**, 86.069 s; `76f77911-95c1-4d26-8ca7-598ff0341270`. This retains their existing assertions while their caller replacement remains open. |
| `just py-test` | Editable extension, Python 3.14.7 unit/component: **86 passed, 0 failed**, 21.82 s. |
| `just quality` | Python/repository checks: **0 findings**, including 14 setup tests. |
| `cargo clippy -p pse-schema -p pse-relations -p pse-mathir -p pse-catalog --all-targets --locked -- -D warnings` | Scoped default/all-target check: **0 findings**. No recipe exposes this package-scoped lint invocation. |
| `just check` | Workspace/all-target/default compilation succeeds. Two registry-copy fixtures were updated to retain native check declarations. |
| `just py-sync`, `just doctor` | Final editable rebuild and actual API stubs succeed; doctor reports **Environment ready**. |

The first broad continuation run had **442 passes and 1 failed retry fixture**;
the corrected broad run above is green. Map-name and fixture/API mistakes were
corrected and retested. These results do not close terminal architecture acceptance.
The last full `just clippy` run still has **65 compiler errors**, baseline 0;
the no-default-features pass was not reached. The upstream `proc-macro-error2`
future-incompatibility notice also remains. No full linked-solver, feature-matrix,
cost or legacy-deletion qualification is claimed.

### Typed keys, composite references and diagnostic evidence — 2026-09-16

**Implemented; SP03 and terminal acceptance remain open.**

- Primary-key declarations distinguish omission from an explicitly empty singleton
  key. Registry admission rejects omission; canonicalization and native candidate
  admission accept zero or one singleton row and reject more. Providers do not
  advertise an empty-column primary-key constraint.
- `ReferenceContract` declares correlated struct-child-to-target mappings with
  required or all-or-none presence. Exact field names remain literal, including dots.
  A mapping cannot split independent collection occurrences. Scalar FK convenience
  declarations enter the same native validation path. Native occurrence projections,
  anti joins and target-key grouping establish existence and uniqueness in the exact
  selected catalog/revision; ambient tables cannot satisfy the reference. Presence
  checks also persist in Delta CHECK and reject partially present composite keys in
  a fresh writer after the original registry and session are dropped.
- `pse_row_key` replaces the deleted JSON key UDF and reversible key-codec module.
  It frames relation identity, ordered declared names/types/domain identity and
  pinned Arrow row bytes into `FixedSizeBinary(32)`. The explicit format contract is
  `pse:row-key:arrow-row-59.3:v1`. This qualifies one pinned encoding; it does not claim
  Arrow row bytes are stable across library upgrades. Primary-key values remain the
  authority and selected revision stays explicit in surrounding references.
- Candidate publication checks group actual distinct key tuples by token and refuse
  collisions. Tests inject a constant token to exercise the refusal. Row keys survive
  payload edits, slicing, reordering and independent converters; dictionary remapping
  retains identity, relation scope distinguishes it, and a frozen vector guards the
  pinned contract. Existing rule/support/source/compiler consumers now carry typed
  tokens. There is no reader or fallback for the removed text-key encoding.
- Expression sources now use that same native token. The list of per-type key-part
  alternatives, parser key-to-Cell decoder and compiler key reconstruction are
  deleted. Configuration guards, property seeds, port/law guards and outer-guard
  queries compare native scoped tokens. The realization algorithm builds its needed
  declaration lookup indexes from native identity/key projections of the selected
  inputs; these are derived layouts, not a persisted graph or key authority.
- Diagnostic evidence is a generated tagged value: a row arm holds relation identity
  and its typed key; an execution arm holds failure class, diagnostic code and attempt
  error. Native invariant projections construct the row arm. The compiler uses the
  generated terminal-finding builder; its positional nine-cell execution writer and
  JSON diagnostic payload are deleted. The selected snapshot/publication boundary
  still needs the complete store replacement described below.
- Concrete invariant fixtures now cover the existing conditional, demand, guard and
  selection obligations previously absent from the generator. Expected violating
  keys follow each invariant's declared key projection, independently of rule
  evaluation. `just conformance-fixtures` regenerates these reviewed examples and
  removes fixture directories for declarations no longer present.

**Interface-checked:** DataFusion/Arrow skill references for
`arrow_row::RowConverter`, `ScalarUDFImpl::return_field_from_args`, native grouping,
anti joins and struct construction; Delta skill references for declared checks and
native create/write builders at commit `58f07cd6`. The RowConverter documentation
explicitly limits cross-converter ordering and format stability; equality bytes here
are qualified by exact-pin tests and an explicit versioned contract. Capability-gap
scans retain the earlier physical-child statistics/pushdown and schema-mode findings;
they do not establish runtime optimization or close those plan obligations.

**Tested, baseline 0; interim receipts:** workspace/all-target `just check` and
three-target `just codegen` succeed. The focused native key/reference/rule run
(`just test-package pse-catalog -p pse-rules --lib --test row_keys --test native_construction --test four_valued_rule_outcomes --no-fail-fast`)
passes **154 tests**, default Nextest/force validation, 22.438 s. The diagnostic/compiler
run (`just test-package pse-rules -p pse-compiler --test invariant_execution --lib --no-fail-fast`)
passes **36 tests**, default/force validation, 14.987 s. The fixture run
(`just test-package pse-tests-conformance --test invariant_fixtures --no-fail-fast`)
passes **2 tests**, default/force validation, 16.029 s; the fixture test iterates
every actual registered invariant's valid and violating examples.

**Corrections during validation:** the initial 584-test run had 578 passes and six
failures. A remaining text concatenation for derivation identity was replaced by the
typed key; truth-outcome tests now match actual keys instead of depending on hash
ordering. New invariant fixtures exposed incomplete guard absence values, corrected
to omit both identity and node. Diagnostic generation exposed remaining JSON writers,
which were replaced together with their generated fields and fixtures. The focused
passing receipts above do not stand in for a final broad rerun.

The fresh-source regression exposed a scope mismatch in configuration domain
projection: it minted a normalized key and looked it up under the authored relation.
The source projection now uses the actual authored scope. Another failure exposed
DataFusion 55's top-level metadata loss through native CASE/COALESCE. Construction
now selects already joined source tokens through checked native CASE expressions,
retaining the established key contract. The expression-key cut exposed four remaining
guard queries that unpacked the removed key representation; they now compare tokens.
The expanded eight-package run initially passed 650 of 651 tests; the failing empty
property-seed test and all 27 compiler unit tests pass after that correction. Scoped
Clippy then exposed fixture-only import placement, an unreachable assertion and an
oversized dispatch function; the fixture helpers now pass without lint suppression.

#### Typed identity verification

**Tested, baseline 0; no terminal architecture acceptance:**

| Command | Actual evidence and limits |
|---|---|
| `just test-package pse-schema -p pse-ids -p pse-relations -p pse-catalog -p pse-rules -p pse-authoring -p pse-templates -p pse-compiler --no-fail-fast --status-level fail --final-status-level fail` | Default Nextest / force validation: **651 passed, 0 failed/skipped**, 66.174 s; `eda716ad-f97c-4e56-96f7-3e76794da8d3`. Includes the final expression-source and key-encoding changes. |
| `just test-package pse-tests-conformance --test invariant_fixtures --no-fail-fast` | Default / force validation: **2 passed, 0 failed/skipped**, 14.093 s; `1c5a77d5-3844-4863-8c79-9baeab999022`. Every registered invariant's concrete valid/violating fixture executes. |
| `just test-package pse-tests-engine --test terminal_attempts --profile ci --no-fail-fast` | CI / force validation: **7 passed, 0 failed/skipped**, 103.045 s; `42230ab6-8129-4512-ad57-7b2608e036a0`. Typed terminal evidence; precedes the subsequent expression-source cut. |
| `just test-package pse-tests-engine --test native_engineering_workflows heater_fctp --profile ci --no-fail-fast` | CI / force validation: **1 passed, 0 failed, 3 filtered**, 790.203 s; `d376f6fb-3994-4f71-bf81-528d09cad612`. Qualifies the construction scope/metadata fixes before the expression-source cut; not final whole-source certification. |
| `just governance` | Default / force validation: **69 passed, 0 failed/skipped**, 3.974 s; `4eae4cac-f989-4c24-b6c1-2847caf24f20`. Generated targets and dependency-family checks pass. |
| `just py-test` | Final editable rebuild, Python 3.14.7 unit/component: **86 passed, 0 failed**, 26.21 s. |
| `cargo clippy --no-deps -p pse-schema -p pse-ids -p pse-relations -p pse-catalog -p pse-rules -p pse-authoring -p pse-tests-conformance --all-targets --locked -- -D warnings` | Selected packages / all targets: **0 findings**. No recipe exposes package-scoped linting. The earlier invocation without `--no-deps` also linted the compiler and failed on its 65 findings. |
| `just check`, `just quality` | Workspace/all-target/default compilation and Python/repository quality pass; quality includes 14 setup tests. |
| `just py-sync`, `just doctor` | Final editable extension and API stubs refreshed; environment ready, **0 blocking issues**. |
| `just clippy` | Default workspace/all targets: **65 compiler findings**, baseline 0; no-default-features pass not reached. |

The final capability-gap scan of the new key function and edited native source/guard
queries produced no syntax matches. It is a discovery check, not a proof of complete
library use or performance. The upstream `proc-macro-error2` future-incompatibility
notice remains. The expanded source workflow suite is still being qualified below.

**Remaining:** E00–E10/SP00–SP13 remain uncertified. Algorithm support relocation,
remaining unsigned ordinals/alternatives, coherent math
and numerical rows, RulePlan/RuleExpr/domain transfer, full provider/compiler/store/
Python replacement, member-attempt reconciliation, native reuse/retention, legacy
deletion and architecture measurements remain open. Useful graph algorithms remain
eligible as derived, dependency-bound layouts under native execution; predecessor
graph authority is not retained by that exception. Full `just clippy` still reports
**65 compiler errors**, baseline 0; the no-default-features pass is not reached.
The final Python/governance receipts are above. Expanded source workflow qualification
remains in progress; no whole-plan completion is claimed by this continuation.

**Next implementation boundary at this checkpoint:** replace
`OutputRows`/configuration origin side vectors and support relocation with
co-located typed occurrences (implemented in the continuation below). Keep the actual
primary-key values and selected input as authority. Then execute E03's coherent expression/numerical cut
before the combined E05–E08 provider/compiler/store/Python replacement. These
remaining mechanisms are not grandfathered by the typed-key implementation.

### Source locations and native field proofs — 2026-09-16

**Implemented:** case specifications, activations and observations carry their
parser-produced `source_span` in the declared row. Hydration supplies it through the
existing `SourceColumn::ParserSpan` contract and refuses authored overrides. P1 reads
that generated value directly. `Document.row_spans`, its allocation accounting and
the concatenated ordinal-to-span lookup are deleted. Parser offsets remain bounded
`u32` values; their declared Arrow fields are bounded nonnegative `Int64`. Conversion
checks range and order explicitly.

**Tested, baseline 0:**
`just test-package pse-authoring --test source_locations --test rename --test owned_documents --no-fail-fast`
passes **13 tests, 0 failed/skipped**, default Nextest / force validation, 6.434 s.
The new source-location test sorts, filters and unions values from two documents
after dropping the parser bundle, then checks each retained span against its exact
original text and row identity. Existing identity-preserving rename tests pass with
the new declared fields. `just codegen-bootstrap` regenerates all three schema
targets; no generated files were edited manually.

**Implemented:** native CASE construction now accepts a flat list of alternatives,
checks their actual field meaning once per alternative and retains only common
annotations. The native row-key UDF reuses its intrinsic output-field declaration
and uses Arrow's `RowConverter::supports_fields` check during planning. Native output
admission explicitly compares the row-key encoding facet, including nested fields;
it cannot turn a generic content hash or an incompatible key encoding into a declared
row key by annotation.

**Tested, baseline 0:**
`just test-package pse-catalog --test row_keys --test native_output --no-fail-fast`
passes **11 tests, 0 failed/skipped**, default / force validation, 9.778 s. The flat
CASE test executes multiple alternatives and checks first-match values and field
metadata. The key suite includes the frozen encoding vector and refusal of an
ordinary hash or another encoding at both top-level and list-child boundaries.

**Measured — one diagnostic run; no speedup or architecture acceptance:** the expanded source suite completed
with **12 passes and 1 template timeout**, CI / force validation, 1561.507 s
(`c6c151eb-7e7d-40ac-a94d-f595d6557add`). A separate heater FCTP run passed in
763.345 s (`21c716f6-6a67-4904-83f1-928539a45718`). These precede the final source-span
and field-admission changes. Neither output-field reuse nor flattening CASE brought
the template workflow below the original 360-second limit. The stage-timed diagnostic
passed in **395.369 s**, CI / force validation, one worker and a 32 GiB shared memory
ceiling (`8f886575-c442-486d-979f-0a8eae7659ae`). It used
`just test-package pse-tests-engine --test native_template_graph --profile ci --config-file /tmp/pse-plan08-nextest-diagnostic.toml --no-fail-fast --success-output immediate`.
That temporary configuration copied the ordinary Nextest configuration and added a
CI override for only this binary with `period = "300s", terminate-after = 3`.
Other qualification/build commands ran concurrently; this is not an isolated benchmark.

The invocation-to-publication trace measured P3 **54.236 s**, P4 **3.349 s**,
P5 **14.521 s**, P6 **22.001 s**, P7 **13.809 s**, P8 **23.669 s**,
P9 **57.829 s** and P10 **3.306 s**. These intervals exclude source admission and
between-stage work and therefore do not partition the complete runtime. They do not
establish a root cause for that remaining time. The checked-in template-only allowance
is now **600 seconds**, retaining its serial group. The duplicate run under that
allowance (`2b7390d1-4a0f-458a-a20f-89670684b756`) was stopped as redundant; the
395.369-second run already established the exercised functional result. The larger
timeout prevents premature termination of future runs; no timeout change establishes
a latency improvement or requires repeating a functional proof by itself.

#### Current identity and source-location verification

**Tested, baseline 0; no whole-plan acceptance:**

| Command | Actual evidence and limits |
|---|---|
| `just test-package pse-schema -p pse-ids -p pse-relations -p pse-catalog -p pse-rules -p pse-authoring -p pse-templates -p pse-compiler --no-fail-fast --status-level fail --final-status-level fail` | Default Nextest / force validation: **654 passed, 0 failed/skipped**, 64.195 s; `76ef46e4-ae79-45dd-904c-1e1c28f02087`. Includes source-span columns, flat CASE, intrinsic output-field reuse and key-encoding admission. |
| `just test-package pse-tests-conformance --test invariant_fixtures --no-fail-fast` | Default / force validation: **2 passed, 0 failed/skipped**, 14.471 s; `2edb9ede-c897-4683-8978-be4067ff7ec1`. Fresh generated fixtures. |
| `just governance` | Default / force validation: **69 passed, 0 failed/skipped**, 4.441 s; `4e707bdf-7841-4669-92fb-8b71c2667d7e`. Three-target regeneration and dependency-family checks pass. |
| `just py-sync`, `just py-test` | Current editable extension and actual API stubs rebuilt; Python 3.14.7 unit/component: **86 passed, 0 failed**, 25.48 s. |
| `cargo clippy --no-deps -p pse-schema -p pse-catalog -p pse-authoring -p pse-tests-conformance --all-targets --locked -- -D warnings` | Selected packages / all targets: **0 findings**. The first source-location test run exposed fixture assertion/import formatting issues; corrected before this receipt. |
| `just check`, `just quality`, `just doctor` | Workspace/all-target/default compilation passes; Python/repository quality passes including 14 setup tests; environment ready with **0 blocking issues**. |
| `just clippy` | Current workspace/default/all-target attempt still fails with **65 compiler findings**, baseline 0. The no-default-features pass is not reached. |

The final targeted DataFusion capability-gap scan reports no syntax matches. This is
discovery evidence only. The upstream `proc-macro-error2` future-incompatibility notice
remains. Solver-linked, complete feature-matrix, docs and terminal architecture gates
are not certified by these receipts.

**Remaining at this checkpoint:** support co-location was still open; the next
continuation replaces `OutputRows` and configuration side vectors and deletes their
ordinal relocation joins. E02 is not yet certified. Coherent math/numerical
rows, common native rule/domain transfer, the complete provider/compiler/store/Python
caller cut, member-attempt reconciliation, native change/reuse/retention, legacy
deletion and final Q01–Q14 qualification remain required. Useful derived graph
algorithms remain eligible; no predecessor graph authority is retained as an exception.

### Co-located algorithm and configuration support — 2026-09-16

**Implemented; qualification in progress.** Each specialized algorithm result now
contains its source set as a required `List<Struct>` column beside its payload. The
shared field declaration includes the exact input role, relation identity and typed
row token; the intrinsic logical-type catalog registers the entire nested contract.
Collection admission requires at least one distinct, nonnull member for each visible
output row. Empty relations remain valid. These are associations; native joins to
actual immutable input owners still establish source membership using actual key
fields and selected inputs.

Native UNNEST replaces the separate occurrence provider, two ordinal-coverage
anti-joins and ordinal inner join. `SnapshotSession::with_columnar_argument` uses the
existing immutable materialized provider and shared memory budget, admitting actual
fields and values without claiming relation keys or producer validity. Buffer leases
remain attached to the resulting arrays. Value admission executes the same native
field predicates used for Delta publication; it adds no Cell validation interpreter.

Configuration uses the same supported batch throughout emission and generated-key
calculation. Domain projection reads keys from the same completed batch as its
payload. `OutputOrigins`, generated-member origin vectors, `Configured::captured`,
the indexed argument provider and `provenance.algorithm_source_occurrences` are
deleted, along with their generated interfaces and fixtures. Instance, value and
domain lookups carry their support in the same derived algorithm record instead of
separate origin maps. The finite configuration algorithm and its derived key lookup
remain; this does not assert their replacement by a complete provider execution cut.

**Interface-checked:** the DataFusion skill's pinned
`datafusion_expr.logical_plan.builder.md`,
`arrow_array.builder.generic_list_builder.md` and
`arrow_array.builder.struct_builder.md` establish native UNNEST and typed nested
Arrow construction. The targeted capability-gap scan reported no syntax matches;
that is discovery evidence, not architectural or performance certification.

**Corrections during verification:** UNNEST removes the expanded column's table
qualifier at DataFusion 55.1.0; subsequent field extraction now uses the actual
unqualified column. The first broad focused run passed 158 tests and failed 2 because
the new nested contract had not been registered in the logical-type catalog. That
declaration is now intrinsic. A maintainer-issued `cargo clean` interrupted subsequent
builds; their missing-artifact errors occurred before test execution and establish no
behavioral result. The complete generator has subsequently rebuilt successfully.
The next 160-test run passed 159 tests and exposed a test-fixture lifetime error: a
shadowed base session retained shared observations after the result was dropped. The
test now explicitly drops every session owner; it does not add a delay or relax the
zero-reservation assertion.

**Implemented — singleton witnesses and package support:** witness declarations now
distinguish an explicit read scope from positive row membership. A positive witness
with an empty declared primary key names the singleton row, not a read scope. Native
anti joins check that row's existence, native aggregation checks output singleton
cardinality, and support records retain its non-null typed key. Algorithm token joins
also require a matched actual key, including for singleton owners. The package graph
now emits each row together with its support list; its separate positional provenance
vector is deleted. Its finite dependency traversal remains a derived algorithm.

**Corrections during singleton/workflow verification:** DataFusion's logical builder
requires a condition on a non-inner join; singleton membership now uses a native anti
join with an explicit true condition, rather than an empty join condition. A schema
lookup initially omitted the required trait import; the caller now uses the inherent
qualified-field API. The heater workflow test initially expected only phase/species
members; its child control volume also declares the element domain. The test now
checks that declared member and its element/composition support as well. These are
implementation/test corrections, not changes to the declared process-model behavior.

**Tested, baseline 0:**
`just test-package pse-compiler -p pse-catalog --lib --test columnar_arguments -E "'test(colocated_support) | binary(columnar_arguments)'" --no-fail-fast`
passes **2 tests, 0 failed, 158 filtered**, default Nextest / force validation,
2.160 s (`ce107872-b2a2-4e4b-a396-278b34f46aac`). This covers native predicate refusal
of empty/duplicate support, filter/union/sort correlation, independent buffer ownership
and complete release, exact membership and refusal of an unbound source token.
The broader default/force-validation command
`just test-package pse-rules -p pse-compiler -p pse-catalog --lib --test native_construction --test columnar_arguments --no-fail-fast`
passes **169 tests, 0 failed/skipped**, 20.777 s
(`a9fce9c9-9821-42bf-a476-94c1c9cfaf5c`). This adds singleton positive/read-scope
distinction, empty-owner refusal and singleton output cardinality. The subsequent
borrow-only configuration cleanup passes `just check` (workspace/all targets).
`just governance` passes **69 tests, 0 failed/skipped**, 3.885 s
(`28a519a7-9b2b-47ce-87b6-d98376570bb0`), plus three-target generation equivalence,
Ipopt binding regeneration and family checks. `just quality` passes with 0 findings,
including 14 setup tests. `just py-sync` rebuilds the editable native extension and
checks its actual API stubs; `just py-test` passes **86 Python 3.14.7 unit/component
tests, 0 failed**, 24.36 s. `just doctor` reports 0 blocking issues.

**Tested, baseline 0 — engine support qualification:**
`just test-package pse-tests-engine --test native_engineering_workflows --test native_normalization --test unified_sources --test native_change_batches --test terminal_attempts -E "'not (binary(native_engineering_workflows) & not test(material_configuration))'" --profile ci --no-fail-fast`
passes **16 tests, 0 failed, 4 filtered** (reported as skipped by Nextest),
CI / force validation, 699.326 s (`e38576d7-a36e-48ba-b082-f2ffb2ac737c`). The four
excluded tests are the other engineering workflows; the new material-configuration
test verifies phase/species/element members, their normalized domain rows and exact
supporting relation families. The run also covers normalization, source binding,
native change batches and terminal failures. Concurrent compilation was active;
this is functional qualification, not an isolated performance measurement.

Earlier receipts,
including the 395.369-second template pass, precede this support implementation. No
unchanged template rerun is needed merely to exercise the 600-second timeout setting.

**Remaining:** E00–E10/SP00–SP13 remain uncertified. Other source-occurrence/index
mechanisms, signed graph ordinals, coherent math/numerical rows, general native
rule/domain transfer, provider/compiler/store/Python replacement, member-attempt
reconciliation, native reuse/retention and terminal deletion/measurement remain open.
The latest `just clippy` workspace/default/all-target attempt reports **63 compiler
findings**, baseline 0; the no-default-features pass is not reached. The unrelated
missing-build-artifact interruption is not a lint or functional receipt.

The expanded conformance command initially passed 5 tests and failed 7, exposing stale
physical dictionary enum fixtures and incomplete snapshot validator assembly. Enum
fixtures now use Arrow's native cast to decode external dictionary codes into their
declared canonical Utf8 enum before admission. Snapshot fixtures bind the shared native
invariant validator and declare its diagnostic output. No canonical hashing contract
change or legacy enum-storage reader was introduced. **Tested, baseline 0:**
`just test-package pse-tests-conformance --no-fail-fast` passes **12 tests, 0 failed/skipped**,
default Nextest / force validation, 12.948 s (`2230b18f-53b9-4b03-9ade-7f73a704cadb`),
including two 128-case property tests. The earlier two invariant-fixture tests were not
a full conformance run.
`cargo clippy --no-deps -p pse-tests-conformance --all-targets --locked -- -D warnings`
reports **0 findings** for that test crate; the separate 63-finding workspace failure
above remains open.

Concrete remaining association paths include `source_row_ordinal` in
`pse-authoring/src/document/binding/native.rs` and its `source_occurrences` declaration,
the derived generated-origin index in P3 configuration, and P4 predicate node-origin
maps. Evaluate each with its producer/consumer cut; a parsed document position and a
derived graph lookup are not automatically replacement model authorities. The old
`ExprGraph`, `StageDag`, compiler `Driver`, `RulePlan`/`RuleExpr` and store callers remain
separate mandatory E03–E09 replacement/deletion work.

### Planning-task verification — 2026-09-16

**Tested — documentation only, baseline 0:** scoped `.venv/bin/typos` over this plan,
the plan index, Plan 07, `STATUS.md` and the evidence JSON reports **0 findings** after
correcting three spelling-lint findings on the plural of `CHECK`. Targeted Python
checks validate front matter, 31 local links, 14 unique work packages, all 11 schema
findings, 8 additional opportunities, 8 outcomes and 14 acceptance rows with **0 errors**.
`git diff --check` for updated tracked documents reports **0 findings**. The 28
inspected project-source hashes remain unchanged by this planning task.

`just docs` generates the book with **0 errors and 2 existing warnings, baseline 0**:
the unclosed example tag in proposed ADR-0068 and the large search index. Those warnings
remain open and book generation is not a zero-warning receipt. No proposed schema
behavior, runtime performance or architecture completion was certified by these checks.

### Coherent expression values and native durable predicates — 2026-09-16

**Implemented; E03/SP04 qualification remains in progress.** All five normalized
expression families and the inferred/compiled family now store ordered children and
one tagged payload on each node. The separate argument and per-opcode payload
relations, their generated Rust/Python types, callback methods and loader joins are
deleted. Compiler, engineering inspection and mathematical adapters consume the new
values directly. Mathematical graph algorithms retain only their derived calculation
layout; the new rows are the storage authority.

Affine rows store ordered coefficients alongside the sole child list. A native CHECK
requires equal lengths; the algorithm writer refuses coefficients whose original
child correspondence differs before conversion. This avoids storing each child twice.
The operator declaration supplies both algorithm and native opcode/payload/arity
admission. Native reference plans check ordinary children and payload-held node
references against the exact selected expression family. Normalized, inferred and
compiled families declare their different unresolved-reference policies as native
row checks. Cycle, scope and physical shape algorithms remain necessary; foreign-key
admission alone does not establish them.

Mathematical node IDs, quantity-selection positions, indexed/grounded equation node
references, free-index positions, kernel input nodes and derivative orders now use
bounded nonnegative Int64 fields. Generated adapters check conversion to bounded
algorithm indices. Other semantic-path/predicate/root ordinal declarations remain
open and are not certified by this cut. The unused source-occurrence row-position
field is deleted; source/resolved occurrence and match ordinals are signed.

The common tagged-value predicate now checks each arm's presence once instead of
expanding a complete absence matrix for each tag. Conjunctions are balanced. Native
nested CHECK expressions persist through DataFusion's pinned expression codec and
bind against the actual caller's function registry after cold opening. This removes
the custom SQL-unparser workaround and the nested-expression dialect property. The
new descriptor explicitly identifies its codec; there is no old descriptor reader.
Portable local SQL checks continue to use Delta CHECK.

**Interface-checked:** local DataFusion skill references for
`datafusion_expr::utils::conjunction`,
`datafusion_proto::bytes::Serializeable`, native lambda/struct/list expressions and
the pinned `datafusion-proto` implementation; the Delta skill's native declared
write/constraint route. The upstream conjunction helper forms a left-deep tree, so
the declaration generator balances its native AND expressions locally. No general
row interpreter or second expression language is introduced.

**Tested, baseline 0 — interim receipt:**
`just test-package pse-catalog -p pse-compiler -p pse-mathir --test coherent_math --test math_families --test mathir_relations --test relation_roundtrip --no-fail-fast`
passed **17 tests, 0 failed/skipped**, default Nextest with force validation, 3.855 s
(`41437bcc-b779-48a6-856c-e764b5a2ac92`). This covers cold codec reconstruction and
wrong-opcode write refusal without a table-version change, candidate arity/payload/
child closure, normalized alternatives, ordered references, canonicalization and
cycle rejection in the mathematical loader. A subsequent affine-correspondence/
signed-overflow negative test is included in the pending broader run.
`just check` passed workspace/all-target/default compilation. Three-target codegen
and conformance fixture regeneration have been run; full qualification is in progress.

**Corrections found during implementation:** a native SQL lambda was not accepted
by the default parser, motivating removal of duplicated affine child references.
Cold reopening then exceeded the parser's recursion bound for a wide nested value;
using the native expression codec removed that textual round trip without changing
caller limits. The first overlapping-arm test expected rejection at builder finish;
the generated builder correctly rejects the invalid value at push. A missing callback
error propagation operator introduced during signed conversion was corrected in the
generator and regenerated.

**Remaining:** E00–E10 are not certified. Finish coherent-row functional/physical
scan qualification, the other signed schema sites and typed numerical programs/
results. General invariant/domain transfer, the combined provider/publication/
compiler/store/Python replacement, member reconciliation, reuse/retention and the
terminal deletion/resource gates remain mandatory. The focused receipts above do not
close those obligations or the prior workspace Clippy findings.

### Prepared numerical vectors and solver outcomes — 2026-09-16

**Implemented; E03/SP05 qualification is in progress.** The registry generates
`runtime.numerical_programs`, `runtime.jacobian_coordinates`,
`runtime.numerical_evaluations` and `runtime.solver_outcomes`. Prepared programs have
an explicit caller-supplied identity, ordered variable columns and signed dimensions.
Inputs carry an explicitly declared `scenario_id`; evaluations preserve it alongside
the program identity. Residual and sparse Jacobian values are native list vectors.
The `residual_<n>` and `jacobian_<r>_<c>` output names are deleted. Zero dimensions
produce visible empty vectors, including zero-row batches. Coordinates carry program
identity and signed vector/residual/variable positions.

The existing Ipopt consumer reads contiguous Arrow vector buffers, retains their
allocation owners, checks the exact decision-column order and converts coordinate
indices at the C boundary. Its former combined scalar-column cache is deleted.
Generated outcomes replace the handwritten Arrow result/iteration schemas and bind
returned values to the program, scenario and variable-column ordering. Termination
is explicit (`success`, `stopped`, `cancelled`, `evaluation_failure`); diagnostics are
a typed optional struct. Unavailable/nonfinite numeric returns remain absent values;
a successful solver status with unavailable result values is refused. Existing
limited-memory derivatives, FFI, cancellation and foreign-work settlement are retained.
No new solving capability is implemented.

**Interface-checked:** DataFusion skill references and pinned implementations for
`datafusion_functions_nested::make_array::array_array`,
`datafusion_expr::udf::ScalarUDFImpl`, Arrow `ListArray` and native scalar extraction.
Native scalar list conversion at this pin drops child metadata. The narrow
`pse_numerical_vector` binding calls native array packing and retains the declared
child field without a scalar round trip. It also checks actual invocation cancellation
and retains the preparation allocation, including for empty vectors. It therefore
remains observable during optimization. All mathematical expressions and derivatives
remain native expressions; this binding does not interpret mathematical operations.

**Tested, baseline 0 — interim receipts:**

| Command | Actual evidence and limits |
|---|---|
| `just test-package pse-schema -p pse-relations -p pse-mathir -p pse-catalog -p pse-authoring -p pse-templates -p pse-rules -p pse-compiler --no-fail-fast --status-level fail --final-status-level fail` | Coherent-expression cut, default Nextest / force validation: **636 passed, 0 failed/skipped**, 74.586 s; `dae31890-a169-47a1-9f89-a6144143d710`. Precedes the numerical declarations. |
| `just test-package pse-numerics -p pse-backend-native --no-fail-fast --success-output immediate` | Default / force validation: **16 passed, 0 failed/skipped**, 3.489 s; `d7bb6aae-fb15-463c-b4b1-f062ee384d45`. Includes analytic derivatives, native projection, zero dimensions, scenario/schema refusal and buffer ownership. |
| `just native-solver-test --no-fail-fast --status-level fail --final-status-level fail` | Linked Ipopt in the pinned solver container / force validation: **12 passed, 0 failed/skipped**, 24.427 s; `36a09953-755f-4ffd-b806-12a28fd2951c`. Precedes the final vector-binding cancellation/ownership refinement. |
| `just test-package pse-catalog --test coherent_math --no-fail-fast --success-output immediate` | Default / force validation: **3 passed, 0 failed/skipped**, 5.154 s; `e68fe1df-f364-4780-85f5-d2547456a04d`. Includes declared Delta projection measurements below. |
| `just py-sync`, `just py-test`, `just quality` | Coherent-expression editable extension rebuilt; Python 3.14.7 unit/component: **86 passed**, 24.66 s; quality passes, including **14 setup tests**. Precedes the numerical declarations. |
| `cargo clippy --no-deps -p pse-schema -p pse-relations -p pse-catalog -p pse-mathir -p pse-numerics -p pse-backend-native --all-targets --locked -- -D warnings` | Selected packages/default/all targets: **0 findings**. No recipe exposes package-scoped Clippy. Full workspace compiler findings remain open. |

**Measured — one diagnostic run, concurrent qualification, no performance acceptance:**
`repeated_callback_vectors_keep_contiguous_values_and_release_their_reservations`
executes 256 one-row evaluations in **22,807 microseconds**, with **32,111 bytes**
reserved for preparation and **32,415 bytes** maximum observed at returned results.
Every released result restores the preparation reservation; dropping the program
restores zero. These are explicit reservation observations, not malloc counts, an
allocation-peak measurement or an isolated timing benchmark.

`declared_delta_projections_measure_the_one_coherent_layout` writes 512 integer nodes
once, then opens the exact committed version in fresh contexts with a 128 MiB native
pool. Its single-run measurements are:

| Projection | Planning microseconds | Execution microseconds | Parquet `bytes_scanned` | Retained result buffers |
|---|---:|---:|---:|---:|
| opcode | 2,687 | 884 | 48 | 4,612 bytes |
| integer payload value | 12,041 | 2,954 | 3,521 | 4,096 bytes |
| whole node | 9,680 | 3,296 | 6,428 | 462,824 bytes |

The scan-byte metric does not include every filesystem/footer/log read. The native
operator pool reported zero reservations for these scans; that does **not** establish
zero allocation or a process-memory ceiling. Returned buffers are measured separately.
Nested file-column pruning and whole-process/retained-memory qualification remain open.
The initial scan fixture incorrectly opened table-creation version zero; it now reads
the actual completed data-write version, without changing the published selection.

**Engineering qualification remains open:** the 18-case coherent-expression run
completed with **8 passed, 9 failed and 1 timed out**, CI / force validation, 1892.570 s.
Eight conservation cases first failed before compilation because the source fixture
omitted required package documentation; after that correction they exposed colliding
symbol/contribution names. Those source declarations now have distinct names, and
`an_absent_explicit_balance_binding_is_not_inferred_from_a_template_name` passes
(**1 passed, 7 filtered**, 213.609 s). The remaining conservation cases are being run.
Kernel realization exposed missing support on intermediate conversion children and
four graph-support lookups using `kernel_binding_id` instead of the declared
`binding_id`; both are corrected and the actual descriptor workflow is being rerun.
`selected_methods_bind_actual_coefficients_and_preserve_seed_on_reopen` exceeded its
360-second allowance. Its timeout is unchanged and it is not certified by the smaller
numerical/solver receipts.

**Remaining:** complete selected-program dimension/coordinate admission at durable
publication, full linked/current-source qualification, representative allocation and
physical-layout measurements, remaining signed schemas and alternatives, and all
E04–E10 provider/rule/compiler/store/Python/change/retention replacement work. The
new numerical descriptors do not close the common native domain-transfer obligation.
There is no old numerical-result reader or compatibility schema.


### Signed local values and complete alternatives — 2026-09-16

**Implemented; E01–E03 and terminal qualification remain open.**

- Remaining ordinary node/path/predicate IDs, positions, ranks, depths and counts
  now use bounded Int64 declarations. Existing narrow bounds remain enforced. The
  generated Rust/Python/Arrow contracts, rule count projections and compiler/template
  consumers use those signed values. Full-width unsigned configuration values and
  explicit unsigned/float-bit rule literals retain their actual value semantics until
  the rule representation is replaced; they are not ordinals.
- Derived graph `NodeId` values use Int64, checked index conversion and negative-ID
  refusal. Graph algorithms remain transient views of coherent Arrow relations.
  Existing graph/reference/key byte framing is not replaced by a second identity.
- Indexed and scalar equation relations carry one tagged `constraint`: comparisons
  and definitions have one target, while ranges require both bounds. Generated
  readers/writers replace the three independent sense/lower/upper columns. P8 law
  and connection producers emit one equality target; malformed derived equation
  records are also refused before graph remapping.
- A method realization always carries its actual output symbol. Its tagged producer
  describes either the template instance or the kernel binding/output ordinal. The
  contradictory predecessor invariant, which rejected a kernel-produced symbol,
  and its fixture are deleted. P7/P9 and inspection tests consume the generated value.
- Method dependency coordinates carry a common domain kind and one tagged source:
  source axis, fixed member or bound domain. Generated local admission replaces
  P6's separate optional-payload shape predicate; actual axis/member/domain checks
  still execute through its native plans. Stock source declarations use this shape.
- Kernel parameters carry one tagged binding: a symbol or a literal with its unit.
  Generated Arrow adapters replace the three independent optional fields. The
  bounded algorithm input is validated and round-tripped through the new value;
  no reader accepts the predecessor stored form.
- P8 projection lookup now uses actual selected source groups as well as newly
  constructed groups. Its completed non-rewritten relations retain existing Arrow
  rows when additions are produced. New symbol ordinals account for existing symbols.
  Unchanged facts retain their derivations; remapped expression/root relations are
  rebuilt by their existing typed algorithms, without retaining predecessor graphs.

**Corrections found during validation:** two conformance generators still emitted
unsigned ordinal cells; both now generate signed values. Equation generation first
exposed an incorrect error conversion and redundant generated conversions, then the
workflow exposed two old equality producers storing the same bound twice. Those
producers now use the declared single-target representation. Failure assertions print
bounded typed finding messages instead of dumping the complete retained session/store.

#### Signed value verification

**Tested, baseline 0 — intermediate source boundaries:**

| Command | Actual evidence and limits |
|---|---|
| `just test-package pse-schema -p pse-relations -p pse-mathir -p pse-catalog -p pse-authoring -p pse-templates -p pse-rules -p pse-compiler -p pse-numerics -p pse-backend-native --no-fail-fast --status-level fail --final-status-level fail` | Default Nextest / force validation: **655 passed, 0 failed/skipped**, 76.292 s; `1555880c-167b-4a8f-a959-7a20b792cae7`. Signed declarations, method producers and equation tags; precedes the subsequent coordinate/kernel-parameter and P8 retention corrections. |
| `just test-package pse-relations --test tagged_values -p pse-mathir --test relation_roundtrip -p pse-compiler --lib --no-fail-fast --status-level fail --final-status-level fail` | Default / force validation: **85 passed, 0 failed/skipped**, 11.099 s; `87ca4913-5044-4028-9db1-fdad58ad76ca`. Includes dependency coordinate alternatives and signed graph/bound admission; precedes the kernel-parameter cut. |
| `just test-package pse-tests-conformance --no-fail-fast --status-level fail --final-status-level fail` | Default / force validation: **12 passed, 0 failed/skipped**, 13.462 s after correcting the signed fixture generators. Earlier run: 11 passed and 1 failed. |
| `cargo clippy --no-deps -p pse-schema -p pse-relations -p pse-mathir -p pse-catalog -p pse-numerics -p pse-backend-native --all-targets --locked -- -D warnings` | Scoped default/all-target check passed before the coordinate/kernel-parameter changes. No package-scoped recipe exists. |

**Engineering is not qualified.** The earlier remaining conservation run finished
with **1 passed, 2 failed, 4 timed out and 1 filtered**, CI / force validation,
2130.666 s (`5707efec-f85f-4c58-bf99-7ab36c966b9e`). The kernel descriptor run timed
out at its existing 360-second limit (`6c240590-681d-4840-87cb-5227a8df0317`).
The fixed-phase regression after group retention exposed the equality producer and,
after its correction, one publication invariant violation (234.590 s). No timeout
was increased and these failures are not covered by the smaller passing suites.

**Remaining, concrete:** other method selection/provision and source-owner/domain
binding alternatives; selected-program dimension/order/coordinate publication checks;
complete native invariant and domain-transfer replacement; native provider/compiler/
store/Python caller replacement; member-attempt reconciliation; dependencies/CDF/reuse/
retention; terminal current-function/deletion/measurement gates. `RulePlan`/`RuleExpr`,
`Catalog`/`Snapshot`, `Driver`/stage/memo, manifests/sidecars and old Python handles still
exist. Their removal remains mandatory; no complete pivot is claimed.

### Exact numerical selections and source ownership — 2026-09-16

**Implemented; whole-pivot qualification remains open.** Native publication admission
now joins numerical evaluations and solver outcomes to the exact selected numerical
program. It verifies dimensions, the ordered variable-column vector, and the objective /
constraint split. Sparse coordinates require unique ordinal keys, bounded residual and
variable positions, unique coordinate pairs and a count equal to the declared Jacobian
dimension. Together these establish complete ordinal coverage, including zero dimensions.
Ambient unselected program/coordinate tables cannot satisfy the obligations; conflicting
selected revisions refuse. Native DataFusion joins, aggregates, list length and equality
implement these checks, without a vector interpreter or another durable authority.

**Tested, baseline 0:** `just test-package pse-catalog --lib delta::numerical --test
unified_delta_contracts numerical_publication --no-fail-fast` passed **6 tests, 0
failures, 152 filtered**, default Nextest / force validation, **6.733 s**. The real
Delta case writes a locally valid result with a forged program dimension, refuses its
publication, and verifies that the visible control version remains **1**. This qualifies
publication admission, not unrestricted standalone Delta writes or complete caller routing.
The preceding five-test admission run passed in 25.101 s; the final test fixture uses
the shared immutable registry instead of repeatedly assembling it.

**Implemented:** normalized expression sources now contain one tagged `owner` with a
complete template or instance reference. P3 source generation / native shape joins,
P4 owner selection, P5 path matching and P7 display-root selection use that value.
The independent owner-kind / optional-ID columns and their RulePlan shape invariant
are deleted; generated local alternative checks and nested reference admission replace
them. Source-owner tests and rebuilt Python interfaces are being qualified with this cut.

**Corrected and tested:** a fixed phase restricts the input slice of a species/element
law without adding a free phase axis to its result. The law invariant now admits that
case while retaining the separate contribution-shape restriction. All **8** focused
`pse-rules` invariant tests passed with force validation in **11.086 s**. The actual
fixed-phase source-to-P8 workflow then passed in **236.106 s**, CI / force validation,
using its unchanged timeout (`964546f4-eaa2-4397-9f7f-4db9b3e6e8c8`).

Four conservation tests previously bundled two independent complete source executions.
They now expose each source case as a separate test with unchanged inputs/assertions and
the existing per-test timeout. The missing-molecular-weight test also redundantly ran
P3–P7 before repeating P3–P8; its single P8 execution now checks the recorded P8 failure
identity to establish successful prior realization. Its earlier two-run form timed out
at **360.082 s**; the corrected form and remaining workflows are not yet certified.
No timeout was increased.

**Governance:** Appendix B coverage now asserts that each removed mathematical side
table is absent and its coherent replacement field exists. The explicit test ledger is
not a runtime alias or a deferred-contract exemption; proposed ADR-0069 records this
obligation. After marking the four new generated numerical modules for inclusion,
`just governance` passed **72 tests, 0 failed/skipped**, **4.047 s**, followed by clean
regeneration and dependency-family checks. This receipt precedes the source-owner cut.
`just check` also passed before that cut. Workspace Clippy remains red with **63 compiler
findings**, baseline zero; its no-default-features pass was not reached.

**Remaining:** other method/domain-binding alternatives; common native invariant/domain
transfer; full provider/compiler/store/Python caller replacement and predecessor deletion;
member-attempt reconciliation; native dependencies/CDF/reuse/retention; and final current-
function, extension, cost, Q01–Q14 and G1–G7 evidence. No complete pivot is claimed.

### Complete method values and retained publication members — 2026-09-16

**Implemented; the complete architecture pivot remains open.**

- `authored.template_domain_bindings.source` carries exactly one complete domain,
  parameter, or payload-free material-domain selection. P3 consumes the generated
  selected arm. Its optional source columns and duplicate shape invariant are deleted.
- `reference.method_specs.realization` couples an equation-template or kernel choice
  with its required identity. `reference.method_provisions.output` couples a template
  symbol or kernel output position with its required payload. Native state/provision
  joins, P3/P7/P8/P9 consumers, stock source declarations and fixtures use those fields;
  the two former RulePlan shape invariants are deleted. The separate flat inferred
  method-resolution representation still requires replacement with native rule binding.
- A solver success claim requires actual objective/vector/dual values, a successful
  Ipopt status and no failure diagnostic. The native declared CHECK owns this local
  obligation; failure rows can retain unavailable values as nulls.
- Publication composition accepts written members and unchanged exact selectors in
  the same native candidate plan. It also supports a publication consisting entirely
  of retained members. Complete-vector admission still checks the exact selected
  versions/slices and references before the control commit. This is not member-attempt
  recovery: interrupted writes and exact-input retry reconciliation remain open.
- The physical declared-output boundary permits an optimizer-proven nonnull field
  where the declared outer field is nullable. A native explicit-field identity TRY_CAST
  restores that weaker nullable promise after checking exact names, types and metadata.
  Nested field changes and stronger nullable claims still refuse. No volatile marker,
  disabled optimizer or metadata-only value reinterpretation was added.

**Interface-checked:** the DataFusion skill and pinned `WithMetadataFunc` establish
that native `with_metadata` merges outer metadata; it cannot remove keys or recover a
struct child's metadata. `TryCastExpr::new_with_target_field` exposes exact target
metadata and a nullable return field. Equal source/target types take Arrow's identity
cast path. The Delta skill's `DeltaTableBuilder::with_version`, native create/write
builders and exact control binding remain the persistence basis. The retained-member
capability-gap scans found no additional syntactic findings. This evidence does not
complete the common domain-transfer implementation or justify retaining its wrappers.

#### Current receipts and source boundaries

**Tested, baseline zero; no terminal acceptance:**

| Command / scope | Receipt and limitation |
|---|---|
| Eleven-package Rust/conformance run recorded below | After the method declaration/provision and retained-publication cuts: **685 passed, 0 failed/skipped**, default Nextest / force validation, **88.250 s**, run `c6643edb-88ed-4acc-91a6-46d341bb4668`. Precedes the shared native-check cut recorded next. |
| `just test-package pse-catalog --lib delta::numerical --test unified_delta_contracts numerical_publication --no-fail-fast` | **7 passed**, default / force validation, **8.889 s**, 152 filtered; includes the success-value CHECK and real durable invalid-selection refusal. |
| `just native-solver-test --no-fail-fast --status-level fail --final-status-level fail` | Linked native solver, default / force validation: **12 passed, 0 failed/skipped**, **25.276 s**, run `50efdfc1-b4d0-43b4-bf57-943f0aad5950`; before the method declaration/provision cut. |
| `just governance` | After the method declaration/provision cut: **76 passed, 0 failed/skipped**, **5.715 s**; regeneration and dependency-family checks clean. Precedes the shared native-check cut. |
| `just test-package pse-relations -p pse-schema -p pse-rules -p pse-compiler -p pse-tests-conformance --lib --test tagged_values --test invariant_fixtures --no-fail-fast --status-level fail --final-status-level fail` | New method producer/provision contracts: **75 passed, 0 failed/skipped**, default / force validation, **19.318 s**, run `5449ff53-36db-49b1-974b-bc736dcf174f`. |
| `just test-package pse-catalog --test unified_delta_contracts publication --no-fail-fast --status-level fail --final-status-level fail` | Native publication composition/control: **11 passed, 0 failed**, **13.768 s**, 12 filtered; run `29dd757c-b3d3-4896-8b07-ca71727f53ca`. One actual write plus an unchanged member, retained-only publication, unavailable-version refusal and unchanged visible root are exercised. |
| `just check` | Workspace/all-target/default compilation passed after the method cut. |
| Scoped Clippy for schema/relations/catalog | **0 findings**, all targets, `--locked -- -D warnings`; no package-scoped lint recipe exists. The last full workspace Clippy receipt remains **63 compiler findings**, baseline zero. |

The eleven-package command is `just test-package pse-schema -p pse-relations -p
pse-mathir -p pse-catalog -p pse-authoring -p pse-templates -p pse-rules -p pse-compiler
-p pse-numerics -p pse-backend-native -p pse-tests-conformance --no-fail-fast
--status-level fail --final-status-level fail`. The completed rerun, method workflow failures and Python/governance receipts
are recorded in the shared native-check continuation below.

The twelve-case conservation run completed with **9 passed, 1 failed, 2 timed out**.
The following continuation records its source boundary, the independent correction to
the broadcast oracle, and the remaining unchanged-timeout failures.

**Corrections:** the method rule projection initially omitted a required method key;
registry assembly refused it before generation. A Python-test command overlapped the
schema bootstrap and could not compile stale generated method fields; that is not a
Python runtime result. The first mixed-publication test exposed the outer-nullability
refinement described above; its corrected native path passed the full publication set.

**Still mandatory:** native RulePlan/RuleExpr replacement and common domain transfer;
complete provider/compiler/store/Python caller replacement with predecessor deletion;
exact-input member-attempt reconciliation; native dependencies/CDF/reuse/retention;
current-function performance and failure closure; full quality/feature qualification;
and final Q01–Q14/G1–G7 evidence. These changes do not certify E00–E10 or SP00–SP13.

### Shared native row checks and qualified diagnostic keys — 2026-09-16

**Implemented; E04 and the complete pivot remain open.**

Named SQL checks now bind through `pse-catalog::contract::row_checks` for candidate
findings, scoped provider requirements, publication admission and durable Delta CHECK.
The former Delta-only binder is deleted. Native checks require neither an `InvariantSpec`
nor a `RulePlan` declaration. Each check has a stable relation-version/name identity;
that identity grants no validity. Missing selected sources or unknown required checks
refuse, including for scalar queries without a table scan. False and SQL NULL both
produce error findings with the actual typed violating keys. Affected-input selection
uses the relation on which the row predicate is declared.

Nine former RulePlan checks now reside on their relation's native SQL declarations:
positive unit scales and nominal magnitudes, ordered continuous bounds, method-resolution
alternatives, three contribution/law subject-coordinate contracts, participation decisions
and nonnegative tear costs. The old semantic-invariant module and its obsolete fixture
branches are deleted; generators regenerate inspection/contracts and the remaining
invariant fixtures. Independent coordinate/physical/alternative cases exercise the native
predicates. Other relational rules and the flat method-resolution schema still remain.

**Correction found by an independent key assertion:** diagnostic keys had used native
SQL-qualified column names. They now frame the declared key names with their actual
values, so changing the scan's catalog/table alias does not change row identity.
The typed diagnostic assertion checks both false and unknown rows against independently
constructed keys. This changes no persisted legacy reader or compatibility route.

**Interface-checked:** DataFusion 55.1.0 `SessionState::create_logical_expr`,
`ExprSchemable::get_type`, native volatility and SQL Boolean predicates, using the
DataFusion skill's `datafusion.execution.session_state.md` and existing pinned binder.
The capability-gap scan reports only the parse-only test's default `SessionContext`;
that test executes no data operators. Delta still binds the actual check functions and
persists the same native expressions through the already qualified declared-contract path.

#### Current verification and explicit failures

**Tested, baseline zero; no terminal architectural acceptance:**

- `just test-package pse-rules --test invariant_execution --no-fail-fast --status-level fail`:
  **12 passed, 0 failed/skipped**, default Nextest / force validation, **10.738 s**,
  run `794e52ba-5cba-45b0-bdf3-001224a4fb00`. Includes typed false/unknown diagnostics,
  no-scan policy refusal, absent-source refusal and independent domain predicate cases.
- `cargo clippy -p pse-schema -p pse-catalog -p pse-rules --all-targets --locked -- -D warnings`:
  **0 findings**. No package-scoped Clippy recipe exists. This does not supersede the
  full workspace's **63 compiler findings**, baseline zero.
- `just check`: workspace/all-target/default compilation passed, **16.05 s**.
  The upstream `proc-macro-error2` future-incompatibility notice remains.
- `just codegen-bootstrap`, `just conformance-fixtures`: regenerated from declarations.
- The eleven-package command above passed **689 tests, 0 failures/skips**, default
  Nextest/force validation, **88.312 s**, run `308937a9-e409-4bf1-96af-cc48e01cae90`.
- `just governance`: **76 passed, 0 failed/skipped**, default/force validation,
  **4.635 s**, run `930b6e0d-8a6e-4e0e-b6ba-336088c01c33`; generation and families clean.
- `just py-sync`, `just py-test`: editable extension/stubs rebuilt; Python 3.14.7
  unit/component **86 passed, 0 failures**, **24.05 s**.
- `just quality`: **0 findings**, including **14 setup tests passed**.

These receipts precede the domain-reference and complete source-syntax cuts below.

**Measured/Tested failures, unchanged CI timeout of 360 seconds:** the earlier
`just test-package pse-tests-engine --test conservation_expansion --profile ci
--no-fail-fast --status-level fail` run completed **12 cases: 9 passed, 1 failed,
2 timed out, 0 skipped**, **3081.673 s**, run
`1ee1a940-27c8-4627-8be5-feeac085d63a`. The element-molar cases timed out. The scalar
energy assertion counted three global broadcasts where it expected only two source
broadcasts. Inspection established that the third is the equality's correctly shaped
zero bound. The corrected oracle separately checks two broadcasts reachable from the
body and one from the typed zero bound. Its fresh CI/force-validation run passed
**1 test, 0 failures**, **258.665 s**, 11 filtered, run
`625f33dc-7565-4937-9cbf-4f7cb4f10565`, using
`just test-package pse-tests-engine --test conservation_expansion
scalar_energy_broadcast_is_explicit --profile ci --no-fail-fast --status-level fail`.
The existing timeout was unchanged.

The trace-only element-molar diagnostic used the same CI timeout and failed again at
**360.038 s**, run `f0093a0d-a466-4b61-b572-b74a00c79dc7`. Its actual stage trace shows
P3–P8 publishing in approximately **239 s**, followed by cold reopening that re-executes
P3/P4/P5 before timeout. This demonstrates repeated predecessor admission work in this
case; it does not certify every timeout's cause or establish a new acceptable cost.
No timeout was raised, and no admission check was disabled.

The method workflow command `just test-package pse-tests-engine --test method_realization
--test demand_seed_closure --profile ci --no-fail-fast --status-level fail` completed
**5 cases: 2 passed, 3 timed out, 0 skipped**, **1586.292 s**, run
`3bd61513-5610-44dd-8ac1-58fa0f67f868`. Timeouts were the transitive demand-support
cold-reopen case, kernel descriptor/natural-unit case and selected-method cold-reopen
case. These runs use the preceding method-tagged source build, before shared native
row-check enforcement. They are unresolved workflow gates, not a passing baseline.

**Still mandatory:** common domain transfer; remaining alternatives and native
relational-rule binding; complete provider/compiler/store/Python replacement and
predecessor deletion; exact-input attempt reconciliation; native dependencies/CDF/
reuse/retention; full quality/feature/current-function/cost qualification; Q01–Q14 and
G1–G7. This continuation does not close E00–E10 or SP00–SP13.

### Complete source syntax and domain alternatives — 2026-09-16

**Implemented; scoped receipts below precede the interrupted E04 edit.** Normalized expression-index bindings now
carry one actual/template domain value. Predicate and equation nodes carry complete
tagged values instead of a discriminator and independent optional payload columns.
Comparisons carry ordered operand alternatives (expression or exact enum/member key)
in a native list whose declared minimum and maximum lengths are both two. Conditional
equations require their guard and both branch ordinals in the selected payload.

P3 lowering and demand traversal, P4 predicate evaluation, P6 index assignments and
P7 binding/equation realization consume generated values directly. Shared dependency
walkers borrow those rows for the existing finite algorithms; no second stored graph
or legacy reader was introduced. The old predicate-row initializer, optional-domain
column converter, unused guard-column converter and flat-domain RulePlan invariant
and fixture are deleted. Domain-name predicates bind through the shared native SQL
admission path; selected-arm completeness and operand cardinality live on fields.

**Corrections discovered during qualification:**

- The domain-only engine run exposed an optimizer ambiguity between a qualified
  source ID and an unqualified derived column. The product-source projection now
  has a distinct name. The failed run was stopped after **6 failed, 0 passed**, six
  unrun, **279.652 s**, run `99f3116b-b974-4de6-8310-30c2334f73ec`. Test diagnostics now
  use the compiler error's bounded display rather than printing its entire plan.
- A fixed-size-list experiment failed a real Delta write when an inactive predicate
  arm's hidden null children crossed Arrow's fixed-to-variable-list cast. The plan
  now uses Delta-native variable lists with an exact length-two declaration. The
  failed probe was **16 passed, 1 failed**, **10.703 s** in the catalog/rules command;
  empty table declaration version 0 had committed, but the data write failed. This
  is not successful persistence qualification. No upstream panic was accepted as
  a fallback behavior.

**Interface-checked:** the DataFusion skill's pinned Arrow `FixedSizeListArray`
constructors/layout and native nested-field functions, and the existing Delta skill
and `DeclaredCheck`/`DeltaWrite` cold-contract route. The source capability scan found
only the intentionally bounded test collection of two rows. This establishes no
performance advantage for either physical list representation.

#### Source-syntax verification recorded at the 2026-09-17 update

**Tested, baseline zero, before the interrupted native invariant rewrite.** These
receipts establish the preceding source-syntax cut only. They are not a current-tree
build or terminal architecture certification, and are not instructions to repeat the
integration campaigns before E00–E09 implementation/deletions are complete.

| Command / mode | Actual receipt and boundary |
|---|---|
| `just test-package pse-schema -p pse-relations -p pse-mathir -p pse-catalog -p pse-authoring -p pse-templates -p pse-rules -p pse-compiler -p pse-numerics -p pse-backend-native -p pse-tests-conformance --no-fail-fast --status-level fail --final-status-level fail` | Default Nextest / force validation: **692 passed, 0 failed/skipped**, **96.253 s**; run `542ed1f4-bbce-4b35-813c-5687f06321c6`. Build **44.61 s** separately. Includes final variable-list contracts and cold-read fixture correction. |
| `just test-package pse-catalog --test declared_row_checks source_syntax --no-fail-fast --status-level fail` | Default / force validation: **1 passed, 0 failed**, 3 filtered, **1.175 s**; run `d04fc42f-4cf6-4396-b74b-eb4e64da0471`. Cold declared predicate persistence and invalid append refusal. |
| `just test-package pse-tests-engine --test native_normalization --test finite_semantic_inference -E "'test(conditional_equations_keep_declared_senses_branches_and_source_key) \| test(separate_phase_species_axes_reject_only_actual_forbidden_pairs)'" --profile ci --no-fail-fast --status-level fail` | CI / force validation, unchanged 360-second limit: **2 passed, 0 failed**, 10 filtered, **253.869 s**; run `2804a7df-60a8-4d1c-8166-df555fe13aab`. Build **1m07s** separately. These two cases qualify the final syntax representation, not all engine workflows. |
| `just governance` | Default / force validation: **77 passed, 0 failed/skipped**, **4.658 s**; run `17037dcd-ed2d-4e27-9107-7ab43d0aa7e7`. Generation and dependency families also passed before the new invariant declaration edits. |
| `just py-sync`, `just py-test` | Editable extension/stubs built for the source-syntax cut; Python 3.14.7 unit/component: **86 passed, 0 failed**, **24.57 s**. That binary is not current with the later invariant edit. |
| `cargo clippy -p pse-schema -p pse-relations -p pse-catalog -p pse-rules --all-targets --locked -- -D warnings` | Scoped check: **0 lint findings**. No package-scoped recipe exists. The last full workspace run still reported **63 compiler findings**, baseline zero; no new full-workspace result is implied. |
| `just check` | Workspace/all targets/default compiled in **46.58 s** before the invariant edit. Current incomplete source has not been checked. |
| `just doctest` | Workspace excluding `pse-py`, locked / force validation: **31 passed, 0 failed/ignored** across 29 reported crate results; preceding source only. |
| `just quality` | **0 findings**, including **14 setup tests passed**. |
| `just docs` | Book built; emitted the large-search-index warning (**14,174,314 bytes**). This was not a warning-free receipt and predates this plan update. |

The domain-only intermediate check passed **13 tests**, default / force validation,
**10.899 s**, run `13edeadc-aab6-4d63-80a0-702fa850ddbb`. The final receipts above
supersede that intermediate scope where applicable. A second cold-test correction
reconciled SQL-projected schema-level relation annotations: the fixture first checks
exact recursive fields against the cold declaration, then admits using that declared
schema. No value/field validation was relaxed.

**Interrupted broader engine run:**
`just test-package pse-tests-engine --test native_normalization --test finite_semantic_inference --profile ci --no-fail-fast --status-level fail`
ran under CI / force validation, unchanged 360-second timeout, run
`c300e2de-a611-4eca-9cc1-717bfc172286`. It started before the final list/cold-fixture
corrections and overlapped source editing, so it is not a uniform final-source receipt.
The maintainer directed integration runs to stop. Final runner accounting was
**9/12 run: 6 passed, 2 failed, 1 timed out; 3 unrun**, **1517.106 s**. The two failures
must be distinguished:

- `scalar_child_guards_and_nested_selector_difference_use_actual_instance_keys`:
  assertion failure at **152.368 s**. The fixture still contains an unsigned depth
  expectation; that is a lead for the final target-fixture review, not a demonstrated
  root cause or a resolved functional gate.
- `relative_parameter_selector_tracks_actual_target_and_refuses_missing_instance`:
  timeout at **360.090 s**. It bundles three fresh source/commit/inference workflows.
- `conditional_equations_keep_declared_senses_branches_and_source_key`: **SIGINT**
  at **65.574 s** when the run was stopped, counted by the runner as the second
  failure. Its separate final-syntax run above passed. This interruption is not an
  additional observed model-processing defect.

**Measured runtime, not a performance acceptance:** the two separate final-syntax
cases took **101.965 s** for one conditional equation through P3 and **151.904 s**
for four phase/species candidate pairs through P5. These include fixture/runtime/
registry initialization, source commit/validation, processing and assertions; neither
performs a numerical solve. The earlier element trace measured P3's body at about
**48.700 s** and its complete stage at **70.032 s**, then observed producer stages
being re-executed on cold open. Those observations do not yet separate native planning,
query execution, materialization and application admission costs. They motivate the
remaining architecture cut and final E10 profiling, not another immediate suite run.

Logs inspected for these receipts: `/tmp/pse-plan08-syntax-packages.log`,
`/tmp/pse-plan08-syntax-current-engine.log`, `/tmp/pse-plan08-syntax-cold.log`,
`/tmp/pse-plan08-syntax-governance.log`, `/tmp/pse-plan08-syntax-python.log`,
`/tmp/pse-plan08-syntax-clippy.log`, `/tmp/pse-plan08-syntax-final-check.log`,
`/tmp/pse-plan08-syntax-doctest.log`, `/tmp/pse-plan08-syntax-quality.log`,
`/tmp/pse-plan08-syntax-docs.log` and `/tmp/pse-plan08-syntax-engine.log`.
These are local ephemeral evidence, not dependencies needed to implement the plan;
the command/count/source boundaries are recorded here for a durable handoff.

**Remaining:** flat inferred method-resolution and other remaining alternatives;
native RulePlan/RuleExpr replacement and common domain transfer; the combined
provider/compiler/store/Python caller replacement and predecessor deletion;
exact-input member-attempt reconciliation; native dependencies/CDF/reuse/retention;
full quality/feature/current-function/cost qualification and terminal Q01–Q14/G1–G7.
This is an implementation checkpoint, not E00–E10 or SP00–SP13 completion.

## Open items

The authoritative execution list is the current E00–E10 section. These are remaining
obligations within the authorized hard pivot, not reasons to preserve legacy objects.

| Item | Current remaining boundary |
|---|---|
| Latest native compile/static closure | Fix `CacheStore` type complexity and by-value maintenance retention call; reconcile remaining workspace lint/feature findings. Earlier full checks predate latest edits — E05/E08 |
| Full route/field/deletion audit | Finish defaults/derived values/function transfer/support/selected-input coverage; audit fixture preparation, orphan declarations, generated remnants and exports — E00–E05/E07/E08 |
| Resource and ownership accounting | Audit dependency-receipt decoding/growth, effects, cancellation/settlement, stream/producer/runtime lifetimes and metadata observations — E03/E05/E08/E09 |
| Durable recovery/history/reclamation | Finish interrupted/empty-attempt and checkpoint/pruned-history fixture coverage; compile latest unrun lifecycle expansion; qualify actual Delta effects only in E10 — E06/E09 |
| Generation, environment and decisions | Refresh stale environment/extension/stubs, generator equality, guidance and governance; ADR-0068/0069 remain proposed — E00/E08/E10 |
| Final existing-function qualification | Native compiler/numerical/solver/publication/CDF/reuse/cold Rust/Python journeys, ordinary extension proof and fresh zero-baseline quality/feature/docs checks — E10 |
| Performance/resource evidence | Small/scaled nested/coherent data, planning/execution, Delta IO/replay/file growth, callback allocations and peak/retained memory; no performance claim yet — E10 |
| Independent architecture acceptance | Fresh Q01–Q14 evidence and G1–G7 decisions; no terminal gate or SP package certified closed — E10 |

The invariant declaration/caller cut and predecessor driver/rule/store removal are
implemented; they are not open tasks to restart. Deep collection/external-reader and
numerical meanings remain mandatory qualification obligations. No metadata string,
unknown upstream behavior or successful compilation substitutes for runtime evidence.
Unsupported cold reuse or remote destructive coordination must refuse conservatively.
No compatibility path or new simulator functionality is introduced to close this plan.

## Outcome (partial; updated during implementation)

### What was built

**Implemented, partially Tested:** exact recursive Arrow fields and durable Delta
contracts; signed and complete tagged values; native checks/invariants and 130 SQL
inference declarations; typed keys/references/spans/support/diagnostics; coherent math
and generated numerical contracts/consumers. Native provider/session preparation owns
actual dependencies, requirements, effects and resources. Source edits and compiler
algorithms compose as native plans; publication and Rust/Python access use exact
control/member selections and owned Arrow streams.

**Implemented, not integration-qualified:** artifact completeness profiles, member/root
attempt reconciliation, typed native dependency receipts, conservative exact reuse,
bounded CDF, local reader/writer/maintenance leases, native retention queries and Delta
optimization/checkpoint/vacuum/log cleanup/reclamation. The latest continuation adds
native insert defaults, three direct generated Arrow conversions, expanded support
units, CDF/retention projection units and common preparation for seven fixture surfaces.

**Deleted:** predecessor compiler driver/stage/memo/producer authority, closed rule
algebra/reflection, authoring change envelopes, catalog store/manifest/ref/control/
encoding/membership/sidecars, old Python store/snapshot handles, obsolete schemas and
predecessor-only fixtures. Final absence/caller audit remains open; these mechanisms
must not be restored for compatibility or to make a test pass.

**Remaining:** the specific cache lint and maintenance fixture API fix; complete
compile/static/feature and caller/declaration/deletion audits; residual resource,
recovery/history and ownership coverage; generated/stub/environment/docs closure.
Then E10 must exercise existing compiler/numerical/Delta/Python journeys, ordinary
extensions and measured costs, and independently resolve Q01–Q14/G1–G7. No integration
campaign has run in this continuation. Implementation is paused for this plan update;
all remaining actions and the restart order are recorded above.

### A mistake made and corrected

Planning corrected two insufficient recommendations before implementation: native
struct compatibility permits extra/missing fields and therefore cannot establish an
exact contract; copying execution extension metadata onto changed Delta storage can
create an invalid Arrow extension. The implemented boundary now addresses both.
Implementation also corrected a fixture that asserted false non-nullability and kept
schema metadata in its own descriptor after identifying that flattening it into field
metadata would shadow same-named annotations. Native float casts required an inverse
value check because their overflow setting does not prohibit rounding. The native
registry cut also removed a generator assumption that bare binary and dictionary
storage implied specific PSE domains. Native Arrow equality omits dictionary ordering,
so the exact boundary checks that declaration explicitly.

Native predicate probes corrected an assumed binary `octet_length` overload, a
lambda SQL round-trip that lost struct-field scope, and a cold-write fixture without
Delta's own extension planner. Stored collection SQL now round-trips through native
unparser hooks and executes under the actual caller state. Initial CHECK installation
must precede member data writes at this Delta pin; first-write configuration alone
does not validate its new constraints.

### Deviations from the plan, deliberate

The maintainer narrowed Plan 07 to architecture and schema engineering over existing
functions. Future simulator functionality is deliberately excluded. Existing unsigned
Arrow inputs remain eligible despite the schema review's broad suggestion to remove
unsigned types from the durable catalog; PSE's own canonical local ordinals become
signed, and external values require exact named conversion or explicit refusal.

The maintainer subsequently directed completion of all implementation and deletion
scope before integration testing. This intentionally permits temporary broken callers
and workflows during the coupled cut. Focused unit checks, compilation and generation
remain available; integration and performance qualification are consolidated at E10.
The final acceptance target remains zero failures and preservation of the existing
high-level process-model functionality.
