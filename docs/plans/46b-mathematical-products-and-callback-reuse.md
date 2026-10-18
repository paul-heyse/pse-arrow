---
title: "Plan46b: Mathematical products and callback reuse"
status: done
date: 2026-10-18
adrs: [ADR-0193]
review_sources: [docs/design_review/reviews/design_review_execution-speed-review-audit_2026-10-18.md]
doc_role: plan
doc_topics: [design, preparation, numerical-execution]
---

# Plan46b: Mathematical products and callback reuse

## Target and foundation

Own AF02/AF03/AF09/AF10, I01/I02 and the mathematical side of RC01/RC03. The [coordinator](46-execution-speed-and-numerical-efficiency.md) owns decisions/dispositions. RC01–RC03 were accepted and this companion was authorized for execution on 2026-10-18; package completion still requires its implementation and targeted controls.

**Interface-checked:** current `execution.rs` already narrows arithmetic regions to active axes, but the whole-worker frame and returned first rows remain dense. `products.rs` uses pairwise symbolic construction for flat weighted products; guarded/provider programs have a different mixed-coordinate Taylor path. `assembly.rs` has useful local completion/projection and failure-safe spare buffers. `library/admission.rs` repeats owned scalar-key work. These owners remain appropriate; no independent derivative cache or AD framework is needed.

The target makes consumed products and validity determine physical work. It preserves original ordered guards, structural support, contribution accumulation, finiteness and foreign-output atomicity. A matrix already required/completed can serve an eligible action; an action-only consumer should not build that matrix merely for uniformity. Provider effects remain a separate current obligation under [46a](46a-provider-effects-and-implicit-products.md).

<a id="b01"></a>

## B01 — Record successful per-order instantiation

**AF09; RC03.** After a stage tree is successfully instantiated for its representation/order, subsequent warm preparation checks scope and takes a constant-time successful state path rather than recursively traversing nested stages and both branch arms. Remove the second cold traversal. Reset/replaced programs clear that state; partial instantiation cannot publish success or prevent required retry.

Do not merge successful native compilation with current numerical validity, and do not perform symbolic optimization inside numerical callbacks. The existing native compiled owner determines executable/scratch readiness. **Targeted acceptance:** cold/warm nested/branch programs, unchanged order and upgrades, reset, cancelled/incomplete instantiation then recovery; count actual stage visits/constructions rather than merely observing a cache hit.

<a id="b02"></a>

## B02 — Feed compact first results into canonical sparse assembly

**AF02; RC01 where product contracts change.** Represent returned first partials by conservative support and precomputed canonical scatter coordinates instead of outputs × all requested coordinates. Migrate Jacobian/gradient/selected-partial consumers together, including repeated contributions and physical scales. Retain full/dense products for actual full consumers; an externally required dense boundary may materialize there rather than forcing every internal result dense.

This package removes returned structural-zero entries even before B03 changes internal frames. It must not claim that compact results remove slot-by-width scratch or all arithmetic work. **Targeted acceptance:** equivalent large sparse flat and many-instance bodies, branch unions, provider output support, numerical zero crossings, repeated contribution ordering and analytic first derivatives; count returned entries/scatter work separately from frame/region arithmetic. Delete the replaced internal dense-only representation and indexing consumers once migrated.

<a id="b03"></a>

## B03 — Qualify compact frame/layout and initialization

**AF02/AF10; I01.** Compare support-shaped stage/Taylor components and eligible library-generated selected partials at current execution owners. Only consider compressed seeds/coloring after exact support, unambiguous recovery, branch/provider capability and total construction/storage fit are established. Dense union support can make compression pointless; no bespoke coloring service is assumed.

Compiler-owned writes-before-reads, branch liveness and complete output initialization may remove redundant poison/zero/gather/scatter work. Preserve structural zeros, original checks and failure/reset invalidation. Symbolica's pinned public evaluator copies parameters into its own stack and results out; do not assume borrowed caller-frame access. Prefer eliminating project-owned duplicate movement first.

