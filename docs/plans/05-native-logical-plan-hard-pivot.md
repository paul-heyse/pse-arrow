---
title: Native logical-plan foundations and semantic compilation — hard pivot
status: abandoned
date: 2026-09-14
adrs: [ADR-0039, ADR-0040, ADR-0041, ADR-0042, ADR-0044, ADR-0045, ADR-0046, ADR-0047, ADR-0048, ADR-0050, ADR-0051, ADR-0052, ADR-0053, ADR-0054, ADR-0055, ADR-0056, ADR-0057, ADR-0058, ADR-0059, ADR-0060, ADR-0061, ADR-0062, ADR-0063, ADR-0064, ADR-0065, ADR-0066, ADR-0067]
phase: 1
---

# Native logical-plan foundations and semantic compilation — hard pivot

## Context

**Execution sequence superseded on 2026-09-15 by
[Plan 06](06-provider-contracts-hard-pivot.md).** Its PC00–PC12 packages carry every
unfinished HP00–HP13 outcome into the provider-contract architecture. `abandoned`
marks this sequence, not the product scope or useful target implementation. See
[STATUS.md](https://github.com/paul-heyse/pse-arrow/blob/main/STATUS.md) for the latest boundary. The
[restart handoff](05-native-logical-plan-hard-pivot-restart.md) and all execution
instructions/receipts below are historical; do not resume their earlier next steps.

This plan originally replaced the misaligned Wave 1/Wave 2 execution architecture
with the target in the
[fundamental review](../design_review/reviews/design_review_wave1-logical-plan-foundations_2026-09-14.md).
The complete foundation-to-P10 and immutable inspection outcome remains required
by Plan 06, together with the newer provider review's complete operation framework.

The maintainer's 2026-09-14 direction changes the implementation strategy from the
review's incremental migration language to a **hard pivot**. Existing code and
stored development artifacts carry no compatibility requirement. Correctness is
judged against the target declarations, mathematical arguments, independently
specified examples and lifecycle contracts. The old implementation is not an
oracle, and recreating its outputs or timing profile is not an acceptance gate.

This plan supersedes the remaining execution instructions in
[Plan 03](03-wave-1-foundations.md), [Plan 04](04-wave-2-semantic-compilation.md),
the review's LP0–LP7 sequence and the
[earlier capability review](../design_review/reviews/design_review_full-arrow-datafusion-capabilities_2026-09-14.md)'s
C0–C7 sequence. Their domain requirements and useful evidence remain inputs;
their compatibility, predecessor-certification and migration procedures do not
add work to this plan. In particular, do not run old-versus-new differential
campaigns, preserve old golden-store bytes, maintain the narrow engine profile for
comparison, or finish qualification of code that this pivot deletes.

The maintainer is the sole developer, with implementation actioned through the
agent. Work in one checkout and one coherent target implementation. There is no
collaborator coordination, worktree choreography, compatibility staging or
multi-team handoff requirement. Temporary compile failures inside a bounded
replacement are acceptable; restore the affected build before taking on the next
dependent package. Git history is sufficient reference for deleted source. Do not
create a second maintained legacy tree, runtime fallback or feature flag.

### Authority and evidence

- Blueprint revision 37 and ADR-0067: D1–D14, §3.3.1–§3.3.2, §4–§8, §6.15, §14,
  §20–§24. HP00 amended the affected contracts before dependent implementation.
  The blueprint remains architecture authority.
- The fundamental review supplies the foundation dispositions, property-transfer
  arguments, G2–G7 mechanisms and exact source anchors. The capability review
  supplies the broad native engine/Arrow deployment inventory.
- [Logical planning specification](../capability-maps/datafusion_logical_planning_capability_spec.md)
  and pinned capability maps guide implementation. Use Context7 for API discovery
  when needed, then exact pinned rustdoc/source, Cargo resolution and small probes
  for unfamiliar load-bearing interfaces. Do not inventory or certify unused APIs.
- **Interface-checked:** current module/dependency boundaries and recipe surface
  were read while preparing this plan. DataFusion is pinned to 55.1.0 and Arrow
  to 59.3.0 at the planning baseline. No dependency change is prescribed merely
  to perform the pivot; libraries and their full capabilities remain available.
- **Tested, prior isolated library evidence:** the
  [construction probe](../design_review/evidence/wave1-logical-plans-2026-09-14/README.md)
  recorded 8 passed, 0 failed, baseline 0, Rust 1.98.1/dev/Arrow `force_validate`.
  Its domain-aware function rejects the wrong semantic domain before execution.
  The earlier probe documents raw ALL-set multiplicity defects at the pin.
  Neither receipt certifies the proposed implementation.
- Planning assumes Wave 1 passing. Startup still reports an outdated Python
  environment and stale extension; refresh the editable extension when the target
  Python boundary is ready in HP11. Repair missing development tools when needed,
  without rebuilding discarded code to establish another predecessor receipt.

## Decisions

### Delivery boundary

| Subject | Required result |
|---|---|
| Foundation | Registry-generated typed columns, semantic constructors, native relational plans, immutable preparation/input/result ownership, explicit execution and publication transitions |
| Product journey | Package documents → P0/P1 → P2 → committed authored model → real P3–P10 → persisted `CanonicalMathGraph` → Rust and Python inspection |
| Compiler breadth | Complete P4 features/compatibility, P5 scopes/topology, P6 demand closure, P7 templates, P8 laws/connections, P9 methods/kernel bindings and production P10 typing/canonicalization |
| Reference scope | Units/elements/quantity operations; ideal properties; FTPx and FcTP states; steady flowsheet, lumped control volume, feed, product, heater, mixer, state junction and equality connections |
| Physical methods | NIST Shomate cp/h/s, RPP4 polynomial cp/h/s, Perry liquid cp/h/s and density forms required by the declared ideal package, each with formula, parameters, domain, natural coordinate and located source evidence |
| Conservation | Material componentTotal/componentPhase/elementTotal, energy enthalpyTotal/isothermal, momentum pressureTotal, and mixer total-flow in its declared context; unsupported bindings remain explicit errors |
| Python | Immutable store/snapshot handles and named Arrow streams with resource configuration and owned lifetimes; opening completes declared read-only admission, and subsequent inspection performs no implicit compilation or physics construction |
| Reuse | Prepared computations and complete-stage results bound to exact semantic inputs; finite rule delta execution and complete dependency capture; no fine-grained memoization project |
| Storage | One current target format and generated manifest contracts; fresh development fixtures/stores; current-format reopening and import establish their actual semantic properties |
| Outside this delivery | P11 discretization, P12 scalarization, P13 case binding, P14 structural closure, solve-ready `CanonicalMathProblem`, numerical solving, Pyomo generation and general distributed execution |

The external user-facing behavior promised here is a compiled indexed mathematical
graph, not a solved flowsheet. Numerical kernel **binding** is distinct from
numerical evaluation/derivatives; do not fabricate backend availability to satisfy
an example. Keep unconsumed future relation families explicitly deferred.

### Hard-pivot implementation rules

1. Build the target APIs directly. Replace a module wholesale when adapting its
   current control flow costs more than implementing the declared operation.
   Update all current callers in the same replacement package.
2. Delete replaced modules, obsolete tests, fixtures, validators, dispatch rosters
   and configuration branches. Existing test assertions survive only when they
   independently express a target contract. Do not port implementation-shaped
   tests just to keep their count or coverage percentage.
3. Keep domain algorithms or infrastructure only where they already fit the
   target: grammar/AST operations, quantity algebra, MathIR semantics, canonical
   encoding mechanisms and actual conditional publication are useful candidates.
   Reuse is an implementation convenience, not a preservation obligation.
   This remains design-phase work: retain a graph or data object only when the
   target computation uses it. A current pass result may feed the next target pass;
   no historical graph, replay object, compatibility copy or predecessor-output
   equivalence obligation belongs in the runtime. Target functional outcomes decide
   what is retained, transformed or deleted.
4. No legacy API adapter, dual execution, shadow traffic, old-store importer,
   fallback engine, deprecation period or old-output comparison. If a format or
   public/internal API needs to change, change it and update its declaration,
   version and current callers together. Existing development artifacts may be
   regenerated; no historical data-conversion program is required.
5. A short compilable foundation slice is useful. A temporary second production
   architecture is not. Delete the old route when its target replacement becomes
   the sole route, rather than accumulating cleanup until the end.
6. Validate new behavior once at the smallest meaningful layer. At completion,
   run the terminal target-code gates once; repeat only after a material change,
   failure or unresolved concern. Do not build or benchmark the deleted baseline.
7. Maintain repository type-universe, generated-source, error and clean-room
   contracts. Administrative ADR acceptance is separate from implementation
   authority; the maintainer's chosen pivot does not need repeated permission
   questions between packages.

### Target construction and ownership

The execution representation is native `LogicalPlan`/`Expr`. Typed domain/rule
declarations are frontends that compile to it. Do not mirror the engine's full
node/expression set in a PSE algebra, build another optimizer, or introduce a proof
DSL. PSE's small semantic layer derives only additional domain facts that native
schema/type/property machinery does not express.

| Responsibility | Target owner / representation | Required boundary |
|---|---|---|
| Semantic declarations | `pse-schema` registry; existing relation/column/pass/rule/manifest declarations extended where necessary | One declaration per meaning; schema/codegen bootstrap starts no engine |
| Arrow construction | `pse-relations` generated column views/builders and low-level layout/extension admission | Raw external buffers are candidates; checked local construction can retain established facts |
| Common computation | `pse-catalog` computation/session modules: native plans, semantic field/property derivation, source bindings and private prepared/result types | All frontends use the same binding/preparation/execution route; exact input roles survive even when their schemas match |
| Domain rule compilation | `pse-rules`: rule/invariant lowering, truth/support/conflicts and relational fixed points | Compose ordinary native plans; use the common computation contracts rather than another result capability system |
| Process execution resources | `pse-runtime`: shared runtime, task context, memory/spill, execution cancellation | Fresh execution state, retained logical preparation, owned streams and leases |
| Source and compilation | `pse-authoring`, `pse-compiler`, `pse-templates` | Parser/native steps have explicit inputs/outputs; relational portions stay visible to native planning |
| Storage and publication | `pse-catalog` current-format codec/admission and explicit commit state machine | Consume complete admitted outputs; no raw caller batch can masquerade as a completed producer |
| Quantity / mathematics | `pse-quantity`, `pse-material`, `pse-mathir` and generated batch adapters | D5/D6/D11 meaning remains authoritative; surrounding joins/grouping/expansion use native plans |

Keep the existing acyclic crate layering as the starting point: catalog depends on
schema/relations, rules depends on catalog, runtime constructs catalog sessions,
and compiler composes those services. Put shared preparation types below rule and
compiler consumers. Catalog must not import those consumers to discover a producer.
Instead, assemble the immutable operation/admission implementations before sealing
the session/context, and bind the actual implementations to their declarations.
Registry-driven invariant compilation has one implementation in the rule/semantic
lowering layer; catalog consumes its prepared obligations through the shared
contract. This permits candidate queries before their relational properties are
established without a catalog/rules dependency cycle or a second PK/FK validator.

The target lifecycle is:

```text
actual source bytes / external arrays
        → decoded, physically safe candidate
        → exact bound input and established properties
declaration + bound inputs + parameters + implementations
        → native plan construction / semantic derivation / analysis
        → immutable prepared computation with explicit residual obligations
        → execution with private result streams and owned allocations
        → complete admitted result
        → finished current-format artifacts
        → conditional publication with explicit outcome
```

Private preparation retains the actual plan, source/port owners, registry and
semantic configuration, parameter values, function/planner/native-step objects,
output contracts and dependency closure. Execution alone can mint a completed
result. Reuse or rewrite the existing `CompletedProgram`/`ConstructedInput`
mechanisms where useful, and delete their separate replay/certificate paths.
An admitted result is not a `RecordBatch` accompanied by a caller-set `valid` flag.

Physical layout safety is established before a query touches untrusted arrays.
Imported values establish remaining domain/relational properties once. Internal
operators inherit or derive facts from exact immutable inputs. Only an unresolved
value requirement creates a specific diagnostic or checked-kernel obligation.
No constraint is advertised to the optimizer before its premise is established,
including while running the query intended to establish that constraint.

### Minimum semantic derivation rules

Implement these rules for actual consumers, using native fields/FDs wherever they
are sufficient. Keep the property set private, scoped and small.

| Native construction | Properties and required premises |
|---|---|
| Scan | Exact semantic fields, registry context, source/port identity and input facts; external mutable backing memory is isolated before facts can be retained |
| Projection/alias | Direct references preserve field meaning/refinements; a complete retained unique key stays unique; computed columns need their own derivation |
| Filter | Preserve row-local facts and existing keys; refine supported predicates; remove unjustified coverage claims |
| Total lookup | Under ordinary equality, covered left keys are nonnull and included in the exact right unique key; null-safe alternatives require their own explicit premise |
| Outer/semi/anti join | Derive matched/unmatched nullability, multiplicity and retained-side facts; negative evidence requires a complete right scope |
| Union/distinct | Compatible semantic fields; uniqueness only when justified by disjoint domains or the declared distinct equality; preserve bag semantics where required |
| Aggregate | Complete group identity, including grouping-set identity when applicable; overflow, finite results and floating precision remain explicit |
| Window/unnest | Child/element type, parent correspondence, frame/order and tie semantics; parent uniqueness alone does not imply child uniqueness |
| Cast/function | Full domain/quantity/shape output contract; exact conversion inputs; value checks for unresolved narrowing, division, finite-value or physical-conversion conditions |
| Recursion | Finite positive domain plus monotone accumulation supports termination; facts and support edges have separate identities; negation uses settled strata |

Unknown facts remain unknown. They do not exclude a library feature; they prevent
a consumer from relying on a property until it is established. Metadata expresses
meaning but cannot manufacture it. Hashes may index artifacts or memo candidates;
actual ownership, values and complete dependency binding establish validity and
reuse. Canonical mathematical equivalence, typed value equality, SQL grouping
equality and encoded byte identity retain distinct contracts.

### Resources and materialization

Use the workstation's 16 cores/32 hardware threads and 192 GiB RAM with a single
configurable runtime budget. The existing 32-GiB workflow setting is a reasonable
initial execution ceiling, adjustable upward for the workload. Do not add a
512-MiB production budget or encode old 1-million-row/256-MiB relation ceilings as
permanent architectural limits. Retain checked allocation arithmetic and explicit
resource failure, with generous configurable relation/object/working-set settings.
Separate these settings from canonical byte-normalization semantics.

Compose native plans across internal fragments. Materialize at actual named stage
artifact boundaries, shared expensive computations, fixed-point state, explicit
checkpoints and specialized algorithm boundaries. A CTE or shared plan pointer
does not promise execution once. Shared results needed by output and diagnostic
branches must retain the same actual computation, especially for volatile work.
Do not impose one enormous whole-lifecycle plan or routine collect/copy between
every operator. Keep buffer leases through the final Rust/Python consumer.

## Plan

### Combined execution order

All packages are **Proposed**. Their checks target the new implementation only.
The table is the execution checklist; package details below give concrete work,
deletions and completion conditions. No parallel execution is required.

| Order | Package | Depends on | Main files / modules | Done when |
|---|---|---|---|---|
| HP00 | Reconcile architecture and target contracts | — | Blueprint, ADRs, registry contract inventory, relevant governance rules | Target construction/admission/storage/API semantics are unambiguous and no active instruction requires legacy preservation |
| HP01 | Generate the columnar substrate | HP00 | `pse-schema/{model,codegen}`, `pse-relations`, generated manifest/DTO outputs | Typed columns/builders and current contracts work without internal Cell round trips |
| HP02 | Build native preparation and private results | HP01 | `pse-catalog/{session,computation}`, `pse-runtime` assembly | Exact bindings produce inspectable native preparations and non-forgeable owned completions |
| HP03 | Implement semantic derivation and one admission program | HP02 | Shared computation contracts, `pse-rules` lowering, registry invariants | Construction transfers facts and residual plans establish only unresolved properties |
| HP04 | Complete native execution and ownership | HP03 | Catalog providers/session/physical integration, runtime/cancellation | Full selected native pipeline executes owned streams with truthful metadata, observations and resources |
| HP05 | Replace authoring and P0–P3 foundations | HP04 | Authoring parsing/binding/change paths, compiler P0–P3, relation evolution | Source-to-model-to-normalized output uses the new route; repeated local source/producer replay deleted |
| HP06 | Complete publication, reopening and exact reuse | HP05 | Catalog store, compiler driver/memo/terminal records | New-format results publish/reopen under exact contexts; partial work cannot become committed; reuse follows complete inputs |
| HP07 | Build relational inference and support closure | HP04, HP06 | `pse-rules/{plan,exec,strata}`, rule declarations | Truth/conflicts/unknowns/support and real input-delta closure operate over Arrow relations |
| HP08 | Complete reference packages and physical adapters | HP05 | `packages/reference`, quantity/material/MathIR adapters and generators | One sourced standard library supplies the required states, methods, units and templates |
| HP09 | Implement P4–P6 on native plans | HP07, HP08 | Compiler P4/P5/P6, templates and domain rules | Features, topology/scopes and complete finite demand resolution derive from actual declarations |
| HP10 | Implement P7–P10 on native plans | HP09 | Compiler P7/P8/P9/P10, templates, quantity/MathIR adapters | Generic realization produces the complete typed indexed graph with derivations |
| HP11 | Complete Python and inspection surfaces | HP06, HP10 | `pse-py`, `python/pse`, generated contracts, inspection tooling | Current Rust/Python handles expose immutable results and truthful lifetimes/errors |
| HP12 | Establish target product and performance evidence | HP10, HP11 | Engine/conformance/lifecycle tests, golden tooling, benches | First-principles workflows and representative target-only measurements pass |
| HP13 | Finish deletion and terminal closure | HP12 | Remaining obsolete consumers/config/tests/docs, plan/status | One architecture remains and all target terminal gates pass |

### HP00 — settle the target once

Create the architecture decision record with `just adr-new` at execution time;
do not reserve an ADR number in advance. Use one coherent decision for native-plan
construction, scoped property derivation and immutable completion. Add a separate
short record only when a distinct storage/identity or Python contract actually
needs one. Update blueprint revision history and relevant stable sections before
implementing those contract changes. Use the documented design-edit mechanism
openly. Formal accepted-status changes follow the existing ADR process; this
authorized execution is not gated on repeated maintainer confirmations.

Reconcile these concrete contracts:

| Area | Required decision | Existing references |
|---|---|---|
| Validity/admission | Established construction facts replace mandatory repeated scans; define raw candidates, native obligations, completed results and imported data separately | D5/D10, §4.3–§4.4, §5.4, §14.2–§14.3; ADR-0039/0052 |
| Rule/pass execution | Native plan is execution IR; one operation/producer binding; complete output ports, findings and support; no arbitrary successful batch map | §6.13/§6.15, §14.1–§14.3; ADR-0040/0048/0053/0056/0062/0065 |
| Semantic coupling | Composite/contextual references, quantity/domain refinements, exact field identities, equality/multiplicity and coverage premises | §4.1–§4.5, §6.2–§6.3, §7–§8; ADR-0054/0057/0058 |
| Prepared context/reuse | Immutable sources, roles, actual implementations, settings and construction-time dependencies; current-format persisted context | D14, §14.3–§14.4; ADR-0041/0042/0044/0052 |
| Ownership/resources | Stream completion, foreign-memory boundary, shared leases, wakeable cancellation and configurable size limits | §14.3, §18.2/§18.8, §20.1; ADR-0045/0046/0047/0050/0055 |
| Publication/current storage | Current manifest/encoding versions, exact expected-ref transition, current-format import/reopen, retired formats explicitly unsupported | §5.3, §20.1–§20.5; ADR-0045/0050/0060 |
| Authoring/Python | One parsed source binding, sequential changes, identity-preserving rename and immutable stream API | §21.1/§21.5, §22.1–§22.2; ADR-0059/0061 |

Accepted ADR arguments remain historical and immutable; supersede affected records
with symmetric status links where necessary. Amend affected proposed records to
match the final design. Preserve unaffected D1–D14 domain decisions and the broad
library policy. Remove stale phase-based engine ceilings, feature restrictions and
mandatory replay assertions from active docs/governance tests/hooks where they
conflict with the new contract. Tests must enforce the target invariants, not old
file/class names. Do not dismantle unrelated type-universe or generated-source rules.

Resolve every schema/pass contract consumed by P0–P10, using blueprint §6.15 and
the Plan 04 domain inventory: finite prospective scopes, typed configuration,
method selection/absence, instance equations, realized contributions and law
participation/exclusion, indexed provenance, tear candidates and complete kernel
binding. Author any missing contract locally. Review the live Appendix B exemption
inventory; close entries consumed here and leave later-consumer entries explicit.

Use already complete domain contracts directly. HP00 settles the shared architecture
and identifies any missing consumed declaration; finish a domain-specific declaration
with its owning HP08–HP10 package before its consumer, without another broad review
or approval round. Do not spend this package re-authoring valid scientific contracts
or auditing the discarded implementation.

**Exit:** a concise contract map identifies where each required fact is declared,
derived and consumed. The common construction/admission boundary is settled, any
remaining domain declaration has its owning package, and no active instruction
requires discarded APIs, stores or algorithms. An implementation detail that fits
this contract does not require another design-review cycle.

### HP01 — generate the Arrow substrate and current wire contracts

Rewrite `pse-schema/src/codegen/rust/relation.rs` and related generators to emit
borrowed typed column views and real Arrow builders, including nested/dictionary
columns, validity masks, extension metadata and stable relation/field references.
Use checked scalar/extension constructors where values need admission. Bulk append
operates on typed arrays; finish creates the declared batch without rebuilding all
values through `Vec<Cell>`. Row APIs remain only where a boundary or specialized
algorithm benefits; they are not the internal execution substrate.

Separate physical Arrow safety, field-contract compatibility and relational/domain
obligations. Candidate input can be physically safe without claiming PK/FK validity.
Expose private construction evidence to the common preparation layer, not a public
Boolean or forgeable metadata token. Keep nested-parent masking, quantity sibling
coupling and ordinal target context explicit.

Generate the current Rust manifest wire types from `ManifestSpec`, strict authoring
DTOs, Python contracts and documentation from the same declarations. Replace any
handwritten competing wire shape. Current format versions may change when the new
representation requires it; regenerate all development fixtures. Do not implement
old wire decoding. Current-format malformed/unknown/missing/duplicate fields still
fail according to the declared codec contract.

Delete one-row-batch validation in generated `Builder::push`, admitted-view
revalidation and generated artifacts owned solely by removed interfaces. Fix
generators and their output inventory, then use `just codegen` or the existing
bootstrap generation path; never edit generated files directly. Preserve actual
regeneration equivalence and exclusion of interpreter caches from source inventory.

**Exit:** generated columns/builders and strict current codecs satisfy hand-specified
field/value cases; a generated view over an admitted owner does not rescan values.
The generation bootstrap remains independent of live DataFusion sessions.

### HP02 — create the common native preparation/result API

Replace the raw-plan → freely assembled batch-map trust boundary with private
prepared computations and executor-owned completion. The API supports bound
sources, typed parameters, named output ports, actual function/native-step bindings,
known properties and residual obligation plans. Same-schema before/after sources
remain distinct through role, alias and exact owner; delete `InputBundle::rows`'s
relation-key flattening assumption.

Freeze admission/preparation configuration before handles are minted. Replace
`Catalog.admission: Arc<()>` with a context that actually owns the registry and
semantic admission implementation, or an equivalent private immutable construction.
Remove post-admission validator mutation. A new semantic configuration creates a
new context; existing work remains bound to its original context.

Use native checked builders and native analyzer/type coercion. Domain-specific
constructors add the semantic rules from HP03. Preparation can expose borrowed
original/analyzed/optimized plans and derived facts for inspection. Mutation of a
prepared computation requires new preparation. Execution receives a fresh task,
resource/cancellation state and appropriately fresh/reset physical operators.

Pre-seal assembly supports scalar, aggregate, window, higher-order and table
functions, selected expression/type/relation/analyzer/optimizer/physical planners,
and codecs where used. Bind actual implementations and signatures, not names alone.
Table-function providers are scoped to their invocation and captured inputs.
Stable/volatile/async/configuration-aware facilities stay available with explicit
purpose, inputs and effects. Reproducible compiler execution binds time/randomness/
configuration or rejects the unbound invocation; exploratory uses may select a
different declared policy. No pure memo depends on hidden mutable captures.

Delete the old general `PassOutput` construction capability, independent provider/
function trust markers and duplicated completed-result wrappers. Keep only the
target route callable by production consumers; temporarily remove obsolete callers
when rewriting them is the next dependent package.

**Exit:** one small registered native transformation can be constructed, inspected
and executed into an immutable completion. Caller-supplied batches cannot be
substituted into it, and changing a source, role, parameter or semantic context
cannot retain the wrong guarantee.

### HP03 — derive semantics and compile one set of obligations

Implement the minimum transfer rules above using exact field identities and scoped
input properties. Generate full domain/quantity metadata from registry declarations;
implement field-aware function/analyzer hooks where native storage types are
insufficient. A metadata alias cannot act as a unit, basis, reference or ordinal
conversion. A dropped key or changed join premise removes the affected guarantee.
Use native schema/FD propagation where correct, supplemented by PSE meaning rather
than a second native-type interpreter.

Compile registry invariants once into native plans: required/valid values, enum
membership, quantity coupling, PK/unique keys, composite/contextual FKs, cardinality,
coverage, ordinal bounds, nested constraints and consumed domain invariants. PK/FK
declarations already exist; replace both the Cell/set engine and its competing
runtime interpretation with this one compilation route. Generate diagnostics with
actual subject/occurrence keys and typed reasons. Use `IS NOT TRUE` where false
and unknown are both invalid; model optional references and null parents explicitly.

Candidate sources advertise only established facts. In particular, uniqueness
diagnostics cannot optimize against the unverified uniqueness they are checking.
Discharge a property through a sound constructor, inherited fact or completed
diagnostic. Do not repeatedly prove it over immutable unchanged data. A new fact
is unavailable to consumers until its prerequisites complete. Global diagnostics
and private output streams share bound inputs/materializations; valid-looking
early batches cannot mint an admitted whole result.

Replace the restricted RulePlan/RuleExpr runtime algebra and independent typing
interpreter. Keep domain declaration syntax only where it is a useful authoring
frontend; allow registered native-plan implementations without extending a closed
enum for every engine feature. Both routes use the same operation contract and
common preparation. Delete output-field “repair” that attaches meaning without a
derivation; necessary schema projection/lowering must be part of the observed plan.

**Exit:** a total lookup, projection, migration and generated integrity obligation
can explain their premises and results. Wrong semantic domains fail during
construction; unresolved duplicate/missing/null cases produce the specified
witnesses. Tests use mathematical expectations, not the removed validator.

### HP04 — execute the native pipeline with owned streams

Use the pinned normal analyzer/logical/physical optimizer pipelines plus named
PSE semantic rules. Resolve known concrete semantic defects locally, including
correct multiplicity lowering for affected ALL-set operations; do not keep the
narrow profile or exclude whole engine families. Do not require comparison with
the discarded optimizer profile. Planning success is a structural milestone;
the final native physical plan and runtime observations identify actual execution.

Replace provider-side bulk filter evaluation during physical planning with native
execution nodes. Projection must retain columns needed internally by pushed
predicates, and Exact/Inexact/Unsupported claims must match actual semantics.
Expose supported statistics and constraints only when established. Metadata/file
discovery needed for planning is explicit; the in-memory provider performs no bulk
row execution merely to construct or explain a plan.

Return owned streams and completions. Delete unconditional intermediate collection,
deep copies and workspace reconstruction. Materialize deliberately at shared or
persisted boundaries and carry the same owners into subsequent plans. Preserve
physical semantic fields through the registered lowering/physical-rule path and
record the final executed plan after those transformations.

Implement cancellation that wakes pending streams and propagates to owned async
work. Release private buffers on failure/drop; retained exported arrays keep their
leases. Use shared DataFusion memory/spill accounting plus explicit platform
allocation ownership, without claiming control of every process allocation.
Replace hard-coded row/object ceilings and fixed phase-0 compression/timezone/thread
restrictions with the target typed configuration and semantic-purpose requirements.
Canonical mathematical policies remain explicit where required.

**Exit:** plan inspection does not execute the in-memory data path; native streaming,
multi-partition execution, field meaning, cancellation and actual allocation
lifetimes satisfy the declared contract. Current-format large relations are limited
by configured resources and real representation limits, not historical tiny caps.

### HP05 — rebuild source coupling, changes and P0–P3

Use grammar/AST code where it fits; rewrite the orchestration around one owned parse
of each changed document. Parsing produces typed declarations and occurrences with
document identity, lexical scope, source span, source field/key and resolved semantic
identity. Plan-based ownership/name/domain/reference joins resolve bindings and
emit zero/multiple-match diagnostics. Reuse unchanged parsed owners directly.

| Operation | Target implementation |
|---|---|
| P0 packages | Dependency rows/unnest, duplicate aggregation, missing-target anti joins and explicit pin compatibility; ordinary version parsing and a named cycle/depth algorithm where advantageous |
| P1 binding | Native joins between parsed occurrences, declared owners, prospective instances, domain members and typed target selectors; keep grammar and lexical interpretation in the parser |
| Change staging | Keyed full outer joins with explicit side-presence fields and exact value comparisons; batched insert/delete/update relations with declared operation ordinals |
| Change application | Apply current-format operations under their ordered before-image contract; multiple edits of one key are not silently collapsed into last-write-wins |
| Rename | Binding-aware source edits; parse changed bytes once; derive rows and bindings from that result; stable semantic IDs and explicit treatment of renamed declarations/occurrences |
| P2 | One generated admission program over complete bound candidate context, producing admitted model inputs or structured findings |
| P3 | Native projection/normalization/seed enrichment; AST lowering, quantity operations and MathIR graph construction behind explicit batch contracts |
| Declared schema evolution | Native projection/literals/aliases/conversions for current target source/target versions; widening nullable needs no scan, tightening needs a fact or null obligation |

The last row preserves explicit schema evolution as a product capability; it does
not create a migration service for pre-pivot stores. Declare new demonstration
versions when testing the feature. Before/after comparison is a declared operation
over user data, not a validation campaign against legacy code.

Delete verification-only ChangeSet construction, repeated internal stage/apply/
decode/rebind/proof-capture chains, and duplicate source inventories. Local source
correspondence is retained by the constructor that produced the rows. An arbitrary
external source/row pair still establishes correspondence from the actual parse
and declared projection before it is admitted.

**Exit:** a newly authored model, an explicit-ID rename and an ordered change set
travel through new P0–P3 with exact expected identities/rows/source bindings. Named-ID
rename rules, lexical shadowing, stale before-images and missing targets follow
their declared semantics. Local producers and changed-document parses execute once
per required computation; no verification replay follows normal local completion.

### HP06 — current storage, publication, reuse and terminal records

Rework catalog publication to consume complete admitted results from the common
executor. Reuse the existing actual conditional-write/local atomic-replacement
mechanisms if they fit; rewrite adapters rather than preserving their former APIs.
Complete output ports and terminal evidence precede the mutable ref transition.
No partial stream, pending obligation or failed attempt publishes a successful
result. Preserve distinct conflict, failed preparation and visible-but-durability-
uncertain outcomes; a lost response is reconciled against exact intended ref state.

Keep immutable current-format artifacts and strict codecs. Bind source inventories,
producer/output roles, schema versions, actual parents and semantic context. Retired
formats return a clear unsupported-version error. Rebuild repository development
stores and goldens from target inputs; no archival fixture must remain readable.
Do not delete unrelated user data merely to clean the checkout.

The current manifest selects a checksum-addressed admission-binding artifact with
exact parents, producing pass, selected policy references and sealed engine settings
(ADR-0067). The artifact has no target-manifest reference. Reopen admits its actual
dependencies and restores settings against available implementations. Delete the
target-keyed receipt and the compiler's default-policy import fallback together;
neither is a supported predecessor format. Keep logical membership framing intact.

Current-format external import/reopen remains a real trust boundary. Decode actual
content and establish its required intrinsic/domain/relational properties with the
same generated programs. Where a claimed derived artifact requires correspondence
to authored inputs that its contents alone cannot establish, compile the current
declared producer through the same native route and establish that correspondence.
Use this only at the boundary requiring it, never as automatic replay of each local
write, scan or stage. There is no second hardcoded producer-dispatch validator and
no legacy engine comparator. A saved “valid” marker or matching hash cannot restore
an in-memory construction guarantee after restart.

Opening performs this documented read-only admission before returning an admitted
handle. Any necessary producer-derived comparison stays private and cannot create
missing model outputs, publish a new stage/ref, or silently turn an incomplete store
into a compiled model. Subsequent table inspection reuses the admitted immutable
owner and does not rerun that admission. Make this lifecycle and its potential cost
explicit in the current API documentation.

Capture dependencies during binding and native plan construction, before
optimization can remove them. Include subqueries/correlation, parameters, source
roles, missing/empty inputs, candidate universes, default/shadowing lookups, negative
scopes, reference/quantity definitions, policies, function implementations and
native-operation reads. Constant folding retains the dependencies that justified
the literal. Distinguish source support from truth identity and physical location.

Share complete immutable context owners rather than copying registry/source/policy
Cells per stage. Use complete-stage reuse as the initial granularity. A hash may
find a candidate; actual input/context equality and required current-format
admission establish eligibility. Compatible old preparations stay bound to their
own exact inputs. New execution gets a new attempt record even when results are
reused. Persistent reuse cannot bypass its declared current-format admission.

Generate terminal diagnostics and pass records from one declaration, retaining
actual findings, failure class, pass/attempt identity, output state and source
support. Failure to record an attempt is explicit alongside the original error.
Failure of a later auxiliary hint/index does not rewrite a completed success into
a contradictory failed stage.

**Exit:** current-format publish/reopen, warm reuse, restart reuse, dependency
mutation and interrupted publication satisfy independently specified states.
Discarded formats and incomplete results are not accepted accidentally.

### HP07 — relational rule execution, conflicts and support

Represent accumulated facts, input deltas, candidate heads, unknowns, conflicts,
support edges and settled negative scopes as typed Arrow relations. Native joins,
aggregates, windows and differences implement matching, grouping, precedence,
conflict detection and state updates. Delete row-map/literal-string fixed-point
state, individual-row batches and the bounded-UNION-only compiler/executor.

Preserve four-valued rule meaning, with SQL null handled explicitly at the relevant
expression boundary. Equal keys with incompatible payloads are conflicts; never
hide them using `DISTINCT`, first/last writer or a hash. Several independent supports
for one truth remain represented. Recursive support uses finite distinct
rule/source edges, not every path around a cycle. Whole-stratum conflict decisions
complete before downstream consumers can claim settled input properties.

For the required finite inference domain, implement positive monotone closure with
real input-delta plans and exact new-row/conflict/support computation. A changed
support set can matter even when truth rows do not change. Current/higher-stratum
negation is rejected under this rule contract. Native recursion remains available
where it expresses the required behavior; keep only a small iteration driver for
fixed-point/stratum transitions, cancellation and resources. Exhaustion is a typed
incomplete failure, not convergence. Finite PSE inference constraints do not limit
DataFusion use elsewhere.

Handle deletions, changed facts and lost supports by invalidating and recomputing
the affected complete stage/strata from their remaining authoritative inputs,
including downstream negative scopes. Positive input-delta plans accelerate
closure within that fresh computation. Do not use support counts alone to retract
facts in recursive cycles, and do not add a differential-dataflow or fine-grained
incremental-view-maintenance project. A cycle with its last external support
removed must disappear from the recomputed closure.

**Exit:** hand-specified cyclic closure, multiple support, negative-scope, conflict,
unknown and retraction cases produce their exact expected relations. Input-delta
execution is visible in the actual plan. No PSE rule feature ceiling duplicates
the native engine surface.

### HP08 — reference packages and physical adapters

Reuse existing authored reference declarations when independently justified by their
sources and target schema; replace incidental format/glue as needed. Ship the
reference scope in Decisions, with explicit package dependencies/IDs and located
scientific evidence. Do not copy IDAES source, comments or docstrings. Author
formulas and representative numerical expectations from permitted primary sources.

Generate leaf-crate test/reference projections from the one standard data source.
Generated representations must agree with their current declaration; that is not
evidence that coefficients or physical conventions are scientifically correct.
Validate dimensions, reference conditions, affine point/difference behavior,
composition bases and method formulas independently. Synthetic data may test
mechanics but does not satisfy the promised sourced physical package.

Build one immutable quantity/material inventory from admitted reference rows. Use
native lookup/composition plans and vectorized conversion/check adapters; delete
repeated inventory decoding or externally supplied-map reconciliation inside trusted
local paths. Keep exact rational/affine/reference algebra as ordinary domain code.

Generate typed MathIR column views/builders. Use plans to select/order graph
relations and join domain/binding context. Preserve roots, ordered arguments,
payload alternatives, binders, occurrence typing and full physical types through
the native algorithm boundary. DataFusion expressions do not replace D6 MathIR.

**Exit:** the standard packages load through the new authoring route and supply all
required state/property/law/template declarations with independent physical cases.
No duplicated defaults, hand-maintained standard inventory or fixture-only context
is needed by production compilation.

### HP09 — features, topology and property demand: P4–P6

**P4:** resolve inherited dynamic/holdup configuration, typed feature implications/
exclusions, useDefault balances, phase/species restrictions and method compatibility
with native plans and declared rules. P3 supplies finite prospective scopes; P4
does not read future P7 outputs. Represent absent restrictions, inherited unknowns
and competing facts explicitly. Method names are not evidence of a provision.

**P5:** bind actual finite child instances/domains, containment, selectors, port
members, topology and boundary crossings. Native joins/unnest/domain products do
the relational work. Distinguish invalid containment cycles from allowed physical
recycles. Compute typed heuristic tear candidates with deterministic tie behavior
and verify the selected edges break cycles; use an ordinary graph library where
advantageous and claim no unimplemented optimization. Port expression members and
quantity/domain compatibility are complete. Boundaries use actual participation/
cut sets, including internal cancellation in containing scopes. Implement tags
only if a shipped selector consumes them; otherwise return the declared unsupported
selector outcome.

**P6:** derive all property demand from P3 seeds, P4 guards/candidates and P5 scopes:
equation, law, port, display, initializer-only and transitive method requirements.
Choose from the complete bound candidate universe with declared precedence and
explicit unresolved/ambiguous outcomes. Retain all requesters and positive/negative
support. Resolve to a finite least fixed point before realization. No P7–P9 producer
may silently introduce an undeclared late demand and restart the stage. Property
inspection reads resolutions without creating new physics.

**Exit:** independent small models establish exact feature decisions, scopes,
ports, topology, candidate inventories, selections and requirement/support closure.
Candidate addition or a previously absent input changes dependencies even if the
selected output value happens to remain the same.

### HP10 — templates, laws, methods and canonical mathematics: P7–P10

**P7:** implement generic parameter/domain binding, decided guards, aliases, indexed
symbols, expression members, indexed equations and contribution realization.
Native plans expand and match declarations/instances/domains; specialized AST/MathIR
steps construct the declared mathematical operators. No unit-name dispatch creates
heater/mixer physics. Identity derives from actual semantic inputs with explicit
uniqueness handling; shared expression storage never erases occurrence provenance.
Keep gathers, reductions, domains and filters indexed; P12 scalarization remains later.

**P8:** realize laws from complete contribution candidates and scope/subject/domain
matching. Native joins/grouping/order assemble typed expression descriptors and
included/excluded participation with reasons. Build residual MathIR under declared
arithmetic/guard policies; do not numerically sum solver values as a substitute
for symbolic conservation. Establish applicable participation exactly once,
internal-transfer cancellation and complete negative evidence. Physical conversions
carry their actual basis/reference data. Equality connections bind actual port
members and realized symbol groups with indexed correspondence. Mixer balances
derive from conservation and outlet-state declarations, not pairwise inlet-
temperature equalities.

**P9:** realize selected equation-template methods, reconcile promised provisions
against actual expressions/symbols through the same bound construction, bind ordered
parameters and insert natural-unit conversions once. Missing output and undeclared
requirements produce the declared errors. Kernel descriptors bind actual signature,
domain, parameter order, outcome/null/error/derivative requirements and available
implementations; absent execution backends remain explicitly unavailable.

**P10:** consume the real P9 and full physical/domain/reference/kernel contexts.
Use native plans for graph correspondence, relational normalization and typed
selection; keep D6 canonicalization/occurrence typing as explicit native algorithms.
Establish acyclicity, constant/conversion validity, residual quantities, shapes,
domains, root and argument ordering. Do not bind case values early. Emit the full
current-format canonical indexed graph and complete source/derivation relations.
Retire fixture-only production pass routes and both hardcoded producer rosters.

**Exit:** the actual P3–P10 graph closes with declared producer ports, explicit
empty outputs, complete physical meaning and derivations. A second unit/template
using existing constructs requires declarations, not another procedural builder.

### HP11 — immutable Python and semantic inspection

Bind current Rust/Python handles to exact admitted snapshots and the shared runtime.
Generate the boundary contracts/stubs; expose named table streams through the
pinned supported pyo3-arrow path. Opening completes the declared read-only admission
in HP06 before exposing the handle. Its private correspondence computations do not
publish physics or repair missing stages. Inspection of the admitted handle never
mutates model facts, resolves new properties, constructs new equations or starts
compilation implicitly.

Retain the result/source owner and allocation lease through exported arrays, even
after a handle closes. Define close, partial drain, cancellation, exception and
foreign-buffer behavior explicitly. Re-entry of external/degraded data establishes
its actual field/value obligations. Do not infer consumer extension support from
successful capsule export. Copies needed for ownership/conversion are explicit;
routine internal copies are removed by HP04.

Expose native typed plan inspection, stage/operation/source mapping, structured
diagnostics and actual execution observations through the existing Rust/tooling
surface. EXPLAIN text is a view of the actual plan. Plan serialization is not a
semantic identity; if a current use exposes durable decoding, bind its real codecs/
sources/functions and test that supported subset. No general plan portability
project is required.

**Exit:** freshly produced graphs reopen and stream through Rust and Python with
correct metadata, ownership and errors. Current host and pinned parity interpreter
contracts work, including Python 3.14 annotation evaluation and import boundaries.

### HP12 — validate the new product from its contracts

Create fresh target fixtures and expected relations from small, independently
reasoned engineering examples. The mandatory workflow uses both FTPx and FcTP
configurations of the declared ideal heater/mixer packages from source documents
through real compilation, persistence, reopening and Rust/Python inspection.
Reference graph expectations include selected methods, indexed equation/root
structure, physical types, contributions/exclusions and source support. A graph
fabricated directly by a helper cannot satisfy this workflow.

Run the Verification subjects below. New-code algebraic/metamorphic checks such
as batching/repartitioning, rename identity, equivalent declared representations,
current codec round trips and new-runtime cold/warm results are appropriate where
they test a target contract. None requires the old implementation. Existing IDAES
behavior/parity obligations remain separate external product evidence; no additional
old-Rust equivalence program is introduced.

Measure only the target code: construction/analysis/optimization, first result,
complete execution, publication/reopen, changed-source parse counts, producer
invocations, row decoding/copies, materialization, memory and spill. Use small
interactive changes and larger/skewed workloads on recorded thread/batch/budget
settings. Use one coordinator with a workstation-sized budget. Tiny refusal tests
remain separate, deliberate failure cases.

The success criteria are coherent target semantics, actual plan-visible relational
work, no repeated local producer validation, and reasonable target-workload behavior.
Do not build an old benchmark baseline or promise a universal speedup/optimal plan.
Investigate material target-code costs and fix avoidable duplication before closure.

**Exit:** the new workflow's complete intended results are established, including
failure/unknown paths; representative target measurements and allocation ownership
are recorded with their actual conditions.

### HP13 — delete the remainder and close the delivery

Finish the deletion inventory below, remove obsolete dependencies/imports where
appropriate, regenerate current artifacts, and update active docs/examples/tooling
to the sole target API. Historical ADRs/reviews/receipts stay as historical evidence;
they must not direct execution into deleted compatibility routes. Regenerate fresh
target goldens without keeping historical byte fixtures as gates.

Run the terminal commands once over the complete target. Fix failures against the
zero baseline; do not waive implementation faults as legacy noise. Perform one final
focused design review of actual G1–G7 enforcement and the deletion/consumer inventory.
Do not reopen the entire research exercise or add new scope without a concrete gap.
Record commands, modes, counts, actual artifact/implementation identity and remaining
future-wave boundaries. Complete Outcome only after the exit evidence exists.

### Explicit replacement and deletion inventory

Paths below identify current code to replace or prune, not files that must remain
unchanged. Delete code as its replacement lands; the final package checks for leftovers.

| Current path / mechanism | Target owner | Remove / replace | Completion package |
|---|---|---|---|
| `pse-schema/src/codegen/rust/relation.rs`, generated views/builders | Generated typed Arrow access | Single-row batch pushes, admitted-view rescans and mandatory row serialization | HP01 |
| `pse-schema/src/model/{rule,rule_expr,rule_validation}.rs` | Domain frontend plus native preparation | Closed runtime engine algebra and duplicate native-type inference; keep only useful authored semantics | HP03 |
| `pse-relations/src/validate/bundle.rs`, row-oriented value validation | One generated admission program | PK/FK/ordinal Cell/set interpreter and duplicate domain predicates | HP03 |
| `pse-relations/src/migrate.rs` | Declared native transformation plans | Row mutation interpreter; no legacy-store conversion replacement | HP05 |
| `pse-catalog/src/session/{admission,snapshot_session,physical_fields,profile}.rs` | Common preparation/execution | Repeated source value admission, narrow profile, detached field repair and raw collected-result trust boundary | HP02–HP04 |
| `pse-catalog/src/provider/table.rs` | Native scan/filter/projection execution | Bulk filter evaluation during planning | HP04 |
| `pse-runtime/src/{budget,cancel}.rs`, allocation adapters | Configurable owned execution | Fixed phase feature/size ceilings and non-wakeable cancellation; retain valid checked allocation mechanisms | HP04 |
| `pse-authoring/src/{p0,p1,document/binding,document/rename,change_set}` | Owned parse + native binding/change plans | Verification-only ChangeSets, duplicate inventory builds, row-map diff and repeated local proof copies | HP05 |
| `pse-compiler/src/passes/{mod,bundle}.rs`, `validator.rs` | Single prepared producer registration and completion | Public successful output assembly and second producer dispatch/replay roster | HP02, HP05–HP06 |
| `pse-compiler/src/validator/{sources,compare}.rs` | Current boundary obligations through common native compilation | Legacy-shaped source staging/comparison; keep only actual current import requirements | HP06 |
| `pse-compiler/src/memo/context.rs`, driver/commit preparation | Exact shared contexts and explicit publication | Per-stage full Cell/source copies, verification staging, hash-based admission if present | HP06 |
| `pse-rules/src/{plan,exec,strata}` | Native rule plans and Arrow state | Bounded UNION ceiling, literal-string/row-batch state and separate completed/constructed replay framework | HP07 |
| Quantity/MathIR/compiler adapter row scaffolding | Generated columns + explicit algorithm contracts | Duplicate unit inventories and handwritten relational joins/grouping around native algorithms | HP08–HP10 |
| Handwritten manifest/DTO/default inventories | Registry generators and packages | Competing declarations and pre-pivot wire compatibility | HP01, HP08 |
| Old golden stores, legacy-store migration/equivalence tests, narrow-profile comparisons | New target fixtures and first-principles cases | Legacy acceptance baseline, fallback tests and byte-compatibility gates | Each replacement, final HP13 |
| Old active instructions/hooks/tests requiring deleted architecture | Current blueprint/contracts and structural checks | Stale feature bans, replay requirements and file-name-based legacy assertions | HP00, HP13 |

This inventory does not prohibit `Cell`, hashing, scalar code, buffering or custom
algorithms in every context. Remove the identified architectural duplication and
mandatory row paths; use those tools where their specific boundary or algorithm
has a distinctive advantage. No new generic ban takes the place of the old one.

### Coverage of the complete review and prior product scope

| Source obligation | Packages that deliver it |
|---|---|
| Review LP0 / decisions and declaration ownership | HP00 |
| Review LP1 / prepared computations and result ownership | HP01–HP02, HP04, HP06 |
| Review LP2 / semantic field/property derivation and obligations | HP01–HP03 |
| Review LP3 / columnar source, migration and change construction | HP01, HP05 |
| Review LP4 / native pipeline, providers, streams and budgets | HP02, HP04 |
| Review LP5 / rule and real compiler consumers | HP07–HP10 |
| Review LP6 / complete dependencies, reuse and publication | HP02, HP06–HP07 |
| Review LP7 / deletion and final product qualification | Every replacement, HP12–HP13 |
| Review I1–I3 / identity, schema, relations | HP00–HP03, HP05–HP06 |
| Review I4–I5 / catalog and runtime | HP02–HP04, HP06, HP11 |
| Review I6–I7 / quantity, material, MathIR | HP03, HP08, HP10 |
| Review I8–I9 / authoring, rules, compiler | HP05–HP10 |
| Review F1–F4 / raw-result trust, duplicate invariants, semantic fields, frozen context | HP01–HP03, HP06 |
| Review F5–F8 / planning execution, row reconstruction, dependencies, source coupling | HP04–HP06 |
| Review F9–F12 / materialization, recursion/effects, identity/commit, evidence | HP04, HP06–HP07, HP12–HP13 |
| Earlier review C0–C5 / full native surface, semantic lowering and runtime/Arrow integration | HP00–HP04, HP07 |
| Earlier review C6–C7 / domain consumers and closure | HP08–HP13 |
| Plan 04 W2-01–03 / consumed schemas, reference packages, manifest | HP00–HP01, HP08 |
| Plan 04 W2-04–09 / complete P4–P10 | HP07–HP10 |
| Plan 04 W2-10–12 / Python, product workflow and closure | HP11–HP13 |
| Plan 04 W2-00 / legacy predecessor receipt reconciliation | Removed by maintainer direction; no discarded-code qualification |

## Verification

### First-principles acceptance subjects

All subjects below are **Proposed** until executed against target code. Implement
focused tests in the existing owning crate/test families; these are semantic
subjects, not a requirement to create another test framework. Baseline is zero
failures, warnings and unexpected skips. Keep evidence labels claim-specific.

| Subject | Independent expectation / falsifying case | Target package |
|---|---|---|
| `native_semantic_construction` | Hand-declared member/unit/quantity fields: wrong domain fails before data execution; direct projection retains meaning; key loss and outer nullability remove invalid assumptions | HP02–HP03 |
| `admission_property_scope` | Duplicate/null PK, absent FK, contextual/composite mismatch and nested null-mask cases have exact expected witnesses; candidate plans never advertise the assumption being checked | HP03 |
| `completion_ownership` | Altered output, wrong source/role/context or unfinished stream cannot create admitted completion; dropped owners do not invalidate live exports; foreign mutable memory is isolated | HP02, HP04 |
| `declared_transforms` | Small exact projection/default/nullable/conversion examples; lookup cardinality proof premises; bag counts `[1,1,2]` with `[1]`; unnest parent/ordinal identity and grouping-set distinctions | HP03, HP05 |
| `native_execution_contract` | Preparation/EXPLAIN invokes zero in-memory row kernels; actual partitioned execution obeys metadata/null/error policy; exact pushdown retains every required result | HP04 |
| `source_binding_and_changes` | Manually specified identities and lexical bindings survive an allowed rename; changed text parsed once; stale preimage, shadowing and repeated-key operation order produce declared results/errors | HP05 |
| `current_artifact_lifecycle` | Fresh current-format round trip preserves declared meaning; malformed/retired formats reject; late violation, CAS conflict, cancellation and uncertain durability produce their specified states | HP06 |
| `complete_dependency_reuse` | Changes to folded definitions, absent candidates, source role, parameter, function, policy or native read cause the required new preparation; unchanged exact context supports reuse | HP06 |
| `finite_truth_and_support` | Explicit finite closure tables, truth/unknown/conflict cases, negative scopes, independent supports and retraction define expected new runtime outcomes | HP07 |
| `physical_reference_contracts` | Independent dimensions, affine/reference conversions, sourced cp/h/s/density calculations and domain failures; metadata or generated equality alone is insufficient | HP08 |
| `feature_scope_demand` | Small model declarations determine exact features, port domains, graph distinctions, candidate selections and multi-requester demand closure | HP09 |
| `template_law_method_graph` | Independently specified symbol/equation families, indexed roots, contribution participation/exclusion, natural-unit conversions, method provisions and kernel availability | HP10 |
| `immutable_inspection` | Python/Rust streams retain actual fields, values and owners; close/drain/error/cancel and degraded re-entry follow the declared boundary without implicit physics | HP11 |
| `target_heater_mixer_workflow` | FTPx and FcTP source models compile through real P0–P10 to the expected indexed graph and complete provenance, then reopen in both inspection surfaces | HP12 |
| `target_work_accounting` | New implementation measurements show where time/memory/work occurs; local producer invocation is once, established properties are not rescanned, and bulk relational work is plan-visible | HP12 |

Written premise/conclusion arguments accompany the small semantic transfer rules.
They explain why internal rescans can be deleted. Tests check the implementation
of those rules and the assumptions at each boundary. Do not describe a successful
`LogicalPlan` constructor or ordinary tests as a general formal proof. Arbitrary
external values, custom kernels and durable effects still have explicit contracts.

Use hand-reasoned expected outputs, primary scientific sources and selected
algebraic/metamorphic properties. New-code round trips, optimized-plan behavior and
cold/warm consistency check the target's own contracts; they are not permission to
reintroduce the legacy implementation as a comparator. Expected scientific results
must not be generated solely by the same formula implementation being tested.

### Command schedule

| When | Command / evidence | Scope |
|---|---|---|
| Implementation start | `just doctor`; relevant bootstrap recipe only for needed missing tools | Record actual tooling; do not rebuild the old extension solely to certify a discarded baseline |
| Replacement package | `just test-package <owner>` with appropriate filters; `just check` when integration requires it | Focused locked target tests with explicit Arrow `force_validate` through the recipe; adapt obsolete tests rather than preserving old APIs |
| Generator changes | `just codegen` / existing bootstrap route, then `just codegen-check` | Fresh target output and actual regeneration equivalence; no manual generated edits |
| Dependency changes | `just family-check` | One pinned Arrow/DataFusion type universe; library admission remains permissive |
| HP11 target boundary ready | `just py-sync`, `just doctor`, focused `just py-test` | Refresh the editable target extension before Python behavior tests; resolve the stale-environment issue here |
| Product evidence | Target fixture command and target benchmark recipe added to `just --list` if missing | Extend existing xtask/bench tooling with portable paths and tested argument handling; record target-only results |
| Complete target once | `just ci-pr` | Actual current recipe: Rust formatting/check/Clippy/default tests/doctests, governance/codegen/family, rustdoc, benchmark smoke, quality, ADR/docs and Python tests |
| Optimized target paths | `just test-release` once at closure | Required optimized Rust behavior with Arrow force validation; separate receipt from default mode |
| Exercised external parity | `just parity-container` | Existing pinned IDAES/interpreter/container contract and actual exercised scope; no invented numerical solve claim |
| Final documents | `just docs`, `just lint-typos`, scoped `git diff --check` as needed after final edits | Documentation validity, not runtime certification |

Do not run `just test` again solely because `just ci-pr` already ran it successfully.
Use the actual recipe definitions at execution time and record any unavailable
command truthfully. Do not add wheel/sdist builds, broad unused-feature powersets,
old-profile comparisons or requalification of deleted code. Target-specific changed
feature paths are tested where used; a broad feature campaign is not a new blocker.
Dependency advisory reports remain advisory under the current policy.

Record every terminal command's mode, failure/skip/warning count, baseline zero,
and what it actually proves. Benchmark smoke is executable coverage; **Measured**
claims require real recorded timings/resources and conditions. Size/concurrency
settings should exercise the intended workstation without turning all tests into
memory stress tests. Resolve warnings in the delivered scope rather than creating
a baseline waiver.

### Gate closure

| Gate | New implementation must establish |
|---|---|
| G1 | One semantic declaration and native execution representation; no competing validator/producer/default registry |
| G2 | Full semantic field/quantity/identity/domain/multiplicity/absence meaning survives the declared construction and physical boundaries |
| G3 | Facts can arise only from admitted inputs, sound construction or completed obligations; raw/partial/invalid results cannot be promoted |
| G4 | Exact sources/functions/configuration/effects are bound; preparation and inspection do not hide data execution or mutate physics |
| G5 | Complete owned execution results feed explicit current-format publication; failure, conflict and uncertain outcomes remain distinguishable |
| G6 | Rewrites and reuse obey actual semantic contracts and complete construction-time dependencies, including absence and folded inputs |
| G7 | Every delivered feature has a real native or explicitly specialized route; unknown/unsupported behavior is truthful; proof/performance claims match evidence |

The final review evaluates these against the charter's full gate definitions. The
delivery is complete only when all seven required gates have evidence for the
claimed scope and HP00–HP13 are complete. A successful schema generator or leaf
probe cannot close the real source-to-graph workflow.

### Planning-document evidence

**Proposed (planning baseline):** the original planning turn created the plan and
execution cross-links without claiming a product-code receipt. Implementation was
subsequently authorized and is tracked below.

**Tested — document only, 2026-09-14:** `just lint-typos` passes with zero findings
against baseline 0. `just docs` builds successfully with one existing nonblocking
large-search-index warning; it is not a zero-warning product receipt. Scoped
`git diff --check` is clean. A read-only `.venv/bin/python` structure/link check
verified 14 unique work packages with 14 detail sections, dependency order,
26 existing ADR references and 36 local links across five affected documents:
zero failures, baseline 0. There is no plan-coverage recipe; the check was scoped
to this document task and introduces no new production validation framework.

An independent read-only plan audit found and corrected two ambiguities: opening
must finish its declared read-only admission before pure inspection begins, and
deletions/support loss use complete affected-stratum recomputation rather than
addition-only closure. This is planning evidence, not implementation acceptance.

## Open items

There is no scope or permission question blocking this plan. HP00 owns concrete
contract amendments and HP08 owns any missing primary scientific sources. Routine
implementation choices follow the approved architecture and do not require another
planning phase. If research exposes a missing required semantic contract, author
it at its owning declaration before implementing the consumer; do not silently
substitute an arbitrary default or declare the required feature complete.

New ADR numbers, any necessary target wire-version change, final internal type names
and workload-tuned resource defaults are resolved while executing their owning
packages. No legacy-compatibility decision is pending. No new crate is assumed;
introduce one only if a concrete dependency/ownership benefit warrants it under the
existing decision rules. All library capabilities remain eligible.

Future problem construction, numerical backends, solving and wider physics retain
their explicit later-wave boundaries. No commit, push or publication is performed
by creating this plan.

## Implementation continuation — 2026-09-14

**Implemented, incomplete:** execution resumed after the reboot checkpoint. P3
now accepts the actual `InputBundle`, `OwnedDocumentSet`, native session and shared
physical inventory. Its primitive, configuration, source, selector, port and product
construction retains native outputs. Source configuration carries per-row origins;
algorithm occurrences include the exact input port. Native projections carry each
typed row, key and selected override in one execution. Demand syntax emits transient
read occurrences; native joins select property mappings and an ordered recursive
plan expands opaque-operation indices. The former demand mapping/product loops and
separate port output/evidence planner are deleted. P3's complete behavior remains
unverified while downstream compiler integration is unfinished.

**Implemented:** field declaration transport uses native `with_metadata` after
checking input meaning. The reusable `pse_preserve_field` UDF retains its argument's
complete field and restores only missing nested metadata during scalar expansion;
changed child layout or meaning is an error. Shared preparation places this immutable
call in final logical projections after constant folding, and records the operation.
The old `AdmittedField` physical-expression wrapper is deleted. A narrow physical
Cartesian-operator selection remains because that engine operator scalarizes values
before an output UDF can observe them. Schema-level relation headers remain the
responsibility of the declared projection boundary. This does not make metadata a
proof of quantity conversion, keys, references or scientific validity.

**Implemented, incomplete:** native result storage retains its allocation claim;
successful checked materialization is cached in the actual completed computation.
Repeated access shares its checked fields and concatenated buffers. The first
materialization still executes local value admission; complete property transfer and
cross-provider reservation accounting remain required.

**Implemented, incomplete:** P4 predicates and index evaluation now use generated
rows and keys from the same native projections. Native finite products replace the
handwritten predicate Cartesian loop. P5 ports use parsed steps, actual owner/child
joins and retained native frontiers; ordered named-axis binding is shared with P3
in `native_construction.rs`. P5 paths and tear selection use typed algorithm inputs
and native outputs. P9 augmented inference invokes these same P4/P5 producers once;
its native evidence union retains both completed rule families and original IDs.
The old predicate/structural replay constructors, unlinked P5 native-port copy and
unused `inferred.port_binding_walk` declaration are deleted.

**Implemented, incomplete:** P6 requirement tuples, scope subjects, tagged dependency
maps and parameter source-coordinate projection use native joins and list operations.
Its read adapter shares the P4 coordinate evaluator and native binder products.
Generated read/failure outputs retain actual source occurrences; native joins select
compatible requirement axes. The old P6 `WorkspaceConstructor` and generic structural
row helper are deleted. P4–P6 have not passed behavioral tests. Native planning,
empty/frontier behavior, exact dependency completeness and resource claims still need
functional qualification after P7–P9 compile integration.

**Implemented and Tested:** native witness validation now checks the retained actual
source owner independently from a relation-only table binding. Ordinary rule scans
still require that binding. Native key joins reject a row from a different role even
when both roles name the same relation. Raw external admission reserves decoding
scratch before isolation/validation; repeated immutable input admission does not
allocate another value-validation inventory. Sealed settings include the pinned
DataFusion runtime entries reported by `df_settings`.

**Implemented, incomplete:** P7 uses coupled typed arguments and generated output
columns. Native current-graph root mapping retains actual source keys in one
completion. Selected realization uses current mathematical inputs; old public
map-based selection and before/after graph comparisons are deleted. Source graph
loading and coordinate evaluation share the actual allocation owner.

**Implemented, incomplete — 2026-09-15:** P8 typed law arguments, checked current
graph/root projections and generated connection/coefficient outputs replace its
Cell inventory, generic output copy and manual derivation fan-out. Coefficient
values and member/eligibility/species/composition keys share one native completion.
P9 parameter/method/kernel adapters and outputs use the same native substrate;
parameter-coordinate matching uses native joins. Pass declarations transfer only
actual mathematical results, not temporary P8 rule facts. The unused rules replay
constructor, mapping and error wrapper, P10 fixture graph/importer and associated
golden subworkflow are deleted. Independent physical test declarations remain useful
source inputs; current source-based tests run ordinary commits and Driver stages.

**Verification — baseline 0:** **Interface-checked** `just check-library pse-compiler`
(locked, dev, library) passed with 0 errors and 0 warnings. **Tested**
`just test-package pse-compiler --lib --test math_families --no-fail-fast
--status-level fail --final-status-level fail`: **22 passed, 0 failed/skipped**,
nextest default with Arrow `force-validate`, run
`a0c6e7ec-1ecb-4f0e-bad0-14c5c4e92c7d`. Actual native tests exposed and corrected
nullary UDF signature omissions and CASE metadata loss. `same_field_case` checks
branch field correspondence before the native `with_metadata` carrier; native
list slicing preserves child fields in subset construction. P3 literal-unit
provenance now includes its recorded origins, pending workflow regression.
Full package/caller migration, source-to-P10 behavior, source completeness,
allocation qualification and terminal acceptance remain open. Earlier partial
Rust-contract generation preceded the latest pass-port changes; full generation
must be rerun after consumer integration.

**Tested:** `just test-package pse-catalog --test native_output --test native_nested_output
--test filter_refinement --no-fail-fast --status-level fail --final-status-level fail`
ran 8 tests, 8 passed, 0 failed/skipped, nextest default with Arrow `force-validate`,
run `92837568-8540-4117-96d7-5dc301676027`. It exercises logical UDF visibility after
constant folding, immutable reuse eligibility, exact child metadata/value recovery,
layout/meaning refusal, buffer sharing and single/multiple-batch materialization
reuse. An initial buffer assertion mistakenly compared against raw ingress before
its intentional isolation copy; the corrected assertion compares the actual bound
immutable input. The nested fixture also needed its list logical type declared in
the registry. These focused corrections and receipts do not close HP03–HP05 or the
full plan.

**Tested — baseline 0:** `just test-package pse-catalog --lib --test indexed_arguments
--test session_admission --test session_validation_budget --no-fail-fast
--status-level fail --final-status-level fail`: 113 passed, 0 failed/skipped,
nextest default with Arrow `force-validate`, run
`5174e575-f0e6-4224-a609-682557aa3238`. This includes runtime setting inventory,
raw-admission scratch refusal, precise nested diagnostics, immutable admission reuse,
allocation release and cancellation.

**Tested — baseline 0:** `just test-package pse-rules --test native_construction
--test ordered_aggregate_unnest --test native_delta_execution --no-fail-fast
--status-level fail --final-status-level fail`: 17 passed, 0 failed/skipped,
nextest default with Arrow `force-validate`, run
`425ffe9c-aa34-4957-be4d-f20f544c2838`. Includes distinct actual source roles and
rejection of swapped row witnesses. These foundation receipts do not qualify the
new P4–P6 consumers or close the full plan.

**Implemented, incomplete — source integration, 2026-09-15:** P3–P10 declare native
plan execution. External predecessor injection and its fixture-only test are deleted,
as are the raw normalization harness and stock-package `normalize_owned` caller.
Source tests use ordinary commits and the public Driver. Current ChangeSet wire 2
stores batches with actual operation ordinals, preserves encoded row order on reopen,
checks ordinal range and complete row coverage, and reads control rows through generated
Arrow views. The actual batch commit/reopen test passed in a six-test run with five
other failing source workflows; that run is not acceptance. The failures identified
array-element field propagation and a missing fixture package unit-set binding.

**Implemented and Tested — native field functions:** `pse_array_agg` delegates native
ordered/distinct/grouped accumulation and state handling while preserving the actual
argument field as its list child. `pse_array_element` preserves the selected actual
child field and uses native selection/index/null behavior. Pinned 55.1 source inspection
and probes identified datatype-only aggregate fields and metadata loss in
`SingleRowListArrayBuilder::with_field`; the adapters retain the actual field without
that scalar reconstruction. Native schema derivation accepts only missing child
metadata established by actual expressions/sources, with layout and meaning refusal.
`just test-package pse-catalog --test native_nested_output --test native_output
--test filter_refinement --no-fail-fast --status-level fail --final-status-level fail`
passed **11 tests, 0 failed/skipped and 0 warnings**, nextest default with Arrow
`force-validate`, baseline 0, run `f4cd08a6-7db0-48b2-830f-ed77ff14d044`. The aggregate
probe covers partial/final execution across two partitions; nested selection covers
repeated scalars, negative indices and out-of-range nulls. Full compiler/rules,
source workflows, generated outputs and terminal acceptance still require validation.


### Native field and source continuation — 2026-09-15

**Tested:** the source-projection admission case passed in run
`85fc3d66-2ab0-45fc-afc5-a478c2faf8c7`. The complete command was
`just test-package pse-tests-engine --test native_template_graph --test
native_reference_packages --test source_projection_admission --no-fail-fast
--status-level fail --final-status-level fail`: **1 passed, 2 failed, 0 skipped**,
nextest default with Arrow `force-validate`, baseline 0. The test reopens an actual
committed Model and rejects changed package values without a source edit.

**Implemented:** P5 guard defaults use an explicitly checked TruthValue dictionary
literal and shared CASE/nonnull expressions. Internal phase names and the isothermal
balance equation now have unambiguous names in shipped source declarations. Source
binding and literal-storage failures include the actual offending field/name.

**Tested:** `just test-package pse-rules --test relational_execution --no-fail-fast
--status-level fail --final-status-level fail` passed **9 tests, 0 failed/skipped,
0 warnings**, run `5ed7fe27-23ab-40d5-8391-a0f1ababc58b`, nextest default with Arrow
`force-validate`, baseline 0. Native null-safe comparisons are wrapped in native
`IS TRUE`, preserving their total Boolean result while exposing non-nullability
through pinned DataFusion 55's field machinery. The regression checks both operators
against Boolean and NULL values.

**Implemented; runtime receipt follows below:** ChangeSet construction reservations
travel with envelope clones retained by candidates. The staging wrapper no longer
holds the only allocation lease. The existing detached-source lifetime test now also
checks an independently retained envelope after all staging handles drop.
`just fmt-check` passed again with **0 findings**, baseline 0.

Current integration continues through actual source commits; full HP00–HP13
acceptance remains open.

### Native integration continuation — 2026-09-15

**Tested:** the native engine integration run
`025b4505-f6ea-415c-99d7-1510b203f4f4` completed **8 passed, 4 failed,
0 skipped** in 197.244 seconds. Command: `just test-package pse-tests-engine
--test native_template_graph --test native_reference_packages --test terminal_attempts
--test memo_dependencies --no-fail-fast --status-level fail --final-status-level fail`.
All seven terminal-attempt tests and the actual declared P3–P10 dependency-chain test
passed. Source preparation now runs within the first pass attempt, so pre-cancellation
retains its terminal record; fault tests use the generated current pass-record version.

**Tested:** the subsequent focused engine run
`f74442e7-f6bf-4615-bbe8-66e1472d7064` passed all three memo dependency tests:
**3 passed, 2 failed, 3 filtered out**, 123.000 seconds. Command: `just test-package
pse-tests-engine --test native_template_graph --test memo_dependencies --test
commit_p0_p2 -E "'test(two_templates) | test(full_rename) | binary(memo_dependencies)'"
--no-fail-fast --status-level fail --final-status-level fail`. The fixture uses the
actual invariant validator with its declared diagnostic output. Remaining failures
were the rename application's absent-empty source member and P6 dependency index
construction. Both corrections are implemented; subsequent workflow results are recorded below.

**Tested:** `just test-package pse-catalog --test native_output --no-fail-fast
--status-level fail --final-status-level fail` passed **4 tests, 0 failed/skipped**,
run `1ebf4846-18a1-4d62-9a38-d1a39c05ed29`. `just test-package pse-authoring
--test rename --no-fail-fast --status-level fail --final-status-level fail` passed
**4 tests, 0 failed/skipped**, run `bb7d277a-804d-44d4-9cd4-3ce3ae6aee39`.
These establish checked dictionary CASE values and the last-reader reservation
lifetime of staged envelopes and detached Arrow buffers.

**Implemented:** P6 maps DomainKind to explicitly constructed ScopeKind values and
uses the shared `pse_index_tuple` UDF to construct its declared index outputs from
ordered actual identities. Parameter-selected source paths accept explicit qualified
enum literals; resolved paths retain their actual leaf enum, and predicate evaluation
requires exact enum identity. No predecessor graph is introduced.

**Tested:** `just test-package pse-authoring --test load_package
enum_predicates_bind_exact_known_members_and_explicit_deferred_types --no-fail-fast
--status-level fail --final-status-level fail` passed **1 test, 0 failed, 5 filtered
out**, run `6bbb05f8-b4d5-4e7e-9915-ee51dfde515d`. It exercises seven cases: known
contextual and deferred explicit bindings, undeclared members, incompatible enums,
non-enum operands, missing context and forbidden enum ordering. The preceding
complete load-package run passed its other five tests; two fixture-declaration errors
in the new case were corrected before this receipt.

All Rust receipts above use nextest **default**, Arrow **force-validate**, failure
baseline **0**. The product workflow, full workspace and Python gates remain open.


### Source configuration, current storage and demand integration — 2026-09-15

**Current result — Tested:** `just test-package pse-tests-engine --test
native_template_graph --test native_reference_packages --test native_normalization
--profile ci --no-fail-fast --status-level fail --final-status-level fail` passed
**6 tests, 0 failed/skipped**, 426.549 s, run
`5a4119a3-1a6e-4668-811e-5ad2a16690f0`. Arrow `force-validate`, nextest `ci`,
failure baseline 0. This covers four actual P3 normalization cases, shipped package
normalization and the two-template source-to-P7 workflow. The template test has
subsequently been extended through P10 with exact equation/owner checks. P8 integration
failures and their corrections are tracked in the continuation below.

**Implemented and Tested:** P3 batches source Arrow columns and exact row-source
associations through the shared output builder, then materializes each complete
relation once. Per-expression native-output branches and their unions are deleted.
The builder accepts registered private algorithm relations; final pass publication
retains its exact port inventory check. The compiler library suite passed **20 tests,
0 failed/skipped**, 2.001 s, run `7a17825e-4605-491f-b98c-433890811e2d`, with
`just test-package pse-compiler --lib --no-fail-fast --status-level fail
--final-status-level fail` (default profile, Arrow force-validation, baseline 0).
This includes appended-batch source ordinal correspondence and the actual P7 reader
inventory. A further final-port rejection test is implemented and awaiting execution.

**Implemented:** the shared physical inventory, pass declarations and P7 source
evidence now use one registered physical-input selector. The stale
`reference.quantity_algebra_rules` lookup is deleted. Shipped unit expressions use
their actual declared unit symbols. Missing compound-unit diagnostics name the
unresolved factor and declaration count.

**Tested:** `just fmt-check`, `just lint-imports lint-typos`, `just lint-agents
setup-test solver-pin-check`, and `just lint-repo` exited 0. The initial stale-stub
Python check reported 18 diagnostics; after native rebuilding and stub generation,
`just fmt-py-check lint-py typecheck lint-imports` exits 0 (zero type diagnostics,
four kept import contracts). Later native edits still require a final extension refresh.

The first batching run exposed the private/public relation boundary above:
run `1eac062c-300f-4ae5-8a52-40f85ad9767a` had **5 failures and 1 interrupted test**
(6 failed total), 211.918 s, under the same six-workflow command. It was stopped
after repeated identical construction failures; the completed green run above
follows the correction. No speedup or full product-acceptance claim is made.

**Implemented:** source binding resolves `parent.<parameter-or-feature>` in its
actual owning template and records Boolean configuration literals explicitly.
P3 consumes submodel configuration through its configuration producer and no longer
emits unused mathematical graphs for those assignments. Shipped nested balance
predicates use exact qualified enum members. Computational reference packages now
select their representation unit set explicitly. The domain-source invariant admits
the already-declared `element` alternative.

**Implemented:** stage hints require complete `StageContext` in format 3. The
context-free writer, optional context and format 1/2 acceptance are deleted. Pass
record admission reads borrowed generated Arrow columns instead of decoding a Cell
mirror. This retains the current publication and reuse contract.

**Implemented:** P6 index outputs use the registered `pse_index_tuple` constructor,
including the empty state-selection index. Rank presence is a membership condition
inside the winner join, establishing its nonnull field for every emitted assertion.
The empty-candidate optimization executes a native bounded query before constructing
support joins, and applies only when the prepared computation contains no volatile
expression. It checks the complete candidate, preserving additional source witnesses
for facts already present. Live stack samples identified support-plan construction
and repeated field admission as expensive paths; no benchmark speedup is claimed.

**Implemented:** P7 declares its actual connection inputs. P9 includes the shared
P7 reader dependencies because it invokes the same realization algorithm. A reader
test loads P7's actual declared input fields to detect missing dependencies.

**Verification — failure baseline 0:** all receipts use Arrow `force-validate`.
Default nextest profile unless explicitly marked `ci`.

| Command / condition | Result | Run |
|---|---|---|
| `just test-package pse-rules --no-fail-fast --status-level fail --final-status-level fail`; every P6 true/false/unknown candidate is now compiled | **Tested:** 53 passed, 0 failed/skipped, 11.853 s | `400a6e90-e19f-4bae-bf6e-7f0d47b2977b` |
| `just test-package pse-authoring --test load_package --no-fail-fast --status-level fail --final-status-level fail`; exact enum and configuration source binding | **Tested:** 6 passed, 0 failed/skipped, 2.750 s | `62224958-4da0-41fc-84bf-4a167ae610fc` |
| `just test-package pse-catalog --lib stage_lookup_accepts_only_the_current_complete_context_format --no-fail-fast --status-level fail --final-status-level fail` | **Tested:** 1 passed, 0 failed, 106 filtered out, 0.004 s | `4852e026-06f2-494b-9aac-3284ba5acba9` |
| Engine `commit_p0_p2`, full rename; run also selected both graph workflows | **Tested:** rename passed; complete run 1 passed, 2 failed, 3 filtered out, 141.503 s | `7ce6b95f-d6be-4634-bbf9-34b606a90b1e` |
| Engine `commit_p0_p2`, actual P3 in-process/durable/uncached reuse; run also selected both graph workflows | **Tested:** P3 reuse passed; complete run 1 passed, 1 failed, 1 timed out, 3 filtered out, 339.207 s | `17032e50-2b84-4e5e-ad3d-cec302dfba90` |
| `just test-package pse-tests-engine --test native_template_graph --profile ci --no-fail-fast --status-level fail --final-status-level fail`; before empty-candidate shortcut | **Tested:** 0 passed, 1 timed out, 0 skipped, 360.163 s | `8b5163b1-1ecc-41b1-a086-ef5c44b8f32e` |
| `just test-package pse-tests-engine --test native_template_graph --test native_reference_packages --profile ci --no-fail-fast --status-level fail --final-status-level fail`; after winner correction | **Tested:** 0 passed, 2 failed, 0 skipped, 140.132 s. Template completed P6 and failed on P7's missing connection input; reference normalization failed on missing explicit package unit sets. Both corrections are covered by the six-workflow green receipt above. | `a5e55aee-77ec-472c-af7e-36c9f090a4a1` |

For the two `commit_p0_p2` receipts, the command was `just test-package
pse-tests-engine --test native_template_graph --test native_reference_packages
--test commit_p0_p2 -E "'test(two_templates) | test(shipped_reference) |
test(full_rename)'" --no-fail-fast --status-level fail --final-status-level fail`;
the P3 receipt used `test(actual_p3_outputs)` in place of `test(full_rename)`.
`just fmt-check` passed with 0 findings after formatting corrections. New declaration
changes still require regeneration and current source-to-P10/Python qualification.
HP00–HP13 completion remains open.


### Native aggregate and fresh Python inspection continuation — 2026-09-15

**Implemented:** `pse_min` and `pse_max` preserve the selected actual argument's
field metadata while delegating accumulators, grouping, coercion and sliding
aggregation to DataFusion 55.1.0. Empty/all-null groups remain nullable; declared
nonempty rule results use the reusable `pse_require_nonnull` expression. P8 axis
outputs consume the validated context ID, and generated axis IDs carry the same
explicit nonnull obligation. No alias may manufacture a semantic ID from bare bytes.

**Tested**, Arrow force-validation, nextest default, baseline 0:

- `just test-package pse-rules --no-fail-fast --status-level fail --final-status-level fail`:
  **53 passed, 0 failed/skipped**, 12.254 s, run
  `04358d4d-e376-4688-9a19-0527910b61d2`. The declaration probe now prepares every
  P6/P8 truth branch, including analysis and optimization.
- `just test-package pse-catalog --test native_selection_output --test native_nested_output
  --no-fail-fast --status-level fail --final-status-level fail`: **8 passed,
  0 failed/skipped**, 1.218 s, run `a40fa2f7-0851-45ec-9622-b96cb3ddd81d`.
  Actual min/max IDs, metadata, empty/all-null groups and partitioned/grouped
  execution are checked alongside native nested-field operations.

**Implemented — legacy deletion:** the persistent golden-store publisher/comparator,
its `values.json` observation model and comparison test, both old fixture variants,
`PSE_GOLDEN_DIR`, pytest golden markers and hardcoded store readers are deleted.
`just py-test` now constructs and admits one fresh current registry/model/case store
through the actual Rust driver, shares it with pytest workers and removes it when
pytest finishes. `just inspection-fixture <new-directory>` exposes that same producer
for targeted inspection. The Python CI job uses the same recipe. Source packages
remain ordinary current test inputs; no predecessor output graph is restored.

**Implemented and Tested:** native snapshot parsing preserves the ID diagnostic at
the Python boundary. Maturin retains the symbol table required by the pinned PyO3
introspector; the generator removes runtime docstrings and normalizes explicit type
annotations using pinned Ruff before emitting the actual API stub. `just py-sync`
successfully rebuilt the extension and generated the stub. Current Python static
checks pass; full rebuilt-extension acceptance remains open.

**Tested — first inspection run:** `just py-test -n 0
python/pse/tests/test_native_registry.py python/pse/tests/test_snapshot_streams.py`
ran **19 passed, 1 failed**, 83.69 s, Python 3.14.7 editable dev extension, baseline 0.
The failed test selected `packages` by name without its namespace; both authored
and normalized declarations exist. Its lookup now selects the exact authored
relation and awaits rerun. Native fixture construction uses Arrow force-validation.

**Tested — source integration failures:** the source-to-P10 template command
(`just test-package pse-tests-engine --test native_template_graph --profile ci
--no-fail-fast --status-level fail --final-status-level fail`) failed at P8's axis
nullability (run `903ccab5-a032-469b-9a52-94e35c960867`, 0 passed/1 failed,
104.441 s) and then MIN metadata (run `7bec53fb-a217-4b5a-b674-5c3ccff7bf5c`, 0 passed/1 failed,
106.4 s). Both defects are corrected in source and subsequent runs completed P8.
The focused declaration probe reproduced exactly `P8.law_ordered_terms` (run
`4f16c545-3aed-4312-855c-a83467695fcc`, 0 passed/1 failed/1 filtered out,
10.962 s) before the green full rule receipt above. HP00–HP13 remain incomplete.

**Implemented — P9 integration:** selected configuration uses the shared immutable
workspace replacement operation. Augmented rule execution checks newly constructed
relations against output ports. P9 declares its actual kernel dependency-key and
template-symbol-property inputs; missing typed input errors identify the relation.
The registry generator has rebuilt current contracts with `just codegen-bootstrap`.

**Tested:** `just test-package pse-compiler --lib --no-fail-fast --status-level fail
--final-status-level fail`, nextest default, Arrow force-validation, baseline 0:
**22 passed, 0 failed/skipped**, 2.050 s, run
`9e74256c-7a50-4b47-b824-80ec50d7e5f1`. The added kernel-reader test consumes only
the registered P9 input inventory. This is component evidence, not full P9 acceptance.

**Tested — integration boundary:** the source-to-P10 command above subsequently
failed at P9's provision input (0 passed/1 failed, 132.307 s, run
`d3fb791d-36f9-4e54-a447-2effbddec84f`). After its declaration was corrected, the
workflow completed P9 and reached P10, failing at physical planning of a UNION
followed by DISTINCT (0 passed/1 failed, 141.927 s, run
`c861b32a-9052-452e-906a-125a5dec8da3`). The logical UNION intersects metadata;
the physical UNION retained the first branch's extra key annotation.

