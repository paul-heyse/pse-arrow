---
status: proposed
reviewed: 2026-09-14
pins: Cargo.lock
blueprint_revision: 36
---

# Full Arrow and DataFusion capability deployment matrix

Companion evidence and implementation direction for the [design review](design_review_full-arrow-datafusion-capabilities_2026-09-14.md).
Policy authority is blueprint D10/§3.3.1 and ADR-0065. Arrow is the default for typed
data and columnar operations; DataFusion is the default for data transformation,
planning and execution across the system. Other libraries are acceptable when they
offer a distinctive advantage. **Every capability is eligible.**
Selection, a complete PSE execution route and qualification are separate questions.
This is a family-level deployment review, not a claim to have tested every public API.

**Evidence:** current consumers below are **Implemented** by source inspection;
pinned interfaces are **Interface-checked**; proposed uses are **Proposed**. Only
entries referring to the [probe receipt](../evidence/full-arrow-datafusion-2026-09-14/README.md)
are **Tested**, and only for those library cases. No speedup is **Measured**.
`DF/` denotes `external/datafusion/datafusion/` at the commit in the receipt;
`AR/` denotes `external/arrow-rs/`. Both match the resolved release versions.
The [feature inventory](../evidence/full-arrow-datafusion-2026-09-14/resolved-features.json)
records actual Cargo resolution, including transitive feature unification.

## Relational rules, transformations and inference

| Capability and pinned route | Current use or restriction | Deployment and semantic obligations |
|---|---|---|
| Scan/filter/project; `LogicalPlanBuilder`, native `Expr` | Core RulePlan operations implemented | Continue using native operators; source capabilities and inferred semantic fields accompany the plan. |
| UNION ALL | Implemented, including bounded recursion | Preserve multiplicity and branch support. A union is not automatically a set or a key merge. Probe passes. |
| UNION DISTINCT; `union_distinct` / `Distinct(Union)` | Already expressible by composition | No new fundamental operator required. Define full-row versus semantic-key equality explicitly. Probe passes. |
| INTERSECT/EXCEPT DISTINCT | No direct RulePlan declaration; native builders available | Use null-safe set equality; ordinary anti-join with nulls unequal is not EXCEPT. Preserve complete right-input absence binding. Probe passes native distinct cases. |
| INTERSECT/EXCEPT ALL | Native APIs have a demonstrated duplicate-count defect at this pin | Use per-full-value occurrence windows plus null-safe join/anti-join, or a separately verified upstream correction. Probe demonstrates both defect and working relational decomposition. Occurrence numbers are temporary multiplicity witnesses, never semantic identity. |
| Inner, left/right/full, semi/anti joins; `JoinType`, `join_detailed` | RulePlan exposes inner equijoin and left anti; internal native semi/left joins already exist | Extend one join contract with kind, equality/null policy and residual condition. Outer/anti joins require settled inputs in inference. Preserve unmatched-row meaning, cardinality and support. |
| Cross/non-equi joins; `cross_join`, `join_on` | RulePlan rejects empty equality keys and supplies no residual | Use for finite domain products, interval matching and contribution correspondence. Forecast cardinality; qualify optimizer extraction and metadata. Probe exercises non-equi and full outer joins. |
| Mark joins | Native engine variants, not exposed by RulePlan | EXISTS-style membership candidate. Pinned `common/src/join_type.rs:64` explicitly lacks full nullable mark semantics; do not substitute for nullable IN/ANY or four-valued truth. |
| EXISTS, IN, scalar subqueries | Native Expr; current profile omits correlated decorrelation rules | Use the same recursive source/function admission and declared input inventory. Test SQL null truth and zero/one/multiple scalar rows. Probe covers correlated EXISTS using upstream default pipeline, not the PSE profile. |
| Lateral joins | Native SQL and decorrelation, not PSE-declared | Candidate for correlated expansion. Qualify the actual supported join forms and subquery shape; parser acceptance alone is insufficient. |
| CASE, casts, arithmetic, null-safe comparisons | RuleExpr has comparisons/Kleene/null predicates but lacks general calls, CASE and arithmetic | Native expressions under a PSE result contract; derive quantity conversions and exactness separately. Branch laziness and error behavior are material. |
| Grouping, DISTINCT/FILTER aggregates, grouping sets/cube/rollup | Current five aggregate kinds; group keys are columns | Use native grouped computations for candidate summaries and completeness. Preserve empty groups, null groups and subtotal markers. Probe covers grouping sets and ordered/FILTER aggregates. |
| Sort/top-k/limit, DISTINCT ON | Output head sorting exists; general rule operations absent | Diagnostics, explicitly ordered selection and bounded presentation. Never silently drop tied valid candidates; rank is policy, not identity. |
| Windows: rank/dense_rank/row_number, lead/lag, aggregate frames | Builtins installed; RulePlan lacks window declaration | Candidate precedence and ambiguity with dense_rank, occurrence matching, ordered diagnostics. Declare peers, partition, frame and tie behavior. Probe preserves candidate ties. |
| UNNEST/list/struct access; nested functions and higher-order lambdas | Single-column unnest, field access and length implemented; broader functions installed | Use native nested transformations and multi-column expansion with explicit zip/product/ordinal/null/empty semantics. Do not scalarize indexed MathIR just because UNNEST exists. Probe covers list functions. |
| Positive distinct recursive CTE | PSE explicitly refuses distinct/FixedPoint; library supports it | Eligible for finite monotone closure. Distinct identity must exclude changing iteration depth; preserve/reconstruct every required support edge. Probe terminates a cycle. |
| Bounded UNION ALL recursion | Implemented finite bound and overflow detection | Retain for depth-sensitive/path multiplicity operations. Resource/termination failure is explicit; no successful truncation. Bounds are declared, not arbitrary tiny ceilings. |
| Multi-rule, multi-head four-valued fixed point | PSE scheduler and support state implemented | Keep domain truth/conflict/stratum semantics; execute relational bodies/deltas in DataFusion. Native CTEs are one lowering, not a universal substitute for the scheduler. |
| Semi-naive delta evaluation | Current code splits complete candidate output by accumulated keys | Prepare native delta-input variants for eligible positive branches; preserve old-key changed payload and alternate support. Measure preparation and repeated scan costs separately. |

