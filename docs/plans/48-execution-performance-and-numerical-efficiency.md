---
title: "Plan48: Execution performance and numerical efficiency"
status: draft
date: 2026-10-20
adrs: []
review_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md, docs/design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md]
scenario_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md]
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [design, preparation, numerical-execution, performance, diagnostics]
---

# Plan48: Execution performance and numerical efficiency

## Purpose and basis

Remove avoidable preparation, mathematical-product, native-lifecycle and coordination work
through the complete process-execution pipeline. Preserve scientific meaning, practical
acceptance and honest completion while improving cold first-result, warm edit/re-solve,
study throughput and demanded result availability. The
[independent audit](../design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md)
and its [recommendation register](../design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md)
are the basis: AF01–AF18, N01–N04, RC01–RC12 and all 162 register rows. Original external
R1/R2 proposals are interpreted through the audit's corrections, not adopted wholesale.

**Authoring baseline, 2026-10-20. Interface-checked:** current implementation at
`1ee81798a9b884f3054b685ee9629e2c8d124498`, affected source/architecture owners and retained
uncommitted review inputs. **Proposed:** all new behavior and packages. The audit's
retained-run interpretation is historical **Measured** evidence under its stated conditions,
not a measured benefit from these remedies. No new simulation, product implementation or
solver qualification was performed during plan creation.

The standard is Core 3.4, Efficient Architecture Heuristics 1.0, Process Simulator 1.5 and
the pse-arrow binding. The target includes value edits without unrelated structural rebuild,
studies/sensitivities/optimization/fitting, recycles/tears, dynamics from the same definitions,
unit/property/solver extension and responsibilities testable with their actual dependencies.
Architecture sections own current contracts; this series owns proposed work and, after
adoption, its dispositions. Other plans keep their scope/status. Document creation does
not authorize product execution, default changes or dependency publication.

## Reading route and ownership

| Companion | Responsibility and packages | Principal obligations |
| --- | --- | --- |
| [48a: Source checking and incremental preparation](48a-source-checking-and-incremental-preparation.md) | A01–A06; indexed checking, mutable overlays and immutable artifacts | AF01/AF02/AF18; I01–I03 |
| [48b: Registry and receiving validation](48b-registry-and-receiving-validation.md) | B01–B05; descriptions, domain/template reuse and receiving eligibility | AF03/AF04; I04–I06 |
| [48c: Mathematical products and compiled execution](48c-mathematical-products-and-compiled-execution.md) | C01–C08; contributor indexes, sparse products, complete validity and modules | AF05–AF08; I07–I11 |
| [48d: Native controls, starts and recycle policies](48d-native-controls-starts-and-recycle-policies.md) | D01–D07; native meanings, initialization and qualified method policies | AF12/AF13; I12–I20 |
| [48e: Dynamics, fitting and preconditioning](48e-dynamics-fitting-and-preconditioning.md) | E01–E08; private allocation, selected products and anchored windows | AF14/AF15/N01; I21–I27 |
| [48f: Scheduling, resources and result completion](48f-scheduling-resources-and-result-completion.md) | F01–F09; compatible dispatch, readiness, continuation/drain and effect lifetimes | AF09–AF11/AF17; I28–I33 |
| [48g: Diagnostics and review publication](48g-diagnostics-and-review-publication.md) | G01–G06; reliable current state, coherent observations and streaming output | AF16/N02–N04; I34/I35 |
| [48h: Qualification and design publication](48h-qualification-and-design-publication.md) | H01–H04; selected assembled acceptance and enduring publication | All selected routes; I36 |

Companions own package state. This coordinator alone owns finding/investigation state,
rule decisions and the next route. The coverage appendix is navigation, not another ledger.
Package identifiers are companion-local; 48a/A01 is not a review finding.

## Combined target and foundation assessment

Retain model/physical authority, library mathematics, native iteration, workflow composition
and result/diagnostic ownership. Improve physical work within those boundaries:

