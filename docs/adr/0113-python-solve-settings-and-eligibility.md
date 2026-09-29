---
id: ADR-0113
title: Project typed backend settings, registry names and typed eligibility across the Python boundary
status: superseded
date: 2026-09-27
deciders: [paul-heyse]
level: decision
principles: [AP-02, DP-01, DP-02, DP-14, DP-24]
blueprint: [§21.1, §21.5]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision
evidence: Proposed
supersedes: []
superseded-by: ADR-0116
revisit: A Python caller needs a backend setting the typed projection cannot express, or one library upgrade forces more than one Python settings-contract version change.
verification: Review scenarios S04 and S05 and architecture scenario S18, settled by the Plan 22 A4/A5 tests test_solve_settings_backend_projection, test_route_and_eligibility_are_typed, initialization_admission_in_rust and identity_covers_every_settings_field, with just python-stubs and just codegen leaving no drift.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s04, docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s05, docs/plans/22-solver-capabilities-architecture.md#s18]
---

# ADR-0113: Project typed backend settings, registry names and typed eligibility across the Python boundary

## Context

Python always sends `BackendSettings::Default` (`pse-py` `settings.rs`), so Python callers
cannot select a HiGHS method, a Clarabel mode or a POUNCE linear solver, and Python conic reuse
always rebuilds ([L-C1](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#l-c1)).
Clarabel's serde encoding is the Python conic wire format and an input to request identity;
Diffsol options sit in `dynamics::Profile` with a hand-written identity
([F09](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f09)). The
boundary exposes Rust `Debug` output, prose eligibility and hand-written string-to-enum tables
([F30](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f30)), and
initialization admission exists only in the Python adapter
([F26](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f26)). Plan 22
adds more settings (Ipopt linear solvers, SCIP, POUNCE methods, dynamics methods).

## Scope

A change to the Python boundary contract (§21), within ADR-0024 (Arrow transport unchanged) and
the generated-contract rule of §21.5. Binds the settings types, the names that cross the
boundary, eligibility reporting, admission placement and contract evolution.

## Drivers

- **AP-02.** A library upgrade must not silently change a Python contract or request identity.
- **DP-02 and DP-01.** Names and reasons are typed and owned once, by the registry.
- **DP-14.** Adapters translate; admission rules do not live in the Python adapter.
- **DP-24.** Durable settings contracts are versioned; an unknown version is refused.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Keep `Default` only in Python | Python cannot reach most backend capability (L-C1) | Rejected |
| Library serde types as the wire format (Clarabel, Diffsol) | Library upgrades change the Python contract and identity (F09) | Rejected |
| Hand-written Python mirrors of the Rust settings | A second authority that drifts (DP-01) | Rejected |
| pse-owned Rust settings types exposed as native classes, registry enums for every name, typed eligibility rows | One authority per fact; generated stubs and contracts | **Selected** |

## Outcome

1. **Settings.** Every `BackendSettings` variant has a Python projection: HiGHS (method,
   diagnostics, sparse start), Clarabel (mode and pse-owned settings), POUNCE (method including
   `L1ExactPenalty`, linear solver), POUNCE-convex, KINSOL, Ipopt (linear solver, ordering and
   the admitted controls of ADR-0108), SCIP (limits, exact mode, concurrency) and the Diffsol and
   IDAS profile methods. They are the pse-owned, serde-versioned `Settings` types of each
   backend adapter (Plan 22 A2), exposed as frozen native classes whose stubs are generated from
   the extension (`just python-stubs`). Library types never cross the boundary or enter request
   identity.
2. **Names.** Every enumeration that crosses the boundary — backend, route, intent, method,
   linear solver, ordering, eligibility reason, termination, assurance, candidate use — is a
   registry enum, and Python receives its registry `as_str` name. There are no hand-written
   string-to-enum tables and no `Debug` output in any contract.
3. **Eligibility.** Reported as typed rows: backend, a registry reason code, affected
   identities and typed detail values. `capabilities` keeps reporting linked libraries only.
4. **Admission in Rust.** Initialization admission moves into Rust `validate_profile`, with
   route-typed linear settings; Python performs no admission and no numerical validation.
5. **Evolution.** Settings envelopes are versioned msgspec structs that forbid unknown fields;
   an unknown version is refused. Registry contracts are regenerated with `just codegen`.
6. **Identity.** A settings identity is derived from the serde encoding of the pse-owned type
   and covers every field (Plan 22 A4).

### Consequences

`pse-py` loses its hand-written tables; Python users gain the full backend surface; each backend
adapter owns a settings type that must stay stable across library upgrades.

### Compensating controls

Stub and codegen regeneration with drift checks; the identity test over every field; strict
structuring that refuses unknown fields and versions.

### Confirmation

The tests named in `verification:` (Plan 22 A4, A5).

## Pros and cons

Typed projection makes the boundary complete and upgrade-safe. The cost is one pse-owned
settings type per backend, which the backend-execution adapter needs anyway.

## More information

- Architecture companion [§4](../plans/22-solver-capabilities-architecture.md#4-backend-execution-adapter-and-shared-runners).
- Capability review F09, F26, F30, L-C1, S04, S05.
- Related: ADR-0024, ADR-0106, ADR-0108. Plan 22 packets A4, A5.

## Status history

- 2026-09-27 — proposed (Plan 22 D0).
- 2026-09-27 — accepted under the maintainer's authorization of the full Plan 22 scope (2026-09-27), after the [Plan 22 target review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision) returned Accept (author review, Proposed evidence level).
- 2026-09-28 — superseded by ADR-0116.
