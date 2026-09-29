---
id: ADR-0116
title: Publish schemas and generated Python types for Rust-owned boundary documents
status: accepted
date: 2026-09-28
deciders: [paul-heyse]
level: decision
principles: [AP-02, DP-01, DP-02, DP-03, DP-04, DP-13, DP-14, DP-21, DP-24]
blueprint: [§4.2, §5.2, §16.5, §21.1, §21.5]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_typed-data-contracts_2026-09-28.md#decision
evidence: Proposed
supersedes: [ADR-0113]
superseded-by: null
revisit: Either of these fires. (a) A Python caller needs a backend setting that the typed projection cannot express, or one library upgrade forces more than one settings-document version change. (b) The generated Python document types cannot be produced byte-deterministically.
verification: Architecture scenarios S21 and S23, and the capability review's S04 and S05. Retained Plan 22 A4/A5 tests: test_solve_settings_backend_projection, test_route_and_eligibility_are_typed, initialization_admission_in_rust, identity_covers_every_settings_field. Plan 22 B5 tests: job_request_identity_independent_of_key_order, termination_detail_versioned_and_typed, invalid_tolerance_refused_at_decode, backend_settings_schema_generated, test_backend_settings_typed, authoring_schema_equivalent_under_schemars. Plan 22 B4: test_solve_settings_enum_types. Also test_postponed_annotations_are_resolved_before_the_lint, and just codegen-check and just python-stubs leaving no drift.
standard: core-3.1/process-simulator-1.1
scenarios: [docs/plans/22-solver-capabilities-architecture.md#s21, docs/plans/22-solver-capabilities-architecture.md#s23, docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s04, docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s05]
---

# ADR-0116: Publish schemas and generated Python types for Rust-owned boundary documents

## Context

ADR-0113 made every backend's pse-owned, serde-versioned settings type the Python projection,
exposed as frozen native classes. The
[typed data contracts review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_typed-data-contracts_2026-09-28.md)
found TD07: durable and boundary documents have no declared schema and no canonical identity.

Specifically:
- `BackendSettings` and the dynamics settings reach Python as `**fields: object`, invisible to type checkers.
- The job payload enters through `enqueue(…, payload: serde_json::Value, …)`. Its request identity hashes `payload.to_string()`, and serde_json's `preserve_order` is enabled by feature unification from DataFusion, so the identity depends on key order and on the build graph.
- The attempt's termination detail is built from `json!` literals with no version.
- The source manifest is unversioned.
- Nothing publishes a JSON Schema for any Rust-owned document.

The review also found TD11: the authoring JSON Schema has a bespoke 301-line emitter. And
ADR-0113 Outcome 2's registry enums were never implemented (TD06, decided in
[ADR-0115](0115-registry-typed-identities-and-vocabularies.md)).

The maintainer removed the ban on `from __future__ import annotations` on 2026-09-28: it had no
capability reason, because `pse.governance` already resolves annotations.

## Scope

**Supersedes ADR-0113.** It restates ADR-0113's outcomes that stand, and replaces the mechanism
by which settings reach Python. It is a change to the Python boundary contract (§21), within
ADR-0024 (Arrow transport unchanged) and the generated-contract rule of §21.5.

It binds:
- Rust-owned boundary documents and their schemas;
- the generated Python document types;
- validated scalar settings;
- canonical request identity for documents;
- the governance lint over the new types.

Registry-declared vocabularies and identities are ADR-0115's.

## Drivers

- **AP-02.** A library upgrade must not silently change a Python contract or request identity.
- **DP-01.** The Rust serde type owns each Rust-owned document, and every other representation (schema, Python type, documentation) is derived.
- **DP-02 and DP-03.** Invalid settings are refused where they are constructed or decoded, with a typed cause (S23).
- **DP-04.** A document's identity is defined over a canonical encoding.
- **DP-24.** Documents are versioned, their schemas are published, and an unknown version is refused (S21).
- **DP-13.** One JSON Schema mechanism, from a library.
- **Maintainer direction (2026-09-28).** Programmatic over static.

## Options

| Option | Assessment | Selection |
|---|---|---|
| ADR-0113 as built: native classes taking `**fields: object`, JSON round trip into serde | Complete in Rust, opaque to Python type checkers | Replaced |
| Hand-written Python mirrors of the settings | A second authority that drifts | Rejected |
| Registry `DocumentSpec` for settings | Settings are backend-adapter types owned in Rust, not relation-backed documents | Rejected |
| **serde types as the authority, schemars 1.x JSON Schema, generated Python msgspec types** | One authority; derived schema and types; msgspec is already the package's wire library | **Selected** |
| typify (JSON Schema → Rust) | The concept is not externally owned | Rejected; revisit when an external standard owns a schema |
| garde (struct validation) | Cross-field rules are admission rules with typed reasons | Rejected |
| **nutype validated scalars**, or hand-written `serde(try_from)` newtypes | "Cannot construct invalid" for single-value domains | Selected: nutype if its schemars-1 path works, otherwise the newtypes |

## Outcome

### Retained from ADR-0113, unchanged

1. **Settings types.**
   - Every `BackendSettings` variant has a Python projection: HiGHS, Clarabel, POUNCE (including `L1ExactPenalty`), POUNCE-convex, KINSOL, Ipopt, SCIP, and the Diffsol and IDAS profile methods.
   - Each projects the pse-owned, serde-versioned settings type of its backend adapter.
   - Library types never cross the boundary or enter request identity.