| Existing foundation / inspected constraint | Target consequence |
| --- | --- |
| Eager named-type environments; granular source/expression inputs and stripped Namespace already exist | Indexed demand-driven lookup with complete membership/absence dependencies and phase-local mutable overlays; extend incrementality rather than replace the checker |
| Repeated immutable descriptions/bootstrap/domain preparation; schema-pointer/templates/retained-contract paths already exist | Reuse immutable supplier products while keeping receiving registry/configuration/functions and invocation-sensitive eligibility current |
| Implicit/derived actions reconstruct columns/dense scratch; factor checks occur after reconstruction | Canonical sparse demand and complete current validity before elision; action vectors rebind without unbounded caching |
| Joint arithmetic and native compilation already exist with ordered barriers | Pack bounded co-used demanded functions without crossing guards/provider effects or preparing every order |
| Native lanes/teams, durable readiness and in-process renewal already exist | Correct compatibility/frontier/capture/access gaps before introducing duplicate pools, counters or supervisors |
| Dynamic storage, current trial and checkpoints have different lifetimes; fitting already has GN curvature | Complete neutral trial reset and private bounded windows; retain constraint curvature and demanded derivative eligibility |
| Lossy diagnostic updates can drop admitted phase closure | Reliable current lifecycle independent from optional history; coherent progress separate from exact local work/cancellation |

Checked source/package artifacts, receiving admission, mathematical structure, current
linearization, native trial allocation and acknowledged result effects have different
lifetimes. Each survives only its complete consumed premises. Existing owners/functions
can express these distinctions; six new services/types are not required.

### Preservation and boundaries

- Physical acceptance stays **0.001 where selected**. Native relative norms, stagnation,
  gaps, stationarity, integration errors and effort limits retain distinct meanings.
- Preserve dimensions/bases/reference states, envelopes, structural roles, normalization,
  source mappings, root/branch permissions, effect order and first relevant refusal.
  Value-valid is not First-valid. Preserve exponent-materialization protection until
  a replacement is qualified at the actual Symbolica release.
- Preserve canonical fingerprint framing and exact equality promises. Hashing hashes is
  an identity change. Arithmetic regrouping needs declared numerical/guard/diagnostic
  equivalence; bit identity only where actually promised.
- Reuse established immutable validity, not old receiving permissions. Fresh/unsafe/native
  construction can create internal validation obligations. Correctness closures explicitly
  activate force-validation.
- Mutable workers, seed bindings, native destruction and charges remain with actual owners.
  Last chunk is not terminal success. Retain post-success stop veto, panic reconciliation,
  producer drain, native exclusion, leases/fences, ACK truth and uncertain-outcome recovery.
- Environment/provenance proof remains permanently eliminated. Actual scientific/ABI/layout
  cache premises do not authorize host attestation, a proof service or broad installation work.

The audit's S01–S06 remain the scenarios. Source/import/data edits invalidate consumed
meaning and negative lookups; larger sparse workloads follow contributors rather than
parent scans; provider/order/branch changes rebind affected products; adapter changes absorb
native mechanics; skewed cancellation/publication preserves other progress and owned drain;
unit/workflow extensions reuse semantic/composition owners without duplicating policies.

## Rule decisions

**RC01–RC12 were explicitly accepted by the operator on 2026-10-20**, selecting
“Accept all 12 recommendations as written.” These are accepted target boundaries and
bounded inquiry routes. Acceptance does not choose unknown numerical/platform winners,
authorize product implementation, or permit broader changes discovered later. The retained-rule
alternatives explain the scope of each choice, not a pending decision. Dependent adoption
still requires the row's concrete decision/design work and qualified inquiry outcome.

