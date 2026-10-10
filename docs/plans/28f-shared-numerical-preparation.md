---
title: Shared numerical projections and preparation
status: in-progress
date: 2026-10-07
adrs: [ADR-0164, ADR-0167]
review_sources: [docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md, docs/design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md, docs/design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md, docs/design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md, docs/design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md, docs/design_review/reviews/design_review_flow-projection-v2_2026-10-09.md]
scenario_sources: [docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md#4-revealing-scenarios]
---

# 28f: Shared numerical projections and preparation

## Responsibility and baseline

This companion develops the numerical portion of the repository-wide efficiency extension.
It supplies shared projection mechanics, immutable checked preparation and compatible native
owner reuse to [28b](28b-selected-compilation-and-reuse.md) and the ordinary, study, dynamic,
fitting and analysis consumers. The [coordinator](28-surrealdb-unified-substrate.md#repository-wide-efficiency-extension)
owns coverage and PE finding dispositions; [28e](28e-rebuild-retirement-and-qualification.md)
owns assembled qualification. Blueprint §7, §13, §14, §16 and §19 retain semantic authority.

The [preparation integration](28-surrealdb-unified-substrate.md#preparation-assurance-and-reuse-review)
adds N11 after confirmed rule decisions. [28j](28j-pure-preparation-and-publication.md) owns
complete basis/effect contracts; B7 adopts them at selected preparation. This companion
migrates actual analysis/mode/experiment consumers. The coordinator owns PA01–PA04 dispositions.

The source baseline is `5260a3e9a3cecd69b6358ab86d4e917ae2508b29` plus the preserved review
and plan documentation. The review establishes PE01 and PE04, not their latency share.
The focused authoring assessment additionally inspected mathematical assembly/reconstruction,
structural preflight, trajectory publication, dynamics, shooting, fitting, uncertainty,
compiler queries, relation admission and physical reuse. Searches identify supported
mechanisms and candidates; they do not establish absence throughout every caller.

Existing foundations are suitable: typed row/column spaces in `pse-math::index`, assembly
maps, fitting contribution layouts, compiler-issued construction witnesses, bounded math
retention, actual native adapter sessions, and physical admission with fresh protection.
Extend them. Semantic mappings must not become a global identity service, independently
editable coordinate registry, or second incremental engine. Thermodynamic production
knowledge uses authored checked mathematics; FeOS is reference-only (blueprint §9.8).
State/composition-dependent property values are not immutable preparation.

## Shared projection contract

Resolve semantic identities once at the owning layout boundary, then consume ordered
projections. The mechanics can use ordinary library maps and typed vectors in existing
modules; the conceptual products below do not require a new public trait or crate.

A projection consumes an admitted source inventory, the ordered requested identities and
the owner's correspondence rule. It establishes uniqueness or the explicitly permitted
multiplicity, resolves every required identity, and retains the source layout identity and
allocation ownership with its ordinal mapping. Execution applies that mapping to current
values without repeating source-vector searches. A changed layout rebuilds the affected map;
a changed value does not. Transient construction refusal is not memoized as a successful map.

One set of construction mechanics serves the consumers, while their semantic keys remain
distinct. Numerical targets use `(kind, semantic ID)`; structural roles use row IDs;
derivative axes and contribution slots keep their typed index spaces; composite correspondence
must preserve the complete admitted row/coordinate contract, not merely a matching ID.
Duplicate contributions retain attribution. Set equality does not imply positional equality.
Missing fixed coordinates in composite reconstruction retain its explicit fixed/retained
rule; other missing coordinates refuse. Do not give every consumer an undocumented fallback.

For PE01, derive normalization, original-space quality allowances and bound accuracy-goal
targets from one resolved target access structure. Values, frozen contextual scales, integer
normalization, error allocations, precedence and provenance remain owned by numerical policy.
Do not introduce another tolerance table or recompute contextual defaults during assessment.

| Consumer migration | Existing work and required replacement |
|---|---|
| Numerical policy | `normalization::Normalization::from_policy`, native `quality::Tolerances::from_policy`, and `engineering_accuracy::bind_goals` share resolved kind/ID access. Preserve source precedence and conflicting-goal refusal. |
| Structural preflight | Native `structural::validate_assessment` resolves original row roles once rather than searching assessment equations for each contract row. Preserve reordered inventories and equality-role checks on the current bounds. |
| Trajectory publication | `workflow/modeling/trajectory.rs` prepares ordered symbol, physical-row, quantity and unit metadata outside the sample loop; each sample supplies only current output/sensitivity values. Parameter projections follow the same layout lifetime. |
| Composite reconstruction | `composite_reconstruction` derives checked local-to-original row/column maps once per admitted contract pair. Fixed auxiliary coordinates and original-scale correspondence remain explicit. |
| Conditional assembly | Compare the existing row/column/request maps with prior-instance `(instance, body)` support lookup. Reuse their mechanics where repeated reconstruction warrants it; do not erase occurrence attribution. |
| Dynamic accuracy, shooting and recycle | Prepare stable target/control/node projections at their actual layout owners. Time-varying schedule selection, event intervals, changed bounds and per-window control values remain execution inputs. |
| Fitting and uncertainty | Retain fitting's existing sparse contribution and Gram plans. Use the shared correspondence mechanics for covariance-to-Jacobian parameter mapping where applicable; preserve output/parameter order and independent covariance admission. |

The first three rows have source-established repeated lookup under coordinate/sample growth.
The remaining rows need their exact multiplicity, equality and layout lifetime settled in N0;
small bounded dispatch or registry inventories are not automatically defects. Once a comparable
variant is confirmed, its migration joins N1 rather than becoming an indefinite follow-up.

## Immutable admission and changing execution

N2 addresses PE04 at the existing modeling/math owners. An immutable checked preparation
contains selected source meaning, complete scientific dependencies, interpretation and
owning-kernel admission. Compatible consumers acquire it under current eligibility and
fresh source protection, then bind their values, starts and attribution separately.

The current `ModelingPackage::prepare` path reopens selected source and constructs a fresh
canonical compiler workspace before generic admission. Reuse a checked selected input or
compatible worker-local workspace through the existing bounded service owner; choose the
smallest retained product that removes this work. Do not retain both a complete source bundle
and equivalent checked selections without a distinct consumed purpose. Persisted description
reconstruction remains B2's mechanism, not another scientific admission implementation.

The reuse key covers the actual selected positive and absent references, membership/visibility,
physical registry and preconditions, providers, compiler/kernel interpretation and consumed
construction/profile inputs. Reuse across revisions requires complete dependency eligibility;
the revision attribution is rebound without relabeling old meaning. A global head/build key
is too broad, and a positive-reference list alone is incomplete. Structural literals invalidate
their consumers; ordinary value bindings do not become structural literals accidentally.

Fresh protection is mandatory on hits as well as misses. Retained immutable witnesses never
retain an expired `SelectedRead`, attempt fence or publication authority. Coalesce compatible
in-flight preparation through existing retention/flight ownership where useful, with one byte
charge for shared allocations and current request attribution. Cancellation, allocation refusal
and transient preparation failures remain retryable; release superseded active-owner state
without discarding deliberately retained durable history.

Equal study values remain distinct occurrences. Every attempt owns mutable evaluators, solver
applications, factor values, cancellation/deadline and original-space assessment. Reusing
mathematics or a native application does not reuse scientific acceptance, a warm start, or
an earlier claim. Ordinary Rust/Python solve, initialization, continuation, fitting,
shooting/dynamics and analysis routes consume this split rather than copy preparation policy.

## Library and lifecycle decisions

Use the review's [math evidence](../design_review/evidence/production-execution-efficiency-2026-10-07/math/README.md)
and exact release source. Context7 supplies current documentation when an API choice needs
new research; release contracts govern implementation. Existing multi-output optimization,
compact formals, Symbolica stack optimization, Salsa backdating and native sessions are
strengths, not missing capabilities to reimplement.

| Investigation in N0 | Decision evidence and consequence |
|---|---|
| Structural Taylor zeros | Inspect per-input structural support for value/First/Second/directional stages, branches and composed providers. Symbolica 3.0.1 consumes `(parameter index, component index)` zero slots. Adopt proven-zero suppression in the existing dualizer when this removes applicable work; preserve complete external Taylor shape, factorial conventions and ordered axes. Unknown or numerical trial zeros remain present. |
| Numeric factor and scratch reuse | Inspect implicit differentiation, Diffsol `FaerLu`, square-response actions and continuation. faer 0.24.4 supports caller-owned numeric LU and scratch; fitting Gram workers already retain buffers. Adopt compatible retained storage where refresh currently reconstructs it. Always factor the current matrix; preserve pattern/profile identity, pivot/rank/nonfinite recovery and backward-error assessment. Buffer reuse does not promise allocation-free factorization. |
| Catalog invalidation | Retained-design decision, 2026-10-09: keep complete canonical premises and current Salsa backdating. Already admitted revision publication avoids authored checking; changed `publish_modeling_with` still calls the full checker before Salsa, even if downstream math backdates. Narrowing only `Catalog.checked` or changing hashes does not remove that work. [GH3](28k-graph-kernels-and-hashing-investigations.md#gh3) owns the supporting comparison. The bounded authored-edit probe records changed-publication cost for its scalar fixture; the [typed checker candidate](../design_review/evidence/graph-hash-followups-2026-10-09/typed-checker-proposal.md) supplies complete-domain design questions and independent acceptance. Prospective selection, integration and current-operation comparison now belong to [Plan 33/EI04](33-efficiency-principles-remediation.md#ei04), including positive/absent lookups, membership/visibility, deletion and consumed physical/provider/policy/structural context. N0 retains the original conclusion and evidence; no narrower checker or persistence rewrite was selected by that inquiry. |
| Native session consumption | The staged owner already retains native state on its scope/thread. Fitting oracle/profile and shooting paths create fresh `Retained` values. Determine which repeat sequences satisfy adapter compatibility, changing pin/bound/data requirements and destruction ownership. Extend the existing scoped owner only for useful compatible sequences. |

N0 ends with an adopted change or a supported retained-design decision for each row, with
its applicability limit and reopen trigger. It blocks only dependent work. An absent fitting
consumer is a reason not to add an integration now; it is not a reason to reject a library.
Do not enable FBBT without original-bound dual recovery, add another stack optimization pass,
or interpret ignored Ipopt callback hints as proof of repeated evaluation: existing exact-point
and order caching must be considered.

## Remaining dynamic and fitting preparation

The [enhancement review](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md)
adds confirmed F04/F05 and a demand-dependent fitting opportunity. Existing N1–N3 foundations
remain; their focused passes do not exercise these remaining consumers. [28c C5](28c-durable-execution-and-studies.md#stored-study-admission-and-creation-cancellation)
also closes stored-study use of N2's basis. Findings are dispositioned only at the coordinator.

### N5 — Admitted dynamic layout with private workers

`DynamicWorker::new` derives coordinate-scale chains, support pairs/refill correspondence and
canonical sparse structure from admitted programs/coordinates. `IntegratedExperiment` retains
the program but creates workers for integration, gradient, Hessian and shooting-window operations.
These layouts do not depend on trial values. Retain checked immutable metadata at the admitted
dynamic program and instantiate private matrix values, evaluators, providers, guards, point caches
and execution scope. Use existing `AssemblyMatrix`/faer mechanics rather than another coordinate
registry or global cache. A frozen template creating fresh storage is a sufficient local design.

Preserve mode/function identity, constants, coordinate order/scales, derivative order, provider and
structural premises, extent/index limits and complete allocation accounting. Charge shared retained
metadata once and private worker storage separately. Changed layout rebuilds; changed values refill.
Move dynamics, integrated fitting and shooting callers with the constructor and remove displaced
layout derivation. Symbolica interpreter state cloning may remain necessary; this correction does
not adopt compiled evaluators or share mutable instruction stacks/history.

### N6 — Sample-local algebraic elimination factor

IDAS `Session::jump` evaluates one RHS partial matrix and passes it to `eliminate` for the first
adjoint jump and each Hessian direction. Factor the sample's algebraic transpose once through faer
and solve distinct right-hand sides from that owner. Sequential solves are the default; multi-RHS
batching is optional only where its extra live storage earns a consumed benefit. Preserve first-order
multipliers used by curvature, differential/algebraic masks, zero enforcement and ordered axes.
Changed sample/state/mode uses current numeric factors; symbolic/scratch reuse across samples requires
compatible structural premises. Singular/nonfinite failure invalidates readiness before another solve.
Retain original physical derivative/backward-error assessment and private attempt ownership.

### N7 — Fitting demand and coherent point upgrade

Source inspection establishes an existing complete operation: `IntegratedExperiment::gradient`
returns both a forward report and converted gradient contributions. `FitOracle::cotangent` derives
included weighted residuals directly from that report after its forward pass. A preceding prediction
integration is unnecessary on a direct gradient-demand cache miss. Objective/gradient callbacks
already identify demand; no new solver callback interface or checkpoint API is required.

Make point construction consume value-versus-gradient demand. For a gradient-first point, populate
gradient-bearing transient predictions, trajectory and adjoint contributions from one combined
operation. Keep objective-only integration cheap, including experiments with no included observations
or free parameter contribution; keep steady constraints/responses at existing owners. Retain exact-bit
candidate keys and publish a cache entry only when its requested components have passed admission.
Repeated same-point gradients reuse that entry; changed candidates or failures clear incompatible state.

A value-only cache hit followed by gradient demand still needs the combined operation's checkpointed
forward pass: sampled reports are not reusable checkpoint sessions. The combined report owns the
upgraded point's predictions/residuals and derivative. An earlier returned objective remains its
actual earlier observation; do not relabel it as the later report or require bit equality. Settle this
upgrade contract before migrating cache consumers: independently computed predictions compare using
the existing composed production physical allowances, and materially inconsistent/nonfinite output
must follow the owning refusal policy. Preserve duplicate-observation weighting, physical conversions,
events/endpoints, outer deadline/cancellation and admitted checkpoint/foreign memory until drain.

N7 begins with bounded source confirmation of callback/cache consumers and records the resulting
upgrade contract. The direct gradient-first correction is adopted; retaining checkpoint continuation
across callbacks or trials is not. If continuation remains worth pursuing, retain an explicit owner
and trigger at N0 rather than add a speculative native history owner now. This improves the actual
combined-demand route without claiming every objective-then-gradient sequence has one forward pass.

The implemented upgrade compares independent reports using frozen production state allowances
propagated through each report's output partials, capped by any declared physical output decision
budget. This is a local physical consistency allowance, not a certified global nonlinear error
enclosure and not a budget inferred from native integration tolerances. Nonfinite or inconsistent
predictions refuse the upgrade and clear the cached point; success installs one coherent latest
report, predictions and gradient. The earlier returned objective remains its earlier observation.

| Package | Prerequisite and consumer scope | Targeted acceptance and deletion | Status |
|---|---|---|---|
| N5 — Immutable dynamic layout | Admitted dynamic program, existing checked projections/sparse assembly and retained allocation owner; integration/gradient/Hessian/shooting workers. | Shared layout across repeated workers; independent concurrent scratch/values/providers; changed order/scales/constants/support, extent refusal and existing event/physical behavior. Delete per-worker stable layout reconstruction. | Implemented; focused controls pass; E3/E4 acceptance pending. |
| N6 — IDAS sample elimination | Existing current partials and faer numeric/factor owner; first/second-order sample jumps. | One factor per sample with direction/multiplier correspondence; changed samples and singular/nonfinite recovery; production-basis derivative checks. Delete repeated sample-local factor construction. | Implemented; focused controls pass; E3/E4 acceptance pending. |
| N7 — Demand-aware transient point | Existing combined gradient/report API and fitting-owned cotangent; settle value-first upgrade semantics before cache migration. | Gradient-first one forward/backward, objective-only no backward, repeated gradient cache reuse, value-first upgrade coherence, changed candidates and cancellation/memory refusal. Remove discarded-forward-report composition where replaced; preserve the justified objective-first recomputation route. | Adopted and implemented; focused demand/coherence controls pass; E3/E4 pending. |

N5 and N6 can proceed independently of replay admission and each other, coordinating shared sparse
owners if changed. N7 consumes N5 when using its migrated workers, but its demand contract can be
settled earlier. N4 closes C5/N5–N7 and every confirmed comparable consumer before E3; no separate
N-only full campaign is added. E4 distinguishes removed construction from whole-operation gain.

## Parallel admission and native lifetimes

The [parallel extension](28-surrealdb-unified-substrate.md#parallel-execution-integration)
adds N8/N9/N10 for Parallel F02/F04/F03. Its accepted RC02/RC03 directions are recorded at
the coordinator; this companion owns the shared runtime/native design and package progress.
The focused source assessment at `84a1caf17656f00f38b22e703c7cbc2b63a44d2d` preserves current scientific policy, private workers,
existing adapter safety guards and the one shared DataFusion pool. No new probe or benchmark
qualifies these directions.

### N8 — Temporary preparation demand and bounded entry

Keep `MathPolicy::worker_bytes` as a maximum allowed extent. In `math/modeling.rs`, nested
provider construction, diagnostics and general rebind use that maximum as immediate job demand.
Inspect each constructor's temporary and escaping allocations, propagated derivative demand,
selected structure and value-dependent rebuilding before replacing that charge. Final
`retained_bytes()` alone is not a pre-construction bound. Use a safe operation-specific bound
or incremental reserve-before-allocation at the owning builder; opaque preparation retains a
conservative bound until a tighter safe contract exists. Do not add a generic estimator or
replace one maximum with another name. B6 migrates the actual modeling/preparation callers.

Add bounded waiting at asynchronous stage entry for temporary contention. `EngineResources`
already supplies the pool shared by queries, caches, compiler/math and retained results. Keep
that actual pool as capacity authority and attach release notification through its existing
pool composition; a wakeup only permits another actual reservation attempt, not a grant from
an independent byte counter. All relevant release paths, including caches, queries and escaped
result owners, must reach that notification. Register the wakeup before checking capacity so a
release cannot be missed. Coalesce notifications, bound waiter storage/count, and avoid spinning.

The selected ingress owns one population ticket before spawning synchronous submissions;
direct asynchronous jobs acquire it on first poll. A failed reservation releases
undispatched CPU before waiting while retaining that ticket, so pending work remains inside
the finite population. Retained session owners keep their own leases and population tickets.
Reacquire CPU and the actual pool reservation coherently without partial byte grants or
an outside waiter queue. Preserve the same absolute task deadline and cancellation owner
across every attempt; no timeout refresh or new scientific retry. For entry points without a
finite enclosing clock, define a finite admission-only wait bound at MathPolicy's owner before
introducing waiting there. The bound never extends an enclosing deadline. Individually oversized
work refuses before dispatch. Unchanged retained pressure ends in deadline or a typed bounded
refusal, not indefinite waiting. Use existing eviction only where its contract permits;
unrelated escaped results/caches are not cleared to manufacture progress.

Do not make synchronous allocation growth inside a running native worker wait while it owns
CPU, native locks or state needed for another worker's progress. Its existing reserve/refuse
behavior stays unless a separate supported safety argument is established. Cancellation before
dispatch removes the waiter; after dispatch, original native drain/retained transfer still applies.

### N9 — Persistent team scope extent

`NativeSession::session_on` reserves the session-thread stack; `serve` retains adapter scopes
across idle receive. Extend the existing adapter scope seam with the narrow known additional
stack extent needed before entry. Reserve it before team creation and retain its lease until
that scope is destroyed; the session's original owner survives its thread join.

Charge actual scopes entered. `execution::scoped` deduplicates by backend, then recursively
enters scopes. Both POUNCE and POUNCE-convex call `with_threads`, so an extended scope containing
both can create multiple teams. The initial correction charges each actual team's checked
thread-count × configured-stack extent. Coalescing a shared pool owner is eligible only after
nested execution and retained-state semantics are established. Serial/no-team scopes retain
the simpler path. Keep fixed foreign allowances separate; these explicit stack reservations
are not a process-RSS guarantee.

Failed entry, arithmetic overflow, replacement, panic and cancellation destroy the matching
team before releasing its lease. Idle scopes keep their charge even when request CPU permits
have been released. Replace the one-stack assumption only on affected retained-team paths;
retain correct one-shot accounting and native thread affinity.

### N10 — Backend strategy and exclusion decision

The review's [library matrix and alternatives](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#8-library-fit-and-alternatives)
are the starting evidence. Select one-thread inner execution for ordinary independent cases
as the initial strategy; larger explicitly selected teams consume the same effective case CPU
budget. Symbolica optimization/construction cores are separate admitted work. Confirm effective
faer features/global parallelism in implicit and Diffsol factor paths, rather than assuming an
explicit sequential context governs all factors.

Record for each supported backend/profile: independent-instance guarantees, internal team
selection, process-global guards, retained-state lifetime, supported mixed use, waiting/cancellation
and teardown. Cover KINSOL/IDAS contexts, Diffsol/faer, POUNCE teams, Clarabel's QDLDL/MKL route,
SCIP's linked thread-safe/TPI build and deterministic portfolio, PETSc process exclusion,
HiGHS retained readers/Uno scheduler reset, and Ipopt MUMPS/SPRAL/PardisoMkl. Existing typed
factorization choices remain eligible; availability proves neither throughput nor scientific
parallel equivalence. Preserve selected environment/threading premises and original assessment.

Settle how exclusion admission waits before occupying general compute permits while the real
native guard and destruction stay on their owning thread. Do not transfer a thread-bound guard,
remove safety exclusion, or invent an independent authority beside it. Explicitly settle release
or isolation of idle retained HiGHS state when Uno needs entry; the same-session adapter already
clears retained state, and no current cross-session deadlock is claimed. Bound waiting and preserve
original clocks, CPU ownership after dispatch and drain after cancellation.

For process-constrained/incompatible mixtures, compare existing managed-process isolation with
in-process exclusion; prefer existing isolation when it meets the workload without a complicated
retained-state arbitration protocol. N10 ends with selected supported combinations and consumed
contracts. L7 then fixes numeric process/team budgets; unsupported premises constrain that
combination, not unrelated N8/N9 work. Tokio/Rayon alternatives earn adoption only by reducing
machinery while preserving these lifetimes; an async poller is not CPU/native admission.

| Package | Inputs and delivered behavior | Migration, deletion and targeted acceptance | Status |
|---|---|---|---|
| N8 — Demand and temporary entry | Accepted RC02; constructor allocation bounds and common EngineResources pool. Implement safe demand admission, bounded waiter/release notification and original clocks. | Move affected preparation/diagnostic/rebind consumers with B6; remove max-capacity-as-demand and obsolete entry waits after controls. Test generous caps/small work, construction peaks, real query/cache/result releases, missed-wakeup races, bounded waiters, retained pressure, oversized refusal, cancellation and deadline without dispatch. | Integrated admission and known-source producers; synchronous burst controls pass. Known class/order, fitting and trajectory-diagnostic producers are integrated; final targeted and live deployment controls precede consumer acceptance. |
| N9 — Actual persistent teams | Existing session/adapter scope contracts; known stack extent and N9 scope ownership. Reserve every actually created team before entry through teardown. | Migrate POUNCE/POUNCE-convex retained scopes, including nested/extended scopes; replace mismatched accounting. Test one/multiple teams, idle retention, repeated/switching scopes, refusal before creation, overflow, failed entry/panic/cancel and final join release. | Integrated actual-team stack ownership; focused lifetime controls pass. Composed E3/E4 remains; no RSS claim. |
| N10 — Native coexistence strategy | Accepted RC03; pinned capabilities, actual guards and effective feature/thread settings. Select supported instance/team/process combinations and exclusion admission. | Move affected runtime/native admission and supply L7/C6/C7. Preserve existing safety guards; remove displaced coordination only after mixed retained-state, wait/cancel/deadline and teardown controls. Record unknown combinations and observable reopen conditions. | Integrated actual HiGHS/PETSc guard waiting and serial faer context; focused exclusion/cancellation controls pass. MUMPS retains its opaque guarded section; composed E3 remains. |

N8/N9 can proceed alongside A4. N10's decision is independent where inputs are settled;
its selected combinations may consume N9's tested scope accounting. N4 extends numerical
consumer reconciliation to B6/C6/C7/N8–N10 and all actual ordinary/analysis consumers before E3.
Source reconciliation previously completed for N1–N7 does not qualify the new extension.

## Complete basis through analysis consumers

N11 moves actual ordinary analysis, dynamic mode discovery/preparation, fitting experiment
and transient branches, and applicable initialization/recycle preparation entries to B7's
complete basis and fresh consuming wrapper. Distinct modes, experiment correspondence,
original/supplier views and stronger derivative demand remain real preparation differences.
Share their compatible basis/body inputs rather than asserting all products are equivalent.

Preserve initialization's existing base/session reuse and private library evaluator/factor
state. Do not generalize the reviewed entry duplication into per-time-step, per-fit-trial or
per-recycle-iteration rebuilding. Changed matrices/coefficients, branch/start and original
physical assessment remain current. No new numerical controller or second cache is selected.

| Package | Prerequisite and target behavior | Completion and status |
|---|---|---|
| N11 — Analysis basis consumption | Working J2/B7 plus applicable I1/I2; existing N1–N10 numerical owners and original scientific contracts. Confirm and migrate affected entries with complete demand/layout/provider identity and fresh value/attribution binding. | Implemented; Tested composed conformance, five fitting/experiment/mode controls and initialization/recycle with original scientific checks. Replaced common-basis preparation is removed; distinct demand/layout and mutable workers retain their owners. 28e records commands, repaired results and qualification limits. |

N11 does not reopen every earlier numerical finding. It supplies functional consumer closure
for the new preparation scope; N4 retains its earlier coverage basis, and 28e owns affected
assembled U02 evidence. Relevant library bulk/lifecycle alternatives remain eligible where
they resolve an actual consumed-contract gap.

<a id="graph-hash-flow-and-suppliers"></a>

## Graph/hash extension — flow policy and supplier registration

This implemented correction and selected V2 extension use the working N11/B7 exact-basis foundation.
[Plan 28](28-surrealdb-unified-substrate.md#graph-and-hashing-review-integration) owns
Graph/hash F02/F03 dispositions. The earlier baseline describes its original migration,
not a reason to reconstruct the already shared preparation basis.

### N12 — Indexed flow decision policy

[Graph/hash F02](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#f02)
arose in `FlowGraph::admit`: sorted tear decisions were linearly searched for every edge
when constructing the forbidden-cycle witness graph. The existing physical binding checks,
occurrence graph and independent cycle witness are suitable foundations. Replace only policy
discovery with binary search over the already sorted unique decisions. This removes the
nested full scan without another map/cache or hash change.

Inputs remain the admitted declaration, registry and limits; outputs remain the same graph,
physical bindings or typed refusal. Keep isolates, parallel connection occurrences, unused
and missing decision rejection, canonical ordering and the complete forbidden-cycle witness.
Preserve existing construction/resource admission. N12 itself changes neither identity framing
nor equality. The separately selected N14 below changes new admission to FlowProjectionV2;
it does not reinterpret V1. No global graph container migration is required.

Extend `pse-structural` flowsheet controls for reordered decisions, many decision groups,
parallel occurrences and every policy, including forbidden cycles with expected connection
and decision identities. Existing physical affine-conversion and construction-limit controls
remain. Source inspection establishes the absence of a per-edge full decision scan; any
latency comparison requires measurement. Compile `pse-structural`, run its narrow unit filters,
and remove the old scan without a retained compatibility branch.

### N13 — Demand-specific supplier registration

Consume [B8's working compiler topology](28b-selected-compilation-and-reuse.md#b8-retained-supplier-topology)
at every `inner_registrations` entry, including observation selection, case preparation and
derivative-demand upgrades. B8 owns immutable discovery; this package owns current demanded
outputs/coordinates, derivative order, hints, policy, controls and fresh registrations.
Automatic reduced suppliers and their actual derivative requirements remain current.

Observation requests retain their selected closure; case requests retain their current
full-scope ordering obligations. Do not narrow a case simply because a demanded-provider map
is smaller, or replace actual derivative demands with B8's value-level topology. Value,
First/Second and other consumed capabilities remain distinct. Each case/attempt receives
private evaluator/factor state, current starts and original numerical/physical assessment.
Topology reuse supplies no seed usability, scientific acceptance or publication authority.

Extend the existing nested observation, sibling-hint, value-demand and composed second-derivative
controls in runtime `workflow/modeling/implicit.rs`. Add selected/full-scope cycle cases,
fresh-case state and repeated order upgrades. Independently specify expected supplier sets
and actual provider-order requests; do not use the retained topology as its own oracle.
Compile compiler/runtime consumers and use narrow runtime unit/native-unit filters as needed,
with force-validation and the current native environment. After all callers use B8, delete
displaced discovery/registration helpers and their mechanism-specific fixtures. Independent
provider/scientific tests remain applicable.

<a id="n14-explicit-flowprojection-v2-framing"></a>

### N14 — Explicit FlowProjectionV2 framing

The maintainer selected Graph/hash RC04 on 2026-10-09.
[ADR-0167](../adr/0167-frame-flow-projection-v2.md) records the change, and the
[independent formal review](../design_review/reviews/design_review_flow-projection-v2_2026-10-09.md)
accepted the bounded target before dependent implementation. The ADR remains proposed until
the decision-PR route; operator implementation authorization and design acceptance do not
change its status. Blueprint §5.3/§17.4 own the enduring contract.

**Implemented:** new flow admission emits FlowProjectionV2 with explicit collection/item tags,
top-level counts and separate port/binding counts for every parent. Canonical field ordering,
float encoding, BLAKE3 and existing physical admission remain. Current paths emit no V1
fallback; the V1 catalog spelling and historical identity meaning remain. Existing flow
fingerprints feed runtime documents, native tear assumptions/compatibility and recycle causal
identity, preserving fresh session/start state and complete witnesses.

This resolves sequence/parent framing only. Quantity/unit identifiers and conversion values
do not encode every physical-registry fact; full graph equality and current checked context
remain necessary. Neither a sole-key cross-context flow cache nor durable fast hashing is
selected. Targeted structural, identity, native tear and runtime fingerprint/recycle controls
passed; complete affected journey status remains at
[28e](28e-rebuild-retirement-and-qualification.md#graph-and-hashing-extension-acceptance).

<a id="pc-saft-numerical-fact-investigation-boundary"></a>

### PC-SAFT numerical-fact investigation boundary

The graph/hash production smoke exercised the corrected PC-SAFT fixture through cold,
warm, value and structural preparation. All four stages returned the preserved typed
`operation.unestablished_invariant` refusal, rather than a prepared product; the completed
harness is not accepted PC-SAFT preparation or a performance result. Raw results and
artifact conditions are at [28k's evidence](../design_review/evidence/graph-hash-followups-2026-10-09/pcsaft-02-production-smoke.json).

**Interface-checked investigation lead:** default numerical allowances derive the
self-difference of point quantities using `NoInvariantFacts` in
`pse-math::numerics::engineering_error_quantity`. The correctly declared
`LogFugacityCoefficient` targets require operand-contract prerequisites for their registered
subtraction, and those facts are present in the fixture's loaded physical inventory.
Conditional-boundary difference projection has an analogous no-facts path. The diagnostic
does not attribute the first failing caller, so source reasoning identifies a concrete
guaranteed refusal path without claiming to have uniquely located the observed first failure.

**Ownership handoff, 2026-10-09:** [Plan 33/EFF06](33-efficiency-principles-remediation.md#eff06)
now owns tracing the actual failing call, threading current immutable physical prerequisites
through fresh numerical difference inference, and the correction's targeted qualification.
Its disposition table is the current owner of Efficiency F08. This boundary retains the
original four refusals and uncertainty about the first caller; they are not accepted
preparation/performance evidence or numerical baselines. Present, missing and changed operand
facts and exact numerical allowances remain required, followed by a freshly prepared fixture
under its current lawful scientific specification. This remains outside the selected graph/hash
implementation; broader Plan 28 scientific consumer/qualification obligations stay here.

| Package | Working prerequisite | Completion boundary | Progress |
|---|---|---|---|
| N12 — Indexed flow policy | Existing admitted flowsheet contracts; independent of J3/B8/GH investigations. | Exact policy access replaces the nested scan; physical occurrence and witness outcomes preserved. | Implemented; targeted structural/consumer controls passed; affected journey status is owned by 28e. |
| N13 — Supplier registration integration | B8 selection/order implementation plus existing N11/private numerical owners. | All confirmed registration variants use retained discovery; actual demand/state remain fresh; displaced path removed. | Implemented through retained B8 ordering; targeted compiler/native-runtime controls passed; affected journey status is owned by 28e. |
| N14 — Explicit V2 framing | Selected Graph/hash RC04; ADR-0167 and formal target review before implementation. | New tagged/count-framed admission and affected fingerprint consumers; preserve historical V1 meaning, full equality and fresh numerical state. | Implemented; targeted structural/identity/native-runtime controls passed; affected journey status is owned by 28e. |

[28k](28k-graph-kernels-and-hashing-investigations.md) records the completed broader
compact-layout/hash decisions. GH1 supports B8's existing petgraph mechanics and N13's fresh
registration boundary. GH2's original retained-hash conclusion predates the implemented local
hashing followups below; durable BLAKE3 remains. GH3 supplies N0's retained catalog decision
above rather than opening a competing investigation. The full authored checker and complete
canonical dependency premises remain production authority; finer checker-domain work remains
a proposal. Real Recipe persistence demonstrates a bounded pure-memo capability without
selecting RC03 or replacing qualified portable reconstruction. Those historical inquiry
conclusions alone did not implement the corrections. N12/N13 source migration is now
implemented: binary policy lookup replaces the edge-by-decision scan, and observation/case/derivative registration
consumers share B8 ordering while calculating current demand and creating private state.
[B8's scoped compiler controls](28b-selected-compilation-and-reuse.md#b8-scoped-functional-evidence)
passed.

**Tested:** the final targeted native runtime selection passed 24/24, including nested demand,
portable ownership/receiving controls, exact-basis/partial-description settlement, forced
prehash collisions, flow waits and causal recycle. Structural flow controls passed 6/6, native
tear controls 3/3 and identity controls 61/61 against the zero-failure baseline. The commands
and their exact modes belong to
[28e's graph/hash acceptance](28e-rebuild-retirement-and-qualification.md#graph-and-hashing-extension-acceptance),
which also owns remaining affected journeys and assembled acceptance. Root coordinates shared
declarations and integration. Package progress lives here; whole-finding closure stays at the
coordinator, and these scoped results do not close broader E3/E4/E5. The four affected native
Python journeys passed 4/4 against the zero-failure baseline, including authored recycle,
solve/join warm-start retention and public preparation/conformance policy; 28e retains their
canonical-owner command/environment details and remaining smoke/qualification status.

**Implemented:** the factorable interner uses Fx with complete `Node` equality; a projection computes its structure key once, and immutable `BasisKey` computes a
process-local Fx prehash once while retaining complete equality. The maintainer selected these
targeted improvements unless incorrect or substantially regressive; marginal gains suffice.
**Tested:** the final native factorable selection passed 47/47 controls, while the runtime
selection above exercises exact-key collision separation. Table callbacks, complete equality
and one initial key traversal remain costs; no end-to-end speedup is inferred from isolated hashing.
[28k's followup outcome](28k-graph-kernels-and-hashing-investigations.md#followup-verification-and-outcome-2026-10-09)
owns inquiry evidence, adoption conclusions and remaining investigation avenues. RC02 durable
hash replacement remains unselected; only the separately selected N14 changes durable flow
framing.

## Work packages and dependencies

| Package | Inputs and delivered behavior | Migration, deletion and focused acceptance | Status |
|---|---|---|---|
| N0 — Settle numerical variants | Coordinator coverage plus current contracts; finish caller multiplicity/identity/lifetime decisions and the bounded library investigations above. | Name every affected consumer and justified retained variant. Source suffices for established repeated joins; use a bounded observation only to select between consequential alternatives. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| N1 — Ordered projections | PE01 and N0's relevant correspondence decisions. Implement common checked projection mechanics and owner-specific prepared maps. | Migrate policy/quality/goals, preflight, trajectory and all confirmed related consumers together with their maps. Delete displaced repeated joins/helpers and tests specific to removed mechanisms. Check kind/ID collisions, duplicate/missing/refused entries, reordered inventories, fixed composite coordinates and sample alignment. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| N2 — Checked preparation reuse | B1/B2 and existing protection/retention; settle complete reusable selected-input identity. | Move durable ready occurrences and every confirmed ordinary/analysis consumer off repeated unchanged hydration/generic admission. Test fresh/reused equivalence, A/B/A, value/structural edits, absence/shadowing/provider changes, protection expiry, eviction and cancelled flights. Remove replaced hydration/admission caches only after callers move. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| N3 — Compatible numerical owners | Applicable N0 session/factor/Taylor decisions and working N1/N2 slices where consumed. | Extend actual existing library owners; migrate fitting/profile/shooting/implicit/dynamic consumers selected by N0. Delete superseded factor/workspace construction paths. Check current-point results, changed compatibility, singular/nonfinite recovery, owning-thread drain and released allocations. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| N4 — Numerical consumer closure | N1/N2, adopted N3, working C5/N5/N6/N7, and the parallel B6/C6/C7/N8–N10 consumer migration. Reconcile coordinator coverage and every confirmed applicable caller. | Every confirmed variant uses the shared target or has a reasoned distinct contract. Exercise a new analysis composition and changed provider/layout to show where customization belongs; hand functional scope to E3/E4. | N1–N10 source reconciliation integrated, including explicit conservative variants below; final targeted/live controls and assembled E3/E4 pending. |

N1 and N2 are independent after their own inputs are settled. Library investigations need
not delay the established projection correction. A native owner can migrate incrementally,
but cannot keep an obsolete fallback after replacement controls and callers are complete.
Root owns shared declarations, contract changes and integration; executor scopes follow
actual file overlap rather than document boundaries.

## Verification and evidence limits

**Interface-checked:** this plan's source assessment identifies the repeated policy/preflight/
trajectory projections and selected-admission path, and the existing reusable owners above.
It does not quantify their latency share or qualify unexamined scientific models.

**Proposed acceptance:** touched packages compile through `just check-package` or the relevant
native compile recipe. Pure projection/retention mechanisms use `just unit-package` or
`just unit-native-package` with narrow actual unit filters; correctness checks explicitly
enable force-validation. Mechanism tests establish both preserved meaning and reuse through
construction/evaluation counters where necessary, not elapsed-time assertions.

Preserve production contextual accuracy and sensitivity policy, frozen scales and separate
base/response/perturbed-output checks. Independent small expectations and original-space
assessment expose a wrong mapping. Do not demand bit-identical separately solved floating
outputs or theoretical convergence order; exact transported rows and identity maps have their
own exact contract. Test ordinary states, branch/provider failures and cancellation, not just
an artificially tight tolerance that hides production behavior.

E3 composes affected solver, dynamics, fit, study and Rust/Python journeys once after the full
functional extension. E4 measures preparation, numerical work and publication separately,
including coordinate/sample growth, cold/warm, value-only and structural changes. Compare
production measurement mode with the same physical decision basis; label forced-validation
overhead separately. No speedup, full conformance or completed implementation is claimed here.

## Checkpoint

N11 is implemented through the common exact B7 basis consumed by ordinary, dynamic-mode
and fit-experiment preparation. Bounds, fixed/free layout, profiles and mutable numerical
workers remain under their downstream owners. Targeted and composed qualification is in
progress; existing N1–N10 evidence does not qualify this added composition. The coordinator
owns PA02/PA03/U02 status and 28e owns assembled acceptance.

N8's release-notifying common pool, one bounded population ticket, oversized refusal,
CPU release while waiting for memory and original-clock admission are integrated. Term demand
and eligible flat rebind construction use producer bounds. Selected artifacts retain the
original scratch ceiling while admitting compact source demand; optimized numeric growth
extends the same reservation before export. First block binding, flow/tear construction
and known declared-root factories use source demand. Class/affine discovery, support-order
upgrades, path curvature/isolation, QP class production and explicit cone preparation are
integrated. Conditional initialization counts schedule/support and matching populations;
value-dependent block bindings have separate demand rather than repeated full-parent bounds.
Fitting counts source sparse response/constraint/Hessian/Gram construction. Trajectory diagnostics
count whole-program workers and guards, actual diagnostic dimensions and every retained
function/sample report. Final targeted controls precede live deployment and assembled acceptance.
Opaque symbolic storage remains a separate conservative allowance, without an RSS claim.
N9 accounts for actual additional adapter-team stacks before scope construction and retains
their leases through idle ownership and owning-thread teardown. Diffsol's serial context is
consumed by its faer factor. N10 waits on the actual HiGHS/PETSc exclusion guards after releasing
CPU, then reacquires the same CPU allocation before native entry; idle HiGHS state is relinquished
on its owning thread where Uno needs scheduler ownership. Required reuse remains truthful.
Focused primitive admission, team lifetime, native wait/cancellation and rebind controls pass;
public study, selected managed placement and assembled acceptance remain separate. The
coordinator owns whole-finding status. Existing N1–N7 evidence retains its original scope.

N4 retains explicit conservative entries for recognized-cone symbolic recognition/epigraph
expansion and SOS polynomial/moment construction. Generic factorable graph cardinality does not
bound their intermediate polynomial populations or library SDP scratch. Their original scientific
limits and whole-entry allowances remain; reopen when those producer owners supply construction
bounds or reserve before each expansion in the same pool. Implicit/provider-envelope factorable
exports and provider-bearing dynamic modes similarly require a producer-issued population/lifetime
contract before replacing conservative entry. These are supported distinct contracts, not claims
that every backend or foreign heap has a source-sized bound. Actual retained workspace capacity
remains charged by its existing owner. The surrogate actor retains its workspace/stacks through
join; no generic transfer of runtime admission callbacks across arbitrary actor threads is claimed.

Continuation seed inspection allocates no numeric payload. Converted seeds reserve before
materialization, share immutable payloads and retain their grant in both modeling and bare
prepared-solve owners. Durable encoding retains the same grant through publication and separately
includes materialized primal and encoding populations. Actual completion attempt and original
coordinate coverage govern provenance and permission; replacement starts release displaced owners.

N5 retains admitted mode-chain/support/refill/matrix templates; workers clone private mutable
values and keep provider/guard state private. Frozen state/output allowance projections are
also prepared once through the checked kind/ID access and shared across program/worker
access; mode composition rebuilds these ordered arrays and their retained extent at its owner.
N6 factors one current IDAS sample matrix for its first- and second-order jump directions;
the independent single-shot Diffsol jump keeps its distinct operation boundary. N7 consumes
the combined report/gradient directly and admits coherent value-first upgrades under the
recorded physical comparison contract. Focused integration, adjoint and cache/refusal controls
pass. Integration, fitting gradient/Hessian and shooting consumers reach the retained worker
owner; forward-response/rank and second-order checkpoint lifetimes remain justified distinct
contracts. Rust/Python and benchmark study callers now propagate the owning cancellation
source. The process registry adds actual Gradient fitting and a two-parameter IDAS exact
Hessian fitting consumer; ordinary dynamic rebind remains a forward operation and cannot
be presented as a backward sample-factor measurement. The public Diffsol forward path
also repairs its pinned-library initial sensitivity coefficient through public state
reinitialization; affine and scheduled controls reproduce the old error and pass with
unchanged production controls. Complete composed E3/E4 before closing N4. Earlier N1–N3
evidence retains its original scope; numerical tolerances and evaluator ABI are unchanged.

Execution is authorized and underway. N0's bounded source decisions adopt structural Taylor
zeros and retained faer numeric/scratch storage at the existing attempt-local Diffsol,
implicit/affine and continuation owners. Salsa checked-Catalog admission and selected-result
backdating remain; reopen on measured unchanged-selection cost. Immutable point-owned response
factors remain isolated. Current standalone fitting/shooting have one native solve; finite
profile chains reuse the existing native owner, while a newly introduced compatible repeat
sequence would reopen the one-shot decision.

N1's checked kind/ID inventory, complete composite maps and hoisted trajectory metadata are
implemented, with dynamic/shooting/control/covariance and conditional-boundary consumers moved.
N3 refreshes current matrices and clears failed owner state; it does not reuse acceptance or
starts. Focused numerical units, refreshed-matrix/failure recovery and actual native profile-chain
consumers have positive evidence. Independently solved chain outcomes use production variable
budgets while exact identity and outcome attribution remain exact.

N2 now retains only immutable checked selected admission under the existing service's bounded
cache. Fresh protected eligibility reuses the same complete premise checker as durable products,
including absent references, and exact lookup premises use bounded bulk acquisition. Compatible
flights share generic admission; request buckets retain eligible A/B/A candidates with byte
bounds and clear-generation fencing. Ordinary preparation, flow/point analysis and selected
revision consumers use this path. Fresh/readmission, protection expiry, structural/unrelated edits, absence and nearer shadowing,
provider configuration, byte-bounded eviction and queued cancellation/drain controls passed
against the selected server. Actual deployed-role eligibility remains an E3 prerequisite.
N4 and enclosing E3/E4 acceptance remain open. No performance improvement is claimed.