**Implemented:** `pse_retain_metadata` projects existing
metadata keys without adding annotations or changing values. UNION branches use
that native expression to expose the common field contract physically. Exact
relation projections similarly remove source-only annotations before declaring
their output roles. Context7 and the pinned DataFusion 55.1.0 `with_metadata`
implementation establish that its native operation merges metadata and therefore
cannot perform this removal. A native test covers both UNION input orders,
repartitioning, DISTINCT and actual semantic ID values.

**Tested:** `just test-package pse-catalog --test native_union_metadata --test
native_selection_output --test native_nested_output --test native_output
--no-fail-fast --status-level fail --final-status-level fail`, nextest default,
Arrow force-validation, baseline 0: **13 passed, 0 failed/skipped**, 3.548 s,
run `8f9c79ea-8ac2-496c-b372-40afe9e2af10`.

**Tested — corrected metadata intersection:** that same 13-test command passed
again in 3.617 s, run `1bb2c7b9-0cfd-4374-b6ae-6017531a42fd`, after adding an
unannotated UNION alternative in both input orders. Native logical schema merging
can retain annotations absent from another input; the implementation now computes
the actual intersection from all inputs. Two paired template/heater runs exposed
this and a source CASE that lost its index meaning (runs
`690620bc-be53-48e9-928f-6f3736495794`, 87.079 s, and
`8c875bf5-017b-40f4-a844-beb84bbd2e1f`, 87.150 s; each 0 passed/2 failed,
3 deliberately filtered tests, `ci`, force-validation, baseline 0).

