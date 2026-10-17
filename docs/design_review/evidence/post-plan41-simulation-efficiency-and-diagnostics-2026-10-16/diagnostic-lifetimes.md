---
doc_provenance: authored
doc_retention: while-dependent
title: Diagnostic lifetimes and receiver discovery inquiry
doc_owner: docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
---

# Diagnostic lifetime evidence

Read-only code inquiry and coordinator decisive-source checks against [the baseline](README.md).
Source claims are **Interface-checked**. Raw observations are historical **Measured** audit
facts under 39d's conditions. No new tests, captures or scientific simulations were run.

## Two distinct losses

In `pse-diagnostics/src/observation.rs`, new operation admission and metric reservation can
trim any completed catalogue node under the shared 16 MiB process allowance/operation bound.
Parent links are weak. `link()` copies a nonempty kind only from an actually allocated child;
the parent snapshot sets `available=false` when the child's weak reference cannot upgrade.

The retained fit image has 55 operations: 54 recent completed `implicit.inner_attempt` nodes
plus `math.job` 84, and 26 omitted owners. Its child `fit.execution` 85 has a nonempty kind
but `available=false`. This is node lifetime/retention loss before that parent snapshot;
exporter omission alone does not change the weak link's availability.

Runtime `Diagnostics::publish` sorts active owners first and then newest completed owners.
Each snapshot must fit the remaining 1 MiB image; history/detail are stripped when necessary,
then whole operations are omitted if still too large. The shooting image has 54 inner attempts
and `math.submission` 59, 29 omitted owners, and an available completed link to `math.job` 60
without that job's payload. This supports a separate export-loss mechanism. Images are atomic
files, but hub/list/parent snapshots are not one simultaneous catalogue cut; precise ordering
of every eviction/export event remains unavailable.

Recorded fit/shooting image sizes are 1,038,571/1,036,009 bytes. Operation sequence numbers
are process-wide diagnostic IDs, not native attempt counts, wall-time denominators or CPU
shares. No percentage-of-runtime claim follows. Historical returned fit/trajectory reports
also do not reconstruct nested experiment/trial work.

## Authored facts and remedy boundaries

Fit creates/enters `fit.execution`, propagates it to native execution and records trial/cache/
new/upgrade/completion and experiment facts. Shooting records trial/cache/upgrade/window/
completion and worker-reuse facts. Implicit attempts create/enter child contexts. Missing
context propagation is not supported as the primary explanation in these cases.

Metric semantics distinguish gauges/latest values from additive cumulative facts, absolute
counter resets and bounded recent buckets/history. Export history reduction keeps available
cumulative facts with explicit loss; whole-owner omission removes access to those facts.
Unavailable/partial is not zero, and inclusive parent/child or final/strategy work is not additive.

Two design obligations therefore remain independent: retain selected responsibility-owned
summaries through terminal consumption, and give their compact representation a credible
export route. Producer-authored cumulative aggregates plus bounded exemplars can suffice;
synthetically summing nested reports cannot. A protected diagnostic node/summary reservation
needs an explicit byte bound, admission refusal/partiality and release point. It must retain
observations only, never native cancellation leases, scientific payloads or execution permission.
Generic tracing context/export is an integration option, not authority for scientific aggregates.

## Exact receiver discovery and terminal prefixes

The corrected collector retains its discovery cursor (four 32-entry pages per read), rotates
eight active-operation reads and traverses completed related owners on terminal inspection.
Those existing focused controls establish collector behavior, not retrospective managed coverage.

The managed target supplies state/database; receiver readiness supplies PID/database. Current
collection obtains start ticks from `/proc` before enumerating or using a cached checkpoint.
The actual managed case passed science, result reads, receiver drain and fixture cleanup, but
had no live diagnostic association. Its retained default checkpoint matches PID 1044282/start
31355390, at page eight/rank 240 of the examined catalogue. The old first-page-reset collector
could not reach it. The later cursor correction enables eventual bounded discovery in a stable
catalogue, but cannot guarantee association before a short receiver drains.

Once a live association is validated, terminal copying can consume the cached exact process
instance/checkpoint and verify its embedded locator without requiring `/proc` again. Label
it the last published prefix, not a final post-drain observation; process liveness and data
freshness are distinct. An exact diagnostic locator in receiver ready handoff improves short-run
discovery but requires shared registration/writer wiring and an explicit contract change.
No locator confers service-stop, resource-release or scientific-result authority.

Blueprint §23.1 already permits independent bounded prefix inspection, exact identities and
visible staleness/loss. Cached post-drain prefix copying fits that contract. Responsibility
protection/export priority and exact ready handoff need explicit review of their target contract;
neither requires deleting old evidence or increasing scientific execution time for discovery.
