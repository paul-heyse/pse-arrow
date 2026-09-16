---
title: Schema-first pivot of the existing codebase to native DataFusion and Delta
status: in-progress
date: 2026-09-16
adrs: [ADR-0065, ADR-0066, ADR-0068, ADR-0069]
phase: 1
---

# Schema-first pivot of the existing codebase to native DataFusion and Delta

**Implementation in progress; no package is certified complete.** This is the
architecture-only successor to Plan 07. ADR-0069 and blueprint revision 40 record
the authorized schema and scope changes. SP00/SP01/SP02/SP03 and the local-check foundations
of SP06/SP07 are active; terminal acceptance
and full predecessor deletion remain open.

**Execution order:** exact field contracts → coupled values and native constraints →
typed identity/provenance → coherent expression/numerical schemas → durable declarations
and common provider admission → complete replacement of existing compiler/storage/Python
paths → native change/retention → architecture qualification and deletion audit.

The first priority is schema engineering because it removes the conventions and
reconstruction work that would otherwise be carried into the provider and Delta pivot.
Each schema cut includes its generators and consumers. There is no old/new schema
compatibility layer, dual store, migration campaign or transitional product mode.

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

**Implemented, inspected:** both a native Delta route and a live custom
`Catalog`/`Snapshot`/stage/memo lifecycle exist. The compiler and Python opening still
depend on the latter. Native fixed-point and Ipopt operators have real executed child
plans. Closed `RulePlan`/`RuleExpr`, field-only metadata conventions, recursive cell
conversion and positional numerical/source correspondences remain.

**Interface-checked:** DataFusion 55.1.0, Arrow/Parquet 59.3.0, object_store 0.13.2,
Delta `58f07cd62bfbce3649a7e1c87c696288068ae184`, kernel
`8ba063f8f84fec222000f66d40d70911d7c79675`. Research used the two local skills,
pinned Cargo sources, provider map and `just metadata`; no Context7 was used.
The [planning evidence](../design_review/evidence/schema-first-native-data-pivot-2026-09-16.json)
records selected hashes, canonical API references and coverage limits.

At plan creation, no runtime probe or benchmark established the proposed schema
changes. Existing component receipts in the completion review established their
recorded scope only. `just doctor` then reported **1 freshness failure, baseline 0**,
and a stale editable extension. The implementation receipts below supersede that
environment status. Planning skill scans produced 28 DataFusion and 4 Delta advisory
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
| `datafusion_session::{catalog,schema,table}` provider traits, native registries/views/factories | Complete native hierarchy and common admission for actual bindings | Provider map; defaults only cover SQL INSERT; consumed statistics use physical/file paths rather than relying on the unused provider statistics hook |
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

### SP00 — Freeze current-function scope and record changed contracts

**Depends on:** none. **Status:** In progress.

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

**Depends on:** SP00. **Status:** In progress (native declaration and caller replacement implemented; qualification in progress).

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

**Depends on:** SP01. **Status:** In progress (canonical string enums, signed extensions and metadata/runtime counts, recursive integer domains and tagged configuration/feature values implemented; remaining scalar/value and collection changes open).

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

**Depends on:** SP02. **Status:** Proposed.

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

**Depends on:** SP02–SP03. **Status:** Proposed.

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

**Depends on:** SP02–SP04. **Status:** Proposed.

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

**Depends on:** SP01–SP05. **Status:** In progress for native local predicates; alternatives, relational invariants and domain transfer remain open.

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

**Depends on:** SP01–SP06. **Status:** In progress for declared member-table properties, native checks and cold reconstruction; complete table policy and publication-root integration remain open.

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

**Depends on:** SP06–SP07. **Status:** Proposed.

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

**Depends on:** SP03, SP07–SP08. **Status:** Proposed.

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

**Depends on:** SP03–SP04, SP06, SP08–SP09. **Status:** Proposed.

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

**Depends on:** SP05, SP08–SP10; part of the same caller-replacement cut. **Status:** Proposed.

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

