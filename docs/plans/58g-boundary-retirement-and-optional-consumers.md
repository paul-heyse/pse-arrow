---
title: "Plan58: Boundary retirement and optional consumers"
status: in-progress
date: 2026-10-28
adrs: []
review_sources: [docs/functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md, docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md, docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md]
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, architecture, design, libraries, validation]
---

# Boundary retirement and optional consumers

## Target and foundation

**Proposed:** retire first-party Python operating machinery and the Python MCP stack while preserving supported scientific outcomes through Rust owners. O03/O04 supply the original opportunity; the [improvement review](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I01), especially I01/I07–I09/I12/I13, supplies the integrated cutover and requirement reductions. The [coordinator](58-rust-owned-system.md) owns their dispositions. The original authoring baseline was `c3b7f136d`; subsequent checkpoints record delivered retirements. This document amendment itself implements no product replacement.

A cutover includes the retained outcome's native owner, real callers, configuration, generator declarations, tests and old-path deletion. Temporary incomplete development states receive no acceptance credit, but need no compatibility bridge. G contributes to each owning change; it is not a final cleanup wave that permits replaced mechanisms to linger. Independent cuts need not become one repository-wide commit.

No full Python API, FastMCP feature parity, CLI or MCP requirement survives merely because it once existed. Existing useful CLIs may stay as thin adapters with direct native calls. No universal operation catalogue, Rust proxy for a FastMCP child, replacement agent framework or embedded Python controller is selected. Qualified native C/Fortran solvers and ordinary third-party subprocess tools remain eligible.

The current facade, control client, generated documents/stubs, native worker bridge, retained MCP sessions, source/extension currency checks and CLI generation have different consumers. Inspect those consumers to locate deletion boundaries, not to infer that each mechanism must be ported. Rust types and validated construction carry established internal premises; remaining external files, requests and persisted records still require their actual boundary validation.

The [database-boundary review](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md)
is incorporated at `a6a144bce` on 2026-10-29. The coordinator owns its source-qualified
opportunity dispositions; D owns the bounded canonical action/connectivity contracts and E
the payload boundary. G supplies integrated caller/declaration retirement when those actual
replacement capabilities work. This adds no automatic engine migration or blanket deletion.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| G01 | in progress | 58g | Retained/lost behavior and integrated caller/declaration deletion boundaries classified |
| G02 | done | 58g | Python MCP, global catalogue, frontend sessions and generated clients retired; joined native consumer qualification remains with B/C/D/H |
| G03 | in progress | 58g | Required Python product projections and operating dependencies retired with their declarations and consumers |
| G04 | planned | 58g | Future optional binding seam defined; unnecessary ABI/catalogue/frontends explicitly unselected |

<a id="g01"></a>
### G01 — Select outcomes and deletion boundaries

Consume T01/T02 and A/D/E/F capability designs. Classify public authoring/result/control use, developer commands, prompts/navigation, source editing and observation/recovery access by retained outcome. Record intentional losses with rationale and any ordinary tool/API replacement. Scientific calculation/progress observation remains; bespoke build observation and its derivative clients/alerts do not. A shell/editor workflow may replace an agent-specific editing service without a new preview state machine.

Use I08's actual consumers to distinguish deletion from replacement. F owns replacement of the active `grpc_wire_schema.py` invocation by Rust parsing/generation and retained native-engine boundary checks. G removes frontend-only generated private/attrs modules, stubs, manifests, dynamic self-check expectations and configuration references with their last retained caller. Explicit resource-conversion actions, SurrealKit beta recipes/pending-state branches, `IDAES_NAMES` and probe assets require reachability/meaning assessment; a pathname or old schema difference alone does not establish disposability. Coordinate native migration infrastructure with D and development reconstruction with F; no existing store or authored/reference evidence is erased by this plan.