**Implemented:** P3 property-demand index and guard-source CASE expressions use
the shared field-checked CASE constructor. Recursive physical group ancestry
projects away the source relation's role/FK annotations on both terms while
retaining intrinsic semantic types. This operation is visible as the same metadata
projection UDF, and adds no new field meaning.

**Tested:** `just test-package pse-compiler --lib --no-fail-fast --status-level fail
--final-status-level fail`, default nextest, force-validation, baseline 0:
**23 passed, 0 failed/skipped**, 2.105 s, run
`08f242e6-dcdd-4b1a-a474-490829c81b57`. The property-seed component test executes its
actual bound plan and retains index meaning through an empty UNION. It does not
mint a producer receipt from component input batches.

**Tested — product integration remains open:** the paired command with binaries
`native_template_graph` and `native_engineering_workflows`, `--profile ci`, and
filter `test(two_templates_produce_declared_symbols_expressions_and_equations) |
test(heater_ftpx_source_to_canonical_graph)` had **0 passed/2 failed**, 3 filtered,
221.033 s, run `e8c712d9-11a0-4dfa-93a8-9dc969caf7cf`. The heater reached 19 P3
publication invariant violations; the template reached P10's recursive field
alignment. That run compiled with one unused-import warning, now removed. A fresh
rerun includes the recursive-field correction and reports the heater's actual
terminal finding witnesses.

