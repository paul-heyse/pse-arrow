---
status: proposed
reviewed: 2026-09-14
depth: deep
blueprint_revision: 36
adrs: [ADR-0039, ADR-0040, ADR-0041, ADR-0042, ADR-0044, ADR-0045, ADR-0046, ADR-0047, ADR-0048, ADR-0050, ADR-0052, ADR-0053, ADR-0055, ADR-0056, ADR-0062, ADR-0065, ADR-0066]
evidence: Interface-checked
---

# Design review: Wave 1 foundations built around native logical plans

## 1. Decision and scope

**Implementation follow-up:** the maintainer subsequently chose a hard pivot:
implement the target directly, delete misaligned legacy paths, and validate the
new code from first principles. [Plan 05](../../plans/05-native-logical-plan-hard-pivot.md)
is the complete execution plan. It replaces the incremental migration, legacy
comparison and preservation strategy described in this historical review, while
carrying forward its target architecture and required product scope.

**Decision: Revise the foundations before completing Wave 2.** Adopt native
DataFusion `LogicalPlan` and `Expr` as the common execution representation for
relational computation. Change the construction and ownership boundaries so that
established semantic properties survive each transformation. Remove repeated
validation and producer replay where those guarantees follow from construction.
Moving existing row validators into SQL without changing these boundaries would
leave the central problem intact.

**Proposed outcome:** a compiler can explain an output's type, identity, reference
domain, cardinality requirements, inputs and permitted effects from the actual
prepared computation. Internal results retain that explanation and their immutable
owners. Only properties not established by inputs or operator construction require
value-dependent work, performed before the result is admitted for its intended use.

Arrow remains the default for typed data and columnar operations; DataFusion remains
the default for data transformation, planning and execution across the system.
Other libraries are acceptable when they offer a distinctive advantage. This review
introduces no library admission list, feature ceiling or certification of unused APIs.

**Reviewer:** Codex, with three read-only investigations of schema/rules,
catalog/runtime and authoring/compiler foundations. Root inspected the code cited
below and ran a separate pinned construction probe.

**Affected revisions:** blueprint revision 36; branch `wave2/semantic-compilation`,
base commit `eebc82e9f2298e4a53e08818cfd2a1dfb6fc310b`, plus substantial uncommitted
Wave 1/Wave 2 and dependency-policy work present on 2026-09-14. A base commit alone
does not reproduce this reviewed working tree. This review changes no production
implementation, blueprint or accepted ADR.

### Method and coverage

- Read [Plan 03](../../plans/03-wave-1-foundations.md), the current
  [Plan 04 checkpoint](../../plans/04-wave-2-semantic-compilation.md), blueprint
  D1–D14 and the relevant schema, identity, compiler, artifact, authoring and
  verification sections; read the related ADR decisions and the design charter.
  The [preceding capability review](design_review_full-arrow-datafusion-capabilities_2026-09-14.md)
  remains the detailed inventory of engine features and Wave 2 execution gaps.
- Used the supplied [logical planning capability specification](../../capability-maps/datafusion_logical_planning_capability_spec.md)
  after `just lib-outline`, including its construction, metadata, extension,
  invariant, dependency and execution qualifications. The follow-up examines
  foundational changes that remove causes of the previous G2–G7 gaps.
- Inspected registry-derived invariants, generated views/builders, migration,
  relation validation, catalog admission/provider/session boundaries, pass outputs
  and producer replay, change staging, rename, source coupling, memo context and
  local publication. All nine Wave 1 interface areas receive a disposition in §4.
  This is a fundamental architecture review, not a line-by-line audit of all
  numerical algorithms, every authoring grammar rule or every P3–P10 producer.
- Used Context7 `/apache/datafusion` for discovery, then pinned source and
  [55.1.0 rustdoc](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/logical_plan/enum.LogicalPlan.html)
  to check the actual interfaces. Main-branch examples are not pin evidence.
  Exact versions remain DataFusion 55.1.0 / Arrow 59.3.0. The prior feature inventory
  is retained; the new probe independently resolves its dependencies from the
  current workspace declarations and verifies every third-party name/version
  against the current lockfile before offline, locked execution.
- **Tested:** [new construction characterization](../evidence/wave1-logical-plans-2026-09-14/README.md),
  `bash docs/design_review/evidence/wave1-logical-plans-2026-09-14/run.sh`, Rust
  1.98.1, dev, Arrow `force_validate`: **8 passed, 0 failed, baseline 0**.
  Six checks exercise construction/contract behavior, with no data for those
  plans; two execute small native diagnostic queries. A domain-aware function
  rejects a wrong semantic domain with the same storage type before its body is
  invoked. This is library characterization, not implementation of the redesign.
- Assume Wave 1 passing for planning, as requested. Do not reopen its broad test
  campaign. No full workspace, Python, parity or performance suite was run here.
  Startup `just doctor` reports one blocking outdated-environment issue and an
  extension warning; refreshing the extension belongs to implementation work.
  Documentation validation is recorded with the probe receipt.

**Evidence boundary:** the target architecture is **Proposed**; inspected library
and repository seams are **Interface-checked**. A small successful probe does not
make the architecture **Formally established**. The compositional arguments below
state precisely what can follow from established premises and trusted operator
semantics. They are design proof sketches, not a machine-checked proof of PSE or
DataFusion.

## 2. Authority and lifecycle map

| Concept | Semantic identity | Authority / owner | Revision boundary | Permitted update | Derived representation |
|---|---|---|---|---|---|
| Engineering declarations | Existing semantic IDs, relation/column contracts | Registry and authored/reference relations | Registry/package/model revision | Declared authoring or migration | Arrow fields, domain-aware expressions, input requirements |
| Relational computation | Existing rule/pass/operation ID and bound roles | Authored rule/transformation declaration | Declaration and binding revision | Compile the declaration into native nodes | Native `LogicalPlan`; no second executable algebra |
| Established input property | Property over exact fields and immutable relation owners | Private admission/construction result | Exact input and semantic context | Admission or sound derivation | Native constraints/FDs where representable; derived PSE property facts elsewhere |
| Quantity and math meaning | Existing quantity/operator/template identities | Quantity registry and MathIR contracts | Exact registry and graph | Declared physical conversion or MathIR lowering | Vectorized adapters and relational graph transforms |
| Prepared computation | Actual plan, bindings, parameter values, function implementations | Frozen preparation object | Input, declaration and configuration closure | New preparation or explicit rebinding | Optimized logical plan, physical plan and diagnostic views |
| Execution result | Output port, attempt, retained preparation | Executor-owned arrays and leases | One execution completion | Private execution transition | Admitted result and later immutable artifact |
| Publication | Destination and exact expected ref state | Existing catalog commit protocol | Conditional ref transition | Complete immutable writes then conditional publication | Visible revision or explicit conflict/uncertain outcome |