For boundary O01/O03/O04/O08, name the native operation that replaces each client-side
readiness traversal, authoritative policy interpretation, reconnect/progress mirror, redundant
codec or data-holder bridge before deleting its last retained caller. Classify the current
worker loop by responsibility: D can absorb shared-state discovery/progression, while Rust
must still prepare, execute, propagate cancellation and join effects. `CandidateGroup` only
suppresses duplicate local preparation; canonical claims do not automatically replace that
useful role. SDK delegation removes only plumbing whose required contract it actually covers,
not original request custody, unknown-ACK reconciliation or native physical drain by name.

Targeted acceptance is a concrete retained consumer transition with input/output/failure expectations and absent obsolete dependencies. It is not a module-count target or blanket ban on reference-only Python.

<a id="g02"></a>
### G02 — Remove Python MCP as a system dependency

In each integrated product/developer cutover, delete serving, fixed worker/native read subprocess bridges, retained Python Runtime sessions, generic invokers, visibility transforms, generated MCP CLI clients and reconnect/admission/currency checks whose consumers disappear. Migrate or intentionally drop each meaningful `scripts/capabilities` consumer. Update recipes, agent attachments/configuration, generation and dependency declarations together. Retire `WorkerFamily`, import eligibility and host-envelope projections when their consumers leave; do not translate them into a global Rust dispatch registry.

Preserve actual executing-artifact selection and physical settlement where retained consumers need them, rather than every frozen-Python closure hash or mirror-consistency fence. No alternate Python server, hidden compatibility invoker or transitional RMCP proxy is permitted. Direct Rust applications are sufficient consumers; progressive disclosure, guidance and editing presentations do not block native delivery.

Targeted tests exercise retained behavior directly at Rust owners and startup/help/configuration with Python/MCP unavailable. Removed interface tests retire with the interface. H owns joined frontend-independent acceptance.

<a id="g03"></a>
### G03 — Retire required Python product/runtime adapters

Integrate canonical-state retirement with D03/D04 and data consumers with E01–E03. A completed
cut includes actual caller/configuration/registry or generator changes, replacement tests and
old-path deletion together. Retain scientific/reference assertions at their real native owner;
remove mechanism-only tests with the mechanism. Do not keep a compatibility shim or duplicate
state journal to bridge an unavailable intermediate path. A payload's accepted metadata and
its physical publication/custody remain different responsibilities after Python disappears.

With each migrated retained consumer, remove the corresponding scientific facade, HTTP orchestration, generated Python mirrors/stubs, PyO3-specific workflow roots and build dependencies. Change generator emissions, manifests, registration and self-check declarations, then regenerate through the owning generator; never hand-edit generated output. Reconcile version/build metadata authority without retaining package-facing Python merely to validate its own removed projections. Generation is explicit development/build publication, never an import/read/help side effect.

Do not first build public pyclasses, frozen document wrappers, pythonize conversion or Python OTel integration to replace a deferred Python product. Native constructors still enforce retained physical context, finiteness and version/unknown-field rules at real inputs; internal values need no repeated mirror walk.

The current parity candidate harness imports `pse`, generated types/test fixtures and constructs `pse.Runtime`. F04 must move candidate execution to native scientific operations and exchange explicit cases/results with isolated IDAES/Pyomo, SciPy or teqp reference execution. Preserve independent calculations, tolerances and interpretation, including a deliberately wrong candidate result. An unchanged facade-dependent parity harness is not oracle isolation. This exchange needs no public product CLI or ongoing extension delivery.

Delete obsolete interpreter wait policies only with their actual consumers; C owns native lifecycle guarantees. Any temporary black-box harness challenges the native replacement without retaining the old production implementation as its expected-answer oracle. Targeted tests establish native production/bootstrap independence and reference-environment isolation, not language purity of every dependency.

<a id="g04"></a>
### G04 — Define future adapters; conditional adoption only

