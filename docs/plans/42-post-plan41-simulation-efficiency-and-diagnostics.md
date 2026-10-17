---
title: Post-Plan41 simulation efficiency and diagnostics
status: done
date: 2026-10-16
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
scenario_sources:
  - docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
doc_owner: docs/plans/42-post-plan41-simulation-efficiency-and-diagnostics.md
doc_topics: [design, preparation, storage, validation, libraries]
---

# Plan 42: Post-Plan41 simulation efficiency and diagnostics

## Purpose, basis and authorization

This series adopts the [post-Plan41 review](../design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md)
and its [supporting inquiries](../design_review/evidence/post-plan41-simulation-efficiency-and-diagnostics-2026-10-16/README.md)
as corrective work and bounded investigation. It covers F01–F08, RC01/RC02 and all seven
remaining inquiries. Its target is efficient interactive edit/re-solve, studies, recycles,
dynamics, fitting and extension from the same authored definitions, with responsibilities
testable using their actual dependencies. Core 3.4, Efficient Architecture 1.0 and Process
Simulator 1.5 supply the design criteria.

Baseline: `main` at `6af92d1dee1ffa0b80dca71e03a63b489f2d940c`, inspected 2026-10-16, with the
seven pending audit/collector/benchmark/documentation changes described by the review.
**Interface-checked:** the review, current contract owners and decisive source were inspected.
**Proposed:** the corrections, investigations and future verification below. No production
implementation, build, simulation, dependency change or scientific measurement was performed
while authoring this series. The operator authorized document creation and existing-document
reconciliation; product execution requires subsequent authorization.