Builder availability is documented in the pinned [LogicalPlanBuilder rustdoc](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/logical_plan/builder/struct.LogicalPlanBuilder.html).
Exact duplicate semantics were checked against `DF/expr/src/logical_plan/builder.rs:1390`
and the retained executable probe. Current restrictions are in
`crates/pse-schema/src/model/rule.rs:344`, `rule_expr.rs:64`, and
`crates/pse-rules/src/plan/recursive.rs:25`.

## Functions and semantic extension points

| Capability | Current use or restriction | Deployment and contract |
|---|---|---|
| Numeric, string, regex, datetime, encoding/crypto builtins | Default feature families installed | Use directly for relational transformations and diagnostics where semantics fit. Crypto functions do not become validation or identity authority. Time/random functions need an execution-purpose policy. |
| Field-aware ScalarUDFImpl | No PSE implementation/assembly route found | Generate adapters from actual KernelSpec/operation bindings. Preserve scalar fast paths and argument/return fields; validate dimensions, datum, basis and domain before kernel execution. Identity UDF field/null probe passes. |
| UDF `coerce_types`, strictness and conditional evaluation | Interface available, not a physical-type proof | Derive flags from the sole contract. Validate exact representability and inactive invalid branches. Do not promise pure behavior merely because the implementation Arc is retained. |
| UDF `simplify`, `preimage`, bounds and ordering hooks | Mostly unselected; preimage has R-25 consumer gate | High-value candidates when they eliminate real work. Exact replacement must preserve nulls, boundaries, precision and failure semantics; conservative pruning retains the original residual. |
| Configuration-aware / stable / volatile functions | Actual builtin Arcs retained, effects not checked in Functions admission | Capture session-time/config/external observations or declare nonreusable effectful execution. A bound configuration input is compatible with deterministic operation. Seed alone may not make parallel random evaluation reproducible. |
| UDAF / Accumulator / GroupsAccumulator | Native builtins used, no custom PSE UDAF | Prefer builtins; add custom aggregates for demonstrated batch-domain needs. Declare partial-state fields, merge/order/null/empty contracts and memory. Grouped acceleration does not eliminate ordinary accumulator support; evaluate `convert_to_state` only with a consumer. |
| UDWF / PartitionEvaluator | Window builtins installed | Use native window functions first. A custom evaluator requires real frame/peer/retract and bounded-state contracts; no implementation merely to tick an extension box. |
| HigherOrderUDFImpl / lambdas | Installed and retained despite historical map rejection | Eligible list transformations with lexical binding and child fields. Lambda expressions operating on relations need not replace the mathematical IR. |
| TableFunctionImpl / `call_with_args` | Defaults installed but not in retained function inventory; produced providers lack admission path | Use bounded range/member/descriptor sources. Bind implementation, arguments, session and output provider as one invocation; reject forged providers and undeclared sources. |
| AsyncScalarUDFImpl | Historically forbidden as a family | Eligible for explicitly effectful execution with cancellation and captured dependencies. Pure solver/kernel evaluation stays pure; service results can enter as admitted observation relations. |
| FunctionFactory / CREATE FUNCTION | Historically forbidden | Eligible compiler frontend to a declared function source and binding. Factory output must enter the same pre-seal inventory; no hidden second kernel definition or mutation of sealed sessions. |
| ExprPlanner, RelationPlanner, TypePlanner | No PSE implementations found | Use only for required authoring/SQL/type syntax; lower to the existing operation/MathIR contracts. `plan_type_field` can carry extension fields. |
| LogicalType / ExtensionTypeRegistry | Extension registry exists; LogicalType historically rejected | Mechanically project registry types when this improves coercion/planning. Do not independently author a second quantity system. Storage/logical coercion remains distinct from physical conversion. |