| Rule | Recommendation and consequence | Decision / date | Route before dependent change | If rejected |
| --- | --- | --- | --- | --- |
| RC01 | Correct KINSOL relative scaled-step/stagnation meaning; qualify native working controls with unchanged original residual/physical acceptance | accepted, 2026-10-20 | Blueprint §16.6 design correction; D01 specifies selected policy before implementation | Correct the false equivalence; retain conservative native settings without an absolute-step claim |
| RC02 | Permit immutable checked-artifact acceleration with complete dependencies and fresh receiving bindings; compare smaller artifacts with whole Salsa persistence | accepted, 2026-10-20 | Blueprint §14.4 design change; ADR for a selected new pass/storage binding where required; A05 | Lazy/granular in-process correction only; fresh processes still check packages |
| RC03 | Permit explicit coherent coalesced ordinary progress cadence/final flush and reliable admitted phase retirement, separate from optional history loss | accepted, 2026-10-20 | Observation contract/design decision at blueprint §23.1/§24.1 before G02; G01 reliable closure remains independently required | Keep per-return freshness; optimize local formatting/updates and reliable closure |
| RC04 | Permit pre-reconstruction product/factor reuse only with complete current linearization validity | accepted, 2026-10-20 | Mathematical kernel/assurance contract decision and blueprint §14.3 before C03 | Sparse construction remains useful; reconstruct coefficients before factor reuse |
| RC05 | Qualify an actual coupled index-1 DAE approximate preconditioner of the exact shifted operator, with refresh/singular-block behavior | accepted, 2026-10-20 | Short ADR for selected native/kernel binding and capability design before E05 adoption | Keep direct KLU and None/Jacobi; still implement selected Jacobi demand |
| RC06 | Investigate separately qualified vector/JIT/device realizations and compiled-default changes; adopt only a demonstrated eligible route | accepted, 2026-10-20 | Concrete realization decision, applicable ADR-0191/0193 route and blueprint §14.5 before adoption; C07/C08 | Scalar execution and bounded demanded-module packing remain |
| RC07 | Investigate restricted Ipopt same-structure reuse, including fixed/free/equality maps, value-dependent scaling, effective options and reset | accepted, 2026-10-20 | Native adapter decision/design at blueprint §18.3 before D04 adoption | Keep `warm_start_same_structure=no`; retain truthful compatible primal/dual seeds |
| RC08 | Investigate a qualified explicit recycle policy/default; do not preselect universal history/delay | accepted, 2026-10-20 | Policy/design decision at blueprint §17.4 after D06 comparison | Current defaults remain; indexed exchange and selectable acceleration still apply |
| RC09 | Investigate process-isolated or broader concurrent native capability for a concrete workload | accepted, 2026-10-20 | Native deployment ADR/design route before F08 adoption | Preserve MPIUNI/scheduler/TPI gates and actual native owner drain |
| RC10 | Investigate additional adapter-specific neutral reset or worker placement with complete private state and owner-thread lifecycle | accepted, 2026-10-20 | Selected reset/threading contract decision at blueprint §18.8 before affected D04/E01/F08 adoption | Reuse only qualified lanes/scopes; optimize immutable preparation/local scratch |
| RC11 | Investigate generated typed authored-document loading only if template/domain reuse is insufficient, retaining identical registry/SQL/receiving obligations | accepted, 2026-10-20 | Selected loader/boundary decision and blueprint §4 design route before B05 adoption | Optimize existing ingress/templates/retained contracts without a second validator |
| RC12 | Investigate compute/commit lifetime separation only for observed blocking, with bounded pending products and unchanged ACK truth | accepted, 2026-10-20 | Workflow/publication lifecycle ADR/design at blueprint §§19/20/24.1 before F06; any commit-contract change requires a separate concrete decision | Joined publication remains; improve dispatch/capture/actual crossings first |

These rows do not change identity framing, allow guard-after-fusion, transfer receiving
eligibility, relax physical acceptance or remove native safety gates by analogy. Newly
exposed impacts return as concrete decisions. Accepted records stay immutable; any required
supersession follows its decision route rather than an edit to history.

## Execution sequence and prerequisites

1. Apply the recorded rule decisions and schedule required ADR/design work before dependent
   implementation. Establish truthful native controls/receipts (D01–D03) and reliable phase
   retirement (G01) early; comparisons depend on those meanings.
2. A01–A04, B01–B03, C01/C02/C04/C05, F01–F03/F09 and G03/G05 supply bounded corrections
   independently. They consume actual receiving, support, compatibility and terminal-producer
   behavior, not merely agreed interfaces.
3. Resolve artifact, arithmetic, native-policy/reset and scheduling inquiries before their
   conditional adoptions. Local work removal does not wait for a persistent compiler,
   universal pool or platform expansion. C01 supplies indexes before D05 physical grouping.
