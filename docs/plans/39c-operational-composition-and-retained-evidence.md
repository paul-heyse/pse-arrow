---
title: Operational composition and retained evidence
status: done
date: 2026-10-14
adrs: []
doc_owner: docs/plans/39-implementation-friction-and-codex-effectiveness.md
doc_topics: [environment, validation]
review_sources: [docs/design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md]
---

# Plan 39c: Operational composition and retained evidence

This companion develops the operational improvements selected by the
[Plan 39 coordinator](39-implementation-friction-and-codex-effectiveness.md).
The coordinator owns shared decisions and finding dispositions; this document owns C1–C5
execution progress. Its target is effective operation on this operator's 192 GiB Linux
workstation, using Codex and the same coordinator model. Concurrent jobs, authentic service
ownership and interruption still matter. Other platforms, agent runtimes and model routing
are outside the target.

The baseline is `36f15656d`. The [principal friction review](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md)
and its [tooling inquiry](../design_review/evidence/implementation-friction-and-codex-effectiveness-2026-10-14/tooling-inquiry.md)
describe earlier source and execution; their historical passes do not qualify this baseline.
No implementation, service operation or product qualification is performed by authoring
this plan. The new selected work and bounded investigations run before the full remaining
Plan 38 work; [39d](39d-qualification-and-plan38-cutover.md) owns that ordering and final
qualification handoff.

## Existing responsibilities and the target boundary

Source inspection supports improving composition through existing owners:

- `scripts/pse_env.py` composes command context and owns the manager endpoint environment.
  `scripts/build_environment.py::configure` preserves explicit compiler selections and
  composes LLVM and cache settings. `scripts/native_operation.py` owns real native
  preparation, operation admission and exact unit observation. Their decisions must remain
  executable at those owners; a description cannot independently decide another environment.
- `scripts/validation.py::run_gates` already supports ordinary composition, live output and
  retained observations. `scripts/validation_receipts.py::reuse_checks` checks selected gate
  scope, origin integrity and applicability. `scripts/test_resources.py` owns borrowing,
  reference promotion and sealed manifest verification. `scripts/resource_store.py` bounds
  metadata transactions. These are distinct responsibilities even when one operation uses
  all of them.
- `scripts/activity.py` is the fixed read-only observer. It preserves independent process
  observations and partial unit results. Its manager helper currently discards useful child
  stderr. `native_operation.unit_observation` already supplies the appropriate manager
  environment and exposes invocation/cgroup fields; callers should consume that capability
  without recreating bus setup.

In the retained-report path, `run_gates` references a parent, `reuse_checks` references it
again, and each selected check may reference its origin. `reference_report` verifies the
receipt and every manifest entry. This establishes a concrete repeated-work question,
not a measured bottleneck. A resource borrow prevents owner-driven reclamation; it does
not make arbitrary filesystem writes impossible. A sealed digest likewise does not turn
a mutable directory into an immutable snapshot. Those distinctions constrain C1/C2.

The target keeps fresh execution, unchanged-input reuse and reviewed applicability
transfer distinct. It preserves exact artifact and receipt integrity, unknown-history
refusal for reuse, reference-before-release ordering and truthful incomplete results.
There is no new evidence registry, workflow language, mandatory runner, hook, scheduler
or universal resource cap. Existing force-validation and scientific acceptance remain
unchanged.

## Decisions and cross-document ownership

Principal review RC01's two-scope target was accepted on 2026-10-14: ordinary deployment-local
compatibility and explicit selective qualification remain. No C packet assumes that a sound
narrower channel binding already exists; 39b B1 investigates it. RC02's shared preparation boundary was accepted by the operator on 2026-10-14;
its implementation and fresh artifact association belong to 39b. This companion may consume
the effective preparation description once available, but it cannot weaken association or
decide that executor refresh is unnecessary.

[39b](39b-deployment-assurance-and-preparation.md) owns protected per-role build evidence,
live producer diagnostics, actual installer context and Python startup scratch lifetime.
Retained validation reports here are a different product from actual compiler-input evidence
there. C1 may reuse an established protected carrier only if its premises fit report reuse;
it must not create a second mutable owner or make 39b wait on an unrelated report optimization.

The supporting inquiry's candidate rule impacts are not additional accepted rule changes.
Its strict evidence-lifetime question routes to 39b under principal RC02. Environment
description and exact selected-unit options preserve existing rules. Host startup simplification
is conditional on C5 establishing the actual owner. If execution discovers a genuine rule
change, the coordinator obtains an explicit decision through the existing route before work
depends on it. No assurance relaxation is selected by this document.

## Packet sequence and progress

