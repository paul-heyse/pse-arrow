---
title: "Plan48h: Qualification and design publication"
status: draft
date: 2026-10-20
adrs: []
review_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md, docs/design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md]
scenario_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md]
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [validation, numerical-execution, performance, documentation]
---

# Plan48h: Qualification and design publication

## Purpose, baseline and boundaries

Establish that the selected Plan48 changes compose, remove their intended unnecessary work
and preserve scientific/ownership meaning. Publish the implemented design and reconcile
other plan documents **after** this series has been implemented and qualified. The
[coordinator](48-execution-performance-and-numerical-efficiency.md) owns all findings,
accepted RC01–RC12 boundaries, I01–I36 outcomes and completion status. This companion owns
only H01–H04 execution. Every package is **Proposed**, unstarted; no product tests or new
simulation measurements were run during authoring.

The current foundation is adequate: `.config/process-cases.json` and
`.config/preparation-cases.json` declare existing workloads; `scripts/pipeline_review.py`
selects explicit cases, consumes prebuilt binaries and runs cooperative stop/cleanup via
the existing case-audit owner. Existing test, native, Python, diagnostic comparison and
offline pack operations provide acceptance surfaces. Extend a declaration/control only
where the new behavior needs one; do not build another campaign runner or proof system.
Blueprint §24.2 and the [qualification guide](../dev/validation-assessment.md) own command
meaning and exclusions.

The audit already corrected the interpretation of the original campaign. R2 unset
diagnostics did not mean off; LLDB thread stacks were not exact on-CPU shares; outer
minus invocation was not exclusive preparation; cold/seeded native records were separable.
The new comparisons must respect those corrections instead of reusing their invalid
labels. Historical records remain valid evidence of their own conditions, not evidence
that new code ran. There is no required all-case rerun, exact environment match or claim
that one small sample establishes industrial capacity.

## Acceptance design

### H01 — source-to-package completeness and revealing controls

Check the real migrated producer/consumer closure for AF01–AF18/N01–N04 and all 162
register rows. Consume the coordinator/coverage appendix, current source and actual package
acceptance. Verify that shared findings close only after all their obligations, such as
AF12's separate KINSOL and cone controls, are complete. Omitted proposals are not missing
implementation. Existing capabilities identified by the audit are not new deliveries.

For each actual replacement, inspect consumers and delete its legacy implementation,
callers, fixtures and obsolete tests as soon as targeted checks establish replacement.
Do not preserve a second production route as evidence. A selected native alternative may
coexist as a real capability with its own explicit eligibility; that is distinct from
keeping superseded plumbing. Reconcile package dependencies with delivered behavior,
including receiving binding, complete linearization, neutral reset and terminal completion.

Targeted acceptance belongs to the package that introduces behavior. H01 identifies any
missing test scenario and returns it to that owner; it does not defer all consumer migration
or regression testing until the end. Use independent expected lexical/physical outcomes
where a production resolver or transformation could reproduce its own error. A copied
counter or successful serialization is not independent scientific assessment.

### H02 — short complete-operation observations and follow-up decision

Use explicit selected cases, existing controls and source/work counters to answer a named
question. Start with one ordinary observation per selected case after required untimed
setup. Capture detail/profile only when ordinary observations leave a concrete question.
Every condition explicitly sets its effective diagnostic mode **before process/import
initialization**; `PSE_DIAGNOSTICS=off` is off, while unset is ordinary-enabled. Selected
recording/capture/export are separate settings. Keep thread widths, start policy and actual
native realization in the descriptive context; do not demand environment identity proof.

The initial selection is by changed capability, not the Cartesian product of all libraries,
methods, dimensions, observer modes and widths:

| Changed capability / question | First revealing existing cases or targeted controls | Observations and preservation |
| --- | --- | --- |
| Source checking, registry and validation | Small synthetic scope/import/edit controls; `pr-inline`, then `pr-nested` or `pcsaft-02` if the changed path needs them | Examined names/dependencies, actual domain/description preparation counts, cold admission and warm edits separately; same identities/refusals |
| Contributor/sparse action work | `compact-root-banded-4`, `compact-root-banded-8`, `compact-implicit-derivatives-4-2`; sparse HVP/provider controls | Entries/directions/reconstructions/factors and retained scratch; current provider/branch/demand and original results |
| Compiled module/artifact selection | One existing co-used scalar body with actual compilation selected; targeted named-export/control fixture | Compiler children/setup versus warm evaluation, demand/orders/layout and private scratch; not capability-flag inference |
| Root/cone control corrections | Shifted/scaled native controls; `review-root-banded-64` only as a bounded specific regression; `review-conic-soc`, `review-conic-psd` and one POUNCE-convex control | Native status/norm/work versus original acceptance; Limit remains distinct from physically invalid; no universal one-retry promise |
| Starts and native retained state | Two-variable Ipopt exact/limited-memory, POUNCE exact/limited-memory and Uno accepted-start cases, plus a small parameter perturbation | Cold/seeded attempts separately; primal/optional payload/reuse receipt truth, iterations/evaluations/factors and actual reset/discard |
| Recycles and grouping | `compact-recycle-basic`, `compact-recycle-coupled`, `compact-recycle-stagnating`; tiny grouped/ungrouped control | Actual map/domain/branch/root eligibility, history/work and original closure; no universal n≈32 or Anderson history |
| Dynamic allocation/products | `compact-idas-ode-5`, `compact-idas-dae-5`, smooth/stiff controls as relevant; selected Jacobi/coupled-preconditioner and failure-reset fixtures | Native creation/reinit/symbolic/numeric counts, selected derivative entries, trial validity, local error and original trajectories |
| Fitting and anchored windows | `review-sparse-fit-2-1`, then size8 only if needed; short two-window fixed-trial fixture and selected objective-only/gradient controls | Integrations/derivative demand, GN constraint curvature, selected axes/checkpoints, private lanes and ordered continuity products |
| Study dispatch/capture/durable work | Four-point 1/4-worker compact/full/continuation cases; existing managed-durable short case only for affected store integration | Compatible dispatch, ready visits, sibling progress under held producer, original assessment, continuation reuse, claims/ACK/drain |
| Diagnostics and aggregate publication | History-zero/current/final native controls, loss/overflow/phase-exit controls; retained JSON/IPC scalar inputs for streamed publication | Effective mode/cadence, exact local work, current-state truth, source pointers/nulls/counts and coherent final files |

Do not run every listed case when a smaller control settles the changed mechanism. Do not
substitute a wall-time pass for unchanged science. Source counts and constructed products
can establish work removal without a statistically significant timing result. Separately
observe cold first-result, warm re-solve, accepted throughput and selected result availability;
do not sum overlapping native intervals, compare incomparable populations or call all
outside-invocation time preparation.

When a default/algorithm choice genuinely depends on latency, use five alternating matched
observations per candidate after untimed setup and report actual medians/ranges and work.
If that short series is inconsistent or a case exhausts its budget, stop the selection,
state the uncertainty and use I36; do not repeatedly enlarge the campaign until a winner
appears. Five observations are descriptive, not a universal significance or tail-latency proof.
No exact speed, exponent, callback-time, RSS ratio, iteration-ratio or core-scaling gate is added.

Keep exploratory science near the existing **60-second cooperative deadline**, followed by
actual registered cancellation/drain. Never terminate unrelated owners or classify interrupted
science as convergence. Preparation, execution, publication and cleanup remain distinct.
The existing prebuilt-binary driver does not build an extension behind a captured run.
Use ordinary fast development builds for exploration and explicit validated correctness
extensions for assertions. Full wheel installation is not required.

**I36 finite decision:** after the short selections, identify any remaining mechanism that
small cases cannot expose: sparse fill/coupling, retained working-set pressure, large-batch
transfer economics or long-horizon integration behavior. State the smallest composed case,
distinctive parameter/size, diagnostics, expected discriminating outcome, budget and actual
cleanup route. A 2,000–5,000-equation flowsheet is a possible later experiment, not a required
Plan48 benchmark or an already authorized implementation. The inquiry closes with either
no currently justified longer work and a concrete reopening trigger, or a scoped follow-up
proposal for later review/planning. Broad follow-up execution is not silently added here.