The property facts above are **derived facts about a computation**, not an
independently authored source of model meaning. Extend existing `ColumnSpec`,
`RelationSpec`, `PassSpec` and rule declarations only where a semantic requirement
is currently missing. Generate metadata, diagnostics and bindings from them.
Do not create a parallel configuration language for the same requirements.

The schema registry remains usable without starting DataFusion or depending on
the compiler. Registry declarations and code generation stay in `pse-schema`;
runtime lowering and composition belong in existing `pse-rules`, catalog and
compiler modules. Removing the small RulePlan as a runtime ceiling does not require
making the schema bootstrap depend on live engine sessions.

**Deliberately specialized mechanisms:** grammar parsing and source editing,
identity allocation, canonical encoding, quantity algebra, MathIR canonicalization,
numerical kernels, buffer ownership and durable publication retain ordinary Rust
implementations. Their input/output and effect contracts integrate with the same
preparation flow. A custom logical node around arbitrary code makes the boundary
visible; it does not make that code's internals relational or prove its behavior.

**Identity:** names and aliases are labels; semantic identity survives a rename
under the existing explicit/named identity policies. Ordinals remain scoped to a
particular target artifact and ordering. New allocation, conversion or hashing is
not presumed injective. Hashes continue to locate artifacts and check encoded
integrity under blueprint D4/§5.3; neither a digest nor serialized plan bytes
establishes validity, source equivalence or permission to reuse an output.

## 3. Semantic contracts and invariants

### The construction guarantee

Use a small private set of properties: semantic field types, nullability,
value-domain refinements, unique keys, reference inclusion, coverage, ordering and
finite-domain bounds. Each fact names the exact input/field scope on which it
depends. Native schema/FD machinery supplies the properties it actually supports;
PSE adds domain-specific derivations, not a replacement type-coercion engine.

The reasoning is inductive: admitted leaves establish facts about actual immutable
inputs; a checked operator derives only facts justified by its semantics and its
inputs; the next operator consumes those facts. If a needed premise is unknown,
the constructor attaches its specific unresolved obligation or rejects that use.
Unknown does not mean false, and an unproved property does not prohibit an
otherwise valid native operation. It prevents a consumer from assuming that
property without establishing it.

| Construction | Derivable guarantee | Premises and limits |
|---|---|---|
| Scan admitted source | Exact field/domain and admitted relational properties | Same immutable owner and admission context; externally mutable Arrow backing memory requires ownership/copy isolation first |
| Direct projection / alias | Selected field meaning and refinements; retained complete keys remain unique | Field correspondence is structural; arbitrary metadata assignment is not a semantic conversion |
| Filter | Input keys and row-local properties survive; supported predicates refine nullability/ranges | Coverage usually does not survive; retain any FK target owner to which inclusion refers |
| Total lookup | One result for each covered left row, with correctly bound right values | Right key unique; every covered left key nonnull and included in that exact right relation under ordinary equality, or an explicit null-safe inclusion/output contract; matching equality policy |
| Left join | Left coverage and at-most-one match under right uniqueness | Right fields become nullable for unmatched rows; additional predicates must not silently turn it into an inner join |
| Semi/anti join | Subset of left input, so left uniqueness survives | Negative evidence requires a complete right scope; absence over a partial recursive relation is insufficient |
| Union all | Compatible field meaning and common row-local properties | Uniqueness needs disjoint branch key domains or a new obligation; branch-local uniqueness is insufficient |
| Aggregate | One row per complete group identity under actual grouping equality | Grouping sets need a distinguishing grouping identity; nonnullness, overflow, finite sums and floating-point reproducibility are separate |
| Window / unnest | Derived field types, frames/ordering, parent-child correspondence | Ranking ties need declared tie behavior; unnest multiplies parents and needs element ordinals if child identity depends on position |
| Conversion / UDF | Declared semantic output derived from input fields and bound parameters | Implementation must realize its contract; narrowing, division and physical conversion can require value checks |
| Recursive closure | Domain membership and supported positive invariants | Termination follows only for a finite domain with monotone accumulation; support closure/conflicts are additional contracts |

For example, if `R.k` is nonnull, `S.k` is unique, and every `R.k` occurs in `S.k`,
then the inner equality join `R ⋈ S` has exactly one match per row of `R`.
Its direct projections preserve the meanings of their source fields. If `R.id`
was unique and remains projected, that output identity remains unique. There is
no reason to recount output keys or replay the join after every such enrichment.
The premises must concern the actual matching domain and equality semantics;
three independent Boolean flags would not establish the argument.

Conversely, a filter on `S`, a changed conversion of `R.k`, an additional join
predicate or an outer-join rewrite can remove a premise. Construction must drop
the affected guarantee or produce the needed obligation. Complete right-key
membership does not prove coverage after an unrelated right-side filter.

### Earliest establishment, rather than blanket retrospective checks

| Contract | Earliest sound boundary | Required failure behavior | Evidence |
|---|---|---|---|
| Names, arity, native type compatibility | Checked builders and native analysis | Planning diagnostic before execution | Native interfaces; new probe |
| Quantity/domain/ordinal target coupling | Generated semantic constructors and field-aware functions/analyzer | Reject incompatible binding before execution | Existing registry fields; prototype probe, full derivation **Proposed** |
| Imported value validity, uniqueness and references | One admission computation over the actual input | Structured violations; no promotion to assumed-valid source | Existing checks; consolidation **Proposed** |
| Locally preserved properties | Operator construction from established input facts | Inapplicable derivation cannot mint a guarantee | **Proposed** transfer rules above |
| New value-dependent facts | Checked kernel or specific diagnostic branch before dependent use | Error/violation is an explicit result, never discarded implicitly | Native predicates/anti joins **Tested** in isolation |
| Complete output inventory and completed obligations | Executor completion transition | Partial stream or failed branch cannot become admitted output | New immutable completion boundary **Proposed** |
| Publication visibility/durability | Existing conditional ref and backend protocol | Conflict, failed preparation, or visible-but-durability-uncertain outcome | Existing local transition **Interface-checked** |