Document a coarse optional PyO3 seam over Rust-owned requests/results/operation handles: adaptation belongs to the binding, policy and lifetime to Rust. This design deliverable does not require implementing a Python package, GUI, public native class family or second cdylib. A `cdylib` artifact does not provide an API/ownership contract; current `_native` has CPython assumptions. Ordinary linked Rust APIs remain the immediate target. No mandatory C ABI, plugin system or stable-ABI framework follows without an actual foreign consumer. PyO3 detachment does not establish native destruction or cancellation.

RMCP remains deferred until a concrete agent workflow earns it beyond direct APIs/tools. If triggered, compare no MCP against a small typed STDIO adapter using `ServerHandler`, `ToolRouter`, `Parameters<T>` and structured results. Reuse domain-owned operations rather than generating three required faces from a catalogue. RMCP3.7 service close can cancel handlers after bounded drain; domain owners settle separately. Actual host support is required before a delivery claim. Source: [tagged RMCP service](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/crates/rmcp/src/service.rs); the source review owns release-specific research. No FastMCP parity or MCP task scheduler is required.

## Verification and handoff

**Proposed:** G01–G04. No runtime execution or product qualification occurred for this amendment. H receives retained/lost outcomes, final dependency closures and actual integrated caller/declaration deletions. Optional consumers cannot hold native core completion open. Applicable rule/architecture changes precede dependent implementation; this document does not change ADR status or authorize data disposal.

## Native repeated-analysis consumer cut — 2026-10-29

