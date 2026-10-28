---
title: "Plan58f: Development and assurance"
status: draft
date: 2026-10-28
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, environment, validation]
review_sources:
  - docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md
  - docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md
---

# Plan58f: Development and assurance

This companion develops O02/O11 and developer O03/O09. The [coordinator](58-rust-owned-system.md)
owns outcomes, rules and qualification; this document owns package state. Plan57 remains superseded historical evidence.

The [hard-cutover review](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md)
supplies I01/I03–I14 for this amendment at `c3b7f136d`; its production source baseline remains
`bd1c4891e`. The coordinator owns recommendation disposition.

## Baseline and foundation choices

The source functional review inspected `bd1c4891ea7d71178e809b11045dd4fd89602c0f`.
Rust already owns xtask, scientific contracts and generators, while `scripts/` still owns
first-party environment/configuration, compiler coordination, correctness composition, test
fixtures, documentation/governance and evidence workflows in Python. Existing capability CLI
invoke/session routes still enter MCP. Moving their presentation alone leaves the operational
dependency intact. Source inspection of `xtask/Cargo.toml`, `scripts/arrow_validation.py`,
`scripts/test_run.py`, compiler launchers and agent configuration confirms these boundaries;
it establishes neither complete native coverage nor a performance result.

Use Core 3.4, efficiency heuristics 1.0, test assessment 1.2 and the selected pse-arrow binding.
Blueprint §3.2 and §24.1/§24.2 own product/development separation and qualification. Retain
the purpose of scientific acceptance and independent references under process-simulator 1.5;
this plan does not broaden the supported scientific modes or replace their numerical owners.
The [environment](../dev/agent-environment.md) and [qualification](../dev/validation-assessment.md)
guides remain current until the coordinator's decision/publication route changes them.

Separate the small bootstrap/host root in [58b](58b-bootstrap-and-host-control.md) from richer
development roots. xtask remains suitable for registry/codegen and scientific development
operations that need its dependencies; it is not the cold-start seed. Developer commands use
typed Rust functions and original operation handles directly. CLI adapters are optional;
help and structured output can serve them. A unified command tree, universal catalogue, generated
dispatcher and MCP presence are not completion conditions. Keep CLI/agent/frontend libraries
outside ordinary product dependency closures. Existing Rust owners can remain separate roots.

## Retained value and retirement

First-party environment composition, agent/runtime configuration, installation, compiler-cache
configuration, necessary preparation, fixture ownership, generation, docs/governance and
evidence collection/assessment move to Rust wherever their value remains. This includes
source-only developer operations: removing Python production imports while retaining Python
as the mandatory development controller would not satisfy O02. Ordinary third-party tools
remain subprocesses with their native arguments and exits, including tools implemented in
Python; no first-party Python wrapper supplies their policy. Independent scientific oracle
drivers remain isolated reference consumers with explicitly selected environments.

Preserve finite effect distinctions, selection inputs, exact integers, omitted versus explicit
values, original invocation identity and complete terminal outcomes at retained interfaces.
Typed in-process calls do not round-trip through JSON or generated Python DTOs. Serialization
belongs at command/file/network boundaries. Configuration changes operate on explicit files,
preserve unrelated user content and refuse ambiguous ownership. Installation remains explicit;
help, offline evidence and configuration explanation do not silently prepare scientific code.

Retire MCP/Python-only preparation/source-currency scans, reconnect/session progress files,
generated CLI glue and duplicate publication fences when their last consumer is gone. Retain
checks for actual compiler inputs, immutable artifact selection, host ownership and external
wire admission where their uncertainty remains. A loaded native process does not inherit
per-read source rescans. Each deletion carries its migrated callers, tests and fixtures;
there is no permanent Python fallback or second catalogue to maintain.

For test migration, preserve the behavioral purpose at the narrowest genuine Rust owner or
public executable. Do not port wrapper identity assertions after that wrapper disappears.
Fixtures own original cancellation, service borrowing and physical drain through 58b; test
selection, scientific outcome, tool failure and fixture failure remain distinguishable.
Expected scientific values must retain analytic/independent reference provenance; deriving
expectations from the same migrated implementation cannot establish scientific correctness.
Original SIGSEGVs, failed runs and interrupted evidence remain visible at their existing owner.

