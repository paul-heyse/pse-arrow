---
title: "Plan58: Boundary retirement and optional consumers"
status: draft
date: 2026-10-28
adrs: []
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md, docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md]
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, architecture, design, libraries, validation]
---

# Boundary retirement and optional consumers

## Target and foundation

**Proposed:** retire first-party Python operating machinery and the Python MCP stack while preserving supported scientific outcomes through Rust owners. O03/O04 supply the original opportunity; the [improvement review](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I01), especially I01/I07–I09/I12/I13, supplies the integrated cutover and requirement reductions. The [coordinator](58-rust-owned-system.md) owns their dispositions. Baseline is `c3b7f136d`; no product replacement is implemented by this amendment.

A cutover includes the retained outcome's native owner, real callers, configuration, generator declarations, tests and old-path deletion. Temporary incomplete development states receive no acceptance credit, but need no compatibility bridge. G contributes to each owning change; it is not a final cleanup wave that permits replaced mechanisms to linger. Independent cuts need not become one repository-wide commit.

No full Python API, FastMCP feature parity, CLI or MCP requirement survives merely because it once existed. Existing useful CLIs may stay as thin adapters with direct native calls. No universal operation catalogue, Rust proxy for a FastMCP child, replacement agent framework or embedded Python controller is selected. Qualified native C/Fortran solvers and ordinary third-party subprocess tools remain eligible.

The current facade, control client, generated documents/stubs, native worker bridge, retained MCP sessions, source/extension currency checks and CLI generation have different consumers. Inspect those consumers to locate deletion boundaries, not to infer that each mechanism must be ported. Rust types and validated construction carry established internal premises; remaining external files, requests and persisted records still require their actual boundary validation.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| G01 | in progress | 58g | Retained/lost behavior and integrated caller/declaration deletion boundaries classified |
| G02 | planned | 58g | Python MCP and its bridges removed with retained consumer cutovers |
| G03 | in progress | 58g | Required Python product projections and operating dependencies retired with their declarations and consumers |
| G04 | planned | 58g | Future optional binding seam defined; unnecessary ABI/catalogue/frontends explicitly unselected |

<a id="g01"></a>
### G01 — Select outcomes and deletion boundaries

Consume T01/T02 and A/D/E/F capability designs. Classify public authoring/result/control use, developer commands, prompts/navigation, source editing and observation/recovery access by retained outcome. Record intentional losses with rationale and any ordinary tool/API replacement. Scientific calculation/progress observation remains; bespoke build observation and its derivative clients/alerts do not. A shell/editor workflow may replace an agent-specific editing service without a new preview state machine.

Use I08's actual consumers to distinguish deletion from replacement. F owns replacement of the active `grpc_wire_schema.py` invocation by Rust parsing/generation and retained native-engine boundary checks. G removes frontend-only generated private/attrs modules, stubs, manifests, dynamic self-check expectations and configuration references with their last retained caller. Explicit resource-conversion actions, SurrealKit beta recipes/pending-state branches, `IDAES_NAMES` and probe assets require reachability/meaning assessment; a pathname or old schema difference alone does not establish disposability. Coordinate native migration infrastructure with D and development reconstruction with F; no existing store or authored/reference evidence is erased by this plan.

Targeted acceptance is a concrete retained consumer transition with input/output/failure expectations and absent obsolete dependencies. It is not a module-count target or blanket ban on reference-only Python.

<a id="g02"></a>
### G02 — Remove Python MCP as a system dependency

In each integrated product/developer cutover, delete serving, fixed worker/native read subprocess bridges, retained Python Runtime sessions, generic invokers, visibility transforms, generated MCP CLI clients and reconnect/admission/currency checks whose consumers disappear. Migrate or intentionally drop each meaningful `scripts/capabilities` consumer. Update recipes, agent attachments/configuration, generation and dependency declarations together. Retire `WorkerFamily`, import eligibility and host-envelope projections when their consumers leave; do not translate them into a global Rust dispatch registry.

Preserve actual executing-artifact selection and physical settlement where retained consumers need them, rather than every frozen-Python closure hash or mirror-consistency fence. No alternate Python server, hidden compatibility invoker or transitional RMCP proxy is permitted. Direct Rust applications are sufficient consumers; progressive disclosure, guidance and editing presentations do not block native delivery.

Targeted tests exercise retained behavior directly at Rust owners and startup/help/configuration with Python/MCP unavailable. Removed interface tests retire with the interface. H owns joined frontend-independent acceptance.

<a id="g03"></a>
### G03 — Retire required Python product/runtime adapters

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