Pinned [ScalarUDFImpl](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/trait.ScalarUDFImpl.html),
[AggregateUDFImpl](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/trait.AggregateUDFImpl.html)
and [WindowUDFImpl](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/trait.WindowUDFImpl.html)
rustdoc establish the extension interfaces. Additional source checks:
`DF/expr/src/{udf,udaf,udwf,higher_order_function,async_udf,planner}.rs`,
`DF/expr-common/src/groups_accumulator.rs:232`, `DF/session/src/table.rs:601`.
Current function ownership is `crates/pse-catalog/src/session/functions.rs:17`.

## Planning, optimization and execution

| Capability | Current use or restriction | Deployment and contract |
|---|---|---|
| Analyzer and function rewrites | Two builtin analyzers plus admission brackets | Start qualification from the full pinned recommended pipeline; preserve PSE semantic admission and source typing. Record actual order/implementations/configuration. |
| Logical optimizer pipeline | Four selectable rules | Qualify union/set rewrites, subquery/lateral decorrelation, equijoin extraction, outer/empty/join elimination, CSE, grouping, projection and limit/filter pushdown. Trace semantic and support equivalence. |
| Physical optimizer pipeline | Three Wave 1, four Wave 2 rules | Qualify native join choice, distribution/repartition, sort/order, aggregate and projection improvements. Keep final requirement/sanity checks after PSE field-preservation transformations. |
| LogicalPlan::Extension / ExtensionPlanner | Explicitly deferred; codec refuses extensions | Eligible attribution or semantic-boundary wrapper when ordinary plans are insufficient. Expose/rebuild all inputs and expressions and declare pushdown behavior. Do not turn every rule into an opaque optimizer barrier. |
| Custom ExecutionPlan / physical expressions | Narrow metadata workaround exists | Use only for demonstrated missing behavior. State output fields, distribution, ordering, statistics, effects, cancellation and ownership truthfully; forward the actual planning context. |
| Prepared engine work inside numerical execution | Baseline D10 imposed a four-role/no-Newton ceiling | Eligible under the corrected policy. Native residual/AD is the current implementation, not a library ban; compare prepared batch/physical-expression work under the same derivative, precision, ownership and run contracts. Replanning queries per iteration is an avoidable cost, not a capability restriction. |
| Prepared plans / `reset_state` | Reanalysis and replanning each call/round | Reuse admitted logical/typed preparation first under exact bindings. Physical reuse requires fresh per-execution state; reset_state alone is not a recursive reset guarantee. |
| RuntimeEnv / memory and spill | Shared runtime, fallible reservation adapters implemented | Keep one generous deployment budget and explicit thread/disk limits. Evaluate spill operators and attribution; never confuse pool usage with total process memory. |
| Streams / backpressure / partitions | Native stream collected to Vec; output copy per batch | Expose owned streams and internal ownership-preserving handoff. Partition finite immutable sources, retain leases to last buffer owner and make pending cancellation wakeable. |
| Metrics / EXPLAIN / plan serialization | Logical observations and protobuf codec implemented | Record final physical pipeline and actual metrics; keep evidence distinct from semantic identity. Profiling must account for planning, admission and transfer, not just operator time. |
| SQL/unparser/protobuf/Substrait | SQL analytics and protobuf available; Substrait historically rejected | Frontends/derived interchange are eligible. One authoritative source; record omitted semantics and re-admit decoded plans against actual providers/functions. |
| DataFusion FFI | Not selected, no runtime plugin consumer | Eligible compiled extension boundary with pinned ABI/build compatibility and retained lifetime/implementation contracts. It is not a generic language bridge or trust certificate. |
| Spark compatibility / ANSI mode | Crate banned; global arithmetic default fixed | Eligible explicitly selected compatibility profile. Keep canonical mathematics on its declared numeric policy; test casts, overflow, division, nulls and decimal behavior. |

