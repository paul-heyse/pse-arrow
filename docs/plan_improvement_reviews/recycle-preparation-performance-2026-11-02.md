---
title: "Recycle preparation performance and meaningful convergence"
date: 2026-11-02
doc_role: review
doc_owner: docs/plans/64c-process-authoring-and-first-delivery.md
doc_topics: [design, validation, preparation, numerical-execution]
---

# Recycle preparation performance and meaningful convergence

## Judgment and inspected scope

The [follow-up investigation](#follow-up-investigation-source-scope-and-retained-evidence)
extends the original assessment below. Its disposable field-selection prototype passed the
preparation control and showed a substantially shorter observed test body; complete requested
query recording nevertheless lost records. The current recommendation remains preparation
first, with explicit capture integration and coverage repair before stronger numerical claims.
All prototype changes are investigative and do not constitute production adoption.

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
values above delimit the claims. The initial assessment established no before/after comparison.

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

## Initial assessment: execution implications and limits

I01 is the first concrete amendment to consider within C02. I02 should follow remaining
profiles rather than becoming a prerequisite rewrite. I03 can reduce duplicated assurance
work while preserving its claims. I04 is a conditional experiment route, not a recommendation
to change backend, curvature or evaluator defaults now. I05 completes the already selected
shared precision paradigm through meaningful context. None requires relaxing original
energy/material closure, applicability, hard domains, source provenance or failure semantics.

The existing function-selection fix is useful and has targeted correctness/projection-count
evidence; it is not evidence that all source projection has become cheap. The initial assessment supplied
no measured improvement, full native qualification, independent-reference comparison or new
completion claim. H02/H03 and wider Plan64 qualification retain their existing owners.

## Follow-up investigation: source scope and retained evidence

The follow-up began at `7db37f4ac`, preserving unrelated CLI and library-documentation
changes. It follows the source/admission, compiler, evaluation, initialization, native,
assessment, observation and retained-result paths consumed by these cases. Core 3.4,
efficiency heuristics 1.0, test-assessment 1.2 and process-simulator 1.5 remain selected.
Source-supported amplification and measured bottlenecks are distinguished below. Neither
the historical four-feed study nor the ordinary one-feed case is silently substituted for
the other. Production adoption and Plan64 package completion remain separately owned.

### Evidence recovered, and evidence that did not survive

**Tested:** the original native 36-control receipt, six-test native-diagnostics receipt,
four-test product-observation receipt and precision diagnostic remain readable. Their
JUnit reports establish individual outcomes and whole-test duration. They do not contain
the successful local scientific reports' full native metrics, options or callback payloads.
The raw perf samples and GDB stacks remain available, with their original sampling limits.

The historical exchanger preservation command additionally returned
`passed=false complete=false coverage=selected checks=1818 fixtures=1 {Inconclusive:1}`.
No report directory was selected. The aggregate reducer ranks Inconclusive above Failed;
this does not establish 1817 passing checks, one failed native solve, or global infeasibility.
The exact inconclusive check is unavailable in retained output. A selected rerun must retain
the existing conformance report export, not infer its cause from the aggregate status.

| Owner/population | Actual available evidence | Interpretation limit |
|---|---|---|
| Invocation/JUnit | Original executable selection, streams, test outcomes and times | Includes setup/teardown; not a native-solve timer |
| Compiler observations | Query execution/validation/wait/discard facts; selected query records; projection starts | Query miss cause and matching wait-exit duration can be unavailable; projection starts are not completed bytes or attributed selector hits |
| Runtime preparation/cache owners | Frontier/basis acquisition, actual producers/waiters, reuse, entries and known extent | Process totals and inclusive intervals overlap; reserved bytes, known retained bytes and RSS differ |
| Native report | Metrics, options, starts, native observations, candidate and original quality | Local successful direct outcomes are not automatically archived before destruction |
| Canonical results/progress | Protected typed result relations and exact attempt progress | B03's selected exchanger archive is not an archive of every local recycle run |
| Inner implicit roots | Source-associated KINSOL attempt observations and provider work | Inner SolveReport is consumed for its candidate; do not presume a separate durable inner-report archive |
| Selected source recording | Original coherent records, family sequences, activation and loss accounting | Requires an actual cooperating entrypoint and independent finalization |
| Offline analysis/export | Frozen interpretation, bounded detail pages, typed IPC and selected packs | A SQL specification is not executed analysis; a bounded explanation is not the whole recording |

Recoverable raw follow-up artifacts are under the workstation directory
`/home/paul/.local/state/pse-arrow/recycle-followup-20261102`, with post-clean recovery
attempts under its `-restored` sibling. These are diagnostic artifacts, not new scientific
authority or a portable result store. The directories retain original stdout/stderr,
sampled checkpoint streams, derived metric selections, outcomes and numerical probe JSON.
Sampled checkpoints are explicitly not complete source-event recordings.

### Disposable experiment and collection outcomes

**Tested and Measured:** the capture-only context baseline, native invocation
`01a19ee2-d4f1-7509-b768-13465ba7a70c`, used the same six features and correctness
validation as the original control. Its only source amendment wrapped the unchanged
preparation body in an opt-in recorder at construction. One selected test passed against
the zero failure baseline: nextest duration 80.282 s, original body 79.985 s. The post-clean
test build took 6 min 4 s and is excluded from those body times.

The three-file I01 prototype then routed each unchanged exact field selector through the
existing tracked selected-package owner before construction. Native invocation
`01a19ee9-e6f8-73cd-a283-d4f3032596f5` passed the same preparation control: nextest
duration 26.277 s, original body 25.951 s. Neither numerical policy, source scenario nor
scientific assertion changed. This is a witnessed promising comparison, not a stable
threefold speedup claim: executions were not isolated repeats, other sampled controls and
build work overlapped, and recorder losses/perturbation differed. No paired peak-memory
benefit is established.

A 250 ms checkpoint sampler retained 27 changed images from the I01 capture. Its latest
observed compiler metrics included 1,093 source projections, specialization 12.734 s,
plan-modeling 16.633 s and one rotation/rebuild. These cumulative metrics are explicitly
**partial**; 1,093 cannot be presented as the complete replacement for the baseline's
nonpartial 7,090, nor can absent remainder be treated as zero. Query-family event counts
are a different population from owned projection constructions.

| Requested compiler-query recording | Science | Terminal collection | Issued / acknowledged / dropped |
|---|---|---|---:|
| Capture-only baseline; complete, stride one, 16 MiB working, 256 MiB output | Passed | Incomplete; OwnerFinished | 25,784 / 10,621 / 15,163 |
| I01 field prototype; same selection | Passed | Incomplete; OwnerFinished | 26,295 / 11,098 / 15,197 |
| I01 field prototype; only working allowance raised to 128 MiB | Passed; body 24.823 s | Incomplete; OwnerFinished | 26,295 / 24,748 / 1,547 |
| Capture-only binary; deliberately 1-byte output allowance | Passed; body 89.716 s | Incomplete; WriterFailed; output exhausted | 0 / 0 / 0 before activation |

The first two captures actually activated four sources; their issued and observed counts
matched, with zero sampled-out, unacknowledged and metadata-dropped counts, no unavailable
scope and no writer error. The retained streams were only about 8 MB, below the 256 MiB
output allowance. This establishes real collection loss despite accepted requests and
successful science; increasing disk space or declaring the request Complete does not make
the data complete. The 1-byte control independently confirms that diagnostic failure does
not change the original scientific result. It does not exercise a native callback or prove
successful whole-solve handoff. Full JSONL, acknowledgements, terminal frames, requests and
patches are retained with the local investigation artifacts.

The source computes a 64 KiB per-record retained-size threshold in both 16 and 128 MiB
conditions, but queue capacity grows from 60 to 508 frames. Raising only that diagnostic
allowance substantially reduced losses with identical issuance. This supports queue pressure
as a consequential mechanism; it still did not produce complete capture. The family counter
merges oversized-record rejection and failed queue reservation (Full/Closed); retained small
JSON frames cannot prove the size of records that were rejected. Add distinct reason counts
at this existing owner before claiming the remaining cause. Metadata/try-lock failures have
different coverage fields; compiler query callback contention can instead lose updates before
detail issuance. Inspect those operation-level dropped updates independently of detail's
`source_partial=false`. Increasing working allowance is a diagnostic probe, not a selected
new default or a correction for writer throughput.

**Tested:** with I01 still applied, native invocation
`01a19eee-ed45-72a5-9fe0-1c15be1e8646` ran eight selected `pse-compiler --lib`
controls through `scripts/pse test`, explicit correctness validation: eight passed, zero
failed against the zero baseline. Selection covered prelude locality, lazy ordinary field
obligations, inactive aliases/membership changes, actual instances/forwarded sources,
temporal replicas, transient specialization cancellation, receiving declaration budget and
cancelled context refresh. These existing controls support semantic feasibility; they do
not establish all proposed eviction, key-growth or quantitative projection-count properties.

**Tested and Measured:** the captured I01 runtime binary additionally ran the three original
historical controls through `scripts/pse-env --native=solver,klu,isolation -- /usr/bin/time -v
<binary> --exact <test> --nocapture`, with no selected recorder. Three passed, zero failed
against the zero baseline. A 500 ms checkpoint sampler and concurrent work remained active.

| Historical control | Original body, s / max RSS, KiB | I01 body, s / max RSS, KiB |
|---|---:|---:|
| Four-feed context | 63.78 / 6,276,432 | 25.98 / 7,376,092 |
| Successful initialization | 127.63 / 11,410,032 | 47.59 / 13,629,588 |
| Overheated rejection without commit | 197.13 / 7,288,404 | 71.96 / 18,815,712 |

These are diagnostic observations from different executions, not paired benchmark estimates.
In particular the larger prototype peaks, especially the rejection case, require investigation
before adoption; no memory improvement is claimed. Retaining intermediate exact selectors can
trade construction for retained working set. Record ingredient ownership and reset behavior
alongside repeat timings rather than accepting elapsed time alone. Boundary wall time can
also include substantial launch/provider/admission delay: for the original ordinary positive
it was 267.521 s while the body was 61.67 s; for the prototype historical positive it was
138.925 s while the body was 47.59 s. The captured command and inner `/usr/bin/time` retain
both clocks; the difference is not native solve time and its precise owner is not established.

After acquiring the binaries and results, all four temporary source files were restored
byte-for-byte to their verified preimages. The recorder wrapper and I01 remain retained
patches, not adopted implementation. No changed production code is part of this review update.

### What the historical positive actually did numerically

**Measured from sampled records:** the fresh historical positive took 129.742 s, with
129 changed checkpoint images. Two preparation owners retained approximately 58.069 s
(nonpartial cumulative phase) and 57.750 s (partial). One completed native-session owner,
sequence 2285, spans 2.288 s and reports about 2.244 s native execution. Its two successful
`native_owner_return.v1` records are original-oracle and coordinate-transport boundaries
of that **one** session; they are not two independent solves or additive time. Another
session is visible without a retained terminal record. No full disjoint wall partition or
exact tear/original association follows from chronology alone.

The completed session's sampled original-oracle callback counters report 732 constraint
calls/0.894 s, 249 Jacobians/0.396 s and 245 Hessians/0.474 s, each partial. Transported
callback intervals overlap these original boundaries. The 68 retained evaluator-demand
batches belong to six producer generations: those counters reset **per batch**, unlike
the separate cumulative operation metrics. Deduplicate by actual source identity before
aggregating; never call the batch sum a lifetime callback or nonlinear iteration count.

The 64 retained nonlinear-iteration records describe two incomplete populations. Their
highest retained source sequences contain iterations 164 and 189, with native primal
errors about 1.225e-9 and 0.01327 respectively; the latter has native dual error about
1.083e21. These are intermediate samples in native coordinates, not final physical closure
errors. Missing prefixes, internal sequence positions and suffixes preclude final residual,
iteration-total or restoration-time conclusions. Successful return does not retroactively
make the trajectory complete. The source-hashed `derived-native-analysis.json` preserves
the exact groups and the interpretation in `native-sampled-analysis.md`.

These actual observations reinforce preparation priority. They do not show that canonical
enthalpy fallback caused the observed restoration or dual growth, or that solver tuning
would remove minute-scale preparation. I05 remains a physical-policy concern on its own
merits. Final original assessment, exact attempt/session links, full native summaries and
factor/root detail still require deliberate retention at their existing owners.

### Fresh preparation evidence

**Tested and Measured:** the captured six-feature context control passed with zero failures
while a 500 ms observer retained 64 changed checkpoints (19,956,423 bytes). Process wall
time including boundary activation was 78.818 s. One observed compiler operation recorded
7,090 source projections; `modeling.specialization` retained 53.005 s, its enclosing
`compiler.query.specialized` 53.018 s, and `compiler.plan_modeling` 56.416 s. These are
inclusive, overlapping intervals, not additive costs or exclusive CPU measurements.
The tracked field-fragment phase accounted for only 2.842 s: timing only the cached
derivation would miss much of the eager caller-side work identified by the earlier stacks.

The same operation reported one retention-limit rotation and one workspace rebuild.
That observation establishes occurrence, not that the rotation caused all repeated work.
Across the sampled context stream, 264 operation identities appeared; observed per-operation
coverage maxima included 28 dropped updates, 22,592 dropped detail records and 110 refused
items. These are maxima of reported counters, not a deduplicated sum of all lost events.
The historical positive control also passed, retaining 129 checkpoint images and 48,461,981
bytes, with additional visible coverage loss. Neither stream can support complete numerical
trajectory or zero-work claims for absent fields.

During the first fresh matrix, the maintainer ran `cargo clean`. Six subsequent boundary
launches refused with exit 125 because the prepared host was gone; no scientific test
executed in those six attempts. Their refusals remain retained. Free space was observed
to increase from about 2 GiB to 326 GiB. The native host was explicitly rebuilt, and
`just ready` passed. These preparation effects are separate from scientific timing;
the clean is not evidence of a solver or model speedup.

**Tested and Measured:** after explicit host recovery, the six refused local controls were
executed from the unchanged original captured binary using the same native boundary,
`/usr/bin/time -v`, exact-test selection and 500 ms observer. Together with the first two
fresh controls, all eight original local cases passed, zero failed against the zero baseline.
Their fresh body and outer launch-wall times remain distinct:

| Original local control | Test body, s | Outer wall, s | Changed checkpoint images |
|---|---:|---:|---:|
| Historical context | 63.78 | 78.818 | 64 |
| Historical successful initialization | 127.63 | 129.742 | 129 |
| Historical overheated rejection | 197.13 | 236.152 | 197 |
| Ordinary successful recycle | 61.67 | 267.521 | 62 |
| Ordinary overheated rejection | 165.48 | 167.202 | 165 |
| Ordinary exchanger design | 97.69 | 111.970 | 98 |
| Ordinary exchanger rating including design predecessor | 527.65 | 529.294 | 475 |
| Ordinary arithmetic closure | 92.80 | 97.961 | 94 |

The rating observer reached its 128 MiB archive bound: 134,094,672 retained bytes,
103 reported output-limit encounters, last retained observer time about 476.360 s.
The scientific test continued and passed; its later checkpoint suffix is unavailable.
Other recovered cases reported no sampler I/O/output-limit errors, which does not remove
their underlying hub curtailment or make 500 ms sampling complete. All raw refusal attempts
remain retained alongside recoveries. Runs overlapped build/diagnostic work and cannot be
used as isolated case-to-case benchmarks.

The rating retained a nonpartial 218.287 s `modeling.prepare` observation. Ordinary positive
retained a nonpartial 35.079 s preparation and 33.816 s specialization observation. Design
and arithmetic also retained long compiler-preparation intervals, though partial. A
`native.worker` or `native.join.wait` interval can include that compiler work; it is not a
timer around the native solver. These corroborate preparation investigation across the
consumed paths without supplying a complete final native breakdown for each case.

### Caloric and energy-flow prototype: refine I05 rather than raise a global floor

**Tested:** an offline probe read the exact frozen FeOS JSON and Perry Parquet, retained
their SHA256 identities, normalized the FeOS kmol basis and evaluated the existing DIPPR100
polynomial at five temperatures. Its 10 phase/temperature records and 30 flow projections
are retained in `numerical-probe.json`. It is a fixed equimolar, local single-phase
conditioning calculation, not a flash solve, output-error certificate or accepted default.
FeOS unknown support and Perry benzene extrapolation above 353.24 K remain explicit.

| At 368 K | Cp, J/(mol K) | Local T equivalent of 0.001 J/mol | Cp × 0.1 K, J/mol |
|---|---:|---:|---:|
| Ideal gas | 115.3299043 | 8.67078e-6 K | 11.53299 |
| Liquid polynomial | 166.50315072 | 6.00589e-6 K | 16.65032 |

This substantiates the local excessive-digit concern. It also rejects the tempting blanket
remedy: for an independently selected flow F, a 1 W energy allowance corresponds locally
to `1/F J/mol`. At 0.5, 1 and 50 mol/s this is 2, 1 and 0.02 J/mol; at the authored
500 mol/s circulation capacity it is 0.002 J/mol. A global 11–17 J/mol allowance could
represent thousands of watts at that capacity. Capacity is not actual solved circulation,
and contextual power allowances may exceed their 1 W floor. Resolve the actual consuming
requirements before deriving a shared allowance; neither this projection nor a conditioning
nominal replaces coupled sensitivity or original physical acceptance.

**Refined I05 recommendation:** retain the existing resolver and full quantity semantics;
connect independently authored caloric differences, flow/energy requirements and selected
output/decision goals. Inspect the tightest actual consumers and preserve affine-datum and
unit invariance. Audit all selected canonical fallbacks, then require meaningful context where
the ordinary case promises it. Do not assign one convenient molar-enthalpy constant globally.

Native row-budget transport preserves each row's own physical allowance. Variable scales,
however, are unchanged by that row transport, while native feasibility remains one conservative
minimum. A small enthalpy budget can therefore tighten native bound convergence for another
coordinate. Original bound enforcement often keeps coordinates inside already, so this is a
potential stopping coupling, not proof of actual microkelvin iteration demand. Inspect which
criterion remains limiting. Hard mathematical domains and fixed specifications remain separate.

### Additional recommendations from the complete consumed path

<a id="I06"></a>
#### I06 — Activate collection at the real construction root and retain successful outcomes

**Observed integration gap; first prerequisite for deeper numerical claims.** The selected
recycle/ordinary entrypoints at the inspected baseline did not call
`source_recording_from_environment`; the existing protocol is implemented and tested but
setting its environment variable alone installs no recorder. The disposable cooperating
context wrapper above demonstrates the required composition, not production integration.
Select it before Runtime/source/preparation construction, propagate its context through the
whole async future, and finalize its owner independently of science. An async `scope` is
required; a thread-local enter guard must not cross await. Retain successful and unsuccessful
original reports while alive. The direct `execute_declared` result differs from `Arc<RunResult>`:
either export its existing typed factual projections or deliberately use the public
`prepare_operation -> PreparedOperation::start -> RunHandle` journey for a separately named
experiment. Do not silently time a different route or fabricate a successful handoff.
AP-05/AP-06/AP-07, H6/H18/H26; closure requires actual activation, terminal coverage, original
science outcome and offline readability for both successful and curtailed collection.

<a id="I07"></a>
#### I07 — Keep complete recording, recent history and bounded analysis distinct

**Source-established contract mismatch risk.** Ordinary history defaults to 16 records per
producer and at most 256; selected detail receives records before this retention layer.
Delegated CPU/timeline capture is capped at 20 s, whereas preconstruction source detail
supports a selected interval up to an hour. Complete means the selected interval, not the
entire later journey: `WindowElapsed` can be complete for a short interval. Source detail also
has independent 256-entry metadata trackers; merely increasing byte budgets cannot remove
all coverage limits. Handoff can emit more than the frozen reader's 64 MiB aggregate bound,
and an analysis explanation can inspect at most 4096 source items. Preserve full raw members,
use cursor pages and bounded operation/attempt/window selections, and never embed the whole
history repeatedly into a single frozen document. H5/H8/H10/H17/H26; closure tests must
challenge expiry, metadata/queue loss, complete prefixes and records beyond the explanation cap.

Local complete detail or accepted ACK also does not cover managed children. The existing
managed group grant is separate and once-only; each actual attempt needs representable slots,
an admitted share, child references and retained refusals. Queued children inherit the
original monotonic expiry rather than restarting the window. A canonical managed campaign
must reconcile these facts after science drains; a successful local context wrapper does
not establish that campaign coverage.

<a id="I08"></a>
#### I08 — Use selective inspection without constructing editing inventories

**Source-grounded avoidable-work opportunity.** `ModelingPackage::declarations()` reaches
the full declaration inventory and delta-parent products, including reference/data products
and parent copies. Existing declaration pages/windows deliberately avoid those products.
Move consumers needing only case identification or a selected page to those existing reads;
retain complete edit inventories where the caller really requires them. Separately distinguish
encoded source decoding, carried metadata cloning and declaration/dataset payload cloning:
the generic 32× source-decode reservation does not apply uniformly to every typed declaration.
Measure those owners before changing conservative admission. AP-06/AP-07, H5/H7/H16;
closure compares returned identity, absence/refusal and source-owner lifetime, with actual
work counts rather than assuming a smaller displayed page did less work.

Also decouple legitimate **full** ordered reads from edit preparation: let
`declaration_inventory` consume the existing ordered-read owner and reserve its actual
requested output; construct delta/dependency `Products` at the edit boundary. The present
cold path constructs those products and copies a parent, and the warm path still copies
that parent. Selective consumer changes alone leave this responsibility coupling intact.
Preserve complete order, exact source/provenance, leases, corrupt/duplicate/absent-input
refusals, cancellation and subsequent real edit/CAS/publication behavior. Do not implement
the full read by looping a public page if that repeatedly decodes the whole ingress frame.
This coherent owner correction is high priority independent of its unmeasured exclusive cost.

<a id="I09"></a>
#### I09 — Include key construction, source-history lookup and retention rotation in reuse

**Observed rotation plus source-grounded amplification leads.** Complete fragment recipes
are assembled before interning; static-package hashing uses a small bucket hint followed by
complete equality. Selected admissions scan retained candidates and validate positive and
absent namespace premises. Compiler retention performs an exclusive census and can rotate
the entire generation when configured bounds are exceeded. Inspect actual key/equality work,
candidate probes, known versus foreign bytes, rotation cause and subsequent reconstruction.
Prefer shared immutable premises, suitable lookup/indexing and accurate shared-owner extent
accounting where justified. Preserve full equality, negative dependencies and memory admission;
raising caps or replacing complete equality with a hash is not the correction. H12–H17/H23;
closure includes A/B/A edits, increasing source history, unrelated changes, and retained
old products across a rotation.

The inspected reference helper overrides worker capacity/jobs, leaving the Math workspace
default at 4 GiB. Its compiler input and retained-byte bounds are each
`min(default 2 GiB, workspace_bytes/2)`, hence 2 GiB here; retained entries are 131,072,
live-family LRU 256 and granular LRU 4,096. The larger test pool does not enlarge this
workspace. These semantic owner bounds are separate from the unbounded host workload mode.
Rotation preserves the admitted revision, service-owned body retention and cancellation,
while discarding local selectors, fragments, preludes and memo state. Retain the actual
triggering dimension, bounds, generation and bounded ingredient census **before** reset,
using the existing exclusive census. Its known heap estimates, unknown external heaps,
reservations, escaped products and RSS are different facts. Shared-owner estimates warrant
inspection, not an unsupported assertion of double charging. The prototype still observed
a rotation; neither its occurrence nor that of the baseline identifies the cause.

For admission-history investigations, record candidates probed, positive/absent premise
comparisons and lock intervals. The newest-first vector and an output limit of 32 rows
do not impose a 32-candidate scan bound. Retain latest-first behavior and exact missing-member
validation. For interning, a stronger once-computed bucket hint is an alternative only if
deep equality is measured material; retain complete semantic equality and physical custody.

<a id="I10"></a>
#### I10 — Make diagnostic work proportionate and retain once for repeated analysis

**Source-grounded opportunity; overhead benefit unmeasured.** Selected source publication
constructs/clones records and performs queue/sequence bookkeeping; the writer allocates JSON
and flushes every frame, with a 5 ms idle poll. Context handoff serializes and decodes copies;
analysis admits/indexes the supplied population, not merely its displayed source-item limit.
Compare ordinary observation, selected detail and boundary CPU clocks separately. If material,
reuse bounded encoding buffers, batch complete-frame flushes with truthful readable-prefix
publication, and use existing channel wakeups. Preserve nonblocking scientific callbacks,
finite ownership, loss reporting and stop/drain. Export once and query retained typed tables
rather than recopying the entire history for each view. AP-07, H16/H17/H20/H26; closure
requires recorder perturbation measurements plus queue-pressure, stop and partial-write tests.

<a id="I11"></a>
#### I11 — Consult prelude reuse before constructing its premises

**Additional source-grounded pre-hit construction defect; outside the I01 prototype.**
`Engine` constructs `PreludePremises::select` before `Fragments::instance_prelude`.
[Prelude selection](../../crates/pse-modeling/src/specialize/prelude.rs) builds member and
conditional maps and selects an owned package; the compiler derives from the supplied
premises and does not select it again. Consequently repeated prelude hits still pay that
first projection and retain the full selected package in their key. This explains why
field reuse alone need not make source projection constant across repeated actual instances.

**Proposed:** express the exact existing static selector plus complete `PreludeBindings`
at the current tracked owner before building `PreludePremises`, and construct maps within
its producer. Keep direct standalone construction. Preserve selected definition, namespaces,
conditional membership, inherited actual environment, facts, permissions and member limits;
an instance-name-only key would be wrong. Compare projection/map construction counts and
retained key extent, with shadowing, missing/inactive members, inherited implicit contexts,
TestOnly refusal and same-definition/different-binding controls. This fits the same compiler
preparation correction and precedes speculative hash tuning. AP-07, DP-09, H5/H12/H14/H16.

<a id="I12"></a>
#### I12 — Carry already-qualified selection through the same invocation

**Conditional source-grounded opportunity.** `CheckedSelection::new` performs receiving
physical/current qualification and can clone declarations into an Arrow builder for scope
validation. Its existing `carries` premises include source, physical/provider/registry and
same-invocation custody. Inspect whether adjacent preparation layers reconstruct a selection
that they could carry unchanged. Count qualification/build/clone work at those boundaries,
then preserve the existing qualified owner where its exact premises remain true.
Cross-invocation admission, changed source or physical context still requires current checks.
Countercontrols must retain explicit force-validation, provenance/permission refusal,
changed context and cancellation. No cross-invocation bypass or new global memo is selected;
benefit is unmeasured. AP-06/AP-07, H6/H12/H16/H20.

### Refined investigation and adoption order

I01's cache-before-materialization mechanism remains the first concrete preparation change;
I02 must account separately for fixed-point discovery, occurrence/state scans, alias expansion,
recipe creation, interning and destruction. I03 should reuse immutable preparation through
its actual validity boundaries, not merge tear/original or design/rating solves indiscriminately.
Inspect current receiving qualification and retention before adding another cache.

For I02's first miss-side probe, use the existing `CheckedPackage::occurrences_of`
declaration index and actual definition/member-to-instance relationship instead of comparing
every selected occurrence to every state. An actual declaration instantiated twice still
needs two actual contexts. Preserve temporal derivative/integral closure and forwarded alias
reachability. A later borrowed-membership view should factor the **same** selection traversal,
not invent a lighter competing dependency walker; measure parent lifetime because a small
view can retain a much larger parent than a compact projection. Recipe sharing follows actual
capture/equality evidence and retains unsuccessful lexical lookups and shadowing.

For I03, separate same-source structural reuse, changed numerical point, changed fixed/free
roles, tear overlay, original A/B/A restoration and accepted-design-to-rating. Reuse compatible
frontiers/bodies/programs through the existing single-flight owners; keep waiter cancellation,
actual starts, factor dependencies and original physical acceptance independent. Helper
consolidation does not share mutable preparation across independent nextest processes.

For I04, recover actual callback populations and exclusive versus inclusive timing before
choosing a native setting. Preserve POUNCE's native factor call/parent-call, factor/refinement
and multipart identities; retain opaque MUMPS internals as unavailable. Inner-root work,
derivative demand and provider reconstruction can dominate without many outer iterations.
The native terminal statistics and bounded trajectory are complementary, not substitutes.
Only then compare evaluator realization, optimization effort, curvature and linear settings
at fixed scientifically meaningful requirements. Retain failed alternatives and classify
each original result independently; output rounding is not convergence policy.

Build/fixture preparation and retained executable copies also contribute to developer feedback
cost. The observed host loss after clean demonstrates that preparation must remain explicit
and separately measured. Reuse the selected executable/profile during a campaign, inspect
feature-induced rebuilds and artifact lifetimes, and preserve scientific/execution custody
when evaluating retention improvements. A test-body speedup alone does not establish a faster
edit/build/test cycle. These are H24/H26/H27 lifecycle opportunities, not authority for
automatic cleanup, a new build orchestrator or removing required validation.

### Evidence to retain for the next adopted correction

Use the existing diagnostic, result and native report owners. The current investigation
retained sampled/checkpoint and selected-query populations, not an exhaustive successful
simulation archive. The following specifies the missing joins and useful measurements;
it does not assert that all fields below were captured here.

| Population | Retain and investigate | Avoided inference |
|---|---|---|
| Experiment identity | Exact source revision and prototype patch, executable identity/features/profile, provider/method, force-validation, input/data identities, resolved policy and cold/warm state | A method label alone does not identify an equivalent experiment |
| Complete journey clocks | Build/fixture preparation, boundary admission, test body, admission/preparation, realization, native work, original assessment, persistence/export and teardown; process CPU/RSS separately | Overlapping phases do not add to a wall partition; RSS is not retained logical extent |
| Compiler/preparation work | Projection starts plus completed rows/bytes if instrumented, fixed-point rounds/occurrence-state visits, recipes/captures/equality, query execute/validate/wait/discard, producers/waiters, exact qualification and source-history probes | Cache lookup or output-page count does not measure work avoided |
| Retention transitions | Existing exclusive census, bounds/trigger, ingredient/generation distribution, evictions/rebuild, escaped owners and subsequent production | One reset or a large reservation is not its causal cost or measured heap allocation |
| Attempt and numerical meaning | Original/tear/design/rating identities, actual start lineage, quantity/instance IDs, units, allowance source, characteristic, fallback, coordinate/row scale, effective options and exact final original checks | Native scaled residuals and tiny policy scalars are not temperature error or decision invariance |
| Native and implicit work | Terminal scalar statistics plus bounded trajectory, callback/evaluator demand, KINSOL roots/trials/rejections, POUNCE factor/refinement call trees and parts where exposed | Two boundary roles do not mean two solves; unexposed MUMPS internals remain unavailable |
| Recording and analysis | Accepted/refused selection, actual activation, full source tuple, issued/observed/queued/acknowledged/lost populations, end reason, valid prefix and writer settlement; managed grants/children separately | Accepted Complete is not obtained completeness, nor is one explanation page the entire population |

Correlate process-instance operation, producer, generation, sequence, family sequence and
source-field ordinal, retaining existing run/attempt/step and POUNCE parent-call/multipart
identities. Reject conflicting duplicates through the existing analysis owner. Preserve raw
records before deriving typed tables or bounded question selections, and retain the exact
selection/query with derived results. Separate reset-per-batch counts from cumulative metrics;
report sampled intervals and unavailable fields explicitly. Keep both diagnostic and science
outcomes, including failed alternatives and curtailed/refused collection. Export/interpret
after the measured science interval unless export itself is the question.

The local `investigation/` artifact directory retains the independent source reports,
numerical and analysis scripts, exact prototype images/patches, recording requests/ACKs,
JSONL streams and native workflow reports. The three checkpoint campaigns retain their
commands, original streams, changed images and derived analyses in their respective roots.
Their files support reproduction and scrutiny on this workstation; they are not a new
canonical result authority or a claim of portable full-plan qualification.

### Integrated disposition and remaining uncertainty

I01 is the first production amendment to consider, supported by original semantic/scientific
passes and promising elapsed observations. Its larger observed working set must be explained
and its full construction counts/repeat conditions established before accepting the benefit.
I11 and I08 are concrete adjacent ownership corrections: prelude reuse must precede premise
construction, and ordinary reads should not accidentally construct editing products. I09's
reset-cause observation is a prerequisite to retention-policy selection. I02's indexed scan
is the next bounded miss-side experiment; deeper membership views, capture sharing, extent
refinement, history policy, stronger interning hints and I12 depend on their actual costs.

I06/I07/I10 are necessary companions to deep performance diagnosis: successful entrypoint
retention, truthful complete/loss accounting and proportionate diagnostic work. Their current
gaps are demonstrated rather than cured by this investigation. I03 preserves meaningful
scientific coverage while seeking valid preparation reuse. I04 remains conditional tuning,
with no selected backend/Hessian/evaluator change. I05 remains a shared physical-context
correction; the caloric/flow probe rules out a convenient universal enthalpy relaxation.

The principal unresolved evidence is exact projection completion/byte attribution, the cause
of increased prototype RSS and workspace rotation, precise boundary-wait ownership, full
attempt/session linkage and successful final native/physical report retention, whole-journey
diagnostic overhead, and the historical exchanger's inconclusive conformance check. None is
silently replaced by a pass, a larger timeout, a relaxed physical check or an unavailable zero.
These are bounded follow-up questions with owners and revealing controls above. Accepted
scope, design changes, adoption and Plan64 completion remain with their existing plan and
decision routes; publishing this investigation does not change them.
