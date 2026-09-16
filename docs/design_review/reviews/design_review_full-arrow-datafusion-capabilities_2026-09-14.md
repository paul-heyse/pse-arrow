---
status: proposed
reviewed: 2026-09-14
depth: deep
blueprint_revision: 36
adrs: [ADR-0065]
evidence: Interface-checked
---

# Design review: full Arrow and DataFusion capabilities

## 1. Decision and scope

**Follow-up:** the maintainer subsequently requested a fundamental Wave 1 review
focused on deriving guarantees through native logical-plan construction.
[That review](design_review_wave1-logical-plan-foundations_2026-09-14.md) supplies
the combined LP0–LP7 foundation/consumer migration sequence. It integrates this
review's C0–C7 work; it does not change the retained capability evidence below.

**Decision: Revise the current Wave 2 execution design before completing its
implementation.** The full Arrow/DataFusion capability policy is the recommended
direction. The existing small RulePlan and engine profile are implementation
checkpoints, not architectural ceilings. No library feature family is categorically
unavailable. This review does not claim that all features are already exposed or
qualified through PSE.

**Proposal:** Arrow is the default for typed data and columnar operations.
DataFusion is the default for data transformation, planning and execution across
the system. Other libraries are acceptable when they offer a distinctive advantage.
Use the complete native library surface through one PSE semantic
contract layer: actual bound sources/functions, typed operation parameters and
outputs, explicit effects, provenance and lifecycle. Prefer native relational and
columnar mechanisms; preserve specialized MathIR/quantity/numerical algorithms.

**Reviewer:** Codex, with three independent read-only investigations of rules,
functions/planning, and Arrow/providers/runtime. Root inspected the cited findings
and ran the retained library characterization.

**Affected revisions:** branch `wave2/semantic-compilation`, base commit
`eebc82e9f2298e4a53e08818cfd2a1dfb6fc310b`, plus the substantial uncommitted Wave 2
implementation present on 2026-09-14. The base commit alone does not identify the
reviewed implementation. Blueprint revision 33 was the investigation baseline;
revisions 34/36 record the authorized capability policy and expanded default roles
under proposed ADR-0065.
Accepted ADRs remain unchanged; formal decision-PR reconciliation remains pending.
Concurrent ADR-0066 / blueprint revision 35 broadens dependency admission across
all libraries. That separately authored policy work is preserved, not attributed
to this review; its dependency configuration already removes the six identified
Arrow/DataFusion bans.

**Observable outcome:** future rules and compiler transformations can use native
joins, set operations, windows, nested functions, recursion, UDF families, optimizer
and planner extensions, streaming and storage facilities without first overcoming
an arbitrary feature prohibition. The first implementation priority is a complete
semantic execution route for the real Wave 2 consumers. Library adoption itself
requires no additional admission process or ADR under the maintainer's policy.
Qualification here means testing the actual behavior being delivered, not certifying
unused APIs or building a new query framework before product work can continue.

### Method and coverage

- Read blueprint D1–D14, §3–§8, §14, §18, §20–§24, relevant accepted/proposed ADRs,
  Plan 04 and the design charter. Inspected capability-map clusters and every
  historical DataFusion Reject row 50–65. The [deployment matrix](full-arrow-datafusion-capability-matrix-2026-09-14.md)
  covers relational operators, function families, planning, execution, sources,
  storage, effects and Arrow representations, including unused optional facilities.
- Inspected actual RulePlan/RuleExpr declarations, lowering, recursion/strata/delta,
  session/profile/function/provider admission, field repair, result ownership,
  generated views/builders and selected P6/P9/P10 relational work. These are the
  load-bearing extension and execution boundaries. This is not a line-by-line
  audit of every P3–P10 pass or all numerical algorithms.
- Used Context7 `/apache/datafusion` and `/apache/arrow-rs` for discovery. Main-branch
  snippets sometimes differ from the pin; exact source/rustdoc and compilation
  determine the current API. Used versioned rustdoc for scalar, aggregate, window
  and logical-plan interfaces, plus the resolved registry sources and verified
  external reading copies at DataFusion 55.1.0 and Arrow 59.3.0.
- `just metadata` and full `cargo metadata --locked --offline --format-version 1`
  establish one resolved version across **49 Arrow/Parquet/DataFusion packages**,
  plus object_store 0.13.2. The retained [feature inventory](../evidence/full-arrow-datafusion-2026-09-14/resolved-features.json)
  includes actual transitive features. It is the default workspace resolution,
  not an all-features run. Optional absent packages remain eligible, not installed.
- **Tested:** [standalone pinned characterization](../evidence/full-arrow-datafusion-2026-09-14/README.md),
  Rust 1.98.1, dev, Arrow force_validate, offline/locked execution, partition
  targets 1 and 4, batch size 2, finite in-memory inputs. **37 checks: 33 passed,
  4 failed semantic expectations, baseline 0; exit 1.** The failures expose two
  raw set-builder defects in both partition settings. The native window/join
  correction passes. This deliberately remains a failing counterexample receipt,
  not a waiver or a product test result.
- No full workspace tests or benchmarks were run for this review. A focused
  governance run covers the compression-lint scope correction; its receipt is in
  §9. Existing Wave 1
  validation is a planning assumption, per maintainer direction. Wave 2 source
  presence and generator bootstrap do not establish terminal acceptance. The
  startup doctor reports an outdated Python environment/extension; repairing it
  belongs to implementation qualification. No numerical parity or speedup is claimed.
