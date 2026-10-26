---
title: "Plan 56d: Publication and workload fit"
status: draft
date: 2026-10-25
adrs: [ADR-0230]
review_sources: [docs/design_review/audit/README.md, docs/design_review/audit/system-architecture.md]
scenario_sources: [docs/design_review/audit/system-architecture.md]
doc_role: plan
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, performance, storage]
---

# Plan 56d: Publication and workload fit

## Target and assessed foundation

**Proposed:** physical publication, population, backup and exchange operations fit their complete
supported workloads while retaining original semantic and recovery guarantees. This companion
owns DA-034/035/044/045; C owns canonical meaning/placement and G owns assembled qualification.
[Plan 56](56-design-principle-alignment.md) owns disposition and rule decisions.

Reuse exact fences/acks, immutable protections, shared scientific policy, selected DataFusion
providers, offline freeze and manifest-last publication. Current `append_many` loops through the
single-item function, and `ResultPayload::new` is later converted to a file: neither is already
the desired bulk/inline realization. Population batching of point rows likewise does not remove
per-point `begin_run`. Complete the physical operation rather than renaming these entry points.

## Publication and population design

### Grouped append and demand-driven flush

Preflight exact requests as a batch, make shared writer/fence/permission admission once, update
set/frontier state in groups, and issue native bulk row/index changes in one bounded transaction.
Keep each logical immutable acknowledgement and all-or-nothing effect boundary. Consume C03's
selected locator: the preferred inline tier removes tiny-file lifecycle work; large compatible
same-relation batches still pack into sealed IPC. Empty demanded sets close without a payload file.

Accumulate compatible study summary batches across metadata pages. Preserve immediate demanded
acknowledgement and live-prefix semantics; a full buffer is an opportunity, not a new visibility
condition. Progress/live cadence follows the existing consumer demand and required durability.
A background-delay policy or unacknowledged success is not selected.

### Resumable population installation

Install run identities, point rows and associated roots/edges in bounded native transactions, with
shared source checks once per applicable page and complete immutable replay comparison. Keep
per-occurrence identity and order. Dependencies use their own bounded phase because endpoints may
not yet exist; activation requires the complete node/edge closure.

Persist an exact installed frontier/page receipt. Retry resumes that original frontier, not point
zero, and never renews creation authority or activates an incomplete population. Keep original
clock semantics per issued bounded operation; remove the single fixed 90-second clock over an
arbitrary population. Total work, resident state, response size and cancellation remain distinct.
The existing 100,000 admission bound is not a qualified scale promise.

Admission supplies compact immutable outcome facts (case identity/binding hash where applicable,
explicit absence/refusal otherwise). Lazy variants populate them when admission settles. Finalizing
and joining outcomes consumes these facts without decoding a 512 KiB descriptor. Detail browsing
stays a separate byte-bound operation with its original admission. Merely raising `LIMIT 4` is not
a correction if the query still acquires full descriptors.

### Exact cohort reads and capture

For existing frozen membership, build one exact multi-attempt manifest/protection and one
relation-shaped provider over paged locators, lazily acquiring inline/sealed batches. Reuse the
native reader/decoder, quantity checks and completion-owned permission; remove per-member full
snapshot construction and long `DataFrame::union` chains.

Increasing cohort limits requires a coherent capture, not paging live rows and calling them
frozen. Select an engine-backed immutable compact cohort selection captured atomically under the
existing SSI/read contract, followed by bounded member reads. D04 must qualify the exact native
capture against changing active studies before increasing the old cap. Preserve complete
included/unavailable membership and snapshot facts. Do not narrow to terminal studies or retry
until an active population happens to stop changing.

If the selected native capture cannot preserve that contract within admitted work, resolve the
engine/transaction realization before dependent cap changes; the finding remains open rather than
claiming cursor paging completed it. No infinite-scale or new service promise is introduced.

## Backup and exchange design

### Recoverable inventory verification and copying

Keep authenticated freeze/drain, stopped copy, collection exclusion, completion-last publication,
fresh restore identity and explicit readmission. Replace the fixed whole-inventory verifier clock
with bounded progress/checkpoints tied to exact frozen inventory, caller cancellation/budget and a
diagnosed stalled-work watchdog. Failure remains closed for explicit resume/recovery; no automatic
thaw or renewed effect authority follows from a retry.

