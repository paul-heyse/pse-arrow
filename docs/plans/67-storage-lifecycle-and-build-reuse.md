---
title: "Plan67: Storage lifecycle and build reuse"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/67-storage-lifecycle-and-build-reuse.md
doc_topics: [current-work, storage, environment, performance, validation]
review_sources: [docs/functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md]
---

# Storage lifecycle and build reuse

## Target and evidence

Prevent the same developer workflow from retaining another copy of its executable or
database state each time it runs. Run a fresh selected test or campaign when a current
decision needs evidence, and retire a consolidated worktree with its settled external
state. Keep useful Cargo state and improve the proportion of expensive compiler work
that sccache can serve.

The [functional review](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md)
is the scope source. Main `c2126df45` and the November 6 working trees are the inspected
baseline. **Measured, review baseline:** the post-clean census found 655.09 GiB of surviving
recently changed files, including 328.81 GiB of invocation ledgers, 276.84 GiB of linked
worktree targets, and 35.15 GiB of main fixture runtime captures. Birth times confirmed
523.89 GiB of newly created large files within the measured subset. These are historical
observations of surviving files. Establish actual cleanup yield from the cleanup population
and its own before/after observations.

**Tested, review baseline:** private 0.19.0 probes established eligible Rust metadata and
library restoration, changed-input invalidation, C output reuse, and loopback read-only
tier promotion and startup failure behavior. Representative workload benefit and reclaimed
space remain open. Use the [selected standard](../design_review/design_principles/standard.toml),
particularly DP-04, DP-09, DP-19, DP-21 and the test-assessment guidance. Preserve PS-13
where a test carries scientific assurance.

## Ownership and sequence

| Companion | Responsibility and working prerequisite |
|---|---|
| [67a](67a-plan66-consolidation-and-incident-cleanup.md) | Integrate the three frozen Plan66 worktrees, then retire their checkouts and settled incident results. |
| [67b](67b-invocation-custody-and-developer-history.md) | Own invocation executable custody, working reports and fresh rerun. |
| [67c](67c-surrealkit-and-rust-test-consolidation.md) | Own database-contract suites and Rust test consolidation. Interpret the suite's original output during the run. |
| [67d](67d-cargo-variants-and-sccache-reuse.md) | Own the baseline and effective build graph, cache coverage and performance. Capture the baseline before C removes variants. Adopt a smaller verification matrix as C lands and qualifies its replacements. |
| [67e](67e-resource-retirement-and-storage-verification.md) | Own recurring retirement, legacy reconciliation, and storage verification. Use A's observed cleanup boundaries. |

Start A with existing native ownership. Preserve any missing baseline needed for D before a
representative target is retired. In parallel, establish B's active-versus-settled
boundary and C's selected suite path. Finish the first ordinary test journey through C and B
before removing its former retention path. E closes the durable ownership loop and assembles
repeat-workflow storage evidence. This sequence permits cache-route experiments as soon as the
representative workloads and tool selections are identified.

[Plan66](66-integrated-simulation-target.md) remains the authority for simulation and its
outstanding acceptance. Its [C](66c-preparation-execution-and-service-lifecycles.md) owns
execution and settlement, [E](66e-protected-data-and-result-delivery.md) owns protected
scientific and selected raw custody, and [F](66f-developer-operations-clients-and-evidence.md)
owns original developer verdicts, acquisition and exact failed selection. Plan67 changes
their storage and retirement consumers through those owners. Keep each accepted obligation
at its original owner. Record any necessary contract or acceptance amendments there when
the corresponding integration lands.

## Decisions and boundaries

The maintainer selected structural prevention, SurrealKit for database contracts, and
consolidation of the three frozen worktrees. On November 7 the maintainer chose to delete
prior ledgers directly, including cited results, without migration or required regeneration.
Use read-only footprint evidence to verify growth reduction. Keep normal checkout-local
Cargo targets and the managed 100 GiB compiler cache. Prior citations can identify a new
selection when a current decision needs evidence. While a working report is still available,
failed rerun resolves its exact failed identity against the current admitted inventory and
produces a new result.

Keep published-data release owner-issued under ADR-0168 and ADR-0234. The incident uses
ordinary Git retirement and existing owners. E evaluates the deferred R-56 boundary for a
repeatable operator workflow. An owner-aware command or automatic collection of published
generations requires the applicable decision route before its implementation. Record the
operator's decision and any superseding ADR in the owning package. ADR-0252 governs finite
queries over explicitly supplied current evidence. The maintainer's regeneration decision
withdraws the persistent history pilot. Package completion, policy decisions and actual
reclamation have separate evidence.
[ADR-0252](../adr/0252-ephemeral-developer-evidence-relations.md)