- Did not execute plugins, remote providers, DML, Spark profiles, custom UDAFs/UDWFs,
  cancellation under stalled I/O, or full PSE admission of the broader SQL surface.
  Those mechanisms are interface/disposition findings with explicit tests below.
  Historical rustdoc-JSON receipts were not regenerated. The separate
  `tooling/dfarrow-apiex/verify/Cargo.toml:4` still describes 55.0.0 and uses an
  Arrow 59.2.0 caret plus external path dependencies; it is not this review's
  fresh pin-qualified compile evidence.

The broad candidate pipeline is not assumed safe merely because it is upstream's
default. Conversely, a missing PSE declaration, absent consumer or unqualified
optimization is not an architectural prohibition. Every use must establish the
semantics it actually needs; a feature list cannot establish that by itself.

## 2. Authority and lifecycle map

| Concept or fact | Semantic type and identity | Authority / owner | Revision boundary | Permitted update | Derived representations |
|---|---|---|---|---|---|
| Engineering facts and quantity/operator declarations | Registry-owned typed relations and semantic IDs | Authored/reference contracts; blueprint D1/D5 | Registry/package/model revision | Declared authoring/migration | Arrow schemas, generated columns/rows, engine types |
| Rule/transformation meaning | Rule ID/version, input roles, parameters, head/truth/support/effect contracts | One rule/operation declaration | Rule/package revision | Versioned declaration | Native Expr/LogicalPlan, prepared binding, explanations |
| Function implementation | Actual implementation plus declared signature/domain/effects/config | Existing operation/KernelSpec binding | Provider/build and binding revision | Assemble before sealing | UDF/UDAF/UDWF/TVF adapters and inventory |
| Engine policy | Ordered implementations and complete relevant settings | Bound engine profile | Explicit profile revision | Construct new session | Analyzer/optimizer pipelines and observations |
| External observations | Typed captured values, source version and acquisition evidence | Import/observation stage | Captured observation revision | Explicit effectful acquisition | Immutable source provider or observation relation |
| Execution workspace | Attempt identity, owned buffers/streams/spill/cancellation | Driver and pass attempt | One attempt | Private execution | Partial batches and metrics, not committed model truth |
| Derived output/support | Admitted relation keys/values plus exact lineage | Producing pass under actual dependencies | Atomic output bundle | Validate then publish | Snapshot, diagnostic plan export, Python inspection |

**Opaque behavior:** numerical kernels, graph algorithms, parsers, provider I/O and
specialized execution may remain ordinary code behind complete contracts. A
DataFusion function body is not automatically pure. A SQL string, protobuf or
Substrait export is not independently editable authority alongside its originating
rule. A single declared SQL body may be a frontend; do not maintain an equivalent
second handwritten RulePlan body.

**Identity:** changing row order, dictionary codes, partitioning, physical encoding
or a display name does not rename an entity. Engine row encodings and hashes may
accelerate comparisons but do not define PSE semantics. Reuse compares the actual
complete dependency descriptor, including absence, candidate universes, policies,
function bindings and lineage; checksums identify evidence/artifact bytes only.

## 3. Semantic contracts and invariants

| Contract | Representation | Enforcement boundary | Failure behavior | Evidence |
|---|---|---|---|---|
| Value and physical compatibility | Actual fields/arrays, registry types, named conversions | Source admission, expression typing, head/output admission | Typed rejection before assumptions or publication | Existing `session/admission.rs:163` and blueprint §14.2; broader operators unqualified |
| Set versus bag and key versus payload | Explicit multiplicity/equality contract | Native lowering and result/support comparison | Refuse unsupported semantics; never silently use semi/anti as bag subtraction | Retained ALL counterexamples and corrected lowering |
| SQL nulls versus inference truth | Kleene expressions; separate True/False/Unknown/Conflict state | Rule predicate, strata and head admission | Explicit undecided/conflict handling | Existing rule contracts; native SQL alone supplies no four-valued scheduler |
| Order, ties, precision | Declared partition/frame/tie/rounding policy | Window/aggregate planning and numeric admission | Ambiguity or invalid numeric operation remains visible | Window probe; platform comparison work remains |
| Immutable source/function ownership | Actual retained implementation/source, registry and snapshot binding | Pre-seal assembly and recursive plan admission | Reject foreign/changed bindings | `functions.rs:32`, `admission.rs:188` |
| Effects and reproducibility | Execution purpose plus captured ambient inputs | Before optimization and execution/reuse | Refuse unbound ambient behavior for reproducible rules | Gap F02; no runtime closure claimed |
| Coherent publication and resource lifetime | Attempt-owned stream/buffers; complete output bundle | Driver/store commit | Failure/cancellation never publishes success | Blueprint §14.3/§20; new stream lifecycle remains unverified |

**Absence:** SQL null, no matching row, no candidate, unknown truth, explicit false,
conflict, absent optional input and failed computation are distinct. Anti/outer
joins and ranking can be non-monotone when candidates arrive later; execute them
after the relevant positive closure or under an explicit decision stratum.

**Equivalence:** compare complete typed values, nulls, multiplicities, required
ordering, metadata, diagnostics and exact support. Distinct recursion may converge
on semantic tuples while support edges continue to accumulate. Compare actual
support sets as well as head keys. Approximate aggregates/numerics are eligible
only under a selected accuracy policy; they cannot replace exact feasibility or
identity checks by accident. Hash collisions are handled by actual equality, as in
`DF/physical-plan/src/aggregates/group_values/row.rs:148`; the same code deliberately
normalizes signed zero at line 120, so its float grouping is not PSE canonical-key
identity. No new hash-based validation mechanism is proposed.

