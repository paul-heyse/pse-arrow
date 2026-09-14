---
title: Wave 2 — semantic compilation and inspectable model graphs
status: draft
date: 2026-09-14
adrs: [ADR-0004, ADR-0005, ADR-0006, ADR-0007, ADR-0009, ADR-0010, ADR-0013, ADR-0016, ADR-0024, ADR-0027, ADR-0031, ADR-0039, ADR-0040, ADR-0041, ADR-0042, ADR-0043, ADR-0044, ADR-0046, ADR-0047, ADR-0048, ADR-0050, ADR-0051, ADR-0052, ADR-0053, ADR-0054, ADR-0055, ADR-0056, ADR-0057, ADR-0058, ADR-0059]
phase: 1
---

# Wave 2 — semantic compilation and inspectable model graphs

## Context

**Proposed target:** turn committed engineering declarations into a complete, typed,
indexed mathematical graph through the real P3–P10 production pipeline, then reopen
and inspect that graph from Rust and Python. Ship the reference data and reusable
templates needed to compile steady-state heater/mixer flowsheets without fixture
predecessors or hand-assembled derived rows.

Wave 2 is an implementation wave within **blueprint phase 1**, not blueprint phase 2
(the recycle/flash Slice B). It completes the Wave 1 preview and the semantic compiler
portion of Slice A. Full Slice A requires scalarization, case binding, structural
analysis, plans, numerical backends and solving; those are the next wave's target.

On 2026-09-14 the maintainer requested this plan using **Wave 1 passing as a planning
assumption**, without waiting for its remaining qualification. That assumption enables
planning and independent implementation; it does not turn an unfinished gate into a
passing receipt. Preserve the [Wave 1 plan](03-wave-1-foundations.md) and
[audit](03-wave-1-foundations-audit.md) as the predecessor evidence.

### Authority and investigation

The [blueprint](../authoritative_design/blueprint.md), revision 31 at planning time,
governs. In particular: §5 (identity and artifacts), §6.2–§6.9 (physical, material,
template and math contracts), §9.6 (demand closure), §10–§13 (laws, units and topology),
§14 (passes and reuse), §20–§22 (persistence and boundaries), and §24–§25 (acceptance).
Its revision history takes precedence over older overview prose that still says
revision 5. The historical proposal §5 explains the template/contribution intent but
does not override the blueprint.

Read alongside this plan:

- [ADR index](../adr/README.md) and [deferred register](../adr/register.md), especially
  ADR-0040–0043, ADR-0046–0048 and ADR-0051–0059. ADR-0052–0059 remain proposed;
  implementation authorization and formal decision-PR acceptance are distinct.
- [Plan 02](02-blueprint-revision-5-contracts.md), especially demand/provision,
  incremental-versus-clean, relational expansion and Arrow lifetime fixtures.
- [Design charter](../design_review/design_principles/DATA_MODEL_DESIGN_CHARTER.md),
  especially DM-02, DM-07–DM-09, DM-14–DM-24, DM-28–DM-31, DM-37–DM-43 and G1–G7.
- Pinned capability maps: [DataFusion §4–§7](../capability-maps/datafusion-rust.md),
  [Arrow §8](../capability-maps/arrow-rust.md),
  [Python §2–§3](../capability-maps/python-libraries.md), and
  [supporting Rust §2, §5](../capability-maps/supporting-rust-libraries.md).
  Their historical recommendations do not reopen decisions such as deferred custom
  rule nodes or alternative memoization. Fetch current Context7 documentation and
  inspect the pinned source before binding a previously unused library API.

**Interface-checked, source inspection:** `pse-templates/src/lib.rs` is still a
declared boundary. `pse-rules/src/exec/mod.rs` executes one admitted rule and retains
unknown candidates; it does not implement the complete multi-rule stratum/conflict
scheduler. `pse-compiler` has P0–P3 adapters and fixture P10. `pse-py/src/lib.rs`
exposes build information, not catalog handles. The Rust manifest is hand-written,
with declaration parity. `pse-quantity/src/standard.rs` supplies a Rust standard
fixture. These are concrete extension points, not evidence that Wave 2 already exists.

### Predecessor checkpoint

**Tested, existing Wave 1 receipts:** the final default-profile `just ci-pr` Rust
stage ran 660 tests with force-validate: 660 passed, zero failed/skipped; doctests
ran 33 with zero failures; its narrower governance test stage ran 48 with zero
failures. The overall recipe subsequently failed generated-tree hygiene because
Python 3.13 parity created twelve ignored `__pycache__/*.pyc` files beneath the
generated Python directory. Log: `/tmp/pse-wave1-final-ci-pr.log` (local receipt,
not a portable artifact). Fix the generator's file-inventory classification and
test both interpreter caches; deleting caches once would not correct the check.

**Tested:** `just parity-container` ran 116 tests on Python 3.13.15 with zero failures
or skips, including ordinary contracts and the declared compatibility names. Its
then-implicit `uv run` interpreter selection emitted a warning; the script now
explicitly selects 3.13 and awaits a confirming rerun. This is not numerical solver
parity. The host quality preflight passed, including warning-level type checking.
`just features-powerset` and `just test-release` were not run before the planning
pivot. No additional Wave 1 build/test process is intentionally left running.

These are a bounded handoff item in W2-00, not a reason to postpone this plan or
repeat all foundation implementation work.

## Decisions

All new scope below is **Proposed**. This plan selects a delivery boundary; it does
not amend the blueprint or accept an ADR.

