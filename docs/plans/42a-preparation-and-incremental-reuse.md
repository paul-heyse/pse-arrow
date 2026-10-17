---
title: Preparation and incremental reuse
status: done
date: 2026-10-16
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
scenario_sources:
  - docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
doc_owner: docs/plans/42-post-plan41-simulation-efficiency-and-diagnostics.md
doc_topics: [design, preparation, libraries, validation]
---

# Plan 42a: Preparation and incremental reuse

## Purpose and consumed foundations

This companion supplies F02/F03/F04/F08 and inquiries I04–I06 to [Plan42](42-post-plan41-simulation-efficiency-and-diagnostics.md).
It owns retained preparation consumers, narrow checker work, operation-local source lookup and
warm structural/attribution lookup. It does not own solver iteration, change scientific identity
or replace current admission with a cache hit. The coordinator owns finding dispositions and
combined acceptance; this document owns A-package progress.

**Interface-checked — baseline 2026-10-16:** the [preparation inquiry](../design_review/evidence/post-plan41-simulation-efficiency-and-diagnostics-2026-10-16/preparation-and-reuse.md)
and current `expression.rs::check_member_indices`, runtime `workflow/modeling/views.rs::bound_case`
and `math/products.rs::attribute_modeling_view` confirm the affected seams. The coordinator
records the exact baseline. **Proposed:** all changes and verification below; none ran in authoring.

Declared execution already retains a prepared model. Solve, declared study and declared dynamics
already consume it in relevant paths; benchmark and Python diagnostic composition detach it.
The compiler already has complete structural products, and Plan41 already supplies indexed root
lookup and immutable lexical bases. Preserve those foundations rather than reopen their resolved
findings. The remaining checker environment, inherited-member expansion and pre-hit construction
are distinct mechanisms.

## Target preparation and consumer composition

### A1 — Retain the preparation the operation actually admitted

The benchmark carries the declared execution product through case resolution and execution,
rather than extracting only its analysis and requesting preparation again. Preserve the original
edit/start/result assertions and the distinction between a changed analysis and an unchanged
retained product. A retained model is usable only for the actual revision, instance, structural
bindings, demand, compiler/physical context and procedure interpretation; no unconditional reuse
across edited horizons or experiment instances is permitted.

Python diagnostic entry points consume a retained model through the diagnostic owner's case
resolution policy. Diagnostics still allows missing free values and does not acquire a solve
procedure requirement. Keep analysis-only preparation where callers actually have only analysis;
remove the affected redundant detachment/re-resolution paths once every caller has moved. A
shared lower case resolver may carry explicit admission policy; do not route all diagnostics
through `prepare_declared` or add a generic preparation cache.

Cover all diagnose variants and retained benchmark consumers, not just one representative call.
Legitimate changed demands still prepare their own model. The old combined cold/warm phase times
do not quantify the duplicate preparation's contribution or promise a saving.

### A2 — Skip unused scalar index environments

`check_member_indices` first checks arity. When both index lists are empty, it can finish without
building `declaration_environment` if the enclosing checked-package contract already establishes
the validity that environment construction would otherwise discover. Inspect actual callers
and malformed-declaration paths before selecting the fast return; if a required validity check
is still needed, retain that explicit narrow check without building an unused broad environment.

Nonempty indices retain sequential local bindings, contextual domains, error location and kind
checking. Contextual/refined references, lowered functions and unindexed `Type::Indexed` exports
remain meaningful controls. This is a checker responsibility with checker dependencies, not
a runtime/store optimization. No package-wide persistent cache is required.

### A3 — Borrow source keys; investigate root memoization separately

Reduce owned key creation and candidate/base copying in `SourceIndex` using borrowed access or
nested maps, preserving direct precedence, declaration ordering, multiplicity, guarded dedup,
potential representatives, shadowing and completeness. Review ingress, local selection, delta
closure and replay consumers together; the index is operation-local.

I04 determines whether additional completed root-result reuse is justified. Both insertion and
consultation must require the same empty probing context and unchanged indexed frontier.
Insertion/completeness changes invalidate relevant reuse; `Pending` must not become cached
absence. The existing recursion cut returns `Absent` for an active key; even a nested positive
result may depend on that path. Never consult a root memo inside arbitrary recursive probes.
Root memoization reproduces the current resolver observation, not a graph-closure certificate.