Pipeline evidence: `DF/optimizer/src/optimizer.rs:290`,
`DF/session/src/{planner,physical_optimizer}.rs`,
`DF/physical-plan/src/execution_plan.rs:473`; current profile
`crates/pse-catalog/src/session/profile.rs:27` and execution
`snapshot_session.rs:647`. Semantic qualification remains work; upstream defaults
are a broad candidate, not an automatic acceptance result.

## Sources, storage and effects

| Capability | Current use or restriction | Deployment and contract |
|---|---|---|
| Custom/listing/external TableProviders | Only retained PSE provider types admitted | Snapshot providers stay immutable; capture external source versions/content through import or bind genuinely immutable sources. Provider eligibility is broader than live mutable catalog access. |
| AsyncSchemaProvider | Historically rejected | Resolve/catalog-discover asynchronously before sealing, or bind explicit versioned observations. Awaiting is not itself competing authority. |
| Projection/predicate/limit pushdown | Exact filters run on full batch during scan planning | Build standard execution operators; project filter columns plus requested output early. Exact means identical complete results after removing residual; Inexact retains residual. |
| Constraints, statistics, ordering, functional dependencies | Admitted constraints and row counts exposed | Derive only from actual validated data/contracts. These are optimizer assumptions, not validators. Enrich statistics only where measured useful. |
| Dynamic filters, pruning, morsels | Historically rejected based on assumed small artifacts | Eligible for real scan/join workloads. Test no false negatives, late filter updates and cancellation. No custom scheduler before evaluating the native mechanism. |
| Parquet row groups/pages, projection, RowFilter/RowSelection, async ranges | Current import reads/concatenates complete artifact | Selective reads for already admitted immutable sources; retain exact source version and contract. Whole-snapshot validity cannot be inferred from only the rows selected by one query. |
| Schema evolution / PhysicalExprAdapterFactory | Not yet a lived storage migration path | Mechanically apply explicit schema migration; never infer semantic defaults from physical absence. Historical SchemaAdapter/SchemaMapper names are removed/deprecated, not excluded viable capabilities. |
| DML, MERGE, COPY, sinks / defaults | Published snapshot mutation refused; historically treated as global exclusions | Eligible inside private import/authoring/attempt workspaces. Convert validated result to ChangeSet/atomic publication; retries and external writes have explicit ownership and idempotency. |
| File path / row-number virtual columns | Historically rejected as position identity | Eligible source-location diagnostics/provenance. Persist semantic identity independently; storage reordering must not rename entities. |
| Remote transport/distributed/wire services | No deployment consumer | Eligible if a workload calls for them. Binding source revisions, provider compatibility and publication remains required; this review does not prescribe building a service. |

Pinned source surfaces: `DF/session/src/table.rs:428`,
`DF/physical-expr-adapter/src/schema_rewriter.rs:175`, `DF/datasource/src/morsel/`;
Parquet `AR/parquet/src/arrow/{arrow_reader,async_reader}/`.
Provider implementation evidence is `crates/pse-catalog/src/provider/table.rs:81`.
Optional integrations above are architectural eligibility decisions; their external
versions/ABIs need consumer-specific qualification before dependency selection.

## Arrow representations and kernels

