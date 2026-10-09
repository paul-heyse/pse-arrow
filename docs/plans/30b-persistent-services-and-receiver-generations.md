---
title: Persistent services and receiver generations
status: draft
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
| B0 — Generation/profile contract | R0 rules, current supervisor/native identity and 28i premises. Specify service/receiver separation, configuration revision and admitted compatibility. | Actual producer/configuration mutation, unknown profile, incompatible interpretation and retained old-generation cases; required authority route precedes changed contracts. | Planned |
| B1 — Owned resident service | B0; working A1 readiness transport; D1 admission slice. Implement owned unit generation, finite readiness, explicit recovery and qualified bounded restart. | Migrate setup/start/status/readiness/backup/restore and service roots. Test independent agent exit, listener impostor, authentication/schema refusal, crash/reopen, restart exhaustion and intentional stop. Delete replaced session-coupled/transient functional startup after controls pass. | Planned |
| B2 — Independent receivers/contexts | B1; C1 context registration; working D1 role allocation; preserved 28h reference profile. Admit immutable receiver generations separately; D2 later verifies their composed placement. | Migrate default Rust/Python managed roots, worker/supervisor and fixture contexts together. Test old/new receiver coexistence, changed bytes at the same build path, stale authority, surviving descendants and reference-lane exclusion. Delete replaced whole-profile readmission coupling. | Planned |
| B3 — Topology decision and handoff | B2's actual current composition, A2 bounds and C2 isolation. Establish whether supported overlap meets existing budgets. | Use bounded current shared-service controls first. If catalog/commit contention obstructs admitted work, compare a two-service functional pool with the same total lane allocation and scientific scope; adopt it only with diagnosed benefit and positive ownership/recovery controls. Otherwise retain shared storage with that observable reopen trigger. D3 owns assembled acceptance. | Planned |

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

No service/configuration has changed during authoring. All packages remain planned;
new generations, restart and coexistence require their named working controls before
use. Plan 30 owns findings and 30d D3 supplies series-level assembled acceptance.