| Packet | Prerequisite | Delivered boundary and verification | State |
|---|---|---|---|
| <a id="c1"></a>C1 — Decide repeated report verification | Current `run_gates`, reuse/reference owners and representative retained reports; no producer guarantee choice required | Inspect representative complete report-read work, mutation exposure and origin fan-out; select existing checks, a protected operation view, or a sufficient consumed subset with exact integrity semantics. Record the decision and C2 boundary here; no optimization is implied by repeated calls alone | Done |
| <a id="c2"></a>C2 — Implement the selected report boundary, if justified | C1 selects a mechanism whose protected premises and consumer guarantee are explicit | Migrate affected validation/reference consumers together; remove superseded repeated verification only where the same guarantee survives. Targeted report/reuse/resource tests distinguish mutation, origin/reference ordering and all three clock lifetimes. If C1 retains current behavior, close C2 with that decision and no speculative mechanism | Done |
| <a id="c3"></a>C3 — Describe effective preparation before effects | Existing environment/native owners; consume 39b's shared role-preparation contract when describing that strict operation | Provide a concise read-only description or documented composition derived from those owners. Targeted owner tests, only if code changes, show explicit override preservation, truthful reuse/rebuild intent and absence of preparation effects during inspection | Done |
| <a id="c4"></a>C4 — Observe exact selected units | Existing fixed observer and manager endpoint composition | Add repeated exact unit selection or an equally small documented command through existing owners; retain global partiality. Targeted observer controls cover a selected vanished unit, unrelated global failure, missing manager and bounded failure detail without cleanup authority | Done |
| <a id="c5"></a>C5 — Investigate Codex startup shim provenance | Actual launch/shell startup sources available for read-only inspection; intentional project tool selections identified | Identify the real source of foreign-checkout shim inheritance; remove accidental injection only if confirmed and within the selected host-local scope. Check the resulting launch PATH and command resolution without native preparation; preserve intentional selections. If unconfirmed, retain the bounded question and explicit-context fallback | Done |

C1, C3, C4 and C5 need no whole-document barrier between them. C2 depends on C1's actual
decision. C3's strict role description consumes the corresponding implemented 39b operation
contract; general environment inspection can proceed earlier. Shared files such as
`native_operation.py`, `pse_env.py`, qualification documentation and tooling tests require
explicit edit coordination with 39b. Packet independence does not grant simultaneous edit
ownership. All packet outcomes feed 39d before the remaining Plan 38 execution.

### C1/C2: report-read work and protected premises

Investigate a representative complete report, including a many-gate reuse whose observations
share one origin and a report with several distinct retained origins. Identify bytes actually
read and hashed, manifest size, repeated origin visits and what each check promises. Inspect
available historical reports read-only; do not resume their campaign or invent timing from
log gaps. A bounded representative measurement is appropriate only if deciding the mechanism
requires it, with scope and conditions recorded. There is no savings claim to validate yet.

Compare three concrete outcomes:

1. Keep complete verification when the supported workload and mutation exposure make it the
   simplest sound choice. Document how existing composition should be used; repeated calls
   alone are not authority to remove checks.
2. Establish one protected manifest/receipt view for the reuse operation, share it for repeated
   visits to that exact origin, and reference it before releasing its protection. State how
   consumed bytes remain stable, how the view is associated with directory/file identity and
   how mutation is refused or isolated. Merely holding a borrow or copying a digest cannot
   establish those premises. Do not copy complete report trees without a consumed need.
3. Verify a sufficient subset only if the consumed claim can be stated exactly and all its
   required provenance, receipt and selected artifacts are covered. A subset does not establish
   integrity of omitted artifacts or grant disposal authority over them. If current full-report
   integrity is part of the guarantee, retain it or use a sound protected full view.

Any implementation stays within `validation`, `validation_receipts` and `test_resources`.
Keep original observation identity and transfer rationale; do not relabel retained checks
as fresh tests. A changed receipt, replaced directory, mutated required artifact or unsealed
origin refuses reuse and cannot create a false retained reference. Cancellation and failure
retain enough ownership for safe recovery; no success or cleanup is inferred from partial work.

Metadata transactions keep their own finite default bounds. A report borrow spans its actual
read/reference lifetime. An explicit operation-wide deadline remains one absolute clock,
including release/reference work; absence of that deadline remains absence. Do not synthesize
a five-second overall clock, renew an explicit one per origin or extend every timeout.

If C2 adds a mechanism, extend local controls in `scripts/tests/test_validation.py` and
`scripts/tests/test_test_resources.py` at the affected responsibility. Include many gates
sharing an origin and several origins, mutation during a yielded read, changed current inputs,
unknown history, failure before reference promotion and the existing long-read/explicit-expiry
cases. Independent mutation and borrower/reference observations must expose false reuse;
an assertion that only counts fewer function calls is insufficient. Delete replaced internal
paths and their obsolete tests once all affected consumers move; preserve tests of surviving
integrity and lifetime contracts.

