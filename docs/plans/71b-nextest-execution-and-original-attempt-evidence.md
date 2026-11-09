---
title: "Plan71b: Nextest execution and original attempt evidence"
status: draft
date: 2026-11-09
adrs: []
doc_role: plan
doc_owner: docs/plans/71-workflow-and-simulation-test-feedback.md
doc_topics: [current-work, validation, diagnostics, preparation, performance]
review_sources: [docs/functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md]
---

# Nextest execution and original attempt evidence

This companion owns the runner version and completed-run consumption for the
[coordinator](71-workflow-and-simulation-test-feedback.md#opportunity-dispositions).
The host already builds once per admitted compiler group, captures executable and
helper identity, then lists and runs with native binary and Cargo metadata. The
[completed report adapter](../../crates/pse-host/src/extraction/completed.rs) reconciles
inventory, JUnit, native exit and settlement. Preserve those responsibilities. Add
original-attempt access and visibility into work that still occurs on a warm invocation.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| [B01](#b01) | Planned | Current native discovery and correctness adapter | Qualified 0.9.151 pin and all selected entry points use it. |
| [B02](#b02) | Planned | B01; A01 | Original attempts, comparison and exact failure rerun preserve identity, coverage and private custody. |
| [B03](#b03) | Planned | A02 for new phase view; existing timings for inquiry | Named build/capture/list costs are distinguishable; any reduction has a verified invariant and no parallel path. |

<a id="b01"></a>
## B01 — Qualify and cut over the native runner

Select cargo-nextest **0.9.151**. Its weighted-group wakeup fix allows a waiting test
to start when an out-of-group test frees sufficient capacity; its custom-target reuse
fix covers the `rustc --print=cfg` route. This does not establish the cause of earlier
setup or drain failures. Update the managed runner selection and relevant version
guidance together; no old-runner execution route. The inherited 0.9.150 Cargo adapter
change forwards arguments as `--flag=value`. Exercise native selection and invocation
through the adapter, not just version output.
[Changelog](https://nexte.st/changelog/) ·
[0.9.151 release](https://github.com/nextest-rs/nextest/releases/tag/cargo-nextest-0.9.151)

Use `list --list-type=binaries-only --message-format=json` for compiled membership and
`--binaries-metadata` with `--cargo-metadata` for the full list and run. This avoids
another Cargo build, but the 0.9.151 run still enumerates harnesses. Preserve selected
helper executables, runtime paths, build-script outputs, compiler/profile/features,
fixture correspondence and force-validation; no mutable cross-checkout target reuse.
Use native filtersets, mismatch reasons and config precedence. Keep supplemental listing
where skipped suites need fixture validation or multi-group effective membership. A
human `show-config test-groups` view is useful for diagnosis, not a machine contract.
[List format](https://nexte.st/docs/machine-readable/list/) ·
[Tagged run source](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/cargo-nextest/src/dispatch/core/run.rs)

Qualification covers pinned version capture, joined Cargo options, default/profile,
filter unions, ignored/platform selection, zero match, fixture and helper membership,
and one admitted list/reuse/run round trip. Exercise a waiting weighted test with
capacity released by a different group; verify original coverage and settlement.
Native custom-target behavior need not add a supported target if none uses it.

<a id="b02"></a>
## B02 — Read original attempts and compare original selections

Extend `pse inspect tests ID` with an exact case/attempt view and comparison against
a second explicit invocation. Reuse authenticated original inventory and JUnit through
the existing owner. `nextest-metadata` **0.15.0** supplies listing and mismatch
semantics; `quick-junit` **0.8.0** supplies private completed case/retry records.
Nextest 0.9.151 writes the primary JUnit row as the first failure for consistently
failing tests and the last attempt for flaky passes. Normalize those roles; respect
runner policy when XML represents success but the invocation fails. Output can be
absent or sanitized. JUnit cannot supply unobserved progress, physical drain, or
completeness by itself.
[Metadata API](https://docs.rs/nextest-metadata/0.15.0/nextest_metadata/) ·
[JUnit API](https://docs.rs/quick-junit/0.8.0/quick_junit/) ·
[Tagged writer](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/reporter/aggregator/junit.rs#L149)

Match cases using authenticated package/target/feature context and binary/case
identity. Do not compare solely by XML class or run-local group ordinal. Distinguish
not selected, selected but unattempted, interrupted, attempted and completed;
distinguish missing report, expired or unavailable coverage, and incompatible
context. Derive the comparison on request, without a retained second data set.
Prefer typed deltas to text diffs; use the [A01](71a-cli-inspection-and-evidence-lifecycle.md#a01)
routes for separately captured bytes and [A03](71a-cli-inspection-and-evidence-lifecycle.md#a03)
for detail lifetime. Expose no out-of-budget material in summary.

Preserve `pse test --rerun-failed ID` as a new currently admitted run of exact
completed original failures. Refuse an original that cannot justify the selection,
and show changes in current availability rather than silently broadening. Native
Nextest `-R` can include interrupted and newly listed tests, so does not implement
this contract. A native replay is a read of a record, not a new execution.
[Native rerun semantics](https://nexte.st/docs/features/record-replay-rerun/rerun/)

Controls distinguish first versus last primary row, flaky policy, retries, setup
failure, interruption, missing report, changed current listing and expired detail.
Compare two retained fixtures where one case is absent from the second selection;
absence must not appear as a fix. Prove original evidence is not rewritten by rerun.
Do not add experimental event ingestion or embed `nextest-runner` to implement these
views; that would recreate reconciliation without replacing owner evidence.

<a id="b03"></a>
## B03 — Explain and remove avoidable work in current-source tests

Keep the native checkout writer, Cargo freshness and incremental defaults, and captured
image custody. Project observed queue, Cargo, compiler cache/link, capture verification
and native enumeration facts without attributing every delay to compilation. Use
`pse test --timings` with [Cargo unit timings](https://doc.rust-lang.org/cargo/reference/timings.html)
for a named build investigation and [sccache 0.19.0 Rust coverage](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Rust.md)
for eligibility and cache behavior. Cargo freshness can eliminate a compiler command;
sccache only applies to eligible commands Cargo actually issues. Neither replaces
binary custody. Expose retained timing evidence through A01 artifact selection.

| Question | Evidence and default |
| --- | --- |
| Can repeated binary verification be collapsed within one held immutable lease? | Map byte checks and mutation/replacement guards; implement only if the held lease carries equivalent protection across the consumers. Otherwise keep the checks. |
| Can repeated list work be removed? | Identify a concrete repeated enumeration, including multi-group cases. Preserve exact admission, effective groups and pruning; use Nextest's existing semantics. Keep current listings unless a safe removal is clear. Consider `nextest-filtering` only for identified semantic duplication; never recreate the DSL. |
| Does a small unrelated edit pull an unnecessary compiler/link unit? | Use exact dependency and build timing facts; identify the producer and what can stop depending on it. Leave a thinner executable or crate split deferred until dependency evidence warrants it and the existing decision route is met. |
| Would runner evidence answer a real unresolved scheduling question? | If existing observations cannot answer it, evaluate an opt-in exact-run recording and `store export-chrome-trace --group-by slot\|binary`, with bounded admission and release through A03. These traces omit Cargo and internal science. Use a private owned store and original UUID, not ambient `latest`; 0.9.150 changed recording format and fixed a portable rerun vulnerability. Consider USDT only for a named remaining runner gap. Otherwise keep recording and external tracing deferred. |

[Perfetto export](https://nexte.st/docs/features/record-replay-rerun/perfetto-chrome-traces/) ·
[Portable recording advisory](https://github.com/nextest-rs/nextest/security/advisories/GHSA-vc25-qg58-w57c)

Retain executable identity and correctness when work is removed. Do not create a second
build cache, use Git identity alone for current-source admission, or reinterpret
historical timings as measured speedup. Apply the coordinator's regression-risk rule
to changes that add a hot-path scan or trade narrow enumeration for a broader one.
