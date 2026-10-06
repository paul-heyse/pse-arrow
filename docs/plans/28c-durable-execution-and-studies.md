---
title: Durable execution and dependency-scoped studies
status: draft
date: 2026-10-05
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md, docs/design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md]
scenario_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#representative-journeys]
---

# 28c: Durable execution and dependency-scoped studies

## Responsibility and current foundation

This companion of [Plan 28](28-surrealdb-unified-substrate.md) owns the ordinary durable-run
default, attempt lifecycle, study readiness, result staging and coherent completion. It
consumes [28a's identities, codecs and transaction contract](28a-canonical-substrate-and-revisions.md)
and [28b's selected compilation descriptions](28b-selected-compilation-and-reuse.md).
[28d](28d-connected-results-and-analysis.md) owns connected reading and analysis of sealed
outcomes. Current shared contracts and finding dispositions belong to the coordinator.

The current `workflow/run.rs::RunResult::finished` and `DurableAttempt::finish` retain attempt
completion, progress, incumbents and seeds. Ordinary scientific result tables in `results.rs`
still require explicit publication. Studies already retain sources/definitions and publish
point results through their workflow. The target extends and consolidates this durability;
it does not start from a wholly ephemeral system.

Reuse physical/modeling admission, solver routing, native attempt ownership, completion and
the scientific portion of `study_policy.rs`. Replace the complete-study snapshot/action
derivation in `Studies::admit_dispatch`, PostgreSQL job claims and the cross-store publication
protocol. Preserve Plan 27's contextual accuracy and partiality distinctions, actual warm
starts, permission restrictions, predecessor usability and occurrence identity.

## Ordinary runs and attempt state

An application run selects an immutable problem revision and records demand, resolved
configuration, compilation/provider interpretation and outer build attestation before native
execution. Scientific results and available diagnostics are retained automatically for normal
completion, failure and cancellation. Storage or admission failure is observable; a caller
does not receive a successfully durable outcome when retention failed. An explicitly selected
ephemeral mode remains available to numerical libraries and local tests; it is not selected
implicitly because a store argument was omitted. Runtime construction supplies or opens the
configured durable substrate through A1. Direct kernel/solver tests need no unrelated server.

Keep semantic run and actual execution attempt separate. A retry of the same effect uses its
operation identity; a genuine new attempt has its own generation and observations. Record
the selected start actually used, including source predecessor and eligibility. Progress and
incumbents are observations rather than successful result sets. A failure can retain useful
scientific data without becoming an eligible predecessor or a usable result by accident.

Candidate discovery is separate from the short transaction that claims a candidate. The
claim transaction rereads the candidate, its relevant dependency states, cancellation and
admission generations under named record conflicts. Mutations of every decision premise
must touch the corresponding guard/generation. Successful claim produces an attempt fence;
native work runs outside the transaction. If the transaction conflicts, retry the entire
decision from fresh reads, not only its final update. `FOR UPDATE` does not provide predicate
locking or turn an unguarded graph query into serializable policy.

Claim expiry/recovery and cancellation revoke stale workers' admission authority. They do
not imply that the native process has stopped. The owning supervisor cancels/drains/releases
native sessions and buffers, and keeps stale generations unable to publish progress or
completion. Storage retries have bounded operation scope; a lost acknowledgment is resolved
by operation identity and recorded state before retrying a scientific solve. Cancellation
checks include the terminal-seal transition, not merely the worker's last polling point.

## Staging and coherent results

C1 establishes the terminal envelope and staging identity before C2 and D1 need them. A
bounded writer accepts typed scientific rows/blocks using A2's exact codec, under
`run/attempt/generation/result-set` identity. It records batches idempotently with sequence
and completeness information. Staged rows are private to that attempt; canonical queries
cannot expose an unsealed prefix as a complete result. A retry with different bytes at the
same batch identity is an error, not an overwrite.

Close result-set membership before terminal admission. Each batch write conflict-checks the
attempt's open-ingestion gate. Closing ingestion fences further writes; reconcile the now
immutable batch selection and coverage outside the terminal transaction, then record its
closed descriptor. The short seal checks that descriptor and its fence/generation. Readers
follow exactly its admitted batch/coordinate selection, never every row sharing a result-set
ID. Closing after partial failure retains truthful coverage; closure is not success. Restart
can finish reconciliation/sealing of the same closed operation without rerunning native work.

After native work and required result ingestion finish, a short transaction verifies the
current fence, cancellation, scientific completion/admissibility and staged result-set
completeness. It atomically records the actual terminal outcome and admits the corresponding
result sets. Do not place an entire trajectory, graph calculation or native solve inside this
transaction. Failed/partial/cancelled outcomes can seal their available observations with
explicit quality and coverage. A success summary cannot bypass the numerical conclusion,
physical eligibility or contextual accuracy requirements.

The reader sees either an admitted terminal selection or the explicitly requested progress
class, never a mixture selected by incidental write timing. D1 uses a recorded monotonically
ordered run/attempt sequence for latest selectors; wall-clock order is not authority. Repeated
finish after lost acknowledgment settles the same terminal operation. A stale generation,
missing batch or cancellation arriving before sealing cannot produce successful visibility.

Long-lived reads and cleanup use A3's protected-selection contract. Unsealed abandoned staging
can be reclaimed after its attempt is fenced and no recovery/read obligation remains.
Retention of failed diagnostics and partial data is deliberate policy, separate from orphan
cleanup. Result backpressure limits queued batches and buffers independently of native work
and solver-state memory. The repository's memory-capped native recipes remain required.

## Study policy and preparation

Store study definitions, distinct point occurrences, dependency edges and consumed start
policies in the same substrate. A point's readiness query examines the relevant predecessor
outcomes and current cancellation/generations, or evaluates a coherent group when policy
truly requires it. The policy remains one semantic owner. Express supported structural
readiness in native queries/functions and share the existing scientific predicate kernels;
do not create separate Rust and SurrealQL versions of the same evolving scientific rule.

Migrate sweep, continuation, fitting, dynamics and composed workflows, including study
finalization and recovery. An unsuccessful predecessor does not satisfy a usable-start rule
unless the explicit policy admits its available data. Repeated parameter values remain
distinct occurrences. A fan-out study should not derive all actions anew for each individual
dispatch. Whole-study checks, where real invariants require them, occur once or in a coherent
batch and expose their actual scope.

Separate pure study-wide admission from expensive numerical preparation. Resolve necessary
definitions and graph validity before dispatch, then prepare expensive artifacts when an
attempt is ready. Share compatible immutable products through B3 across equal structural
selections. Bound case preparation and native concurrency together; separate pools cannot
independently fill the machine. The local application supervisor owns a finite worker group
and partitions its configured process budgets among the database and native workers. Existing
process-local admission enforces each worker's allocation. Expired attempts remain charged
until their native work drains. Unmanaged external workers and distributed capacity scheduling
are outside the initial qualification profile. Cancellation drains prepared work and releases native state
without deleting canonical scientific history.

## Work packages

| Package | Prerequisite and delivered behavior | Consumer migration and deletion | Status |
|---|---|---|---|
| C1 — Lifecycle and result envelope | R0; working A1/A2 operations, codec and guard slice. Implement run/attempt identities, terminal classes, operation deduplication, cancellation fences and staging contract. | Migrate lifecycle types/registry declarations and unit policy consumers; expose the real shared contract to D1 and C2. | Scheduled |
| C2 — Automatic durable execution | C1, B2 descriptions and D1's minimal canonical result reader. Connect ordinary Rust/Python runs, bounded staging, atomic sealing and restart recovery. | Replace optional-store implicit ephemerality and ordinary explicit publication. Delete displaced `DurableAttempt`/publication mechanisms and callers when covered. | Scheduled |
| C3 — Dependency-scoped studies | C1, B1 selected inputs and B3 preparation slice; C2 terminal selection. Implement native discovery, fenced claims, scoped/batched policy, starts and finalization. | Move all study drivers/workers and recovery consumers. Delete PostgreSQL jobs/leases and complete-snapshot-per-dispatch paths, fixtures and tests for those mechanisms. | Scheduled |
| C4 — Interruptions and retained lifecycle | C2/C3 and A3 retention/read protection. Complete drain, stale generation rejection, uncertain acknowledgments and bounded staging reclamation. | Move operational maintenance and diagnostics. Remove cross-store reconciliation, publication intents and member-prefix cleanup as their last consumers migrate. | Scheduled |

C1 supplies only the shared lifecycle contract; its completion is not completion of automatic
retention or studies. D1's small sealed-result reader breaks the apparent execution/query
cycle. Root owns edits to shared workflow declarations and generator invocation. Numerical
accuracy owners remain responsible for their guarantees even when their observations move.

## Verification

**Proposed acceptance:** targeted unit checks and touched-package compilation accompany
each mechanism, using repository recipes with explicit force-validation. Native checks use
memory-capped recipes. [28e](28e-rebuild-retirement-and-qualification.md) owns assembled
journeys and the final campaign; these packages do not repeat integration suites at each row.

Exercise S03/S07/S08 with independent policy expectations: distinct repeated occurrences;
eligible and ineligible predecessors; concurrent claims of one candidate; disjoint candidates;
cancellation/revision change between discovery and claim; expiry followed by a stale worker;
and whole-decision retry. Race batch writes against ingestion closure and sealing; late writes
must be refused and readers must use only the frozen admitted selection. A graph predicate
race must conflict through its named premise,
including a previously absent identity, rather than pass only because the test is sequential.

Force failure after a staged batch, after terminal commit and before acknowledgment, during
native work and during cancellation. Reopening by operation ID must distinguish unsealed
partial work, an already admitted terminal outcome and a recoverable claim. Compare expected
visible result coverage and actual start coordinates directly. A failed or partial terminal
run remains queryable without satisfying a usable-result selector.

Check that increasing independent study occurrences does not invoke full-study policy per
dispatch, and that deferred preparation prepares only ready consumers. Mechanism controls
establish removal of repeated work; timing improvement remains unmeasured until E4. Durable
process-kill recovery and storage acknowledgment behavior require A1/E3's selected server
profile, beyond pure in-memory policy units.

## Checkpoint and next step

No C package is implemented by plan authoring. C1 follows R0 and A's working guard/codec
slice. C2 and D1 then integrate together; C3 can develop policy against C1 while awaiting
actual compiled/run consumers. This document owns C progress/local evidence; the coordinator
owns US01/US02/US04/EF03 finding dispositions.

## Outcome (recorded after implementation)

### What was built

Pending implementation and named evidence.

### A mistake made and corrected

Pending execution.

### Deviations from the plan, deliberate

Pending execution; consequential decision changes follow the decision route.
