---
title: Native setup lifetime and artifact identity
status: in-progress
date: 2026-10-07
adrs: [ADR-0164]
review_sources: [docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md, docs/design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md, docs/design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md, docs/design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md]
scenario_sources: [docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md#4-revealing-scenarios]
---

# 28h: Native setup lifetime and artifact identity

## Responsibility and accepted direction

This companion develops PE05/PE06 across native/build/tooling consumers. It supplies scoped
setup and provenance contracts to [28b](28b-selected-compilation-and-reuse.md) and continuing
E2 retirement/build integration. [28e](28e-rebuild-retirement-and-qualification.md) remains the
sole assembled qualification owner; the [coordinator](28-surrealdb-unified-substrate.md#repository-wide-efficiency-extension)
owns coverage and finding dispositions. Blueprint §5, §14 and §20 retain authority.

The [preparation integration](28-surrealdb-unified-substrate.md#preparation-assurance-and-reuse-review)
distinguishes current native setup's once-per-operation admission from runtime body replay
observation. [28i](28i-runtime-validity-and-interruption.md) owns runtime premises/validity
and interruption after confirmed RC01/RC04; this companion supplies actual worker/Python
root, artifact/configuration and native generation integration to I0/I1/I2. Installation
pins alone do not protect Python's loaded closure. PA01–PA04 remain with the coordinator.

The maintainer explicitly accepted the production review's
[RC01](../design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md#rc01)
on **2026-10-07**: retain actual artifact association and complete dirty outer provenance,
but stop making complete outer source observation a compiled input of every associated
consumer. This is distinct from the older unified-substrate review's RC01. It is an operator
target decision, not accepted ADR status or implemented qualification.

At `5260a3e9a3cecd69b6358ab86d4e917ae2508b29`, `native_cache.prepare` verifies every receipt
file under its per-identity exclusive lock. Native environment composition requests solver,
KLU, root-isolation and pipeline setup; Uno obtains a Cargo-owned HiGHS installation before
looking up its prepared prefix. `pse-buildinfo` embeds the complete crates/vendor/xtask source
inventory; current direct consumers are Python and optional xtask, not compiler/runtime.
Relevant scientific producer keys already differ from outer attestation. Preserve that strength.

Remaining execution follows [28e's development-phase evidence policy](28e-rebuild-retirement-and-qualification.md#development-phase-evidence-and-optional-artifact-requalification).
The identity mechanisms below describe exact execution/reuse and explicitly requested strict
qualification. Changes to their recorded inputs do not invalidate prior scientific results
or impose a mandatory rebuild/requalification campaign during design. No change-impact
classification system or frozen/copied Cargo environment is in scope.

## Plan 30 service and receiver integration

[30b](30b-persistent-services-and-receiver-generations.md#b0--generation-and-compatibility-contract)
consumes H's exact artifact/producer association while separating compatible disk-service
residency from immutable receiver generation admission. Rebuilding a mutable worker path
does not reidentify an existing receiver. Its actual runtime and I's receiving premises
still govern eligibility; unchanged storage is not a new provenance authority.

[30d](30d-host-admission-and-timing-qualification.md#d0--concrete-initial-profiles)
retains L7's exact 160 GiB/sixteen-physical-core reference profile as a qualification
lane and declares different bounded functional/timing profiles. Host admission must
coordinate every actual role without duplicating full-profile allowances or charging
parent/children twice. Plan 30 owns the new lifecycle/admission mechanisms; H's Outcome
and existing placement evidence are not promoted to their qualification.

## Native capability selection and installation lifetime

One setup owner interprets the selected recipe's target/features/profile and constructs the
actual required native capability closure. Build, unit/native qualification, Python, worker,
producer capture and measurement routes consume it rather than independently source broad
environment fragments. A default/compiler-only target initializes no unrelated native
provider. Whole linked qualification still prepares every capability actually in its feature
closure; selecting an Ipopt test does not imply the binary links only Ipopt.

Use the existing declared native bindings and consumed build inputs as authority. Avoid a
second handwritten dependency inventory or arbitrary inference from command text. An explicit
recipe request is resolved by the common owner, and actual producer capture checks its selected
unit/native closure. Preserve pinned versions, ABI widths, compiler/target/features/flags,
single LP64 BLAS provider, runtime reproducibility/thread settings and relevant external prefixes.

Adopt a conservative **top-level operation lifetime** for verified installation reuse:

- On admission, select the exact installation generation and fully verify its receipt,
  required interfaces and bytes. Hold a shared generation-use lease while the operation and
  its actual child consumers use it. Nested setup consumes this established validity instead
  of hashing the same immutable files and rediscovering the same tools again.
- Publish manager-owned generations immutably from private staging. Construction has its
  own per-identity owner; short publication coordination and generation-use protection do
  not make ordinary verified readers wait behind unrelated building or exclusive hashing.
  Never replace/delete a generation still in use. Retired generations are reclaimable after
  their final consumer drains.
- A new top-level operation performs admission again. No permanent verified bit or timestamp-only
  shortcut crosses that boundary. Unmanaged/explicit external prefixes do not inherit cache
  validity merely because their path or receipt looks similar; validate their consumed inputs
  under their own trust boundary, or refuse eligible use.
- Changed/corrupt/incomplete installations are ineligible and rebuilt or quarantined by the
  owner. Manager-controlled replacement changes generation. Arbitrary concurrent privileged
  mutation violates immutable-use premises and requires readmission; this design does not
  claim protection against undetected writes outside that controlled lifecycle.

The owner must keep generation selection and verification coherent with publication, and
release leases only after child cancellation/drain. Reuse can be an ordinary scoped Python
owner/context plus process supervision in the existing modules; no daemon or credential-like
environment boolean is required. A caller-supplied marker never grants scientific producer
qualification. If the owner cannot establish a stable generation, retain full verification
at the affected boundary rather than pretend the optimization is safe.

L0 identifies the actual supervising process and how generation-use protection reaches every
shell/Cargo/native child. The current short-lived Python command substitutions exit before
the consumers start, so a context held only inside `prepare()` is insufficient. Each surviving
consumer must keep the generation unreclaimable, or supervision must guarantee its drain before
the final lease disappears. Test abrupt parent exit with a surviving child as well as ordinary
cancellation; do not silently rely on Python finalizers or a normally completed shell.

This reduces repeated assurance within one complete operation, not all verification forever.
Keep full corruption/ABI checks on admission. Source archive, compiler and build-provider
identity work likewise occurs once per consumed input per admitted operation. Different
capabilities can share established provider observations without merging their own ABI contracts.

For Uno/HiGHS, resolve a usable installation candidate and its recorded relevant input closure
before performing nested Cargo discovery. Reuse only an actual Cargo-owned archive/header
association whose current consumed inputs are established. Missing/changed/unqualified state
runs the required Cargo discovery/build once through its owner. A source pin or existing
archive path alone is insufficient. Do not skip actual native linking or provider association
merely to improve a cache-hit number.

## Artifact identity and complete outer observation

Use three distinct products under one declaration/framing owner:

| Product | Meaning, placement and consumers |
|---|---|
| Relevant producer/build-input identity | Complete actual selected unit/native/configuration closure for the role, including its composition root. Identify inputs before link; finalize qualified identity from actual compiler consumption where necessary, and embed it only if an executable needs it before deployment admission. Incidental files outside that closure are not inputs. |
| Actual deployed artifact association | After successful build/install, bind observed artifact bytes to the reviewed capture and role-specific target/profile/feature/ABI contract. Worker startup and the imported Python module independently identify the actual artifact. |
| Complete outer source/build observation | Observe the complete dirty source/configuration inventory under its existing definition at deployment/run admission. Record it with commit/context and all participating role associations; it is not another scientific reuse key or a mandatory compiled all-tree digest. |

Retain actual source bytes, generated/build-script/plugin inputs, vendor override presence or
absence, native libraries and compiler effects wherever the role consumed them. Unknown inputs
continue to refuse persistent eligibility. Compiler/tool observations are relevant when they
determine the actual artifact or result-producing implementation; unrelated host Rust/tool
inventory is contextual, not automatic scientific refusal. Secrets are not raw provenance fields.

Producer input identity precedes linking; artifact-byte identity follows it. Avoid circular
self-hashing or embedding the final executable's own digest into itself. Extend the existing
actual unit-DAG capture and association mechanism instead of manufacturing a matching pair
of runtime JSON documents. Target archive/library, worker executable and Python extension
roles remain explicit.

Role-specific artifact identities need not be equal. Qualification establishes their required
shared scientific/ABI compatibility and binds each actual artifact to the deployment record.
The worker and Python can share one outer observation without embedding identical all-tree
attestations. An unrelated outer edit changes that observation while leaving unaffected
artifacts eligible; an edit to a consumed root, native input or unit changes the affected role.
Dirty consumed bytes count regardless of Git commit. A changed file name/presence matters
where selection or interpretation consumed it.

Keep Python's actual installed/imported extension association, including the currently admitted
Maturin RPATH transformation. A same-path replacement, wrong selected Cargo root, stale worker,
changed ABI/native provider or forged capture refuses. An independently observed import/start
is necessary; receipt metadata alone does not prove the deployed artifact.

L0 uses the ADR skill and decision/design route to amend proposed ADR-0164 and the governed
hashing/deployment contract before L3. If a governed record has become accepted by execution
time, supersede it rather than edit it. Version changed identity/receipt meanings and admit
version before decoding current shape. Do not recompute historical keys or relabel historical
qualification. Remove all-tree build-script watches/embedding only after replacement artifact
association and outer observation have working consumers.

## Deployment-local replay and build-output derivation

The enhancement review's [RC01](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#rc01)
was explicitly accepted on **2026-10-07**, separately from the production-efficiency identity
decision above. [The coordinator](28-surrealdb-unified-substrate.md#rule-changes-confirmed-by-the-maintainer)
records it. L5 supplies narrow exact deployment-local compatibility to [28b B5](28b-selected-compilation-and-reuse.md#ordinary-deployment-local-replay);
it does not broaden or relabel `QualifiedProducer`'s strict guarantee.

`pse-buildinfo::identity` already observes actual files and verifies the loaded worker/Python
module mapping. Native receipts and operation pins establish installation contents and admitted
generation lifetimes. These are reusable foundations, but an installed receipt alone does not
prove which shared libraries or runtime-loaded artifacts the process consumed. Default admission
must independently associate those artifacts with the receiving code and their actual use.

Start with the supported Linux worker and imported Python deployment. Frame the actual role/artifact,
reconstruction interpretation, conservatively complete supported loaded native/runtime artifact set
and effective configuration required by reconstruction. Existing scientific keys own providers,
selected dependencies and compiler/kernel construction profiles. Observe actual consumed bytes and
stable use through existing file/mapping and generation owners; paths, mtimes, shell markers and
complete outer source hashes confer no eligibility. Identical deployed code need not reproduce
its compiler/source capture to establish this narrower compatibility.

L5 first settles the supported loader/runtime closure, late loading, observation-to-use stability
and configuration boundary. Pin or otherwise establish unchanged actual use at the existing owner;
do not falsely treat every receipt-listed library as mapped. Deleted/replaced or unidentified mappings,
opaque plugins/provider code, unobserved JIT artifacts and undeclared runtime inputs refuse local
cache reuse when their consumed premises cannot be established. Mutable/external prefixes need
their actual stable-use guarantee or honest refusal. Scientific execution/history remain available
under their independent safety contracts. No general relevance classifier or artifact framework is
required; a conservative supported artifact set is acceptable.

The initial API extends the existing deployment's optional replay admission to distinguish local
and strict guarantees, minted at real worker startup/Python import rather than by receipt fields.
B5 owns the single persisted/reconstruction implementation. Version the changed key/admission
meaning at its current declarations; leave old product provenance intact. L5 includes amendment of
proposed ADR-0164 and the governed explanation/revision before dependent implementation; if the
record has become accepted, supersede it. This document schedules that route, not ADR acceptance.

[Enhancement F03](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#f03)
is independent: `seal_generation` makes receipts read-only, while backend-native `root_isolation`
copies their permission bits into reusable Cargo `OUT_DIR`. L6 consumes verified exact bytes and
materializes writable derived output. Preserve unchanged bytes/mtime where useful; replace changed
or existing read-only destinations through owner-local writable sibling/atomic replacement. Never
weaken the sealed generation, clean Cargo targets or rebuild providers to repair this boundary.

| Package | Delivered contract and dependent consumers | Focused acceptance | Status |
|---|---|---|---|
| L5 — Local replay observation and decision route | Settle and document Linux actual runtime closure/stability; route accepted enhancement RC01; supply independently minted versioned local admission to B5 and actual worker/Python roots. Unknown consumed premises refuse reuse only. | Same effective deployment across restart, receiving role/artifact substitution, mapped-library/configuration changes, late/deleted mappings, missing checkout/context and explicit strict/local separation. The observation-to-use premise must be established, not a caller assertion. | Implemented; independently observed scoped controls pass; assembled E3/E4 pending. |
| L6 — Writable build derivation | Consume immutable root-isolation receipt into Cargo-owned exact output, with idempotent repeat/changed-byte replacement. E2 receives a repeatable native build route. | Disposable read-only source; repeat unchanged bytes/mtime, changed receipt, existing mode-444 output and untouched source; then affected native build/test recipe. | Implemented and targeted-tested; affected native/Python builds execute. |

L6 can proceed independently of L5/B5. L5 is bounded source/contract investigation followed by
its working observation slice; an unresolved context cannot mint a token merely to unblock tests.
Unrelated code/environment changes never automatically trigger strict capture or invalidate history.

## Parallel deployment and supported native placement

L7 consumes [N10's native strategy](28f-shared-numerical-preparation.md#parallel-admission-and-native-lifetimes)
and the accepted Parallel RC03/RC04 directions. It supplies one supported finite local
placement to [C7](28c-durable-execution-and-studies.md#parallel-study-frontiers-and-durable-workers)
and E3. The `84a1caf17656f00f38b22e703c7cbc2b63a44d2d` supervisor validates server plus managed-worker memory allocations and
applies worker MemoryMax. SharedRuntime creates CPU/memory admission within each runtime;
neither this nor sixteen independent process defaults is aggregate deployment coordination.

First select the primary durable concurrency realization with C7 using N10's actual backend
constraints. Existing sequential worker processes are a fitting isolation option; one shared
runtime with bounded concurrent calls is eligible where native coexistence permits. Compare
artifact reconstruction, IPC/progress, retained state, exclusion waiting and lifetime costs,
not just process counts. Do not introduce a host-wide broker or another executor solely to
name the budget. Reuse existing server/worker scopes, recipe profiles and typed resource owners.

Before C7 activation, record the actual selected numeric profile at the existing configuration
owner and verify it can be materialized by the supervisor/runtime. These decisions are L7's
explicit prerequisite deliverable, not implementation defaults inferred from the review:

| Profile premise | Required selected value/behavior |
|---|---|
| Application width and native capacity | Sixteen independent ready case workers; effective aggregate native CPU ceiling, per-process case capacity and admitted optimizer/internal team widths. Ordinary cases start with one-thread inner execution; larger selected teams draw from the same allocation. |
| Placement | Managed process count and role/backend assignment, foreground assistance allocation, parent placement and applicable CPU quota/cpuset/runtime limits. Processes that require isolation do not each receive an independent sixteen-core default. |
| Memory | Finite parent/deployment cap; server cache/process allocation; each foreground/native child's pool and process cap; caches, temporary preparation, live/idle sessions, teams and retained outputs. Memory-pool limits and configured ceilings are not measured RSS guarantees. |
| Other finite resources | Job/waiter limits, progress/transport queues, selected batch extents, spill/disk and original task/admission clocks at their existing owners. Idle session slots and active temporary work remain distinct. |
| Tooling/test placement | Cargo build jobs, Nextest/xdist outer workers and tests containing inner case teams. Retain exclusive sixteen-slot parallel controls and bounded memory-heavy groups; account for server and test/runtime processes together. |
| Receiving lifecycle | Actual managed/foreground/Python receiver construction consumes L5/B5 compatibility and selected native settings; process replacement retains immutable receipts, cancellation/drain and child cleanup. |

Reconcile effective host/affinity/quota and the command's capped scope, including whether a
server or surviving worker belongs to another scope. A sum of server/child allocations within
one supervisor is not sufficient if the foreground runtime or selected tests are outside it.
Choose explicit fixed profile partitions or existing scoped admission where they suffice;
provide an aggregate argument and refusal when the supported placement cannot fit. Do not
silently raise the 128 GiB reference pool, per-worker capacity or command cap to hide F02,
reduce case width to substitute for the required target, or equate configured maxima with
simultaneous demand. N8 supplies safe preparation demand and bounded progress.

Keep native team/environment controls scoped to the owning operation. Distinguish private
solver instances from MKL/OpenMP/Rayon teams and process-global scheduler state. A changed
factorization/profile is qualified under its original scientific checks and actual linked
availability; no unmeasured default switch follows from Ipopt's alternate linear solvers.
Configuration/provenance consumers must observe the selected settings without creating a
second artifact identity or repeating immutable provider validation for every point.

The selected reference realization preserves the declaration in
`packages/reference/conformance.toml`: pool 128 GiB, worker capacity 16 GiB,
16 CPU/case lanes, 32 population tickets and one construction core. One primary process
has a 140 GiB cap including 12 GiB operational headroom; the server has 16 GiB and the
single observer 4 GiB, under one 160 GiB slice with sixteen physical CPUs and CPUQuota
1600%. Observers perform admission/inspection and do not assist native execution. The
receiver consumes exact typed profile values and startup validates kernel placement,
actual executable bytes, database and its sustained runtime. These checks establish
identity/placement, not sixteen active numerical owners or an RSS guarantee.

The maintainer confirmed available memory need not equal the entire configured ceiling.
Physical host capacity and effective ancestor limits remain finite prerequisites; available
and currently owned memory are observations. Qualification monitors actual use. Ordinary
Python inspection and managed studies have distinct process budgets, so the managed
journeys use a separate observer process while ordinary tests retain sixteen outer workers.
The primary role receives only its explicitly selected worker producer receipt. Reuse
compares the actual receiver's environment and startup receipt-byte observation; a changed
path or file cannot inherit a live receiver's association. Readiness does not establish
native overlap. The durable thermodynamic campaign also enters the observer before loading
the linked extension and reports caller counters separately from primary execution.
Native generation handoff preserves the existing operation owner across those scopes.

| Package | Inputs and delivered behavior | Migration, deletion and focused acceptance | Status |
|---|---|---|---|
| L7 — Supported aggregate deployment | N10 supported combinations; C7 occurrence/worker realization; existing supervisor/scopes and L5/B5 receivers. Select numeric CPU/memory/job/team placement and implement its existing config/launch enforcement. | Move server/native/foreground/Python and selected test/measurement roots together. Remove replaced duplicate per-process/full-budget assumptions where controls prove replacement. Test materialized limits/readback, insufficient envelope refusal, actual sixteen-case placement, inner-team selection, restart/reconstruction, abrupt parent/child loss and cancellation/drain. | Implemented placement/profile and targeted supervisor controls; actual receiving journeys and L4/E3/E4 remain. |

L4's consumer reconciliation now includes L7's settings, launch and receiving roots. Existing
strict/local artifact qualification remains distinct; L7 introduces no mandatory full rebuild,
source-copy campaign or remote/distributed capacity scheduler. E4 measures actual selected
placement only after corresponding functional evidence and reports its conditions.

## Consumer scope and packages

Inspect all recipe environment roots and their callers, native caches/build scripts, bootstrap/
doctor discovery, compiler and solver input capture, producer deployment, Python admission,
worker launch and validation/measurement entry points. `surreal_server` release installation
also verifies binary identity: assess whether it can consume the same immutable installation
mechanics while retaining its release and supervisor contracts. Its lifetime is not silently
equated to an ephemeral compiler operation. Current bounded read-only doctor discovery can
remain when it supplies a different guarantee.

| Package | Inputs and delivered behavior | Migration, deletion and focused acceptance | Status |
|---|---|---|---|
| L0 — Adopt lifecycle/provenance contracts | Accepted RC01, current bindings and actual capture. Route identity/deployment changes; settle coherent generation-use ownership and exact consumed setup closure. | Record operating/trust assumptions, version changes and affected composition roots. Confirm every remaining setup/capture consumer and justified distinct boundary. This does not accept the ADR or qualify deployment. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| L1 — Scoped verified setup | L0's installation lifetime and capability request. Implement the common setup owner, immutable generation publication and operation-scoped reuse. | Migrate native scripts, build/test/Python/worker/measurement consumers. Add a bounded `native-setup-unit` recipe since current tooling recipes do not select these lifecycle units alone. Test unchanged nested use, input changes, corruption/missing files, explicit-prefix refusal/readmission, concurrent builders/readers, surviving children after parent exit and cancellation/drain. Delete repeated same-lifetime validation and obsolete broad setup paths. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| L2 — Provider discovery and build locality | Working L1 and relevant native/Cargo input association. Perform actual HiGHS/compiler/archive discovery only where the current selected closure needs it. | Migrate pipeline setup and remaining generator/build consumers; unchanged outputs keep bytes/mtimes and stale outputs disappear through the generator. Test cache hit/miss and changed provider/flags/target; never infer eligibility from existence alone. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| L3 — Artifact and outer observation | L0's recorded decision route and actual producer tooling; compatible L1/L2 input observations. Implement separated identities and actual role association. | Migrate `pse-buildinfo`, producer capture/deployment, runtime qualification, worker/Python admission and assessment reuse together. Test unrelated/relevant edits, dirty root/native changes, wrong roles, replaced artifacts and actual imported extension. Delete replaced all-tree compiled identity and equality assumptions. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| L4 — Tooling consumer closure | L1–L3, working L5/B5 composition, L6 and L7's selected deployment/receiving consumer migration. Reconcile coordinator coverage and every confirmed applicable caller. | No displaced environment/capture mechanism remains; E3 receives actual local replay observations and E4 receives setup/build cases. Strict deployment qualification remains separate optional scope at E, not repeated here. | Prior L1–L6 source reconciliation complete; parallel consumer source reconciliation complete; assembled E3/E4 pending. |

L1/L2 can proceed independently of bulk result corrections. L3 cannot remove the old guarantee
before actual replacement association works. Scope executor files narrowly; root owns manifest/
recipe/generator declarations and shared qualification integration. Use existing checkout,
pinned toolchain and Cargo target ownership; never clean or share target trees to manufacture
cache comparisons. No dependency/toolchain change is planned by default. L7 owns any selected server/worker
profile change; existing profile values are not changed by document adoption.

## Verification and evidence limits

**Interface-checked:** the authoring assessment confirms full receipt-file verification under
exclusive preparation, nested provider discovery and all-tree compiled inventory in the current
direct consumer scope. It does not establish their share of the interrupted test run.

**Proposed acceptance:** targeted Python tooling units run through the repository's `.venv`
and L1's bounded `just native-setup-unit` recipe, with disposable installations and no real provider rebuild when
mocked file/ownership inputs suffice. Rust identity/deployment units use the relevant package/
producer unit recipes and explicit force-validation. New lifecycle controls must attack stale
and forged validity, not merely assert fewer calls. No per-test environment exception may admit
an artifact whose material consumed inputs are unknown.

E3 executes actual build/import/start association and affected native/Rust/Python journeys after
the complete extension. A capture-only false-rejection repair requires its actual association
controls, not automatic repetition of every prior series. L1/L3 change real setup/identity
contracts, so their affected consumer qualification must be fresh. E4 separates setup hashing,
discovery/build work and deployment admission; observe no-op, relevant and unrelated edits
under comparable checkout/toolchain/cache conditions. Target-only observations carry no speedup
claim when no comparable historical baseline exists.

## Checkpoint

Current execution handoff, 2026-10-10: implemented source mechanisms and scoped controls
below retain their limits. The maintainer authorized remaining Plan 28 including full E3/E4/E5;
[28e's checkpoint](28e-rebuild-retirement-and-qualification.md#checkpoint-and-next-step)
supersedes older readiness/pause instructions. N4/T5/L4 source reconciliation is implemented;
composed acceptance remains open. Next complete bounded campaign preparation and actual consumer
qualification, rather than reopening implemented mechanisms. Plan 33 is complete and separate.

I0/I1/I2 own receiving-runtime validity and interruption. Actual worker and imported-Python
roots now mint local admission through completion-owned scoped jobs with the original finite
clock. Actual managed receiver controls have scoped evidence; final composed E3 remains open. H retains installation/setup and actual artifact association; native generation
pins alone do not qualify whole-runtime protection.

Reference roles start with inherited scheduler affinity before creating children or native
threads; readiness checks the actual leader and every live thread. When the user manager
delegates cpuset, its effective CPU set is checked too. Missing cpuset delegation uses this
verified process-affinity route, while the shared cgroup still enforces the finite CPU quota
and memory ceiling. An unimplemented `AllowedCPUs` property alone establishes nothing.

L7 implements the selected one-primary reference realization on `84a1caf17656f00f38b22e703c7cbc2b63a44d2d` plus the working
tree, consuming N10 and supplying C7/E3. The supervisor derives capacities from the reference
manifest and checks actual receiver and cgroup readback. Script controls pass; live placement,
primary-only native overlap and original public scientific journeys precede E3 acceptance.
Earlier L0–L6 setup, artifact and receiver evidence retains its original scope.

L6 now materializes receipt bytes into a writable sibling and atomically replaces changed
Cargo outputs. Repeated content preserves modification time and repairs legacy sealed output
permissions without altering the sealed source. Its focused native mechanism control passes;
the assembled native route remains E3 scope. L5's decision route is recorded in proposed
ADR-0164 and architecture revision135. Independently observed local/strict admissions and
bounded synchronous reconstruction are integrated; actual default worker/Python receivers have
positive quiet-reconstruction and fresh-miss controls. L4 consumer source reconciliation is
complete, with assembled E3/E4 acceptance pending. Mathematical initialization completes before
local context observation so license/NSS startup loading does not invalidate its own admission.
The supported glibc 2.39 scope warms the owning module's
TLS before taking the loader write lock; callback construction uses only that existing
module's immutable mathematical code. No imports, newly loaded-module TLS, provider callbacks,
thread creation or loader-thread waits run under the lock. E3/E4 include actual composition;
earlier controls retain their original scope, and strict artifact qualification remains optional.
Its Rust association control is explicitly ignored in ordinary selection and selected exactly
by the producer assessment; its Python association uses `--producer-deployment`. These opt-ins
retain failure on absent captures. The ordinary campaign keeps producer-fixture scientific
controls active. Build locality instrumentation now includes no-op, relevant compiler edits and
unrelated documentation edits in an isolated source snapshot. Sufficient free storage is now
available under the existing launch floor; measurements remain queued behind functional qualification.

L0 recorded confirmed RC01 in proposed ADR-0164 and architecture revision132 before dependent
changes. Formal ADR acceptance remains separate. L1 implements an actual operation scope begun
before setup, immutable native generations, separate build/publication coordination and
scope-bound admission/use records. Reclamation verifies surviving descendants; unverifiable
owners retain conservative pins. Explicit unmanaged mutable prefixes retain full boundary
verification. Focused setup, actual ABI and recipe-root controls have positive evidence. Ordinary native
recipes share the operation owner. Manual helpers and long-lived server installation keep
their distinct full-verification and supervision contracts.

L2's unchanged installation path uses actual Cargo-owned HiGHS archive/header association before
nested build discovery. Its narrower relevant input closure is being integrated with L3's common
producer capture rather than another dependency inventory. L3 uses external versioned deployment
receipts: current actual unit-DAG/dep-info qualification finalizes the selected input key, then
post-build/import association binds actual role bytes. No current consumer needs that key
embedded before startup, so it is not injected into shared buildinfo and no circular self-hash
is introduced. ProducerV1 scientific identity keeps its existing meaning; the changed deployment
receipt shape has separate version admission. Actual worker/Python observation, allowed RPATH
transformation and complete outer observation remain obligations, not manufactured receipt pairs.

L3/L4 mechanisms are implemented and focused identity/deployment controls passed. Actual
three-role deployment qualification and enclosing E3/E4 remain open. Compiler-cache compilation
now remains in the supervised client and performs preprocessing; conflicting logging or
distribution settings fall back to direct compilation. The existing local cache is a trusted
build input, not evidence of artifact provenance. Earlier artifacts/receipts keep their historical scope.
A new strict deployment claim requires the affected real build/import/start association;
changing installation/source context alone does not require making that claim or requalifying
earlier behavior.
Independent implementation review found and corrected an unintended checkout requirement
in Python/worker admission. Available source provenance is explicit: an installed artifact
without its owning checkout records absence, retains actual mapped-artifact verification,
and gains no producer qualification from that absence. A copied actual executable inside
an unrelated Rust project exercises this boundary. Cold installation generation during
integration is not a comparable speed measurement.

Actual capture also exposed cached Rust dep-info whose output rules retain the previous
Cargo profile directory. Association now accepts only that output-directory relocation
with the full unit suffix and actual Cargo context preserved; source prerequisites are
never rewritten. Cargo wrapper configuration is excluded when an explicit environment
override selects the effective wrapper, whose native closure remains mandatory. Python
capture selects the actual Maturin ABI3 configuration required by the reviewed PyO3 branch;
that configuration does not enter the independent Rust library/worker caller. These capture
corrections retain unknown-input refusal and require real eligible recapture before E3.

Assembled integration also exposed environment reentry and fixture ownership mistakes.
Selected library paths now remain stable across nested recipes while preserving explicit
caller search components. Disposable setup fixtures establish their own admission owner
instead of inheriting the enclosing assessment's verified installations. HiGHS candidate
selection separates the proof locator from material compilation inputs; changing that
locator still triggers verification. Provider capture must use the actual workspace
composition (`pse-uno-sys` with `highs-provider`) and associate its reachable Cargo-owned
HiGHS dependency, rather than trying to select an inactive external dependency directly.
The real provider association and assembled deployment remain acceptance obligations.