Use cycles, diamonds, guarded inheritance, shadowing, pending acquisition and frontier mutation
to distinguish alternatives. Adopt memoization only if those controls establish equivalence
and source/work observations justify its extra retained state. A supported no-adoption result
settles I04; borrowed access/reduced copying remains the required narrow correction. No global
cache, SCC rewrite or long-lived source authority is selected.

## Target warm lookup and evaluator investigation

### A4 — Lookup before immutable expansion, with independent attribution

Current `bound_case` constructs full bound structure before calling the solver-view cache, then
rebuilds occurrences before comparing attribution. Split the operation into a compact complete
lookup derivation and miss-only expansion from immutable prepared products. The public scientific
view-key meaning remains complete and unchanged. Preserve fixed/discrete state, row/instance
ordering, result-affecting bounds, derivative order, compiler profile, demand and physical context.

Separate bound tightenings and fresh validity from reusable mathematical construction. A hit must
still reject stale or invalid state and bind new values; it cannot inherit expired permissions.
An allocation/owner token alone is insufficient identity. Any internal precursor identity must
be derived from all relevant immutable premises and state overlays, with exact equality/validation
where the existing contract requires it.

Attribution uses its own retained exact occurrence identity. Equal mathematics does not prove
equal current-revision spans, document identities or occurrence mapping. Retain/share already
owned immutable occurrence data where its exact identity agrees, otherwise construct current
attribution. Do not keep a previous complete revision owner alive merely to reuse one map.

I05 records which hit-path construction disappears and which fresh checks remain. Use local
construction observations and value/bound/fixed/discrete/structural/span-only edits before adding
more retained products. The duplicated observation construction on a miss was already removed
by Plan41 and is not a new package. Any proposed change to scientific identity returns for a
concrete rule decision; internal representation improvements do not require one automatically.

### A5 — Settle repeated subtree typing before changing evaluation

I06 inspects `specialize/value.rs::Evaluator::typed_result`, especially additive expressions
whose children are evaluated/typed before inference over the enclosing subtree. Compare depth
and shape with maximal-product handling and count repeated inference without making a wall-time
or asymptotic dominance claim from sampled frames alone.

If avoidable repeated inference is established, design a specialization-local typed product or
composed evaluation that consumes complete current environment/presence/path/binder premises.
Preserve integer overflow and integrality, physical quantity typing, optional/refined paths,
lowered functions, fold/reduction binders and error behavior. Stable lexical bases may be reused;
mutable overlays cannot be frozen. No owner-only cache or new interpreter is presumed.

A supported no-adoption outcome is valid if the consumed semantics make the suspected work
necessary or no worthwhile narrow correction is established. Record the reason and actual
evidence, not a promise of later investigation. This inquiry cannot delay A1–A4 unnecessarily.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| <a id="a1"></a>A1 | Existing retained declarations and diagnostic case policy | Migrate all affected benchmark/diagnostic callers; retain legitimate analysis-only use; targeted missing-free/non-solve, demand/context and edited-result controls; delete redundant paths. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="a2"></a>A2 | Checked-package premise and actual caller assessment | Remove empty-index unused environment construction while retaining required errors; pure scalar/contextual/refined/indexed/lowered-function controls show both semantics and eliminated construction. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="a3"></a>A3 | Existing operation-local index semantics | Borrowed lookup/reduced copying across all current source consumers; settle I04 with equivalence/frontier controls before optional memo adoption; remove replaced key/copy paths. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="a4"></a>A4 | Complete current view/occurrence contracts | Compact pre-expansion lookup, miss-only structure and exact attribution reuse with fresh state/permission; settle I05 and exercise value/structural/span changes. | Completed — targeted and selected assembled acceptance passed; shared scope-end checks accepted at coordinator Outcome |
| <a id="a5"></a>A5 | Current evaluator semantics and shape controls | Settle I06 and implement a qualified narrow correction if supported; independent typing/overflow/binder controls, or supported no-adoption outcome. | Completed — supported no-adoption outcome; inquiry settled |

A1/A2/A3 can begin under existing contracts independently of G0. A4 consumes immutable preparation
contracts, not completion of every A package. Shared checker/compiler/runtime files require
coordinated editing; logical independence is not permission to overwrite another agent's work.

## Verification and documentation

**Proposed:** use touched-package compilation and focused checker/compiler/runtime/Python tests
that exercise the changed seam. Pure checker/lookup tests require no store/native startup.
Compare expected semantics independently of the changed memo/key path; merely hashing a product
through the same transformation twice cannot establish complete identity. Include invalidation,
missing inputs, wrong contexts and span-only edits, not just successful warm values.