## 4. Derivation and execution design

The [matrix](full-arrow-datafusion-capability-matrix-2026-09-14.md) gives the broad
capability disposition. The proposed implementation is a thin semantic layer over
native plans, with the following lifecycle.

| Stage | Inputs/dependencies | Output contract | Preconditions | Effects/ownership | Provenance/invalidation |
|---|---|---|---|---|---|
| Declare and bind | Rule/operation contract, actual sources/functions/profile | One complete typed binding | Registry compatibility; explicit execution purpose | Immutable assembly | All actual dependencies, including absent roles |
| Prepare | Bound native expressions/plans | Typed admitted logical plan and support plan | Correct operator cardinality/truth/field derivations | No model mutation or hidden reads | Retain semantic source mapping; reuse only under exact binding |
| Optimize and physical-plan | Full pinned candidate pipelines plus PSE rules | Executable plan with truthful properties | Differential qualification; final requirements/sanity checks | Shared runtime, planned resources | Observe actual analyzer/logical/physical passes |
| Execute | Immutable inputs; fresh attempt state | Owned stream of provisional admitted batches | Captured ambient inputs/effects; resource settings | Cancellation, spill, backpressure | Actual support and execution observations |
| Settle inference | Delta/accumulated assertions and support | Complete stratum or typed failure | Finite domain/termination policy; lower-stratum negation | Private scheduler workspace | Exact value/conflict/support convergence |
| Publish or reuse | Complete validated output bundle | Immutable committed outputs and terminal evidence | Preconditions and output invariants hold | Existing atomic commit | New attempt attribution; no backdated invented execution |

### Use native plans without constructing a second query engine

Retain the existing RulePlan forms as supported authoring conveniences. Add native
join/null/residual, expression-call, window/group and recursion semantics needed by
real consumers to the **one** operation contract. For broader transformations, use
a registered native-plan implementation with typed input roles/parameters/head,
truth/cardinality/effects and support requirements. Its actual implementation and
version are bound like other providers. It is not an opaque closure serialized as
a rule, and it cannot read undeclared state. Persist the declarative binding; derive
its native plan. Generate mechanical projections/visitors from the shared contract
where possible. Do not mirror every DataFusion node/function in another universal AST.

This extends D10 while preserving D6: DataFusion operates over the relational MathIR;
it does not become the equation language. A CASE/list/window in contribution matching
is a relational operation. A derivative, unit conversion or indexed symbolic integral
retains its explicit MathIR/quantity meaning until the appropriate lowering stage.

### Make the broad engine pipeline selectable and observable

Start with the **complete ordered pinned recommended analyzer/logical/physical
pipelines** as the candidate profile. Preserve admission brackets and add required
PSE transformations as named rules. Capture installed implementations and complete
relevant configuration, including function rewrites, type/expression planners and
compatibility semantics. Compare against the existing narrow profile and independent
small oracles. A failed semantic case produces a specific documented exception or
correction; it does not justify excluding an entire optimizer family.

Correct field-preservation at producer hooks where possible. Keep the necessary
physical repair as a registered physical rule with checks **after** it, and record
the final executed physical plan. Adding an opaque logical wrapper solely for
EXPLAIN is optional; use it when attribution/support cannot be retained cleanly by
ordinary plan metadata and observations. Expose every child/expression and pushdown
obligation if adopted. Do not sacrifice standard optimization unnecessarily.

### Expand the default role across the system

Baseline D10's exhaustive four-role/no-Newton description is replaced by the default
above and the expanded responsibility table in blueprint D10. Apply Arrow and
DataFusion to ingestion/normalization, validation queries, semantic compilation,
inference, dependency and incremental computation, provenance, batch evaluation,
case/scenario data and results processing. This corrects both the capability ceiling
and the description that confined DataFusion to a few subsystems. Existing MathIR,
physical meaning, explicit run effects and numerical contracts remain applicable.

### Complete extension assembly once, before sealing

One pre-seal assembly route must retain scalar, aggregate, window, higher-order and
table functions, plus selected planners/codecs. Contracts project from existing
operation/kernel declarations. Retain actual implementation objects, signatures,
field derivations, effects and semantic configuration. Table-function output needs
an invocation-bound provider capability; allowing arbitrary providers by schema/name
would destroy the current useful admission boundary.

Stable/volatile/configuration-aware functions remain available. Reproducible compiler
execution captures inputs or rejects the unbound invocation; exploratory analytics
can explicitly select different reproducibility/effect semantics. Async I/O belongs
to a declared effectful stage or invocation with owned cancellation and capture.
No post-seal registry mutation or ambient time/randomness enters a pure stage memo.

### Use columnar preparation and execution end to end

Generate borrowed typed column views and column builders. Keep Row/Cell conversion
for explicit boundary or specialized algorithm work. Establish a private immutable
admitted-batch capability bound to the actual registry/context, so unchanged values
need not be decoded again during every plan traversal. Transformation schemas,
foreign provider identity, changed contracts and newly produced values are still
actively checked. This is ownership-based retention of validation, not a hash cache.

Move provider filtering into normal native execution. Project requested plus filter
columns early; preserve limit/residual correctness. Offer an owned result stream
retaining session/context and final-buffer leases. Reuse owned admitted outputs
between private workspaces; copy only when ownership or representation requires it.
Compare retained and visible buffer size, especially sliced nested/string arrays.
Keep the canonical persistence frame separate from execution batch layout.