**Interface-checked, not yet Tested:** source workflow tests now select the shipped
nitrogen package with FTPx/FcTP and instantiate the declared heater/two-inlet mixer.
They introduce no procedural physics builder or predecessor output graph. Full
graph, conservation, provenance, persistence and Python acceptance remain open.

### Current source and port integration — 2026-09-15

**Implemented:** normalized domain/instance foreign keys now reference the actual
normalized domains and configured instance bindings, including generated children.
The declaration generator rebuilt all current targets. Stage admission consumes
its explicit current input ports and primitive model/case scope. It does not flatten
undeclared ancestor stage outputs into the current relation inventory.

**Tested:** the paired template/heater command above reports **1 passed, 1 failed,
3 deliberately filtered**, 231.447 s, `ci`, force-validation, baseline 0, run
`28301a56-79fb-4e13-a166-80d0f5238e1f`. The two-template source workflow passes
through P10. The heater passes P3 and fails P4's redundant child-template selection.
P3 already resolves a property-package selector to its actual state-method template;
path traversal now consumes that admitted correspondence rather than interpreting
the package identity as another template identity.

**Tested:** `just test-package pse-tests-engine --test stage_ports --no-fail-fast
--status-level fail --final-status-level fail`, default/force-validation/baseline 0:
**2 passed, 0 failed/skipped**, 2.302 s, run
`70c23900-d5e0-438a-8ae8-94bb94e8fb63`. These engine tests register actual producers,
execute private completion, publish and reopen with the real invariant validator.
The raw producerless catalog fixture is deleted. Repeated output ports admit their
own local keys independently; cross-port references remain explicit and ambiguous
relation-only access is refused.