4. E03 consumes actual private workers and shared capacity/affinity/joined drain; fresh
   sessions are valid. It consumes E01 retained reuse/E02 optimizations only when available,
   not as prerequisites to independence. E04 is independently useful.
   F04 shares only immutable admission; F05/F06 preserve durable
   identities. G01 precedes increased span volume; G02 requires the selected cadence.
5. Migrate actual consumers and remove proven replacements with targeted acceptance.
   H01 reconciles completeness, H02 runs revealing short cases, H03 qualifies selected
   assembled scope/final checks once, and H04 publishes enduring meaning and reconciles
   existing plans after implementation/qualification is complete.

Document order is not a synchronization barrier. Shared file ownership is separate from
logical dependency: assign one writer for generated declarations, math products, native
adapters, observation state and lifecycle metadata. No compatibility API or second
production path remains after a replacement is proven and callers have moved.

## Finding dispositions

All findings are **open**: scheduling a proposal is not acceptance, implementation or resolution.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
| --- | --- | --- | --- | --- |
| AF01 | S01/S02 | open | 48a/A01–A02 | Independent lookup/overlay outcomes and reduced examined-reference work |
| AF02 | S01 | open | 48a/A03/A05–A06 | Correct invalidation, bounded retention and qualified artifact decisions |
| AF03 | S01/S02 | open | 48b/B01/B04 | Same identities/custom fallback; fewer repeated descriptions/bootstrap forces |
| AF04 | S01/S03 | open | 48b/B02–B03/B05 | Domain/template reuse, receiving eligibility and malformed-input rejection |
| AF05 | S02/S04 | open | 48c/C01; 48d/D05 | Contributor indexes and separately qualified grouping/root policy |
| AF06 | S02/S03 | open | 48c/C02–C03 | Sparse products and complete validity before reconstruction elision |
| AF07 | S02/S03 | open | 48c/C04 | Sparse flat direct HVP eligibility without losing entry/reverse routes |
| AF08 | S01/S02 | open | 48c/C05/C07 | Bounded named modules, fewer children and safe code lifetime |
| AF09 | S02/S04 | open | 48f/F01 | Compatible/incompatible dispatch and unchanged assessment |
| AF10 | S02/S05 | open | 48f/F02 | Transition-driven ready visits, continuation/retry order preserved |
| AF11 | S02/S05 | open | 48f/F03 | Sibling progress during skewed capture; continuation/producer drain retained |
| AF12 | S04 | open | 48d/D01–D02 | Separate KINSOL/cone obligations; original acceptance unchanged |
| AF13 | S04 | open | 48d/D03–D04 | Submitted primal, optional payload and internal reuse reported distinctly |
| AF14 | S03/S04 | open | 48e/E01–E02 | Complete reset/failure discard and current scratch/product reuse |
| AF15 | S02/S03 | open | 48e/E03 | Bounded private fixed-trial windows with ordered results |
| AF16 | S02/S05 | open | 48g/G02–G03 | Coherent current/final progress and reduced shared work |
| AF17 | S02 | open | 48f/F09 | Indexed transport preserves subsets/reordering/unknown-ID errors |
| AF18 | S01/S02 | open | 48a/A04 | Typed masking preserves field roles/signed zero/conservative fallback |
| N01 | S02/S03 | open | 48e/E04 | Jacobi demand avoids unconsumed full Jacobian products |
| N02 | S05 | open | 48g/G01 | Reliable admitted phase retirement under optional-history pressure |
| N03 | S02/S05 | open | 48g/G03 | Prepared progress IDs preserve all actual consumers |
| N04 | S02/S05 | open | 48g/G05 | Equivalent coherent publication with bounded row retention |

## Investigation outcomes

Companions specify finite question/options/evidence/stopping conditions. Completion means
qualified adoption or justified no-adoption/support limit with an observable reopening
trigger. Declining an alternative does not resolve an uncorrected finding. No unknown
method, format, allocator, platform or default is selected by this table.