Candidate scans must **not advertise the constraint being checked**. Otherwise an
optimizer can use an assumed uniqueness or inclusion fact to simplify the very
diagnostic intended to establish it. Current candidate providers already avoid
this circularity (`crates/pse-catalog/src/session/admission.rs:204`). Preserve that
distinction when replacing the imperative validator with generated plans.

For unresolved checks, use `IS NOT TRUE(predicate)` when both false and unknown
are invalid; plain `NOT predicate` loses null witnesses. Nullable references
follow the declared absent-reference policy. Composite references include their
whole key, context and null policy. Nested validation respects null parent masks.
Violation plans retain source row/occurrence identity and reason, not just counts.
Rejecting input and deliberately selecting its valid subset are different declared
operations. A diagnostic must not silently convert one into the other.

**Equality and absence:** distinguish undeclared port, declared absent input,
present empty relation, SQL null, rule unknown, invalid value and failed execution.
Declare bag/set semantics, key equality, semantic quantity equality, canonical
float/NaN/signed-zero policy and numerical tolerance where relevant. Native SQL
equality is not automatically canonical byte equality. `DISTINCT` cannot be used
to conceal a duplicate-key violation. The previous pinned ALL-set counterexamples
remain applicable; use corrected native multiplicity formulations for the affected
operations, not those shortcuts as an equality oracle.

## 4. Derivation and execution design

### One native plan, preserved through an owned preparation

```text
registry declarations + typed frontend + exact input owners
                         |
       native LogicalPlan / Expr + semantic derivation
                         |
   bound and analyzed computation + unresolved obligations
                         |
     optimized logical plan -> physical plan -> execution
                         |
     private result buffers + completion + established facts
                         |
       admitted output -> existing publication protocol
```

Do not mirror the DataFusion node/Expr enumeration in another runtime IR. Existing
declarative rule syntax may remain a frontend; SQL, DataFrame and custom domain
frontends converge on the same native plan and semantic binding path. A small
semantic analyzer and checked constructor library should extend native planning
where PSE meaning needs it. They should not independently redo every native
expression's typing or maintain a second optimizer.

| State / operation | Bound dependencies | Output and ownership | What is established |
|---|---|---|---|
| Input admission | Exact declaration, buffers/source bytes, target contexts | Existing immutable loaded relation/snapshot, or explicitly limited candidate | Only actual admitted properties; candidate integrity facts are not invented |
| Preparation | Original native plan, resolved fields, actual source owners, parameter values, native-operation reads, function objects, semantic settings | Private immutable prepared computation | Structural and semantic composition; outputs and pending obligations fixed |
| Analysis/optimization | Same input closure; selected native/analyzer/optimizer implementations | Analyzed and optimized plans; mechanically derived output field mapping | Required semantic interface preserved; no scanning of input values to rediscover established facts |
| Execution | Prepared computation plus fresh task/cancellation/resource state | Executor-owned streams and completion result | Values computed by the bound implementation; residual checks completed before promotion |
| Admission of local result | Completion and retained input context | Immutable admitted result handle, not a caller-assembled batch map | Established output properties and exact producer/source correspondence |
| Publication | Complete output inventory, finished objects, expected ref state | Existing manifest/ref result | Visibility and durability according to the actual backend protocol |

Extend existing capabilities rather than adding typestates for every invariant.
Illustrative `PreparedComputation` and `OwnedResult` names denote missing
responsibilities, not a mandate for new crates or a general certificate service.
The existing private `CompletedProgram`
(`crates/pse-rules/src/strata/completed.rs:17`) retains results and their original
context, and `ConstructedInput`
(`crates/pse-rules/src/strata/constructed.rs:138`) retains a construction recipe.
Consolidate and extend those mechanisms; replace their remaining replay-based
local admission rather than layering another competing receipt system over them.
Expose borrowed typed plans for inspection, but no mutation after preparation.
Keep physical operators fresh/reset as their contracts require for each execution;
retaining a logical preparation does not imply that a stateful physical operator
can be executed twice safely.

The current catalog's `admission: Arc<()>` identifies catalog lineage but not a
frozen semantic policy: `with_semantic_validator` replaces the validator without
changing that marker (`crates/pse-catalog/src/store/open.rs:54`, `:209`). Before
using it to eliminate validation, freeze configuration before minting handles or
replace the marker with an immutable context owning the actual registry and
validator/policy implementations. Old handles remain valid under their old
context; they do not gain new guarantees from a later validator installation.

Retain actual source/function owners. Current inner-`Arc` function comparison
(`crates/pse-catalog/src/session/functions.rs:32`) is a useful existing mechanism.
Identity alone does not freeze mutable captures: semantic configuration, clocks,
randomness, external reads and captures must be bound or explicitly effectful.
Pure kernel entry points receive their semantic inputs explicitly. Stable functions
need a bound query-time context for reproducible reuse; volatile evaluation follows
the chosen execution contract. Use constant folding only when that contract permits
planning-time invocation. This is behavior selection, not a ban on function families.

Preserve input role/port identity separately from schema identity. The current
`InputBundle::rows` flattens to relation keys and refuses the same relation through
multiple ports (`crates/pse-compiler/src/passes/bundle.rs:126`). Native scans can
bind before/after revisions of the same relation under distinct aliases and exact
owners. This removes an unnecessary composition restriction without inventing
new relation schemas just to distinguish input roles.

### Complete foundations disposition