| Decision | Wave 2 target and boundary |
|---|---|
| Product result | A persisted `CanonicalMathGraph` with indexed equations, complete physical typing, selected property provisions and inspectable derivations. It is not a `CanonicalMathProblem` or a solve-ready artifact. |
| Compiler | Implement P4–P9; connect production P10 to its real predecessors and complete declared context. Extend P3 only where necessary to normalize the newly authored contracts and finite candidate bindings. Retain the explicit P10 fixture as a leaf/regression test. |
| Reference library | Shipped units/elements/quantity operations; ideal property declarations; single-phase FTPx and FcTP states; steady flowsheet, lumped control volume, feed, product, heater, mixer and state junction; equality connections. Generic mechanisms must also pass adversarial extension fixtures. |
| Physics breadth | Core pure-component methods: NIST Shomate cp/h/s, RPP4 polynomial cp/h/s, and Perry liquid cp/h/s and density forms needed by the declared ideal package. Each shipped provision has an explicit formula, domain, natural coordinate, parameters and source evidence. Unshipped method provisions are refused explicitly. |
| Conservation bindings | Material componentTotal/componentPhase and elementTotal, energy enthalpyTotal/isothermal, momentum pressureTotal; mixer total-flow law in its own declared context. Honor unsupported control-volume total material balance and other absent law bindings under blueprint §10.2. |
| Kernels | Implement P9 descriptor/parameter/unit binding and refusal paths. Stock Wave 2 examples use equation-template methods. Kernel binding fixtures exercise complete descriptors; they do not certify numerical execution, derivatives, a UDF or an exported backend. No fake implementation digest or claimed available execution binding. |
| Python | `pse.open(...).head()` and immutable snapshot table streams, with explicit resource configuration. Keep `pse.compile(...)->ProblemHandle`, solve, Pyomo construction and numerical arrays for the problem/backend wave. Rust Driver and golden tooling exercise compilation now. |
| Identity and reuse | Keep existing identity framing. Validate declarations, values, keys, domains, physical contracts and complete actual dependencies first. Whole-stage reuse only; hashes are lookup/integrity tools. |
| Execution organization | One combined dependency-ordered execution. Agents may own bounded disjoint work, but parallel implementation is not a plan requirement. No new crate or dependency family is assumed. |
| Resources | Ordinary workflow qualification uses the shared 32 GiB ceiling and explicit thread budgets, with additional declared configurations for measurements. The maintainer's 16-core/32-thread, 192 GiB workstation supports generous budgets. Tiny-budget refusal tests remain separate stress tests. No new 512 MiB production default or arbitrary low iteration/row cap. |

The first accepted graph must come from package documents through commit and the
real compiler. An output constructed directly by a fixture is useful to test a
leaf operation, but cannot satisfy the vertical workflow exit.

### Arrow and DataFusion execution contract

This is a required implementation constraint for every packet, not an optional
performance improvement. Blueprint D1/D6/D7/D10, §4.1–§4.4 and §14.2 govern placement.
Typed Arrow relations are the authoritative facts and derived outputs; DataFusion
executes the relational algebra that joins and transforms them. Adapters and Rust
workspaces must not accumulate a competing object model.

| Work | Required mechanism | Boundary and design principle |
|---|---|---|
| Schema/field/value admission | Registry-generated Arrow schemas, recursive extension/value checks and current semantic analyzer/head admission | `RecordBatch::try_new`, a schema match or extension registration alone does not prove units, references, domains or validity; DM-06/07/42. |
| Snapshot inputs and inspection | Existing snapshot-scoped CatalogProvider/SchemaProvider/TableProvider routes, explicit named stage bindings and read-only queries | Retain active admission before planner constraints or optimizer assumptions; inspection cannot build model facts; DM-02/20/28. |
| P4 membership, inheritance and compatibility | Declared RulePlanSpec scans, filters, projections, joins and anti-joins lowered with LogicalPlanBuilder | No hand-coded per-unit/species loops deciding these relations, and no handwritten SQL string as a second rule definition; DM-16/19/22. |
| P5 containment and scope sets | The same relational closure/delta machinery over typed instance, domain and connection tables | Use petgraph only for bounded graph algorithms such as tear candidates, with typed inputs/outputs and deterministic ordering; different graph relationships remain distinct; DM-25/34/38. |
| P6 candidates, precedence and demand | Joins, lower-stratum anti-joins, grouping and set differences over the complete candidate/demand relations | A Rust scheduler manages rounds and provenance; it must not reimplement selection as a dictionary of property-name callbacks; DM-19/31/32. |
| Fixed-point state | Admitted Arrow delta/accumulated/support batches and relational semantic-key comparisons | Private executor worktables have declared schemas and scoped lifetimes; they do not mutate snapshot providers or become hidden stage inputs. Use native recursive queries only when their exact semantics and reconstructable provenance suffice; DM-27/29/35. |
| P8 participation and completeness | DataFusion joins/anti-joins for candidate classification, aggregate with explicit ordering for expression descriptors, and declared unnest operations | Preserve parent IDs, explicit ordinals and null/empty behavior. Group descriptors, not floating solver values; DM-18/24/40. |
| P7/P9/P10 mathematical construction | Existing Arrow-free typed MathIR/quantity algorithms and generated Arrow relation adapters | Rust may walk the ordered DAG, substitute declaration bindings and construct indexed IR. It must not replace the math IR with DataFusion Expr or implement unit physics in per-model builders; DM-03/21/25/38. |
| Batch transformations | Generated column views/builders and pinned Arrow filter/take/concat/slice/cast kernels where their contracts fit | Validate physical meaning and exact representability before attaching destination metadata. No blanket whole-snapshot conversion to `Vec<Cell>`, row DTOs, pandas or Python objects; DM-36/37/41. |
| Runtime and transfer | Existing shared DataFusion RuntimeEnv, fallible reservations, bounded batch streams and pyo3-arrow capsule adapters | Retain final-buffer-owner accounting, cancellation and explicit copies. Batch streaming is a mechanism, not proof that all memory is bounded; DM-30/35/39. |
| Provenance and evidence | Typed support/participation/undecided relations plus actual rule observations and EXPLAIN | Explanations come from declared facts and actual execution, not inferred from display strings, physical arrival order or serialized plan bytes; DM-23/49/50. |