### Apply this to Wave 2 consumers

- **P4/P5:** native relational compatibility and scope closure; qualify distinct
  recursion for positive single-head finite closure. Preserve the scheduler for
  multi-head/conflict/provenance semantics.
- **P6:** native candidate/demand joins and anti-joins, grouping and tie-preserving
  ranking. Derive the minimum precedence set first; an unresolved tie is a diagnostic,
  not `row_number = 1`. Bind the entire candidate universe for invalidation.
- **P7/P8:** domain/member joins, contribution matching, grouping, list transforms
  and unnest produce typed expansion descriptors and exact support. Keep ordered
  symbolic substitution and graph construction in the MathIR algorithms.
- **P9/P10:** relationally join method parameters, template symbols, instance/domain
  bindings and products before physical validation and graph typing. Replace repeated
  full-inventory scans; do not rewrite quantity algebra or graph traversal in SQL.
- **Analytics and later waves:** native window/statistical/approximate operations,
  batch UDF kernels and selective file scans are eligible. Add UDAFs, plugins,
  transports or distributed execution only when their real consumer warrants them.

**Coherent publication:** input capture, DML/MERGE and sinks can operate in private
attempts, but the existing validated atomic model publication boundary remains.
Read-only inspection still cannot mutate model truth. External path/row virtual
columns carry physical location evidence, never stable semantic identity.

## 5. Representative journeys

### Ordinary extension: property candidate ranking

A new provision adds declarative eligibility, precedence and a method binding.
DataFusion joins candidate/instance/domain relations and uses dense_rank over the
explicit precedence tuple. All rank-1 ties remain visible. A separate uniqueness
check decides whether selection is valid. Support identifies actual matched rows
and the complete candidate-set dependency. No property-name callback or new engine
operator is necessary. Shuffle input/partitions: selected facts, tie diagnostics and
support must remain identical. The retained window probe establishes only the
native tie behavior, not this P6 implementation.

### Meaningful change: another provider becomes eligible

Change an initially absent candidate or bound function configuration while keeping
the previously selected output values equal. The exact dependency descriptor changes;
the old stage cannot be reused solely because the head hash matches. Recompute
selection and support; identical immutable relation storage can still be shared.
Also test same-name/different-implementation substitution: retained actual function
identity must reject it. This attacks reuse at the semantic cause, not at a digest.

### Boundary: compressed Arrow transport and alternate working layouts

An admitted result uses view arrays internally, crosses an owned C Stream boundary
or a compressed IPC transport, then re-enters under the declared schema. Canonical
identity still uses its fixed normalization/framing. Check child fields, dictionary
values, nulls, exact numerics and retained leases. Physical file/row location may
change without semantic IDs changing. Neither a matching checksum nor an Arrow
schema match permits loss of quantity meaning. The broadened transport route is
proposed, not a new implemented encoding.

### Failure: pending I/O, resource pressure and retry

An explicit async observation or query stream becomes Pending indefinitely. Cancel
without waking the source: the current AtomicBool token cannot wake that await.
The corrected owned stream must observe cancellation directly, release resources,
record a cancelled attempt and publish no successful partial bundle. Repeat with
spill exhaustion and crash before commit. A retry reuses only fully admitted captured
inputs or reacquires according to the declared effect policy.

### Adversarial operator counterexample

For `A=[1,1,2,NULL]`, `B=[1,NULL]`, INTERSECT ALL should produce `[1,NULL]`, and
EXCEPT ALL should produce `[1,2]`. At the pin, raw builders produce `[1,1,NULL]`
and `[2]`. The source uses semi/anti joins without occurrence accounting.
Pair occurrences per full value using a native window and null-safe equality:
the retained probe returns the expected bags under both partition settings.
Extend the correction to multi-column/null/dictionary inputs and source support
before PSE admission. SQL API names are not a correctness oracle.

## 6. Acceptance gates

Verdicts address this proposed broad PSE execution route, not whether optional
future services must be implemented now. Unresolved means no acceptance claim.

| Gate | Result | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | **Pass, scoped** | One registry and actual retained provider/function binding in inspected paths; proposed native projections preserve those owners | Keep single declaration/implementation binding; no independent SQL/function/type authority |
| G2 — Semantic fidelity | **Unresolved** | Raw ALL counterexamples refute naive lowering; corrected library decomposition passes; broader PSE null/field/support behavior untested | Qualify corrected lowering and each consumed operator's exact output/support contract |
| G3 — Validity | **Unresolved** | Existing admission is substantive; new TVF/type/function/native-plan routes have no complete implemented admission path | Implement invocation-bound source/output admission and adversarial negatives |
| G4 — Hidden behavior | **Unresolved** | Actual function identity is checked, but volatility/config/time capture is not an execution policy | Probe admitted ambient functions; bind purpose/effects/observations before pure execution/reuse |
| G5 — Consistency and recovery | **Unresolved** | Existing commit contract retained; pending cancellation and new owned-stream/effectful lifecycle not qualified | Pending-stream cancellation, final-owner release, interrupted publication and retry tests |
| G6 — Transformation and reuse | **Unresolved** | Physical repair is outside declared optimizer observation; broad rewrites/delta/preparation reuse need equivalence | Named physical rule, final invariants, actual dependency and incremental-versus-clean tests |
| G7 — Truthful capability claims | **Unresolved** | Installed functions/native APIs exceed selectable/admitted/executed PSE capabilities | Generate capability status from actual bindings/profile and named qualification; never equate installed with supported |