| Wave 1 interface / subsystem | Retain | Re-engineer around native planning | Retire after replacement |
|---|---|---|---|
| I1 `pse-ids` | Semantic/artifact/ordinal distinctions, canonical codec, owned buffer leases | Bind ordinal target context and expose registry-derived identity/reference operations to plans | Hash-as-validation shortcuts, if introduced; no removal of legitimate integrity hashing |
| I2 `pse-schema` | Single declarations, generator/bootstrap independence | Generate semantic fields, column accessors/builders, invariant plans and constructor requirements from the same contracts; add composite/contextual references where real consumers require them | Independent runtime-like expression typing and separately authored copies of invariants |
| I3 `pse-relations` | Arrow layout admission, fallible extension construction, low-level codec boundary | Column builders and borrowed typed views; batch predicates; migration projections/literals/aliases | One-row-batch-per-push; repeated admitted-view validation; Cell-based migration and duplicate PK/FK engine |
| I4 `pse-catalog` | Immutable snapshots, exact source binding, manifests, conditional publication | Frozen admission context, prepared plans and owned results; native runtime filtering/projection; inherited source properties | Whole-value rescans on every scan admission and unconditional result-to-workspace rebuilding |
| I5 `pse-runtime` | Shared runtime, configurable budget, reservations, cancellation, attempt state | Execution owns native streams and leases; physical observations correspond to executed operators; wakeable cancellation | Forced collect/copy between routine operators; arbitrary restrictive default relation ceilings |
| I6 `pse-quantity` / `pse-material` | Registered physical distinctions, exact scalar algebra and material contracts | Relational lookup/composition of quantity definitions, batch conversion/check adapters, contextual domain joins | Repeated full-registry/full-row reconstruction per operation |
| I7 `pse-mathir` | D6 equation/operator semantics, exact canonicalization and specialized graph algorithms | Native joins/aggregates/order/unnest for indexed domains, graph relations, binding and provenance; explicit batch boundaries for specialized kernels | Handwritten relational matching and grouping around those kernels |
| I8 `pse-authoring` | Parser/AST edits, stable identity rules, exact source spans, sequential external change semantics | Parse once to bound Arrow relations; name/reference joins; keyed diff; source/row coupling retained by construction; relational migration | Verification-only construction of whole ChangeSets and repeated internal decode/reparse/rebind chains where unchanged owned artifacts already establish correspondence |
| I9 `pse-rules` / `pse-compiler` | Pass/port declarations, truth/conflict/support semantics, immutable stage boundaries | Native plan preparation, property transfer, outputs/findings/support from bound computations; relational delta state and downstream P3–P10 transforms | Public arbitrary batch output as the trusted boundary; duplicate producer roster/replay for local outputs; miniature engine ceiling |

This is a substantial redesign of common boundaries, not wholesale replacement of
working domain algorithms. Implement the shared preparation and ownership change
once, then move consumers through it. The existing D6/D11 numerical layout choices
remain useful and are compatible with plan-visible batch inputs and outputs.

### Specific replacements and their construction contracts

**Integrity rules.** PK/FK algorithms already exist twice:
`crates/pse-schema/src/catalog/invariants.rs:9` generates aggregate/anti-join
violation rules; `crates/pse-relations/src/validate/bundle.rs:17` decodes whole
batches and independently checks keys/references with sets. Make the generated
obligation the sole implementation of each relational invariant. Broaden its
declaration coverage where necessary before deleting the duplicate. Physical Arrow
layout safety stays at ingestion; it cannot depend on executing a query over
malformed arrays.

**Generated access.** `crates/pse-schema/src/codegen/rust/relation.rs:171` validates
a view, while `rows()` decodes through another validation path at `:181`.
`Builder::push` makes an admitted one-row batch at `:195`, then finish rebuilds the
whole relation at `:210`. Generate direct Arrow column builders and admitted
column views. Scalar constructors check their actual extension values once;
finish preserves construction state. Row decoding remains an explicit convenience
boundary, not the mandatory representation for internal transformations.

**Migration.** `crates/pse-relations/src/migrate.rs:36` decodes and mutates rows for
projection/default/rename/drop. Compile those declared steps to projections,
typed literals and aliases. Widening nullability needs no data scan; tightening
requires an established nonnull refinement or a specific null diagnostic. A
quantity conversion is an explicit operation, not a metadata rename.

**Source coupling and changes.** `crates/pse-authoring/src/change_set/stage.rs:157`
implements keyed diff in Rust maps, and `:114` builds one batch per staged member.
Use full outer joins with explicit side-presence fields and exact before/after
comparison. Absence must not be inferred from a nullable payload. Batch operations
by relation while retaining operation identity/order. External sequential change
sets must still check each declared before-image; collapsing repeated edits into
last-write-wins would change their meaning.

Rename currently edits actual source text, stages/applies it, decodes and rebinds
the result, and captures another proof
(`crates/pse-authoring/src/document/rename/owned.rs:69`). Preserve actual parsing
of changed bytes and binding-aware source edits. Reuse unchanged immutable parsed
documents and derive changed rows/bindings from the one updated parse. The commit
path currently constructs another complete change set just to require no ops
(`crates/pse-compiler/src/driver/commit/prepare.rs:78`). A locally owned
source-derived completion can replace this replay. Imported source/row pairs still
require actual correspondence; a source digest or claim of a rename is insufficient.

**Pass outputs.** `Pass::run` returns publicly assembled `PassOutput.ports`
(`crates/pse-compiler/src/passes/mod.rs:51`, `:177`). Admission dispatches a second
producer roster and executes that producer again
(`crates/pse-compiler/src/validator.rs:70`, `:112`). Replace the trusted boundary
with executor-minted outputs from prepared plans and declared native steps. Ordinary
callers can submit raw candidates but cannot manufacture completed provenance or
property evidence. Registration binds the implementation and declared outputs
once. Imported/reopened derived results retain semantic admission and, where the
contract requires producer correspondence, recomputation or equivalent actual
semantic evidence. Do not remove those checks merely because local replay becomes
unnecessary.

**Planning and execution.** Provider `scan_plan` currently evaluates filters over
the full batch during physical planning
(`crates/pse-catalog/src/provider/table.rs:95`). Return native execution nodes and
let the engine perform that data work during execution. Capture metadata needed
for planning explicitly. The current session analyzes/optimizes then collects to
`Vec<RecordBatch>` (`crates/pse-catalog/src/session/snapshot_session.rs:647`).
Retain prepared plans and return owned streams/completions. Carry semantic fields
through checked projections and physical lowering; do not repair meaning after an
unobserved execution. Lightweight structural checks remain appropriate; full value
rescans are not the means of preserving an already established field derivation.

### Composition across passes, recursion and materialization

Compose plans across purely internal pass fragments so pruning, filtering, joins
and expression optimization can see the work together. Preserve declared output
ports and source-to-pass diagnostics even when the optimizer fuses work. Do not
silently remove blueprint D3/§14.1 immutable stage boundaries or change their
identity/reuse behavior. Treat persisted stage outputs, expensive shared results,
repeated iterations, external algorithm boundaries and explicit checkpoints as
deliberate materializations. Several plans may share one preparation/context;
requiring one enormous plan for the entire lifecycle would not improve correctness.