The current native helper validates file extent and framed canonical digest, not deeper IPC row/
scientific semantics. Preserve that canonical digest owner and independent application/scientific
closure checks. Compute canonical digest and copy SHA256 during a controlled native source read;
copy in bounded streams with pre/post source identity and extent checks. Keep destination readback
where it establishes copy fidelity, and reuse its verified digest in the final manifest instead
of rereading the same immutable target for another digest. Resume revalidates changed/untrusted
premises; an old digest is not proof that an externally mutable destination stayed unchanged.

A universal one-pass claim is unselected. Payload source comparison, copied database destination
checksums and logical restore verification remain distinct guarantees. The declared power-loss
exclusion remains unchanged.

### Incremental executable closure and export

Separate compact exact member/source selection and retained protections from hydration. Admit and
encode one source revision/useful document unit at a time; release its hydrated package and encoded
bytes after last use while preserving original immutable premises/protection through final manifest.
Replace both the all-packages `BTreeMap` and `ResolvedClosure::documents` aggregate payload vector.

Use the same exact traversal with an export sink and a counting/checksum explanation sink.
Executable explanation still performs original scientific admission, then discards encoded units;
descriptive-only metadata retains its explicit narrower contract. Apply remaining output allowance
before unnecessary encoding/copy; preserve deterministic member ordering/ids, provider premises,
checksums, cancellation and manifest-last visibility. No early partial export is advertised complete.

## Packages

### D01 — Real bulk publication

Replace `surreal_execution::append_many`'s repeated single-item realization and integrate C03
placement, grouped set/frontier updates, empty close entries and cross-page summary packing. Migrate
runtime result projection, opaque completion/seed/progress appends, live and horizon publication.
Delete per-item shared-work loops after exact acknowledgements and read paths are integrated.

### D02 — Resumable run-and-point creation

Replace `create_study`'s per-point calls/fixed outer clock with bounded install pages, immutable
cursor/receipt and separate edge closure/activation. Migrate eager and lazy/variant admission,
replay, cancellation and recovery using the same operation owner; preserve original authority.

### D03 — Compact outcomes and shared cohort provider

Produce compact outcome projection at admission and migrate finalization/joined observers. Reuse
one protected relation provider for existing exact cohort membership. Delete descriptor hydration
from summary paths, four-row-driven output files and per-member full snapshots after callers move.
The existing cohort admission cap stays until D04 establishes its replacement contract.

### D04 — Coherent compact cohort capture

Implement the selected immutable engine capture plus bounded reads. A focused probe of the exact
pinned engine must distinguish coherent snapshot facts from inter-page mutation before production
cap changes. If it exposes a missing engine mechanism, settle that bounded design choice and its
required decision route; do not silently substitute weaker semantics. Derive practical admission
from actual capture/transport/resident limits, not descriptor size or an inherited unexplained 128.

### D05 — Offline integrity/copy recovery

Consume C04/E02's product helper for exact native file observations and digest work. Migrate frozen
inventory verification, copy/destination manifest construction and restore recovery, eliminating
redundant scans where the same immutable evidence can be reused. Retain independent checks with
a distinct failure purpose. Publish actionable progress/failure state and resume without thaw.

### D06 — Bounded-live selected exchange

Refactor `project_closure.rs` and `project_portability.rs` around compact closure and incremental
sinks; migrate executable explanation/export and all relevant host callers. Remove aggregate
encoded-body retention, enforce allowance before avoidable copying, and preserve reader ownership
until the final manifest. C03's locators support both physical tiers without duplicate scientific
interpretation.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| D01 | planned; C03 inline/sealed placement available | 56d | Grouped append changes actual guard/frontier/filesystem work; demand/ack semantics preserved |
| D02 | planned | 56d | Resume begins at exact installed frontier; incomplete population never activates |
| D03 | planned | 56d | Compact outcomes and one lazy relation provider replace descriptor/full-snapshot loops |
| D04 | planned; capture evidence required | 56d | Coherent live capture and bounded membership read qualify any changed admission cap |
| D05 | planned | 56d | Inventory/copy progress is resumable and integrity passes reused with distinct guarantees retained |
| D06 | planned | 56d | Executable explanation/export retain only useful hydrated units, with exact closure and final visibility |

