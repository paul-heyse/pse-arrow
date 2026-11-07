---
id: ADR-0257
title: Rebuild canonical storage on SurrealDB v4
status: accepted
date: 2026-11-07
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, DP-09, DP-15, PS-09, PS-13]
blueprint: [§3.3, §20.6]
review: docs/design_review/reviews/design_review_surrealdb-v4-fresh-generation_2026-11-07.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: The exact v4 candidate cannot preserve a supported canonical write, receipt or protected-read contract.
verification: Run existing native write-conflict, Study extension, installed catalog, HTTP authority and protected scientific readback tests against the exact candidate. Qualify reconstruction, rollback and forward recovery in Plan68e.
standard: Core-3.4 / Efficient-Architecture-1.0 / Test-Assessment-1.2 / Process-Simulator-1.5
---

# ADR-0257 — Rebuild canonical storage on SurrealDB v4

## Context

ADR-0214 explains the persistent SurrealMX and bounded native transport design.
The deployed source build is SurrealDB 3.3.4. Plan67 has closed and its
regeneration and custody routes supply the basis for replacing the catalog.

## Scope

Amend blueprint §3.3 and §20.6 for the exact v4 supplier, generated catalog and
Study extension. Plan68 owns platform adoption and Plan66 owns scientific
interpretation and acceptance. The decision preserves the current canonical
write, permission, settlement and protected-read contracts.

## Drivers

Regenerate derived state through its existing Rust owners and preserve named
originals with actual readers. Keep one preparation, runtime and transport
generation so identity, failure recovery and release have a definite owner.

## Options

Staying on 3.3.4 preserves current qualification and carries its supplier patches.
The vendor's experimental in-place conversion retains existing files but makes
their format irreversible. A fresh v4 catalog gives generated schema and indexes
one owner and makes the old selection available before v4 writes are admitted.
Keeping two production routes would duplicate authority and lifecycle controls.
[Vendor migration guide](https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x)

## Outcome

Use source-built SurrealDB 4.0.0, persistent MX 1.1.0, the matching client
family and the rebuilt ABI 4 Study extension. Prepare the exact server and
extension builder together. Regenerate each default catalog in fresh owned
level-4 state. Adopt native v4 behavior where it preserves the consumed contract.

### Consequences

Reconstruction must receive named protected originals through existing portable
readers or a one-time owned conversion when required by a receiving route.
After v4 admits writes, recovery proceeds on v4.

### Compensating controls

Keep the frozen v3 generation through the pre-write rollback rehearsal. Keep
the isolated candidate's tool selection and state private through qualification.

### Confirmation

Use the exact candidate and current fixture owners. Existing tests satisfy the
behavior they distinguish. Add a focused case only for a changed, consumed
contract with an uncovered regression, including newly introduced receipt
fields. The managed native-service and recovery selections exercised current
initialization, ingress, actor grants, connection-bound sessions, original
checkpoint restart and exact activation receipt on the paired private release.
Plan68e owns authoritative selection. Plan66l owns the remaining aqueous
fixture teardown and scientific acceptance.

## Pros and cons

Fresh reconstruction gives a single schema and supplier path and limits
conversion to actual consumers. The new supplier and extension still need
build integration, applicable patch ports and native receiving tests.

## More information

[ADR-0214](0214-surrealmx-canonical-store.md) ·
[Plan68](../plans/68-surrealdb-v4-cutover.md) ·
[Functional review](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md)

## Status history

- 2026-11-07 — proposed for the maintainer-authorized private implementation.
- 2026-11-07 — accepted for fresh default selection and isolated historical custody.