The authorized A01–A07 execution retires Python `FrozenStudyCohort.aggregate` and
`StudyKpiAnalysis`, their PyO3 method/class, module registration, aliases, exports and generated
stub. The native `FrozenStudyCohort::aggregate` and retained-data export operations remain.
The direct Rust repeated-analysis caller preserves original captured-unavailable membership,
typed aggregate statistics, exact attempt/manifest attribution and corrected-context comparison;
its selected evidence is [owned by A](58a-native-product-and-scientific-workflows.md#native-repeated-analysis-and-scientific-caller-cut).
The existing CLI export test retains its independent expected aggregate rows directly rather
than calling the deleted Python aggregate projection. No replacement Python adapter is added.

**Implemented:** native retained readers/cohorts retain `RetainedDataServices`; mathematical
services are no longer part of their actual capability. Separately consumed Python frozen
browsing, stored-result and stream wrappers still retain their full Python runtime owners.
Those wrappers and remaining scientific consumers keep G01/G03 open; removing this one projection
does not resolve the entire Python read-owner finding or establish complete facade retirement.

**Interface-checked:** the reduced native module builds with zero failures through
`scripts/pse-env --native=compiler,solver,klu -- scripts/cargo-bin/cargo-pse-correctness build
--locked -p pse-py --lib --features solver-kinsol,solver-ipopt,solver-scip,solver-highs,solver-diffsol
--message-format=json`. `just python-stubs --extension target/debug/lib_native.so`, under the
same prepared environment, publishes the actual compiled metadata. Inspection confirms only
the retired class/method and its provenance leave generated output. Independent bounded source
review found no material actionable issues. No extension installation, Python component journey,
scope-end static campaign or complete A/G/H qualification is claimed.

The following A05 cut also deletes Python `GeneratedDesign`,
`ModelingPackage.generate_design`, their complete PyO3 sampling module, registration,
aliases/exports and dedicated `test_designs.py` wrapper/replay tests. The public native design
target supplies store-free geometry/refusal/retention controls; the actual recorded-study
consumer retains generated-point replay with non-positional occurrence identities.
[A's generation checkpoint](58a-native-product-and-scientific-workflows.md#native-generation-and-prerequisite-acceptance)
owns the passing evidence and the corrected ordinal/key distinction. The reduced native build
and compiled-metadata publication pass with the same commands above; the generated diff removes
only this retired projection and its provenance. Native design/document contracts remain.
This additional deletion removes the Python handle's retained PreparationServices/math owner,
without redesigning another Python adapter. Generic conformance and thermodynamic campaign
orchestration remain separate F consumers with their shared document reader intact.

## Selected ordinary host retirement checkpoint — 2026-10-28

The B02/B03/F01/F02 wave now cuts ordinary environment/foreground/Cargo entry points to Rust.
Direnv, the SessionStart hook, Just shell/interpreters, Cargo aliases and generic check/unit
recipes call the native owner. Python pse-env and Arrow command entries explicitly refuse;
no Python compatibility launch or native-guard fallback was added. Four Python-evaluating native
export scripts and the Python per-compiler cache watcher/launcher and their dedicated watcher
tests are deleted. Remaining frontend compiler configuration selects the real native sccache
client; it cannot veto valid output on cache-service death. Frozen installer source lists and
validation support declarations no longer require the deleted compiler wrapper.

Bespoke build observation/measurement/cache-probe producers and dedicated tests, old Python
activity CLI, build-only telemetry instruments/stages and derivative clients are deleted.
`just activity` now reads current/named manager ownership directly. Detached scientific result
watches/cursors, pressure observations, generic engineering telemetry, historical paired-report
comparison and independent scientific/reference assets remain. Regression-report comparison is
read-only; old execution/bisect/frozen-controller experiment machinery is removed. Corresponding
recipes/inventory declarations were reconciled; capability reference output was regenerated
through its owner. No authored inputs, databases, cache contents or historical result evidence
were deleted by source retirement.

Retirement is deliberately scoped: legacy `pse_env`, `host_admission`, `native_operation`,
`operation_lifetime`, `systemd_environment`, frontend graph helpers and related report/native
fixture libraries still have unmigrated D/F/product consumers. They are not an accepted
alternative for this native path. G01/G03 remain in progress; G02's Python MCP/product facade
removal and G04's optional consumers were not expanded by this wave. Source-only report
readers retain old-run interpretation; their existence does not establish native host coverage.
Remaining callers must migrate or intentionally retire with their owning integrated cuts.

**Tested:** retained telemetry/watch controls pass 20 tests and report/investigation/inventory
controls pass 26 tests against zero failures. The legacy per-compiler watcher suite and replaced
command/controller tests retired with their mechanism; replacement acceptance comes from native
owner controls, not ported expectations of deleted machinery. The interrupted broader legacy
build-environment test has no acceptance credit; its relevant pure configuration class passes
15 tests. Live telemetry backend/export acceptance and full frontend retirement are not claimed.
H owns joined static, scientific and system qualification after the selected functional scope.

The follow-on compiler/KLU slice deletes the Python KLU constructor, construction CLI option
and provider options; `native_operation` refuses KLU setup/environment requests before effects.
It moves `unit-native-capability-package` from the Python unit selector to the native
compiler/KLU boundary and Cargo/nextest composer, retaining its arguments and correctness
closure. Legacy KLU file/receipt readers remain for unmigrated consumers of historical or
other provider artifacts; they are not an alternative KLU constructor. Other native providers,
their helper-image consumers, Python MCP and product frontends retain their existing owners.
G01/G03 remain in progress. [58b](58b-bootstrap-and-host-control.md#explicit-compilerklu-consumer-checkpoint--2026-10-28)
owns the selected replacement's verification and limitations.

## Selected Python asyncio run-wait retirement — 2026-10-28

The maintainer selected deletion of NativeRunHandle.wait_async, Python RunHandle.wait_async
and CancelWait. Their dedicated cancellation test retires; retained scientific assertions
use synchronous joins, with asynchronous cursor work finished/detached before joining.
The shared synchronous run handle and unrelated asynchronous cursor adapters remain for
later cutovers, including their still-needed pyo3-async-runtimes dependency. No shim,
thread-offload adapter or second Python wait coordinator replaces the removed bridge.
Compiled metadata regenerates the changed stub through the existing owner.
[58a's local checkpoint](58a-native-product-and-scientific-workflows.md#native-local-operation-checkpoint--2026-10-28)
owns results and remaining qualification; G01/G03 remain in progress.

## Python MCP retirement checkpoint — 2026-10-29

The Python FastMCP server, fixed capability/native-read workers, retained frontend sessions,
generic invokers, generated capability CLI, Codex attachment/reconnect helpers and mcpls
presentation clients are deleted with their dedicated tests. Their recipes, validation
declarations, dependencies and agent settings are reconciled. The global Runtime operation
catalogue, worker-family/eligibility/envelope projections and PyO3 catalogue exposure are removed;
the retained project-operation declaration owns its still-consumed effect facts directly.
No Rust dispatch catalogue, replacement MCP server or compatibility invoker was introduced.

Passive telemetry/evidence/report readers retain their real schema and report owners without
importing endpoint code. Structural migration retains its original preview/apply/reconcile
mechanism. Schema generation prunes retired MCP outputs through its owner; the obsolete
Python rendering dependency is removed from the lockfile without upgrading other packages.
The current architecture and interface guide now describe the retirement, preserving historical
MCP qualification as original evidence. The installed Codex CLI confirms that `pse-capabilities`
is no longer configured; an already-running session can retain its earlier attachment snapshot.

**Tested:** the retained pure-tooling selection passes 128 controls against zero failures.
After refreshing the selected force-validated development extension, `scripts/pse-env --
.venv/bin/python -m unittest scripts.tests.test_telemetry_reads
scripts.tests.test_telemetry_process` passes all 31 controls against zero failures; this resolves
the earlier stale native-extension import error. An initial pytest invocation stopped before
collection on a foreign-owned temporary root and supplies no test evidence. The stdlib unittest
route is the owner for these controls; no foreign temporary directory was changed.
`just py-extension-prepare dev` and `just py-stubs-prepare dev` complete successfully; compiled
metadata no longer exports the catalogue. Full `just codegen` publishes all ten schema targets.

G02's functional retirement is complete. G01/G03, the still-used scientific Python/PyO3 facade,
parity candidate isolation and remaining host/runtime/control consumers are separate open work.
These focused results do not establish assembled B/C/D/H or live exporter acceptance, and no
performance benefit is measured. Formal ADR statuses remain unchanged.

The native covariance, sensitivity and authored PID exchanges now replace their three Python
candidate parity tests. [58f's reference cutover](58f-development-and-assurance.md#native-fitting-sensitivity-and-pid-reference-cutover--2026-10-29)
owns their preserved scientific assertions, independent oracles, adverse controls and targeted
results. Together with the earlier degeneracy exchange, these remove four candidate families;
the remaining actual parity support and scientific Python/PyO3 consumers keep G01/G03 open.

## Scientific worker boundary cut and stop checkpoint — 2026-10-29

The Rust scientific path no longer locates/starts a Python primary receiver or invokes Python
to construct its allocation. `NativeStudyWorker` owns the actual process and lends a bound
observer to callers. The child directly constructs the Rust runtime and serves canonical work;
its private handoff is supplied only by the native performing owner. The obsolete standalone
`pse-worker` recipe is removed; `worker-test` exercises the native route. Process journeys use
private native services, exact prepared module selection and original typed kill/drain authority.
The test-only environment projection removes runner metadata with invalid service variable names;
production environment preflight validates names/values and preserves literal values.

Existing library-level managed fixture cases are source-migrated but still consume historical
fixture/context construction and are not accepted by these process results. Remaining Python
store/provider helpers, scientific/PyO3 callers and other independent-reference families retain
their actual consumers and open G01/G03 ownership. Historical receiver metadata is not falsely
claimed deleted. [58c](58c-runtime-lifecycles.md#native-scientific-worker-closeout--2026-10-29)
owns targeted results; the whole Plan58 B/C/D/G selection remains incomplete and paused.