## 7. Principle findings

`DF/` paths refer to the pinned external DataFusion source defined in the matrix.
Source findings are **Implemented / Interface-checked**; proposed corrections are
not reported as delivered production behavior.

| Finding | Principle IDs | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|
| **F01 — Native ALL set builders do not implement bag multiplicity** | DM-24, DM-40, DM-43, DM-53 | `DF/expr/src/logical_plan/builder.rs:1426`; retained probe has 4 failures/baseline 0 | A naive new rule lowering loses or duplicates valid assertions | Native occurrence-window plus null-safe join/anti-join or verified upstream correction; no library-wide exclusion | Multi-column exact bag oracle, duplicate/null cases, partition permutations and support |
| **F02 — Function identity admission lacks an effects/reproducibility policy** | DM-28, DM-31, DM-32 | `crates/pse-catalog/src/session/functions.rs:68` checks retained calls without volatility; `snapshot_session.rs:286` omits captured ambient values; pinned random is Volatile and now Stable | A metadata-admitted ambient expression may change under unchanged reusable stage context; actual PSE exploit remains unprobed | Bind execution purpose and actual config/time/random/external observations; qualify effectful routes separately | Tagged ambient rule invocation; repeated/captured-input controls; unchanged-name changed implementation |
| **F03 — Physical semantic repair runs after the declared planning pipeline** | DM-22, DM-24, DM-50, DM-53 | `snapshot_session.rs:696` finalizes logical observation; `:705` invokes repair; `physical_fields.rs:41` replaces CrossJoinExec with NestedLoopJoinExec and wraps projections | Executed physical properties/transforms lack an observed qualified final pipeline; schema equality alone is insufficient | Registered named physical rule and subsequent requirement/invariant checks; retain necessary metadata correction | Full output/nested fields, distribution/order and final plan under multiple partitions |
| **F04 — Cancellation cannot wake a pending stream** | DM-30, DM-35 | `crates/pse-ids/src/resource.rs:256` only AtomicBool; `snapshot_session.rs:713` waits on source wake | Cancellation can stall indefinitely while pending I/O/stream retains resources | Wakeable cancellation integrated with polling and ownership | Source remains Pending; cancellation alone terminates and releases owners |
| **F05 — Library category bans encode initial non-use as architecture** | DM-19, DM-38, DM-43, DM-57, DM-58 | Baseline blueprint D10 and §3.1/§3.3, Plan 04 bounded-UNION paragraph, map Reject rows 50–65, `deny.toml`; baseline compression ban `clippy.toml:53` | Native transformations/extensions are precluded even when a use preserves authority and meaning | ADR-0065/revisions 34/36 policy and explicit dispositions; remove six companion bans; scope compression lint to the canonical identity path | Policy/doc checks; consumer-specific capability qualification |
| **F06 — Small profile disables useful native planning, not just optimization** | DM-26, DM-38, DM-43, DM-59 | `session/profile.rs:27` contains 2 analyzer/4 logical/3 physical rules, Wave 2 adds one physical rule; `snapshot_session.rs:226` replaces lists; pinned optimizer `:290` includes decorrelation/extraction/projection/CSE | A native API's availability does not give the PSE profile its required rewrites; ordinary plans miss optimization opportunities | Broad pinned candidate profile, explicit catalog/assembly, semantic qualification and narrow exceptions | Required plan routes plus complete result/error/metadata/support comparison; preparation and execution measurements |
| **F07 — Complete extension assembly stops short of custom bindings and TVFs** | DM-19, DM-43, DM-44 | `functions.rs:17` retains four families, no table functions; `admission.rs:188` only retained PSE providers; `snapshot_session.rs:176` captures defaults | Builtin-generated sources and future kernel/planner adapters have no complete admitted construction route | One pre-seal assembly path; invocation-bound TVF providers and generated operation adapters | Real range/member TVF, forged provider, custom function and codec controls |
| **F08 — Repeated full admission and row conversion defeats columnar preparation** | DM-07, DM-26, DM-36, DM-37 | `provider/table.rs:48`; `session/admission.rs:202,225`; generator `pse-schema/src/codegen/rust/relation.rs:171,180,195` | Repeated queries decode complete immutable values; builders construct a batch per push, then rebuild all rows | Immutable registry-bound validation ownership; generated borrowed columns and column builders; explicit Row boundary | Same invalid cases rejected; count scans/allocations; compare diagnostics and complete nested values |
| **F09 — Provider data work happens during planning before projection** | DM-26, DM-36, DM-38 | `provider/table.rs:95` evaluates full-batch filters, `:121` filters every column, `:138` finally projects | Narrow queries copy irrelevant wide/nested data and repeat work when replanned | Native source/filter/projection execution pipeline with required predicate columns | Exact-pushdown versus unpruned oracle including limit/nulls; phase-specific allocation/timing |
| **F10 — Public streaming collects and copies, then workspace admission copies again** | DM-29, DM-36, DM-37 | `snapshot_session.rs:710,722`; `pse-ids/src/owned_buffer.rs:177,199`; `snapshot_session.rs:397` | Consumers cannot release results incrementally; retained slices and repeated passes multiply memory work | Owned query stream and internal admitted-owner handoff, explicit compact copy where required | Backpressure, retained slice after session drop, last-owner release and peak memory |
| **F11 — Output key partitioning is not semi-naive input evaluation** | DM-26, DM-38, DM-39 | `pse-rules/src/plan/delta.rs:12` clones complete candidate plan into anti/semi joins; `strata/mod.rs:117,138,158` recompiles head/support each round | Full candidate computation and preparation recur even when few facts change | Prepared positive delta-input variants, retain conflict and alternate-support comparison | Chains/cycles/diamonds; actual rows scanned/planning counts; same complete assertions/support |
| **F12 — Some ordinary relational correspondence remains repeated Rust scans** | DM-25, DM-38, DM-56 | `pse-compiler/src/passes/p9/parameters.rs:40,58,81,248`; `quantity_relations/source/groups.rs:241` | Method×instance×member growth repeats decoded joins already represented as relations | Native joined descriptor inputs before quantity/graph algorithms; prepared typed indices only for justified bounded algorithm work | Independent dimensional scaling, exact outputs/support and inspected native joins |
| **F13 — Generous workflow memory does not remove the small relation envelope** | DM-36, DM-39, DM-43, DM-59 | ADR-0050/blueprint §5.3; `owned_buffer.rs:186` applies PHASE1 envelope separately from 32 GiB pool | Increasing the pool alone still rejects a large valid relation/result; hardware policy can be misrepresented | Explicitly characterize and revise the supported relation/encoding envelope when needed; do not silently change identity framing or impose arbitrary tiny production limits | Representative relation sizes and copies, explicit envelope rejection; separate normal workloads from stress tests |