**Decision/targeted acceptance:** separately count frame, region-input/output, returned-result and arithmetic work on sparse/dense/branch/provider controls. Adopt the simplest conforming representation that removes material unnecessary work without adding a competing authority; retain an explicit dense route for its genuine consumer. Test late failure/cancellation and reused slots. Inconclusive coloring does not block B02 or established liveness savings.

<a id="b04"></a>

## B04 — Qualify contracted flat Hessian actions

**AF03; I02.** For flat arithmetic compare (a) current sparse supported pairs with retained first derivatives, (b) directional derivative of the weighted gradient and (c) gradient of the weighted directional scalar. Use exact Symbolica source/skill contracts and existing product preparation. Include construction calls, expression/instruction/code size, scratch and expected repeated execution, not only an isolated HVP.

Preserve the current original value-stage/derivative-eligibility pass even when weights are zero or symbolic cancellation hides an invalid intermediate. Guard/provider/branch exact-matrix products retain their mixed-coordinate route. Contracted actions consume A03/A04 through original forward stages and bounded library-generated local pullbacks, without full local provider jets. Sparse Hessian entries for Ipopt are not replaced by HVP; limited memory remains an explicit different algorithm.

**Decision/targeted acceptance:** dense reductions, sparse chains, shared expressions, zero weights and invalid original domains. Adopt qualified in-bound construction choices at the existing owner, with exact product identity and finite limits; remove unadopted candidates. No dedicated reverse-mode API or speedup is assumed from current documentation.

<a id="b05"></a>

## B05 — Reuse completed assembled products and consolidate staging

**AF09; RC03; requires A01 and relevant A02/A03 behavior.** Owner-issued point/fixed-value revisions plus assembly/profile, demand/order and weight/direction identity authorize completed assembled reuse. New-state finiteness is checked once where its immutable premise is issued, not repeatedly at every same-trust layer. Retain checks of newly produced provider/arithmetic data and uncontrolled input boundaries. Never use pointer identity or an unqualified native new-point hint.

Preserve local-group reuse for partly changed global points. Current required effects must run before a completed numerical product is newly authorized. Independently complete weaker data may survive a typed cancellation/resource/order refusal; unknown provider/library failure invalidates conservatively. Integrate eligible matrix→action consumption at existing product/projection owners without preparing both unconditionally.

Consolidate assembly/oracle/callback copies into one sufficient completed staging owner. Foreign output stays untouched on late failure or unwind; remove redundant owned copies only after all callers use that completion boundary. **Targeted acceptance:** real repeated constraint/Jacobian/gradient/action callbacks, unchanged free x with changed fixed values/effects, weights/directions/order changes, pure hits, late failure/reset/recovery, accumulation order and independent final original assessment.

<a id="b06"></a>

## B06 — Remove warm scalar-key construction and duplicate lookup

**AF10.** Use borrowed lookup or exact prepared scalar handles at the existing scalar cache owner so warm lookup does not clone expression/parameter vectors or probe the map twice. Retain source/options/order/limits and one cache authority; a handle does not retain an unrelated attempt or evade admission.

This is preparation metadata work, not a numerical callback speed claim. **Targeted acceptance:** equal prepared lookup, changed expressions/parameters/options/limits, miss insertion, eviction/escaped lease and failed construction. Count key allocation and actual construction separately. Do not add another scalar cache to hide the first one's overhead.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| B01 | RC03 decision; stage/native reset ownership | Successful instantiation state and no duplicate traversal; failure/reset controls | done |
| B02 | Actual sparse consumer inventory; RC01 for changed contract; A03 semantics | Compact first result/scatter in all affected consumers; separate frame accounting | done |
| B03 | B02 representation; exact library/support contracts; A03 for provider actions | I01 decision and supported frame/liveness realization; branch/zero/failure tests | done |
| B04 | Exact product contracts; A03/A04 only for provider consumers | I02 comparison and qualified flat action construction; original-obligation controls | done |
| B05 | A01; working invocation-validity/demand boundary; RC03 | Effect-safe completed assembly/matrix-action reuse and one atomic staging path | done |
| B06 | Existing scalar cache identity/lease owner | Borrowed key/handle lookup; miss/eviction/limit controls | done |

