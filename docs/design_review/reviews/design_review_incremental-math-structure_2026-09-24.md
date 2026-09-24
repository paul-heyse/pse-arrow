---
title: Incremental mathematical structure and native ownership
status: complete
date: 2026-09-24
scope: Plan 14 M09–M10
---

# Incremental mathematical structure and native ownership

## 1. Decision and scope

**Proposal:** library-owned selected-case structural analysis, Salsa semantic compilation,
and shared native artifact/worker ownership. **Evidence:** Implemented; targeted controls
are recorded in the [execution packet](../../plans/14-m09-m10-execution.md).

**Method and coverage:** inspected the new compiler, physical identity framing, immutable
case plan, pounce adapter, typed provider descriptor, single flights, runtime cache and
worker supervisor. Attacked incomplete incidence, alternate maximum matchings, stale
input reuse, cancellation, eviction and native teardown. This is a scoped implementation
review, not solver, Python, storage, physical-reference or end-to-end qualification.
Those remain M11–M22. Existing accepted blueprint/ADRs are not silently amended.

## 2. Authority and lifecycle map

| Fact | Identity/owner | Revision/update boundary | Derived representation |
|---|---|---|---|
| Process definitions and bindings | Semantic IDs; admitted source inventory | Validated workspace batch before Salsa setters | Typed bodies and CasePlan |
| Physical declarations | Actual QuantityRegistry/PhysicalPreconditions | Canonical inventory framing and re-admission | Typed operations, refreshed unit gathers |
| Provider meaning | AdmittedProvider; full implementation/data/component/phase/derivative contract | Immutable descriptor input; factory remains runtime owned | Symbolica provider stages |
| Numerical values | CaseValues; fixed/parameter assumptions distinct from free trials | Separate values input and consumed-assumptions query | Coefficient snapshot or runtime evaluation |
| Structural facts | Complete selected case, semantic rows/columns/contributions | All-branch support projection | pounce matching/DM/BTF; versioned provenance |
| Compiled programs | Compiler-issued ArtifactRequest | Semantic demand + exact build/source + evaluator profile/ABI | DataFusion-retained immutable program |
| Mutable native state | Attempt/worker thread | Creation through join, including TLS destruction | Finite completed values; no Salsa memo |

## 3. Semantic contracts and invariants

| Contract | Enforcement | Failure behavior | Evidence |
|---|---|---|---|
| Complete equality/free-variable incidence includes isolates | `incidence.rs` validates inventory, finite exact bounds and references before ProbeView | Partial/dangling/invalid input rejected | Complete/partial, rectangular, isolate and invalid-index unit controls |
| Matching is structural, not numerical rank | `jac_values=None`; matching/DM/component/BTF validation | Invalid library output rejected | Exhaustive tiny independent oracle and singular numerical example |
| Coupling is not independent eliminability | `classify_block` uses structural objective support; independent extraction checks all crossings | Crossing equality, inequality or objective rejected | Coupling/region test |
| Semantic reuse observes absence and meaning | Tracked lookup Option values, actual descriptors and physical declarations | Missing differs from empty; deterministic typed errors | Workspace edit and negative-lookup units |
| Stale asynchronous work cannot repopulate invalidated cache | Epoch checked under publication mutex | Existing readers survive; old population is not retained | Actual artifact epoch test |
| Cancellation does not release live native resources | Completion-owned flight and join supervisor | Bounded refusal or cooperative cancellation; retry after exit | Last-waiter and TLS-destructor controls |

## 4. Derivation and execution design

