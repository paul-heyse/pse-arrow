---
id: ADR-0214
title: Select persistent SurrealMX and bounded gRPC canonical execution
status: proposed
date: 2026-10-21
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, DP-09, PS-09]
blueprint: [§3.3, §19.3, §20, §20.5, §20.6, §23.2]
review: docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md
evidence: Tested — selected Plan49a native store, durability, gRPC, sixteen-lane and Python journeys on local Linux, 2026-10-21; exact commands and composite limits at Plan49 Outcome.
supersedes: []
superseded-by: null
revisit: Actual integration cannot preserve required conflict, bounded response ownership or durable receipt semantics.
verification: Plan49a STO01–STO08 qualify the patched server, lifecycle, bounded gRPC, fresh activation, recovery and sixteen-lane publication.
standard: Core-3.4 / Efficient-Architecture-1.0 / Process-Simulator-1.5
scenarios: [PSE-S04, PSE-S05, PSE-S07]
---

# ADR-0214: Select persistent SurrealMX and bounded gRPC canonical execution

## Context

The integrated review/audit motivate generated lifecycle, typed refusals and native
record/graph access. The maintainer accepted Plan49 RC01–RC21, authorized all Plan49a
and selected fresh regeneration of all existing data. This supplements ADR-0164;
implementation authorization does not change either record's proposed status. On
2026-10-22 the maintainer explicitly selected RC22: same-source Surrealism study policy.

## Scope

Amend blueprint §3.3, §19.3, §20, §20.5, §20.6 and §23.2 for the engine, transport, physical
lifecycle and refusal contracts. Other Plan49 work retains its existing owner.

## Drivers

Keep one supervised authority, sixteen durable lanes, independent callers and caller-
independent execution. Registry declarations own storage; Rust owners retain scientific
meaning. Original deadlines, receipts, protected payloads and physical drain stay truthful.

## Options

RocksDB/WS avoids integration but misses the selected capabilities. Per-process embedded
MX creates competing lifecycle authority. Select narrow source-built MX and native gRPC
integration within existing owners. This accepts resident-memory, source-build and codec
obligations; performance improvement remains unmeasured.

## Outcome

Execute readiness policy inside the generated SurrealDB transaction through a pinned,
read-only Surrealism module. The new `pse-study-module` boundary crate consumes the existing
pure `pse-model::study_policy` source; native claim/execution callers consume that same
source. No second SQL policy or persistent module state is introduced. Acquire one bounded,
deduplicated complete scope, compute decisions and write hints/cursors atomically. Claims
still validate authoritative facts; readiness grants no execution permission.

Pin the SDK/ABI and immutable module artifact hash with schema activation. The supervised
server enables the required module/file features and admits only the private immutable
artifact directory through a read-only bucket. Install and verify the module before schema
activation; missing or mismatched artifacts fail closed. Explicit finite controller, memory,
execution-time and scope limits apply; deny arbitrary SQL, network, scripting and module KV.
Trap, timeout and malformed returns abort the enclosing transaction. Uncertain RPCs retain
an exact durable operation identity and receipt reconciliation under the original deadline;
module failure is never evidence of absent effects. Retire the external scientific-scope
transfer and stale-decision apply protocol when the replacement controls pass. Preserve
only the smaller native intent/settlement owner needed for uncertainty, rather than a
parallel production execution path.

Build pinned SurrealDB3.3.4 with exact MX1.0.0 and source/patch/resolution/binary provenance.
First integrate the version under explicit SI with necessary owned-value conversions;
then select SSI, finite memory/transaction age, synchronous AOL/EveryAppend, snapshots
and bounded stats. SQL VERSION stays unsupported; domain history retains separate meaning.

Use one gRPC production path with bounded application/control admission, original clocks,
truthful dispatch uncertainty, acknowledged session restoration, response allocation
ownership and physical drain. Never automatically replay mutations or publish provisional
stream rows. Native engine failures carry explicit stable codes across the server/client
boundary, separately from generated domain THROW codes and their native is_thrown decoder.

Generate native physical lifecycle constraints/refusals once. Keep explicit observation
projections, full-u64 generations, mutable fences, guarded commit-order publication and
creator/reclaimer guards. Migrate direct records/paging before index retirement. Attributed
edges retain identity; enforced dependencies use nodes-first bounded ingestion before
edge installation and complete-graph activation, including interruption/replay controls.

Fresh fenced installation regenerates existing data. No old-data import/backfill or
rollback is required. New data still requires durable recovery and closed backup/restore.
Activation consumes actual shared interfaces, not all RES/TOOL packages by association.

### Consequences

An SDK bump alone cannot upgrade the engine. Resident/snapshot/replay work must fit the
outer cap. gRPC requires real codec/session/disconnect integration, using an owned HTTP/2
connection driver where tonic Channel cannot establish physical drain. Server gRPC sessions
are owned by the actual physical connection: another connection cannot reuse their IDs,
and disconnect reclaims that connection's sessions/transaction/live-query state. A new
connection restores acknowledged session setup under the original clock rather than
resurrecting a dead connection's session. Cleanup retains ownership through actual drain. Move affected
consumers together and remove replaced production paths after replacement controls.

### Compensating controls

Exact pins, executable provenance, finite limits, typed failures and receipt settlement
bound failures. SSI retains shared conflict keys and external-effect fences. Unknown and
schema errors fail closed; retryability grants no replay. Fresh incarnations fence old callers.

### Confirmation

Interface-checked support is the integrated review/audit, source owners and tagged sources
retained in Plan49a. **Tested, selected local Linux scope, 2026-10-21:** native synchronous
durability/fault controls, bounded gRPC peer/reconnect controls, lifecycle/guard controls,
sixteen actual native owners, independent caller/worker continuation, new-data restore and
Python result/study/interruption consumers. Exact commands, initial failures and affected
reruns, static results and limits belong to the [Plan49 Outcome](../plans/49-library-native-remediation.md#outcome--plan49a-canonical-store).
No performance, hardware power-loss or whole-product qualification is inferred.
**Interface-checked, RC22, 2026-10-22:** the tagged 3.3.4 module executor composes a read-only
export with the current transaction and query timeout; SDK 0.6.4 supports the selected WASI
boundary. The focused I26 assessment in Plan49a supplies the architectural argument and
required native/module equivalence, rollback, resource, activation and uncertainty controls.
RC22 is selected; module implementation and assembled qualification remain open. No speed
or memory improvement is claimed. Status remains proposed until the decision-PR route
changes it.

## Pros and cons

Native capabilities fit existing owners without another daemon/catalog, at the cost of a
maintained narrow upstream patch and explicit resident/RPC capacity obligations.

## More information

- [Plan49 decisions and status](../plans/49-library-native-remediation.md)
- [Plan49a and tagged sources](../plans/49a-surrealmx-canonical-store.md)
- [Recommendation audit](../design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md)

## Status history

- 2026-10-21 — proposed before dependent implementation; maintainer-selected full execution
  and fresh-data hard pivot recorded without formal ADR acceptance.
- 2026-10-21 — implementation and selected test evidence recorded; formal status unchanged.
- 2026-10-22 — maintainer selected RC22; same-source bounded Surrealism readiness and its
  boundary crate recorded before dependent implementation; formal status unchanged.