D01 consumes C01/C03; D02 consumes C01/C02 and B03 operation facts. D03 needs B01 permission
and D02's compact admission product, not all population work. D04 may start its engine inquiry
independently of the provider migration. D05 consumes C04/E02; D06 can advance once C03's exact
locator/closure contract is ready. Backup/export are not forced behind all study work.

## Acceptance and benefit limits

**Planned:** exact replay conflict, lost ack, stale fence, interruption after each installed page,
expired issued pages and late-commit readback without renewed authority, lazy admitted/refused
variants, dependency failure, cancellation and concurrent independent lanes.
Test publication below/at/above placement/message boundaries, empty/full/partial relations, demanded
flushes, retained readers through retirement and mixed locator reopening.

Mutate live cohort facts between page reads and prove one captured fact set, preserving unavailable
members and quantities. Exercise corrupt/aliased/truncated backup files, changed destination after
checkpoint, timeout/stall/cancellation and explicit recovery without premature admission. Export
several revisions with repeated physical documents, provider premises and allowance exhaustion;
assert exact content and bounded live encoding units rather than merely small returned metadata.

Structural file/transaction/retained-buffer controls can establish removed work. Current-engine
latency, peak memory and capacity claims require representative measurements with a stated purpose.
No benchmark campaign, scale SLA or product qualification was run during authoring.

## Selected 56c prerequisites — 2026-10-26

**Implemented/Tested:** [56c Outcome](56c-canonical-contracts-and-effects.md#verification-and-outcome--2026-10-26) owns canonical declarations/settlement, bounded inline/sealed placement and the native product recovery helper at its recorded scopes. D01 consumes that placement; its grouped guard/frontier realization remains planned. D02 retains population admission, D05 retains incremental inventory/copy and frozen-tooling work, and D06 retains projection/spill ownership. Availability of C's mechanisms does not complete these D packages or establish measured workload benefit.

## Execution checkpoint — 2026-10-26

All packages in this companion are authorized for joint 56d/56e execution. The coordinator's
[joint checkpoint](56-design-principle-alignment.md#joint-56d56e-execution-checkpoint--2026-10-26)
records selected rule changes, dependency order, preservation and qualification boundaries.
Contract publication and independent E04/E05 work have begun; package completion remains open.

## D04 exact-engine choice — 2026-10-26

The direct ungated 10,000-member surrogate capture failed with a native SSI read conflict after
3.601 seconds; rollback left zero captured members while 22 writer transactions completed.
Installed `getu`/`FOR UPDATE` is optimistic, so it does not provide blocking exclusion. A separately
committed scoped-gate surrogate captured 10,000 facts and 6,666 protection references in 1.962
seconds, preserving 3,334 unavailable members through forty bounded pages while later live updates
continued. Exact replay, abort, protected-retirement refusal and release controls passed.

**Tested:** `scripts/pse-env -- .venv/bin/python /tmp/pse-active-capture-20261026/probe.py`, pinned
server revision `1ac2b1dba7cd177621a4cf5d507caebeb2414395`, binary SHA256
`f555c7819b2f8654912d5937020830397ca259421b54c9c0717784f18832707a`; both owned services
stopped. Evidence remains under `/tmp/pse-active-capture-20261026-attempt3/` and
`/tmp/pse-active-capture-20261026-guarded/`. Two earlier harness setup failures were retained.
These are surrogate engine controls, with the ungated failure preserved against the zero target;
no canonical production, crash/expiry, genuine lost-ACK or full 10,000-point claim follows.

The [independent reassessment](../design_review/reviews/design_review_product-and-workload-ownership_2026-10-26.md)
accepts the revised gated target at Proposed design evidence. ADR-0230 and blueprint §19.3/§20
now require complete writer participation, atomic facts/protection/receipt/gate closure, original
clocks and exact incarnation-changing expiry recovery before dependent implementation. No cap
change is authorized by surrogate success alone.