**Tested:** the 13-test metadata/output command above passes in 3.572 s, run
`dfca4dda-c1b7-4929-9c00-40991fb99ef7`, default/force-validation/baseline 0.
UNION field projection retains actual metadata intersection and leaves ordinary
primitive coercion to DataFusion's analyzer, as its native builder does. Both
Int32/Int64 input orders execute with the expected widened values.

**Implemented and Tested:** the shared free-index collector includes coordinates
stored directly on Gather payloads, and respects lexical reduction/broadcast/integral
binders. A shared gather used inside and outside a reduction retains only its free
occurrence's index. The compiler library command above reports **24 passed,
0 failed/skipped**, 2.083 s, run `077e460f-e057-49be-b59d-36c56c40b904`.

**Tested — engineering boundary:** the heater command
`just test-package pse-tests-engine --test native_engineering_workflows --profile ci
-E 'test(heater_ftpx_source_to_canonical_graph)' --no-fail-fast --status-level fail
--final-status-level fail` reached P6. Run `af077bf4-1f25-442c-a740-5581ede913e0`
failed in 104.529 s on dependency-frontier column order; run
`b2a1131b-e11b-4662-ba38-6aa61115749f` failed in 108.761 s on the missing Gather
binder. Each reports 0 passed/1 failed, 3 deliberately filtered, force-validation,
baseline 0. Seed and step now keep the same column order. The next run,
`66b1bb6d-7368-47cf-969b-254d45a43de1`, reports **0 passed, 1 timed out,
3 deliberately filtered**, 360.155 s under the same conditions. No new semantic
error was reported. Stage and finite-rule timing events are now being used to locate
the work before changing any timeout; full engineering acceptance remains open.

**Implemented and Tested:** child configuration bindings carry actual
typed parent values into the selected child's declaration checks. The value-to-text
serializer and parse-again path are deleted. Enum identity, signed zero and integers
above floating-point precision are retained; incompatible target types and integer
narrowing outside the declared range fail. `just test-package pse-compiler --lib
--no-fail-fast --status-level fail --final-status-level fail` reports **25 passed,
0 failed/skipped**, 2.181 s, default/force-validation/baseline 0, run
`723bb83b-c57e-4560-95bb-fd4f5ac8c3e8`. This tests typed transfer; complete reuse of
the parsed configuration AST and product workflows remain open.

**Implemented — deletion:** all remaining 385 predecessor-store files under
`tests/golden` are deleted, including Arrow artifacts, manifests, source copies and
value observations. No active code, recipe or Python fixture refers to that path.
Fresh source declarations and current pipeline-created temporary stores are the
runtime test inputs. HP12/HP13, faithful durable invocation context, allocation and
property transfer, Python refresh and terminal gates remain incomplete.

#### Native witness sharing — 2026-09-15

**Measured — diagnostic scope:** stage/rule tracing in heater run
`7ead40ac-4983-4bd1-8a25-b7138d2d87e4` localized the 360.152 s timeout to
`P6.candidate_applicable` after completed P3–P5 and P6 scope closure. A read-only
GDB stack sample identified repeated field derivation inside support-plan construction.
The initial shared-computation run `475a4a01-f207-4360-9a9c-dc1edd915e06` reached
the candidate merge with 1,003 support plans, then timed out at 360.295 s. Each
command was `just test-package pse-tests-engine --test native_engineering_workflows
--profile ci -E 'test(heater_ftpx_source_to_canonical_graph)' --no-capture`:
**0 passed, 1 timed out, 3 filtered**, force-validation, baseline 0. These are
diagnostic observations, not performance acceptance; timeout configuration is unchanged.

**Implemented:** fixed-input rule fan-out now retains actual immutable native
completions and grouped source witnesses. Physical column addresses are unique while
logical source qualifiers remain available to binding. Reads of evolving heads stay
visible to delta substitution; fixed scopes are not rebuilt in subsequent rounds.
Stable/volatile expressions are excluded from sharing using their actual native
bindings. Empty immutable results have no row witnesses; their exact original negative
scopes remain available to consuming anti-joins. No predecessor graph is retained.

**Implemented and Tested:** private reuse distinguishes exact floating literal bits.
Native grouping chooses a representative assertion identity, then a native join
retains that actual assertion's payload. This preserves signed zero without changing
native equality. The intermediate window approach exposed ordering/grouping differences
and was removed. `just test-package pse-rules --no-fail-fast --status-level fail
--final-status-level fail` passed **55 tests, 0 failed/skipped**, 11.814 s, run
`d0ac9ce3-ceee-42bc-b6b5-e8fc0c67feba`, default/force-validation/baseline 0.
The subsequently added negative-scope test initially omitted its required negation
declaration; after correction, `just test-package pse-rules --test native_shared_execution
--no-fail-fast --status-level fail --final-status-level fail` passed **3 tests,
0 failed/skipped**, 0.776 s, run `bcfe2e74-8492-4786-9d6a-7a3622665442` under the
same conditions. These cover qualified joined fan-out, exact floating literals and
negative scope through an empty shared result. A fresh engineering run remains required.

#### Explicit truth and candidate classification — 2026-09-15

**Measured — diagnostic scope:** the grouped-witness heater run
`6ec052e2-e88f-4c31-8e1d-ade76b1ef4bb` still timed out at 360.173 s with
867 support plans. Repeated scans in an anti-join were emitting identical absence
scopes. Deduplicating the exact `(relation, port)` absence scope at construction
reduced the candidate merge to 121 plans. The same heater command above then
completed P6 candidate closure and failed on conflicting candidate assertions in
154.393 s, run `a11012f2-ec44-49ce-a05a-ece2b8568cba`: **0 passed, 1 failed,
3 filtered**, ci/force-validation/baseline 0. Timeout limits remain unchanged.

**Tested:** `just test-package pse-rules --no-fail-fast --status-level fail
--final-status-level fail` passed **56 tests, 0 failed/skipped**, 11.927 s, run
`dbbac654-2674-4214-a551-417e0a0f5e22`, default/force-validation/baseline 0,
after absence-scope deduplication. `just test-package pse-rules --test
native_delta_execution --test four_valued_rule_outcomes --no-fail-fast
--status-level fail --final-status-level fail` passed **14 tests, 0 failed/skipped**,
1.395 s, run `ff86d056-dcd9-451c-bdcd-68ef1dd8f07a`. Conflict diagnostics now
retain up to three actual producer/truth/payload tuples. The added payload assertion
passed with `just test-package pse-rules --test native_delta_execution --no-fail-fast
--status-level fail --final-status-level fail`: **5 passed, 0 failed/skipped**,
1.386 s, run `a31be026-9070-40ac-867e-0d68e6302682`, same conditions.