### Applicability and principle verdicts

All twelve charter groups bear on these boundaries, but not every principle needs a
new independent finding. The following are the specifically assessed principles;
other IDs are not silently certified. Verdicts apply to the reviewed mechanisms,
not the whole repository. Proposed extensions with missing proof stay Unresolved.

| Principle IDs (each has the stated verdict) | Verdict | Basis |
|---|---|---|
| DM-02, DM-11, DM-12, DM-15, DM-20 | Satisfied in inspected scope | Actual immutable source/contract ownership, identity separation and read-only SQL; existing canonical/reuse contracts preserved |
| DM-03, DM-18, DM-25 | Satisfied in proposed placement | Distinct execution layouts and specialized MathIR/quantity algorithms remain legitimate; no forced universal Expr model |
| DM-07, DM-09 | Unresolved for broadened route | Existing source/value checks are real; additional invocation/generated-source contracts remain to implement |
| DM-08, DM-22, DM-24, DM-40, DM-42, DM-53 | Unresolved | New null/bag/field/precision transformations require qualification; F01/F03 expose concrete pitfalls |
| DM-19, DM-43, DM-44 | Unresolved | Policy now admits all features; actual capability selection/extension paths remain incomplete |
| DM-26, DM-36, DM-37, DM-38 | Violated by identified preparation/relational paths | F08–F12 repeat full decoding/planning or implement relational correspondence as repeated row scans; this is not a measured speed claim |
| DM-28, DM-31, DM-32, DM-33 | Unresolved for broader functions/reuse | Exact existing dependencies are useful; ambient invocation and new prepared/delta routes need proof |
| DM-14, DM-29, DM-30, DM-35 | Unresolved for new execution lifecycle | Atomic contract is preserved, but streaming/effectful cancellation and recovery are untested; F04 is concrete |
| DM-23, DM-46, DM-48, DM-50 | Unresolved for broader transforms | Exact support and final physical attribution need extension; plan encodings remain diagnostics |
| DM-41, DM-51, DM-52, DM-56, DM-57 | Unresolved | One declaration exists, but mechanical adapter/traversal work and narrow extension paths need correction |
| DM-39, DM-54, DM-58, DM-59, DM-60 | Satisfied for this review's evidence discipline | No performance certification; explicit counterexamples, simpler alternative, bounded priority and regression plan |

No weighted score is used. The category-ban defect has been corrected in policy;
that does not upgrade unresolved implementation gates.

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication / extension locality | Correctness and operational risk | Cost | Performance evidence | Decision |
|---|---|---|---|---|---|
| Current small RulePlan/profile plus growing Rust row work | Operator knowledge spread across validation, lowering, support and projection; repeated joins in consumers | Conservative refusal avoids some invalid uses but encourages manual alternatives and misses native planning | Growing coordination and repeated preparation | Source costs identified, no timings | Revise |
| Thin PSE semantic contracts with native plans and full eligible engine surface | One domain contract; generated adapters and actual bound implementations; native engine owns relational mechanisms | Requires effects, metadata, provenance and transformation qualification | Targeted extension assembly/profile/column work; avoids reimplementing a query engine | Expected benefit only; must measure end to end | Recommended target |
| Simpler viable first increment: broaden profile, add only consumed native operations and generated columns | Keep current scheduler and RulePlan, extend joins/windows/functions as needed; no general plugin/transport framework | Smaller migration surface; some traversal duplication remains explicit debt | Lowest route to Wave 2 completion | Compare to current at fixed semantics | Recommended delivery strategy toward target |
| Enable every feature and construct a generic extensible execution platform immediately | New registries, codecs and deployment machinery without consumers | Larger effect/ABI/state surface with little qualification | Highest; competes with product scope | No supporting workload | Reject as implementation strategy, while preserving eligibility |

**Justified machinery:** the existing registry, declared operation bindings, shared
session assembly, typed source/output admission and scheduler solve current meaning
and lifecycle needs. A new universal rule AST, second function registry, optimizer
framework, microservice or distributed scheduler does not follow from this review.

