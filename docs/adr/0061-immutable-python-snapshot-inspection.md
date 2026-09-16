---
id: ADR-0061
title: Expose admitted immutable Python snapshot streams under one resource budget
status: superseded
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-20, DM-28, DM-30, DM-35, DM-37, DM-39, DM-41]
blueprint: [§14.3, §18.8, §20.1, §20.3, §21.1, §21.5]
review: docs/design_review/reviews/design_review_wave2-contracts_2026-09-14.md
evidence: Proposed
supersedes: []
superseded-by: ADR-0068
revisit: A consumer needs queries, mutation, compilation, solving, remote stores or process resource reconfiguration.
verification: python/pse/tests/test_snapshot_streams.py; pse-catalog inspection tests; just py-test; just parity-container; just quality; just family-check.
---

# ADR-0061: Expose admitted immutable Python snapshot streams under one resource budget

## Context

Wave 2 needs Python inspection of exact committed models and persisted stages.
The catalog already admits complete relations and attaches reservations to their
final Arrow buffer owners. Its mutable local opener creates directories, and its
query facade collects results; neither is an appropriate implicit operation for
read-only named-table export.

## Scope

**Current construction target:** ADR-0067 and blueprint revision 37 replace this
record's former local replay, row-copy and phase-limited execution mechanisms.
Plan 05 owns implementation; the domain/identity/lifetime requirements retained
below are implemented through its single native preparation/completion route.

This amends the Python inspection boundary within the existing catalog, resource
and Arrow C Stream contracts. It adds existing-directory local opening, immutable
snapshot handles and named streams. Compilation, solving, mutation, arbitrary SQL
and unchecked Arrow re-entry are outside this API.

## Drivers

Pin actual admitted content independently of mutable refs. Share one finite
process budget. Preserve nested field metadata and final-owner accounting across
Python lifetimes. Surface admission, cancellation and resource failures instead
of returning partial successful data.

## Options

Adopt bounded slices of already admitted table batches. Keep the sealed session
route for queries. Reject per-open unbounded engines, implicit collection and
trusting manifest identities without actual admission.

## Outcome

`pse.open(path, *, settings=EngineSettings(...))` opens only an existing local
directory. `EngineSettings` projects the existing Rust `ResourceBudget`,
`ThreadBudget`, `ExecutionSettings` and `StoreLimits`: callers supply memory bytes,
threads, spill directory and capacity, batch rows, object bytes and control bytes.
Other execution semantics use the existing declared defaults. All opens in one
process share one configured runtime; conflicting configuration is refused rather
than multiplying the deployment limit.

`Store.head(ref_name="main")` pins the exact observed ref, manifest encoding and
optional authored revision. Persisted exact stage receipts provide predecessor
bindings; readers reopen and validate actual parents and producer outputs.
Missing or ambiguous bindings fail. Later ref movement cannot alter an existing
snapshot. Hashes select/check encodings; schema, rows, invariants and producer
semantics are independently admitted.

An immutable `contexts/<exact manifest checksum>.json` receipt retains the target
`ManifestRef`, optional exact producer and role-to-`ManifestRef` parent bindings.
Publication writes it through the existing `ensure_create` protocol before a
snapshot handle/ref becomes visible. Reopening follows those exact references and
actively revalidates each parent and producer under explicit control-byte limits
and shared graph reservations, with cycle detection.
The receipt and its lookup key are not validity certificates. A context-dependent
legacy manifest without the receipt fails explicitly; a different stored physical
context conflicts unless actual equivalence has been established.

`Snapshot.table("namespace.name", port=None)` returns a one-consumption Arrow C
Stream. Repeated stage schemas require the exact output port. The catalog reader
slices the admitted batch at the configured batch size while retaining snapshot
and recursive buffer ownership. It performs no query, per-cell conversion,
materialization or implicit compilation. Current-format reopening establishes unresolved properties through the ADR-0067
common admission route. Read-only producer recomputation is required only where
source correspondence cannot otherwise be established; it creates no new published
physics, stage or ref. The separate `CompilerValidator` roster is replaced. Consumer-requested collection is explicit.

Store and snapshot `close()` release their own handles and prevent new operations;
existing descendants retain independent ownership. Stream `close()`/`cancel()`
stop unused work and release unread sources; exhaustion releases sources too.
Exported arrays keep actual buffer reservations until their final consumer drops
them. Cancellation becomes an error at the next read, not successful empty data.
Errors retain diagnostic codes and source details; Arrow consumers may expose
stream errors through their own exception classes.

Generated extension registration remains the sole Python vocabulary. Export
preserves nested metadata and supports per-field comparison against a supplied
consumer schema: retained interpretation, storage-only interpretation, lost
metadata or incompatible shape. A capsule does not reveal another process's
registered implementations. There is no unchecked degraded-input admission route.

### Consequences

The native boundary retains the configured process runtime and existing semantic
validator. Complete catalog admission uses owned current-format inputs and the common residual-obligation program; early valid-looking batches cannot mint a complete result. Named export streams already admitted outputs and does not rerun their producer. The process budget cannot be silently
replaced after configuration.

### Compensating controls

Use pinned pyo3-arrow `PyRecordBatchReader` and existing Arrow buffer leases.
Reject requested schema casts. Keep one native-import gateway and exact stubs.
Add only existing dependency-family edges; no new pins or families.

### Confirmation

`python_snapshot_streams` covers ref pinning, empty/multi-batch/nested data, partial
drain, cancellation, retained arrays after handle close, actual reservation release
and malformed inputs. Rust fixtures cover slices and repeated output ports. Run
host and parity interpreters with strict quality/family gates and zero failures.

## Pros and cons

The small surface preserves existing authority and ownership. Full semantic
reopening can cost more than reading unchecked IPC, and resource configuration
must precede store access.

## More information

See [plan 04 W2-10](../plans/04-wave-2-semantic-compilation.md#w2-10--immutable-python-inspection),
ADR-0046 and the combined scoped review. Interface investigation inspected pinned
pyo3-arrow 0.19.0 one-consumption readers and PyO3 0.29.2 detached calls and frozen,
synchronized classes.

## Status history

- 2026-09-14 — proposed before boundary implementation.

- 2026-09-14 — reconciled with ADR-0067 and Plan 05; prior receipts describe their original code and do not certify the hard-pivot implementation.
- 2026-09-15 — superseded by ADR-0068.