Shared CTEs and repeated `Arc<LogicalPlan>` references do not promise execution
once. If data output and diagnostic branches reuse an expensive or volatile
computation, explicitly materialize its owned result or use an execution mechanism
that guarantees the intended sharing. Do not rerun a nondeterministic expression
and compare its second result with the first. Do not expose a stream as admitted
while its global obligations are still incomplete. It may be a private candidate
stream until successful completion; already admitted source inspection can stream
directly.

Represent fixed-point accumulated rows, new assertions, conflicts and support as
Arrow relations. Native joins/grouping/differences replace row maps and literal
string matching. Keep a small iteration driver for convergence, cancellation and
stratum transitions. A finite monotone truth domain can provide a termination
argument; rules that invent values need a different explicit termination contract.
Negative dependencies require a settled lower stratum. DataFusion recursion is an
available implementation route, not an automatic substitute for PSE conflict and
support semantics. Input-delta plans must actually restrict changed inputs; merely
partitioning output keys does not make execution incremental.

Truth identity and support identity are different. A distinct truth row may have
several independent derivations. Generate support from the same bound expressions
and relationships, retaining required multiplicity through optimization. Do not
assume ordinary optimizer row lineage equals complete PSE rule justification.

### Dependencies and reuse derived during construction

Capture dependencies while resolving and constructing the unoptimized/analyzed
computation, including subqueries, correlated references, parameters, native
operation reads, defaults, candidate inventories, negative scopes, functions and
semantic settings. Traverse actual typed nodes, not EXPLAIN text. Retain facts
consumed by binding and simplification even when their nodes disappear. For
example, a unit lookup folded into a literal still depends on that unit definition;
a previously empty candidate set can change method selection after an insertion.
Inspecting only optimized table scans misses both kinds of dependency.

Use exact shared immutable contexts first; existing whole-stage dependency scope
is a sound starting point under ADR-0041/0042. Current memo context copies registry
rows, source bytes and policy values per stage
(`crates/pse-compiler/src/memo/context.rs:27`). Shared owners remove copying without
requiring fine-grained invalidation immediately. Narrow reuse only where complete
dependency derivation exists. Hashes may index candidates; retained exact inputs
and context determine reuse. Cross-process reopening re-establishes these bindings.

### Publication and resource ownership

Keep the existing small commit state machine. A prepared result can establish the
complete intended content before writes; a conditional ref update determines
whether it becomes current. Concurrent changes yield conflict. Lost responses or
post-publication failures require the existing explicit outcome/reconciliation
protocol. The local implementation already distinguishes a visible replacement
from unconfirmed durability (`crates/pse-catalog/src/store/local.rs:198`). This is
a constructive G5 mechanism worth retaining, not another after-the-fact validator.

Do not place commits/fsync/ref updates in scalar UDFs. Represent their dependencies
and effects in the host operation contract consuming the prepared result. Native
DML/Copy/extensions remain available where useful; transaction syntax does not
supply this catalog's atomicity or durable recovery.

Keep generous configurable resource settings suitable for the stated 16-core,
32-thread, 192-GiB workstation. Preserve the current 32-GiB workflow-fixture budget
as a starting point, not a target to squeeze into or a universal production limit.
Production runtime budgets remain explicitly configurable. Revisit independent
relation/object ceilings that can still reject larger work despite that budget.
Separate supported size from canonical byte identity when resolving ADR-0050's
envelope. Continue accountable buffer ownership and explicit spill/lifetime
management. Removing duplicated validation, decoding and intermediate copies
should reduce work; no measurement here establishes globally optimal performance
or a hard bound on all process allocations.

The runtime still rejects non-default spill compression and timezone settings
under its phase-0 contract (`crates/pse-runtime/src/budget.rs:82`). Replace fixed
phase-based restrictions with typed selected settings and the computation's actual
semantic requirements as the native preparation path lands. Preserve a canonical
math policy where required; do not turn that policy into a library-wide ban on
other execution configurations.

## 5. Representative journeys

### Ordinary extension: a quantity-aware total lookup

Declare a reference mapping and an operation that enriches each observation with
one matching physical definition. Registry-derived field handles express semantic
quantity and reference domain. The constructor binds the exact reference owner,
requires uniqueness and coverage, and builds a native join/projection. Already
established premises avoid output validation scans. Imported mappings first run
their unresolved duplicate/missing-reference plans. A new field-aware conversion
function is needed only when existing native functions and quantity adapters do
not express the operation. Metadata, requirements and diagnostics come from the
same declaration; adding the feature does not require editing a second producer
roster or writing another row validator.

Adversarial case: two reference rows match the same key. The mapping never acquires
the unique-key property; total lookup cannot silently multiply outputs. Another
case: a key-preserving projection is followed by `UNION ALL` of overlapping
branches. The constructor loses uniqueness rather than carrying it incorrectly.

### Meaningful change: rename and a changed reference definition

An explicit-ID rename edits the actual relevant source spans and parses changed
documents once. Binding joins use stable semantic identity, so name changes do not
change entity identity or redirect same-spelling references in another scope.
The source-derived completion ties changed text, occurrences and rows together.
The exact expected base remains part of commit preparation. An imported changeset
with a stale before-image fails before publication.

A subsequent physical definition change creates a new registry/reference context.
Existing prepared work remains pinned to its old context; new work rebinds.
Even when a lookup disappeared through constant folding, its captured dependency
forces the appropriate new preparation. The same-name/same-signature replacement
of a function cannot inherit the previous implementation's prepared result.

### Boundary: Python, persistence and native MathIR

An admitted Arrow stream keeps its source/result owners and leases through the
Python export. Borrowing external memory requires an actual immutability/lifetime
contract; an `Arc` wrapper alone does not provide it. Reimported arrays enter the
candidate path and establish their real value/metadata obligations. Reopened
artifacts resolve their exact contexts and admit actual content; serialized plans
remain diagnostic/rebindable representations rather than restored validity tokens.

MathIR graph binding and indexed realization use native relational plans for
domain expansion, joins and aggregation. The specialized canonicalizer consumes
typed batches and returns a privately completed typed graph result. Its D6
operator and numerical policies remain authority. A valid DataFusion expression
cannot silently substitute SQL arithmetic for guarded physical equation semantics.

### Interruption: late violation and uncertain publication