| Inquiry | Decision question | State | Decision / work owner | Completion or next trigger |
| --- | --- | --- | --- | --- |
| I01 | Which checked artifacts and imported body/data dependencies support fresh-process reuse? | open | 48a/A05 | Complete artifact/receiving contract or documented fresh-check route |
| I02 | Does whole Salsa persistence improve on smaller immutable artifacts? | open | 48a/A05 | Exact-version representation/accumulator/interning decision; no duplicate store |
| I03 | Which immutable checking phases can run in parallel? | open | 48a/A06 | Eligible phase boundary, deterministic results and shared admission |
| I04 | Is a frozen catalog/archive useful after description/bootstrap fixes? | open | 48b/B04 | Representability/startup/retention comparison; custom registry preserved |
| I05 | Which repeated validations truly share one receiving scope? | open | 48b/B03/B05 | Actual binding/lifetime identity, not equal timestamps |
| I06 | Can typed loading remove remaining work without a second validator? | open | 48b/B05 | Same registry/SQL/null/key/reference obligations and selected route |
| I07 | What complete child validity permits reconstruction elision? | open | 48c/C03 | Zero/subset/provider/regime changes cannot hide consequential children |
| I08 | Which arithmetic/instance/value-order reuse preserves ordered meaning? | open | 48c/C06 | Qualified regions, dependency preservation and product sufficiency |
| I09 | Which scalar artifact/cache/default improves cold/warm total behavior? | open | 48c/C07 | Complete compatibility, code/child lifetime and actual total cost |
| I10 | Which remaining provider contractions/reverse actions remove per-output work? | open | 48c/C08 | Actual derivative capability and preserved effects/branches |
| I11 | Does a concrete vector/JIT/device workload justify expansion? | open | 48c/C08 | Eligible bounded prototype or support limit, including transfers/scratch/preparation |
| I12 | Which native KINSOL working policy meets original acceptance? | open | 48d/D01 | Short scaled/shifted roots; native iteration, no custom polish |
| I13 | Which conic working/refinement controls are sound per adapter? | open | 48d/D02 | Magnitude/recovery/roundoff cases; POUNCE judged independently |
| I14 | Which restart/barrier/options policy benefits supplied starts? | open | 48d/D04 | Perturbed cases, effective options and truthful work receipts |
| I15 | Can Ipopt retain structure including maps/scaling safely? | open | 48d/D04 | Compatible/incompatible structure and value-dependent scaling controls |
| I16 | Which additional handles have complete neutral reset? | open | 48d/D04; 48e/E01 | Adapter-specific reset/discard/owner-thread contract |
| I17 | When do grouped tiny/dense/sparse blocks and setup refresh help? | open | 48d/D05 | Root/branch/start-preserving comparisons; no universal cutoff |
| I18 | Should the explicit recycle default/history/damping change? | open | 48d/D06 | Qualified maps/domain cases and complete history/work comparison |
| I19 | Are library Wegstein/Broyden or derivative-capable Newton tears useful? | open | 48d/D06 | Admitted map/derivative/root capabilities and concrete library route |
| I20 | Which eligible sparse-direct/Krylov/native linear backend helps? | open | 48d/D07 | Actual integration, fill/memory/preconditioner/assurance comparison |
| I21 | Which coupled shifted-DAE preconditioner is useful? | open | 48e/E05 | Correct approximation/refresh/singular fallback and original checks |
| I22 | Which IDAS corrector/h0/statistics/reset choices are safe? | open | 48e/E06 | Route-specific local-error/IC/sensitivity/failure behavior |
| I23 | Can a narrower pure-ODE backward route reduce reset/IC work? | open | 48e/E06 | Jumps/history preserved; DAE obligations unchanged |
| I24 | Is LM/TR useful versus existing constrained Ipopt-GN? | open | 48e/E07 | Actual loss/constraint/derivative eligibility and library route |
| I25 | Which forward/adjoint policy fits demand and checkpoint budget? | open | 48e/E07 | Axes/output/sample/event/rank/Jacobian demand and complete work |
| I26 | Are explicit inexact integration controls useful for fitting? | open | 48e/E07 | Objective/gradient consistency and unchanged final physical endpoint |
| I27 | Which shooting starts/optimizer allocations can be reused? | open | 48e/E08 | Anchors/primal/duals/history/maps distinguished across trial/profile changes |
| I28 | What immutable admission can local lanes share? | open | 48f/F04 | Complete basis identity; private values/permissions/provider state |
| I29 | Which durable discovery/claim/wakeup crossings remain unnecessary? | open | 48f/F05 | Existing hints/repair/renewal retained; reconnect-safe authoritative wakeups |
| I30 | Does publication blocking justify a finite commit stage? | open | 48f/F06 | Observed blocking and bounded products/lease/ACK/uncertain outcome ownership |
| I31 | Which producer bounds/growth/stack charges improve admission? | open | 48f/F07 | Cover effects before allocation; no RSS-equating or CPU-held memory waits |
| I32 | Is remaining glue/thread churn better spawned/isolated/pooled? | open | 48f/F08 | Context propagation, affinity, drain and shared-capacity comparison |
| I33 | Does a concrete workload justify isolated/concurrent native execution? | open | 48f/F08 | Qualified capabilities and total startup/IPC/memory/recovery cost |
| I34 | Which bounded transport separates lifecycle from optional history? | open | 48g/G04 | Overflow/exit/stall behavior; finite rings are not inherently lossless |
| I35 | Does an allocator change help after materialization fixes? | open | 48g/G06 | Matched complete-operation latency/memory/native comparison |
| I36 | Which remaining question needs a larger composed/longer run? | open | 48h/H02 | Short results identify unresolved mechanism and minimal follow-up case |

