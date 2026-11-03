---
title: "Recycle preparation performance and meaningful convergence"
date: 2026-11-02
doc_role: review
doc_owner: docs/plans/64c-process-authoring-and-first-delivery.md
doc_topics: [design, validation, preparation, numerical-execution]
---

# Recycle preparation performance and meaningful convergence

## Judgment and inspected scope

**Proposed:** correct repeated source-package construction before changing solver defaults
or increasing test allowances. Preparation alone takes substantial time and memory, and
the same local-field selection path was observed in both the recycle context control and
the long-running exchanger rating control. The earlier function-selection correction
does not cover this field path. There is a concrete AP-07/G9 execution-fit problem even
though the scientific controls can pass.

This focused Plan64 improvement review responds to the maintainer's request to investigate
slow recycle tests. It covers C02/C03/C06 and their B03/B04 scientific setup, including
adjacent specialization, compiler reuse, numerical configuration and test scheduling.
The [64c owner](../plans/64c-process-authoring-and-first-delivery.md) and
[64b owner](../plans/64b-native-scientific-intake.md) retain status and accepted scope;
recommendations here do not amend either plan or authorize a broader redesign.
This is not a refreshed whole-codebase audit, formal design acceptance, or qualification
of all process-simulator workloads.

The inspected source baseline is `e8157f799bbfbc4f69e636d060ad18ef50d9c409`, including
the pending explicit POUNCE selection for the three historical recycle controls and
explicit Ipopt selection for all five ordinary delivery cases. Those exact source changes
were included in the captured test binary. Concurrent CLI/skill and library-documentation
changes are outside this assessment. Two independent focused investigations examined
preparation/reuse and numerical configuration; the coordinator collected live profiles
and reconciled their findings.

The assessment uses the selected core 3.4, efficiency heuristics 1.0, test-assessment 1.2
and process-simulator 1.5, through
[the standard](../design_review/design_principles/standard.toml). AP-06/AP-07,
DP-09/DP-10, PS-07–PS-11 and PS-13 are especially relevant. Scientific acceptance remains
independent of the architectural judgment.

## What the measurements establish

**Tested and Measured:** the captured optimized runtime binary was executed directly through
`scripts/pse-env --native=solver,klu,isolation -- /usr/bin/time -v <captured-binary>
--exact workflow::modeling::recycle_tests::reference_recycle_context_reaches_original_transport_obligations
--nocapture`. The binary came from native invocation
`01a19ea9-a0af-72f7-bfa1-2c93082e2127`, with `solver-pounce`, `solver-ipopt`,
`solver-kinsol`, `solver-root-isolation`, `canonical-tests` and `native-study-tests`,
and the correctness route's explicit force-validation. Failure baseline: zero;
one test passed, zero failed. No Cargo compilation occurred during this reproduction.

| Observation | Result and interpretation |
|---|---|
| Context-only recycle test | Test body 84.91 seconds; process wall time 85.00 seconds, user CPU 78.73 seconds, system CPU 1.59 seconds. This test prepares the model and checks transport budgets; it never submits a native solve. |
| Resident memory | Maximum RSS 6,277,600 KiB, approximately 6.0 GiB. |
| Paging | Zero major page faults and zero swaps in this process. These observations do not support disk paging as the explanation for this run. |
| Context CPU sample | A 20-second `perf record -F 49 -g --call-graph fp -p <pid>` window retained 908 samples with no lost samples. Exclusive cycle samples attributed 19.00% to `malloc`, 8.92% to `_int_free`, 6.52% to `_int_malloc` and 9.51% to `memmove`: approximately 44% in these four allocation/copy symbols. Declaration-map lookup accounted for another 10.61%. |
| Context stack | A deeper GDB sample showed destruction of a `CheckedPackage` inside `rewrite_local_field`, at the end of a selection fixed-point iteration. Another sample showed destruction of dataset rows and cells. |
| Rating CPU sample | A separate 20-second window in the already-running ordinary rating control retained 1,018 samples with no lost samples. The same four allocation/copy symbols accounted for approximately 44% of exclusive cycle samples. |
| Rating stack | GDB showed `selected_map -> select_mode -> select_static -> select_field -> rewrite_local_field`, under compiler preparation. At a later observation the process had used about eight CPU minutes and roughly 18.6 GiB RSS. This was a live snapshot, not a completed test duration or measured peak. |

