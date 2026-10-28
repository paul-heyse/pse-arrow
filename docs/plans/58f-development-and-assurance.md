---
title: "Plan58f: Development and assurance"
status: draft
date: 2026-10-28
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, environment, validation]
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md]
---

# Plan58f: Development and assurance

This companion develops O02/O11 and developer O03/O09. The [coordinator](58-rust-owned-system.md)
owns outcomes, rules and qualification; this document owns package state. Plan57 remains superseded historical evidence.

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
policy, preparation/build coordination, fixture ownership, generation, docs/governance and
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

Development evidence keeps original authority/codec/identity, selected population and coverage,
explicit unknowns and protected raw detail. Failed, skipped, unreported, reused, transferred
and command-failure outcomes stay distinct. Existing optional tooling indexes are disposable
projections, not a new canonical source or host-liveness store. Offline original-owner/detail
inspection must work without database, mathematical Runtime or compiler availability.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [F01](#f01) | planned | 58f | Direct development/configuration operations |
| [F02](#f02) | planned | 58f | Rust preparation/compiler/test composition |
| [F03](#f03) | planned | 58f | Rust generators/docs/governance consumers |
| [F04](#f04) | planned | 58f | Evidence and behavioral assurance migrated |
| [F05](#f05) | planned | 58f | Retired bridges and native developer acceptance |
| [F06](#f06) | deferred | 58f | Discrete explanation inquiry only when an actual consumer justifies it |

<a id="f01"></a>
### F01 — Make retained development operations directly callable

Use [T02's retained-outcome contracts](58-rust-owned-system.md#t02) and 58b B01's root. Group
operations by their real owner and dependency need; move configuration/skill synchronization,
tool installation and preparation explanation from `agent-config.py`, `library_skills.py`,
bootstrap/setup helpers and retained capability operations into appropriate Rust owners.
Targeted tests cover selected preparation, missing tools, ambiguous aliases, preserved configuration and
offline help with Python/MCP unavailable. Migrate recipe and agent consumers with each owner.

<a id="f02"></a>
### F02 — Move compiler, validation and fixture orchestration together

Consume 58b's environment/admission contract; low-effect argument/graph policy can develop
before the final host cutover. Move `arrow_validation`, compiler-cache/build observation,
producer-Cargo selection, validation/test-run composition and retained resource fixtures into
Rust. Preserve Cargo-owned optional/target/dev selection and actual target/host Arrow-leaf
`force_validate` coverage; unknown closure refuses, pure scope adds no unrelated target.
Keep pinned toolchain, unwinding and selected host parallelism. Resolve shared artifact-writer
lock conflicts at invocation ownership rather than impose CPU caps or blanket serialization.
Targeted tests challenge omitted validation leaves, host/build contexts, changed compiler
inputs, logging modes, selected-child failure, canceled fixture settlement and original exit.
Delete corresponding Python orchestration when every retained caller has moved.

<a id="f03"></a>
### F03 — Keep declarations and generation authoritative in Rust

F01 supplies direct development entrypoints; use existing xtask/registry owners where suitable.
Move retained first-party docs/lifecycle/ADR/governance and generator orchestration from Python
to Rust, preserving native metadata/status owners and sparse scope relationships. Consolidate
checks under their real declarations rather than preserve every Python helper boundary.
Change generators, regenerate through `just codegen`, and migrate publication/consumer routes
together; never edit generated output directly. Remove Python-only projections only after
the optional-boundary owner confirms no retained consumer. Targeted tests cover malformed or
duplicate metadata, stale output detection without publication, exact generation determinism,
and read-only comparison leaving files unchanged. Delete replaced scripts and obsolete fixtures.

<a id="f04"></a>
### F04 — Migrate valuable evidence and scientific acceptance purposes

F02 supplies original invocation/fixture terminals; product/control/results packages supply
the native interfaces being assessed. Move retained evidence readers, assessment/projection,
diagnostic selection and portable handoff into Rust, reusing existing native evidence/query
owners rather than creating another store. Move valuable Python product scenarios to native
owner/public executable tests as each replaced interface becomes available. Retain isolated
IDAES/Pyomo, SciPy and teqp oracles where they supply independent evidence. Targeted controls
cover mixed pass/fail/unreported populations, stale/missing supplied sources, raw-detail custody,
unavailable optional index, cancellation and one deliberately wrong scientific result. Fixture
or conversion failure must never be relabeled scientific success; preserve original failures.

<a id="f05"></a>
### F05 — Complete the independent developer journey and retire bridges

Compose F01–F04 with 58b through Rust APIs/executables: explain and prepare selected capabilities,
execute a targeted check, inspect the result, diagnose failure and settle resources without
first-party Python or MCP. Include fixture refusal/interruption. Remove replaced scripts, imports,
recipe routes, generated glue, wrapper-specific tests and dependencies in their owning slices;
check retained reference-only and optional-adapter dependencies separately before removal.
Documentation examples and agent instructions move with their actual callable owners. Report
any deliberately lost presentation feature through the coordinator's selected-outcome account.

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