## Complete recommendation coverage

The [coverage appendix](48-recommendation-coverage.md) maps every register row once to
packages/inquiries or explicit non-adoption. Repeated causes share implementation owners.
Omitted diagnoses remain corrected context/preservation, not hidden tasks. External RC rows
map to the audit's twelve decisions. This coordinator owns all finding/inquiry dispositions;
the appendix has no editable status column.

## Verification and completion

Targeted compile/unit checks and immediate deletion of proven replaced paths accompany each
coherent package. Correctness routes explicitly activate validation. H01 owns completeness;
H02 owns short revealing cases and descriptive work/latency comparison; H03 owns selected
assembled Rust/native/Python/managed journeys, affected integration/parity and full continuing
Clippy/final hygiene/manual feature checks under existing policy. H04 owns enduring publication
and subsequent existing-plan reconciliation. An accepted design/ADR is not implementation.

No numerical speed target, fitted exponent, libc share, reservation/RSS ratio or universal
iteration benefit is a project gate. Source-established amplification needs a correct
replacement without a benchmark. Timing-dependent selection needs matched whole-operation
observations. Exploratory science stays near the existing 60-second cooperative budget
with actual drain; longer work needs I36's specific unresolved question. Keep original failed,
interrupted and repaired acceptance distinct.

Completion requires all AF/N obligations corrected, investigations settled with qualified
selected adoptions implemented, real consumers migrated, replacement leftovers removed,
selected assembled checks passing against zero failures and enduring contracts published.
No-adoption closes an alternative, not a defect. Observable further-investigation triggers
do not imply industrial-capacity qualification or blanket platform implementation.

## Current checkpoint

**Document series authored; product execution unstarted.** RC01–RC12 are accepted within
their stated correction/inquiry limits. The eight companions contain 53 packages; this
coordinator owns 22 findings and 36 consolidated inquiries. All delivery state remains
open or conditional. Existing review bodies and other plans remain unchanged.

**Tested, documentation only:** static inspection confirmed exact once-only coverage of
all 162 register rows and historical positions, finding/inquiry/rule counts, packet/outcome
headings, local file/heading links and ten native scope bindings. Bounded independent
integration review identified and corrected optional-reset blocking of parallel windows,
missing numerical-preparation parallelism, compiler-observation scope, successive-study
presentation keys and final/profile assessment ownership. These checks do not qualify
product implementation or claimed performance.

The host RAM startup guard currently blocks admitted documentation validation/publication;
the guard is preserved. The authored sources remain available for execution planning, with
broader document command results to be reported separately. The next product step is a
separately authorized execution-planning request for selected packages, not automatic
implementation or another decision on the already accepted twelve rule boundaries.

## Outcome (recorded after implementation)

### What was built

Not implemented; the execution owner will record actual delivered behavior and evidence.

### A mistake made and corrected

To be recorded from actual execution.

### Deviations from the plan, deliberate

None recorded. Future rule changes follow the decision route above.