## Opportunity dispositions

| Review reference | Disposition | Owning intervention |
|---|---|---|
| O01 | Planned | E verifies allocated footprint and release after repeat workflows. A records the incident's actual result. |
| O02 | Planned | B replaces per-run retained executable copies with active custody and selected evidence. |
| O03 | Planned | E defines checkout and store identity, preview and release. A consolidates the existing worktrees. |
| O04 | Planned | E applies owner-issued release and reports active, eligible and unresolved holdings. |
| O05 | Planned | C migrates database contract suites and application test constructs. |
| O06 | Planned | E retires selected generations, redundant intermediates and proven unused service fragments. |
| O07 | Planned | D reduces build variants and owns campaign layout and Cargo evidence. C owns test-scenario migration. |
| O08 | Planned | D qualifies eligible Rust and C/C++ reuse and optional tiering. Release-specific limits constrain adoption. |
| O09 | Deferred | Regenerate selected evidence when needed. B uses existing working-report and current-inventory rerun routes. |

## November 5 recommendation crosswalk

| Source item | Plan67 treatment |
|---|---|
| 1. Checkout removal and orphan GC | A performs the bounded consolidation. E plans explicit state identity and retirement and assesses R-56. |
| 2. Stop copying binaries | B releases working captures at settlement. |
| 3. Retention eligibility and collection | E uses explicit release and class budgets. Automatic published collection stays behind a separate decision. |
| 4. Pin cited evidence | Prior ledger citations carry no hold. Run the named case or campaign when a current decision needs evidence. |
| 5. Release plan stores | E ties release to closeout and native ownership. |
| 6. Route and clean scratch | E releases owned session scratch after promotion and actual settlement. |
| 7. Bound tool and native generations | E separates useful warm state, selected generations, rollback, legacy import and retired intermediates. |
| 8. Remove service fragments | E reconciles legacy files and the transient or persistent native owner. |
| 9. Bound build directories | C removes test-driven feature variants. D bounds campaign state and measures reusable verification. |
| 10. Reduce sccache to about 40 GiB | D retains the 100 GiB baseline. Capacity changes require workload evidence. |
| 11. Report footprint and guard admission | E delivers read-only measurement. Free-space warnings, reservations and admission gates are declined. |

## Integration checkpoint — November 7

Main contains the selected Plan66 worktree changes and this storage cutover. Git lists
only the main checkout. [67a](67a-plan66-consolidation-and-incident-cleanup.md) owns the
incident release and the later direct deletion of prior ledgers. [67b](67b-invocation-custody-and-developer-history.md)
owns settled executable custody and exact failed rerun. [67c](67c-surrealkit-and-rust-test-consolidation.md)
records the SurrealKit suite, consolidated Rust tests and actual native qualification.
[67d](67d-cargo-variants-and-sccache-reuse.md) records the managed 0.19.0 Rust and C/C++
restore measurements and the 100 GiB capacity choice. [67e](67e-resource-retirement-and-storage-verification.md)
records the ordinary Git retirement journey, owner-issued release and footprint observations.

The remaining work is the real standalone native-lane preparation and consumer journey,
execution of the independent PID oracle, the remaining compatibility and matched operations
timing matrix, recurring scratch and named-store retirement, published-generation retirement,
and assembled scope-end qualification. Keep each open obligation in its companion.

## Completion

Record the incident cleanup, functional cutovers, and performance evidence independently.
Complete A when the three checkouts and their retired external state are gone, or identify
the exact unresolved owner for any preserved exception. Confirm incident result deletion and
exercise a current-listing rerun while its working report is available. Complete the recurring scope when repeated ordinary
test and preparation workflows add zero retained executable or physical database copies,
released checkout and campaign roots return to their declared state, and deliberate holds
remain readable. C must show equivalent named assertions and failure distinctions. D must
record effective compiled units and cache behavior against a comparable baseline and support
each claimed latency gain with measurements. Keep targeted checks during execution and run
the selected assembled, documentation and hygiene scope after functional integration.

At closeout, transfer enduring commands and contracts to their existing owners. Retire the
series and resolved review through the repository's documentation lifecycle once current
consumers have moved. [Documentation lifecycle](../dev/documentation.md#metadata-and-aggregate-scope)
