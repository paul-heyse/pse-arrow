---
id: ADR-0024
title: Cross the Python boundary with pyo3-arrow and the PyCapsule stream protocol
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-37, DM-41, DM-42, DM-43]
blueprint: [§3.1, §21.1]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: `arrow-pyarrow` drops its build-time interpreter requirement, or `pyo3-arrow` stops tracking the pinned arrow family
verification: `python / test` `test_native_stub_surface` and `test_extension_round_trip`; `rust / family-check` (pyo3 family)

---

# ADR-0024: Cross the Python boundary with pyo3-arrow and the PyCapsule stream protocol

## Context

Blueprint §3.1 retains `pyo3-arrow` 0.19.0 over `arrow-pyarrow` because the platform needs the PyCapsule stream protocol and the numpy bridge, and because `arrow-pyarrow` requires a Python interpreter at build time, which breaks the offline build. The second review's R2-4 sharpened the measured Rust/Python asymmetry at that boundary.

## Scope

Binds the Rust-side Arrow-to-Python mechanism and the contract that crosses it. The Python-side contract classes and their strictness are ADR-0015's subject.

## Drivers

The capsule protocol, not `pyarrow`, is the contract: any Arrow implementation can consume it, and none of the typed contract classes names a `pyarrow` type; an interpreter at build time would make the Rust build depend on the Python environment.

## Options

`arrow-pyarrow` — rejected for the build-time interpreter and the narrower protocol. Hand-rolled FFI over the C data interface — rejected: `pyo3-arrow` already carries the capsule and numpy bridges.

## Outcome

`pyo3` 0.29 with `pyo3-arrow` 0.19.0; every table crosses as `__arrow_c_stream__`, drained as a `RecordBatchReader`, never as row objects and never materialized on both sides at once.

### Consequences

`pyo3-arrow` depends on `thiserror` 1.x beside the platform's 2.x, which is an accepted `deny.toml` skip with a reason: no type crosses that boundary. Wheels are `abi3-py311`.

### Compensating controls

The bundle's loss profile declares, per table and field, whether a `pse.*` extension type is carried and whether the consumer registered it — the enforcement is asymmetric and the declaration says which side enforces what.

### Confirmation

`test_native_stub_surface` diffs the hand-written `_native.pyi` against `dir(_native)`; the extension round-trip test runs on both interpreters.

## Pros and cons

Accepting a duplicate `thiserror` generation is the price of the capsule protocol and an interpreter-free build; the alternative is a build that cannot run offline.

## More information

Blueprint §3.1 (pyo3 row), §21.1 (extension module); review finding R2-4; ADR-0018 for the family rule and the skip.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