**Implemented and Tested:** an ordinary relational filter now selects
rows only. Explicit `RulePlan::Assert` declares true/false/unknown classification
and is represented in the registry's queryable rule-plan vocabulary. One shared
lowering produces its native truth queries. Assertion nodes are legal only at the
root beneath output projections. The implicit root-filter interpretation and its
duplicate lowering are deleted. `just test-package pse-rules --no-fail-fast
--status-level fail --final-status-level fail` passed **57 tests, 0 failed/skipped**,
8.205 s, run `7c2e535b-3817-441e-a21d-5e3e42dc5c7f`, default/force-validation/
baseline 0. The first compile found one remaining diagnostic-test caller of the
deleted splitter; it now consumes the shared lowering. Heater diagnostic run
`de66b045-bee9-44b4-859c-23012de78e34` confirmed the defect: the applicable rule
asserted accepted rank 100 as true and rejected rank 110 as false for the same key.
It reported **0 passed, 1 failed, 3 filtered**, 152.772 s, ci/force-validation/
baseline 0. The test build had one unreachable-public warning, now corrected.

**Implemented, verification pending:** engineering fixtures now assert authored
state formulation, outlet-only composition closure, pressure equations, material
and enthalpy conservation, and enabled/disabled equipment symbols. These expectations
use independently stated declarations, not predecessor stores. Complete physical
and provenance acceptance remains open until actual workflows exercise the checks.

**Tested — tooling boundary:** `just codegen-bootstrap` regenerated all three schema
targets; bindgen remains deferred under R-3. `just fmt-check` passed before the explicit
assertion edit. `just family-check` passed the exact pinned families and evidence locks.
After removing two obsolete deferrals for already-declared relations, `just governance`
passed **59 tests, 0 failed/skipped**, 7.218 s, default/force-validation/baseline 0,
then failed its generation hygiene check because
`crates/pse-relations/src/generated/provenance/property_read_occurrences.rs` is
untracked. No shared-index mutation or weakening of the tracking check was performed.
Full governance and terminal acceptance remain incomplete.

**Tested — current schema:** `just test-package pse-schema -p pse-relations
-E 'package(pse-schema)' --no-fail-fast --status-level fail --final-status-level fail`
passed **85 tests, 0 failed/skipped**, 4.470 s, run
`2cf96206-09de-441d-a1e5-51781c27193e`, default/force-validation/baseline 0.
Selecting `pse-relations` enables its required feature while the filter exercises
schema tests. The preceding run reported 83 passed/2 failed: the connection contract
test retained expanded tuple keys, and a standalone manifest unnecessarily invoked
the mathematical bridge generator. The test now checks indexed products; graph
bridge generation is scoped to the registry's declared mathematical family.
The explicit assertion scope accepts output projections and rejects nested assertions,
unions and enclosing membership-changing operators.

**Tested — current engineering boundary:** the same heater command above,
run `7cf01c35-4243-4564-9042-66d0d0262416`, reported **0 passed, 1 failed,
3 filtered**, 139.311 s, ci/force-validation/baseline 0, without compile warnings.
P6 completed all rule strata, admission and publication (52.046 s invocation).
P7 then required the consuming template package to duplicate its physical dependency's
mathematical context. **Implemented, verification pending:** P7 now consumes the
Boolean kind established by the shared physical inventory. Its native context query
checks all selected explicit definitions and rejects disagreement. The package-local
second lookup and copied `math_context` rows are deleted; current physical source keys
remain in realization support. The heater rerun is active.

**Tested:** `just quality` passed with **0 errors/findings**, baseline 0, after the
explicit assertion and schema-generator edits; Python 3.14.7, project-pinned tools.
This includes 14 setup/guard tests. Runtime Python/native refresh remains open.

#### Shared physical context — 2026-09-15

**Tested:** the next heater run (`just test-package pse-tests-engine --test
native_engineering_workflows --profile ci -E
"'test(heater_ftpx_source_to_canonical_graph)'" --no-capture`, Arrow force-validation,
baseline 0, run `0a01dcb5-df64-42d6-aa88-e2596cbd983c`) completed P6 and reached
P7's Boolean type lookup: **0 passed, 1 failed, 3 filtered**, 139.703 s. P7 requested
`difference`, whereas the selected physical package declares Boolean `point` types.

**Implemented, verification pending:** Boolean lookup uses the declared point
contract. Numeric configuration uses the shared registry's admitted neutral type.
The package-local context copy and duplicate neutral validator are deleted; the
latter incorrectly required `difference`, contradicting the registry's actual
`quantity_type.neutral_scalar` obligation.

**Tested:** the same heater command, run `2fc3ba31-ffd9-4694-a505-661d4918f823`,
advanced to contribution realization: **0 passed, 1 failed, 3 filtered**, 136.818 s.
The generic subject validator incorrectly rejected phase-indexed energy terms.
**Implemented, verification pending:** total/energy/momentum subjects may have
independent fixed or indexed phase coordinates; subject coordinates remain absent,
and overlapping alternatives or a non-phase domain still reject. No package-name
dispatch or authored graph substitution was added.

**Tested:** `just test-package pse-compiler --lib --no-fail-fast --status-level fail
--final-status-level fail` passed **26 tests, 0 failed/skipped**, 2.054 s, default
nextest/force-validation/baseline 0, run `166be7a1-0f39-40fa-8c49-e451f04a2474`.
The following heater run completed P7 construction, then failed publication with
two subject-alternative findings: **0 passed, 1 failed, 3 filtered**, 190.713 s,
run `17c7f9a9-3ec9-4530-acc8-dfe521bf3a1c`.

**Implemented:** the generated subject invariant now treats a subject/phase as
fixed or indexed. It is shared by authored contribution contracts, compiled
contributions and law applications. P7's duplicate alternative validator is
deleted; its actual-domain axis check remains. `just codegen-bootstrap` regenerated
all schema targets. **Tested:** `just test-package pse-rules --test
invariant_execution --no-fail-fast --status-level fail --final-status-level fail`
passed **6 tests, 0 failed/skipped**, 6.458 s, default/force-validation/baseline 0,
run `b3ecdc9d-0405-46eb-9ec2-6e21a720ba88`. The new case exercises scalar/fixed/indexed
subjects and exact invalid-coordinate keys using the native generated rule.

#### Shared durable context ownership — 2026-09-15

**Implemented:** `RequestContexts` captures registry/source/policy values once per
request engine mode and checks the actual sealed engine values before sharing.
`OwnedStageContext` retains the same allocation and reservation across stage work
and hints. Decoding moves the actual wire context into its owner. The per-stage
descriptor reconstruction, deep hint-context clone and compiler's duplicate fixed
16/64 MiB limits are deleted; actual construction reservations and the catalog's
configured control limits remain. The wire format remains the complete current
format 3. Faithful imported policies/parameters and auxiliary invocation context
remain open; ownership sharing does not establish those semantics.

**Tested:** `just test-package pse-tests-engine --test stage_ports --no-fail-fast
--status-level fail --final-status-level fail` passed **3 tests, 0 failed/skipped**,
10.882 s, default nextest/force-validation/baseline 0, run
`6cd56345-f4f4-4cd7-b80b-e1d07a014896`. The new test runs registered producers through
the actual Driver, retains its real successful attempt record, checks local context
allocation identity, reopens under another catalog and retains context after its
reader/hint are dropped. It uses the existing 32 GiB workflow ceiling. Earlier
128/512 MiB attempts refused the conservative construction/decode forecasts;
this is functional ownership evidence, not memory-efficiency qualification.
The obsolete catalog-only hint fixture and fabricated record/context helper are
deleted. The fixture's producer reference now uses its declared pass name.

**Interface-checked, incomplete:** `just check` completed all targets before the
indexed-subject changes with 0 errors and one PyO3 deprecation warning. The pinned
0.29.2 source and Context7 document the explicit `skip_from_py_object` opt-out;
`EngineSettings` now uses it because the Python boundary borrows that class.
`just fmt-check` passes. Clippy, the fresh editable extension and full current
workflow verification remain pending.

#### Conservation decisions and current result ownership — 2026-09-15

**Tested:** the heater command `just test-package pse-tests-engine --test
native_engineering_workflows --profile ci -E
"'test(heater_ftpx_source_to_canonical_graph)'" --no-capture` reached P8 after P7
publication: **0 passed, 1 failed, 3 filtered**, 195.748 s, nextest ci with Arrow
force-validation, baseline 0, run `28291b71-2b19-4bb1-912a-cffa09058f7b`.

**Implemented:** the native participation rule assigns zero to excluded terms and
retains declared ±1 orientation only for included terms. P8's duplicated procedural
reason/decision/sign validator is deleted; the generated publication invariant
remains authoritative. All three schema targets were regenerated.

**Tested:** the same heater command completed P8 admission and publication, then
failed terminal recording with duplicate derivation identities: **0 passed,
1 failed, 3 filtered**, 209.043 s, run `1d0dd3f6-68b1-4be6-b64f-502b765a436d`.
**Implemented, verification pending:** P8 forwards unchanged completed relation
owners directly. Its graph adapter rebuilds only actual mathematical nodes and
remapped roots. The old copy/rematerialization of every current output is removed;
the terminal uniqueness check remains. This is target computation reuse, not
historical graph retention.

**Interface-checked, incomplete:** workspace Clippy advanced through quantity,
MathIR and schema libraries. Generator integer literals now use digit grouping;
ordinary schema checks use named comparison, union-shape and stage-graph helpers.
Explicit imports replace wildcard imports. Declarative relation/rule families and
mechanical mathematical adapter mappings carry function-scoped length explanations.
The latest completed `just clippy` run stopped at one schema test closure and 72
generated allocation literals; both source causes are corrected. `just codegen-bootstrap` subsequently regenerated
all three schema targets. No workspace Clippy or full-plan acceptance is claimed.

**Tested:** `just test-package pse-compiler -p pse-schema -p pse-rules -p
pse-mathir --no-fail-fast --status-level fail --final-status-level fail` passed
**259 tests, 0 failed/skipped**, nextest default/force-validation/baseline 0,
10.403 s, run `508f873b-7291-4526-b4c0-3fd4c3c1798f`. This covers current schema,
rules and mathematical algorithms and compiles P8's owner-sharing integration.
The full heater rerun remains active. `just fmt-check` and `just quality` also pass
with **0 findings/errors**, including the 14 setup-guard tests; native extension
refresh and full product acceptance remain open.

#### P9 admission cost — 2026-09-15

**Tested:** the same focused heater command passed P8 publication and terminal
recording, then timed out in P9: **0 passed, 1 timed out, 3 filtered**, 360.708 s,
nextest ci/force-validation/baseline 0, run
`742db3cb-49a9-4bab-bc84-6fe7fe1fc12b`. The 360-second timeout is unchanged.

**Measured:** a read-only GDB sample of that task's test process found
`CompletedComputation::checked_relation` → `FieldCheckedBatch::admit_owned` →
`validate_batch` → `decode_dictionary` → `SemanticId::to_hex`. Enum lookup formatted
every registry enum identity for every decoded enum cell. **Implemented:** lookup
parses the actual field identity and compares binary IDs directly. All existing
schema, membership and recursive value checks remain; broader native property
transfer is still open. P9 emits timings for selection, configuration, augmented
inference, bindings, mathematics and provisions so its next material cost is
localized without an additional producer path.

**Tested:** `just test-package pse-relations --no-fail-fast --status-level fail
--final-status-level fail` passed **29 tests, 0 failed/skipped**, default nextest,
Arrow force-validation, baseline 0, 1.298 s, run
`6d46f9de-6f3e-4def-ac28-1aba5d119f1b`. The next full heater run is active; no workload
speedup or P9/P10 acceptance is claimed yet.

#### Native support ordering and literal operation context — 2026-09-15

**Tested:** after binary enum lookup, the focused heater command above completed
P9 publication and terminal recording, then failed in P10: **0 passed, 1 failed,
3 filtered**, 305.454 s, ci/force-validation/baseline 0, run
`4a97dd7b-e809-42f4-8809-3ce62b2019bb`. P10 rejected an ambiguous Kelvin literal in
the declared Shomate normalization `T / 1000{K}`.

**Implemented:** occurrence typing now uses the actual registered Mul/Div inference
to select a unique compatible literal type from its unit candidates and resolved
sibling. A declared result contract also constrains that selection. Explicit
literal types remain authoritative; unresolved ambiguity and registry/prerequisite
errors remain errors. Scalar candidates carry no free indices. No unit name,
package name, candidate order or parallel operation roster selects the type.

**Interface-checked:** Context7 discovery for `/apache/datafusion` was checked
against pinned DataFusion 55.1.0 source: the ordered `array_agg` accumulator compacts
each nested scalar; `array_sort` uses Arrow row conversion and array `take` for
struct elements. **Implemented:** support materialization now collects distinct
source structs with unordered `array_agg`, then applies native
`array_sort(..., 'ASC', 'NULLS FIRST')`. The existing checked list-field constructor
retains the declared semantic field. There is no custom sorting engine.

**Tested:** `just test-package pse-compiler -p pse-schema -p pse-rules -p pse-mathir
-p pse-quantity --no-fail-fast --status-level fail --final-status-level fail`
passes **327 tests, 0 failed/skipped**, default/force-validation/baseline 0,
10.501 s, run `4c0d66c2-7530-4c6b-9c42-25d33530db66`. This includes exact ordered
support lists from a two-partition joined fanout, unique temperature-scale inference,
ambiguous neutral scaling in either operand order and explicit-type preservation.
The first broader run found one obsolete test-local pressure type already supplied
by the current reference registry; that duplicate fixture declaration is deleted.
`just fmt-check` passes with 0 findings. The heater rerun is active; no workload
speedup, full P10 acceptance or complete-plan acceptance is claimed.

#### Explicit schema transforms and indexed literal context — 2026-09-15

**Implemented — hard-pivot deletion:** the `pse-relations` row migration engine,
automatic version-path search and generated migration lookup are deleted.
`SnapshotSession::prepare_schema_transform` compiles an explicitly named registered
edit declaration against its exact source role into native projections, aliases,
checked defaults and reusable nullability UDFs. Declarative schema edits remain
inputs to this target operation (blueprint §20.5); no legacy store interpreter is
retained. Exact-version registry lookup and duplicate native table-name checks
prevent one relation version from silently replacing another. Distinct roles can
bind both versions explicitly.

**Interface-checked:** Context7 `/apache/datafusion` projection/alias discovery was
checked against pinned DataFusion 55.1.0 `LogicalPlanBuilder`. A live nested-default
test exposed metadata loss in `ScalarValue::try_from_array` for a semantic list.
The shared `checked_array_literal` constructor retains the actual one-element
nested Arrow array in the native scalar; the existing checked literal constructor
still establishes its field/value contract. No completed output is relabeled.

**Tested:** the first native schema-transform run passed **2 tests, 0 failed/skipped**,
default/force-validation/baseline 0, 0.019 s, run
`23d982e1-fbba-420e-a34b-99796628c290`, command
`just test-package pse-catalog --test native_schema_transform --no-fail-fast
--status-level fail --final-status-level fail`. It preserves exact negative-zero
defaults, nested semantic IDs and actual rows, and rejects nulls during execution.
The extended exact-version test then found a missing explicit output declaration
in its source scan (1 passed/1 failed, run
`cd16d40a-9e13-4a01-a6ce-44d6138818b8`); corrected, fresh verification pending.

**Implemented and Tested:** the reference package declares neutral dimensionless
Pow. Additive literal context uses actual point/difference algebra and the declared
result contract; explicit incompatible types remain errors. The foundation command
`just test-package pse-compiler -p pse-schema -p pse-rules -p pse-mathir
-p pse-quantity -p pse-relations --no-fail-fast --status-level fail
--final-status-level fail` passed **358 tests, 0 failed/skipped**, 10.725 s,
default/force-validation/baseline 0, run
`48e9a7ad-b15d-43c9-968f-2cde51f6061e`. This precedes the latest schema-transform
and broadcast edits.

**Tested — latest engineering boundary:** the actual heater command
`just test-package pse-tests-engine --test native_engineering_workflows
--profile ci -E "'test(heater_ftpx_source_to_canonical_graph)'" --no-capture`
completed with **0 passed, 1 failed, 3 filtered**, 296.205 s,
ci/force-validation/baseline 0, run
`1088112f-a8db-42c9-9969-cc1b95b2080c`. P3–P9 publish and record; P10 identifies
an unbroadcast scalar zero in the mixture-enthalpy source expressions.
**Implemented, verification pending:** `broadcast(value, index)` lowers the existing
Broadcast operator against an actual lexical binding; declared output context is
transferred to its input by removing exactly the added domain kind. The reference
mixture and phase-density expressions now state their broadcasts explicitly. A raw
literal cannot acquire indices implicitly. `just codegen-bootstrap` regenerated
all three schema targets; solver bindgen remains deferred under R-3.

**Implemented, verification pending:** generated builder allocation/accessor lint
causes are corrected in the generator, including owned-row append, local extension
checks and actual borrowed values. Latest completed `just clippy` had 296 generated
iterator/borrow findings before the final generator fixes. Workspace Clippy,
complete target engineering acceptance and HP00–HP13 closure remain open.

#### Current native admission integration — 2026-09-15

**Implemented:** external-stage admission now invokes the catalog's registered
producer and compares complete normalized output values in one shared path.
The overridable `SemanticValidator::validate_stage` callback and separate compiler
comparison module are deleted. Generic invariant success cannot establish producer
semantics; an absent producer fails closed. Local completed production still uses
its existing single-execution publication path. Faithful imported invocation
context remains an HP06 requirement.

**Implemented — test integration:** store, pinned reopening, immutable inspection
and diagnostic plan-codec tests moved from `pse-catalog` to `pse-tests-engine`.
Their fixtures now use the actual native invariant validator and explicit registered
projection producers. Hand-written change-set declarations are deleted and replaced
by the canonical registry declarations. The forged-parent test checks the exact
native foreign-key finding and invalid key, then releases its owned diagnostic
before checking reservation cleanup. No runtime admission requirement was relaxed.
The engine test family uses the existing workspace `datafusion-proto` pin; resolved
Cargo metadata retains one package version.

**Tested:** `just test-package pse-tests-engine --test catalog_inspection
--test catalog_pinned_reopen --test catalog_session_codec --test catalog_store_protocol
--no-fail-fast --status-level fail --final-status-level fail` passes **26 tests,
0 failed/skipped**, 15.376 s, default/force-validation/baseline 0, run
`1d3b9010-4d2a-4619-8160-e69cd871c51b`. This covers actual publication/reopening,
producer refusal, changed parent rows, source correspondence, immutable receipts,
stream lifetime/cancellation and plan-codec provider bindings.

**Tested:** `just test-package pse-mathir -p pse-relations -p pse-authoring
-p pse-catalog -E "'binary(indexed_driver) | binary(dsl_examples) |
binary(native_schema_transform)'" --no-fail-fast --status-level fail
--final-status-level fail` passes **15 tests, 0 failed/skipped**, 0.025 s,
default/force-validation/baseline 0, run
`6d0664d8-1e77-4173-87f6-b1aed2c7d88c`. It includes exact-version schema transforms,
duplicate native-name refusal and explicit literal broadcasts with actual indices.

