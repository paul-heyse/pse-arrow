---
doc_provenance: authored
doc_retention: while-dependent
title: Result publication and caller-stack inquiry
doc_owner: docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
---

# Result publication and caller-stack evidence

Read-only source inquiry, reconciled by the coordinator against [the baseline](README.md).
**Interface-checked:** the source paths below. Existing run observations retain 39d's exact
**Tested / Measured** conditions. No new product tests or simulations were run.

## Separate physical files from metadata transaction grouping

One `workflow/result_projection.rs::ResultAppendQueue` spans a durable completion, but
table changes and relation `finish` flush homogeneous file contents and immediately submit
their metadata. `flush` encodes a schema-compatible IPC file, reserves/seals it and constructs
`ResultBlockAppend` values. The file's schema restriction is real; it does not imply that
metadata for several already-sealed files must use separate append transactions.

`pse-operations/src/canonical_execution.rs::append_result_blocks` already prepares files,
validates bounded items, submits one `fn::pse_execution_v1::append_many` call and checks every
ordered exact receipt before ACKs escape. On uncertain transport it settles each original
operation. `validate_append_group` requires a common fence, unique operations, consecutive
ordinals within each result set and bounded encoded extent; it does not require one table/file.
An existing test covers interleaved result sets and exact replay. The generated function's
preflight precedes mutation, original append operations run serially, and a later gap/conflict/
deadline failure aborts the new group. The declaration/generator remains the owner; reading
generated SurrealQL is evidence, not permission to edit it.

Plan41's `canonical_artifacts` grouping belongs to a separate staged-artifact lifecycle.
Run results already have their own suitable grouped owner. Using an artifact queue for run
results would unnecessarily conflate activation and result-ingestion contracts.

Empty memberships use `declare_empty_result_set` and a different opaque empty ACK. They are
not `ResultBlockAppend` items. The narrow candidate is a bounded already-sealed metadata
queue across nonempty relation/file boundaries, retaining separate homogeneous file packing
and the existing group API. Keep empty declarations separately acknowledged initially;
grouping them would require a concrete extension and corresponding contract decision.

Keep common attempt fencing, exact file/payload binding, per-set order, every demanded
empty/nonempty output, original clocks, memory ownership, cancellation and actual cursor/
transport drain. A relation is complete after its real acknowledgements; sealing a file is
not result membership or publication. Closed manifests must still consume the complete exact
ACK inventory. Flush at an actual demanded acknowledgement/live-prefix boundary rather than
waiting for unrelated work. A new wire protocol or cross-run transaction is not required by
the inspected narrow candidate.

The retained scalar point's 100 builders/232 received-successful statements are whole-owner
facts, not publication-only counts, a whole-sweep total or a measured avoidable fraction.
File preparation and other writes remain after metadata grouping. Server execute time,
transport, local Arrow work and waiting cannot be inferred from statement count alone.

## Joint lifetime caveat for the proposed queue

The coordinator additionally inspected `ResultPayload::from_sealed_owned` and
`ResultBlockAppend`: a sealed payload has no resident byte vector, but can retain the caller's
encoder allowance. Current projection `flush` reserves three maximum-file extents and carries
that allowance through append/ACK settlement, alongside per-item metadata owners. Extending
that unchanged lifetime across several sealed files could retain multiple large encoder
allowances and defeat a small observer pool even though queued items appear to be metadata.
This is a source-established composition risk, not a measured new failure.

Separate file encoding/sealing from bounded metadata retention. The future queue must release
actual batches/encoded bytes and their encoding reservation after their final required use
and actual file/transport drain, while retaining sufficient small metadata ownership, sealed
file intent/protection and per-operation request identity until authentic append settlement.
Do not release real outstanding transport ownership to make the group fit. Test simultaneous
reservations and pressure-triggered flushing as well as query count; semantic stage separation
must also be reflected in live owners.

## Caller stack and nested async composition

Both default result cases abort on the benchmark main thread during original trajectory/
analysis assessment after simulation completion. Graph construction inspected here is bounded
and reachability uses iterative traversal. No direct self-recursive result-analysis/promotion
call was found; that bounded search does not establish absence of every recursion.

The retained development binary has large fixed poll-prologue reservations: approximately
1.90 MiB for benchmark `results_analysis::prepare`, 0.967 MiB for the runtime analysis inner
future and 0.471 MiB for artifact promotion. These exclude callees/wrappers and do not prove
that all reservations overlap or add linearly. Exact faulting-frame attribution is unavailable.
The unchanged publication workload completes under a separately recorded 64 MiB main-stack
limit versus the original 8 MiB limit. This supports a finite-stack-demand diagnosis; it
does not qualify the default route or demonstrate every production boundary's failure.

`Runtime::result_analysis` embeds protected result snapshots, dispatch, artifact promotion,
artifact reading, graph building and activation. Its direct Rust callers choose their own
polling context. Python `workflow.rs::blocking_on_with_interrupted_value` boxes the original
borrowed future, then uses `py.detach` and `executor.block_on(timeout(100ms,&mut future))`.
**Correction to the preliminary mapper interpretation:** a multithread runtime does not
move that root future to a worker. Pinned Tokio explicitly polls it on the calling thread.
Thus worker-stack settings do not resize the Python caller/main stack. The short timeout
is polled cooperatively and cannot preempt a nonyielding poll or a stack overflow.

The first design comparison is narrowing large async composition/state lifetimes and selective
child heap ownership while preserving borrowed lifetimes and completion/recovery values.
Heap allocation by itself does not bound compiled poll frames. Spawning creates a scheduling
boundary but adds owned/'static inputs, task failure, detach/abort/join and diagnostic-context
obligations. A dedicated larger-stack thread relocates demand and adds another owner; a
documented stack policy remains a comparison, not evidence that the underlying composition
is efficient. Verify the default caller boundary and original full assertions after any future
correction; the existing large-stack probe is not that verification.