### H03 — assembled behavior and final quality

After functional packages and selected conditional adoptions are complete, execute the
relevant integration, component, native/Python and managed journeys once, broadening only
for a new change/failure/unresolved concern. The assembled paths must cover source edit →
prepared reuse → native solve → independent original assessment → demanded result, plus
affected study/recycle/dynamic/fitting and offline-review paths. Include one revealing
failure/cancellation/partial-publication journey for changed ownership. Use initialized
authenticated fixtures only for tests that actually need a store; pure scopes retain
their actual dependencies. A missing required solver/capability is a failure, not a skip.

Select recipe/filter names from the current owners at execution; do not promise that a
historical test identity still exists. The intended command routes are:

- During packages: `just check-package`, `just unit-package` and affected explicit native
  correctness routes; generators run `just codegen` when declarations change.
- At functional scope end: `just hygiene`, which includes Python Ruff/Pyrefly, generated
  drift and default/no-default continuing Clippy; fix failures and rerun the failing recipe.
- Also select the linked native-solvers Clippy closure with `--keep-going`, workspace/all
  targets, `-D warnings` and explicit validation through `scripts.arrow_validation`; a
  default/no-default pass does not cover opt-in native code. Keep all three modes distinct.
- Run manual `just governance`, `just docs`, applicable `just lint-native-contracts` /
  `just lint-native-data`, and `just features-powerset` / affected feature checks. Run
  affected scientific parity against the pinned reference where the changed scope consumes it.
- Rebuild only the needed editable Python extension through the fast validated route for
  correctness. No full wheel, alternate toolchain, release campaign or provenance proof
  follows merely from changed environment bytes.

Report actual commands/modes/scopes/results against the **zero** failure/warning baseline.
Targeted, static, native assembled, measured and unexamined scopes stay distinct. Original
failed/interrupted outcomes remain visible when a repair passes; tests that never ran are
not passed. No command is introduced as a commit/push/merge prerequisite.

### H04 — enduring publication and reconciliation

After accepted behavior is implemented and qualified, move enduring meaning to its owning
architecture/development guides through the appropriate ADR/design route and blueprint
revision row. Record selected native defaults, unsupported alternatives, compatibility/reset
and observation meanings where their consumers look. Accepted ADR history stays immutable.
Dependency pins move only if a selected capability actually requires it; no wholesale resolve.

Then review existing active/retained plans for changed dependencies, replaced scope and
qualification references. Update their native owners rather than copying live status into
Plan48. This future reconciliation does not restart or close unrelated Plan28/32/40 scope.
Retire completed plans/reviews only after enduring meaning and actual inbound consumers
are accounted for, using lifecycle preflight; Git history remains the archive. Do not add
an archive tree or delete raw campaign evidence merely because Plan48 is complete.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| H01 | Actual deliveries and selected inquiry outcomes from A–G | Reconcile every finding/register row, inspect real migrated consumers and replacement deletion, return missing targeted controls to their owner | open |
| H02 | Correct native receipts/controls and reliable current-phase semantics for changed paths; relevant packages ready | Run the smallest revealing short selections, document descriptive work/latency and settle I36's next-stage question without an automatic long campaign | open |
| H03 | Functional scope and selected conditional adoptions complete; H01/H02 interpretation available | Pass selected assembled/scientific/ownership journeys and scope-end checks, including all three continuing Clippy modes, with exact zero-baseline reporting | open |
| H04 | H03 selected acceptance and settled decision outcomes | Publish enduring contracts through their decision owners, then reconcile existing plan documents and lifecycle references without changing unrelated scope | open |

H01 is a continuing completeness responsibility, not a reason to postpone ordinary tests.
H02 can run a selected completed path before all unrelated packages finish. H03/H04 are
the final assembled/closeout stage; document ordering does not require repeating campaigns.

## Outcome (recorded after implementation)

### What was built

Not implemented or qualified. Future execution records actual deliveries and named conditions.

### A mistake made and corrected

To be recorded from execution, without rewriting historical failures as success.

### Deviations from the plan, deliberate

None recorded. Any broader method/platform/contract selection uses its concrete decision route.