These are diagnostic samples, not isolated performance benchmarks: the context reproduction
ran alongside the ongoing rating control, and GDB briefly paused each sampled process.
Sampling with frame pointers does not provide complete optimized call stacks; GDB supplies
the explicit path attribution. The percentages describe individual windows, not the fraction
of total journey time or a predicted speedup. The context test's elapsed time includes source
admission, preparation, assertions and teardown; it is not a timer around `select_field` alone.

**Tested and Measured:** the coordinated invocation subsequently completed all 36 selected
runtime controls successfully, with zero failures against the zero baseline. Its native
run took 1,177.760 seconds, excluding build/preparation of the test execution fixture. The
invocation-owned `workflow/inventory/group-0/junit.xml` records these individual durations:

| Selected control | Seconds |
|---|---:|
| Historical four-feed context preparation | 91.879 |
| Historical successful initialization | 183.725 |
| Historical overheated initialization | 171.451 |
| Ordinary one-feed successful recycle | 56.895 |
| Ordinary overheated initialization | 136.284 |
| Ordinary exchanger design | 121.803 |
| Ordinary exchanger rating with explicit accepted-design start | 496.817 |
| Ordinary arithmetic closure | 119.666 |
| Canonical intake/publication/reopened consuming case | 202.694 |

The controls intentionally overlap differently under nextest's selected scheduling groups;
individual times must not be summed as the run's critical path or compared as isolated
benchmarks. The rating duration includes the sampling/debugger investigation. Passing
scientific controls do not resolve the measured preparation cost or establish sensible
precision for every resolved physical quantity.

Raw local diagnostics are `/tmp/pse-recycle-review-context.{stdout,stderr,perf.data}`,
`/tmp/pse-recycle-review-context-deep.stacks.txt`,
`/tmp/pse-recycle-review-rating.perf.data` and
`/tmp/pse-recycle-review-rating-deep.stacks.txt`. These workstation scratch files are
not portable retained performance artifacts; the reproduction conditions and observed
values above delimit the claims. No before/after speedup is established by this review.

## Why the tests do different amounts of work

**Interface-checked:** the historical positive and context controls select
`recycle_flash_converges_from_defaults` in
[the recycle campaign](../../packages/reference/campaign/models/recycle-flash.pse).
Its study contains four complete feed cases spanning 0.5–50 mol/s, with purge 0.1.
The ordinary positive case contains one equimolar 1 mol/s feed with purge 0.25.
Comparing their times as equivalent one-case solves would be misleading.

[The recycle tests](../../crates/pse-runtime/src/workflow/modeling/recycle_tests.rs)
also serve distinct purposes: transport-context preparation, successful initialization,
and unsuccessful overheated initialization with no commit. The context control does
`declared_execution` and `prepare_analysis`, with no solver call. Successful initialization
executes a tear stage and then a separate original-specification solve; homotopy is disabled.
`attempts(4)` is a ceiling, not four unconditional retries, and a failed named stage stops.
The original solve and its physical checks remain required.

Source setup in
[the shared scientific helper](../../crates/pse-runtime/src/workflow/modeling/pr_jacobian_tests.rs)
admits the full seed campaign dependency closure afresh in each independent test process.
The ordinary helper adds its data and delivery package. Local controls already use the
local runtime, so they do not need a canonical database merely to solve. Recorded authoring
controls intentionally retain their canonical publication and readback coverage.

## Recommended amendments

<a id="I01"></a>
### I01 — Reach the existing selector cache before constructing field packages

**Observed defect; highest priority.** In
[field specialization](../../crates/pse-modeling/src/specialize/field_fragments.rs),
`rewrite_local_field` repeatedly calls `self.p.select_field` while closing dependencies and
instance references. Each round constructs owned declaration, occurrence, dataset and other
maps, scans selected occurrences against engine states, and destroys the intermediate
package when more dependencies are discovered. After the loop it builds a complete local
recipe. The compiler's `QueryFragments::field_fragment` then discards the supplied package
and asks the existing tracked selector for the same selection again.

