---
title: "Plan 56d: Publication and workload fit"
status: complete
date: 2026-10-25
adrs: [ADR-0230]
review_sources: [docs/design_review/audit/README.md, docs/design_review/audit/system-architecture.md]
scenario_sources: [docs/design_review/audit/system-architecture.md]
doc_role: plan
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, performance, storage]
---

# Plan 56d: Publication and workload fit

## Completion audit — 2026-10-27

**Interface-checked:** independent review of D01–D06 at
`57cce1756318357a5797f99e83f1ad9115661368` supports completion at the recorded selected scope.
Native grouped append/frontiers, resumable study pages, compact outcomes/lazy exact cohort scans,
gated atomic capture/protection, fused offline integrity/copy and incremental selected-source
exchange reach ordinary consumers. The retained actual 10,000-point mixed capture and one-GiB
mixed-placement interruption/resume evidence was inspected, including refusals and original drain.
No new native/service runs were made. No additional material D gap was established. The historical
ordinary-reference activation blocker was subsequently settled by 56e's final activation; it is not
a current D blocker. The 100,000-point ceiling and wider fleet/platform scope remain unqualified;
no speed or memory benefit is measured. Unfinished series obligations remain with the
[coordinator audit](56-design-principle-alignment.md#completion-audit--2026-10-27).

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
| D01 | complete at selected native publication acceptance | 56d | Grouped append changes actual guard/frontier/filesystem work; demand/ack semantics preserved |
| D02 | complete at selected native frontier/replay/expiry acceptance | 56d | Resume begins at exact installed frontier; incomplete population never activates |
| D03 | complete at native provider and installed public workbench/CLI acceptance | 56d | Compact outcomes and one lazy relation provider replace descriptor/full-snapshot loops |
| D04 | complete at actual public 10,000-point creation/capture/provider/summary acceptance | 56d | Coherent live capture and bounded membership read qualify any changed admission cap |
| D05 | complete at selected fused-helper one-GiB recovery and interruption/resume acceptance | 56d | Inventory/copy progress is resumable and integrity passes reused with distinct guarantees retained |
| D06 | complete at native selected multi-revision exchange acceptance | 56d | Executable explanation/export retain only useful hydrated units, with exact closure and final visibility |

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

## D06 incremental source checkpoint — 2026-10-26

The selected closure now retains compact revision/result/artifact selection and original
protections. It consumes the admitted origin first, then one useful revision and current encoded
document, staging or counting/checksumming through the same sink. Sorted revision ordinals retain
deterministic member ids. Both the all-packages map and retained encoded-document vector are
removed; borrowed leased source/physical documents are copied directly without extra blob clones.
Remaining allowance bounds replay encoding and each copy before destination effects. Original
reader protections, destination verification and final manifest publication remain.

Shared contained-copy and scientific IPC lineage admission now live in product-owned
`workflow::portable_payload`, beneath diagnostic pack and project consumers. The old development-
owned module and project-to-review-pack validation dependency are deleted.

**Tested:** `just unit-package pse-runtime 'test(project_portability_tests) | test(review_pack)'`,
force-validation, default/non-native closure: 24 selected tests passed, zero failed against the
zero baseline (1,011 unselected tests). This includes exact count/export receipts, refusal before
copy, cancellation, contained-file mutation and original IPC lineage controls. Compilation took
3m56s and selected tests 14.294s on this local Linux host. Three warnings in concurrent cohort work
remain for its owner; this is not a zero-warning whole-plan result. Earlier malformed filter,
stale test-module name and source-fence interruptions were corrected; only the stated final
selection passed.

The correction was to borrow already leased document collections rather than move or clone their
backing vectors. Processing the already admitted origin first deliberately bounds live hydration
while ids retain sorted-source ordinals. Native multi-revision export/explanation, interruption
and focused benefit measurements remain for joint scope-end qualification.

## Canonical integration checkpoint — 2026-10-26

D01/D02 and the D04 lifecycle source now implement native bulk publication, durable installation
frontiers/original page grants, compact admission facts, gated capture, atomic member and parent
manifest protection, local replay/drain fencing and explicit failed-cleanup recovery. Root-owned
project/research/retention participation uses the same gate and capture protection. D03 now supplies
the compact receipt, paged public metadata and shared lazy provider; the admission cap remains 128 pending
actual 10,000-point qualification. Source maturity is not native execution evidence.

An independent bounded implementation review found a pure readiness-policy helper calling the
gate with an undeclared deadline parameter. The redundant call was removed; the effectful caller
remains guarded. The reviewer re-read that correction and found no additional high-confidence
issue in the stable bulk/install/capture subset. Incomplete D03 and deployment work were excluded;
no tests were run by the reviewer. Required native SQL/replay/expiry/drain controls remain open.

The single handwritten canonical interpretation declaration now selects `pse.substrate.v10`.
All current declarations/writers/readers/retention/backup outputs must publish before first fresh
owned-state activation. Existing selected and 56a/b/c states, artifacts and qualification evidence
remain preserved; v10 has no product qualification claim yet.

## Provider and publication checkpoint — 2026-10-26

The shared cohort provider now traverses bounded member and locator pages under one original
capture. Opaque payload references are minted only after exact captured-manifest validation and
retain original custody through physical readers. Included, unavailable and unvisited members
remain distinct through predicates, limits and concurrent scans. Existing shared scientific
decoding and quantity/permission checks remain the payload interpretation. Public consumers use
compact receipts and explicit bounded metadata pages; optional NDJSON export publishes its final
receipt only after the original capture's pages settle.

A focused independent implementation review of the current provider, captured payload seam and
Rust/Python consumers found no high-confidence contract defect. It was read-only and established
no native SQL, DataFusion execution, cancellation-race or measured-memory result. Actual native
controls and the selected mixed-availability 10,000-point journey remain required. Full generation
now publishes the closed owner-declared export envelopes and all ten selected generation targets.

The new product initializer successfully installed the v10 schema in the fresh owned disposable
`build/qualification/56de-v10-gdxn2u4_` deployment. This establishes initialization only; the earlier
56a/b/c deployments remain untouched and retain their original qualification boundaries.

## Native integration repair checkpoint — 2026-10-26

The first prepared canonical publication/install/capture selection ran 25 tests: seven passed
and eighteen failed against the zero failure target. Seventeen failures shared a new installation
root-read query rejected by the pinned engine before population creation; its static `FOR UPDATE`
target checker accepts record literals, parameters and function calls, but rejects object-field
expressions such as `$root.id`. The generated source now constructs that same exact root through
`type::record`; regeneration preserves the original conflict registration and bulk insertion.
The remaining busy-publication deadline assertion needs exact returned-error and cleanup timing
investigation; unknown effects must remain conservative. The owning failure log is
`/tmp/pse-56de-native-publication-controls.log`. Dependent native and 10,000-point acceptance
remain open until the corrected paths execute successfully.

## Joint acceptance checkpoint — 2026-10-26

All D01–D06 mechanisms and migrated consumers are implemented. Native controls exercise grouped
publication, original installation frontiers/grants, capture writer participation, exact protections,
shared provider and incremental exchange. The public 10,000-point route now passes creation,
coherent changing-live capture, aggregate/detail reads, cancellation and summary completion; the
sole shared admitted cohort maximum is 10,000. Earlier cap-128 and surrogate-only statements above
record their historical checkpoints. The 100,000 population ceiling remains unqualified.

Independent source reconciliation exposed the remaining D05 repeated integrity scans. Canonical
framing and archive SHA256 now consume each protected source/readback stream together; fresh copies
retain four physical phases and exact resumed copies two. Original file identity, locks, refusal
priority, independent unpublished/published readbacks, clocks and resume checkpoints remain.
Both helper-selection entrypoints also refuse changes before modifying an original pending backup.
Earlier positive large archives remain bound to their original helper bytes.

The current helper's fresh one-GiB mixed-placement recovery and interruption/resume, corrected
installed public workbench journey and final shared scope-end checks are still in progress.
Package implementation and earlier focused passes do not close these remaining acceptance tasks.

## Verification and Outcome — 2026-10-26

### What was built

**Implemented:** D01–D06 use native grouped publication, original resumable installation pages,
compact admission outcomes, atomic gated live capture/protection, one lazy relation provider,
recoverable fused integrity/copy phases and incremental selected-source exchange. The replaced
per-item shared-work loops, per-point creation path, full-descriptor summary reads, per-member
full snapshots/union chains and aggregate encoded-source retention are removed. Current fresh
interpretation is `pse.substrate.v10`; prior qualification states and original archives remain
preserved. Proposed ADR-0230 retains its formal status.

**Tested:** selected local Linux correctness uses explicit force-validation, the pinned nightly,
current native providers and owned v10 state. The initial joint native selection passed 78 of 82;
its four corrected cohort/observer controls then passed 4/4. This is composite evidence, not a
clean monolithic 82-case run. `just unit-native-package pse-runtime canonical-tests
'test(workbench_actual_10000_)' --profile local --success-output immediate` subsequently passed
the single actual public 10,000-point journey. The later five-control selection
`test(workbench_actual_10000_) | test(workbench_public_cohort_) | test(workbench_capture_continuation_) | test(study_observer_public_)`
passed 5/5 against zero, including a repeated actual 10,000-point run and original capture release
after caller close followed by terminal drain. It covers bounded creation, changing-live capture,
forty member pages, one shared provider, exact quantities, late completion remaining unavailable
in the original capture, cancellation and final compact summary. The capture includes nine usable,
two failed, one active and 9,988 unattempted points; the final summary includes ten usable points.
This does not claim 10,000 numerical solves or qualification of the 100,000 admission ceiling.

**Measured:** that selected 10,000-point run observed creation 52.340 s, capture 1.639 s, aggregate
5.579 s, cancellation 23.446 s and summary 8.373 s. Accounted capture/aggregate pool peaks were
22,872,064/58,680,171 bytes, with 103,314 bytes maximum retained member page. These are local
single-run phase observations under a 4 GiB fixture pool, not comparative speedup, private RSS,
a workstation cap or an SLA. Native selected multi-revision exchange retained 543,259 source bytes
across twelve source members/two revisions; explanation/export took 2.043403/2.450874 s with
accounted phase peaks 34,883,551/34,987,639 bytes. Process-lifetime RSS observations are separate
and do not establish phase RSS. Original admission, exclusion of the unselected third revision,
output exhaustion and cancellation before the next hydration were exercised.

**Tested:** `just unit-package pse-operations
'test(canonical_files::offline::tests::) | test(canonical_files::tests::sealed_)'` passed 22/22;
the supervised backup image/resume selection passed 19/19. Canonical framing and archive SHA256
share each controlled read while fresh payload copies retain four physical phases and exact
resumed copies two. Source extent refuses before payload reads and original source/destination
refusal priorities remain independent. `scripts.tests.canonical_recovery_check` with explicit
`--backup-scale-bytes 1073741824`, the retained private fixture and rebuilt product helper passed
SIGKILL/reopen, backup, offline lower/original-cap recovery, fresh restore identity and original
semantic checks. Sixty-four sealed 16 MiB bodies plus eight inline bytes matched all 65 original
descriptors and exact bytes after reopen/restore. The fixture remains Partial/SeedUnavailable;
opaque storage bodies establish no numerical success.

The fused helper selected for the large positive controls has SHA256
`f79f2bf0c0e6c0b6b4cabd0609183294985df24cc39c85dce977f63a279cb7d8`.
An authenticated helper SIGKILL during the next copy after one acknowledged entry left the backup
unpublished and source stopped. Explicit normal resume preserved the prefix inode, completed 65
entries and independently verified the archive. Wrong destination and later changed destination
bytes refused; the positive original backup and receipt stayed unchanged. The negative destination
is intentionally retained corrupt. Source, positive backup and restored state are retained under
`build/qualification/56de-fused-scale-20261026T174725`; logs are
`/tmp/pse-56de-fused-scale-journey.log` and `/tmp/pse-56de-fused-backup-interruption.log`.
Earlier positive archives remain bound to their original helper bytes.

**Tested:** the final installed correctness selection passes 8/8 against zero using
`scripts/pse-env --native -- .venv/bin/python -m scripts.python_tests`
with `test_study_workbench.py`, the retained-revision capability adapter, `test_project_cli.py`
and `test_project_portability.py`, on owned v10 state and a fresh owned pytest directory.
The artifact SHA256 is `f2f0dd191d89013d402142e03acc38cd74746c2134c840d0ae9bb9625ac01c2d`,
with `force-validate,native-solvers,solver-highs`. Actual two-point execution preserves the original
unavailable capture, then returns exact completed quantity/unit rows and aggregate values through
Python, NDJSON and Arrow IPC. CLI output exhaustion retains the original refusal; terminal cleanup
joins original reader releases. Actual completed-parent observation preserves status and truthfully
reports no parent numeric history. The separate actual STDIO MCP and pack/CLI/offline/import/reopen/
corruption journeys pass in the same selection. Log: `/tmp/pse-56de-public-product-final.log`.
The original failed runs remain separate evidence.

**Tested:** the final native selection
`test(study_summary_conclusion) | test(study_observer_terminal_parent_conclusion_has_no_numeric_history) | test(study_observer_public_disconnect_reconnect_reads_full_history_or_reports_missing_prefix)`
passes 4/4 through `just unit-native-package pse-runtime canonical-tests`, explicit force-validation,
local profile and the original owned v10 state. Strict current summary tuples, malformed/versioned
refusals, real parent finalization, repeated polls and original point history/missing-prefix behavior
are exercised. The pure `pse-project` dual action/drain failure control passes 1/1. Logs are
`/tmp/pse-56de-final-parent-observer-controls.log` and `/tmp/pse-56de-cli-dual-failure-unit.log`.
Shared scope-end reconciliation is recorded in the final closeout below.

### A mistake made and corrected

Independent reconciliation found that D05 still performed redundant canonical/SHA scans despite
its initial implementation claim; those scans are now fused with distinct readbacks preserved.
Real large recovery also exposed intermediate backup directories created with the caller's umask;
all newly created ancestors now retain private owned directory custody. Two helper-selection
entrypoints could invalidate an original pending backup before refusal; both now validate its
original identity before any write. Actual public acceptance exposed a completed parent summary being read as a scientific
completion receipt; its strict study-owned conclusion decoder now preserves the parent status
and absent numeric-history meaning while point receipts retain their original path. It also
exposed queued capture cleanup racing CLI disconnect; the terminal owner now joins original releases and SDK callbacks without relaxing
live-reader refusal. The expanded test also corrected its own queue-driving, feasible-point,
result-document and relation-specific `step:0` selection assumptions. Original failures remain
in the scope's logs rather than being promoted to successful qualification.

### Deviations from the plan, deliberate

The ungated native SSI mechanism failed, so the independently reviewed committed gate replaces it;
optimistic `FOR UPDATE` is not described as blocking exclusion. The selected cap is the shared
public 10,000-member bound actually exercised. Large storage qualification explicitly uses private
opaque bodies and original descriptors, independent of numerical solver success. No universal
one-pass, full feature matrix, solver-fleet/parity, distribution, desktop attachment or other-platform
claim follows. Full F/G work retains its owner.

### Final closeout and handoff

**Implemented/Tested:** D01–D06 are complete for the selected local functional scope and
acceptance described above. Enduring publication/capture/offline contracts are in blueprint
§19.3/§20.6; this companion remains retained for active E/F/G and proposed-ADR readers.
E02/E03 ordinary receiver activation is separately blocked at E's Outcome; it does not turn the
passing capture, publication, exchange or private recovery controls into receiver qualification.

**Tested:** final `just governance` passes governance controls, all generated comparisons and
family agreement against zero. Full native/product/private-development Clippy passes with
`--workspace --all-targets --no-default-features`,
`pse-runtime/product-operators,pse-runtime/native-solvers,pse-runtime/canonical-tests,xtask/canonical-tools,xtask/full-codegen`,
explicit force-validation and `-D warnings`. This includes the relocated development families;
it is one selected feature graph, not a feature powerset. Final default workspace `just lint-native-data`, `just lint-native-contracts` and
`just clippy-no-default` also pass; G's selected contribution retains their exact scope.
`just ready`, `just lint-py`, `just typecheck` (including warnings), `just lint-license` and strict `just audit-shear` pass against zero. Pyrefly reports
zero diagnostics with seven existing suppressions; REUSE covers all 12,589 files. Unused xtask
optional dependencies and empty generated modules were removed through the generator and lockfile,
with full ten-target regeneration and generated equivalence passing. `just docs` publishes all
683 chapters and scoped search after the host/docs interpreter correction recorded in E; the
original failed documentation attempt remains separate evidence.

The original `just hygiene` had seven unsuccessful leaves. Corrected affected leaves pass,
except global `just lint-typos`, which still reports one known `ORDER` misspelling in unrelated dirty
`history.txt` (zero target). That user-owned file remains unedited and uncommitted. The scoped
spelling run excluding that file passes. This is composite scope-end evidence, with the original
failures retained, and no clean full-hygiene claim. Logs include
`/tmp/pse-56de-governance-closeout.log`, `/tmp/pse-56de-final-full-private-native-clippy.log`,
`/tmp/pse-56de-typecheck-closeout.log` and `/tmp/pse-56de-audit-shear-repaired.log`.
The 505-case feature campaign, full-series F/G, full parity/fleet, distribution and other platforms
remain not_run in this selected closeout.

**Tested:** the owned v10 test deployment was quiesced, authenticated-drained and stopped after
passing public/native tests, with all failed-run databases and receipts retained. Positive private
backup/restored services are also stopped; the corrupt negative archive remains explicitly corrupt.
The attempted reference deployment never started. All earlier qualification states and unrelated
live services remain preserved. Work stops after committing the integrated changes and this
handoff; E's activation blocker and wider F/G work remain explicit.
