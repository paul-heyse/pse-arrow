---
title: "Plan59i: Compiler investigations"
status: complete
date: 2026-10-29
doc_role: plan
doc_retention: while-dependent
doc_retention_reason: "Retained complete series at maintainer request; Plan58, Plan60, proposed ADRs and qualification references consume its evidence."
doc_retirement_trigger: owner-release
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, design, libraries, diagnostics, validation]
review_sources: [docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md, docs/functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md, docs/functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md]
---

# Compiler investigations

**Complete:** selected delivery is closed; the [final Plan59 conclusion](59-agent-cli.md#current-state-and-next-step)
records the worktree-removal deferral and qualification limits. Earlier targets and dated
checkpoints below retain their original context; package tables and Outcomes record delivery.

## Current extension — 2026-10-31

**Interface-checked and Tested within the bounded inquiry:** I06 completed the ownership/async/dataflow investigation selected from
[O12](../functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md#O12).
The maintainer explicitly accepted investigation plus conditional implementation on 2026-10-31
without requiring historical recurrence. I05's completed 2026-10-30 no-driver decision and its
evidence remain historical; its recurring-question trigger does not govern this new inquiry.
I06 assessed exact pinned APIs and actual source cases, selecting the existing original
diagnostic and explicit MIR/probe routes. I07 is not adopted; its typed location-specific facts
and revisit trigger remain explicit in the outcome below. No compiler component is installed.

The same accepted `search`/`inspect`/`propose` cutover moves passive original diagnostics and
artifact reads into common inspection through M03, preserving distinct evidence, context,
outcome and coverage. Compiler generation/probes retain explicit effectful owners. K owns
result/locator meaning and A10 its publication; compact shared provenance does not turn current
source into original compiler evidence. Existing completed rows and outcomes do not qualify
I06/I07. M03/P02 own the actual composed public journey and integrated acceptance.

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
| [I01](#i01) | complete | 59i | Original B02 detail, explicit generation, context/index and same-descriptor digest reads delivered; main public probe→read→safe removal→retained read passes |
| [I02](#i02) | complete | 59i | Native rejected-move detail and selected drop/coroutine MIR/THIR evidence tested; structured flow facts remain explicitly unsupported |
| [I03](#i03) | complete | 59i | Authored isolated constant/layout evaluation and encountered type-size output tested; unresolved substitutions preserve actual compiler failure |
| [I04](#i04) | complete | 59i | Native selected-unit mono-items and body/pass-filtered before/after MIR retained as opaque original artifacts and tested |
| [I05](#i05) | complete | 59i | Lighter routes selected; no structured driver adopted without an unanswered recurring question meeting the criteria below |
| [I06](#i06) | complete | 59i | Named borrow/await, partial-move and saved-state cases assessed against exact pinned compiler APIs and original diagnostic/MIR evidence; bounded no-driver decision below |
| [I07](#i07) | complete | 59i | Conditional branch not adopted for I06's selected consumer; typed loan/initialization-at-location queries remain unavailable, with an explicit revisit trigger |

<a id="i01"></a>
## I01 — Original evidence and explicit generation foundation

The first slice exposes `compiler diagnostics INVOCATION` and addressed `compiler diagnostic`
reads through the existing authenticated performing report and checked private artifact descriptor.
It retains primary/secondary spans, child notes, exact replacement/applicability, macro context and
optional opaque original JSON. Related records page against the original report, selected bytes,
descriptor metadata and decoder/context identity. These observations never compile. Optional
`--source` compares original native lines to independently acquired current source, maps original
line/Unicode-scalar coordinates into current bytes and distinguishes changed raw offsets. B02 did
not retain a full original-source digest or actual rustc version; both remain explicitly unknown,
even when visible lines match. The original prepared host and admitted selected compiler context
remain mandatory envelope provenance under projection. A native E0382 fixture already exercises
the existing correctness admission/adapter and B02 decoder, then changes source before reading
the retained native diagnostic. No pin-specific driver or implicit replay is selected.

The follow-on native slice exposes explicit `compiler generate`, authored `compiler probe`,
passive `compiler artifacts` and digest-guarded bounded `compiler file` reads. It consumes the
shared correctness owner’s selected-cwd `rustc` admission and the original managed foreground
capture/finalization owner. A unique input context retains actual selected units/features/platforms,
tool versions, flags, declared root/probe source and copied authored inputs. These source snapshots
do not prove every transitive byte consumed by rustc. Successful native outputs are copied into
private retention with file digests before terminal publication; failed/unmatched output retains
its original child/capture/finalization facts and distinguishes an unpublished index from a
subsequently missing one. Repeated reads verify original full-file SHA on the same checked
descriptor before returning only the requested byte extent. That scan cost is explicit; no opaque
ETag is substituted for a content identity and no file path is reopened after its custody check.

Native dump generation selects `CARGO_INCREMENTAL=0` and `SCCACHE_NO_CACHE=1` locally so repeated
MIR selections reproduce their side-effect files and THIR does not ask the cache to store an rlib
it intentionally did not emit. Normal work and authored probe evaluation keep their selected
cache behavior. MIR uses rustc's body/pass filter grammar; unsupported body filters for THIR,
layout and mono output refuse instead of implying narrower scope. On the pinned Linux x86-64
toolchain, the targeted native cases retain partial-move `ElaborateDrops` before/after files,
saved coroutine-value `StateTransform` output, THIR, a concrete two-String layout, distinct u32/u64
mono instantiations, an authored constant/layout result and an unresolved generic compiler error.
These establish original evidence delivery, not a structured flow/ABI/runtime interpretation.

**Tested (worker checkout, Linux x86-64, pinned nightly, 2026-10-30):** public authored layout
evaluation and filtered MIR generation returned native exit 0 with complete settlement; passive
index and 16-byte file reads verified original whole-file SHA without compilation. A real B02
check retained E0382 and the additional original `non_ascii_idents` diagnostic. Its Cargo phase
exit 101 remained distinct from helper exit 1. After editing the temporary source, addressed
detail reported `visible_lines_differ`, left whole original-source identity unknown and paged
to the native `.clone()` suggestion with `MachineApplicable` confidence. The temporary source
was removed; original native evidence remains outside `target/`. These are local component/public
journeys, not main-checkout assembled acceptance.

The new compiler-probe consumer also closes D02's exposed lifetime gap through its original
owner: each run publishes a durable exact invocation binding before launch, and explicit removal
requires matching original metadata plus actual foreground root drain under the same probe lease.
Worker public journeys removed both a successful plain probe and a failed E0282 compiler probe
after complete native settlement, while an unknown legacy capture refused removal. Original
calculation outcome stays separate from native release; borrowed enclosing roots remain protected.

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
an original location can remain useful without a live semantic session. I exposes source/context
links directly, with unavailable API/semantic/compiler detail visible; no generic composer,
complete G/H/I population or new driver is needed merely to explain a failure.

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
Follow that retained failure directly to a selected typed next action without a second
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

**Decision, 2026-10-30 — retain lighter routes.** The selected questions are original move/borrow
explanation, remaining drops after a partial move, saved values across suspension, concrete
constant/layout evaluation and selected instantiated/code-generation evidence. Actual native
diagnostics, before/after MIR, coroutine/THIR/type-size/mono outputs and authored probes answer
those questions at their stated scope. A compiler-private driver would add a second compiler
integration and pin-specific maintenance without a selected unanswered structured query.
No driver is installed. Structured loans/liveness, exact substitutions and typed compiler
queries remain unavailable, rather than being inferred from text. Reopen I05 when an actual
recurring question cannot use those retained original outputs and a bounded typed query plus its
source/toolchain/preparation ownership justifies the component. This completes the conditional
inquiry, not the unavailable capability.

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
link as they become available. I supplies original evidence, context and optional detail
directly; no generic investigation-packet or handoff/status ledger is required.

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

## Outcome

**Implemented:** selected I01–I04 and the I05 non-adoption decision are integrated. Compiler
work uses existing correctness admission, selected cwd and the original managed foreground
owner. Original diagnostics and artifact reads stay passive. Probe deletion uses its existing
exclusive lease and authenticated original native drain, independently of calculation success.

**Tested, 2026-10-30:** the merged main-checkout targeted command selected `worktree::`,
`probe::`, `compiler_investigation::`, `artifacts::` and `cli::compiler::`:34/34 passed, zero failures
and zero compiler warnings. It includes real native partial-move/coroutine/THIR/layout/mono
outputs, original related diagnostics, descriptor/hash controls and probe-reference refusals.
The main public `pse compiler probe plan59-i-main --question ...` produced invocation
`01a18f3b-b91e-7474-92e7-7bfe3b28a407`, child0, complete capture and settlement.
`compiler artifacts` and `compiler file` reopened the original index and whole-file-verified
bounded output, reporting `count=13; size=16; align=8` for the authored captured-host question.
`probe rm plan59-i-main` succeeded after native drain; the retained compiler output remained
readable with its original digest after probe removal. Worker native and original-failure
journeys above retain their original checkout/context, not main identities. Scope-end whole-series
qualification remains with P01.

**Mistakes corrected:** native dump side effects cannot rely on ordinary cache hits;
`SCCACHE_RECACHE` attempted to cache absent THIR outputs. Explicit dump stages now select
`SCCACHE_NO_CACHE=1` and disable incremental reuse locally. Original probe leases also needed
durable prelaunch references to retain protection after an unresolved foreground return; missing
or legacy custody now refuses deletion rather than assuming that a terminal row proves release.

**Deliberate limits:** compiler-private output is opaque and exact-pin-specific. It establishes
neither a structured borrow model, portable ABI, runtime reach nor a performance benefit.
Arbitrary original full-source identity cannot be reconstructed from snippets; a guarded semantic
next action is explicitly a separate current-source observation. No performance claim is made.

## Planned search extension packages

I06/I07 feed the [composed investigation plan](59m-composed-rust-investigations.md), using
the [result/locator foundations](59k-query-and-result-foundations.md) at their public boundary.
They supersede the recurring-question restriction for this new inquiry only; I01–I05 evidence
and the historical no-driver outcome are unchanged.

<a id="i06"></a>
### I06 — Named ownership, async and dataflow feasibility

**Consumes:** working I01 original-evidence/artifact identity, I02's current ownership/async
scope and C07's selected/admitted compiler context. No G11/G12 or live semantic session is
required. The maintainer selected this inquiry on 2026-10-31 without a recurrence prerequisite.

Select a concrete structured question: which initialization state and active loans apply at a
named MIR location, or why a named value remains live/stored across a suspension. Compare the
actual diagnostic, saved coroutine/drop MIR and explicit probe routes against a pin-specific
driver; explain the useful relationship the existing opaque artifacts cannot faithfully answer.
This preserves I05's lighter-route evidence without treating opaque dump availability as typed
dataflow support. Native analyzer MIR/evaluation is a distinct provider, not rustc borrow facts.

Inspect exact pinned `rustc_driver` callbacks and
`rustc_borrowck::consumers::get_bodies_with_borrowck_facts`/`ConsumerOptions`, plus native
`iterate_to_fixpoint` with `ResultsCursor`/`ResultsVisitor`. Keep value liveness, storage liveness, initialization and
active loans separate; record body, MIR phase, location and may/must interpretation. Inspect
query timing, copied/stolen MIR, `TyCtxt`/body lifetime and when facts must be requested before
other consumers take the body. Do not recover borrow facts from source spans or facts generated
under a different Polonius mode. Check `rustc-dev`, LLVM, target, build-script/proc-macro and
source-custody prerequisites against this checkout's pinned nightly rather than floating docs.

**Decision and acceptance:** use bounded source cases with independently expected partial-move
initialization and value/guard retention across await, including a branch where storage and
value liveness differ. Record exact API/phase evidence, selected relationship, source mappings,
preparation/lifecycle/upgrade burden, limits and an adoption or no-adoption decision. A positive
decision must show why the typed answer is useful and feasible; a negative decision names the
unanswered relationship and observable trigger, such as a compatible extraction API/component
becoming available. No historical usage count is needed. A feasibility probe is evidence only
for its named compiler/context, not production implementation or a safety proof.

**Handoff/deletions:** A12 publishes only selected compiler-driver governing changes. This
inquiry does not gate A11 or ordinary discovery and deletes no original diagnostics/artifacts.
If lighter routes suffice or exact-pin ownership cannot be qualified, record no adoption and
leave structured facts explicitly unavailable. The plan revision itself installs no component,
changes no toolchain and runs no experiment.

#### I06 outcome — original ownership and suspension evidence

**Interface-checked:** the selected compiler is `rustc 1.101.0-nightly`, commit
`c1070d69382b8d2f2eb65119c738a77d9e324c9e` (2026-09-28), from the repository's pinned
nightly. Its exact source exposes
[`get_bodies_with_borrowck_facts` and `ConsumerOptions`](https://github.com/rust-lang/rust/blob/c1070d69382b8d2f2eb65119c738a77d9e324c9e/compiler/rustc_borrowck/src/consumers.rs),
including copied body/promoted MIR, borrow sets, region inference and optional Polonius inputs.
`PoloniusInputFacts` does not run Polonius; `PoloniusOutputFacts` does. Neither can be
substituted for facts from another checker mode. The API requires a type-check root and warns
that already-stolen bodies can panic; its supported interception lead is an override of
`mir_borrowck`, rather than a later opportunistic read. The exact
[`after_analysis` callback](https://github.com/rust-lang/rust/blob/c1070d69382b8d2f2eb65119c738a77d9e324c9e/compiler/rustc_driver_impl/src/lib.rs)
runs after the compiler's analysis query. Native
[`iterate_to_fixpoint` and results cursors/visitors](https://github.com/rust-lang/rust/blob/c1070d69382b8d2f2eb65119c738a77d9e324c9e/compiler/rustc_mir_dataflow/src/framework/mod.rs)
can answer location-specific analysis questions while the matching `TyCtxt`, body, phase and
domain remain valid. A structured driver is feasible in principle; a naive after-analysis
adapter would not preserve these ownership/timing requirements. `rustc-dev` is not installed
in the selected local toolchain; this is a preparation obligation, not capability absence.

**Tested:** bounded standalone source cases in `/tmp/plan59-i06.WF55r6`, using the pinned
compiler from the activated checkout, address the selected consumer question: which source
operation conflicts with a borrow retained across suspension, and which values the generated
coroutine retains. `rustc --edition=2024 --crate-type=lib --emit=metadata --error-format=json
borrow.rs` returns the independently expected `E0502` (exit 1): original borrow, conflicting
mutation and later use remain source-linked in the original diagnostic. The valid companion
`retained.rs`, compiled with `--emit=mir`, succeeds and preserves the original coroutine's
`Vec`, reference and awaiter fields. A second successful `--emit=mir -Zmir-opt-level=0
partial.rs` case conditionally moves one `Pair` field before suspension, retains the other,
and exposes original conditional drop flags. Its consumed-before-await companion keeps only
the awaiter in the coroutine's saved fields, while the original MIR still shows the local's
storage operations. This distinguishes saved state, conditional initialization/drop behavior
and source-level storage scope without asserting a parsed may/must dataflow result. The
expected invalid case is the probe's subject, not a passing production correctness test.
Earlier launcher attempts rejected moving host preparation before reaching `rustc` and supply
no compiler acceptance evidence.

**Decision:** retain I01/I02's original diagnostics, explicit compiler generation and opaque
saved MIR for this selected consumer; do not adopt a compiler-private driver or I07 production
protocol. They answer the concrete conflict/retention investigation without a new query
override, compiler component, checker-mode selection or pin-specific typed protocol. This
does **not** answer the separate structured question “which active loans or maybe-initialized
move paths hold immediately before MIR location L?” Those facts remain explicitly unavailable;
MIR text, source spans and the native analyzer must not synthesize them. Reopen I06/A12/I07
when a consumer selects that exact location/phase/domain relationship and needs machine-readable
states rather than the original evidence. There is no recurrence threshold. No component,
toolchain, Polonius mode or production compiler lifecycle was changed, and no safety or
performance claim follows from these bounded probes.

<a id="i07"></a>
### I07 — Conditional structured compiler extraction

**Consumes:** a positive I06 decision with exact selected query/phase/context, the applicable
A12 governing publication and existing I01/C07 execution/custody owners. Public inspection
integration requires working K02 result/locator contracts. There is no A11/G11/G12 prerequisite.
If I06 selects no driver, record the branch as not adopted with its trigger; no driver is built.

Implement only the selected extraction in an explicitly prepared compiler-private component,
separate from ordinary host initialization. Use driver callbacks and native dataflow/query
machinery; extract plain typed facts while compiler-local handles are valid. Preserve original
selected units, compiler/tool identity, flags, phase/body/location, original diagnostics and
native calculation/capture/settlement outcomes. Generation remains an explicit admitted action;
passive inspection reuses retained evidence and never installs, compiles or changes checker mode.
No stable typed protocol is fabricated by parsing MIR debug text.

**Public consumer and acceptance:** explicitly generate a selected ownership/async observation,
then inspect the source-linked phase/location facts and original evidence without regeneration.
Challenge partial moves, branch-dependent initialization, storage versus value liveness, active
loans and suspension captures using independently authored cases. An unsupported phase/query,
stolen/unavailable body, wrong pinned component, unknown substitutions, source/context mismatch,
stale locator and unavailable preparation must refuse or report unavailable distinctly from no
facts. Cancel actual generation and verify native settlement; retained original diagnostics and
ordinary source/API inspection remain usable. No result establishes general safety, runtime
reach, actual allocation, callback drain, ABI portability or performance.

**Deletion and limits:** remove only the replaced inspection adapter/duplicate extraction and
its callers/tests when the selected typed query is proven. Preserve opaque original artifacts
and useful explicit compiler generation/probe routes. M03 migrates passive diagnostic/artifact
read parser branches, guidance and tests only with working common replacements. No custom MIR
pass, replacement borrow checker, whole-workspace database or automatic compiler loop follows.
Targeted compilation and selected native consumer tests cover the adopted branch; the coordinator
owns scope-end integration/static qualification. I06 no-adoption evidence cannot claim I07 facts.
