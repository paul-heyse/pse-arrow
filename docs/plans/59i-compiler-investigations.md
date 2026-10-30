---
title: "Plan59i: Compiler investigations"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, design, libraries, diagnostics, validation]
review_sources: [docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md]
---

# Compiler investigations

## Purpose and assessed foundation

**Proposed:** give an agent an explicit route to investigate a selected compiler question:
ownership and destruction, values retained across an await, a constant, a concrete native
representation, or an unexpected generic/code-generation consequence. This companion carries
[O14](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O14) and
[O15](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O15). The
[coordinator](59-agent-cli.md) owns their dispositions and the operator-confirmed RC01–RC05;
this document owns I01–I05 and their eventual evidence.

Source and review baseline: `a569b52e4`, 2026-10-29. The preceding review read both supplied
compiler documents end to end; they are byte-identical. Their capability evidence and primary
references are retained here without restarting a general compiler/library review. Core3.4,
efficiency heuristics1.0, test assessment1.2 and process-simulator1.5 apply through the binding.
This authoring work leaves the coordinator's paused 2026-10-30 checkpoint intact and does not
restart production execution, install compiler components,
change the checker, or qualify numerical/resource behavior.

**Interface-checked:** the existing host correctness adapter, planned C07 selection/B02 result
boundaries, toolchain declaration and documented compiler mechanisms. **Proposed:** all new
investigation behavior. No pin-specific driver, consumer journey or performance claim is Tested
or Measured by this plan. The toolchain declares `rust-analyzer`, `rust-src` and `llvm-tools`,
but not `rustc-dev`. Overview/compiler development pages span compiler revisions; exact pinned
API compatibility remains a question for I05, not an established capability.

### Focused foundation assessment