[The compiler fragment owner](../../crates/pse-compiler/src/workspace/modeling.rs) and
[shared package selector](../../crates/pse-compiler/src/workspace/modeling/shared_packages.rs)
already provide bounded Salsa-owned reuse. However, that reuse is consulted after the
expensive eager field projection. This reproduces the kind of boundary defect removed for
function calls in `e8157f799`; that correction left the field path unchanged. The sampled
allocation, cloning and destruction are consistent with the source and were observed in
the actual field loop, rather than inferred solely from allocation symbols.

**Proposed:** let each fixed-point round express its existing exact
`StaticPackageSelection::field(source, additional)` request to the shared selector before
materialization. A narrow `Fragments` operation can return the selected immutable package
owner; the compiler resolves it through `select_package` and `read_static_package`, while
standalone specialization uses the same selector directly. The fixed-point dependency
algorithm remains authoritative. The final fragment consumes the same selected owner or
resolves the same already-cached selection. Do not add another cache.

Preserve lexical and missing-lookup dependencies, imported data, source attribution,
zeroed declaration extents used for field identity, physical owners, permissions,
construction limits and cancellation. A guessed dependency set or a key based only on
the field's name would be incorrect. Retain the direct derivation as the same algorithm,
not a second production policy.

Closure evidence should count actual projection construction for repeated identical field
selections, compare tracked and direct executable meaning and refusals, and challenge changes
to values, sources, imports and previously absent lookups. Then rerun the exact context and
successful/unsuccessful recycle controls under unchanged scientific methods. Record elapsed
time and peak memory separately from passing correctness. Relevant principles: AP-07/G9,
DP-09, H5/H7/H12/H14/H16, PS-11.

<a id="I02"></a>
### I02 — Separate dependency discovery from owned projection and recipe reconstruction

**Alignment opportunity, with measured evidence of the current materialization cost.**
I01 avoids reconstruction on matching selector hits, but distinct fields, instances and
successive fixed-point sets can still miss. Selection currently requires an owned
`CheckedPackage` even when the immediate consumer only needs declaration membership and
checked occurrences to discover the next dependency set. The loop also compares each
selected occurrence against all engine states, and all set-alias sources enter discovery
before the later relevance filtering. These are concrete amplification sites; their separate
shares of elapsed time are not measured.

**Proposed:** if substantial cost remains after I01, expose a borrowed or shared selected
membership/occurrence view under the existing checked-package authority. Finish dependency
discovery without repeatedly cloning dataset payloads and type inventories, and materialize
only when a derivation needs owned data. Index the already-known definition/member-to-instance
relationship when it removes repeated scans. Narrow aliases only after proving actual lexical
and forwarded-parameter reachability; deleting conservative dependencies would risk stale hits.

Likewise inspect the cost of rebuilding full field and function recipes before interning:
the recipe includes environments, states, captures, source context and permission premises,
and equality remains complete despite a small hash hint. Reusing an immutable existing owner
can help; reducing equality to the hint cannot. This is a follow-on to the measured path,
not a justification for a new general cache, catalog or dependency engine. Relevant principles:
AP-04/AP-07, H5/H9/H10/H14–H16.

<a id="I03"></a>
### I03 — Reuse valid preparation across the actual lifecycle and keep test scope explicit

**Proposed investigation.** The historical context test and positive initializer independently
admit and prepare the same four-case root, while initialization itself prepares a changed
tear specification and the original specification. Rating intentionally prepares an accepted
design and the requested rating, whose fixed/free roles differ. These are not all value-only
changes, so sharing a complete compiled solve or numerical factor across them is not justified.

Use the existing compiler query observations to distinguish cold admission, original structure,
overlay-specific specialization, evaluator realization, value binding, native execution and
assessment. Reuse unchanged source fragments and valid symbolic work through their existing
owners. Require fresh numerical-state checks, actual start lineage and compatible factor
dependencies. No claim is made here that the entire original/tear artifact is interchangeable.

For routine feedback, a smaller independently meaningful context case can complement the
four-case campaign. Alternatively, some context assertions can inspect the actual prepared
original retained by the positive journey, avoiding an extra preparation where the evidence
claim is unchanged. Preserve a preparation-only contract test and the four-case default-start
scientific coverage; do not silently shrink the tested scenario. Avoid a persistent cross-test
prepared fixture that bypasses source admission or adds another writable authority.