### C3: effective operation description

Describe provider selection and reuse/rebuild intent, Rust wrappers, selected PATH/tool paths,
PyO3 SDK selector and selected role/profile/features before launching the effectful operation.
Derive these from existing composition, using the same precedence and explicit overrides as
execution. If reuse cannot be known without preparation, state that uncertainty and the effect
execution would require. Inspection must not build a native prefix, acquire execution authority,
refresh Cargo outputs, install Python or replay an old owner token. Expose selected nonsecret
context only, not an environment dump.

A documented composition of existing APIs is sufficient if it removes the recurring need to
reconstruct provider-before-wrapper ordering. Add an option or helper only for a demonstrated
independent choice. An operator deliberately selecting a different wrapper or provider sees
that selection and its consequence; ordinary development uses current context and never
silently restores a historical PATH. The actual installer observation in 39b remains authoritative
for what the installer did; this description is intent and current selection, not installation
evidence or a claim that a compiler child ran.

Use `pse_env`, `build_environment` and `native_operation` as the decision owners. Retire
superseded temporary-controller fragments only after their inputs, effects and cancellation
behavior have a supported replacement. Temporary scripts remain legitimate composition tools;
their existence is not a requirement for a wholesale harness replacement or generic language.
If this packet is documentation/example-only and introduces no mechanism, source review is
enough; do not add tests for a reversible low-impact wording change.

### C4: selected-unit observation

The caller supplies exact existing unit names. The fixed observer queries only that selection
and reports completeness for it, using the current manager endpoint helper. Validate selection
as unit identities rather than treating it as an arbitrary command or wildcard. Include
invocation/cgroup identity when the consumer requires it. A vanished selected unit remains
unavailable; an unrelated vanished unit does not contaminate a successful selected result.
Global mode retains its global partial result and independent process observations.

Observation failures retain bounded child stderr and the relevant selected manager endpoint
context. Avoid arbitrary environment values and unbounded diagnostic output. A missing bus,
timeout, malformed response and disappeared unit are distinguishable from a successfully empty
selection. An empty cgroup or inactive state does not establish authenticated process drain,
allocation release or permission to signal. Existing ownership-sensitive operations still
establish their own exact authority.

Prefer extending `activity`'s existing fixed options or documenting `unit_observation` through
the existing manager owner. Keep the observer directly usable when workload admission is
unavailable. A bulk systemd query, new dashboard or another runner is not the default remedy:
no measured latency or researched bulk parsing contract justifies it. Code changes use
`scripts/tests/test_activity.py` and the affected native-operation tests, preserving existing
partial-disappearance and manager-address controls. A documented existing-API route requires
no invented product test.

### C5: host-local launch investigation

Trace the incoming Codex launch PATH to actual startup inputs, shell configuration and the
launching process context as available. The inquiry observed a foreign project's shim path;
it did not identify the injecting file. Do not assume `.bashrc` or a repository defect.
Compare intended tool selection with accidental inherited state before editing any host file.

When the injection owner and accidental behavior are confirmed, make the smallest host-local
change and inspect a newly created matching launch context for PATH and tool resolution.
Preserve deliberate project activation. If inspection cannot establish provenance, record the
bounded unresolved question and continue with explicit strict-context selection through C3;
do not guess an edit or freeze historical PATH globally. There is no new portability layer,
multi-model configuration or runtime abstraction. A simple reversible confirmed configuration
edit needs no new test harness; inspection of its resulting launch behavior is the relevant
verification.

## Coverage and disposition route

The principal coordinator remains the single finding/disposition owner. The following routes
cover every optional row in principal review §10 and the supporting tooling opportunities;
they do not create another mutable finding ledger.