Retained scientific/development evidence keeps original authority/codec/identity, selected population and coverage,
explicit unknowns and protected raw detail. Failed, skipped, unreported, reused, transferred
and command-failure outcomes stay distinct. Existing optional tooling indexes are disposable
projections, not a new canonical source or host-liveness store. Offline original-owner/detail
inspection must work without database, mathematical Runtime or compiler availability.

## Native tool and fixture design

Delegate complete runner responsibilities to pinned nextest 0.9.146: inventory, filters, static
groups/overrides and result/JUnit surfaces. Do not project per-selected-test TOML, maintain a
second CLI grammar, repeat inventories or embed nextest's internal runner. The overview targets
0.9.150, so use the [pinned setup](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.146/site/src/docs/configuration/setup-scripts.md),
[wrappers](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.146/site/src/docs/configuration/wrapper-scripts.md)
and [groups](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.146/site/src/docs/configuration/test-groups.md)
contracts. Setup/wrappers at this pin are experimental. Setup runs after discovery and can
supply `NEXTEST_ENV`; it cannot create compilation/listing prerequisites and has no general
teardown hook. A native enclosing owner establishes prerequisites before discovery and settles
resources after the run. `NEXTEST_RUN_ID` identifies one run's internal discovery/execution,
not arbitrary independent list/run commands. Keep setup failures separate from scientific cases.

Tests receive real typed store/worker capabilities through private construction and owned or
borrowed handles. Nextest groups schedule work; they do not grant credentials or effects.
Remove `test_requirements.py` and duplicated role/effect/test-name tables with their consumers.
Prefer private per-run resources; isolate any genuinely shared service borrowing. A fixture
settles its owned resources after assertions pass or fail; retain meaningful diagnostics outside
the disposable store. Do not require successful science before reclamation, infer success from
`Drop` or build a global terminal/evidence reconciler. Process death, actor authentication,
external identifiers and shared-service uncertainty still need enforcement at their boundary.
Fresh owned regenerable stores use D's current schema construction; arbitrary existing stores
and uniquely authored inputs/reference evidence are not disposable by implication.

