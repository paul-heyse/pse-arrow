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

Current qualification scope: [post-Plan51 acceptance reconciliation](#post-plan51-acceptance-reconciliation--2026-10-23).
The maintainer subsequently authorized [remaining 49f–49h execution](49-library-native-remediation.md#execution-resumption--remaining-49f49h-2026-10-23)
on 2026-10-23. Original baselines, failed runs and receipts retain their dates and conditions.

## Purpose and evidence boundary

Establish that the selected Plan 49 target works as an assembled system and that its
claimed benefits have the evidence they require. The
[coordinator](49-library-native-remediation.md) owns state and the combined completion
decision; this document specifies qualification, measurements and enduring publication.
It does not repeat individual package test lists or maintain another progress ledger.

**Interface-checked, original authoring:** the source review, net-change companion, published
probes, validation guide and relevant owners. Subsequent Plan49/50/51 Outcomes supply the
recorded **Implemented/Tested** baseline reconciled below. **Proposed:** the remaining acceptance,
not a requirement to repeat every listed control. Revising this plan performs no product test,
store migration, native simulation or performance measurement. Original receipts retain their
engine, fixture, version, date and actual failed/repaired result boundaries.

The governing standard is Core 3.4, Efficient Architecture Heuristics 1.0, Process
Simulator 1.5 and the pse-arrow binding. The source's S01–S11 remain source definitions;
the following table explains how selected work exercises them. A qualified package is
not automatically a resolved cross-area finding. G4/G6/PS-G1/PS-G2 were unresolved in the
review and remain so until evidence in their actual scope establishes a narrower claim.

## Qualification design

QUAL01 first credits applicable existing evidence and identifies the exact uncovered or
invalidated conditions. The sections below describe required behavior and evidence selection;
they are not a fresh blanket execution list. Completed package evidence remains valid only
for its actual assertions and premises. A new consumer does not automatically invalidate it,
and a compile/generator pass does not run a behavioral journey.

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

### Remaining-scope review acceptance additions

The [2026-10-22 review](../functional_reviews/plan49-remaining-scope-and-library-integration-2026-10-22.md)
supplied the following acceptance distinctions to existing QUAL owners. The post-Plan51
reconciliation credits subsequent evidence and retains only uncovered or invalidated conditions.
These are contract-focused controls, not an authorization to repeat the canceled native sweep.

| Change | Revealing representative control | Owning join |
| --- | --- | --- |
| STO09 selected Surrealism policy transaction | Shared predecessor deduplication, encoded-byte boundary, explicit oversized scope refusal, stale candidate alongside independent progress, both notification/cursor orders, rollback, ack loss and unknown-effect reconciliation | QUAL02/QUAL03 with RUN03/RUN05; preserve exact per-item premises, module resource limits and durable unfinished work; credit completed STO09 controls |
| RES07 joined protected metadata | Missing/duplicate descriptor refuses instead of disappearing; current file/selection freshness survives join; canceled/escaped page keeps composite ownership | QUAL02/QUAL03 with TOOL04; preserve full protected file admission |
| RES07 native plan retention | More than one statistics page/batch uses one physical plan; unknown statistics keeps batch; same residual preserves original bits/offsets and limit semantics | QUAL02/QUAL04; actual session/schema/function premises and pool match |
| PRE01 Salsa patch | Exclusive accounting drains readers without self-deadlock; owned observation has stated freshness; live memo reuse/churn/oversize still hold | QUAL04; patch safety/API change independent of finer specialization |
| PRE08 typed fragments | Incremental versus clean root after local/source/membership/provider edits and root switches; original signed contributions, guards and diagnostics remain correct | QUAL04 with PRE07/MAT05; legitimate complete root checks remain |
| TOOL08 explanations | Immediate blocker/source/consumer edges, distinct roles, selected revision, partial continuation and actual native work bound on selective/later pages | QUAL02/QUAL03/BND05; credit current semantic API/CLI controls, complete native bound; retrieval is not a scientific certificate |
| PRE09 only after a separate future selection | Known compatible assignment, attributed contradiction, complete assumption binding, Unknown/limit/cancel and native drain; bounded enumeration as independent small-case oracle | Not current remaining scope: inquiry closed with no adoption and RC23 unselected; no numerical feasibility claim |

QUAL01 must reconcile implemented consumers and these additions before final selection.
RC22 is selected and STO09's Surrealism policy execution is implemented and tested; preserve
its actual module identity, install/activation, resource/failure and exact-effect contracts.
Do not reintroduce the retired external scope/apply execution path. PRE09's inquiry is complete
with no adoption; RC23 remains unselected and adds no configuration/Z3 campaign. Existing
S01–S11, native eligibility, covariance and physical custody obligations remain. No additional
full sweep or benefit campaign follows merely from a new review or consumer.

### Post-Plan51 acceptance reconciliation — 2026-10-23

The maintainer authorized this scope revision following source assessment at `d16a253ac`.
It changes the remaining execution selection, not the scientific target or accepted rules.
The coordinator remains the single package/finding/inquiry status owner. QUAL01 consumes
the following evidence at its original scope before selecting more work:

- [49a–49c Outcome](49-library-native-remediation.md#outcome--remaining-49a49c-2026-10-22):
  selected Surrealism readiness, protected result selection, Salsa/resolved preparation and
  actual consumer retirement. The earlier store Outcome owns engine/SSI/gRPC/durability controls.
- [49d–49e Outcome](49-library-native-remediation.md#outcome--remaining-49d49e-2026-10-22):
  mathematical/native/runtime controls, physical publication, explicit fitting policies and
  actual sixteen-owner lifecycle acceptance. The final reference selection passed 6/6 with
  original production Surrealism limits; no comparative throughput benefit follows.
- [Plan50 Outcome](50-codex-effectiveness-and-diagnostics.md#outcome): actual query/graph
  inspection, diagnostic commands, environment and bounded observation tools. Its graph
  consumer does not establish TOOL08's native examined-edge bound.
- [Plan51g Outcome](51g-qualification-and-publication.md#verification-and-outcome--assembled-execution-2026-10-23):
  selected Rust/installed-Python/actual-CLI composition, calibrated fit → parameter publication
  → portable import/edit → supported downstream study, original protected IPC/artifact checks,
  lifecycle refusals and nonempty disposable application recovery. It also records generator
  equivalence, repaired hygiene, native/optional builds and composed feature-check coverage.

**Tested, exact Plan51 limits:** the 100-instance Rust selection initially passed 73 and failed
27 at fixture startup with sixteen concurrent tests. The exact 27 failed binary/test instances
then passed at four concurrent tests. This is composed selected coverage, not a clean 100-case
run or qualification at sixteen concurrent tests. Its original 476-case feature matrix failed
25 cases; all exact affected reruns passed, while the original command retains failure. No new
speed/capacity, whole-product/parity, full scientific recovery, wheel/distribution or other-platform
qualification is established. Do not conflate test-runner concurrency with the separately tested
sixteen native study owners.

Plan51 extends the real consumer surface. The following are acceptance connections to reconcile,
not new APIs or an instruction to repeat already-covered journeys:

| Connection | Required distinction | Existing evidence and residual owner |
| --- | --- | --- |
| TOOL01/BND05 → project/research/lifecycle/import documents | Actual catalogue, standalone/nested versions, typed identity and physical context agree; wrong schema identity refuses before interpretation | TOOL01 direction/header/standalone controls and matching installed Plan51 consumers; add the boundary-schema mismatch negative control, then select uncovered BND adoption at QUAL04 |
| TOOL04 → current application closure | Generated maintenance and current backup-v4 inventory include participating project/research/draft/pending-import owners; original nonce, generation and receipt survive interruption and restored authority is fresh | Plan51 host/recovery controls cover selected routing/activation; QUAL03 retains uncovered live function replay/refusal/protection behavior, not historical conversion |
| RES07/TOOL05 → protected project copy/import/release | Original selected rows, lineage, framing and protection survive copying; missing files refuse; pending import is unavailable; imported evidence grants no receiving run/seed/covariance authority | Plan51 protected-copy/import/lifecycle controls plus remaining review consumers; QUAL02/QUAL04 select only uncovered or changed connections |
| TOOL08 → actual operator inspection | Typed roles, selected revision, protected evidence and continuation survive a native seek/range work bound | Existing graph API/CLI controls and Plan50 plan inspection; TOOL08 supplies the remaining implementation before QUAL02/QUAL03 integration |
| Evidence/health → investigation | Authentic failed/interrupted/missing populations and unavailable metric fields remain distinct; an index grants no scientific, retention or causal authority | Reuse Plan50/51 tools and their original evidence owners to investigate selected failures; no second test registry or per-command plan ledger |

Keep lifecycle-release and project-closure explainers with their existing owners; graph retrieval
does not replace them. Descriptive/offline availability is not executable availability. The actual
fit/publication journey preserves eligible estimates with covariance withheld; it does not replace
49f's independent provenance/PSD controls or full uncertainty qualification.

### Investigation dispositions exposed by Plan51

QUAL03 owns bounded follow-up selection when either observation obstructs remaining acceptance:

- **Fixture startup:** the cause of the sixteen-concurrent-test failures is unproven; contention
  is an inference. The primary stayed active and authenticated reads succeeded. Use existing
  setup/body/drain timing and service/query diagnostics to isolate the failing startup phase
  before changing fixture admission or claiming repair. A bounded concurrency check is required
  only for that diagnosed repair or a retained concurrency claim; no broad native sweep follows.
- **Metric availability:** primary capture independently verified service/cgroup identity but
  returned partial metrics because the response exceeded the unchanged 4 MiB bound. If a remaining
  engine/pinned-reader assertion needs those fields, establish a bounded supported observation
  path or report the assertion unresolved. Do not parse a truncated prefix, silently raise the
  limit or treat disposable-store metric success as primary-store coverage. This is not an
  automatic new telemetry platform or performance campaign.

These are observed evidence limits, not diagnosed product defects. Completed Plan51 scope is
unchanged. Any repair follows its original owner and retains truthful missing/unknown results.

### Representative functional checks

The maintainer selected representative functional assessment on 2026-10-22 and canceled
the broad native sweep. The [coordinator pause handoff](49-library-native-remediation.md#pause-handoff--requested-closeout-2026-10-21)
owns actual results, interruption and retained-state status. This section specifies the
revised approach; the scale/setup corrections below are historical proposals whose current
implementation and evidence must be reconciled by QUAL01 before scheduling them again.
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
| S05; F03; selected MX/gRPC | Credit existing sixteen-lane store and final sixteen-owner lifecycle controls; select only remaining independent-client/restart/uncertain-reply conditions affected by changes | Required commit-order paging, durable claim/settle, cancellation-wins and preserved lane capability; no automatic repeat of a canceled campaign |
| S06; R4 | Append/select all four scalar relation families with owner/field/partition and limits, reopen protected payloads | Block-oriented writes, exact residual semantics, original row coordinates, full protected admission and no locator writes |
| S07; R14/R15 | Settings-dependent capability and a declared recycle map through the seam, plus Uno status matrix | Mechanism knowledge stays in adapters; no derivative claim for derivative-free maps or false infeasibility from small steps |
| S08; R18 | Python keyed starts, block stages, samples, per-state tolerances and signed/sub-microsecond physical time | Identity/unit contracts survive ingress, ownership and execution without positional conventions |
| S09; R12 | Parameter-only Simulation/Fit study points compared with individually fresh runs | Prepare/rebind separation with fresh history/IC admission and recorded starts; changed inputs do not masquerade as reusable structure |
| S10; R11 | LM eligibility using explicit policy inputs, without constructing a runtime/store/native process | Local reasoning boundary; separately exercise eligible execution and independent qualification |
| S11; R5/R6/R7 | Existing-physics unit extension plus derivative/closure conformance across the changed lowering path | Package extension remains package-owned; no manual field-grammar inventory or physical meaning loss |
| F10; R20 | Genuine fit id plus altered matrix; indefinite and singular PSD external matrices; changed parameter order/units | Provenance is content-sensitive and mathematical PSD admission is independent from statistical certification |
| R21/R22 | Current-format restore/bootstrap, interrupted generated maintenance, source launch, exported/reopened result selection and affected Plan51 consumers | Correct phase ownership, no script domain writer, no product-to-tooling dependency or lost protected reference; no legacy conversion requirement |

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
| QUAL01 | Current contracts, selected rules, package Outcomes and actual Plan49/50/51 consumers | Credit evidence at original conditions, identify uncovered/invalidated assertions and select the remaining dependency-ordered acceptance |
| QUAL02 | Remaining functional changes and existing composed journey evidence | Short public Rust/Python/CLI journeys only for uncovered or changed connections across S01–S11 and Plan51 consumers |
| QUAL03 | Delivered MX1.0/gRPC/Surrealism target and existing store/lifecycle/recovery evidence | Remaining durability, fault, live maintenance and bounded-query conditions; current-format recovery and receipt identity, with sixteen-lane evidence reused |
| QUAL04 | Delivered MAT/PRE/RUN, remaining BND/TOOL consumers and selected scientific evidence | Uncovered/changed scientific/native/Python/conformance connections, independent covariance/physical/mathematical qualification and preserved solver eligibility |
| QUAL05 | Reconciled QUAL02–QUAL04 and selected inquiry outcomes | Scope-end hygiene/manual checks after remaining changes, measurements only for retained quantitative claims, residual enduring publication and owner-based closeout |

### QUAL01: Agree completion evidence before expensive execution

Read actual changed consumers and package outcomes, not merely completion labels. Confirm
that schema/code generation, fresh-data regeneration, payload protection, retained compiled
artifacts, Python imports and maintenance activation use the same interpretations. Separate
new design questions from integration defects. A rejected rule must have a conforming target
and revised downstream acceptance before execution continues through that route.

Record the reconciliation in the existing package/Outcome owners: each remaining assertion
must name its actual consumer, applicable evidence and uncovered or invalidated condition.
Do not create a parallel case registry. Plan51's full compilation/generation evidence discharges
those specific older pending obligations; named selected behavioral tests discharge only their
own assertions. Existing conformance, source-launch and live maintenance controls are available
at 49g; they need no new foundation implementation merely because Plan51 did not select them.

Sequence the remainder as QUAL01 reconciliation → residual TOOL/BND implementation with targeted
controls → uncovered QUAL02–QUAL04 public journeys → QUAL05 scope-end checks/publication.
TOOL08's minimal semantic producer already exists; its native work bound remains required before
claiming that package complete. No whole-document dependency should block an otherwise ready slice.

Current Plan 28 E3/E4/E5, Plan 32 and Plan 40 retain their own campaigns. Share an actual result
only when its exact scope satisfies both owners; a Plan 49 pass does not close the other plan.
Completed Plan 48 remains prior qualification context, not a fresh backlog or substitute for
testing a changed mechanism.

### QUAL02: Short complete journeys

Reconcile existing ordinary simulation, optimization, parameter fitting, recycle and dynamics
evidence through public Rust/Python owners, then exercise remaining or changed paths. Include
uncovered refused admission, deadline, cancelled entered work,
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

The store and later Plan49/51 Outcomes already exercise selected conditions below. QUAL01
credits them and selects only unqualified or invalidated conditions; this is not a fresh
whole-engine fault campaign. Current-format application closure recovery, engine durability
and rebuilding derived evidence indexes establish different facts.

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

The existing store/sixteen-owner results establish their named progress, cancellation and
physical-drain conditions. Preserve that capability and necessary commit-order sequencing;
rerun affected controls only when changes invalidate their premises or leave an actual gap.
No automatic fresh sixteen-lane throughput campaign is required. Any selected throughput
comparison uses equivalent persistence, payload, hardware and workload settings; no speed
ratio substitutes for durable correctness. Fixture startup at sixteen concurrent tests is a
separate investigation above, not evidence that sixteen durable lanes fail.

### QUAL04: Scientific and boundary qualification

Credit existing derivative-consistency/closure evidence and run affected routes after changes
that invalidate it. Include uncovered representative reference packages and the guarded PR flash case;
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

Include affected Plan51 project/research/import consumers in this boundary selection. Copied
historical fit evidence gains no receiving execution permission or new covariance certification.
Use the existing wrong-attempt/unit/bounds and withheld-covariance scientific controls where
they cover the assertion; preserve independent provenance/content and PSD admission controls.

### QUAL05: Benefits, static checks and enduring owners

Required structural claims have direct checks, including retired per-point scope/repair and
per-block predicate construction where their replacements are selected: per-row locator declarations/callers gone,
no runtime→tooling edge, no script domain mutations, one schema catalogue, no parallel WS
production path after gRPC cutover, no old storage activation after MX cutover, and no old
rendered-text identity path after resolved consumers migrate.

Measure only benefits claimed quantitatively. For a retained claim, select the relevant comparison
among cold preparation, warm no-edit/unreached/reached edits, guarded products, append/select or
sixteen-lane work; this list is not a mandatory campaign in the absence of such claims.
Keep total preparation plus execution, peak retained memory, workload size/skew and durability
settings visible. JIT/device/tracing alternatives have their bounded inquiry-specific
measurement route. No universal benchmark campaign is added for a small correctness fix.

At scope end run `just hygiene` and fix failures, then relevant manual `just governance`,
`just docs`, native and powerset checks from the qualification guide. `just turn-end` is
end-of-turn formatting/indexing, not product qualification. `just ready` is for dependency/
toolchain/skill changes or environment-shaped failure, not a per-package ritual.

Plan51's static/manual results remain evidence at their recorded tree and conditions. Remaining
functional changes receive required scope-end checks; documentation-only reconciliation does
not rerun product qualification. Preserve the original failed matrix/run results alongside
successful affected reruns. Use the existing environment: sixteen compiler jobs remains distinct
from test concurrency and from the documented overlapping Cargo-driver locking limitation.

Plan49/51 already published their delivered contracts, including Plan51's blueprint revision255.
Publish remaining changed enduring contracts to their architecture owners through confirmed
decision routes and blueprint revision rows. Preserve accepted ADR immutability. Outcome
records delivered behavior, a mistake corrected, deliberate deviations, exact Tested/Measured
scope, command/mode/baseline and remaining limits. Reconcile finding and package states at
the coordinator, then apply lifecycle retirement only after inbound consumers have current
owners. This document schedules no automatic completion audit; an explicitly requested audit
updates the same owning plan.


## Outcome — selected assembled acceptance, 2026-10-23

**Implemented:** QUAL01's post-Plan51 evidence reconciliation selected the uncovered boundary,
maintenance, graph and command joins rather than repeating completed engine/runtime campaigns.
QUAL02–QUAL04 consume the integrated BND/TOOL tests and native scanner evidence. The
[coordinator Outcome](49-library-native-remediation.md#outcome--remaining-49f49h-2026-10-23)
records exact commands/modes, failed and repaired selections, receipts and scope boundaries;
it remains the sole owner of package/finding/inquiry state.

**Tested:** selected Linux force-validated Rust/native/Python/CLI and host acceptance establishes
physical custody, content-sensitive covariance plus independent PSD admission, current catalogue
identity, conformance/launch/review consumers, generated maintenance replay/protection and all
seven bounded native graph scopes. Earlier Plan49/50/51 evidence retains its original conditions.
This is composed acceptance: original failures remain visible alongside successful affected
reruns. QUAL05's final checks and enduring architecture publication are recorded at the same
coordinator Outcome; no whole-product/parity, exhaustive feature, release-wheel or new-platform
qualification is implied.

**Mistake corrected:** ordinary table-provider construction did not consume a lazy propagated
covariance, and a canonical negative fixture republished a stale parent. Tests now consume the
actual Arrow stream and reuse one exact target revision, preserving PSD refusal and CAS.
**Deliberate deviations:** no canceled sweep, new benchmark or complete engine fault campaign
was resumed; original qualified premises were reused. Native index-entry budgets exclude disk,
MVCC-version and latency work. Proposed ADR status and other plans' campaigns remain distinct.