2. **Names.**
   - Every enumeration that crosses the boundary is a registry enum; Python receives its registry `as_str` name. This covers backend, route, intent, method, linear solver, ordering, eligibility reason, termination, assurance and candidate use.
   - There are no hand-written string-to-enum tables and no `Debug` output in any contract (ADR-0115 Outcome 3 carries this out).
3. **Eligibility** is reported as typed rows: backend, a registry reason code, affected identities and typed detail values. `capabilities` reports linked libraries only.
4. **Admission in Rust.** Initialization admission runs in Rust `validate_profile`. Python performs no admission and no numerical validation beyond the typed decoding of item 8.
5. **Settings identity** is derived from the serde encoding of the pse-owned type and covers every field.

### Replaced and added

6. **Rust-owned documents are typed and versioned.**
   - This covers the backend settings document, dynamics settings, the durable job payload (`ModelingJob`, `JobProfile`), the attempt's termination detail (a typed `TerminationDetail` per termination class) and the source manifest.
   - Each is a serde type in its owning crate, behind an envelope with an explicit version.
   - An unknown version is refused, never reinterpreted.
   - Untyped JSON values do not appear inside them: a field whose interior drives a decision is a typed field.
7. **Schemas and Python types are generated.**
   - schemars 1.x derives a JSON Schema (draft 2020-12) for every document in item 6. The generator writes the schemas to `docs/generated/schema/`.
   - It also writes generated msgspec `Struct` types into `python/pse/contracts/`: by datamodel-code-generator (pinned in `[dependency-groups]`, run with `--disable-timestamp`), or by the `pse-codegen` Python target, whichever gives byte-deterministic output.
   - Python callers construct those types. The native entry points accept the encoded, typed document, so `**fields: object` signatures disappear.
   - A schema difference between two versions of a document is its compatibility record.
8. **Validated scalars.**
   - Single-value setting domains are types that cannot hold an invalid value: tolerance (finite, > 0), fraction, positive count and finite bound.
   - They are nutype 0.8 types if `derive_unchecked(schemars::JsonSchema)` works with schemars 1.x, and hand-written `serde(try_from)` newtypes otherwise. Their schema states the constraint.
   - Decoding refuses a violation with a typed cause.
   - `admit_settings` keeps the cross-field and environment rules, with typed reasons (DP-21).
9. **Canonical request identity.**
   - A request identity is framed from the typed document (§5.3 keyed derivation over its canonical serde encoding), never from the text of an order-preserving JSON value.
   - `enqueue` takes the typed payload.
10. **One JSON Schema mechanism.** The authoring JSON Schema is derived with schemars from the generated authoring document structs, with registry facets emitted as `#[schemars(...)]` attributes, provided the output is at least as precise as the bespoke emitter's. Otherwise the emitter stays, and the reason is recorded in the generator.
11. **Governance.**
    - `pse.governance` checks the generated msgspec types for `Any`, bare `dict` and bare `list`, alongside the attrs contract classes.
    - Postponed annotations (`from __future__ import annotations`) are allowed anywhere in `python/pse`, because the checks resolve annotations before inspecting them (`attrs.resolve_types`, cattrs, and msgspec's own resolution).

### Consequences

- **Python API.** Python settings become generated classes rather than native keyword classes. Each document change means `just codegen` and `just python-stubs`.
- **Settings identity unchanged.** A document's serde encoding is already its identity (item 5); the typed enqueue changes only how a caller builds the payload, not the encoding.
- **Tooling.** The generator gains a schema path, and the Python tool pins gain datamodel-code-generator if it is chosen.

### Compensating controls

- The regeneration check over the schemas and Python types.
- The identity test independent of key order.
- The typed decode test.
- The `Any` lint, including its postponed-annotation test.
- Strict msgspec and serde structuring that refuses unknown fields and versions.

### Confirmation

**Architectural reasoning.** The review's slots 3, 4, 7 and 8 (TD07, TD11).

**Measured.** `cargo tree -e features -i serde_json` shows `preserve_order` unified in.

**Tested.** `test_postponed_annotations_are_resolved_before_the_lint`, run with `just py-unit-native python/pse/tests/test_any_lint.py` (Python 3.14.7, 2026-09-28): 12 passed, 0 failed.

**Interface-checked.**
- datamodel-code-generator's msgspec output and `--disable-future-imports`.
- schemars 1.2.2.
- nutype 0.8.0.

The remaining tests in `verification:` settle Plan 22 B5.

## Pros and cons

Generated Python types make the boundary visible to type checkers and make invalid settings
unconstructible. The cost is a second generated Python class system (msgspec beside attrs),
kept honest by the same governance lint.

## More information

- [Typed data contracts review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_typed-data-contracts_2026-09-28.md): TD06, TD07, TD11.
- Architecture companion [§12.5](../plans/22-solver-capabilities-architecture.md#12-typed-data-contracts).
- Capability review F09, F26, F30, L-C1, S04, S05 (through ADR-0113).
- Related: ADR-0024, ADR-0051, ADR-0106, ADR-0108, ADR-0115.
- Plan 22 packets A4 and A5 (retained evidence), B4 and B5.

## Status history

- 2026-09-28 — proposed and accepted under the maintainer's approval of the typed data contracts review (Accept-scoped, author review, Proposed evidence level). Supersedes ADR-0113, restating its Outcomes 1–4 and 6 (as Outcomes 1–5 here) and replacing its projection mechanism and its evolution clause.