**Ordinary code:** graph traversal, exact quantity checking, semantic-ID framing,
parser mechanics, typed DAG substitution, numerical kernels and atomic store
protocols remain specialized. A small prepared index inside such an algorithm can
be simpler than a query; a repeated relational join over model inventories belongs
in DataFusion when it expresses the same declared relationship directly.

## 9. Verification and measurement plan

| Claim/risk | Evidence | Test/analysis | Conditions and expected result | Current result |
|---|---|---|---|---|
| Native capability behavior | Tested | `bash docs/design_review/evidence/full-arrow-datafusion-2026-09-14/run.sh` | Dev/force_validate, 1/4 target partitions, batch 2, baseline 0 | 33/37 pass; 4 raw ALL semantic failures; corrected native lowerings pass |
| Exact pinned universe/features | Interface-checked | `just metadata`; full locked offline Cargo metadata | Actual resolved packages/features, not manifest intentions | 49 family packages consistent; receipt retained |
| Broad engine semantic equivalence | Proposed | New catalog/rule integration cases under reference and full candidate profiles | Complete values/bags/fields/typed failures/support match independent small fixtures; inspect required rewrites | Not run |
| Set/null/truth/ordering | Proposed | Union/set/join/window/group/recursive metamorphic cases | Empty, all-null, unequal duplicates, ties, CASE inactive failure, decimal/overflow, dictionary permutations | Library subset only |
| Function/TVF/type/provider admission | Proposed | Bound real implementations plus same-name foreign and malformed output controls | Scalar/array/zero-row/null, wrong physical quantity, provider hidden in subquery, generated source bound to invocation | Not run |
| Effects and reuse | Proposed | Captured/uncaptured time/random/config/service input matrix | Reproducible rule refuses unbound effects; exact changed dependencies invalidate even if values/hash equal | Not run |
| Semi-naive and prepared execution | Proposed | Chain/cycle/diamond/multi-head/conflict/absence fixtures | Full assertions/support equal clean computation; no cached state leakage; delta scan work observable | Not run |
| Column and ownership preservation | Proposed | Row-versus-column value/diagnostic comparison; stream lifetime/backpressure | Same schema/values; only final owner releases charge; no unaccounted coexistence | Not run |
| Failure/recovery | Proposed | Permanently pending stream cancellation, spill failure, commit interruption/retry | Typed terminal failure, resources released, no partial success | Not run |
| Optimizer/storage pushdown | Proposed | Full-result oracle with pushdown/repartition/spill disabled | No dropped rows/support; truthful Exact/Inexact; invalid unread source cannot gain whole-snapshot admission | Not run |
| Performance and sizing | Proposed | Cold/warm real P3–P10 flowsheets and scalable synthetic relation families | Fixed semantic result, explicit hardware/profile/budget and source shape; report medians/distributions and peaks | No performance measurements |
| Documentation/policy | Tested within stated scope | ADR/index/docs/spelling/agent checks and focused governance | Baseline 0; preserve existing dirty source | See completion receipt |

Run appropriate existing recipes for changed production boundaries, then the Wave 2
terminal recipes in Plan 04; all Rust test invocations explicitly enable
`pse-relations/force-validate`. A focused library probe or compile is not a substitute
for P0–P10 golden output, derivations, invalidation matrix and reopened Python inspection.

**Cost accounting:** separately measure source capture/load, value admission, row/column
conversion, logical preparation, optimization/physical planning, execution, support
construction, canonicalization, transfer/copies and publication. Record actual
operator rows, allocations/bytes, retained buffer size, spill I/O, pool peak and
process peak. Compare 1, 16 and 32 execution threads with a shared 32 GiB ordinary
budget on the stated 16-core/32-thread, 192 GiB workstation. Tune from evidence;
this budget is a configurable starting point, not a limit on the architecture.
Use distinct tiny-budget tests for failure. Avoid `target-cpu=native` and changed
numerical policies when comparing performance. Vary instances, domains, members,
methods, columns and fan-out independently; tiny examples cannot qualify scaling.


### Completion receipt — 2026-09-14

- **Tested:** `just test-package pse-tests-governance -p pse-relations --test banned_patterns`,
  default nextest profile, explicit `pse-relations/force-validate`: **4 passed,
  0 failed/skipped, baseline 0**. This checks the narrowed compression policy and
  existing source-pattern rules. The compilation emitted **2 warnings** from the
  pre-existing Wave 2 work (`pse-mathir/src/canonical.rs:294` and
  `pse-schema/src/catalog/s6_6_templates.rs:369`, unnecessary qualification).
  Those remain part of Wave 2 cleanup; this is not a zero-warning full-workspace receipt.
- **Tested:** `just adr-lint`, 66 records and 31 register rows: **0 failures,
  baseline 0**. `just adr-index` regenerated the index before the final check.
- **Tested:** `just lint-typos` and `just lint-agents`: **0 failures, baseline 0**.
  Agent checking covered 20 files, 74 path references and 55 recipe references.
  The legitimate `flate2` identifier in captured Cargo metadata is explicitly recognized.
- **Tested:** `just docs`: **0 build errors, baseline 0**; **1 warning** about
  the 11,067,333-byte search index. Book build success is not semantic acceptance.