A query yields several candidate batches, then a duplicate-key diagnostic finds
a violation or the stream fails. No admitted completion is minted and no current
ref changes. Private buffers release their leases. A cancelled pending stream must
be woken through the execution cancellation mechanism; plan construction cannot
make an atomic flag wake stalled I/O.

If all computation succeeds but conditional publication conflicts, the complete
immutable result may remain reusable under its pinned inputs; it is not the new
current revision. If replacement occurred but final durability confirmation fails,
report the visible/uncertain outcome and reconcile exact ref state before retry.
These transitions are explicit mechanisms, not inferred from a successful query.

## 6. Acceptance gates

Gate verdicts concern the proposed fundamental pivot and its current implementation
coverage. Wave 1's assumed historical test result is not being rescored as a failed
product release. **G1 is Pass for the authority direction; G2–G7 are Unresolved for
the redesigned guarantee until the named construction and lifecycle paths exist.**

| Gate | Verdict | Evidence and proposed resolution | Required action |
|---|---|---|---|
| G1 — Authority | Pass, scoped | Existing registry/relations retain meaning; native plans are compiled representations; property facts are derived, not new authored truth | Reconcile decision documents before changing contracts; keep accepted ADRs immutable |
| G2 — Semantic fidelity | Unresolved | Semantic fields, quantity/reference domains, multiplicity and NULL contracts can be derived through checked construction; current output repair and metadata alone are incomplete | Implement field-aware derivation and loss-aware adapters for actual transformations |
| G3 — Validity | Unresolved | Admitted inputs plus sound transfers eliminate repeated checks; generated diagnostics establish remaining input/value properties | Close raw-result construction boundary; make completed obligations a prerequisite for dependent guarantees |
| G4 — Hidden behavior | Unresolved | Frozen function/configuration/source bindings and plan-visible computation expose intended behavior; current provider planning performs bulk evaluation | Freeze preparation inputs/effects and move data execution into execution nodes |
| G5 — Consistency and recovery | Unresolved | Retain existing explicit commit protocol; owned completion prevents partial outputs being admitted | Connect execution/obligation completion to publication; preserve cancellation, conflict and uncertain-outcome transitions |
| G6 — Transformation and reuse | Unresolved | Native plans plus derived contracts and exact dependency closure can justify compatible reuse | Preserve facts through rewrites, retain binding-time dependencies, eliminate only checks discharged by that construction |
| G7 — Truthful capability claims | Unresolved | Broad library access and exact prepared implementations replace capability-name assertions | Bind real lowering/execution routes and explicit unsupported outcomes; do not call plan well-formedness universal proof or optimality |

## 7. Principle findings

**Applicability:** all twelve groups bear on this cross-cutting foundation review,
but only applicable individual principles are assessed below. Detailed solver
convergence, distributed execution and eventual Pyomo numerical equivalence were
not inspected. They receive no implied satisfied verdict. These findings distinguish
observed current mechanisms from proposed replacements; unresolved is not pass.

| Finding | Principle verdict | Concrete evidence / gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|
| F1: Internal results discard their construction guarantees | DM-07, DM-22, DM-26, DM-44 — Unresolved | Public `PassOutput` and duplicate producer replay, §4 | Correctness is recovered by repeating computation; extension semantics split across rosters | Private prepared/result completion consumed by admission | Cannot manufacture completion from altered batches; local producer runs once |
| F2: Declared relational invariants have two engines | DM-17, DM-52, DM-56 — Violated | Registry Aggregate/AntiJoin rules and Cell/set PK/FK loops, §4 | Two algorithms must evolve identically; avoidable row decoding | One generated relational obligation per meaning | Missing/duplicate/null/composite cases preserve exact diagnostics |
| F3: Semantic properties are not sufficiently carried through native planning | DM-06, DM-09, DM-24, DM-42 — Unresolved | `ColumnSpec` declares meaning; rule outputs reconstruct fields and validate arrays at `crates/pse-rules/src/exec/mod.rs:134` | Same physical type can disguise incompatible roles; blanket scans stand in for derivation | Field-aware construction and explicit transfer rules over exact field identities | Wrong quantity/domain fails before execution; aliases preserve meaning |
| F4: Catalog marker is not an immutable semantic context | DM-28, DM-31, DM-32 — Unresolved | `open.rs:54`, `:209`; current rescans partly compensate | Dropping checks with that marker alone would permit invalid property reuse | Freeze context or create a new owner on semantic changes | Old handles retain old guarantees only |
| F5: A scan's preparation executes its data predicate | DM-26, DM-38 — Violated; DM-20, DM-28 — Unresolved for broader effects | `provider/table.rs:95` evaluates/allocates before native execution | Planning cost and execution behavior are mixed | Return native execution operators; bind preparation metadata | Planning consumes no source rows for this in-memory provider |
| F6: Generated access and migration repeatedly reconstruct rows | DM-37, DM-41, DM-52 — Violated for identified paths; DM-51 — Satisfied for the inspected declared versioned migration path | Generated one-row pushes/revalidation; Cell migration still checks declared source/target versions, §4 | Large boundary amplification for mechanically columnar work; no migration-version defect alleged | Column builders/views and native migration plans preserving the explicit migration contract | Layout/domain correctness plus reduced decode/allocation work |
| F7: Dependency capture is conservative but copied, and must survive optimization | DM-31–33, DM-46, DM-48 — Unresolved for the pivot | Whole contexts copied at `memo/context.rs:27`; optimized-only discovery would miss folded reads | A performance shortcut could break reuse/source lineage | Share exact owners; capture construction-time reads; narrow only complete scopes | Change absent candidate, folded definition, parameter and policy independently |
| F8: Source/row correspondence is repeatedly reconstructed | DM-22, DM-23, DM-41, DM-49 — Unresolved | Rename and commit replay at §4 anchors | Full staging/rebinding consumes resources to recover known local construction | One bound parsed result; plan-based identity/occurrence/diff operations | Imported mismatch still fails; local unchanged parses are retained |
| F9: Materialization is incidental to current APIs | DM-18, DM-29, DM-36, DM-37 — Unresolved | Session returns collected batches, then callers build more workspaces | Missed optimization and redundant buffers | Compose native fragments; explicit owned materialization only where needed | Trace allocations and executed plan; maintain named stage outputs |
| F10: Recursion, provenance and effects need distinct contracts | DM-04, DM-27, DM-30, DM-34, DM-35, DM-43 — Unresolved | Prior review's delta/support/cancellation gaps; native plan presence alone does not resolve them | Truth duplicates, incomplete supports or partial results may be misinterpreted | Relational state with explicit stratum/commit transitions | Cycles, retraction, independent supports, cancellation and failed branches |
| F11: Existing identity and commit mechanisms provide a sound boundary to preserve | DM-11–15 — Satisfied for the inspected explicit mechanisms, not full new-pipeline certification | Semantic/artifact distinctions; local exact expected-state replacement with explicit durability outcome | Replacing these with generic UDF effects would lose guarantees | Retain and connect to executor completion | Existing identity/ref regressions plus completion integration |
| F12: Performance and proof claims must remain conditional on their premises | DM-39, DM-40, DM-53, DM-54, DM-59, DM-60 — Unresolved for implementation acceptance | Eight library checks; no end-to-end performance or formal proof | Planner visibility can be overstated as optimality or universal correctness | State compositional arguments, implementation assumptions and targeted evidence | Work accounting, independent semantic fixtures, representative cold/warm measurements |