Historical recycle controls currently have the ordinary 360-second runner timeout and do not
join the scientific exclusion group used by ordinary delivery controls. Align their scheduling
with actual scientific ownership if whole-suite measurements show harmful overlap. A runner
timeout should cover the intended bounded operation and preparation, but raising it is not a
performance repair. Group tokens control simultaneous owners, not the CPU width of a process.
Relevant principles: AP-06/AP-07, H2/H6/H12/H20, PS-08/PS-10/PS-11/PS-13 and test-assessment.

<a id="I04"></a>
### I04 — Tune numerical and evaluator configuration from actual work, after preparation

**Interface-checked:** the current dev/test Rust code is already optimized at level 2 and
third-party libraries at level 3. A switch from an assumed unoptimized test build does not
explain the observed delay. [The conformance preparation policy](../../packages/reference/conformance.toml)
selects interpreted evaluation, one symbolic optimizer core, ten Horner iterations and ten
common-expression iterations. These are explicit experiment settings, not an absence of host
CPU availability. The source-projection path is not accelerated by giving the numerical
evaluator more threads.

The profiles allow 3,000 native iterations and the recycle caller allows 600 seconds;
initialization has its shared deadline. These limits do not imply unconditional work.
The conformance manifest's million-cell derivative-check allowance is not consumed by these
`execute_declared` controls, and their native derivative checking is not enabled.
Hessian Auto selects exact curvature when the required second derivatives exist, otherwise
limited-memory. POUNCE uses FERAL; default Ipopt uses sequential MUMPS/METIS. The historical
and ordinary controls now name POUNCE and Ipopt explicitly, preserving the qualified experiment.

**Proposed:** read the already-retained native callback counts/seconds, iteration counts,
evaluation counts, linear-factorization observations and resolved Hessian policy for each
tear/original attempt. Their producers are
[callback diagnostics](../../crates/pse-backend-native/src/callback.rs),
[Ipopt observations](../../crates/pse-backend-native/src/ipopt/sequence.rs) and
[POUNCE execution](../../crates/pse-backend-native/src/pounce.rs). This separates expensive
evaluations from excessive iteration and factorization before selecting a change.