Use native pinned sccache 0.19 for Rust and C/C++ compilation and ordinary Cargo diagnostics/exits.
Select the native executable/configuration at explicit setup/build establishment. The operator
selected on 2026-10-28 that cache-service loss alone does not invalidate a valid compiler result.
Remove per-compiler Python hashing, polling, identity ledgers and settlement wrappers instead of
translating them. Select server/client lifetime and compiler placement from the
[startup source](https://github.com/mozilla/sccache/blob/v0.19.0/src/commands.rs) and
[client-side architecture](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Architecture.md).
`SCCACHE_NO_DAEMON` changes detachment, not lazy-start eligibility. Qualify selected logging and
distribution modes; a real retained gap can justify a narrow adapter, not a new cache tier.

Retire `build_observation.py`, live build-stage/wait-lock/freshness consumers and build-specific
telemetry/alerts deliberately. Preserve scientific calculation/progress, diagnostics and durable
outcomes. Ordinary tool outputs do not reproduce every removed field, and no trivial build-time
or speedup claim follows. Implement the useful native environment/local telemetry resolver below
scientific dependencies: Rust `Host::from_environment` consumes serialized settings rather than
resolving all current `OTEL_*` inputs. Review assessment/pressure signals individually. Optional
exporters cannot gate scientific admission or local computation; collector removal/direct backend
ingestion needs an explicit disposition of queue/failure alerts rather than assumed equivalence.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [F01](#f01) | planned | 58f | Retained direct development/configuration operations and native settings resolution |
| [F02](#f02) | planned | 58f | Native tool delegation, Arrow closure selection and typed fixture composition |
| [F03](#f03) | planned | 58f | Rust generators/docs/governance consumers and retired Python emission |
| [F04](#f04) | planned | 58f | Proportionate evidence and independent native-candidate scientific assurance |
| [F05](#f05) | planned | 58f | Integrated bridge retirement and native developer acceptance |
| [F06](#f06) | deferred | 58f | Discrete explanation inquiry only when an actual consumer justifies it |

<a id="f01"></a>
### F01 — Make retained development operations directly callable

Use [T02's retained-outcome contracts](58-rust-owned-system.md#t02) and B01's small root. Group
operations by their real owner and dependency need; move useful configuration/skill synchronization,
tool installation and preparation explanation from `agent-config.py`, `library_skills.py` and
bootstrap/setup helpers into appropriate Rust owners. Retire unneeded Codex/runtime glue or
isolate useful host-specific integration outside the production closure. No generic agent
framework, global catalogue or mandatory three-face adapter surface is selected. Implement the
useful native environment/telemetry resolver with B, retaining scientific consumers and removing
build-only signals. Targeted controls cover configuration precedence/invalid inputs, optional
telemetry absence, selected preparation, missing tools, ambiguous aliases, preserved unrelated
configuration and offline help with Python/MCP unavailable. Migrate recipe and agent callers.

<a id="f02"></a>
### F02 — Delegate compiler/test execution and construct real fixtures

Consume B's usable host/environment primitives; database-backed fixtures additionally consume
D's schema-ready store capability. Pure compiler/list/test operations require no database. Low-effect
argument/graph policy may develop earlier. Establish the native sccache/nextest composition above,
including compile/list prerequisites outside nextest setup, then remove Python test-run/cache and
build-observation owners with all retained callers. Use ordinary native tool arguments and exits,
without a new umbrella runner. Reuse one Cargo feature resolution within an invocation for actual
target/host normal/build/dev Arrow-leaf `force_validate` coverage. Unknown closure refuses; pure
scopes add no unrelated targets. Do not persist a manifest/lock/argv-only cache: Cargo config,
environment, toolchain, target and source overrides affect the answer. Keep the pinned toolchain,
unwinding and configured parallelism; artifact-writer conflicts do not justify CPU caps.

Use libtest-mimic only where a correctness catalogue replaces a custom loop, with exact-case lookup
and execution-free listing rather than repeated whole-catalogue preparation. Criterion/scientific
measurement remains separate, as do doctests. Construct fixtures through the actual native owner,
not a global permission table. Targeted controls cover omitted validation leaves, host/build
contexts, cache hit/miss/compiler failure, cache loss with valid output, selected logging modes,
setup versus test failure, listing without execution, exact-case dispatch, fixture cancellation,
failed assertions with owned cleanup and genuine shared-service borrowing. Remove wrapper-specific
assertions and duplicated inventory/configuration grammars with their mechanisms.

<a id="f03"></a>
### F03 — Keep declarations and explicit generation authoritative in Rust

Use existing xtask/registry owners for retained docs/lifecycle/ADR/governance and generation. Keep
native metadata/status authority and sparse scope relationships. Generation is explicit build/dev
publication; normal execution, reads, imports and help do not regenerate or reconstruct a mirror
registry to validate compiled declarations. Rust types/macros/Serde and consumed external schema
emission replace redundant runtime mirror languages. No mandatory pyclass/pythonize replacement,
C ABI, plugin system or second cdylib is a prerequisite to deleting deferred Python delivery.

Replace `xtask/src/codegen/grpc_wire.rs`'s active `grpc_wire_schema.py` invocation with `syn`/existing
Rust parsing, then regenerate and delete the Python parser. Choose native Markdown/front-matter
parsers against the metadata semantics actually consumed, including duplicate-key and malformed
input handling. Move the retained dependency boundary contract from recipe-invoked
`native-engine-boundaries.py` to Rust closure ownership. Remove unused private/attrs Python
emission together with output declarations, manifests, registrations, dynamic self-checks and
last consumers. `check_generated.py` is a deletion candidate after remaining presentation/config
references and actual generation-check ownership are reconciled; do not create another checker.

Use library private staging/atomic filesystem mechanics where appropriate. Executable descriptor
replacement (B), immutable payload no-replace publication (E) and canonical scientific commits (D)
have distinct contracts; share compatible mechanics without a universal generation protocol.
Delete loaded-extension/source-hash/frozen-code publication checks whose purpose disappears.
External files, requests and stored records still need real boundary validation, established once
and carried structurally. Targeted controls cover consumed malformed/duplicate metadata, native
wire extraction, deterministic generation, stale-output detection without publication and read-only
comparison. Change generators and run `just codegen` as part of implementation, never hand-edit
outputs. Remove obsolete scripts, projections and fixtures in the same integrated cutover.

<a id="f04"></a>
### F04 — Preserve valuable evidence and independent science

Consume native product/control/data operations and F02's actual runner/fixture outcomes. Move useful
evidence readers, diagnostics and handoff through existing native owners; canonical SurrealDB remains
the selected index/state path, with no independent embedded evidence authority. Offline supplied-detail
inspection remains usable without a database. Preserve actual scientific failures, setup failures,
missing reports and leaks as distinct observations without another universal evidence framework.

Move valuable Python scenarios to the narrowest real Rust owner or consumer; retained black-box
assertions can temporarily challenge the replacement, but the old implementation is not its oracle.
`python/pse/parity/tests/support.py` currently constructs `pse.Runtime` and imports generated types
and fixtures. Replace candidate execution with native scientific operations and explicit case/result
exchange with the isolated IDAES/Pyomo, SciPy and teqp reference side. Preserve independently derived
answers, tolerances and physical interpretation. This requires neither a public CLI nor a Python
product extension. Controls include a deliberately wrong candidate result, mixed pass/fail/missing
populations, protected raw detail and optional-index failure. Initial durable evidence consumes D's
worker-replacement/lineage and adjacent-version migration cases; full historical conversion and
backup/disaster-recovery qualification is a separate persistent-deployment release obligation in H.

<a id="f05"></a>
### F05 — Finish integrated cutovers and the native developer journey

Compose F01–F04 and B through actual Rust APIs/executables: explain/prepare selected capabilities,
run a targeted check, inspect its outcome, diagnose failure and settle resources without first-party
Python/MCP. Each coherent cutover includes new owner, callers, configuration, generation, tests and
deletion. Temporary broken development paths have no acceptance credit; no FastMCP proxy, Python
fallback or compatibility DTO is needed to hide them. Independent cuts need not form one mega-commit.

The [I08 deletion assessment](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I08)
identifies real active consumers, not automatic preservation obligations. In addition to F02/F03:

| Candidate | Required integrated disposition |
| --- | --- |
| `resource_conversion.py` | Prefer reconstruction of explicitly regenerable development state; retire converter actions/fences/callers together. Protect existing persistent records and uniquely authored inputs. |
| SurrealKit beta tooling/schema copies | Reconcile explicit recipes and pending-state/operator branches with D's delivered native migration owner; retire the experiment when no retained consumer remains. Historical schema difference alone is not proof of invalidity. |
| `python/stubs`, `IDAES_NAMES`, probes/conversion assets | Check actual configuration/declaration reachability and independent reference needs; remove only retired meaning with consumers, tests and generated declarations. |
| Python boundary/developer dependencies | Coordinate G's facade/MCP removal with recipe, agent, generation and reference-environment consumers; retain isolated external oracles without retaining a required product extension. |

A live caller says where the cut must occur, not that its mechanism must survive. Do not port old
recovery/terminal-registry/build-observation assertions simply to preserve test counts. Documentation
and agent instructions move with actual callable owners and the coordinator's decision route.

<a id="f06"></a>
### F06 — Conditional discrete configuration explanations

Source review O14 remains deferred, not a production requirement. Reopen only when a retained
developer needs concrete witnesses or conflicting premise sets beyond ordinary typed validation.
Compare Z3 `assert_and_track`, `check_assumptions`, `get_unsat_core` and `get_reason_unknown`
against that named question; see [Solver API](https://docs.rs/z3/0.21.1/z3/struct.Solver.html).
A decision must explain thread/context ownership, bounded execution, Sat/Unsat/Unknown meaning
and source-premise mapping. An unsat core need not be minimal. Do not replace continuous
scientific feasibility or routine host admission with a solver. Reject/defer if the explanation
does not earn another dependency and lifecycle. If selected later, migrate its consumer and
add independent satisfiable/contradictory/unknown controls; this branch does not block F05/H.

## Verification and handoff

**Proposed:** all changes and tests above. Future implementation uses the relevant
`just check-package <pkg>` and `just unit-package <pkg> <filter>` routes, with explicit
Arrow force-validation for actual consuming closures. Generation is part of its owning change.
F05 contributes a developer journey to [H's](58h-qualification-and-publication.md) scope-end integration/hygiene/manual
qualification after all functional scope; it does not start a campaign per package. H also
owns numerical/reference evidence and measurements. No build, test or dependency update ran.
Structural dependency removal is planned; time, memory and reliability gains remain unmeasured.
