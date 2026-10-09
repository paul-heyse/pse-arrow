---
title: Persistent services and receiver generations
status: done
date: 2026-10-08
adrs: [ADR-0164, ADR-0166]
review_sources: [docs/design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md]
scenario_sources: [docs/design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#4-revealing-scenarios]
---

# 30b: Persistent services and receiver generations

## Responsibility and foundation

This companion owns WP05's service/receiver separation and AE-26's bounded topology
investigation. [Plan 30](30-websocket-and-persistent-agent-environment.md#finding-dispositions)
owns their disposition. Extend the existing `surreal_server` owner and immutable native
generation machinery; do not create a second service supervisor or scientific dispatcher.

**Implemented foundation, source-inspected:** the server uses synchronous disk-backed
RocksDB, owned state/configuration, systemd units and actual process/receiver checks.
Its disk is already persistent; `Restart=no` and artifact-coupled readmission do not
establish continuously available cross-session service. The existing exact primary/
observer topology remains a valuable qualification lane, not the required shape of
all ordinary functional work.

**Proposed:** one shared functional service initially, with isolated databases selected
by 30c. A separate timing service/context avoids functional fixture churn. The unchanged
reference profile has its own admitted lifecycle. Incompatible interpretations/releases
and destructive server tests use distinct owned service generations; there is no
automatic pool expansion, universal server-per-test policy or database-clone dependency.

## B0 — Generation and compatibility contract

Separate the following lifetimes without weakening any association:

| Owner | Identity and admission | Ends when |
|---|---|---|
| Disk/service generation | Exact server artifact, backend/options, protected state directory, schema/interpretation and endpoint/authentication configuration | An explicit owned lifecycle drains dependents and stops/retires that generation; disk retention is a separate decision. |
| Receiver generation | Immutable worker/supervisor artifacts and configuration, producer receipt, receiving validity premises and finite runtime allocation | Its admitted operations and descendants drain; old bytes cannot inherit a new association. |
| Invocation/test context | Service generation plus unique database and reader/writer capabilities | 30c establishes terminal disposition, drain and any retention obligations. |

Changing compatible worker bytes admits another receiver generation without stopping
the storage service. Changing storage interpretation or server release requires a
new explicit service-generation decision and validation; neither a new client path
nor a live listener grants compatibility. Ordinary open does not create or upgrade
schema. Test provisioning is an explicit create operation before consumers open.

Use existing artifact-specific identity and 28i's actual receiving lifetime. Preparation
validity, canonical roots and fresh 28j publication remain separate. Receiver reuse
checks actual admitted bytes/configuration rather than mutable build-path equality.
Publish receiver artifacts into immutable generation-owned locations before launching;
source rebuilds cannot overwrite a live receiver's executable or supervisor closure.

B0 specifies the profile revision that separates service configuration from receiver
selection, with an explicit validated upgrade/readmission route for an operator-selected
existing profile. Preserve unknown and older saved profiles until selected. There is
no permissive reader that silently invents missing identity or capacity fields.

## Resident service lifecycle and operator surface

Use the systemd user manager for functional service residency independent of Codex
session exit. Materialize an owned unit from admitted immutable service/supervisor
artifacts, enable functional startup for the user manager, and verify the actual
manager lifetime/linger capability when continuous availability across logout is selected.
The initial support guarantee is across agent sessions and user-manager restart with
the stored profile; host reboot availability requires the verified user-manager startup
route, not a claim derived from a transient unit. Timing/reference receivers start only
on admitted demand; ordinary services do not start scientific work at boot.

User-manager startup, restart-on-failure and explicit recovery consume D1's service
admission before launching. A parked service cannot recreate itself during an
exclusive mode. Its startup clock begins once and covers admission and readiness;
retry does not renew that same attempt's clock. Intentional parking suppresses its
owned restart/start route until the admitting owner permits restoration.

Readiness separates process/listener ownership, authenticated protocol readiness,
interpretation/schema admission and receiver admission. Probe those properties with
finite clocks and expose their distinct failure reasons. A socket accepting connections
is insufficient, and readiness never installs schema or reruns science.

The target functional unit uses bounded restart-on-failure: initially three starts
within five minutes with a five-second restart delay. Enable it only after B1's
process-crash/reopen and uncertain-effect controls qualify recovery. Exhaustion leaves
an observable failed generation for explicit recovery; there is no restart storm or
automatic deletion/rebuild of its database. Intentional lifecycle stop does not restart.

Extend existing commands with typed selection/status/ensure/drain/recover operations
for service and receiver generations. A thin invocation entrypoint returns the selected
context, evidence location and capacity owner supplied by B/C/D. Redact credentials
and keep authentication private. Use XDG state/runtime placement and resolve checkout
paths through existing root discovery rather than introducing machine-specific repo
path literals. Scientific retry, retention and schema policy do not move into the CLI.

Application RPC is authenticated loopback native WS. Installation, stopped coherent
backup/restore and explicitly supported fixture loading are administrative operations.
Use existing CLI/file control-plane capabilities where required; do not imply the
checked WS SDK supports backup/import/export. An import into an existing database is
not reset. Setup/restore validate their actual resulting interpretation before open.
Retain the supported process-restart durability contract and power-loss exclusions.

## Receiver coexistence and transition

Functional receiver generations are selected per isolated context and admitted under
30d capacity. Several compatible receivers may borrow one service, but each native
group has a finite allocation and its own exact database/runtime association. A shared
physical server does not authorize sharing a mutable worker or problem head.

Keep one exact primary and observer for reference qualification. Independent functional
contexts cannot borrow its native capacity or change its database. Generation handoff
lets existing work finish under its admitted artifact and makes new work choose the
new generation. Uncertain effects settle by their original IDs. A stopping observer
does not destroy a surviving native owner.

Inventory existing live services and saved configurations read-only before any adoption.
Classify owned, externally selected and unknown state explicitly. Reuse only a selected
compatible service through validation; otherwise create the new owned generation.
No existing server, saved state, archive or worktree is stopped or removed merely to
make room. Service stop/reconfiguration affects only its established ownership and
admitted dependents. If preserved external load prevents a profile, admission reports
that boundary instead of quietly shrinking the scientific workload.

## Packages and bounded topology investigation

| Package | Inputs and delivered behavior | Migration/deletion and focused acceptance | Status |
|---|---|---|---|
| B0 — Generation/profile contract | R0 rules, current supervisor/native identity and 28i premises. Specify service/receiver separation, configuration revision and admitted compatibility. | Actual producer/configuration mutation, unknown profile, incompatible interpretation and retained old-generation cases; required authority route precedes changed contracts. | Implemented; focused controls and selected D3 handoff Tested |
| B1 — Owned resident service | B0; working A1 readiness transport; D1 admission slice. Implement owned unit generation, finite readiness, explicit recovery and qualified bounded restart. | Migrate setup/start/status/readiness/backup/restore and service roots. Test independent agent exit, listener impostor, authentication/schema refusal, crash/reopen, restart exhaustion and intentional stop. Delete replaced session-coupled/transient functional startup after controls pass. | Implemented; focused controls and selected D3 handoff Tested |
| B2 — Independent receivers/contexts | B1; C1 context registration; working D1 role allocation; preserved 28h reference profile. Admit immutable receiver generations separately; D2 later verifies their composed placement. | Migrate default Rust/Python managed roots, worker/supervisor and fixture contexts together. Test old/new receiver coexistence, changed bytes at the same build path, stale authority, surviving descendants and reference-lane exclusion. Delete replaced whole-profile readmission coupling. | Implemented; focused controls and selected D3 handoff Tested |
| B3 — Topology decision and handoff | B2's actual current composition, A2 bounds and C2 isolation. Establish whether supported overlap meets existing budgets. | Use bounded current shared-service controls first. If catalog/commit contention obstructs admitted work, compare a two-service functional pool with the same total lane allocation and scientific scope; adopt it only with diagnosed benefit and positive ownership/recovery controls. Otherwise retain shared storage with that observable reopen trigger. D3 owns assembled acceptance. | Implemented; focused controls and selected D3 handoff Tested |

A pool partitions the existing functional service allocation rather than multiplying
full server/worker envelopes. If the two-service comparison cannot fit that budget,
declare and admit a changed profile through D0; no invisible resource addition follows.
Incompatible release or destructive lifecycle testing already justifies an isolated
service independently of throughput findings.

Target supervisor unit checks (`scripts/tests/test_surreal_server.py`) and actual owned
server recovery controls, then `just unit-package` for affected receiver/canonical
contracts. Process-kill tests establish only their process-restart conditions; a fake
ready response cannot qualify actual storage recovery or scientific execution.

## Verification and checkpoint

**Proposed:** B1/B2 qualify lifecycle independence without losing exact producer,
interpretation, capacity or canonical effect authority. B3 closes AE-26 with a current
selected topology and evidence or a reasoned retained topology plus explicit trigger;
it does not rewrite Plan 29's old measurements or resurrect its fixed DDL defect.

The implementation checkpoint records the exercised scope. Plan 30 owns findings and
30d D3 supplies series-level assembled acceptance; bounded lifecycle controls do not
qualify unexercised scientific consumers or the reference lane.

## Implementation checkpoint — 2026-10-09

Owned user-manager units and immutable supervisor/worker closures are implemented.
Contexts admit independent receivers and producer receipts. All-context lifecycle
reservations gate registration and pending launches; offline backups include immutable
closures and restored paths are rerooted. Authentication readiness and explicit
selection-account migration are separate from listener and schema admission. The frozen
closure includes the executable compiler-cache entrypoint and checks its execution mode.
Explicit startup clears a deliberate park only through admitted startup/readiness;
automatic `_serve` retains its parked guard. `--no-resident` makes an owned disposable
control ineligible for resident restoration. CLI observers consume the selected profile,
including the existing `plan28-reference` to `reference` selection.

**Tested, bounded lifecycle scope:**
`scripts/pse-env --resource-class light -- .venv/bin/python -m scripts.tests.surreal_fixture_check`
passed on a new owned released-server fixture with a separately admitted storage role.
It exercised authenticated native-WS readiness, a finite toy worker and launcher-death
slot fence, SIGKILL/reopen, exact acknowledged/unknown-effect readback, one actual
automatic restart, coherent offline backup/restore, and the restored validation gate.
The backup unit control also made the original state directory unavailable before
verifying the restored immutable closure and rerooted managed paths.

The subsequent exclusive-lane control
`scripts/pse-env --resource-class exclusive -- .venv/bin/python -m scripts.tests.surreal_fixture_check --bounded-b-controls`
used one 144 GiB caller envelope, a small owned storage fixture and two declared 32 GiB
functional toy groups. Its receiver control passed: actual `/proc` executable bytes
and paths, frozen producer receipts and receiving environment associations, distinct
context/generation/unit identities, mutation of the same newly owned build paths,
stale selected-byte refusal, one unchanged storage invocation and independent drain.
It does not establish scientific primary startup or mathematical/runtime-pool behavior.
Its positive proof is retained at
`build/plan30-b-closures-20261009T093200/passed.json`.

**Tested, process recovery scope:** the B1-only released-server control passed product
`qualify-recovery` on its own immutable generation: acknowledged bytes survived a real
process crash; an abandoned acknowledgment was reconciled by the original operation ID
without replay. Its matching positive proof is retained at
`build/plan30-b-recovery-qualified-20261009T094700/passed.json`.
Only the failing restart-exhaustion control was then rerun, through the exclusive
`scripts/pse-env` boundary against that same retained qualified fixture. It passed three
actual started invocations, the actual restart counter of three, an exact unit-specific
timestamped start-limit denial, six seconds of failed/empty stability, explicit recovery
with exact readback, and six seconds without restart after intentional stop. The proof is
`build/plan30-b-restart-exhaustion-20261009T095500/passed.json`. The unchanged policy is
three starts in 300 seconds with a five-second restart delay. This host retained
`Result=signal` on exhaustion; the classifier uses the actual denial and runtime
identities rather than inventing a `Result=start-limit-hit` value.

The actual controls exposed and corrected a 15-second abrupt-stop limit that was shorter
than the unit's 45-second stop budget, plus graceful stop skipping deactivating units.
Stop now shares one original 90-second local drain clock across IPC and physical drain.
Focused controls cover that clock, a dead observer leader's surviving recorded cgroup,
parked startup, disposable residency, timestamped denial classification and observer
profile routing. Failed/incomplete fixture directories remain intact. Positive compact
proofs require actual context/kernel drain; only successful newly owned disposable
fixtures may be removed. Recovery evidence remains bound to its exact admitted
generation; explicit readmission resets eligibility rather than transferring old proof.

The composed lifecycle guard also recognizes registered native-free database borrowers
and actual live database/control cleanup owners; a drained fixture flag does not release
its active cleaner. C1 publishes registration and cleanup claims under the same short
service admission exclusion, with IPC outside metadata locks. Targeted supervisor and
cross-owner cleanup controls exercised refusal while those owners are live, and preserved
the independent treatment of evidence-file cleanup and dead-cleaner settlement.

The reference startup failure exposed a concrete placement mismatch: execution storage
inherited SMT siblings while owned-listener readiness required its declared physical
role cores. Storage now uses that exact role affinity, and explicit reference demand
starts the unchanged reference service before observer admission, normalizing a selected
receiver to its actual service owner. The ordinary start boundary retains restored
validation and active-unit readiness refusal; unattended `_serve` still cannot unpark.
One focused affinity control and four focused reference-demand controls passed. Actual
reference startup reached native WebSocket readiness under its unchanged 160 GiB
aggregate/16 GiB storage allocation. The preserved receipt at
`build/plan30-reference-affinity-20261009/passed.json` names the frozen generation and
invocation, followed by actual terminal/context drain. The controller failed afterward
while rendering evidence with a wrong resource key; the proof was recovered from its
matching persisted readiness receipt and terminal observations without repeating startup.
The later demand-factory composition is covered by focused controls; D3 supplies its
assembled native journey. These receipts do not transfer recovery eligibility to a
newly readmitted generation.

B3 retains shared storage for the current supported selection. C1's actual overlapping
ownership/collection controls and five passing ordinary Python scientific journeys
support that scoped choice. The exclusive native-Python selection's original receipt
is `build/plan30-d3-python-ordinary-20261009-1/native-python.{xml,terminal.json,native.json}`:
five passed, one producer-dependent identity was deselected and two study setup errors
were repaired by affected-only reruns, against a zero failure baseline. The original receipt is partial; the later seven-pass composite is recorded in the parent Outcome. Those setup failures identified nested
collection replacing the parent invocation's node inventory; C1 repaired that ownership
defect. It was not evidence of service catalog/commit contention. The explicit reopen
trigger remains diagnosed catalog/commit contention obstructing admitted work within
its existing budgets, at which point the bounded two-service comparison applies.
This is not qualification of full scientific overlap, topology performance or host-crash
durability.

The selected D3 native/Python failures and assembled handoff are complete through failed-only retries. The unchanged reference allocation and shared-service contention trigger remain; broader Plan 28 campaigns remain paused.

## Outcome (recorded after implementation)

**Implemented:** this packet's target mechanisms and required consumer migration are
complete. **Tested:** the focused controls above and the selected assembled D3 journeys
passed against the zero failure baseline under their recorded conditions. The
[series Outcome](30-websocket-and-persistent-agent-environment.md#outcome-recorded-after-implementation)
owns the repaired composite results, commands, current storage restart proof and
qualification exclusions. Earlier failed receipts retain their original outcome;
there was no restart of the former full Python suite or broader Plan 28 campaign.

A mistake made and corrected, and deliberate deviations, are recorded at that same
series Outcome with their owning repair. Enduring contracts and operation guidance
live in blueprint §20.6/§24.1 and the substrate/environment/validation guides. This
completed handoff remains while retained Plan 28/29 readers depend on it; it is not an
active implementation backlog. Benefit measurement and the conditional topology
investigation retain their explicit authorization and observable triggers.