**Depends on:** SP09–SP11. **Status:** Proposed.

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

**Depends on:** SP00–SP12. **Status:** Proposed.

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

| Existing mechanism | Replacement | Owning cut |
|---|---|---|
| Top-level-only field semantics, bare nested child tuples and closed physical type catalog | Recursive contract over native Arrow fields plus domain facets | SP01–SP02 |
| Tag/payload and per-row quantity conventions, PSE unsigned ordinal conversion cascade | Generated alternatives/quantity values/signed range contracts | SP02 |
| JSON row keys, parallel spans, positional diagnostics, post-hoc support reconstruction | Typed scoped row references and co-located spans/support/generated diagnostics | SP03 |
| Canonical expression edge/payload table families and their rejoin APIs | Coherent node rows with native derived views | SP04 |
| Name-encoded numerical outputs and handwritten parallel result schemas | Generated program/vector/coordinate and outcome contracts | SP05 |
| Metadata-only function twins and closed general invariant/rule/type machinery | Native plans/functions plus one domain transfer/check declaration | SP06/SP10 |
| Metadata-stripped Delta layout and monolithic UDF-only local CHECK | Self-describing compatible fields/properties/native checks | SP07 |
| Root operation callback dispatch, native-config mirrors, duplicate inventories and unadmitted paths | Actual shared native hierarchy, composition and common policy | SP08 |
| Old publish journal/ref/CAS and member retry assumptions | Typed native Delta attempts, exact selections and reconciliation | SP08–SP11 |
| Snapshot/stage driver, artifact memo/hints and general Cell transformations | Native current-function plans, explicit algorithm boundaries and dependencies | SP10–SP12 |
| Store/manifests/sidecars/contexts/encoding membership and old Python/codecs | Target-only publication/provider handles and owned streams | SP11 |
| Old golden stores/fixture generators and simulator-only terminal prerequisite | Fresh target current-function architecture qualification | Each cut; SP13 audit |

Every row includes callers, generated schema artifacts, fixtures and exports. SP13
audits absence; it is not a holding area for deletions postponed from earlier cuts.

## Verification

### Architecture acceptance matrix

All checks are **Proposed** until executed; baseline is **0**. “Tested” receipts must
name command, feature/profile, current source state and failure/skip counts. Test
selection follows the newly scoped current-function contract, not old implementation
forms or new simulator features.

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
| `just doctor`, `just metadata`, `just family-check` | Scope/dependency refresh; metadata alone does not prove behavior or the full resolved feature graph |
| `just test-package pse-schema ...`, `pse-relations`, `pse-catalog` | Field/generation/native contract changes; recipes supply force-validate |
| `just test-package pse-authoring ...`, `pse-rules`, `pse-numerics`, `pse-tests-engine` | Affected existing semantic consumers and current-function journeys |
| `just native-solver-test --no-fail-fast` | Preserve/requalify the implemented Ipopt operator in the pinned container; no new simulator scope |
| `just codegen`, `just codegen-check`, `just governance` | Change generators, regenerate protected outputs, compare all targets, enforce target invariant fixtures |
| `just py-sync`, `just py-test`, `just quality` | Current editable extension and target Python boundary; replace old store fixture dependencies |
| `just ci-fast`, required feature checks, `just adr-lint`, `just lint-agents`, `just docs` | Final current-source architecture/quality/decision closure |
| Proposed `just architecture-acceptance <new-output-directory>` | Existing ordinary tests in one target-only terminal journey, replacing the inappropriate new-simulator prerequisite |
| Focused ordinary benchmarks / `just bench-smoke` | Real named measurements; smoke alone is not a throughput, latency or memory result |

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

All are currently **Unresolved for this proposed plan**; this does not reverse the
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

The maintainer authorized the detailed execution plan after this checkpoint.
The current working tree is the starting implementation; prior receipts are not
terminal acceptance. Each replacement includes its callers and deletions.