| Capability | Current use or restriction | Deployment and contract |
|---|---|---|
| Primitive/Boolean/string compute, filter/take/interleave/concat | Filter/take/concat used; many transformations still materialize Cells | Use vectorized kernels and generated borrowed columns. Probe checks selected values and metadata; physical validity does not establish engineering meaning. |
| Typed primitive/dictionary/list/struct builders | Generated builders buffer Row objects and validate one-row batches | Generate column builders from the same registry declaration. Validate values as appended and cross-row constraints at finish; avoid a second handwritten row schema. |
| Slice/coalesce / BatchCoalescer / MutableArrayData | Some zero-copy slicing; complete materialization at query boundary | Tune batch size by workload and ownership. A tiny visible slice may retain a large backing allocation; compact only where beneficial and account for it. |
| Utf8View/BinaryView/ListView, large/nested/struct/map arrays | Persistent layout set is intentionally narrower | Eligible transient execution layouts with explicit normalization at persistence/export. Preserve child metadata/nulls/offsets. Canonical format support and engine working layout need not be identical. |
| Dictionary values and compaction | Canonicalization decodes values | Use native dictionary kernels; codes are not semantic IDs. Test null dictionary entries, reordered dictionaries and unreferenced values. |
| RowConverter/sort/partition | Used for canonical key ordering | Keep reversible exact typed comparisons. Native row encoding is a working representation, not a replacement for canonical identity framing. Probe distinguishes signed zero; DF grouping intentionally normalizes it. |
| Casts / decimal / temporal / timezone | Physical conversions available | Exact representability and physical compatibility precede relabeling. Approximation, calendar and rounding policies are explicit; no global ban on numeric operations. |
| Extension fields and Arrow validation | Active PSE admission implemented | Retain recursive registry validation and source derivation; Arrow layout checks cannot prove FK, units, enum membership or provenance. |
| IPC compression and writer options | Baseline global Clippy ban corrected in this review | Canonical bytes stay fixed/uncompressed; compressed transport is eligible with separate encoding checksum and exact decode/admission. Governance is now scoped to the canonical identity path. |
| CSV/JSON/Avro/Parquet readers and writers | Explicit schema import exists; Avro absent/banned | Eligible adapters using declared schemas. Schema inference may assist exploratory import, but cannot silently become authoritative RelationSpec. |
| Flight / C Data / C Stream / PyArrow | Flight/PyArrow companion banned; pyo3-arrow is selected | Flight and alternate bridges are eligible consumers, not automatically dependencies. Preserve ownership, schema metadata, nulls and loss-aware roundtrip. Keep the selected Python contract single. |
| Pool / Array::claim | Optional pool not selected | Useful ownership measurement candidate, not a fallible budget replacement: `MemoryPool::reserve` is infallible and may overfill. |
| Parquet Variant / derive-generated types / encryption / alternate encodings | Some companion crates banned or features unselected | Eligible boundary/transport/derived implementation tools. A Variant payload does not replace required typed model fields; generated structs remain registry-derived. Encryption/encoding belongs to physical transport, not semantic identity. |

Pinned sources: `AR/arrow-select/src/{filter,take,concat,coalesce}.rs`,
`AR/arrow-data/src/transform/mod.rs:136`, `AR/arrow-buffer/src/pool.rs:73`,
`AR/arrow-row/src/lib.rs:568`. Consumer bottleneck evidence:
`crates/pse-schema/src/codegen/rust/relation.rs:171` and
`crates/pse-ids/src/owned_buffer.rs:177`.

## Disposition of every historical DataFusion Reject row

The old map's rows 50–65 remain historical evidence, not policy. This table is an
explicit review disposition; it avoids silently reinterpreting old probe results.

| Old row | Current disposition |
|---|---|
| 50 configuration-aware functions | Eligible; bind actual configuration and effects. |
| 51 column defaults | Eligible derived authoring/attempt behavior; no independent model defaults. |
| 52 removed schema adapters | Use current PhysicalExprAdapterFactory; unavailable API is a version fact. |
| 53 Substrait | Eligible derived interchange; no second authoritative plan. |
| 54 compiler SQL | Eligible single-source frontend with typed binding; no duplicate hand-maintained rule. |
| 55 higher-order/lambda | Eligible; already partly installed/admitted. |
| 56 async schema resolution | Eligible; seal or version actual resolved inputs. |
| 57 async UDF | Eligible effectful execution; pure kernels retain their contract. |
| 58 FunctionFactory | Eligible frontend to the one declared function/binding inventory. |
| 59 virtual file/row columns | Eligible physical-location evidence; not semantic identity. |
| 60 MERGE | Eligible private attempt transformation with coherent publication. |
| 61 Spark/ANSI | Eligible explicit compatibility profile. |
| 62 morsels | Eligible scan scheduling; measure the consumer. |
| 63 LogicalType | Eligible registry-derived planning projection. |
| 64 external/distributed/wire integrations | Eligible deployment choices; no service construction implied. |
| 65 dynamic filters | Eligible native optimization with full-result differential checks. |

Arrow Flight, Avro, PyArrow, Parquet Variant and derive companion exclusions are
likewise replaced by the placement contracts above. No feature is disqualified
simply because the initial wave did not use it. Actual upstream limitations and
failed probes remain visible; they guide correction and qualification.