**Tested — engineering boundary:** the heater command above then failed in P9
after **217.690 s**, 0 passed/1 failed/3 filtered, ci/force-validation/baseline 0,
run `11ea49a0-5ec0-41bd-967f-699911d0c868`: template instantiation treated Broadcast
as a new lexical binder. **Implemented:** Broadcast now reuses the actual lexical
index while adding an axis to the value; reduction binding remains strict. A focused
root/reduction instantiation regression was added. Fresh broader foundations and the
heater rerun remain pending. `just quality` passes with 0 findings/errors and all
14 setup-guard tests; workspace Clippy and full engineering acceptance remain open.

**Tested — current foundations:** `just test-package pse-compiler -p pse-schema
-p pse-rules -p pse-mathir -p pse-quantity -p pse-relations -p pse-catalog
-p pse-authoring -p pse-templates --no-fail-fast --status-level fail
--final-status-level fail` passes **562 tests, 0 failed/skipped**, 15.084 s,
default/force-validation/baseline 0, run
`0b6d1636-0740-4657-bf63-bb5dc4bdff94`. This includes the lexical broadcast
correction. Together with the separate 26-test catalog integration receipt, it
supersedes the first wider run's fixture failures. The full heater rerun is active.

#### Actual fixed-coordinate broadcast selection — 2026-09-15

**Tested — failure boundary:** the heater command above failed in P9 after
208.805 s, 0 passed/1 failed/3 filtered, ci/force-validation/baseline 0, run
`c1e8cee7-d5ac-489b-89d9-74ebe72b57de`. A symbol-member expression had already fixed
its outer coordinate, so Broadcast could not require that coordinate to remain free.

**Implemented:** instantiation selects the broadcast value at the actual fixed
member and retains the exact source-node correspondence. Actual environment/member
and operator-domain checks remain mandatory; free-axis broadcasts remain explicit.
P3 precondition queries now bind the inputs' existing checked owners instead of
re-admitting their raw field values before executing the same invariant program.

**Tested:** `just test-package pse-templates --test instantiation --status-level fail
--final-status-level fail`: 13 passed, 0 failed/skipped, 0.013 s,
default/force-validation/baseline 0, run
`dc2ccfd8-7e67-46d3-b99d-b959e6e3fdc0`. Coverage includes free/reduced/fixed broadcasts
and refusal of a foreign fixed member. `just family-check` passes with all shared
family packages matching the evidence locks. Full engineering acceptance is pending.

#### Native engineering declaration integration — 2026-09-15

**Tested:** the nine-package foundation command above passes **563 tests,
0 failed/skipped**, 15.618 s, default/force-validation/baseline 0, run
`f6e6be2c-6330-4e16-a0e8-f4f953f5ad6b`. The focused command
`just test-package pse-compiler -p pse-tests-engine -p pse-templates -p pse-mathir
-E "'binary(quantity_relations) | binary(pushdown_vs_unpruned) | binary(stage_ports) |
binary(instantiation) | binary(indexed_driver)'" --no-fail-fast --status-level fail
--final-status-level fail` passes **31 tests**, 0 failed/skipped, 12.095 s,
default/force-validation/baseline 0, run
`ff8d868c-b2ca-41a1-ae87-9cdab723bbe8`.

**Implemented — deletion:** removed unused `SnapshotCatalog::new`, the unused
`RelationTable::new` reservation argument, the no-op `ThreadBudget::validate` route,
and the engine fixture's forwarding-only `semantic_pipeline` module. Active callers
use the target native constructors, actual configuration validation and shared
`native_pipeline` fixture. Provider tests use actual invariant admission.

**Tested — failure boundary:** `just test-package pse-tests-engine
--test native_engineering_workflows --profile ci --no-fail-fast --status-level fail
--final-status-level fail` has **0 passed, 4 failed**, ci/force-validation/baseline 0,
run `5eabbfe4-2539-402d-9ce5-117049f182b4`. Heaters complete P9 and fail P10's
normalization prerequisite; mixers fail P3 on the inlet parameter's type. The
nextest engineering group permits two concurrent workflows, each with its own
32 GiB runtime ceiling; actual host available memory exceeded their combined
ceilings when configured. The timeout permits the observed 293.374 s heater
invocation; it is a diagnostic bound, not a numerical timing gate.

**Implemented, verification pending:** normalization checks the weighted
operation's actual 1 bar thermochemistry difference, preserving all physical axes
and rejecting foreign datums, points and component subjects. The positive test
composes weighted component enthalpy with normalization, rather than testing an
unconnected division alone. The mixer declaration now uses `inlet_domain` with a
typed semantic ID; its source workflow declares the actual finite domain and two
members. No list-domain compatibility branch is introduced. Physical prerequisite
failures now include the operation, invariant, operand and actual/required axes.
`just codegen-bootstrap` regenerates all three schema targets; bindgen remains R-3.

**Implemented, verification pending:** cohesive native aggregate helpers separate
input/null admission, ordering obligations, empty-result handling and final output
fields. Rule and authoring lint cleanup remains active; a lease-method spelling
regression during cleanup was caught by codegen compilation and corrected. Full
workspace Clippy, engineering acceptance and HP00–HP13 closure remain open.

#### Physical basis and source-inventory integration — 2026-09-15

**Tested:** the four-workflow command above reaches P10 for every heater/mixer and
FTPx/FcTP combination: **0 passed, 4 failed**, 600.289 s,
ci/force-validation/baseline 0, run `1f5332fe-2539-481b-be4f-7de14b0d3567`.
Both earlier declaration failures are cleared. Each workflow now fails at the same
energy-density operation, whose molar basis policy did not match its declared
energy-per-volume result. **Implemented:** require the actual common molar basis,
cancel it, and preserve the enthalpy datum. The focused regression checks both
operand orders and rejects a wrong datum and difference operand.

**Tested:** `just test-package pse-compiler -p pse-authoring -p pse-rules
--no-fail-fast --status-level fail --final-status-level fail` passes **137 tests,
0 failed/skipped**, 11.428 s, default/force-validation/baseline 0, run
`3995bc7f-2b43-4234-908a-6c81bbbef32c`. This covers the physical declaration and the
native aggregate, trace, witness, invariant, source and change-application helper
extractions up to that run. `just codegen-bootstrap` regenerated all three schema
targets. Its one unused-qualification warning was corrected. Bindgen remains R-3.

**Tested — environment boundary:** an earlier broader compile exhausted filesystem
space before tests ran. Package-scoped `cargo clean -p pse-tests-engine` removed
27.9 GiB of regenerable local build artifacts; no source or Git index was changed.
The narrower 137-test command then completes normally. The failed build is not a
test receipt.

**Implemented, verification pending — deletion:** `TargetContext` and its unused
nine-relation row decoder are removed. The lexical algorithm keeps only used entity,
symbol and template-domain inputs alongside its existing single instance inventory.
Rename selects only its entity ancestry. P3 borrows generated domain/symbol Arrow
columns instead of decoding the broad inventory. Target tests build and mutate
actual checked relation batches directly. Current lexical inputs are retained only
for actual target computation; no predecessor graph or compatibility object returns.

The latest workspace Clippy run reaches only two rule and three authoring production
findings before compilation stops. Further helper extraction addresses these but
requires fresh compilation/lint; downstream workspace findings remain unmeasured.
Complete target workflows, Python, durable invocation and HP00–HP13 gates remain open.

#### Target-context deletion and indexed equation bounds — 2026-09-15

**Implemented and Tested:** `TargetContext` and its nine-family decoder are deleted.
P3 uses checked Arrow domain/symbol columns, lexical binding retains one instance
inventory, and rename loads its actual entity ancestry. The targeted compile caught
an incorrect fixture `Arc` borrow; corrected before tests ran. The command
`just test-package pse-compiler -p pse-authoring -p pse-rules --no-fail-fast
--status-level fail --final-status-level fail` then passes **137 tests, 0 failed/skipped**,
11.417 s, default/force-validation/baseline 0, run
`6480e468-1e0b-432f-b50f-66efa2cacaf6`.

**Tested — engineering boundary:** the two representative heater FTPx and mixer
FcTP workflows clear the corrected energy-density declaration but fail with an
index mismatch: **0 passed, 2 failed, 2 filtered**, 305.069 s,
ci/force-validation/baseline 0, run `3230aff2-9c0f-405c-af71-c334901d241b`.
Command: `just test-package pse-tests-engine --test native_engineering_workflows
--profile ci -E "'test(heater_ftpx_source_to_canonical_graph) |
test(mixer_fctp_source_to_canonical_graph)'" --no-fail-fast --status-level fail
--final-status-level fail`.

**Implemented, verification pending:** law/connection expansion now constructs
explicit Broadcast nodes for zero bounds across the equation's actual free binders.
No scalar-to-indexed inference shortcut is added. A new regression feeds scalar,
single-axis and multiple-axis constructed bounds through actual P10 inference and
checks refusal when the required Broadcast is absent. The misleading source-node
error lookup is deleted: canonical node IDs cannot identify rows in the loaded
source graph. The representative engineering rerun and new regression remain open.
The quantity crate's obsolete `uom` prohibition wording is removed; library
eligibility follows the current authority.

**Tested — indexed-bound result:** the representative command above now reports
**1 passed, 1 failed, 2 filtered**, 305.315 s, ci/force-validation/baseline 0, run
`71cbac20-0dad-46dd-aa7d-6628289d118f`. Heater FTPx passes through P10. Mixer FcTP
still fails. Diagnostics now carry each actual quantity and free-binder contract;
the next run identifies scalar molar flow multiplied by species-indexed fraction.
The stock state expressions now explicitly broadcast their missing phase/species
axes, and the mixer pressure declaration broadcasts the common pressure over inlets.
These source corrections await a fresh complete workflow receipt.

**Tested:** `just test-package pse-compiler -p pse-quantity --no-fail-fast
--status-level fail --final-status-level fail`: **105 passed, 0 failed/skipped**,
2.781 s, default/force-validation/baseline 0, run
`3b592ad9-ea8f-431f-a625-83109e05d376`. This exercises actual canonical inference of
constructed scalar and indexed bounds, including refusal of an omitted Broadcast.

**Implemented, verification pending:** the separate realization wrappers and their
repeated eleven-argument control path are replaced by one function receiving the
actual `PassContext`, session and optional selected-method inputs. Current graph
extension is tied to that selected computation. The source/sink generators now
emit direct nonempty refusal checks for grounded families, shorthand row fields
and explicit family alternatives; regeneration is pending. Workspace Clippy remains
open: its last run reports 256 compiler library findings, including these repeated
generator causes, after clearing rules/authoring production code. No lint baseline
or whole-plan closure is introduced.

#### Complete engineering graph matrix — 2026-09-15

**Tested:** `just test-package pse-tests-engine --test native_engineering_workflows
--profile ci --no-fail-fast --status-level fail --final-status-level fail` reports
**4 passed, 0 failed/skipped**, 599.359 s, ci/force-validation/baseline 0, run
`becb477b-1262-4c5c-82cc-81bd527c7d45`. All heater/mixer FTPx/FcTP source pipelines
publish P10 and satisfy the independent graph expectations. The preceding diagnostic
run of the remaining three configurations (`-E "'not test(heater_ftpx_source_to_canonical_graph)'"`)
reported **0 passed, 3 failed, 1 filtered**, 587.773 s, run
`49155d58-0b60-4536-a27e-7e7ed5ca0c82`: both FcTP cases exposed missing state
broadcasts, while mixer FTPx required an indexed absolute-pressure difference.
Those exact source contracts are now explicit. This is graph acceptance; broader
scientific, cold reopening, Python and terminal gates remain open.

**Tested:** the nine-package foundation command above passes **566 tests,
0 failed/skipped**, 15.011 s, default/force-validation/baseline 0, run
`b5bc710c-c9c4-4ecb-8139-22b3344dd3e3`. The new pressure test checks exact residual
scale/datum/axes and refuses different binders. `just codegen-bootstrap` completes
all three schema targets. Two unused-import warnings were corrected; bindgen is R-3.

**Implemented, verification pending:** subsequent cleanup separates P10 root
contracts from canonical output remapping, shares native source-support construction,
and separates parameter/feature/dynamic configuration binding. Operand helpers borrow
actual values and symbol provenance records only the needed symbol ID instead of
cloning whole rows. The latest Clippy boundary is **122 compiler library findings**
(125 for its library-test build), baseline 0. Further generator formatting corrections
are being regenerated before the fresh native suite. Whole-plan completion is not claimed.

#### Manifest-selected invocation bindings — 2026-09-15

**Implemented — durable invocation bindings:** the manifest now selects one immutable
checksum-addressed binding containing exact parent encodings, selected policy owners
and sealed engine settings. Publication, explicit-context admission, pinned reopening
and durable reuse consume that binding. The target-keyed receipt and default-policy
replay fallback are deleted. Native engine restoration retains the supplied actual
implementations, restores explicit absent settings and refuses incompatible context.
Invocation/configuration allocations retain their own reservations. Complete copied
stage-context removal and the wider HP06 lifecycle/implementation-binding gates remain open.

**Tested:** `just test-package pse-tests-engine --test catalog_pinned_reopen
--test catalog_session_codec --test catalog_store_protocol --test stage_ports
--test native_normalization --profile ci --no-fail-fast --status-level fail
--final-status-level fail` reports **31 passed, 0 failed/skipped**, 244.627 s,
ci/force-validation/baseline 0, run `9bc1efe9-9ae0-408d-adee-77e1eedd145e`.
The selected-policy P3 case reopens under another catalog, retains actual policy
owners and values, and rejects an explicit context that omits the invocation.
Engine tests restore explicit absence, retain implementation ownership, check
information-schema readback and refuse a missing function or invalid setting.
Equal logical membership with distinct parent encodings yields distinct physical
manifests; both reopen with their own exact dependencies. Retired binding versions,
malformed claims and forged stage rows fail. Source/port fixtures now explicitly
construct invocation bindings rather than relying on defaults.

**Implemented, verification pending:** pinned traversal retains each decoded envelope
until its actual dependency owners are admitted, avoiding a second envelope/binding
read and decode. The Driver restart test now requires reused snapshots with fresh
attempt records. A focused rerun covers those changes and invocation reservations.

**Tested — follow-up ownership:** `just test-package pse-tests-engine --test stage_ports
--profile ci --no-fail-fast --status-level fail --final-status-level fail` reports
**3 passed, 0 failed/skipped**, 17.584 s, ci/force-validation/baseline 0, run
`055409a1-f471-445b-99d9-76dff762e66b`. Restarted Driver invocations reuse both
published snapshots and create new attempt records. The test follows ownership
through the retained input snapshots and in-memory store; their stored record bytes
release only after the final store owner is dropped.

**Tested — foundations after the binding change:** the nine-package command above
passes **566 tests, 0 failed/skipped**, 14.963 s, default/force-validation/baseline 0,
run `d47fd845-74c7-407b-9ec9-bfc7f01a8903`. The recursive wire-contract fixture now
supplies the new optional binding object so its nested required/unknown fields are
exercised. `just quality` reports zero findings/errors and 14 setup-guard tests pass.

**Tested — foundations before the binding change:** the nine-package command above
passes **566 tests, 0 failed/skipped**, 14.972 s, default/force-validation/baseline 0,
run `ead9ae33-0a75-40ba-9f65-4bcd33d516dd`. This includes P3/P10/native-construction
helpers and generated adapters. `just clippy` subsequently reaches **72 compiler
library findings, 73 including library tests**; this predates binding changes.
`just codegen-bootstrap` regenerated all three schema targets (bindgen is R-3).
`just adr-lint` reports 67 records, an up-to-date index and 31 register rows, zero
errors. ADR-0067 remains proposed; no formal acceptance is implied.

#### Original documents outside relational ports — 2026-09-15

**Implemented and Tested:** an invocation retains the exact committed source owner
from which its original documents were loaded. This auxiliary binding is independent
of the producer's relational input ports. Reopening admits that source graph and
loads the selected original documents; it never substitutes the port inventory or
an inferred default. The current wire requires the source-selection field, including
explicit empty selection. A foreign catalog source is refused.

**Tested:** `just test-package pse-tests-engine --test stage_ports --profile ci
--no-fail-fast --status-level fail --final-status-level fail` reports **4 passed,
0 failed/skipped**, 56.209 s, ci/force-validation/baseline 0, run
`ac4da91c-ec7b-40a6-92f9-61a3a228c864`. A registered producer with no relational input
ports counts actual original documents; another catalog reopens its source and
reproduces the nonzero count. The same run covers exact port selection, restarted
reuse, fresh attempts and detached ownership. The preceding P3/source-context rerun
(`--test stage_ports --test native_normalization -E "'test(cold_stage) | binary(stage_ports)'"`)
passed **4 tests, 0 failed, 4 filtered**, 108.491 s, run
`9025a179-994a-4986-b702-db917709c87d`, under the same ci/force-validation conditions.

**Implemented and Tested:** invocation and wire reservations now use the common typed
BTreeMap node accounting for their distinct native and encoded value sizes. Pinned
traversal reserves complete map nodes and stack growth before allocation. The full
nine-package foundation command reports **567 passed, 0 failed/skipped**, 14.926 s,
default/force-validation/baseline 0, run `c5ff2ea8-f3f4-4a2b-a700-6ede82f27df3`.
The additional test requires explicit document-source selection in current bindings.
The last workspace Clippy boundary is **71 compiler library findings, 72 including
library tests**, before this document-source follow-up; no baseline is accepted.

#### Current Python and workspace integration — 2026-09-15

**Tested:** `just py-sync` rebuilt the editable dev extension and generated its actual
API stubs. `just py-test -n 0` reports **80 passed, 0 failed, 55 parity tests deselected**,
48.83 s, Python 3.14.7, unit/component mode, baseline 0. The recipe first publishes and
admits two fresh current snapshots. This covers manifest contracts, generated rows,
reference movement, exact manifest opening, read-only streams, cancellation and final
exported-owner lifetimes. It does not establish numerical parity or P10 Python inspection.

**Tested:** `just check` passes all workspace targets with **0 warnings/errors**;
`just doctor` reports Environment ready and matching lockfiles. The full workspace
Clippy and broader HP00–HP13 scientific/performance/architectural gates remain open.

#### Cached configuration syntax and reference-only stage hints — 2026-09-15

**Implemented:** child-template configuration borrows each cached authoring AST by
its exact template/submodel key and binding ordinal. The loader exposes original
decoded text with its cached syntax; configuration verifies the selected field and
interprets its structure without reparsing or copying the AST. P3 and selected-method
configuration use the same route. The `parent.` string-prefix interpretation is
deleted from child bindings. Exact i64/u64 token decoding remains at the declared
scalar boundary; the AST's f64 approximation cannot replace those values. Indexed
references and arbitrary arithmetic are refused under blueprint §14.3's configuration
contract. Plain-text domain/feature declarations retain their separate declared grammar.