Transient native data structures are allowed for a bounded algorithm, parser or IR
workspace with a demonstrated consumer and complete Arrow ingress/egress contracts.
They cannot own independent defaults, mutable engineering facts or hidden dependencies.
Every proposed manual row loop must identify why the operation belongs in that native
algorithm rather than the existing relational plan. Every new custom execution node
or UDF must likewise establish why standard pinned operators cannot express its real
consumer; R-02/custom rule nodes remain deferred.

Hashes used internally by DataFusion for joins or deduplication are compatible with
this plan when equality is checked against actual admitted typed keys/values. The
prohibition is using a digest as semantic validation, proof of fixed-point convergence,
an absence certificate or a replacement for the complete reuse dependency contract.

**Interface-checked on 2026-09-14:** Context7 returned the upstream
[logical-plan construction guide](https://github.com/apache/datafusion/blob/main/docs/source/library-user-guide/building-logical-plans.md),
[streaming architecture](https://github.com/apache/datafusion/blob/main/docs/source/user-guide/arrow-introduction.md),
and [Arrow C Stream API](https://docs.rs/arrow/latest/arrow/array/ffi_stream/index.html).
These establish library routes, not this application's behavior. The current runtime
documentation is discovery evidence; implementation remains pinned by Cargo.toml.

Additional pinned-source checks against the capability maps found:

- DataFusion 55.1.0 exposes `join_detailed`, `aggregate`, `distinct`,
  `unnest_columns_with_options` and `to_recursive_query`; the last performs schema
  coercion. Platform exact head/quantity admission must still guard that coercion.
  Native recursion supports a distinct mode, but the current platform RulePlan
  deliberately admits only bounded UNION ALL. W2-04 must preserve that distinction;
  library availability does not silently extend the platform contract.
- Existing SnapshotSession already calls the physical streaming execution route.
  Extend its admitted stream boundary for Python rather than adding a second
  SessionContext/query facade that bypasses the snapshot/provider restrictions.
- Arrow 59.3.0 `Array::claim` and `TrackingMemoryPool` track shared-buffer ownership,
  but `MemoryPool::reserve` and reservation resize are infallible. They may help a
  measured ownership audit; they cannot replace pre-allocation fallible budget
  admission. DataFusion's pinned `MemoryReservation::try_grow` remains the budget
  mechanism through the existing shared runtime.
- Pinned pyo3-arrow exposes `PyRecordBatchReader::new` over an owned Send reader
  and a one-consumption reader contract. Use its stream route; `into_table` collects
  the stream and is not the default export path. Actual release, cancellation and
  generated semantic metadata still require W2-10 tests.

**Proposed verification:** `wave2_relational_execution` must inspect the actual
compiled plans for representative P4/P5/P6/P8 rules and show their semantic work
uses the expected standard relational operators. Pair plan inspection with full
result/derivation equality against an independent small reference computation,
including repartitioning, shuffled input batches, nulls and empty relations. A
plan containing a join is not itself proof of correct participation. Record any
deliberate native algorithm boundary and its typed inputs/outputs in the review.

### Contracts to settle before their consumers

W2-01 must produce reviewable declarations, invariants, producer ownership and
diagnostic outcomes for the following. Author the missing design in this repository;
do not depend on an unavailable document or silently fill a payload with JSON/text.
Allocate ADR numbers with `just adr-new` when writing them, not in this plan.

| Contract issue found | Required decision and enforcement | First consumer |
|---|---|---|
| P7 contributions and P8 negative completeness are described in §10/§14, but the existing template-contribution rows are declarations, not a complete realized substrate | Specify typed realized contribution, law-application/participation and exclusion records: identities, instance/scope/domain, expression root, physical subject/basis, orientation, candidate universe and reason/support references. Compare actual contributions; an exclusion digest cannot certify absence. Reuse an existing provenance relation only if it expresses these meanings completely. | W2-07/08 |
| `inferred.method_resolutions` has unresolved/ambiguous status but requires `method_id` in the current declaration | Make missing selection and competing candidates representable without NIL IDs or a fabricated winner. Define exact requirement identity, multi-requester support, candidate applicability, explicit/default precedence, version pinning and tie behavior. Bind negative decisions to the complete candidate inventory. | W2-05/06 |
| P4 decides features before P5 expands instances and P7 realizes symbols | Specify finite prospective child scopes and inherited parameter/feature bindings available from P3 declarations. P5 port members bind actual declarations and domains; validate the P7 realization correspondence later without adding a P4/P5 read of P7. Detect recursive template expansion, invalid inheritance and unbounded candidate creation. | W2-05 |
| Text-valued feature/parameter/options declarations must acquire declared types | Define one normalization/admission route from the declared logical type, enum, quantity and domain contract. Rules consume typed normalized values. Never interpret arbitrary expressions, infer meaning from a name, or introduce per-template parsing rules. | W2-02/05 |
| P4 method compatibility and four-valued support need complete row/ownership contracts | Map compatibility, unknown/conflict keys, supporting rows, rule version and stage binding to declared relations. Check row/rule derivation granularity explicitly; the current declaration helper defaults derived relations to row granularity. Preserve every competing support even when semantic head rows deduplicate. | W2-04/05 |
| `authored.instance_equations` remains an Appendix B exemption | Specify the instance-owned analogue of template equations, its expression-owner/target binding and generated DocumentSpec before parsing or realizing it. No separately maintained parser DTO or guessed owner. | W2-02/07 |
| `inferred.tear_candidates` has no row shape | Specify connection-keyed policy/cost, chosen edge set, method and support. Implement deterministic heuristic candidate computation using the existing graph library; prove by removing the selected actual edges that the remaining graph is acyclic. Do not claim minimum cost or FOQUS parity without evidence. MIP selection remains unavailable until a suitable backend exists. | W2-05 |
| Phase-equilibrium-species contract conflicts over pair identity and intersection versus l_only/v_only membership | Resolve it before claiming multiphase closure. Wave 2 stock packages are single-phase; unsupported phase-equilibrium realization fails explicitly. The relation may remain deferred if unused, with a negative scope test and R-30 retained. | Future multiphase consumer |
| Closure-report payload/associations remain undefined | Define the full typed report contract only if needed for partial graph inspection. Never emit a P14 final closure verdict at P10. Ordinary stage postconditions, undecided rows and provenance suffice for Wave 2; the final problem-publication gate remains later. | Future P14, or explicit earlier reporting consumer |
| P10 currently accepts fixture-specific roots/quantity context | Move required production selections and ordered roots into actual declared production inputs or deterministic derivations from them. Do not carry fixture-only constructor settings into production or select a default physical type by name. | W2-09 |
| Python handles, stream ownership and manifest generation activate R-27 and a new boundary | Specify handle/ref pinning, errors, cancellation/close/drain behavior, resource parameters and truthful per-field metadata loss reporting. Generate Rust wire fields from ManifestSpec; keep semantic admission in the catalog. | W2-03/10 |

Changes to decisions, metadata, Python contracts or governance require the ADR and
design-review route in AGENTS.md. New relation/pass contracts need the short ADR
route. Make narrow blueprint amendments with stable section numbers and revision
rows before dependent code. Accepted ADRs remain immutable. This plan does not
require reopening accepted hashing or package pin decisions.

### Explicit carryover disposition

Use `tests/governance/appendix-b-deferred.toml` as the exact current inventory,
not a blanket waiver. W2-01 must review all eighteen rows and assign their consumer.
Remove an exemption only with the complete declaration and positive/negative tests.

- Required now: instance equations and tear candidates; resolve any additional
  exemption actually reached by the declared package/pass graph.
- Keep pending their real consumers: phase-equilibrium species; tags unless a shipped
  selector uses tags; template scaling defaults, diagnostic thresholds, alternatives
  and alternative sets, math static analysis, scaling plans and final closure report;
  measurement models, cost indices, location factors, sweep/profile results, run state
  and solver events. Unsupported selector/report/method requests must fail explicitly.
- R-27 fires now (Rust manifest generation); R-30 fires for the newly executable
  passes and consumed contracts. R-28 plan-codec capture remains optional: existing
  actual EXPLAIN/observations and derivations serve this wave's inspection. If durable
  plan decoding is offered, its complete round-trip qualification becomes mandatory.
- R-29 bindgen waits for native solver work. R-01/R-22, R-24/R-25 and R-26 remain
  measured triggers for finer memoization, scan/transfer optimization and mmap.
  Reuse existing supported operators and safe ownership before adding machinery.

## Plan

### Dependency-ordered work packages

Sizes describe relative implementation/review effort, not elapsed-time promises:
S = bounded integration, M = one subsystem, L = several semantic contracts. All
package completion claims remain Proposed until their named checks run.

| Order | Package and responsibility | Depends on | Main owned scope | Size |
|---|---|---|---|---|
| W2-00 | Preserve Wave 1 checkpoint and reconcile outstanding receipts | Planning assumption | Plan 03/audit, generated-inventory tooling and exact regression | S |
| W2-01 | Close consumed design/schema/pass contracts and review them | W2-00 context | Proposed ADRs, narrow blueprint amendments, registry declarations, exemption register | L |
| W2-02 | Author reference packages and normalize newly consumed primitive facts | W2-01 | `packages/` (new source-data root), `pse-schema` DocumentSpec, `pse-authoring`, P3, quantity fixture generation | L |
| W2-03 | Generate the Rust manifest from ManifestSpec | W2-01 | `pse-schema/src/codegen/rust`, `pse-catalog` manifest module, `xtask` codegen | M |
| W2-04 | Complete stratified rule execution and derivations | W2-01 | `pse-rules`, rule registry declarations and conformance fixtures | L |
| W2-05 | Implement P4 feature/capability and P5 topology/scope closure | W2-02/04 | `pse-compiler/src/passes/p4*`, `p5*`, `pse-templates`, registered rules | L |
| W2-06 | Implement P6 finite property-demand resolution | W2-05 | `pse-compiler/src/passes/p6*`, registered demand rules, method declarations | L |
| W2-07 | Implement P7 generic template realization | W2-06 | `pse-templates`, `pse-compiler/src/passes/p7*`, MathRelation adapters | L |
| W2-08 | Implement P8 conservation and connection expansion | W2-07 | `pse-templates` law machinery, registered rules, `pse-compiler/src/passes/p8*` | L |
| W2-09 | Implement P9 methods/bindings and connect production P10 | W2-08 | `pse-templates`, `pse-compiler/src/passes/{p9,p10}*`, quantity/MathIR adapters | L |
| W2-10 | Expose immutable catalog/snapshot Arrow streams in Python | W2-03, Wave 1 catalog | `pse-py`, narrow `python/pse` facade/stubs, generated boundary declarations | M |
| W2-11 | Qualify complete flowsheets, lifecycle and resource behavior | W2-02–10 | Engine/conformance/lifecycle/parity tests, golden tooling, benchmarks | L |
| W2-12 | Run terminal gates, review scope and record outcome | W2-11 | Plan, implementation audit/review, docs and evidence receipts | M |

This is a combined execution order, not a worktree/parallel-agent prescription.
W2-03 and W2-10 can be delivered independently of inference, but are not substitutes
for the compiled-graph exit. Keep one native build lane while editing shared crates.

### W2-00 — predecessor handoff

Preserve dirty work and logs. Correct the generated-tree inventory so interpreter
runtime caches are excluded while every generated source file remains checked in
both directions. Add the narrow regression rather than weakening codegen equivalence.
Confirm the explicit parity interpreter selection. Reconcile the remaining Wave 1
feature/release/full-PR receipts in the normal validation schedule. Do not mark plan
03 done merely because this plan assumes it is ready. Independent Wave 2 design and
package work can proceed; unresolved foundation behavior affecting a consumer must
be fixed before claiming that consumer's acceptance.

### W2-01 — an executable graph begins with complete contracts

Produce the contract decisions above and a registry-driven pass/port inventory for
P3–P10. Each pass names actual relation versions, required/optional inputs, exact
producer ports, complete outputs (including empties), pre/postconditions, diagnostics
and derivation granularity. Include reference package rows, rule bodies/versions,
method universe, unit/quantity conventions, engine profile and function inventory
where consumed. Register implementations only after the graph and schemas close.

Separate local row admission from cross-stage correspondence. No stage can inspect
an undeclared catalog relation to make a binding work. P6 cannot consume P7/P8/P15.
P5 may bind a symbol declaration and finite domain, but cannot pretend the later
realized symbol is already present. Finalize this distinction in the typed contract.

For each new invariant, supply an actual valid and violating fixture, generated rows,
document schema where authored, and complete serialization. If a requested feature
requires an unresolved contract, resolve the design first or reject that feature at
its owning boundary; do not publish a partially successful graph.

### W2-02 — reference data and reusable packages

Create versioned packages following blueprint §22.1, with explicit identity policy,
pinned dependencies and located data provenance. Separate reference physical data,
method/state/unit templates and the benzene–toluene example's parameter values.
Author these clean-room; external IDAES sources are behavior references only.

Make the units/elements/physical operations package the single declaration source
for the shipped standard data. Generate any required Arrow-free Rust test projection
through `pse-schema`/`just codegen`; no production leaf crate imports the authoring
engine to load YAML. `standard_fixture_matches_yaml` compares complete decoded
quantities, conversion rules and element data, not just fingerprints. Keep scientific
checks independent of generated fixture equality: dimensions, reference conditions,
atomic weights and coefficients need their own source/tolerance evidence.

Ship the exact template/method scope in Decisions, with the four state-interface
property signatures and explicit demand requirements. Retain indexed domains,
defined-state behavior, guard predicates, flow basis, default balance choices and
expression versus variable provisions. Methods carry explicit natural units and
reference conditions; a unit string never supplies a missing physical kind.

Exercise new primitive declarations through generated strict DTOs, target/source
binding, commit and P3. Required method parameters, bounds and feature values are
checked by their declared types. No duplicate hand-written defaults or authored
copies of demand that can already be extracted from an expression.

### W2-03 — one manifest declaration in Rust and Python

Generate the Rust wire shape in `crates/pse-catalog/src/generated/` using the registry's
ManifestSpec; keep codecs, semantic admission and storage operations hand-written
against that generated type. Extend the generator's output-root inventory and the
existing codegen-check route, then remove the shadow wire declaration. R-27 closes
only after strict nested-field/nullability/version parity and existing manifest
round trips pass, including unknown fields, duplicate/missing members and invalid
parent bindings. Preserve existing bytes/field spellings; any wire-format change
requires an explicit version/compatibility decision.

### W2-04 — complete rule semantics

Extend the current compiler/executor rather than creating a second rule framework.
Rules declare dependencies and strata. Reject negation into the current/higher
stratum and non-monotone recursive dependencies. Compute positive closures over
finite declared domains with delta relations, semantic-key deduplication and exact
value comparison. Equal keys with incompatible values are conflicts, not duplicate
rows to discard. Repeated support for one valid fact must retain its derivations.
Represent recursive provenance as finite distinct rule/support edges, not an
enumeration of every proof path through a cycle. Declare that support key and its
identity once; a converged fact set with endlessly growing duplicate provenance is
not a completed fixed point.

Materialize true heads and explicit unknown/conflict outcomes with located supporting
keys; preserve declared false/exclusion evidence where completeness requires it.
Do not map unknown to false, or use arbitrary first/last writer precedence. SQL NULL
handling and four-valued inference are different contracts. Enforce declared conflict
policies after complete stratum evaluation; conflicting candidates never become an
eligible downstream winner.

Use existing DataFusion relational operators for set-oriented work. The orchestration
loop may manage rounds, cancellation, resources and provenance. Keep the existing
bounded UNION ALL RulePlan semantics distinct from set closure. A resource/iteration
limit produces an explicit incomplete failure; neither exhausted work nor a digest
matching a previous round proves convergence. Compare the actual admitted key/value
sets. Preserve exact head conversions and ordered aggregate/unnest behavior.

### W2-05 — features, compatibility, containment and topology

P4 resolves inherited dynamic/holdup choices, feature implications/exclusions,
representative-state `useDefault` balances, phase–species membership and compatible
method candidates through declared rules. Exercise explicit restrictions versus
absent restrictions, non-aqueous defaults, unknown inheritance and competing facts.
Do not conflate a declared method name with a supported provision.

P5 binds finite child instances/domains, containment, typed ports and members, scope
selectors, topology edges and boundary crossings. Detect containment cycles and
dangling/mismatched connections. Physical flow topology may contain recycles; it
must not be mistaken for an invalid containment graph. Persist heuristic tear
candidates with deterministic tie ordering and actual acyclicity checks, while
keeping sequential-modular execution and MIP optimization out of this wave.

Ports derive from state declarations, including expression members. Compare member
domains and full physical contracts, not labels or storage types. Check boundaries
by actual cut sets, including internal cancellation when a containing scope expands.
For tagged selectors, either complete the tag contract in W2-01 or fail the unsupported
selector explicitly; an empty result is not an acceptable substitute.

### W2-06 — property demand before realization

Begin from P3 seeds, P4 decided guards and candidate compatibility, and P5 bound
scopes/domains. Include equation, law, port, display and initializer-only demand.
Initializers contribute declarations now; their execution plans remain later work.
Form candidates from the complete pinned universe and declared selection precedence;
persist resolved, unsupported and ambiguous outcomes and their support.

Expand each selected method's declared requirements to a least fixed point over the
finite `(scope, property kind, bound index)` universe. Preserve multiple requesters.
Reject methods that introduce unbounded scopes or unstable selection. A newly added
candidate or a changed absence can invalidate a resolution even if the selected
method/result happens to remain equal. No realization pass may add a late requirement
and quietly restart P6. Property inspection reads these rows without creating physics.

### W2-07 — template realization without per-unit builders

Implement generic parameter/domain substitution, decided guards, aliases, indexed
symbols, expression symbols, indexed equations and contribution instantiation in
`pse-templates`. The compiler adapter consumes named admitted bundles and emits a
complete immutable P7 bundle. Standard unit physics remains package declarations;
adding a second unit using the same constructs must require no new unit-name branch.

Derived IDs use the existing identity formulas and actual declaration/instance/domain
identities. Reorder or rename must preserve appropriate symbol/equation identities;
shared expression nodes must not erase per-occurrence source or physical binding.
Keep roots and provenance tied to declarations/instances, not the optional shared
node scope alone. If that is insufficient, settle the scope contract before coding.

Preserve indexed reductions, gathers and equation filters. P7–P10 do not expand
equations into scalar rows; P12 owns that lowering. Check missing/extra providers,
undecidable guards, duplicate identities and mismatches between promised and realized
domains. Family hints do not establish P14 classification; use the conservative
EquationFamily contract and producer-declared roles from blueprint §7.5.

### W2-08 — laws and connections from actual participation

Match actual contributions by law family, subject, scope and domain. Emit explicit
unit/basis/reference conversions when required, with their actual conversion data.
Group ordered expression descriptors through declared relational operations; do not
numerically sum floating solver values in DataFusion. Build ordered residual IR and
preserve the strict arithmetic/guarded semantics of blueprint §7.3–§7.4.

For each law, partition its complete candidate set into included and excluded rows
with reasons. Verify each applicable contribution participates exactly once, and
internal transfers cancel according to the scope/law contract. Positive participation
and negative completeness must be independently queryable. Reject missing basis
conversions, unknown composition and unsupported balance bindings.

Expand equality connections from P5 bound members and P7 realized symbol groups;
record connection/declaration/index correspondence alongside indexed equations.
Reconcile the blueprint §12.3 per-index provenance inventory with its indexed P8
equation representation explicitly: do not create scalar equations early just to
populate a provenance row. Mixer physics comes from conservation and outlet-state
templates, not pairwise equality of inlet temperatures.

### W2-09 — method realization and production typing

P9 realizes selected equation-template methods, checks every advertised provision
against actual symbols/expressions, binds parameters by semantic identity and inserts
natural-unit conversions once. Preserve molar/mass basis and reference state; reuse
the existing quantity algebra. Missing output is `prop.incomplete_provider`, and a
new undeclared property read is `prop.undeclared_requirement`.

For KernelCall descriptors, verify actual signature, ordered parameters, domain,
null/error/derivative contract and declared implementation identity. A backend remains
unavailable unless its real binding exists. Binding is distinct from evaluation;
neither a matching digest nor a test descriptor is execution capability evidence.

P10 consumes the actual P9 bundle and all physical/domain/reference/kernel context
through declared ports. Use existing canonicalization and occurrence typing; check
acyclicity, finite constants/conversions, residual quantity types, shapes and indexed
domains. Do not substitute fixed/parameter case values before P13. Preserve all
unaffected predecessor rows. Replace the fixture-only production availability gap
with actual P3–P10 registration, then verify stored import by rerunning each actual
producer and comparing complete admitted output rows and lineage.

### W2-10 — immutable Python inspection

Implement store and snapshot handles that pin the exact admitted manifest/revision.
Opening or inspecting a snapshot cannot resolve methods, realize expressions, change
the catalog or opportunistically compile. Expose named table streams via pyo3-arrow's
supported capsule route; keep Python row contracts generated and consumer-neutral.

Define handle/runtime ownership, close/drain behavior, error propagation, cancellation
and resource configuration before implementation. A stream retains every immutable
buffer and reservation until its last consumer releases it, even if the original
handle closes. Partial drain/cancellation releases unused work; live exported arrays
remain charged. Copies are explicit and reserved. Test actual Rust/Python stream
round trips, nested metadata and storage-only degradation; refuse an unchecked
degraded re-entry. Do not promise to discover registrations inside an arbitrary
consumer merely by exporting a capsule.

Run host and pinned parity interpreters. Force runtime annotation evaluation where
Python 3.14 laziness would otherwise hide incompatibility. Keep the native stub
surface/import boundaries exact; do not add per-node mutation/build calls or imports
of IDAES/Pyomo to ordinary product paths.

### W2-11 — workflow and change qualification

The mandatory positive journey is documents → P0/P1 → unpublished P2 validation →
atomic committed model → P3 → P4 → P5 → P6 → P7 → P8 → P9 → P10 → persisted graph →
Rust and Python inspection. Use both FTPx and FcTP configurations of the shipped
ideal heater/mixer examples. Keep the original golden stores and add a named Wave 2
golden that records all actual parents, source inventories and complete stage outputs.

Qualify uncached execution first, then in-process reuse, durable reuse after restart,
and each declared dependency mutation. Compare decoded semantic values, complete
candidate/participation sets, ordered IR and lineage; hashes are supplementary checks.
Expected outputs must include manually reasoned small fixtures and independent
behavior observations, not only output generated by the implementation under test.

Record whole-workflow timing and resources with the shared generous budget, and run
small refusal/cancellation cases separately. Exercise multiple sessions and retained
Python arrays. Account for candidate sets, delta/provenance state, source documents,
Arrow buffers, result copies and temporary conversion/canonicalization work. Report
pool-accounted peaks beside process peaks, thread configuration and spill/copy counts.
Use the workstation's available CPU budget under one runtime coordinator; a serial
reference profile is a reproducibility condition, not a production throughput cap.

### W2-12 — closure

Run the matrix below, refresh the implementation review against the current code,
and require no unresolved MUST defect in the claimed scope. Fill all three Outcome
subsections only after the declared exits are met. Keep phase-1 solve acceptance and
formal ADR status separate. Update the plan index and deferred register with actual
evidence and remaining triggers. No commit/push/release is implied by this plan.

## Verification

**Proposed acceptance matrix — baseline zero failures, warnings and unexpected skips.**
Names below identify tests to implement; their presence here is not a passing claim.

| Test / evidence | Required discriminating cases and oracle | Owner |
|---|---|---|
| `standard_fixture_matches_yaml`, `reference_package_admission` | Full decoded equality to the single physical-data declaration; located source evidence; altered coefficient/unit/dimension/element rejected or reflected in dependent results. Generated equality alone is not scientific validity. | schema, quantity, conformance |
| `manifest_generated_contract` | Generated Rust/Python nested fields and strict codecs agree; old valid manifest still reopens; malformed metadata/member/parent rejected. Codegen inventory survives imports on all exercised interpreters and still catches stale/missing source files. | schema, catalog, governance |
| `stratified_fixed_point` | Cyclic positive closure terminates on an unchanged actual key/value set; shuffled rules/partitions agree; illegal negation rejected; same key/different value conflicts; duplicate support retained; bound exhaustion fails. Small independent set oracle. | rules, engine |
| `four_valued_rule_outcomes` | True/false/unknown/conflict; null versus false; multiple conflicting producers; actual support keys and lower-stratum absence. No undecided row reaches a true head. | rules, conformance |
| `wave2_relational_execution` | Actual P4/P5/P6/P8 logical plans use declared relational operators; compare complete values/support against independent reference results under repartitioned/shuffled batches. No alternative object model or unvalidated metadata restoration path. | rules, engine, review |
| `feature_topology_closure` | Inheritance/defaults, recursive submodels, phase restrictions, unbound ports, mismatched domains/reference states, nested cut sets; recycle versus containment cycle; tear removal really breaks cycles without an optimality claim. | templates, engine |
| `demand_seed_closure`, `realization_completeness` | Law-only, port/display-only, initializer-only and transitive demands; zero/tied candidates; candidate addition/removal; stable finite recursive demand; omitted provision and late demand fail. Check actual requirement/support sets. | engine |
| `template_instantiation` | Second data-defined unit requires no compiler branch; shared-node occurrences retain source/physical context; guarded symbols, aliases and gathers bind exactly; rename/reorder preserves declared identities. | templates, conformance |
| `law_participation_completeness`, `relational_expansion_reference` | Included/excluded partition is exhaustive and disjoint; each applicable term once; internal cancellation; species/material/scope changes; empty/null/duplicate groups; ordered terms, missing composition/basis conversion and unsupported balances. | engine, conformance |
| `method_realization_units` | Natural/package units differ; conversion occurs once; physical reference mismatch fails; parameter omission/domain violation; symbolic coefficients/forms checked independently against sources and pinned IDAES observations. No solver/derivative claim. | templates, parity |
| `authored_to_typed_graph` | Both state formulations and full heater/mixer documents through real P3–P10; no external fixture bindings; complete stored parents/ports; reload yields the same actual relations/ordered IR and Python table values. | engine, golden, Python |
| `incremental_vs_clean_wave2` | Change species/domain membership, package units/reference data, child/template binding, feature, connection, candidate presence/absence, initializer requirement, rule version, engine/function inventory or kernel contract. Compare full results and current lineage; force equal lookup keys in negative controls. | engine |
| `wave2_publication_lifecycle` | Fail/cancel each new stage and terminal-record write; retry, stale import, wrong parent, missing/extra port, correctly rehashed semantically wrong output, concurrent revision conflict. Previous visible snapshot remains complete. | lifecycle |
| `python_snapshot_streams` | Exact ref pinning, empty/multibatch/sliced/nested data, unknown/malformed extensions, partial drain, handle close with live arrays, error mapping and immutable read-only behavior; consumer registration/loss reporting and checked re-entry. | Python, lifecycle |
| `wave2_resource_envelope` | 32 GiB normal workflow ceiling; generous declared concurrent-session configuration; separately tiny refusal tests; all reservations released after final owners drop. Record actual process and accounted peaks, not budget values as memory measurements. | lifecycle, benches |

Use clean-room IDAES 2.12.0 parity for the exercised scope: phase/component membership,
feature/default behavior, state declarations/ports, conservation term participation,
and selected method formulas at valid reference points. Normalize identifiers to
declared engineering roles and domain tuples. Do not compare object identity or claim
full model/solver equivalence from enum names, row counts or formula checks. Full
variable/equation counts and DOF belong to the scalar, case-bound Slice A gate.

### Commands and terminal receipts

Prefer existing recipes; if new targeted golden/benchmark recipes are needed, add
their tested argument forwarding before using them as an acceptance command.

| Command | Mode and what must be recorded |
|---|---|
| `just doctor` | Ready working copy and exact environment; no stale editable extension. |
| `just test-package <owner> -p pse-relations` | Focused Rust tests, locked default nextest with explicit force-validate; name filters/features and zero failure baseline. Use `-p pse-relations` only when not already the owner. |
| `just codegen` then `just codegen-check` | Actual regeneration equivalence and exact generated-source inventory; qualify both narrow and workspace feature graphs. Bindgen remains a separately declared deferral. |
| `just family-check`, `just governance`, `just adr-lint` | Pins, declaration/producer closure, generated manifest/source equality, precise remaining exemptions, ADR/index/register consistency. |
| `just py-sync`, `just py-test`, `just quality` | Current dev extension, host unit/component behavior, lint/types/imports/config with warnings included; record interpreter and workers. |
| `just parity-container` | Pinned container/interpreter, ordinary contracts plus exercised IDAES behavior; failures are not skips. |
| `just ci-pr` | Complete default Rust/doctest, governance, policy, documentation, benchmark-smoke, Python quality/test chain. Report the recipe's overall result, not only its Rust substage. |
| `just features-powerset`, `just test-release` | Feature compilation matrix and optimized Rust tests with force-validate; record them separately. |
| Golden write/check and `just bench-smoke` | Reopen/compare real rows, sources, support and ordered IR. Benchmark smoke proves execution only; record separately measured cold/warm timings and resource conditions. |
| `just docs`, `git diff --check` | Book builds, new links resolve and edits are clean; no runtime correctness inference. |

At exit, link portable receipts or committed evidence summaries with command, profile,
pins, test counts, zero baseline and artifact provenance. A local `/tmp` path alone
is a checkpoint aid, not a durable release record.

**Tested — planning document only, 2026-09-14:** `just docs`, `just lint-typos` and
`git diff --check` pass against a zero-error/warning baseline. A focused read-only
check verifies this plan's local links, ADR references and all thirteen work-package
table/detail pairs with zero failures. These checks do not establish Wave 2 runtime
behavior; no Wave 2 production implementation is claimed by this plan.

### Gate interpretation

| Gate | Wave 2 exit requirement |
|---|---|
| G1 — authority | One declared reference/template/rule/manifest meaning; compiler output cannot mutate authored facts. |
| G2 — Semantic fidelity | Preserve physical kinds, bases/references, identity kinds, indexed domains and unknown/conflict/absence. |
| G3 — validity | Actual boundary/invariant checks reject invalid source, rows, heads, provisions and graphs, including valid-looking hashes. |
| G4 — Hidden behavior | Inspection, optimization and method selection introduce no undeclared effects, ambient inputs or new model facts. |
| G5 — Consistency and recovery | Complete stage/revision publication, typed failures, cancellation, retries and immutable owner lifetimes; no partial result is visible as committed. |
| G6 — Transformation and reuse | Template/rule/law/method lowering, Arrow transfer and reuse preserve meaning; complete positive/negative dependencies and incremental-versus-clean checks cover changes. |
| G7 — Truthful capability claims | Every claimed capability has its actual implementation/validation route; unsupported behavior and remaining numerical acceptance are explicit. |

These are planned acceptance obligations, not a current Accept verdict. Formal
design reviews must use the charter's full gate definitions, not this abbreviated
mapping as a replacement authority.

## Open items

1. W2-01 must settle the concrete schema/ownership gaps above before their consumers.
   The default is to author the missing contract, not ask for an unknown historical
   source or infer one from existing field names. Scope exclusions require explicit
   supported-boundary decisions and tests.
2. Reference property coefficients need a documented, redistribution-compatible
   source and validity ranges. No source code/docstring copying from IDAES. If a
   coefficient set lacks defensible evidence, use a clearly labeled synthetic
   mechanics fixture until the real reference dataset is sourced; it cannot satisfy
   the promised physical-package acceptance.
3. Python ownership/cancellation and the exact exposed resource-configuration shape
   require the W2-01 boundary decision and a pinned pyo3-arrow API check. Keep existing
   dependency pins unless an actual incompatibility is established.
4. Formal disposition of proposed Wave 1 ADRs remains the decision-PR workflow.
   Planning assumes the authorized contracts; it does not silently accept them.

### Next wave and the remainder of blueprint phase 1

After this graph/inspection wave, target P12 scalarization (with an explicit P11
bypass contract for non-discretized models), P13 case binding, P14 structural and
equation-family analysis, P15 scaling/initialization/solve plans and P16 backend
lowering. Add native residual/Jacobian execution, supported KernelSpec adapters,
Ipopt, NL/SOL and the coarse Pyomo adapter, then qualify both heater cases, counts/DOF,
residuals/Jacobians, solutions, stream tables and initialization under blueprint §24.2.

Resolve that wave's static-analysis, scaling, final closure, run-state and solver-event
contracts before registration. Dynamic P11/distributed models, cubic/VLE/recycle solve
breadth, costing/estimation and optional providers stay with their blueprint phases.
Successful Wave 2 graph compilation is a prerequisite for those claims, not their
substitute.

## Outcome (recorded after implementation)

### What was built

### A mistake made and corrected

### Deviations from the plan, deliberate
