---
title: Shared numerical projections and preparation
status: in-progress
date: 2026-10-07
adrs: [ADR-0164]
review_sources: [docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md, docs/design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md]
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
| Catalog invalidation | Inspect `Catalog.checked`, equal publication and actual selection work for unrelated/selected edits, additions/deletions and visibility changes. Keep current Salsa 0.28.4 backdating unless repeated selection warrants finer complete inputs. A short execution-event observation is sufficient if source cannot settle the choice; no new query engine or persistence rewrite. |
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

| Package | Prerequisite and consumer scope | Targeted acceptance and deletion | Status |
|---|---|---|---|
| N5 — Immutable dynamic layout | Admitted dynamic program, existing checked projections/sparse assembly and retained allocation owner; integration/gradient/Hessian/shooting workers. | Shared layout across repeated workers; independent concurrent scratch/values/providers; changed order/scales/constants/support, extent refusal and existing event/physical behavior. Delete per-worker stable layout reconstruction. | Scheduled. |
| N6 — IDAS sample elimination | Existing current partials and faer numeric/factor owner; first/second-order sample jumps. | One factor per sample with direction/multiplier correspondence; changed samples and singular/nonfinite recovery; production-basis derivative checks. Delete repeated sample-local factor construction. | Scheduled. |
| N7 — Demand-aware transient point | Existing combined gradient/report API and fitting-owned cotangent; settle value-first upgrade semantics before cache migration. | Gradient-first one forward/backward, objective-only no backward, repeated gradient cache reuse, value-first upgrade coherence, changed candidates and cancellation/memory refusal. Remove discarded-forward-report composition where replaced; preserve the justified objective-first recomputation route. | Scheduled; upgrade contract must be recorded before dependent implementation. |

N5 and N6 can proceed independently of replay admission and each other, coordinating shared sparse
owners if changed. N7 consumes N5 when using its migrated workers, but its demand contract can be
settled earlier. N4 closes C5/N5–N7 and every confirmed comparable consumer before E3; no separate
N-only full campaign is added. E4 distinguishes removed construction from whole-operation gain.

## Work packages and dependencies

| Package | Inputs and delivered behavior | Migration, deletion and focused acceptance | Status |
|---|---|---|---|
| N0 — Settle numerical variants | Coordinator coverage plus current contracts; finish caller multiplicity/identity/lifetime decisions and the bounded library investigations above. | Name every affected consumer and justified retained variant. Source suffices for established repeated joins; use a bounded observation only to select between consequential alternatives. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| N1 — Ordered projections | PE01 and N0's relevant correspondence decisions. Implement common checked projection mechanics and owner-specific prepared maps. | Migrate policy/quality/goals, preflight, trajectory and all confirmed related consumers together with their maps. Delete displaced repeated joins/helpers and tests specific to removed mechanisms. Check kind/ID collisions, duplicate/missing/refused entries, reordered inventories, fixed composite coordinates and sample alignment. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| N2 — Checked preparation reuse | B1/B2 and existing protection/retention; settle complete reusable selected-input identity. | Move durable ready occurrences and every confirmed ordinary/analysis consumer off repeated unchanged hydration/generic admission. Test fresh/reused equivalence, A/B/A, value/structural edits, absence/shadowing/provider changes, protection expiry, eviction and cancelled flights. Remove replaced hydration/admission caches only after callers move. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| N3 — Compatible numerical owners | Applicable N0 session/factor/Taylor decisions and working N1/N2 slices where consumed. | Extend actual existing library owners; migrate fitting/profile/shooting/implicit/dynamic consumers selected by N0. Delete superseded factor/workspace construction paths. Check current-point results, changed compatibility, singular/nonfinite recovery, owning-thread drain and released allocations. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| N4 — Numerical consumer closure | N1/N2, adopted N3, working C5/N5/N6 and N7's settled demand/cache contract and migration. Reconcile the coordinator's numerical capability rows and actual callers. | Every confirmed variant uses the shared target or has a reasoned distinct contract. Exercise a new analysis composition and changed provider/layout to show where customization belongs; hand functional scope to E3/E4. | Earlier migrations implemented; expanded consumer closure and E3/E4 acceptance pending. |

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

The enhancement review adds scheduled N5–N7 and C5's stored use of N2. Earlier N1–N3 evidence
retains its original scope; N4 remains open until these consumer obligations are implemented and
exercised. First settle N7's point-upgrade contract, then migrate complete consumers with retained
layouts/sample factors. No numerical tolerance change, evaluator ABI pivot or history sharing is
authorized by this document update.

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