Observe reuse/construction directly. A full cold 32-block simulation is not required for every
counter; Q0 selects it once only if it discriminates a material remaining question. Numerical
assertions remain unchanged, Arrow correctness validation explicit, and no exact timing target
is introduced. Installed Python testing checks actual diagnostic consumers, not environment proof.

Update enduring preparation/edit/reuse explanations at their existing architecture owners through
the decision route when contracts change. Any generator/API declaration change includes consumers
and regeneration. Remove replaced implementations/tests once targeted replacement controls pass;
do not preserve compatibility paths or old whole-package caches as evidence.

## Current checkpoint

A1–A4 source changes and their targeted controls are integrated. The benchmark retains declared
execution; all four native Python diagnostic variants use retained diagnostic resolution, with
missing free values allowed independently of execution procedure. Empty scalar index checks skip
unused environments after the enclosing checked-declaration premise. The operation-local source
index uses borrowed nested keys and an empty-probing-only completed root memo; insertions and
completeness transitions invalidate it, and Pending is never retained.

I05 selected a complete internal bound lookup before row/instance expansion. State/domain
admission and tightenings remain fresh on hits; values rebind, and exact document/role/position/
span identity governs separately leased attribution. Public scientific view-key meaning is
unchanged. The old production solver-cache entry route is removed; cache-pressure controls now
inject an explicitly test-only product.

I06 concludes with no evaluator change. `typed_result` infers the enclosing expression using
current environment and expected type; evaluated child `Value`s do not retain the refinement,
physical-operation, presence and binder premises required to replace that inference. A complete
typed product would be a broader evaluator design without established benefit here. New controls
count typed-result calls and submitted subtree-node extents for left-deep addition, balanced addition and maximal products;
these controls passed with the pure-owner suite. This conclusion claims neither dominant quadratic
runtime nor a performance improvement. The full modeling/compiler/diagnostics owner suite passed
548 tests with zero failures. The related runtime owner selection passed 36 tests with zero
failures; the retained-diagnostics control now explicitly admits its edited value in the original
demand. Independent source review is complete. Installed Python modeling diagnostics pass (two selected integration controls), and the selected
scalar-preparation assembled case passes with its original assertions. Scope-end checks and coordinator reconciliation are complete; their exact scope is at the coordinator Outcome.

## Outcome (recorded after implementation)

### What was built

**Implemented:** A1–A4 retain admitted declared preparation in all affected benchmark and Python
diagnostic consumers; remove unused scalar-index environment construction; use borrowed source
keys/shared lexical bases with completed-root, empty-probing memoization and frontier invalidation;
and reach a complete bound lookup before immutable expansion. Fresh state/domain checking and
separately leased current occurrence attribution remain independent of structural reuse. The old
production solver-cache entry route is removed; pressure injection is explicitly test-only.

**Interface-checked / Tested — 2026-10-16, zero-failure baseline:** independent source review and
the pure-owner nextest selection (`-p pse-modeling -p pse-compiler -p pse-diagnostics --lib
--locked --no-fail-fast`) passed 548 tests. Related runtime owners passed 36 tests. Installed
modeling diagnostics passed two selected integration controls; `just case-measure ... --smoke
--audit-diagnostics --case k4-scalar-preparation` passed its original scientific/reuse and cleanup
assertions. These are construction/semantic and selected assembled observations, not a speedup
estimate. The coordinator Outcome records the completed shared scope-end result.

I04 adopts only the qualified root memo; I05 adopts complete compact lookup and exact independent
attribution. I06 is settled with **no evaluator rewrite**. Shape-sensitive controls count submitted
inference extent (left-deep addition 7/63, balanced addition 7/41, maximal product 1/15). Evaluated
values do not preserve the refinement, physical-operation, presence and binder premises needed
for the proposed replacement. This neither proves dominant quadratic CPU cost nor promises later
implementation. Enduring contracts are at blueprint §14.4 and the workflow owners.

### A mistake made and corrected

A retained-diagnostic test initially edited a value outside its original demand. It was corrected
to admit the edited value explicitly and then passed in the runtime-owner selection. The final
Clippy readback also required iterating the occurrence map's values directly; the retained-byte
calculation and its terms are unchanged.

### Deviations from the plan, deliberate

No generic preparation cache or broad evaluator product was added. The supported I06 no-adoption
outcome settles that inquiry. Selected direct work counts and original numerical assertions replace
speculative latency attribution; no repeated timing campaign or environment proof was introduced.
