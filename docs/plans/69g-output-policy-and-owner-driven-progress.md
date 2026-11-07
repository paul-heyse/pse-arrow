---
title: "Plan69g: Output policy and owner-driven progress"
status: planned
date: 2026-11-07
adrs: []
doc_role: plan
doc_owner: docs/plans/69g-output-policy-and-owner-driven-progress.md
doc_topics: [current-work, design, libraries, validation]
review_sources: [docs/plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md]
---

# Output policy and owner-driven progress

This companion owns A04, [I06](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I06)
and [I07](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I07).
A03 owns the typed diagnostic and recovery projection. The review reproduced progress
glyphs and ANSI controls on attended stderr during explicit JSON and NDJSON search
with piped stdout. The current search renderer gates only on whether stderr is a
terminal. Fix that route first with the existing resolved-format contract.
[Search renderer](../../crates/pse-host/src/cli/investigation.rs) ·
[Output format](../../crates/pse-host/src/envelope/presentation.rs) ·
[Review evidence](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#evidence-and-limits)

## A04 — Gate presentation on format, then integrate owner events

Resolve format before constructing progress. For JSON and NDJSON skip interactive
presentation, including message formatting done solely for it. If a shared adapter
requires a draw target, indicatif 0.18.6 `ProgressDrawTarget::hidden()` keeps that target
from rendering. For attended human output draw to `ProgressDrawTarget::stderr()`.
Use a spinner with an explicit style during discovery, where the owner cannot supply
a total. Use a counted bar only with a known owner total. For unattended human output
start quiet and add occasional phase lines only where the owner has a useful update.
Keep structured logging independent of terminal drawing.
[Indicatif 0.18.6 draw targets](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressDrawTarget.html) ·
[Spinner and styles](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBar.html#method.new_spinner)

Use the same format decision for A03's local human diagnostics. With the pinned
miette 7.6.0 `fancy-no-syscall` feature, select graphical rendering for an attended
human terminal and narratable rendering for non-graphical human output. Respect
explicit color policy. Both consume the typed diagnostic and render locally.
Machine output uses the native envelope. Preserve delivery failures and the
original verdict separately. Prefer the current local handlers over adding global
hooks, alternative serializers or highlighting without a named use.
[Current failure rendering](../../crates/pse-host/src/cli.rs) ·
[Graphical handler](https://docs.rs/miette/7.6.0/miette/struct.GraphicalReportHandler.html) ·
[Narratable handler](https://docs.rs/miette/7.6.0/miette/struct.NarratableReportHandler.html)

Map existing owner events to phase, optional total and terminal presentation as
receiving packages become available. A complete count can precede failure or
settlement. Choose finish, abandon, or clear explicitly from the owner outcome
and settlement. An iterator exhausting or the last handle dropping can finish the
display independently. Prefer coarsened events and position updates over per-item
message construction. Draw throttling limits terminal writes independently of
position updates. Suppress unusable ETA values. Keep one bar or phase lines as the
default. Add a steady tick only when animation is useful across event gaps, and
stop it on exit. Consider `MultiProgress` only for a named concurrent consumer. One
renderer registers bars before drawing and removes its own members. `clear` erases
the display while retaining membership. `remove` detaches and hides the bar, so
removal must go through the owning group.
[Progress finish policies](https://docs.rs/indicatif/0.18.6/indicatif/enum.ProgressFinish.html) ·
[Progress state](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressState.html) ·
[MultiProgress lifecycle](https://docs.rs/indicatif/0.18.6/indicatif/struct.MultiProgress.html)

Use adapter instrumentation only for a concrete boundary where it replaces a
counter. For example, `ProgressBar::wrap_write` wraps a writer and counts bytes
accepted by successful writes, while flush and durable publication stay with D03's
owner. A buffered reader advances when bytes are consumed. Check the actual unit
and behavior on partial transfer, seek or pending I/O if that adapter is selected.
Keep optional Tokio, Rayon, and stream features conditional on a receiving workload.
[Indicatif adapter contracts](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBarIter.html) ·
[Pinned writer implementation](https://docs.rs/indicatif/0.18.6/src/indicatif/iter.rs.html)

For the first deliverable, reproduce the review search fixture with stdout piped
and stderr alternately a pipe and PTY. Require valid JSON or NDJSON frames and zero
progress or terminal escapes on either stream. Check structured usage and failure
on the machine route. For human output, test no color, narrow and Unicode output,
and interleaved logs where supported. For receiving operations check unknown totals,
cancellation, terminal write failure, and full count before a failed owner verdict.
Use the current PTY harness first. Enable the test-only indicatif `in_memory` feature
and use `InMemoryTerm` only if layout or removal needs assertions beyond stream purity.
[Terminal abstraction](https://docs.rs/indicatif/0.18.6/indicatif/trait.TermLike.html) ·
[In-memory terminal](https://docs.rs/indicatif/0.18.6/indicatif/struct.InMemoryTerm.html)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| A04 | planned | Existing resolved format and search renderer for the first deliverable. A03 projection and actual owner events for later receiving work. | PTY machine-output regression passes first. Receiving controls preserve owner verdict and settlement through success, failure and cancellation. |