## Verification and handoff

**Implemented:** B01–B06 and bounded I01/I02 choices are integrated at the existing mathematical,
assembly, scalar-cache and native callback owners. Actual demand/factor/effect contracts follow
[46a](46a-provider-effects-and-implicit-products.md). **Tested / Interface-checked:** the shared
[46a verification](46a-provider-effects-and-implicit-products.md#verification-and-handoff)
records exact commands, zero-baseline final results and their exclusions, including final
compiled math/kernel/native callback selection and fitting/native consumers. G03 retains its
independent hardware SIMD/JIT inquiry; G06 owns whole-series qualification.

## Outcome

### What was built

**Implemented:** Successful per-order executable instantiation avoids repeated stage walks;
compact conservative first results and support-shaped frames feed canonical sparse assembly.
Qualified pure original-input First regions use library-generated symbolic partials; other
regions retain Taylor. Flat complete-pair HVP construction uses a directional weighted gradient;
sparse/mixed products retain supported pairs with retained first derivatives. Nonflat products
use original forward stages and bounded library-generated local pullbacks with actual provider
actions. Completed assembled products and eligible pure matrices serve exact matching demand
without redundant preparation/copies, while provider effects remain current. Strict-lower CSC
matrix actions share original coefficient storage without diagonal subtraction. Borrowed scalar
lookup removes warm key-vector cloning and duplicate lookup, retaining one cache authority.

**Measured, I01 small sparse control:** frame 40 versus dense 408 cells; region inputs/outputs
8/16 versus 72/72; returned 16 versus 136; 16 versus 24 arithmetic operations. Two scalar products
replace one broad product. Dense/dependent/control/provider routes retain their qualified
realization. I02 short dense/sparse/shared controls compare preparation, operations and scratch
separately; neither choice establishes a universal latency winner or statistical SLA.

**Implemented supporting scope:** Native contracts validate before provider preparation;
fitting lanes construct their assigned owners on the actual lane thread before native attachment.
Startup failure drains started peers; exact affine fitting still prepares the coordinator's
actual Hessian contract even when local curvature is empty. This removes callback-time symbolic
preparation rather than hiding it behind a new scheduler or cache.

### A mistake made and corrected

A temporary nonflat weighted-scalar bridge still demanded broad derivatives and was deleted
when direct forward/reverse composition replaced it. Review caught an affine parallel fitting
owner omitted because local curvature was empty: original Second obligations now determine
preparation. Compiled-policy selection alone did not prove native pullback execution; the
new compiled control checks actual compiled arithmetic blocks. Test fixtures now implement
the direct actions their consumers request instead of relying on a removed full-jet path.
The joined selection also exposed lower-triangle validation incorrectly imposed on generic
upper-triangle coefficient matrices. Generic square construction and the private lower-action
layout now have distinct contracts; coefficient convexity and cancellation-sensitive action
controls pass. Stale native MUMPS fixtures were updated to its existing explicit fields/current
identity, with the historical typed vector retained unchanged. No production settings or
identity algorithm changed. Current provider effects require the third identical-key visit;
reverse branch-action census correctly reports no complete Hessian pairs.

### Deviations from the plan, deliberate

No coloring service, dedicated host reverse-AD framework, automatic JIT choice or new cache
was added. Exact sparse Hessian consumers keep their matrix contract; actions do not replace
Ipopt's exact entries. Fitting startup preparation is necessary enabling scope, with no general
terminal-retirement platform. AF10 remains partly open for G03 despite completed B03/B06.
The companion remains retained for other Plan46 consumers and G06 qualification.