The review's decision remains **Revise**. Historical scientific successes retain their exact
conditions; they neither resolve these findings nor qualify the default result-analysis route.
[39d's dated audit](39d-qualification-and-plan38-cutover.md#post-plan41-simulation-audit--2026-10-16)
owns the observations, eleven case outcomes and AUD39-01–07 reconciliation. This plan owns
the new corrective dispositions and their relationship to AUD39-08–15.

## Resulting system and responsibility boundaries

Preparation consumers retain the admitted model they actually need. Scalar checking and
source lookup avoid irrelevant construction. Warm mathematical reuse reaches lookup before
expanding reusable structure, while attribution and current scientific permission retain their
own dependencies. These changes consume existing compiler/Salsa products rather than add a
second generic cache or model authority.

Result analysis must execute on an ordinary caller stack. The first remedy narrows large async
composition while retaining borrowed caller ownership. File schema determines IPC packing,
not every metadata transaction: separately sealed nonempty files can share the existing
fenced append owner after large encoder ownership has drained. Exact ACKs, original clocks,
live-prefix demand, protection and complete manifests remain governing contracts.

Scientific compositions declare compact responsibility summaries through the generic diagnostic
boundary. Retention and export must both preserve useful explanation within finite budgets,
without keeping completed science falsely active. Exact receiver readiness supplies observation
association; a validated retained checkpoint remains readable after process exit as a last
published prefix. Observation never grants scientific permission or cleanup authority.

| Document | Owned design and execution progress |
| --- | --- |
| [42a: Preparation and incremental reuse](42a-preparation-and-incremental-reuse.md) | F02/F03/F04/F08; retained consumers, scalar checks, source lookup, compact lookup/attribution and subtree-inference inquiry |
| [42b: Result publication and analysis](42b-result-publication-and-analysis.md) | F01/F05; ordinary caller execution, encoder/metadata ownership, demanded ACK boundaries and immutable-byte carrying inquiry |
| [42c: Responsibility diagnostics and receiver discovery](42c-responsibility-diagnostics-and-receiver-discovery.md) | F06/F07; producer summary contracts, independent retention/export and exact ready/terminal inspection |

This coordinator owns shared decisions, finding dispositions, cross-plan dependencies and
combined acceptance. Companions own their native package states. Architecture sections own
enduring contracts; reviews and audits remain evidence, not implementation authority.

## Foundation assessment and consequential alternatives

The inspected foundations are suitable for targeted correction. `DeclaredExecution` retains
preparation, `append_result_blocks` already groups interleaved nonempty sets under one fence,
typed observation already carries fit/shooting facts, and checkpoints already identify exact
process instances. Reuse these owners. Plan41's staged-artifact grouping is distinct from
run-result append; its delivered correction is not absent merely because this append route
still makes many calls.

Four compositions need particular care. Diagnostic preparation permits missing free values,
unlike solve admission. Recursive member lookup's result depends on the active probing set.
A sealed result payload can retain a large encoder lease despite holding no resident bytes.
Python's borrowed future is polled by `Runtime::block_on` on the calling thread, so worker
stack configuration and heap placement alone do not establish stack safety. These premises
are built into the companions' correction and acceptance contracts.

Use the [library inquiry](../design_review/evidence/post-plan41-simulation-efficiency-and-diagnostics-2026-10-16/library-alternatives.md)
for resolved Tokio/futures, Salsa, tracing and SurrealDB limits. Refresh consequential APIs
against the current exact lock/source and relevant library skills/Context7 when needed.
Salsa cannot eliminate construction before lookup; tracing cannot recover discarded summaries;
bulk insertion cannot substitute for exact fenced ACK settlement. No dependency upgrade,
global subscriber, new solver method, universal operation registry or new wire protocol is
selected. Stale capability maps and the utilization catalogue are not evidence for this work.

Preserve PS-G1 physical meaning, PS-G2 structural admission and PS-G3 original numerical
assessment independently. No new unit/property conformance status follows from these changes.
The surrounding unexamined scientific breadth is not additional implementation scope.

## Rule decisions

| Rule impact | Decision and date | Selected target and route | Dependent work |
| --- | --- | --- | --- |
| [RC01](../design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md#rc01) | Accepted explicitly by operator — 2026-10-16 | Add bounded producer-declared responsibility-summary retention through terminal consumption and compact export priority. Install the amendment at blueprint §23.1 through the `design:` route with a blueprint revision; preserve truthful completion, active work, refusal/loss and finite release. | G0 precedes C1–C3 implementation depending on the changed retention/export policy; I03 can investigate under current rules. |
| [RC02](../design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md#rc02) | Accepted explicitly by operator — 2026-10-16 | Extend receiver readiness with an exact diagnostic locator tied to the real registered receiver. Amend its contract/operational owner and blueprint §23.1 through the existing design route before dependent wiring. Preserve optional observation and last-prefix semantics. | G0 precedes C4's readiness contract change; cached exact terminal-prefix handling within current rules need not wait. |

Acceptance selects the target; neither amendment is installed by publishing this plan. Do not
edit accepted ADRs. An ADR is needed only if the concrete realization meets an actual trigger
in the decision rules. Changed scientific/view identities, spawned/dedicated Python execution,
grouped empty ACKs and relaxed settlement remain unselected alternatives; return their concrete
rule changes for decision before adoption. Narrow checker/lookup changes receive no blanket ADR.

## Coordination packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| <a id="g0"></a>G0 | Accepted RC01/RC02; concrete C1/C4 contracts | Install governing amendments before dependent production changes, with revision and appropriate decision route. Authoring itself does not install them. | Completed — blueprint revision 179; §23.1 target installed before dependent code |
| <a id="g1"></a>G1 | Companion contracts and bounded inquiry outcomes | Reconcile producer/consumer, lifetime, identity, ACK and generated-interface changes; verify migrated consumers and removal of replaced paths. Identify any new concrete rule decision before its dependent implementation. | Completed — owner/generated boundaries and selected assembled acceptance reconciled |
| <a id="q0"></a>Q0 | G1 and integrated functional scope | Select one affected assembled acceptance set shared with the existing Plan39/40 owners; identify which remaining obligations it can actually establish. | Completed — six once-only cases plus installed Python analysis interruption/recovery selected |
| <a id="q1"></a>Q1 | Q0; working ordinary native/Python and managed boundaries | Execute revealing assembled journeys below with unchanged scientific assertions, ordinary caller placement, original relevant pools and actual cleanup/drain. Repair exposed defects with targeted checks. | Completed — six selected cases and current installed Python analysis interruption/recovery passed |
| <a id="q2"></a>Q2 | Functional scope and Q1 repairs complete | Relevant scope-end integration/component/native/Python checks, `just hygiene`, governance/docs and applicable native/powerset checks; report actual zero-baseline results and exclusions. | Completed — scope-end checks passed after affected repairs |
| <a id="q3"></a>Q3 | Q1/Q2 evidence and all required decisions settled | Reconcile findings/inquiries, write actual Outcomes, transfer enduring contracts/guides, update dependency handoffs and preflight eligible retirement. Leave other campaigns open. | Completed — dispositions, Outcomes, enduring contracts and dependency handoffs reconciled |

## Finding dispositions

This table alone owns adopted F01–F08 disposition. Resolved records the delivered correction and
its selected acceptance; it does not complete the broader Plan39/40 campaigns. Packet detail is
at each companion and the actual combined evidence is in the Outcome below.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
| --- | --- | --- | --- | --- |
| [F01](../design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md#f01), AUD39-14 | Review S05/S06/S07 | Resolved | [42b B0/B1](42b-result-publication-and-analysis.md#work-packages); Q1 | Both unchanged result cases passed on the ordinary caller stack, each with native join and fixture cleanup; enclosing registered stop passed. Current installed Python interruption/recovery and registered stop also passed. Shared scope-end checks passed. |
| [F02](../design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md#f02), AUD39-08 | Review S01/S02/S07 | Resolved | [42a A1](42a-preparation-and-incremental-reuse.md#work-packages); Q1 | Retained scalar-preparation case and installed Python modeling diagnostics passed; missing-input/applicability controls passed in the owner suites. Shared scope-end checks passed. |
| [F03](../design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md#f03), AUD39-09 | Review S04/S07 | Resolved | [42a A2](42a-preparation-and-incremental-reuse.md#work-packages) | Scalar/contextual/refined/indexed controls passed in the 548-test pure owner suite and related runtime controls. Shared scope-end checks passed. |
| [F04](../design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md#f04), AUD39-10 | Review S04/S07 | Resolved | [42a A3](42a-preparation-and-incremental-reuse.md#work-packages), I04 | Borrowed access and ingress/local/delta/replay controls passed; I04 selected only completed root empty-probing memoization with frontier invalidation. Shared scope-end checks passed. |
| [F05](../design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md#f05), AUD39-11 | Review S02/S06/S07 | Resolved | [42b B2/B3](42b-result-publication-and-analysis.md#work-packages); Q1 | Real-store encoder/group/ACK/manifest and reply-loss controls passed, plus both original result journeys. The stale file-byte metadata fixture was corrected and affected controls passed. Shared scope-end checks passed. |
| [F06](../design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md#f06), AUD39-12 | Review S02/S03/S07 | Resolved | [42c C1–C3](42c-responsibility-diagnostics-and-receiver-discovery.md#work-packages); Q1 | Pure retention/export/refusal/expiry controls and assembled fit/shooting summaries passed; partiality and cumulative meaning remain explicit. Shared scope-end checks passed. |
| [F07](../design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md#f07), AUD39-13 | Review S02/S03/S05 | Resolved | [42c C4/C5](42c-responsibility-diagnostics-and-receiver-discovery.md#work-packages); Q1 | Managed sixteen-point journey used actual readiness association under the unchanged 256 MiB observer pool; exact typed checkpoint remained readable as a 36-operation last prefix after receiver exit. Shared scope-end checks passed. |
| [F08](../design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md#f08), AUD39-15 | Review S01/S04/S07 | Resolved | [42a A4](42a-preparation-and-incremental-reuse.md#work-packages), I05 | Complete hit identity, fresh value/domain checking and exact current occurrence controls passed in checker/compiler/runtime owners; scalar-preparation journey passed. Shared scope-end checks passed. |

## Investigation coverage

These rows route the review's seven inquiries; their outcomes live in the named packages,
not in a second mutable status table. Required corrections cannot close merely by planning
another investigation. Conditional optimizations may conclude with a supported no-adoption result.

| Inquiry | Decision to settle and evidence | Outcome owner and constrained work |
| --- | --- | --- |
| I01 — review inquiry 1 | Identify consequential caller poll/state boundary; compare narrowing with explicit placement using source/layout and a revealing ordinary-stack control. Do not add frame reservations as if they necessarily overlap. | 42b B0; B1 and result-route Q1 |
| I02 — inquiry 2 | Classify each result `finish`/flush consumer by real ACK/live-prefix demand and original clocks before delaying metadata submission. | 42b B2; B3 |
| I03 — inquiry 3 | Specify summary selection, byte/count admission, publication handoff, refusal and finite terminal release under existing diagnostic/image bounds. | 42c C1; G0/C2/C3 |
| I04 — inquiry 4 | Determine whether empty-context root memo reuse is useful and equivalent on an unchanged frontier. Check both consultation and insertion; nested positive results can depend on recursion context. | 42a A3; optional memo adoption only |
| I05 — inquiry 5 | Distinguish immutable hit-path reconstruction from required fresh checking; use construction observations before retaining additional products. | 42a A4; additional retention only |
| I06 — inquiry 6 | Shape-sensitive subtree inference controls with independent typing/overflow/binder expectations; establish repeated work without assuming dominant quadratic time. | 42a A5; conditional evaluator correction |
| I07 — inquiry 7 | Trace physical encode-for-hash/upload ownership, compare carrying already-owned immutable bytes, and preserve fresh eligibility/protection. | 42b B4; conditional byte-carry correction |

Compiled scalar evaluation, sparse symbolic reuse, Krylov methods and derivative upgrades
retain Plan41's settled applicability fences. They are not new implementation packages in this
series. Reopen only on concrete evidence that an eligible operation lacks the consumed capability;
do not enable a method universally, parallelize causally ordered windows or weaken physical checks.

## Prerequisites and execution sequence

Resolve B0 early because F01 blocks ordinary result analysis. Develop C1/I03 and C4's locator
contract early because useful retained diagnostics assist later inspection; install their G0
amendments before dependent implementation. A1/A2 and borrowed-access A3 can proceed independently
under current contracts. They do not wait for every diagnostic detail or the whole 42b document.

B2 first establishes actual encoding/sealing/transport drain and small metadata ownership;
only then may B3 retain multiple sealed-file appends. C2 must retain meaningful summaries before
C3 export ordering can preserve them. A4 establishes complete lookup identity before miss-only
expansion and exact occurrence retention. C5 requires implemented ready wiring for its managed
acceptance, not just an agreed field name. A5/B4 investigations do not block unrelated packages,
but must settle their selected adoption/no-adoption routes before G1.

Integrate affected producers and consumers in each package, with compile/targeted tests and
removal of replaced paths. Shared source/generator surfaces require coordinated editing even
when packages are logically independent. G1/Q0 then select Q1 assembled journeys; Q2 runs once
functional scope is complete. Q3 transfers durable meaning and reconciles existing-plan handoffs.

## Verification and cross-plan acceptance

**Proposed:** pure checker, lookup, summary and collector controls use local inputs and actual
dependencies. Transaction, native, Python and managed guarantees retain the integration they
need. Arrow-consuming correctness closures explicitly activate force-validation. Generator
changes update their authority and run `just codegen`; no generated output is hand edited.
Packages run touched compile checks and targeted tests. Root runs `just turn-end` at turn end;
static/hygiene and broad integration run at scope end under the repository execution policy.

| Assembled distinction | Minimum revealing route and ownership |
| --- | --- |
| Ordinary analysis and recovery | Both original results cases with full graph/publication/reopen/retirement assertions on the ordinary stack; synchronous Python entry and signal/recovery boundary. Q1 owns Plan42 evidence; 39d consumes applicable results. |
| Reuse without lost validity | Retained declared/diagnostic preparation and value/bound/fixed-state/structural/span-only edits, with direct work observations and unchanged numerical expectations. A full 32-block run is needed only if a remaining question cannot be settled more narrowly. |
| Durable nonempty/empty results | Interleaved file/set grouping, actual demanded ACK flush, uncertain response settlement, ordering/deadline conflicts, interruption and full manifest inventory; original relevant observer pool. |
| Responsibility explanation | Short real fit/shooting journeys plus heavy-inner-attempt synthetic pressure; separate retention/export controls, cumulative versus gauge/reset meaning and opt-out/failure. 40e consumes matching diagnostic evidence. |
| Managed observation and drain | Existing sixteen-point managed numerical/read assertions within the original 256 MiB observer pool, exact readiness association, busy reads and post-drain last-prefix handling. No longer run is used solely to make discovery succeed. |

The final selection must distinguish absence, partiality, refusal, scientific failure,
publication failure and actual native drain. A once-only successful observation establishes
its assertions, not statistical performance. No exact speedup, latency threshold, full feature
cross-product or environment/provenance proof campaign is introduced. Existing timing labels
are not a mandate to rerun eliminated comparative measurements.

[Plan39d's selected Outcome](39d-qualification-and-plan38-cutover.md#outcome-recorded-after-implementation)
now records completed D0–D5 inherited acceptance, including the thousand-point and native
cancellation journeys. [Plan40e](40e-qualification-and-closeout.md) retains its separate E0–E6
assembled diagnostics/profile/locality qualification. Shared evidence retains one execution
owner and its actual conditions; Plan42 does not absorb other plans' acceptance conclusions.

## Documentation transfer and completion

G0 owns governing diagnostic/readiness amendments. Implementation packages update enduring
preparation and result contracts at their existing architecture owners through the design route,
with blueprint revision when required. Update the diagnostics guide with actual retention,
loss, readiness and last-prefix behavior after delivery; amend the environment/operational
guide for receiver handoff where relevant. Do not describe planned behavior as implemented.

Completion requires integrated required corrections, settled inquiries, targeted and selected
assembled evidence, scope-end checks, deletion of replaced paths, truthful dispositions and
actual Outcomes. Q3 checks inbound dependencies before retiring resolved review/evidence or
completed companions. Plan41 remains a completed evidence source while existing readers need
it. Neither a newer number nor this series' closure closes Plan39, Plan40, Plan28 or Plan32.

## Current checkpoint

Plan42 is complete: all F01–F08 corrections, accepted RC01/RC02, seven inquiries, selected
assembled acceptance and scope-end checks are reconciled. Companions record delivered mechanisms
and supported no-adoption outcomes. Ordinary caller-stack result analysis/reading now passes;
encoder/metadata ownership, exact ACK boundaries, retained summaries and exact receiver inspection
retain their original scientific/resource contracts. Enduring meaning is at blueprint §14.4,
§19–§21 and §23.1/revision 179 and the diagnostics guide.

All worktree changes are integrated. The final repair worktree's 72 changed files were compared
with main before removal; its temporary same-HEAD branch was deleted. Both delegated assignments
are complete. Failed probes and initial check results remain in local artifacts; incomplete data
and pins were preserved through exact registered cleanup. No unrelated owner was stopped.

The highest-numbered coordinator and its consumed companions/review remain retained for
current Plan40 and completed Plan39 readers and because their new Outcomes are not yet archived.
This is retention, not additional Plan42 work. Plan39d now records completed selected Plan39/38
final qualification, sharing the six applicable once-only cases. Its composed native/managed/Python,
scientific recovery, entered-pull interruption, parity, remaining cases and inherited reconciliation
are complete; original failures and repaired boundaries remain explicit. Plan40e retains its
broader independent campaign. Environment/provenance proof and statistical repetition remain
permanently eliminated.

## Outcome (recorded after implementation)

### What was built

**Implemented:** retained declared preparation and diagnostic consumers; scalar checking without
unused environments; borrowed operation-local source lookup with qualified empty-probing root memo;
complete pre-expansion mathematical lookup and separately valid current attribution; prepaid borrowed
ordinary-stack analysis/selection/batch factories; real encoder-to-small-metadata transfer and bounded
nonempty append grouping with demanded ACKs; bounded producer-declared responsibility summaries,
compact export, typed checkpoint-v3 exact readiness and same-image last-prefix inspection. Replaced
cache/caller/composition paths are removed. G0 installs accepted RC01/RC02 at revision 179 before
dependence. The enabling wrapper joins its exact systemd terminal transition under the existing
cleanup/earliest-cancellation clocks; it relaxes no admission or drain rule.

**Interface-checked:** independent implementation and closure reviews found no material remaining
Plan42 defect or missing contract transfer. I01–I05 are settled by actual boundary, consumer,
retention and reuse evidence. I06/I07 conclude with supported no-adoption of a broad evaluator product
and retained encoded-byte carrying. Those are completed decisions, not deferred requirements.

**Tested — local Linux, zero-failure baseline, explicit Arrow force-validation for consuming
correctness closures:** the pure nextest selection of modeling/compiler/diagnostics passed 548 tests;
related runtime owners passed 36; all twelve new runtime controls passed across targeted runs.
Real-store ACK/group/manifest/reply-loss/rollback/order and corrected metadata controls passed.
The final exclusive native reader selection passed four controls with actual registered cleanup.
`python -m unittest scripts.tests.test_case_audit scripts.tests.test_primary_observation
scripts.tests.test_pse_env` passed 87 controls; installed correctness diagnostics passed all 13
unit tests, and two selected modeling-diagnostic integrations passed.

**Tested / Measured — selected assembled observations, not comparative performance:**
`just case-measure ... --smoke --audit-diagnostics` passed `k4-scalar-preparation`,
`fit-gradient-cold-1`, `shooting-single-idas-cold-1`, `results-chain-1`,
`results-publish-chain-1` and `k4-study-managed-durable-16` with original scientific assertions.
The two original result cases pass on the ordinary caller stack, each with a native join and
fixture cleanup; the current-worker managed reader retains its original 256 MiB observer.
Current installed `pse.tests.canonical_signal_control --analysis` preserves the original exception
and cause, entered response drain and following analysis. Enclosing registered store stops and
commands passed. Fit/shooting retain useful responsibility counts with truthful partiality;
managed exact last-prefix inspection after receiver exit validates identity without claiming final
science. The original failed assembled invocation is not relabelled as a success.

**Tested — scope-end checks:** `just check` passed the all-target workspace check. The continuing
`just hygiene` attempted 28 checks: 22 passed and six initially failed. Corrected `just lint-py`,
`just typecheck`, `just codegen-docs-check`, `just codegen-schemas-check`, `just clippy-default` and
`just clippy-no-default` all subsequently passed. Full continuing native `just lint-solver-contracts`
also passed. All three Clippy modes use keep-going and warnings-as-errors. `just features-powerset`
passed its 393 depth-two combinations and separate no-default check; it is not exhaustive. `just
governance` passed. The documentation site build passed, and scope-end formatting completed with
no Ruff findings. Local reports under `build/assessment/p42-execution-20261016/` preserve initial
failures, final `clippy-complete/` passes, probe conditions and cleanup results. Equivalent checks
were not repeated merely for an exact-source/environment identity claim.

This qualifies the selected correction scope. It does not qualify the unfinished Plan39 scalar
thousand-point/later-cancellation obligations, its full physical sweeps/reference/recovery breadth,
Plan40e's deeper-profile/locality campaign, all solver/configuration combinations, other platforms
or whole-product scientific breadth. No speedup, exact timing or complete CPU-attribution claim is
made. Plans28/32 and formal proposed ADR status are unchanged.

### A mistake made and corrected

Initial analysis boxing left connected selection and cancelled batch-reader stack failures. Actual
debugger captures found the responsible further boundaries; prepaid borrowed factories and full
ordinary-stack reruns corrected them without larger stacks or pools. Individual frame extents are
not an established simultaneous sum. Initial metadata and immutable-tuple fixtures were stale and
were corrected with affected passing reruns. One pressure invocation selected an incompatible
functional CPU lane and ran no tests; exclusive placement passed. Initial static checks exposed
Python typing/style and downstream Clippy diagnostics plus three untracked generated schemas;
repairs preserve original assertions, clocks and exceptions, and generator-owned new schemas are
tracked. Earlier failures remain unsuccessful evidence.

A reboot reset inotify limits; the maintainer restored 524288 watches/1024 instances. Exact dead
creator markers from aborted owned probes were archived/released under standing authorization,
retaining incomplete outcomes, data, evidence and pins; registered cleanup then passed. An old idle
fixture was parked through its frozen lifecycle owner, preserving its data and other owners.

### Deviations from the plan, deliberate

Actual ordinary-stack evidence required narrowing connected result selection and batch-reader state
as well as analysis/artifacts. Exact heap grants are independent of SDK/metadata allowances. Summary
expiry is semantic and interaction-based, with no autonomous reaper or promised physical latency.
I06/I07 settle with no evaluator/byte-carry implementation for the documented reasons. One complete
observation per selected case preserves scientific assertions without statistical repetition or
latency thresholds. Environment/provenance proof is eliminated, not deferred. Applicable evidence
is shared with Plan39/40 without claiming their independent campaign completion.