The F2/F5/F6 findings identify architectural duplication or boundary violations;
they do not allege that their current outputs failed the assumed Wave 1 suite.
Unlisted principles have no individual verdict in this review.

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication / extension locality | Correctness and operational risks | Cost | Performance evidence | Disposition |
|---|---|---|---|---|---|
| Continue current output validation/replay | Every pass retains arbitrary result assembly; validators/replay repair trust later | Safe checks can remain, but the same meaning must survive multiple implementations | Low initial change, continued maintenance and execution duplication | Prior review shows copies/row paths; no universal timing claim | Reject as the long-term foundation |
| Simpler viable alternative: immutable admitted batches plus shared native diagnostic plans | Removes duplicate PK/FK engine and repeated import admission; keep opaque `Pass::run` | Cannot derive pass output semantics or remove producer replay without another basis | Smaller useful first slice | Likely avoids repeated input decoding; unmeasured | Use as a migration step; insufficient final scope |
| Native plans with registry-derived properties and private completion | One execution IR; exact input properties compose into outputs; domain declarations stay singular | Transfer rules, bindings and implementation contracts must be sound | Moderate shared-boundary redesign, fewer independent semantics afterward | Strong optimization opportunity; measure actual consumers | Recommended |
| Build a general proof DSL / second relational optimizer | Another meaning system and translation layer | New proof checker and equivalence obligations can exceed the product scope | High ongoing cost | No demonstrated advantage | Reject |
| Wrap every parser, kernel and commit in an opaque logical extension | Uniform-looking plan tree but opaque internals and effects remain | Pushdown/reordering can be unsafe; wrapper shape disguises rather than removes assumptions | High adapter cost | Can block useful optimization | Reject blanket approach; extensions remain available for a concrete benefit |

The recommended abstraction earns its cost by replacing the existing independent
validation/replay mechanisms. Start with the properties required by actual P2,
migration, authoring and compiler consumers. Unknown properties can produce a
specific diagnostic; there is no requirement to model every possible mathematical
theorem before shipping a functional product.

Native `LogicalPlan` supplies substantial structural reasoning and an optimization
surface. Pinned `assert_expected_schema` deliberately ignores metadata/nullability
(`external/datafusion/datafusion/expr/src/logical_plan/invariants.rs:114`);
`check_invariants(Always)` checks current-node name uniqueness
(`external/datafusion/datafusion/expr/src/logical_plan/plan.rs:1217`). Default
extension invariant checking returns `Ok(())`. `Constraints::new_unverified`
accepts caller assertions without validating their data
(`external/datafusion/datafusion/common/src/functional_dependencies.rs:53`).
These precise boundaries identify the small domain-specific layer needed;
they are not reasons to limit DataFusion's role.

## 9. Verification and measurement plan

This is **implementation qualification**, not a proposal to replay every production
transformation. Tests establish the small trusted constructors, transfer rules,
adapters and lifecycle transitions. Production code then relies on their explicit
contracts and retained inputs. Residual data checks execute only where an actual
input or operation needs them.

| Claim | Evidence label | Named check / analysis | Conditions and expected result | Current result / gap |
|---|---|---|---|---|
| Native construction can reject invalid fields and custom semantic domains | Tested | `wave1-logical-plans-2026-09-14/run.sh` | Rust 1.98.1, dev, force_validate; no input batches for structural cases; UDF body invocation count zero | 8 total checks passed, 0 failed, baseline 0; isolated library only |
| Operator facts compose soundly | Proposed | Transfer-rule contract cases and written premise/conclusion arguments | Projection/filter/lookup/union/aggregate/unnest; nulls, duplicate keys, grouping sets, equality domains, lost premises | New implementation required; no formal proof claimed |
| No duplicate local producer execution | Proposed | P0–P3 and rename execution counters plus output fixtures | One actual producer per changed computation; no verification-only ChangeSet; imported tampering still rejected | Existing replay boundary remains |
| Facts cannot be forged or detached | Proposed | Constructor/API and ownership tests | Caller batches cannot create admitted completion; changed function/context/port cannot inherit facts; external mutable buffers isolated | New private boundary required |
| One generated invariant implementation covers current validity rules | Proposed | P2 diagnostics equivalence and adversarial inputs | Exact witnesses for keys, references, ordinals, null masks and incomplete contexts; candidates advertise no assumed constraints | Consolidation not implemented |
| Native rewrites preserve required field and behavioral contracts | Proposed | Analyzer/optimizer/physical integration cases | Compare semantic fields, outputs and actual typed bags; logical/subquery traversal complete; corrected ALL semantics | Existing probe provides native building-block evidence only |
| Dependency-derived reuse is complete | Proposed | Mutation cases for reads removed by optimization | Folded quantity lookup, empty candidate scope, defaults, function replacement, params and lower-stratum negation | Retain conservative complete context until narrower derivation exists |
| Partial work cannot publish | Proposed | Stream failure/cancellation and commit conflict/fault cases | Failure before EOF/obligation completion produces no admitted result; post-ref uncertainty stays explicit | Existing commit behavior inspected; new execution integration needed |
| Columnar redesign improves total work and latency | Proposed | Before/after P0–P3 import/rename/migration and P3–P10 compilation | Same inputs/output contracts, native CPU instruction policy, batch/partition settings recorded; cold/warm; workstation-sized configurable budget | No timing claim or optimality claim |