The current `crates/pse-host/src/correctness.rs` owns Cargo command qualification and
force-validation. [C07](59c-tests-and-fixtures.md#c07) develops its shared selected-operation
contract; [B02](59b-runs-and-continuity.md#b02) develops capture of original compiler messages.
[D02](59d-knowledge-and-workflow-verbs.md#d02) supplies context-addressed API acquisition and
isolated probes. These are useful foundations; a new compiler inspection route must not
reconstruct their configuration or run a second check just to explain existing diagnostics.

The required addition is composition and selected interpretation. Preserve the original source,
selected package/target/features, compiler/tool generation, command result and diagnostic/artifact
origin. Cached compiler output and a current source read can disagree without either being
corrupt. Present the mismatch instead of attributing an old failure to edited code.

This separation follows AP-01–AP-06: the host presents, the existing operation resolver selects,
the compiler owns compiler facts, and scientific/runtime owners retain their own claims.
AP-07/DP-09/DP-10 favor original evidence reuse and body/pass-scoped extraction over a second
compiler database or a full workspace analysis for every question. A small rendered response
does not establish that a compiler invocation examined little code.

This revision carries the library-integration review's
[I01](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I01),
[I07](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I07),
[I10](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I10),
[I12](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I12),
[I13](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I13) and
[I14](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I14).
These review-qualified I references are distinct from this companion's I01–I05 packages.
The coordinator owns the operator's 2026-10-30 RC01–RC04 confirmations and decision routing.

## Target behavior and boundaries

Start with the least machinery that answers the selected question:

1. Read the original structured compiler diagnostic, its related spans and suggestions.
2. Use a selected source/API explanation or an explicit isolated probe where sufficient.
3. Produce a selected compiler artifact, such as filtered MIR/THIR or type-size output.
4. Use a separately prepared, pin-specific structured driver only if I05 establishes that a
   repeated useful question cannot be served faithfully and proportionately by those routes.

The response answers one question about a selected item/body/type/build context, includes
relevant source spans and conditions, and offers an optional raw artifact. The compiler adapter
owns query-specific detail/pages: original diagnostic and related spans first, selected body/
phase facts next, with raw windows or larger neighborhoods only on request. A03 supplies the
envelope and mechanical projection over those owned views, not recursive truncation of arbitrary
compiler records. Typed next actions retain arguments/effects; A04 resolves retained artifacts;
A05 identifies/protects actual prepared tools. Report examined compiler/unit/body/pass scope
separately from returned size and known omissions/unknown totals. No universal compiler query
language or permanent symbol graph is required.
Voluminous debug dumps remain inspectable provider artifacts, not a stable typed protocol
created by parsing unstable textual output.

Continuations bind original source digest/revision, artifact identity and compiler/backend
context. Refuse stale/expired continuations or explicitly serve retained immutable source.
Classify retention by responsibility: sole original diagnostics, dirty source bytes unavailable
elsewhere and uniquely authored probes require preservation; a derived explanation is regenerable
only while those inputs/context remain available. Do not persist a whole rebuildable query result
for every requested page. A04's addressed/ranged/streaming artifact reads can supply selected
windows without replacing private local publication or imposing one deletion policy on scientific
records, authored inputs and preparation caches.

Use version-checked typed decoding and checked constructors for source/context/phase applicability.
Serialize owner-validated results through borrowed typed views and directional Serde/Schemars
schemas; schema agreement does not establish a compiler fact or a valid domain combination.
Retain opaque original upstream evidence as `RawValue` when normalization would erase numeric
lexemes or duplicate keys. Fields used for selection/control are typed rather than reinterpreted
from generic JSON. Unsupported version/shape, unknown value, failed evaluation and nonfinite
categories remain explicit; numerical serialization must not silently turn them into null or zero.

Reading retained evidence is separate from generating new evidence. A missing compiler artifact
returns preparation/generation-required with an explicit next action; it does not start a build,
install `rustc-dev`, run a probe or capture profiling automatically. New compiler/probe work
uses the original correctness/placement route and the pinned toolchain, including force-validation
where the actual correctness closure consumes Arrow. Ordinary source reading, help, existing-run
control and semantic lookup do not depend on I05 or compiler-private preparation.

G's live analyzer is optional enrichment, not a prerequisite. Analyzer inference, analyzer MIR,
compiler THIR/MIR, compiler diagnostics, CTFE, emitted code and observed execution are different
providers/stages. A layout result cannot establish portable ABI agreement; a drop edge cannot
establish external-process/native callback settlement; an emitted instance cannot establish that
it executed. Rust compilation cannot establish physical-model or numerical correctness.

Illustrative activities are “explain the original borrow failure”, “show retained locals at this
await”, “inspect this concrete layout” and “compare these selected MIR passes”. Settle parser
spelling in A02/A06 when the operations are delivered; do not expose compiler-private type names
as a mandatory CLI grammar. Unknown substitutions, unsupported stages, changed source and failed
extraction remain explicit instead of becoming empty successful answers.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [I01](#i01) | planned | 59i | Original diagnostic/probe/artifact foundation with explicit generation and context |
| [I02](#i02) | planned | 59i | Selected ownership, destruction and async investigations through lighter evidence routes |
| [I03](#i03) | planned | 59i | Context-bound constant and concrete layout/ABI investigations |
| [I04](#i04) | planned | 59i | Selected instance and MIR/code-generation investigations |
| [I05](#i05) | planned | 59i | Pin-specific structured-driver decision and conditional selected implementation |

<a id="i01"></a>
## I01 — Original evidence and explicit generation foundation

**Consumes:** A03's output/source-location contract, A04's artifact resolver and A05's prepared
tool identity; C07's working selected-operation slice for new compiler actions; B02's generic
Cargo diagnostic capture for original-failure navigation. D02's isolated-probe slice supplies
probe execution. These are slices, not a dependency on all test fixtures, nextest reconciliation,
D conveniences or G sessions. Design can proceed against the agreed contracts; assembled
consumer acceptance requires the named working producers.

New generation actions consume the same C07 resolved/admitted stage as explain and execution;
explicit artifact production is a later observation stage. Missing preparation is an explicit
requirement, and changed source/tool/configuration premises refuse or re-resolve before effects.

**Delivers:** selected original diagnostic navigation preserving primary/secondary spans,
children, suggestions/applicability, macro context, source generation and actual check outcome.
Use Cargo `CompilerMessage`/rustc JSON already captured by B02; do not re-run compilation or
write a custom diagnostic emitter merely to retrieve those fields. Preserve full original
rendered/raw evidence separately when a summary cannot carry every detail.
Use [J01](59j-source-queries-and-change-proposals.md#j01) for addressed source when available;
an original location can remain useful without a live semantic session. D01 can compose these
diagnostic/source/context sections immediately, with unavailable API/semantic/compiler detail
visible; it does not wait for all G/H/I work or prepare a driver merely to explain a failure.

Explicit artifact generation accepts the selected question and unit/body/pass context, retains
the command/tool identity and returns the result plus available artifact locations. Use native
body/pass filters for MIR and selected THIR/type-size output before a custom extractor. Capture
failure can coexist with a valid compiler diagnostic or partial raw output; it never fabricates
a successful semantic result. A canceled generator must settle its actual owned work through
the existing host boundary. Original retained evidence stays readable without preparing a new
driver or current scientific product.

**Acceptance:** an original borrow/type failure retains all relevant related spans and compiler
suggestion applicability; source edited afterward produces an explicit mismatch. Reading it
does not compile. A requested body-filtered artifact identifies the actual selected context;
missing, unsupported, truncated and failed generation are distinguishable. Verify one real
selected invocation/probe through the existing correctness route, including original diagnostics
and actual failure status, rather than accepting JSON shape alone.
Follow that retained failure through D01 to a selected typed next action without a second
compilation. Include paged related spans/body detail, unknown totals, failed/stale continuation,
unknown schema/protocol and invalid source/context construction. Removing a regenerable summary
must preserve the sole original diagnostic and authored inputs needed for the original question.

**Sources:** review O10/O14; [rustc JSON](https://doc.rust-lang.org/rustc/json.html),
[MIR debugging](https://rustc-dev-guide.rust-lang.org/mir/debugging.html),
[THIR](https://rustc-dev-guide.rust-lang.org/thir.html).

<a id="i02"></a>
## I02 — Ownership, drop and async explanations

**Consumes:** I01's working original-evidence/artifact route. G02/G04 may supply an available
source-semantic explanation later, but are not required. I05 is not a prerequisite to delivering
diagnostic/artifact investigations.

**Delivers:** an agent selects an original ownership failure or one body/await/return path and
receives a source-linked explanation with supporting diagnostic/artifact references. Start with
the compiler's own diagnostic and optional source capture/type information. Use THIR to inspect
explicit automatic references/dereferences, desugared calls and destruction scopes where that
is clearer than immediately presenting a CFG. Selected MIR can expose initialization-dependent
drops, remaining fields after a partial move, cleanup/unwind edges and saved coroutine locals.

Keep storage existence, initialization, liveness and active loans distinct. Every interpreted
flow fact names the selected compiler phase/location and may/must meaning. A control-flow
neighborhood is bounded by the chosen question; do not enumerate every path or materialize a
whole-workspace ownership graph. If available evidence cannot answer the requested relationship,
return the unresolved question and I05 lead, not a guessed loan/lifetime explanation.

**Structured-mechanism leads for I05:** `TyCtxt::typeck`,
`TypeckResults::{expr_ty,expr_ty_adjusted,expr_adjustments,type_dependent_def_id}`, capture and
coroutine saved-local information; initialization/move/liveness analysis with
`Analysis::iterate_to_fixpoint`, `ResultsCursor` or `ResultsVisitor`; drop elaboration and move
paths. Borrow extraction can use `get_bodies_with_borrowck_facts`, `BorrowSet` and
`RegionInferenceContext`. It copies bodies, has phase/stolen-MIR restrictions and is not
appropriate as automatic typical-compilation work. `MaybeBorrowedLocals` is not a borrow checker.

**Acceptance:** inspect a rejected move or mutation with related source locations; inspect one
partially moved aggregate's remaining drop obligations and one guard/value crossing an await.
The response distinguishes a compiler rejection from advisory analysis and unsupported detail.
It makes no external-resource settlement, actual allocation, executor scheduling or numerical
correctness claim. Use independent small source examples with known relevant branches, not an
expected explanation generated by the same adapter under test.

**Sources:** [compiler overview §§2–5](../libraries/rust_analyzers/rust-compiler-internals.md),
[MIR dataflow](https://rustc-dev-guide.rust-lang.org/mir/dataflow.html),
[borrow extraction restrictions](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/consumers/fn.get_bodies_with_borrowck_facts.html).

<a id="i03"></a>
## I03 — Constants and concrete representations

**Consumes:** I01; D02's working isolated-probe route for selected new observations. No live
analyzer or structured compiler driver is required for the initial useful investigations.

**Delivers:** explicit questions about a constant/discriminant/const generic, a concrete type's
size/alignment/offsets, or a concrete native function boundary. A selected probe using
`size_of`, `align_of`, `offset_of!` or compile-time assertions is the first route where sufficient.
Native `-Zprint-type-sizes` and selected compiler artifacts provide deeper representation detail.
The result includes target/toolchain/substitution context and unresolved/evaluation failure states.

Do not flatten generic or pointer-bearing constant values into misleading scalar JSON. CTFE,
static initialization, constant promotion and optimization constant propagation are distinct.
Layout inspection states whether its source is a probe, analyzer computation or compiler result;
neither Rustdoc `repr`/ABI metadata nor analyzer recursive layout proves C-side compatibility.
When relevant, surface ABI/unwind and unsafe declaration context alongside the observed layout.

**Structured-mechanism leads for I05:** `const_eval_*`, static-initializer queries,
`LayoutOf::layout_of` and `FnAbiOf::{fn_abi_of_fn_ptr,fn_abi_of_instance}`. These are pin-sensitive
capabilities, not already verified bindings. Detailed niches/variants or argument-passing reports
are conditional on an actual question requiring them and a qualified extraction route.

**Acceptance:** one concrete constant and a generic/unresolved case remain distinct; a selected
type's known size/alignment/field offsets match independent compiler/probe expectations for that
target. A missing component or unsupported representation is reported without toolchain override
or automatic install. Results do not claim cross-target layout stability or portable Rust ABI.

**Sources:** review O15; [constant evaluation](https://rustc-dev-guide.rust-lang.org/const-eval.html),
[layout interface](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/layout/trait.LayoutOf.html),
[ABI interface](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/layout/trait.FnAbiOf.html).

<a id="i04"></a>
## I04 — Instances and selected code-generation consequences

**Consumes:** I01's selected compiler artifacts. Start independently of I02/I03 and I05.

**Delivers:** a selected generic-definition/instance report or a body/pass-filtered before/after
MIR investigation, linked to source and the actual compiler context. These help answer why an
abstraction causes concrete instantiations or how a chosen lowering changes it. Use native mono-item
output or established binary/LLVM inspection tooling when the question is emitted code size;
avoid reproducing those analyses in a general compiler database.

`Instance` resolution and monomorphization collection (`collect_and_partition_mono_items`) are
structured-driver leads if raw artifacts cannot support a repeated question. Preserve concrete
substitutions, shims/glue, codegen stage and unit relationships where available. Static direct
calls, dynamic/indirect edges, emitted instances and runtime execution remain distinct.

An optimization change in MIR proves neither faster generated code nor faster simulation.
Backend IR/assembly and a named workload measurement are separate explicit follow-ups if that
is the question. No baseline profiling or build-stage alerts are introduced by this package.

**Acceptance:** inspect a small selected generic example with distinguishable instantiations and
one filtered pass comparison; identify source/context and unsupported outputs. Preserve indirect
calls or unknown runtime reach instead of calling this an exact runtime graph. Repeated reads
reuse the original artifact and do not compile again. Malformed/debug-format changes fail as
unsupported artifact interpretation rather than yielding invented structured facts.

**Sources:** [compiler overview §§4–6](../libraries/rust_analyzers/rust-compiler-internals.md),
[monomorphization](https://rustc-dev-guide.rust-lang.org/backend/monomorph.html),
[filtered MIR](https://rustc-dev-guide.rust-lang.org/mir/debugging.html).

<a id="i05"></a>
## I05 — Pin-specific driver decision and conditional implementation

**Consumes:** I01's selected context/output contract and an identified recurring structured
question from I02, I03 or I04 that lighter routes cannot answer adequately. The question can be
identified during design or consumer use; completion of all three packages is not required.
Their initial artifact/probe delivery never depends on this decision.

**Question:** does a separately prepared compiler adapter supply a materially useful structured
answer without imposing compiler-private initialization on the ordinary CLI host or changing
the production compiler/checker behavior?

**Evidence:** inspect exact pinned source/interfaces, select one representative question, and
in authorized implementation qualify the minimal `rustc_driver` route. Prefer driver callbacks
over manually rebuilding `rustc_interface` orchestration. Required `rustc-dev`/LLVM components,
native/proc-macro/build-script prerequisites, phase lifetime and `TyCtxt` ownership must be
explicit. Do not substitute a floating nightly or claim current development documentation is
exact-pin compatibility evidence. Use native query caching internally; compiler-local IDs
remain scoped handles, not durable cross-revision symbols.

**Decision branches:**
- **Adopt a bounded driver capability** only when the selected question cannot be served
  adequately by original diagnostics/probes/artifacts, the pinned interface works, context and
  source mapping are preserved, original compilation semantics remain intact, and the separate
  component's preparation/lifetime/maintenance burden is proportionate. Route any new crate or
  changed enduring contract through A07's scoped decision route before dependent implementation.
  Implement that selected query, its typed availability and focused consumer tests; do not
  promise every listed compiler query.
- **Retain lighter routes** when they answer the need or the driver criteria fail. Record the
  exact unsupported structured question and observable revisit trigger here. A completed inquiry
  with a no-driver decision does not establish availability of the deferred structured query.

**Acceptance:** record the selected question, alternatives, exact-pin evidence, decision and
affected consumer scope. An adopted component has one actual source-linked consumer journey
and unsupported/phase/context failure controls; missing preparation leaves original diagnostics,
syntax/API reads and ordinary control usable. Report cancellation/settlement independently.
No universal driver, whole-workspace pass or compulsory performance campaign is required.

**Source:** [external compiler drivers](https://rustc-dev-guide.rust-lang.org/rustc-driver/external-rustc-drivers.html)
and review O14/O15 realization discussion.

## Conditional capabilities and non-adoption

| Mechanism | Disposition and observable trigger |
| --- | --- |
| Structured typeck/THIR/MIR/borrow facts, layouts and instances | I05 candidate only after a selected recurring question exposes a gap in I01–I04's lighter routes. |
| Polonius input/output fact export | Deferred research lead. Revisit when an actual ownership visualization/research consumer needs relations beyond the selected compiler explanation; preserve source/location/phase scope. |
| Polonius execution/comparison | Not part of routine investigation. Requires a separately selected research question and exact-pin semantics; never change the production checker just to explain code. |
| Custom MIR passes, rewriting or replacement borrow checker | Not adopted. This target inspects compiler behavior rather than modifying compilation or owning new soundness obligations. |
| Analyzer internal function interpretation | Not a trusted evaluator. Keep experimental provider identity; use compiler CTFE/probes/native execution for their respective claims. |
| Compiler timings/self-profile/measureme | Conditional tool-debugging only when analysis/compiler cost is itself an explicit blocking question. No build telemetry, routine profiling or application-performance conclusion. |
| Whole-workspace compiler database or retained phase graph | Not selected. Revisit only for an actual repeated cross-item consumer that cannot use the scoped native outputs; account for source/toolchain identity and retention before adoption. |

## Dependencies, verification and handoff

I01 establishes the usable foundation. I02, I03 and I04 can then proceed independently with
their actual available prerequisite slices. I05 consumes a concrete unanswered question and
can branch into one specialized structured capability without delaying useful artifact/probe
investigation. There is no dependency on completing G/H, all nextest reconciliation, native
scientific fixtures or product observability. Optional source navigation and API explanation
compose as they become available. D01 owns any combined investigation packet; I supplies its
original evidence, context and optional detail rather than creating a new handoff/status ledger.

During implementation, run targeted compile checks and unit/consumer tests for changed adapters,
plus the explicitly selected compiler/probe journeys above. Delete only genuinely replaced
inspection consumers with their migration; raw tools and independently useful source/API paths
are not legacy merely because a CLI wrapper exists. Scope-end static/integrated qualification
belongs to the coordinator under AGENTS.md; scientific campaigns are not implied by this tool scope.

**Proposed completion:** I01–I04 expose their selected useful questions through qualified lighter
routes, I05 records a supported decision and completes only any actually selected branch, and
an agent uses a representative ownership or native-boundary journey. A deferred specialist
question remains unavailable explicitly. No speed, memory, ABI portability, scientific success
or external-resource drain claim follows from this completion.

**Current state:** all packages planned; no compiler component installed and no new command
implemented or tested by this document. The coordinator owns the next authorized execution wave.