| Source obligation or opportunity | Work/decision route and retained boundary |
|---|---|
| Repeated retained-report verification; observed episodes' separate remaining cost question | C1 decides from representative complete work; C2 implements only a justified sound boundary |
| Inspect effective strict preparation; tooling T03 and its RC02 candidate | C3 derives description/composition from existing owners; 39b owns shared preparation and installer truth |
| Actual installer context | 39b owns actual child command, selected SDK/config inputs, raw/installed identities and authorized transformation; C3 links that evidence |
| Owned-resource observation; T04 and candidate RC03 | C4 supplies exact selected results, bounded diagnostics and endpoint context; global partiality and cleanup ownership stay intact |
| Codex launch simplification; T03 and candidate RC04 | C5 investigates actual injection provenance and conditionally removes accidental host inheritance |
| Python startup scratch lifetime | 39b owns joined native observation, escaped-result ownership and shortened scratch lifetime; no duplicate C packet |
| Strict capture evidence lifetime; T01 and candidate RC01 | 39b owns protected role evidence under principal RC02; retained validation-report optimization is separately C1/C2 |
| Buffered capture diagnostics; T02 | 39b owns streaming into existing live/result consumers, parse separation and cancellation/drain |
| Corrected optional deadline behavior; T05 | Preserve in C1/C2; not reopened as an unimplemented defect |
| Fixture premises, scope and cleanup; T06 | Update affected fixtures with any changed owner in the same implementation packet; existing corrections remain preserved. 39d owns aggregate tooling verification. Share a constructor only for demonstrated repeated authenticated launch/drain behavior; no generic fixture pooling by default |
| Existing validation API, private resource registry and temporary controllers | Consume existing `run_gates`, live/result and origin semantics. Explicit private registry selection does not authorize migration, reset or disposal of the default registry. C3 removes only superseded composition fragments; 39d owns assembled journeys |

## Verification and current checkpoint

**Proposed:** C1–C5 are planned behavior/investigations. Source inspection informs these
boundaries; no new product test, measurement, native build, service or retained-evidence
campaign ran while authoring them. No performance or token-efficiency benefit is established.

During implementation, use targeted tests for changed mechanisms through `scripts/pse-env --`
and the selected pinned environment. Report command, mode, scope, zero-failure baseline and
actual result. Documentation-only or straightforward reversible configuration work does not
acquire unrelated scientific tests. The root runs `just turn-end` after file changes;
ADR-0161 and the repository execution rhythm govern readiness and final static checks.

[39d](39d-qualification-and-plan38-cutover.md) selects combined integrated tooling, affected
product journeys and static/manual qualification, including the full remaining Plan 38 scope,
as the final stage after selected new functional scope and investigations complete. A C packet's local pass neither
qualifies the host globally nor closes Plan 38's assembled scientific acceptance. Record
decisions and next dependencies here as C1–C5 advance, retaining exact unresolved premises;
link execution outcomes from the coordinator without copying per-command receipts.

## Selected execution outcome — 2026-10-14

**Implemented:** C3 replaces effectful explanation with a selection-only text/JSON view derived from `compose` and native `runtime_selection`. It never sources `.envrc.local`, imports the extension, prepares providers or acquires admission. Unevaluated local configuration makes the entire view provisional; provider reuse remains unknown. Explicit argv selectors and selected nonsecret tools/SDK/wrappers are intent, separate from 39b actual installer evidence. C4 adds repeated exact `activity --unit` with selected completeness, validated manager properties and bounded endpoint diagnostics. Global partial results and independent process/limit observations remain.

C1 inspected representative historical complete reports without resuming them. The qualified report has 34 retained checks across two origins, a 78,862,175-byte receipt and 24 manifest entries totaling 86,388,000 bytes; its principal complete origin has 58 entries totaling 33,803,234 bytes. The setup report has 35 retained checks across two origins and a 93,431,193-byte receipt. These are time-specific inventories, not integrity requalification or elapsed-time measurements. A borrow prevents reclamation but not same-user mutation. Retain full verification; revisit broader reuse only with a consumed sound immutable carrier/subset guarantee. **Implemented:** C2 removes only the eagerly evaluated discarded digest; explicit-null refusal and absent-field fallback retain their meanings. No integrity check or clock was weakened, and no speedup is claimed.

C5 identified the source: `~/.profile` and `~/.bashrc` load `~/.config/shell/user-session-env.sh`, whose explicit cwd-sensitive dispatcher block prepends the other repository's `scripts/agent-shims` only when present. That directory is currently absent; old Codex ancestry still carries its former PATH entry. The source documents intentional dispatch rather than an accidental project activation. No host edit is justified or made. Revisit actual dispatcher relocation/resolution if the directory returns or a concrete unwanted selection is observed; C3 supplies explicit current selection meanwhile.

**Tested, zero-failure baseline:** `scripts/pse-env --resource-class light -- .venv/bin/python -m unittest scripts.tests.test_pse_env scripts.tests.test_activity` passed 56 tests. The operations worker's focused manager/runtime/drain/reuse selection passed 21 tests and its independent borrow/deadline/reference/refusal selection passed five. The corrected explicit-light run followed an admission refusal; that refused invocation was not a test failure. `just activity --json --unit pse-compiler-cache-7b08ddbbbbee458e8a79d28812d8e2b6.service` returned the selected active service, complete selected observation and no errors. C3's real structured description reports the Python role/profile/features, provisional local selection and unknown provider reuse without preparation. These establish bounded mechanisms, not whole-host or simulator qualification. 39d's integrated/static/scientific/performance campaign was not run here.