**Cost accounting:** report planning/analysis/optimization time separately from
first-result and complete execution; source bytes/rows read, rows decoded into
Cells, full validation passes, producer invocations, intermediate materializations,
copied bytes, reserved/peak memory, spill, publication and reopen cost. Include
small interactive edits and larger/skewed relations, not only throughput-friendly
scans. Native planning reduces redundant work only if consumers actually keep
their operations in the plan and avoid re-materializing them through row APIs.

Use existing recipes for terminal implementation verification: `just test`,
`just ci-fast`, generator/family/governance checks, and `just py-sync`, `just py-test`,
`just quality` for changed Python boundaries. Solver parity remains tied to actual
numerical scope. The review does not add distribution builds or a whole-library
qualification gate. Failures always have baseline zero and a named command/mode.

## 10. Exceptions and unresolved decisions

| Decision | Recommended resolution | Owner / consequence / trigger |
|---|---|---|
| Active admission wording versus trusted internal construction | Amend blueprint §4.3/§5.4/§14.2/§14.3 so validity may be established by retained construction and completed obligations; actual external content still establishes its properties | Architecture maintainer; reconcile ADR-0039 and proposed ADR-0052 before implementation removes mandated checks |
| Arbitrary pass output and duplicate replay contract | Specify private prepared/result completion and imported-result admission separately; preserve output inventory/lineage | Compiler/catalog owners; reconcile §14.1/§14.3/§20.1 and ADR-0040/0056 |
| Native plans versus existing bounded RulePlan | Treat declarative syntax as a frontend and remove implementation ceilings; reuse ADR-0065's broad default policy | Rules/schema owners; supersession/reconciliation for ADR-0048 and any affected proposed rule records |
| Cross-pass fusion and persisted stages | Compose internal fragments now; retain named artifact boundaries unless an explicit stage-contract amendment is approved | Compiler owner; later change only if stage materialization itself is redesigned |
| Canonical envelope versus runtime size | Separate byte-normalization identity from configurable supported size; choose generous defaults from actual workstation/workload needs | Identity/runtime owners; ADR-0045/0050 contain the current contract, so resolve before raising hard-coded ceilings |
| Foreign buffers, native kernels and publication effects | Keep concrete specialized ownership/algorithm/protocol contracts; do not disguise them as pure relational semantics | Boundary owners; revisit only when a native/library adapter gives a demonstrated advantage |

No new SHOULD exception or library ban is requested. Architectural records are
implementation prerequisites where contracts change, not permission to postpone
ordinary library adoption. This review does not set accepted status, supersede
accepted records in place, or silently redefine the blueprint. It supplies the
coherent target those decision changes should describe.

## 11. Decision and implementation changes

**Decision: Revise.** Proceed with the fundamental construction-and-ownership pivot.
The preceding review's C0–C7 work should be integrated into this sequence rather
than implemented as a second layer of retrospective checks. G1 has a coherent
authority path; G2–G7 remain unresolved for the redesigned implementation. The
goal is to make invalid assumptions unavailable to downstream code, with explicit
value and effect boundaries wherever construction cannot establish the fact.

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| LP0 — decisions and contracts | Reconcile §10 against blueprint/ADRs; specify the limited property set, exact contexts, local/imported paths and raw-candidate behavior | DM-02, DM-07, DM-22, DM-59 | One authoritative contract per meaning; no second executable IR; no feature ceiling | Decision/document checks and API inventory |
| LP1 — owned preparation and results | Freeze semantic context; bind native plans, parameters, function/native-operation reads; introduce private execution completion using existing owners | DM-26–32, DM-41, DM-44 | Prepared work retains exact inputs; arbitrary `RecordBatch` cannot mint completion; producer invoked once on one migrated pass | Context mutation, stale port, partial stream and imported candidate cases |
| LP2 — shared invariant and field construction | Lower generated obligations through native plans; add semantic field/property transfers; keep candidate constraints unasserted | DM-06–10, DM-17, DM-24, DM-52 | P2 key/FK/domain checks have one implementation; lookup/projection facts derived; residual obligations explicit | Negative premise/equality/null cases and physical-field integration |
| LP3 — columnar foundations | Generate column builders/views; plan-based migration, source projections and change diffs; preserve parsing and sequential edit meaning | DM-21, DM-37, DM-41, DM-49, DM-51 | P0–P3 import/rename/migration use prepared results; retire redundant Cell/reparse/replay paths that the new construction discharges | Exact source/before-image/rename and generated contract fixtures |
| LP4 — execution integration | Defer provider bulk filtering to execution; retain streams/leases; use normal native optimizer surface and explicit semantic lowering; preserve generous budgets | DM-20, DM-26–30, DM-36–40 | Executed physical plan matches observation; no mandatory intermediate collect/copy; cancellation/size behavior explicit | Pending-stream cancellation, resource ownership and scan projection/filter tests |
| LP5 — compiler/rule consumers | Migrate P3–P10 relational work, truth/support/delta and binding to native composition; retain specialized quantity/MathIR algorithms | DM-18, DM-24–25, DM-34, DM-38, DM-46 | Real end-to-end semantic compilation with correct bag/NULL/conflict/support behavior; full library facilities available by actual execution route | Original Wave 2 product fixtures plus semantic counterexamples |
| LP6 — reuse and publication | Capture complete construction-time dependencies; share exact context owners; connect completed results to existing atomic publication | DM-14, DM-29–35, DM-48 | Compatible local reuse without producer replay; changed absent/folded dependencies handled; imported/reopened content admitted; conflict/uncertainty preserved | Mutation, interrupted execution and publication journeys |
| LP7 — retire duplication and qualify | Delete superseded local validators/replay/row interpreters after migrated routes satisfy contracts; run original Wave 2 exits and representative work/latency measurements | DM-39, DM-53–60 | Zero-failure named terminal gates; fewer independent semantic implementations and reduced redundant work demonstrated | Keep focused constructor/adapter/lifecycle tests, not production double execution |

LP0–LP2 establish the shared basis; LP3–LP6 migrate the actual consumers; LP7 closes
the original product scope. This is a dependency-ordered combined execution, not a
parallel-agent plan. The implementation may use small slices, but the target is
the complete construction path: declared meaning → native plan → owned execution
result → admitted artifact → explicit publication.