| Stage | Input/equality and backend | Scope, cost and termination | Ownership/effects and invalidation |
|---|---|---|---|
| Typed preparation | Selected source/formals/membership/providers; actual registry and prerequisites; Symbolica math equality | Finite source/domain/body bounds; cycles are not numerical fixpoints | Salsa memo; spans separated from reusable arithmetic; no evaluator construction |
| Case planning | Semantic bodies and admitted structure; faer sparse support | Complete selected rows/free columns; aliases and duplicate contributions retained | Immutable CasePlan; fixedness/binding changes invalidate; trial values do not |
| Structural analysis | All-branch incidence; pounce-presolve 0.12.0 HK/DM/components/BTF | 100,000-row ceiling, qualified 32 MiB stack; no sampled sparsity; cancellation between indivisible stages | Semantic result; canonical member ordering and library topological tie breaks; one matching witness is not required to be unique |
| Coefficients | Value-independent plan plus consumed fixed/parameter bits | Bounded Symbolica derivative/expansion/polynomial calls; unsupported classes fail | Immutable assumption-sensitive result; no executed solve cache |
| Artifact requests | Ordered outputs/coordinates/order, optimizer and evaluator limits, source/build/ABI identity | Demand deduplication via standard BTreeMap | Immutable request; runtime cannot reconstruct an alternative key |
| Native compilation/execution | Symbolica/Numerica/faer and provider factories | Shared CPU semaphore, finite jobs/flights, stack and pool admission; no nested CPU acquisition | Worker-local mutable state; join witnesses native exit; completion-only cache publication |

Salsa has no persisted store, leaked database handle, numerical recycle cycle strategy or
universal expression interning. The single writer publishes whole admitted batches;
LRU maintenance and generation reconstruction occur between owned prepare calls.
Existing package/rule graph consumers remain. Old containment/tear products are not
restored without M13/M14 consumers.

## 5. Representative journeys

A membership edit replaces the admitted domain/group input, invalidates its lookup and
consuming typed body, and regenerates the selected plan/request. An unrelated provider
lookup backdates. Whitespace edits refresh occurrence spans while arithmetic products
remain reusable. Fixed/parameter changes refresh coefficient assumptions while free
trial changes retain structural and numeric programs.

A caller abandoning compilation drops its waiter and requests cancellation. The native
flight stays discoverable until join. Another waiter remains unaffected. Invalidation
increments the cache epoch; a completing old build cannot repopulate it. Evicted programs
retain their allocation owner through all surviving evaluator/worker clones.
Prepared body handles likewise retain the preparation allowance after their case is
dropped, without including allocation ownership in semantic equality.

## 6. Acceptance gates

| Gate | Verdict | Evidence/scope rationale | Action |
|---|---|---|---|
| G1 Authority | Pass for scope | Actual declarations and one production workspace route; BodyStore removed | Preserve source authority in M16 adapter |
| G2 Semantic fidelity | Pass for scope | Physical re-admission, all-branch support, aliases, provenance, coefficient assumptions | Independent physical/solver comparisons at M22 |
| G3 Validity | Pass for scope | Typed boundaries reject incomplete/invalid input and capacity overflow | Extend native ABI controls at M11 |
| G4 Hidden behavior | Pass for scope | Native work outside semantic memos; cancellation and foreign allowances explicit | No claim of allocator/RSS enforcement |
| G5 Consistency and recovery | Pass for scope | Atomic application batch, epoch publication fence, completion-owned cancellation | Full workflow interruption checks at M22 |
| G6 Transformation and reuse | Pass for exercised scope | Clean/reused comparisons, negative lookups, event-backed reuse | Add cases alongside future solver/dynamic inputs |
| G7 Truthful capability claims | Pass for scoped claims | No native solve, empirical physics or end-to-end performance claim | M22 remains open |

## 7. Principle findings

