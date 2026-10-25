---
title: "Plan 52n: Readiness, progress and edit lifecycles"
status: in-progress
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, environment, diagnostics, design, validation]
---

# Plan 52n: Readiness, progress and edit lifecycles

## Scheduled follow-up — 2026-10-24

The maintainer authorized this plan amendment after the current-checkout MCP assessment.
LIFE04 extends the delivered LIFE01–LIFE03 foundation; their recorded Outcome and evidence remain
unchanged. The [coordinator amendment](52-agent-effectiveness-and-engineering-workflows.md#execution-freshness-amendment--2026-10-24)
owns its selection, progress and sequencing. This document revision does not implement the follow-up.

## Target and foundation assessment

**Proposed:** make preparation, running work and retained editing state independently inspectable
through their existing owners. This develops usability-review
[O08](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O08),
[O10](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O10) and
[O11](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O11).
The [coordinator](52-agent-effectiveness-and-engineering-workflows.md#usability-opportunity-dispositions)
owns state; completed ADP/MCP worker, observer and preview contracts are the starting point.

**Interface-checked:** fixed worker transport currently waits for the final structured response,
while scoped Rust execution already publishes an original invocation reference before settlement.
`Specialists` retains exact preview bytes under count/byte bounds without evicting issued handles;
reset changes presentation rather than capacity. The 2026-10-24 Plan53 preparation commit
`deeb37cc4` separates Python stages. This is inspected implementation, not evidence that its MCP
readiness/preparation consumers or remaining Plan53 qualification are complete.

Apply Core3.4, Heuristics1.0, Test Assessment1.2 and PS-10/PS-11 where science is involved.
AP-05/AP-06/AP-07 and H12/H17/H19/H25/H27 favor exposing existing states and releasing abandoned
state without introducing a second lifecycle/scheduler. No whole-host health sweep, endpoint-wide
workload allocation or new persistent preview database is selected.

## Shared lifecycle meanings

Declared availability, prepared artifacts, compatible imported bytes, live selected service and
actually exercised operation are different facts. Passive description stays source-only. Explicit
preparation and selected live checking disclose their effects, use their native environment/admission
and preserve actual binary/schema/source fences. A pure test target does not prove that its current
worker's broader import closure is prepared. Missing readiness must not trigger an automatic build
or source synchronization merely because an agent browses the catalogue.

An authentic issued handle enables original-owner watch/read. It does not convert an attached action
to safe detached execution. Preparation, execution, publication, assurance and drain retain separate
outcomes. Transport loss/cancellation cannot manufacture native settlement or retry eligibility.
Preview release ends future handle resolution; it does not undo or certify edits.

## Work packages

<a id="LIFE01"></a>
### LIFE01 — Selected readiness and explicit preparation consumers

Expose owner-defined prerequisites and readiness facts for a selected capability, with exact
preparation routes and clear evidence of which checks are passive versus live. Consume Plan53d's
actual environment/extension/stub stages and source capture, not a duplicated builder or stale
assumption that synchronization must always regenerate everything. Keep replacement, installation,
publication and preparation effects distinguishable in schemas/descriptions.

Use the original operation kernels for explicit actions. A described command is not a fabricated
readiness receipt; unknown state stays unknown until a selected check establishes it. Check only
the chosen dependencies and retain identity/source membership where it matters. Do not weaken native
byte/import fences or add a universal cache/health service. CON03 supplies the safe remedy view;
NAV03 keeps pure readiness/schema inspection independent of execution workers where actually possible.

Targeted acceptance: browse/source-only task with unavailable compiler/science/store providers;
prepared-but-incompatible imported extension; selected live service unavailable; explicit preparation
followed by a real operation. Independently check stage outcomes and absence of eager startup or
implicit publication. Keep Plan53d progress/qualification there. Remove superseded combined adapter
preparation calls only when all actual consumers use the owner stages.

<a id="LIFE02"></a>
### LIFE02 — Authentic early handles and foreground progress

Extend the existing fixed worker exchange with bounded observation of owner-issued stage/handle facts.
Use existing callbacks/structured references such as `rust_tests.execute`'s invocation-reference
publication, not log scraping or a predicted invocation ID. Bind the observation to the exact worker
request/owner and publish only once the original identity exists. Keep sensitive payloads private;
coalesce bounded stage notifications rather than one event per native primitive.

For a supporting host use FastMCP4.1.3
[`Context.report_progress(progress, total, message)`](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/context.py)
with the actual caller progress token. Injected
[`Progress()`](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/dependencies.py)
alone uses foreground `InMemoryProgress`, so it is not the selected wire bridge. Represent unknown
totals truthfully; no time-derived percentage. Preserve original handle/watch/read as the fallback
when notifications are not rendered. A source-only readiness read does not start observation workers.

Preserve shielded spawn/wait ownership, signaling and physical drain for attached actions. Detached
observer cancellation still leaves its subject running. Endpoint loss exposes original references
where actually available; a helper exit is not a native terminal record. Reuse existing correlation
facts only when useful; this work does not resolve F02's application/database trace seam.

Targeted acceptance: preparation refusal before handle issuance, actual handle before final return,
stage transitions, notification coalescing, no-token fallback, interrupted observer and repeated action
cancellation during spawn/execution/drain. Compare final results with original native readers. H02
requires actual Codex notification behavior for that host claim; an in-process client proves only its
own path. O16's optional tasks are evaluated later against this delivered foreground/native baseline.
Remove replaced final-only adapter observation when the bridge serves its callers; keep native owners.

<a id="LIFE03"></a>
### LIFE03 — Explicit preview retention inspection and release

Add owner-local retention status and exact-digest release to `Specialists`, exposed through the same
canonical endpoint with truthful effects. Status reports retained preview count/bytes, configured
capacity and selected handle availability without returning every source body. Keep current capacity
defaults unless evidence calls for a deliberate change; do not add a machine-memory governor.

Successful release makes future resolution of that endpoint's exact handle unavailable. Report
released versus unavailable; a well-formed unknown or foreign digest returns unavailable without
distinguishing its history or affecting siblings. Malformed digests retain structural refusal.
No tombstone ledger is needed. Release never applies/rolls back an edit or declares reconciliation
complete. Admission and release synchronize at the existing preview/apply owner: refuse release
while work is admitted or its original handle is still needed for unresolved reconciliation.
Keep that digest resolvable through the existing `edit_preview_reconcile(digest)` route. In particular,
an interrupted/partial apply cannot become unreconcilable merely because its bytes remain allocated
somewhere. Mark required reconciliation before effects; clear that obligation only from the authentic
apply/reconcile outcome. Completed reconciliation must classify all affected files, including
diverged/missing states; observing those states does not mean repair or rollback. Then an explicit
release may reclaim the retained capacity. Account for all
still-required bytes; no general lease/history framework. Reset continues to affect presentation
only; endpoint close retains its existing semantics and I14's independent inquiry.

Targeted acceptance uses small explicit capacities: fill retention, refuse without eviction, release
one handle, reclaim its available capacity, preserve siblings, return unavailable for well-formed
foreign/unknown/repeated releases, and refuse release during admitted work or unresolved reconciliation.
After an interrupted apply, the same original digest must still reconcile; only settled obligations
permit subsequent release. No release result invents rollback or erases required evidence.
Remove the close-only assumption in current guide/callers/tests
once the extension is integrated; no silent LRU, TTL or durable preview store is introduced.

<a id="LIFE04"></a>
### LIFE04 — Current native artifact readiness and admission

**Proposed:** distinguish historical preparation, compatible schema/import and an installed artifact
corresponding to the selected checkout's current supported inputs. Current readiness explicitly leaves
receipt currency unknown; native worker schema equality cannot detect a same-schema Rust body edit.
Reuse `scripts/python_extension.py`'s preparation inputs, installed-artifact identity and existing
source/provider admission contracts. Plans53/54 retain build/preparation authority; this package adds
their selected read/admission consumer, not another builder, receipt ledger or scientific schema.

Expose current, stale, unknown and unsupported evidence with the actual checkout/interpreter,
mode/features/validation and preparation/artifact basis needed to explain it. Keep secrets private.
Cheap passive description remains source-only; a selected check may inspect the required artifacts
and inputs but cannot synchronize dependencies, build/install, start services or repair providers.
For native operations requiring current implementation, bind the checked basis at existing admission
and refuse stale or unestablished required prerequisites before native effects, with CON03's exact
explicit preparation/reattachment remedy. An import-compatible result alone is insufficient.

Keep the check scoped and reusable through the existing owner's complete dependency contract.
Do not run full preparation-input reconstruction, compiler/provider setup or a whole-repository hash
for every catalogue/read call, or narrow dependencies through an unsound handwritten whitelist.
If a sound cheap check is unavailable for a mode, disclose that limit and select an explicit check;
never label unknown current. Preparation remains an explicit action, and its publication does not
mutate the running endpoint's environment. MCP06 consumes this basis at dispatch. Checks establish
admission-time correspondence, not an immutable checkout throughout execution; subsequent source
changes cannot manufacture rollback, cancellation or permission to replay effects.

Targeted acceptance includes a same-schema Rust implementation edit, changed relevant configuration,
installed-artifact replacement, absent/unsupported preparation evidence and unchanged valid reuse.
Verify that browsing stays available without native providers and that stale refusal causes no
build or native effect. Explicit preparation followed by the supported fresh attachment/check must
admit the expected current artifact. Reuse owner fixtures and small controlled changes rather than
rebuilding a native solver for every negative case; H02 owns one actual native consumer journey.
Delete replaced readiness/admission shortcuts and migrate their consumers together; retain useful
separate historical-receipt and import-compatibility observations.

## Dependencies, verification and publication

**Proposed:** LIFE01 consumes actual Plan53d contracts and CON03 diagnostics; it does not take over
Plan53 status or wait for all Plan53 qualification. LIFE02 can advance from existing original-owner
references, with CON02/CON03 contracts for useful typed presentation. LIFE03 is independent of protocol
tasks and CodeMode; coordinate source edits with NAV03 when both change specialist/worker wiring.
LIFE04 establishes the selected native basis before MCP06 relies on it; their host/native identity
contract can be agreed together without waiting for unrelated OPT or broader Plan53/54 completion.

Use focused owner/adapter/cancellation tests and affected compilation during implementation. Existing
Python/asyncio/context-manager and native cancellation primitives are preferred where adequate;
`AsyncExitStack` or structured sibling settlement needs a real acquisition/cleanup responsibility.
AnyIO solely to rewrite working asyncio and Tokio `spawn_blocking` as an imagined abort mechanism are
not selected. No new scheduler, per-command receipt ledger or global retry policy.

G01/G02 compose prepare→execute→handle→watch/read using independent native expectations; H02/H03 own
assembled and scope-end checks. Publish actual readiness/progress/release semantics in existing
capability/environment/diagnostic guides and derived reference. **Interface-checked** foundations and
**Proposed** delivery remain distinct; no new product or actual-host test was run to author this plan.


## Verification and Outcome — 2026-10-24

### What was built

**Implemented:** LIFE01–LIFE03 are complete. Structural prerequisites and passive readiness expose
independent recorded-preparation, import, selected-service, language-provider and operation facts.
Receipt currency and unobserved facts stay unknown. Explicit live checks use their original owner;
import checks use the fixed read interpreter and inherited endpoint provider environment without
preparing providers. Native stage preparation does not mutate the running endpoint's environment.
Separate development/native stage tools consume `python_extension.run_stages` directly, with typed
stage status/exit/artifact/receipt/publication outcomes, one admitted environment and no duplicate
builder. Stage callbacks observe actual transitions; stubs-check does not publish a receipt.

**Implemented:** request-private bounded atomic progress snapshots carry actual owner-issued stages
and references. The observer coalesces them through `Context.report_progress` with the caller's token,
unknown total and no invented percentage. It retains issued identities despite notification loss;
attached spawn/wait/signal/drain ownership remains shielded. `rust_test_reference_read` reads the
original bounded reference under held no-follow directory descriptors and authenticates its registry.
It leads to the existing native watch/read path; absence never establishes admission or retry safety.
No tasks scheduler, detached-action promise, second outcome database or tracing claim was added.

**Implemented:** preview status and exact-digest release expose retained counts/bytes/capacity and
per-handle admission/reconciliation state. The original 32-preview/64-MiB defaults remain. Admission
and release share the owner lock; release refuses while admitted or unresolved. Interrupted apply
keeps its digest reconcilable; all edited/basis paths, including missing/diverged/non-UTF-8 bytes,
must be classified before explicit release. Unknown/foreign/repeated release stays unavailable,
siblings survive, and release neither edits nor rolls back source. Reset changes presentation only.

**Tested:** the shared final 260-capability controls in the [52m Outcome](52m-progressive-guidance-and-operation-availability.md#verification-and-outcome--2026-10-24)
passed against zero failures, including small-capacity release/race/cancellation controls, incompatible
imports, passive no-startup, selected unavailable providers, no-token fallback and notification loss.
`scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_python_extension
scripts.tests.test_capability_observation` passed **36 tests, zero failures** after final repairs.

**Tested:** fresh actual STDIO sessions established explicit native prepare→admit→live import→native
checkpoint inspection, preserving independent expected operation identity/outcome and correctness
features. An unprepared endpoint separately reported import-incompatible and the real read refused;
no readiness check built or repaired it. The final prepared read passed after worker contract
migration. Retained drivers and observations are under `build/capabilities-52mn/`; positive preparation
is `stdio-1792863539918165052`, negative import is `stdio-1792863423667882569`, and final native read is
`stdio-1792864069378515519`. Drivers used the actual fixed STDIO transport and explicit native
environment, with complete environment inheritance for the prepared endpoint.

**Tested:** actual pure Rust invocation `b0a09071f9ca468b84a81cd55531b127` exposed its authentic reference
both over wire progress and the original-reference reader **before action return**. The final reader
matched original native terminal identity, one selected pass, workload exit zero and assurance pass.
The native owner reported drain **unobserved**; the first probe's overly strong complete-drain
assertion failed. Independent assessment of the retained raw observations confirms the selected
handle/progress/result behavior without upgrading that drain fact. Evidence is
`build/capabilities-52mn/early-1792863933911280110/{observations,outcome}.json`.

**Tested:** the fresh actual Codex journey in the 52m Outcome independently froze one pass with
complete owned drain for invocation `fd954ed1611d4eb2a9ba681c9086c49a`. Its JSONL exposed no progress
records; Codex notification rendering remains **unestablished**, while its authentic final native
reader path is qualified. Wire progress was observed by the real STDIO client, not inferred from
Codex prose. Scope-end checks are recorded by the [coordinator](52-agent-effectiveness-and-engineering-workflows.md#52m52n-execution-checkpoint--2026-10-24).

### A mistake made and corrected

**Tested:** independent review found missing/foreign `VIRTUAL_ENV` aliases in the already admitted
stage worker. The original environment owner now binds installer aliases without repeating admission,
with absent/foreign regressions. Actual STDIO then exposed a readiness false positive: the checker
prepared providers while the read did not. Import checking is now a separate fixed environment-neutral
worker using the actual read interpreter; preparation and reattachment remain explicit. Another review
found an ancestor-symlink race in reference reads and non-UTF-8 reconciliation failures; held directory
descriptors and raw-byte hashes now have deterministic regressions. Initial test/probe failures remain
attributed; final targeted controls pass against zero.

### Deviations from the plan, deliberate

**Implemented:** no generic readiness boolean or capability-exercised ledger was introduced.
Preparation receipts remain historical evidence, and import checks accurately represent the inherited
read environment. Foreground snapshots are bounded observation, not a resumable protocol task. Actual
Codex progress rendering is not claimed from an in-process or STDIO pass. Native drain uncertainty is
preserved rather than changing the native runner merely to satisfy an acceptance expectation.
No dependency or new scientific boundary was added. Enduring operation is documented in the existing
[interface](../dev/capability-interface.md) and [environment](../dev/agent-environment.md) guides.
This completed companion stays for active coordinator/G/H consumers; Plan53, I14, F02 and 52o retain
their independent obligations. Full solver-fleet, performance and assembled Plan52 qualification
were not run or claimed.

## LIFE04 verification and outcome — 2026-10-24

**Implemented:** selected native readiness now distinguishes historical preparation/import evidence
from a current installed artifact for the admitted checkout, interpreter, mode, profile and validated
features. Passive discovery stays source-only. The executing native worker checks installed currency
before importing the extension, binds the selected file witness, and checks the imported path/file
again before entering the receiver. A changed or already unbound loaded artifact refuses rather than
borrowing currency from a different subprocess import. Original preparation remains explicit and
retains its actual publication/receipt result; no new builder, universal readiness bit or replay
permission was added. MCP06 owns attachment, launch and private route/source agreement.

**Tested — zero failure target, actual current native consumer:**
`scripts/pse-env -- .venv/bin/python -m scripts.tests.plan52_conditional_acceptance --host build/capabilities-52o/current-native-consumer-20261024 --fixture build/capabilities-52o/conditional-f01-authentic-inputs-20261024 --prepare-native --timeout 900`
passed 30 real Codex calls. The outer launcher used the ordinary environment; the explicit endpoint
wrapper owned its complete native allocation. The original comparison first reported
`pse.native.artifact_not_current` with an `inputs-changed` basis. Exactly one separately authorized
`python_extension_native_stage` request selected `stage=prepare`, `profile=dev`, `validated=true`.
Its original owner returned exit zero and publication of a native artifact with
`force-validate`, `native-solvers` and `solver-highs`, SHA256
`03d64563c90eeb89f78ceebb2eff680fcac2f72f68ef013f956e3ba643b5995c`.
The subsequent fresh native worker returned the independently expected F01 document and exact cut
`62dc140dacb87de90786b44f078d83dc27f052c6d2d75aec7f49c668ec2bfd58`.
Original source coverage, imported-cut-only freshness, three/four test denominators and unexecuted
membership survived the actual serialized response. The original preparation result, before/after
answers and callback provenance remain in that host directory's `outcome.json`, raw JSONL and wire
capture. No database service or scientific workload was started for this consumer.

**Tested:** the inquiry guard retained one exact setup outside the read-only composition closure.
Eleven controls in
`scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_plan52_conditional_acceptance`
passed with zero failures, including actual-current and exact stale before-bases and refusal to treat
an unrelated native error as repairable currency. The actual run observed stale inputs; the
actual-current branch is controlled-test evidence. Setup was not replayed. The additional successful
CodeMode read in this disposable inquiry establishes no production adoption or productivity benefit.

**Implemented; Tested — mistake corrected:** checking a fresh import in one subprocess could
previously approve a different artifact from the executor's already loaded extension. Currency now
precedes native import and is bound to the executor's observed file. Earlier preparation/readiness
and source-change refusals retain their original outcomes. An explicit setup updates its selected
artifact; it does not mutate an existing attachment's inherited environment or certify science.

**Proposed — evidence limit:** this actual native proof was recorded after admission review baseline
`3e99a220f74b31a063d3678534ff445d38ad8b7f` and before the ongoing study-result writer repair.
The retained artifact/source basis is the proof boundary; subsequent Rust changes require fresh
current-artifact admission and affected evidence. Earlier LIFE01–LIFE03 observations remain at their
recorded scope, including unobserved drain and Codex progress-rendering limits. Production F03,
CodeMode/tasks, interactive UI and human operator use remain unselected or unestablished at their
existing owners. H02/H03 retain assembled and scope-end qualification; no full campaign, scientific
adequacy or measured productivity claim follows from this read/admission delivery.