- **Tested:** targeted relative-link/section inspection of the four new Markdown
  artifacts checked 14 relative links and sections 1–11 in order: **0 failures,
  baseline 0**. `bash -n` on the probe runner and pinned Rust 1.98.1 `rustfmt --check`
  on the probe source both exited 0.
- The standalone library probe's **4 failed semantic expectations / baseline 0**
  remain recorded above. No full PSE runtime, numerical parity or performance gate
  is claimed. The six dependency-ban removals belong to concurrent ADR-0066;
  this review additionally removed the global IPC-compression Clippy ban and kept
  the prohibition scoped to canonical identity encoding.

## 10. Exceptions and unresolved decisions

No new SHOULD deviation or MUST waiver is accepted here. The broad architecture is
Proposed and the verdict is Revise. Existing immutable snapshots, pure kernels and
canonical encoding are placement contracts, not feature-family bans.

| Item | Owner | Resolution / revisit trigger |
|---|---|---|
| Formal ADR lifecycle | Maintainer | ADR-0065 decision PR with needs-review and blueprint revision; accepted ADR-0026 Arrow-only exclusion reconciled without rewriting its history |
| General native-plan binding shape | Rules/schema/catalog owners | Settle typed operation parameters, implementation binding and support contract in C1 before broad consumers; use existing structures where possible |
| Native recursion versus scheduler | Rules owner | Choose per declared truth/termination/support requirements; native finite distinct closure qualifies only when complete support is reconstructable |
| UDF effects policy | Catalog/kernel owners | Required before exposing ambient/config/effectful calls as reproducible rule execution |
| Compression lint scope | Governance/catalog owners | Corrected in C0: global Clippy ban removed; governance pattern limited to canonical encoding; no canonical-format change |
| Canonical relation envelope | Identity/runtime owners | Characterize real Wave 2 sizes/copies; expand versioned supported envelope if it obstructs normal workstation workloads, with ADR-0050/R-23 reconciliation |
| Optional companion crates/features | Consumer owner | Available now with no additional library admission process; test actual consumed behavior and preserve compatible pins |
| Library bug correction | Rules owner | Raw ALL builders remain unqualified for SQL bags at 55.1.0; use tested decomposition and extend conformance before platform support |

## 11. Decision and implementation changes

**Decision: Revise.** Adopt the full-capability policy and the simpler staged delivery
strategy. Complete the following in dependency order before declaring Wave 2 done.
The user requested a design review at this point; production remediation below is
planned, not silently reported as implemented.

| Priority / packet | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| **C0 — Policy and truthful checkpoint** | ADR-0065, blueprint revisions 34/36 and expanded D10, corrected Plan 04/map dispositions, companion-crate bans removed by concurrent ADR-0066; compression lint scoped to actual identity construction | DM-43, DM-57, DM-59 | Review and doc/policy checks; explicit distinction eligibility/exposure/qualification | No library or capability family is categorically precluded; document actual unsupported behavior and corrections |
| **C1 — Semantic execution binding** | One pre-seal function/planner/TVF/native-operation assembly and execution-purpose/effect contract; settle broad lowering contracts | DM-19, DM-22, DM-28, DM-44 | Real source/function positives and foreign/ambient negatives; direct dependency comparison | Generated/mechanical inventory and contract parity |
| **C2 — Correctness before broad optimization** | Correct bag ALL lowering, null-aware joins, registered physical-field repair, final invariants and physical observations; wakeable cancellation | DM-24, DM-30, DM-40, DM-50 | Counterexamples become passing PSE cases; permanently pending cancellation terminates | Exact bag/metadata/support and lifecycle suites |
| **C3 — Full pinned candidate pipelines** | Expand actual selectable analyzer/logical/physical rules; qualify full recommended order plus PSE brackets, not a new tiny hard ceiling | DM-26, DM-38, DM-43 | Correlated/native operator routes; full semantic/profile differentials; recorded exclusions only where evidence demands | Profile inventories and actual final plans |
| **C4 — Columnar preparation/ownership** | Generated borrowed columns/builders, immutable admitted validation ownership, execution-time source filtering, owned streams and internal handoff | DM-07, DM-26, DM-36, DM-37 | Same invalid-input failures and outputs; lifetime/backpressure; preparation/copy measurements | Negative admission and final-owner tests |
| **C5 — Wave 2 relational consumers** | Use richer native rules for P4–P10 relationship work; P6 tie policy, P8 descriptors, P9/P10 joins; prepare true eligible delta inputs/support | DM-18, DM-25, DM-31, DM-38, DM-46 | Exact full graph/support/diagnostics, incremental-versus-clean and new-consumer extension fixture | Real document-to-reopened-graph workflow |
| **C6 — Workload qualification** | Compare candidate/current preparation and execution; choose batch/partition/spill/settings and resolve practical envelope limits | DM-36, DM-39, DM-59 | Reproducible phase and end-to-end timings, memory, actual cardinality; same semantics | Benchmarks outside timing CI gates; normal/stress budgets separate |
| **C7 — Complete original Wave 2 exits** | Finish all remaining Plan 04 scope and terminal checks after corrections; update evidence/status honestly | DM-53, DM-54, DM-60 | Plan 04 terminal Rust/Python/golden/derivation/invalidation receipts, baseline 0 | Existing named project recipes |

C0's policy/document and compression-lint changes are part of this review delivery. C1–C7 remain
implementation scope. Optional FFI/Flight/Avro/Spark/remote/async deployments are
eligible without being mandatory Wave 2 deliverables. Their actual consumers may
pull them forward; none is precluded by the initial language or dependency profile.
