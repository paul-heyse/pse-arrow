---
title: "Plan 49h: Qualification and design publication"
status: in-progress
date: 2026-10-21
adrs: []
review_sources: [docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md, docs/design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md]
scenario_sources: [docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md]
doc_role: plan
doc_owner: docs/plans/49-library-native-remediation.md
doc_topics: [design, validation, performance, documentation]
---

# Plan 49h: Qualification and design publication

Execution state and pause handoff: [the coordinator](49-library-native-remediation.md#pause-handoff--requested-closeout-2026-10-21).

## Purpose and evidence boundary

Establish that the selected Plan 49 target works as an assembled system and that its
claimed benefits have the evidence they require. The
[coordinator](49-library-native-remediation.md) owns state and the combined completion
decision; this document specifies qualification, measurements and enduring publication.
It does not repeat individual package test lists or maintain another progress ledger.

**Proposed:** all future acceptance below. **Interface-checked:** the source review,
net-change companion, published probes, current validation guide and relevant source
owners were read during authoring. No product test, store migration, native simulation
or performance measurement is claimed by publishing the plan. Original probe receipts
retain their original engine, fixture, version and date; they are neither rerun nor
promoted to assembled evidence.

The governing standard is Core 3.4, Efficient Architecture Heuristics 1.0, Process
Simulator 1.5 and the pse-arrow binding. The source's S01–S11 remain source definitions;
the following table explains how selected work exercises them. A qualified package is
not automatically a resolved cross-area finding. G4/G6/PS-G1/PS-G2 were unresolved in the
review and remain so until evidence in their actual scope establishes a narrower claim.

## Qualification design

Use compile checks and revealing unit controls with each functional package. Run assembled
journeys and hygiene/static/manual checks once the selected functional scope is implemented,
under AGENTS.md's execution rhythm. A failed command is reported against zero; repair the
failure and rerun the affected command rather than establish a tolerated baseline.

The engine/transport cutover additionally consumes its own targeted crash, receipt and
drain controls before activating the replacement. These test the replacement mechanism;
they do not replace the assembled campaign or trigger broad integration suites mid-plan.

All actual correctness closures select explicit `force-validate`. Use the pinned toolchain
and tools through `scripts/pse-env`; linked native and Python journeys use the native route
where the recipe requires it. Never invent an environment bypass to get an acceptance pass.
An existing shared-resource-registry migration is an environment prerequisite at its
owner, not a product repair adopted by this plan. A deliberately isolated test registry
may establish local evidence, but is named as such and does not qualify the canonical
resource registry or actual canonical store.

### Representative functional checks

The maintainer selected representative functional assessment on 2026-10-22 and canceled
the broad native sweep. The [coordinator pause handoff](49-library-native-remediation.md#pause-handoff--requested-closeout-2026-10-21)
owns actual results, interruption and retained-state status. This section specifies the
revised approach; the scale/setup corrections below are **Proposed**, not implemented.
No further campaign or benchmark is authorized by documenting them.

Routine checks use the smallest fixture that exercises each required transition, refusal,
source attribution, conservation/accuracy assertion and ownership boundary. Resource policy,
fan-out, payload extent and validity windows need not copy production defaults. They must
remain valid inputs to the same checked implementation. Retain real schema/query semantics
when those contracts are the subject; never stamp a reduced schema with the complete
production schema digest or bypass normal admission to accelerate a test.

Separate pure model/policy/codec controls, native-local execution and selected live-store
contract checks using the existing package/filter routes. A whole-workspace native sweep
includes expensive store qualification and is not the default representative functional
selection. Explicit 16-client characterization, large declaration inventories, large-payload
multi-page campaigns and real production-window coverage remain separately selected
full-store component qualification. They are not silently replaced by a small smoke pass,
and they do not require a benchmark for this pause closeout.

| Existing case | Representative correction | Preserve |
| --- | --- | --- |
| Artifact original intent / bounded collection | Each already stores one four-byte block but waits for the production 60-second creation window. Supply a short legally admitted original window, fixed before sealing/submission, through the same checked path; implementing that input remains necessary if the current constructor cannot express it. | Real server-clock expiry, pending-window refusal, immutable intent, late-creation fencing, settlement and replay; keep production-default coverage separately |
| ACK close corruption variants | Initialize one database for the four variants; assign distinct run/claim/operation keys and independent attempts. | Every original corruption refusal and the open-attempt assertion; remove three repeated full-schema installations |
| Seed inventory variants | Initialize one database for three states with distinct attempt/seed/operation identities. | Unregistered, registered-but-unacknowledged and fully acknowledged states; remove two repeated installations |
| Edge-first retirement | Replace 128 nodes/1,024 edges with 2 nodes/65 edges for the 64-edge page boundary; use 65 nodes/65 edges if the same case must also cross the node-page boundary. | Edge-first deletion, continuation, live roots, final absence and replay |
| Result paging / concurrent effects | Use one item past the actual page limit and the smallest distinct clients needed for the interleaving. | Actual production page/serialization boundary and both relevant orders; fixed-limit acceptance remains separately identifiable |

Share a database only within independent variants whose identities and effects remain
separate. Fresh installation, absent/corrupt databases, fixture lifetime and true cross-database
isolation controls still require their own fixtures. Preserve native numerical value,
derivative, cancellation, original-clock and physical lifetime assertions.

The interrupted report observed 112.698 s for retirement, 98.138 s for ACK close, 84.112 s for seed
inventory, and 88.032 s/85.917 s for the two artifact controls. These are whole-test durations
under concurrent load, not phase measurements. Add revealing setup/body/drain timings before
attributing latency to SurrealDB, Rust, schema installation, transport or cleanup. Repeated
full-schema setup and explicit 60-second waits are established work, but their aggregate
contribution has not been measured. Scope the next selection by the changed contracts and
reuse already completed case evidence; do not automatically repeat the canceled sweep.

### Scenario coverage and independent controls

| Source scenario / affected obligation | Revealing assembled control | What it establishes |
| --- | --- | --- |
| S01/S02; F09; PRE retention | Reference ThreeUnits at selected production defaults: no edit, unreached edit, reached edit, repeated bounded churn and oversized live root | Reuse/backdating and truthful finite refusal; original 143-body result is a fixture condition, not a universal reuse requirement |
| S03; F01; native lifetime | Cancel a running horizon/study at a gated native stage; expire a deadline; verify join before charge release | Caller lineage, typed Cancelled versus TimeLimit, partial trajectory and actual physical drain |
| S04; R1/R17/R19 | Add a test lifecycle vocabulary at the registry declaration and regenerate all receiving surfaces; refuse unknown tags and invalid state combinations | One declaration, explicit physical/observational projections and strict versioned boundary behavior |
| S05; F03; selected MX/gRPC | Sixteen durable lanes over one problem, multiple independent clients, caller exit, server restart and uncertain reply | Required commit-order paging, durable claim/settle, cancellation-wins and preserved lane capability |
| S06; R4 | Append/select all four scalar relation families with owner/field/partition and limits, reopen protected payloads | Block-oriented writes, exact residual semantics, original row coordinates, full protected admission and no locator writes |
| S07; R14/R15 | Settings-dependent capability and a declared recycle map through the seam, plus Uno status matrix | Mechanism knowledge stays in adapters; no derivative claim for derivative-free maps or false infeasibility from small steps |
| S08; R18 | Python keyed starts, block stages, samples, per-state tolerances and signed/sub-microsecond physical time | Identity/unit contracts survive ingress, ownership and execution without positional conventions |
| S09; R12 | Parameter-only Simulation/Fit study points compared with individually fresh runs | Prepare/rebind separation with fresh history/IC admission and recorded starts; changed inputs do not masquerade as reusable structure |
| S10; R11 | LM eligibility using explicit policy inputs, without constructing a runtime/store/native process | Local reasoning boundary; separately exercise eligible execution and independent qualification |
| S11; R5/R6/R7 | Existing-physics unit extension plus derivative/closure conformance across the changed lowering path | Package extension remains package-owned; no manual field-grammar inventory or physical meaning loss |
| F10; R20 | Genuine fit id plus altered matrix; indefinite and singular PSD external matrices; changed parameter order/units | Provenance is content-sensitive and mathematical PSD admission is independent from statistical certification |
| R21/R22 | Historical restore/bootstrap, interrupted generated maintenance, source launch, exported/reopened result selection | Correct phase ownership, no script domain writer, no product-to-tooling dependency or lost protected reference |

For result predicates include null, NaN, infinities, signed zero, absent owner, empty selection,
All-None predicates, key boundaries, duplicate/unsorted rows and residual-before-limit cases.
Compare selected rows to an independently specified row-wise interpretation, including
original coordinates. A producer/consumer round trip through the same wrong predicate
does not establish equivalence. Projection must not weaken the protected schema decoder.

For mathematical products include a dependent intermediate shared by outputs, mixed second
derivatives, a provider barrier, a branch/Require refusal and parameter-only rebind. Compare
regional derivatives with independent analytical examples and existing derivative conformance,
not merely two paths using the same newly generated derivative. Preserve root/branch identity,
physical validity and exactness promises; do not assert bit equality where arithmetic
regrouping was only admitted under a declared numerical equivalence.

For covariance use the explicit counterexample `[[1,2],[2,1]]`, `J=[1,-1]` and a singular
PSD matrix. The first must not yield certified finite negative variance. A real fit id
attached to edited contents must not grant fit provenance. Eigensolver failure, admitted
scratch exhaustion and unresolved statistical assumptions are truthful refusal/withholding,
not covariance zeroing or silent positive-definite regularization.

## Packages

| Package | Required input | Delivered qualification |
| --- | --- | --- |
| QUAL01 | Final proposed contracts, confirmed rule outcomes and stable package interfaces | Acceptance inventory reconciled to actual consumer migrations, source findings and explicit support limits; identifies which evidence closes which obligation |
| QUAL02 | All selected functional packages and their targeted controls | Short composed Rust/Python journeys over S01–S11 and the revealing failures above; diagnose assembled mismatches before expensive campaigns |
| QUAL03 | STO server/adapter/transport target, maintenance and result migration | Actual MX1.0/gRPC restart, durability, fault and 16-lane campaign at supported settings; storage conversion and receipt identity preserved |
| QUAL04 | MAT/PRE/RUN/BND integration plus affected generated catalogue | Selected scientific/native/Python/conformance journeys, mathematical identity and original-space qualification; unchanged solver eligibility accounted for |
| QUAL05 | QUAL02–QUAL04 and resolved selected inquiries | Required hygiene/manual checks, declared benefit measurements, enduring publication and final owner-based completeness assessment |

### QUAL01: Agree completion evidence before expensive execution

Read actual changed consumers and package outcomes, not merely completion labels. Confirm
that schema/code generation, old-row conversion, payload protection, retained compiled
artifacts, Python imports and maintenance activation use the same interpretations. Separate
new design questions from integration defects. A rejected rule must have a conforming target
and revised downstream acceptance before execution continues through that route.

Current Plan 28 E3/E4/E5, Plan 32 and Plan 40 retain their own campaigns. Share an actual result
only when its exact scope satisfies both owners; a Plan 49 pass does not close the other plan.
Completed Plan 48 remains prior qualification context, not a fresh backlog or substitute for
testing a changed mechanism.

### QUAL02: Short complete journeys

Exercise ordinary simulation, optimization, parameter fitting, a recycle and dynamics through
the public Rust/Python owners. Include refused admission, deadline, cancelled entered work,
successful completion and post-effect unknown outcome. Test prepared value reuse after edits
without broadening the request domain. Early local F01/F02 tests remain necessary but cannot
stand in for this assembled path.

Use `just check-package <affected-package>` and `just unit-package <package> <filter>` in
functional work. At this scope-end point select the applicable component/integration recipes
from `just --list` and the [qualification guide](../dev/validation-assessment.md), including
`just py-unit`/`just py-test` for their respective correctness extension and canonical-store
contracts. Record the exact commands/modes actually selected in Outcome. The plan does not
invent test names for controls that have not yet been implemented.

### QUAL03: Storage, transport and 16-lane truth

Run the managed server actually built from pinned SurrealDB 3.3.4 plus the MX1.0 integration.
Confirm the selected synchronous AOL/fsync settings are effective, not just configured in an
SDK. Kill/restart at pre-dispatch, post-dispatch/pre-reply, acknowledged commit, snapshot and
log-replay boundaries. Inject fsync/write failure and torn persistence tail where the engine's
supported test interfaces permit. A commit acknowledged as durable must survive; an unknown
mutation is reconciled by immutable receipt rather than automatically replayed.

Separate STO01 version compatibility from STO02's deliberate SSI adoption. Exercise fresh
framed records and exact-build reopen; no legacy AOL input or old-reader compatibility
is required under the maintainer's fresh-data decision. Verify typed poison/resource/age
failures, encoded-transaction limits and useful reader-pin statistics. Through actual
SurrealQL indexes, ranges and adjacency, exercise write skew, absent-key interference,
phantoms and bounded retries while keeping lifecycle guards. A library-level SSI test does
not establish that every database query path supplied the required read/range tracking.

Use the fresh catalogue and newly generated nontrivial lifecycle history: runs, close
membership, sealed manifests, cancellation, generations, visible-run ordering and protected
payload references. Restart independent clients and resume a study after its caller exits.
Document that history `VERSION` queries are outside the selected MX contract; preservation
of domain history/receipts remains required.

Expose finite server engine capacity, transaction age/pinned versions, active query streams
and aggregate client response ownership. Frame size alone is insufficient. Include a slow
reader, late statement error, oversized result and cancelled RPC. Nothing provisional may
publish committed success. Physical disconnect and failed session re-establishment must drain
actual requests and preserve dispatch uncertainty. Test control admission under saturated
application work so cancellation/settlement cannot deadlock behind it.

The 16-lane result distinguishes necessary commit-order sequence contention from avoidable
local pacing. Demonstrate all lanes can progress and cancellation remains fair. Compare
throughput only under equivalent persistence, payload, hardware and workload settings.
No speed ratio is an acceptance substitute for durable correctness.

### QUAL04: Scientific and boundary qualification

Run the affected derivative-consistency and closure conformance routes after resolved/region
lowering changes. Include representative reference packages and the guarded PR flash case;
the original guard-free operation counts are an upper-bound investigation, not an expected
whole-program speedup. Original-space feasibility, stationarity, conservation, validity and
derivative order remain independent from solver native termination.

Select the relevant linked native controls for actual changed adapters and modes. Preserve
the full current library eligibility: moving capability knowledge must not silently remove
Uno SQP-LBFGS, derivative-free maps, native safety gates or existing backend support. Existing
four Uno numerical hunks are tested under their declared scope, separately from transport/
integration hooks. LM cosine gtol is not absolute stationarity. Fresh-session comparisons
govern any selected reset/reuse inquiry; absent equality, keep fresh execution.

Rust/Python catalogue identity must match the actual supported build. Test generated enums,
version-first documents, identity hook round trips and all migrated Arrow side inputs with
the actual extension. The earlier msgspec probe establishes only selective hook behavior.
Run parity when the changed path reaches selected parity scope; name the reference version
and cases. Do not claim full numerical IDAES equivalence or qualification of property envelopes
that this work did not examine.

### QUAL05: Benefits, static checks and enduring owners

Required structural claims have direct checks: per-row locator declarations/callers gone,
no runtime→tooling edge, no script domain mutations, one schema catalogue, no parallel WS
production path after gRPC cutover, no old storage activation after MX cutover, and no old
rendered-text identity path after resolved consumers migrate.

Measure only benefits claimed quantitatively. Compare cold preparation, warm no-edit and
unreached/reached edits; repeated guarded products; append/select work; and 16-lane studies.
Keep total preparation plus execution, peak retained memory, workload size/skew and durability
settings visible. JIT/device/tracing alternatives have their bounded inquiry-specific
measurement route. No universal benchmark campaign is added for a small correctness fix.

At scope end run `just hygiene` and fix failures, then relevant manual `just governance`,
`just docs`, native and powerset checks from the qualification guide. `just turn-end` is
end-of-turn formatting/indexing, not product qualification. `just ready` is for dependency/
toolchain/skill changes or environment-shaped failure, not a per-package ritual.

Publish implemented enduring contracts to their architecture owners through confirmed
decision routes and blueprint revision rows. Preserve accepted ADR immutability. Outcome
records delivered behavior, a mistake corrected, deliberate deviations, exact Tested/Measured
scope, command/mode/baseline and remaining limits. Reconcile finding and package states at
the coordinator, then apply lifecycle retirement only after inbound consumers have current
owners. This document schedules no automatic completion audit; an explicitly requested audit
updates the same owning plan.
