---
id: ADR-0268
title: Separate semantic library dependencies from product runtime authority
status: proposed
date: 2026-11-10
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, DP-09]
blueprint: [§3.1, §3.2]
review: docs/design_review/reviews/design_review_semantic_boundaries_2026-11-10.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A semantic owner reaches product execution or a fixture authority enters a product workflow.
verification: family-check excludes product runtime owners; quantity receipt replay controls preserve membership/refusal; unsafe_allowlist governs the narrow test mint and scientific-replay Miri includes it.

---

# ADR-0268: Separate semantic library dependencies from product runtime authority

## Context

The maintainer clarified during Plan72b/c execution that using Tokio does not mean
depending on product runtime code. Scope-end checks still enforce the older package
name prohibition. They also found an existing isolated quantity receipt test calling
the production qualification constructor outside the governed assertion allowlist.

## Scope

Amend blueprint §3.1/§3.2 to express the actual semantic responsibility boundary and
govern the existing test-only assertion. Production scientific admission remains sealed.

## Drivers

Enforce dependency responsibility rather than incidental third-party execution needs.
Exercise authentic and deliberately corrupt physical receipt populations without
misrepresenting an isolated fixture as a canonical compiler-qualified product.

## Options

Retaining the Tokio prohibition rejects supported native library composition without
establishing isolation from product infrastructure. Remove that prohibition and explicitly
exclude product engine/runtime/operations/backend owners. Existing columnar exclusions remain.
For receipt tests, use the existing nondefault isolated-fixture constructor rather than
invent a safe public authority factory, delete useful corruption controls or invoke the
production constructor on locally manufactured hashes.

## Outcome

Semantic closures may use concurrency libraries. They cannot reach the named product
runtime owners. Quantity's allowlist entry covers only its cfg(test) receipt helper;
that helper uses `from_admitted_record_fixture` through a dev-dependency feature.

### Consequences

The manifest, architecture and dependency checker agree. The existing helper retains its
reasoned unsafe assertion and owns the complete private fixture population. It grants no
production qualification or publication authority. No new production API is introduced.

### Compensating controls

Retain resolved normal-closure checks, the unsafe allowlist and reason checks, explicit
fixture-source qualification and the existing scientific-replay Miri route extended to
the quantity replay tests. Fixture authority never makes arbitrary product data admitted.

### Confirmation

The [Plan72 scope-end correction](../plans/72-surrealdb-functional-improvements.md#scope-end-corrections)
owns review and executed evidence. This record remains proposed; implementation and
formal decision acceptance remain separate.

The [independent focused assessment](../design_review/reviews/design_review_semantic_boundaries_2026-11-10.md)
supports the target at Interface-checked strength and identified one stale layering sentence;
the architecture owner is reconciled. The review does not claim executed qualification.

## Pros and cons

The rule permits native library composition and preserves the product boundary. The extra
test-only allowlist entry requires explicit review whenever its assertion changes.

## More information

[Scientific authority](0164-unify-simulation-substrate.md) and
[Plan72](../plans/72-surrealdb-functional-improvements.md).

## Status history

- 2026-11-10 — proposed.