**Tested:** `just test-package pse-compiler -p pse-authoring --no-fail-fast
--status-level fail --final-status-level fail` passes **82 tests, 0 failed/skipped**,
5.241 s, default/force-validation/baseline 0, run
`3f808361-e58b-4c5d-9d29-0479f46397e1`. The unchanged-document test checks that the
public borrowed AST is the same allocation after another document changes.
`just test-package pse-tests-engine --test native_normalization --profile ci
--no-fail-fast --status-level fail --final-status-level fail` passes **6 tests,
0 failed/skipped**, 258.279 s, ci/force-validation/baseline 0, run
`f2578456-c87f-4008-a300-64d1be8877db`. This includes exact parent transfer and u64
maximum literals in two actual parent instances, plus the existing source/quantity
and cold selected-policy cases. These receipts precede stage-context deletion.

**Implemented — deletion:** `StageContext`, `StageSource`, `StagePolicy`, their
serializers and the compiler's tagged-Cell allocation forecast are deleted. Stage
index version 4 contains only exact input/output and attempt-record references;
older formats are refused. The actual manifest-selected admission binding supplies
durable invocation inputs. One request shares its actual admitted policy/document
owners and sealed engine settings. In-memory and durable reuse share comparison of
actual Arrow values and recursive parent/document/policy owners, without a second
registry/source/policy row inventory. Registry admission and imported producer
construction remain authoritative. Full implementation/effect dependency framing,
comparison-work accounting and terminal publication/cancellation gates remain open.

**Tested:** the nine-package foundation command above passes **568 tests,
0 failed/skipped**, 14.899 s, default/force-validation/baseline 0, run
`f23e6746-975a-406c-ac31-37f9f1e736ca`, including stage-context deletion and current
index refusal. `just fmt` completes successfully. The workspace check compiled all
targets with one unused-qualification warning; its cause is corrected and the next
native integration run checks that correction. Broader acceptance is not claimed.

**Tested — preceding workspace documentation:** `just doctest` passes **33 doctests,
0 failed/ignored**, workspace/force-validation/baseline 0. It predates the cached
configuration and stage-context deletion changes.


**Tested — admitted hint and ownership integration:** `just test-package
pse-tests-engine --test stage_ports --test memo_dependencies --test catalog_pinned_reopen
--test catalog_session_codec --test catalog_store_protocol --profile ci --no-fail-fast
--status-level fail --final-status-level fail` passes **30 tests, 0 failed/skipped**,
70.166 s, ci/force-validation/baseline 0, run
`52149e65-b5aa-4f0b-bcaa-ae5e8c0fb71d`. This covers stage-context deletion, restart
reuse with new attempt records, actual invocation-owner sharing and detached-owner release.

**Implemented and Tested — bounded dependency comparison:** the shared traversal
now reserves its pending/visited snapshot pairs before allocating and checks the
request cancellation token. It follows actual parents and auxiliary invocation
owners once per pair. `Memo::lookup` and `Dependencies::equivalent` require the
request's reserver and cancellation token. `just test-package pse-tests-engine
--test memo_dependencies --test stage_ports --profile ci --no-fail-fast
--status-level fail --final-status-level fail` passes **9 tests, 0 failed/skipped**,
59.468 s, ci/force-validation/baseline 0, run
`a07a4fe5-9b4b-493e-b19d-8915b32d1448`. The added regression checks refusal with no
available budget, exact separate-handle comparison, cancellation and release on
both success and refusal. The preceding expanded run was **8 passed, 1 failed**,
58.103 s, run `dfd2fa99-020e-4ea1-acf5-36d93c978655`: a stale fixture attempted
bare publication without a registered producer/validator. Its distinct-port assertions
now use the existing executable port producer; `stage_bundle_graph.rs` is deleted.

**Interface-checked:** after separating configuration queue/context setup, finite
child membership, domain candidates, assignment refusal and quantity-value decoding,
`just check` passes all workspace targets with **0 warnings/errors**, 2.65 s,
baseline 0. `just fmt` completes. These helpers preserve the same native selections
and actual origins. The subsequent four-case engineering receipt is recorded below.
The latest `just clippy` receipt (before those helper extractions) has **70 compiler
library findings, 71 with library tests**, baseline 0; no-default lint was not reached.

#### Common producer results and native input ports — 2026-09-15

**Tested — cached syntax and current hints through P10:** `just test-package
pse-tests-engine --test native_engineering_workflows --profile ci --no-fail-fast
--status-level fail --final-status-level fail` passes **4 tests, 0 failed/skipped**,
593.216 s, ci/force-validation/baseline 0, run
`10c8df55-bca4-4a03-8b6f-072c68c8dbef`. This includes configuration helper extraction,
reference-only hints and reserved dependency comparison. It precedes the producer
result and named-input changes below.

**Implemented — deletion:** compiler `PassOutput`, `PassRecordDraft` and the second
output inventory/field validator are deleted. P3–P10 return the catalog's
`ProducedStage` directly. Its executor alone creates a private completed production
after declared output obligations; the Driver owns actual attempt identity, duration
and terminal status. Failed/cancelled producers return their structured errors and
findings. The adapter retains actual native plan observations without another output map.

**Tested:** `just test-package pse-tests-engine --test stage_ports
--test terminal_attempts --profile ci --no-fail-fast --status-level fail
--final-status-level fail` passes **13 tests, 0 failed/skipped**, 103.378 s,
ci/force-validation/baseline 0, run `86e7d8e6-300f-432d-b67c-d127610fa8f5`.
This includes explicit private/missing output refusal through registered producer
completion and complete failed/cancelled terminal records. It precedes named-input binding.

**Implemented — named inputs:** Driver and cold producer restoration bind each
present input to its declared native role. Relation-qualified aliases point to the
same immutable provider only when the relation name is unambiguous, including across
versions. Optional absence has no provider; a present empty relation keeps its role.
`InputBundle::rows` is deleted. Schema-keyed extraction remains an explicit operation
for algorithms requiring one selected relation per schema; physical inventory selection
now refuses ambiguous physical inputs instead of choosing the first. The added native
before/after test exercises distinct producer outputs through the Driver and cold reopen.

**Tested — incomplete first named-input run:** `just test-package pse-tests-engine
--test stage_ports --test memo_dependencies --profile ci --no-fail-fast
--status-level fail --final-status-level fail` reports **9 passed, 2 failed,
0 skipped**, 59.972 s, ci/force-validation/baseline 0, run
`9f113787-a575-430a-bd53-bffbac9e6412`. Both failures are nextest launch errors: the
compiled `stage_ports` executable disappeared before their test bodies started.
The actual before/after test therefore remains unverified by this receipt. The
subsequent rebuild and complete rerun passed; unrelated editor processes were left untouched.

**Tested — named inputs:** the complete same-command rerun passes **11 tests,
0 failed/skipped**, 72.737 s, ci/force-validation/baseline 0, run
`27c7e189-357d-4f22-8d27-70bdd518d36f`, after rebuilding the cleared target artifacts.
The before/after producer computes from two exact same-schema output ports; Driver
execution and cold admission both produce the independently expected value. This
includes the version-aware alias binding, and precedes the policy-key decoding deletion.

**Implemented — policy selection:** stage-key construction now borrows the admitted
Arrow identity column directly and checks cancellation while selecting the actual
policy key. The projected batch, generic Cell row decoding and its allocation forecast
are deleted. Full selection membership remains required; no identity/hash shortcut is added.

**Tested:** `just quality` passes with **0 findings**, including Python typing,
import boundaries, repository checks and 14 setup tests. This receipt follows the
producer/input changes and precedes the small policy-key follow-up.

**Tested — complete foundation rerun:** `just test-package pse-compiler -p pse-schema
-p pse-rules -p pse-mathir -p pse-quantity -p pse-relations -p pse-catalog -p pse-authoring
-p pse-templates --no-fail-fast --status-level fail --final-status-level fail` passes
**568 tests, 0 failed/skipped**, 15.122 s, default/force-validation/baseline 0, run
`1dffcf59-475a-4f3e-be6a-be4e4ccd966b`. This includes direct Arrow policy selection,
the common producer result, and native port tests for repeated schemas, distinct
versions, absent/empty inputs and exact shared-provider aliases.

## Pre-reboot execution checkpoint — 2026-09-14 (historical)

**Paused at the maintainer's request before reboot. The full implementation is
incomplete, and the compiler is not buildable.** Current source is preserved in
the working tree with no commit/push. The
[restart handoff](05-native-logical-plan-hard-pivot-restart.md) records concrete
APIs, file ownership, source integration order, compiler diagnostics and all
remaining work. Resume that work rather than reconstructing the former row APIs.

| Package | Saved implementation boundary | Still required |
|---|---|---|
| HP00 | **Implemented:** ADR-0067, blueprint revision 37, active instruction reconciliation. Earlier documentation gates passed within their recorded scope. | Final active-doc/deletion audit belongs to HP13. |
| HP01 | **Implemented, integration incomplete:** generated typed Arrow builders/views, private checked batches, native collection and reserved concat, generated wire/physical/MathIR adapters. | Complete regeneration and consumers; remove residual row encoders. |
| HP02 | **Implemented, incomplete:** actual prepared/completed native computations, frozen session, exact source/function ownership, registered private producer completion. | Transfer established field properties without full output value re-admission; complete consumer and ownership tests. |
| HP03 | **Implemented, incomplete:** native constructors/schema semantics, PK/FK/nested-ordinal obligation plans, native scalar field adapters and affected-input selection. | Remove remaining repeated value comparisons/full P2 checks; test affected FK/negative scopes and actual compiler consumers. |
| HP04 | **Implemented, incomplete:** native provider execution, owned streams, wakeable cancellation, configurable generous budgets, actual physical observations. | Eliminate duplicate buffer reservation/re-admission, complete runtime tests and mutable publication cancellation semantics. |
| HP05 | **Implemented, incomplete:** owned authoring/change/source-binding path; typed P3 lowering and native primitive/selector/product helpers. | Replace P3 main; couple configuration/units/paths to exact native provenance; replace demand-seed relational loops; finish batch ChangeSet wire. |
| HP06 | **Implemented, incomplete:** local source/stage private completion and exact in-memory memo owners. | Faithful durable policies/parameters/auxiliary contexts; remove full copied context limits; CAS visibility/durability outcomes; complete producer/terminal-record/physical observation integration and lifecycle tests. |
| HP07 | **Implemented, incomplete:** native relation/rule/delta/support construction; focused 27-test gate green. | Real compiler integration, complete retraction/negative-scope behavior and deletion of remaining callback paths. |
| HP08 | **Implemented source, unverified product:** reference packages, shared native physical inventory, checked quantity/MathIR adapters, primary-source scientific tests. | Run scientific and adapter tests; verify complete physical/state/method contracts in actual workflows. |
| HP09 | **Incomplete:** shared native source binder; P5 port-walk declaration; unlinked port traversal WIP. | Async checked IndexEvaluator, complete native P4–P6, checked output ports, exact demand/support behavior. |
| HP10 | **Incomplete:** P7 typed ingress, P8 native contexts, P9 native selection and P10 checked production replacement. | P7 seed/output/evidence, P8 graph/law expansion, P9 callback/parameter/kernel replacement, all caller integration and complete graph/provenance behavior. |
| HP11 | **Implemented source, unverified:** settings, reader lifetimes/errors, introspection stubs and Python stream tests. | Rebuild extension, regenerate/check stubs and execute immutable inspection tests against actual admitted graphs. |
| HP12 | **Not established:** no complete target workflow or performance receipt. | Real FTPx/FcTP heater+mixer P0–P10, publication/reopen/Rust/Python inspection and independent graph/physics/provenance expectations; target measurements. |
| HP13 | **Not established:** historical paths have been deleted as replacements land, but final removal is incomplete. | Delete fixture/legacy routes, regenerate target artifacts, run terminal recipes and focused actual-code G1–G7 review. |

**Verification — baseline 0:** all nextest receipts below use the default profile
and explicit Arrow `force-validate` through `just test-package`. They cover the
named targets only and do not certify later source edits or complete packages.

- **Tested:** native rules, 27 passed / 0 failed / 0 skipped; authoring,
  31 passed / 0 failed / 0 skipped; catalog fields/constructed ordinals,
  6 passed / 0 failed / 0 skipped. Exact commands, run IDs and local log paths are
  in the restart handoff. These supersede earlier partial receipts for those
  focused targets only.
- **Tested:** `just test-package pse-catalog --lib scalar::index_tuple
  scalar::list_concat --no-fail-fast --status-level fail --final-status-level fail`:
  4 passed, 0 failed, 102 filtered; run `1d753a83-e538-4a77-ab62-a8de554f4300`.
  The two concat failures in the first attempt were fixed and this rerun passed.
- **Tested:** `just test-package pse-relations --test generated_contracts
  --test typed_collection --no-fail-fast --status-level fail --final-status-level fail`:
  13 passed, 0 failed/skipped; run `5980567d-b8c4-4870-9f37-a201e370c81e`.
  This covers the new reserved concat/canonicalization and typed collection cases.
- **Tested — failing build:** `just check-library pse-compiler`, dev/locked/library
  mode: 56 compiler errors, exit 101, no tests run. Errors occur at the documented
  P3–P9 caller transitions. The new P3 helper/product and P10 modules reported no
  errors in this check; their behavior remains untested.
- **Tested — incomplete generation:** `just codegen-bootstrap` generated Rust
  contracts successfully, then failed while building the full compiler-dependent
  generator. Complete generation is still required.
- **Not run to completion:** real HP12 workflows, fresh Python runtime acceptance,
  performance evidence, terminal release/parity suites and final G1–G7 review.
  Doctor's stale Python environment/extension remains to be refreshed once the
  native replacement compiles.

**Tested — checkpoint documentation:** `just docs` builds with the existing
large-search-index warning; `just lint-typos` and scoped `git diff --check` report
zero failures, baseline 0. A scoped `.venv/bin/python` check of six edited documents,
required plan front matter, whitespace and 136 local links found zero failures.
These checks do not establish runtime behavior.

Final checkpoint receipts are also recorded in [STATUS.md](https://github.com/paul-heyse/pse-arrow/blob/main/STATUS.md).
All three implementation agents are stopped; the final process inventory contains
no Cargo, rustc, nextest, xtask, maturin, pytest or mdBook process.
There is no scope or approval question pending. Restart with the saved integration
work, then complete all remaining packages and terminal gates. This user-requested
pause does not change the zero baseline or narrow the plan's scope.

## Outcome (recorded after implementation)

### What was built

Implementation is in progress; see the execution checkpoint. Replace this section
with actual HP00–HP13 completion and terminal evidence when achieved.

### A mistake made and corrected

Record an actual implementation correction when execution is complete.

### Deviations from the plan, deliberate

Record actual deviations and their reasons; decision changes reference the relevant
ADR. The hard-pivot strategy above is the planned approach, not a deviation.


**Implemented and Tested — current native rules:** rule column bindings now preserve
logical names independently from native join scopes. Assertion deduplication uses a
native window to retain original Arrow payloads, including signed zero and nested
fields. Fact derivation links select an actual ordered assertion identity and use the
shared `pse_require_nonnull` UDF; the NIL fallback is deleted. The complete rules
command `just test-package pse-rules --no-fail-fast --status-level fail
--final-status-level fail` passed **49 tests, 0 failed/skipped, 0 warnings**, nextest
default with Arrow `force-validate`, baseline 0, run
`967692fe-169f-4d8e-b112-54a69eb06dd7`.

**Tested — complete compiler package:** `just test-package pse-compiler
--no-fail-fast --status-level fail --final-status-level fail` passed **28 tests,
0 failed/skipped, 0 warnings**, nextest default with Arrow `force-validate`, baseline
0, run `91f999f0-8861-475a-abe2-c0bce1a9bf0d`. This supersedes the earlier library-only
and selected-test receipt, without claiming source-to-P10 acceptance.

**Implemented, pending workflow verification:** package unit normalization derives
its target from the explicit package base-unit set, allowing equivalent component
spellings. The source tests check resolved symbol identities and explicit conditional
demand guards. The last six-test workflow run (`b33c59c0-76ec-42c5-b9f5-7610fbfeb5d4`)
passed 3 and failed 3, with 0 skipped: the failures exposed the unit ambiguity, a
lexical-reference test expectation and P4 derivation-link metadata. Corrections are
undergoing fresh source tests. Engine admission/terminal fixtures are being migrated
to sealed actual producer registration; the empty-carrier and permissive source
projection fixtures are deleted.

**Implemented — legacy data deletion:** all 132 files in the former P10 golden
fixture store and its golden observation are deleted. Remaining golden artifacts
still require target-format regeneration; they are not an acceptance baseline.


### Native integration receipt — 2026-09-15

**Tested:** all four actual P0–P3 normalization cases passed in run
`768d4c0a-97d2-4252-af42-d2c6fa335341`; the complete six-test command had **4 passed,
2 failed, 0 skipped**. It was `just test-package pse-tests-engine --test
native_normalization --test native_template_graph --test native_reference_packages
--no-fail-fast --status-level fail --final-status-level fail`, nextest default,
Arrow `force-validate`, baseline 0. The template case completed P4 and failed in P5
on optional state identity meaning. Reference loading found an entity-name collision.

**Implemented, under fresh workflow verification:** P5 optional state targets use
checked typed NULL and the shared CASE carrier. Value meaning must agree; only common
role/FK annotations transfer. Internal method templates have distinct `.template`
qualified names; public method names are retained. The templates document declaration
now includes its existing port-member relation. Package closure recognizes an exact
version requirement with an optional `=` prefix through native `regexp_replace`.

**Tested:** `just codegen-bootstrap` completed its Rust-contract bootstrap and all
three complete schema targets, **0 errors/warnings** (bindgen remains deferred R-3).
`just fmt-check` passed Rustfmt and pinned Taplo, **0 findings**, baseline 0.
**Interface-checked:** `just test-package pse-tests-engine --no-run` compiled all
engine test targets, **0 errors/warnings**, locked with Arrow `force-validate`.

**Implemented:** engine callers use immutable source loading and sealed producer
registration. Commit requests retain `OwnedChangeSet`, including its allocation lease,
through application. Mutable parsed-document and detached rename test paths are deleted;
source mutation tests construct freshly parsed inputs. Their runtime acceptance remains
pending, including terminal fault injection and complete rename publication.

**Tested:** `just test-package pse-rules --no-fail-fast --status-level fail
--final-status-level fail` passed **50 tests, 0 failed/skipped, 0 warnings**, run
`6df7217b-d35d-40c3-9b51-b009eefcd0a7`. The added fact-link test checks the smallest
actual assertion ID and empty-input behavior. `just test-package pse-catalog --test
native_nested_output --test native_output --test filter_refinement --no-fail-fast
--status-level fail --final-status-level fail` passed **12 tests, 0 failed/skipped,
0 warnings**, run `8299221d-6d27-4147-95ac-37f4f7c6ac54`. Both use nextest default,
Arrow `force-validate`, baseline 0.