Only then compare the existing compiled-scalar evaluator with interpreted execution over
cold preparation and repeated evaluation, preserving guarded-provider support and derivative
orders. More optimizer iterations or cores need the same complete-lifecycle comparison.
If Hessian work dominates, compare exact and limited-memory modes while retaining the same
physical checks and reporting iteration changes. Ipopt's
[official guidance](https://github.com/coin-or/Ipopt/blob/stable/3.14/doc/special.dox)
describes the curvature tradeoff; it does not establish which wins for this model.
Inspect actual normalization and conditioning before changing scaling or linear solvers.
Do not borrow historical engineering contexts into the ordinary case where their scientific
meaning does not apply. Existing library capabilities, not a bespoke solver iteration,
should own any selected numerical mechanism. Relevant principles: H4/H9/H11/H12,
PS-07/PS-09/PS-10.

<a id="I05"></a>
### I05 — Complete the existing shared engineering-precision interpretation

**Interface-checked and Tested:** the maintainer
clarified that these controls must consume the same programmatic precision system already
used in production. That system exists: [ADR-0163](../adr/0163-contextual-engineering-accuracy.md),
blueprint §16.1 and the existing `NumericalPolicy`/`ResolvedNumericalPolicy` owners separate
conditioning, physical residual allowances and output/decision accuracy. The correction
belongs there and in the shared authored engineering rules, not in per-test tolerance knobs.

The shared [numerical policy](../../packages/reference/domain/models/numerical-policy.pse)
supplies these physical floors. Where an independently admitted characteristic exists, the
resolver takes the larger of the floor and the shared 0.001 fraction of that characteristic.

| Quantity | Shared physical floor |
|---|---:|
| Temperature difference | 0.1 K |
| Pressure difference | 100 Pa |
| Component/total molar flow | 0.001 mol/s |
| Energy transfer/power | 1 W |
| Mole fraction, scalar and reduced Gibbs energy | 0.001 dimensionless |
| Stored energy | 100 J |

**The temperature rule does not request microkelvin convergence.** Native feasibility is
derived from the tightest comparable normalized budget, but the NLP pipeline transports
each row's own physical budget: `native_row_scale = physical_budget / native_feasibility`.
Thus a small scalar native tolerance does not force every temperature residual to that
number of kelvins. Ipopt retains separate component budgets and an overall tolerance equal
to their maximum; POUNCE uses their conservative minimum because its overall tolerance also
limits a scaled dual-error floor. Altering POUNCE's mapping requires validating its actual
convergence contract. Original residual/bound checks still use physical allowances.

**Observed coverage gap in the inspected rule set:** no shared MolarEnthalpy/enthalpy-difference
rule appears alongside those floors. The resolver permits a recorded canonical fallback when
no meaningful allowance or engineering characteristic is selected; strict engineering
completeness is false by default. This fallback is `0.001` of one canonical unit, rather than
a decision-based physical resolution. In a molar-enthalpy equation it can mean 0.001 J/mol.
[Equilibrium states](../../packages/reference/thermodynamics/models/equilibrium.pse) bind
enthalpy with a 50,000 J/mol conditioning nominal; single-phase states use a 10,000 J/mol
nominal. Absent another selected requirement, those imply normalized ratios of 2e-8 and
1e-7 before equation-specific coordinate scaling.

**Tested:** a diagnostic rerun of the same historical context test, invocation
`01a19ec1-7abb-7483-afd3-65d161e977ab`, captured the actual prepared original's
`solve.accuracy()` and smallest normalized variable/row targets. One test passed against
the zero failure baseline. The actual native feasibility was **1.5484708139343546e-8**,
with stationarity and complementarity each **0.001**. The 24 smallest printed targets
were enthalpy-binding rows, each with **0.001 J/mol** physical budget, coordinate scale
**64,579.84167355405 J/mol**, and `canonical_fallback=true`. The complete numerical
target inventory contained 1,360 entries, of which 184 recorded canonical fallback;
these totals include roles other than active native rows and do not mean 184 independent
solver constraints. The invocation's `workflow/run-0.stderr` retains the original output.
Temporary diagnostic printing was removed after capture. This confirms the fallback is
active in the historical original; it is not yet a distribution of every ordinary case,
tear overlay or native attempt, and no timing benefit from changing it has been measured.

**Proposed:** extend the enumeration to the other selected variables/rows and their resolved allowance,
coordinate scale, rule, characteristic, fallback and resulting native accuracy. Complete
missing engineering coverage at the existing shared owner, using full quantity meaning and
an enthalpy-difference scale grounded in heat capacity, useful temperature resolution and
the relevant energy/flow decision. Do not use the arbitrary absolute enthalpy datum as an
engineering scale. Audit other canonical fallbacks in the selected ordinary cases similarly;
once their meaningful context is complete, consider strict engineering completeness for
these production-representative controls. No chosen replacement magnitude or performance
benefit is established here.

An ordinary shared physical allowance is a starting engineering resolution, not proof that
no possible decision changes. The existing optional accuracy-goal path supplies output
resolution and quantitative decision criteria where that distinction matters. No goals
means NotRequested, not decision invariance. Retain this system rather than requiring every
ordinary solve to perform a paired certification campaign.

Small smoothing widths, derivative perturbations, homotopy step fractions, exact fixed
inputs, arithmetic-identity assertions, integrality and hard-domain protection have different
meanings. They must not be mistaken for a demanded temperature output accuracy or loosened
as a group. Relevant principles: AP-04/AP-07, H6/H12/H20, PS-01/PS-07/PS-10/PS-11.

## Execution implications and limits

I01 is the first concrete amendment to consider within C02. I02 should follow remaining
profiles rather than becoming a prerequisite rewrite. I03 can reduce duplicated assurance
work while preserving its claims. I04 is a conditional experiment route, not a recommendation
to change backend, curvature or evaluator defaults now. I05 completes the already selected
shared precision paradigm through meaningful context. None requires relaxing original
energy/material closure, applicability, hard domains, source provenance or failure semantics.

The existing function-selection fix is useful and has targeted correctness/projection-count
evidence; it is not evidence that all source projection has become cheap. This review supplies
no measured improvement, full native qualification, independent-reference comparison or new
completion claim. H02/H03 and wider Plan64 qualification retain their existing owners.