| Finding | Principles | Concrete evidence | Consequence | Correction/decision | Verification |
|---|---|---|---|---|---|
| Successful job completion originally triggered cancellation | DM-30, DM-35; RCA §7 | `math/jobs.rs`: Caller is now disarmed after completion receipt | Successful artifact could be refused as cancelled | Corrected before closure | Actual artifact and multi-workspace tests |
| Cancellation could be lost when replacing a Salsa generation | DM-30, DM-35; RCA §8 | `workspace.rs` `rebuild` transfers pending cancellation to replacement storage | A cancelled request could run after rotation | Corrected before closure | Generation/cancellation/retry unit |
| Physical unit gathers must be re-admitted on registry edits | DM-07, DM-24, DM-31; RCA §2–3 | `binding.rs:212` `readmit`; compiler plan invokes it for every typed formal | Reusing old numeric conversions would preserve stale physical interpretation | Corrected; actual declarations remain authority | Clean/reused physical edit control and existing affine-gather tests |
| Whole-registry fingerprint conservatively invalidates bodies | DM-33; RCA §3, §8 | `physical_identity.rs` frames the actual admitted inventory | An unrelated registry change can rebuild a body | Deliberate safe granularity; narrow only with measured benefit | No smallest-possible-invalidation claim |
| Foreign memory is an admission estimate | DM-39, DM-48; RCA §8 | MathPolicy separates known numeric capacity, workspace/container allowance and foreign/stack reservation; reports leave unobservable extents unknown | Pool reservations cannot prove a process RSS ceiling | Keep claim scoped; measure actual RSS/TLS at M22 | Lease/cancellation controls pass; RSS qualification remains open |

**Applicability:** DM-01–15, DM-21–35, DM-38–48 and DM-56–60 apply to authority,
reuse, transformations and ownership. RCA §1–9 apply to mechanism selection and the
compiler/graph/runtime boundary. Temporal graph analysis, distributed persistence and
Python interchange changes are outside this packet. Satisfied findings are bounded by
the exercised contracts; no performance score is inferred from library choice.

## 8. Alternatives and architectural leverage

| Alternative | Duplication/locality | Risks and cost | Performance evidence | Decision |
|---|---|---|---|---|
| Preparation-local BodyStore | Manual physical identity and repeated admission; local numeric cache | Does not establish cross-edit semantic dependencies or shared ownership | M08 only | Replaced |
| Salsa semantic compiler plus existing native cache/runtime primitives | Library dependency tracking and standard retention; project code maps semantic contracts | Requires explicit equality, finite generations and cancellation ownership | Targeted reuse/ownership controls; no end-to-end speed claim | Selected |
| Always reprepare plus one bounded artifact map | Simpler dependency lifecycle | Repeats physical typing/structural analysis after unrelated edits; still needs exact artifact keys and native lifetime rules | No measurement establishing superior total cost | Viable small-model alternative, not the selected editing target |

No project matching, generic graph engine, scalar AD, optimizer, solver or custom cache
replacement is added. Ordinary code remains responsible for physical meaning, checked
library adaptation, canonical semantic IDs, workload admission and publication.

## 9. Verification and measurement

See the execution packet for exact commands and conditions. The combined force-validation
selection passed 63 units with zero failures against baseline zero, including provider
revision and finite/ragged membership controls. Exhaustive structural controls cover 74,963 graphs and
259,537 maximum matching witnesses. Native ownership controls execute actual Symbolica
compilation, two-core admission, failed builds, retained clones and slow TLS destruction.

`just check` compiles all workspace targets; the final targeted selection closes this package. M22
retains full integration, solver, Python, policy/docs/governance and performance campaigns.
The transitive proc-macro-error2 future-compatibility notice remains recorded there.

## 10. Exceptions and unresolved decisions

Foreign allocator interception, empirical property validity, solver convergence, public
orchestration and end-to-end performance are not claimed. No immutable accepted ADR or
blueprint is edited here. ADR-0082/0083 remain proposed pending formal review/decision PR
and blueprint reconciliation. Additional graph/connectivity/tear consumers belong to
M13/M14; they do not justify restoring old Plan 13 artifacts now.

## 11. Decision and implementation changes

| Priority | Decision/change | Principles | Acceptance evidence | Remaining obligation |
|---|---|---|---|---|
| 1 | Accept scoped implementation | DM-07, DM-24, DM-30; RCA §2–8 | Execution packet | M09–M10 closed; full qualification remains M22 |
| 2 | Keep a single production compiler and completion-owned native boundary | DM-02, DM-26, DM-31–35, DM-56 | Deletion search; compiler/runtime controls | M11 consumes this boundary |
| 3 | Retain explicit admission estimates and conservative physical invalidation | DM-33, DM-39; RCA §8 | Scope described above | M22 measurement; narrow only on evidence |