| Step | Plan packages | Execution and closing evidence | Status |
|---|---|---|---|
| E00 | SP00 | Current-function oracles, deletion inventory and final acceptance entry point | In progress |
| E01 | SP01–SP02 | Native schema/math dependency placement, signed local integers, alternatives, quantities and collections; Q01–Q02 | In progress |
| E02 | SP03 | Declared keys, exact references, co-located spans/support and structured diagnostics; Q03–Q04 | In progress; typed keys, composite references, source spans, tagged diagnostics and algorithm/configuration support co-location implemented; selected support qualification passed, complete caller replacement open |
| E03 | SP04–SP05 | Coherent expression rows and dimensioned numerical vectors, existing solver contracts; Q05–Q06 | Proposed |
| E04 | SP06/SP10 | Native invariant/rule bindings and one domain-transfer implementation; Q04/Q07/Q11 | In progress; declared native SQL row checks tested, general rule/domain replacement open |
| E05 | SP08 | Actual native session/provider/planner composition and complete admission; Q09 | In progress; native configuration preservation tested, complete caller cut open |
| E06 | SP07/SP09 | Declared tables/control facts, complete selections, member/root reconciliation; Q07/Q08/Q10 | In progress; declared roots/members and selections tested, member-attempt reconciliation open |
| E07 | SP10 | Existing authoring and P0–P10 responsibilities as native plans; delete Driver/stage/memo; Q11 | Proposed |
| E08 | SP11 | Exact Rust/Python publications, owned streams and full predecessor-store deletion; Q12 | Proposed |
| E09 | SP12 | Typed dependencies, CDF, exact reuse and coordinated native retention; Q13 | Proposed |
| E10 | SP13 | Ordinary extension proof, measurements, deletion audit and final Q01–Q14/G1–G7 receipts | Proposed |

E05–E08 remain one caller-replacement cut. E10 verifies deletions completed in
their owning steps. No compatibility API, migration reader or new simulator
functionality is part of this sequence.

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

## Open items

These are implementation proof obligations within the authorized target, not requests
to preserve legacy code or defer the hard pivot.

| Item | Chosen direction / resolution point |
|---|---|
| Registry/native dependency placement | Keep declarations lightweight and generators authoritative; place native plan binding where dependencies remain acyclic. Use native types/expressions rather than copying their AST into schema — SP01/SP06 |
| Deep collection validation and field masks | Generate exact visible-value checks; qualify native/Delta coverage and add only the smallest missing domain function — SP01/SP06/SP07 |
| Persisted extension metadata | Generated logical descriptor plus compatible physical extension annotations; qualify raw and registered external readers — SP07 |
| Expression/numerical physical packing | Canonical typed contracts are selected; measured costs choose the native physical layout/tuning, without a second model authority — SP04/SP05 |
| Attempt recovery and active-reader exclusion | Typed operation input/outcome identity and exact versions; qualify supported local coordination, refuse unqualified destructive remote cleanup — SP09/SP12 |
| Existing quality failures | Replace obsolete forms with target coverage, fix valid findings, rerun full final current source; never adopt the prior failure count as a baseline — SP13 |

No performance claim, unknown upstream feature, default session or metadata string
may stand in for executable qualification. The implementation may adjust module/API
names and package granularity while preserving A01–A08, scope exclusions and deletion
obligations. It must not reinstate predecessor compatibility to make intermediate
builds pass.

## Outcome (partial; updated during implementation)

### What was built

**Implemented, partially Tested:** the scope/decision records, current-function oracle
inventory, shared exact Arrow/Delta boundary, native recursive registry declarations,
canonical string enums, signed extensions and metadata/runtime fields, tagged
configuration/feature values, native local value checks, explicit singleton keys,
correlated references, native scoped row tokens and generated tagged diagnostic evidence
described in the checkpoints. Declared member tables persist native checks and identity,
with registry-free cold reconstruction. The old declaration types and monolithic local
Cell validation callbacks, JSON key codec, source key-part structure and positional
execution-finding constructor on the replaced routes are deleted; later architecture
cuts and full legacy deletion are unfinished. No terminal architectural acceptance is
claimed.

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
